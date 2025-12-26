# Issue #198: Base Asset Legs Support Implementation Guide

## Executive Summary

This document outlines the implementation strategy for supporting base asset legs (spot positions) in custom strategies, enabling strategies like Covered Call, Protective Put, and Collar.

---

## Problem Statement

Currently, `Position` only supports option contracts (Call/Put). Strategies requiring underlying asset positions cannot be implemented:

- **Covered Call** = Long Stock + Short Call
- **Protective Put** = Long Stock + Long Put  
- **Collar** = Long Stock + Long Put + Short Call

The `CustomStrategy.positions` field is `Vec<Position>`, and `Position.option` requires `OptionStyle::Call` or `OptionStyle::Put`.

---

## Recommended Solution: Separate Leg Types (Option B)

### Why This Approach

| Consideration | Option A (Extend Position) | Option B (Separate Leg Types) |
|---------------|---------------------------|-------------------------------|
| Breaking changes | High - modifies core struct | None - additive only |
| Type safety | Mixed concerns in one struct | Each type has relevant fields |
| Extensibility | Limited | Easy to add Futures, CFDs, etc. |
| Existing code impact | All 22+ strategies affected | Zero impact on existing code |
| Greeks handling | Complex conditional logic | Clean separation (Spot delta=±1) |

### Core Principle

Create a new `Leg` enum that can represent multiple instrument types, while keeping the existing `Position` struct unchanged for backward compatibility.

---

## Architecture Overview

### New Module Structure

```
src/model/
├── mod.rs                 # Add leg module export
├── position.rs            # UNCHANGED
├── leg/                   # NEW MODULE
│   ├── mod.rs             # Module exports and re-exports
│   ├── traits.rs          # Common leg traits (LegAble)
│   ├── leg_enum.rs        # Leg enum definition
│   ├── spot.rs            # SpotPosition struct
│   ├── future.rs          # FuturePosition struct
│   ├── perpetual.rs       # PerpetualPosition struct (crypto)
│   ├── cfd.rs             # CfdPosition struct
│   └── forward.rs         # ForwardPosition struct
```

### Type Hierarchy

```
Leg (enum)
├── Option(Position)           # Existing option positions
├── Spot(SpotPosition)         # NEW: Underlying asset positions
├── Future(FuturePosition)     # Futures contracts (expiring)
├── Perpetual(PerpetualPosition) # Crypto perpetual contracts
├── Cfd(CfdPosition)           # Contracts for Difference
└── Forward(ForwardPosition)   # OTC forward contracts
```

### Instrument Characteristics Comparison

| Instrument | Expiration | Margin | Funding | Greeks | Primary Use |
|------------|------------|--------|---------|--------|-------------|
| **Spot** | No | No | No | Delta only | Stock, crypto spot |
| **Option** | Yes | No* | No | Full | Hedging, speculation |
| **Future** | Yes | Yes | No | Delta, Rho | Hedging, arbitrage |
| **Perpetual** | No | Yes | Yes | Delta | Crypto leverage |
| **CFD** | No | Yes | Overnight | Delta | Retail leverage |
| **Forward** | Yes | Varies | No | Delta, Rho | OTC hedging |

*Options may require margin for short positions

---

## Implementation Phases

### Phase 1: SpotPosition Struct

Create a new struct to represent spot/underlying asset positions.

**Fields to include:**
- `symbol` - Ticker symbol of the underlying asset
- `quantity` - Number of shares/units held
- `cost_basis` - Average cost per unit
- `side` - Long or Short position
- `date` - Position open date
- `open_fee` - Transaction fee to open
- `close_fee` - Transaction fee to close

**Methods to implement:**
- `new()` - Constructor
- `total_cost()` - Calculate total position cost
- `pnl_at_price()` - Calculate P&L at a given price
- `fees()` - Total fees for the position

**Traits to implement:**
- `Profit` - For P&L calculations
- `PnLCalculator` - For detailed P&L analysis
- `Display` - For formatted output
- `Serialize/Deserialize` - For JSON support

### Phase 2: Leg Enum

Create an enum that wraps different position types.

**Variants:**
- `Option(Position)` - Wraps existing option positions
- `Spot(SpotPosition)` - Wraps new spot positions

**Common interface methods:**
- `symbol()` - Get the underlying symbol
- `quantity()` - Get position quantity
- `side()` - Get position side (Long/Short)
- `total_cost()` - Calculate total cost
- `pnl_at_price()` - Calculate P&L at price
- `fees()` - Get total fees
- `delta()` - Get position delta (Spot = ±1.0, Option = calculated)

### Phase 3: LegAble Trait

