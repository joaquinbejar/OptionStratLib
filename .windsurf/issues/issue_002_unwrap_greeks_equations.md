# Issue #2: Reduce `.unwrap()` usage in `greeks/equations.rs`

## Title
`refactor: Replace unwrap() calls with proper error handling in greeks/equations.rs`

## Labels
- `refactor`
- `error-handling`
- `priority-high`

## Description

The file `src/greeks/equations.rs` contains **156 occurrences** of `.unwrap()`. Greeks calculations are critical for options pricing and should never panic.

### Current State
- 156 `.unwrap()` calls in production code
- Greeks calculations can panic on edge cases
- Mathematical errors are not properly propagated

### Target State
- Zero `.unwrap()` calls in production code
- All mathematical edge cases handled gracefully
- Proper `GreeksError` types for domain-specific errors

## Tasks

- [ ] Audit all `.unwrap()` calls in `greeks/equations.rs`
- [ ] Identify mathematical edge cases:
  - Division by zero
  - Negative values under square root
  - Overflow/underflow conditions
- [ ] Replace with `?` operator where the function returns `Result`
- [ ] Use `GreeksError` for domain-specific errors
- [ ] Add `#[must_use]` annotations where appropriate
- [ ] Update tests to verify error handling behavior
- [ ] Run `make lint-fix` and `make pre-push` to verify

## Acceptance Criteria

- [ ] No `.unwrap()` calls remain in production code
- [ ] All existing tests pass
- [ ] Error messages are clear and actionable
- [ ] Mathematical edge cases are documented
- [ ] Greeks calculations remain accurate

## Technical Notes

### Common Mathematical Edge Cases

```rust
// Division by zero
let d1 = (ln(S/K) + (r + sigma^2/2) * T) / (sigma * sqrt(T));
// When T = 0 or sigma = 0, this fails

// Square root of negative
let sqrt_t = T.sqrt(); // Fails if T < 0

// Logarithm of non-positive
let ln_ratio = (S / K).ln(); // Fails if S <= 0 or K <= 0
```

### Error Handling Pattern

```rust
// Before
let d1 = calculate_d1(s, k, r, sigma, t).unwrap();

// After
let d1 = calculate_d1(s, k, r, sigma, t)
    .map_err(|e| GreeksError::CalculationFailed {
        greek: "d1",
        reason: e.to_string(),
    })?;
```

### Files to Update
- `src/greeks/equations.rs` (primary)
- `src/error/greeks.rs` (add error variants)
- Related test files

## Estimated Effort

**Medium (3-5 hours)**

## Dependencies

None

## Related Issues

- Issue #1: Reduce unwrap() in chains/chain.rs
- Issue #3: Reduce unwrap() in model/option.rs
