#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AudioSource {
    pub id: String,
    pub label: String,
    pub kind: AudioSourceKind,
    pub process_id: Option<u32>,
}

#[derive(Debug)]
pub struct AudioPcmBlock {
    pub samples: Vec<i16>,
    pub timestamp_100ns: i64,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioSourceKind {
    Microphone,
    Desktop,
    Application,
}
#[derive(Debug, thiserror::Error)]
pub enum AudioError {
    #[error("WASAPI capture requires Windows")]
    UnsupportedPlatform,
    #[error("Application audio capture requires Windows build 20348 or later")]
    UnsupportedWindowsBuild,
    #[error("Audio capture failed: {0}")]
    Backend(String),
}

#[cfg(windows)]
pub mod wasapi;

#[cfg(windows)]
pub use wasapi::AudioInputLevel;

pub fn process_loopback_supported(build: u32) -> bool {
    build >= 20348
}

#[cfg(test)]
mod tests {
    #[test]
    fn process_audio_has_explicit_os_floor() {
        assert!(!super::process_loopback_supported(20347));
        assert!(super::process_loopback_supported(20348));
    }
}
