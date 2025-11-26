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
    
    let srw_input = "sample/07270143.SRW";
    
    let profiles = vec![
        ("fast_preview", RawOptions::fast_preview()),
        ("ml_training", RawOptions::ml_training()),
        ("professional", RawOptions::professional()),
        ("web_delivery", RawOptions::web_delivery()),
        ("archive", RawOptions::archive()),
    ];
    
    println!("Testing all RawOptions presets with AAHD:\n");
    
    for (name, options) in profiles {
        let output = format!("/tmp/test_profile_{}.webp", name);
        
        print!("{:20} ... ", name);
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
