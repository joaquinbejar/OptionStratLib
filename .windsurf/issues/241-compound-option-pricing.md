# Issue #241: Compound Option Pricing Model

## Overview

Compound options are options on options. They have two layers: an outer option (compound) and an inner option (underlying). The holder of a compound option can exercise at T1 to receive (or sell) the underlying option that expires at T2.

## Implementation Phases

### Phase 1: Core Data Model Verification
- Verify `OptionType::Compound { compound_type: CompoundType }` exists
- Check `CompoundType` enum for the four variants:
  - CallOnCall, CallOnPut, PutOnCall, PutOnPut
- Ensure data model captures both strikes (K1, K2) and expiries (T1, T2)

### Phase 2: Bivariate Normal Distribution
- Implement bivariate CDF approximation (Drezner-Wesolowsky)
- Required for Geske (1979) compound option formulas
- Create helper function: `bivariate_normal_cdf(a, b, rho)`

### Phase 3: Critical Value Calculation
- Find critical underlying price S* where compound option is ATM at T1
- Solve: f(S*) = 0 where f is the underlying option value minus K1
- Use Newton-Raphson or bisection method

### Phase 4: Compound Option Pricing (Geske 1979)
- **Call-on-Call**: Right to buy a call option
- **Call-on-Put**: Right to buy a put option  
- **Put-on-Call**: Right to sell a call option
- **Put-on-Put**: Right to sell a put option

### Phase 5: Integration
- Create `src/pricing/compound.rs` with all pricing functions
- Route `OptionType::Compound` in `black_scholes_model.rs`
- Export from `mod.rs`

### Phase 6: Greeks
- Use numerical Greeks from existing module

### Phase 7: Testing
- Test all four compound types
- Test edge cases: T1 ≈ T2, deep ITM/OTM

## Technical Notes

### Geske Formula Key Components
- Two cumulative normal distributions N(d1), N(d2)
- One bivariate normal distribution M(a, b, rho)
- Correlation rho = sqrt(T1/T2)

### Parameters
- S: current underlying price
- K1: strike of compound option
- K2: strike of underlying option  
- T1: time to compound expiry
- T2: time to underlying expiry (T2 > T1)
- r: risk-free rate
- σ: volatility

## Files to Modify/Create

- `src/pricing/compound.rs` - NEW: Compound option pricing functions
- `src/pricing/mod.rs` - Export new module
- `src/pricing/black_scholes_model.rs` - Route Compound to new functions

## Estimated Effort

8-12 hours
