# Issue #20: Implement Asian option pricing

## Title
`feat: Implement Asian option pricing model`

## Labels
- `enhancement`
- `pricing`
- `priority-medium`

## Description

Implement pricing support for Asian options. Asian options have payoffs that depend on the average price of the underlying asset over a specified period.

### Current State
- Asian options return `PricingError::UnsupportedOptionType`
- No pricing model for path-dependent averaging

### Target State
- Fully functional Asian option pricing
- Support for arithmetic and geometric averaging
- Support for fixed and floating strike variants

## Tasks

- [ ] Implement geometric average Asian option (closed-form solution)
- [ ] Implement arithmetic average Asian option (Monte Carlo or approximation)
- [ ] Support both fixed strike and floating strike variants
- [ ] Handle discrete vs continuous averaging
- [ ] Implement Greeks calculations
- [ ] Add comprehensive tests
- [ ] Add documentation with examples
- [ ] Run `make lint-fix` and `make pre-push` to verify

## Acceptance Criteria

- [ ] Asian options are priced correctly
- [ ] Both averaging types are supported
- [ ] Tests validate against known benchmark values
- [ ] Documentation explains the pricing methodology

## Technical Notes

### Asian Option Types

**By Average Type:**
- Geometric average (closed-form solution exists)
- Arithmetic average (requires numerical methods)

**By Strike Type:**
- Fixed strike: payoff = max(Average - K, 0)
- Floating strike: payoff = max(S_T - Average, 0)

### Pricing Formulas

**Geometric Average (Closed-form)**
```
Adjusted volatility: σ_adj = σ / √3
Adjusted rate: r_adj = (r - σ²/6) / 2
Then use standard Black-Scholes with adjusted parameters
```

**Arithmetic Average**
- Turnbull-Wakeman approximation
- Monte Carlo simulation
- Moment matching methods

### Files to Update
- `src/pricing/black_scholes_model.rs`
- `src/pricing/asian.rs` (new module)
- `tests/unit/pricing/asian_test.rs` (new)

## Estimated Effort

**High (8-12 hours)**

## Dependencies

None

## Related Issues

- Issue #214: Resolve TODOs in black_scholes_model.rs (completed)
