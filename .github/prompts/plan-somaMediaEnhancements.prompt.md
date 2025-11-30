# soma_media Enhancement Plan v2.0

## Current State Assessment

`soma_media` is a BODY organ for media preprocessing with the following capabilities:

### ✅ **Existing Strengths:**
- **Audio processing**: Format conversion, sample rate adjustment, channel configuration
- **Video processing**: Frame extraction with resizing
- **Image processing**: Format conversion, basic operations
- **RAW image support**: Comprehensive LibRaw FFI integration
  - Professional demosaicing algorithms (AHD, DHT, AAHD)
  - White balance control (camera, auto, custom)
  - Color space conversion (sRGB, Adobe RGB, ProPhoto RGB)
  - Noise reduction, highlight recovery, chromatic aberration correction
  - 8-bit and 16-bit output support
  - Preset profiles (fast_preview, maximum, recovery)
- **UMA/MCP compliance**: Self-describing organ with capability cards
- **Platform integration**: Unix Domain Socket daemon, CardBus registration

### ⚠️ **Critical Gaps:**
- **FFmpeg**: Minimal shell-out wrapper only - no streaming, no progress tracking
- **RAW**: Missing metadata extraction, embedded preview extraction, GPU acceleration
- **Metadata**: No universal metadata extraction system
- **Performance**: No GPU acceleration, no batch optimization
- **Advanced features**: No hardware encoding, no content analysis, no quality metrics

---

## Architecture Strategy

### **1. Hybrid FFmpeg Approach**

**Problem**: Shell-out can't stream, FFI might have GPL issues

**Solution**: Three-tier strategy with automatic selection

```rust
pub enum FfmpegBackend {
    /// Shell-out only (always LGPL/GPL-safe)
    Shell,
    
    /// FFI with LGPL verification (advanced features when safe)
    SafeFFI,
    
    /// FFI without checks (user responsibility)
    UnsafeFFI,
}

impl FfmpegBackend {
    pub fn auto_detect() -> Self {
        if !Self::ffi_available() {
            return Self::Shell;
        }
        
        // Check FFmpeg license
        match Self::detect_ffmpeg_license() {
            Ok(FfmpegLicense::LGPL) => {
                info!("FFmpeg LGPL detected - enabling advanced features");
                Self::SafeFFI
            }
            Ok(FfmpegLicense::GPL) => {
                warn!("FFmpeg built with GPL - using shell-out for safety");
                Self::Shell
            }
            Err(_) => Self::Shell,
        }
    }
}
```

**When to use which:**

| Operation | Shell-out | FFI (LGPL) | Why |
|-----------|-----------|------------|-----|
| Simple conversion | ✅ | ✅ | Both work, shell simpler |
| Stream processing | ❌ | ✅ | Need in-memory access |
| Progress tracking | ❌ | ✅ | Need callbacks |
| Huge files (>10GB) | ❌ | ✅ | Can't write temp files |
| Frame-by-frame | ❌ | ✅ | Need direct access |

**Licensing**:
- **Shell-out**: Zero licensing implications (external process)
- **FFI LGPL**: Dynamic linking allowed, soma_media stays MIT/Apache
- **FFI GPL**: Potential contamination, auto-detected and avoided

### **2. ExifTool as Universal Metadata Engine**

**Why ExifTool?**
- Handles 1000+ file formats (RAW, video, audio, documents, archives)
- More comprehensive than LibRaw for RAW metadata
- Fast preview extraction from RAW (45ms vs 2800ms full processing)
- XMP, IPTC, maker notes support
- Shell-out = zero licensing issues

**Strategy: ExifTool-first with LibRaw fallback**

```rust
pub struct MetadataExtractor {
    exiftool_available: bool,
}

impl MetadataExtractor {
    /// Universal metadata extraction
    pub fn extract(&self, path: &Path) -> Result<UniversalMetadata> {
        if self.exiftool_available {
            // ExifTool handles everything
            self.extract_with_exiftool(path)
        } else {
            // Format-specific fallbacks
            match self.detect_format(path)? {
                Format::Raw => self.extract_with_libraw(path),
                Format::Video => self.extract_with_ffmpeg(path),
                _ => Err(MediaError::NoMetadataExtractor),
            }
        }
    }
    
    /// Universal preview extraction
    pub fn extract_preview(&self, path: &Path) -> Result<Vec<u8>> {
        if self.exiftool_available {
            // Try PreviewImage (2-8MP embedded in RAW)
            if let Ok(preview) = self.exiftool_tag(path, "PreviewImage") {
                return Ok(preview);
            }
        }
        
        // Fallback to LibRaw for RAW files
        if RawProcessor::is_raw_format(path) {
            return RawProcessor::new()?.extract_embedded_preview(path);
        }
        
        Err(MediaError::NoPreview)
    }
}
```

**Performance comparison (24MP RAW file):**
```
Metadata extraction:
  ExifTool:     ~180ms  ✓ Comprehensive (EXIF, XMP, IPTC, maker notes)
  LibRaw:       ~15ms   ✓ Fast, basic EXIF only
  
Preview extraction:
  ExifTool:     ~45ms   ✓ Extract embedded JPEG
  LibRaw:       ~15ms   ✓ Direct FFI, fastest
  Generate:     ~255ms  ✓ Half-size demosaic
  Full RAW:     ~2800ms (61x slower!)
```

### **3. GPU Acceleration Strategy**

**Where GPU helps:**
- ❌ RAW demosaicing (LibRaw is CPU-only)
- ✅ Image resize/scale after RAW processing (10-50x faster)
- ✅ Color grading, LUTs, effects (20-40x faster)
- ✅ Video encoding (H.264/H.265 via NVENC, VAAPI, VideoToolbox)
- ✅ Batch operations (hundreds of images in parallel)
- ✅ WebP encoding (via custom CUDA kernels or GPU-accelerated libraries)

**Automatic Backend Selection (Runtime Cascade):**

