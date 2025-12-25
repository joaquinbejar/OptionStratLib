:::::::::: width-limiter
::::::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../index.html)
:::

# Module strategies Copy item path

[[Source](../../src/optionstratlib/strategies/mod.rs.html#7-298){.src}
]{.sub-heading}
::::

Expand description

:::::: docblock
- `strategies` - Pre-defined option strategy templates and building
  blocks.

Library of common option strategies (spreads, straddles, condors, etc.)
with implementation helpers, parameter optimization, and analysis tools.
Supports strategy composition and customization.

## [§](#options-strategies-module){.doc-anchor}Options Strategies Module

This module contains implementations for various options trading
strategies. These strategies combine multiple options contracts to form
risk-managed trades suitable for different market conditions. Each
strategy provides calculation utilities for profit/loss scenarios and
risk assessment.

### [§](#sub-modules){.doc-anchor}Sub-modules

- `base`: Provides the base traits and structures for the strategies.
- `bear_call_spread`: Implements the Bear Call Spread strategy.
- `bear_put_spread`: Implements the Bear Put Spread strategy.
- `bull_call_spread`: Implements the Bull Call Spread strategy.
- `bull_put_spread`: Implements the Bull Put Spread strategy.
- `butterfly_spread`: Implements the Butterfly Spread strategy.
- `call_butterfly`: Implements the Call Butterfly strategy.
- `collar`: Implements the Collar strategy.
- `covered_call`: Implements the Covered Call strategy.
- `custom`: Provides utilities for creating custom strategies.
- `iron_butterfly`: Implements the Iron Butterfly strategy.
- `iron_condor`: Implements the Iron Condor strategy.
- `poor_mans_covered_call`: Implements the Poor Man's Covered Call
  strategy.
- `probabilities`: Provides probability calculations for the strategies.
- `protective_put`: Implements the Protective Put strategy.
- `straddle`: Implements the Straddle strategy.
- `strangle`: Implements the Strangle strategy.
- `utils`: Provides utility functions for the strategies.

### [§](#usage){.doc-anchor}Usage

To use a specific strategy, import the corresponding module and create
an instance of the strategy struct. Each strategy provides methods for
calculating key metrics such as maximum profit, maximum loss, and
breakeven points.

Example usage of the Bull Call Spread strategy:

::: example-wrap
``` {.rust .rust-example-rendered}
use rust_decimal_macros::dec;
use tracing::info;
use optionstratlib::ExpirationDate;
use optionstratlib::strategies::bull_call_spread::BullCallSpread;
use optionstratlib::Positive;
use optionstratlib::pos_or_panic;
use optionstratlib::strategies::Strategies;

let spread = BullCallSpread::new(
    "SP500".to_string(),
    pos!(5780.0),   // underlying_price
    pos!(5750.0),   // long_strike_itm  
    pos!(5820.0),   // short_strike
    ExpirationDate::Days(pos!(2.0)),
    pos!(0.18),   // implied_volatility
    dec!(0.05),   // risk_free_rate
    Positive::ZERO,   // dividend_yield
    pos!(2.0),   // long quantity
    pos!(85.04),   // premium_long
    pos!(29.85),   // premium_short
    pos!(0.78),   // open_fee_long
    pos!(0.78),   // open_fee_long
    pos!(0.73),   // close_fee_long
    pos!(0.73),   // close_fee_short
);

let profit = spread.get_max_profit().unwrap_or(Positive::ZERO);
let loss = spread.get_max_loss().unwrap_or(Positive::ZERO);
info!("Max Profit: {}, Max Loss: {}", profit, loss);
```
:::

Refer to the documentation of each sub-module for more details on the
specific strategies and their usage.

## [§](#base-module){.doc-anchor}Base Module

This module provides the base traits and structures for defining and
working with options trading strategies. It includes the `StrategyType`
enum, `Strategy` struct, and the `Strategies`, `Validable`, and
`Optimizable` traits.

### [§](#strategytype){.doc-anchor}StrategyType

The `StrategyType` enum represents different types of trading
strategies. Each variant corresponds to a specific strategy, such as
`BullCallSpread`, `BearPutSpread`, `IronCondor`, etc.

### [§](#strategy){.doc-anchor}Strategy

The `Strategy` struct represents a trading strategy. It contains
properties such as the strategy's name, type, description, legs
(positions), maximum profit, maximum loss, and break-even points.

### [§](#strategies-trait){.doc-anchor}Strategies Trait

The `Strategies` trait defines the common methods that a trading
strategy should implement. It includes methods for adding legs,
retrieving legs, calculating break-even points, maximum profit, maximum
loss, total cost, and more.

### [§](#validable-trait){.doc-anchor}Validable Trait

The `Validable` trait provides a method for validating a trading
strategy. Strategies should implement this trait to ensure they are
valid before being used.

### [§](#optimizable-trait){.doc-anchor}Optimizable Trait

The `Optimizable` trait extends the `Validable` and `Strategies` traits
and adds methods for optimizing a trading strategy. It includes methods
for finding the optimal strategy based on different criteria, such as
best ratio or best area.

### [§](#usage-1){.doc-anchor}Usage

To define a new trading strategy, create a struct that implements the
`Strategies`, `Validable`, and optionally, the `Optimizable` traits.
Implement the required methods for each trait based on the specific
behavior of your strategy.

Example:

::: example-wrap
``` {.rust .rust-example-rendered}
use optionstratlib::error::position::PositionError;
use optionstratlib::model::position::Position;
use optionstratlib::Positive;
use optionstratlib::strategies::base::{BreakEvenable, Positionable, Strategies, Validable};
use optionstratlib::strategies::{BasicAble, Strategable};

struct MyStrategy {
    legs: Vec<Position>,
    // Other strategy-specific fields
}

impl Validable for MyStrategy {
    fn validate(&self) -> bool {
       true
    }
}


impl Positionable for MyStrategy {
    fn add_position(&mut self, position: &Position) -> Result<(), PositionError> {
        Ok(self.legs.push(position.clone()))
    }

 fn get_positions(&self) -> Result<Vec<&Position>, PositionError> {
        Ok(self.legs.iter().collect())
    }
}

impl BreakEvenable for MyStrategy {}


impl BasicAble for MyStrategy {}

impl Strategies for MyStrategy {}
```
:::

//! Example usage of the Iron Condor strategy:

::: example-wrap
``` {.rust .rust-example-rendered}
use rust_decimal_macros::dec;
use tracing::info;
use optionstratlib::ExpirationDate;
use optionstratlib::strategies::iron_condor::IronCondor;
use optionstratlib::Positive;
use optionstratlib::pos_or_panic;
use optionstratlib::strategies::Strategies;

let condor = IronCondor::new(
    "AAPL".to_string(),
    pos!(150.0),   // underlying_price
    pos!(155.0),   // short_call_strike
    pos!(145.0),   // short_put_strike  
    pos!(160.0),   // long_call_strike
    pos!(140.0),   // long_put_strike
    ExpirationDate::Days(pos!(30.0)),
    pos!(0.2),   // implied_volatility
    dec!(0.01),   // risk_free_rate
    pos!(0.02),   // dividend_yield
    pos!(1.0),   // quantity
    pos!(1.5),   // premium_short_call
    Positive::ONE,   // premium_short_put
    Positive::TWO,   // premium_long_call
    pos!(1.8),   // premium_long_put
    pos!(5.0),   // open_fee
    pos!(5.0),   // close_fee
);

let max_profit = condor.get_max_profit().unwrap_or(Positive::ZERO);
let max_loss = condor.get_max_loss().unwrap_or(Positive::ZERO);
info!("Max Profit: {}, Max Loss: {}", max_profit, max_loss);
```
:::

Refer to the documentation of each sub-module for more details on the
specific strategies and their usage.
::::::

## Re-exports[§](#reexports){.anchor} {#reexports .section-header}

`pub use base::`[`BasicAble`](base/trait.BasicAble.html "trait optionstratlib::strategies::base::BasicAble"){.trait}`;`

`pub use base::`[`Strategable`](base/trait.Strategable.html "trait optionstratlib::strategies::base::Strategable"){.trait}`;`

`pub use base::`[`Strategies`](base/trait.Strategies.html "trait optionstratlib::strategies::base::Strategies"){.trait}`;`

`pub use base::`[`StrategyBasics`](base/struct.StrategyBasics.html "struct optionstratlib::strategies::base::StrategyBasics"){.struct}`;`

`pub use base::`[`Validable`](base/trait.Validable.html "trait optionstratlib::strategies::base::Validable"){.trait}`;`

`pub use bear_call_spread::`[`BearCallSpread`](bear_call_spread/struct.BearCallSpread.html "struct optionstratlib::strategies::bear_call_spread::BearCallSpread"){.struct}`;`

`pub use bear_put_spread::`[`BearPutSpread`](bear_put_spread/struct.BearPutSpread.html "struct optionstratlib::strategies::bear_put_spread::BearPutSpread"){.struct}`;`

`pub use bull_call_spread::`[`BullCallSpread`](bull_call_spread/struct.BullCallSpread.html "struct optionstratlib::strategies::bull_call_spread::BullCallSpread"){.struct}`;`

`pub use bull_put_spread::`[`BullPutSpread`](bull_put_spread/struct.BullPutSpread.html "struct optionstratlib::strategies::bull_put_spread::BullPutSpread"){.struct}`;`

`pub use call_butterfly::`[`CallButterfly`](call_butterfly/struct.CallButterfly.html "struct optionstratlib::strategies::call_butterfly::CallButterfly"){.struct}`;`

`pub use delta_neutral::`[`DELTA_THRESHOLD`](delta_neutral/constant.DELTA_THRESHOLD.html "constant optionstratlib::strategies::delta_neutral::DELTA_THRESHOLD"){.constant}`;`

`pub use delta_neutral::`[`DeltaAdjustment`](delta_neutral/enum.DeltaAdjustment.html "enum optionstratlib::strategies::delta_neutral::DeltaAdjustment"){.enum}`;`

`pub use delta_neutral::`[`DeltaInfo`](delta_neutral/struct.DeltaInfo.html "struct optionstratlib::strategies::delta_neutral::DeltaInfo"){.struct}`;`

`pub use delta_neutral::`[`DeltaNeutrality`](delta_neutral/trait.DeltaNeutrality.html "trait optionstratlib::strategies::delta_neutral::DeltaNeutrality"){.trait}`;`

`pub use iron_butterfly::`[`IronButterfly`](iron_butterfly/struct.IronButterfly.html "struct optionstratlib::strategies::iron_butterfly::IronButterfly"){.struct}`;`

`pub use iron_condor::`[`IronCondor`](iron_condor/struct.IronCondor.html "struct optionstratlib::strategies::iron_condor::IronCondor"){.struct}`;`

`pub use long_butterfly_spread::`[`LongButterflySpread`](long_butterfly_spread/struct.LongButterflySpread.html "struct optionstratlib::strategies::long_butterfly_spread::LongButterflySpread"){.struct}`;`

`pub use long_call::`[`LongCall`](long_call/struct.LongCall.html "struct optionstratlib::strategies::long_call::LongCall"){.struct}`;`

`pub use long_put::`[`LongPut`](long_put/struct.LongPut.html "struct optionstratlib::strategies::long_put::LongPut"){.struct}`;`

`pub use long_straddle::`[`LongStraddle`](long_straddle/struct.LongStraddle.html "struct optionstratlib::strategies::long_straddle::LongStraddle"){.struct}`;`

`pub use long_strangle::`[`LongStrangle`](long_strangle/struct.LongStrangle.html "struct optionstratlib::strategies::long_strangle::LongStrangle"){.struct}`;`

`pub use poor_mans_covered_call::`[`PoorMansCoveredCall`](poor_mans_covered_call/struct.PoorMansCoveredCall.html "struct optionstratlib::strategies::poor_mans_covered_call::PoorMansCoveredCall"){.struct}`;`

`pub use short_butterfly_spread::`[`ShortButterflySpread`](short_butterfly_spread/struct.ShortButterflySpread.html "struct optionstratlib::strategies::short_butterfly_spread::ShortButterflySpread"){.struct}`;`

`pub use short_call::`[`ShortCall`](short_call/struct.ShortCall.html "struct optionstratlib::strategies::short_call::ShortCall"){.struct}`;`

`pub use short_put::`[`ShortPut`](short_put/struct.ShortPut.html "struct optionstratlib::strategies::short_put::ShortPut"){.struct}`;`

`pub use short_straddle::`[`ShortStraddle`](short_straddle/struct.ShortStraddle.html "struct optionstratlib::strategies::short_straddle::ShortStraddle"){.struct}`;`

`pub use short_strangle::`[`ShortStrangle`](short_strangle/struct.ShortStrangle.html "struct optionstratlib::strategies::short_strangle::ShortStrangle"){.struct}`;`

`pub use utils::`[`FindOptimalSide`](utils/enum.FindOptimalSide.html "enum optionstratlib::strategies::utils::FindOptimalSide"){.enum}`;`

## Modules[§](#modules){.anchor} {#modules .section-header}

[base](base/index.html "mod optionstratlib::strategies::base"){.mod}
:   Options trading strategies module collection

[bear_call_spread](bear_call_spread/index.html "mod optionstratlib::strategies::bear_call_spread"){.mod}
:   Bear Call Spread strategy implementation

[bear_put_spread](bear_put_spread/index.html "mod optionstratlib::strategies::bear_put_spread"){.mod}
:   Bear Put Spread strategy implementation

[bull_call_spread](bull_call_spread/index.html "mod optionstratlib::strategies::bull_call_spread"){.mod}
:   Bull Call Spread strategy implementation

[bull_put_spread](bull_put_spread/index.html "mod optionstratlib::strategies::bull_put_spread"){.mod}
:   Bull Put Spread strategy implementation

[call_butterfly](call_butterfly/index.html "mod optionstratlib::strategies::call_butterfly"){.mod}
:   Call Butterfly strategy implementation

[collar](collar/index.html "mod optionstratlib::strategies::collar"){.mod}
:   Collar strategy implementation

[covered_call](covered_call/index.html "mod optionstratlib::strategies::covered_call"){.mod}
:   Covered Call strategy implementation

[custom](custom/index.html "mod optionstratlib::strategies::custom"){.mod}
:   Custom strategy implementation and utilities

[default](default/index.html "mod optionstratlib::strategies::default"){.mod}
:   Default implementation for strategies

[delta_neutral](delta_neutral/index.html "mod optionstratlib::strategies::delta_neutral"){.mod}
:   Delta-neutral strategy implementation and utilities

[graph](graph/index.html "mod optionstratlib::strategies::graph"){.mod}
:   The `graph` module provides functionality for creating, managing,
    and manipulating graph data structures. Common use cases include
    representing networks, dependency graphs, and other graph-based
    relationships.

[iron_butterfly](iron_butterfly/index.html "mod optionstratlib::strategies::iron_butterfly"){.mod}
:   Iron Butterfly strategy implementation

[iron_condor](iron_condor/index.html "mod optionstratlib::strategies::iron_condor"){.mod}
:   Iron Condor strategy implementation

[long_butterfly_spread](long_butterfly_spread/index.html "mod optionstratlib::strategies::long_butterfly_spread"){.mod}
:   Butterfly Spread strategy implementation

[long_call](long_call/index.html "mod optionstratlib::strategies::long_call"){.mod}
:   Long Call strategy implementation

[long_put](long_put/index.html "mod optionstratlib::strategies::long_put"){.mod}
:   Long Put strategy implementation

[long_straddle](long_straddle/index.html "mod optionstratlib::strategies::long_straddle"){.mod}
:   Long Straddle strategy implementation

[long_strangle](long_strangle/index.html "mod optionstratlib::strategies::long_strangle"){.mod}
:   Strangle strategy implementation

[macros](macros/index.html "mod optionstratlib::strategies::macros"){.mod}
:   Macros for options strategies Simpler macro to test that a strategy
    implements all required traits

[poor_mans_covered_call](poor_mans_covered_call/index.html "mod optionstratlib::strategies::poor_mans_covered_call"){.mod}
:   Poor Man's Covered Call strategy implementation

[probabilities](probabilities/index.html "mod optionstratlib::strategies::probabilities"){.mod}
:   Probability calculations for options strategies

[protective_put](protective_put/index.html "mod optionstratlib::strategies::protective_put"){.mod}
:   Protective Put strategy implementation

[short_butterfly_spread](short_butterfly_spread/index.html "mod optionstratlib::strategies::short_butterfly_spread"){.mod}
:   Short Call strategy implementation

[short_call](short_call/index.html "mod optionstratlib::strategies::short_call"){.mod}
:   Short Call strategy implementation

[short_put](short_put/index.html "mod optionstratlib::strategies::short_put"){.mod}
:   Short Put strategy implementation

[short_straddle](short_straddle/index.html "mod optionstratlib::strategies::short_straddle"){.mod}
:   Short Straddle strategy implementation

[short_strangle](short_strangle/index.html "mod optionstratlib::strategies::short_strangle"){.mod}
:   Short Strangle strategy implementation

[utils](utils/index.html "mod optionstratlib::strategies::utils"){.mod}
:   Utility functions for options calculations and analysis Price Range
    Utilities

## Structs[§](#structs){.anchor} {#structs .section-header}

[StrategyRequest](struct.StrategyRequest.html "struct optionstratlib::strategies::StrategyRequest"){.struct}
:   A request structure for creating and analyzing options trading
    strategies.

## Traits[§](#traits){.anchor} {#traits .section-header}

[StrategyConstructor](trait.StrategyConstructor.html "trait optionstratlib::strategies::StrategyConstructor"){.trait}
:   Defines a common interface for constructing financial option
    strategies from collections of option positions.
:::::::::
::::::::::
