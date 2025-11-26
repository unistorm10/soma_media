use soma_media::{RawProcessor, RawOptions};
use std::path::Path;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let raw_file = Path::new("/run/user/1000/gvfs/smb-share:server=main.local,share=test_data/sample_ext/07270143.SRW");
    
    if !raw_file.exists() {
        eprintln!("RAW file not found: {:?}", raw_file);
        return Ok(());
    }

    let processor = RawProcessor::new()?;
    let options = RawOptions::fast_preview();
    
    println!("Testing fast_preview preset:");
    println!("  white_balance: {:?}", options.white_balance);
    println!("  auto_brightness: {}", options.auto_brightness);
    println!("  half_size: {}", options.half_size);
    println!("  demosaic_algorithm: {:?}", options.demosaic_algorithm);
    println!();
    
    println!("Reading file...");
    let start = Instant::now();
    match processor.process_raw(raw_file, &options) {
        Ok(data) => {
            let elapsed = start.elapsed();
            println!("✓ Success: {:.3}s ({} bytes)", elapsed.as_secs_f64(), data.len());
        }
        Err(e) => {
            println!("✗ Error: {:?}", e);
        }
    }

    Ok(())
}
