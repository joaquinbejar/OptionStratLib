# Issue #1: Reduce `.unwrap()` usage in `chains/chain.rs`

## Title
`refactor: Replace unwrap() calls with proper error handling in chains/chain.rs`

## Labels
- `refactor`
- `error-handling`
- `priority-high`

## Description

The file `src/chains/chain.rs` contains **257 occurrences** of `.unwrap()` which can cause panics in production. This issue focuses on replacing these with proper error handling.

### Current State
- 257 `.unwrap()` calls in production code
- Potential for runtime panics when invalid data is encountered
- Poor error messages for debugging

### Target State
- Zero `.unwrap()` calls in production code (only in tests)
- Proper `Result` types with meaningful error messages
- Graceful error handling for edge cases

## Tasks

- [ ] Audit all `.unwrap()` calls in `chains/chain.rs`
- [ ] Categorize calls by:
  - Can be replaced with `?` operator
  - Can use `unwrap_or_default()`
  - Can use `unwrap_or_else()`
  - Requires new error variant in `ChainError`
- [ ] Add new error variants to `error/chains.rs` if needed
- [ ] Replace `.unwrap()` calls systematically
- [ ] Update function signatures to return `Result` where needed
- [ ] Update tests to verify error handling behavior
- [ ] Run `make lint-fix` and `make pre-push` to verify

## Acceptance Criteria

- [ ] No `.unwrap()` calls remain in production code (only in tests)
- [ ] All existing tests pass
- [ ] New error cases are properly documented
- [ ] Error messages are clear and actionable
- [ ] No performance regression

## Technical Notes

### Common Patterns to Apply

```rust
// Before
let value = some_option.unwrap();

// After - Option 1: Propagate error
let value = some_option.ok_or(ChainError::MissingValue)?;

// After - Option 2: Default value
let value = some_option.unwrap_or_default();

// After - Option 3: Computed default
let value = some_option.unwrap_or_else(|| compute_default());
```

### Files to Update
- `src/chains/chain.rs` (primary)
- `src/error/chains.rs` (add error variants)
- Related test files

## Estimated Effort

**Medium (4-6 hours)**

## Dependencies

None

## Related Issues

- Issue #2: Reduce unwrap() in greeks/equations.rs
- Issue #3: Reduce unwrap() in model/option.rs
