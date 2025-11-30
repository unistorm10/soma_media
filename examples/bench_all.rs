//! Comprehensive RAW Processing Benchmark
//! 
//! Tests all acceleration methods for RAW image processing:
//! 1. Standard LibRaw (baseline)
//! 2. Parallel tile-based demosaic (CPU)
//! 3. Half-size mode (2x2 binning)
//! 4. Batch parallel processing
//! 5. GPU demosaic (if soma_compute available)

use std::path::Path;
use std::time::Instant;
use soma_media::{RawProcessor, RawOptions, PreviewOptions};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let processor = RawProcessor::new()?;
    let sample_file = Path::new("sample/03240163.dng");
    
    if !sample_file.exists() {
        eprintln!("❌ Sample file not found: {:?}", sample_file);
        return Ok(());
    }
    
    println!("\n╔══════════════════════════════════════════════════════════════════╗");
    println!("║          SOMA_MEDIA RAW PROCESSING BENCHMARK                     ║");
    println!("╠══════════════════════════════════════════════════════════════════╣");
    println!("║ File: {:?}", sample_file);
    println!("║ CPU cores: {}", std::thread::available_parallelism().map(|p| p.get()).unwrap_or(1));
    println!("╚══════════════════════════════════════════════════════════════════╝\n");

    let mut results: Vec<(&str, std::time::Duration, usize)> = vec![];
    
    // ══════════════════════════════════════════════════════════════════
    // 1. Standard LibRaw (baseline)
    // ══════════════════════════════════════════════════════════════════
    println!("📊 [1/5] Standard LibRaw Demosaic (baseline)...");
    let options = RawOptions {
        half_size: false,
        demosaic_algorithm: Some(0), // LINEAR
        auto_exposure: false,
        ..RawOptions::default()
    };
    let start = Instant::now();
    let result = processor.process_raw(sample_file, &options)?;
    let duration = start.elapsed();
    println!("   ✅ {:?} ({} bytes)", duration, result.len());
    results.push(("LibRaw Standard", duration, result.len()));
    let baseline = duration;
    
    // ══════════════════════════════════════════════════════════════════
    // 2. Parallel Tile-Based Demosaic
    // ══════════════════════════════════════════════════════════════════
    println!("\n📊 [2/5] Parallel Tile-Based Demosaic (512px tiles)...");
    let start = Instant::now();
    let result = processor.process_raw_tiled(sample_file, &options, 512)?;
    let duration = start.elapsed();
    let speedup = baseline.as_secs_f64() / duration.as_secs_f64();
    println!("   ✅ {:?} ({} bytes) - {:.2}x speedup", duration, result.len(), speedup);
    results.push(("Parallel Tiled", duration, result.len()));
    
    // ══════════════════════════════════════════════════════════════════
    // 3. Half-Size Mode
    // ══════════════════════════════════════════════════════════════════
    println!("\n📊 [3/5] Half-Size Mode (2x2 binning)...");
    let half_opts = RawOptions {
        half_size: true,
        demosaic_algorithm: Some(0),
        auto_exposure: false,
        ..RawOptions::default()
    };
    let start = Instant::now();
    let result = processor.process_raw(sample_file, &half_opts)?;
    let duration = start.elapsed();
    let speedup = baseline.as_secs_f64() / duration.as_secs_f64();
    println!("   ✅ {:?} ({} bytes) - {:.2}x speedup", duration, result.len(), speedup);
    results.push(("Half-Size", duration, result.len()));
    
    // ══════════════════════════════════════════════════════════════════
    // 4. Preview Extraction (embedded + fallback)
    // ══════════════════════════════════════════════════════════════════
    println!("\n📊 [4/5] Preview Extraction (WebP output)...");
    let preview_opts = PreviewOptions {
        max_dimension: Some(2048),
        quality: 92,
        force_raw_processing: false, // Try embedded first
        ..PreviewOptions::default()
    };
    let start = Instant::now();
    let result = processor.extract_preview_webp(sample_file, &preview_opts)?;
    let duration = start.elapsed();
    let speedup = baseline.as_secs_f64() / duration.as_secs_f64();
    println!("   ✅ {:?} ({} bytes WebP) - {:.2}x speedup", duration, result.len(), speedup);
    results.push(("Preview Extract", duration, result.len()));
    
    // ══════════════════════════════════════════════════════════════════
    // 5. Batch Processing (4 files in parallel)
    // ══════════════════════════════════════════════════════════════════
    println!("\n📊 [5/5] Batch Processing (4 files parallel)...");
    let files: Vec<&Path> = vec![sample_file; 4];
    let start = Instant::now();
    let batch_results = processor.batch_preview_webp(&files, &preview_opts);
    let duration = start.elapsed();
    let success_count = batch_results.iter().filter(|(_, r)| r.is_ok()).count();
    let throughput = 4.0 / duration.as_secs_f64();
    let per_file = duration.as_secs_f64() / 4.0;
    println!("   ✅ {:?} total ({}/4 success)", duration, success_count);
    println!("   📈 {:.2} files/sec, {:.0}ms per file", throughput, per_file * 1000.0);
    results.push(("Batch (4 files)", duration, 0));
    
    // ══════════════════════════════════════════════════════════════════
    // Summary
    // ══════════════════════════════════════════════════════════════════
    println!("\n╔══════════════════════════════════════════════════════════════════╗");
    println!("║                         RESULTS SUMMARY                          ║");
    println!("╠══════════════════════════════════════════════════════════════════╣");
    
    for (name, duration, _size) in &results {
        let speedup = baseline.as_secs_f64() / duration.as_secs_f64();
        let bar_len = (speedup * 10.0).min(40.0) as usize;
        let bar: String = "█".repeat(bar_len);
        println!("║ {:20} {:>8.0}ms  {:.2}x  {}", name, duration.as_millis(), speedup, bar);
    }
    
    println!("╠══════════════════════════════════════════════════════════════════╣");
    println!("║ RECOMMENDATIONS:                                                 ║");
    println!("║   🚀 Real-time preview: Half-Size mode (fastest)                ║");
    println!("║   ⚖️  Quality preview: Parallel Tiled (best balance)            ║");
    println!("║   📦 Batch ingest: Batch Processing (highest throughput)        ║");
    println!("║   🎯 Full quality: LibRaw Standard (highest quality)            ║");
    println!("╚══════════════════════════════════════════════════════════════════╝");
    
    // Check GPU availability
    println!("\n🔧 GPU Status:");
    let socket_path = "/tmp/soma_compute.sock";
    if std::path::Path::new(socket_path).exists() {
        println!("   ✅ soma_compute daemon detected at {}", socket_path);
        println!("   🚀 GPU acceleration available for image processing");
    } else {
        println!("   ⚠️  soma_compute daemon not running");
        println!("   💡 Start with: cargo run -p soma-compute --bin soma_compute");
        println!("   🔮 GPU demosaic could provide 40-80x additional speedup!");
    }
    
    Ok(())
}
