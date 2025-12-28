# Issue #29: Implement Quanto option pricing

## Title
`feat: Implement Quanto option pricing model`

## Labels
- `enhancement`
- `pricing`
- `priority-low`

## Description

Implement pricing support for Quanto options. Quanto options are derivatives where the underlying is denominated in one currency but the payoff is settled in another currency at a fixed exchange rate.

### Current State
- Quanto options return `PricingError::UnsupportedOptionType`
- No pricing model for currency-protected options

### Target State
- Fully functional Quanto option pricing
- Proper handling of currency correlation
- Support for various quanto structures

## Tasks

- [ ] Implement basic quanto option pricing
- [ ] Handle correlation between asset and exchange rate
- [ ] Support quanto forwards and futures
- [ ] Implement Greeks calculations (including FX sensitivity)
- [ ] Add comprehensive tests
- [ ] Add documentation with examples
- [ ] Run `make lint-fix` and `make pre-push` to verify

## Acceptance Criteria

- [ ] Quanto options are priced correctly
- [ ] Currency correlation is handled properly
- [ ] Tests validate against known benchmark values
- [ ] Documentation explains the pricing methodology

## Technical Notes

### Quanto Option Characteristics

- Underlying asset in foreign currency
- Payoff converted at fixed exchange rate
- Eliminates currency risk for the holder
- Requires correlation between asset and FX rate

### Pricing Formula (Quanto Adjustment)

The quanto adjustment modifies the drift of the underlying:

```
Adjusted drift: μ_q = r_d - q - ρ*σ_S*σ_FX

Quanto Call = e^(-r_d*T) * [F_q*N(d1) - K*N(d2)]

where F_q = S * e^((r_d - q - ρ*σ_S*σ_FX)*T) is the quanto forward
```

### Key Parameters
- r_d: Domestic risk-free rate
- r_f: Foreign risk-free rate
- σ_S: Volatility of underlying asset
- σ_FX: Volatility of exchange rate
- ρ: Correlation between asset and FX rate

### Files to Update
- `src/pricing/black_scholes_model.rs`
- `src/pricing/quanto.rs` (new module)
- `tests/unit/pricing/quanto_test.rs` (new)

## Estimated Effort

**Medium (6-8 hours)**

## Dependencies

None

## Related Issues

- Issue #214: Resolve TODOs in black_scholes_model.rs (completed)
