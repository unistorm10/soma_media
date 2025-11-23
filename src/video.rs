//! Video preprocessing via FFmpeg

use crate::ffmpeg::{FfmpegCommand, FfmpegError};
use std::path::{Path, PathBuf};
use image::DynamicImage;

pub struct VideoConfig {
    pub fps: u8,
    pub width: u32,
    pub height: u32,
    pub max_frames: Option<usize>,
}

impl Default for VideoConfig {
    fn default() -> Self {
        Self {
            fps: 1,           // 1 frame per second
            width: 336,       // CLIP ViT-H/14 input
            height: 336,
            max_frames: Some(10),  // Limit to 10 frames
        }
    }
}

pub struct VideoFrame {
    pub image: DynamicImage,
    pub timestamp_ms: u64,
    pub frame_number: usize,
}

pub struct VideoPreprocessor {
    config: VideoConfig,
}

impl VideoPreprocessor {
    pub fn new(config: VideoConfig) -> Self {
        Self { config }
    }
    
    /// Extract frames from video and save to directory
    pub fn extract_frames(&self, video: impl AsRef<Path>, output_dir: impl AsRef<Path>) -> Result<Vec<PathBuf>, FfmpegError> {
        let output_pattern = output_dir.as_ref().join("frame_%04d.jpg");
        
        let mut cmd = FfmpegCommand::new()
            .input(video)
            .args(&[
                "-vf", &format!("fps={},scale={}:{}", 
                    self.config.fps, 
                    self.config.width, 
                    self.config.height
                ),
                "-f", "image2",
            ]);
        
        if let Some(max) = self.config.max_frames {
            cmd = cmd.args(&["-frames:v", &max.to_string()]);
        }
        
        cmd.output(output_pattern).execute()?;
        
        // Collect extracted frame paths
        let mut frames = Vec::new();
        for i in 1..=self.config.max_frames.unwrap_or(1000) {
            let frame_path = output_dir.as_ref().join(format!("frame_{:04}.jpg", i));
            if frame_path.exists() {
                frames.push(frame_path);
            } else {
                break;
            }
        }
        
        Ok(frames)
    }
    
    /// Load extracted frames as VideoFrame objects
    pub fn load_frames(&self, frame_paths: &[PathBuf]) -> Result<Vec<VideoFrame>, FfmpegError> {
        let mut frames = Vec::new();
        
        for (idx, path) in frame_paths.iter().enumerate() {
            let image = image::open(path)
                .map_err(|e| FfmpegError::InvalidOutput(e.to_string()))?;
            
            frames.push(VideoFrame {
                image,
                timestamp_ms: (idx as u64 * 1000) / self.config.fps as u64,
                frame_number: idx,
            });
        }
        
        Ok(frames)
    }
}
