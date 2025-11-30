# soma_media Deployment Guide

## 📦 Installation

### **Requirements**

- Rust 1.70+ (MSRV)
- FFmpeg 4.0+ (for audio/video operations)
- LibRaw (included via rsraw-sys)
- CUDA Toolkit 11.0+ (optional, for NVIDIA GPU acceleration)
- Vulkan/Metal drivers (optional, for GPU acceleration)

### **From Source**

```bash
git clone https://github.com/unistorm10/soma_media
cd soma_media

# CPU-only build (default)
cargo build --release

# With GPU acceleration
cargo build --release --features gpu-auto
```

### **As a Library**

```toml
[dependencies]
# Basic
soma_media = "0.1"

# With GPU
soma_media = { version = "0.1", features = ["gpu-auto"] }
```

## 🚀 Execution Modes

### **1. Embedded Mode** (Library)

Use soma_media directly in your Rust application:

```rust
use soma_media::{RawProcessor, PreviewOptions};

let processor = RawProcessor::new()?;
let webp = processor.extract_preview_webp(
    Path::new("photo.CR2"),
    &PreviewOptions::default()
)?;
```

**Pros**: Lowest latency, no IPC overhead  
**Cons**: Requires Rust integration  
**Use for**: ML pipelines, batch processing

### **2. Sidecar Mode** (UDS Daemon)

Run soma_media as a Unix Domain Socket daemon:

```bash
# Start daemon
./target/release/soma_media --socket-path /tmp/soma_media.sock

# With custom settings
./target/release/soma_media \
    --socket-path /var/run/soma/media.sock \
    --cardbus-socket /var/run/soma/cardbus.sock \
    --register-cardbus true
```

**Communication via UDS:**

```bash
# Send stimulus via nc/socat
echo '{"op":"raw.preview","input":{"input_path":"photo.CR2","output_path":"/tmp/preview.webp"},"context":{}}' | \
    nc -U /tmp/soma_media.sock
```

**Pros**: Language-agnostic, process isolation  
**Cons**: IPC overhead (~1-5ms)  
**Use for**: Microservices, polyglot systems

### **3. Server Mode** (TCP/HTTP)

Run as network service (future enhancement):

```bash
# Not yet implemented - use sidecar mode with proxy
```

## ⚙️ Configuration

### **Environment Variables**

```bash
# Logging level
export RUST_LOG=soma_media=info

# FFmpeg path (if not in PATH)
export FFMPEG_PATH=/usr/local/bin/ffmpeg

# LibRaw settings
export LIBRAW_USE_DNGSDK=0

# GPU settings
export SOMA_GPU_BACKEND=auto  # auto|cuda|wgpu|cpu
```

### **Daemon Configuration**

```toml
# config/soma_media.toml (future enhancement)
[daemon]
socket_path = "/var/run/soma/media.sock"
max_concurrent = 10
timeout_ms = 30000

[logging]
level = "info"
format = "json"

[gpu]
backend = "auto"  # auto|cuda|wgpu|cpu
max_batch_size = 100
```

## 🔧 System Setup

### **Ubuntu/Debian**

```bash
# Install FFmpeg
sudo apt update
sudo apt install -y ffmpeg libavcodec-dev libavformat-dev libavutil-dev

# Install CUDA (for NVIDIA GPU)
sudo apt install -y nvidia-cuda-toolkit

# Install Vulkan (for GPU acceleration)
sudo apt install -y vulkan-tools libvulkan-dev
```

### **RHEL/CentOS**

```bash
# Install FFmpeg
sudo yum install -y epel-release
sudo yum install -y ffmpeg ffmpeg-devel

# Install CUDA
# Follow NVIDIA's official installation guide

# Install Vulkan
sudo yum install -y vulkan-tools vulkan-loader-devel
```

### **macOS**

```bash
# Install FFmpeg
brew install ffmpeg

# Metal is built-in (no additional setup)
```

## 🐳 Docker Deployment

### **Dockerfile**

```dockerfile
FROM rust:1.70 as builder

# Install dependencies
RUN apt-get update && apt-get install -y \
    ffmpeg libavcodec-dev libavformat-dev libavutil-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY . .

# Build with GPU support
RUN cargo build --release --features gpu-wgpu

FROM debian:bookworm-slim

# Runtime dependencies
RUN apt-get update && apt-get install -y \
    ffmpeg libvulkan1 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/soma_media /usr/local/bin/

# Create socket directory
RUN mkdir -p /var/run/soma

EXPOSE 8080
VOLUME ["/var/run/soma"]

CMD ["soma_media", "--socket-path", "/var/run/soma/media.sock"]
```

