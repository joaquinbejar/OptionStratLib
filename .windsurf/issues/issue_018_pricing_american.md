# Issue #18: Implement American option pricing

## Title
`feat: Implement American option pricing model`

## Labels
- `enhancement`
- `pricing`
- `priority-high`

## Description

Implement pricing support for American options in the Black-Scholes pricing module. American options can be exercised at any time before expiration, which requires different pricing methods than European options.

### Current State
- American options return `PricingError::UnsupportedOptionType`
- No pricing model implemented for early exercise

### Target State
- Fully functional American option pricing
- Support for both calls and puts
- Accurate early exercise premium calculation

## Tasks

- [ ] Research and select appropriate pricing method:
  - Binomial tree (Cox-Ross-Rubinstein)
  - Barone-Adesi and Whaley approximation
  - Bjerksund-Stensland approximation
- [ ] Implement the chosen pricing algorithm
- [ ] Handle early exercise boundary conditions
- [ ] Add dividend handling for American options
- [ ] Implement Greeks calculations for American options
- [ ] Add comprehensive tests with known values
- [ ] Add documentation with examples
- [ ] Run `make lint-fix` and `make pre-push` to verify

## Acceptance Criteria

- [ ] American options are priced correctly
- [ ] Early exercise premium is calculated accurately
- [ ] Tests validate against known benchmark values
- [ ] Documentation explains the pricing methodology
- [ ] Performance is acceptable for real-time use

## Technical Notes

### Pricing Methods

**Binomial Tree (Recommended for accuracy)**
```
Option value at each node = max(exercise value, continuation value)
Continuation value = e^(-r*dt) * [p * V_up + (1-p) * V_down]
```

**Barone-Adesi and Whaley (Recommended for speed)**
- Analytical approximation
- Good balance of speed and accuracy
- Widely used in practice

### Key Differences from European Options

1. **Early Exercise**: American options can be exercised before expiration
2. **Put-Call Parity**: Does not hold for American options
3. **Dividends**: More significant impact due to early exercise

### Files to Update
- `src/pricing/black_scholes_model.rs` (add American pricing)
- `src/pricing/mod.rs` (potentially add new module)
- `tests/unit/pricing/american_test.rs` (new tests)

## Estimated Effort

**High (8-12 hours)**

## Dependencies

None

## Related Issues

- Issue #214: Resolve TODOs in black_scholes_model.rs (completed)
