use soma_media::{ImagePreprocessor, ImageConfig, ImageOutputFormat, RawOptions};
use std::time::Instant;

fn main() {
    let config = ImageConfig {
        width: 1920,
        height: 1920,
        format: ImageOutputFormat::Jpeg,
        quality: 85,
    };
    
    let processor = ImagePreprocessor::new(config);
    
    let files = vec![
        "sample/03240053.SRW",
        "sample/202309101781.SRW",
        "sample/202310042332.SRW",
        "sample/201811174456.dng",
    ];
    
    let options = RawOptions::default();  // Use sensible defaults
    
    println!("Full pipeline: RAW -> 1920 long side JPEG @ quality 85\n");
    
    for file in files {
        let filename = std::path::Path::new(file)
            .file_stem()
            .unwrap()
            .to_string_lossy();
        let output = format!("sample/output/full_pipeline_{}.jpg", filename);
        
        print!("{:20} ... ", filename);
        let start = Instant::now();
        
        match processor.convert_raw_with_options(file, &output, &options) {
            Ok(_) => {
                let elapsed = start.elapsed();
                let size = std::fs::metadata(&output).map(|m| m.len()).unwrap_or(0);
                println!("{:.3}s ({} bytes)", elapsed.as_secs_f64(), size);
            }
            Err(e) => {
                println!("ERROR: {:?}", e);
            }
        }
    }
}
