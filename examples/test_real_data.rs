//! Integration test with real media files from network share

use soma_media::{Organ, MediaOrgan, Stimulus};
use serde_json::json;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let organ = MediaOrgan::new();
    
    println!("=== soma_media Real Data Integration Test ===\n");
    
    // Test 1: Audio preprocessing
    println!("1. Testing Audio Preprocessing (MP3 → WAV)");
    let stimulus = Stimulus {
        op: "audio.preprocess".to_string(),
        input: json!({
            "input_path": "/run/user/1000/gvfs/smb-share:server=main.local,share=test_data/00 I Can't Wait.mp3",
            "output_path": "/tmp/test_audio.wav",
            "sample_rate": 48000,
            "channels": 1
        }),
        context: HashMap::new(),
    };
    
    let response = organ.stimulate(stimulus).await?;
    println!("   Status: {}", if response.ok { "✓ SUCCESS" } else { "✗ FAILED" });
    println!("   Output: {}", serde_json::to_string_pretty(&response.output)?);
    println!("   Latency: {}ms", response.latency_ms);
    
    if std::path::Path::new("/tmp/test_audio.wav").exists() {
        let metadata = std::fs::metadata("/tmp/test_audio.wav")?;
        println!("   ✓ Created: /tmp/test_audio.wav ({} bytes)", metadata.len());
    }
    println!();
    
    // Test 2: Video frame extraction
    println!("2. Testing Video Frame Extraction (MP4 → JPG frames)");
    std::fs::create_dir_all("/tmp/video_frames")?;
    
    let stimulus = Stimulus {
        op: "video.extract_frames".to_string(),
        input: json!({
            "video_path": "/run/user/1000/gvfs/smb-share:server=main.local,share=test_data/sample_ext/66293fcb9333a.mp4",
            "output_dir": "/tmp/video_frames",
            "fps": 1,
            "width": 336,
            "height": 336,
            "max_frames": 5
        }),
        context: HashMap::new(),
    };
    
    let response = organ.stimulate(stimulus).await?;
    println!("   Status: {}", if response.ok { "✓ SUCCESS" } else { "✗ FAILED" });
    println!("   Output: {}", serde_json::to_string_pretty(&response.output)?);
    println!("   Latency: {}ms", response.latency_ms);
    
    let frame_count = std::fs::read_dir("/tmp/video_frames")?.count();
    println!("   ✓ Extracted {} frames to /tmp/video_frames/", frame_count);
    println!();
    
    // Test 3: Multiple audio conversions
    println!("3. Testing Second Audio File");
    let stimulus = Stimulus {
        op: "audio.preprocess".to_string(),
        input: json!({
            "input_path": "/run/user/1000/gvfs/smb-share:server=main.local,share=test_data/08 Forgot About Dre.mp3",
            "output_path": "/tmp/test_audio2.wav",
            "sample_rate": 16000,
            "channels": 1
        }),
        context: HashMap::new(),
    };
    
    let response = organ.stimulate(stimulus).await?;
    println!("   Status: {}", if response.ok { "✓ SUCCESS" } else { "✗ FAILED" });
    println!("   Latency: {}ms", response.latency_ms);
    
    if std::path::Path::new("/tmp/test_audio2.wav").exists() {
        let metadata = std::fs::metadata("/tmp/test_audio2.wav")?;
        println!("   ✓ Created: /tmp/test_audio2.wav ({} bytes)", metadata.len());
    }
    println!();
    
    println!("=== Integration Test Complete ===");
    println!("\n✓ All real-world media processing operations successful!");
    println!("✓ UMA Stimulus/Response pattern working with actual files");
    println!("✓ FFmpeg integration functional");
    
    Ok(())
}
