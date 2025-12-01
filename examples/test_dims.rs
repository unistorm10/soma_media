use std::path::Path;
use soma_media::{RawProcessor, RawOptions};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let processor = RawProcessor::new()?;
    let path = Path::new("sample/202310042332.SRW");
    
    let options = RawOptions {
        auto_brightness: false,
        ..RawOptions::fast_preview()
    };
    
    // Get dimensions
    let (w, h) = processor.get_dimensions(path, &options)?;
    println!("Dimensions from get_dimensions: {}x{}", w, h);
    println!("Expected bytes: {} = {}x{}x3", w as usize * h as usize * 3, w, h);
    
    // Get actual data
    let data = processor.process_raw(path, &options)?;
    println!("Actual data size: {} bytes", data.len());
    
    // Calculate what dimensions would match
    let pixels = data.len() / 3;
    println!("Pixels: {}", pixels);
    
    // Try some common ratios
    for w in [3248, 3240, 3264, 6496, 6480].iter() {
        if pixels % w == 0 {
            let h = pixels / w;
            println!("  Could be: {}x{}", w, h);
        }
    }
    
    Ok(())
}
