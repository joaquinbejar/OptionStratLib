:::::::::::::::::::::::::::::::::::::::::::: width-limiter
::::::::::::::::::::::::::::::::::::::::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../../index.html)::[strategies](../index.html)::[base](index.html)
:::

# Trait [Positionable]{.trait} Copy item path

[[Source](../../../src/optionstratlib/strategies/base.rs.html#1290-1480){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub trait Positionable {
    // Provided methods
    fn add_position(
        &mut self,
        _position: &Position,
    ) -> Result<(), PositionError> { ... }
    fn get_positions(&self) -> Result<Vec<&Position>, PositionError> { ... }
    fn get_position(
        &mut self,
        _option_style: &OptionStyle,
        _side: &Side,
        _strike: &Positive,
    ) -> Result<Vec<&mut Position>, PositionError> { ... }
    fn get_position_unique(
        &mut self,
        _option_style: &OptionStyle,
        _side: &Side,
    ) -> Result<&mut Position, PositionError> { ... }
    fn get_option_unique(
        &mut self,
        _option_style: &OptionStyle,
        _side: &Side,
    ) -> Result<&mut Options, PositionError> { ... }
    fn modify_position(
        &mut self,
        _position: &Position,
    ) -> Result<(), PositionError> { ... }
    fn replace_position(
        &mut self,
        _position: &Position,
    ) -> Result<(), PositionError> { ... }
    fn valid_premium_for_shorts(&self, min_premium: &Positive) -> bool { ... }
}
```

Expand description

::: docblock
The `Positionable` trait defines methods for managing positions within a
trading strategy. These methods allow for adding, retrieving, and
modifying positions, providing a common interface for different
strategies to interact with position data.
:::

## Provided Methods[§](#provided-methods){.anchor} {#provided-methods .section-header}

::::::::::::::::::: methods
::: {#method.add_position .section .method}
[Source](../../../src/optionstratlib/strategies/base.rs.html#1306-1311){.src
.rightside}

#### fn [add_position](#method.add_position){.fn}(&mut self, \_position: &[Position](../../model/position/struct.Position.html "struct optionstratlib::model::position::Position"){.struct}) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[()](https://doc.rust-lang.org/1.91.1/std/primitive.unit.html){.primitive}, [PositionError](../../error/position/enum.PositionError.html "enum optionstratlib::error::position::PositionError"){.enum}\> {#fn-add_positionmut-self-_position-position---result-positionerror .code-header}
:::

::: docblock
Adds a position to the strategy.

##### [§](#arguments){.doc-anchor}Arguments

- `_position` - A reference to the `Position` to be added.

##### [§](#returns){.doc-anchor}Returns

- `Result<(), PositionError>` - Returns `Ok(())` if the position was
  successfully added, or a `PositionError` if the operation is not
  supported by the strategy.

##### [§](#default-implementation){.doc-anchor}Default Implementation

The default implementation returns an error indicating that adding a
position is not supported. Strategies that support adding positions
should override this method.
:::

::: {#method.get_positions .section .method}
[Source](../../../src/optionstratlib/strategies/base.rs.html#1325-1330){.src
.rightside}

#### fn [get_positions](#method.get_positions){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Vec](https://doc.rust-lang.org/1.91.1/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<&[Position](../../model/position/struct.Position.html "struct optionstratlib::model::position::Position"){.struct}\>, [PositionError](../../error/position/enum.PositionError.html "enum optionstratlib::error::position::PositionError"){.enum}\> {#fn-get_positionsself---resultvecposition-positionerror .code-header}
:::

::: docblock
Retrieves all positions held by the strategy.

##### [§](#returns-1){.doc-anchor}Returns

- `Result<Vec<&Position>, PositionError>` - A `Result` containing a
  vector of references to the `Position` objects held by the strategy,
  or a `PositionError` if the operation is not supported.

##### [§](#default-implementation-1){.doc-anchor}Default Implementation

The default implementation returns an error indicating that getting
positions is not supported. Strategies that manage positions should
override this method.
:::

::: {#method.get_position .section .method}
[Source](../../../src/optionstratlib/strategies/base.rs.html#1345-1352){.src
.rightside}

#### fn [get_position](#method.get_position){.fn}( &mut self, \_option_style: &[OptionStyle](../../model/types/enum.OptionStyle.html "enum optionstratlib::model::types::OptionStyle"){.enum}, \_side: &[Side](../../model/types/enum.Side.html "enum optionstratlib::model::types::Side"){.enum}, \_strike: &[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, ) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Vec](https://doc.rust-lang.org/1.91.1/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<&mut [Position](../../model/position/struct.Position.html "struct optionstratlib::model::position::Position"){.struct}\>, [PositionError](../../error/position/enum.PositionError.html "enum optionstratlib::error::position::PositionError"){.enum}\> {#fn-get_position-mut-self-_option_style-optionstyle-_side-side-_strike-positive---resultvecmut-position-positionerror .code-header}
:::

::: docblock
Retrieves a specific position based on option style, side, and strike.

##### [§](#arguments-1){.doc-anchor}Arguments

- `_option_style` - The style of the option (Call or Put).
- `_side` - The side of the position (Long or Short).
- `_strike` - The strike price of the option.

##### [§](#returns-2){.doc-anchor}Returns

- `Result<Vec<&mut Position>, PositionError>` - A `Result` containing a
  vector of mutable references to the matching `Position` objects, or a
  `PositionError` if the operation is not supported. This function
  currently uses `unimplemented!()`.
:::

::: {#method.get_position_unique .section .method}
[Source](../../../src/optionstratlib/strategies/base.rs.html#1368-1374){.src
.rightside}

#### fn [get_position_unique](#method.get_position_unique){.fn}( &mut self, \_option_style: &[OptionStyle](../../model/types/enum.OptionStyle.html "enum optionstratlib::model::types::OptionStyle"){.enum}, \_side: &[Side](../../model/types/enum.Side.html "enum optionstratlib::model::types::Side"){.enum}, ) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<&mut [Position](../../model/position/struct.Position.html "struct optionstratlib::model::position::Position"){.struct}, [PositionError](../../error/position/enum.PositionError.html "enum optionstratlib::error::position::PositionError"){.enum}\> {#fn-get_position_unique-mut-self-_option_style-optionstyle-_side-side---resultmut-position-positionerror .code-header}
:::

::: docblock
Retrieves a unique position based on the given option style and side.

##### [§](#parameters){.doc-anchor}Parameters

- `_option_style`: A reference to an `OptionStyle` which defines the
  style of the options (e.g., American, European).
- `_side`: A reference to a `Side` which specifies whether the position
  is on the buy or sell side.

##### [§](#returns-3){.doc-anchor}Returns

A mutable reference to the `Position` if found. If the position could
not be determined or does not exist, returns a `PositionError`.

##### [§](#errors){.doc-anchor}Errors

This function always returns an error as it is not implemented for this
strategy but provides a placeholder for functionality to be added later.
:::

::: {#method.get_option_unique .section .method}
[Source](../../../src/optionstratlib/strategies/base.rs.html#1404-1410){.src
.rightside}

#### fn [get_option_unique](#method.get_option_unique){.fn}( &mut self, \_option_style: &[OptionStyle](../../model/types/enum.OptionStyle.html "enum optionstratlib::model::types::OptionStyle"){.enum}, \_side: &[Side](../../model/types/enum.Side.html "enum optionstratlib::model::types::Side"){.enum}, ) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<&mut [Options](../../model/option/struct.Options.html "struct optionstratlib::model::option::Options"){.struct}, [PositionError](../../error/position/enum.PositionError.html "enum optionstratlib::error::position::PositionError"){.enum}\> {#fn-get_option_unique-mut-self-_option_style-optionstyle-_side-side---resultmut-options-positionerror .code-header}
:::

::: docblock
Retrieves a unique option based on the given style and side.

This function is intended to retrieve a unique financial option of a
specific style (`_option_style`) and side (`_side`). However, the
functionality has not been implemented for the current strategy, and
calling this function will result in a runtime panic.

##### [§](#parameters-1){.doc-anchor}Parameters

- `_option_style`: A reference to an `OptionStyle` that specifies the
  style of the option to retrieve (e.g., American, European).
- `_side`: A reference to a `Side` that indicates the side of the
  option, such as a call or put.

##### [§](#returns-4){.doc-anchor}Returns

- `Result<&mut Options, PositionError>`:
  - On success, a mutable reference to an `Options` object would be
    returned. However, the current implementation always results in an
    unimplemented error.

##### [§](#errors-1){.doc-anchor}Errors

- Always returns a `PositionError` due to the `unimplemented!` macro
  indicating that this functionality is not yet supported for the
  strategy.

##### [§](#notes){.doc-anchor}Notes

This function should be implemented to support strategies that require
retrieving unique options. Until implemented, usage of this function is
not recommended.
:::

::: {#method.modify_position .section .method}
[Source](../../../src/optionstratlib/strategies/base.rs.html#1423-1425){.src
.rightside}

#### fn [modify_position](#method.modify_position){.fn}(&mut self, \_position: &[Position](../../model/position/struct.Position.html "struct optionstratlib::model::position::Position"){.struct}) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[()](https://doc.rust-lang.org/1.91.1/std/primitive.unit.html){.primitive}, [PositionError](../../error/position/enum.PositionError.html "enum optionstratlib::error::position::PositionError"){.enum}\> {#fn-modify_positionmut-self-_position-position---result-positionerror .code-header}
:::

::: docblock
Modifies an existing position.

##### [§](#arguments-2){.doc-anchor}Arguments

- `_position` - A reference to the `Position` to be modified.

##### [§](#returns-5){.doc-anchor}Returns

- `Result<(), PositionError>` - A `Result` indicating success or failure
  of the modification, or a `PositionError` if the operation is not
  supported. This function currently uses `unimplemented!()`.
:::

::: {#method.replace_position .section .method}
[Source](../../../src/optionstratlib/strategies/base.rs.html#1443-1445){.src
.rightside}

#### fn [replace_position](#method.replace_position){.fn}( &mut self, \_position: &[Position](../../model/position/struct.Position.html "struct optionstratlib::model::position::Position"){.struct}, ) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[()](https://doc.rust-lang.org/1.91.1/std/primitive.unit.html){.primitive}, [PositionError](../../error/position/enum.PositionError.html "enum optionstratlib::error::position::PositionError"){.enum}\> {#fn-replace_position-mut-self-_position-position---result-positionerror .code-header}
:::

::: docblock
Attempts to replace the current position with a new position.

##### [§](#parameters-2){.doc-anchor}Parameters

- `_position`: A reference to a `Position` object that represents the
  new position to replace the current one.

##### [§](#returns-6){.doc-anchor}Returns

- `Ok(())`: If the position replacement is successful.
- `Err(PositionError)`: If an error occurs while replacing the position.

##### [§](#notes-1){.doc-anchor}Notes

This function is currently not implemented for this strategy and will
panic with a `not implemented` message when called.

##### [§](#panics){.doc-anchor}Panics

This function will always panic with `unimplemented!()` since it hasn't
been implemented yet.
:::

::: {#method.valid_premium_for_shorts .section .method}
[Source](../../../src/optionstratlib/strategies/base.rs.html#1467-1479){.src
.rightside}

#### fn [valid_premium_for_shorts](#method.valid_premium_for_shorts){.fn}(&self, min_premium: &[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}) -\> [bool](https://doc.rust-lang.org/1.91.1/std/primitive.bool.html){.primitive} {#fn-valid_premium_for_shortsself-min_premium-positive---bool .code-header}
:::

::: docblock
Checks if all short positions have a net premium received that meets or
exceeds a specified minimum.

##### [§](#parameters-3){.doc-anchor}Parameters

- `min_premium`: A reference to a `Positive` value representing the
  minimum premium required for the short positions to be considered
  valid.

##### [§](#returns-7){.doc-anchor}Returns

- `true` if all short positions in the portfolio have a net premium
  received that is greater than or equal to `min_premium`.
- `false` if any of the following conditions occur:
  - Unable to retrieve positions (e.g., `get_positions` fails).
  - At least one short position has a net premium less than
    `min_premium`.
  - At least one short position's net premium calculation fails with an
    error.

##### [§](#implementation-details){.doc-anchor}Implementation Details

- Retrieves positions using the `get_positions` method. If this
  operation fails, the function returns `false`.
- Filters positions to only include shorts (based on `is_short` method).
- For each short position, determines if the net premium received is
  available (`is_ok`) and satisfies the minimum threshold
  (`>= *min_premium`).
:::
:::::::::::::::::::

## Implementors[§](#implementors){.anchor} {#implementors .section-header}

:::::::::::::::::::::: {#implementors-list}
::: {#impl-Positionable-for-BearCallSpread .section .impl}
[Source](../../../src/optionstratlib/strategies/bear_call_spread.rs.html#343-447){.src
.rightside}[§](#impl-Positionable-for-BearCallSpread){.anchor}

### impl [Positionable](trait.Positionable.html "trait optionstratlib::strategies::base::Positionable"){.trait} for [BearCallSpread](../bear_call_spread/struct.BearCallSpread.html "struct optionstratlib::strategies::bear_call_spread::BearCallSpread"){.struct} {#impl-positionable-for-bearcallspread .code-header}
:::

::: {#impl-Positionable-for-BearPutSpread .section .impl}
[Source](../../../src/optionstratlib/strategies/bear_put_spread.rs.html#341-445){.src
.rightside}[§](#impl-Positionable-for-BearPutSpread){.anchor}

### impl [Positionable](trait.Positionable.html "trait optionstratlib::strategies::base::Positionable"){.trait} for [BearPutSpread](../bear_put_spread/struct.BearPutSpread.html "struct optionstratlib::strategies::bear_put_spread::BearPutSpread"){.struct} {#impl-positionable-for-bearputspread .code-header}
:::

::: {#impl-Positionable-for-BullCallSpread .section .impl}
[Source](../../../src/optionstratlib/strategies/bull_call_spread.rs.html#345-449){.src
.rightside}[§](#impl-Positionable-for-BullCallSpread){.anchor}

### impl [Positionable](trait.Positionable.html "trait optionstratlib::strategies::base::Positionable"){.trait} for [BullCallSpread](../bull_call_spread/struct.BullCallSpread.html "struct optionstratlib::strategies::bull_call_spread::BullCallSpread"){.struct} {#impl-positionable-for-bullcallspread .code-header}
:::

::: {#impl-Positionable-for-BullPutSpread .section .impl}
[Source](../../../src/optionstratlib/strategies/bull_put_spread.rs.html#351-455){.src
.rightside}[§](#impl-Positionable-for-BullPutSpread){.anchor}

### impl [Positionable](trait.Positionable.html "trait optionstratlib::strategies::base::Positionable"){.trait} for [BullPutSpread](../bull_put_spread/struct.BullPutSpread.html "struct optionstratlib::strategies::bull_put_spread::BullPutSpread"){.struct} {#impl-positionable-for-bullputspread .code-header}
:::

::: {#impl-Positionable-for-CallButterfly .section .impl}
[Source](../../../src/optionstratlib/strategies/call_butterfly.rs.html#392-509){.src
.rightside}[§](#impl-Positionable-for-CallButterfly){.anchor}

### impl [Positionable](trait.Positionable.html "trait optionstratlib::strategies::base::Positionable"){.trait} for [CallButterfly](../call_butterfly/struct.CallButterfly.html "struct optionstratlib::strategies::call_butterfly::CallButterfly"){.struct} {#impl-positionable-for-callbutterfly .code-header}
:::

::: {#impl-Positionable-for-CustomStrategy .section .impl}
[Source](../../../src/optionstratlib/strategies/custom.rs.html#387-510){.src
.rightside}[§](#impl-Positionable-for-CustomStrategy){.anchor}

### impl [Positionable](trait.Positionable.html "trait optionstratlib::strategies::base::Positionable"){.trait} for [CustomStrategy](../custom/struct.CustomStrategy.html "struct optionstratlib::strategies::custom::CustomStrategy"){.struct} {#impl-positionable-for-customstrategy .code-header}
:::

::: {#impl-Positionable-for-IronButterfly .section .impl}
[Source](../../../src/optionstratlib/strategies/iron_butterfly.rs.html#477-595){.src
.rightside}[§](#impl-Positionable-for-IronButterfly){.anchor}

### impl [Positionable](trait.Positionable.html "trait optionstratlib::strategies::base::Positionable"){.trait} for [IronButterfly](../iron_butterfly/struct.IronButterfly.html "struct optionstratlib::strategies::iron_butterfly::IronButterfly"){.struct} {#impl-positionable-for-ironbutterfly .code-header}
:::

::: {#impl-Positionable-for-IronCondor .section .impl}
[Source](../../../src/optionstratlib/strategies/iron_condor.rs.html#492-612){.src
.rightside}[§](#impl-Positionable-for-IronCondor){.anchor}

### impl [Positionable](trait.Positionable.html "trait optionstratlib::strategies::base::Positionable"){.trait} for [IronCondor](../iron_condor/struct.IronCondor.html "struct optionstratlib::strategies::iron_condor::IronCondor"){.struct} {#impl-positionable-for-ironcondor .code-header}
:::

::: {#impl-Positionable-for-LongButterflySpread .section .impl}
[Source](../../../src/optionstratlib/strategies/long_butterfly_spread.rs.html#415-541){.src
.rightside}[§](#impl-Positionable-for-LongButterflySpread){.anchor}

### impl [Positionable](trait.Positionable.html "trait optionstratlib::strategies::base::Positionable"){.trait} for [LongButterflySpread](../long_butterfly_spread/struct.LongButterflySpread.html "struct optionstratlib::strategies::long_butterfly_spread::LongButterflySpread"){.struct} {#impl-positionable-for-longbutterflyspread .code-header}
:::

::: {#impl-Positionable-for-LongCall .section .impl}
[Source](../../../src/optionstratlib/strategies/long_call.rs.html#322-406){.src
.rightside}[§](#impl-Positionable-for-LongCall){.anchor}

### impl [Positionable](trait.Positionable.html "trait optionstratlib::strategies::base::Positionable"){.trait} for [LongCall](../long_call/struct.LongCall.html "struct optionstratlib::strategies::long_call::LongCall"){.struct} {#impl-positionable-for-longcall .code-header}
:::

::: {#impl-Positionable-for-LongPut .section .impl}
[Source](../../../src/optionstratlib/strategies/long_put.rs.html#319-403){.src
.rightside}[§](#impl-Positionable-for-LongPut){.anchor}

### impl [Positionable](trait.Positionable.html "trait optionstratlib::strategies::base::Positionable"){.trait} for [LongPut](../long_put/struct.LongPut.html "struct optionstratlib::strategies::long_put::LongPut"){.struct} {#impl-positionable-for-longput .code-header}
:::

::: {#impl-Positionable-for-LongStraddle .section .impl}
[Source](../../../src/optionstratlib/strategies/long_straddle.rs.html#372-471){.src
.rightside}[§](#impl-Positionable-for-LongStraddle){.anchor}

### impl [Positionable](trait.Positionable.html "trait optionstratlib::strategies::base::Positionable"){.trait} for [LongStraddle](../long_straddle/struct.LongStraddle.html "struct optionstratlib::strategies::long_straddle::LongStraddle"){.struct} {#impl-positionable-for-longstraddle .code-header}
:::

::: {#impl-Positionable-for-LongStrangle .section .impl}
[Source](../../../src/optionstratlib/strategies/long_strangle.rs.html#385-487){.src
.rightside}[§](#impl-Positionable-for-LongStrangle){.anchor}

### impl [Positionable](trait.Positionable.html "trait optionstratlib::strategies::base::Positionable"){.trait} for [LongStrangle](../long_strangle/struct.LongStrangle.html "struct optionstratlib::strategies::long_strangle::LongStrangle"){.struct} {#impl-positionable-for-longstrangle .code-header}
:::

::: {#impl-Positionable-for-PoorMansCoveredCall .section .impl}
[Source](../../../src/optionstratlib/strategies/poor_mans_covered_call.rs.html#368-476){.src
.rightside}[§](#impl-Positionable-for-PoorMansCoveredCall){.anchor}

### impl [Positionable](trait.Positionable.html "trait optionstratlib::strategies::base::Positionable"){.trait} for [PoorMansCoveredCall](../poor_mans_covered_call/struct.PoorMansCoveredCall.html "struct optionstratlib::strategies::poor_mans_covered_call::PoorMansCoveredCall"){.struct} {#impl-positionable-for-poormanscoveredcall .code-header}
:::

::: {#impl-Positionable-for-ShortButterflySpread .section .impl}
[Source](../../../src/optionstratlib/strategies/short_butterfly_spread.rs.html#408-534){.src
.rightside}[§](#impl-Positionable-for-ShortButterflySpread){.anchor}

### impl [Positionable](trait.Positionable.html "trait optionstratlib::strategies::base::Positionable"){.trait} for [ShortButterflySpread](../short_butterfly_spread/struct.ShortButterflySpread.html "struct optionstratlib::strategies::short_butterfly_spread::ShortButterflySpread"){.struct} {#impl-positionable-for-shortbutterflyspread .code-header}
:::

::: {#impl-Positionable-for-ShortCall .section .impl}
[Source](../../../src/optionstratlib/strategies/short_call.rs.html#330-414){.src
.rightside}[§](#impl-Positionable-for-ShortCall){.anchor}

### impl [Positionable](trait.Positionable.html "trait optionstratlib::strategies::base::Positionable"){.trait} for [ShortCall](../short_call/struct.ShortCall.html "struct optionstratlib::strategies::short_call::ShortCall"){.struct} {#impl-positionable-for-shortcall .code-header}
:::

::: {#impl-Positionable-for-ShortPut .section .impl}
[Source](../../../src/optionstratlib/strategies/short_put.rs.html#324-408){.src
.rightside}[§](#impl-Positionable-for-ShortPut){.anchor}

### impl [Positionable](trait.Positionable.html "trait optionstratlib::strategies::base::Positionable"){.trait} for [ShortPut](../short_put/struct.ShortPut.html "struct optionstratlib::strategies::short_put::ShortPut"){.struct} {#impl-positionable-for-shortput .code-header}
:::

::: {#impl-Positionable-for-ShortStraddle .section .impl}
[Source](../../../src/optionstratlib/strategies/short_straddle.rs.html#386-485){.src
.rightside}[§](#impl-Positionable-for-ShortStraddle){.anchor}

### impl [Positionable](trait.Positionable.html "trait optionstratlib::strategies::base::Positionable"){.trait} for [ShortStraddle](../short_straddle/struct.ShortStraddle.html "struct optionstratlib::strategies::short_straddle::ShortStraddle"){.struct} {#impl-positionable-for-shortstraddle .code-header}
:::

::: {#impl-Positionable-for-ShortStrangle .section .impl}
[Source](../../../src/optionstratlib/strategies/short_strangle.rs.html#373-539){.src
.rightside}[§](#impl-Positionable-for-ShortStrangle){.anchor}

### impl [Positionable](trait.Positionable.html "trait optionstratlib::strategies::base::Positionable"){.trait} for [ShortStrangle](../short_strangle/struct.ShortStrangle.html "struct optionstratlib::strategies::short_strangle::ShortStrangle"){.struct} {#impl-positionable-for-shortstrangle .code-header}
:::
::::::::::::::::::::::
:::::::::::::::::::::::::::::::::::::::::::
::::::::::::::::::::::::::::::::::::::::::::
