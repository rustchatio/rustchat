# Membership Policies Implementation Review

## Summary

A thorough review of the membership policies feature identified **several critical and medium-priority issues** that should be addressed before production use.

---

## 🔴 Critical Issues

### 1. **Race Condition in `create_policy` - Permission Check After Body Read**
**File:** `backend/src/api/admin_membership_policies.rs:69-99`

The permission check happens **after** the request body is read and deserialized. While this is now necessary due to our debug logging, the proper order should be:
1. Check permissions first
2. Then read/validate the body

**Current flow:**
```rust
async fn create_policy(
    State(state): State<AppState>,
    auth: AuthUser,
    req: axum::extract::Request,  // Body read happens here
) -> ApiResult<Json<PolicyWithTargets>> {
    // ... body read happens before permission check ...
    
    // Permission check happens AFTER body processing
    if !auth.has_permission(&permissions::TEAM_MANAGE) {
        return Err(...);
    }
}
```

**Impact:** An attacker could potentially exploit resource exhaustion by sending large payloads to an endpoint they don't have permission to use.

**Fix:** Restructure to check permissions before processing the body.

---

### 2. **Inconsistent Permission Checks Across Endpoints**
**Files:** Various in `backend/src/api/admin_membership_policies.rs`

- `list_policies`: No permission check (only requires authentication)
- `get_policy`: No permission check (only requires authentication)
- `get_policy_audit`: No permission check (only requires authentication)
- `get_policy_status`: No permission check (only requires authentication)

**Impact:** Information disclosure - any authenticated user can enumerate all policies and view audit logs.

**Fix:** Add appropriate permission checks to all endpoints.

---

### 3. **No Validation That Target Teams/Channels Exist**
**File:** `backend/src/services/membership_policies.rs:176-222`

When creating/updating a policy, there's no validation that:
- The `team_id` (for team-scoped policies) exists
- The target teams exist
- The target channels exist

**Impact:** Policies can be created with invalid targets, causing failures during policy application.

**Fix:** Add existence checks before inserting targets.

---

### 4. **SQL Injection Risk in `list_policies` Dynamic Query Building**
**File:** `backend/src/services/membership_policies.rs:247-298`

The query building uses string concatenation for parameter numbers:
```rust
query.push_str(&format!(" AND team_id = ${}",
    if scope_type.is_some() { 2 } else { 1 }
));
```

While this particular instance is safe (parameter numbers are calculated, not user input), the pattern is risky.

**Impact:** Low immediate risk, but dangerous pattern that could be copied elsewhere.

**Fix:** Use a proper query builder pattern or the `sqlx::QueryBuilder`.

---

### 5. **Missing Transaction Rollback on Error**
**File:** `backend/src/services/membership_policies.rs:176-222`

In `create_policy`, if target insertion fails after the policy is created, the transaction is not properly rolled back (though `?` operator should handle this). However, we should be explicit:

```rust
let mut tx = self.db.begin().await?;
// ... operations ...
tx.commit().await?;  // What if this fails?
```

**Impact:** Potential database inconsistency.

**Fix:** Ensure proper error handling with explicit rollback or verify sqlx's behavior.

---

## 🟡 Medium Priority Issues

### 6. **No Duplicate Target Validation**
**File:** `backend/src/services/membership_policies.rs:199-217`

The code doesn't validate that the targets array doesn't contain duplicates before insertion. The database has a unique constraint, but the error will be a database error, not a user-friendly validation error.

**Fix:** Check for duplicates in the request and return a clear validation error.

---

### 7. **Empty Targets Array Allowed**
**File:** `backend/src/services/membership_policies.rs:117-130`

`CreatePolicyRequest.targets` is a `Vec<CreatePolicyTarget>` with no minimum length validation. A policy with no targets is essentially useless.

**Frontend has validation, but backend should also enforce this.**

**Fix:** Add validation: `if req.targets.is_empty() { return Err(...) }`

---

### 8. **Channel Team Validation Missing for Global Policies**
**File:** `backend/src/services/membership_policies.rs:611-623`

When applying policies, channel targets are validated to belong to the correct team. However, this validation could fail silently if the channel doesn't exist or doesn't belong to the team.

**Fix:** Add explicit error handling and logging.

---

### 9. **Inconsistent Error Handling for `apply_auto_membership_for_team_join`**
**File:** `backend/src/services/team_membership.rs:179-206`

The function is called but errors are only logged, not returned to the user. This means a user could join a team but not get the expected channel memberships, with no indication of the failure.

---

### 10. **Missing Audit Log Entry for Manual Resync**
**File:** `backend/src/api/admin_membership_policies.rs:214-263`

The `trigger_user_resync` endpoint performs actions but doesn't create an audit log entry for who triggered the resync.

**Fix:** Add audit logging.

---

### 11. **No Rate Limiting on Policy Application**
**File:** `backend/src/api/admin_membership_policies.rs`

The manual resync endpoint could be abused to trigger expensive operations repeatedly.

**Fix:** Add rate limiting.

---

### 12. **Policy Name Uniqueness Not Enforced at Application Level**
**File:** `backend/src/services/membership_policies.rs:176-222`

The database has a unique index, but there's no application-level check with a user-friendly error message.

---

## 🟢 Minor Issues

### 13. **Inconsistent Naming: `auth_service` vs `auth_provider`**
**Files:** Multiple

The frontend uses `sourceConfig.auth_service` but the backend also looks for `auth_provider`. This is handled in the code but is confusing.

**Recommendation:** Standardize on one name.

---

### 14. **Missing `Serialize` on `CreatePolicyRequest`**
**File:** `backend/src/services/membership_policies.rs:118`

`CreatePolicyRequest` only derives `Deserialize`, not `Serialize`. This makes debugging/logging harder.

---

### 15. **No Pagination on `list_policies`**
**File:** `backend/src/services/membership_policies.rs:247-298`

If there are many policies, this could return a large result set.

---

### 16. **Frontend: Missing Error Handling for Missing Targets**
**File:** `frontend/src/components/admin/PolicyEditorModal.vue:186-191`

The validation is good, but there's no handling for the case where the selected team/channel was deleted between loading the options and saving.

---

### 17. **Reconciliation Worker: No Timeout on Individual Operations**
**File:** `backend/src/services/membership_reconciliation.rs`

The reconciliation worker processes tasks without timeouts, which could lead to stuck workers.

---

### 18. **Missing Index on `auto_membership_policy_audit.run_id`**
**File:** `backend/migrations/20260228140000_auto_membership_policies.sql:79`

There's an index on `run_id`, but the query in `get_policy_last_run_status` might benefit from a composite index on `(policy_id, run_id, created_at)`.

---

## Recommended Priority Fixes

### Immediate (Before Production)
1. Fix permission checks on all endpoints
2. Add validation for target existence
3. Add validation for non-empty targets array
4. Fix race condition in permission check order

### Short Term
5. Add duplicate target validation
6. Add audit logging for manual resync
7. Add rate limiting
8. Improve error messages for unique constraint violations

### Long Term
9. Add pagination to list endpoints
10. Add timeouts to background workers
11. Standardize naming conventions
