use soma_media::{ImagePreprocessor, ImageConfig, ImageOutputFormat, RawOptions};

fn main() {
    let config = ImageConfig {
        width: 800,
        height: 600,
        format: ImageOutputFormat::Jpeg,
        quality: 85,
    };
    
    let processor = ImagePreprocessor::new(config);
    let srw_input = "/run/user/1000/gvfs/smb-share:server=main.local,share=test_data/sample_ext/07270143.SRW";
    
    // Test all presets
    let presets = vec![
        ("Fast Preview", RawOptions::fast_preview(), "/tmp/srw_fast_preview.jpg"),
        ("Maximum Quality", RawOptions::maximum(), "/tmp/srw_maximum.jpg"),
        ("Recovery Mode", RawOptions::recovery(), "/tmp/srw_recovery.jpg"),
    ];
    
    for (name, options, output) in &presets {
        println!("\nTesting {} preset...", name);
        let start = std::time::Instant::now();
        
        match processor.convert_raw_with_options(srw_input, output, options) {
            Ok(_) => {
                let elapsed = start.elapsed();
                if std::path::Path::new(output).exists() {
                    let size = std::fs::metadata(output).unwrap().len();
                    println!("✓ {} processed in {:?} ({} bytes)", name, elapsed, size);
                }
            }
            Err(e) => eprintln!("✗ {} failed: {}", name, e),
        }
    }
}
