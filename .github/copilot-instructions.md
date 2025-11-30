# GitHub Copilot Instructions for soma_media

## Project Context
This is a Rust crate for professional RAW image processing using LibRaw. The goal is to achieve **complete feature parity with rawpy** (the Python LibRaw wrapper).

## Critical Current Issue
**exp_shift parameter doesn't work** - We can set it but it has no effect on output. This blocks RAW-level exposure control. See AGENTS.md for details.

## Development Priorities

1. **Fix exp_shift** - Make LibRaw's exposure shift actually work
2. **Study rawpy source** - See how they configure LibRaw correctly
3. **Add missing parameters** - user_black, user_sat, dcb_iterations, etc.
4. **Test against rawpy** - Compare output for same parameters
5. **Improve API** - Make it more ergonomic than rawpy while being compatible

## Code Standards

### When Adding LibRaw Parameters
```rust
// 1. Add to RawOptions struct
pub struct RawOptions {
    /// Documentation from rawpy/LibRaw
    pub exp_shift: f32,  // default: 1.0
}

// 2. Update Default impl
impl Default for RawOptions {
    fn default() -> Self {
        Self {
            exp_shift: 1.0,
            // ...
        }
    }
}

// 3. Update ALL presets (fast_preview, maximum, recovery)
// 4. Set in both process_raw functions (around line 380 and 580)
// 5. Test with examples/test_raw_exposure.rs
```

### Testing Requirements
- Always output JPEG (not WebP) for visual inspection
- Calculate average brightness to verify changes
- Compare with rawpy output when possible
- Test with sample/03240163.dng and sample/202309101781.SRW

### Reference Material
- **rawpy API:** https://letmaik.github.io/rawpy/api/rawpy.Params.html
- **rawpy source:** https://github.com/letmaik/rawpy
- **LibRaw docs:** https://www.libraw.org/docs/
- **Current plan:** See AGENTS.md

## Common Pitfalls

❌ **Don't use `params.bright` for exposure** - It's post-processing, clips highlights
❌ **Don't scale `user_mul` directly** - Makes image darker, not brighter
❌ **Don't use `exp_correc`** - It doesn't work in our configuration
✅ **Do use `exp_shift`** - Once we figure out why it's not working
✅ **Do test numerically** - Check average brightness, not just visual
✅ **Do check AGENTS.md** - Before starting work on exposure/parameters

## Key Files
- `src/raw.rs` - Main RAW processing (lines 360-650 = param configuration)
- `examples/test_raw_exposure.rs` - Current test file
- `AGENTS.md` - Development plan and status
- `sample/*.{dng,srw}` - Test RAW files

## When Stuck
1. Check rawpy source code for the same parameter
2. Search LibRaw documentation
3. Update AGENTS.md with findings
4. Consider if we need to patch rsraw-sys bindings

## Success Criteria
- exp_shift produces 2x brightness at shift=2.0
- Output matches rawpy pixel-for-pixel for same parameters
- All 30+ rawpy parameters implemented
- Performance within 2x of rawpy
