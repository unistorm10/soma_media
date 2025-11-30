use soma_media::RawProcessor;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let processor = RawProcessor::new()?;
    let path = Path::new("sample/03240163.dng");
    
    println!("Testing orientation detection...\n");
    
    // Extract metadata to see orientation
    let metadata = processor.extract_metadata(path)?;
    println!("Camera: {} {}", metadata.make, metadata.model);
    println!("Dimensions: {}x{}", metadata.width, metadata.height);
    
    Ok(())
}
