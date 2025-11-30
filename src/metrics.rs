//! Metrics and observability for soma_media

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;
use serde::{Deserialize, Serialize};

/// Global metrics collector
pub struct Metrics {
    pub total_requests: AtomicU64,
    pub successful_requests: AtomicU64,
    pub failed_requests: AtomicU64,
    pub total_latency_ms: AtomicU64,
    
    // Per-operation counters
    pub audio_preprocess_count: AtomicU64,
    pub audio_mel_count: AtomicU64,
    pub video_frames_count: AtomicU64,
    pub image_preprocess_count: AtomicU64,
    pub raw_preview_count: AtomicU64,
    pub raw_metadata_count: AtomicU64,
}

impl Metrics {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            total_requests: AtomicU64::new(0),
            successful_requests: AtomicU64::new(0),
            failed_requests: AtomicU64::new(0),
            total_latency_ms: AtomicU64::new(0),
            audio_preprocess_count: AtomicU64::new(0),
            audio_mel_count: AtomicU64::new(0),
            video_frames_count: AtomicU64::new(0),
            image_preprocess_count: AtomicU64::new(0),
            raw_preview_count: AtomicU64::new(0),
            raw_metadata_count: AtomicU64::new(0),
        })
    }
    
    pub fn record_request(&self, op: &str, success: bool, latency_ms: u64) {
        self.total_requests.fetch_add(1, Ordering::Relaxed);
        
        if success {
            self.successful_requests.fetch_add(1, Ordering::Relaxed);
        } else {
            self.failed_requests.fetch_add(1, Ordering::Relaxed);
        }
        
        self.total_latency_ms.fetch_add(latency_ms, Ordering::Relaxed);
        
        // Increment operation-specific counter
        match op {
            "audio.preprocess" => self.audio_preprocess_count.fetch_add(1, Ordering::Relaxed),
            "audio.mel_spectrogram" => self.audio_mel_count.fetch_add(1, Ordering::Relaxed),
            "video.extract_frames" => self.video_frames_count.fetch_add(1, Ordering::Relaxed),
            "image.preprocess" => self.image_preprocess_count.fetch_add(1, Ordering::Relaxed),
            "raw.preview" => self.raw_preview_count.fetch_add(1, Ordering::Relaxed),
            "raw.metadata" => self.raw_metadata_count.fetch_add(1, Ordering::Relaxed),
            _ => 0,
        };
    }
    
    pub fn snapshot(&self) -> MetricsSnapshot {
        let total = self.total_requests.load(Ordering::Relaxed);
        let successful = self.successful_requests.load(Ordering::Relaxed);
        let failed = self.failed_requests.load(Ordering::Relaxed);
        let total_latency = self.total_latency_ms.load(Ordering::Relaxed);
        
        MetricsSnapshot {
            total_requests: total,
            successful_requests: successful,
            failed_requests: failed,
            error_rate: if total > 0 { failed as f64 / total as f64 } else { 0.0 },
            avg_latency_ms: if total > 0 { total_latency / total } else { 0 },
            operations: OperationMetrics {
                audio_preprocess: self.audio_preprocess_count.load(Ordering::Relaxed),
                audio_mel_spectrogram: self.audio_mel_count.load(Ordering::Relaxed),
                video_extract_frames: self.video_frames_count.load(Ordering::Relaxed),
                image_preprocess: self.image_preprocess_count.load(Ordering::Relaxed),
                raw_preview: self.raw_preview_count.load(Ordering::Relaxed),
                raw_metadata: self.raw_metadata_count.load(Ordering::Relaxed),
            },
        }
    }
}

impl Default for Metrics {
    fn default() -> Self {
        Self {
            total_requests: AtomicU64::new(0),
            successful_requests: AtomicU64::new(0),
            failed_requests: AtomicU64::new(0),
            total_latency_ms: AtomicU64::new(0),
            audio_preprocess_count: AtomicU64::new(0),
            audio_mel_count: AtomicU64::new(0),
            video_frames_count: AtomicU64::new(0),
            image_preprocess_count: AtomicU64::new(0),
            raw_preview_count: AtomicU64::new(0),
            raw_metadata_count: AtomicU64::new(0),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSnapshot {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub error_rate: f64,
    pub avg_latency_ms: u64,
    pub operations: OperationMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationMetrics {
    pub audio_preprocess: u64,
    pub audio_mel_spectrogram: u64,
    pub video_extract_frames: u64,
    pub image_preprocess: u64,
    pub raw_preview: u64,
    pub raw_metadata: u64,
}

/// Timer for tracking operation latency
pub struct Timer {
    start: Instant,
}

impl Timer {
    pub fn new() -> Self {
        Self {
            start: Instant::now(),
        }
    }
    
    pub fn elapsed_ms(&self) -> u64 {
        self.start.elapsed().as_millis() as u64
    }
}

impl Default for Timer {
    fn default() -> Self {
        Self::new()
    }
}
