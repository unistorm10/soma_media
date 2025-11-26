//! Image preprocessing

use crate::ffmpeg::{FfmpegCommand, FfmpegError};
use crate::raw::{RawProcessor, RawOptions, WhiteBalance};
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
    raw: Option<RawProcessor>,
}

impl ImagePreprocessor {
    pub fn new(config: ImageConfig) -> Self {
        // Try to initialize RAW processor (libraw FFI)
        let raw = RawProcessor::new().ok();
        Self { config, raw }
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
    
    /// Convert RAW files using libraw FFI or FFmpeg for standard image formats
    pub fn convert_raw(&self, input: impl AsRef<Path>, output: impl AsRef<Path>) -> Result<(), FfmpegError> {
        self.convert_raw_with_options(input, output, &RawOptions::default())
    }
    
    /// Convert RAW files with custom processing options
    pub fn convert_raw_with_options(
        &self,
        input: impl AsRef<Path>,
        output: impl AsRef<Path>,
        raw_options: &RawOptions,
    ) -> Result<(), FfmpegError> {
        let input_path = input.as_ref();
        let output_path = output.as_ref();
        
        // Read file once to avoid double read over network
        let file_data = std::fs::read(input_path)
            .map_err(|e| FfmpegError::ExecutionFailed(format!("Failed to read file: {}", e)))?;
        
        // Check if it's a RAW format by trying to open with libraw
        let is_raw = if let Some(raw_proc) = &self.raw {
            raw_proc.is_raw_data(&file_data)
        } else {
            false
        };
        
        if is_raw {
            // Use libraw FFI for all RAW formats
            if let Some(raw_proc) = &self.raw {
                // Process from memory (file already read)
                let rgb_data = raw_proc.process_raw_from_memory(&file_data, raw_options)
                    .map_err(|e| FfmpegError::ExecutionFailed(format!("libraw failed: {}", e)))?;
                
                // Get dimensions from memory
                let (width, height) = raw_proc.get_dimensions_from_memory(&file_data, raw_options)
                    .map_err(|e| FfmpegError::ExecutionFailed(format!("Failed to get dimensions: {}", e)))?;
                
                // Fast SIMD resize using fast_image_resize
                use fast_image_resize as fr;
                use fr::images::Image as FrImage;
                
                let src_image = FrImage::from_vec_u8(
                    width,
                    height,
                    rgb_data,
                    fr::PixelType::U8x3,
                ).map_err(|e| FfmpegError::ExecutionFailed(format!("Failed to create source image: {:?}", e)))?;
                
                let mut dst_image = FrImage::new(
                    self.config.width,
                    self.config.height,
                    src_image.pixel_type(),
                );
                
                let mut resizer = fr::Resizer::new();
                resizer.resize(&src_image, &mut dst_image, None)
                    .map_err(|e| FfmpegError::ExecutionFailed(format!("Resize failed: {:?}", e)))?;
                
                let rgb_bytes = dst_image.buffer().to_vec();
                
                // Fast direct encoding via FFI (no subprocess, no temp files)
                match self.config.format {
                    ImageOutputFormat::Webp => {
                        // Direct libwebp FFI encoding
                        let encoder = webp::Encoder::from_rgb(&rgb_bytes, self.config.width, self.config.height);
                        let webp_data = encoder.encode(self.config.quality as f32);
                        
                        std::fs::write(output_path, &*webp_data)
                            .map_err(|e| FfmpegError::ExecutionFailed(format!("Failed to write WEBP: {}", e)))?;
                    }
                    ImageOutputFormat::Jpeg | ImageOutputFormat::Png | ImageOutputFormat::Avif => {
                        // Fall back to image crate for other formats
                        // Recreate image from resized RGB bytes
                        let img = image::RgbImage::from_raw(self.config.width, self.config.height, rgb_bytes.clone())
                            .ok_or_else(|| FfmpegError::ExecutionFailed("Failed to create image from resized data".to_string()))?;
                        let dynamic_img = image::DynamicImage::ImageRgb8(img);
                        
                        let format = match self.config.format {
                            ImageOutputFormat::Jpeg => ImageFormat::Jpeg,
                            ImageOutputFormat::Png => ImageFormat::Png,
                            ImageOutputFormat::Avif => ImageFormat::Avif,
                            _ => unreachable!(),
                        };
                        
                        dynamic_img.save_with_format(output_path, format)
                            .map_err(|e| FfmpegError::ExecutionFailed(format!("Failed to save image: {}", e)))?;
                    }
                }
                
                Ok(())
            } else {
                Err(FfmpegError::ExecutionFailed(
                    "libraw not available - ensure rsraw crate is properly linked".to_string()
                ))
            }
        } else {
            // Use FFmpeg for standard image formats (JPEG, PNG, TIFF, etc.)
            FfmpegCommand::new()
                .input(input_path)
                .args(&[
                    "-vf", &format!("scale={}:{}", self.config.width, self.config.height),
                    "-pix_fmt", "rgb24",
                ])
                .output(output_path)
                .execute()?;
            
            Ok(())
        }
    }
}
