# Issue #28: Implement Spread option pricing

## Title
`feat: Implement Spread option pricing model`

## Labels
- `enhancement`
- `pricing`
- `priority-medium`

## Description

Implement pricing support for Spread options. Spread options have payoffs based on the difference between two underlying asset prices.

### Current State
- Spread options return `PricingError::UnsupportedOptionType`
- No pricing model for price differentials

### Target State
- Fully functional Spread option pricing
- Support for various spread types
- Proper correlation handling

## Tasks

- [ ] Implement basic spread option pricing (Kirk's approximation)
- [ ] Implement calendar spreads
- [ ] Implement crack spreads (commodity specific)
- [ ] Handle correlation between underlying assets
- [ ] Implement Greeks calculations
- [ ] Add comprehensive tests
- [ ] Add documentation with examples
- [ ] Run `make lint-fix` and `make pre-push` to verify

## Acceptance Criteria

- [ ] Spread options are priced correctly
- [ ] Correlation is handled properly
- [ ] Tests validate against known benchmark values
- [ ] Documentation explains the pricing methodology

## Technical Notes

### Spread Option Payoff

```
Call: payoff = max(S1 - S2 - K, 0)
Put: payoff = max(K - (S1 - S2), 0)
```

### Pricing Methods

**Kirk's Approximation (Most Common)**
```
Treat as option on S1 with adjusted strike (S2 + K)
Adjusted volatility: σ_adj = √(σ1² + (σ2*S2/(S2+K))² - 2ρσ1σ2*S2/(S2+K))
```

**Margrabe's Formula (K = 0)**
```
Exchange option: C = S1*e^(-q1*T)*N(d1) - S2*e^(-q2*T)*N(d2)
where σ = √(σ1² + σ2² - 2ρσ1σ2)
```

### Common Applications
- Energy markets (crack spreads, spark spreads)
- Agricultural markets (crush spreads)
- Interest rate markets (yield curve spreads)

### Files to Update
- `src/pricing/black_scholes_model.rs`
- `src/pricing/spread.rs` (new module)
- `tests/unit/pricing/spread_test.rs` (new)

## Estimated Effort

**Medium (6-8 hours)**

## Dependencies

None

## Related Issues

- Issue #214: Resolve TODOs in black_scholes_model.rs (completed)
- Issue #27: Rainbow option pricing (related multi-asset)
