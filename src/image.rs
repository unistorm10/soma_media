//! Image preprocessing

use crate::ffmpeg::{FfmpegCommand, FfmpegError};
use std::path::Path;
use image::{DynamicImage, ImageFormat};

pub struct ImageConfig {
    pub width: u32,
    pub height: u32,
    pub format: ImageOutputFormat,
    pub quality: u8, // 1-100 for JPEG/WEBP
}

impl Default for ImageConfig {
    fn default() -> Self {
        Self {
            width: 336,   // CLIP ViT-H/14 default
            height: 336,
            format: ImageOutputFormat::Jpeg,
            quality: 90,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ImageOutputFormat {
    Jpeg,
    Png,
    Webp,
    Avif,
}

impl ImageOutputFormat {
    pub fn as_str(&self) -> &str {
        match self {
            ImageOutputFormat::Jpeg => "jpg",
            ImageOutputFormat::Png => "png",
            ImageOutputFormat::Webp => "webp",
            ImageOutputFormat::Avif => "avif",
        }
    }
    
    pub fn extension(&self) -> &str {
        match self {
            ImageOutputFormat::Jpeg => "jpg",
            ImageOutputFormat::Png => "png",
            ImageOutputFormat::Webp => "webp",
            ImageOutputFormat::Avif => "avif",
        }
    }
}

pub struct ImagePreprocessor {
    config: ImageConfig,
}

impl ImagePreprocessor {
    pub fn new(config: ImageConfig) -> Self {
        Self { config }
    }
    
    /// Resize and convert image using FFmpeg (supports more formats than image crate)
    pub fn preprocess(&self, input: impl AsRef<Path>, output: impl AsRef<Path>) -> Result<(), FfmpegError> {
        let mut cmd = FfmpegCommand::new()
            .input(input)
            .args(&[
                "-vf", &format!("scale={}:{}", self.config.width, self.config.height),
            ]);
        
        // Add format-specific options
        match self.config.format {
            ImageOutputFormat::Jpeg => {
                cmd = cmd.args(&["-q:v", &((100 - self.config.quality) / 3).to_string()]);
            }
            ImageOutputFormat::Webp => {
                cmd = cmd.args(&["-quality", &self.config.quality.to_string()]);
            }
            ImageOutputFormat::Avif => {
                cmd = cmd.args(&["-crf", &((100 - self.config.quality) / 2).to_string()]);
            }
            ImageOutputFormat::Png => {
                // PNG is lossless, no quality setting
            }
        }
        
        cmd.output(output).execute()?;
        
        Ok(())
    }
    
    /// Load image using image crate (for in-memory processing)
    pub fn load(&self, path: impl AsRef<Path>) -> Result<DynamicImage, FfmpegError> {
        image::open(path)
            .map_err(|e| FfmpegError::ExecutionFailed(format!("Failed to load image: {}", e)))
    }
    
    /// Resize image using image crate
    pub fn resize(&self, img: &DynamicImage) -> DynamicImage {
        img.resize_exact(
            self.config.width,
            self.config.height,
            image::imageops::FilterType::Lanczos3,
        )
    }
    
    /// Save image using image crate
    pub fn save(&self, img: &DynamicImage, path: impl AsRef<Path>) -> Result<(), FfmpegError> {
        let format = match self.config.format {
            ImageOutputFormat::Jpeg => ImageFormat::Jpeg,
            ImageOutputFormat::Png => ImageFormat::Png,
            ImageOutputFormat::Webp => ImageFormat::WebP,
            ImageOutputFormat::Avif => ImageFormat::Avif,
        };
        
        img.save_with_format(path, format)
            .map_err(|e| FfmpegError::ExecutionFailed(format!("Failed to save image: {}", e)))
    }
    
    /// Convert DNG/RAW using FFmpeg
    pub fn convert_raw(&self, input: impl AsRef<Path>, output: impl AsRef<Path>) -> Result<(), FfmpegError> {
        FfmpegCommand::new()
            .input(input)
            .args(&[
                "-vf", &format!("scale={}:{}", self.config.width, self.config.height),
                "-pix_fmt", "rgb24",
            ])
            .output(output)
            .execute()?;
        
        Ok(())
    }
}
