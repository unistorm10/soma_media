//! GPU Acceleration Module
//!
//! Provides GPU-accelerated post-processing for RAW images.
//!
//! ## Architecture: UMA Integration with soma_compute
//!
//! This module communicates with `soma_compute` via UMA (Universal Module Architecture)
//! for unified GPU device management. This avoids device conflicts when multiple
//! SOMA modules need GPU access.
//!
//! ## Processing Cascade
//!
//! 1. **soma_compute GPU** (via UMA) - Preferred, coordinated GPU access
//! 2. **CPU SIMD** (rayon) - Automatic fallback when soma_compute unavailable
//!
//! ## Post-Processing Operations
//!
//! All operations try GPU first via soma_compute, fall back to CPU automatically:
//!
//! - Resize/downscale (high-quality Lanczos)
//! - Gamma correction (LUT-based, parallel)
//! - Chromatic aberration correction (channel warping)
//! - Median filtering (artifact reduction)
//! - Noise reduction (wavelet)
//!
//! ## Sync vs Async
//!
//! - **Sync methods** (e.g., `resize`, `apply_gamma`): Use CPU implementation
//! - **Async methods** (e.g., `resize_async`, `apply_gamma_async`): Try GPU via UMA first
//!
//! ## Example
//!
//! ```rust,no_run
//! use soma_media::GpuProcessor;
//!
//! // Auto-detect: tries soma_compute GPU, falls back to CPU
//! let gpu = GpuProcessor::new();
//! println!("Using: {}", gpu.backend_info());
//!
//! // Sync (CPU): Always works
//! let resized = gpu.resize(&rgb_data, 6000, 4000, 2048, 2048)?;
//!
//! // Async (GPU if available): Use in async context
//! // let resized = gpu.resize_async(&rgb_data, 6000, 4000, 2048, 2048).await?;
//! ```

use crate::error::{MediaError, Result};
use crate::compute_client::ComputeClient;
use tracing::{info, debug};

/// GPU backend type
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GpuBackend {
    /// soma_compute managed GPU (via UMA - coordinated with ML inference)
    SomaCompute,
    /// CPU fallback with SIMD (always available)
    Cpu,
}

/// GPU processor with automatic backend selection
/// 
/// Communicates with soma_compute via UMA for coordinated GPU access.
/// Falls back to CPU when soma_compute is unavailable.
pub struct GpuProcessor {
    backend: GpuBackend,
}

impl GpuProcessor {
    /// Create new GPU processor
    /// 
    /// Starts with CPU backend. Async methods will try soma_compute GPU first.
    pub fn new() -> Self {
        info!("🔧 GpuProcessor initialized (GPU via soma_compute UMA, CPU fallback)");
        Self { 
            backend: GpuBackend::Cpu,
        }
    }
    
