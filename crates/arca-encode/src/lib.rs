#[derive(Debug, thiserror::Error)]
pub enum EncodeError {
    #[error(
        "No supported Media Foundation hardware H.264 encoder was found. Update your GPU driver or connect supported hardware."
    )]
    HardwareEncoderUnavailable,
    #[error("Hardware encoder initialization failed: {0}")]
    Initialization(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HardwareEncoder {
    Nvidia,
    Amd,
    Intel,
    Other,
}

pub fn discover_hardware_encoder() -> Result<HardwareEncoder, EncodeError> {
    #[cfg(windows)]
    {
        mf::discover()
    }
    #[cfg(not(windows))]
    {
        Err(EncodeError::HardwareEncoderUnavailable)
    }
}

#[cfg(windows)]
mod mf_encoder;

#[cfg(windows)]
mod mf_aac;

#[cfg(windows)]
pub use mf_encoder::{EncodedSample, H264Settings, HardwareH264Encoder};

#[cfg(windows)]
pub use mf_aac::{EncodedAudioSample, HardwareAacEncoder};

#[cfg(windows)]
mod mf {
    use super::{EncodeError, HardwareEncoder};
    use windows::Win32::Media::MediaFoundation::*;
    use windows::Win32::System::Com::CoTaskMemFree;

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
            let list = unsafe { std::slice::from_raw_parts(self.0, self.1 as usize) };
            for item in list.iter().flatten() {
                let _ = unsafe { item.ShutdownObject() };
            }
            unsafe {
                CoTaskMemFree(Some(self.0.cast()));
            }
        }
    }

    pub fn discover() -> Result<HardwareEncoder, EncodeError> {
        let _mf = MediaFoundation::start()?;
        let output_type = MFT_REGISTER_TYPE_INFO {
            guidMajorType: MFMediaType_Video,
            guidSubtype: MFVideoFormat_H264,
        };
        let mut count = 0;
        let mut activated: *mut Option<IMFActivate> = std::ptr::null_mut();
        let result = unsafe {
            MFTEnumEx(
                MFT_CATEGORY_VIDEO_ENCODER,
                MFT_ENUM_FLAG_HARDWARE | MFT_ENUM_FLAG_SORTANDFILTER,
                None,
                Some(&output_type),
                &mut activated,
                &mut count,
            )
        };
        if let Err(error) = result {
            if !activated.is_null() {
                unsafe {
                    CoTaskMemFree(Some(activated.cast()));
                }
            }
            return Err(EncodeError::Initialization(error.to_string()));
        }
        let _activations = Activations(activated, count);
        if count == 0 || activated.is_null() {
            return Err(EncodeError::HardwareEncoderUnavailable);
        }
        let list = unsafe { std::slice::from_raw_parts(activated, count as usize) };
        let mut selected = None;
        for item in list.iter().flatten() {
            let name = unsafe { item.GetStringLength(&MFT_FRIENDLY_NAME_Attribute) }
                .ok()
                .and_then(|len| {
                    let mut buf = vec![0u16; len as usize + 1];
                    unsafe { item.GetString(&MFT_FRIENDLY_NAME_Attribute, &mut buf, None) }
                        .ok()
                        .map(|_| String::from_utf16_lossy(&buf))
                })
                .unwrap_or_default()
                .to_lowercase();
            let vendor = if name.contains("nvidia") {
                Some(HardwareEncoder::Nvidia)
            } else if name.contains("amd") || name.contains("advanced micro") {
                Some(HardwareEncoder::Amd)
            } else if name.contains("intel") {
                Some(HardwareEncoder::Intel)
            } else {
                None
            };
            if vendor.is_some() {
                selected = vendor;
                break;
            }
        }
        Ok(selected.unwrap_or(HardwareEncoder::Other))
    }
}
