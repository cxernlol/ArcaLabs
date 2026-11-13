#[derive(Debug, thiserror::Error)]
pub enum CaptureError {
    #[error("Windows Graphics Capture is available on Windows only")]
    UnsupportedPlatform,
    #[error("Capture target is no longer available")]
    TargetUnavailable,
    #[error("Windows capture failed: {0}")]
    Backend(String),
}

#[cfg(windows)]
pub mod windows;

pub const MAX_OUTPUT: (u32, u32) = (1920, 1080);