    /// Get backend info for debugging
    pub fn backend_info(&self) -> &'static str {
        match self.backend {
            GpuBackend::SomaCompute => "soma_compute GPU (via UMA)",
            GpuBackend::Cpu => "CPU (SIMD via rayon)",
        }
    }
    
    /// Check if using GPU (not CPU fallback)
    pub fn has_gpu(&self) -> bool {
        self.backend == GpuBackend::SomaCompute
    }
    
    // ========================================================================
    // SYNC METHODS (CPU implementation - always available)
    // ========================================================================
    
    /// Resize image using CPU (sync)
    /// 
    /// For GPU acceleration, use `resize_async` in an async context.
    pub fn resize(
        &self,
        rgb_data: &[u8],
        src_width: u32,
        src_height: u32,
        dst_width: u32,
        dst_height: u32,
    ) -> Result<Vec<u8>> {
        self.resize_cpu(rgb_data, src_width, src_height, dst_width, dst_height)
    }
    
    /// Apply gamma correction using CPU (sync)
    pub fn apply_gamma(
        &self,
        rgb_data: &[u8],
        _width: u32,
        _height: u32,
        power: f32,
        slope: f32,
    ) -> Result<Vec<u8>> {
        self.apply_gamma_cpu(rgb_data, power, slope)
    }
    
    /// Correct chromatic aberration using CPU (sync)
    pub fn correct_chromatic_aberration(
        &self,
        rgb_data: &[u8],
        width: u32,
        height: u32,
        red_scale: f64,
        blue_scale: f64,
    ) -> Result<Vec<u8>> {
        self.chromatic_aberration_cpu(rgb_data, width, height, red_scale, blue_scale)
    }
    
    /// Apply median filter using CPU (sync)
    pub fn median_filter(
        &self,
        rgb_data: &[u8],
        width: u32,
        height: u32,
        passes: u8,
    ) -> Result<Vec<u8>> {
        let mut data = rgb_data.to_vec();
        for _ in 0..passes {
            data = self.median_filter_pass(&data, width, height)?;
        }
        Ok(data)
    }
    
    /// Denoise using CPU (sync) - placeholder
    pub fn denoise_wavelet(
        &self,
        rgb_data: &[u8],
        _width: u32,
        _height: u32,
        _threshold: f32,
    ) -> Result<Vec<u8>> {
        // Wavelet denoising is complex - placeholder for now
        debug!("Wavelet denoising not yet implemented, passing through");
        Ok(rgb_data.to_vec())
    }
    
    // ========================================================================
    // ASYNC METHODS (try GPU via soma_compute, fallback to CPU)
    // ========================================================================
    
    /// Resize image - tries GPU via soma_compute, falls back to CPU
    pub async fn resize_async(
        &self,
        rgb_data: &[u8],
        src_width: u32,
        src_height: u32,
        dst_width: u32,
        dst_height: u32,
    ) -> Result<Vec<u8>> {
        let mut client = ComputeClient::new();
        
        // Try GPU first
        match client.resize(rgb_data, src_width, src_height, dst_width, dst_height).await {
            Ok(result) => {
                debug!("GPU resize completed via soma_compute");
                Ok(result)
            }
            Err(_) => {
                debug!("GPU unavailable, using CPU resize");
                self.resize_cpu(rgb_data, src_width, src_height, dst_width, dst_height)
            }
        }
    }
    
    /// Apply gamma correction - tries GPU via soma_compute, falls back to CPU
    pub async fn apply_gamma_async(
        &self,
        rgb_data: &[u8],
        width: u32,
        height: u32,
        power: f32,
        slope: f32,
    ) -> Result<Vec<u8>> {
        let mut client = ComputeClient::new();
        
        match client.apply_gamma(rgb_data, width, height, power, slope).await {
            Ok(result) => {
                debug!("GPU gamma correction completed via soma_compute");
                Ok(result)
            }
            Err(_) => {
                debug!("GPU unavailable, using CPU gamma correction");
                self.apply_gamma_cpu(rgb_data, power, slope)
            }
        }
    }
    
    /// Correct chromatic aberration - tries GPU via soma_compute, falls back to CPU
    pub async fn chromatic_aberration_async(
        &self,
        rgb_data: &[u8],
        width: u32,
        height: u32,
        red_scale: f64,
        blue_scale: f64,
    ) -> Result<Vec<u8>> {
        let mut client = ComputeClient::new();
        
        match client.correct_chromatic_aberration(rgb_data, width, height, red_scale, blue_scale).await {
            Ok(result) => {
                debug!("GPU chromatic aberration correction completed via soma_compute");
                Ok(result)
            }
            Err(_) => {
                debug!("GPU unavailable, using CPU chromatic aberration correction");
                self.chromatic_aberration_cpu(rgb_data, width, height, red_scale, blue_scale)
            }
        }
    }
    
    /// Apply median filter - tries GPU via soma_compute, falls back to CPU
    pub async fn median_filter_async(
        &self,
        rgb_data: &[u8],
        width: u32,
        height: u32,
        passes: u8,
    ) -> Result<Vec<u8>> {
        let mut client = ComputeClient::new();
        
        match client.median_filter(rgb_data, width, height, passes).await {
            Ok(result) => {
                debug!("GPU median filter completed via soma_compute");
                Ok(result)
            }
            Err(_) => {
                debug!("GPU unavailable, using CPU median filter");
                self.median_filter(rgb_data, width, height, passes)
            }
        }
    }
    
    /// Denoise - tries GPU via soma_compute, falls back to CPU
    pub async fn denoise_async(
        &self,
        rgb_data: &[u8],
        width: u32,
        height: u32,
        threshold: f32,
    ) -> Result<Vec<u8>> {
        let mut client = ComputeClient::new();
        
        match client.denoise(rgb_data, width, height, threshold).await {
            Ok(result) => {
                debug!("GPU denoise completed via soma_compute");
                Ok(result)
            }
            Err(_) => {
                debug!("GPU unavailable, using CPU denoise");
                self.denoise_wavelet(rgb_data, width, height, threshold)
            }
        }
    }
    
    /// Batch resize multiple images - uses parallelism
    pub fn batch_resize(
        &self,
        images: Vec<(Vec<u8>, u32, u32)>,
        target_width: u32,
        target_height: u32,
    ) -> Result<Vec<Vec<u8>>> {
        use rayon::prelude::*;
        
        images.par_iter()
            .map(|(data, w, h)| {
                self.resize(data, *w, *h, target_width, target_height)
            })
            .collect()
    }
    
    // ========================================================================
    // CPU IMPLEMENTATIONS (SIMD-optimized via rayon)
    // ========================================================================
    
    /// CPU resize using fast_image_resize (SIMD-optimized)
    fn resize_cpu(
        &self,
        rgb_data: &[u8],
        src_w: u32,
        src_h: u32,
        dst_w: u32,
        dst_h: u32,
    ) -> Result<Vec<u8>> {
        use fast_image_resize as fr;
        use fr::images::Image as FrImage;
        
        let src_image = FrImage::from_vec_u8(
            src_w,
            src_h,
            rgb_data.to_vec(),
            fr::PixelType::U8x3,
        ).map_err(|e| MediaError::ProcessingError(format!("Failed to create source image: {:?}", e)))?;
        
        let mut dst_image = FrImage::new(
            dst_w,
            dst_h,
            src_image.pixel_type(),
        );
        
        let mut resizer = fr::Resizer::new();
        resizer.resize(&src_image, &mut dst_image, None)
            .map_err(|e| MediaError::ProcessingError(format!("Resize failed: {:?}", e)))?;
        
        Ok(dst_image.buffer().to_vec())
    }
    
    /// CPU gamma correction using LUT (parallel)
    fn apply_gamma_cpu(&self, rgb_data: &[u8], power: f32, slope: f32) -> Result<Vec<u8>> {
        use rayon::prelude::*;
        
        // Build LUT for speed
        let mut lut = vec![0u8; 256];
        for i in 0..256 {
            let normalized = i as f32 / 255.0;
            let gamma_corrected = if normalized < slope {
                normalized / slope
            } else {
                ((normalized + (power - 1.0) * slope) / power).powf(power)
            };
            lut[i] = (gamma_corrected * 255.0).clamp(0.0, 255.0) as u8;
        }
        
        // Apply LUT in parallel
        Ok(rgb_data.par_iter().map(|&v| lut[v as usize]).collect())
    }
    
    /// CPU chromatic aberration correction (parallelized)
    fn chromatic_aberration_cpu(
        &self,
        rgb_data: &[u8],
        width: u32,
        height: u32,
        red_scale: f64,
        blue_scale: f64,
    ) -> Result<Vec<u8>> {
        use rayon::prelude::*;
        
        let w = width as usize;
        let h = height as usize;
        let center_x = w as f64 / 2.0;
        let center_y = h as f64 / 2.0;
        
        // Process rows in parallel
        let output: Vec<u8> = (0..h)
            .into_par_iter()
            .flat_map(|y| {
                let mut row = vec![0u8; w * 3];
                for x in 0..w {
                    let dx = x as f64 - center_x;
                    let dy = y as f64 - center_y;
                    let out_idx = (y * w + x) * 3;
                    
                    // Red channel
                    let rx = center_x + dx * red_scale;
                    let ry = center_y + dy * red_scale;
                    if rx >= 0.0 && rx < w as f64 && ry >= 0.0 && ry < h as f64 {
                        let src_idx = ((ry as usize) * w + (rx as usize)) * 3;
                        row[x * 3] = rgb_data[src_idx];
                    }
                    
                    // Green channel (no correction)
                    row[x * 3 + 1] = rgb_data[out_idx + 1];
                    
                    // Blue channel
                    let bx = center_x + dx * blue_scale;
                    let by = center_y + dy * blue_scale;
                    if bx >= 0.0 && bx < w as f64 && by >= 0.0 && by < h as f64 {
                        let src_idx = ((by as usize) * w + (bx as usize)) * 3;
                        row[x * 3 + 2] = rgb_data[src_idx + 2];
                    }
                }
                row
            })
            .collect();
        
        Ok(output)
    }
    
    /// CPU median filter pass (3x3 kernel)
    fn median_filter_pass(&self, rgb_data: &[u8], width: u32, height: u32) -> Result<Vec<u8>> {
        use rayon::prelude::*;
        
        let w = width as usize;
        let h = height as usize;
        let mut output = vec![0u8; rgb_data.len()];
        
        // Process rows in parallel
        output.par_chunks_mut(w * 3).enumerate().for_each(|(y, row)| {
            for x in 0..w {
                for c in 0..3 {
                    let mut values = [0u8; 9];
                    let mut count = 0;
                    
                    for dy in -1i32..=1 {
                        for dx in -1i32..=1 {
                            let nx = (x as i32 + dx).max(0).min((w - 1) as i32) as usize;
                            let ny = (y as i32 + dy).max(0).min((h - 1) as i32) as usize;
                            values[count] = rgb_data[(ny * w + nx) * 3 + c];
                            count += 1;
                        }
                    }
                    
                    // Sort and take median
                    values[..count].sort_unstable();
                    row[x * 3 + c] = values[count / 2];
                }
            }
        });
        
        Ok(output)
    }
}

impl Default for GpuProcessor {
    fn default() -> Self {
        Self::new()
    }
}
