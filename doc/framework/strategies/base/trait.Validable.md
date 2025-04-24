:::::::::::::::::::::::::: width-limiter
::::::::::::::::::::::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../../index.html)::[strategies](../index.html)::[base](index.html)
:::

# Trait [Validable]{.trait}Copy item path

[[Source](../../../src/optionstratlib/strategies/base.rs.html#735-746){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub trait Validable {
    // Provided method
    fn validate(&self) -> bool { ... }
}
```

Expand description

::: docblock
This trait defines a way to validate a strategy.

The default implementation panics with a message indicating that
validation is not applicable for the specific strategy. Implementors of
this trait should override the `validate` method to provide specific
validation logic.
:::

## Provided Methods[§](#provided-methods){.anchor} {#provided-methods .section-header}

::::: methods
::: {#method.validate .section .method}
[Source](../../../src/optionstratlib/strategies/base.rs.html#743-745){.src
.rightside}

#### fn [validate](#method.validate){.fn}(&self) -\> [bool](https://doc.rust-lang.org/1.86.0/std/primitive.bool.html){.primitive} {#fn-validateself---bool .code-header}
:::

::: docblock
Validates the strategy.

The default implementation panics, indicating that validation is not
applicable. Implementors should override this method to provide
appropriate validation logic.

Returns `true` if the strategy is valid, and `false` otherwise.
:::
:::::

## Implementors[§](#implementors){.anchor} {#implementors .section-header}

:::::::::::::::::: {#implementors-list}
::: {#impl-Validable-for-BearCallSpread .section .impl}
[Source](../../../src/optionstratlib/strategies/bear_call_spread.rs.html#508-524){.src
.rightside}[§](#impl-Validable-for-BearCallSpread){.anchor}

### impl [Validable](trait.Validable.html "trait optionstratlib::strategies::base::Validable"){.trait} for [BearCallSpread](../bear_call_spread/struct.BearCallSpread.html "struct optionstratlib::strategies::bear_call_spread::BearCallSpread"){.struct} {#impl-validable-for-bearcallspread .code-header}
:::

::: {#impl-Validable-for-BearPutSpread .section .impl}
[Source](../../../src/optionstratlib/strategies/bear_put_spread.rs.html#486-505){.src
.rightside}[§](#impl-Validable-for-BearPutSpread){.anchor}

### impl [Validable](trait.Validable.html "trait optionstratlib::strategies::base::Validable"){.trait} for [BearPutSpread](../bear_put_spread/struct.BearPutSpread.html "struct optionstratlib::strategies::bear_put_spread::BearPutSpread"){.struct} {#impl-validable-for-bearputspread .code-header}
:::

::: {#impl-Validable-for-BullCallSpread .section .impl}
[Source](../../../src/optionstratlib/strategies/bull_call_spread.rs.html#495-515){.src
.rightside}[§](#impl-Validable-for-BullCallSpread){.anchor}

### impl [Validable](trait.Validable.html "trait optionstratlib::strategies::base::Validable"){.trait} for [BullCallSpread](../bull_call_spread/struct.BullCallSpread.html "struct optionstratlib::strategies::bull_call_spread::BullCallSpread"){.struct} {#impl-validable-for-bullcallspread .code-header}
:::

::: {#impl-Validable-for-BullPutSpread .section .impl}
[Source](../../../src/optionstratlib/strategies/bull_put_spread.rs.html#506-522){.src
.rightside}[§](#impl-Validable-for-BullPutSpread){.anchor}

### impl [Validable](trait.Validable.html "trait optionstratlib::strategies::base::Validable"){.trait} for [BullPutSpread](../bull_put_spread/struct.BullPutSpread.html "struct optionstratlib::strategies::bull_put_spread::BullPutSpread"){.struct} {#impl-validable-for-bullputspread .code-header}
:::

::: {#impl-Validable-for-LongButterflySpread .section .impl}
[Source](../../../src/optionstratlib/strategies/butterfly_spread.rs.html#358-393){.src
.rightside}[§](#impl-Validable-for-LongButterflySpread){.anchor}

### impl [Validable](trait.Validable.html "trait optionstratlib::strategies::base::Validable"){.trait} for [LongButterflySpread](../butterfly_spread/struct.LongButterflySpread.html "struct optionstratlib::strategies::butterfly_spread::LongButterflySpread"){.struct} {#impl-validable-for-longbutterflyspread .code-header}
:::

::: {#impl-Validable-for-ShortButterflySpread .section .impl}
[Source](../../../src/optionstratlib/strategies/butterfly_spread.rs.html#1334-1369){.src
.rightside}[§](#impl-Validable-for-ShortButterflySpread){.anchor}

### impl [Validable](trait.Validable.html "trait optionstratlib::strategies::base::Validable"){.trait} for [ShortButterflySpread](../butterfly_spread/struct.ShortButterflySpread.html "struct optionstratlib::strategies::butterfly_spread::ShortButterflySpread"){.struct} {#impl-validable-for-shortbutterflyspread .code-header}
:::

::: {#impl-Validable-for-CallButterfly .section .impl}
[Source](../../../src/optionstratlib/strategies/call_butterfly.rs.html#581-606){.src
.rightside}[§](#impl-Validable-for-CallButterfly){.anchor}

### impl [Validable](trait.Validable.html "trait optionstratlib::strategies::base::Validable"){.trait} for [CallButterfly](../call_butterfly/struct.CallButterfly.html "struct optionstratlib::strategies::call_butterfly::CallButterfly"){.struct} {#impl-validable-for-callbutterfly .code-header}
:::

::: {#impl-Validable-for-CustomStrategy .section .impl}
[Source](../../../src/optionstratlib/strategies/custom.rs.html#494-516){.src
.rightside}[§](#impl-Validable-for-CustomStrategy){.anchor}

### impl [Validable](trait.Validable.html "trait optionstratlib::strategies::base::Validable"){.trait} for [CustomStrategy](../custom/struct.CustomStrategy.html "struct optionstratlib::strategies::custom::CustomStrategy"){.struct} {#impl-validable-for-customstrategy .code-header}
:::

::: {#impl-Validable-for-IronButterfly .section .impl}
[Source](../../../src/optionstratlib/strategies/iron_butterfly.rs.html#444-460){.src
.rightside}[§](#impl-Validable-for-IronButterfly){.anchor}

### impl [Validable](trait.Validable.html "trait optionstratlib::strategies::base::Validable"){.trait} for [IronButterfly](../iron_butterfly/struct.IronButterfly.html "struct optionstratlib::strategies::iron_butterfly::IronButterfly"){.struct} {#impl-validable-for-ironbutterfly .code-header}
:::

::: {#impl-Validable-for-IronCondor .section .impl}
[Source](../../../src/optionstratlib/strategies/iron_condor.rs.html#451-467){.src
.rightside}[§](#impl-Validable-for-IronCondor){.anchor}

### impl [Validable](trait.Validable.html "trait optionstratlib::strategies::base::Validable"){.trait} for [IronCondor](../iron_condor/struct.IronCondor.html "struct optionstratlib::strategies::iron_condor::IronCondor"){.struct} {#impl-validable-for-ironcondor .code-header}
:::

::: {#impl-Validable-for-PoorMansCoveredCall .section .impl}
[Source](../../../src/optionstratlib/strategies/poor_mans_covered_call.rs.html#352-356){.src
.rightside}[§](#impl-Validable-for-PoorMansCoveredCall){.anchor}

### impl [Validable](trait.Validable.html "trait optionstratlib::strategies::base::Validable"){.trait} for [PoorMansCoveredCall](../poor_mans_covered_call/struct.PoorMansCoveredCall.html "struct optionstratlib::strategies::poor_mans_covered_call::PoorMansCoveredCall"){.struct} {#impl-validable-for-poormanscoveredcall .code-header}
:::

::: {#impl-Validable-for-LongStraddle .section .impl}
[Source](../../../src/optionstratlib/strategies/straddle.rs.html#1321-1327){.src
.rightside}[§](#impl-Validable-for-LongStraddle){.anchor}

### impl [Validable](trait.Validable.html "trait optionstratlib::strategies::base::Validable"){.trait} for [LongStraddle](../straddle/struct.LongStraddle.html "struct optionstratlib::strategies::straddle::LongStraddle"){.struct} {#impl-validable-for-longstraddle .code-header}
:::

::: {#impl-Validable-for-ShortStraddle .section .impl}
[Source](../../../src/optionstratlib/strategies/straddle.rs.html#521-527){.src
.rightside}[§](#impl-Validable-for-ShortStraddle){.anchor}

### impl [Validable](trait.Validable.html "trait optionstratlib::strategies::base::Validable"){.trait} for [ShortStraddle](../straddle/struct.ShortStraddle.html "struct optionstratlib::strategies::straddle::ShortStraddle"){.struct} {#impl-validable-for-shortstraddle .code-header}
:::

::: {#impl-Validable-for-LongStrangle .section .impl}
[Source](../../../src/optionstratlib/strategies/strangle.rs.html#1572-1578){.src
.rightside}[§](#impl-Validable-for-LongStrangle){.anchor}

### impl [Validable](trait.Validable.html "trait optionstratlib::strategies::base::Validable"){.trait} for [LongStrangle](../strangle/struct.LongStrangle.html "struct optionstratlib::strategies::strangle::LongStrangle"){.struct} {#impl-validable-for-longstrangle .code-header}
:::

::: {#impl-Validable-for-ShortStrangle .section .impl}
[Source](../../../src/optionstratlib/strategies/strangle.rs.html#567-573){.src
.rightside}[§](#impl-Validable-for-ShortStrangle){.anchor}

### impl [Validable](trait.Validable.html "trait optionstratlib::strategies::base::Validable"){.trait} for [ShortStrangle](../strangle/struct.ShortStrangle.html "struct optionstratlib::strategies::strangle::ShortStrangle"){.struct} {#impl-validable-for-shortstrangle .code-header}
:::
::::::::::::::::::
:::::::::::::::::::::::::
::::::::::::::::::::::::::
