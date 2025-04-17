:::::::::::::::::::::::::::::::::::::::: width-limiter
::::::::::::::::::::::::::::::::::::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../../index.html)::[strategies](../index.html)::[delta_neutral](index.html)
:::

# Trait [DeltaNeutrality]{.trait}Copy item path

[[Source](../../../src/optionstratlib/strategies/delta_neutral/model.rs.html#317-755){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub trait DeltaNeutrality:
    Greeks
    + Positionable
    + Strategies {
    // Provided methods
    fn delta_neutrality(&self) -> Result<DeltaInfo, GreeksError> { ... }
    fn is_delta_neutral(&self) -> bool { ... }
    fn get_atm_strike(&self) -> Result<Positive, StrategyError> { ... }
    fn delta_adjustments(&self) -> Result<Vec<DeltaAdjustment>, GreeksError> { ... }
    fn apply_delta_adjustments(
        &mut self,
        action: Option<Action>,
    ) -> Result<(), Box<dyn Error>> { ... }
    fn apply_single_adjustment(
        &mut self,
        adjustment: &DeltaAdjustment,
    ) -> Result<(), Box<dyn Error>> { ... }
    fn adjust_underlying_position(
        &mut self,
        _quantity: Positive,
        _side: Side,
    ) -> Result<(), Box<dyn Error>> { ... }
    fn adjust_option_position(
        &mut self,
        quantity: Decimal,
        strike: &Positive,
        option_type: &OptionStyle,
        side: &Side,
    ) -> Result<(), Box<dyn Error>> { ... }
}
```

Expand description

::: docblock
A trait that provides functionality for managing and maintaining delta
neutrality in trading strategies.

This trait extends the `Greeks` trait and introduces methods to
calculate net delta, check neutrality status, suggest adjustments, and
generate delta adjustments for a trading strategy. It implements key
concepts needed to manage the delta exposure efficiently.

## [§](#methods){.doc-anchor}Methods

- `calculate_net_delta`: Calculates the net delta of the trading
  strategy and provides detailed delta-related information.
- `is_delta_neutral`: Determines if the strategy is delta-neutral within
  a specified threshold.
- `suggest_delta_adjustments`: Suggests potential actions to achieve
  delta neutrality.
- `generate_delta_reducing_adjustments`: Produces adjustments required
  to reduce a positive delta.
- `get_atm_strike`: Retrieves the ATM (At-The-Money) strike price
  closest to the current underlying asset price.
:::

## Provided Methods[§](#provided-methods){.anchor} {#provided-methods .section-header}

::::::::::::::::::: methods
::: {#method.delta_neutrality .section .method}
[Source](../../../src/optionstratlib/strategies/delta_neutral/model.rs.html#329-353){.src
.rightside}

#### fn [delta_neutrality](#method.delta_neutrality){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[DeltaInfo](struct.DeltaInfo.html "struct optionstratlib::strategies::delta_neutral::DeltaInfo"){.struct}, [GreeksError](../../error/greeks/enum.GreeksError.html "enum optionstratlib::error::greeks::GreeksError"){.enum}\> {#fn-delta_neutralityself---resultdeltainfo-greekserror .code-header}
:::

::: docblock
Calculates the net delta of the strategy and provides detailed
information.

##### [§](#returns){.doc-anchor}Returns

A `DeltaInfo` struct containing:

- The net delta of the strategy.
- Individual deltas of all components in the strategy.
- Whether the strategy is considered delta neutral.
- Threshold for neutrality determination.
- The current price of the underlying asset.

This provides an overview of the delta position and helps in determining
adjustments.
:::

::: {#method.is_delta_neutral .section .method}
[Source](../../../src/optionstratlib/strategies/delta_neutral/model.rs.html#364-369){.src
.rightside}

#### fn [is_delta_neutral](#method.is_delta_neutral){.fn}(&self) -\> [bool](https://doc.rust-lang.org/1.86.0/std/primitive.bool.html){.primitive} {#fn-is_delta_neutralself---bool .code-header}
:::

::: docblock
Checks if the strategy is delta neutral within the specified threshold.

##### [§](#arguments){.doc-anchor}Arguments

- `threshold` - A `Decimal` value representing the maximum allowed
  deviation from ideal delta neutrality.

##### [§](#returns-1){.doc-anchor}Returns

A boolean (`true` or `false`):

- `true` if the absolute value of the net delta is within the threshold.
- `false` otherwise.
:::

::: {#method.get_atm_strike .section .method}
[Source](../../../src/optionstratlib/strategies/delta_neutral/model.rs.html#393-395){.src
.rightside}

#### fn [get_atm_strike](#method.get_atm_strike){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, [StrategyError](../../error/strategies/enum.StrategyError.html "enum optionstratlib::error::strategies::StrategyError"){.enum}\> {#fn-get_atm_strikeself---resultpositive-strategyerror .code-header}
:::

::: docblock
##### [§](#get_atm_strike){.doc-anchor}get_atm_strike

Returns the at-the-money (ATM) strike price for a strategy by obtaining
the underlying asset's price.

An at-the-money strike is a strike price that is equal (or very close)
to the current market price of the underlying asset. This is often used
as a reference point for constructing option strategies.

###### [§](#returns-2){.doc-anchor}Returns

- `Result<Positive, StrategyError>` - The underlying price as a
  `Positive` value wrapped in a `Result`. Returns an error if retrieving
  the underlying price fails.

###### [§](#errors){.doc-anchor}Errors

This function may return a `StrategyError` if the call to
`get_underlying_price()` fails.

###### [§](#notes){.doc-anchor}Notes

This implementation assumes that the ATM strike is exactly equal to the
current price of the underlying asset. In practice, the actual ATM
strike might be the nearest available strike price offered by the
exchange.
:::

::: {#method.delta_adjustments .section .method}
[Source](../../../src/optionstratlib/strategies/delta_neutral/model.rs.html#410-520){.src
.rightside}

#### fn [delta_adjustments](#method.delta_adjustments){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Vec](https://doc.rust-lang.org/1.86.0/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<[DeltaAdjustment](enum.DeltaAdjustment.html "enum optionstratlib::strategies::delta_neutral::DeltaAdjustment"){.enum}\>, [GreeksError](../../error/greeks/enum.GreeksError.html "enum optionstratlib::error::greeks::GreeksError"){.enum}\> {#fn-delta_adjustmentsself---resultvecdeltaadjustment-greekserror .code-header}
:::

::: docblock
Calculates required position adjustments to maintain delta neutrality

##### [§](#arguments-1){.doc-anchor}Arguments

None - Uses internal position state

##### [§](#returns-3){.doc-anchor}Returns

- `Result<Vec<DeltaAdjustment>, GreeksError>` - Vector of suggested
  position adjustments or error if calculations fail

##### [§](#notes-1){.doc-anchor}Notes

- Uses DELTA_THRESHOLD to determine if adjustments are needed
- Suggests opposite positions to neutralize current delta exposure
- Accounts for both option style (Put/Call) and position side
  (Long/Short)
:::

::: {#method.apply_delta_adjustments .section .method}
[Source](../../../src/optionstratlib/strategies/delta_neutral/model.rs.html#552-600){.src
.rightside}

#### fn [apply_delta_adjustments](#method.apply_delta_adjustments){.fn}( &mut self, action: [Option](https://doc.rust-lang.org/1.86.0/core/option/enum.Option.html "enum core::option::Option"){.enum}\<[Action](../../model/types/enum.Action.html "enum optionstratlib::model::types::Action"){.enum}\>, ) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[()](https://doc.rust-lang.org/1.86.0/std/primitive.unit.html){.primitive}, [Box](https://doc.rust-lang.org/1.86.0/alloc/boxed/struct.Box.html "struct alloc::boxed::Box"){.struct}\<dyn [Error](https://doc.rust-lang.org/1.86.0/core/error/trait.Error.html "trait core::error::Error"){.trait}\>\> {#fn-apply_delta_adjustments-mut-self-action-optionaction---result-boxdyn-error .code-header}
:::

::: docblock
##### [§](#apply-delta-adjustments){.doc-anchor}Apply Delta Adjustments

Applies delta-neutralizing adjustments to the current strategy based on
calculated delta imbalances. This function ensures that the strategy
remains delta-neutral by executing the appropriate position adjustments.

###### [§](#parameters){.doc-anchor}Parameters

- `action`: Optional filtering parameter that specifies which type of
  adjustments to apply:
  - `Some(Action::Buy)`: Only apply options buying adjustments
  - `Some(Action::Sell)`: Only apply options selling adjustments
  - `None`: Apply all adjustment types, including paired adjustments

###### [§](#process){.doc-anchor}Process

1.  Calculates the current delta neutrality status of the strategy
2.  If the strategy is already delta-neutral (within the configured
    threshold), returns early
3.  Determines necessary adjustments to achieve delta neutrality
4.  Applies appropriate adjustments based on the specified action
    filter:
    - BuyOptions adjustments when Action::Buy is specified
    - SellOptions adjustments when Action::Sell is specified
    - All adjustments including paired SameSize adjustments when no
      action is specified

###### [§](#returns-4){.doc-anchor}Returns

- `Result<(), Box<dyn Error>>` - Success if adjustments were applied
  successfully, or an error if any adjustment operations failed

###### [§](#notes-2){.doc-anchor}Notes

- The function uses the strategy's internal delta_neutrality() and
  delta_adjustments() methods to determine the current state and
  required actions
- SameSize adjustments are only applied when no specific action filter
  is provided
- Incompatible adjustments for the specified action are skipped with a
  debug message
:::

::: {#method.apply_single_adjustment .section .method}
[Source](../../../src/optionstratlib/strategies/delta_neutral/model.rs.html#632-664){.src
.rightside}

#### fn [apply_single_adjustment](#method.apply_single_adjustment){.fn}( &mut self, adjustment: &[DeltaAdjustment](enum.DeltaAdjustment.html "enum optionstratlib::strategies::delta_neutral::DeltaAdjustment"){.enum}, ) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[()](https://doc.rust-lang.org/1.86.0/std/primitive.unit.html){.primitive}, [Box](https://doc.rust-lang.org/1.86.0/alloc/boxed/struct.Box.html "struct alloc::boxed::Box"){.struct}\<dyn [Error](https://doc.rust-lang.org/1.86.0/core/error/trait.Error.html "trait core::error::Error"){.trait}\>\> {#fn-apply_single_adjustment-mut-self-adjustment-deltaadjustment---result-boxdyn-error .code-header}
:::

::: docblock
##### [§](#apply-single-adjustment){.doc-anchor}Apply Single Adjustment

Applies a single delta adjustment to the current position or strategy.

This method processes a single `DeltaAdjustment` and modifies the
object's state accordingly. It handles different types of adjustments
that can be made to maintain or achieve delta neutrality in an options
strategy.

###### [§](#parameters-1){.doc-anchor}Parameters

- `adjustment` - A reference to the `DeltaAdjustment` to apply, which
  can be one of several variants specifying different types of position
  adjustments.

###### [§](#returns-5){.doc-anchor}Returns

- `Result<(), Box<dyn Error>>` - Returns `Ok(())` if the adjustment was
  applied successfully, or an `Error` if something went wrong during the
  process.

###### [§](#supported-adjustments){.doc-anchor}Supported Adjustments

- `BuyOptions` - Adds option contracts to the position with the
  specified parameters.
- `SellOptions` - Removes option contracts from the position with the
  specified parameters.
- `SameSize` - Currently not supported at the nested level (logs a debug
  message).
- Other variants - Currently not implemented (logs a debug message).

###### [§](#notes-3){.doc-anchor}Notes

The actual position adjustment is performed by the
`adjust_option_position` method, which is called with positive
quantities for buying options and negative quantities for selling
options.
:::

::: {#method.adjust_underlying_position .section .method}
[Source](../../../src/optionstratlib/strategies/delta_neutral/model.rs.html#697-705){.src
.rightside}

#### fn [adjust_underlying_position](#method.adjust_underlying_position){.fn}( &mut self, \_quantity: [Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, \_side: [Side](../../model/types/enum.Side.html "enum optionstratlib::model::types::Side"){.enum}, ) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[()](https://doc.rust-lang.org/1.86.0/std/primitive.unit.html){.primitive}, [Box](https://doc.rust-lang.org/1.86.0/alloc/boxed/struct.Box.html "struct alloc::boxed::Box"){.struct}\<dyn [Error](https://doc.rust-lang.org/1.86.0/core/error/trait.Error.html "trait core::error::Error"){.trait}\>\> {#fn-adjust_underlying_position-mut-self-_quantity-positive-_side-side---result-boxdyn-error .code-header}
:::

::: docblock
##### [§](#adjust-underlying-position){.doc-anchor}Adjust Underlying Position

Adjusts the quantity of the underlying asset held in a position based on
the specified side (Long/Short) and quantity.

This method allows modifying the underlying asset position to achieve
specific strategy goals, such as:

- Adjusting delta exposure for hedging purposes
- Implementing delta-neutral adjustments
- Rebalancing positions after market movements
- Executing rolling strategies

###### [§](#parameters-2){.doc-anchor}Parameters

- `_quantity`: The amount of the underlying asset to adjust, represented
  as a `Positive` value
- `_side`: The direction of the adjustment - `Side::Long` to increase
  the position, `Side::Short` to decrease it

###### [§](#returns-6){.doc-anchor}Returns

- `Result<(), Box<dyn Error>>`: Returns Ok(()) on successful adjustment,
  or an Error if the adjustment fails

###### [§](#details){.doc-anchor}Details

When adjusting the underlying position, the method will take into
account current market conditions, available capital, and potentially
transaction costs to determine the optimal execution strategy.

###### [§](#example-use-cases){.doc-anchor}Example Use Cases

- Delta hedging an options position
- Rebalancing after a significant price move in the underlying
- Implementing a dynamic hedging strategy
- Gradually liquidating or building a position

###### [§](#note){.doc-anchor}Note

Adjustments to the underlying position directly affect the overall
strategy's risk profile, particularly its delta exposure and potential
profit/loss characteristics.
:::

::: {#method.adjust_option_position .section .method}
[Source](../../../src/optionstratlib/strategies/delta_neutral/model.rs.html#734-754){.src
.rightside}

#### fn [adjust_option_position](#method.adjust_option_position){.fn}( &mut self, quantity: Decimal, strike: &[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, option_type: &[OptionStyle](../../model/types/enum.OptionStyle.html "enum optionstratlib::model::types::OptionStyle"){.enum}, side: &[Side](../../model/types/enum.Side.html "enum optionstratlib::model::types::Side"){.enum}, ) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[()](https://doc.rust-lang.org/1.86.0/std/primitive.unit.html){.primitive}, [Box](https://doc.rust-lang.org/1.86.0/alloc/boxed/struct.Box.html "struct alloc::boxed::Box"){.struct}\<dyn [Error](https://doc.rust-lang.org/1.86.0/core/error/trait.Error.html "trait core::error::Error"){.trait}\>\> {#fn-adjust_option_position-mut-self-quantity-decimal-strike-positive-option_type-optionstyle-side-side---result-boxdyn-error .code-header}
:::

::: docblock
##### [§](#adjust-option-position){.doc-anchor}Adjust Option Position

Modifies the quantity of an existing option position in a trading
strategy.

This method adjusts the quantity of an existing option position that
matches the provided option type, side, and strike price. If the
position is found, its quantity is increased or decreased by the
specified amount. If the position is not found, an error is returned.

###### [§](#parameters-3){.doc-anchor}Parameters

- `quantity`: The decimal amount by which to adjust the position.
  Positive values increase the position size, while negative values
  decrease it.
- `strike`: The strike price of the option position to adjust.
- `option_type`: The option style (Call or Put) of the position to
  adjust.
- `side`: The side (Long or Short) of the position to adjust.

###### [§](#returns-7){.doc-anchor}Returns

- `Ok(())` if the position was successfully adjusted.
- `Err(PositionError::ValidationError)` if the position was not found.

###### [§](#errors-1){.doc-anchor}Errors

Returns a boxed error if:

- The specified position does not exist in the strategy
- The `get_position` or `modify_position` methods fail
:::
:::::::::::::::::::

## Implementors[§](#implementors){.anchor} {#implementors .section-header}

:::::::::::::::::: {#implementors-list}
::: {#impl-DeltaNeutrality-for-BearCallSpread .section .impl}
[Source](../../../src/optionstratlib/strategies/bear_call_spread.rs.html#781){.src
.rightside}[§](#impl-DeltaNeutrality-for-BearCallSpread){.anchor}

### impl [DeltaNeutrality](trait.DeltaNeutrality.html "trait optionstratlib::strategies::delta_neutral::DeltaNeutrality"){.trait} for [BearCallSpread](../bear_call_spread/struct.BearCallSpread.html "struct optionstratlib::strategies::bear_call_spread::BearCallSpread"){.struct} {#impl-deltaneutrality-for-bearcallspread .code-header}
:::

::: {#impl-DeltaNeutrality-for-BearPutSpread .section .impl}
[Source](../../../src/optionstratlib/strategies/bear_put_spread.rs.html#771){.src
.rightside}[§](#impl-DeltaNeutrality-for-BearPutSpread){.anchor}

### impl [DeltaNeutrality](trait.DeltaNeutrality.html "trait optionstratlib::strategies::delta_neutral::DeltaNeutrality"){.trait} for [BearPutSpread](../bear_put_spread/struct.BearPutSpread.html "struct optionstratlib::strategies::bear_put_spread::BearPutSpread"){.struct} {#impl-deltaneutrality-for-bearputspread .code-header}
:::

::: {#impl-DeltaNeutrality-for-BullCallSpread .section .impl}
[Source](../../../src/optionstratlib/strategies/bull_call_spread.rs.html#784){.src
.rightside}[§](#impl-DeltaNeutrality-for-BullCallSpread){.anchor}

### impl [DeltaNeutrality](trait.DeltaNeutrality.html "trait optionstratlib::strategies::delta_neutral::DeltaNeutrality"){.trait} for [BullCallSpread](../bull_call_spread/struct.BullCallSpread.html "struct optionstratlib::strategies::bull_call_spread::BullCallSpread"){.struct} {#impl-deltaneutrality-for-bullcallspread .code-header}
:::

::: {#impl-DeltaNeutrality-for-BullPutSpread .section .impl}
[Source](../../../src/optionstratlib/strategies/bull_put_spread.rs.html#885){.src
.rightside}[§](#impl-DeltaNeutrality-for-BullPutSpread){.anchor}

### impl [DeltaNeutrality](trait.DeltaNeutrality.html "trait optionstratlib::strategies::delta_neutral::DeltaNeutrality"){.trait} for [BullPutSpread](../bull_put_spread/struct.BullPutSpread.html "struct optionstratlib::strategies::bull_put_spread::BullPutSpread"){.struct} {#impl-deltaneutrality-for-bullputspread .code-header}
:::

::: {#impl-DeltaNeutrality-for-LongButterflySpread .section .impl}
[Source](../../../src/optionstratlib/strategies/butterfly_spread.rs.html#976){.src
.rightside}[§](#impl-DeltaNeutrality-for-LongButterflySpread){.anchor}

### impl [DeltaNeutrality](trait.DeltaNeutrality.html "trait optionstratlib::strategies::delta_neutral::DeltaNeutrality"){.trait} for [LongButterflySpread](../butterfly_spread/struct.LongButterflySpread.html "struct optionstratlib::strategies::butterfly_spread::LongButterflySpread"){.struct} {#impl-deltaneutrality-for-longbutterflyspread .code-header}
:::

::: {#impl-DeltaNeutrality-for-ShortButterflySpread .section .impl}
[Source](../../../src/optionstratlib/strategies/butterfly_spread.rs.html#1943){.src
.rightside}[§](#impl-DeltaNeutrality-for-ShortButterflySpread){.anchor}

### impl [DeltaNeutrality](trait.DeltaNeutrality.html "trait optionstratlib::strategies::delta_neutral::DeltaNeutrality"){.trait} for [ShortButterflySpread](../butterfly_spread/struct.ShortButterflySpread.html "struct optionstratlib::strategies::butterfly_spread::ShortButterflySpread"){.struct} {#impl-deltaneutrality-for-shortbutterflyspread .code-header}
:::

::: {#impl-DeltaNeutrality-for-CallButterfly .section .impl}
[Source](../../../src/optionstratlib/strategies/call_butterfly.rs.html#952){.src
.rightside}[§](#impl-DeltaNeutrality-for-CallButterfly){.anchor}

### impl [DeltaNeutrality](trait.DeltaNeutrality.html "trait optionstratlib::strategies::delta_neutral::DeltaNeutrality"){.trait} for [CallButterfly](../call_butterfly/struct.CallButterfly.html "struct optionstratlib::strategies::call_butterfly::CallButterfly"){.struct} {#impl-deltaneutrality-for-callbutterfly .code-header}
:::

::: {#impl-DeltaNeutrality-for-CustomStrategy .section .impl}
[Source](../../../src/optionstratlib/strategies/custom.rs.html#755){.src
.rightside}[§](#impl-DeltaNeutrality-for-CustomStrategy){.anchor}

### impl [DeltaNeutrality](trait.DeltaNeutrality.html "trait optionstratlib::strategies::delta_neutral::DeltaNeutrality"){.trait} for [CustomStrategy](../custom/struct.CustomStrategy.html "struct optionstratlib::strategies::custom::CustomStrategy"){.struct} {#impl-deltaneutrality-for-customstrategy .code-header}
:::

::: {#impl-DeltaNeutrality-for-IronButterfly .section .impl}
[Source](../../../src/optionstratlib/strategies/iron_butterfly.rs.html#1018){.src
.rightside}[§](#impl-DeltaNeutrality-for-IronButterfly){.anchor}

### impl [DeltaNeutrality](trait.DeltaNeutrality.html "trait optionstratlib::strategies::delta_neutral::DeltaNeutrality"){.trait} for [IronButterfly](../iron_butterfly/struct.IronButterfly.html "struct optionstratlib::strategies::iron_butterfly::IronButterfly"){.struct} {#impl-deltaneutrality-for-ironbutterfly .code-header}
:::

::: {#impl-DeltaNeutrality-for-IronCondor .section .impl}
[Source](../../../src/optionstratlib/strategies/iron_condor.rs.html#1053){.src
.rightside}[§](#impl-DeltaNeutrality-for-IronCondor){.anchor}

### impl [DeltaNeutrality](trait.DeltaNeutrality.html "trait optionstratlib::strategies::delta_neutral::DeltaNeutrality"){.trait} for [IronCondor](../iron_condor/struct.IronCondor.html "struct optionstratlib::strategies::iron_condor::IronCondor"){.struct} {#impl-deltaneutrality-for-ironcondor .code-header}
:::

::: {#impl-DeltaNeutrality-for-PoorMansCoveredCall .section .impl}
[Source](../../../src/optionstratlib/strategies/poor_mans_covered_call.rs.html#809){.src
.rightside}[§](#impl-DeltaNeutrality-for-PoorMansCoveredCall){.anchor}

### impl [DeltaNeutrality](trait.DeltaNeutrality.html "trait optionstratlib::strategies::delta_neutral::DeltaNeutrality"){.trait} for [PoorMansCoveredCall](../poor_mans_covered_call/struct.PoorMansCoveredCall.html "struct optionstratlib::strategies::poor_mans_covered_call::PoorMansCoveredCall"){.struct} {#impl-deltaneutrality-for-poormanscoveredcall .code-header}
:::

::: {#impl-DeltaNeutrality-for-LongStraddle .section .impl}
[Source](../../../src/optionstratlib/strategies/straddle.rs.html#1616){.src
.rightside}[§](#impl-DeltaNeutrality-for-LongStraddle){.anchor}

### impl [DeltaNeutrality](trait.DeltaNeutrality.html "trait optionstratlib::strategies::delta_neutral::DeltaNeutrality"){.trait} for [LongStraddle](../straddle/struct.LongStraddle.html "struct optionstratlib::strategies::straddle::LongStraddle"){.struct} {#impl-deltaneutrality-for-longstraddle .code-header}
:::

::: {#impl-DeltaNeutrality-for-ShortStraddle .section .impl}
[Source](../../../src/optionstratlib/strategies/straddle.rs.html#836){.src
.rightside}[§](#impl-DeltaNeutrality-for-ShortStraddle){.anchor}

### impl [DeltaNeutrality](trait.DeltaNeutrality.html "trait optionstratlib::strategies::delta_neutral::DeltaNeutrality"){.trait} for [ShortStraddle](../straddle/struct.ShortStraddle.html "struct optionstratlib::strategies::straddle::ShortStraddle"){.struct} {#impl-deltaneutrality-for-shortstraddle .code-header}
:::

::: {#impl-DeltaNeutrality-for-LongStrangle .section .impl}
[Source](../../../src/optionstratlib/strategies/strangle.rs.html#1920){.src
.rightside}[§](#impl-DeltaNeutrality-for-LongStrangle){.anchor}

### impl [DeltaNeutrality](trait.DeltaNeutrality.html "trait optionstratlib::strategies::delta_neutral::DeltaNeutrality"){.trait} for [LongStrangle](../strangle/struct.LongStrangle.html "struct optionstratlib::strategies::strangle::LongStrangle"){.struct} {#impl-deltaneutrality-for-longstrangle .code-header}
:::

::: {#impl-DeltaNeutrality-for-ShortStrangle .section .impl}
[Source](../../../src/optionstratlib/strategies/strangle.rs.html#932){.src
.rightside}[§](#impl-DeltaNeutrality-for-ShortStrangle){.anchor}

### impl [DeltaNeutrality](trait.DeltaNeutrality.html "trait optionstratlib::strategies::delta_neutral::DeltaNeutrality"){.trait} for [ShortStrangle](../strangle/struct.ShortStrangle.html "struct optionstratlib::strategies::strangle::ShortStrangle"){.struct} {#impl-deltaneutrality-for-shortstrangle .code-header}
:::
::::::::::::::::::
:::::::::::::::::::::::::::::::::::::::
::::::::::::::::::::::::::::::::::::::::
