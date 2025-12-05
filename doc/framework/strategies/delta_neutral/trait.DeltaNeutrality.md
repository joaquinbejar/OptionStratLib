:::::::::::::::::::::::::::::::::::::::::::::: width-limiter
::::::::::::::::::::::::::::::::::::::::::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../../index.html)::[strategies](../index.html)::[delta_neutral](index.html)
:::

# Trait [DeltaNeutrality]{.trait} Copy item path

[[Source](../../../src/optionstratlib/strategies/delta_neutral/model.rs.html#334-971){.src}
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
    fn generate_delta_adjustments(
        &self,
        net_delta: Decimal,
        option_delta_per_contract: Decimal,
        option: &Options,
    ) -> Result<DeltaAdjustment, GreeksError> { ... }
    fn delta_adjustments(&self) -> Result<Vec<DeltaAdjustment>, GreeksError> { ... }
    fn apply_delta_adjustments(
        &mut self,
        action: Option<Action>,
    ) -> Result<(), StrategyError> { ... }
    fn apply_single_adjustment(
        &mut self,
        adjustment: &DeltaAdjustment,
    ) -> Result<(), StrategyError> { ... }
    fn adjust_option_position(
        &mut self,
        quantity: Decimal,
        strike: &Positive,
        option_type: &OptionStyle,
        side: &Side,
    ) -> Result<(), StrategyError> { ... }
    fn trade_from_delta_adjustment(
        &mut self,
        action: Action,
    ) -> Result<Vec<Trade>, StrategyError> { ... }
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

::::::::::::::::::::: methods
::: {#method.delta_neutrality .section .method}
[Source](../../../src/optionstratlib/strategies/delta_neutral/model.rs.html#346-371){.src
.rightside}

#### fn [delta_neutrality](#method.delta_neutrality){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[DeltaInfo](struct.DeltaInfo.html "struct optionstratlib::strategies::delta_neutral::DeltaInfo"){.struct}, [GreeksError](../../error/greeks/enum.GreeksError.html "enum optionstratlib::error::greeks::GreeksError"){.enum}\> {#fn-delta_neutralityself---resultdeltainfo-greekserror .code-header}
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
[Source](../../../src/optionstratlib/strategies/delta_neutral/model.rs.html#382-387){.src
.rightside}

#### fn [is_delta_neutral](#method.is_delta_neutral){.fn}(&self) -\> [bool](https://doc.rust-lang.org/1.91.1/std/primitive.bool.html){.primitive} {#fn-is_delta_neutralself---bool .code-header}
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
[Source](../../../src/optionstratlib/strategies/delta_neutral/model.rs.html#411-413){.src
.rightside}

#### fn [get_atm_strike](#method.get_atm_strike){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, [StrategyError](../../error/strategies/enum.StrategyError.html "enum optionstratlib::error::strategies::StrategyError"){.enum}\> {#fn-get_atm_strikeself---resultpositive-strategyerror .code-header}
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

::: {#method.generate_delta_adjustments .section .method}
[Source](../../../src/optionstratlib/strategies/delta_neutral/model.rs.html#471-575){.src
.rightside}

#### fn [generate_delta_adjustments](#method.generate_delta_adjustments){.fn}( &self, net_delta: [Decimal](../../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}, option_delta_per_contract: [Decimal](../../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}, option: &[Options](../../model/option/struct.Options.html "struct optionstratlib::model::option::Options"){.struct}, ) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[DeltaAdjustment](enum.DeltaAdjustment.html "enum optionstratlib::strategies::delta_neutral::DeltaAdjustment"){.enum}, [GreeksError](../../error/greeks/enum.GreeksError.html "enum optionstratlib::error::greeks::GreeksError"){.enum}\> {#fn-generate_delta_adjustments-self-net_delta-decimal-option_delta_per_contract-decimal-option-options---resultdeltaadjustment-greekserror .code-header}
:::

::: docblock
Generates delta adjustments based on the given net delta and option
delta per contract.

This function calculates the necessary adjustments (buying or selling
options) to bring the net delta of a position closer to neutral based on
the delta of the options and current positions.

##### [§](#parameters){.doc-anchor}Parameters

- `net_delta`: The net delta of the current portfolio or position. A
  positive value indicates excess positive delta, while a negative value
  indicates excess negative delta.
- `option_delta_per_contract`: The delta value of an individual option
  contract. A positive value represents a positive delta option (e.g.,
  calls for long positions), while a negative value represents a
  negative delta option (e.g., puts for long positions).
- `option`: A reference to an instance of the `Options` struct,
  representing the specific option for which adjustments are to be made.
  This includes attributes such as the option strike price, style, side,
  and current quantity of contracts held.

##### [§](#returns-3){.doc-anchor}Returns

- `Ok(DeltaAdjustment)`: An adjustment object indicating the number of
  contracts to buy or sell, along with relevant option details (e.g.,
  strike price, style, side). If no adjustment is needed, the function
  returns `DeltaAdjustment::NoAdjustmentNeeded`.
- `Err(GreeksError)`: Returns an error if an adjustment cannot be made
  due to invalid input (e.g., delta per contract is zero) or if the
  required adjustment would violate contract limits (e.g., attempting to
  sell more contracts than currently held).

##### [§](#behavior){.doc-anchor}Behavior

- If `net_delta` is zero, no adjustment is needed, and the function
  immediately returns `DeltaAdjustment::NoAdjustmentNeeded`.
- If `option_delta_per_contract` is zero, the function returns a
  `GreeksError` as it is invalid to use an option with no delta.
- The number of contracts required to neutralize the delta is calculated
  as the absolute value of `net_delta / option_delta_per_contract`.
- Based on whether the net delta and option delta are positive or
  negative, the function determines whether to buy or sell options. It
  also checks whether sufficient contracts are available for sale or if
  additional contracts need to be acquired.
- If the required contracts match the current quantity held in the
  portfolio, no adjustment is needed.

##### [§](#adjustment-logic){.doc-anchor}Adjustment Logic

1.  **Sell Options**:
    - If the net delta and option delta per contract are both positive,
      selling options reduces the delta.
    - If the net delta and option delta per contract are both negative,
      selling options reduces the negative delta.
    - If the required number of contracts to sell exceeds the quantity
      currently held, an error is returned because selling more than
      available is not possible.
2.  **Buy Options**:
    - If the net delta is positive and the option delta per contract is
      negative, buying options adds negative delta (neutralizing the
      positive net delta).
    - If the net delta is negative and the option delta per contract is
      positive, buying options adds positive delta (neutralizing the
      negative net delta).

##### [§](#errors-1){.doc-anchor}Errors

- `GreeksError::StdError`:
  - If `option_delta_per_contract` is zero because adjustments with zero
    delta options are invalid.
  - If insufficient contracts are available for an adjustment when
    trying to sell options.
:::

::: {#method.delta_adjustments .section .method}
[Source](../../../src/optionstratlib/strategies/delta_neutral/model.rs.html#590-686){.src
.rightside}

#### fn [delta_adjustments](#method.delta_adjustments){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Vec](https://doc.rust-lang.org/1.91.1/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<[DeltaAdjustment](enum.DeltaAdjustment.html "enum optionstratlib::strategies::delta_neutral::DeltaAdjustment"){.enum}\>, [GreeksError](../../error/greeks/enum.GreeksError.html "enum optionstratlib::error::greeks::GreeksError"){.enum}\> {#fn-delta_adjustmentsself---resultvecdeltaadjustment-greekserror .code-header}
:::

::: docblock
Calculates required position adjustments to maintain delta neutrality

##### [§](#arguments-1){.doc-anchor}Arguments

None - Uses internal position state

##### [§](#returns-4){.doc-anchor}Returns

- `Result<Vec<DeltaAdjustment>, GreeksError>` - Vector of suggested
  position adjustments or error if calculations fail

##### [§](#notes-1){.doc-anchor}Notes

- Uses DELTA_THRESHOLD to determine if adjustments are needed
- Suggests opposite positions to neutralize current delta exposure
- Accounts for both option style (Put/Call) and position side
  (Long/Short)
:::

::: {#method.apply_delta_adjustments .section .method}
[Source](../../../src/optionstratlib/strategies/delta_neutral/model.rs.html#718-766){.src
.rightside}

#### fn [apply_delta_adjustments](#method.apply_delta_adjustments){.fn}( &mut self, action: [Option](https://doc.rust-lang.org/1.91.1/core/option/enum.Option.html "enum core::option::Option"){.enum}\<[Action](../../model/types/enum.Action.html "enum optionstratlib::model::types::Action"){.enum}\>, ) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[()](https://doc.rust-lang.org/1.91.1/std/primitive.unit.html){.primitive}, [StrategyError](../../error/strategies/enum.StrategyError.html "enum optionstratlib::error::strategies::StrategyError"){.enum}\> {#fn-apply_delta_adjustments-mut-self-action-optionaction---result-strategyerror .code-header}
:::

::: docblock
##### [§](#apply-delta-adjustments){.doc-anchor}Apply Delta Adjustments

Applies delta-neutralizing adjustments to the current strategy based on
calculated delta imbalances. This function ensures that the strategy
remains delta-neutral by executing the appropriate position adjustments.

###### [§](#parameters-1){.doc-anchor}Parameters

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

###### [§](#returns-5){.doc-anchor}Returns

- `Result<(), StrategyError>` - Success if adjustments were applied
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
[Source](../../../src/optionstratlib/strategies/delta_neutral/model.rs.html#798-830){.src
.rightside}

#### fn [apply_single_adjustment](#method.apply_single_adjustment){.fn}( &mut self, adjustment: &[DeltaAdjustment](enum.DeltaAdjustment.html "enum optionstratlib::strategies::delta_neutral::DeltaAdjustment"){.enum}, ) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[()](https://doc.rust-lang.org/1.91.1/std/primitive.unit.html){.primitive}, [StrategyError](../../error/strategies/enum.StrategyError.html "enum optionstratlib::error::strategies::StrategyError"){.enum}\> {#fn-apply_single_adjustment-mut-self-adjustment-deltaadjustment---result-strategyerror .code-header}
:::

::: docblock
##### [§](#apply-single-adjustment){.doc-anchor}Apply Single Adjustment

Applies a single delta adjustment to the current position or strategy.

This method processes a single `DeltaAdjustment` and modifies the
object's state accordingly. It handles different types of adjustments
that can be made to maintain or achieve delta neutrality in an options
strategy.

###### [§](#parameters-2){.doc-anchor}Parameters

- `adjustment` - A reference to the `DeltaAdjustment` to apply, which
  can be one of several variants specifying different types of position
  adjustments.

###### [§](#returns-6){.doc-anchor}Returns

- `Result<(), StrategyError>` - Returns `Ok(())` if the adjustment was
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

::: {#method.adjust_option_position .section .method}
[Source](../../../src/optionstratlib/strategies/delta_neutral/model.rs.html#859-880){.src
.rightside}

#### fn [adjust_option_position](#method.adjust_option_position){.fn}( &mut self, quantity: [Decimal](../../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}, strike: &[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, option_type: &[OptionStyle](../../model/types/enum.OptionStyle.html "enum optionstratlib::model::types::OptionStyle"){.enum}, side: &[Side](../../model/types/enum.Side.html "enum optionstratlib::model::types::Side"){.enum}, ) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[()](https://doc.rust-lang.org/1.91.1/std/primitive.unit.html){.primitive}, [StrategyError](../../error/strategies/enum.StrategyError.html "enum optionstratlib::error::strategies::StrategyError"){.enum}\> {#fn-adjust_option_position-mut-self-quantity-decimal-strike-positive-option_type-optionstyle-side-side---result-strategyerror .code-header}
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

###### [§](#errors-2){.doc-anchor}Errors

Returns a boxed error if:

- The specified position does not exist in the strategy
- The `get_position` or `modify_position` methods fail
:::

::: {#method.trade_from_delta_adjustment .section .method}
[Source](../../../src/optionstratlib/strategies/delta_neutral/model.rs.html#890-970){.src
.rightside}

#### fn [trade_from_delta_adjustment](#method.trade_from_delta_adjustment){.fn}( &mut self, action: [Action](../../model/types/enum.Action.html "enum optionstratlib::model::types::Action"){.enum}, ) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Vec](https://doc.rust-lang.org/1.91.1/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<[Trade](../../model/struct.Trade.html "struct optionstratlib::model::Trade"){.struct}\>, [StrategyError](../../error/strategies/enum.StrategyError.html "enum optionstratlib::error::strategies::StrategyError"){.enum}\> {#fn-trade_from_delta_adjustment-mut-self-action-action---resultvectrade-strategyerror .code-header}
:::

::: docblock
Generates a `Trade` object based on the given delta adjustment action.

##### [§](#parameters-4){.doc-anchor}Parameters

- `_action`: An `Action` representing the delta adjustment based on
  which the trade will be formulated.

##### [§](#returns-8){.doc-anchor}Returns

A `Trade` object derived from the delta adjustment logic.
:::
:::::::::::::::::::::

## Implementors[§](#implementors){.anchor} {#implementors .section-header}

:::::::::::::::::::::: {#implementors-list}
::: {#impl-DeltaNeutrality-for-BearCallSpread .section .impl}
[Source](../../../src/optionstratlib/strategies/bear_call_spread.rs.html#820){.src
.rightside}[§](#impl-DeltaNeutrality-for-BearCallSpread){.anchor}

### impl [DeltaNeutrality](trait.DeltaNeutrality.html "trait optionstratlib::strategies::delta_neutral::DeltaNeutrality"){.trait} for [BearCallSpread](../bear_call_spread/struct.BearCallSpread.html "struct optionstratlib::strategies::bear_call_spread::BearCallSpread"){.struct} {#impl-deltaneutrality-for-bearcallspread .code-header}
:::

::: {#impl-DeltaNeutrality-for-BearPutSpread .section .impl}
[Source](../../../src/optionstratlib/strategies/bear_put_spread.rs.html#816){.src
.rightside}[§](#impl-DeltaNeutrality-for-BearPutSpread){.anchor}

### impl [DeltaNeutrality](trait.DeltaNeutrality.html "trait optionstratlib::strategies::delta_neutral::DeltaNeutrality"){.trait} for [BearPutSpread](../bear_put_spread/struct.BearPutSpread.html "struct optionstratlib::strategies::bear_put_spread::BearPutSpread"){.struct} {#impl-deltaneutrality-for-bearputspread .code-header}
:::

::: {#impl-DeltaNeutrality-for-BullCallSpread .section .impl}
[Source](../../../src/optionstratlib/strategies/bull_call_spread.rs.html#833){.src
.rightside}[§](#impl-DeltaNeutrality-for-BullCallSpread){.anchor}

### impl [DeltaNeutrality](trait.DeltaNeutrality.html "trait optionstratlib::strategies::delta_neutral::DeltaNeutrality"){.trait} for [BullCallSpread](../bull_call_spread/struct.BullCallSpread.html "struct optionstratlib::strategies::bull_call_spread::BullCallSpread"){.struct} {#impl-deltaneutrality-for-bullcallspread .code-header}
:::

::: {#impl-DeltaNeutrality-for-BullPutSpread .section .impl}
[Source](../../../src/optionstratlib/strategies/bull_put_spread.rs.html#927){.src
.rightside}[§](#impl-DeltaNeutrality-for-BullPutSpread){.anchor}

### impl [DeltaNeutrality](trait.DeltaNeutrality.html "trait optionstratlib::strategies::delta_neutral::DeltaNeutrality"){.trait} for [BullPutSpread](../bull_put_spread/struct.BullPutSpread.html "struct optionstratlib::strategies::bull_put_spread::BullPutSpread"){.struct} {#impl-deltaneutrality-for-bullputspread .code-header}
:::

::: {#impl-DeltaNeutrality-for-CallButterfly .section .impl}
[Source](../../../src/optionstratlib/strategies/call_butterfly.rs.html#980){.src
.rightside}[§](#impl-DeltaNeutrality-for-CallButterfly){.anchor}

### impl [DeltaNeutrality](trait.DeltaNeutrality.html "trait optionstratlib::strategies::delta_neutral::DeltaNeutrality"){.trait} for [CallButterfly](../call_butterfly/struct.CallButterfly.html "struct optionstratlib::strategies::call_butterfly::CallButterfly"){.struct} {#impl-deltaneutrality-for-callbutterfly .code-header}
:::

::: {#impl-DeltaNeutrality-for-CustomStrategy .section .impl}
[Source](../../../src/optionstratlib/strategies/custom.rs.html#928){.src
.rightside}[§](#impl-DeltaNeutrality-for-CustomStrategy){.anchor}

### impl [DeltaNeutrality](trait.DeltaNeutrality.html "trait optionstratlib::strategies::delta_neutral::DeltaNeutrality"){.trait} for [CustomStrategy](../custom/struct.CustomStrategy.html "struct optionstratlib::strategies::custom::CustomStrategy"){.struct} {#impl-deltaneutrality-for-customstrategy .code-header}
:::

::: {#impl-DeltaNeutrality-for-IronButterfly .section .impl}
[Source](../../../src/optionstratlib/strategies/iron_butterfly.rs.html#1054){.src
.rightside}[§](#impl-DeltaNeutrality-for-IronButterfly){.anchor}

### impl [DeltaNeutrality](trait.DeltaNeutrality.html "trait optionstratlib::strategies::delta_neutral::DeltaNeutrality"){.trait} for [IronButterfly](../iron_butterfly/struct.IronButterfly.html "struct optionstratlib::strategies::iron_butterfly::IronButterfly"){.struct} {#impl-deltaneutrality-for-ironbutterfly .code-header}
:::

::: {#impl-DeltaNeutrality-for-IronCondor .section .impl}
[Source](../../../src/optionstratlib/strategies/iron_condor.rs.html#1082){.src
.rightside}[§](#impl-DeltaNeutrality-for-IronCondor){.anchor}

### impl [DeltaNeutrality](trait.DeltaNeutrality.html "trait optionstratlib::strategies::delta_neutral::DeltaNeutrality"){.trait} for [IronCondor](../iron_condor/struct.IronCondor.html "struct optionstratlib::strategies::iron_condor::IronCondor"){.struct} {#impl-deltaneutrality-for-ironcondor .code-header}
:::

::: {#impl-DeltaNeutrality-for-LongButterflySpread .section .impl}
[Source](../../../src/optionstratlib/strategies/long_butterfly_spread.rs.html#1025){.src
.rightside}[§](#impl-DeltaNeutrality-for-LongButterflySpread){.anchor}

### impl [DeltaNeutrality](trait.DeltaNeutrality.html "trait optionstratlib::strategies::delta_neutral::DeltaNeutrality"){.trait} for [LongButterflySpread](../long_butterfly_spread/struct.LongButterflySpread.html "struct optionstratlib::strategies::long_butterfly_spread::LongButterflySpread"){.struct} {#impl-deltaneutrality-for-longbutterflyspread .code-header}
:::

::: {#impl-DeltaNeutrality-for-LongCall .section .impl}
[Source](../../../src/optionstratlib/strategies/long_call.rs.html#491){.src
.rightside}[§](#impl-DeltaNeutrality-for-LongCall){.anchor}

### impl [DeltaNeutrality](trait.DeltaNeutrality.html "trait optionstratlib::strategies::delta_neutral::DeltaNeutrality"){.trait} for [LongCall](../long_call/struct.LongCall.html "struct optionstratlib::strategies::long_call::LongCall"){.struct} {#impl-deltaneutrality-for-longcall .code-header}
:::

::: {#impl-DeltaNeutrality-for-LongPut .section .impl}
[Source](../../../src/optionstratlib/strategies/long_put.rs.html#488){.src
.rightside}[§](#impl-DeltaNeutrality-for-LongPut){.anchor}

### impl [DeltaNeutrality](trait.DeltaNeutrality.html "trait optionstratlib::strategies::delta_neutral::DeltaNeutrality"){.trait} for [LongPut](../long_put/struct.LongPut.html "struct optionstratlib::strategies::long_put::LongPut"){.struct} {#impl-deltaneutrality-for-longput .code-header}
:::

::: {#impl-DeltaNeutrality-for-LongStraddle .section .impl}
[Source](../../../src/optionstratlib/strategies/long_straddle.rs.html#830){.src
.rightside}[§](#impl-DeltaNeutrality-for-LongStraddle){.anchor}

### impl [DeltaNeutrality](trait.DeltaNeutrality.html "trait optionstratlib::strategies::delta_neutral::DeltaNeutrality"){.trait} for [LongStraddle](../long_straddle/struct.LongStraddle.html "struct optionstratlib::strategies::long_straddle::LongStraddle"){.struct} {#impl-deltaneutrality-for-longstraddle .code-header}
:::

::: {#impl-DeltaNeutrality-for-LongStrangle .section .impl}
[Source](../../../src/optionstratlib/strategies/long_strangle.rs.html#889){.src
.rightside}[§](#impl-DeltaNeutrality-for-LongStrangle){.anchor}

### impl [DeltaNeutrality](trait.DeltaNeutrality.html "trait optionstratlib::strategies::delta_neutral::DeltaNeutrality"){.trait} for [LongStrangle](../long_strangle/struct.LongStrangle.html "struct optionstratlib::strategies::long_strangle::LongStrangle"){.struct} {#impl-deltaneutrality-for-longstrangle .code-header}
:::

::: {#impl-DeltaNeutrality-for-PoorMansCoveredCall .section .impl}
[Source](../../../src/optionstratlib/strategies/poor_mans_covered_call.rs.html#823){.src
.rightside}[§](#impl-DeltaNeutrality-for-PoorMansCoveredCall){.anchor}

### impl [DeltaNeutrality](trait.DeltaNeutrality.html "trait optionstratlib::strategies::delta_neutral::DeltaNeutrality"){.trait} for [PoorMansCoveredCall](../poor_mans_covered_call/struct.PoorMansCoveredCall.html "struct optionstratlib::strategies::poor_mans_covered_call::PoorMansCoveredCall"){.struct} {#impl-deltaneutrality-for-poormanscoveredcall .code-header}
:::

::: {#impl-DeltaNeutrality-for-ShortButterflySpread .section .impl}
[Source](../../../src/optionstratlib/strategies/short_butterfly_spread.rs.html#993){.src
.rightside}[§](#impl-DeltaNeutrality-for-ShortButterflySpread){.anchor}

### impl [DeltaNeutrality](trait.DeltaNeutrality.html "trait optionstratlib::strategies::delta_neutral::DeltaNeutrality"){.trait} for [ShortButterflySpread](../short_butterfly_spread/struct.ShortButterflySpread.html "struct optionstratlib::strategies::short_butterfly_spread::ShortButterflySpread"){.struct} {#impl-deltaneutrality-for-shortbutterflyspread .code-header}
:::

::: {#impl-DeltaNeutrality-for-ShortCall .section .impl}
[Source](../../../src/optionstratlib/strategies/short_call.rs.html#499){.src
.rightside}[§](#impl-DeltaNeutrality-for-ShortCall){.anchor}

### impl [DeltaNeutrality](trait.DeltaNeutrality.html "trait optionstratlib::strategies::delta_neutral::DeltaNeutrality"){.trait} for [ShortCall](../short_call/struct.ShortCall.html "struct optionstratlib::strategies::short_call::ShortCall"){.struct} {#impl-deltaneutrality-for-shortcall .code-header}
:::

::: {#impl-DeltaNeutrality-for-ShortPut .section .impl}
[Source](../../../src/optionstratlib/strategies/short_put.rs.html#493){.src
.rightside}[§](#impl-DeltaNeutrality-for-ShortPut){.anchor}

### impl [DeltaNeutrality](trait.DeltaNeutrality.html "trait optionstratlib::strategies::delta_neutral::DeltaNeutrality"){.trait} for [ShortPut](../short_put/struct.ShortPut.html "struct optionstratlib::strategies::short_put::ShortPut"){.struct} {#impl-deltaneutrality-for-shortput .code-header}
:::

::: {#impl-DeltaNeutrality-for-ShortStraddle .section .impl}
[Source](../../../src/optionstratlib/strategies/short_straddle.rs.html#877){.src
.rightside}[§](#impl-DeltaNeutrality-for-ShortStraddle){.anchor}

### impl [DeltaNeutrality](trait.DeltaNeutrality.html "trait optionstratlib::strategies::delta_neutral::DeltaNeutrality"){.trait} for [ShortStraddle](../short_straddle/struct.ShortStraddle.html "struct optionstratlib::strategies::short_straddle::ShortStraddle"){.struct} {#impl-deltaneutrality-for-shortstraddle .code-header}
:::

::: {#impl-DeltaNeutrality-for-ShortStrangle .section .impl}
[Source](../../../src/optionstratlib/strategies/short_strangle.rs.html#1136){.src
.rightside}[§](#impl-DeltaNeutrality-for-ShortStrangle){.anchor}

### impl [DeltaNeutrality](trait.DeltaNeutrality.html "trait optionstratlib::strategies::delta_neutral::DeltaNeutrality"){.trait} for [ShortStrangle](../short_strangle/struct.ShortStrangle.html "struct optionstratlib::strategies::short_strangle::ShortStrangle"){.struct} {#impl-deltaneutrality-for-shortstrangle .code-header}
:::
::::::::::::::::::::::
:::::::::::::::::::::::::::::::::::::::::::::
::::::::::::::::::::::::::::::::::::::::::::::
