# Quick Wins: Rust Build Optimization Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Reduce incremental Rust build times from ~15 minutes to ~3-5 minutes by configuring SQLx offline mode, fast linker, sccache, and optimized Cargo settings.

**Architecture:** Configure build toolchain and caching without code reorganization. SQLx offline mode eliminates DB connections; mold/zld linker speeds up linking; sccache caches compiled artifacts.

**Tech Stack:** Rust, Cargo, SQLx, mold (Linux) / zld (macOS), sccache

---

## File Structure

| File | Purpose |
|------|---------|
| `.cargo/config.toml` | Cargo configuration: linker, rustflags, build settings |
| `backend/.sqlx/query-*.json` | SQLx offline query cache (generated, not hand-written) |
| `scripts/setup-dev.sh` | One-time dev environment setup script |
| `scripts/check-sqlx-cache.sh` | CI script to verify SQLx cache is current |

---

## Prerequisites

Ensure these tools are available:
- `cargo sqlx` CLI (install: `cargo install sqlx-cli`)
- `mold` (Linux) or `zld` (macOS) linker
- `sccache` (install: `cargo install sccache` or package manager)

---

## Task 1: Configure Fast Linker and sccache

**Files:**
- Create: `.cargo/config.toml`
- Modify: `.gitignore`

**Context:** The default system linker is slow. Using mold (Linux) or zld (macOS) can reduce link times by 2-3x. sccache caches compiled artifacts across builds.

- [ ] **Step 1: Create Cargo configuration with fast linker and sccache**

Create `.cargo/config.toml`:

```toml
[build]
# Use sccache for caching
rustc-wrapper = "sccache"

# Use fast linker - mold on Linux, zld on macOS
[target.x86_64-unknown-linux-gnu]
linker = "clang"
rustflags = ["-C", "link-arg=-fuse-ld=mold", "-C", "incremental=true"]

[target.aarch64-unknown-linux-gnu]
linker = "clang"
rustflags = ["-C", "link-arg=-fuse-ld=mold", "-C", "incremental=true"]

[target.x86_64-apple-darwin]
rustflags = ["-C", "link-arg=-fuse-ld=zld", "-C", "incremental=true"]

[target.aarch64-apple-darwin]
rustflags = ["-C", "link-arg=-fuse-ld=zld", "-C", "incremental=true"]

[env]
# sccache configuration (adjust as needed for your environment)
SCCACHE_CACHE_SIZE = "10G"
# Cache stored in workspace root - adjust path or use absolute path if needed
SCCACHE_DIR = ".sccache"

[profile.dev]
# Optimize dependencies even in dev mode for faster builds
opt-level = 0
# Use less aggressive debug info
debug = 1

[profile.dev.package."*"]
# Build dependencies with some optimization for faster overall builds
opt-level = 1
```

- [ ] **Step 2: Add .sccache to .gitignore**

Append to `.gitignore`:

```
# sccache directory
.sccache/
```

- [ ] **Step 3: Verify sccache is installed**

Run: `which sccache && sccache --version`

Expected: Path to sccache and version number

- [ ] **Step 4: Verify configuration syntax**

Run: `cargo check --manifest-path backend/Cargo.toml`

