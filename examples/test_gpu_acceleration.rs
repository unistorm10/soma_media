//! GPU Acceleration Demo
//!
//! Demonstrates automatic GPU backend selection: CUDA → Vulkan/Metal → CPU
//! Shows performance comparison between backends for RAW preview generation
//!
//! Run with different features:
//!   cargo run --example test_gpu_acceleration --features gpu-auto <raw_file>
//!   cargo run --example test_gpu_acceleration --features gpu-wgpu <raw_file>
//!   cargo run --example test_gpu_acceleration --no-default-features <raw_file>

#[cfg(feature = "gpu-auto")]
use soma_media::{RawProcessor, PreviewOptions, GpuProcessor};

#[cfg(not(feature = "gpu-auto"))]
use soma_media::{RawProcessor, PreviewOptions};

use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n╔═══════════════════════════════════════════════════════════════╗");
    println!("║              GPU Acceleration Demonstration                   ║");
    println!("╚═══════════════════════════════════════════════════════════════╝\n");
    
    // Check if GPU features are enabled
    #[cfg(feature = "gpu-auto")]
    {
        run_gpu_demo()
    }
    
    #[cfg(not(feature = "gpu-auto"))]
    {
        run_cpu_only_demo()
    }
}

#[cfg(feature = "gpu-auto")]
fn run_gpu_demo() -> Result<(), Box<dyn std::error::Error>> {
    // Get test file
    let test_file = std::env::args()
        .nth(1)
        .unwrap_or_else(|| {
            eprintln!("❌ Error: No RAW file provided");
            eprintln!("\nUsage:");
            eprintln!("  cargo run --example test_gpu_acceleration --features gpu-auto <path_to_raw_file>\n");
            std::process::exit(1);
        });
    
    let file_path = Path::new(&test_file);
    
    if !file_path.exists() {
        eprintln!("❌ Error: File not found: {}\n", test_file);
        std::process::exit(1);
    }
    
    println!("📁 Test file: {}\n", test_file);
    
    // Initialize GPU processor (auto-detects best backend)
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("GPU Backend Detection");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    let gpu = GpuProcessor::new();
    println!("✓ Backend: {}", gpu.backend_info());
    println!("✓ GPU Available: {}\n", if gpu.has_gpu() { "Yes" } else { "No (CPU fallback)" });
    
    // Initialize RAW processor
    let processor = RawProcessor::new()?;
    
    // Test 1: GPU-accelerated preview extraction
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Test 1: GPU-Accelerated Preview Extraction");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    
    let output_gpu = "/tmp/soma_media_gpu_preview.webp";
    let options = PreviewOptions::default();
    
    let start = std::time::Instant::now();
    let webp_data = processor.extract_preview_webp_gpu(file_path, &options, &gpu)?;
    let elapsed_gpu = start.elapsed();
    
    std::fs::write(output_gpu, &webp_data)?;
    
    println!("✓ GPU-accelerated processing complete!");
    println!("  • Output:     {}", output_gpu);
    println!("  • Size:       {} bytes ({:.1} KB)", 
             webp_data.len(), 
             webp_data.len() as f64 / 1024.0);
    println!("  • Time:       {:?}", elapsed_gpu);
    println!("  • Backend:    {}", gpu.backend_info());
    
    // Test 2: CPU-only preview extraction (for comparison)
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Test 2: CPU-Only Preview (Comparison)");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    
    let output_cpu = "/tmp/soma_media_cpu_preview.webp";
    
    let start = std::time::Instant::now();
    let webp_cpu = processor.extract_preview_webp(file_path, &options)?;
    let elapsed_cpu = start.elapsed();
    
    std::fs::write(output_cpu, &webp_cpu)?;
    
    println!("✓ CPU processing complete!");
    println!("  • Output:     {}", output_cpu);
    println!("  • Size:       {} bytes ({:.1} KB)", 
             webp_cpu.len(), 
             webp_cpu.len() as f64 / 1024.0);
    println!("  • Time:       {:?}", elapsed_cpu);
    
    // Performance comparison
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Performance Comparison");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    
    println!("  GPU ({}):", gpu.backend_info());
    println!("    Time: {:?}", elapsed_gpu);
    
    println!("\n  CPU (SIMD):");
    println!("    Time: {:?}", elapsed_cpu);
    
    if elapsed_cpu > elapsed_gpu {
        let speedup = elapsed_cpu.as_millis() as f64 / elapsed_gpu.as_millis() as f64;
        println!("\n  🚀 GPU Speedup: {:.2}x faster", speedup);
    } else {
        println!("\n  ℹ️  No speedup (likely using embedded preview, no resize needed)");
    }
    
    // Test 3: Batch resize demonstration
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Test 3: Batch Resize (10 images simulation)");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    
    // Simulate 10 24MP images
    let test_images: Vec<(Vec<u8>, u32, u32)> = (0..10)
        .map(|_| {
            let rgb = vec![128u8; 6000 * 4000 * 3]; // 24MP dummy data
            (rgb, 6000, 4000)
        })
        .collect();
    
    let start = std::time::Instant::now();
    let _resized = gpu.batch_resize(test_images, 2048, 2048)?;
    let elapsed_batch = start.elapsed();
    
    println!("✓ Batch processing complete!");
    println!("  • Images:     10");
    println!("  • Source:     24MP (6000x4000)");
    println!("  • Target:     2MP (2048x2048)");
    println!("  • Total time: {:?}", elapsed_batch);
    println!("  • Per image:  {:?}", elapsed_batch / 10);
    println!("  • Throughput: {:.1} images/second", 
             10.0 / elapsed_batch.as_secs_f64());
    
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("✅ GPU Acceleration Demo Complete!");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    println!("\n📝 Summary:");
    println!("  • Backend: {}", gpu.backend_info());
    println!("  • GPU Available: {}", if gpu.has_gpu() { "Yes" } else { "No" });
    println!("  • Automatic Selection: ✓");
    println!("  • Zero Configuration: ✓");
    
    println!("\n📚 Features Tested:");
    println!("  ✓ Automatic backend detection (CUDA → Vulkan → CPU)");
    println!("  ✓ GPU-accelerated RAW preview extraction");
    println!("  ✓ Performance comparison");
    println!("  ✓ Batch processing\n");
    
    Ok(())
}

