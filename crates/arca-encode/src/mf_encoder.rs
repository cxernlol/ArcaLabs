use crate::EncodeError;
use std::{mem::ManuallyDrop, ptr};
use windows::{
    Win32::{
        Graphics::Direct3D11::{ID3D11Device, ID3D11Texture2D},
        Media::MediaFoundation::*,
        System::Com::{COINIT_MULTITHREADED, CoInitializeEx, CoUninitialize},
    },
    core::Interface,
};

#[derive(Debug, Clone, Copy)]
pub struct H264Settings {
    pub width: u32,
    pub height: u32,
    pub frame_rate: u32,
    pub bitrate: u32,
}

impl H264Settings {
    fn validate(self) -> Result<Self, EncodeError> {
        if self.width == 0
            || self.height == 0
            || self.width > 1920
            || self.height > 1080
            || self.width % 2 != 0
            || self.height % 2 != 0
            || !(1..=60).contains(&self.frame_rate)
            || self.bitrate == 0
        {
            return Err(EncodeError::Initialization(
                "H.264 output requires even dimensions up to 1920×1080, 1–60 fps, and a non-zero bitrate".into(),
            ));
        }
        Ok(self)
    }
}

#[derive(Debug)]
pub struct EncodedSample {
    pub bytes: Vec<u8>,
    pub timestamp_100ns: i64,
    pub duration_100ns: i64,
    pub keyframe: bool,
}

struct ComApartment;

impl ComApartment {
    fn initialize() -> Result<Self, EncodeError> {
        unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) }
            .ok()
            .map_err(|error| EncodeError::Initialization(error.to_string()))?;
        Ok(Self)
    }
}

impl Drop for ComApartment {
    fn drop(&mut self) {
        unsafe { CoUninitialize() };
    }
}

struct MediaFoundation;

impl MediaFoundation {
    fn start() -> Result<Self, EncodeError> {
        unsafe { MFStartup(MF_VERSION, MFSTARTUP_FULL) }
            .map_err(|error| EncodeError::Initialization(error.to_string()))?;
        Ok(Self)
    }
}

impl Drop for MediaFoundation {
    fn drop(&mut self) {
        unsafe {
            let _ = MFShutdown();
        }
    }
}

struct Activations(*mut Option<IMFActivate>, u32);

impl Drop for Activations {
    fn drop(&mut self) {
        if self.0.is_null() {
            return;
        }
        for activation in unsafe { std::slice::from_raw_parts(self.0, self.1 as usize) }
            .iter()
            .flatten()
        {
            let _ = unsafe { activation.ShutdownObject() };
        }
        unsafe { windows::Win32::System::Com::CoTaskMemFree(Some(self.0.cast())) };
    }
}

/// Hardware-only Media Foundation H.264 encoder. Construct and use on one dedicated MTA worker.
pub struct HardwareH264Encoder {
    transform: IMFTransform,
    _device_manager: IMFDXGIDeviceManager,
    _mf: MediaFoundation,
    _com: ComApartment,
    settings: H264Settings,
    input_stream: u32,
    output_stream: u32,
    output_buffer_size: u32,
    sequence_header: Vec<u8>,
}

