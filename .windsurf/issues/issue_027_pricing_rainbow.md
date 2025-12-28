# Issue #27: Implement Rainbow option pricing

## Title
`feat: Implement Rainbow option pricing model`

## Labels
- `enhancement`
- `pricing`
- `priority-low`

## Description

Implement pricing support for Rainbow options. Rainbow options are multi-asset options whose payoff depends on the performance of two or more underlying assets.

### Current State
- Rainbow options return `PricingError::UnsupportedOptionType`
- No pricing model for multi-asset options

### Target State
- Fully functional Rainbow option pricing
- Support for common rainbow variants
- Proper correlation handling between assets

## Tasks

- [ ] Implement best-of options (option on maximum)
- [ ] Implement worst-of options (option on minimum)
- [ ] Implement spread options (difference between assets)
- [ ] Handle correlation between underlying assets
- [ ] Implement Greeks calculations
- [ ] Add comprehensive tests
- [ ] Add documentation with examples
- [ ] Run `make lint-fix` and `make pre-push` to verify

## Acceptance Criteria

- [ ] Rainbow options are priced correctly
- [ ] Multiple underlying assets are handled properly
- [ ] Tests validate against known benchmark values
- [ ] Documentation explains the pricing methodology

## Technical Notes

### Rainbow Option Types

**Best-of Options**
- Call on max: payoff = max(max(S1, S2, ...) - K, 0)
- Put on max: payoff = max(K - max(S1, S2, ...), 0)

**Worst-of Options**
- Call on min: payoff = max(min(S1, S2, ...) - K, 0)
- Put on min: payoff = max(K - min(S1, S2, ...), 0)

### Pricing Formula (Two Assets - Stulz 1982)

```
Call on max(S1, S2) = S1*e^(-q1*T)*M(y1, d; -ρ1)
                    + S2*e^(-q2*T)*M(y2, -d+σ√T; -ρ2)
                    - K*e^(-rT)*[1 - M(-y1+σ1√T, -y2+σ2√T; ρ)]

where M() is bivariate normal CDF
σ = √(σ1² + σ2² - 2ρσ1σ2)
```

### Correlation Considerations
- Correlation significantly impacts rainbow option prices
- Higher correlation → lower best-of value, higher worst-of value
- Need robust correlation estimation

### Files to Update
- `src/pricing/black_scholes_model.rs`
- `src/pricing/rainbow.rs` (new module)
- `tests/unit/pricing/rainbow_test.rs` (new)

## Estimated Effort

**High (10-14 hours)**

## Dependencies

- Requires bivariate/multivariate normal distribution

## Related Issues

- Issue #214: Resolve TODOs in black_scholes_model.rs (completed)
