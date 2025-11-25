---

description: 'Dependency management guidelines - prefer Git dependencies over local paths'
---
applyTo: "**/Cargo.toml"
# Dependency Management Guidelines

## 🎯 Core Principle: Use Git Dependencies

**Always prefer Git dependencies over local path dependencies** for all platform modules and libraries. This ensures:
- ✅ Modules work regardless of local file system layout
- ✅ CI/CD can build without local checkouts
- ✅ Other developers can use your modules immediately
- ✅ Dependency versions are explicit and trackable

## 📦 Dependency Patterns

### Platform Modules (from unistorm10 GitHub)

For all your platform modules, use Git dependencies:

```toml
[dependencies]
# ✅ CORRECT - Git dependency (works everywhere)
sigil = { git = "https://github.com/unistorm10/sigil.git", branch = "main" }
umbrafs = { git = "https://github.com/unistorm10/umbrafs.git", branch = "main" }
model_vault = { git = "https://github.com/unistorm10/model_vault.git", branch = "main" }

# ❌ WRONG - Local path (only works on your machine)
sigil = { path = "../../sigil" }
umbrafs = { path = "../umbrafs" }
```

### Specifying Versions

Use specific tags or commits for production stability:

```toml
[dependencies]
# Use tagged release
sigil = { git = "https://github.com/unistorm10/sigil.git", tag = "v0.1.0" }

# Use specific commit (for unreleased fixes)
sigil = { git = "https://github.com/unistorm10/sigil.git", rev = "abc1234" }

# Use branch (for active development)
sigil = { git = "https://github.com/unistorm10/sigil.git", branch = "main" }
```

### Development Override

For local development, use Cargo's `[patch]` or path override instead of changing dependencies:

**Option 1: In workspace root Cargo.toml**
```toml
[patch."https://github.com/unistorm10/sigil.git"]
sigil = { path = "../sigil" }
```

**Option 2: In .cargo/config.toml (user-level)**
```toml
[patch."https://github.com/unistorm10/sigil.git"]
sigil = { path = "/home/unistorm10/revealed/sigil" }
```

This way, dependencies remain Git-based in `Cargo.toml`, but you can test local changes.

## 🔧 Common Platform Modules

Here are your common platform modules with correct Git dependencies:

```toml
[dependencies]
# Fingerprinting and content addressing
sigil = { git = "https://github.com/unistorm10/sigil.git", branch = "main" }

# File system and storage
umbrafs = { git = "https://github.com/unistorm10/umbrafs.git", branch = "main" }

# Model management
model_vault = { git = "https://github.com/unistorm10/model_vault.git", branch = "main" }

# SOMA modules (when available)
soma_vault = { git = "https://github.com/unistorm10/soma_vault.git", branch = "main" }
soma_storage = { git = "https://github.com/unistorm10/soma_storage.git", branch = "main" }
# Add other soma_* modules as needed
```

## 🚫 When NOT to Use Git Dependencies

**Only use local path dependencies for:**
- ❌ Workspace members in a multi-crate workspace
- ❌ Unpublished experimental code not in Git
- ❌ Temporary local testing (use `[patch]` instead)

## ✅ Best Practices

1. **Use `branch = "main"`** for active development dependencies
2. **Use `tag = "vX.Y.Z"`** for stable release dependencies
3. **Use `rev = "commit-hash"`** when you need a specific unreleased commit
4. **Document why** if you must use a specific commit/branch in comments
5. **Keep dependencies up to date** - periodically update branch/tag references
6. **Use workspace dependencies** to share versions across multi-crate projects

## 🔄 Converting Local to Git Dependencies

If you find local path dependencies, follow these steps:

### Step 1: Check if Git Repository Exists

Before converting, verify the repository exists on GitHub:

```bash
# Check if repo exists for user unistorm10 or your organization
curl -s https://api.github.com/repos/unistorm10/MODULE_NAME
```

### Step 2: Create and Publish Repository if Needed

**If the repository does NOT exist**, create and publish it first:

1. **Create the GitHub repository**:
   - Use GitHub CLI: `gh repo create unistorm10/MODULE_NAME --private`
   - Or create via GitHub web interface at https://github.com/new
   - Make it **private** for proprietary platform modules

2. **Initialize and push the local module**:
   ```bash
   cd /path/to/module
   git init
   git add .
   git commit -m "Initial commit"
   git branch -M main
   git remote add origin https://github.com/unistorm10/MODULE_NAME.git
   git push -u origin main
   ```

3. **Add essential files**:
   - `README.md` - Module description and usage
   - `LICENSE` - Proprietary or appropriate license
   - `.gitignore` - Rust-specific (include `/target`, `Cargo.lock` for libraries)

### Step 3: Convert the Dependency

Once the repository exists and is published:

```toml
# Before
[dependencies]
sigil = { path = "../../sigil" }

# After
[dependencies]
sigil = { git = "https://github.com/unistorm10/sigil.git", branch = "main" }
```

### Step 4: Verify the Conversion

```bash
# Clean and rebuild to ensure Git dependency works
cargo clean
cargo build
```

## 🆕 Creating New Platform Modules

When creating a new platform module from scratch:

1. **Create GitHub repository first**: `gh repo create unistorm10/new_module --private`
2. **Clone and initialize**: `git clone https://github.com/unistorm10/new_module.git`
3. **Set up Cargo project**: `cargo init --lib` (or `--bin`)
4. **Configure dependencies**: Use Git dependencies from the start
5. **Commit and push**: Establish the module on GitHub immediately

This ensures the module is always portable and accessible.

## 🏗️ Workspace Pattern

For multi-crate workspaces, define shared dependencies in the workspace root:

```toml
# Workspace Cargo.toml
[workspace]
members = ["crate_a", "crate_b"]

[workspace.dependencies]
sigil = { git = "https://github.com/unistorm10/sigil.git", branch = "main" }
serde = { version = "1.0", features = ["derive"] }

# Individual crate Cargo.toml
[dependencies]
sigil = { workspace = true }
serde = { workspace = true }
```

## 🔐 Private Repositories

Your platform modules are private repositories. Cargo will use your Git credentials:

- **SSH**: Configure `~/.ssh/config` with GitHub keys
- **HTTPS**: Use Git credential helper or personal access token
- **CI/CD**: Use deploy keys or repository secrets

No special syntax needed in `Cargo.toml` - authentication is handled by Git.

## 📋 Checklist

When adding or reviewing dependencies:

- [ ] All platform modules use Git dependencies (not paths)
- [ ] Dependencies specify branch, tag, or rev
- [ ] Local path overrides use `[patch]` if needed
- [ ] Workspace dependencies defined in root if multi-crate
- [ ] No commented-out local path dependencies
- [ ] Dependencies are from `github.com/unistorm10/*`

---

**Remember**: Git dependencies make your modules portable and CI-friendly. Use `[patch]` for local development, not path dependencies in `Cargo.toml`.
