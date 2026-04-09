# Migration Plan: Replace `bincode` with `postcard`

## Current State

`bincode` 1.3.3 is flagged as unmaintained (RUSTSEC-2025-0141). It enters our dependency tree as a **transitive dependency**:

```
rustchat
└── webrtc v0.12.0
    └── webrtc-dtls v0.11.0
        └── bincode v1.3.3
```

There is **zero direct usage** of `bincode` in the `rustchat` source code. The only usage is inside `webrtc-dtls/src/state.rs`:
- `bincode::serialize(&serialized)` — line 235
- `bincode::deserialize(data)` — line 243

Because this is a transitive dependency, we cannot eliminate `bincode` by changing our own code. We must **patch the upstream crate** (`webrtc-dtls`) or wait for an upstream release.

---

## Migration Path: Fork + Patch `webrtc-dtls`

### Why this is the only viable short-term path

- `webrtc-dtls` owns the `bincode` dependency.
- The `webrtc` crate pulls `webrtc-dtls` via crates.io.
- Cargo `[patch.crates-io]` lets us substitute our fork without touching `webrtc`'s source.

### Why `postcard` is the right replacement

| Attribute | `bincode` 1.x | `postcard` |
|-----------|---------------|------------|
| Maintenance | Unmaintained | Actively maintained |
| `no_std` | No | Yes |
| API surface | `serialize` / `deserialize` | `to_stdvec` / `from_bytes` |
| serde support | Yes | Yes |
| Wire compatibility | No guarantees | Deterministic, compact |
| Migration effort | N/A (same serde traits) | ~2 lines + Cargo.toml change |

`postcard` is the de-facto community replacement for `bincode` in the Rust embedded/networking ecosystem.

---

## Concrete Changes Required

### Step 1: Fork `webrtc-dtls`

Create a fork of `https://github.com/webrtc-rs/dtls` at tag `v0.11.0`.

**Files to change (exactly 2):**

#### 1. `Cargo.toml` in the fork root

```toml
[dependencies]
# REMOVE:
# [dependencies.bincode]
# version = "1"

# ADD:
postcard = { version = "1", features = ["use-std"] }
```

#### 2. `src/state.rs` in the fork

Replace the two `bincode` call sites:

```rust
// OLD:
pub async fn marshal_binary(&self) -> Result<Vec<u8>> {
    let serialized = self.serialize().await?;
    match bincode::serialize(&serialized) {
        Ok(enc) => Ok(enc),
        Err(err) => Err(Error::Other(err.to_string())),
    }
}

// NEW:
pub async fn marshal_binary(&self) -> Result<Vec<u8>> {
    let serialized = self.serialize().await?;
    match postcard::to_stdvec(&serialized) {
        Ok(enc) => Ok(enc),
        Err(err) => Err(Error::Other(err.to_string())),
    }
}
```

```rust
// OLD:
pub async fn unmarshal_binary(&mut self, data: &[u8]) -> Result<()> {
    let serialized: SerializedState = match bincode::deserialize(data) {
        Ok(dec) => dec,
        Err(err) => return Err(Error::Other(err.to_string())),
    };
    self.deserialize(&serialized).await?;
    self.init_cipher_suite().await?;
    Ok(())
}

// NEW:
pub async fn unmarshal_binary(&mut self, data: &[u8]) -> Result<()> {
    let serialized: SerializedState = match postcard::from_bytes(data) {
        Ok(dec) => dec,
        Err(err) => return Err(Error::Other(err.to_string())),
    };
    self.deserialize(&serialized).await?;
    self.init_cipher_suite().await?;
    Ok(())
}
```

**Risk assessment:** Very low. The `SerializedState` struct already derives `Serialize` + `Deserialize`. `postcard` is serde-compatible. The two formats are **not** binary-compatible, but `marshal_binary` / `unmarshal_binary` are used for in-memory / in-flight serialization of DTLS state, not persistent storage. Any state persisted to disk with the old `bincode` format would become unreadable after the swap, but `webrtc-dtls` does not appear to do long-term persistence of this blob.

### Step 2: Host the fork