Define a common trait for leg operations that both `Position` and `SpotPosition` implement.

**Trait methods:**
- `get_symbol()` - Returns the symbol
- `get_quantity()` - Returns the quantity
- `get_side()` - Returns Long/Short
- `calculate_pnl()` - Calculates profit/loss
- `get_delta()` - Returns delta value

This trait enables polymorphic handling of different leg types.

### Phase 4: Covered Call Strategy

Implement the first strategy using the new leg system.

**Structure:**
- `spot_leg: SpotPosition` - Long underlying position
- `option_leg: Position` - Short call option
- `break_even_points: Vec<Positive>`

**Key calculations:**
- Max profit = Strike - Cost Basis + Premium Received
- Max loss = Cost Basis - Premium Received (if stock goes to zero)
- Break-even = Cost Basis - Premium Received

**Traits to implement:**
- `Strategies`
- `BasicAble`
- `Positionable` (adapted for mixed legs)
- `BreakEvenable`
- `Profit`
- `DeltaNeutrality`

### Phase 5: Protective Put and Collar

Apply the same pattern to implement remaining strategies.

**Protective Put:**
- `spot_leg: SpotPosition` - Long underlying
- `option_leg: Position` - Long put option

**Collar:**
- `spot_leg: SpotPosition` - Long underlying
- `put_leg: Position` - Long put (protection)
- `call_leg: Position` - Short call (income)

### Phase 6: CustomStrategy Extension (Optional)

Extend `CustomStrategy` to optionally support mixed legs.

**Two approaches:**

1. **New field approach:**
   - Add `legs: Vec<Leg>` field
   - Keep `positions: Vec<Position>` for backward compatibility
   - Use `legs` when present, fall back to `positions`

2. **New strategy type approach:**
   - Create `MixedStrategy` struct with `legs: Vec<Leg>`
   - Keep `CustomStrategy` unchanged
   - Users choose which to use based on needs

---

## Greeks Handling

### Spot Position Greeks

| Greek | Value | Rationale |
|-------|-------|-----------|
| Delta | ±1.0 per share | Long = +1, Short = -1 |
| Gamma | 0 | No convexity |
| Theta | 0 | No time decay |
| Vega | 0 | No volatility sensitivity |
| Rho | 0 | Simplified (could model dividend yield) |

### Portfolio Greeks Aggregation

When combining spot and option legs:
- Delta = Σ(option deltas) + Σ(spot deltas)
- Gamma = Σ(option gammas) only
- Theta = Σ(option thetas) only
- Vega = Σ(option vegas) only

---

## Integration Points

### Existing Traits to Consider

| Trait | SpotPosition Support | Notes |
|-------|---------------------|-------|
| `Profit` | Yes | Simple linear P&L |
| `PnLCalculator` | Yes | Cost basis tracking |
| `Greeks` | Partial | Only delta is meaningful |
| `Validable` | Yes | Quantity > 0, valid symbol |
| `Serialize/Deserialize` | Yes | JSON support |

### Strategy Builder Integration

The `StrategyRequest` in `src/strategies/build/model.rs` needs updates:
- Add `CoveredCall`, `ProtectivePut`, `Collar` to the match statement
- These strategies will use `Leg` instead of just `Position`

---

## Testing Strategy

### Unit Tests

1. **SpotPosition tests:**
   - Construction and validation
   - P&L calculations at various prices
   - Fee calculations
   - Serialization/deserialization

2. **Leg enum tests:**
   - Wrapping and unwrapping positions
   - Common interface methods
   - Delta calculations

3. **CoveredCall tests:**
   - Strategy construction
   - Break-even calculation
   - Max profit/loss
   - P&L at various prices
   - Greeks aggregation

### Integration Tests

1. Strategy validation with mixed legs
2. Visualization/graphing support
3. Delta neutrality calculations

---

## Migration Considerations

### Backward Compatibility

- All existing strategies continue to work unchanged
- `Position` struct remains the same
- Existing `CustomStrategy` usage is unaffected
- New functionality is purely additive

### API Stability

- New types are added, none removed
- Existing method signatures unchanged
- New strategies follow established patterns

---

## Derivative Types Detailed Specification

### SpotPosition

Represents direct ownership of the underlying asset (stocks, crypto spot, forex spot).

**Fields:**
- `symbol` - Asset ticker/symbol
- `quantity` - Number of units held
- `cost_basis` - Average acquisition price per unit
- `side` - Long or Short
- `date` - Position open timestamp
- `open_fee` / `close_fee` - Transaction fees

