//! soma_media - FFmpeg-based audio/video preprocessing for SOMA platform
//!
//! Provides unified interface for media preprocessing via FFmpeg shell commands.
//! All operations shell out to system `ffmpeg` binary (LGPL-safe, no linking).

mod audio;
mod video;
mod image;
mod ffmpeg;
mod organ;

pub use audio::{AudioPreprocessor, AudioConfig, MelSpectrogram, AudioFormat};
pub use video::{VideoPreprocessor, VideoConfig, VideoFrame};
pub use image::{ImagePreprocessor, ImageConfig, ImageOutputFormat};
pub use ffmpeg::{FfmpegCommand, FfmpegError};

// UMA interface exports
pub use organ::{
    Organ, MediaOrgan, Stimulus, Response, OrganError, 
    OrganCard, FunctionCard
};

pub type Result<T> = std::result::Result<T, FfmpegError>;
