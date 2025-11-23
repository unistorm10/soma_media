//! FFmpeg command wrapper utilities

use std::process::{Command, Output};
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum FfmpegError {
    #[error("FFmpeg not found in system PATH")]
    NotInstalled,
    
    #[error("FFmpeg execution failed: {0}")]
    ExecutionFailed(String),
    
    #[error("Invalid output: {0}")]
    InvalidOutput(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub struct FfmpegCommand {
    args: Vec<String>,
}

impl FfmpegCommand {
    pub fn new() -> Self {
        Self { args: Vec::new() }
    }
    
    pub fn input(mut self, path: impl AsRef<Path>) -> Self {
        self.args.push("-i".to_string());
        self.args.push(path.as_ref().display().to_string());
        self
    }
    
    pub fn output(mut self, path: impl AsRef<Path>) -> Self {
        self.args.push(path.as_ref().display().to_string());
        self
    }
    
    pub fn args(mut self, args: &[&str]) -> Self {
        self.args.extend(args.iter().map(|s| s.to_string()));
        self
    }
    
    pub fn execute(self) -> Result<Output, FfmpegError> {
        // Check FFmpeg exists
        if !is_ffmpeg_installed() {
            return Err(FfmpegError::NotInstalled);
        }
        
        let output = Command::new("ffmpeg")
            .args(&self.args)
            .output()
            .map_err(|e| FfmpegError::ExecutionFailed(e.to_string()))?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(FfmpegError::ExecutionFailed(stderr.to_string()));
        }
        
        Ok(output)
    }
}

impl Default for FfmpegCommand {
    fn default() -> Self {
        Self::new()
    }
}

fn is_ffmpeg_installed() -> bool {
    Command::new("ffmpeg")
        .arg("-version")
        .output()
        .is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_ffmpeg_installed() {
        assert!(is_ffmpeg_installed(), "FFmpeg not found - install via: sudo apt install ffmpeg");
    }
}
