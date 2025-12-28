# Issue #19: Implement Bermuda option pricing

## Title
`feat: Implement Bermuda option pricing model`

## Labels
- `enhancement`
- `pricing`
- `priority-medium`

## Description

Implement pricing support for Bermuda options. Bermuda options can be exercised on specific dates before expiration, making them a hybrid between European and American options.

### Current State
- Bermuda options return `PricingError::UnsupportedOptionType`
- No pricing model implemented for discrete exercise dates

### Target State
- Fully functional Bermuda option pricing
- Support for arbitrary exercise dates
- Accurate pricing considering exercise opportunities

## Tasks

- [ ] Implement Bermuda option pricing using binomial/trinomial tree
- [ ] Handle discrete exercise dates in the tree structure
- [ ] Support variable number of exercise opportunities
- [ ] Implement Greeks calculations
- [ ] Add comprehensive tests
- [ ] Add documentation with examples
- [ ] Run `make lint-fix` and `make pre-push` to verify

## Acceptance Criteria

- [ ] Bermuda options are priced correctly
- [ ] Multiple exercise dates are handled properly
- [ ] Tests validate against known benchmark values
- [ ] Documentation explains the pricing methodology

## Technical Notes

### Bermuda Option Characteristics

- Exercise allowed only on specific dates (e.g., monthly, quarterly)
- Value is between European and American options
- Common in interest rate derivatives and convertible bonds

### Pricing Approach

**Modified Binomial Tree**
```
1. Build tree with nodes at exercise dates
2. At exercise nodes: value = max(exercise value, continuation value)
3. At non-exercise nodes: value = continuation value only
```

### Files to Update
- `src/pricing/black_scholes_model.rs`
- `src/model/types.rs` (Bermuda variant already exists)
- `tests/unit/pricing/bermuda_test.rs` (new)

## Estimated Effort

**Medium (6-8 hours)**

## Dependencies

- Issue #18: American option pricing (similar tree structure)

## Related Issues

- Issue #18: Implement American option pricing
