//! Test universal metadata extraction
use std::path::Path;
use soma_media::metadata::{extract_metadata, exiftool_available, ffprobe_available};

fn main() {
    println!("=== Universal Metadata Extraction Test ===\n");
    println!("ExifTool available: {}", exiftool_available());
    println!("FFprobe available: {}", ffprobe_available());
    println!();
    
    // Test on DNG file
    let path = Path::new("sample/03240163.dng");
    if path.exists() {
        println!("--- Testing DNG file ---");
        match extract_metadata(path) {
            Ok(meta) => {
                println!("File: {}", meta.source_file);
                println!("MIME: {}", meta.mime_type);
                println!("File Type: {}", meta.file_type);
                println!("File Size: {} bytes", meta.file_size);
                println!("Backend: {:?}", meta.backend);
                println!("Camera: {:?} {:?}", meta.make, meta.model);
                println!("Lens: {:?}", meta.lens.as_ref().and_then(|l| l.model.as_ref()));
                if let Some(dims) = &meta.dimensions {
                    println!("Dimensions: {}x{}", dims.width, dims.height);
                }
                println!("Megapixels: {:?}", meta.megapixels());
                println!("Aspect Ratio: {:?}", meta.aspect_ratio());
                println!("Exposure: {:?}", meta.exposure_summary());
                if let Some(exp) = &meta.exposure {
                    println!("  ISO: {:?}, Aperture: {:?}, Shutter: {:?}", exp.iso, exp.aperture, exp.shutter_speed);
                }
                println!("GPS: {:?}", meta.gps);
                println!("Date Created: {:?}", meta.date_created);
                println!("Software: {:?}", meta.software);
                println!("Orientation: {:?}", meta.orientation);
                println!();
                
                // Show some raw tags
                println!("Sample raw tags ({} total):", meta.raw_tags.len());
                for (key, val) in meta.raw_tags.iter().take(10) {
                    println!("  {}: {}", key, val);
                }
            }
            Err(e) => println!("Error: {:?}", e),
        }
    } else {
        println!("Sample file not found: {:?}", path);
    }
    
    // Test on SRW file if available
    let srw_path = Path::new("sample/202309101781.SRW");
    if srw_path.exists() {
        println!("\n--- Testing SRW file ---");
        match extract_metadata(srw_path) {
            Ok(meta) => {
                println!("Camera: {:?} {:?}", meta.make, meta.model);
                println!("Backend: {:?}", meta.backend);
                println!("Exposure: {:?}", meta.exposure_summary());
            }
            Err(e) => println!("Error: {:?}", e),
        }
    }
    
    println!("\n=== Test Complete ===");
}
