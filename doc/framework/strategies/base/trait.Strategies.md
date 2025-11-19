:::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::: width-limiter
::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../../index.html)::[strategies](../index.html)::[base](index.html)
:::

# Trait [Strategies]{.trait} Copy item path

[[Source](../../../src/optionstratlib/strategies/base.rs.html#760-1105){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub trait Strategies:
    Validable
    + Positionable
    + BreakEvenable
    + BasicAble {
Show 17 methods    // Provided methods
    fn get_volume(&mut self) -> Result<Positive, StrategyError> { ... }
    fn get_max_profit(&self) -> Result<Positive, StrategyError> { ... }
    fn get_max_profit_mut(&mut self) -> Result<Positive, StrategyError> { ... }
    fn get_max_loss(&self) -> Result<Positive, StrategyError> { ... }
    fn get_max_loss_mut(&mut self) -> Result<Positive, StrategyError> { ... }
    fn get_total_cost(&self) -> Result<Positive, PositionError> { ... }
    fn get_net_cost(&self) -> Result<Decimal, PositionError> { ... }
    fn get_net_premium_received(&self) -> Result<Positive, StrategyError> { ... }
    fn get_fees(&self) -> Result<Positive, StrategyError> { ... }
    fn get_profit_area(&self) -> Result<Decimal, StrategyError> { ... }
    fn get_profit_ratio(&self) -> Result<Decimal, StrategyError> { ... }
    fn get_range_to_show(&self) -> Result<(Positive, Positive), StrategyError> { ... }
    fn get_best_range_to_show(
        &self,
        step: Positive,
    ) -> Result<Vec<Positive>, StrategyError> { ... }
    fn get_max_min_strikes(&self) -> Result<(Positive, Positive), StrategyError> { ... }
    fn get_range_of_profit(&self) -> Result<Positive, StrategyError> { ... }
    fn roll_in(
        &mut self,
        _position: &Position,
    ) -> Result<HashMap<Action, Trade>, StrategyError> { ... }
    fn roll_out(
        &mut self,
        _position: &Position,
    ) -> Result<HashMap<Action, Trade>, StrategyError> { ... }
}
```

Expand description

::: docblock
Defines a set of strategies for options trading. Provides methods for
calculating key metrics such as profit/loss, cost, break-even points,
and price ranges. Implementations of this trait must also implement the
`Validable`, `Positionable`, and `BreakEvenable` traits.
:::

## Provided Methods[§](#provided-methods){.anchor} {#provided-methods .section-header}

::::::::::::::::::::::::::::::::::::: methods
::: {#method.get_volume .section .method}
[Source](../../../src/optionstratlib/strategies/base.rs.html#778-785){.src
.rightside}

#### fn [get_volume](#method.get_volume){.fn}(&mut self) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, [StrategyError](../../error/strategies/enum.StrategyError.html "enum optionstratlib::error::strategies::StrategyError"){.enum}\> {#fn-get_volumemut-self---resultpositive-strategyerror .code-header}
:::

::: docblock
Retrieves the current volume of the strategy as sum of quantities in
their positions

This function returns the volume as a `Positive` value, ensuring that
the result is always greater than zero. If the method fails to retrieve
the volume, an error of type `StrategyError` is returned.

##### [§](#returns){.doc-anchor}Returns

- `Ok(Positive)` - The current volume as a positive numeric value.
- `Err(StrategyError)` - An error indicating why the volume could not be
  retrieved.

##### [§](#errors){.doc-anchor}Errors

This function may return a `StrategyError` in cases such as:

- Internal issues within the strategy's calculation or storage.
- Other implementation-specific failures.
:::

::: {#method.get_max_profit .section .method}
[Source](../../../src/optionstratlib/strategies/base.rs.html#793-798){.src
.rightside}

#### fn [get_max_profit](#method.get_max_profit){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, [StrategyError](../../error/strategies/enum.StrategyError.html "enum optionstratlib::error::strategies::StrategyError"){.enum}\> {#fn-get_max_profitself---resultpositive-strategyerror .code-header}
:::

::: docblock
Calculates the maximum possible profit for the strategy. The default
implementation returns an error indicating that the operation is not
supported.

##### [§](#returns-1){.doc-anchor}Returns

- `Ok(Positive)` - The maximum possible profit.
- `Err(StrategyError)` - If the operation is not supported for this
  strategy.
:::

::: {#method.get_max_profit_mut .section .method}
[Source](../../../src/optionstratlib/strategies/base.rs.html#806-808){.src
.rightside}

#### fn [get_max_profit_mut](#method.get_max_profit_mut){.fn}(&mut self) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, [StrategyError](../../error/strategies/enum.StrategyError.html "enum optionstratlib::error::strategies::StrategyError"){.enum}\> {#fn-get_max_profit_mutmut-self---resultpositive-strategyerror .code-header}
:::

::: docblock
Calculates the maximum possible profit for the strategy, potentially
using an iterative approach. Defaults to calling `max_profit`.

##### [§](#returns-2){.doc-anchor}Returns

- `Ok(Positive)` - The maximum possible profit.
- `Err(StrategyError)` - If the operation is not supported for this
  strategy.
:::

::: {#method.get_max_loss .section .method}
[Source](../../../src/optionstratlib/strategies/base.rs.html#816-821){.src
.rightside}

#### fn [get_max_loss](#method.get_max_loss){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, [StrategyError](../../error/strategies/enum.StrategyError.html "enum optionstratlib::error::strategies::StrategyError"){.enum}\> {#fn-get_max_lossself---resultpositive-strategyerror .code-header}
:::

::: docblock
Calculates the maximum possible loss for the strategy. The default
implementation returns an error indicating that the operation is not
supported.

##### [§](#returns-3){.doc-anchor}Returns

- `Ok(Positive)` - The maximum possible loss.
- `Err(StrategyError)` - If the operation is not supported for this
  strategy.
:::

::: {#method.get_max_loss_mut .section .method}
[Source](../../../src/optionstratlib/strategies/base.rs.html#829-831){.src
.rightside}

#### fn [get_max_loss_mut](#method.get_max_loss_mut){.fn}(&mut self) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, [StrategyError](../../error/strategies/enum.StrategyError.html "enum optionstratlib::error::strategies::StrategyError"){.enum}\> {#fn-get_max_loss_mutmut-self---resultpositive-strategyerror .code-header}
:::

::: docblock
Calculates the maximum possible loss for the strategy, potentially using
an iterative approach. Defaults to calling `max_loss`.

##### [§](#returns-4){.doc-anchor}Returns

- `Ok(Positive)` - The maximum possible loss.
- `Err(StrategyError)` - If the operation is not supported for this
  strategy.
:::

::: {#method.get_total_cost .section .method}
[Source](../../../src/optionstratlib/strategies/base.rs.html#838-845){.src
.rightside}

#### fn [get_total_cost](#method.get_total_cost){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, [PositionError](../../error/position/enum.PositionError.html "enum optionstratlib::error::position::PositionError"){.enum}\> {#fn-get_total_costself---resultpositive-positionerror .code-header}
:::

::: docblock
Calculates the total cost of the strategy, which is the sum of the
absolute cost of all positions.

##### [§](#returns-5){.doc-anchor}Returns

- `Ok(Positive)` - The total cost of the strategy.
- `Err(PositionError)` - If there is an error retrieving the positions.
:::

::: {#method.get_net_cost .section .method}
[Source](../../../src/optionstratlib/strategies/base.rs.html#853-860){.src
.rightside}

#### fn [get_net_cost](#method.get_net_cost){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Decimal](../../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}, [PositionError](../../error/position/enum.PositionError.html "enum optionstratlib::error::position::PositionError"){.enum}\> {#fn-get_net_costself---resultdecimal-positionerror .code-header}
:::

::: docblock
Calculates the net cost of the strategy, which is the sum of the costs
of all positions, considering premiums paid and received.

##### [§](#returns-6){.doc-anchor}Returns

- `Ok(Decimal)` - The net cost of the strategy.
- `Err(PositionError)` - If there is an error retrieving the positions.
:::

::: {#method.get_net_premium_received .section .method}
[Source](../../../src/optionstratlib/strategies/base.rs.html#868-884){.src
.rightside}

#### fn [get_net_premium_received](#method.get_net_premium_received){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, [StrategyError](../../error/strategies/enum.StrategyError.html "enum optionstratlib::error::strategies::StrategyError"){.enum}\> {#fn-get_net_premium_receivedself---resultpositive-strategyerror .code-header}
:::

::: docblock
Calculates the net premium received for the strategy. This is the total
premium received from short positions minus the total premium paid for
long positions. If the result is negative, it returns zero.

##### [§](#returns-7){.doc-anchor}Returns

- `Ok(Positive)` - The net premium received.
- `Err(StrategyError)` - If there is an error retrieving the positions.
:::

::: {#method.get_fees .section .method}
[Source](../../../src/optionstratlib/strategies/base.rs.html#891-908){.src
.rightside}

#### fn [get_fees](#method.get_fees){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, [StrategyError](../../error/strategies/enum.StrategyError.html "enum optionstratlib::error::strategies::StrategyError"){.enum}\> {#fn-get_feesself---resultpositive-strategyerror .code-header}
:::

::: docblock
Calculates the total fees for the strategy by summing the fees of all
positions.

##### [§](#returns-8){.doc-anchor}Returns

- `Ok(Positive)` - The total fees.
- `Err(StrategyError)` - If there is an error retrieving positions or
  calculating fees.
:::

::: {#method.get_profit_area .section .method}
[Source](../../../src/optionstratlib/strategies/base.rs.html#916-921){.src
.rightside}

#### fn [get_profit_area](#method.get_profit_area){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Decimal](../../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}, [StrategyError](../../error/strategies/enum.StrategyError.html "enum optionstratlib::error::strategies::StrategyError"){.enum}\> {#fn-get_profit_areaself---resultdecimal-strategyerror .code-header}
:::

::: docblock
Calculates the profit area for the strategy. The default implementation
returns an error indicating that the operation is not supported.

##### [§](#returns-9){.doc-anchor}Returns

- `Ok(Decimal)` - The profit area.
- `Err(StrategyError)` - If the operation is not supported.
:::

::: {#method.get_profit_ratio .section .method}
[Source](../../../src/optionstratlib/strategies/base.rs.html#929-934){.src
.rightside}

#### fn [get_profit_ratio](#method.get_profit_ratio){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Decimal](../../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}, [StrategyError](../../error/strategies/enum.StrategyError.html "enum optionstratlib::error::strategies::StrategyError"){.enum}\> {#fn-get_profit_ratioself---resultdecimal-strategyerror .code-header}
:::

::: docblock
Calculates the profit ratio for the strategy. The default implementation
returns an error indicating that the operation is not supported.

##### [§](#returns-10){.doc-anchor}Returns

- `Ok(Decimal)` - The profit ratio.
- `Err(StrategyError)` - If the operation is not supported.
:::

::: {#method.get_range_to_show .section .method}
[Source](../../../src/optionstratlib/strategies/base.rs.html#944-968){.src
.rightside}

#### fn [get_range_to_show](#method.get_range_to_show){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<([Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, [Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}), [StrategyError](../../error/strategies/enum.StrategyError.html "enum optionstratlib::error::strategies::StrategyError"){.enum}\> {#fn-get_range_to_showself---resultpositive-positive-strategyerror .code-header}
:::

::: docblock
Determines the price range to display for the strategy's profit/loss
graph. This range is calculated based on the break-even points, the
underlying price, and the maximum and minimum strike prices. The range
is expanded by applying `STRIKE_PRICE_LOWER_BOUND_MULTIPLIER` and
`STRIKE_PRICE_UPPER_BOUND_MULTIPLIER` to the minimum and maximum prices
respectively.

##### [§](#returns-11){.doc-anchor}Returns

- `Ok((Positive, Positive))` - A tuple containing the start and end
  prices of the range.
- `Err(StrategyError)` - If there is an error retrieving necessary data
  for the calculation.
:::

::: {#method.get_best_range_to_show .section .method}
[Source](../../../src/optionstratlib/strategies/base.rs.html#975-978){.src
.rightside}

#### fn [get_best_range_to_show](#method.get_best_range_to_show){.fn}( &self, step: [Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, ) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Vec](https://doc.rust-lang.org/1.91.1/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\>, [StrategyError](../../error/strategies/enum.StrategyError.html "enum optionstratlib::error::strategies::StrategyError"){.enum}\> {#fn-get_best_range_to_show-self-step-positive---resultvecpositive-strategyerror .code-header}
:::

::: docblock
Generates a vector of prices within the display range, using a specified
step.

##### [§](#returns-12){.doc-anchor}Returns

- `Ok(Vec<Positive>)` - A vector of prices.
- `Err(StrategyError)` - If there is an error calculating the display
  range.
:::

::: {#method.get_max_min_strikes .section .method}
[Source](../../../src/optionstratlib/strategies/base.rs.html#986-1018){.src
.rightside}

#### fn [get_max_min_strikes](#method.get_max_min_strikes){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<([Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, [Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}), [StrategyError](../../error/strategies/enum.StrategyError.html "enum optionstratlib::error::strategies::StrategyError"){.enum}\> {#fn-get_max_min_strikesself---resultpositive-positive-strategyerror .code-header}
:::

::: docblock
Returns the minimum and maximum strike prices from the positions in the
strategy. Considers underlying price when applicable, ensuring the
returned range includes it.

##### [§](#returns-13){.doc-anchor}Returns

- `Ok((Positive, Positive))` - A tuple containing the minimum and
  maximum strike prices.
- `Err(StrategyError)` - If no strikes are found or if an error occurs
  retrieving positions.
:::

::: {#method.get_range_of_profit .section .method}
[Source](../../../src/optionstratlib/strategies/base.rs.html#1026-1040){.src
.rightside}

#### fn [get_range_of_profit](#method.get_range_of_profit){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, [StrategyError](../../error/strategies/enum.StrategyError.html "enum optionstratlib::error::strategies::StrategyError"){.enum}\> {#fn-get_range_of_profitself---resultpositive-strategyerror .code-header}
:::

::: docblock
Calculates the range of prices where the strategy is profitable, based
on the break-even points.

##### [§](#returns-14){.doc-anchor}Returns:

- `Ok(Positive)` - The difference between the highest and lowest
  break-even points. Returns `Positive::INFINITY` if there is only one
  break-even point.
- `Err(StrategyError)` - if there are no break-even points.
:::

::: {#method.roll_in .section .method}
[Source](../../../src/optionstratlib/strategies/base.rs.html#1064-1066){.src
.rightside}

#### fn [roll_in](#method.roll_in){.fn}( &mut self, \_position: &[Position](../../model/position/struct.Position.html "struct optionstratlib::model::position::Position"){.struct}, ) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[HashMap](https://doc.rust-lang.org/1.91.1/std/collections/hash/map/struct.HashMap.html "struct std::collections::hash::map::HashMap"){.struct}\<[Action](../../model/types/enum.Action.html "enum optionstratlib::model::types::Action"){.enum}, [Trade](../../model/struct.Trade.html "struct optionstratlib::model::Trade"){.struct}\>, [StrategyError](../../error/strategies/enum.StrategyError.html "enum optionstratlib::error::strategies::StrategyError"){.enum}\> {#fn-roll_in-mut-self-_position-position---resulthashmapaction-trade-strategyerror .code-header}
:::

::: docblock
Attempts to execute the roll-in functionality for the strategy.

##### [§](#parameters){.doc-anchor}Parameters

- `&mut self`: A mutable reference to the current instance of the
  strategy.
- `_position: &Position`: A reference to the `Position` object,
  representing the current position in the market. This parameter is
  currently unused in the implementation.

##### [§](#returns-15){.doc-anchor}Returns

- `Result<HashMap<Action, Trade>, StrategyError>`:
  - `Ok(HashMap<Action, Trade>)`: On success, a map of actions to
    trades, representing the changes made during the roll-in process.
  - `Err(StrategyError)`: If an error occurs during the roll-in
    operation.

##### [§](#errors-1){.doc-anchor}Errors

- Returns a `StrategyError` if the roll-in operation fails (not
  currently implemented).

##### [§](#panics){.doc-anchor}Panics

- This function will panic if called, as it is currently unimplemented.

##### [§](#note){.doc-anchor}Note

- This method is not implemented and will panic upon invocation. Future
  implementations should define the specific logic for handling the
  roll-in operation for the associated strategy.
:::

::: {#method.roll_out .section .method}
[Source](../../../src/optionstratlib/strategies/base.rs.html#1102-1104){.src
.rightside}

#### fn [roll_out](#method.roll_out){.fn}( &mut self, \_position: &[Position](../../model/position/struct.Position.html "struct optionstratlib::model::position::Position"){.struct}, ) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[HashMap](https://doc.rust-lang.org/1.91.1/std/collections/hash/map/struct.HashMap.html "struct std::collections::hash::map::HashMap"){.struct}\<[Action](../../model/types/enum.Action.html "enum optionstratlib::model::types::Action"){.enum}, [Trade](../../model/struct.Trade.html "struct optionstratlib::model::Trade"){.struct}\>, [StrategyError](../../error/strategies/enum.StrategyError.html "enum optionstratlib::error::strategies::StrategyError"){.enum}\> {#fn-roll_out-mut-self-_position-position---resulthashmapaction-trade-strategyerror .code-header}
:::

::: docblock
Executes the roll-out strategy for the provided position.

This function is intended to evaluate and execute trading actions based
on the given `Position`. It returns a mapping of `Action` to `Trade`
that represents the proposed trades resulting from the strategy.
However, this method currently is not implemented and will panic if
called.

##### [§](#arguments){.doc-anchor}Arguments

- `_position` - A reference to a `Position` object which represents the
  current state of a trading position.

##### [§](#returns-16){.doc-anchor}Returns

- `Result<HashMap<Action, Trade>, StrategyError>` - A `Result` object
  containing:
  - `Ok(HashMap<Action, Trade>)` with the mapping of actions to trades
    if successfully implemented in the future.
  - `Err(StrategyError)` if an error occurs during execution (currently
    always unimplemented).

##### [§](#errors-2){.doc-anchor}Errors

- Returns an error of type `StrategyError` if the strategy encounters
  execution issues (in this case, always unimplemented).

##### [§](#panics-1){.doc-anchor}Panics

This function will panic with a message "roll_out is not implemented for
this strategy" since it is currently not implemented.

##### [§](#note-1){.doc-anchor}Note

Until implemented, calling this method will result in a runtime panic.
:::
:::::::::::::::::::::::::::::::::::::

## Implementors[§](#implementors){.anchor} {#implementors .section-header}

:::::::::::::::::::::: {#implementors-list}
::: {#impl-Strategies-for-BearCallSpread .section .impl}
[Source](../../../src/optionstratlib/strategies/bear_call_spread.rs.html#582-623){.src
.rightside}[§](#impl-Strategies-for-BearCallSpread){.anchor}

### impl [Strategies](trait.Strategies.html "trait optionstratlib::strategies::base::Strategies"){.trait} for [BearCallSpread](../bear_call_spread/struct.BearCallSpread.html "struct optionstratlib::strategies::bear_call_spread::BearCallSpread"){.struct} {#impl-strategies-for-bearcallspread .code-header}
:::

::: {#impl-Strategies-for-BearPutSpread .section .impl}
[Source](../../../src/optionstratlib/strategies/bear_put_spread.rs.html#572-611){.src
.rightside}[§](#impl-Strategies-for-BearPutSpread){.anchor}

### impl [Strategies](trait.Strategies.html "trait optionstratlib::strategies::base::Strategies"){.trait} for [BearPutSpread](../bear_put_spread/struct.BearPutSpread.html "struct optionstratlib::strategies::bear_put_spread::BearPutSpread"){.struct} {#impl-strategies-for-bearputspread .code-header}
:::

::: {#impl-Strategies-for-BullCallSpread .section .impl}
[Source](../../../src/optionstratlib/strategies/bull_call_spread.rs.html#584-623){.src
.rightside}[§](#impl-Strategies-for-BullCallSpread){.anchor}

### impl [Strategies](trait.Strategies.html "trait optionstratlib::strategies::base::Strategies"){.trait} for [BullCallSpread](../bull_call_spread/struct.BullCallSpread.html "struct optionstratlib::strategies::bull_call_spread::BullCallSpread"){.struct} {#impl-strategies-for-bullcallspread .code-header}
:::

::: {#impl-Strategies-for-BullPutSpread .section .impl}
[Source](../../../src/optionstratlib/strategies/bull_put_spread.rs.html#582-627){.src
.rightside}[§](#impl-Strategies-for-BullPutSpread){.anchor}

### impl [Strategies](trait.Strategies.html "trait optionstratlib::strategies::base::Strategies"){.trait} for [BullPutSpread](../bull_put_spread/struct.BullPutSpread.html "struct optionstratlib::strategies::bull_put_spread::BullPutSpread"){.struct} {#impl-strategies-for-bullputspread .code-header}
:::

::: {#impl-Strategies-for-CallButterfly .section .impl}
[Source](../../../src/optionstratlib/strategies/call_butterfly.rs.html#683-727){.src
.rightside}[§](#impl-Strategies-for-CallButterfly){.anchor}

### impl [Strategies](trait.Strategies.html "trait optionstratlib::strategies::base::Strategies"){.trait} for [CallButterfly](../call_butterfly/struct.CallButterfly.html "struct optionstratlib::strategies::call_butterfly::CallButterfly"){.struct} {#impl-strategies-for-callbutterfly .code-header}
:::

::: {#impl-Strategies-for-CustomStrategy .section .impl}
[Source](../../../src/optionstratlib/strategies/custom.rs.html#602-740){.src
.rightside}[§](#impl-Strategies-for-CustomStrategy){.anchor}

### impl [Strategies](trait.Strategies.html "trait optionstratlib::strategies::base::Strategies"){.trait} for [CustomStrategy](../custom/struct.CustomStrategy.html "struct optionstratlib::strategies::custom::CustomStrategy"){.struct} {#impl-strategies-for-customstrategy .code-header}
:::

::: {#impl-Strategies-for-IronButterfly .section .impl}
[Source](../../../src/optionstratlib/strategies/iron_butterfly.rs.html#771-825){.src
.rightside}[§](#impl-Strategies-for-IronButterfly){.anchor}

### impl [Strategies](trait.Strategies.html "trait optionstratlib::strategies::base::Strategies"){.trait} for [IronButterfly](../iron_butterfly/struct.IronButterfly.html "struct optionstratlib::strategies::iron_butterfly::IronButterfly"){.struct} {#impl-strategies-for-ironbutterfly .code-header}
:::

::: {#impl-Strategies-for-IronCondor .section .impl}
[Source](../../../src/optionstratlib/strategies/iron_condor.rs.html#793-848){.src
.rightside}[§](#impl-Strategies-for-IronCondor){.anchor}

### impl [Strategies](trait.Strategies.html "trait optionstratlib::strategies::base::Strategies"){.trait} for [IronCondor](../iron_condor/struct.IronCondor.html "struct optionstratlib::strategies::iron_condor::IronCondor"){.struct} {#impl-strategies-for-ironcondor .code-header}
:::

::: {#impl-Strategies-for-LongButterflySpread .section .impl}
[Source](../../../src/optionstratlib/strategies/long_butterfly_spread.rs.html#723-778){.src
.rightside}[§](#impl-Strategies-for-LongButterflySpread){.anchor}

### impl [Strategies](trait.Strategies.html "trait optionstratlib::strategies::base::Strategies"){.trait} for [LongButterflySpread](../long_butterfly_spread/struct.LongButterflySpread.html "struct optionstratlib::strategies::long_butterfly_spread::LongButterflySpread"){.struct} {#impl-strategies-for-longbutterflyspread .code-header}
:::

::: {#impl-Strategies-for-LongCall .section .impl}
[Source](../../../src/optionstratlib/strategies/long_call.rs.html#274-313){.src
.rightside}[§](#impl-Strategies-for-LongCall){.anchor}

### impl [Strategies](trait.Strategies.html "trait optionstratlib::strategies::base::Strategies"){.trait} for [LongCall](../long_call/struct.LongCall.html "struct optionstratlib::strategies::long_call::LongCall"){.struct} {#impl-strategies-for-longcall .code-header}
:::

::: {#impl-Strategies-for-LongPut .section .impl}
[Source](../../../src/optionstratlib/strategies/long_put.rs.html#271-310){.src
.rightside}[§](#impl-Strategies-for-LongPut){.anchor}

### impl [Strategies](trait.Strategies.html "trait optionstratlib::strategies::base::Strategies"){.trait} for [LongPut](../long_put/struct.LongPut.html "struct optionstratlib::strategies::long_put::LongPut"){.struct} {#impl-strategies-for-longput .code-header}
:::

::: {#impl-Strategies-for-LongStraddle .section .impl}
[Source](../../../src/optionstratlib/strategies/long_straddle.rs.html#598-623){.src
.rightside}[§](#impl-Strategies-for-LongStraddle){.anchor}

### impl [Strategies](trait.Strategies.html "trait optionstratlib::strategies::base::Strategies"){.trait} for [LongStraddle](../long_straddle/struct.LongStraddle.html "struct optionstratlib::strategies::long_straddle::LongStraddle"){.struct} {#impl-strategies-for-longstraddle .code-header}
:::

::: {#impl-Strategies-for-LongStrangle .section .impl}
[Source](../../../src/optionstratlib/strategies/long_strangle.rs.html#614-665){.src
.rightside}[§](#impl-Strategies-for-LongStrangle){.anchor}

### impl [Strategies](trait.Strategies.html "trait optionstratlib::strategies::base::Strategies"){.trait} for [LongStrangle](../long_strangle/struct.LongStrangle.html "struct optionstratlib::strategies::long_strangle::LongStrangle"){.struct} {#impl-strategies-for-longstrangle .code-header}
:::

::: {#impl-Strategies-for-PoorMansCoveredCall .section .impl}
[Source](../../../src/optionstratlib/strategies/poor_mans_covered_call.rs.html#611-655){.src
.rightside}[§](#impl-Strategies-for-PoorMansCoveredCall){.anchor}

### impl [Strategies](trait.Strategies.html "trait optionstratlib::strategies::base::Strategies"){.trait} for [PoorMansCoveredCall](../poor_mans_covered_call/struct.PoorMansCoveredCall.html "struct optionstratlib::strategies::poor_mans_covered_call::PoorMansCoveredCall"){.struct} {#impl-strategies-for-poormanscoveredcall .code-header}
:::

::: {#impl-Strategies-for-ShortButterflySpread .section .impl}
[Source](../../../src/optionstratlib/strategies/short_butterfly_spread.rs.html#708-756){.src
.rightside}[§](#impl-Strategies-for-ShortButterflySpread){.anchor}

### impl [Strategies](trait.Strategies.html "trait optionstratlib::strategies::base::Strategies"){.trait} for [ShortButterflySpread](../short_butterfly_spread/struct.ShortButterflySpread.html "struct optionstratlib::strategies::short_butterfly_spread::ShortButterflySpread"){.struct} {#impl-strategies-for-shortbutterflyspread .code-header}
:::

::: {#impl-Strategies-for-ShortCall .section .impl}
[Source](../../../src/optionstratlib/strategies/short_call.rs.html#286-321){.src
.rightside}[§](#impl-Strategies-for-ShortCall){.anchor}

### impl [Strategies](trait.Strategies.html "trait optionstratlib::strategies::base::Strategies"){.trait} for [ShortCall](../short_call/struct.ShortCall.html "struct optionstratlib::strategies::short_call::ShortCall"){.struct} {#impl-strategies-for-shortcall .code-header}
:::

::: {#impl-Strategies-for-ShortPut .section .impl}
[Source](../../../src/optionstratlib/strategies/short_put.rs.html#276-315){.src
.rightside}[§](#impl-Strategies-for-ShortPut){.anchor}

### impl [Strategies](trait.Strategies.html "trait optionstratlib::strategies::base::Strategies"){.trait} for [ShortPut](../short_put/struct.ShortPut.html "struct optionstratlib::strategies::short_put::ShortPut"){.struct} {#impl-strategies-for-shortput .code-header}
:::

::: {#impl-Strategies-for-ShortStraddle .section .impl}
[Source](../../../src/optionstratlib/strategies/short_straddle.rs.html#620-648){.src
.rightside}[§](#impl-Strategies-for-ShortStraddle){.anchor}

### impl [Strategies](trait.Strategies.html "trait optionstratlib::strategies::base::Strategies"){.trait} for [ShortStraddle](../short_straddle/struct.ShortStraddle.html "struct optionstratlib::strategies::short_straddle::ShortStraddle"){.struct} {#impl-strategies-for-shortstraddle .code-header}
:::

::: {#impl-Strategies-for-ShortStrangle .section .impl}
[Source](../../../src/optionstratlib/strategies/short_strangle.rs.html#675-851){.src
.rightside}[§](#impl-Strategies-for-ShortStrangle){.anchor}

### impl [Strategies](trait.Strategies.html "trait optionstratlib::strategies::base::Strategies"){.trait} for [ShortStrangle](../short_strangle/struct.ShortStrangle.html "struct optionstratlib::strategies::short_strangle::ShortStrangle"){.struct} {#impl-strategies-for-shortstrangle .code-header}
:::
::::::::::::::::::::::
:::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
