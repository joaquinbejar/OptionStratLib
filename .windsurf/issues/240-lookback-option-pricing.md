# Issue #240: Lookback Option Pricing Model

## Overview

Lookback options are path-dependent options with payoffs that depend on the maximum or minimum price of the underlying asset during the option's life. This implementation uses the Goldman-Sosin-Gatto (1979) closed-form solutions.

## Implementation Phases

### Phase 1: Core Data Model Verification
- Verify `OptionType::Lookback { lookback_type: LookbackType }` exists
- Check `LookbackType` enum for FixedStrike and FloatingStrike variants
- Ensure the data model captures the observed min/max prices

### Phase 2: Floating Strike Lookback Options
- **Call**: buyer pays minimum price (S_min), always exercised
  - Formula: Standard Black-Scholes + premium for lookback feature
- **Put**: buyer receives maximum price (S_max), always exercised
  - Uses Goldman-Sosin-Gatto closed-form

### Phase 3: Fixed Strike Lookback Options
- **Call**: Payoff = max(S_max - K, 0)
- **Put**: Payoff = max(K - S_min, 0)
- Uses Conze-Viswanathan formulas

### Phase 4: Goldman-Sosin-Gatto Formulas
For floating strike lookback call (new contract):
```
C = S*N(a1) - S*e^(-rT)*(sigma^2/(2r))*N(-a1) 
    - S*e^(-rT)*N(a2) + S*(sigma^2/(2r))*N(a3)
```
Where a1, a2, a3 are adjusted d-values.

### Phase 5: Integration
- Create `src/pricing/lookback.rs` with all pricing functions
- Route `OptionType::Lookback` in `black_scholes_model.rs`
- Export from `mod.rs`

### Phase 6: Greeks
- Use numerical Greeks from existing `src/greeks/numerical.rs`

### Phase 7: Testing
- Test floating and fixed strike variants
- Test call/put pricing
- Test edge cases: ATM, deep ITM/OTM, zero time

### Phase 8: Documentation
- Add docstrings explaining lookback behavior
- Document the path-dependent nature

## Technical Notes

### Floating Strike Lookback
- Call: `C = S*N(a1) - S_min*e^(-rT)*N(a2) + Y(lambda=1)`
- Put: `P = S_max*e^(-rT)*N(b1) - S*N(b2) + Y(lambda=-1)`

### Fixed Strike Lookback  
- Uses standard BS formulas with S_max or S_min instead of S

### Key Parameters
- For new contracts: S_min = S_max = S (current price)
- For seasoned contracts: actual observed min/max prices

## Files to Modify/Create

- `src/pricing/lookback.rs` - NEW: Lookback option pricing functions
- `src/pricing/mod.rs` - Export new module
- `src/pricing/black_scholes_model.rs` - Route Lookback to new functions

## Estimated Effort

8-12 hours
