#!/usr/bin/env bash
# Real-world integration test for soma_media organ

set -e

echo "╔═══════════════════════════════════════════════════════════════╗"
echo "║     soma_media Real-World Integration Test                   ║"
echo "╚═══════════════════════════════════════════════════════════════╝"
echo ""

# Test files
RAW_FILE="sample/03240053.SRW"
DNG_FILE="sample/201811174456.dng"
OUTPUT_DIR="/tmp/soma_media_integration_test"

if [ ! -f "$RAW_FILE" ]; then
    echo "❌ Test file not found: $RAW_FILE"
    exit 1
fi

mkdir -p "$OUTPUT_DIR"

echo "📁 Test files:"
echo "  • SRW: $RAW_FILE ($(du -h "$RAW_FILE" | cut -f1))"
echo "  • DNG: $DNG_FILE ($(du -h "$DNG_FILE" | cut -f1))"
echo ""

# Test 1: RAW Metadata Extraction
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Test 1: RAW Metadata Extraction (UMA Interface)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Create metadata test
cat > /tmp/metadata_test.rs << 'EOF'
use soma_media::organ::{MediaOrgan, Organ, Stimulus};
use serde_json::json;
use std::collections::HashMap;

#[tokio::main]
async fn main() {
    let organ = MediaOrgan::new();
    
    let stimulus = Stimulus {
        op: "raw.metadata".to_string(),
        input: json!({
            "input_path": "sample/03240053.SRW"
        }),
        context: HashMap::new(),
    };
    
    match organ.stimulate(stimulus).await {
        Ok(response) => {
            if response.ok {
                println!("✓ Metadata extracted successfully!");
                println!("  Latency: {}ms", response.latency_ms);
                println!("  Camera: {} {}", 
                         response.output["make"].as_str().unwrap_or("Unknown"),
                         response.output["model"].as_str().unwrap_or("Unknown"));
                println!("  ISO: {}", response.output["iso"]);
                println!("  Dimensions: {}x{}", 
                         response.output["width"], 
                         response.output["height"]);
            } else {
                eprintln!("✗ Error: {:?}", response.output);
                std::process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("✗ Failed: {}", e);
            std::process::exit(1);
        }
    }
}
EOF

cd /home/unistorm10/revealed/soma/body/soma_media
cargo run --quiet --bin metadata_test 2>&1 || \
    (cd /home/unistorm10/revealed/soma/body/soma_media && \
     rustc --edition 2021 -L target/debug/deps /tmp/metadata_test.rs \
     --extern soma_media=target/debug/libsoma_media.rlib \
     --extern tokio --extern serde_json -o /tmp/metadata_test && \
     /tmp/metadata_test)

echo ""

# Test 2: RAW Preview with different qualities
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Test 2: RAW Preview Generation (Multiple Qualities)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

for QUALITY in 75 85 92 95; do
    OUTPUT="$OUTPUT_DIR/preview_q${QUALITY}.webp"
    
    START=$(date +%s%3N)
    
    # Use library directly
    cargo run --quiet --release --example test_raw_preview -- \
        "$RAW_FILE" "$OUTPUT" "$QUALITY" 2>/dev/null || true
    
    END=$(date +%s%3N)
    ELAPSED=$((END - START))
    
    if [ -f "$OUTPUT" ]; then
        SIZE=$(du -h "$OUTPUT" | cut -f1)
        echo "✓ Q${QUALITY}: $SIZE (${ELAPSED}ms)"
    else
        echo "✗ Q${QUALITY}: Failed"
    fi
done

echo ""

# Test 3: Batch Processing
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Test 3: Batch Processing (All RAW files)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

TOTAL_START=$(date +%s%3N)
PROCESSED=0
FAILED=0

for RAW in sample/*.SRW sample/*.dng; do
    if [ -f "$RAW" ]; then
        BASENAME=$(basename "$RAW" | sed 's/\.[^.]*$//')
        OUTPUT="$OUTPUT_DIR/${BASENAME}_preview.webp"
        
        if cargo run --quiet --release --example test_raw_preview -- \
            "$RAW" "$OUTPUT" 92 2>/dev/null; then
            SIZE=$(du -h "$OUTPUT" | cut -f1)
            echo "  ✓ $BASENAME: $SIZE"
            PROCESSED=$((PROCESSED + 1))
        else
            echo "  ✗ $BASENAME: Failed"
            FAILED=$((FAILED + 1))
        fi
    fi
done

TOTAL_END=$(date +%s%3N)
TOTAL_ELAPSED=$((TOTAL_END - TOTAL_START))

echo ""
echo "📊 Batch Results:"
echo "  • Processed: $PROCESSED files"
echo "  • Failed: $FAILED files"
echo "  • Total time: ${TOTAL_ELAPSED}ms"
if [ $PROCESSED -gt 0 ]; then
    AVG=$((TOTAL_ELAPSED / PROCESSED))
    echo "  • Average: ${AVG}ms per file"
    
    # Calculate throughput
    if [ $TOTAL_ELAPSED -gt 0 ]; then
        THROUGHPUT=$((PROCESSED * 1000 / TOTAL_ELAPSED))
        echo "  • Throughput: ${THROUGHPUT} files/second"
    fi
fi

echo ""

# Test 4: File verification
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Test 4: Output File Verification"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

WEBP_COUNT=$(find "$OUTPUT_DIR" -name "*.webp" | wc -l)
TOTAL_SIZE=$(du -sh "$OUTPUT_DIR" | cut -f1)

echo "✓ Generated $WEBP_COUNT WebP files"
echo "✓ Total size: $TOTAL_SIZE"
echo ""

# Show sample files
echo "Sample outputs:"
find "$OUTPUT_DIR" -name "*.webp" | head -5 | while read FILE; do
    SIZE=$(du -h "$FILE" | cut -f1)
    NAME=$(basename "$FILE")
    echo "  • $NAME: $SIZE"
    
    # Verify WebP format
    if file "$FILE" | grep -q "Web/P"; then
        echo "    ✓ Valid WebP format"
    else
        echo "    ✗ Invalid format"
    fi
done

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✅ Integration Test Complete!"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "📁 Output directory: $OUTPUT_DIR"
echo "🔍 View images: eog $OUTPUT_DIR/*.webp"
echo ""