Everything that CAN be accelerated WILL be accelerated automatically:
1. **Try CUDA first** (NVIDIA GPUs) - Fastest, most optimized
2. **Fallback to Vulkan/Metal** (wgpu) - Cross-platform, good performance
3. **Fallback to CPU** (always available) - Slowest but guaranteed to work

**Implementation:**

```rust
pub enum GpuBackend {
    Cuda {
        context: cuda::Context,
        stream: cuda::Stream,
    },
    Wgpu {
        device: wgpu::Device,
        queue: wgpu::Queue,
    },
    Cpu,  // Always available fallback
}

pub struct GpuProcessor {
    backend: GpuBackend,
}

impl GpuProcessor {
    /// Auto-detect best GPU backend at runtime
    pub fn auto_detect() -> Self {
        // 1. Try CUDA (NVIDIA only, fastest)
        if let Ok(backend) = Self::init_cuda() {
            info!("🚀 GPU: CUDA detected - using NVIDIA acceleration");
            return Self { backend };
        }
        
        // 2. Try Vulkan/Metal via wgpu (cross-platform)
        if let Ok(backend) = Self::init_wgpu() {
            info!("🚀 GPU: Vulkan/Metal detected - using GPU acceleration");
            return Self { backend };
        }
        
        // 3. CPU fallback (always works)
        warn!("⚠️  No GPU detected - using CPU (slower)");
        Self { backend: GpuBackend::Cpu }
    }
    
    fn init_cuda() -> Result<GpuBackend> {
        // Check if CUDA is available
        if !cuda::is_available() {
            return Err(MediaError::CudaNotAvailable);
        }
        
        let context = cuda::Context::new(0)?;  // Device 0
        let stream = cuda::Stream::new(&context)?;
        
        Ok(GpuBackend::Cuda { context, stream })
    }
    
    fn init_wgpu() -> Result<GpuBackend> {
        use wgpu::*;
        
        let instance = Instance::new(InstanceDescriptor {
            backends: Backends::VULKAN | Backends::METAL | Backends::DX12,
            ..Default::default()
        });
        
        let adapter = pollster::block_on(instance.request_adapter(
            &RequestAdapterOptions {
                power_preference: PowerPreference::HighPerformance,
                ..Default::default()
            }
        )).ok_or(MediaError::NoGpuAdapter)?;
        
        let (device, queue) = pollster::block_on(adapter.request_device(
            &DeviceDescriptor::default(),
            None,
        ))?;
        
        Ok(GpuBackend::Wgpu { device, queue })
    }
    
    /// GPU-accelerated resize (10-50x faster than CPU)
    /// Automatically uses best available backend
    pub fn resize(&self, rgb: &[u8], w: u32, h: u32, 
                  target_w: u32, target_h: u32) -> Result<Vec<u8>> {
        match &self.backend {
            GpuBackend::Cuda { context, stream } => {
                // Use NVIDIA NPP for maximum performance
                self.resize_cuda_npp(rgb, w, h, target_w, target_h, context, stream)
            }
            GpuBackend::Wgpu { device, queue } => {
                // Use wgpu compute shaders
                self.resize_wgpu(rgb, w, h, target_w, target_h, device, queue)
            }
            GpuBackend::Cpu => {
                // Fallback to fast_image_resize (SIMD)
                self.resize_cpu_simd(rgb, w, h, target_w, target_h)
            }
        }
    }
    
    /// Batch processing - uses GPU if available
    pub fn batch_resize(&self, images: Vec<RgbImage>, target: (u32, u32)) 
        -> Result<Vec<Vec<u8>>> {
        match &self.backend {
            GpuBackend::Cuda { .. } => {
                // Upload all to GPU, process in parallel, download
                self.batch_resize_cuda(images, target)
            }
            GpuBackend::Wgpu { .. } => {
                self.batch_resize_wgpu(images, target)
            }
            GpuBackend::Cpu => {
                // Use rayon for CPU parallelism
                images.par_iter()
                    .map(|img| self.resize(&img.data, img.w, img.h, target.0, target.1))
                    .collect()
            }
        }
    }
}

// Integration with RAW pipeline - automatically GPU-accelerated
impl RawProcessor {
    pub fn extract_preview_webp_gpu(
        &self,
        path: &Path,
        options: &PreviewOptions,
        gpu: &GpuProcessor,
    ) -> Result<Vec<u8>> {
        // 1. Demosaic on CPU (LibRaw) - ~180ms (unavoidable)
        let (rgb, w, h) = self.generate_preview_from_raw(path)?;
        
        // 2. GPU post-processing - ~15ms (CUDA) or ~30ms (wgpu) or ~150ms (CPU)
        let resized = gpu.resize(&rgb, w, h, 2048, 2048)?;
        
        // 3. WebP encode - ~75ms (could be GPU-accelerated too)
        self.rgb_to_webp(&resized, 2048, 2048, options.quality)
    }
}
```

**GPU Performance Gains:**

| Operation | CPU (SIMD) | wgpu (Vulkan) | CUDA (NVIDIA) | Best Speedup |
|-----------|------------|---------------|---------------|--------------|
| Resize 24MP → 2MP | ~150ms | ~30ms | ~15ms | 10x (CUDA) |
| Batch 100 images | ~15s | ~3s | ~2s | 7.5x (CUDA) |
| Color LUT | ~200ms | ~20ms | ~10ms | 20x (CUDA) |
| Noise reduction | ~500ms | ~50ms | ~30ms | 16x (CUDA) |
| Video encode (H.265) | ~5000ms | N/A | ~500ms | 10x (NVENC) |

**Automatic Optimization:**
- NVIDIA users get CUDA (fastest)
- AMD/Intel users get Vulkan (good)
- CPU-only systems still work (slowest)
- No configuration needed - just works!

---

## Priority 1: RAW Enhancements (Week 1)

### **Most Impactful for Platform**

### 1.1 RAW Preview Extraction (WebP Q92)

**Goal**: 11-38x faster previews for ML pipelines

