---
description: 'Universal fingerprinting system for all hashing, content addressing, and entity identification'
applyTo: '**/*'
---

# Sigil - Universal Fingerprinting System

## 🎯 MANDATORY: Use Sigil for ALL Hashing & Fingerprinting

**Locations**: 
- Local development: `/home/unistorm10/revealed/sigil/`
- GitHub repo: `https://github.com/unistorm10/sigil` (private)

Sigil is the **ONLY** approved system for content fingerprinting, hashing, deduplication, and entity identification across the entire Revealed Platform.

## ⚠️ Critical Rules

1. **NEVER use Blake3 directly** - Always use `sigil::Sigil::from_bytes()`
2. **NEVER use SHA hashes directly** - Use `sigil::HashAlgorithm::Sha256` or other variants
3. **NEVER implement custom hashing** - Sigil provides all needed algorithms
4. **ALWAYS use Fingerprintable trait** - For entities that can be serialized
5. **NEVER create duplicate sigil modules** - Use the external crate

## 📦 Integration

### Add to Cargo.toml

**Option 1: Path dependency (local development)**
```toml
[dependencies]
sigil = { path = "../../sigil" }  # Adjust path relative to your crate
```

**Option 2: Git dependency (from sigil repo - RECOMMENDED)**
```toml
[dependencies]
sigil = { git = "https://github.com/unistorm10/sigil.git", branch = "main" }
```

**Note**: The sigil repository is private, so cargo will use your git credentials for authentication.

### Import in Code
```rust
use sigil::{Sigil, HashAlgorithm, Fingerprintable, EntityType};
```

## 🔨 Common Use Cases

### 1. Content Hashing (Blobs, Files, Data)
```rust
// For raw bytes
let data = b"content to hash";
let sigil = Sigil::from_bytes(data, HashAlgorithm::Blake3);
let content_address = sigil.content_address(); // Use as storage key

// For files
let sigil = Sigil::from_file("path/to/file", HashAlgorithm::Blake3)?;
```

### 2. Entity Fingerprinting (Modules, Models, Configs, etc.)
```rust
use sigil::{Fingerprintable, EntityType};
use serde::Serialize;

#[derive(Serialize)]
struct MyEntity {
    name: String,
    data: Vec<u8>,
}

let entity = MyEntity { /* ... */ };

// Automatic fingerprinting via trait
let sigil = entity.fingerprint()?;  // Uses Blake3 by default

// With specific algorithm
let sigil = entity.fingerprint_with(HashAlgorithm::Sha256)?;

// With entity type
let sigil = entity.fingerprint()?
    .with_entity_type(EntityType::Module);
```

### 3. Content-Addressable Storage with Deduplication
```rust
// Chunk large files for deduplication
let sigil = Sigil::from_bytes_chunked(data, HashAlgorithm::Blake3);

// Access chunks
for chunk in sigil.chunks() {
    println!("Chunk at {} ({} bytes): {}", 
             chunk.offset, chunk.length, chunk.hash);
    // Store chunk separately for deduplication
}
```

### 4. Cache Keys
```rust
// Generate deterministic cache key
let cache_data = "user query or content";
let sigil = Sigil::from_bytes(cache_data.as_bytes(), HashAlgorithm::Blake3);
let cache_key = format!("cache/{}", sigil.content_address());
```

### 5. Verification
```rust
// Verify data matches fingerprint
let data = b"some data";
let sigil = Sigil::from_bytes(data, HashAlgorithm::Blake3);

// Later...
if sigil.verify(data) {
    println!("Data matches!");
}

// Or for entities
let entity = MyEntity { /* ... */ };
if entity.verify_fingerprint(&sigil)? {
    println!("Entity matches!");
}
```

## 🎨 Available Hash Algorithms

Choose based on use case:

- **Blake3** (default) - Fast, parallel, cryptographically secure - USE THIS for most cases
- **Sha256** - Industry standard, wide compatibility, regulatory compliance
- **Sha512** - Higher security margin than SHA-256
- **Sha3_256** - Post-quantum resistant, long-term archival
- **XxHash3** - Ultra-fast non-cryptographic, checksums only (optional feature)

```rust
// Specify algorithm
let sigil = Sigil::from_bytes(data, HashAlgorithm::Sha256);
let sigil = Sigil::from_bytes(data, HashAlgorithm::Sha3_256);
```

## 📋 Entity Types (28 Total)

Sigil supports comprehensive entity classification:

### Code Entities
- `EntityType::Module` - Code modules/packages
- `EntityType::Function` - Individual functions
- `EntityType::Habit` - SOMA habits/skills
- `EntityType::SourceCode` - Raw source files

### AI/ML Entities
- `EntityType::Model` - ML models
- `EntityType::Adapter` - Model adapters (LoRA, etc.)
- `EntityType::Checkpoint` - Training checkpoints
- `EntityType::Dataset` - Training datasets
- `EntityType::Embedding` - Vector embeddings