**Characteristics:**
- No expiration
- No margin requirements (fully funded)
- Delta = ±1.0 per unit
- No funding costs

---

### FuturePosition

Represents standardized exchange-traded futures contracts.

**Fields:**
- `symbol` - Contract symbol (e.g., "ES", "BTC-PERP")
- `quantity` - Number of contracts
- `entry_price` - Average entry price
- `side` - Long or Short
- `expiration_date` - Contract expiry
- `contract_size` - Multiplier (e.g., 50 for ES, 1 for BTC)
- `margin` - Initial margin requirement
- `maintenance_margin` - Maintenance margin level
- `date` - Position open timestamp
- `fees` - Commission and exchange fees

**Characteristics:**
- Fixed expiration date
- Margin-based (leveraged)
- Mark-to-market daily settlement
- Delta ≈ ±1.0 per contract (adjusted for contract size)
- Rho sensitivity for interest rate changes

**Use Cases:**
- Hedging underlying exposure
- Calendar spreads (long near / short far)
- Basis trades (spot vs futures)
- Index futures strategies

---

### PerpetualPosition (Crypto)

Represents perpetual swap contracts common in cryptocurrency markets.

**Fields:**
- `symbol` - Trading pair (e.g., "BTC-USDT-PERP")
- `quantity` - Position size in base currency
- `entry_price` - Average entry price
- `side` - Long or Short
- `leverage` - Applied leverage (1x to 125x typical)
- `margin` - Collateral posted
- `margin_type` - Cross or Isolated margin mode
- `liquidation_price` - Price at which position is liquidated
- `funding_rate` - Current funding rate (updated periodically)
- `next_funding_time` - Timestamp of next funding payment
- `unrealized_pnl` - Current unrealized P&L
- `date` - Position open timestamp
- `fees` - Trading fees (maker/taker)

**Characteristics:**
- No expiration (perpetual)
- High leverage available
- Funding rate mechanism to anchor price to spot
- Funding payments every 8 hours (typically)
- Delta = ±leverage per unit of margin
- Liquidation risk

**Funding Rate Mechanics:**
- Positive rate: Longs pay shorts
- Negative rate: Shorts pay longs
- Rate based on premium/discount to spot index

**Use Cases:**
- Leveraged directional trades
- Delta-neutral funding rate arbitrage
- Hedging spot crypto holdings
- Basis trades (spot vs perp)

---

### CfdPosition

Represents Contracts for Difference (common in forex and retail trading).

**Fields:**
- `symbol` - Instrument symbol
- `quantity` - Position size (lots or units)
- `entry_price` - Average entry price
- `side` - Long or Short
- `leverage` - Applied leverage
- `margin` - Required margin
- `overnight_rate` - Swap/rollover rate
- `date` - Position open timestamp
- `fees` - Spread cost and commissions

**Characteristics:**
- No expiration
- Leveraged product
- Overnight financing charges (swap rates)
- No ownership of underlying
- Delta = ±leverage per unit

**Use Cases:**
- Retail forex trading
- Index CFD trading
- Commodity exposure without physical delivery

---

### ForwardPosition

Represents OTC forward contracts (customizable terms).

**Fields:**
- `symbol` - Underlying asset
- `quantity` - Contract size
- `forward_price` - Agreed forward price
- `side` - Long or Short
- `settlement_date` - Contract maturity
- `settlement_type` - Physical or Cash settlement
- `counterparty` - Optional counterparty identifier
- `date` - Contract inception date
- `fees` - Arrangement fees

**Characteristics:**
- Customizable expiration
- OTC (counterparty risk)
- No daily settlement (settled at maturity)
- Delta ≈ ±1.0
- Rho sensitivity

**Use Cases:**
- Corporate hedging (FX forwards)
- Commodity price locking
- Custom expiration requirements

---

## Greeks by Instrument Type

| Greek | Spot | Option | Future | Perpetual | CFD | Forward |
|-------|------|--------|--------|-----------|-----|---------|
| **Delta** | ±1.0 | Calculated | ±1.0 × size | ±leverage | ±leverage | ±1.0 |
| **Gamma** | 0 | Calculated | 0 | 0 | 0 | 0 |
| **Theta** | 0 | Calculated | ~0* | Funding** | Overnight** | ~0* |
| **Vega** | 0 | Calculated | 0 | 0 | 0 | 0 |
| **Rho** | 0 | Calculated | Yes | 0 | 0 | Yes |

*Futures have time value that decays as basis converges
**Perpetuals have funding; CFDs have overnight financing

---

## Crypto-Specific Considerations

### Exchange Integration

Different exchanges have varying perpetual contract specifications:

