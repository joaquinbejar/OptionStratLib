# Issue #26: Implement Cliquet option pricing

## Title
`feat: Implement Cliquet option pricing model`

## Labels
- `enhancement`
- `pricing`
- `priority-low`

## Description

Implement pricing support for Cliquet options (also known as Ratchet options). Cliquet options are a series of forward-starting options where the strike is reset periodically based on the underlying price.

### Current State
- Cliquet options return `PricingError::UnsupportedOptionType`
- No pricing model for periodic reset options

### Target State
- Fully functional Cliquet option pricing
- Support for various reset frequencies
- Support for local and global caps/floors

## Tasks

- [ ] Implement basic cliquet option pricing
- [ ] Support periodic strike resets
- [ ] Implement local caps and floors on returns
- [ ] Implement global caps and floors
- [ ] Implement Greeks calculations
- [ ] Add comprehensive tests
- [ ] Add documentation with examples
- [ ] Run `make lint-fix` and `make pre-push` to verify

## Acceptance Criteria

- [ ] Cliquet options are priced correctly
- [ ] Reset mechanics work properly
- [ ] Tests validate against known benchmark values
- [ ] Documentation explains the pricing methodology

## Technical Notes

### Cliquet Option Characteristics

- Series of consecutive forward-starting options
- Strike resets to spot price at each reset date
- Returns are locked in at each period
- Often includes caps/floors on individual period returns

### Payoff Structure

```
Total Payoff = Σ max(min(R_i, cap), floor)

where R_i = (S_i - S_{i-1}) / S_{i-1} is the return in period i
```

### Pricing Approach

**Monte Carlo Simulation**
- Simulate price paths
- Calculate returns at each reset date
- Apply caps/floors
- Sum up the capped returns

**Analytical Approximation**
- Treat each period as independent forward-starting option
- Adjust for caps/floors using spread options

### Files to Update
- `src/pricing/black_scholes_model.rs`
- `src/pricing/cliquet.rs` (new module)
- `tests/unit/pricing/cliquet_test.rs` (new)

## Estimated Effort

**High (10-14 hours)**

## Dependencies

- May require Monte Carlo simulation framework

## Related Issues

- Issue #214: Resolve TODOs in black_scholes_model.rs (completed)
