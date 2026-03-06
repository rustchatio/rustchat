# Architecture Gaps

1. Route module archival was applied to active runtime files.
- Real v4 route implementations were moved to `archive/20260306-163915-remaining-dirty/...`.
- Active files under `backend/src/api/v4/` were replaced with stubs.

2. Compatibility-critical bootstrap surface is now disconnected.
- `mod.rs` still merges `system::router()`, but the module returns `Router::new()`.
- Result: required routes are absent at runtime.

3. This is a regression in code assembly, not a protocol design gap.
- Archived files already contain compatible behavior and should be restored.
