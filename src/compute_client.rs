//! UMA Client for GPU Operations
//!
//! Provides async interface to soma_compute for GPU-accelerated image processing.
//! Falls back to CPU when soma_compute is unavailable.

use crate::error::{MediaError, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;
use tracing::{debug, info, warn};

/// Resize algorithm for GPU processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResizeAlgorithm {
    Nearest,
    Bilinear,
    Lanczos3,
}

/// UMA request to soma_compute for image processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComputeRequest {
    /// Image resize (GPU-accelerated)
    ImageResize {
        data: Vec<u8>,
        src_width: u32,
        src_height: u32,
        dst_width: u32,
        dst_height: u32,
        algorithm: ResizeAlgorithm,
    },
    /// Gamma correction (GPU-accelerated)
    ImageGamma {
        data: Vec<u8>,
        width: u32,
        height: u32,
        power: f32,
        slope: f32,
    },
    /// Chromatic aberration correction (GPU-accelerated)
    ChromaticAberration {
        data: Vec<u8>,
        width: u32,
        height: u32,
        red_scale: f64,
        blue_scale: f64,
    },
    /// Median filter (GPU-accelerated)
    MedianFilter {
        data: Vec<u8>,
        width: u32,
        height: u32,
        passes: u8,
    },
    /// Wavelet denoise (GPU-accelerated)
    Denoise {
        data: Vec<u8>,
        width: u32,
        height: u32,
        threshold: f32,
    },
    /// Check if soma_compute GPU is available
    Ping,
}

/// UMA response from soma_compute
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComputeResponse {
    /// Image processing result
    ImageResult {
        data: Vec<u8>,
        width: u32,
        height: u32,
        compute_time_ms: f64,
    },
    /// Pong response (soma_compute is available)
    Pong {
        backend: String,
        device_name: String,
    },
    /// Error
    Error {
        message: String,
    },
}

/// Client for GPU operations via UMA to soma_compute
pub struct ComputeClient {
    /// IPC socket path for soma_compute
    #[allow(dead_code)]
    socket_path: String,
    /// Timeout for requests
    #[allow(dead_code)]
    timeout: Duration,
    /// Whether soma_compute is available
    available: Option<bool>,
}

impl ComputeClient {
    /// Create new compute client with default socket path
    pub fn new() -> Self {
        Self {
            socket_path: "/tmp/soma_compute.sock".to_string(),
            timeout: Duration::from_millis(5000),
            available: None,
        }
    }
    
    /// Create with custom socket path
    pub fn with_socket(socket_path: &str) -> Self {
        Self {
            socket_path: socket_path.to_string(),
            timeout: Duration::from_millis(5000),
            available: None,
        }
    }
    
    /// Check if soma_compute GPU is available (with caching)
    pub async fn is_available(&mut self) -> bool {
        if let Some(available) = self.available {
            return available;
        }
        
        // Check if socket exists first (fast path)
        if !Path::new(&self.socket_path).exists() {
            debug!("soma_compute socket not found: {}", self.socket_path);
            self.available = Some(false);
            return false;
        }
        
        // Try to ping soma_compute
        match self.send_request(ComputeRequest::Ping).await {
            Ok(ComputeResponse::Pong { backend, device_name }) => {
                info!("🚀 soma_compute GPU available: {} ({})", device_name, backend);
                self.available = Some(true);
                true
            }
            Err(e) => {
                warn!("soma_compute not available: {}", e);
                self.available = Some(false);
                false
            }
            _ => {
                debug!("Unexpected ping response");
                self.available = Some(false);
                false
            }
        }
    }
    