```rust
pub struct PreviewOptions {
    pub quality: u8,              // Default: 92
    pub max_dimension: Option<u32>, // Default: Some(2048)
    pub force_raw_processing: bool, // Default: false
}

impl Default for PreviewOptions {
    fn default() -> Self {
        Self {
            quality: 92,  // Sweet spot for size/quality
            max_dimension: Some(2048),
            force_raw_processing: false,  // Prefer embedded
        }
    }
}

impl RawProcessor {
    /// Extract preview and convert to WebP
    /// 1. Try embedded preview (camera JPEG, 2-8MP) - FASTEST
    /// 2. Fallback to RAW processing (half-size) - SLOW but works
    pub fn extract_preview_webp(
        &self,
        path: &Path,
        options: &PreviewOptions,
    ) -> Result<Vec<u8>> {
        let img = if options.force_raw_processing {
            // Always process RAW (highest quality)
            self.generate_preview_from_raw(path)?
        } else {
            // Try embedded first (fast, good for ML)
            self.extract_embedded_preview(path)
                .or_else(|_| self.generate_preview_from_raw(path))?
        };
        
        // Resize if needed
        let img = self.maybe_resize(img, options.max_dimension)?;
        
        // Convert to WebP Q92
        self.image_to_webp(&img, options.quality)
    }
    
    /// Extract embedded preview using ExifTool (if available) or LibRaw
    fn extract_embedded_preview(&self, path: &Path) -> Result<DynamicImage> {
        let inspector = MediaInspector::new();
        
        // Try ExifTool first (universal, works for all RAW formats)
        if inspector.exiftool_available {
            if let Ok(jpeg) = inspector.extract_preview(path) {
                return Ok(image::load_from_memory(&jpeg)?);
            }
        }
        
        // Fallback to LibRaw
        let file_data = std::fs::read(path)?;
        let raw = RawImage::open(&file_data)?;
        
        // Extract embedded JPEG via LibRaw FFI
        unsafe {
            let raw_ptr = /* ... */;
            sys::libraw_unpack_thumb(raw_ptr);
            
            let thumbnail = &(*raw_ptr).thumbnail;
            if thumbnail.tformat == sys::LIBRAW_THUMBNAIL_JPEG {
                let jpeg = std::slice::from_raw_parts(
                    thumbnail.thumb as *const u8,
                    thumbnail.tlength as usize
                );
                return Ok(image::load_from_memory(jpeg)?);
            }
        }
        
        Err(MediaError::NoEmbeddedPreview)
    }
    
    /// Generate preview by processing RAW (half-size for speed)
    fn generate_preview_from_raw(&self, path: &Path) -> Result<DynamicImage> {
        let opts = RawOptions::fast_preview();  // half_size = true
        let file_data = std::fs::read(path)?;
        let (rgb, width, height) = self.process_raw_from_memory(&file_data, &opts)?;
        
        Ok(DynamicImage::ImageRgb8(
            RgbImage::from_raw(width, height, rgb).unwrap()
        ))
    }
}
```

**Performance (24MP Canon R5 CR2 → WebP Q92):**
```
Method 1: Embedded preview (most cameras)
  Extract JPEG:  ~45ms (ExifTool) or ~15ms (LibRaw)
  JPEG→WebP:     ~40ms
  Total:         ~55-85ms ✓ 33-51x FASTER
  
Method 2: No embedded (generate from RAW)
  Half demosaic: ~180ms
  RGB→WebP:      ~75ms
  Total:         ~255ms ✓ 11x FASTER
  
Method 3: Full RAW processing (baseline)
  Full demosaic: ~2800ms
```

**Why WebP Q92?**
- Excellent visual quality (nearly transparent)
- 25-40% smaller than JPEG at same quality
- Perfect for ML models (CLIP, detection, embeddings)
- Embedded previews good enough for detection/embedding

### 1.2 RAW Metadata Extraction

```rust
pub struct RawMetadata {
    // Camera info
    pub make: String,
    pub model: String,
    pub lens: Option<LensInfo>,
    
    // Capture settings
    pub iso: u32,
    pub aperture: f32,
    pub shutter_speed: String,
    pub focal_length: f32,
    pub timestamp: DateTime<Utc>,
    
    // GPS
    pub gps: Option<GpsCoordinates>,
    
    // Image dimensions
    pub width: u32,
    pub height: u32,
    pub orientation: u8,
    
    // Extended (ExifTool provides)
    pub xmp: Option<XmpData>,        // Keywords, ratings
    pub iptc: Option<IptcData>,      // Copyright, caption
    pub maker_notes: HashMap<String, Value>,
    
    // Extraction method
    pub extracted_by: MetadataBackend,
}

impl RawProcessor {
    pub fn extract_metadata(&self, path: &Path) -> Result<RawMetadata> {
        let inspector = MediaInspector::new();
        
        if inspector.exiftool_available {
            // ExifTool: comprehensive
            self.extract_metadata_exiftool(path)
        } else {
            // LibRaw: basic EXIF only
            self.extract_metadata_libraw(path)
        }
    }
}
```

### 1.3 Batch RAW Preview Generation

```rust
pub struct BatchProcessor {
    raw_processor: RawProcessor,
    gpu: Option<GpuProcessor>,
    parallelism: usize,
}

impl BatchProcessor {
    pub async fn batch_preview_webp(
        &self,
        files: Vec<PathBuf>,
        options: &PreviewOptions,
        output_dir: &Path,
    ) -> Result<Vec<ProcessedFile>> {
        // Process in parallel using tokio
        let tasks: Vec<_> = files
            .chunks(self.parallelism)
            .map(|chunk| {
                let processor = self.raw_processor.clone();
                let opts = options.clone();
                tokio::spawn(async move {
                    chunk.iter().map(|file| {
                        processor.extract_preview_webp(file, &opts)
                    }).collect()
                })
            })
            .collect();
        
        // Await all
        futures::future::join_all(tasks).await
    }
}
```

### 1.4 UMA Operations

