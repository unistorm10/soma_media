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
    
    // Test all demosaic algorithms
    let algorithms = vec![
        (0, "Linear"),
        (1, "VNG"),
        (2, "PPG"),
        (3, "AHD"),
        (4, "DCB"),
        (5, "Modified AHD"),
        (11, "DHT"),
        (12, "AAHD"),
    ];
    
    println!("Testing demosaic algorithms on fast_preview preset:\n");
    
    for (algo, name) in algorithms {
        let mut options = RawOptions::fast_preview();
        options.demosaic_algorithm = Some(algo);
        
        let output = format!("/tmp/test_demosaic_{}.webp", algo);
        
        print!("{}: {} ... ", algo, name);
        let start = Instant::now();
        
        match processor.convert_raw_with_options(srw_input, &output, &options) {
            Ok(_) => {
                let elapsed = start.elapsed();
                let size = std::fs::metadata(&output).map(|m| m.len()).unwrap_or(0);
                println!("✓ {:?} ({} bytes)", elapsed, size);
            }
            Err(e) => {
                println!("✗ {:?}", e);
            }
        }
    }
}
