use std::mem::ManuallyDrop;
use windows::{
    Win32::{
        Media::MediaFoundation::*,
        System::Com::{
            CLSCTX_INPROC_SERVER, COINIT_MULTITHREADED, CoCreateInstance, CoInitializeEx,
            CoUninitialize,
        },
    },
    core::GUID,
};

const AAC_ENCODER: GUID = GUID::from_u128(0x93af0c51_2275_45d2_a35b_f2ba21caed00);
const SAMPLE_RATE: u32 = 48_000;
const CHANNELS: u32 = 2;
const FRAMES_PER_SAMPLE: usize = 1024;
const BITRATE: u32 = 128_000;
const AAC_LC_48K_STEREO: [u8; 2] = [0x11, 0x90];

pub struct EncodedAudioSample {
    pub bytes: Vec<u8>,
    pub timestamp_100ns: i64,
    pub duration_100ns: i64,
}

struct ComApartment;

impl ComApartment {
    fn initialize() -> Result<Self, super::EncodeError> {
        unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) }
            .ok()
            .map_err(|error| super::EncodeError::Initialization(error.to_string()))?;
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
    fn start() -> Result<Self, super::EncodeError> {
        unsafe { MFStartup(MF_VERSION, MFSTARTUP_FULL) }
            .map_err(|error| super::EncodeError::Initialization(error.to_string()))?;
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

/// Media Foundation AAC-LC encoder. Construct and use on one dedicated MTA worker.
pub struct HardwareAacEncoder {
    transform: IMFTransform,
    _mf: MediaFoundation,
    _com: ComApartment,
    input_stream: u32,
    output_stream: u32,
    output_buffer_size: u32,
    next_timestamp_100ns: i64,
}

impl HardwareAacEncoder {
    pub fn new() -> Result<Self, super::EncodeError> {
        let com = ComApartment::initialize()?;
        let mf = MediaFoundation::start()?;
        let transform: IMFTransform =
            unsafe { CoCreateInstance(&AAC_ENCODER, None, CLSCTX_INPROC_SERVER) }
                .map_err(init_error)?;

        let mut input_count = 0;
        let mut output_count = 0;
        unsafe { transform.GetStreamCount(&mut input_count, &mut output_count) }
            .map_err(init_error)?;
        if input_count == 0 || output_count == 0 {
            return Err(super::EncodeError::Initialization(
                "Media Foundation AAC encoder exposes no streams".into(),
            ));
        }
        let mut input_ids = vec![0; input_count as usize];
        let mut output_ids = vec![0; output_count as usize];
        if unsafe { transform.GetStreamIDs(&mut input_ids, &mut output_ids) }.is_err() {
            input_ids[0] = 0;
            output_ids[0] = 0;
        }
        let input_stream = input_ids[0];
        let output_stream = output_ids[0];
        let input_type = audio_type(MFAudioFormat_PCM, false)?;
        let output_type = audio_type(MFAudioFormat_AAC, true)?;
        unsafe {
            transform
                .SetInputType(input_stream, &input_type, 0)
                .map_err(init_error)?;
            transform
                .SetOutputType(output_stream, &output_type, 0)
                .map_err(init_error)?;
            transform
                .ProcessMessage(MFT_MESSAGE_NOTIFY_BEGIN_STREAMING, 0)
                .map_err(init_error)?;
            transform
                .ProcessMessage(MFT_MESSAGE_NOTIFY_START_OF_STREAM, 0)
                .map_err(init_error)?;
        }
        let output_info =
            unsafe { transform.GetOutputStreamInfo(output_stream) }.map_err(init_error)?;
        Ok(Self {
            transform,
            _mf: mf,
            _com: com,
            input_stream,
            output_stream,
            output_buffer_size: output_info.cbSize.max(4096),
            next_timestamp_100ns: 0,
        })
    }

    pub fn sequence_header(&self) -> &[u8] {
        &AAC_LC_48K_STEREO
    }

    /// Encodes exactly 1024 interleaved stereo frames and advances by their 48 kHz duration.
    pub fn encode(&mut self, pcm: &[i16]) -> Result<Vec<EncodedAudioSample>, super::EncodeError> {
        self.encode_at(pcm, self.next_timestamp_100ns)
    }

    pub fn encode_at(
        &mut self,
        pcm: &[i16],
        timestamp: i64,
    ) -> Result<Vec<EncodedAudioSample>, super::EncodeError> {
        if pcm.len() != FRAMES_PER_SAMPLE * CHANNELS as usize {
            return Err(super::EncodeError::Initialization(
                "AAC input must contain exactly 1024 stereo PCM frames".into(),
            ));
        }
        let duration = FRAMES_PER_SAMPLE as i64 * 10_000_000 / i64::from(SAMPLE_RATE);
        let bytes = unsafe {
            std::slice::from_raw_parts(pcm.as_ptr().cast::<u8>(), std::mem::size_of_val(pcm))
        };
        let media_buffer =
            unsafe { MFCreateMemoryBuffer(bytes.len() as u32) }.map_err(init_error)?;
        let mut destination = std::ptr::null_mut();
        unsafe {
            media_buffer
                .Lock(&mut destination, None, None)
                .map_err(init_error)?;
            if destination.is_null() {
                let _ = media_buffer.Unlock();
                return Err(super::EncodeError::Initialization(
                    "Media Foundation returned an empty AAC input buffer".into(),
                ));
            }
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), destination, bytes.len());
            media_buffer.Unlock().map_err(init_error)?;
            media_buffer
                .SetCurrentLength(bytes.len() as u32)
                .map_err(init_error)?;
        }
        let sample = unsafe { MFCreateSample() }.map_err(init_error)?;
        unsafe {
            sample.AddBuffer(&media_buffer).map_err(init_error)?;
            sample.SetSampleTime(timestamp).map_err(init_error)?;
            sample.SetSampleDuration(duration).map_err(init_error)?;
            self.transform
                .ProcessInput(self.input_stream, &sample, 0)
                .map_err(init_error)?;
        }
        self.next_timestamp_100ns = timestamp.saturating_add(duration);

        let mut encoded = Vec::new();
        loop {
            let output_info = unsafe { self.transform.GetOutputStreamInfo(self.output_stream) }
                .map_err(init_error)?;
            let supplied = output_info.dwFlags & MFT_OUTPUT_STREAM_PROVIDES_SAMPLES.0 as u32 != 0;
            let caller_sample = if supplied {
                None
            } else {
                let buffer =
                    unsafe { MFCreateMemoryBuffer(self.output_buffer_size) }.map_err(init_error)?;
                let sample = unsafe { MFCreateSample() }.map_err(init_error)?;
                unsafe {
                    sample.AddBuffer(&buffer).map_err(init_error)?;
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
            drop(unsafe { ManuallyDrop::take(&mut output[0].pEvents) });
            if let Err(error) = result {
                if error.code() == MF_E_TRANSFORM_NEED_MORE_INPUT {
                    break;
                }
                return Err(init_error(error));
            }
            let Some(sample) = sample else { break };
            let buffer = unsafe { sample.ConvertToContiguousBuffer() }.map_err(init_error)?;
            let mut data = std::ptr::null_mut();
            let mut length = 0;
            unsafe { buffer.Lock(&mut data, None, Some(&mut length)) }.map_err(init_error)?;
            let bytes = if data.is_null() || length == 0 {
                Vec::new()
            } else {
                unsafe { std::slice::from_raw_parts(data, length as usize) }.to_vec()
            };
            unsafe { buffer.Unlock() }.map_err(init_error)?;
            if !bytes.is_empty() {
                encoded.push(EncodedAudioSample {
                    bytes,
                    timestamp_100ns: unsafe { sample.GetSampleTime() }.unwrap_or(timestamp),
                    duration_100ns: unsafe { sample.GetSampleDuration() }.unwrap_or(duration),
                });
            }
        }
        Ok(encoded)
    }
}

impl Drop for HardwareAacEncoder {
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

fn audio_type(subtype: GUID, encoded: bool) -> Result<IMFMediaType, super::EncodeError> {
    let media_type = unsafe { MFCreateMediaType() }.map_err(init_error)?;
    unsafe {
        media_type
            .SetGUID(&MF_MT_MAJOR_TYPE, &MFMediaType_Audio)
            .map_err(init_error)?;
        media_type
            .SetGUID(&MF_MT_SUBTYPE, &subtype)
            .map_err(init_error)?;
        media_type
            .SetUINT32(&MF_MT_AUDIO_SAMPLES_PER_SECOND, SAMPLE_RATE)
            .map_err(init_error)?;
        media_type
            .SetUINT32(&MF_MT_AUDIO_NUM_CHANNELS, CHANNELS)
            .map_err(init_error)?;
        media_type
            .SetUINT32(&MF_MT_AUDIO_BITS_PER_SAMPLE, 16)
            .map_err(init_error)?;
        media_type
            .SetUINT32(&MF_MT_AUDIO_BLOCK_ALIGNMENT, if encoded { 1 } else { 4 })
            .map_err(init_error)?;
        media_type
            .SetUINT32(
                &MF_MT_AUDIO_AVG_BYTES_PER_SECOND,
                if encoded {
                    BITRATE / 8
                } else {
                    SAMPLE_RATE * 4
                },
            )
            .map_err(init_error)?;
        media_type
            .SetUINT32(
                &MF_MT_AVG_BITRATE,
                if encoded {
                    BITRATE
                } else {
                    SAMPLE_RATE * CHANNELS * 16
                },
            )
            .map_err(init_error)?;
        if encoded {
            media_type
                .SetUINT32(&MF_MT_AAC_PAYLOAD_TYPE, 0)
                .map_err(init_error)?;
            media_type
                .SetUINT32(&MF_MT_AAC_AUDIO_PROFILE_LEVEL_INDICATION, 0x29)
                .map_err(init_error)?;
        } else {
            media_type
                .SetUINT32(&MF_MT_ALL_SAMPLES_INDEPENDENT, 1)
                .map_err(init_error)?;
            media_type
                .SetUINT32(&MF_MT_FIXED_SIZE_SAMPLES, 1)
                .map_err(init_error)?;
        }
    }
    Ok(media_type)
}

fn init_error(error: windows::core::Error) -> super::EncodeError {
    super::EncodeError::Initialization(error.to_string())
}
