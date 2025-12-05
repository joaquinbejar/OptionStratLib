:::::::::::::::::::::::::::::: width-limiter
::::::::::::::::::::::::::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../../index.html)::[strategies](../index.html)::[base](index.html)
:::

# Trait [Validable]{.trait} Copy item path

[[Source](../../../src/optionstratlib/strategies/base.rs.html#1143-1154){.src}
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
[Source](../../../src/optionstratlib/strategies/base.rs.html#1151-1153){.src
.rightside}

#### fn [validate](#method.validate){.fn}(&self) -\> [bool](https://doc.rust-lang.org/1.91.1/std/primitive.bool.html){.primitive} {#fn-validateself---bool .code-header}
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

:::::::::::::::::::::: {#implementors-list}
::: {#impl-Validable-for-BearCallSpread .section .impl}
[Source](../../../src/optionstratlib/strategies/bear_call_spread.rs.html#625-641){.src
.rightside}[§](#impl-Validable-for-BearCallSpread){.anchor}

### impl [Validable](trait.Validable.html "trait optionstratlib::strategies::base::Validable"){.trait} for [BearCallSpread](../bear_call_spread/struct.BearCallSpread.html "struct optionstratlib::strategies::bear_call_spread::BearCallSpread"){.struct} {#impl-validable-for-bearcallspread .code-header}
:::

::: {#impl-Validable-for-BearPutSpread .section .impl}
[Source](../../../src/optionstratlib/strategies/bear_put_spread.rs.html#613-632){.src
.rightside}[§](#impl-Validable-for-BearPutSpread){.anchor}

### impl [Validable](trait.Validable.html "trait optionstratlib::strategies::base::Validable"){.trait} for [BearPutSpread](../bear_put_spread/struct.BearPutSpread.html "struct optionstratlib::strategies::bear_put_spread::BearPutSpread"){.struct} {#impl-validable-for-bearputspread .code-header}
:::

::: {#impl-Validable-for-BullCallSpread .section .impl}
[Source](../../../src/optionstratlib/strategies/bull_call_spread.rs.html#625-645){.src
.rightside}[§](#impl-Validable-for-BullCallSpread){.anchor}

### impl [Validable](trait.Validable.html "trait optionstratlib::strategies::base::Validable"){.trait} for [BullCallSpread](../bull_call_spread/struct.BullCallSpread.html "struct optionstratlib::strategies::bull_call_spread::BullCallSpread"){.struct} {#impl-validable-for-bullcallspread .code-header}
:::

::: {#impl-Validable-for-BullPutSpread .section .impl}
[Source](../../../src/optionstratlib/strategies/bull_put_spread.rs.html#629-645){.src
.rightside}[§](#impl-Validable-for-BullPutSpread){.anchor}

### impl [Validable](trait.Validable.html "trait optionstratlib::strategies::base::Validable"){.trait} for [BullPutSpread](../bull_put_spread/struct.BullPutSpread.html "struct optionstratlib::strategies::bull_put_spread::BullPutSpread"){.struct} {#impl-validable-for-bullputspread .code-header}
:::

::: {#impl-Validable-for-CallButterfly .section .impl}
[Source](../../../src/optionstratlib/strategies/call_butterfly.rs.html#729-754){.src
.rightside}[§](#impl-Validable-for-CallButterfly){.anchor}

### impl [Validable](trait.Validable.html "trait optionstratlib::strategies::base::Validable"){.trait} for [CallButterfly](../call_butterfly/struct.CallButterfly.html "struct optionstratlib::strategies::call_butterfly::CallButterfly"){.struct} {#impl-validable-for-callbutterfly .code-header}
:::

::: {#impl-Validable-for-CustomStrategy .section .impl}
[Source](../../../src/optionstratlib/strategies/custom.rs.html#742-765){.src
.rightside}[§](#impl-Validable-for-CustomStrategy){.anchor}

### impl [Validable](trait.Validable.html "trait optionstratlib::strategies::base::Validable"){.trait} for [CustomStrategy](../custom/struct.CustomStrategy.html "struct optionstratlib::strategies::custom::CustomStrategy"){.struct} {#impl-validable-for-customstrategy .code-header}
:::

::: {#impl-Validable-for-IronButterfly .section .impl}
[Source](../../../src/optionstratlib/strategies/iron_butterfly.rs.html#459-475){.src
.rightside}[§](#impl-Validable-for-IronButterfly){.anchor}

### impl [Validable](trait.Validable.html "trait optionstratlib::strategies::base::Validable"){.trait} for [IronButterfly](../iron_butterfly/struct.IronButterfly.html "struct optionstratlib::strategies::iron_butterfly::IronButterfly"){.struct} {#impl-validable-for-ironbutterfly .code-header}
:::

::: {#impl-Validable-for-IronCondor .section .impl}
[Source](../../../src/optionstratlib/strategies/iron_condor.rs.html#474-490){.src
.rightside}[§](#impl-Validable-for-IronCondor){.anchor}

### impl [Validable](trait.Validable.html "trait optionstratlib::strategies::base::Validable"){.trait} for [IronCondor](../iron_condor/struct.IronCondor.html "struct optionstratlib::strategies::iron_condor::IronCondor"){.struct} {#impl-validable-for-ironcondor .code-header}
:::

::: {#impl-Validable-for-LongButterflySpread .section .impl}
[Source](../../../src/optionstratlib/strategies/long_butterfly_spread.rs.html#378-413){.src
.rightside}[§](#impl-Validable-for-LongButterflySpread){.anchor}

### impl [Validable](trait.Validable.html "trait optionstratlib::strategies::base::Validable"){.trait} for [LongButterflySpread](../long_butterfly_spread/struct.LongButterflySpread.html "struct optionstratlib::strategies::long_butterfly_spread::LongButterflySpread"){.struct} {#impl-validable-for-longbutterflyspread .code-header}
:::

::: {#impl-Validable-for-LongCall .section .impl}
[Source](../../../src/optionstratlib/strategies/long_call.rs.html#246-254){.src
.rightside}[§](#impl-Validable-for-LongCall){.anchor}

### impl [Validable](trait.Validable.html "trait optionstratlib::strategies::base::Validable"){.trait} for [LongCall](../long_call/struct.LongCall.html "struct optionstratlib::strategies::long_call::LongCall"){.struct} {#impl-validable-for-longcall .code-header}
:::

::: {#impl-Validable-for-LongPut .section .impl}
[Source](../../../src/optionstratlib/strategies/long_put.rs.html#243-251){.src
.rightside}[§](#impl-Validable-for-LongPut){.anchor}

### impl [Validable](trait.Validable.html "trait optionstratlib::strategies::base::Validable"){.trait} for [LongPut](../long_put/struct.LongPut.html "struct optionstratlib::strategies::long_put::LongPut"){.struct} {#impl-validable-for-longput .code-header}
:::

::: {#impl-Validable-for-LongStraddle .section .impl}
[Source](../../../src/optionstratlib/strategies/long_straddle.rs.html#625-631){.src
.rightside}[§](#impl-Validable-for-LongStraddle){.anchor}

### impl [Validable](trait.Validable.html "trait optionstratlib::strategies::base::Validable"){.trait} for [LongStraddle](../long_straddle/struct.LongStraddle.html "struct optionstratlib::strategies::long_straddle::LongStraddle"){.struct} {#impl-validable-for-longstraddle .code-header}
:::

::: {#impl-Validable-for-LongStrangle .section .impl}
[Source](../../../src/optionstratlib/strategies/long_strangle.rs.html#667-673){.src
.rightside}[§](#impl-Validable-for-LongStrangle){.anchor}

### impl [Validable](trait.Validable.html "trait optionstratlib::strategies::base::Validable"){.trait} for [LongStrangle](../long_strangle/struct.LongStrangle.html "struct optionstratlib::strategies::long_strangle::LongStrangle"){.struct} {#impl-validable-for-longstrangle .code-header}
:::

::: {#impl-Validable-for-PoorMansCoveredCall .section .impl}
[Source](../../../src/optionstratlib/strategies/poor_mans_covered_call.rs.html#362-366){.src
.rightside}[§](#impl-Validable-for-PoorMansCoveredCall){.anchor}

### impl [Validable](trait.Validable.html "trait optionstratlib::strategies::base::Validable"){.trait} for [PoorMansCoveredCall](../poor_mans_covered_call/struct.PoorMansCoveredCall.html "struct optionstratlib::strategies::poor_mans_covered_call::PoorMansCoveredCall"){.struct} {#impl-validable-for-poormanscoveredcall .code-header}
:::

::: {#impl-Validable-for-ShortButterflySpread .section .impl}
[Source](../../../src/optionstratlib/strategies/short_butterfly_spread.rs.html#371-406){.src
.rightside}[§](#impl-Validable-for-ShortButterflySpread){.anchor}

### impl [Validable](trait.Validable.html "trait optionstratlib::strategies::base::Validable"){.trait} for [ShortButterflySpread](../short_butterfly_spread/struct.ShortButterflySpread.html "struct optionstratlib::strategies::short_butterfly_spread::ShortButterflySpread"){.struct} {#impl-validable-for-shortbutterflyspread .code-header}
:::

::: {#impl-Validable-for-ShortCall .section .impl}
[Source](../../../src/optionstratlib/strategies/short_call.rs.html#254-262){.src
.rightside}[§](#impl-Validable-for-ShortCall){.anchor}

### impl [Validable](trait.Validable.html "trait optionstratlib::strategies::base::Validable"){.trait} for [ShortCall](../short_call/struct.ShortCall.html "struct optionstratlib::strategies::short_call::ShortCall"){.struct} {#impl-validable-for-shortcall .code-header}
:::

::: {#impl-Validable-for-ShortPut .section .impl}
[Source](../../../src/optionstratlib/strategies/short_put.rs.html#248-256){.src
.rightside}[§](#impl-Validable-for-ShortPut){.anchor}

### impl [Validable](trait.Validable.html "trait optionstratlib::strategies::base::Validable"){.trait} for [ShortPut](../short_put/struct.ShortPut.html "struct optionstratlib::strategies::short_put::ShortPut"){.struct} {#impl-validable-for-shortput .code-header}
:::

::: {#impl-Validable-for-ShortStraddle .section .impl}
[Source](../../../src/optionstratlib/strategies/short_straddle.rs.html#650-656){.src
.rightside}[§](#impl-Validable-for-ShortStraddle){.anchor}

### impl [Validable](trait.Validable.html "trait optionstratlib::strategies::base::Validable"){.trait} for [ShortStraddle](../short_straddle/struct.ShortStraddle.html "struct optionstratlib::strategies::short_straddle::ShortStraddle"){.struct} {#impl-validable-for-shortstraddle .code-header}
:::

::: {#impl-Validable-for-ShortStrangle .section .impl}
[Source](../../../src/optionstratlib/strategies/short_strangle.rs.html#853-859){.src
.rightside}[§](#impl-Validable-for-ShortStrangle){.anchor}

### impl [Validable](trait.Validable.html "trait optionstratlib::strategies::base::Validable"){.trait} for [ShortStrangle](../short_strangle/struct.ShortStrangle.html "struct optionstratlib::strategies::short_strangle::ShortStrangle"){.struct} {#impl-validable-for-shortstrangle .code-header}
:::
::::::::::::::::::::::
:::::::::::::::::::::::::::::
::::::::::::::::::::::::::::::
