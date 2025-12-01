//! Test camera profile extraction and application
//!
//! This demonstrates how to extract the camera's color processing
//! from MakerNotes and apply it to RAW-processed images to match
//! the in-camera JPEG output.

use std::path::Path;
use soma_media::profiles::extract_camera_profile;
use soma_media::{RawProcessor, RawOptions, PreviewOptions};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Camera Profile Test ===\n");
    
    // Test SRW file (Samsung NX1)
    let srw_path = Path::new("sample/202310042332.SRW");
    if srw_path.exists() {
        println!("--- Samsung NX1 SRW Profile ---\n");
        
        let profile = extract_camera_profile(srw_path)?;
        println!("{}", profile.summary());
        
        // Show detailed info
        if let Some(ref matrix) = profile.color_matrix_srgb {
            println!("sRGB Color Matrix:");
            println!("  [{:6.3} {:6.3} {:6.3}]", matrix.m[0][0], matrix.m[0][1], matrix.m[0][2]);
            println!("  [{:6.3} {:6.3} {:6.3}]", matrix.m[1][0], matrix.m[1][1], matrix.m[1][2]);
            println!("  [{:6.3} {:6.3} {:6.3}]", matrix.m[2][0], matrix.m[2][1], matrix.m[2][2]);
        }
        
        if let Some(ref curve) = profile.tone_curve {
            println!("\nTone Curve ({} points):", curve.input.len());
            println!("  Input:  {:?}", curve.input);
            println!("  Output: {:?}", curve.output);
        }
        
        // Process RAW with linear settings
        println!("\n--- Processing RAW with Profile ---\n");
        
        let processor = RawProcessor::new()?;
        
        // Get RAW-processed image with MINIMAL processing
        // We want linear RGB output so we can apply the camera profile ourselves
        use soma_media::{ColorSpace, WhiteBalance};
        
        let options = RawOptions {
            color_space: ColorSpace::Raw,  // Linear output, no color conversion
            auto_brightness: false,
            white_balance: WhiteBalance::None,  // Don't apply WB (we'll use profile WB)
            half_size: true,  // Half-size for speed
            ..Default::default()
        };
        
        // Get RGB data with correct dimensions (may be rotated by LibRaw)
        let (rgb_data, width, height) = processor.process_raw_with_dims(srw_path, &options)?;
        let width = width as usize;
        let height = height as usize;
        let expected_size = width * height * 3;
        
        println!("Dimensions: {}x{}", width, height);
        println!("Raw data: {} bytes (expected {})", rgb_data.len(), expected_size);
        
        if rgb_data.len() == expected_size {
            let rgb = rgb_data.clone();
            
            // Save WITHOUT profile first to check base processing
            std::fs::create_dir_all("test_output")?;
            let img_no_profile = image::RgbImage::from_raw(width as u32, height as u32, rgb)
                .expect("Failed to create image");
            img_no_profile.save("test_output/no_profile.jpg")?;
            println!("Saved: test_output/no_profile.jpg (no profile applied)");
            
            // Now apply profile
            let mut rgb = rgb_data;
            
            // Calculate average before profile
            let avg_before: f32 = rgb.iter().map(|&v| v as f32).sum::<f32>() / rgb.len() as f32;
            
            // Apply camera profile
            profile.apply_to_rgb(&mut rgb, width, height);
            
            // Calculate average after profile
            let avg_after: f32 = rgb.iter().map(|&v| v as f32).sum::<f32>() / rgb.len() as f32;
            
            println!("Average brightness: {:.1} → {:.1}", avg_before, avg_after);
            
            let img_out = image::RgbImage::from_raw(width as u32, height as u32, rgb)
                .expect("Failed to create image");
            img_out.save("test_output/profile_applied.jpg")?;
            println!("Saved: test_output/profile_applied.jpg");
        } else {
            println!("Unexpected data size - skipping image save");
        }
        
        // Also save the embedded preview for comparison (as JPEG)
        let preview_opts = PreviewOptions::default();
        if let Ok(preview) = processor.extract_preview_webp(srw_path, &preview_opts) {
            // Convert WebP to JPEG for consistency
            let img = image::load_from_memory(&preview)?;
            img.save("test_output/camera_preview.jpg")?;
            println!("Camera preview saved: test_output/camera_preview.jpg");
        }
    } else {
        println!("Sample file not found: {:?}", srw_path);
    }
    
    // Test DNG file
    let dng_path = Path::new("sample/03240163.dng");
    if dng_path.exists() {
        println!("\n--- DNG Profile ---\n");
        
        let profile = extract_camera_profile(dng_path)?;
        println!("{}", profile.summary());
    }
    
    println!("\n=== Test Complete ===");
    Ok(())
}
