::::::::::::::::::::::::::::::: width-limiter
:::::::::::::::::::::::::::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../index.html)::[strategies](index.html)
:::

# Trait [StrategyConstructor]{.trait} Copy item path

[[Source](../../src/optionstratlib/strategies/build/traits.rs.html#26-50){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub trait StrategyConstructor: Strategies + Greeks {
    // Provided method
    fn get_strategy(_vec_positions: &[Position]) -> Result<Self, StrategyError>
       where Self: Sized { ... }
}
```

Expand description

::: docblock
Defines a common interface for constructing financial option strategies
from collections of option positions.

This trait extends both the `Strategies` and `Greeks` traits, ensuring
that implementers can both operate as option strategies and calculate
Greek values for risk analysis. It provides a default implementation of
the strategy construction method that returns a "not implemented" error,
which concrete implementations should override.

## [§](#type-requirements){.doc-anchor}Type Requirements

Implementers must also implement:

- `Strategies`: Provides strategy-specific operations and calculations
- `Greeks`: Provides access to option sensitivity calculations (delta,
  gamma, etc.)
:::

## Provided Methods[§](#provided-methods){.anchor} {#provided-methods .section-header}

:::::: methods
:::: {#method.get_strategy .section .method}
[Source](../../src/optionstratlib/strategies/build/traits.rs.html#44-49){.src
.rightside}

#### fn [get_strategy](#method.get_strategy){.fn}(\_vec_positions: &\[[Position](../model/position/struct.Position.html "struct optionstratlib::model::position::Position"){.struct}\]) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<Self, [StrategyError](../error/strategies/enum.StrategyError.html "enum optionstratlib::error::strategies::StrategyError"){.enum}\> {#fn-get_strategy_vec_positions-position---resultself-strategyerror .code-header}

::: where
where Self:
[Sized](https://doc.rust-lang.org/1.91.1/core/marker/trait.Sized.html "trait core::marker::Sized"){.trait},
:::
::::

::: docblock
Attempts to construct a strategy from a vector of option positions.

This method analyzes the provided option positions and attempts to
recognize and construct a specific options strategy. The default
implementation returns a `NotImplemented` error, so concrete types must
provide their own implementation.

##### [§](#parameters){.doc-anchor}Parameters

- `_vec_options` - A slice of `Position` objects representing the option
  positions to analyze

##### [§](#returns){.doc-anchor}Returns

- `Ok(Self)` - The successfully constructed strategy
- `Err(StrategyError)` - If the positions don't match the expected
  pattern for this strategy type
:::
::::::

## Implementors[§](#implementors){.anchor} {#implementors .section-header}

:::::::::::::::::::::: {#implementors-list}
::: {#impl-StrategyConstructor-for-BearCallSpread .section .impl}
[Source](../../src/optionstratlib/strategies/bear_call_spread.rs.html#228-323){.src
.rightside}[§](#impl-StrategyConstructor-for-BearCallSpread){.anchor}

### impl [StrategyConstructor](trait.StrategyConstructor.html "trait optionstratlib::strategies::StrategyConstructor"){.trait} for [BearCallSpread](bear_call_spread/struct.BearCallSpread.html "struct optionstratlib::strategies::bear_call_spread::BearCallSpread"){.struct} {#impl-strategyconstructor-for-bearcallspread .code-header}
:::

::: {#impl-StrategyConstructor-for-BearPutSpread .section .impl}
[Source](../../src/optionstratlib/strategies/bear_put_spread.rs.html#226-321){.src
.rightside}[§](#impl-StrategyConstructor-for-BearPutSpread){.anchor}

### impl [StrategyConstructor](trait.StrategyConstructor.html "trait optionstratlib::strategies::StrategyConstructor"){.trait} for [BearPutSpread](bear_put_spread/struct.BearPutSpread.html "struct optionstratlib::strategies::bear_put_spread::BearPutSpread"){.struct} {#impl-strategyconstructor-for-bearputspread .code-header}
:::

::: {#impl-StrategyConstructor-for-BullCallSpread .section .impl}
[Source](../../src/optionstratlib/strategies/bull_call_spread.rs.html#231-325){.src
.rightside}[§](#impl-StrategyConstructor-for-BullCallSpread){.anchor}

### impl [StrategyConstructor](trait.StrategyConstructor.html "trait optionstratlib::strategies::StrategyConstructor"){.trait} for [BullCallSpread](bull_call_spread/struct.BullCallSpread.html "struct optionstratlib::strategies::bull_call_spread::BullCallSpread"){.struct} {#impl-strategyconstructor-for-bullcallspread .code-header}
:::

::: {#impl-StrategyConstructor-for-BullPutSpread .section .impl}
[Source](../../src/optionstratlib/strategies/bull_put_spread.rs.html#237-331){.src
.rightside}[§](#impl-StrategyConstructor-for-BullPutSpread){.anchor}

### impl [StrategyConstructor](trait.StrategyConstructor.html "trait optionstratlib::strategies::StrategyConstructor"){.trait} for [BullPutSpread](bull_put_spread/struct.BullPutSpread.html "struct optionstratlib::strategies::bull_put_spread::BullPutSpread"){.struct} {#impl-strategyconstructor-for-bullputspread .code-header}
:::

::: {#impl-StrategyConstructor-for-CallButterfly .section .impl}
[Source](../../src/optionstratlib/strategies/call_butterfly.rs.html#250-363){.src
.rightside}[§](#impl-StrategyConstructor-for-CallButterfly){.anchor}

### impl [StrategyConstructor](trait.StrategyConstructor.html "trait optionstratlib::strategies::StrategyConstructor"){.trait} for [CallButterfly](call_butterfly/struct.CallButterfly.html "struct optionstratlib::strategies::call_butterfly::CallButterfly"){.struct} {#impl-strategyconstructor-for-callbutterfly .code-header}
:::

::: {#impl-StrategyConstructor-for-CustomStrategy .section .impl}
[Source](../../src/optionstratlib/strategies/custom.rs.html#343-356){.src
.rightside}[§](#impl-StrategyConstructor-for-CustomStrategy){.anchor}

### impl [StrategyConstructor](trait.StrategyConstructor.html "trait optionstratlib::strategies::StrategyConstructor"){.trait} for [CustomStrategy](custom/struct.CustomStrategy.html "struct optionstratlib::strategies::custom::CustomStrategy"){.struct} {#impl-strategyconstructor-for-customstrategy .code-header}
:::

::: {#impl-StrategyConstructor-for-IronButterfly .section .impl}
[Source](../../src/optionstratlib/strategies/iron_butterfly.rs.html#318-436){.src
.rightside}[§](#impl-StrategyConstructor-for-IronButterfly){.anchor}

### impl [StrategyConstructor](trait.StrategyConstructor.html "trait optionstratlib::strategies::StrategyConstructor"){.trait} for [IronButterfly](iron_butterfly/struct.IronButterfly.html "struct optionstratlib::strategies::iron_butterfly::IronButterfly"){.struct} {#impl-strategyconstructor-for-ironbutterfly .code-header}
:::

::: {#impl-StrategyConstructor-for-IronCondor .section .impl}
[Source](../../src/optionstratlib/strategies/iron_condor.rs.html#324-451){.src
.rightside}[§](#impl-StrategyConstructor-for-IronCondor){.anchor}

### impl [StrategyConstructor](trait.StrategyConstructor.html "trait optionstratlib::strategies::StrategyConstructor"){.trait} for [IronCondor](iron_condor/struct.IronCondor.html "struct optionstratlib::strategies::iron_condor::IronCondor"){.struct} {#impl-strategyconstructor-for-ironcondor .code-header}
:::

::: {#impl-StrategyConstructor-for-LongButterflySpread .section .impl}
[Source](../../src/optionstratlib/strategies/long_butterfly_spread.rs.html#232-347){.src
.rightside}[§](#impl-StrategyConstructor-for-LongButterflySpread){.anchor}

### impl [StrategyConstructor](trait.StrategyConstructor.html "trait optionstratlib::strategies::StrategyConstructor"){.trait} for [LongButterflySpread](long_butterfly_spread/struct.LongButterflySpread.html "struct optionstratlib::strategies::long_butterfly_spread::LongButterflySpread"){.struct} {#impl-strategyconstructor-for-longbutterflyspread .code-header}
:::

::: {#impl-StrategyConstructor-for-LongCall .section .impl}
[Source](../../src/optionstratlib/strategies/long_call.rs.html#408-412){.src
.rightside}[§](#impl-StrategyConstructor-for-LongCall){.anchor}

### impl [StrategyConstructor](trait.StrategyConstructor.html "trait optionstratlib::strategies::StrategyConstructor"){.trait} for [LongCall](long_call/struct.LongCall.html "struct optionstratlib::strategies::long_call::LongCall"){.struct} {#impl-strategyconstructor-for-longcall .code-header}
:::

::: {#impl-StrategyConstructor-for-LongPut .section .impl}
[Source](../../src/optionstratlib/strategies/long_put.rs.html#405-409){.src
.rightside}[§](#impl-StrategyConstructor-for-LongPut){.anchor}

### impl [StrategyConstructor](trait.StrategyConstructor.html "trait optionstratlib::strategies::StrategyConstructor"){.trait} for [LongPut](long_put/struct.LongPut.html "struct optionstratlib::strategies::long_put::LongPut"){.struct} {#impl-strategyconstructor-for-longput .code-header}
:::

::: {#impl-StrategyConstructor-for-LongStraddle .section .impl}
[Source](../../src/optionstratlib/strategies/long_straddle.rs.html#242-345){.src
.rightside}[§](#impl-StrategyConstructor-for-LongStraddle){.anchor}

### impl [StrategyConstructor](trait.StrategyConstructor.html "trait optionstratlib::strategies::StrategyConstructor"){.trait} for [LongStraddle](long_straddle/struct.LongStraddle.html "struct optionstratlib::strategies::long_straddle::LongStraddle"){.struct} {#impl-strategyconstructor-for-longstraddle .code-header}
:::

::: {#impl-StrategyConstructor-for-LongStrangle .section .impl}
[Source](../../src/optionstratlib/strategies/long_strangle.rs.html#255-358){.src
.rightside}[§](#impl-StrategyConstructor-for-LongStrangle){.anchor}

### impl [StrategyConstructor](trait.StrategyConstructor.html "trait optionstratlib::strategies::StrategyConstructor"){.trait} for [LongStrangle](long_strangle/struct.LongStrangle.html "struct optionstratlib::strategies::long_strangle::LongStrangle"){.struct} {#impl-strategyconstructor-for-longstrangle .code-header}
:::

::: {#impl-StrategyConstructor-for-PoorMansCoveredCall .section .impl}
[Source](../../src/optionstratlib/strategies/poor_mans_covered_call.rs.html#260-343){.src
.rightside}[§](#impl-StrategyConstructor-for-PoorMansCoveredCall){.anchor}

### impl [StrategyConstructor](trait.StrategyConstructor.html "trait optionstratlib::strategies::StrategyConstructor"){.trait} for [PoorMansCoveredCall](poor_mans_covered_call/struct.PoorMansCoveredCall.html "struct optionstratlib::strategies::poor_mans_covered_call::PoorMansCoveredCall"){.struct} {#impl-strategyconstructor-for-poormanscoveredcall .code-header}
:::

::: {#impl-StrategyConstructor-for-ShortButterflySpread .section .impl}
[Source](../../src/optionstratlib/strategies/short_butterfly_spread.rs.html#224-339){.src
.rightside}[§](#impl-StrategyConstructor-for-ShortButterflySpread){.anchor}

### impl [StrategyConstructor](trait.StrategyConstructor.html "trait optionstratlib::strategies::StrategyConstructor"){.trait} for [ShortButterflySpread](short_butterfly_spread/struct.ShortButterflySpread.html "struct optionstratlib::strategies::short_butterfly_spread::ShortButterflySpread"){.struct} {#impl-strategyconstructor-for-shortbutterflyspread .code-header}
:::

::: {#impl-StrategyConstructor-for-ShortCall .section .impl}
[Source](../../src/optionstratlib/strategies/short_call.rs.html#416-420){.src
.rightside}[§](#impl-StrategyConstructor-for-ShortCall){.anchor}

### impl [StrategyConstructor](trait.StrategyConstructor.html "trait optionstratlib::strategies::StrategyConstructor"){.trait} for [ShortCall](short_call/struct.ShortCall.html "struct optionstratlib::strategies::short_call::ShortCall"){.struct} {#impl-strategyconstructor-for-shortcall .code-header}
:::

::: {#impl-StrategyConstructor-for-ShortPut .section .impl}
[Source](../../src/optionstratlib/strategies/short_put.rs.html#410-414){.src
.rightside}[§](#impl-StrategyConstructor-for-ShortPut){.anchor}

### impl [StrategyConstructor](trait.StrategyConstructor.html "trait optionstratlib::strategies::StrategyConstructor"){.trait} for [ShortPut](short_put/struct.ShortPut.html "struct optionstratlib::strategies::short_put::ShortPut"){.struct} {#impl-strategyconstructor-for-shortput .code-header}
:::

::: {#impl-StrategyConstructor-for-ShortStraddle .section .impl}
[Source](../../src/optionstratlib/strategies/short_straddle.rs.html#254-357){.src
.rightside}[§](#impl-StrategyConstructor-for-ShortStraddle){.anchor}

### impl [StrategyConstructor](trait.StrategyConstructor.html "trait optionstratlib::strategies::StrategyConstructor"){.trait} for [ShortStraddle](short_straddle/struct.ShortStraddle.html "struct optionstratlib::strategies::short_straddle::ShortStraddle"){.struct} {#impl-strategyconstructor-for-shortstraddle .code-header}
:::

::: {#impl-StrategyConstructor-for-ShortStrangle .section .impl}
[Source](../../src/optionstratlib/strategies/short_strangle.rs.html#242-345){.src
.rightside}[§](#impl-StrategyConstructor-for-ShortStrangle){.anchor}

### impl [StrategyConstructor](trait.StrategyConstructor.html "trait optionstratlib::strategies::StrategyConstructor"){.trait} for [ShortStrangle](short_strangle/struct.ShortStrangle.html "struct optionstratlib::strategies::short_strangle::ShortStrangle"){.struct} {#impl-strategyconstructor-for-shortstrangle .code-header}
:::
::::::::::::::::::::::
::::::::::::::::::::::::::::::
:::::::::::::::::::::::::::::::
