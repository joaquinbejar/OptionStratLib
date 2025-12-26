# Issue #17: Reduce `.unwrap()` usage in remaining `src/model` files

## Title
`refactor: Replace unwrap() calls with proper error handling in src/model (excluding option.rs and positive.rs)`

## Labels
- `refactor`
- `error-handling`
- `priority-high`

## Description

The `src/model` directory contains **440 occurrences** of `.unwrap()` across 18 files. This issue focuses on the files **not already covered** by other issues:

- **Issue #3** covers `model/option.rs` (134 occurrences)
- **Issue #16** covers `model/positive.rs` (50 occurrences)

This issue covers the remaining **256 occurrences** across **16 files**.

### Current State

| File | `.unwrap()` Count |
|------|-------------------|
| `position.rs` | 116 |
| `expiration.rs` | 72 |
| `trade.rs` | 14 |
| `decimal.rs` | 12 |
| `profit_range.rs` | 9 |
| `utils.rs` | 7 |
| `leg/leg_enum.rs` | 5 |
| `leg/spot.rs` | 5 |
| `axis.rs` | 4 |
| `format.rs` | 2 |
| `leg/future.rs` | 2 |
| `leg/mod.rs` | 2 |
| `leg/perpetual.rs` | 2 |
| `leg/traits.rs` | 2 |
| `balance.rs` | 1 |
| `types.rs` | 1 |
| **Total** | **256** |

### Target State
- Zero `.unwrap()` calls in production code (only in tests)
- Proper `Result` types with meaningful error messages
- Graceful error handling for edge cases

## Tasks

### Phase 1: High-Impact Files (116 + 72 = 188 occurrences)

#### `position.rs` (116 occurrences)
- [ ] Audit all `.unwrap()` calls in `position.rs`
- [ ] Replace with `?` operator where function returns `Result`
- [ ] Use `unwrap_or_default()` or `unwrap_or_else()` where appropriate
- [ ] Add new error variants to `PositionError` if needed

#### `expiration.rs` (72 occurrences)
- [ ] Audit all `.unwrap()` calls in `expiration.rs`
- [ ] Handle date/time parsing errors gracefully
- [ ] Add proper error types for expiration date operations

### Phase 2: Medium-Impact Files (14 + 12 + 9 + 7 = 42 occurrences)

#### `trade.rs` (14 occurrences)
- [ ] Audit all `.unwrap()` calls in `trade.rs`
- [ ] Replace with proper error handling

#### `decimal.rs` (12 occurrences)
- [ ] Audit all `.unwrap()` calls in `decimal.rs`
- [ ] Handle decimal conversion errors

#### `profit_range.rs` (9 occurrences)
- [ ] Audit all `.unwrap()` calls in `profit_range.rs`
- [ ] Replace with proper error handling

#### `utils.rs` (7 occurrences)
- [ ] Audit all `.unwrap()` calls in `utils.rs`
- [ ] Replace with proper error handling

### Phase 3: Low-Impact Files (26 occurrences)

#### `leg/` submodule (18 occurrences total)
- [ ] `leg/leg_enum.rs` (5 occurrences)
- [ ] `leg/spot.rs` (5 occurrences)
- [ ] `leg/future.rs` (2 occurrences)
- [ ] `leg/mod.rs` (2 occurrences)
- [ ] `leg/perpetual.rs` (2 occurrences)
- [ ] `leg/traits.rs` (2 occurrences)

#### Other files (8 occurrences total)
- [ ] `axis.rs` (4 occurrences)
- [ ] `format.rs` (2 occurrences)
- [ ] `balance.rs` (1 occurrence)
- [ ] `types.rs` (1 occurrence)

### Phase 4: Verification
- [ ] Run `make lint-fix` and `make pre-push` to verify
- [ ] Ensure all existing tests pass
- [ ] Add tests for new error handling paths

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
let value = some_option.ok_or(ModelError::MissingValue)?;

// After - Option 2: Default value
let value = some_option.unwrap_or_default();

// After - Option 3: Computed default
let value = some_option.unwrap_or_else(|| compute_default());

// After - Option 4: Log and default
let value = some_option.unwrap_or_else(|| {
    tracing::warn!("Value missing, using default");
    default_value()
});
```

### Date/Time Handling in `expiration.rs`

```rust
// Before
let date = NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap();

// After
let date = NaiveDate::parse_from_str(s, "%Y-%m-%d")
    .map_err(|e| ExpirationError::ParseError {
        input: s.to_string(),
        reason: e.to_string(),
    })?;
```

### Position Calculations

```rust
// Before
let delta = position.option.delta().unwrap();

// After
let delta = position.option.delta()
    .map_err(|e| PositionError::GreeksCalculationFailed(e))?;
```

## Files to Update

### Primary Files
- `src/model/position.rs`
- `src/model/expiration.rs`
- `src/model/trade.rs`
- `src/model/decimal.rs`
- `src/model/profit_range.rs`
- `src/model/utils.rs`

### Leg Submodule
- `src/model/leg/leg_enum.rs`
- `src/model/leg/spot.rs`
- `src/model/leg/future.rs`
- `src/model/leg/mod.rs`
- `src/model/leg/perpetual.rs`
- `src/model/leg/traits.rs`

### Other Files
- `src/model/axis.rs`
- `src/model/format.rs`
- `src/model/balance.rs`
- `src/model/types.rs`

### Error Files (may need updates)
- `src/error/position.rs`
- `src/error/decimal.rs`

## Estimated Effort

**High (10-14 hours)**

- Phase 1 (position.rs + expiration.rs): 6-8 hours
- Phase 2 (trade, decimal, profit_range, utils): 2-3 hours
- Phase 3 (leg submodule + others): 1-2 hours
- Phase 4 (verification): 1 hour

## Dependencies

- Issue #3: Reduce unwrap() in model/option.rs (should be done first or in parallel)
- Issue #16: Improve Positive type safety (should be done first)

## Related Issues

- Issue #1: Reduce unwrap() in chains/chain.rs
- Issue #2: Reduce unwrap() in greeks/equations.rs
- Issue #3: Reduce unwrap() in model/option.rs
- Issue #15: Reduce expect() usage
- Issue #16: Improve Positive type safety and API

## Notes

This issue is part of a broader effort to eliminate panics from the codebase. The `src/model` directory is critical because it contains the core data structures used throughout the library.

### Priority Order Within This Issue

1. **`position.rs`** - Most critical, used everywhere
2. **`expiration.rs`** - Date handling is error-prone
3. **`trade.rs`** - Trading operations should never panic
4. **`decimal.rs`** - Numeric conversions need safety
5. **Remaining files** - Lower impact but still important

---

*Generated on: 2024-12-25*