#[cfg(not(feature = "gpu-auto"))]
fn run_cpu_only_demo() -> Result<(), Box<dyn std::error::Error>> {
    println!("ℹ️  Running in CPU-only mode (no GPU features enabled)\n");
    println!("To enable GPU acceleration, rebuild with:");
    println!("  cargo run --example test_gpu_acceleration --features gpu-auto <raw_file>\n");
    
    // Get test file
    let test_file = std::env::args()
        .nth(1)
        .unwrap_or_else(|| {
            eprintln!("Usage: <command> <path_to_raw_file>\n");
            std::process::exit(1);
        });
    
    let file_path = Path::new(&test_file);
    
    if !file_path.exists() {
        eprintln!("❌ Error: File not found: {}\n", test_file);
        std::process::exit(1);
    }
    
    println!("📁 Test file: {}\n", test_file);
    
    // CPU-only processing
    let processor = RawProcessor::new()?;
    let options = PreviewOptions::default();
    
    println!("Processing with CPU (SIMD)...");
    let start = std::time::Instant::now();
    let webp_data = processor.extract_preview_webp(file_path, &options)?;
    let elapsed = start.elapsed();
    
    let output = "/tmp/soma_media_cpu_only_preview.webp";
    std::fs::write(output, &webp_data)?;
    
    println!("✓ Complete!");
    println!("  • Output: {}", output);
    println!("  • Time:   {:?}", elapsed);
    println!("  • Size:   {} bytes\n", webp_data.len());
    
    Ok(())
}
