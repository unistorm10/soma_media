use soma_media::{ImagePreprocessor, ImageConfig, ImageOutputFormat, RawOptions};

fn main() {
    let config = ImageConfig {
        width: 1920,
        height: 1920,
        format: ImageOutputFormat::Jpeg,
        quality: 85,
    };
    
    let processor = ImagePreprocessor::new(config);
    
    // Test recovery preset on the silhouette photo
    let file = "sample/202309101781.SRW";
    let options = RawOptions::recovery();
    
    println!("Testing recovery preset on silhouette...\n");
    
    let start = std::time::Instant::now();
    match processor.convert_raw_with_options(file, "sample/output/recovered_silhouette.jpg", &options) {
        Ok(_) => {
            let elapsed = start.elapsed();
            let size = std::fs::metadata("sample/output/recovered_silhouette.jpg")
                .map(|m| m.len())
                .unwrap_or(0);
            println!("✓ Recovered in {:.3}s ({} bytes)", elapsed.as_secs_f64(), size);
        }
        Err(e) => {
            println!("✗ Error: {:?}", e);
        }
    }
}
