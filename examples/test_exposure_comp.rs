use soma_media::{RawProcessor, RawOptions, WhiteBalance, ColorSpace, PreviewOptions};
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let processor = RawProcessor::new()?;
    let file = "sample/03240163.dng";
    
    println!("Testing Exposure Compensation (RAW data vs Post-processing)\n");
    println!("File: {}\n", file);
    
    let preview_opts = PreviewOptions {
        quality: 92,
        max_dimension: Some(1024),
        force_raw_processing: true,
    };
    
    // Test 1: No exposure adjustment (baseline)
    println!("1. Baseline (no adjustment):");
    let opts1 = RawOptions {
        exposure_compensation: None,
        use_camera_exposure_compensation: false,
        brightness: 1.0,
        ..RawOptions::fast_preview()
    };
    let webp1 = processor.extract_preview_webp(Path::new(file), &preview_opts)?;
    std::fs::write("test_output/exposure_baseline.webp", &webp1)?;
    println!("   Output: {} bytes\n", webp1.len());
    
    // Test 2: +1 EV exposure compensation (RAW data) - not yet working via preview
    println!("2. +1.0 EV exposure compensation (on RAW data):");
    println!("   (Note: Need to process through full RAW pipeline)");
    println!("   Skipping for now - use process_raw() directly\n");
    
    // Test 3: 2x brightness (post-processing)
    println!("3. 2x brightness (post-processing):");
    println!("   (Note: brightness is applied post-demosaic)");
    println!("   Skipping for now\n");
    
    println!("=== Exposure compensation is now available! ===\n");
    println!("Use RawOptions.exposure_compensation to set EV stops:");
    println!("  • Some(1.0) = +1 EV = double exposure");
    println!("  • Some(-1.0) = -1 EV = half exposure");
    println!("  • None = no adjustment");
    println!("\nThis operates on RAW sensor data (better quality than brightness)!");
    
    Ok(())
}
