use soma_media::{ImagePreprocessor, ImageConfig, ImageOutputFormat, RawOptions};
use std::time::Instant;

fn main() {
    // 1024px on long side (3:2 aspect ratio like original)
    let config = ImageConfig {
        width: 1024,
        height: 683,
        format: ImageOutputFormat::Webp,
        quality: 85,
    };
    
    let processor = ImagePreprocessor::new(config);
    
    let srw_input = "sample/03240053.SRW";
    
    let profiles = vec![
        ("fast_preview", RawOptions::fast_preview()),
        ("maximum", RawOptions::maximum()),
    ];
    
    println!("Testing RawOptions presets @ 1024x683:\n");
    
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
    
    println!("\nOutputs saved to:");
    println!("  /tmp/test_profile_fast_preview.webp");
    println!("  /tmp/test_profile_maximum.webp");
}