    /// Send request to soma_compute via Unix Domain Socket
    async fn send_request(&self, request: ComputeRequest) -> Result<ComputeResponse> {
        // Check if socket exists
        if !Path::new(&self.socket_path).exists() {
            return Err(MediaError::ProcessingError(
                format!("soma_compute socket not found: {}", self.socket_path)
            ));
        }
        
        // Connect to soma_compute UDS
        let mut stream = tokio::time::timeout(
            self.timeout,
            UnixStream::connect(&self.socket_path)
        ).await
            .map_err(|_| MediaError::ProcessingError("Connection timeout".into()))?
            .map_err(|e| MediaError::ProcessingError(format!("Connect failed: {}", e)))?;
        
        // Serialize request as Organ Stimulus (matches soma_compute protocol)
        let stimulus = serde_json::json!({
            "op": match &request {
                ComputeRequest::Ping => "health",
                ComputeRequest::ImageResize { .. } => "image_resize",
                ComputeRequest::ImageGamma { .. } => "image_gamma",
                ComputeRequest::ChromaticAberration { .. } => "chromatic_aberration",
                ComputeRequest::MedianFilter { .. } => "median_filter",
                ComputeRequest::Denoise { .. } => "denoise",
            },
            "input": serde_json::to_value(&request).unwrap_or_default(),
        });
        
        let request_bytes = serde_json::to_vec(&stimulus)
            .map_err(|e| MediaError::ProcessingError(format!("Serialize failed: {}", e)))?;
        
        // Write length prefix (4 bytes big-endian) + body
        let len_bytes = (request_bytes.len() as u32).to_be_bytes();
        stream.write_all(&len_bytes).await
            .map_err(|e| MediaError::ProcessingError(format!("Write failed: {}", e)))?;
        stream.write_all(&request_bytes).await
            .map_err(|e| MediaError::ProcessingError(format!("Write failed: {}", e)))?;
        stream.flush().await
            .map_err(|e| MediaError::ProcessingError(format!("Flush failed: {}", e)))?;
        
        // Read response length
        let mut len_buf = [0u8; 4];
        tokio::time::timeout(self.timeout, stream.read_exact(&mut len_buf)).await
            .map_err(|_| MediaError::ProcessingError("Read timeout".into()))?
            .map_err(|e| MediaError::ProcessingError(format!("Read failed: {}", e)))?;
        
        let response_len = u32::from_be_bytes(len_buf) as usize;
        
        // Read response body
        let mut response_buf = vec![0u8; response_len];
        tokio::time::timeout(self.timeout, stream.read_exact(&mut response_buf)).await
            .map_err(|_| MediaError::ProcessingError("Read timeout".into()))?
            .map_err(|e| MediaError::ProcessingError(format!("Read failed: {}", e)))?;
        
        // Parse Organ Response
        #[derive(Deserialize)]
        struct OrganResponse {
            ok: bool,
            output: serde_json::Value,
            #[allow(dead_code)]
            latency_ms: u64,
        }
        
        let response: OrganResponse = serde_json::from_slice(&response_buf)
            .map_err(|e| MediaError::ProcessingError(format!("Parse failed: {}", e)))?;
        
        if !response.ok {
            let error_msg = response.output.get("error")
                .and_then(|e| e.as_str())
                .unwrap_or("Unknown error");
            return Err(MediaError::ProcessingError(error_msg.to_string()));
        }
        
        // Convert to ComputeResponse based on request type
        match request {
            ComputeRequest::Ping => {
                let backend = response.output.get("backend")
                    .and_then(|b| b.as_str())
                    .unwrap_or("unknown")
                    .to_string();
                let device_name = response.output.get("device")
                    .and_then(|d| d.as_str())
                    .unwrap_or("unknown")
                    .to_string();
                Ok(ComputeResponse::Pong { backend, device_name })
            }
            _ => {
                // Image processing responses
                if let Some(data) = response.output.get("data") {
                    let data: Vec<u8> = serde_json::from_value(data.clone())
                        .unwrap_or_default();
                    let width = response.output.get("width")
                        .and_then(|w| w.as_u64())
                        .unwrap_or(0) as u32;
                    let height = response.output.get("height")
                        .and_then(|h| h.as_u64())
                        .unwrap_or(0) as u32;
                    let compute_time_ms = response.output.get("compute_time_ms")
                        .and_then(|t| t.as_f64())
                        .unwrap_or(0.0);
                    
                    Ok(ComputeResponse::ImageResult {
                        data,
                        width,
                        height,
                        compute_time_ms,
                    })
                } else {
                    Err(MediaError::ProcessingError("No data in response".into()))
                }
            }
        }
    }
    