### **Docker Compose**

```yaml
version: '3.8'

services:
  soma-media:
    build: .
    volumes:
      - /var/run/soma:/var/run/soma
      - ./data:/data:ro
    environment:
      - RUST_LOG=soma_media=info
      - SOMA_GPU_BACKEND=auto
    deploy:
      resources:
        reservations:
          devices:
            - driver: nvidia
              count: 1
              capabilities: [gpu]
```

## 📊 Health Checks

### **Liveness Check**

```bash
# UDS health endpoint
echo '{"op":"media.capabilities","input":{},"context":{}}' | nc -U /tmp/soma_media.sock
```

**Expected response:**
```json
{
  "ok": true,
  "output": {"name": "soma_media", "version": "0.1.0", ...},
  "latency_ms": 1,
  "cost": null
}
```

### **Readiness Check**

```bash
# Verify FFmpeg is accessible
ffmpeg -version

# Verify daemon is responding
timeout 5 bash -c 'echo "{\"op\":\"media.capabilities\",\"input\":{},\"context\":{}}" | nc -U /tmp/soma_media.sock'
```

## 🔒 Security

### **File Access**

- Daemon runs with minimal permissions
- Only reads input files (no writes except output)
- Validates all file paths to prevent traversal

### **Resource Limits**

```bash
# Systemd service with limits
[Service]
LimitNOFILE=10000
LimitNPROC=100
CPUQuota=200%
MemoryLimit=4G
```

### **Network Isolation**

- UDS mode: No network access needed
- Server mode: Use firewall rules to restrict access

## 📈 Performance Tuning

### **For Maximum Throughput**

```bash
# Enable GPU
cargo build --release --features gpu-auto

# Increase file descriptors
ulimit -n 10000

# Use ramdisk for temporary files
export TMPDIR=/dev/shm
```

### **For Low Latency**

```bash
# Use embedded mode (no IPC)
# Keep temporary files on fast SSD
# Enable CPU pinning for daemon process

taskset -c 0-3 soma_media --socket-path /tmp/soma_media.sock
```

### **For Batch Processing**

```rust
// Use batch operations
let gpu = GpuProcessor::new();
let results = gpu.batch_resize(images, 2048, 2048)?;
```

## 🔍 Monitoring

### **Metrics Collection**

```rust
// Built-in metrics API
let metrics = organ.metrics.summary();
println!("Operations: {}", metrics.total_operations);
println!("Success rate: {:.1}%", metrics.success_rate * 100.0);
println!("P95 latency: {}ms", metrics.p95_latency_ms);
```

### **Logging**

```bash
# Structured JSON logging
export RUST_LOG=soma_media=info,soma_media::organ=debug

# Log to file
soma_media 2>&1 | tee -a /var/log/soma_media.log
```

## 🚨 Troubleshooting

### **FFmpeg not found**

```bash
# Verify installation
which ffmpeg
ffmpeg -version

# Set explicit path
export FFMPEG_PATH=/usr/local/bin/ffmpeg
```

### **GPU not detected**

```bash
# Check CUDA
nvidia-smi

# Check Vulkan
vulkaninfo | head -20

# Force CPU mode
export SOMA_GPU_BACKEND=cpu
```

### **High memory usage**

```bash
# Reduce batch size
# Process files sequentially
# Enable memory limits in systemd
```

### **Daemon not responding**

```bash
# Check socket exists
ls -la /tmp/soma_media.sock

# Check daemon is running
ps aux | grep soma_media

# Check logs
journalctl -u soma-media -f
```

## 📚 Production Checklist

- [ ] FFmpeg installed and accessible
- [ ] GPU drivers installed (if using GPU features)
- [ ] Resource limits configured
- [ ] Health checks implemented
- [ ] Logging configured
- [ ] Metrics collection enabled
- [ ] Backup/recovery procedures defined
- [ ] Monitoring alerts configured
- [ ] Security audit completed
- [ ] Performance testing done

## 🆘 Support

- **Documentation**: Run `cargo doc --open`
- **Issues**: https://github.com/unistorm10/soma_media/issues
- **Examples**: See `examples/` directory
- **Tests**: Run `cargo test` for verification
