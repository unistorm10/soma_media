use soma_media::{ImagePreprocessor, ImageConfig, ImageOutputFormat, RawOptions};
use std::time::Instant;

fn main() {
    let config = ImageConfig {
        width: 800,
        height: 600,
        format: ImageOutputFormat::Webp,
        quality: 85,
    };
    
    let processor = ImagePreprocessor::new(config);
    
    let srw_input = "/run/user/1000/gvfs/smb-share:server=main.local,share=test_data/sample_ext/07270143.SRW";
    let output = "/tmp/test_webp.webp";
    
    let options = RawOptions::fast_preview();
    
    println!("Testing RAW → WEBP pipeline (no FFmpeg, direct encoding):");
    println!("  Input: {}", srw_input);
    println!("  Output: {}", output);
    println!();
    
    let start = Instant::now();
    match processor.convert_raw_with_options(srw_input, output, &options) {
        Ok(_) => {
            let elapsed = start.elapsed();
            let size = std::fs::metadata(output).map(|m| m.len()).unwrap_or(0);
            println!("✓ Success: {:.3}s ({} bytes)", elapsed.as_secs_f64(), size);
        }
        Err(e) => {
            println!("✗ Error: {:?}", e);
        }
    }
}
