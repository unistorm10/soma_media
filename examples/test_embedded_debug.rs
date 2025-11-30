use soma_media::{RawProcessor};
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing embedded preview extraction with debug info\n");
    
    let files = vec![
        "sample/03240163.dng",
        "sample/03240198.dng", 
        "sample/202309101781.SRW",
    ];
    
    let processor = RawProcessor::new()?;
    
    for file in files {
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("File: {}", file);
        
        if !Path::new(file).exists() {
            println!("  ✗ File not found\n");
            continue;
        }
        
        // Try to extract embedded preview
        println!("  Attempting embedded preview extraction...");
        let result = processor.extract_preview_webp(
            Path::new(file),
            &soma_media::PreviewOptions {
                quality: 92,
                max_dimension: Some(2048),
                force_raw_processing: false,
            }
        );
        
        match result {
            Ok(webp_data) => {
                println!("  ✓ Success! Size: {} bytes ({:.1} KB)", 
                         webp_data.len(),
                         webp_data.len() as f64 / 1024.0);
                
                let output = format!("test_output/debug_{}.webp", 
                                    Path::new(file).file_stem().unwrap().to_str().unwrap());
                std::fs::write(&output, &webp_data)?;
                println!("  ✓ Saved to: {}", output);
            }
            Err(e) => {
                println!("  ✗ Failed: {}", e);
            }
        }
        
        // Try forced RAW processing for comparison
        println!("  Testing forced RAW processing...");
        let result2 = processor.extract_preview_webp(
            Path::new(file),
            &soma_media::PreviewOptions {
                quality: 92,
                max_dimension: Some(2048),
                force_raw_processing: true,
            }
        );
        
        match result2 {
            Ok(webp_data) => {
                println!("  ✓ RAW processing: {} bytes ({:.1} KB)", 
                         webp_data.len(),
                         webp_data.len() as f64 / 1024.0);
            }
            Err(e) => {
                println!("  ✗ RAW processing failed: {}", e);
            }
        }
        
        println!();
    }
    
    Ok(())
}