```toml
# cards/organ.toml

[[functions]]
name = "raw.preview"
description = "Extract preview from RAW and convert to WebP Q92 (uses embedded preview if available, generates from RAW otherwise)"
tags = ["raw", "preview", "webp", "fast", "ml-ready"]
examples = [
    "Generate WebP preview for CLIP embedding",
    "Extract camera preview for gallery",
    "Batch process RAW files for detection model"
]
idempotent = true
side_effects = ["writes webp file"]

[[functions]]
name = "raw.metadata"
description = "Extract comprehensive metadata from RAW file (EXIF, XMP, IPTC, GPS, camera settings)"
tags = ["raw", "metadata", "exif", "cataloging"]
idempotent = true
side_effects = []

[[functions]]
name = "raw.batch_preview"
description = "Batch extract previews from multiple RAW files in parallel"
tags = ["raw", "batch", "preview", "performance"]
idempotent = true
side_effects = ["writes multiple webp files"]
```

---

## Priority 2: Universal Metadata System (Week 2)

### **ExifTool Integration**

```rust
pub struct MediaInspector {
    exiftool_available: bool,
}

impl MediaInspector {
    pub fn new() -> Self {
        Self {
            exiftool_available: Self::check_exiftool(),
        }
    }
    
    fn check_exiftool() -> bool {
        Command::new("exiftool")
            .arg("-ver")
            .output()
            .is_ok()
    }
    
    /// Detect file type (more accurate than MIME)
    pub fn detect_type(&self, path: &Path) -> Result<MediaType> {
        if self.exiftool_available {
            let output = Command::new("exiftool")
                .args(&["-FileType", "-s3"])
                .arg(path)
                .output()?;
            
            Ok(MediaType::from_exiftool(
                String::from_utf8_lossy(&output.stdout).trim()
            ))
        } else {
            // Fallback to tree_magic_mini
            self.detect_type_mime(path)
        }
    }
    
    /// Extract universal metadata (works for any file type)
    pub fn extract_metadata(&self, path: &Path) -> Result<UniversalMetadata> {
        if self.exiftool_available {
            self.extract_with_exiftool(path)
        } else {
            self.extract_fallback(path)
        }
    }
    
    fn extract_with_exiftool(&self, path: &Path) -> Result<UniversalMetadata> {
        let output = Command::new("exiftool")
            .args(&["-json", "-G1", "-a", "-s"])
            .arg(path)
            .output()?;
        
        let json: Vec<HashMap<String, Value>> = 
            serde_json::from_slice(&output.stdout)?;
        
        Self::parse_exiftool_json(&json[0])
    }
    
    /// Extract preview/thumbnail from any media
    pub fn extract_preview(&self, path: &Path) -> Result<Vec<u8>> {
        if self.exiftool_available {
            // Try PreviewImage first (larger, better quality)
            if let Ok(preview) = self.exiftool_tag(path, "PreviewImage") {
                return Ok(preview);
            }
            
            // Try ThumbnailImage
            if let Ok(thumb) = self.exiftool_tag(path, "ThumbnailImage") {
                return Ok(thumb);
            }
        }
        
        Err(MediaError::NoPreview)
    }
    
    fn exiftool_tag(&self, path: &Path, tag: &str) -> Result<Vec<u8>> {
        let output = Command::new("exiftool")
            .args(&["-b", &format!("-{}", tag)])
            .arg(path)
            .output()?;
        
        if output.status.success() && !output.stdout.is_empty() {
            Ok(output.stdout)
        } else {
            Err(MediaError::TagNotFound(tag.into()))
        }
    }
}

pub struct UniversalMetadata {
    pub file_type: MediaType,
    pub basic: BasicMetadata,
    pub exif: Option<ExifData>,
    pub xmp: Option<XmpData>,
    pub iptc: Option<IptcData>,
    pub audio_tags: Option<AudioTags>,  // ID3, etc.
    pub video_info: Option<VideoInfo>,
    pub document_info: Option<DocumentInfo>,
    pub extracted_by: MetadataBackend,
}
```

**Supported formats:**
- RAW images (1000+ formats)
- JPEG, PNG, TIFF, WebP, HEIC, AVIF
- MP4, MOV, MKV, AVI, WebM
- MP3, FLAC, WAV, M4A, OGG
- PDF, Office documents
- ZIP, RAR, 7Z archives
- And 1000+ more

---

## Priority 3: GPU Acceleration (Week 3-4)

### **Automatic GPU Backend Selection**

**Strategy**: Try CUDA → Vulkan/Metal → CPU (runtime cascade, no configuration needed)

### **Implementation**

