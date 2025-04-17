:::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::: width-limiter
::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../../index.html)::[strategies](../index.html)::[base](index.html)
:::

# Trait [Strategies]{.trait}Copy item path

[[Source](../../../src/optionstratlib/strategies/base.rs.html#347-697){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub trait Strategies:
    Validable
    + Positionable
    + BreakEvenable {
Show 20 methods    // Provided methods
    fn get_underlying_price(&self) -> Positive { ... }
    fn set_underlying_price(
        &mut self,
        _price: &Positive,
    ) -> Result<(), StrategyError> { ... }
    fn volume(&mut self) -> Result<Positive, StrategyError> { ... }
    fn max_profit(&self) -> Result<Positive, StrategyError> { ... }
    fn max_profit_iter(&mut self) -> Result<Positive, StrategyError> { ... }
    fn max_loss(&self) -> Result<Positive, StrategyError> { ... }
    fn max_loss_iter(&mut self) -> Result<Positive, StrategyError> { ... }
    fn total_cost(&self) -> Result<Positive, PositionError> { ... }
    fn net_cost(&self) -> Result<Decimal, PositionError> { ... }
    fn net_premium_received(&self) -> Result<Positive, StrategyError> { ... }
    fn fees(&self) -> Result<Positive, StrategyError> { ... }
    fn profit_area(&self) -> Result<Decimal, StrategyError> { ... }
    fn profit_ratio(&self) -> Result<Decimal, StrategyError> { ... }
    fn range_to_show(&self) -> Result<(Positive, Positive), StrategyError> { ... }
    fn best_range_to_show(
        &self,
        step: Positive,
    ) -> Result<Vec<Positive>, StrategyError> { ... }
    fn strikes(&self) -> Result<Vec<Positive>, StrategyError> { ... }
    fn max_min_strikes(&self) -> Result<(Positive, Positive), StrategyError> { ... }
    fn range_of_profit(&self) -> Result<Positive, StrategyError> { ... }
    fn expiration_dates(&self) -> Result<Vec<ExpirationDate>, StrategyError> { ... }
    fn set_expiration_date(
        &mut self,
        _expiration_date: ExpirationDate,
    ) -> Result<(), StrategyError> { ... }
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

::::::::::::::::::::::::::::::::::::::::::: methods
::: {#method.get_underlying_price .section .method}
[Source](../../../src/optionstratlib/strategies/base.rs.html#353-355){.src
.rightside}

#### fn [get_underlying_price](#method.get_underlying_price){.fn}(&self) -\> [Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#fn-get_underlying_priceself---positive .code-header}
:::

::: docblock
Retrieves the underlying asset price. The default implementation panics
with a message indicating that the underlying price is not applicable
for the strategy.

##### [§](#panics){.doc-anchor}Panics

Panics if the underlying price is not applicable for this strategy.
:::

::: {#method.set_underlying_price .section .method}
[Source](../../../src/optionstratlib/strategies/base.rs.html#376-378){.src
.rightside}

#### fn [set_underlying_price](#method.set_underlying_price){.fn}( &mut self, \_price: &[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, ) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[()](https://doc.rust-lang.org/1.86.0/std/primitive.unit.html){.primitive}, [StrategyError](../../error/strategies/enum.StrategyError.html "enum optionstratlib::error::strategies::StrategyError"){.enum}\> {#fn-set_underlying_price-mut-self-_price-positive---result-strategyerror .code-header}
:::

::: docblock
Sets the underlying price for a strategy.

##### [§](#arguments){.doc-anchor}Arguments

- `price` - A reference to a `Positive` value representing the new price
  to set

##### [§](#returns){.doc-anchor}Returns

A `Result` that will always panic with an informative message

##### [§](#errors){.doc-anchor}Errors

This function always panics as it's not applicable for the current
strategy type. It's implemented this way to fulfill a trait requirement
but intentionally prevents usage for strategies where underlying price
setting doesn't make sense.

##### [§](#panics-1){.doc-anchor}Panics

Always panics with the message "Set Underlying price is not applicable
for this strategy"
:::

::: {#method.volume .section .method}
[Source](../../../src/optionstratlib/strategies/base.rs.html#393-395){.src
.rightside}

#### fn [volume](#method.volume){.fn}(&mut self) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, [StrategyError](../../error/strategies/enum.StrategyError.html "enum optionstratlib::error::strategies::StrategyError"){.enum}\> {#fn-volumemut-self---resultpositive-strategyerror .code-header}
:::

::: docblock
Returns the volume for this strategy.

##### [§](#returns-1){.doc-anchor}Returns

A `Result` containing a `Positive` value representing the volume, or a
`StrategyError` if the operation is not applicable.

##### [§](#errors-1){.doc-anchor}Errors

Returns a `StrategyError` of type `OperationError` if volume is not
applicable for this strategy.

##### [§](#panics-2){.doc-anchor}Panics

This function will panic with the message "volume is not applicable for
this strategy".
:::

::: {#method.max_profit .section .method}
[Source](../../../src/optionstratlib/strategies/base.rs.html#403-408){.src
.rightside}

#### fn [max_profit](#method.max_profit){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, [StrategyError](../../error/strategies/enum.StrategyError.html "enum optionstratlib::error::strategies::StrategyError"){.enum}\> {#fn-max_profitself---resultpositive-strategyerror .code-header}
:::

::: docblock
Calculates the maximum possible profit for the strategy. The default
implementation returns an error indicating that the operation is not
supported.

##### [§](#returns-2){.doc-anchor}Returns

- `Ok(Positive)` - The maximum possible profit.
- `Err(StrategyError)` - If the operation is not supported for this
  strategy.
:::

::: {#method.max_profit_iter .section .method}
[Source](../../../src/optionstratlib/strategies/base.rs.html#416-418){.src
.rightside}

#### fn [max_profit_iter](#method.max_profit_iter){.fn}(&mut self) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, [StrategyError](../../error/strategies/enum.StrategyError.html "enum optionstratlib::error::strategies::StrategyError"){.enum}\> {#fn-max_profit_itermut-self---resultpositive-strategyerror .code-header}
:::

::: docblock
Calculates the maximum possible profit for the strategy, potentially
using an iterative approach. Defaults to calling `max_profit`.

##### [§](#returns-3){.doc-anchor}Returns

- `Ok(Positive)` - The maximum possible profit.
- `Err(StrategyError)` - If the operation is not supported for this
  strategy.
:::

::: {#method.max_loss .section .method}
[Source](../../../src/optionstratlib/strategies/base.rs.html#426-431){.src
.rightside}

#### fn [max_loss](#method.max_loss){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, [StrategyError](../../error/strategies/enum.StrategyError.html "enum optionstratlib::error::strategies::StrategyError"){.enum}\> {#fn-max_lossself---resultpositive-strategyerror .code-header}
:::

::: docblock
Calculates the maximum possible loss for the strategy. The default
implementation returns an error indicating that the operation is not
supported.

##### [§](#returns-4){.doc-anchor}Returns

- `Ok(Positive)` - The maximum possible loss.
- `Err(StrategyError)` - If the operation is not supported for this
  strategy.
:::

::: {#method.max_loss_iter .section .method}
[Source](../../../src/optionstratlib/strategies/base.rs.html#439-441){.src
.rightside}

#### fn [max_loss_iter](#method.max_loss_iter){.fn}(&mut self) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, [StrategyError](../../error/strategies/enum.StrategyError.html "enum optionstratlib::error::strategies::StrategyError"){.enum}\> {#fn-max_loss_itermut-self---resultpositive-strategyerror .code-header}
:::

::: docblock
Calculates the maximum possible loss for the strategy, potentially using
an iterative approach. Defaults to calling `max_loss`.

##### [§](#returns-5){.doc-anchor}Returns

- `Ok(Positive)` - The maximum possible loss.
- `Err(StrategyError)` - If the operation is not supported for this
  strategy.
:::

::: {#method.total_cost .section .method}
[Source](../../../src/optionstratlib/strategies/base.rs.html#448-455){.src
.rightside}

#### fn [total_cost](#method.total_cost){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, [PositionError](../../error/position/enum.PositionError.html "enum optionstratlib::error::position::PositionError"){.enum}\> {#fn-total_costself---resultpositive-positionerror .code-header}
:::

::: docblock
Calculates the total cost of the strategy, which is the sum of the
absolute cost of all positions.

##### [§](#returns-6){.doc-anchor}Returns

- `Ok(Positive)` - The total cost of the strategy.
- `Err(PositionError)` - If there is an error retrieving the positions.
:::

::: {#method.net_cost .section .method}
[Source](../../../src/optionstratlib/strategies/base.rs.html#463-470){.src
.rightside}

#### fn [net_cost](#method.net_cost){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<Decimal, [PositionError](../../error/position/enum.PositionError.html "enum optionstratlib::error::position::PositionError"){.enum}\> {#fn-net_costself---resultdecimal-positionerror .code-header}
:::

::: docblock
Calculates the net cost of the strategy, which is the sum of the costs
of all positions, considering premiums paid and received.

##### [§](#returns-7){.doc-anchor}Returns

- `Ok(Decimal)` - The net cost of the strategy.
- `Err(PositionError)` - If there is an error retrieving the positions.
:::

::: {#method.net_premium_received .section .method}
[Source](../../../src/optionstratlib/strategies/base.rs.html#478-494){.src
.rightside}

#### fn [net_premium_received](#method.net_premium_received){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, [StrategyError](../../error/strategies/enum.StrategyError.html "enum optionstratlib::error::strategies::StrategyError"){.enum}\> {#fn-net_premium_receivedself---resultpositive-strategyerror .code-header}
:::

::: docblock
Calculates the net premium received for the strategy. This is the total
premium received from short positions minus the total premium paid for
long positions. If the result is negative, it returns zero.

##### [§](#returns-8){.doc-anchor}Returns

- `Ok(Positive)` - The net premium received.
- `Err(StrategyError)` - If there is an error retrieving the positions.
:::

::: {#method.fees .section .method}
[Source](../../../src/optionstratlib/strategies/base.rs.html#501-518){.src
.rightside}

#### fn [fees](#method.fees){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, [StrategyError](../../error/strategies/enum.StrategyError.html "enum optionstratlib::error::strategies::StrategyError"){.enum}\> {#fn-feesself---resultpositive-strategyerror .code-header}
:::

::: docblock
Calculates the total fees for the strategy by summing the fees of all
positions.

##### [§](#returns-9){.doc-anchor}Returns

- `Ok(Positive)` - The total fees.
- `Err(StrategyError)` - If there is an error retrieving positions or
  calculating fees.
:::

::: {#method.profit_area .section .method}
[Source](../../../src/optionstratlib/strategies/base.rs.html#526-531){.src
.rightside}

#### fn [profit_area](#method.profit_area){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<Decimal, [StrategyError](../../error/strategies/enum.StrategyError.html "enum optionstratlib::error::strategies::StrategyError"){.enum}\> {#fn-profit_areaself---resultdecimal-strategyerror .code-header}
:::

::: docblock
Calculates the profit area for the strategy. The default implementation
returns an error indicating that the operation is not supported.

##### [§](#returns-10){.doc-anchor}Returns

- `Ok(Decimal)` - The profit area.
- `Err(StrategyError)` - If the operation is not supported.
:::

::: {#method.profit_ratio .section .method}
[Source](../../../src/optionstratlib/strategies/base.rs.html#539-544){.src
.rightside}

#### fn [profit_ratio](#method.profit_ratio){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<Decimal, [StrategyError](../../error/strategies/enum.StrategyError.html "enum optionstratlib::error::strategies::StrategyError"){.enum}\> {#fn-profit_ratioself---resultdecimal-strategyerror .code-header}
:::

::: docblock
Calculates the profit ratio for the strategy. The default implementation
returns an error indicating that the operation is not supported.

##### [§](#returns-11){.doc-anchor}Returns

- `Ok(Decimal)` - The profit ratio.
- `Err(StrategyError)` - If the operation is not supported.
:::

::: {#method.range_to_show .section .method}
[Source](../../../src/optionstratlib/strategies/base.rs.html#554-578){.src
.rightside}

#### fn [range_to_show](#method.range_to_show){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<([Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, [Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}), [StrategyError](../../error/strategies/enum.StrategyError.html "enum optionstratlib::error::strategies::StrategyError"){.enum}\> {#fn-range_to_showself---resultpositive-positive-strategyerror .code-header}
:::

::: docblock
Determines the price range to display for the strategy's profit/loss
graph. This range is calculated based on the break-even points, the
underlying price, and the maximum and minimum strike prices. The range
is expanded by applying `STRIKE_PRICE_LOWER_BOUND_MULTIPLIER` and
`STRIKE_PRICE_UPPER_BOUND_MULTIPLIER` to the minimum and maximum prices
respectively.

##### [§](#returns-12){.doc-anchor}Returns

- `Ok((Positive, Positive))` - A tuple containing the start and end
  prices of the range.
- `Err(StrategyError)` - If there is an error retrieving necessary data
  for the calculation.
:::

::: {#method.best_range_to_show .section .method}
[Source](../../../src/optionstratlib/strategies/base.rs.html#585-588){.src
.rightside}

#### fn [best_range_to_show](#method.best_range_to_show){.fn}( &self, step: [Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, ) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Vec](https://doc.rust-lang.org/1.86.0/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\>, [StrategyError](../../error/strategies/enum.StrategyError.html "enum optionstratlib::error::strategies::StrategyError"){.enum}\> {#fn-best_range_to_show-self-step-positive---resultvecpositive-strategyerror .code-header}
:::

::: docblock
Generates a vector of prices within the display range, using a specified
step.

##### [§](#returns-13){.doc-anchor}Returns

- `Ok(Vec<Positive>)` - A vector of prices.
- `Err(StrategyError)` - If there is an error calculating the display
  range.
:::

::: {#method.strikes .section .method}
[Source](../../../src/optionstratlib/strategies/base.rs.html#595-615){.src
.rightside}

#### fn [strikes](#method.strikes){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Vec](https://doc.rust-lang.org/1.86.0/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\>, [StrategyError](../../error/strategies/enum.StrategyError.html "enum optionstratlib::error::strategies::StrategyError"){.enum}\> {#fn-strikesself---resultvecpositive-strategyerror .code-header}
:::

::: docblock
Returns a sorted vector of unique strike prices for all positions in the
strategy.

##### [§](#returns-14){.doc-anchor}Returns

- `Ok(Vec<Positive>)` - A vector of strike prices.
- `Err(StrategyError)` - If there are no positions or an error occurs
  retrieving them.
:::

::: {#method.max_min_strikes .section .method}
[Source](../../../src/optionstratlib/strategies/base.rs.html#623-651){.src
.rightside}

#### fn [max_min_strikes](#method.max_min_strikes){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<([Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, [Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}), [StrategyError](../../error/strategies/enum.StrategyError.html "enum optionstratlib::error::strategies::StrategyError"){.enum}\> {#fn-max_min_strikesself---resultpositive-positive-strategyerror .code-header}
:::

::: docblock
Returns the minimum and maximum strike prices from the positions in the
strategy. Considers underlying price when applicable, ensuring the
returned range includes it.

##### [§](#returns-15){.doc-anchor}Returns

- `Ok((Positive, Positive))` - A tuple containing the minimum and
  maximum strike prices.
- `Err(StrategyError)` - If no strikes are found or if an error occurs
  retrieving positions.
:::

::: {#method.range_of_profit .section .method}
[Source](../../../src/optionstratlib/strategies/base.rs.html#659-673){.src
.rightside}

#### fn [range_of_profit](#method.range_of_profit){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, [StrategyError](../../error/strategies/enum.StrategyError.html "enum optionstratlib::error::strategies::StrategyError"){.enum}\> {#fn-range_of_profitself---resultpositive-strategyerror .code-header}
:::

::: docblock
Calculates the range of prices where the strategy is profitable, based
on the break-even points.

##### [§](#returns-16){.doc-anchor}Returns:

- `Ok(Positive)` - The difference between the highest and lowest
  break-even points. Returns `Positive::INFINITY` if there is only one
  break-even point.
- `Err(StrategyError)` - if there are no break-even points.
:::

::: {#method.expiration_dates .section .method}
[Source](../../../src/optionstratlib/strategies/base.rs.html#680-682){.src
.rightside}

#### fn [expiration_dates](#method.expiration_dates){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Vec](https://doc.rust-lang.org/1.86.0/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<[ExpirationDate](../../model/types/enum.ExpirationDate.html "enum optionstratlib::model::types::ExpirationDate"){.enum}\>, [StrategyError](../../error/strategies/enum.StrategyError.html "enum optionstratlib::error::strategies::StrategyError"){.enum}\> {#fn-expiration_datesself---resultvecexpirationdate-strategyerror .code-header}
:::

::: docblock
Returns a vector of expiration dates for the strategy.

##### [§](#returns-17){.doc-anchor}Returns

- `Result<Vec<ExpirationDate>, StrategyError>` - A vector of expiration
  dates, or an error if not implemented for the specific strategy.
:::

::: {#method.set_expiration_date .section .method}
[Source](../../../src/optionstratlib/strategies/base.rs.html#691-696){.src
.rightside}

#### fn [set_expiration_date](#method.set_expiration_date){.fn}( &mut self, \_expiration_date: [ExpirationDate](../../model/types/enum.ExpirationDate.html "enum optionstratlib::model::types::ExpirationDate"){.enum}, ) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[()](https://doc.rust-lang.org/1.86.0/std/primitive.unit.html){.primitive}, [StrategyError](../../error/strategies/enum.StrategyError.html "enum optionstratlib::error::strategies::StrategyError"){.enum}\> {#fn-set_expiration_date-mut-self-_expiration_date-expirationdate---result-strategyerror .code-header}
:::

::: docblock
Sets the expiration date for the strategy.

##### [§](#arguments-1){.doc-anchor}Arguments

- `expiration_date` - The new expiration date.

##### [§](#returns-18){.doc-anchor}Returns

- `Result<(), StrategyError>` - An error if not implemented for the
  specific strategy.
:::
:::::::::::::::::::::::::::::::::::::::::::

## Implementors[§](#implementors){.anchor} {#implementors .section-header}

:::::::::::::::::: {#implementors-list}
::: {#impl-Strategies-for-BearCallSpread .section .impl}
[Source](../../../src/optionstratlib/strategies/bear_call_spread.rs.html#458-506){.src
.rightside}[§](#impl-Strategies-for-BearCallSpread){.anchor}

### impl [Strategies](trait.Strategies.html "trait optionstratlib::strategies::base::Strategies"){.trait} for [BearCallSpread](../bear_call_spread/struct.BearCallSpread.html "struct optionstratlib::strategies::bear_call_spread::BearCallSpread"){.struct} {#impl-strategies-for-bearcallspread .code-header}
:::

::: {#impl-Strategies-for-BearPutSpread .section .impl}
[Source](../../../src/optionstratlib/strategies/bear_put_spread.rs.html#438-484){.src
.rightside}[§](#impl-Strategies-for-BearPutSpread){.anchor}

### impl [Strategies](trait.Strategies.html "trait optionstratlib::strategies::base::Strategies"){.trait} for [BearPutSpread](../bear_put_spread/struct.BearPutSpread.html "struct optionstratlib::strategies::bear_put_spread::BearPutSpread"){.struct} {#impl-strategies-for-bearputspread .code-header}
:::

::: {#impl-Strategies-for-BullCallSpread .section .impl}
[Source](../../../src/optionstratlib/strategies/bull_call_spread.rs.html#447-493){.src
.rightside}[§](#impl-Strategies-for-BullCallSpread){.anchor}

### impl [Strategies](trait.Strategies.html "trait optionstratlib::strategies::base::Strategies"){.trait} for [BullCallSpread](../bull_call_spread/struct.BullCallSpread.html "struct optionstratlib::strategies::bull_call_spread::BullCallSpread"){.struct} {#impl-strategies-for-bullcallspread .code-header}
:::

::: {#impl-Strategies-for-BullPutSpread .section .impl}
[Source](../../../src/optionstratlib/strategies/bull_put_spread.rs.html#453-504){.src
.rightside}[§](#impl-Strategies-for-BullPutSpread){.anchor}

### impl [Strategies](trait.Strategies.html "trait optionstratlib::strategies::base::Strategies"){.trait} for [BullPutSpread](../bull_put_spread/struct.BullPutSpread.html "struct optionstratlib::strategies::bull_put_spread::BullPutSpread"){.struct} {#impl-strategies-for-bullputspread .code-header}
:::

::: {#impl-Strategies-for-LongButterflySpread .section .impl}
[Source](../../../src/optionstratlib/strategies/butterfly_spread.rs.html#533-595){.src
.rightside}[§](#impl-Strategies-for-LongButterflySpread){.anchor}

### impl [Strategies](trait.Strategies.html "trait optionstratlib::strategies::base::Strategies"){.trait} for [LongButterflySpread](../butterfly_spread/struct.LongButterflySpread.html "struct optionstratlib::strategies::butterfly_spread::LongButterflySpread"){.struct} {#impl-strategies-for-longbutterflyspread .code-header}
:::

::: {#impl-Strategies-for-ShortButterflySpread .section .impl}
[Source](../../../src/optionstratlib/strategies/butterfly_spread.rs.html#1509-1564){.src
.rightside}[§](#impl-Strategies-for-ShortButterflySpread){.anchor}

### impl [Strategies](trait.Strategies.html "trait optionstratlib::strategies::base::Strategies"){.trait} for [ShortButterflySpread](../butterfly_spread/struct.ShortButterflySpread.html "struct optionstratlib::strategies::butterfly_spread::ShortButterflySpread"){.struct} {#impl-strategies-for-shortbutterflyspread .code-header}
:::

::: {#impl-Strategies-for-CallButterfly .section .impl}
[Source](../../../src/optionstratlib/strategies/call_butterfly.rs.html#531-579){.src
.rightside}[§](#impl-Strategies-for-CallButterfly){.anchor}

### impl [Strategies](trait.Strategies.html "trait optionstratlib::strategies::base::Strategies"){.trait} for [CallButterfly](../call_butterfly/struct.CallButterfly.html "struct optionstratlib::strategies::call_butterfly::CallButterfly"){.struct} {#impl-strategies-for-callbutterfly .code-header}
:::

::: {#impl-Strategies-for-CustomStrategy .section .impl}
[Source](../../../src/optionstratlib/strategies/custom.rs.html#388-492){.src
.rightside}[§](#impl-Strategies-for-CustomStrategy){.anchor}

### impl [Strategies](trait.Strategies.html "trait optionstratlib::strategies::base::Strategies"){.trait} for [CustomStrategy](../custom/struct.CustomStrategy.html "struct optionstratlib::strategies::custom::CustomStrategy"){.struct} {#impl-strategies-for-customstrategy .code-header}
:::

::: {#impl-Strategies-for-IronButterfly .section .impl}
[Source](../../../src/optionstratlib/strategies/iron_butterfly.rs.html#592-650){.src
.rightside}[§](#impl-Strategies-for-IronButterfly){.anchor}

### impl [Strategies](trait.Strategies.html "trait optionstratlib::strategies::base::Strategies"){.trait} for [IronButterfly](../iron_butterfly/struct.IronButterfly.html "struct optionstratlib::strategies::iron_butterfly::IronButterfly"){.struct} {#impl-strategies-for-ironbutterfly .code-header}
:::

::: {#impl-Strategies-for-IronCondor .section .impl}
[Source](../../../src/optionstratlib/strategies/iron_condor.rs.html#601-660){.src
.rightside}[§](#impl-Strategies-for-IronCondor){.anchor}

### impl [Strategies](trait.Strategies.html "trait optionstratlib::strategies::base::Strategies"){.trait} for [IronCondor](../iron_condor/struct.IronCondor.html "struct optionstratlib::strategies::iron_condor::IronCondor"){.struct} {#impl-strategies-for-ironcondor .code-header}
:::

::: {#impl-Strategies-for-PoorMansCoveredCall .section .impl}
[Source](../../../src/optionstratlib/strategies/poor_mans_covered_call.rs.html#478-525){.src
.rightside}[§](#impl-Strategies-for-PoorMansCoveredCall){.anchor}

### impl [Strategies](trait.Strategies.html "trait optionstratlib::strategies::base::Strategies"){.trait} for [PoorMansCoveredCall](../poor_mans_covered_call/struct.PoorMansCoveredCall.html "struct optionstratlib::strategies::poor_mans_covered_call::PoorMansCoveredCall"){.struct} {#impl-strategies-for-poormanscoveredcall .code-header}
:::

::: {#impl-Strategies-for-LongStraddle .section .impl}
[Source](../../../src/optionstratlib/strategies/straddle.rs.html#1290-1319){.src
.rightside}[§](#impl-Strategies-for-LongStraddle){.anchor}

### impl [Strategies](trait.Strategies.html "trait optionstratlib::strategies::base::Strategies"){.trait} for [LongStraddle](../straddle/struct.LongStraddle.html "struct optionstratlib::strategies::straddle::LongStraddle"){.struct} {#impl-strategies-for-longstraddle .code-header}
:::

::: {#impl-Strategies-for-ShortStraddle .section .impl}
[Source](../../../src/optionstratlib/strategies/straddle.rs.html#485-519){.src
.rightside}[§](#impl-Strategies-for-ShortStraddle){.anchor}

### impl [Strategies](trait.Strategies.html "trait optionstratlib::strategies::base::Strategies"){.trait} for [ShortStraddle](../straddle/struct.ShortStraddle.html "struct optionstratlib::strategies::straddle::ShortStraddle"){.struct} {#impl-strategies-for-shortstraddle .code-header}
:::

::: {#impl-Strategies-for-LongStrangle .section .impl}
[Source](../../../src/optionstratlib/strategies/strangle.rs.html#1498-1570){.src
.rightside}[§](#impl-Strategies-for-LongStrangle){.anchor}

### impl [Strategies](trait.Strategies.html "trait optionstratlib::strategies::base::Strategies"){.trait} for [LongStrangle](../strangle/struct.LongStrangle.html "struct optionstratlib::strategies::strangle::LongStrangle"){.struct} {#impl-strategies-for-longstrangle .code-header}
:::

::: {#impl-Strategies-for-ShortStrangle .section .impl}
[Source](../../../src/optionstratlib/strategies/strangle.rs.html#476-565){.src
.rightside}[§](#impl-Strategies-for-ShortStrangle){.anchor}

### impl [Strategies](trait.Strategies.html "trait optionstratlib::strategies::base::Strategies"){.trait} for [ShortStrangle](../strangle/struct.ShortStrangle.html "struct optionstratlib::strategies::strangle::ShortStrangle"){.struct} {#impl-strategies-for-shortstrangle .code-header}
:::
::::::::::::::::::
:::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
