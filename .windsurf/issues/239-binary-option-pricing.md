# Issue #239: Binary Option Pricing Model

## Overview

Binary options (also called digital options) have a fixed payout if the option expires in-the-money, regardless of how far in-the-money it is. This implementation will support cash-or-nothing and asset-or-nothing variants.

## Implementation Phases

### Phase 1: Core Data Model Verification
- Verify `OptionType::Binary { binary_type: BinaryPayoffType }` exists
- Check `BinaryPayoffType` enum for CashOrNothing and AssetOrNothing variants
- Ensure the data model captures the payout amount (Q) for cash-or-nothing

### Phase 2: Cash-or-Nothing Binary Options
- Implement `cash_or_nothing_call()`: `C = Q * e^(-rT) * N(d2)`
- Implement `cash_or_nothing_put()`: `P = Q * e^(-rT) * N(-d2)`
- Q = fixed cash payout amount

### Phase 3: Asset-or-Nothing Binary Options
- Implement `asset_or_nothing_call()`: `C = S * e^(-qT) * N(d1)`
- Implement `asset_or_nothing_put()`: `P = S * e^(-qT) * N(-d1)`

### Phase 4: Integration
- Create `src/pricing/binary.rs` with all pricing functions
- Route `OptionType::Binary` in `black_scholes_model.rs`
- Export from `mod.rs`

### Phase 5: Greeks (Numerical)
- Use numerical Greeks from existing `src/greeks/numerical.rs`
- Note: Delta is discontinuous at strike, Gamma very large near expiration

### Phase 6: Testing
- Test cash-or-nothing and asset-or-nothing variants
- Test call/put symmetry where applicable
- Test edge cases: ATM, deep ITM/OTM, zero time

### Phase 7: Documentation
- Add docstrings explaining binary option behavior
- Note discontinuous Delta at strike

## Technical Notes

### Cash-or-Nothing
- **Call**: Pays Q if S_T > K
- **Put**: Pays Q if S_T < K
- Formula: `C = Q * e^(-rT) * N(d2)` where d2 is standard BS d2

### Asset-or-Nothing
- **Call**: Pays S_T if S_T > K
- **Put**: Pays S_T if S_T < K
- Formula: `C = S * e^(-qT) * N(d1)` where d1 is standard BS d1

## Files to Modify/Create

- `src/pricing/binary.rs` - NEW: Binary option pricing functions
- `src/pricing/mod.rs` - Export new module
- `src/pricing/black_scholes_model.rs` - Route Binary to new functions

## Estimated Effort

4-6 hours
