::::::::::::::::::::::::::: width-limiter
:::::::::::::::::::::::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../index.html)::[strategies](index.html)
:::

# Trait [StrategyConstructor]{.trait}Copy item path

[[Source](../../src/optionstratlib/strategies/build/traits.rs.html#26-50){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub trait StrategyConstructor: Strategies + Greeks {
    // Provided method
    fn get_strategy(_vec_options: &[Position]) -> Result<Self, StrategyError>
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

#### fn [get_strategy](#method.get_strategy){.fn}(\_vec_options: &\[[Position](../model/position/struct.Position.html "struct optionstratlib::model::position::Position"){.struct}\]) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<Self, [StrategyError](../error/strategies/enum.StrategyError.html "enum optionstratlib::error::strategies::StrategyError"){.enum}\> {#fn-get_strategy_vec_options-position---resultself-strategyerror .code-header}

::: where
where Self:
[Sized](https://doc.rust-lang.org/1.86.0/core/marker/trait.Sized.html "trait core::marker::Sized"){.trait},
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

:::::::::::::::::: {#implementors-list}
::: {#impl-StrategyConstructor-for-BearCallSpread .section .impl}
[Source](../../src/optionstratlib/strategies/bear_call_spread.rs.html#232-322){.src
.rightside}[§](#impl-StrategyConstructor-for-BearCallSpread){.anchor}

### impl [StrategyConstructor](trait.StrategyConstructor.html "trait optionstratlib::strategies::StrategyConstructor"){.trait} for [BearCallSpread](bear_call_spread/struct.BearCallSpread.html "struct optionstratlib::strategies::bear_call_spread::BearCallSpread"){.struct} {#impl-strategyconstructor-for-bearcallspread .code-header}
:::

::: {#impl-StrategyConstructor-for-BearPutSpread .section .impl}
[Source](../../src/optionstratlib/strategies/bear_put_spread.rs.html#213-303){.src
.rightside}[§](#impl-StrategyConstructor-for-BearPutSpread){.anchor}

### impl [StrategyConstructor](trait.StrategyConstructor.html "trait optionstratlib::strategies::StrategyConstructor"){.trait} for [BearPutSpread](bear_put_spread/struct.BearPutSpread.html "struct optionstratlib::strategies::bear_put_spread::BearPutSpread"){.struct} {#impl-strategyconstructor-for-bearputspread .code-header}
:::

::: {#impl-StrategyConstructor-for-BullCallSpread .section .impl}
[Source](../../src/optionstratlib/strategies/bull_call_spread.rs.html#221-311){.src
.rightside}[§](#impl-StrategyConstructor-for-BullCallSpread){.anchor}

### impl [StrategyConstructor](trait.StrategyConstructor.html "trait optionstratlib::strategies::StrategyConstructor"){.trait} for [BullCallSpread](bull_call_spread/struct.BullCallSpread.html "struct optionstratlib::strategies::bull_call_spread::BullCallSpread"){.struct} {#impl-strategyconstructor-for-bullcallspread .code-header}
:::

::: {#impl-StrategyConstructor-for-BullPutSpread .section .impl}
[Source](../../src/optionstratlib/strategies/bull_put_spread.rs.html#227-317){.src
.rightside}[§](#impl-StrategyConstructor-for-BullPutSpread){.anchor}

### impl [StrategyConstructor](trait.StrategyConstructor.html "trait optionstratlib::strategies::StrategyConstructor"){.trait} for [BullPutSpread](bull_put_spread/struct.BullPutSpread.html "struct optionstratlib::strategies::bull_put_spread::BullPutSpread"){.struct} {#impl-strategyconstructor-for-bullputspread .code-header}
:::

::: {#impl-StrategyConstructor-for-LongButterflySpread .section .impl}
[Source](../../src/optionstratlib/strategies/butterfly_spread.rs.html#218-327){.src
.rightside}[§](#impl-StrategyConstructor-for-LongButterflySpread){.anchor}

### impl [StrategyConstructor](trait.StrategyConstructor.html "trait optionstratlib::strategies::StrategyConstructor"){.trait} for [LongButterflySpread](butterfly_spread/struct.LongButterflySpread.html "struct optionstratlib::strategies::butterfly_spread::LongButterflySpread"){.struct} {#impl-strategyconstructor-for-longbutterflyspread .code-header}
:::

::: {#impl-StrategyConstructor-for-ShortButterflySpread .section .impl}
[Source](../../src/optionstratlib/strategies/butterfly_spread.rs.html#1194-1303){.src
.rightside}[§](#impl-StrategyConstructor-for-ShortButterflySpread){.anchor}

### impl [StrategyConstructor](trait.StrategyConstructor.html "trait optionstratlib::strategies::StrategyConstructor"){.trait} for [ShortButterflySpread](butterfly_spread/struct.ShortButterflySpread.html "struct optionstratlib::strategies::butterfly_spread::ShortButterflySpread"){.struct} {#impl-strategyconstructor-for-shortbutterflyspread .code-header}
:::

::: {#impl-StrategyConstructor-for-CallButterfly .section .impl}
[Source](../../src/optionstratlib/strategies/call_butterfly.rs.html#242-347){.src
.rightside}[§](#impl-StrategyConstructor-for-CallButterfly){.anchor}

### impl [StrategyConstructor](trait.StrategyConstructor.html "trait optionstratlib::strategies::StrategyConstructor"){.trait} for [CallButterfly](call_butterfly/struct.CallButterfly.html "struct optionstratlib::strategies::call_butterfly::CallButterfly"){.struct} {#impl-strategyconstructor-for-callbutterfly .code-header}
:::

::: {#impl-StrategyConstructor-for-CustomStrategy .section .impl}
[Source](../../src/optionstratlib/strategies/custom.rs.html#333-346){.src
.rightside}[§](#impl-StrategyConstructor-for-CustomStrategy){.anchor}

### impl [StrategyConstructor](trait.StrategyConstructor.html "trait optionstratlib::strategies::StrategyConstructor"){.trait} for [CustomStrategy](custom/struct.CustomStrategy.html "struct optionstratlib::strategies::custom::CustomStrategy"){.struct} {#impl-strategyconstructor-for-customstrategy .code-header}
:::

::: {#impl-StrategyConstructor-for-IronButterfly .section .impl}
[Source](../../src/optionstratlib/strategies/iron_butterfly.rs.html#303-421){.src
.rightside}[§](#impl-StrategyConstructor-for-IronButterfly){.anchor}

### impl [StrategyConstructor](trait.StrategyConstructor.html "trait optionstratlib::strategies::StrategyConstructor"){.trait} for [IronButterfly](iron_butterfly/struct.IronButterfly.html "struct optionstratlib::strategies::iron_butterfly::IronButterfly"){.struct} {#impl-strategyconstructor-for-ironbutterfly .code-header}
:::

::: {#impl-StrategyConstructor-for-IronCondor .section .impl}
[Source](../../src/optionstratlib/strategies/iron_condor.rs.html#309-428){.src
.rightside}[§](#impl-StrategyConstructor-for-IronCondor){.anchor}

### impl [StrategyConstructor](trait.StrategyConstructor.html "trait optionstratlib::strategies::StrategyConstructor"){.trait} for [IronCondor](iron_condor/struct.IronCondor.html "struct optionstratlib::strategies::iron_condor::IronCondor"){.struct} {#impl-strategyconstructor-for-ironcondor .code-header}
:::

::: {#impl-StrategyConstructor-for-PoorMansCoveredCall .section .impl}
[Source](../../src/optionstratlib/strategies/poor_mans_covered_call.rs.html#254-333){.src
.rightside}[§](#impl-StrategyConstructor-for-PoorMansCoveredCall){.anchor}

### impl [StrategyConstructor](trait.StrategyConstructor.html "trait optionstratlib::strategies::StrategyConstructor"){.trait} for [PoorMansCoveredCall](poor_mans_covered_call/struct.PoorMansCoveredCall.html "struct optionstratlib::strategies::poor_mans_covered_call::PoorMansCoveredCall"){.struct} {#impl-strategyconstructor-for-poormanscoveredcall .code-header}
:::

::: {#impl-StrategyConstructor-for-LongStraddle .section .impl}
[Source](../../src/optionstratlib/strategies/straddle.rs.html#1053-1152){.src
.rightside}[§](#impl-StrategyConstructor-for-LongStraddle){.anchor}

### impl [StrategyConstructor](trait.StrategyConstructor.html "trait optionstratlib::strategies::StrategyConstructor"){.trait} for [LongStraddle](straddle/struct.LongStraddle.html "struct optionstratlib::strategies::straddle::LongStraddle"){.struct} {#impl-strategyconstructor-for-longstraddle .code-header}
:::

::: {#impl-StrategyConstructor-for-ShortStraddle .section .impl}
[Source](../../src/optionstratlib/strategies/straddle.rs.html#246-345){.src
.rightside}[§](#impl-StrategyConstructor-for-ShortStraddle){.anchor}

### impl [StrategyConstructor](trait.StrategyConstructor.html "trait optionstratlib::strategies::StrategyConstructor"){.trait} for [ShortStraddle](straddle/struct.ShortStraddle.html "struct optionstratlib::strategies::straddle::ShortStraddle"){.struct} {#impl-strategyconstructor-for-shortstraddle .code-header}
:::

::: {#impl-StrategyConstructor-for-LongStrangle .section .impl}
[Source](../../src/optionstratlib/strategies/strangle.rs.html#1258-1357){.src
.rightside}[§](#impl-StrategyConstructor-for-LongStrangle){.anchor}

### impl [StrategyConstructor](trait.StrategyConstructor.html "trait optionstratlib::strategies::StrategyConstructor"){.trait} for [LongStrangle](strangle/struct.LongStrangle.html "struct optionstratlib::strategies::strangle::LongStrangle"){.struct} {#impl-strategyconstructor-for-longstrangle .code-header}
:::

::: {#impl-StrategyConstructor-for-ShortStrangle .section .impl}
[Source](../../src/optionstratlib/strategies/strangle.rs.html#234-333){.src
.rightside}[§](#impl-StrategyConstructor-for-ShortStrangle){.anchor}

### impl [StrategyConstructor](trait.StrategyConstructor.html "trait optionstratlib::strategies::StrategyConstructor"){.trait} for [ShortStrangle](strangle/struct.ShortStrangle.html "struct optionstratlib::strategies::strangle::ShortStrangle"){.struct} {#impl-strategyconstructor-for-shortstrangle .code-header}
:::
::::::::::::::::::
::::::::::::::::::::::::::
:::::::::::::::::::::::::::