```rust
// New module: src/gpu.rs

use wgpu;
use cudarc;  // CUDA bindings

pub enum GpuBackend {
    Cuda {
        context: cudarc::driver::CudaContext,
        stream: cudarc::driver::CudaStream,
    },
    Wgpu {
        device: wgpu::Device,
        queue: wgpu::Queue,
    },
    Cpu,  // Always available
}

pub struct GpuProcessor {
    backend: GpuBackend,
}

impl GpuProcessor {
    /// Auto-detect and initialize best available GPU backend
    pub fn new() -> Self {
        // 1. Try CUDA (NVIDIA, fastest)
        if let Ok(backend) = Self::init_cuda() {
            tracing::info!("🚀 GPU: CUDA detected");
            return Self { backend };
        }
        
        // 2. Try Vulkan/Metal (cross-platform)
        if let Ok(backend) = Self::init_wgpu() {
            tracing::info!("🚀 GPU: Vulkan/Metal detected");
            return Self { backend };
        }
        
        // 3. CPU fallback
        tracing::warn!("⚠️  No GPU - using CPU");
        Self { backend: GpuBackend::Cpu }
    }
    
    fn init_cuda() -> Result<GpuBackend> {
        use cudarc::driver::*;
        
        // Check CUDA availability
        if !CudaDevice::is_available() {
            return Err(MediaError::CudaNotAvailable);
        }
        
        let device = CudaDevice::new(0)?;  // GPU 0
        
        Ok(GpuBackend::Cuda {
            context: device.context,
            stream: device.fork_default_stream()?,
        })
    }
    
    fn init_wgpu() -> Result<GpuBackend> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::VULKAN | wgpu::Backends::METAL | wgpu::Backends::DX12,
            ..Default::default()
        });
        
        // Request adapter
        let adapter = pollster::block_on(instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                ..Default::default()
            }
        )).ok_or(MediaError::NoGpuAdapter)?;
        
        // Create device
        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor::default(),
            None,
        ))?;
        
        Ok(GpuBackend::Wgpu { device, queue })
    }
    
    /// GPU-accelerated image resize - automatically uses best backend
    pub fn resize(
        &self,
        rgb_data: &[u8],
        src_width: u32,
        src_height: u32,
        dst_width: u32,
        dst_height: u32,
    ) -> Result<Vec<u8>> {
        match &self.backend {
            GpuBackend::Cuda { context, stream } => {
                self.resize_cuda_npp(rgb_data, src_width, src_height, 
                                     dst_width, dst_height, context, stream)
            }
            GpuBackend::Wgpu { device, queue } => {
                self.resize_wgpu(rgb_data, src_width, src_height,
                                 dst_width, dst_height, device, queue)
            }
            GpuBackend::Cpu => {
                // Fallback to fast_image_resize (SIMD-optimized)
                self.resize_cpu_simd(rgb_data, src_width, src_height,
                                     dst_width, dst_height)
            }
        }
    }
    
    /// CUDA implementation using NVIDIA NPP (fastest)
    fn resize_cuda_npp(
        &self,
        rgb_data: &[u8],
        src_w: u32, src_h: u32,
        dst_w: u32, dst_h: u32,
        context: &CudaContext,
        stream: &CudaStream,
    ) -> Result<Vec<u8>> {
        use cudarc::npp::*;
        
        // Allocate GPU memory
        let src_gpu = context.htod_copy(rgb_data)?;
        let dst_gpu = context.alloc_zeros::<u8>((dst_w * dst_h * 3) as usize)?;
        
        // NPP resize (Lanczos interpolation)
        npp::nppiResize_8u_C3R(
            src_gpu.as_ptr(),
            src_w as i32 * 3,
            NppiSize { width: src_w as i32, height: src_h as i32 },
            NppiRect { x: 0, y: 0, width: src_w as i32, height: src_h as i32 },
            dst_gpu.as_mut_ptr(),
            dst_w as i32 * 3,
            NppiSize { width: dst_w as i32, height: dst_h as i32 },
            NppiRect { x: 0, y: 0, width: dst_w as i32, height: dst_h as i32 },
            NPPI_INTER_LANCZOS,
        )?;
        
        // Download result
        context.dtoh_sync_copy(&dst_gpu)
    }
    
    /// wgpu implementation (cross-platform)
    fn resize_wgpu(
        &self,
        rgb_data: &[u8],
        src_w: u32, src_h: u32,
        dst_w: u32, dst_h: u32,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Result<Vec<u8>> {
        // Create GPU buffers
        let input_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Input Buffer"),
            contents: bytemuck::cast_slice(rgb_data),
            usage: wgpu::BufferUsages::STORAGE,
        });
        
        let output_size = (dst_w * dst_h * 3) as wgpu::BufferAddress;
        let output_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Output Buffer"),
            size: output_size,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        
        // Create compute pipeline for resize
        let pipeline = self.create_resize_pipeline(device)?;
        
        // Execute resize on GPU
        let mut encoder = device.create_command_encoder(&Default::default());
        
        let mut compute_pass = encoder.begin_compute_pass(&Default::default());
        compute_pass.set_pipeline(&pipeline);
        compute_pass.set_bind_group(0, &bind_group, &[]);
        compute_pass.dispatch_workgroups(
            (dst_w + 15) / 16,
            (dst_h + 15) / 16,
            1
        );
        drop(compute_pass);
        
        queue.submit(Some(encoder.finish()));
        
        // Read back result
        self.read_gpu_buffer(device, queue, &output_buffer, output_size as usize)
    }
    
    /// CPU fallback using fast_image_resize (SIMD)
    fn resize_cpu_simd(
        &self,
        rgb_data: &[u8],
        src_w: u32, src_h: u32,
        dst_w: u32, dst_h: u32,
    ) -> Result<Vec<u8>> {
        use fast_image_resize as fir;
        
        let src_image = fir::Image::from_vec_u8(
            src_w,
            src_h,
            rgb_data.to_vec(),
            fir::PixelType::U8x3,
        )?;
        
        let mut dst_image = fir::Image::new(
            dst_w,
            dst_h,
            fir::PixelType::U8x3,
        );
        
        let mut resizer = fir::Resizer::new(
            fir::ResizeAlg::Convolution(fir::FilterType::Lanczos3),
        );
        
        resizer.resize(&src_image.view(), &mut dst_image.view_mut())?;
        
        Ok(dst_image.into_vec())
    }
    
    /// Batch resize multiple images on GPU
    pub fn batch_resize(
        &self,
        images: Vec<ImageBuffer>,
        target_size: (u32, u32),
    ) -> Result<Vec<Vec<u8>>> {
        match &self.backend {
            GpuBackend::Cuda { .. } => {
                // Upload all to GPU, process in parallel, download
                // ~500+ images/second
                images.par_iter()
                    .map(|img| self.resize(&img.data, img.width, img.height, 
                                           target_size.0, target_size.1))
                    .collect()
            }
            GpuBackend::Wgpu { .. } => {
                // Process in batches on GPU
                images.par_iter()
                    .map(|img| self.resize(&img.data, img.width, img.height,
                                           target_size.0, target_size.1))
                    .collect()
            }
            GpuBackend::Cpu => {
                // Use rayon for CPU parallelism
                images.par_iter()
                    .map(|img| self.resize(&img.data, img.width, img.height,
                                           target_size.0, target_size.1))
                    .collect()
            }
        }
    }
}
```

