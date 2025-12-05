:::::::::::::::::::::::::::::::: width-limiter
::::::::::::::::::::::::::::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../../index.html)::[strategies](../index.html)::[base](index.html)
:::

# Trait [BreakEvenable]{.trait} Copy item path

[[Source](../../../src/optionstratlib/strategies/base.rs.html#1111-1136){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub trait BreakEvenable {
    // Provided methods
    fn get_break_even_points(&self) -> Result<&Vec<Positive>, StrategyError> { ... }
    fn update_break_even_points(&mut self) -> Result<(), StrategyError> { ... }
}
```

Expand description

::: docblock
Trait for strategies that can calculate and update break-even points.

This trait provides methods for retrieving and updating break-even
points, which are crucial for determining profitability in various
trading scenarios.
:::

## Provided Methods[§](#provided-methods){.anchor} {#provided-methods .section-header}

::::::: methods
::: {#method.get_break_even_points .section .method}
[Source](../../../src/optionstratlib/strategies/base.rs.html#1119-1124){.src
.rightside}

#### fn [get_break_even_points](#method.get_break_even_points){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<&[Vec](https://doc.rust-lang.org/1.91.1/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\>, [StrategyError](../../error/strategies/enum.StrategyError.html "enum optionstratlib::error::strategies::StrategyError"){.enum}\> {#fn-get_break_even_pointsself---resultvecpositive-strategyerror .code-header}
:::

::: docblock
Retrieves the break-even points for the strategy.

Returns a `Result` containing a reference to a vector of `Positive`
values representing the break-even points, or a `StrategyError` if the
operation is not supported for the specific strategy.

The default implementation returns a `StrategyError::OperationError`
with `OperationErrorKind::NotSupported`. Strategies implementing this
trait should override this method if they support break-even point
calculations.
:::

::: {#method.update_break_even_points .section .method}
[Source](../../../src/optionstratlib/strategies/base.rs.html#1133-1135){.src
.rightside}

#### fn [update_break_even_points](#method.update_break_even_points){.fn}(&mut self) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[()](https://doc.rust-lang.org/1.91.1/std/primitive.unit.html){.primitive}, [StrategyError](../../error/strategies/enum.StrategyError.html "enum optionstratlib::error::strategies::StrategyError"){.enum}\> {#fn-update_break_even_pointsmut-self---result-strategyerror .code-header}
:::

::: docblock
Updates the break-even points for the strategy.

This method is responsible for recalculating and updating the break-even
points based on the current state of the strategy.

The default implementation returns a `NotImplemented` error. Strategies
implementing this trait should override this method to provide specific
update logic.
:::
:::::::

## Implementors[§](#implementors){.anchor} {#implementors .section-header}

:::::::::::::::::::::: {#implementors-list}
::: {#impl-BreakEvenable-for-BearCallSpread .section .impl}
[Source](../../../src/optionstratlib/strategies/bear_call_spread.rs.html#325-341){.src
.rightside}[§](#impl-BreakEvenable-for-BearCallSpread){.anchor}

### impl [BreakEvenable](trait.BreakEvenable.html "trait optionstratlib::strategies::base::BreakEvenable"){.trait} for [BearCallSpread](../bear_call_spread/struct.BearCallSpread.html "struct optionstratlib::strategies::bear_call_spread::BearCallSpread"){.struct} {#impl-breakevenable-for-bearcallspread .code-header}
:::

::: {#impl-BreakEvenable-for-BearPutSpread .section .impl}
[Source](../../../src/optionstratlib/strategies/bear_put_spread.rs.html#323-339){.src
.rightside}[§](#impl-BreakEvenable-for-BearPutSpread){.anchor}

### impl [BreakEvenable](trait.BreakEvenable.html "trait optionstratlib::strategies::base::BreakEvenable"){.trait} for [BearPutSpread](../bear_put_spread/struct.BearPutSpread.html "struct optionstratlib::strategies::bear_put_spread::BearPutSpread"){.struct} {#impl-breakevenable-for-bearputspread .code-header}
:::

::: {#impl-BreakEvenable-for-BullCallSpread .section .impl}
[Source](../../../src/optionstratlib/strategies/bull_call_spread.rs.html#327-343){.src
.rightside}[§](#impl-BreakEvenable-for-BullCallSpread){.anchor}

### impl [BreakEvenable](trait.BreakEvenable.html "trait optionstratlib::strategies::base::BreakEvenable"){.trait} for [BullCallSpread](../bull_call_spread/struct.BullCallSpread.html "struct optionstratlib::strategies::bull_call_spread::BullCallSpread"){.struct} {#impl-breakevenable-for-bullcallspread .code-header}
:::

::: {#impl-BreakEvenable-for-BullPutSpread .section .impl}
[Source](../../../src/optionstratlib/strategies/bull_put_spread.rs.html#333-349){.src
.rightside}[§](#impl-BreakEvenable-for-BullPutSpread){.anchor}

### impl [BreakEvenable](trait.BreakEvenable.html "trait optionstratlib::strategies::base::BreakEvenable"){.trait} for [BullPutSpread](../bull_put_spread/struct.BullPutSpread.html "struct optionstratlib::strategies::bull_put_spread::BullPutSpread"){.struct} {#impl-breakevenable-for-bullputspread .code-header}
:::

::: {#impl-BreakEvenable-for-CallButterfly .section .impl}
[Source](../../../src/optionstratlib/strategies/call_butterfly.rs.html#365-390){.src
.rightside}[§](#impl-BreakEvenable-for-CallButterfly){.anchor}

### impl [BreakEvenable](trait.BreakEvenable.html "trait optionstratlib::strategies::base::BreakEvenable"){.trait} for [CallButterfly](../call_butterfly/struct.CallButterfly.html "struct optionstratlib::strategies::call_butterfly::CallButterfly"){.struct} {#impl-breakevenable-for-callbutterfly .code-header}
:::

::: {#impl-BreakEvenable-for-CustomStrategy .section .impl}
[Source](../../../src/optionstratlib/strategies/custom.rs.html#358-385){.src
.rightside}[§](#impl-BreakEvenable-for-CustomStrategy){.anchor}

### impl [BreakEvenable](trait.BreakEvenable.html "trait optionstratlib::strategies::base::BreakEvenable"){.trait} for [CustomStrategy](../custom/struct.CustomStrategy.html "struct optionstratlib::strategies::custom::CustomStrategy"){.struct} {#impl-breakevenable-for-customstrategy .code-header}
:::

::: {#impl-BreakEvenable-for-IronButterfly .section .impl}
[Source](../../../src/optionstratlib/strategies/iron_butterfly.rs.html#438-457){.src
.rightside}[§](#impl-BreakEvenable-for-IronButterfly){.anchor}

### impl [BreakEvenable](trait.BreakEvenable.html "trait optionstratlib::strategies::base::BreakEvenable"){.trait} for [IronButterfly](../iron_butterfly/struct.IronButterfly.html "struct optionstratlib::strategies::iron_butterfly::IronButterfly"){.struct} {#impl-breakevenable-for-ironbutterfly .code-header}
:::

::: {#impl-BreakEvenable-for-IronCondor .section .impl}
[Source](../../../src/optionstratlib/strategies/iron_condor.rs.html#453-472){.src
.rightside}[§](#impl-BreakEvenable-for-IronCondor){.anchor}

### impl [BreakEvenable](trait.BreakEvenable.html "trait optionstratlib::strategies::base::BreakEvenable"){.trait} for [IronCondor](../iron_condor/struct.IronCondor.html "struct optionstratlib::strategies::iron_condor::IronCondor"){.struct} {#impl-breakevenable-for-ironcondor .code-header}
:::

::: {#impl-BreakEvenable-for-LongButterflySpread .section .impl}
[Source](../../../src/optionstratlib/strategies/long_butterfly_spread.rs.html#349-376){.src
.rightside}[§](#impl-BreakEvenable-for-LongButterflySpread){.anchor}

### impl [BreakEvenable](trait.BreakEvenable.html "trait optionstratlib::strategies::base::BreakEvenable"){.trait} for [LongButterflySpread](../long_butterfly_spread/struct.LongButterflySpread.html "struct optionstratlib::strategies::long_butterfly_spread::LongButterflySpread"){.struct} {#impl-breakevenable-for-longbutterflyspread .code-header}
:::

::: {#impl-BreakEvenable-for-LongCall .section .impl}
[Source](../../../src/optionstratlib/strategies/long_call.rs.html#256-272){.src
.rightside}[§](#impl-BreakEvenable-for-LongCall){.anchor}

### impl [BreakEvenable](trait.BreakEvenable.html "trait optionstratlib::strategies::base::BreakEvenable"){.trait} for [LongCall](../long_call/struct.LongCall.html "struct optionstratlib::strategies::long_call::LongCall"){.struct} {#impl-breakevenable-for-longcall .code-header}
:::

::: {#impl-BreakEvenable-for-LongPut .section .impl}
[Source](../../../src/optionstratlib/strategies/long_put.rs.html#253-269){.src
.rightside}[§](#impl-BreakEvenable-for-LongPut){.anchor}

### impl [BreakEvenable](trait.BreakEvenable.html "trait optionstratlib::strategies::base::BreakEvenable"){.trait} for [LongPut](../long_put/struct.LongPut.html "struct optionstratlib::strategies::long_put::LongPut"){.struct} {#impl-breakevenable-for-longput .code-header}
:::

::: {#impl-BreakEvenable-for-LongStraddle .section .impl}
[Source](../../../src/optionstratlib/strategies/long_straddle.rs.html#347-370){.src
.rightside}[§](#impl-BreakEvenable-for-LongStraddle){.anchor}

### impl [BreakEvenable](trait.BreakEvenable.html "trait optionstratlib::strategies::base::BreakEvenable"){.trait} for [LongStraddle](../long_straddle/struct.LongStraddle.html "struct optionstratlib::strategies::long_straddle::LongStraddle"){.struct} {#impl-breakevenable-for-longstraddle .code-header}
:::

::: {#impl-BreakEvenable-for-LongStrangle .section .impl}
[Source](../../../src/optionstratlib/strategies/long_strangle.rs.html#360-383){.src
.rightside}[§](#impl-BreakEvenable-for-LongStrangle){.anchor}

### impl [BreakEvenable](trait.BreakEvenable.html "trait optionstratlib::strategies::base::BreakEvenable"){.trait} for [LongStrangle](../long_strangle/struct.LongStrangle.html "struct optionstratlib::strategies::long_strangle::LongStrangle"){.struct} {#impl-breakevenable-for-longstrangle .code-header}
:::

::: {#impl-BreakEvenable-for-PoorMansCoveredCall .section .impl}
[Source](../../../src/optionstratlib/strategies/poor_mans_covered_call.rs.html#345-360){.src
.rightside}[§](#impl-BreakEvenable-for-PoorMansCoveredCall){.anchor}

### impl [BreakEvenable](trait.BreakEvenable.html "trait optionstratlib::strategies::base::BreakEvenable"){.trait} for [PoorMansCoveredCall](../poor_mans_covered_call/struct.PoorMansCoveredCall.html "struct optionstratlib::strategies::poor_mans_covered_call::PoorMansCoveredCall"){.struct} {#impl-breakevenable-for-poormanscoveredcall .code-header}
:::

::: {#impl-BreakEvenable-for-ShortButterflySpread .section .impl}
[Source](../../../src/optionstratlib/strategies/short_butterfly_spread.rs.html#341-369){.src
.rightside}[§](#impl-BreakEvenable-for-ShortButterflySpread){.anchor}

### impl [BreakEvenable](trait.BreakEvenable.html "trait optionstratlib::strategies::base::BreakEvenable"){.trait} for [ShortButterflySpread](../short_butterfly_spread/struct.ShortButterflySpread.html "struct optionstratlib::strategies::short_butterfly_spread::ShortButterflySpread"){.struct} {#impl-breakevenable-for-shortbutterflyspread .code-header}
:::

::: {#impl-BreakEvenable-for-ShortCall .section .impl}
[Source](../../../src/optionstratlib/strategies/short_call.rs.html#264-284){.src
.rightside}[§](#impl-BreakEvenable-for-ShortCall){.anchor}

### impl [BreakEvenable](trait.BreakEvenable.html "trait optionstratlib::strategies::base::BreakEvenable"){.trait} for [ShortCall](../short_call/struct.ShortCall.html "struct optionstratlib::strategies::short_call::ShortCall"){.struct} {#impl-breakevenable-for-shortcall .code-header}
:::

::: {#impl-BreakEvenable-for-ShortPut .section .impl}
[Source](../../../src/optionstratlib/strategies/short_put.rs.html#258-274){.src
.rightside}[§](#impl-BreakEvenable-for-ShortPut){.anchor}

### impl [BreakEvenable](trait.BreakEvenable.html "trait optionstratlib::strategies::base::BreakEvenable"){.trait} for [ShortPut](../short_put/struct.ShortPut.html "struct optionstratlib::strategies::short_put::ShortPut"){.struct} {#impl-breakevenable-for-shortput .code-header}
:::

::: {#impl-BreakEvenable-for-ShortStraddle .section .impl}
[Source](../../../src/optionstratlib/strategies/short_straddle.rs.html#359-384){.src
.rightside}[§](#impl-BreakEvenable-for-ShortStraddle){.anchor}

### impl [BreakEvenable](trait.BreakEvenable.html "trait optionstratlib::strategies::base::BreakEvenable"){.trait} for [ShortStraddle](../short_straddle/struct.ShortStraddle.html "struct optionstratlib::strategies::short_straddle::ShortStraddle"){.struct} {#impl-breakevenable-for-shortstraddle .code-header}
:::

::: {#impl-BreakEvenable-for-ShortStrangle .section .impl}
[Source](../../../src/optionstratlib/strategies/short_strangle.rs.html#347-371){.src
.rightside}[§](#impl-BreakEvenable-for-ShortStrangle){.anchor}

### impl [BreakEvenable](trait.BreakEvenable.html "trait optionstratlib::strategies::base::BreakEvenable"){.trait} for [ShortStrangle](../short_strangle/struct.ShortStrangle.html "struct optionstratlib::strategies::short_strangle::ShortStrangle"){.struct} {#impl-breakevenable-for-shortstrangle .code-header}
:::
::::::::::::::::::::::
:::::::::::::::::::::::::::::::
::::::::::::::::::::::::::::::::
