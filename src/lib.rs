//! soma_media - FFmpeg-based audio/video preprocessing for SOMA platform
//!
//! Provides unified interface for media preprocessing via FFmpeg shell commands.
//! All operations shell out to system `ffmpeg` binary (LGPL-safe, no linking).

mod audio;
mod video;
mod ffmpeg;

pub use audio::{AudioPreprocessor, AudioConfig, MelSpectrogram, AudioFormat};
pub use video::{VideoPreprocessor, VideoConfig, VideoFrame};
pub use ffmpeg::{FfmpegCommand, FfmpegError};

pub type Result<T> = std::result::Result<T, FfmpegError>;
