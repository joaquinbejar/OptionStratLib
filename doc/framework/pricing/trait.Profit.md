::::::::::::::::::::::::::::::::::::::::: width-limiter
:::::::::::::::::::::::::::::::::::::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../index.html)::[pricing](index.html)
:::

# Trait [Profit]{.trait} Copy item path

[[Source](../../src/optionstratlib/pricing/payoff.rs.html#193-226){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub trait Profit {
    // Required method
    fn calculate_profit_at(
        &self,
        price: &Positive,
    ) -> Result<Decimal, PricingError>;

    // Provided method
    fn get_point_at_price(
        &self,
        _price: &Positive,
    ) -> Result<(Decimal, Decimal), PricingError> { ... }
}
```

Expand description

::: docblock
Defines the profit calculation behavior for financial instruments.

This trait is used to calculate and visualize profit values at different
price points for various financial instruments and strategies. It
provides:

1.  A required method to calculate the actual profit value at a given
    price
2.  A default implementation to convert the profit calculation into a
    visualization point

## [§](#usage){.doc-anchor}Usage

Implement this trait for any type that can calculate profit at a
specific price point, such as options contracts, spreads, or complex
trading strategies.
:::

## Required Methods[§](#required-methods){.anchor} {#required-methods .section-header}

::::: methods
::: {#tymethod.calculate_profit_at .section .method}
[Source](../../src/optionstratlib/pricing/payoff.rs.html#204){.src
.rightside}

#### fn [calculate_profit_at](#tymethod.calculate_profit_at){.fn}(&self, price: &[Positive](../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Decimal](../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}, [PricingError](../error/pricing/enum.PricingError.html "enum optionstratlib::error::pricing::PricingError"){.enum}\> {#fn-calculate_profit_atself-price-positive---resultdecimal-pricingerror .code-header}
:::

::: docblock
Calculates the profit at a specified price.

##### [§](#parameters){.doc-anchor}Parameters

- `price` - A positive value representing the price at which to
  calculate profit

##### [§](#returns){.doc-anchor}Returns

- `Result<Decimal, PricingError>` - The calculated profit as a Decimal
  value, or an error if the calculation fails
:::
:::::

## Provided Methods[§](#provided-methods){.anchor} {#provided-methods .section-header}

::::: methods
::: {#method.get_point_at_price .section .method}
[Source](../../src/optionstratlib/pricing/payoff.rs.html#219-225){.src
.rightside}

#### fn [get_point_at_price](#method.get_point_at_price){.fn}( &self, \_price: &[Positive](../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, ) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<([Decimal](../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}, [Decimal](../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}), [PricingError](../error/pricing/enum.PricingError.html "enum optionstratlib::error::pricing::PricingError"){.enum}\> {#fn-get_point_at_price-self-_price-positive---resultdecimal-decimal-pricingerror .code-header}
:::

::: docblock
Creates a chart point representation of the profit at the given price.

This method automatically determines the appropriate visualization
properties based on the profit value, such as color (green for positive
profit, red for negative).

##### [§](#parameters-1){.doc-anchor}Parameters

- `price` - A positive value representing the price for which to create
  a chart point

##### [§](#returns-1){.doc-anchor}Returns

- `ChartPoint<(f64, f64)>` - A formatted chart point with coordinates
  (price, profit), styling, and a formatted profit label
:::
:::::

## Implementors[§](#implementors){.anchor} {#implementors .section-header}

:::::::::::::::::::::::::::::: {#implementors-list}
::: {#impl-Profit-for-Options .section .impl}
[Source](../../src/optionstratlib/model/option.rs.html#779-783){.src
.rightside}[§](#impl-Profit-for-Options){.anchor}

### impl [Profit](trait.Profit.html "trait optionstratlib::pricing::Profit"){.trait} for [Options](../model/option/struct.Options.html "struct optionstratlib::model::option::Options"){.struct} {#impl-profit-for-options .code-header}
:::

:::: {#impl-Profit-for-Position .section .impl}
[Source](../../src/optionstratlib/model/position.rs.html#978-995){.src
.rightside}[§](#impl-Profit-for-Position){.anchor}

### impl [Profit](trait.Profit.html "trait optionstratlib::pricing::Profit"){.trait} for [Position](../model/position/struct.Position.html "struct optionstratlib::model::position::Position"){.struct} {#impl-profit-for-position .code-header}

::: docblock
Implementation of the Profit trait for the Position struct.
:::
::::

::: docblock
This allows calculating the profit of a position at a given price by
using the position's profit and loss (PnL) calculation at expiration.
:::

::: {#impl-Profit-for-BearCallSpread .section .impl}
[Source](../../src/optionstratlib/strategies/bear_call_spread.rs.html#749-757){.src
.rightside}[§](#impl-Profit-for-BearCallSpread){.anchor}

### impl [Profit](trait.Profit.html "trait optionstratlib::pricing::Profit"){.trait} for [BearCallSpread](../strategies/bear_call_spread/struct.BearCallSpread.html "struct optionstratlib::strategies::bear_call_spread::BearCallSpread"){.struct} {#impl-profit-for-bearcallspread .code-header}
:::

::: {#impl-Profit-for-BearPutSpread .section .impl}
[Source](../../src/optionstratlib/strategies/bear_put_spread.rs.html#740-745){.src
.rightside}[§](#impl-Profit-for-BearPutSpread){.anchor}

### impl [Profit](trait.Profit.html "trait optionstratlib::pricing::Profit"){.trait} for [BearPutSpread](../strategies/bear_put_spread/struct.BearPutSpread.html "struct optionstratlib::strategies::bear_put_spread::BearPutSpread"){.struct} {#impl-profit-for-bearputspread .code-header}
:::

::: {#impl-Profit-for-BullCallSpread .section .impl}
[Source](../../src/optionstratlib/strategies/bull_call_spread.rs.html#753-761){.src
.rightside}[§](#impl-Profit-for-BullCallSpread){.anchor}

### impl [Profit](trait.Profit.html "trait optionstratlib::pricing::Profit"){.trait} for [BullCallSpread](../strategies/bull_call_spread/struct.BullCallSpread.html "struct optionstratlib::strategies::bull_call_spread::BullCallSpread"){.struct} {#impl-profit-for-bullcallspread .code-header}
:::

::: {#impl-Profit-for-BullPutSpread .section .impl}
[Source](../../src/optionstratlib/strategies/bull_put_spread.rs.html#850-855){.src
.rightside}[§](#impl-Profit-for-BullPutSpread){.anchor}

### impl [Profit](trait.Profit.html "trait optionstratlib::pricing::Profit"){.trait} for [BullPutSpread](../strategies/bull_put_spread/struct.BullPutSpread.html "struct optionstratlib::strategies::bull_put_spread::BullPutSpread"){.struct} {#impl-profit-for-bullputspread .code-header}
:::

::: {#impl-Profit-for-CallButterfly .section .impl}
[Source](../../src/optionstratlib/strategies/call_butterfly.rs.html#883-891){.src
.rightside}[§](#impl-Profit-for-CallButterfly){.anchor}

### impl [Profit](trait.Profit.html "trait optionstratlib::pricing::Profit"){.trait} for [CallButterfly](../strategies/call_butterfly/struct.CallButterfly.html "struct optionstratlib::strategies::call_butterfly::CallButterfly"){.struct} {#impl-profit-for-callbutterfly .code-header}
:::

::: {#impl-Profit-for-CustomStrategy .section .impl}
[Source](../../src/optionstratlib/strategies/custom.rs.html#826-834){.src
.rightside}[§](#impl-Profit-for-CustomStrategy){.anchor}

### impl [Profit](trait.Profit.html "trait optionstratlib::pricing::Profit"){.trait} for [CustomStrategy](../strategies/custom/struct.CustomStrategy.html "struct optionstratlib::strategies::custom::CustomStrategy"){.struct} {#impl-profit-for-customstrategy .code-header}
:::

::: {#impl-Profit-for-IronButterfly .section .impl}
[Source](../../src/optionstratlib/strategies/iron_butterfly.rs.html#954-962){.src
.rightside}[§](#impl-Profit-for-IronButterfly){.anchor}

### impl [Profit](trait.Profit.html "trait optionstratlib::pricing::Profit"){.trait} for [IronButterfly](../strategies/iron_butterfly/struct.IronButterfly.html "struct optionstratlib::strategies::iron_butterfly::IronButterfly"){.struct} {#impl-profit-for-ironbutterfly .code-header}
:::

::: {#impl-Profit-for-IronCondor .section .impl}
[Source](../../src/optionstratlib/strategies/iron_condor.rs.html#982-990){.src
.rightside}[§](#impl-Profit-for-IronCondor){.anchor}

### impl [Profit](trait.Profit.html "trait optionstratlib::pricing::Profit"){.trait} for [IronCondor](../strategies/iron_condor/struct.IronCondor.html "struct optionstratlib::strategies::iron_condor::IronCondor"){.struct} {#impl-profit-for-ironcondor .code-header}
:::

::: {#impl-Profit-for-LongButterflySpread .section .impl}
[Source](../../src/optionstratlib/strategies/long_butterfly_spread.rs.html#915-922){.src
.rightside}[§](#impl-Profit-for-LongButterflySpread){.anchor}

### impl [Profit](trait.Profit.html "trait optionstratlib::pricing::Profit"){.trait} for [LongButterflySpread](../strategies/long_butterfly_spread/struct.LongButterflySpread.html "struct optionstratlib::strategies::long_butterfly_spread::LongButterflySpread"){.struct} {#impl-profit-for-longbutterflyspread .code-header}
:::

::: {#impl-Profit-for-LongCall .section .impl}
[Source](../../src/optionstratlib/strategies/long_call.rs.html#315-320){.src
.rightside}[§](#impl-Profit-for-LongCall){.anchor}

### impl [Profit](trait.Profit.html "trait optionstratlib::pricing::Profit"){.trait} for [LongCall](../strategies/long_call/struct.LongCall.html "struct optionstratlib::strategies::long_call::LongCall"){.struct} {#impl-profit-for-longcall .code-header}
:::

::: {#impl-Profit-for-LongPut .section .impl}
[Source](../../src/optionstratlib/strategies/long_put.rs.html#312-317){.src
.rightside}[§](#impl-Profit-for-LongPut){.anchor}

### impl [Profit](trait.Profit.html "trait optionstratlib::pricing::Profit"){.trait} for [LongPut](../strategies/long_put/struct.LongPut.html "struct optionstratlib::strategies::long_put::LongPut"){.struct} {#impl-profit-for-longput .code-header}
:::

::: {#impl-Profit-for-LongStraddle .section .impl}
[Source](../../src/optionstratlib/strategies/long_straddle.rs.html#742-747){.src
.rightside}[§](#impl-Profit-for-LongStraddle){.anchor}

### impl [Profit](trait.Profit.html "trait optionstratlib::pricing::Profit"){.trait} for [LongStraddle](../strategies/long_straddle/struct.LongStraddle.html "struct optionstratlib::strategies::long_straddle::LongStraddle"){.struct} {#impl-profit-for-longstraddle .code-header}
:::

::: {#impl-Profit-for-LongStrangle .section .impl}
[Source](../../src/optionstratlib/strategies/long_strangle.rs.html#801-806){.src
.rightside}[§](#impl-Profit-for-LongStrangle){.anchor}

### impl [Profit](trait.Profit.html "trait optionstratlib::pricing::Profit"){.trait} for [LongStrangle](../strategies/long_strangle/struct.LongStrangle.html "struct optionstratlib::strategies::long_strangle::LongStrangle"){.struct} {#impl-profit-for-longstrangle .code-header}
:::

::: {#impl-Profit-for-PoorMansCoveredCall .section .impl}
[Source](../../src/optionstratlib/strategies/poor_mans_covered_call.rs.html#751-759){.src
.rightside}[§](#impl-Profit-for-PoorMansCoveredCall){.anchor}

### impl [Profit](trait.Profit.html "trait optionstratlib::pricing::Profit"){.trait} for [PoorMansCoveredCall](../strategies/poor_mans_covered_call/struct.PoorMansCoveredCall.html "struct optionstratlib::strategies::poor_mans_covered_call::PoorMansCoveredCall"){.struct} {#impl-profit-for-poormanscoveredcall .code-header}
:::

::: {#impl-Profit-for-ShortButterflySpread .section .impl}
[Source](../../src/optionstratlib/strategies/short_butterfly_spread.rs.html#893-900){.src
.rightside}[§](#impl-Profit-for-ShortButterflySpread){.anchor}

### impl [Profit](trait.Profit.html "trait optionstratlib::pricing::Profit"){.trait} for [ShortButterflySpread](../strategies/short_butterfly_spread/struct.ShortButterflySpread.html "struct optionstratlib::strategies::short_butterfly_spread::ShortButterflySpread"){.struct} {#impl-profit-for-shortbutterflyspread .code-header}
:::

::: {#impl-Profit-for-ShortCall .section .impl}
[Source](../../src/optionstratlib/strategies/short_call.rs.html#323-328){.src
.rightside}[§](#impl-Profit-for-ShortCall){.anchor}

### impl [Profit](trait.Profit.html "trait optionstratlib::pricing::Profit"){.trait} for [ShortCall](../strategies/short_call/struct.ShortCall.html "struct optionstratlib::strategies::short_call::ShortCall"){.struct} {#impl-profit-for-shortcall .code-header}
:::

::: {#impl-Profit-for-ShortPut .section .impl}
[Source](../../src/optionstratlib/strategies/short_put.rs.html#317-322){.src
.rightside}[§](#impl-Profit-for-ShortPut){.anchor}

### impl [Profit](trait.Profit.html "trait optionstratlib::pricing::Profit"){.trait} for [ShortPut](../strategies/short_put/struct.ShortPut.html "struct optionstratlib::strategies::short_put::ShortPut"){.struct} {#impl-profit-for-shortput .code-header}
:::

::: {#impl-Profit-for-ShortStraddle .section .impl}
[Source](../../src/optionstratlib/strategies/short_straddle.rs.html#776-794){.src
.rightside}[§](#impl-Profit-for-ShortStraddle){.anchor}

### impl [Profit](trait.Profit.html "trait optionstratlib::pricing::Profit"){.trait} for [ShortStraddle](../strategies/short_straddle/struct.ShortStraddle.html "struct optionstratlib::strategies::short_straddle::ShortStraddle"){.struct} {#impl-profit-for-shortstraddle .code-header}
:::

::: {#impl-Profit-for-ShortStrangle .section .impl}
[Source](../../src/optionstratlib/strategies/short_strangle.rs.html#1039-1053){.src
.rightside}[§](#impl-Profit-for-ShortStrangle){.anchor}

### impl [Profit](trait.Profit.html "trait optionstratlib::pricing::Profit"){.trait} for [ShortStrangle](../strategies/short_strangle/struct.ShortStrangle.html "struct optionstratlib::strategies::short_strangle::ShortStrangle"){.struct} {#impl-profit-for-shortstrangle .code-header}
:::

:::: {#impl-Profit-for-RandomWalk%3CX,+Y%3E .section .impl}
[Source](../../src/optionstratlib/simulation/randomwalk.rs.html#277-287){.src
.rightside}[§](#impl-Profit-for-RandomWalk%3CX,+Y%3E){.anchor}

### impl\<X, Y\> [Profit](trait.Profit.html "trait optionstratlib::pricing::Profit"){.trait} for [RandomWalk](../simulation/randomwalk/struct.RandomWalk.html "struct optionstratlib::simulation::randomwalk::RandomWalk"){.struct}\<X, Y\> {#implx-y-profit-for-randomwalkx-y .code-header}

::: where
where X:
[AddAssign](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.AddAssign.html "trait core::ops::arith::AddAssign"){.trait} +
[Copy](https://doc.rust-lang.org/1.91.1/core/marker/trait.Copy.html "trait core::marker::Copy"){.trait} +
[Display](https://doc.rust-lang.org/1.91.1/core/fmt/trait.Display.html "trait core::fmt::Display"){.trait} +
[Into](https://doc.rust-lang.org/1.91.1/core/convert/trait.Into.html "trait core::convert::Into"){.trait}\<[Positive](../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\>,
Y:
[Into](https://doc.rust-lang.org/1.91.1/core/convert/trait.Into.html "trait core::convert::Into"){.trait}\<[Positive](../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\> +
[Display](https://doc.rust-lang.org/1.91.1/core/fmt/trait.Display.html "trait core::fmt::Display"){.trait} +
[Clone](https://doc.rust-lang.org/1.91.1/core/clone/trait.Clone.html "trait core::clone::Clone"){.trait},
:::
::::

:::: {#impl-Profit-for-Simulator%3CX,+Y%3E .section .impl}
[Source](../../src/optionstratlib/simulation/simulator.rs.html#376-386){.src
.rightside}[§](#impl-Profit-for-Simulator%3CX,+Y%3E){.anchor}

### impl\<X, Y\> [Profit](trait.Profit.html "trait optionstratlib::pricing::Profit"){.trait} for [Simulator](../simulation/simulator/struct.Simulator.html "struct optionstratlib::simulation::simulator::Simulator"){.struct}\<X, Y\> {#implx-y-profit-for-simulatorx-y .code-header}

::: where
where X:
[AddAssign](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.AddAssign.html "trait core::ops::arith::AddAssign"){.trait} +
[Copy](https://doc.rust-lang.org/1.91.1/core/marker/trait.Copy.html "trait core::marker::Copy"){.trait} +
[Display](https://doc.rust-lang.org/1.91.1/core/fmt/trait.Display.html "trait core::fmt::Display"){.trait} +
[Into](https://doc.rust-lang.org/1.91.1/core/convert/trait.Into.html "trait core::convert::Into"){.trait}\<[Positive](../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\>,
Y:
[Clone](https://doc.rust-lang.org/1.91.1/core/clone/trait.Clone.html "trait core::clone::Clone"){.trait} +
[Display](https://doc.rust-lang.org/1.91.1/core/fmt/trait.Display.html "trait core::fmt::Display"){.trait} +
[Into](https://doc.rust-lang.org/1.91.1/core/convert/trait.Into.html "trait core::convert::Into"){.trait}\<[Positive](../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\>,
:::
::::
::::::::::::::::::::::::::::::
::::::::::::::::::::::::::::::::::::::::
:::::::::::::::::::::::::::::::::::::::::
