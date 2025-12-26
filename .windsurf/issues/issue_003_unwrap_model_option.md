# Issue #3: Reduce `.unwrap()` usage in `model/option.rs`

## Title
`refactor: Replace unwrap() calls with proper error handling in model/option.rs`

## Labels
- `refactor`
- `error-handling`
- `priority-high`

## Description

The file `src/model/option.rs` contains **134 occurrences** of `.unwrap()`. The `Options` struct is fundamental to the library and should handle errors gracefully.

### Current State
- 134 `.unwrap()` calls in production code
- Core option operations can panic
- Invalid option parameters cause crashes instead of errors

### Target State
- Zero `.unwrap()` calls in production code
- All public methods return `Result` where failure is possible
- Clear error messages for invalid parameters

## Tasks

- [ ] Audit all `.unwrap()` calls in `model/option.rs`
- [ ] Replace with proper error handling using `OptionsError`
- [ ] Ensure all public methods return `Result` where failure is possible
- [ ] Add validation for option parameters:
  - Strike price > 0
  - Underlying price > 0
  - Volatility > 0
  - Time to expiration >= 0
- [ ] Update documentation with error conditions
- [ ] Run `make lint-fix` and `make pre-push` to verify

## Acceptance Criteria

- [ ] No `.unwrap()` calls remain in production code
- [ ] All existing tests pass
- [ ] API documentation includes error conditions
- [ ] Invalid parameters produce clear error messages
- [ ] Option creation validates all inputs

## Technical Notes

### Validation Pattern

```rust
impl Options {
    pub fn new(
        strike: Positive,
        underlying: Positive,
        volatility: Positive,
        // ...
    ) -> Result<Self, OptionsError> {
        // Validation is handled by Positive type
        // Additional business logic validation here
        
        Ok(Self {
            strike,
            underlying,
            volatility,
            // ...
        })
    }
}
```

### Error Handling Pattern

```rust
// Before
let price = self.calculate_price().unwrap();

// After
let price = self.calculate_price()
    .map_err(|e| OptionsError::PricingFailed(e.to_string()))?;
```

### Files to Update
- `src/model/option.rs` (primary)
- `src/error/options.rs` (add error variants)
- Related test files

## Estimated Effort

**Medium (4-6 hours)**

## Dependencies

None

## Related Issues

- Issue #1: Reduce unwrap() in chains/chain.rs
- Issue #2: Reduce unwrap() in greeks/equations.rs