| Exchange | Funding Interval | Max Leverage | Margin Modes |
|----------|-----------------|--------------|--------------|
| Binance | 8h | 125x | Cross, Isolated |
| Bybit | 8h | 100x | Cross, Isolated |
| OKX | 8h | 125x | Cross, Isolated |
| dYdX | 1h | 20x | Cross |
| GMX | Continuous | 50x | Isolated |

### Funding Rate Calculation

The funding rate typically consists of:
1. **Interest Rate Component** - Usually fixed (e.g., 0.01% per 8h)
2. **Premium/Discount Component** - Based on mark price vs index price

### Liquidation Mechanics

For perpetuals, track:
- **Maintenance Margin Ratio** - Minimum margin to avoid liquidation
- **Liquidation Price** - Price at which position is force-closed
- **Insurance Fund** - Exchange backstop for bankrupt positions

### Position Sizing

Perpetual position size can be expressed as:
- **Notional Value** = Quantity × Mark Price
- **Margin Used** = Notional Value / Leverage
- **Available Margin** = Total Collateral - Margin Used

---

## Implementation Priority

Given the crypto focus, recommended implementation order:

| Priority | Instrument | Rationale |
|----------|------------|-----------|
| 1 | SpotPosition | Foundation for all strategies |
| 2 | PerpetualPosition | High demand in crypto |
| 3 | FuturePosition | Traditional markets + crypto quarterly |
| 4 | CfdPosition | Retail trading support |
| 5 | ForwardPosition | OTC/institutional use |

---

## Example Strategies Enabled

### Traditional Markets

| Strategy | Legs | Description |
|----------|------|-------------|
| Covered Call | Spot + Short Call | Income on holdings |
| Protective Put | Spot + Long Put | Downside protection |
| Collar | Spot + Long Put + Short Call | Range-bound protection |
| Synthetic Long | Long Call + Short Put | Replicate stock |
| Conversion | Spot + Long Put + Short Call | Arbitrage |

### Crypto Markets

| Strategy | Legs | Description |
|----------|------|-------------|
| Cash & Carry | Spot + Short Perp | Funding rate arbitrage |
| Basis Trade | Spot + Short Future | Basis capture |
| Delta Neutral Funding | Spot + Short Perp (equal size) | Pure funding income |
| Hedged Perp | Perp + Options | Leveraged with protection |
| Perp Spread | Long Perp (Exchange A) + Short Perp (Exchange B) | Cross-exchange arb |

### Funding Rate Arbitrage Example

A common crypto strategy:
1. Buy 1 BTC spot at $50,000
2. Short 1 BTC perpetual at $50,000
3. Net delta = 0 (delta neutral)
4. Collect funding rate (if positive) every 8 hours
5. Expected APY = Funding Rate × 3 × 365 (annualized)

---

## Estimated Effort

### Core Infrastructure

| Phase | Complexity | Estimated Time |
|-------|------------|----------------|
| Phase 1: SpotPosition | Low | 2-3 hours |
| Phase 2: Leg Enum | Low | 1-2 hours |
| Phase 3: LegAble Trait | Medium | 2-3 hours |
| Phase 4: CoveredCall | Medium | 3-4 hours |
| Phase 5: ProtectivePut/Collar | Low | 2-3 hours |
| Phase 6: CustomStrategy Extension | Medium | 3-4 hours |

**Core subtotal: 13-19 hours**

### Additional Derivatives

| Derivative | Complexity | Estimated Time |
|------------|------------|----------------|
| PerpetualPosition | Medium-High | 4-6 hours |
| FuturePosition | Medium | 3-4 hours |
| CfdPosition | Low-Medium | 2-3 hours |
| ForwardPosition | Low | 2-3 hours |

**Derivatives subtotal: 11-16 hours**

### Testing & Documentation

| Task | Estimated Time |
|------|----------------|
| Unit tests per derivative | 1-2 hours each |
| Integration tests | 2-3 hours |
| Documentation | 2-3 hours |
| Examples | 2-3 hours |

**Testing subtotal: 10-15 hours**

### Total Estimated Effort

| Scope | Time Range |
|-------|------------|
| Core only (Spot + Options strategies) | 16-23 hours |
| Core + Perpetuals | 22-32 hours |
| Full implementation (all derivatives) | 34-50 hours |

---

## References

- Issue #198: https://github.com/joaquinbejar/OptionStratLib/issues/198
- Existing placeholder files:
  - `src/strategies/covered_call.rs`
  - `src/strategies/collar.rs`
- Related implementation patterns:
  - `src/model/position.rs`
  - `src/strategies/custom.rs`
