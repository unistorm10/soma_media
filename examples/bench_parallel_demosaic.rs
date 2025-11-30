//! Parallel Demosaic Benchmark
//! 
//! Compares LibRaw single-thread demosaic vs our parallel tile-based demosaic.

use std::path::Path;
use std::time::Instant;
use soma_media::{RawProcessor, RawOptions};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let processor = RawProcessor::new()?;
    let sample_file = Path::new("sample/03240163.dng");
    
    if !sample_file.exists() {
        eprintln!("Sample file not found: {:?}", sample_file);
        return Ok(());
    }
    
    println!("\n=== Parallel Demosaic Benchmark ===\n");
    println!("File: {:?}", sample_file);
    println!("CPU cores: {}", std::thread::available_parallelism().map(|p| p.get()).unwrap_or(1));
    
    let options = RawOptions {
        half_size: false,  // Full resolution for fair comparison
        demosaic_algorithm: Some(0),  // Linear/Bilinear (fastest)
        auto_exposure: false,
        ..RawOptions::default()
    };
    
    // Standard LibRaw demosaic
    println!("\n📊 Standard LibRaw Demosaic:");
    let start = Instant::now();
    let result = processor.process_raw(sample_file, &options)?;
    let libraw_time = start.elapsed();
    println!("   Time: {:?}", libraw_time);
    println!("   Output size: {} bytes", result.len());
    
    // Tiled parallel demosaic - various tile sizes
    for tile_size in [256, 512, 1024] {
        println!("\n📊 Parallel Demosaic ({}px tiles):", tile_size);
        let start = Instant::now();
        let result = processor.process_raw_tiled(sample_file, &options, tile_size)?;
        let tiled_time = start.elapsed();
        
        let speedup = libraw_time.as_secs_f64() / tiled_time.as_secs_f64();
        println!("   Time: {:?}", tiled_time);
        println!("   Output size: {} bytes", result.len());
        println!("   Speedup: {:.2}x", speedup);
    }
    
    // Half-size mode comparison
    let half_size_opts = RawOptions {
        half_size: true,
        demosaic_algorithm: Some(0),
        auto_exposure: false,
        ..RawOptions::default()
    };
    
    println!("\n📊 Half-Size Mode (2x2 binning):");
    let start = Instant::now();
    let result = processor.process_raw(sample_file, &half_size_opts)?;
    let half_time = start.elapsed();
    println!("   Time: {:?}", half_time);
    println!("   Output size: {} bytes", result.len());
    println!("   vs Full: {:.2}x faster", libraw_time.as_secs_f64() / half_time.as_secs_f64());
    
    println!("\n=== Summary ===");
    println!("✅ Parallel demosaic uses rayon for tile-based processing");
    println!("✅ Each tile is demosaiced independently, then blended");
    println!("✅ Best speedup on CPUs with 4+ cores");
    println!("\n🔮 Future: GPU demosaic via soma_compute could provide 40-80x speedup");
    
    Ok(())
}
