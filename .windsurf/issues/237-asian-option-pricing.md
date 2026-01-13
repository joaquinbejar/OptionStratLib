# Issue #237: Asian Option Pricing Model

## Overview

Asian options are path-dependent options where the payoff depends on the average price of the underlying asset over a specified period. This implementation will support both geometric and arithmetic averaging, with fixed and floating strike variants.

## Implementation Phases

### Phase 1: Core Data Model Verification
- Verify `OptionType::Asian { averaging_type: AsianAveragingType }` exists
- Ensure `AsianAveragingType` enum has Arithmetic and Geometric variants
- Add any missing fields for fixed/floating strike distinction

### Phase 2: Geometric Average Asian Option (Closed-Form)
- Implement `geometric_asian_black_scholes()` in new `src/pricing/asian.rs`
- Use adjusted Black-Scholes formula:
  - Adjusted volatility: `σ_adj = σ / √3`
  - Adjusted rate: `r_adj = (r + σ²/6) / 2` for the cost-of-carry adjustment
- Support both call and put options

### Phase 3: Arithmetic Average Asian Option
- Implement Turnbull-Wakeman approximation for closed-form pricing
- Match first two moments of arithmetic average to lognormal distribution
- Fallback to Monte Carlo for higher accuracy if needed

### Phase 4: Fixed vs Floating Strike
- Fixed strike: payoff = max(Average - K, 0) for calls
- Floating strike: payoff = max(S_T - Average, 0) for calls
- Handle both variants in pricing functions

### Phase 5: Integration
- Route `OptionType::Asian` to new pricing functions in `black_scholes_model.rs`
- Integrate with unified pricing API

### Phase 6: Greeks
- Use numerical Greeks from existing `src/greeks/numerical.rs` module
- Route Asian options to numerical implementations in `equations.rs`

### Phase 7: Testing
- Test geometric average closed-form against known values
- Test arithmetic approximation accuracy
- Verify put-call parity relationships
- Edge cases: zero volatility, zero time, extreme averaging periods

### Phase 8: Documentation
- Add docstrings and examples
- Document formula sources and limitations

## Technical Notes

### Geometric Average Closed-Form
For a geometric average Asian call:
```
C = S * e^((b_adj - r) * T) * N(d1) - K * e^(-r * T) * N(d2)
```
where:
- `σ_adj = σ / √3`
- `b_adj = 0.5 * (r - q - σ²/6)`

### Turnbull-Wakeman Approximation
Matches moments of the arithmetic average to a lognormal distribution.

## Dependencies

- Existing Black-Scholes infrastructure
- Numerical Greeks module

## Files to Modify/Create

- `src/pricing/asian.rs` - NEW: Asian option pricing functions
- `src/pricing/mod.rs` - Export new module
- `src/pricing/black_scholes_model.rs` - Route Asian to new functions
- `src/greeks/equations.rs` - Route to numerical Greeks

## Estimated Effort

8-12 hours
