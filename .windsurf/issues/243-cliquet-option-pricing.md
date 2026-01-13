# Issue #243: Cliquet Option Pricing Model

## Overview

Cliquet options (ratchet options) reset their strike periodically, locking in gains. The payoff is the sum of capped/floored returns over each period.

## Implementation Phases

### Phase 1: Core Data Model Verification
- Verify `OptionType::Cliquet { reset_dates: Vec<f64> }` exists
- Ensure reset_dates captures the periodic reset schedule

### Phase 2: Forward-Starting Option Approach
- Treat each period as an independent forward-starting option
- Price each period's contribution and sum them
- Apply local caps/floors to each period's return

### Phase 3: Cliquet Pricing Formula
- For each period i with return R_i = (S_i - S_{i-1}) / S_{i-1}
- Capped return: min(max(R_i, floor), cap)
- Total value: Sum of discounted capped returns

### Phase 4: Integration
- Create `src/pricing/cliquet.rs`
- Route `OptionType::Cliquet` in `black_scholes_model.rs`
- Export from `mod.rs`

### Phase 5: Testing
- Test with various reset frequencies
- Test caps and floors
- Test edge cases

## Technical Notes

### Payoff Structure
- Total Payoff = Σ max(min(R_i, cap), floor)
- R_i = return in period i

### Pricing Approach
- Analytical: Sum of forward-starting call/put spreads
- Each period contributes: BS_call(K=1, T_i) - BS_call(K=1+cap, T_i)

### Default Parameters
- Local cap: 10% per period
- Local floor: 0% per period
- Global cap/floor: Not implemented in first version

## Files to Modify/Create

- `src/pricing/cliquet.rs` - NEW
- `src/pricing/mod.rs` - Export
- `src/pricing/black_scholes_model.rs` - Route

## Estimated Effort

10-14 hours
