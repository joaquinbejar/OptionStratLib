:::::::::::::::::::::::::::: width-limiter
::::::::::::::::::::::::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../../index.html)::[strategies](../index.html)::[base](index.html)
:::

# Trait [BreakEvenable]{.trait}Copy item path

[[Source](../../../src/optionstratlib/strategies/base.rs.html#703-728){.src}
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
[Source](../../../src/optionstratlib/strategies/base.rs.html#711-716){.src
.rightside}

#### fn [get_break_even_points](#method.get_break_even_points){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<&[Vec](https://doc.rust-lang.org/1.86.0/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\>, [StrategyError](../../error/strategies/enum.StrategyError.html "enum optionstratlib::error::strategies::StrategyError"){.enum}\> {#fn-get_break_even_pointsself---resultvecpositive-strategyerror .code-header}
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
[Source](../../../src/optionstratlib/strategies/base.rs.html#725-727){.src
.rightside}

#### fn [update_break_even_points](#method.update_break_even_points){.fn}(&mut self) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[()](https://doc.rust-lang.org/1.86.0/std/primitive.unit.html){.primitive}, [StrategyError](../../error/strategies/enum.StrategyError.html "enum optionstratlib::error::strategies::StrategyError"){.enum}\> {#fn-update_break_even_pointsmut-self---result-strategyerror .code-header}
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

:::::::::::::::::: {#implementors-list}
::: {#impl-BreakEvenable-for-BearCallSpread .section .impl}
[Source](../../../src/optionstratlib/strategies/bear_call_spread.rs.html#324-340){.src
.rightside}[§](#impl-BreakEvenable-for-BearCallSpread){.anchor}

### impl [BreakEvenable](trait.BreakEvenable.html "trait optionstratlib::strategies::base::BreakEvenable"){.trait} for [BearCallSpread](../bear_call_spread/struct.BearCallSpread.html "struct optionstratlib::strategies::bear_call_spread::BearCallSpread"){.struct} {#impl-breakevenable-for-bearcallspread .code-header}
:::

::: {#impl-BreakEvenable-for-BearPutSpread .section .impl}
[Source](../../../src/optionstratlib/strategies/bear_put_spread.rs.html#305-320){.src
.rightside}[§](#impl-BreakEvenable-for-BearPutSpread){.anchor}

### impl [BreakEvenable](trait.BreakEvenable.html "trait optionstratlib::strategies::base::BreakEvenable"){.trait} for [BearPutSpread](../bear_put_spread/struct.BearPutSpread.html "struct optionstratlib::strategies::bear_put_spread::BearPutSpread"){.struct} {#impl-breakevenable-for-bearputspread .code-header}
:::

::: {#impl-BreakEvenable-for-BullCallSpread .section .impl}
[Source](../../../src/optionstratlib/strategies/bull_call_spread.rs.html#313-329){.src
.rightside}[§](#impl-BreakEvenable-for-BullCallSpread){.anchor}

### impl [BreakEvenable](trait.BreakEvenable.html "trait optionstratlib::strategies::base::BreakEvenable"){.trait} for [BullCallSpread](../bull_call_spread/struct.BullCallSpread.html "struct optionstratlib::strategies::bull_call_spread::BullCallSpread"){.struct} {#impl-breakevenable-for-bullcallspread .code-header}
:::

::: {#impl-BreakEvenable-for-BullPutSpread .section .impl}
[Source](../../../src/optionstratlib/strategies/bull_put_spread.rs.html#319-335){.src
.rightside}[§](#impl-BreakEvenable-for-BullPutSpread){.anchor}

### impl [BreakEvenable](trait.BreakEvenable.html "trait optionstratlib::strategies::base::BreakEvenable"){.trait} for [BullPutSpread](../bull_put_spread/struct.BullPutSpread.html "struct optionstratlib::strategies::bull_put_spread::BullPutSpread"){.struct} {#impl-breakevenable-for-bullputspread .code-header}
:::

::: {#impl-BreakEvenable-for-LongButterflySpread .section .impl}
[Source](../../../src/optionstratlib/strategies/butterfly_spread.rs.html#329-356){.src
.rightside}[§](#impl-BreakEvenable-for-LongButterflySpread){.anchor}

### impl [BreakEvenable](trait.BreakEvenable.html "trait optionstratlib::strategies::base::BreakEvenable"){.trait} for [LongButterflySpread](../butterfly_spread/struct.LongButterflySpread.html "struct optionstratlib::strategies::butterfly_spread::LongButterflySpread"){.struct} {#impl-breakevenable-for-longbutterflyspread .code-header}
:::

::: {#impl-BreakEvenable-for-ShortButterflySpread .section .impl}
[Source](../../../src/optionstratlib/strategies/butterfly_spread.rs.html#1305-1332){.src
.rightside}[§](#impl-BreakEvenable-for-ShortButterflySpread){.anchor}

### impl [BreakEvenable](trait.BreakEvenable.html "trait optionstratlib::strategies::base::BreakEvenable"){.trait} for [ShortButterflySpread](../butterfly_spread/struct.ShortButterflySpread.html "struct optionstratlib::strategies::butterfly_spread::ShortButterflySpread"){.struct} {#impl-breakevenable-for-shortbutterflyspread .code-header}
:::

::: {#impl-BreakEvenable-for-CallButterfly .section .impl}
[Source](../../../src/optionstratlib/strategies/call_butterfly.rs.html#349-374){.src
.rightside}[§](#impl-BreakEvenable-for-CallButterfly){.anchor}

### impl [BreakEvenable](trait.BreakEvenable.html "trait optionstratlib::strategies::base::BreakEvenable"){.trait} for [CallButterfly](../call_butterfly/struct.CallButterfly.html "struct optionstratlib::strategies::call_butterfly::CallButterfly"){.struct} {#impl-breakevenable-for-callbutterfly .code-header}
:::

::: {#impl-BreakEvenable-for-CustomStrategy .section .impl}
[Source](../../../src/optionstratlib/strategies/custom.rs.html#348-357){.src
.rightside}[§](#impl-BreakEvenable-for-CustomStrategy){.anchor}

### impl [BreakEvenable](trait.BreakEvenable.html "trait optionstratlib::strategies::base::BreakEvenable"){.trait} for [CustomStrategy](../custom/struct.CustomStrategy.html "struct optionstratlib::strategies::custom::CustomStrategy"){.struct} {#impl-breakevenable-for-customstrategy .code-header}
:::

::: {#impl-BreakEvenable-for-IronButterfly .section .impl}
[Source](../../../src/optionstratlib/strategies/iron_butterfly.rs.html#423-442){.src
.rightside}[§](#impl-BreakEvenable-for-IronButterfly){.anchor}

### impl [BreakEvenable](trait.BreakEvenable.html "trait optionstratlib::strategies::base::BreakEvenable"){.trait} for [IronButterfly](../iron_butterfly/struct.IronButterfly.html "struct optionstratlib::strategies::iron_butterfly::IronButterfly"){.struct} {#impl-breakevenable-for-ironbutterfly .code-header}
:::

::: {#impl-BreakEvenable-for-IronCondor .section .impl}
[Source](../../../src/optionstratlib/strategies/iron_condor.rs.html#430-449){.src
.rightside}[§](#impl-BreakEvenable-for-IronCondor){.anchor}

### impl [BreakEvenable](trait.BreakEvenable.html "trait optionstratlib::strategies::base::BreakEvenable"){.trait} for [IronCondor](../iron_condor/struct.IronCondor.html "struct optionstratlib::strategies::iron_condor::IronCondor"){.struct} {#impl-breakevenable-for-ironcondor .code-header}
:::

::: {#impl-BreakEvenable-for-PoorMansCoveredCall .section .impl}
[Source](../../../src/optionstratlib/strategies/poor_mans_covered_call.rs.html#335-350){.src
.rightside}[§](#impl-BreakEvenable-for-PoorMansCoveredCall){.anchor}

### impl [BreakEvenable](trait.BreakEvenable.html "trait optionstratlib::strategies::base::BreakEvenable"){.trait} for [PoorMansCoveredCall](../poor_mans_covered_call/struct.PoorMansCoveredCall.html "struct optionstratlib::strategies::poor_mans_covered_call::PoorMansCoveredCall"){.struct} {#impl-breakevenable-for-poormanscoveredcall .code-header}
:::

::: {#impl-BreakEvenable-for-LongStraddle .section .impl}
[Source](../../../src/optionstratlib/strategies/straddle.rs.html#1154-1177){.src
.rightside}[§](#impl-BreakEvenable-for-LongStraddle){.anchor}

### impl [BreakEvenable](trait.BreakEvenable.html "trait optionstratlib::strategies::base::BreakEvenable"){.trait} for [LongStraddle](../straddle/struct.LongStraddle.html "struct optionstratlib::strategies::straddle::LongStraddle"){.struct} {#impl-breakevenable-for-longstraddle .code-header}
:::

::: {#impl-BreakEvenable-for-ShortStraddle .section .impl}
[Source](../../../src/optionstratlib/strategies/straddle.rs.html#347-372){.src
.rightside}[§](#impl-BreakEvenable-for-ShortStraddle){.anchor}

### impl [BreakEvenable](trait.BreakEvenable.html "trait optionstratlib::strategies::base::BreakEvenable"){.trait} for [ShortStraddle](../straddle/struct.ShortStraddle.html "struct optionstratlib::strategies::straddle::ShortStraddle"){.struct} {#impl-breakevenable-for-shortstraddle .code-header}
:::

::: {#impl-BreakEvenable-for-LongStrangle .section .impl}
[Source](../../../src/optionstratlib/strategies/strangle.rs.html#1359-1382){.src
.rightside}[§](#impl-BreakEvenable-for-LongStrangle){.anchor}

### impl [BreakEvenable](trait.BreakEvenable.html "trait optionstratlib::strategies::base::BreakEvenable"){.trait} for [LongStrangle](../strangle/struct.LongStrangle.html "struct optionstratlib::strategies::strangle::LongStrangle"){.struct} {#impl-breakevenable-for-longstrangle .code-header}
:::

::: {#impl-BreakEvenable-for-ShortStrangle .section .impl}
[Source](../../../src/optionstratlib/strategies/strangle.rs.html#335-360){.src
.rightside}[§](#impl-BreakEvenable-for-ShortStrangle){.anchor}

### impl [BreakEvenable](trait.BreakEvenable.html "trait optionstratlib::strategies::base::BreakEvenable"){.trait} for [ShortStrangle](../strangle/struct.ShortStrangle.html "struct optionstratlib::strategies::strangle::ShortStrangle"){.struct} {#impl-breakevenable-for-shortstrangle .code-header}
:::
::::::::::::::::::
:::::::::::::::::::::::::::
::::::::::::::::::::::::::::
