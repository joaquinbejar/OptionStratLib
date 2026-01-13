# Issue #242: Chooser Option Pricing Model

## Overview

Chooser options allow the holder to choose at a specified date (choice date) whether the option becomes a call or a put. Uses Rubinstein (1991) decomposition.

## Implementation Phases

### Phase 1: Core Data Model Verification
- Verify `OptionType::Chooser { choice_date: f64 }` exists
- Ensure choice_date is captured as time until choice

### Phase 2: Simple Chooser (Rubinstein 1991)
- Same strike K and expiration T for both call and put
- Choice date t < T
- Formula: Chooser = Call(K, T) + Put(K, t) * e^(-(r-q)(T-t))
- Alternative: C + Se^(-q*t)*N(-y2) - Ke^(-r*t)*N(-y1 + sigma*sqrt(t))

### Phase 3: Complex Chooser (Optional)
- Different strikes and/or expirations for call vs put
- More complex valuation

### Phase 4: Integration
- Create `src/pricing/chooser.rs`
- Route `OptionType::Chooser` in `black_scholes_model.rs`
- Export from `mod.rs`

### Phase 5: Testing
- Test simple chooser pricing
- Test edge cases: choice_date = 0, choice_date = T
- Test call/put parity relationships

## Technical Notes

### Simple Chooser Formula
```
V = S*e^(-qT)*N(d1) - K*e^(-rT)*N(d2) 
    + K*e^(-rT)*N(-y2) - S*e^(-qT)*N(-y1)
```
Where:
- d1, d2 are standard BS d-values for T
- y1 = [ln(S/K) + (b + σ²/2)t] / (σ√t)
- y2 = y1 - σ√t
- t = choice date, T = expiration

## Files to Modify/Create

- `src/pricing/chooser.rs` - NEW
- `src/pricing/mod.rs` - Export
- `src/pricing/black_scholes_model.rs` - Route

## Estimated Effort

6-8 hours
