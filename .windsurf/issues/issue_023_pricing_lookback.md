# Issue #23: Implement Lookback option pricing

## Title
`feat: Implement Lookback option pricing model`

## Labels
- `enhancement`
- `pricing`
- `priority-medium`

## Description

Implement pricing support for Lookback options. Lookback options have payoffs that depend on the maximum or minimum price of the underlying asset during the option's life.

### Current State
- Lookback options return `PricingError::UnsupportedOptionType`
- No pricing model for path-dependent extrema

### Target State
- Fully functional Lookback option pricing
- Support for fixed and floating strike variants
- Support for both calls and puts

## Tasks

- [ ] Implement floating strike lookback options
- [ ] Implement fixed strike lookback options
- [ ] Handle continuous vs discrete monitoring
- [ ] Implement Greeks calculations
- [ ] Add comprehensive tests
- [ ] Add documentation with examples
- [ ] Run `make lint-fix` and `make pre-push` to verify

## Acceptance Criteria

- [ ] Lookback options are priced correctly
- [ ] Both strike types are supported
- [ ] Tests validate against known benchmark values
- [ ] Documentation explains the pricing methodology

## Technical Notes

### Lookback Option Types

**Floating Strike**
- Call: payoff = S_T - S_min (buy at the lowest price)
- Put: payoff = S_max - S_T (sell at the highest price)

**Fixed Strike**
- Call: payoff = max(S_max - K, 0)
- Put: payoff = max(K - S_min, 0)

### Pricing Formulas (Goldman-Sosin-Gatto)

**Floating Strike Lookback Call**
```
C = S*e^(-qT)*N(a1) - S*e^(-qT)*(σ²/2r)*N(-a1)
    - S_min*e^(-rT)*[N(a2) - (σ²/2r)*e^(Y)*N(-a3)]

where Y = 2*(r-q-σ²/2)*ln(S/S_min)/σ²
```

### Files to Update
- `src/pricing/black_scholes_model.rs`
- `src/pricing/lookback.rs` (new module)
- `tests/unit/pricing/lookback_test.rs` (new)

## Estimated Effort

**High (8-12 hours)**

## Dependencies

None

## Related Issues

- Issue #214: Resolve TODOs in black_scholes_model.rs (completed)