    /// Resize image via soma_compute GPU
    pub async fn resize(
        &mut self,
        data: &[u8],
        src_width: u32,
        src_height: u32,
        dst_width: u32,
        dst_height: u32,
    ) -> Result<Vec<u8>> {
        if !self.is_available().await {
            return Err(MediaError::ProcessingError("GPU not available".to_string()));
        }
        
        let request = ComputeRequest::ImageResize {
            data: data.to_vec(),
            src_width,
            src_height,
            dst_width,
            dst_height,
            algorithm: ResizeAlgorithm::Lanczos3,
        };
        
        match self.send_request(request).await? {
            ComputeResponse::ImageResult { data, .. } => Ok(data),
            ComputeResponse::Error { message } => {
                Err(MediaError::ProcessingError(format!("GPU resize failed: {}", message)))
            }
            _ => Err(MediaError::ProcessingError("Unexpected response".to_string())),
        }
    }
    
    /// Apply gamma correction via soma_compute GPU
    pub async fn apply_gamma(
        &mut self,
        data: &[u8],
        width: u32,
        height: u32,
        power: f32,
        slope: f32,
    ) -> Result<Vec<u8>> {
        if !self.is_available().await {
            return Err(MediaError::ProcessingError("GPU not available".to_string()));
        }
        
        let request = ComputeRequest::ImageGamma {
            data: data.to_vec(),
            width,
            height,
            power,
            slope,
        };
        
        match self.send_request(request).await? {
            ComputeResponse::ImageResult { data, .. } => Ok(data),
            ComputeResponse::Error { message } => {
                Err(MediaError::ProcessingError(format!("GPU gamma failed: {}", message)))
            }
            _ => Err(MediaError::ProcessingError("Unexpected response".to_string())),
        }
    }
    
    /// Correct chromatic aberration via soma_compute GPU
    pub async fn correct_chromatic_aberration(
        &mut self,
        data: &[u8],
        width: u32,
        height: u32,
        red_scale: f64,
        blue_scale: f64,
    ) -> Result<Vec<u8>> {
        if !self.is_available().await {
            return Err(MediaError::ProcessingError("GPU not available".to_string()));
        }
        
        let request = ComputeRequest::ChromaticAberration {
            data: data.to_vec(),
            width,
            height,
            red_scale,
            blue_scale,
        };
        
        match self.send_request(request).await? {
            ComputeResponse::ImageResult { data, .. } => Ok(data),
            ComputeResponse::Error { message } => {
                Err(MediaError::ProcessingError(format!("GPU CA correction failed: {}", message)))
            }
            _ => Err(MediaError::ProcessingError("Unexpected response".to_string())),
        }
    }
    
    /// Apply median filter via soma_compute GPU
    pub async fn median_filter(
        &mut self,
        data: &[u8],
        width: u32,
        height: u32,
        passes: u8,
    ) -> Result<Vec<u8>> {
        if !self.is_available().await {
            return Err(MediaError::ProcessingError("GPU not available".to_string()));
        }
        
        let request = ComputeRequest::MedianFilter {
            data: data.to_vec(),
            width,
            height,
            passes,
        };
        
        match self.send_request(request).await? {
            ComputeResponse::ImageResult { data, .. } => Ok(data),
            ComputeResponse::Error { message } => {
                Err(MediaError::ProcessingError(format!("GPU median filter failed: {}", message)))
            }
            _ => Err(MediaError::ProcessingError("Unexpected response".to_string())),
        }
    }
    
    /// Apply wavelet denoising via soma_compute GPU
    pub async fn denoise(
        &mut self,
        data: &[u8],
        width: u32,
        height: u32,
        threshold: f32,
    ) -> Result<Vec<u8>> {
        if !self.is_available().await {
            return Err(MediaError::ProcessingError("GPU not available".to_string()));
        }
        
        let request = ComputeRequest::Denoise {
            data: data.to_vec(),
            width,
            height,
            threshold,
        };
        
        match self.send_request(request).await? {
            ComputeResponse::ImageResult { data, .. } => Ok(data),
            ComputeResponse::Error { message } => {
                Err(MediaError::ProcessingError(format!("GPU denoise failed: {}", message)))
            }
            _ => Err(MediaError::ProcessingError("Unexpected response".to_string())),
        }
    }
}

impl Default for ComputeClient {
    fn default() -> Self {
        Self::new()
    }
}