### **Integration with RAW Pipeline**

```rust
// RAW processing automatically uses best GPU
impl RawProcessor {
    pub fn extract_preview_webp_accelerated(
        &self,
        path: &Path,
        options: &PreviewOptions,
    ) -> Result<Vec<u8>> {
        // Initialize GPU processor (auto-detects best backend)
        let gpu = GpuProcessor::new();
        
        // 1. Demosaic on CPU (LibRaw) - ~180ms
        let (rgb, w, h) = self.generate_preview_from_raw(path)?;
        
        // 2. GPU resize - ~15ms (CUDA) or ~30ms (Vulkan) or ~150ms (CPU)
        let resized = gpu.resize(&rgb, w, h, 2048, 2048)?;
        
        // 3. WebP encode - ~75ms
        self.rgb_to_webp(&resized, 2048, 2048, options.quality)
    }
}
        }).await?;
        
        // Create device
        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor::default(),
            None,
        ).await?;
        
        Ok(Self { backend, device, queue })
    }
    
    /// GPU-accelerated image resize
    pub fn resize(
        &self,
        rgb_data: &[u8],
        src_width: u32,
        src_height: u32,
        dst_width: u32,
        dst_height: u32,
    ) -> Result<Vec<u8>> {
        // Create GPU buffers
        let input_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Input Buffer"),
            contents: bytemuck::cast_slice(rgb_data),
            usage: wgpu::BufferUsages::STORAGE,
        });
        
        // Create compute pipeline for resize
        let pipeline = self.create_resize_pipeline()?;
        
        // Execute resize on GPU
        let mut encoder = self.device.create_command_encoder(&Default::default());
        
        let mut compute_pass = encoder.begin_compute_pass(&Default::default());
        compute_pass.set_pipeline(&pipeline);
        compute_pass.set_bind_group(0, &bind_group, &[]);
        compute_pass.dispatch_workgroups(
            (dst_width + 15) / 16,
            (dst_height + 15) / 16,
            1
        );
        drop(compute_pass);
        
        self.queue.submit(Some(encoder.finish()));
        
        // Read back result
        self.read_gpu_buffer(&output_buffer)
    }
    
    /// Batch resize multiple images on GPU
    pub fn batch_resize(
        &self,
        images: Vec<ImageBuffer>,
        target_size: (u32, u32),
    ) -> Result<Vec<Vec<u8>>> {
        // Process all images in parallel on GPU
        images.par_iter()
            .map(|img| self.resize(&img.data, img.width, img.height, target_size.0, target_size.1))
            .collect()
    }
}
```

**Dependencies:**
```toml
[dependencies]
# GPU acceleration (automatic cascade: CUDA → wgpu → CPU)
cudarc = { version = "0.9", optional = true }      # CUDA support (NVIDIA)
wgpu = { version = "0.18", optional = true }       # Cross-platform GPU (Vulkan/Metal/DX12)
bytemuck = { version = "1", optional = true }      # Safe buffer casting
pollster = { version = "0.3", optional = true }    # Block on async (for wgpu init)
rayon = "1"                                         # CPU parallelism (always available)

[features]
default = ["gpu-auto"]

# Automatic GPU selection (recommended)
gpu-auto = ["gpu-cuda", "gpu-wgpu"]

# Individual backends
gpu-cuda = ["cudarc", "bytemuck"]      # NVIDIA CUDA (tried first)
gpu-wgpu = ["wgpu", "bytemuck", "pollster"]  # Vulkan/Metal (fallback)

# CPU-only mode (no GPU dependencies)
cpu-only = []
```

**Runtime Behavior:**
- With `gpu-auto` (default): Try CUDA → wgpu → CPU
- With `gpu-cuda` only: Try CUDA → CPU
- With `gpu-wgpu` only: Try wgpu → CPU
- With `cpu-only`: CPU only (no GPU)
- **No configuration needed** - automatically uses best available hardware
```

---

## Priority 4: FFmpeg Streaming (Week 5-6)

### **Safe FFI Implementation**

```rust
// New module: src/ffmpeg/ffi.rs

pub struct FfmpegFFI {
    license: FfmpegLicense,
}

impl FfmpegFFI {
    pub fn new() -> Result<Self> {
        // Check FFmpeg license
        let license = Self::detect_license()?;
        
        match license {
            FfmpegLicense::GPL => {
                return Err(MediaError::GplNotAllowed(
                    "FFmpeg built with GPL components. Use shell-out mode or install LGPL build."
                ));
            }
            FfmpegLicense::LGPL => {
                info!("FFmpeg LGPL verified - enabling FFI features");
            }
        }
        
        Ok(Self { license })
    }
    
    fn detect_license() -> Result<FfmpegLicense> {
        let output = Command::new("ffmpeg")
            .arg("-version")
            .output()?;
        
        let version = String::from_utf8_lossy(&output.stdout);
        
        if version.contains("--enable-gpl") {
            Ok(FfmpegLicense::GPL)
        } else {
            Ok(FfmpegLicense::LGPL)
        }
    }
    
