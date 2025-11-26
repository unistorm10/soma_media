use soma_media::{RawProcessor, RawOptions};
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let raw_file = Path::new("/run/user/1000/gvfs/smb-share:server=main.local,share=test_data/sample_ext/07270143.SRW");
    
    let processor = RawProcessor::new()?;
    
    // Test 1: Default fast_preview (AHD built-in)
    let options1 = RawOptions::fast_preview();
    println!("Test 1 - fast_preview default (AHD=3):");
    println!("  demosaic_algorithm: {:?}", options1.demosaic_algorithm);
    let rgb1 = processor.process_raw(raw_file, &options1)?;
    println!("  RGB data length: {} bytes", rgb1.len());
    
    // Test 2: Modified fast_preview with explicit AHD=3
    let mut options2 = RawOptions::fast_preview();
    options2.demosaic_algorithm = Some(3);
    println!("\nTest 2 - fast_preview with explicit AHD=3:");
    println!("  demosaic_algorithm: {:?}", options2.demosaic_algorithm);
    let rgb2 = processor.process_raw(raw_file, &options2)?;
    println!("  RGB data length: {} bytes", rgb2.len());
    
    // Test 3: Linear algorithm
    let mut options3 = RawOptions::fast_preview();
    options3.demosaic_algorithm = Some(0);
    println!("\nTest 3 - fast_preview with Linear=0:");
    println!("  demosaic_algorithm: {:?}", options3.demosaic_algorithm);
    let rgb3 = processor.process_raw(raw_file, &options3)?;
    println!("  RGB data length: {} bytes", rgb3.len());
    
    Ok(())
}
