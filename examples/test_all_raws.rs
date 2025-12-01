//! Test demosaic on all RAW files in sample folder
//! Tests both DNG and SRW files to verify RAW processing works correctly

use std::path::{Path, PathBuf};
use soma_media::{RawProcessor, RawOptions};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Testing Demosaic on All Sample RAW Files ===\n");
    
    let processor = RawProcessor::new()?;
    
    // List of all RAW files
    let raw_files = vec![
        "sample/03240163.dng",
        "sample/03240198.dng",
        "sample/03240272.dng",
        "sample/03240300.dng",
        "sample/202310042332.SRW",
    ];
    
    // Create output directory
    std::fs::create_dir_all("test_output")?;
    
    // Test with standard settings
    let options = RawOptions {
        auto_brightness: true,
        half_size: true,  // Half-size for speed
        ..Default::default()
    };
    
    for raw_path in &raw_files {
        let path = Path::new(raw_path);
        
        if !path.exists() {
            println!("⚠️  Skipping {} (not found)", raw_path);
            continue;
        }
        
        println!("Processing: {}", raw_path);
        
        // Get file size
        let metadata = std::fs::metadata(path)?;
        println!("  File size: {:.1} MB", metadata.len() as f64 / 1_048_576.0);
        
        // Process with dimensions
        let start = std::time::Instant::now();
        match processor.process_raw_with_dims(path, &options) {
            Ok((rgb_data, width, height)) => {
                let elapsed = start.elapsed();
                
                println!("  Dimensions: {}x{}", width, height);
                println!("  RGB data: {:.1} MB", rgb_data.len() as f64 / 1_048_576.0);
                println!("  Processing time: {:.2}s", elapsed.as_secs_f64());
                
                // Calculate average brightness
                let avg: f32 = rgb_data.iter().map(|&v| v as f32).sum::<f32>() / rgb_data.len() as f32;
                println!("  Average brightness: {:.1}", avg);
                
                // Save output
                let filename = path.file_stem().unwrap().to_str().unwrap();
                let output_path = format!("test_output/{}.jpg", filename);
                
                let img = image::RgbImage::from_raw(width, height, rgb_data)
                    .expect("Failed to create image");
                img.save(&output_path)?;
                
                println!("  ✅ Saved: {}", output_path);
            }
            Err(e) => {
                println!("  ❌ Error: {:?}", e);
            }
        }
        
        println!();
    }
    
    println!("=== Test Complete ===");
    println!("Check test_output/ for results");
    
    Ok(())
}