impl HardwareH264Encoder {
    pub fn new(device: &ID3D11Device, settings: H264Settings) -> Result<Self, EncodeError> {
        let settings = settings.validate()?;
        let com = ComApartment::initialize()?;
        let mf = MediaFoundation::start()?;
        let (manager, reset_token) = create_device_manager(device)?;
        let transform = activate_hardware_encoder(&manager, reset_token, settings)?;

        let mut input_count = 0;
        let mut output_count = 0;
        unsafe {
            transform
                .GetStreamCount(&mut input_count, &mut output_count)
                .map_err(init_error)?;
        }
        if input_count == 0 || output_count == 0 {
            return Err(EncodeError::Initialization(
                "Hardware encoder exposes no input or output streams".into(),
            ));
        }
        let mut input_ids = vec![0; input_count as usize];
        let mut output_ids = vec![0; output_count as usize];
        let ids = unsafe { transform.GetStreamIDs(&mut input_ids, &mut output_ids) };
        if ids.is_err() {
            input_ids[0] = 0;
            output_ids[0] = 0;
        }
        let input_stream = input_ids[0];
        let output_stream = output_ids[0];
        let current_type =
            unsafe { transform.GetOutputCurrentType(output_stream) }.map_err(init_error)?;
        let header_size =
            unsafe { current_type.GetBlobSize(&MF_MT_MPEG_SEQUENCE_HEADER) }.map_err(init_error)?;
        if header_size == 0 {
            return Err(EncodeError::Initialization(
                "Hardware encoder returned no H.264 sequence header".into(),
            ));
        }
        let mut sequence_header = vec![0; header_size as usize];
        unsafe {
            current_type
                .GetBlob(&MF_MT_MPEG_SEQUENCE_HEADER, &mut sequence_header, None)
                .map_err(init_error)?;
        }
        let output_info =
            unsafe { transform.GetOutputStreamInfo(output_stream) }.map_err(init_error)?;

        unsafe {
            transform
                .ProcessMessage(MFT_MESSAGE_NOTIFY_BEGIN_STREAMING, 0)
                .map_err(init_error)?;
            transform
                .ProcessMessage(MFT_MESSAGE_NOTIFY_START_OF_STREAM, 0)
                .map_err(init_error)?;
        }

        Ok(Self {
            transform,
            _device_manager: manager,
            _mf: mf,
            _com: com,
            settings,
            input_stream,
            output_stream,
            output_buffer_size: output_info.cbSize.max(1024 * 1024),
            sequence_header,
        })
    }

    /// Media Foundation's Annex B SPS/PPS header for the configured output type.
    pub fn sequence_header(&self) -> &[u8] {
        &self.sequence_header
    }

    /// Encodes an NV12 texture on the D3D11 device passed to `new`; no CPU frame readback is used.
    pub fn encode(
        &mut self,
        texture: &ID3D11Texture2D,
        timestamp_100ns: i64,
    ) -> Result<Vec<EncodedSample>, EncodeError> {
        let mut desc = Default::default();
        unsafe { texture.GetDesc(&mut desc) };
        if desc.Width != self.settings.width
            || desc.Height != self.settings.height
            || desc.Format.0 != 103
        {
            return Err(EncodeError::Initialization(
                "The encoder accepts only a matching-size NV12 D3D11 texture".into(),
            ));
        }

        let buffer = unsafe {
            MFCreateDXGISurfaceBuffer(&ID3D11Texture2D::IID, texture, 0, false)
                .map_err(init_error)?
        };
        let input = unsafe { MFCreateSample() }.map_err(init_error)?;
        unsafe {
            input.AddBuffer(&buffer).map_err(init_error)?;
            input.SetSampleTime(timestamp_100ns).map_err(init_error)?;
            input
                .SetSampleDuration(10_000_000 / i64::from(self.settings.frame_rate))
                .map_err(init_error)?;
            self.transform
                .ProcessInput(self.input_stream, &input, 0)
                .map_err(init_error)?;
        }

        let output_info = unsafe { self.transform.GetOutputStreamInfo(self.output_stream) }
            .map_err(init_error)?;
        let caller_sample =
            if output_info.dwFlags & MFT_OUTPUT_STREAM_PROVIDES_SAMPLES.0 as u32 != 0 {
                None
            } else {
                let output_buffer =
                    unsafe { MFCreateMemoryBuffer(self.output_buffer_size) }.map_err(init_error)?;
                let sample = unsafe { MFCreateSample() }.map_err(init_error)?;
                unsafe {
                    sample.AddBuffer(&output_buffer).map_err(init_error)?;
                }
                Some(sample)
            };
        let mut output = [MFT_OUTPUT_DATA_BUFFER {
            dwStreamID: self.output_stream,
            pSample: ManuallyDrop::new(caller_sample),
            dwStatus: 0,
            pEvents: ManuallyDrop::new(None),
        }];
        let mut status = 0;
        let result = unsafe { self.transform.ProcessOutput(0, &mut output, &mut status) };
        let sample = unsafe { ManuallyDrop::take(&mut output[0].pSample) };
        let events = unsafe { ManuallyDrop::take(&mut output[0].pEvents) };
        drop(events);
        if let Err(error) = result {
            if error.code() == MF_E_TRANSFORM_NEED_MORE_INPUT {
                return Ok(Vec::new());
            }
            return Err(init_error(error));
        }
        let Some(sample) = sample else {
            return Ok(Vec::new());
        };
        let buffer = unsafe { sample.ConvertToContiguousBuffer() }.map_err(init_error)?;
        let mut data = ptr::null_mut();
        let mut length = 0;
        unsafe { buffer.Lock(&mut data, None, Some(&mut length)) }.map_err(init_error)?;
        let bytes = if data.is_null() || length == 0 {
            Vec::new()
        } else {
            unsafe { std::slice::from_raw_parts(data, length as usize) }.to_vec()
        };
        unsafe { buffer.Unlock() }.map_err(init_error)?;
        let timestamp = unsafe { sample.GetSampleTime() }.unwrap_or(timestamp_100ns);
        let duration = unsafe { sample.GetSampleDuration() }
            .unwrap_or(10_000_000 / i64::from(self.settings.frame_rate));
        let keyframe = unsafe { sample.GetUINT32(&MFSampleExtension_CleanPoint) }
            .is_ok_and(|clean| clean != 0);
        Ok((!bytes.is_empty())
            .then_some(EncodedSample {
                bytes,
                timestamp_100ns: timestamp,
                duration_100ns: duration,
                keyframe,
            })
            .into_iter()
            .collect())
    }
}

