#[derive(Debug, thiserror::Error)]
pub enum ComposeError {
    #[error("GPU composition is not initialized")]
    NotInitialized,
    #[error("D3D11 composition failed: {0}")]
    Backend(String),
}

#[derive(Debug, Clone, Copy)]
pub struct Canvas {
    pub width: u32,
    pub height: u32,
}

#[cfg(windows)]
pub mod windows;

impl Canvas {
    pub fn new(width: u32, height: u32) -> Result<Self, ComposeError> {
        if width == 0 || height == 0 || width > 1920 || height > 1080 {
            return Err(ComposeError::NotInitialized);
        }
        Ok(Self { width, height })
    }
}