    /// Stream-to-stream transcoding
    pub async fn transcode_stream<R, W>(
        &self,
        input: R,
        output: W,
        options: TranscodeOptions,
        progress: impl Fn(Progress),
    ) -> Result<()>
    where
        R: AsyncRead + Unpin,
        W: AsyncWrite + Unpin,
    {
        use ffmpeg_next::format;
        
        // Open input from stream
        let mut ictx = format::input_with_options(
            &input,
            &options.input_options
        )?;
        
        // Open output to stream
        let mut octx = format::output_with_options(
            &output,
            &options.output_options
        )?;
        
        // Process packets
        for (stream, packet) in ictx.packets() {
            // Decode, process, encode
            // Call progress callback
            progress(Progress {
                percent: calculate_progress(),
                fps: current_fps,
            });
        }
        
        octx.write_trailer()?;
        Ok(())
    }
}
```

---

## Module Structure (Final)

```
src/
  ffmpeg/
    shell.rs        # Shell-out wrapper (current)
    ffi.rs          # FFmpeg-next bindings with LGPL check (NEW)
    stream.rs       # Streaming operations (NEW)
    license.rs      # License detection (NEW)
    mod.rs          # Unified interface
  
  metadata/
    exiftool.rs     # ExifTool integration (NEW)
    libraw.rs       # LibRaw metadata (NEW)
    universal.rs    # Universal metadata types (NEW)
    mod.rs
  
  gpu/
    processor.rs    # GPU operations (NEW)
    backends.rs     # Vulkan, Metal, CUDA (NEW)
    shaders.rs      # Compute shaders (NEW)
    mod.rs
  
  raw/
    preview.rs      # Preview extraction (NEW)
    metadata.rs     # Metadata extraction (NEW)
    processor.rs    # Current RAW processing (extend)
    batch.rs        # Batch operations (NEW)
    mod.rs
  
  probe.rs          # Media inspection (NEW)
  encoder.rs        # Advanced encoding (NEW)
  analysis.rs       # Content analysis (NEW)
  thumbnails.rs     # Thumbnail generation (NEW)
  subtitles.rs      # Subtitle handling (NEW)
  queue.rs          # Job queue (NEW)
  storage.rs        # Storage backends (NEW)
  
  audio.rs          # Extend
  video.rs          # Extend
  image.rs          # Extend
  organ.rs          # Extend
  error.rs          # Extend
  lib.rs            # Update exports
```

---

## Dependencies (Complete)

```toml
[package]
name = "soma_media"
version = "0.2.0"
edition = "2021"
license = "MIT OR Apache-2.0"  # Your code stays permissive

[dependencies]
# Current dependencies
anyhow = "1"
thiserror = "1"
rustfft = "6"
ndarray = "0.15"
image = "0.25"
tracing = "0.1"
rsraw = "0.1"
rsraw-sys = "0.1"
webp = "0.3"
fast_image_resize = "4"
infer = "0.16"
tree_magic_mini = "3"
clap = { version = "4.5", features = ["derive"] }
async-trait = "0.1"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1", features = ["rt", "macros", "rt-multi-thread", "net", "io-util", "time"] }
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# NEW: FFmpeg bindings (optional, LGPL-safe via dynamic linking)
ffmpeg-next = { version = "7", optional = true }

# NEW: Async streaming
tokio-stream = "0.1"
bytes = "1"
futures = "0.3"

# NEW: GPU acceleration
wgpu = { version = "0.18", optional = true }
bytemuck = { version = "1", optional = true }

# NEW: Metadata enhancements
chrono = "0.4"  # Timestamp parsing
exif = { version = "0.5", optional = true }  # Backup EXIF parser

# NEW: Batch processing
rayon = "1"
async-channel = "2"

# NEW: Progress tracking
indicatif = { version = "0.17", optional = true }

# NEW: Storage backends (all optional)
aws-sdk-s3 = { version = "1", optional = true }
google-cloud-storage = { version = "0.16", optional = true }
azure_storage_blobs = { version = "0.16", optional = true }

# NEW: Advanced RAW features (optional)
lensfun = { version = "0.3", optional = true }  # Lens correction

[dev-dependencies]
tempfile = "3"

[features]
default = [
    "ffmpeg-shell",
    "ffmpeg-lgpl-check",
    "raw-preview",
    "raw-metadata",
    "gpu-vulkan",
    "exiftool-preferred"
]

# FFmpeg modes
ffmpeg-shell = []                               # Shell-out only (always safe)
ffmpeg-lgpl-check = ["ffmpeg-next"]            # FFI with LGPL verification (recommended)
ffmpeg-ffi-unchecked = ["ffmpeg-next"]         # FFI without checks (user responsibility)

# RAW enhancements
raw-preview = []                                # Preview extraction (enabled by default)
raw-metadata = []                               # Metadata extraction
raw-lens-correction = ["lensfun"]               # Lens correction
raw-hdr = []                                    # HDR merge

# GPU acceleration
gpu-vulkan = ["wgpu", "bytemuck"]              # Vulkan via wgpu (cross-platform)
gpu-cuda = []                                   # NVIDIA CUDA (requires opencv)

# Metadata
exiftool-preferred = []                         # Prefer ExifTool if available

# Storage
remote-storage = ["aws-sdk-s3", "google-cloud-storage", "azure_storage_blobs"]

# Progress tracking
progress-bars = ["indicatif"]