impl Drop for HardwareH264Encoder {
    fn drop(&mut self) {
        unsafe {
            let _ = self
                .transform
                .ProcessMessage(MFT_MESSAGE_NOTIFY_END_OF_STREAM, 0);
            let _ = self
                .transform
                .ProcessMessage(MFT_MESSAGE_NOTIFY_END_STREAMING, 0);
        }
    }
}

fn create_device_manager(
    device: &ID3D11Device,
) -> Result<(IMFDXGIDeviceManager, u32), EncodeError> {
    let mut reset_token = 0;
    let mut manager = None;
    unsafe { MFCreateDXGIDeviceManager(&mut reset_token, &mut manager) }.map_err(init_error)?;
    let manager = manager.ok_or_else(|| {
        EncodeError::Initialization("Media Foundation returned no D3D11 device manager".into())
    })?;
    unsafe { manager.ResetDevice(device, reset_token) }.map_err(init_error)?;
    Ok((manager, reset_token))
}

fn activate_hardware_encoder(
    manager: &IMFDXGIDeviceManager,
    _reset_token: u32,
    settings: H264Settings,
) -> Result<IMFTransform, EncodeError> {
    let output_type = MFT_REGISTER_TYPE_INFO {
        guidMajorType: MFMediaType_Video,
        guidSubtype: MFVideoFormat_H264,
    };
    let mut count = 0;
    let mut raw = ptr::null_mut();
    unsafe {
        MFTEnumEx(
            MFT_CATEGORY_VIDEO_ENCODER,
            MFT_ENUM_FLAG_HARDWARE | MFT_ENUM_FLAG_SORTANDFILTER,
            None,
            Some(&output_type),
            &mut raw,
            &mut count,
        )
    }
    .map_err(init_error)?;
    let _activations = Activations(raw, count);
    if raw.is_null() || count == 0 {
        return Err(crate::EncodeError::HardwareEncoderUnavailable);
    }
    let transforms = unsafe { std::slice::from_raw_parts_mut(raw, count as usize) };
    let mut last_error = None;
    for index in 0..transforms.len() {
        let Some(activation) = transforms[index].as_ref() else {
            continue;
        };
        let transform = match unsafe { activation.ActivateObject::<IMFTransform>() } {
            Ok(transform) => transform,
            Err(error) => {
                last_error = Some(error);
                continue;
            }
        };
        let attributes = match unsafe { transform.GetAttributes() } {
            Ok(attributes) => attributes,
            Err(error) => {
                last_error = Some(error);
                continue;
            }
        };
        if unsafe { attributes.GetUINT32(&MF_SA_D3D11_AWARE) }.unwrap_or(0) == 0 {
            continue;
        }
        let mut input_count = 0;
        let mut output_count = 0;
        if let Err(error) = unsafe { transform.GetStreamCount(&mut input_count, &mut output_count) }
        {
            last_error = Some(error);
            continue;
        }
        if input_count == 0 || output_count == 0 {
            continue;
        }
        let mut input_ids = vec![0; input_count as usize];
        let mut output_ids = vec![0; output_count as usize];
        if unsafe { transform.GetStreamIDs(&mut input_ids, &mut output_ids) }.is_err() {
            input_ids[0] = 0;
            output_ids[0] = 0;
        }
        let input_id = input_ids[0];
        let output_id = output_ids[0];
        let mut manager_ptr = ptr::null_mut();
        unsafe {
            manager
                .cast::<windows::core::IUnknown>()
                .map(|unknown| manager_ptr = Interface::as_raw(&unknown) as *mut _)
                .map_err(init_error)?;
            transform
                .ProcessMessage(MFT_MESSAGE_SET_D3D_MANAGER, manager_ptr as usize)
                .map_err(init_error)?;
        }
        let input_type = video_type(settings, MFVideoFormat_NV12)?;
        let output_type = video_type(settings, MFVideoFormat_H264)?;
        let configured = unsafe {
            transform
                .SetInputType(input_id, &input_type, 0)
                .and_then(|_| transform.SetOutputType(output_id, &output_type, 0))
        };
        if let Err(error) = configured {
            last_error = Some(error);
            continue;
        }
        transforms[index] = None;
        return Ok(transform);
    }
    if let Some(error) = last_error {
        Err(init_error(error))
    } else {
        Err(EncodeError::Initialization(
            "No hardware H.264 transform supports D3D11 NV12 input at this resolution".into(),
        ))
    }
}

