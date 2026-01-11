# Issue #235: Implement American Option Pricing Model

## Summary

Enhance American option pricing support with analytical approximation methods and improved documentation.

## Current State

- ✅ Binomial Tree model already supports American options (`price_binomial`, `generate_binomial_tree`)
- ✅ `Options::calculate_price_binomial()` works with `OptionType::American`
- ❌ Black-Scholes returns `UnsupportedOptionType` for American options (correct behavior)
- ❌ No analytical approximation for faster pricing
- ❌ Limited documentation for American option usage

## Implementation Plan

### Phase 1: Barone-Adesi-Whaley Approximation (This PR)
Implement the BAW analytical approximation for American options:
- Fast O(1) computation vs O(n²) for binomial
- Good accuracy for most practical cases
- Widely used in industry

### Phase 2: Documentation & Tests
- Add comprehensive tests with known values
- Document usage examples for American options
- Add comparison between binomial and BAW methods

### Phase 3: Greeks for American Options (Future)
- Delta, Gamma, Theta, Vega for American options
- Early exercise boundary calculation

## Barone-Adesi-Whaley Algorithm

The BAW model provides an analytical approximation for American options:

### For American Calls:
```
C_american = C_european + A2 * (S/S*)^q2  if S < S*
C_american = S - K                         if S >= S*
```

### For American Puts:
```
P_american = P_european + A1 * (S/S**)^q1  if S > S**
P_american = K - S                          if S <= S**
```

Where:
- S* and S** are the critical stock prices (early exercise boundaries)
- q1, q2 are functions of r, σ, and T
- A1, A2 are coefficients derived from boundary conditions

## Files to Create/Modify

### New File: `src/pricing/american.rs`
- `barone_adesi_whaley()` - Main BAW pricing function
- `calculate_critical_price()` - Find early exercise boundary
- Helper functions for q1, q2, A1, A2 calculations

### Modify: `src/pricing/mod.rs`
- Export new `american` module
- Update documentation

### Modify: `src/model/option.rs`
- Add `calculate_price_american()` method using BAW
- Keep `calculate_price_binomial()` for more accurate pricing

## Acceptance Criteria

- [ ] BAW approximation implemented and tested
- [ ] Tests with known values from literature
- [ ] Documentation with usage examples
- [ ] `make lint-fix` passes
- [ ] `make pre-push` passes

## References

- Barone-Adesi, G., & Whaley, R. E. (1987). "Efficient Analytic Approximation of American Option Values"
- Hull, J. C. "Options, Futures, and Other Derivatives"
