::::::::::::::::::::::::::::::::::::::::::::::::::: width-limiter
:::::::::::::::::::::::::::::::::::::::::::::::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../index.html)::[greeks](index.html)
:::

# Trait [Greeks]{.trait} Copy item path

[[Source](../../src/optionstratlib/greeks/equations.rs.html#108-264){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub trait Greeks {
    // Required method
    fn get_options(&self) -> Result<Vec<&Options>, GreeksError>;

    // Provided methods
    fn greeks(&self) -> Result<Greek, GreeksError> { ... }
    fn delta(&self) -> Result<Decimal, GreeksError> { ... }
    fn gamma(&self) -> Result<Decimal, GreeksError> { ... }
    fn theta(&self) -> Result<Decimal, GreeksError> { ... }
    fn vega(&self) -> Result<Decimal, GreeksError> { ... }
    fn rho(&self) -> Result<Decimal, GreeksError> { ... }
    fn rho_d(&self) -> Result<Decimal, GreeksError> { ... }
    fn alpha(&self) -> Result<Decimal, GreeksError> { ... }
}
```

Expand description

::: docblock
Trait that provides option Greeks calculation functionality for
financial instruments.

The `Greeks` trait enables implementing types to calculate option
sensitivity metrics (Greeks) across multiple option positions. Any type
that can provide access to a collection of options can implement this
trait to gain the ability to calculate aggregate Greek values.

This trait uses a composition approach where implementation only
requires defining the `get_options()` method, while default
implementations for all Greek calculations are provided.

## [§](#greek-calculations){.doc-anchor}Greek Calculations

The trait provides calculations for:

- Delta: Sensitivity to changes in the underlying asset's price
- Gamma: Rate of change of delta (acceleration of price movement)
- Theta: Time decay of option value
- Vega: Sensitivity to changes in volatility
- Rho: Sensitivity to changes in interest rates
- Rho_d: Sensitivity to changes in dividend yield
- Alpha: Ratio between gamma and theta

## [§](#usage){.doc-anchor}Usage

Implementers only need to provide the `get_options()` method which
returns a vector of references to option contracts. The trait will
handle aggregating the Greek values across all options in the
collection.

## [§](#errors){.doc-anchor}Errors

Methods return `Result<T, GreeksError>` to handle various calculation
errors that may occur during Greek computations.
:::

## Required Methods[§](#required-methods){.anchor} {#required-methods .section-header}

::::: methods
::: {#tymethod.get_options .section .method}
[Source](../../src/optionstratlib/greeks/equations.rs.html#117){.src
.rightside}

#### fn [get_options](#tymethod.get_options){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Vec](https://doc.rust-lang.org/1.91.1/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<&[Options](../model/option/struct.Options.html "struct optionstratlib::model::option::Options"){.struct}\>, [GreeksError](../error/greeks/enum.GreeksError.html "enum optionstratlib::error::greeks::GreeksError"){.enum}\> {#fn-get_optionsself---resultvecoptions-greekserror .code-header}
:::

::: docblock
Returns a vector of references to the option contracts for which Greeks
will be calculated.

This is the only method that must be implemented by types adopting this
trait. All other methods have default implementations based on this
method.

##### [§](#errors-1){.doc-anchor}Errors

Returns a `GreeksError` if there is an issue retrieving the options.
:::
:::::

## Provided Methods[§](#provided-methods){.anchor} {#provided-methods .section-header}

::::::::::::::::::: methods
::: {#method.greeks .section .method}
[Source](../../src/optionstratlib/greeks/equations.rs.html#127-144){.src
.rightside}

#### fn [greeks](#method.greeks){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Greek](struct.Greek.html "struct optionstratlib::greeks::Greek"){.struct}, [GreeksError](../error/greeks/enum.GreeksError.html "enum optionstratlib::error::greeks::GreeksError"){.enum}\> {#fn-greeksself---resultgreek-greekserror .code-header}
:::

::: docblock
Calculates and returns all Greeks as a single `Greek` struct.

This method provides a convenient way to obtain all Greek values at
once. It calls each individual Greek calculation method and compiles the
results.

##### [§](#errors-2){.doc-anchor}Errors

Returns a `GreeksError` if any individual Greek calculation fails.
:::

::: {#method.delta .section .method}
[Source](../../src/optionstratlib/greeks/equations.rs.html#154-161){.src
.rightside}

#### fn [delta](#method.delta){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Decimal](../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}, [GreeksError](../error/greeks/enum.GreeksError.html "enum optionstratlib::error::greeks::GreeksError"){.enum}\> {#fn-deltaself---resultdecimal-greekserror .code-header}
:::

::: docblock
Calculates the aggregate delta value for all options.

Delta measures the rate of change in an option's price with respect to
changes in the underlying asset's price.

##### [§](#errors-3){.doc-anchor}Errors

Returns a `GreeksError` if the options can't be retrieved or delta
calculation fails.
:::

::: {#method.gamma .section .method}
[Source](../../src/optionstratlib/greeks/equations.rs.html#171-178){.src
.rightside}

#### fn [gamma](#method.gamma){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Decimal](../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}, [GreeksError](../error/greeks/enum.GreeksError.html "enum optionstratlib::error::greeks::GreeksError"){.enum}\> {#fn-gammaself---resultdecimal-greekserror .code-header}
:::

::: docblock
Calculates the aggregate gamma value for all options.

Gamma measures the rate of change of delta with respect to changes in
the underlying asset's price.

##### [§](#errors-4){.doc-anchor}Errors

Returns a `GreeksError` if the options can't be retrieved or gamma
calculation fails.
:::

::: {#method.theta .section .method}
[Source](../../src/optionstratlib/greeks/equations.rs.html#188-195){.src
.rightside}

#### fn [theta](#method.theta){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Decimal](../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}, [GreeksError](../error/greeks/enum.GreeksError.html "enum optionstratlib::error::greeks::GreeksError"){.enum}\> {#fn-thetaself---resultdecimal-greekserror .code-header}
:::

::: docblock
Calculates the aggregate theta value for all options.

Theta measures the rate of change of the option price with respect to
time, also known as time decay.

##### [§](#errors-5){.doc-anchor}Errors

Returns a `GreeksError` if the options can't be retrieved or theta
calculation fails.
:::

::: {#method.vega .section .method}
[Source](../../src/optionstratlib/greeks/equations.rs.html#205-212){.src
.rightside}

#### fn [vega](#method.vega){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Decimal](../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}, [GreeksError](../error/greeks/enum.GreeksError.html "enum optionstratlib::error::greeks::GreeksError"){.enum}\> {#fn-vegaself---resultdecimal-greekserror .code-header}
:::

::: docblock
Calculates the aggregate vega value for all options.

Vega measures the sensitivity of the option price to changes in the
volatility of the underlying asset.

##### [§](#errors-6){.doc-anchor}Errors

Returns a `GreeksError` if the options can't be retrieved or vega
calculation fails.
:::

::: {#method.rho .section .method}
[Source](../../src/optionstratlib/greeks/equations.rs.html#222-229){.src
.rightside}

#### fn [rho](#method.rho){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Decimal](../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}, [GreeksError](../error/greeks/enum.GreeksError.html "enum optionstratlib::error::greeks::GreeksError"){.enum}\> {#fn-rhoself---resultdecimal-greekserror .code-header}
:::

::: docblock
Calculates the aggregate rho value for all options.

Rho measures the sensitivity of the option price to changes in the
risk-free interest rate.

##### [§](#errors-7){.doc-anchor}Errors

Returns a `GreeksError` if the options can't be retrieved or rho
calculation fails.
:::

::: {#method.rho_d .section .method}
[Source](../../src/optionstratlib/greeks/equations.rs.html#239-246){.src
.rightside}

#### fn [rho_d](#method.rho_d){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Decimal](../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}, [GreeksError](../error/greeks/enum.GreeksError.html "enum optionstratlib::error::greeks::GreeksError"){.enum}\> {#fn-rho_dself---resultdecimal-greekserror .code-header}
:::

::: docblock
Calculates the aggregate rho_d value for all options.

Rho_d measures the sensitivity of the option price to changes in the
dividend yield of the underlying asset.

##### [§](#errors-8){.doc-anchor}Errors

Returns a `GreeksError` if the options can't be retrieved or rho_d
calculation fails.
:::

::: {#method.alpha .section .method}
[Source](../../src/optionstratlib/greeks/equations.rs.html#256-263){.src
.rightside}

#### fn [alpha](#method.alpha){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Decimal](../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}, [GreeksError](../error/greeks/enum.GreeksError.html "enum optionstratlib::error::greeks::GreeksError"){.enum}\> {#fn-alphaself---resultdecimal-greekserror .code-header}
:::

::: docblock
Calculates the aggregate alpha value for all options.

Alpha represents the ratio between gamma and theta, providing insight
into the option's risk/reward efficiency with respect to time decay.

##### [§](#errors-9){.doc-anchor}Errors

Returns a `GreeksError` if the options can't be retrieved or alpha
calculation fails.
:::
:::::::::::::::::::

## Implementors[§](#implementors){.anchor} {#implementors .section-header}

:::::::::::::::::::::::::: {#implementors-list}
::: {#impl-Greeks-for-Options .section .impl}
[Source](../../src/optionstratlib/model/option.rs.html#714-718){.src
.rightside}[§](#impl-Greeks-for-Options){.anchor}

### impl [Greeks](trait.Greeks.html "trait optionstratlib::greeks::Greeks"){.trait} for [Options](../model/option/struct.Options.html "struct optionstratlib::model::option::Options"){.struct} {#impl-greeks-for-options .code-header}
:::

:::: {#impl-Greeks-for-Position .section .impl}
[Source](../../src/optionstratlib/model/position.rs.html#677-690){.src
.rightside}[§](#impl-Greeks-for-Position){.anchor}

### impl [Greeks](trait.Greeks.html "trait optionstratlib::greeks::Greeks"){.trait} for [Position](../model/position/struct.Position.html "struct optionstratlib::model::position::Position"){.struct} {#impl-greeks-for-position .code-header}

::: docblock
Implementation of the `Greeks` trait for the `Position` struct.
:::
::::

::: docblock
This implementation allows a `Position` to calculate option Greeks
(delta, gamma, theta, vega, rho, etc.) by accessing its underlying
option contract. The implementation provides a way to expose the
position's option for use in Greek calculations.
:::

::: {#impl-Greeks-for-BearCallSpread .section .impl}
[Source](../../src/optionstratlib/strategies/bear_call_spread.rs.html#814-818){.src
.rightside}[§](#impl-Greeks-for-BearCallSpread){.anchor}

### impl [Greeks](trait.Greeks.html "trait optionstratlib::greeks::Greeks"){.trait} for [BearCallSpread](../strategies/bear_call_spread/struct.BearCallSpread.html "struct optionstratlib::strategies::bear_call_spread::BearCallSpread"){.struct} {#impl-greeks-for-bearcallspread .code-header}
:::

::: {#impl-Greeks-for-BearPutSpread .section .impl}
[Source](../../src/optionstratlib/strategies/bear_put_spread.rs.html#810-814){.src
.rightside}[§](#impl-Greeks-for-BearPutSpread){.anchor}

### impl [Greeks](trait.Greeks.html "trait optionstratlib::greeks::Greeks"){.trait} for [BearPutSpread](../strategies/bear_put_spread/struct.BearPutSpread.html "struct optionstratlib::strategies::bear_put_spread::BearPutSpread"){.struct} {#impl-greeks-for-bearputspread .code-header}
:::

::: {#impl-Greeks-for-BullCallSpread .section .impl}
[Source](../../src/optionstratlib/strategies/bull_call_spread.rs.html#827-831){.src
.rightside}[§](#impl-Greeks-for-BullCallSpread){.anchor}

### impl [Greeks](trait.Greeks.html "trait optionstratlib::greeks::Greeks"){.trait} for [BullCallSpread](../strategies/bull_call_spread/struct.BullCallSpread.html "struct optionstratlib::strategies::bull_call_spread::BullCallSpread"){.struct} {#impl-greeks-for-bullcallspread .code-header}
:::

::: {#impl-Greeks-for-BullPutSpread .section .impl}
[Source](../../src/optionstratlib/strategies/bull_put_spread.rs.html#921-925){.src
.rightside}[§](#impl-Greeks-for-BullPutSpread){.anchor}

### impl [Greeks](trait.Greeks.html "trait optionstratlib::greeks::Greeks"){.trait} for [BullPutSpread](../strategies/bull_put_spread/struct.BullPutSpread.html "struct optionstratlib::strategies::bull_put_spread::BullPutSpread"){.struct} {#impl-greeks-for-bullputspread .code-header}
:::

::: {#impl-Greeks-for-CallButterfly .section .impl}
[Source](../../src/optionstratlib/strategies/call_butterfly.rs.html#970-978){.src
.rightside}[§](#impl-Greeks-for-CallButterfly){.anchor}

### impl [Greeks](trait.Greeks.html "trait optionstratlib::greeks::Greeks"){.trait} for [CallButterfly](../strategies/call_butterfly/struct.CallButterfly.html "struct optionstratlib::strategies::call_butterfly::CallButterfly"){.struct} {#impl-greeks-for-callbutterfly .code-header}
:::

::: {#impl-Greeks-for-CustomStrategy .section .impl}
[Source](../../src/optionstratlib/strategies/custom.rs.html#918-926){.src
.rightside}[§](#impl-Greeks-for-CustomStrategy){.anchor}

### impl [Greeks](trait.Greeks.html "trait optionstratlib::greeks::Greeks"){.trait} for [CustomStrategy](../strategies/custom/struct.CustomStrategy.html "struct optionstratlib::strategies::custom::CustomStrategy"){.struct} {#impl-greeks-for-customstrategy .code-header}
:::

::: {#impl-Greeks-for-IronButterfly .section .impl}
[Source](../../src/optionstratlib/strategies/iron_butterfly.rs.html#1043-1052){.src
.rightside}[§](#impl-Greeks-for-IronButterfly){.anchor}

### impl [Greeks](trait.Greeks.html "trait optionstratlib::greeks::Greeks"){.trait} for [IronButterfly](../strategies/iron_butterfly/struct.IronButterfly.html "struct optionstratlib::strategies::iron_butterfly::IronButterfly"){.struct} {#impl-greeks-for-ironbutterfly .code-header}
:::

::: {#impl-Greeks-for-IronCondor .section .impl}
[Source](../../src/optionstratlib/strategies/iron_condor.rs.html#1071-1080){.src
.rightside}[§](#impl-Greeks-for-IronCondor){.anchor}

### impl [Greeks](trait.Greeks.html "trait optionstratlib::greeks::Greeks"){.trait} for [IronCondor](../strategies/iron_condor/struct.IronCondor.html "struct optionstratlib::strategies::iron_condor::IronCondor"){.struct} {#impl-greeks-for-ironcondor .code-header}
:::

::: {#impl-Greeks-for-LongButterflySpread .section .impl}
[Source](../../src/optionstratlib/strategies/long_butterfly_spread.rs.html#1015-1023){.src
.rightside}[§](#impl-Greeks-for-LongButterflySpread){.anchor}

### impl [Greeks](trait.Greeks.html "trait optionstratlib::greeks::Greeks"){.trait} for [LongButterflySpread](../strategies/long_butterfly_spread/struct.LongButterflySpread.html "struct optionstratlib::strategies::long_butterfly_spread::LongButterflySpread"){.struct} {#impl-greeks-for-longbutterflyspread .code-header}
:::

::: {#impl-Greeks-for-LongCall .section .impl}
[Source](../../src/optionstratlib/strategies/long_call.rs.html#485-489){.src
.rightside}[§](#impl-Greeks-for-LongCall){.anchor}

### impl [Greeks](trait.Greeks.html "trait optionstratlib::greeks::Greeks"){.trait} for [LongCall](../strategies/long_call/struct.LongCall.html "struct optionstratlib::strategies::long_call::LongCall"){.struct} {#impl-greeks-for-longcall .code-header}
:::

::: {#impl-Greeks-for-LongPut .section .impl}
[Source](../../src/optionstratlib/strategies/long_put.rs.html#482-486){.src
.rightside}[§](#impl-Greeks-for-LongPut){.anchor}

### impl [Greeks](trait.Greeks.html "trait optionstratlib::greeks::Greeks"){.trait} for [LongPut](../strategies/long_put/struct.LongPut.html "struct optionstratlib::strategies::long_put::LongPut"){.struct} {#impl-greeks-for-longput .code-header}
:::

::: {#impl-Greeks-for-LongStraddle .section .impl}
[Source](../../src/optionstratlib/strategies/long_straddle.rs.html#824-828){.src
.rightside}[§](#impl-Greeks-for-LongStraddle){.anchor}

### impl [Greeks](trait.Greeks.html "trait optionstratlib::greeks::Greeks"){.trait} for [LongStraddle](../strategies/long_straddle/struct.LongStraddle.html "struct optionstratlib::strategies::long_straddle::LongStraddle"){.struct} {#impl-greeks-for-longstraddle .code-header}
:::

::: {#impl-Greeks-for-LongStrangle .section .impl}
[Source](../../src/optionstratlib/strategies/long_strangle.rs.html#883-887){.src
.rightside}[§](#impl-Greeks-for-LongStrangle){.anchor}

### impl [Greeks](trait.Greeks.html "trait optionstratlib::greeks::Greeks"){.trait} for [LongStrangle](../strategies/long_strangle/struct.LongStrangle.html "struct optionstratlib::strategies::long_strangle::LongStrangle"){.struct} {#impl-greeks-for-longstrangle .code-header}
:::

::: {#impl-Greeks-for-PoorMansCoveredCall .section .impl}
[Source](../../src/optionstratlib/strategies/poor_mans_covered_call.rs.html#817-821){.src
.rightside}[§](#impl-Greeks-for-PoorMansCoveredCall){.anchor}

### impl [Greeks](trait.Greeks.html "trait optionstratlib::greeks::Greeks"){.trait} for [PoorMansCoveredCall](../strategies/poor_mans_covered_call/struct.PoorMansCoveredCall.html "struct optionstratlib::strategies::poor_mans_covered_call::PoorMansCoveredCall"){.struct} {#impl-greeks-for-poormanscoveredcall .code-header}
:::

::: {#impl-Greeks-for-ShortButterflySpread .section .impl}
[Source](../../src/optionstratlib/strategies/short_butterfly_spread.rs.html#983-991){.src
.rightside}[§](#impl-Greeks-for-ShortButterflySpread){.anchor}

### impl [Greeks](trait.Greeks.html "trait optionstratlib::greeks::Greeks"){.trait} for [ShortButterflySpread](../strategies/short_butterfly_spread/struct.ShortButterflySpread.html "struct optionstratlib::strategies::short_butterfly_spread::ShortButterflySpread"){.struct} {#impl-greeks-for-shortbutterflyspread .code-header}
:::

::: {#impl-Greeks-for-ShortCall .section .impl}
[Source](../../src/optionstratlib/strategies/short_call.rs.html#493-497){.src
.rightside}[§](#impl-Greeks-for-ShortCall){.anchor}

### impl [Greeks](trait.Greeks.html "trait optionstratlib::greeks::Greeks"){.trait} for [ShortCall](../strategies/short_call/struct.ShortCall.html "struct optionstratlib::strategies::short_call::ShortCall"){.struct} {#impl-greeks-for-shortcall .code-header}
:::

::: {#impl-Greeks-for-ShortPut .section .impl}
[Source](../../src/optionstratlib/strategies/short_put.rs.html#487-491){.src
.rightside}[§](#impl-Greeks-for-ShortPut){.anchor}

### impl [Greeks](trait.Greeks.html "trait optionstratlib::greeks::Greeks"){.trait} for [ShortPut](../strategies/short_put/struct.ShortPut.html "struct optionstratlib::strategies::short_put::ShortPut"){.struct} {#impl-greeks-for-shortput .code-header}
:::

::: {#impl-Greeks-for-ShortStraddle .section .impl}
[Source](../../src/optionstratlib/strategies/short_straddle.rs.html#871-875){.src
.rightside}[§](#impl-Greeks-for-ShortStraddle){.anchor}

### impl [Greeks](trait.Greeks.html "trait optionstratlib::greeks::Greeks"){.trait} for [ShortStraddle](../strategies/short_straddle/struct.ShortStraddle.html "struct optionstratlib::strategies::short_straddle::ShortStraddle"){.struct} {#impl-greeks-for-shortstraddle .code-header}
:::

::: {#impl-Greeks-for-ShortStrangle .section .impl}
[Source](../../src/optionstratlib/strategies/short_strangle.rs.html#1130-1134){.src
.rightside}[§](#impl-Greeks-for-ShortStrangle){.anchor}

### impl [Greeks](trait.Greeks.html "trait optionstratlib::greeks::Greeks"){.trait} for [ShortStrangle](../strategies/short_strangle/struct.ShortStrangle.html "struct optionstratlib::strategies::short_strangle::ShortStrangle"){.struct} {#impl-greeks-for-shortstrangle .code-header}
:::
::::::::::::::::::::::::::
::::::::::::::::::::::::::::::::::::::::::::::::::
:::::::::::::::::::::::::::::::::::::::::::::::::::
