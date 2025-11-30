use thiserror::Error;

#[derive(Error, Debug)]
pub enum MediaError {
    #[error("Dependency not found: {0}")]
    DependencyNotFound(String),
    
    #[error("Processing failed: {0}")]
    ProcessingError(String),
    
    #[error("Validation failed: {0}")]
    ValidationError(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("FFmpeg error: {0}")]
    Ffmpeg(#[from] crate::ffmpeg::FfmpegError),
}

pub type Result<T> = std::result::Result<T, MediaError>;
