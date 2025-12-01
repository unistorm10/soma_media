//! soma_media - FFmpeg-based audio/video preprocessing for SOMA platform
//!
//! Provides unified interface for media preprocessing via FFmpeg shell commands.
//! All operations shell out to system `ffmpeg` binary (LGPL-safe, no linking).

//! # soma_media - SOMA Media Processing Organ
//!
//! FFmpeg-based audio/video/image preprocessing organ for the SOMA platform.
//! Provides UMA-compliant media processing operations for AI/ML pipelines.
//!
//! ## Features
//!
//! - **Audio Processing**: Format conversion, mel spectrograms for CLAP/Whisper
//! - **Video Processing**: Frame extraction for CLIP and vision models
//! - **Image Processing**: Format conversion, resizing, quality control
//! - **RAW Processing**: Fast preview extraction (11-38x faster), metadata extraction
//! - **GPU Acceleration**: Automatic CUDA → Vulkan → CPU cascade (optional)
//! - **UMA Compliant**: Full Universal Module Architecture support
//!
//! ## Usage
//!
//! ### As a Library (Embedded Mode)
//!
//! ```rust,no_run
//! use soma_media::{RawProcessor, PreviewOptions};
//! use std::path::Path;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Extract RAW preview
//! let processor = RawProcessor::new()?;
//! let webp = processor.extract_preview_webp(
//!     Path::new("photo.CR2"),
//!     &PreviewOptions::default()
//! )?;
//! # Ok(())
//! # }
//! ```
//!
//! ### As a Daemon (Server Mode)
//!
//! ```bash
//! # Start the UMA organ daemon
//! soma_media --socket-path /tmp/soma_media.sock
//! ```
//!
//! ### Via UMA Interface
//!
//! ```rust,no_run
//! use soma_media::organ::{MediaOrgan, Organ, Stimulus};
//! use serde_json::json;
//! use std::collections::HashMap;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let organ = MediaOrgan::new();
//!
//! let stimulus = Stimulus {
//!     op: "raw.preview".to_string(),
//!     input: json!({
//!         "input_path": "photo.CR2",
//!         "output_path": "/tmp/preview.webp",
//!         "quality": 92
//!     }),
//!     context: HashMap::new(),
//! };
//!
//! let response = organ.stimulate(stimulus).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Available Operations
//!
//! - `audio.preprocess` - Convert audio to target format/sample rate
//! - `audio.mel_spectrogram` - Generate mel spectrograms
//! - `video.extract_frames` - Extract frames at specified FPS
//! - `image.preprocess` - Convert/resize images
//! - `raw.preview` - Fast RAW preview extraction (WebP Q92)
//! - `raw.metadata` - Extract EXIF/camera metadata
//! - `media.capabilities` - Get organ capability card
//!
//! ## GPU Acceleration
//!
//! Enable GPU features for 5-10x performance boost:
//!
//! ```toml
//! [dependencies]
//! soma_media = { version = "*", features = ["gpu-auto"] }
//! ```
//!
//! GPU backend automatically selected: CUDA → Vulkan → CPU
//!
//! ## Performance
//!
//! - **RAW preview**: ~15-85ms (embedded preview) or ~255ms (RAW processing)
//! - **Speedup**: 11-38x faster than full RAW processing
//! - **Batch processing**: 500+ images/second with GPU
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────┐
//! │              soma_media Organ                   │
//! ├─────────────────────────────────────────────────┤
//! │  UMA Interface (Stimulus/Response)              │
//! ├──────────────┬──────────────┬───────────────────┤
//! │ Audio        │ Video        │ RAW               │
//! │ - FFmpeg     │ - FFmpeg     │ - LibRaw          │
//! │ - Mel spec   │ - Frames     │ - Fast preview    │
//! │              │              │ - Metadata        │
//! ├──────────────┴──────────────┴───────────────────┤
//! │  GPU Acceleration (optional)                    │
//! │  CUDA → Vulkan/Metal → CPU (SIMD)              │
//! └─────────────────────────────────────────────────┘
//! ```

mod audio;
mod video;
mod image;
mod ffmpeg;
mod raw;
mod validation;
mod metrics;

// GPU acceleration via soma_compute (UMA)
pub mod compute_client;
pub mod gpu;

// Parallel demosaic for RAW processing
pub mod demosaic;

// RAW metadata extraction
pub mod metadata;

// Camera color profiles (MakerNotes-based)
pub mod profiles;

pub mod organ;
pub mod error;

pub use metrics::{Metrics, MetricsSnapshot};

pub use audio::{AudioPreprocessor, AudioConfig, MelSpectrogram, AudioFormat};
pub use video::{VideoPreprocessor, VideoConfig, VideoFrame};
pub use image::{ImagePreprocessor, ImageConfig, ImageOutputFormat};
pub use ffmpeg::{FfmpegCommand, FfmpegError};
pub use raw::{RawProcessor, RawOptions, WhiteBalance, ColorSpace, PreviewOptions};

// Universal metadata extraction (ExifTool + fallbacks)
pub use metadata::{
    MediaMetadata, RawMetadata,  // RawMetadata is alias for backward compatibility
    extract_metadata, exiftool_available, ffprobe_available,
    detect_mime_from_extension,
    GpsCoordinates, LensInfo, ExposureInfo, ColorInfo, Dimensions,
    SensorInfo, ShootingInfo, MetadataBackend,
};

// Camera color profiles (extract from MakerNotes, apply to match in-camera JPEG)
pub use profiles::{
    CameraProfile, extract_camera_profile,
    ColorMatrix, WbMultipliers, ToneCurve, PictureStyle,
};

// GPU acceleration via soma_compute UMA (always available, falls back to CPU)
pub use gpu::{GpuProcessor, GpuBackend};
pub use compute_client::{ComputeClient, ComputeRequest, ComputeResponse};


// UMA interface exports
pub use organ::{
    Organ, MediaOrgan, Stimulus, Response, OrganError,
    OrganCard, FunctionCard
};

pub type Result<T> = std::result::Result<T, FfmpegError>;
