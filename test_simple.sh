#!/usr/bin/env bash
# Simpler real-world test for soma_media

set -e

echo "╔═══════════════════════════════════════════════════════════════╗"
echo "║     soma_media Real-World Test Suite                          ║"
echo "╚═══════════════════════════════════════════════════════════════╝"
echo ""

OUTPUT_DIR="/tmp/soma_media_test_$(date +%s)"
mkdir -p "$OUTPUT_DIR"

echo "📁 Output directory: $OUTPUT_DIR"
echo ""

# Test 1: Process all RAW files
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Test 1: Batch RAW Preview Generation"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

TOTAL_START=$(date +%s%N)
PROCESSED=0

for RAW in sample/*.SRW sample/*.dng; do
    if [ -f "$RAW" ]; then
        BASENAME=$(basename "$RAW" .SRW)
        BASENAME=$(basename "$BASENAME" .dng)
        OUTPUT="$OUTPUT_DIR/${BASENAME}_preview.webp"
        
        echo -n "Processing $(basename "$RAW")... "
        
        FILE_START=$(date +%s%N)
        
        # Use Rust directly
        if timeout 30s cargo run --quiet --release --bin soma_media -- \
            raw-preview "$RAW" "$OUTPUT" 2>/dev/null || \
           cargo run --quiet --release --example test_raw_preview -- "$RAW" "$OUTPUT" 92 2>&1 >/dev/null; then
            
            FILE_END=$(date +%s%N)
            ELAPSED=$(( (FILE_END - FILE_START) / 1000000 ))
            
            if [ -f "$OUTPUT" ]; then
                SIZE=$(du -h "$OUTPUT" | cut -f1)
                echo "✓ ${SIZE} (${ELAPSED}ms)"
                PROCESSED=$((PROCESSED + 1))
            else
                echo "✗ No output"
            fi
        else
            echo "✗ Failed"
        fi
    fi
done

TOTAL_END=$(date +%s%N)
TOTAL_MS=$(( (TOTAL_END - TOTAL_START) / 1000000 ))

echo ""
echo "📊 Results:"
echo "  • Files processed: $PROCESSED"
echo "  • Total time: ${TOTAL_MS}ms"

if [ $PROCESSED -gt 0 ]; then
    AVG=$((TOTAL_MS / PROCESSED))
    echo "  • Average: ${AVG}ms per file"
    
    THROUGHPUT=$(echo "scale=2; $PROCESSED * 1000 / $TOTAL_MS" | bc)
    echo "  • Throughput: ${THROUGHPUT} files/second"
fi

echo ""

# Test 2: Verify outputs
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Test 2: Output Verification"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

WEBP_COUNT=$(find "$OUTPUT_DIR" -name "*.webp" 2>/dev/null | wc -l)

if [ $WEBP_COUNT -gt 0 ]; then
    echo "✓ Generated $WEBP_COUNT WebP files"
    echo ""
    echo "Files:"
    
    find "$OUTPUT_DIR" -name "*.webp" | while read FILE; do
        SIZE=$(du -h "$FILE" | cut -f1)
        NAME=$(basename "$FILE")
        
        # Verify it's actually WebP
        if command -v file >/dev/null && file "$FILE" | grep -q "Web/P"; then
            STATUS="✓"
        else
            STATUS="?"
        fi
        
        # Get dimensions if possible
        if command -v identify >/dev/null 2>&1; then
            DIMS=$(identify -format "%wx%h" "$FILE" 2>/dev/null || echo "unknown")
            echo "  $STATUS $NAME: $SIZE ($DIMS)"
        else
            echo "  $STATUS $NAME: $SIZE"
        fi
    done
    
    echo ""
    TOTAL_SIZE=$(du -sh "$OUTPUT_DIR" 2>/dev/null | cut -f1)
    echo "Total size: $TOTAL_SIZE"
else
    echo "✗ No WebP files generated"
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✅ Test Complete!"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "📁 Results: $OUTPUT_DIR"
echo ""