Options (in order of preference):

| Option | Effort | Notes |
|--------|--------|-------|
| **Git submodule in `rustchat` repo** | Medium | Fork lives under `third_party/webrtc-dtls`. No external dependency on your GitHub account staying alive. |
| **Personal/org GitHub fork** | Low | Standard open-source pattern. Add a `README` explaining the patch. |
| **Local path patch** | Lowest | Point `[patch]` to a local directory. Breaks CI unless the directory is committed. |

**Recommendation:** Create a GitHub fork `your-org/webrtc-dtls-postcard` from `webrtc-rs/dtls` at the `v0.11.0` tag, apply the patch, and tag it `v0.11.0-postcard.1`.

### Step 3: Patch `rustchat/backend/Cargo.toml`

Add at the bottom of `backend/Cargo.toml`:

```toml
[patch.crates-io]
webrtc-dtls = { git = "https://github.com/your-org/webrtc-dtls-postcard", tag = "v0.11.0-postcard.1" }
```

Then run:

```bash
cd backend
cargo update -p webrtc-dtls
cargo check
```

If `cargo check` passes and `cargo tree -i bincode:1.3.3` returns nothing, the migration is successful.

---

## Compatibility & Risk Analysis

### Wire format incompatibility
`postcard` and `bincode` produce different byte sequences. If `webrtc-dtls` stores `marshal_binary` output to Redis/DB and later calls `unmarshal_binary`, those stored blobs would fail to deserialize.

**Investigation result:** `webrtc-dtls` uses `marshal_binary` / `unmarshal_binary` for:
- Cloning the internal `State` struct (`state.clone()`).
- Potential use by the higher-level `webrtc` crate for connection serialization/resumption.

There is **no evidence** in `webrtc-dtls` v0.11.0 of long-term disk persistence. The primary consumer appears to be `State::clone()`. This means the format change is safe at runtime because the same process that serializes immediately deserializes.

**Caveat:** If `webrtc` (the parent crate) uses these bytes for session resumption across restarts, a format change could break resumption. We should test the calls feature after the swap.

### API differences between `bincode` and `postcard`

| `bincode` | `postcard` | Notes |
|-----------|------------|-------|
| `bincode::serialize` | `postcard::to_stdvec` | Both return `Vec<u8>` |
| `bincode::deserialize` | `postcard::from_bytes` | Takes `&[u8]`, returns `T` |
| Error type | `postcard::Error` | Both implement `Display` |

No changes to `SerializedState` are needed. It already derives `Serialize` and `Deserialize`.

---

## Verification Plan

1. **Dependency check:** `cargo tree -i bincode:1.3.3` should return nothing.
2. **Compile check:** `cargo check --all-targets` passes.
3. **Clippy check:** `cargo clippy --all-targets --all-features -- -D warnings` passes.
4. **Runtime test:** Start the backend and initiate a RustChat call. The DTLS handshake (`webrtc-dtls`) must succeed for audio/video to work.
5. **Cargo audit:** `cargo audit` no longer reports RUSTSEC-2025-0141.

---

## Estimated Effort

| Task | Human effort | CC effort |
|------|--------------|-----------|
| Fork + patch `webrtc-dtls` | 15 min | 5 min |
| Update `rustchat/Cargo.toml` | 2 min | 1 min |
| Verify compilation + audit | 10 min | 5 min |
| Manual call test | 10 min | — |
| **Total** | **~40 min** | **~15 min** |

---

## Recommendation

**Proceed with the fork-and-patch approach.** The change is surgical (2 lines of code in one upstream file) and the risk is low because `postcard` is a mature, serde-compatible replacement. The alternative (waiting for upstream) has no guaranteed timeline; the `webrtc-rs` ecosystem has been slow to release updates for transitive dependency churn.

If you approve this plan, I can:
1. Create the forked `webrtc-dtls` repository locally or on GitHub.
2. Apply the `postcard` patch.
3. Update `rustchat/backend/Cargo.toml` with the `[patch.crates-io]` directive.
4. Run verification (`cargo check`, `cargo audit`).

Do you want me to proceed?