fn video_type(
    settings: H264Settings,
    subtype: windows::core::GUID,
) -> Result<IMFMediaType, EncodeError> {
    let media_type = unsafe { MFCreateMediaType() }.map_err(init_error)?;
    unsafe {
        media_type
            .SetGUID(&MF_MT_MAJOR_TYPE, &MFMediaType_Video)
            .map_err(init_error)?;
        media_type
            .SetGUID(&MF_MT_SUBTYPE, &subtype)
            .map_err(init_error)?;
        media_type
            .SetUINT64(
                &MF_MT_FRAME_SIZE,
                (u64::from(settings.width) << 32) | u64::from(settings.height),
            )
            .map_err(init_error)?;
        media_type
            .SetUINT64(&MF_MT_FRAME_RATE, u64::from(settings.frame_rate) << 32 | 1)
            .map_err(init_error)?;
        media_type
            .SetUINT64(&MF_MT_PIXEL_ASPECT_RATIO, 1u64 << 32 | 1)
            .map_err(init_error)?;
        media_type
            .SetUINT32(&MF_MT_INTERLACE_MODE, MFVideoInterlace_Progressive.0 as u32)
            .map_err(init_error)?;
        if subtype == MFVideoFormat_H264 {
            media_type
                .SetUINT32(&MF_MT_AVG_BITRATE, settings.bitrate)
                .map_err(init_error)?;
        }
    }
    Ok(media_type)
}

fn init_error(error: windows::core::Error) -> EncodeError {
    EncodeError::Initialization(error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_output_dimensions_and_rates_outside_the_pipeline_limits() {
        let valid = H264Settings {
            width: 1920,
            height: 1080,
            frame_rate: 60,
            bitrate: 6_000_000,
        };
        assert!(valid.validate().is_ok());
        assert!(
            H264Settings {
                width: 1921,
                ..valid
            }
            .validate()
            .is_err()
        );
        assert!(
            H264Settings {
                width: 1919,
                ..valid
            }
            .validate()
            .is_err()
        );
        assert!(
            H264Settings {
                frame_rate: 0,
                ..valid
            }
            .validate()
            .is_err()
        );
        assert!(
            H264Settings {
                frame_rate: 61,
                ..valid
            }
            .validate()
            .is_err()
        );
    }
}

#[allow(dead_code)]
fn _wide_name(activation: &IMFActivate) -> Option<String> {
    unsafe { activation.GetStringLength(&MFT_FRIENDLY_NAME_Attribute) }
        .ok()
        .and_then(|length| {
            let mut value = vec![0u16; length as usize + 1];
            unsafe { activation.GetString(&MFT_FRIENDLY_NAME_Attribute, &mut value, None) }
                .ok()
                .map(|_| String::from_utf16_lossy(&value))
        })
}
