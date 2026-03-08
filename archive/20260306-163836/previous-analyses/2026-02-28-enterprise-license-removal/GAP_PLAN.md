# GAP_PLAN

## Rustchat target path: `backend/src/config/mod.rs`
- Required behavior: No internal enterprise-license feature toggles controlling availability.
- Current gap: `CompatibilityConfig` still defines `is_licensed` and `ldap_groups_enabled` plus env overrides.
- Planned change: Remove those fields and env parsing; keep only mobile SSO compatibility flag.
- Verification test: `cargo check` and config-related tests compile/pass.
- Status: implemented

## Rustchat target path: `backend/src/api/v4/ldap.rs`
- Required behavior: LDAP stubs must not return enterprise-license-required semantics.
- Current gap: Returns `api.license.enterprise_needed.error` and enterprise wording.
- Planned change: Replace helper with neutral `not implemented` error payload and IDs.
- Verification test: endpoint smoke tests (existing suites) and compile.
- Status: implemented

## Rustchat target path: `backend/src/api/v4/saml.rs`
- Required behavior: SAML stubs must not return enterprise-license-required semantics.
- Current gap: Returns enterprise license messages and metadata XML mentions enterprise license.
- Planned change: Replace with neutral not-configured/not-implemented messages.
- Verification test: compile + relevant API tests.
- Status: implemented

## Rustchat target path: `backend/src/api/v4/system.rs`
- Required behavior: Avoid license-based denial text where feature is stubbed.
- Current gap: support packet path says "require a license".
- Planned change: keep current status behavior but neutralize message to no-license wording.
- Verification test: compile and smoke where applicable.
- Status: implemented

## Rustchat target path: `backend/tests/common/mod.rs`, `backend/tests/api_v4_config.rs`, `backend/tests/security_integration.rs`
- Required behavior: test configs should reflect removed fields.
- Current gap: tests still initialize removed compatibility fields.
- Planned change: update config initializers to only include remaining compatibility fields.
- Verification test: `cargo test --test api_v4_config -- --nocapture` and targeted suites.
- Status: implemented
