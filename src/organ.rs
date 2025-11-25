//! UMA Organ Interface for soma_media
//!
//! Implements the Universal Module Architecture (UMA) interface for FFmpeg-based
//! audio/video preprocessing, enabling dynamic discovery and invocation by SOMA orchestrators.

use crate::{AudioPreprocessor, AudioConfig, AudioFormat, VideoPreprocessor, VideoConfig, ImagePreprocessor, ImageConfig, ImageOutputFormat, FfmpegError};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::time::Instant;
use thiserror::Error;

/// UMA Stimulus - input to organ operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stimulus {
    pub op: String,
    pub input: Value,
    pub context: HashMap<String, String>,
}

/// UMA Response - output from organ operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    pub ok: bool,
    pub output: Value,
    pub latency_ms: u64,
    pub cost: Option<f64>,
}

/// Organ trait - all SOMA organs implement this
#[async_trait]
pub trait Organ: Send + Sync {
    async fn stimulate(&self, stimulus: Stimulus) -> Result<Response, OrganError>;
    fn describe(&self) -> OrganCard;
}

/// Organ-level errors
#[derive(Debug, Error)]
pub enum OrganError {
    #[error("Unsupported operation: {0}")]
    UnsupportedOperation(String),
    
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    
    #[error("Processing error: {0}")]
    ProcessingError(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    
    #[error("FFmpeg error: {0}")]
    FfmpegError(#[from] FfmpegError),
}

/// Organ capability card
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganCard {
    pub name: String,
    pub version: String,
    pub description: String,
    pub division: String,
    pub subsystem: String,
    pub tags: Vec<String>,
    pub functions: Vec<FunctionCard>,
}

/// Function capability card
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCard {
    pub name: String,
    pub description: String,
    pub tags: Vec<String>,
    pub examples: Vec<String>,
    pub idempotent: bool,
    pub side_effects: Vec<String>,
    pub input_schema: Option<Value>,
    pub output_schema: Value,
}

/// Media Processing Organ
pub struct MediaOrgan;

impl MediaOrgan {
    pub fn new() -> Self {
        Self
    }
    
    /// Handle audio.preprocess operation
    async fn handle_audio_preprocess(&self, input: Value) -> Result<Value, OrganError> {
        let input_path = input["input_path"]
            .as_str()
            .ok_or_else(|| OrganError::InvalidInput("Missing input_path".to_string()))?;
        let output_path = input["output_path"]
            .as_str()
            .ok_or_else(|| OrganError::InvalidInput("Missing output_path".to_string()))?;
        
        let sample_rate = input["sample_rate"].as_u64().unwrap_or(48000) as u32;
        let channels = input["channels"].as_u64().unwrap_or(1) as u16;
        
        let config = AudioConfig {
            sample_rate,
            channels,
            format: AudioFormat::Wav,
        };
        
        let processor = AudioPreprocessor::new(config);
        processor.preprocess(input_path, output_path)?;
        
        Ok(json!({
            "processed": true,
            "output_path": output_path,
            "sample_rate": sample_rate,
            "channels": channels
        }))
    }
    
    /// Handle audio.mel_spectrogram operation
    async fn handle_audio_mel_spectrogram(&self, input: Value) -> Result<Value, OrganError> {
        let _audio_path = input["audio_path"]
            .as_str()
            .ok_or_else(|| OrganError::InvalidInput("Missing audio_path".to_string()))?;
        
        let n_fft = input["n_fft"].as_u64().unwrap_or(2048) as usize;
        let _hop_length = input["hop_length"].as_u64().unwrap_or(512) as usize;
        let n_mels = input["n_mels"].as_u64().unwrap_or(128) as usize;
        
        // Note: Full mel spectrogram generation requires preprocessing audio first
        // This is a placeholder that shows the structure
        Ok(json!({
            "note": "Mel spectrogram generation requires audio preprocessing first",
            "n_fft": n_fft,
            "n_mels": n_mels,
            "implemented": false
        }))
    }
    
    /// Handle video.extract_frames operation
    async fn handle_video_extract_frames(&self, input: Value) -> Result<Value, OrganError> {
        let video_path = input["video_path"]
            .as_str()
            .ok_or_else(|| OrganError::InvalidInput("Missing video_path".to_string()))?;
        let output_dir = input["output_dir"]
            .as_str()
            .ok_or_else(|| OrganError::InvalidInput("Missing output_dir".to_string()))?;
        
        let fps = input["fps"].as_u64().unwrap_or(1) as u8;
        let width = input["width"].as_u64().unwrap_or(336) as u32;
        let height = input["height"].as_u64().unwrap_or(336) as u32;
        let max_frames = input["max_frames"].as_u64().map(|v| v as usize);
        
        let config = VideoConfig {
            fps,
            width,
            height,
            max_frames,
        };
        
        let processor = VideoPreprocessor::new(config);
        let frames = processor.extract_frames(video_path, output_dir)?;
        
        Ok(json!({
            "extracted": true,
            "frame_count": frames.len(),
            "frames": frames.iter().map(|p| p.to_string_lossy()).collect::<Vec<_>>()
        }))
    }
    
    /// Handle image.preprocess operation
    async fn handle_image_preprocess(&self, input: Value) -> Result<Value, OrganError> {
        let input_path = input["input_path"]
            .as_str()
            .ok_or_else(|| OrganError::InvalidInput("Missing input_path".to_string()))?;
        let output_path = input["output_path"]
            .as_str()
            .ok_or_else(|| OrganError::InvalidInput("Missing output_path".to_string()))?;
        
        let width = input["width"].as_u64().unwrap_or(336) as u32;
        let height = input["height"].as_u64().unwrap_or(336) as u32;
        let quality = input["quality"].as_u64().unwrap_or(90) as u8;
        
        let format_str = input["format"].as_str().unwrap_or("jpg");
        let format = match format_str.to_lowercase().as_str() {
            "jpg" | "jpeg" => ImageOutputFormat::Jpeg,
            "png" => ImageOutputFormat::Png,
            "webp" => ImageOutputFormat::Webp,
            "avif" => ImageOutputFormat::Avif,
            _ => ImageOutputFormat::Jpeg,
        };
        
        let config = ImageConfig {
            width,
            height,
            format,
            quality,
        };
        
        let processor = ImagePreprocessor::new(config);
        processor.preprocess(input_path, output_path)?;
        
        Ok(json!({
            "processed": true,
            "output_path": output_path,
            "width": width,
            "height": height,
            "format": format_str,
            "quality": quality
        }))
    }
    
    /// Handle media.capabilities operation
    fn handle_capabilities(&self) -> Result<Value, OrganError> {
        let card = self.describe();
        serde_json::to_value(&card).map_err(OrganError::SerializationError)
    }
}

impl Default for MediaOrgan {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Organ for MediaOrgan {
    async fn stimulate(&self, stimulus: Stimulus) -> Result<Response, OrganError> {
        let start = Instant::now();
        
        let result = match stimulus.op.as_str() {
            "audio.preprocess" => self.handle_audio_preprocess(stimulus.input).await?,
            "audio.mel_spectrogram" => self.handle_audio_mel_spectrogram(stimulus.input).await?,
            "video.extract_frames" => self.handle_video_extract_frames(stimulus.input).await?,
            "image.preprocess" => self.handle_image_preprocess(stimulus.input).await?,
            "media.capabilities" => self.handle_capabilities()?,
            _ => {
                return Ok(Response {
                    ok: false,
                    output: json!({
                        "error": "UnsupportedOperation",
                        "op": stimulus.op,
                        "available_operations": [
                            "audio.preprocess",
                            "audio.mel_spectrogram",
                            "video.extract_frames",
                            "image.preprocess",
                            "media.capabilities"
                        ]
                    }),
                    latency_ms: start.elapsed().as_millis() as u64,
                    cost: None,
                })
            }
        };
        
        Ok(Response {
            ok: true,
            output: result,
            latency_ms: start.elapsed().as_millis() as u64,
            cost: None,
        })
    }
    
    fn describe(&self) -> OrganCard {
        OrganCard {
            name: "soma_media".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            description: "FFmpeg-based audio/video preprocessing organ for media feature extraction and format conversion".to_string(),
            division: "media".to_string(),
            subsystem: "preprocessing".to_string(),
            tags: vec![
                "media".to_string(),
                "audio".to_string(),
                "video".to_string(),
                "ffmpeg".to_string(),
                "preprocessing".to_string(),
                "spectrogram".to_string(),
                "frames".to_string(),
            ],
            functions: vec![
                FunctionCard {
                    name: "audio.preprocess".to_string(),
                    description: "Preprocess audio file to specified format with sample rate and channel configuration via FFmpeg".to_string(),
                    tags: vec!["audio".to_string(), "preprocessing".to_string(), "conversion".to_string()],
                    examples: vec![
                        "Convert MP3 to WAV at 48kHz mono for CLAP embedding".to_string(),
                        "Normalize audio format for model input".to_string(),
                        "Resample audio to target sample rate".to_string(),
                    ],
                    idempotent: true,
                    side_effects: vec!["writes audio file".to_string(), "invokes ffmpeg".to_string()],
                    input_schema: Some(json!({
                        "type": "object",
                        "properties": {
                            "input_path": { "type": "string", "description": "Path to input audio file" },
                            "output_path": { "type": "string", "description": "Path to output audio file" },
                            "sample_rate": { "type": "integer", "description": "Target sample rate (default: 48000)" },
                            "channels": { "type": "integer", "description": "Number of channels (default: 1)" }
                        },
                        "required": ["input_path", "output_path"]
                    })),
                    output_schema: json!({
                        "type": "object",
                        "properties": {
                            "processed": { "type": "boolean" },
                            "output_path": { "type": "string" },
                            "sample_rate": { "type": "integer" },
                            "channels": { "type": "integer" }
                        }
                    }),
                },
                FunctionCard {
                    name: "audio.mel_spectrogram".to_string(),
                    description: "Generate mel-scale spectrogram from audio file for audio model input (e.g., CLAP, Whisper)".to_string(),
                    tags: vec!["audio".to_string(), "spectrogram".to_string(), "features".to_string(), "mel".to_string()],
                    examples: vec![
                        "Extract mel spectrogram for CLAP audio embedding".to_string(),
                        "Generate audio features for classification model".to_string(),
                        "Create time-frequency representation of audio".to_string(),
                    ],
                    idempotent: true,
                    side_effects: vec!["reads audio file".to_string(), "performs FFT computation".to_string()],
                    input_schema: Some(json!({
                        "type": "object",
                        "properties": {
                            "audio_path": { "type": "string", "description": "Path to audio file" },
                            "n_fft": { "type": "integer", "description": "FFT window size (default: 2048)" },
                            "hop_length": { "type": "integer", "description": "Hop length between frames (default: 512)" },
                            "n_mels": { "type": "integer", "description": "Number of mel bands (default: 128)" }
                        },
                        "required": ["audio_path"]
                    })),
                    output_schema: json!({
                        "type": "object",
                        "properties": {
                            "shape": { "type": "array", "items": { "type": "integer" } },
                            "n_mels": { "type": "integer" },
                            "time_steps": { "type": "integer" },
                            "sample_rate": { "type": "integer" }
                        }
                    }),
                },
                FunctionCard {
                    name: "video.extract_frames".to_string(),
                    description: "Extract frames from video at specified FPS and resolution for vision model input (e.g., CLIP)".to_string(),
                    tags: vec!["video".to_string(), "frames".to_string(), "extraction".to_string(), "vision".to_string()],
                    examples: vec![
                        "Extract 1 FPS frames at 336x336 for CLIP ViT-H/14".to_string(),
                        "Sample video frames for video classification".to_string(),
                        "Generate thumbnail sequence from video".to_string(),
                    ],
                    idempotent: true,
                    side_effects: vec!["writes image files".to_string(), "invokes ffmpeg".to_string()],
                    input_schema: Some(json!({
                        "type": "object",
                        "properties": {
                            "video_path": { "type": "string", "description": "Path to video file" },
                            "output_dir": { "type": "string", "description": "Directory to save extracted frames" },
                            "fps": { "type": "integer", "description": "Frames per second to extract (default: 1)" },
                            "width": { "type": "integer", "description": "Frame width (default: 336)" },
                            "height": { "type": "integer", "description": "Frame height (default: 336)" },
                            "max_frames": { "type": "integer", "description": "Maximum number of frames (optional)" }
                        },
                        "required": ["video_path", "output_dir"]
                    })),
                    output_schema: json!({
                        "type": "object",
                        "properties": {
                            "extracted": { "type": "boolean" },
                            "frame_count": { "type": "integer" },
                            "frames": { "type": "array", "items": { "type": "string" } }
                        }
                    }),
                },
                FunctionCard {
                    name: "image.preprocess".to_string(),
                    description: "Resize and convert image to specified format and dimensions for vision model input (supports JPG, PNG, WEBP, AVIF, DNG/RAW)".to_string(),
                    tags: vec!["image".to_string(), "preprocessing".to_string(), "resize".to_string(), "conversion".to_string()],
                    examples: vec![
                        "Resize image to 336x336 for CLIP vision encoder".to_string(),
                        "Convert DNG/RAW to JPEG for model input".to_string(),
                        "Batch resize images for dataset preparation".to_string(),
                    ],
                    idempotent: true,
                    side_effects: vec!["writes image file".to_string(), "invokes ffmpeg".to_string()],
                    input_schema: Some(json!({
                        "type": "object",
                        "properties": {
                            "input_path": { "type": "string", "description": "Path to input image file" },
                            "output_path": { "type": "string", "description": "Path to output image file" },
                            "width": { "type": "integer", "description": "Target width (default: 336)" },
                            "height": { "type": "integer", "description": "Target height (default: 336)" },
                            "format": { "type": "string", "enum": ["jpg", "png", "webp", "avif"], "description": "Output format (default: jpg)" },
                            "quality": { "type": "integer", "minimum": 1, "maximum": 100, "description": "Quality for lossy formats (default: 90)" }
                        },
                        "required": ["input_path", "output_path"]
                    })),
                    output_schema: json!({
                        "type": "object",
                        "properties": {
                            "processed": { "type": "boolean" },
                            "output_path": { "type": "string" },
                            "width": { "type": "integer" },
                            "height": { "type": "integer" },
                            "format": { "type": "string" },
                            "quality": { "type": "integer" }
                        }
                    }),
                },
                FunctionCard {
                    name: "media.capabilities".to_string(),
                    description: "Return organ capability card with all available functions and metadata".to_string(),
                    tags: vec!["metadata".to_string(), "discovery".to_string(), "mcp".to_string()],
                    examples: vec![
                        "Discover available media preprocessing operations".to_string(),
                        "Query organ capabilities for orchestration".to_string(),
                    ],
                    idempotent: true,
                    side_effects: vec![],
                    input_schema: None,
                    output_schema: json!({
                        "type": "object",
                        "properties": {
                            "name": { "type": "string" },
                            "version": { "type": "string" },
                            "functions": { "type": "array" }
                        }
                    }),
                },
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_organ_capabilities() {
        let organ = MediaOrgan::new();
        let stimulus = Stimulus {
            op: "media.capabilities".to_string(),
            input: json!({}),
            context: HashMap::new(),
        };
        
        let response = organ.stimulate(stimulus).await.unwrap();
        assert!(response.ok);
        assert!(response.output.get("name").is_some());
        assert_eq!(response.output["name"], "soma_media");
    }
    
    #[tokio::test]
    async fn test_unsupported_operation() {
        let organ = MediaOrgan::new();
        let stimulus = Stimulus {
            op: "invalid.operation".to_string(),
            input: json!({}),
            context: HashMap::new(),
        };
        
        let response = organ.stimulate(stimulus).await.unwrap();
        assert!(!response.ok);
        assert!(response.output.get("error").is_some());
    }
    
    #[test]
    fn test_organ_card() {
        let organ = MediaOrgan::new();
        let card = organ.describe();
        
        assert_eq!(card.name, "soma_media");
        assert_eq!(card.division, "media");
        assert_eq!(card.subsystem, "preprocessing");
        assert!(card.functions.len() >= 4);
        assert!(card.tags.contains(&"audio".to_string()));
        assert!(card.tags.contains(&"video".to_string()));
    }
}
