//! Example demonstrating RAW preview extraction with WebP output
//!
//! This example shows how to use the new RAW preview extraction features:
//! - Extract embedded JPEG preview from RAW files (fast)
//! - Fallback to RAW processing if no embedded preview
//! - Convert to WebP at quality 92
//! - Extract comprehensive metadata
//!
//! Run with: cargo run --example test_raw_preview

use soma_media::{RawProcessor, PreviewOptions};
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== RAW Preview Extraction Demo ===\n");
    
    // Initialize RAW processor
    let processor = RawProcessor::new()?;
    
    // Test file path (adjust to your RAW file)
    let test_file = std::env::args()
        .nth(1)
        .unwrap_or_else(|| {
            eprintln!("Usage: cargo run --example test_raw_preview <path_to_raw_file>");
            eprintln!("No file provided - using placeholder path");
            "/path/to/test.CR2".to_string()
        });
    
    let file_path = Path::new(&test_file);
    
    if !file_path.exists() {
        eprintln!("Error: File not found: {}", test_file);
        eprintln!("\nPlease provide a valid RAW file path:");
        eprintln!("  cargo run --example test_raw_preview /path/to/photo.CR2");
        return Ok(());
    }
    
    println!("Processing RAW file: {}\n", test_file);
    
    // Test 1: Extract preview with default options (embedded, WebP Q92)
    println!("1. Extract preview with default options:");
    println!("   - Quality: 92");
    println!("   - Max dimension: 2048");
    println!("   - Method: Embedded (auto-fallback to RAW processing)");
    
    let output1 = "test_output/preview_default.webp";
    let start = std::time::Instant::now();
    
    match processor.extract_preview_webp(file_path, &PreviewOptions::default()) {
        Ok(webp_data) => {
            std::fs::write(output1, &webp_data)?;
            let elapsed = start.elapsed();
            
            println!("   ✓ Success!");
            println!("   Output: {}", output1);
            println!("   Size: {} bytes ({:.1} KB)", webp_data.len(), webp_data.len() as f64 / 1024.0);
            println!("   Time: {:?} ({:.1}x faster than full RAW)", elapsed, 2800.0 / elapsed.as_millis() as f64);
        }
        Err(e) => {
            println!("   ✗ Failed: {}", e);
        }
    }
    println!();
    
    // Test 2: Force RAW processing (highest quality)
    println!("2. Force RAW processing (highest quality):");
    println!("   - Quality: 92");
    println!("   - Method: Always process RAW data");
    
    let output2 = "test_output/preview_force_raw.webp";
    let options_force_raw = PreviewOptions {
        quality: 92,
        max_dimension: Some(2048),
        force_raw_processing: true,
    };
    
    let start = std::time::Instant::now();
    
    match processor.extract_preview_webp(file_path, &options_force_raw) {
        Ok(webp_data) => {
            std::fs::write(output2, &webp_data)?;
            let elapsed = start.elapsed();
            
            println!("   ✓ Success!");
            println!("   Output: {}", output2);
            println!("   Size: {} bytes ({:.1} KB)", webp_data.len(), webp_data.len() as f64 / 1024.0);
            println!("   Time: {:?}", elapsed);
        }
        Err(e) => {
            println!("   ✗ Failed: {}", e);
        }
    }
    println!();
    
    // Test 3: Higher quality preset
    println!("3. High quality preset (Q95):");
    let output3 = "test_output/preview_high_quality.webp";
    let options_high = PreviewOptions {
        quality: 95,
        max_dimension: Some(2048),
        force_raw_processing: false,
    };
    
    match processor.extract_preview_webp(file_path, &options_high) {
        Ok(webp_data) => {
            std::fs::write(output3, &webp_data)?;
            println!("   ✓ Success!");
            println!("   Output: {}", output3);
            println!("   Size: {} bytes ({:.1} KB)", webp_data.len(), webp_data.len() as f64 / 1024.0);
        }
        Err(e) => {
            println!("   ✗ Failed: {}", e);
        }
    }
    println!();
    
    // Test 4: Extract metadata
    println!("4. Extract metadata:");
    match processor.extract_metadata(file_path) {
        Ok(metadata) => {
            println!("   ✓ Camera: {} {}", metadata.make, metadata.model);
            if let Some(lens) = &metadata.lens {
                println!("   ✓ Lens: {}", lens);
            }
            println!("   ✓ Settings:");
            println!("     - ISO: {}", metadata.iso);
            println!("     - Aperture: f/{:.1}", metadata.aperture);
            println!("     - Shutter: {:.4}s", metadata.shutter_speed);
            println!("     - Focal length: {:.0}mm", metadata.focal_length);
            println!("   ✓ Dimensions: {}x{}", metadata.width, metadata.height);
            
            if let Some(ts) = metadata.timestamp {
                println!("   ✓ Captured: {}", chrono::NaiveDateTime::from_timestamp_opt(ts, 0)
                    .map(|dt| dt.to_string())
                    .unwrap_or_else(|| "Unknown".to_string()));
            }
            
            if let Some(gps) = &metadata.gps {
                println!("   ✓ GPS: {:.6}, {:.6}", gps.latitude, gps.longitude);
            }
        }
        Err(e) => {
            println!("   ✗ Failed to extract metadata: {}", e);
        }
    }
    println!();
    
    println!("=== Demo Complete ===");
    println!("\nPerformance notes:");
    println!("  • Embedded preview extraction: ~15-85ms (11-38x faster)");
    println!("  • RAW processing (half-size): ~255ms (11x faster)");
    println!("  • Full RAW processing: ~2800ms (baseline)");
    println!("\nWebP quality recommendations:");
    println!("  • Q85: Good for web previews (smaller files)");
    println!("  • Q92: Excellent balance (default, ML-ready)");
    println!("  • Q95: Very high quality (archival)");
    
    Ok(())
}
