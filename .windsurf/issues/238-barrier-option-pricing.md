# Issue #238: Implement Barrier Option Pricing Model

## Overview
Implement the Black-Scholes extension for pricing Barrier options (path-dependent). Support both knock-in and knock-out variants with up and down barriers.

## Implementation Phases

### Phase 1: Model & Schema Updates
- [ ] Add `rebate` field to `OptionType::Barrier` variant.
- [ ] Add `MonitoringType` (Continuous vs Discrete) to `OptionType::Barrier`.
- [ ] Ensure `ExoticParams` can hold necessary historical data for discrete monitoring if we implement it, or just stick to continuous for close-form.

### Phase 2: Mathematical Implementation (`src/pricing/barrier.rs`)
- [ ] Implement the 8 standard closed-form solutions for continuously monitored barriers:
    - Down-and-In Call
    - Up-and-In Call
    - Down-and-Out Call
    - Up-and-Out Call
    - Down-and-In Put
    - Up-and-In Put
    - Down-and-Out Put
    - Up-and-Out Put
- [ ] Implement rebate pricing logic (payout when barrier is hit/not hit).

### Phase 3: Greeks Calculation
- [ ] Implement Greeks (Delta, Gamma, Vega, Theta, Rho) for Barrier options. Note: These are different from vanilla Greeks.

### Phase 4: Integration
- [ ] Update `src/pricing/black_scholes_model.rs` to route `OptionType::Barrier` to the new pricing function.
- [ ] Update `Payoff` implementation for `OptionType` in `src/model/types.rs` to account for rebates and activation state.

### Phase 5: Testing & Validation
- [ ] Unit tests for each barrier type vs external calculators (e.g., QuantLib or standard tables).
- [ ] Verify In-Out Parity: Knock-In + Knock-Out = Vanilla.
- [ ] Property-based tests for boundary conditions.

### Phase 6: Examples & Documentation
- [ ] Add example in `examples/` folder.
- [ ] Update documentation.