### Configuration Entities
- `EntityType::Config` - Configuration files
- `EntityType::Manifest` - Package manifests
- `EntityType::Policy` - Security/access policies
- `EntityType::Schema` - Data schemas

### Data Entities
- `EntityType::Blob` - Binary blobs
- `EntityType::Document` - Text documents
- `EntityType::Image` - Image files
- `EntityType::Video` - Video files
- `EntityType::Audio` - Audio files
- `EntityType::Record` - Database records

### Identity Entities
- `EntityType::User` - User accounts
- `EntityType::Session` - User sessions
- `EntityType::Transaction` - Transactions

### Network Entities
- `EntityType::Node` - Network nodes
- `EntityType::Peer` - P2P peers

### Semantic Entities
- `EntityType::SemanticProjection` - Semantic projections
- `EntityType::KnowledgeNode` - Knowledge graph nodes
- `EntityType::Expert` - Expert agents

### Fallback
- `EntityType::Unknown` - Unclassified entities

## 🔍 Advanced Features

### Canonical Serialization
Entities always produce the same fingerprint regardless of field order:

```rust
// These produce IDENTICAL fingerprints
let entity1 = json!({"name": "test", "value": 42});
let entity2 = json!({"value": 42, "name": "test"});

assert_eq!(
    entity1.fingerprint()?.content_address(),
    entity2.fingerprint()?.content_address()
);
```

### Metadata Attachment
```rust
let sigil = entity.fingerprint()?
    .with_entity_type(EntityType::Model)
    .with_metadata(json!({
        "version": "1.0.0",
        "author": "SOMA",
        "trained_on": "2024-11-09"
    }));
```

### Multi-Encoding
```rust
use sigil::Encoding;

let hex = sigil.content_address(); // Default hex
let base64 = sigil.encode_hash(Encoding::Base64)?;
let multibase = sigil.encode_hash(Encoding::Multibase)?; // IPFS-compatible
```

## 🚫 What NOT to Do

```rust
// ❌ WRONG - Direct Blake3 usage
let hash = blake3::hash(data).to_hex();

// ✅ CORRECT - Use Sigil
let sigil = Sigil::from_bytes(data, HashAlgorithm::Blake3);
let hash = sigil.content_address();

// ❌ WRONG - Direct SHA usage
use sha2::{Sha256, Digest};
let hash = Sha256::digest(data);

// ✅ CORRECT - Use Sigil with SHA algorithm
let sigil = Sigil::from_bytes(data, HashAlgorithm::Sha256);

// ❌ WRONG - Custom entity hashing
fn hash_module(module: &Module) -> String {
    blake3::hash(serde_json::to_vec(module).unwrap()).to_hex()
}

// ✅ CORRECT - Use Fingerprintable trait
let sigil = module.fingerprint()?;
```

## 📊 Performance Notes

- **Blake3**: ~3 GB/s (parallelized) - DEFAULT
- **SHA-256**: ~500 MB/s
- **SHA-512**: ~800 MB/s  
- **SHA3-256**: ~200 MB/s
- **XxHash3**: ~15 GB/s (non-cryptographic)

FastCDC chunking: ~1 GB/s with 50%+ chunk stability

## 🔗 Platform Integration

Sigil is already integrated in:
- ✅ soma_vault (content-addressable blob storage)
- ✅ soma_storage (all projects can access it)
- 📋 soma (brain/executive/src/core/sigil - DEPRECATED, use external crate)
- 📋 model_vault - Should migrate to external sigil

## 📚 Testing

Sigil has comprehensive test coverage:
- ✅ 16/16 unit tests passing
- ✅ 5/5 doc tests passing
- ✅ Content-defined chunking verified
- ✅ Multi-algorithm consistency tested
- ✅ Canonical serialization validated

## 🎓 When to Choose Which Algorithm

| Use Case | Algorithm | Reason |
|----------|-----------|---------|
| General purpose | Blake3 | Fast, secure, parallel |
| Blob storage keys | Blake3 | Performance critical |
| Cache keys | Blake3 | Speed matters |
| Compliance/audit | Sha256 | Industry standard |
| Legal/regulatory | Sha512 | Higher security bar |
| Long-term archive | Sha3_256 | Post-quantum resistance |
| Quick checksums | XxHash3 | Non-cryptographic, ultra-fast |

## 🔐 Security Notes

- Blake3, SHA-2, and SHA-3 are cryptographically secure
- Use for: content verification, deduplication, integrity checks
- xxHash3 is NOT cryptographic - use only for checksums
- All algorithms produce deterministic outputs
- Sigil includes timestamps but not in hash calculation

## 📦 Status

**Production Ready** ✅
- No deprecation warnings
- All tests passing
- Zero-copy operations
- Future-proof versioning (SIGIL_VERSION = 1)
- Comprehensive documentation

---

**Remember**: If you're about to hash anything, fingerprint anything, or generate content addresses - USE SIGIL!