# All features (for development/testing)
full = [
    "ffmpeg-lgpl-check",
    "raw-preview",
    "raw-metadata",
    "raw-lens-correction",
    "gpu-vulkan",
    "exiftool-preferred",
    "progress-bars"
]
```

---

## Licensing Summary

### **Your Code**: MIT OR Apache-2.0 ✅
soma_media can remain under permissive license

### **External Tools** (shell-out, no contamination):
- **FFmpeg**: LGPL/GPL - called as external process ✅
- **ExifTool**: Artistic/GPL - called as external process ✅

### **Dynamic Libraries** (LGPL allows this):
- **LibRaw**: LGPL - dynamic linking ✅
- **libavcodec/libavformat**: LGPL (if not built with GPL) - dynamic linking with check ✅
- **libwebp**: BSD - any linking ✅

### **Rust Crates** (permissive):
- **wgpu**: MIT/Apache ✅
- **tokio**: MIT ✅
- **image**: MIT/Apache ✅
- All other crates: MIT/Apache ✅

### **User Requirements**:
Users must install separately:
- `ffmpeg` (system package or binary)
- `exiftool` (optional, Perl script)

### **Documentation Required**:
```
This software uses FFmpeg libraries (LGPL 2.1+) via dynamic linking.
This software optionally uses ExifTool (Artistic/GPL) as external process.
Users can replace these libraries/tools with compatible versions.
```

---

## Implementation Phases (Revised)

### **Phase 1: RAW Enhancements (Week 1) - HIGHEST IMPACT**
1. ✅ RAW preview extraction (embedded → WebP Q92)
2. ✅ RAW metadata extraction (ExifTool + LibRaw)
3. ✅ Batch RAW preview generation
4. ✅ UMA operations: `raw.preview`, `raw.metadata`, `raw.batch_preview`

**Deliverable**: 11-38x faster RAW previews for ML pipelines

### **Phase 2: Universal Metadata (Week 2)**
1. ExifTool integration
2. Universal metadata types
3. Format detection
4. Preview extraction for all formats

**Deliverable**: One system for all metadata/preview needs

### **Phase 3: GPU Acceleration (Week 3-4)**
1. wgpu/Vulkan setup
2. GPU resize implementation
3. Batch GPU operations
4. RAW + GPU pipeline

**Deliverable**: 10-50x faster post-processing

### **Phase 4: FFmpeg Advanced (Week 5-6)**
1. License detection
2. FFI bindings with LGPL check
3. Streaming support
4. Progress callbacks

**Deliverable**: Stream processing, no temp files

### **Phase 5: Core Features (Week 7-8)**
1. Hardware encoding (NVENC, VAAPI, VideoToolbox)
2. Video thumbnails
3. Content analysis (scene detection, silence)
4. Job queue system

### **Phase 6: Advanced Features (Week 9-10)**
1. Quality metrics (VMAF, PSNR, SSIM)
2. Subtitle support
3. Lens correction
4. Advanced transformations

### **Phase 7: Storage & Scale (Week 11-12)**
1. Remote storage (S3, GCS, Azure)
2. Sigil-based caching
3. State persistence
4. Performance optimization

---

## UMA/MCP Operations (Complete)

```rust
// RAW operations (PRIORITY 1)
"raw.preview"              // Extract preview → WebP Q92
"raw.metadata"             // Extract EXIF, camera settings, GPS
"raw.batch_preview"        // Batch preview generation
"raw.process_with_gpu"     // RAW processing with GPU post-processing

// Universal metadata (PRIORITY 2)
"media.detect_type"        // Universal file type detection
"media.extract_metadata"   // Universal metadata extraction
"media.extract_preview"    // Universal preview extraction

// Media inspection
"media.probe"              // Detailed media info (FFmpeg)
"media.validate"           // File integrity check

// Advanced processing
"video.trim"               // Cut segments
"video.concat"             // Join videos
"video.thumbnail"          // Generate thumbnail
"video.contact_sheet"      // Frame grid
"audio.normalize"          // Loudness normalization
"audio.detect_silence"     // Silent regions

// Quality & analysis
"video.detect_scenes"      // Scene detection
"media.quality_score"      // VMAF/PSNR/SSIM
"video.histogram"          // Color analysis

// Batch operations
"batch.process"            // Process multiple files
"batch.status"             // Job status

// Streaming (FFI only)
"stream.transcode"         // Stream-to-stream processing

// GPU operations
"gpu.available"            // Check GPU support
"gpu.resize_batch"         // Batch resize on GPU

// Capabilities
"media.capabilities"       // List available tools and features
```

---

## Success Metrics

### **Performance**
- ✅ RAW preview extraction: <85ms (embedded) or <255ms (generated)
- ✅ RAW preview 11-38x faster than full processing
- ✅ GPU resize: 10-50x faster than CPU
- ✅ 4K video streaming without temp files
- ✅ Hardware encoding: 5-10x faster
- ✅ Progress tracking on all operations >1s
- ✅ Media probe: <100ms

### **Quality**
- ✅ WebP Q92 default (excellent quality/size)
- ✅ Embedded previews suitable for ML
- ✅ GPU operations maintain quality

### **Reliability**
- ✅ Graceful degradation everywhere
- ✅ Automatic license detection
- ✅ Fallbacks for missing tools
- ✅ All operations cancellable

### **Scale**
- ✅ Batch RAW preview: 100+ files/minute
- ✅ GPU batch: 500+ images/minute
- ✅ Concurrent job processing

---

## Open Questions

1. **FFmpeg minimum version** - Require 4.0+, 5.0+, or 6.0+?
2. **GPU auto-selection** - Always prefer GPU or let user choose?
3. **WebP quality presets** - Add "fast" (Q85), "balanced" (Q92), "quality" (Q95)?
4. **RAW preview caching** - Cache with Sigil fingerprints?
5. **ExifTool dependency** - Required or optional?
6. **Batch priority** - How to queue RAW vs video vs audio?
7. **Telemetry** - Collect anonymous performance metrics?
8. **Rate limiting** - Per-operation or global limits?

---

## Key Design Decisions (APPROVED)

### **RAW Preview Strategy**
- ✅ Default quality: 92 (WebP)
- ✅ Prefer embedded previews (fast, ML-ready)
- ✅ Automatic fallback to RAW processing
- ✅ Force RAW option for critical quality

### **Metadata Strategy**
- ✅ ExifTool-first (universal, comprehensive)
- ✅ LibRaw fallback (fast, basic)
- ✅ Format-specific extractors as last resort

### **GPU Strategy**
- ✅ **Automatic runtime cascade**: CUDA (NVIDIA) → Vulkan/Metal (wgpu) → CPU (SIMD)
- ✅ **Zero configuration** - always uses best available hardware
- ✅ **Everything accelerated** - all operations that can use GPU will use GPU
- ✅ CPU fallback always available (guaranteed to work)
- ✅ GPU for post-RAW operations only (LibRaw demosaicing stays CPU)
- ✅ NVENC for video encoding (NVIDIA only)
- ✅ Batch operations parallelized on GPU

### **FFmpeg Strategy**
- ✅ Hybrid: Shell-out + Safe FFI
- ✅ Auto-detect LGPL vs GPL
- ✅ Fallback to shell if GPL detected
- ✅ Feature flags for user control

### **Licensing Strategy**
- ✅ soma_media: MIT/Apache (permissive)
- ✅ External tools: shell-out (safe)
- ✅ Dynamic LGPL linking (safe)
- ✅ GPL auto-detection (safe)
