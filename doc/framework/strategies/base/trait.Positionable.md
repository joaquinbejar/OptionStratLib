:::::::::::::::::::::::::::::::: width-limiter
::::::::::::::::::::::::::::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../../index.html)::[strategies](../index.html)::[base](index.html)
:::

# Trait [Positionable]{.trait}Copy item path

[[Source](../../../src/optionstratlib/strategies/base.rs.html#891-969){.src}
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
    fn modify_position(
        &mut self,
        _position: &Position,
    ) -> Result<(), PositionError> { ... }
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

::::::::::: methods
::: {#method.add_position .section .method}
[Source](../../../src/optionstratlib/strategies/base.rs.html#907-912){.src
.rightside}

#### fn [add_position](#method.add_position){.fn}(&mut self, \_position: &[Position](../../model/position/struct.Position.html "struct optionstratlib::model::position::Position"){.struct}) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[()](https://doc.rust-lang.org/1.86.0/std/primitive.unit.html){.primitive}, [PositionError](../../error/position/enum.PositionError.html "enum optionstratlib::error::position::PositionError"){.enum}\> {#fn-add_positionmut-self-_position-position---result-positionerror .code-header}
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
[Source](../../../src/optionstratlib/strategies/base.rs.html#926-931){.src
.rightside}

#### fn [get_positions](#method.get_positions){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Vec](https://doc.rust-lang.org/1.86.0/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<&[Position](../../model/position/struct.Position.html "struct optionstratlib::model::position::Position"){.struct}\>, [PositionError](../../error/position/enum.PositionError.html "enum optionstratlib::error::position::PositionError"){.enum}\> {#fn-get_positionsself---resultvecposition-positionerror .code-header}
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
[Source](../../../src/optionstratlib/strategies/base.rs.html#946-953){.src
.rightside}

#### fn [get_position](#method.get_position){.fn}( &mut self, \_option_style: &[OptionStyle](../../model/types/enum.OptionStyle.html "enum optionstratlib::model::types::OptionStyle"){.enum}, \_side: &[Side](../../model/types/enum.Side.html "enum optionstratlib::model::types::Side"){.enum}, \_strike: &[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, ) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Vec](https://doc.rust-lang.org/1.86.0/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<&mut [Position](../../model/position/struct.Position.html "struct optionstratlib::model::position::Position"){.struct}\>, [PositionError](../../error/position/enum.PositionError.html "enum optionstratlib::error::position::PositionError"){.enum}\> {#fn-get_position-mut-self-_option_style-optionstyle-_side-side-_strike-positive---resultvecmut-position-positionerror .code-header}
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
  `PositionError` if the operation is not supported.
:::

::: {#method.modify_position .section .method}
[Source](../../../src/optionstratlib/strategies/base.rs.html#966-968){.src
.rightside}

#### fn [modify_position](#method.modify_position){.fn}(&mut self, \_position: &[Position](../../model/position/struct.Position.html "struct optionstratlib::model::position::Position"){.struct}) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[()](https://doc.rust-lang.org/1.86.0/std/primitive.unit.html){.primitive}, [PositionError](../../error/position/enum.PositionError.html "enum optionstratlib::error::position::PositionError"){.enum}\> {#fn-modify_positionmut-self-_position-position---result-positionerror .code-header}
:::

::: docblock
Modifies an existing position.

##### [§](#arguments-2){.doc-anchor}Arguments

- `_position` - A reference to the `Position` to be modified.

##### [§](#returns-3){.doc-anchor}Returns

- `Result<(), PositionError>` - A `Result` indicating success or failure
  of the modification, or a `PositionError`.
:::
:::::::::::

## Implementors[§](#implementors){.anchor} {#implementors .section-header}

:::::::::::::::::: {#implementors-list}
::: {#impl-Positionable-for-BearCallSpread .section .impl}
[Source](../../../src/optionstratlib/strategies/bear_call_spread.rs.html#342-446){.src
.rightside}[§](#impl-Positionable-for-BearCallSpread){.anchor}

### impl [Positionable](trait.Positionable.html "trait optionstratlib::strategies::base::Positionable"){.trait} for [BearCallSpread](../bear_call_spread/struct.BearCallSpread.html "struct optionstratlib::strategies::bear_call_spread::BearCallSpread"){.struct} {#impl-positionable-for-bearcallspread .code-header}
:::

::: {#impl-Positionable-for-BearPutSpread .section .impl}
[Source](../../../src/optionstratlib/strategies/bear_put_spread.rs.html#322-426){.src
.rightside}[§](#impl-Positionable-for-BearPutSpread){.anchor}

### impl [Positionable](trait.Positionable.html "trait optionstratlib::strategies::base::Positionable"){.trait} for [BearPutSpread](../bear_put_spread/struct.BearPutSpread.html "struct optionstratlib::strategies::bear_put_spread::BearPutSpread"){.struct} {#impl-positionable-for-bearputspread .code-header}
:::

::: {#impl-Positionable-for-BullCallSpread .section .impl}
[Source](../../../src/optionstratlib/strategies/bull_call_spread.rs.html#331-435){.src
.rightside}[§](#impl-Positionable-for-BullCallSpread){.anchor}

### impl [Positionable](trait.Positionable.html "trait optionstratlib::strategies::base::Positionable"){.trait} for [BullCallSpread](../bull_call_spread/struct.BullCallSpread.html "struct optionstratlib::strategies::bull_call_spread::BullCallSpread"){.struct} {#impl-positionable-for-bullcallspread .code-header}
:::

::: {#impl-Positionable-for-BullPutSpread .section .impl}
[Source](../../../src/optionstratlib/strategies/bull_put_spread.rs.html#337-441){.src
.rightside}[§](#impl-Positionable-for-BullPutSpread){.anchor}

### impl [Positionable](trait.Positionable.html "trait optionstratlib::strategies::base::Positionable"){.trait} for [BullPutSpread](../bull_put_spread/struct.BullPutSpread.html "struct optionstratlib::strategies::bull_put_spread::BullPutSpread"){.struct} {#impl-positionable-for-bullputspread .code-header}
:::

::: {#impl-Positionable-for-LongButterflySpread .section .impl}
[Source](../../../src/optionstratlib/strategies/butterfly_spread.rs.html#395-521){.src
.rightside}[§](#impl-Positionable-for-LongButterflySpread){.anchor}

### impl [Positionable](trait.Positionable.html "trait optionstratlib::strategies::base::Positionable"){.trait} for [LongButterflySpread](../butterfly_spread/struct.LongButterflySpread.html "struct optionstratlib::strategies::butterfly_spread::LongButterflySpread"){.struct} {#impl-positionable-for-longbutterflyspread .code-header}
:::

::: {#impl-Positionable-for-ShortButterflySpread .section .impl}
[Source](../../../src/optionstratlib/strategies/butterfly_spread.rs.html#1371-1497){.src
.rightside}[§](#impl-Positionable-for-ShortButterflySpread){.anchor}

### impl [Positionable](trait.Positionable.html "trait optionstratlib::strategies::base::Positionable"){.trait} for [ShortButterflySpread](../butterfly_spread/struct.ShortButterflySpread.html "struct optionstratlib::strategies::butterfly_spread::ShortButterflySpread"){.struct} {#impl-positionable-for-shortbutterflyspread .code-header}
:::

::: {#impl-Positionable-for-CallButterfly .section .impl}
[Source](../../../src/optionstratlib/strategies/call_butterfly.rs.html#402-519){.src
.rightside}[§](#impl-Positionable-for-CallButterfly){.anchor}

### impl [Positionable](trait.Positionable.html "trait optionstratlib::strategies::base::Positionable"){.trait} for [CallButterfly](../call_butterfly/struct.CallButterfly.html "struct optionstratlib::strategies::call_butterfly::CallButterfly"){.struct} {#impl-positionable-for-callbutterfly .code-header}
:::

::: {#impl-Positionable-for-CustomStrategy .section .impl}
[Source](../../../src/optionstratlib/strategies/custom.rs.html#359-376){.src
.rightside}[§](#impl-Positionable-for-CustomStrategy){.anchor}

### impl [Positionable](trait.Positionable.html "trait optionstratlib::strategies::base::Positionable"){.trait} for [CustomStrategy](../custom/struct.CustomStrategy.html "struct optionstratlib::strategies::custom::CustomStrategy"){.struct} {#impl-positionable-for-customstrategy .code-header}
:::

::: {#impl-Positionable-for-IronButterfly .section .impl}
[Source](../../../src/optionstratlib/strategies/iron_butterfly.rs.html#462-580){.src
.rightside}[§](#impl-Positionable-for-IronButterfly){.anchor}

### impl [Positionable](trait.Positionable.html "trait optionstratlib::strategies::base::Positionable"){.trait} for [IronButterfly](../iron_butterfly/struct.IronButterfly.html "struct optionstratlib::strategies::iron_butterfly::IronButterfly"){.struct} {#impl-positionable-for-ironbutterfly .code-header}
:::

::: {#impl-Positionable-for-IronCondor .section .impl}
[Source](../../../src/optionstratlib/strategies/iron_condor.rs.html#469-589){.src
.rightside}[§](#impl-Positionable-for-IronCondor){.anchor}

### impl [Positionable](trait.Positionable.html "trait optionstratlib::strategies::base::Positionable"){.trait} for [IronCondor](../iron_condor/struct.IronCondor.html "struct optionstratlib::strategies::iron_condor::IronCondor"){.struct} {#impl-positionable-for-ironcondor .code-header}
:::

::: {#impl-Positionable-for-PoorMansCoveredCall .section .impl}
[Source](../../../src/optionstratlib/strategies/poor_mans_covered_call.rs.html#358-466){.src
.rightside}[§](#impl-Positionable-for-PoorMansCoveredCall){.anchor}

### impl [Positionable](trait.Positionable.html "trait optionstratlib::strategies::base::Positionable"){.trait} for [PoorMansCoveredCall](../poor_mans_covered_call/struct.PoorMansCoveredCall.html "struct optionstratlib::strategies::poor_mans_covered_call::PoorMansCoveredCall"){.struct} {#impl-positionable-for-poormanscoveredcall .code-header}
:::

::: {#impl-Positionable-for-LongStraddle .section .impl}
[Source](../../../src/optionstratlib/strategies/straddle.rs.html#1179-1278){.src
.rightside}[§](#impl-Positionable-for-LongStraddle){.anchor}

### impl [Positionable](trait.Positionable.html "trait optionstratlib::strategies::base::Positionable"){.trait} for [LongStraddle](../straddle/struct.LongStraddle.html "struct optionstratlib::strategies::straddle::LongStraddle"){.struct} {#impl-positionable-for-longstraddle .code-header}
:::

::: {#impl-Positionable-for-ShortStraddle .section .impl}
[Source](../../../src/optionstratlib/strategies/straddle.rs.html#374-473){.src
.rightside}[§](#impl-Positionable-for-ShortStraddle){.anchor}

### impl [Positionable](trait.Positionable.html "trait optionstratlib::strategies::base::Positionable"){.trait} for [ShortStraddle](../straddle/struct.ShortStraddle.html "struct optionstratlib::strategies::straddle::ShortStraddle"){.struct} {#impl-positionable-for-shortstraddle .code-header}
:::

::: {#impl-Positionable-for-LongStrangle .section .impl}
[Source](../../../src/optionstratlib/strategies/strangle.rs.html#1384-1486){.src
.rightside}[§](#impl-Positionable-for-LongStrangle){.anchor}

### impl [Positionable](trait.Positionable.html "trait optionstratlib::strategies::base::Positionable"){.trait} for [LongStrangle](../strangle/struct.LongStrangle.html "struct optionstratlib::strategies::strangle::LongStrangle"){.struct} {#impl-positionable-for-longstrangle .code-header}
:::

::: {#impl-Positionable-for-ShortStrangle .section .impl}
[Source](../../../src/optionstratlib/strategies/strangle.rs.html#362-464){.src
.rightside}[§](#impl-Positionable-for-ShortStrangle){.anchor}

### impl [Positionable](trait.Positionable.html "trait optionstratlib::strategies::base::Positionable"){.trait} for [ShortStrangle](../strangle/struct.ShortStrangle.html "struct optionstratlib::strategies::strangle::ShortStrangle"){.struct} {#impl-positionable-for-shortstrangle .code-header}
:::
::::::::::::::::::
:::::::::::::::::::::::::::::::
::::::::::::::::::::::::::::::::
