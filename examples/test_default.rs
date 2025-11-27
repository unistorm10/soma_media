use soma_media::{ImagePreprocessor, ImageConfig, ImageOutputFormat, RawOptions, WhiteBalance};
use std::time::Instant;

fn main() {
    let config_webp = ImageConfig {
        width: 1024,
        height: 683,
        format: ImageOutputFormat::Webp,
        quality: 85,
    };
    
    let config_jpg = ImageConfig {
        width: 1024,
        height: 683,
        format: ImageOutputFormat::Jpeg,
        quality: 85,
    };
    
    let processor_webp = ImagePreprocessor::new(config_webp);
    let processor_jpg = ImagePreprocessor::new(config_jpg);
    
    let files = vec![
        "sample/03240053.SRW",
        "sample/202309101781.SRW",
        "sample/202310042332.SRW",
    ];
    
    println!("Testing default RawOptions (half_size) @ 1024x683:\n");
    
    let mut default_options = RawOptions::default();
    default_options.half_size = true; // Re-enable half_size for speed
    default_options.gamma = Some((2.222, 4.5)); // sRGB gamma
    default_options.white_balance = WhiteBalance::Auto;
    default_options.auto_brightness = true;
    default_options.demosaic_algorithm = Some(12); // AAHD for quality
    
    for file in files {
        let filename = std::path::Path::new(file)
            .file_stem()
            .unwrap()
            .to_string_lossy();
        
        // Create output directory if it doesn't exist
        std::fs::create_dir_all("sample/output").ok();
        
        // WEBP output
        let output_webp = format!("sample/output/test_default_{}.webp", filename);
        print!("{:20} WEBP ... ", filename);
        let start = Instant::now();
        
        match processor_webp.convert_raw_with_options(file, &output_webp, &default_options) {
            Ok(_) => {
                let elapsed = start.elapsed();
                let size = std::fs::metadata(&output_webp).map(|m| m.len()).unwrap_or(0);
                println!("✓ {:?} ({} bytes)", elapsed, size);
            }
            Err(e) => {
                println!("✗ {:?}", e);
            }
        }
        
        // JPEG output
        let output_jpg = format!("sample/output/test_default_{}.jpg", filename);
        print!("{:20} JPEG ... ", filename);
        let start = Instant::now();
        
        match processor_jpg.convert_raw_with_options(file, &output_jpg, &default_options) {
            Ok(_) => {
                let elapsed = start.elapsed();
                let size = std::fs::metadata(&output_jpg).map(|m| m.len()).unwrap_or(0);
                println!("✓ {:?} ({} bytes)", elapsed, size);
            }
            Err(e) => {
                println!("✗ {:?}", e);
            }
        }
    }
    
    println!("\nOutputs saved to sample/output/test_default_*.(webp|jpg)");
}