Expected: Should pass without linker errors (may fail later on SQLx if no DB, that's OK)

- [ ] **Step 5: Commit**

```bash
git add .cargo/config.toml .gitignore
git commit -m "build: configure fast linker (mold/zld), sccache, and incremental compilation"
```

---

## Task 2: Setup SQLx Offline Mode

**Files:**
- Create: `backend/.sqlx/query-*.json` (multiple files, generated)
- Modify: `backend/Cargo.toml`

**Context:** SQLx validates queries against a live database at compile time. Offline mode uses pre-generated JSON cache files instead.

- [ ] **Step 1: Verify sqlx-cli is installed**

Run: `cargo sqlx --version`

Expected: Version number like `0.8.2`

If not installed: `cargo install sqlx-cli --features native-tls`

- [ ] **Step 2: Update backend Cargo.toml to enable SQLx offline feature**

Modify `backend/Cargo.toml` around line 22:

```toml
# Database
sqlx = { version = "0.8", features = ["runtime-tokio", "postgres", "uuid", "chrono", "json", "offline"] }
```

Add `"offline"` to the existing features list.

- [ ] **Step 3: Ensure DATABASE_URL is set**

Run: `echo $DATABASE_URL`

Expected: A PostgreSQL connection string like `postgres://user:pass@localhost/rustchat`

If not set, check `.env` file and source it: `source .env`

- [ ] **Step 4: Generate SQLx offline query cache**

Run:
```bash
cd backend
cargo sqlx prepare -- --all-targets
```

Expected: Success message, creates `backend/.sqlx/` directory with `query-*.json` files

- [ ] **Step 5: Verify offline cache was created**

Run: `ls -la backend/.sqlx/ | head -10`

Expected: Multiple `query-*.json` files (typically 50-100+ files)

- [ ] **Step 6: Test build without database connection**

Run:
```bash
cd backend
# Temporarily unset DATABASE_URL to simulate offline build
unset DATABASE_URL
cargo check
```

Expected: Build succeeds without database connection

- [ ] **Step 7: Commit**

```bash
git add backend/Cargo.toml backend/.sqlx/
git commit -m "build: enable SQLx offline mode for faster builds

Add offline feature to sqlx dependency and generate query cache.
Builds no longer require database connection."
```

---

## Task 3: Add SQLx Prepare Check Script

**Files:**
- Create: `scripts/check-sqlx-cache.sh`

**Context:** The offline cache can become stale when queries change. This script verifies cache is up-to-date.

- [ ] **Step 1: Create SQLx cache check script**

Create `scripts/check-sqlx-cache.sh`:

```bash
#!/bin/bash
# Check that SQLx offline cache is up-to-date
# Run this in CI or before committing query changes

set -e

cd "$(dirname "$0")/../backend"

# Check if sqlx-cli is installed by testing cargo sqlx
if ! cargo sqlx --version &> /dev/null; then
    echo "Error: sqlx-cli not installed. Run: cargo install sqlx-cli"
    exit 1
fi

# Verify DATABASE_URL is set
if [ -z "$DATABASE_URL" ]; then
    echo "Error: DATABASE_URL not set"
    exit 1
fi

# Check cache is up-to-date
cargo sqlx prepare --check -- --all-targets

echo "SQLx cache is up-to-date!"
```

- [ ] **Step 2: Make script executable**

Run: `chmod +x scripts/check-sqlx-cache.sh`

- [ ] **Step 3: Test the script**

Run: `source .env && ./scripts/check-sqlx-cache.sh`

Expected: "SQLx cache is up-to-date!" (assuming cache is fresh)

- [ ] **Step 4: Commit**

```bash
git add scripts/check-sqlx-cache.sh
git commit -m "build: add script to verify SQLx cache freshness"
```

---

## Task 4: Create Development Setup Script

**Files:**
- Create: `scripts/setup-dev.sh`

**Context:** Document and automate the one-time dev environment setup.

- [ ] **Step 1: Create setup script**

Create `scripts/setup-dev.sh`:

```bash
#!/bin/bash
# One-time development environment setup for fast builds

set -e

echo "=== Setting up Rust development environment ==="

# Install sqlx-cli if not present
if ! cargo sqlx --version &> /dev/null; then
    echo "Installing sqlx-cli..."
    cargo install sqlx-cli --features native-tls
else
    echo "sqlx-cli already installed"
fi

# Install sccache if not present
if ! command -v sccache &> /dev/null; then
    echo "Installing sccache..."
    cargo install sccache
else
    echo "sccache already installed"
fi

# Platform-specific linker setup
if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    if ! command -v mold &> /dev/null; then
        echo "Installing mold linker..."
        # Debian/Ubuntu
        if command -v apt-get &> /dev/null; then
            sudo apt-get update && sudo apt-get install -y mold
        # Fedora
        elif command -v dnf &> /dev/null; then
            sudo dnf install -y mold
        else
            echo "Please install mold manually: https://github.com/rui314/mold"
        fi
    fi
elif [[ "$OSTYPE" == "darwin"* ]]; then
    # Note: zld is no longer actively maintained. Consider using lld via:
    # brew install llvm && export RUSTFLAGS="-C link-arg=-fuse-ld=lld"
    if ! command -v zld &> /dev/null; then
        echo "Installing zld linker..."
        if command -v brew &> /dev/null; then
            brew install michaeleisel/zld/zld
        else
            echo "Please install Homebrew first, then run: brew install michaeleisel/zld/zld"
            echo "Alternatively, consider using lld: brew install llvm"
        fi
    fi
fi

# Generate SQLx cache if DATABASE_URL is set
if [ -n "$DATABASE_URL" ]; then
    echo "Generating SQLx offline cache..."
    cd backend
    cargo sqlx prepare -- --all-targets
else
    echo "Warning: DATABASE_URL not set. Run 'source .env' and rerun to generate SQLx cache."
fi

echo ""
echo "=== Setup complete! ==="
echo "Next steps:"
echo "  1. Ensure .env file has DATABASE_URL set"
echo "  2. Run: source .env"
echo "  3. Run: cargo build (should be much faster!)"
```

- [ ] **Step 2: Make script executable**

Run: `chmod +x scripts/setup-dev.sh`

- [ ] **Step 3: Commit**

```bash
git add scripts/setup-dev.sh
git commit -m "build: add development environment setup script"
```

---

## Task 5: Verify and Benchmark

**Files:** None (verification only)

- [ ] **Step 1: Clean build artifacts**

Run: `cd backend && cargo clean`

- [ ] **Step 2: Run initial build to populate caches**

Run:
```bash
cd backend
time cargo build
```

Expected: Build completes (may still take ~15 min for first build). Note the actual time.

- [ ] **Step 3: Make a small change and rebuild**

Run:
```bash
# Add a comment to any .rs file
echo "// test" >> backend/src/api/health.rs
# Time the rebuild
time cargo build
```

Expected: Build completes significantly faster than before. Note the actual time.

- [ ] **Step 4: Revert test change**

Run: `git checkout backend/src/api/health.rs`

- [ ] **Step 5: Record results**

Document the measured times (optional - add to README.md or keep notes):

| Build Type | Before | After |
|------------|--------|-------|
| Clean build | ~15 min | [measured in step 2] |
| Incremental (1 file change) | ~15 min | [measured in step 3] |

---

## Verification Checklist

After all tasks complete, verify:

- [ ] `.cargo/config.toml` exists with linker + sccache config
- [ ] `backend/.sqlx/` directory has query cache files
- [ ] `backend/Cargo.toml` has `"offline"` feature for sqlx
- [ ] `scripts/setup-dev.sh` exists and is executable
- [ ] `scripts/check-sqlx-cache.sh` exists and is executable
- [ ] Incremental builds take significantly less time than before
- [ ] Measured build times are documented (optional)

---

## Rollback Instructions

If issues arise, these changes can be reverted:

1. **Disable sccache**: Remove `rustc-wrapper` from `.cargo/config.toml`
2. **Disable fast linker**: Remove rustflags from `.cargo/config.toml`
3. **Disable SQLx offline**: Remove `"offline"` feature from `backend/Cargo.toml`
4. **Full reset**: `git checkout main -- .cargo/ backend/Cargo.toml backend/.sqlx/`

---

## Notes for Future Maintainers

- **SQLx cache must be regenerated** when queries change: `cargo sqlx prepare`
- **Cache staleness check**: Run `scripts/check-sqlx-cache.sh` in CI
- **sccache stats**: Run `sccache --show-stats` to see cache hit rates
- **macOS linker**: zld is no longer actively maintained. Consider migrating to lld via `brew install llvm`
