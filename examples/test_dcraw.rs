use soma_media::{ImagePreprocessor, ImageConfig, ImageOutputFormat};

fn main() {
    let config = ImageConfig {
        width: 800,
        height: 600,
        format: ImageOutputFormat::Jpeg,
        quality: 85,
    };
    
    let processor = ImagePreprocessor::new(config);
    
    // Test SRW (Samsung RAW - should use libraw FFI)
    let srw_input = "/run/user/1000/gvfs/smb-share:server=main.local,share=test_data/sample_ext/07270143.SRW";
    let srw_output = "/tmp/test_srw_output.jpg";
    
    println!("Processing SRW with libraw FFI...");
    let start = std::time::Instant::now();
    match processor.convert_raw(srw_input, srw_output) {
        Ok(_) => {
            let elapsed = start.elapsed();
            if std::path::Path::new(srw_output).exists() {
                let size = std::fs::metadata(srw_output).unwrap().len();
                println!("✓ SRW processed in {:?} ({} bytes)", elapsed, size);
            }
        }
        Err(e) => eprintln!("✗ SRW processing failed: {}", e),
    }
}
