:::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::: width-limiter
::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../index.html)::[simulation](index.html)
:::

# Enum [WalkType]{.enum}Copy item path

[[Source](../../src/optionstratlib/simulation/model.rs.html#9-137){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub enum WalkType {
    Brownian {
        dt: Positive,
        drift: Decimal,
        volatility: Positive,
    },
    GeometricBrownian {
        dt: Positive,
        drift: Decimal,
        volatility: Positive,
    },
    LogReturns {
        dt: Positive,
        expected_return: Decimal,
        volatility: Positive,
        autocorrelation: Option<Decimal>,
    },
    MeanReverting {
        dt: Positive,
        volatility: Positive,
        speed: Positive,
        mean: Positive,
    },
    JumpDiffusion {
        dt: Positive,
        drift: Decimal,
        volatility: Positive,
        intensity: Positive,
        jump_mean: Decimal,
        jump_volatility: Positive,
    },
    Garch {
        dt: Positive,
        drift: Decimal,
        volatility: Positive,
        alpha: Positive,
        beta: Positive,
        omega: Positive,
    },
    Heston {
        dt: Positive,
        drift: Decimal,
        volatility: Positive,
        kappa: Positive,
        theta: Positive,
        xi: Positive,
        rho: Decimal,
    },
    Custom {
        dt: Positive,
        drift: Decimal,
        volatility: Positive,
        vov: Positive,
        vol_speed: Positive,
        vol_mean: Positive,
    },
    Historical {
        timeframe: TimeFrame,
        prices: Vec<Positive>,
    },
}
```

Expand description

::: docblock
Enum defining different types of random walks
:::

## Variants[§](#variants){.anchor} {#variants .variants .section-header}

:::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::: variants
::: {#variant.Brownian .section .variant}
[§](#variant.Brownian){.anchor}

### Brownian {#brownian .code-header}
:::

::: docblock
Standard Brownian motion (normal increments)
:::

::::::::: {#variant.Brownian.fields .sub-variant}
#### Fields

:::: sub-variant-field
[[§](#variant.Brownian.field.dt){.anchor
.field}`dt: `[`Positive`](../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}]{#variant.Brownian.field.dt
.section-header}

::: docblock
Time step size (fraction of year: daily=1/365, weekly=1/52, etc.)
:::
::::

:::: sub-variant-field
[[§](#variant.Brownian.field.drift){.anchor
.field}`drift: Decimal`]{#variant.Brownian.field.drift .section-header}

::: docblock
Drift parameter (expected return or growth rate)
:::
::::

:::: sub-variant-field
[[§](#variant.Brownian.field.volatility){.anchor
.field}`volatility: `[`Positive`](../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}]{#variant.Brownian.field.volatility
.section-header}

::: docblock
Volatility parameter (annualized standard deviation)
:::
::::
:::::::::

::: {#variant.GeometricBrownian .section .variant}
[§](#variant.GeometricBrownian){.anchor}

### GeometricBrownian {#geometricbrownian .code-header}
:::

::: docblock
Geometric Brownian motion (log-normal increments)
:::

::::::::: {#variant.GeometricBrownian.fields .sub-variant}
#### Fields

:::: sub-variant-field
[[§](#variant.GeometricBrownian.field.dt){.anchor
.field}`dt: `[`Positive`](../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}]{#variant.GeometricBrownian.field.dt
.section-header}

::: docblock
Time step size (fraction of year: daily=1/365, weekly=1/52, etc.)
:::
::::

:::: sub-variant-field
[[§](#variant.GeometricBrownian.field.drift){.anchor
.field}`drift: Decimal`]{#variant.GeometricBrownian.field.drift
.section-header}

::: docblock
Drift parameter (expected return or growth rate)
:::
::::

:::: sub-variant-field
[[§](#variant.GeometricBrownian.field.volatility){.anchor
.field}`volatility: `[`Positive`](../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}]{#variant.GeometricBrownian.field.volatility
.section-header}

::: docblock
Volatility parameter (annualized standard deviation)
:::
::::
:::::::::

::: {#variant.LogReturns .section .variant}
[§](#variant.LogReturns){.anchor}

### LogReturns {#logreturns .code-header}
:::

::: docblock
Log-Returns model (simulates directly log-returns instead of prices)
:::

::::::::::: {#variant.LogReturns.fields .sub-variant}
#### Fields

:::: sub-variant-field
[[§](#variant.LogReturns.field.dt){.anchor
.field}`dt: `[`Positive`](../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}]{#variant.LogReturns.field.dt
.section-header}

::: docblock
Time step size (fraction of year: daily=1/365, weekly=1/52, etc.)
:::
::::

:::: sub-variant-field
[[§](#variant.LogReturns.field.expected_return){.anchor
.field}`expected_return: Decimal`]{#variant.LogReturns.field.expected_return
.section-header}

::: docblock
Expected return (mean of log returns)
:::
::::

:::: sub-variant-field
[[§](#variant.LogReturns.field.volatility){.anchor
.field}`volatility: `[`Positive`](../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}]{#variant.LogReturns.field.volatility
.section-header}

::: docblock
Volatility parameter (annualized standard deviation of log returns)
:::
::::

:::: sub-variant-field
[[§](#variant.LogReturns.field.autocorrelation){.anchor
.field}`autocorrelation: `[`Option`](https://doc.rust-lang.org/1.86.0/core/option/enum.Option.html "enum core::option::Option"){.enum}`<Decimal>`]{#variant.LogReturns.field.autocorrelation
.section-header}

::: docblock
Optional autocorrelation parameter (-1 to 1)
:::
::::
:::::::::::

::: {#variant.MeanReverting .section .variant}
[§](#variant.MeanReverting){.anchor}

### MeanReverting {#meanreverting .code-header}
:::

::: docblock
Mean-reverting process (Ornstein-Uhlenbeck)
:::

::::::::::: {#variant.MeanReverting.fields .sub-variant}
#### Fields

:::: sub-variant-field
[[§](#variant.MeanReverting.field.dt){.anchor
.field}`dt: `[`Positive`](../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}]{#variant.MeanReverting.field.dt
.section-header}

::: docblock
Time step size (fraction of year: daily=1/365, weekly=1/52, etc.)
:::
::::

:::: sub-variant-field
[[§](#variant.MeanReverting.field.volatility){.anchor
.field}`volatility: `[`Positive`](../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}]{#variant.MeanReverting.field.volatility
.section-header}

::: docblock
Volatility parameter (annualized standard deviation)
:::
::::

:::: sub-variant-field
[[§](#variant.MeanReverting.field.speed){.anchor
.field}`speed: `[`Positive`](../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}]{#variant.MeanReverting.field.speed
.section-header}

::: docblock
Mean reversion speed (rate at which process reverts to mean)
:::
::::

:::: sub-variant-field
[[§](#variant.MeanReverting.field.mean){.anchor
.field}`mean: `[`Positive`](../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}]{#variant.MeanReverting.field.mean
.section-header}

::: docblock
Long-term mean (equilibrium level)
:::
::::
:::::::::::

::: {#variant.JumpDiffusion .section .variant}
[§](#variant.JumpDiffusion){.anchor}

### JumpDiffusion {#jumpdiffusion .code-header}
:::

::: docblock
Jump diffusion process (normal increments with occasional jumps)
:::

::::::::::::::: {#variant.JumpDiffusion.fields .sub-variant}
#### Fields

:::: sub-variant-field
[[§](#variant.JumpDiffusion.field.dt){.anchor
.field}`dt: `[`Positive`](../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}]{#variant.JumpDiffusion.field.dt
.section-header}

::: docblock
Time step size (fraction of year: daily=1/365, weekly=1/52, etc.)
:::
::::

:::: sub-variant-field
[[§](#variant.JumpDiffusion.field.drift){.anchor
.field}`drift: Decimal`]{#variant.JumpDiffusion.field.drift
.section-header}

::: docblock
Drift parameter (expected return of continuous part)
:::
::::

:::: sub-variant-field
[[§](#variant.JumpDiffusion.field.volatility){.anchor
.field}`volatility: `[`Positive`](../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}]{#variant.JumpDiffusion.field.volatility
.section-header}

::: docblock
Volatility parameter (annualized standard deviation of continuous part)
:::
::::

:::: sub-variant-field
[[§](#variant.JumpDiffusion.field.intensity){.anchor
.field}`intensity: `[`Positive`](../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}]{#variant.JumpDiffusion.field.intensity
.section-header}

::: docblock
Jump intensity (annual frequency of jumps)
:::
::::

:::: sub-variant-field
[[§](#variant.JumpDiffusion.field.jump_mean){.anchor
.field}`jump_mean: Decimal`]{#variant.JumpDiffusion.field.jump_mean
.section-header}

::: docblock
Jump size mean (average jump magnitude)
:::
::::

:::: sub-variant-field
[[§](#variant.JumpDiffusion.field.jump_volatility){.anchor
.field}`jump_volatility: `[`Positive`](../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}]{#variant.JumpDiffusion.field.jump_volatility
.section-header}

::: docblock
Jump size volatility (standard deviation of jump size)
:::
::::
:::::::::::::::

::: {#variant.Garch .section .variant}
[§](#variant.Garch){.anchor}

### Garch {#garch .code-header}
:::

::: docblock
GARCH process (time-varying volatility)
:::

::::::::::::::: {#variant.Garch.fields .sub-variant}
#### Fields

:::: sub-variant-field
[[§](#variant.Garch.field.dt){.anchor
.field}`dt: `[`Positive`](../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}]{#variant.Garch.field.dt
.section-header}

::: docblock
Time step size (fraction of year: daily=1/365, weekly=1/52, etc.)
:::
::::

:::: sub-variant-field
[[§](#variant.Garch.field.drift){.anchor
.field}`drift: Decimal`]{#variant.Garch.field.drift .section-header}

::: docblock
Drift parameter (expected return)
:::
::::

:::: sub-variant-field
[[§](#variant.Garch.field.volatility){.anchor
.field}`volatility: `[`Positive`](../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}]{#variant.Garch.field.volatility
.section-header}

::: docblock
Initial volatility parameter (starting volatility level)
:::
::::

:::: sub-variant-field
[[§](#variant.Garch.field.alpha){.anchor
.field}`alpha: `[`Positive`](../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}]{#variant.Garch.field.alpha
.section-header}

::: docblock
GARCH alpha parameter (impact of past observations)
:::
::::

:::: sub-variant-field
[[§](#variant.Garch.field.beta){.anchor
.field}`beta: `[`Positive`](../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}]{#variant.Garch.field.beta
.section-header}

::: docblock
GARCH beta parameter (persistence of volatility)
:::
::::

:::: sub-variant-field
[[§](#variant.Garch.field.omega){.anchor
.field}`omega: `[`Positive`](../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}]{#variant.Garch.field.omega
.section-header}

::: docblock
Long-term variance (unconditional variance)
:::
::::
:::::::::::::::

::: {#variant.Heston .section .variant}
[§](#variant.Heston){.anchor}

### Heston {#heston .code-header}
:::

::: docblock
Heston model (stochastic volatility)
:::

::::::::::::::::: {#variant.Heston.fields .sub-variant}
#### Fields

:::: sub-variant-field
[[§](#variant.Heston.field.dt){.anchor
.field}`dt: `[`Positive`](../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}]{#variant.Heston.field.dt
.section-header}

::: docblock
Time step size (fraction of year: daily=1/365, weekly=1/52, etc.)
:::
::::

:::: sub-variant-field
[[§](#variant.Heston.field.drift){.anchor
.field}`drift: Decimal`]{#variant.Heston.field.drift .section-header}

::: docblock
Drift parameter (expected return)
:::
::::

:::: sub-variant-field
[[§](#variant.Heston.field.volatility){.anchor
.field}`volatility: `[`Positive`](../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}]{#variant.Heston.field.volatility
.section-header}

::: docblock
Initial volatility parameter (starting volatility level)
:::
::::

:::: sub-variant-field
[[§](#variant.Heston.field.kappa){.anchor
.field}`kappa: `[`Positive`](../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}]{#variant.Heston.field.kappa
.section-header}

::: docblock
Mean reversion speed of volatility
:::
::::

:::: sub-variant-field
[[§](#variant.Heston.field.theta){.anchor
.field}`theta: `[`Positive`](../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}]{#variant.Heston.field.theta
.section-header}

::: docblock
Long-term variance (equilibrium level of variance)
:::
::::

:::: sub-variant-field
[[§](#variant.Heston.field.xi){.anchor
.field}`xi: `[`Positive`](../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}]{#variant.Heston.field.xi
.section-header}

::: docblock
Volatility of volatility (standard deviation of variance process)
:::
::::

:::: sub-variant-field
[[§](#variant.Heston.field.rho){.anchor
.field}`rho: Decimal`]{#variant.Heston.field.rho .section-header}

::: docblock
Correlation between price and volatility processes
:::
::::
:::::::::::::::::

::: {#variant.Custom .section .variant}
[§](#variant.Custom){.anchor}

### Custom {#custom .code-header}
:::

::: docblock
Custom process defined by a function
:::

::::::::::::::: {#variant.Custom.fields .sub-variant}
#### Fields

:::: sub-variant-field
[[§](#variant.Custom.field.dt){.anchor
.field}`dt: `[`Positive`](../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}]{#variant.Custom.field.dt
.section-header}

::: docblock
Time step size (fraction of year: daily=1/365, weekly=1/52, etc.)
:::
::::

:::: sub-variant-field
[[§](#variant.Custom.field.drift){.anchor
.field}`drift: Decimal`]{#variant.Custom.field.drift .section-header}

::: docblock
Drift parameter (expected change)
:::
::::

:::: sub-variant-field
[[§](#variant.Custom.field.volatility){.anchor
.field}`volatility: `[`Positive`](../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}]{#variant.Custom.field.volatility
.section-header}

::: docblock
Volatility parameter (may be interpreted differently based on custom
implementation)
:::
::::

:::: sub-variant-field
[[§](#variant.Custom.field.vov){.anchor
.field}`vov: `[`Positive`](../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}]{#variant.Custom.field.vov
.section-header}

::: docblock
Volatility of Volatility parameter (annualized standard deviation)
:::
::::

:::: sub-variant-field
[[§](#variant.Custom.field.vol_speed){.anchor
.field}`vol_speed: `[`Positive`](../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}]{#variant.Custom.field.vol_speed
.section-header}

::: docblock
Mean reversion speed (rate at which process reverts to mean)
:::
::::

:::: sub-variant-field
[[§](#variant.Custom.field.vol_mean){.anchor
.field}`vol_mean: `[`Positive`](../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}]{#variant.Custom.field.vol_mean
.section-header}

::: docblock
Long-term mean (equilibrium level)
:::
::::
:::::::::::::::

::: {#variant.Historical .section .variant}
[§](#variant.Historical){.anchor}

### Historical {#historical .code-header}
:::

::: docblock
Represents historical price data for a given timeframe.

This encapsulates the historical price data, including the timeframe
over which the data was collected and a vector of positive price values.
It is typically used to store and process historical market data for
financial analysis and simulation purposes.

#### [§](#fields-1){.doc-anchor}Fields {#fields-1}

- `timeframe`: The `TimeFrame` over which the historical data is
  relevant.
- `prices`: A `Vec` of `Positive` values representing the historical
  prices.
:::

::::::: {#variant.Historical.fields .sub-variant}
#### Fields {#fields-8}

:::: sub-variant-field
[[§](#variant.Historical.field.timeframe){.anchor
.field}`timeframe: `[`TimeFrame`](../utils/time/enum.TimeFrame.html "enum optionstratlib::utils::time::TimeFrame"){.enum}]{#variant.Historical.field.timeframe
.section-header}

::: docblock
The timeframe of the historical data.
:::
::::

:::: sub-variant-field
[[§](#variant.Historical.field.prices){.anchor
.field}`prices: `[`Vec`](https://doc.rust-lang.org/1.86.0/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}`<`[`Positive`](../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}`>`]{#variant.Historical.field.prices
.section-header}

::: docblock
The vector of positive price values.
:::
::::
:::::::
::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::

## Trait Implementations[§](#trait-implementations){.anchor} {#trait-implementations .section-header}

:::::::::::::::::::::::::::::::::: {#trait-implementations-list}
::: {#impl-Clone-for-WalkType .section .impl}
[Source](../../src/optionstratlib/simulation/model.rs.html#8){.src
.rightside}[§](#impl-Clone-for-WalkType){.anchor}

### impl [Clone](https://doc.rust-lang.org/1.86.0/core/clone/trait.Clone.html "trait core::clone::Clone"){.trait} for [WalkType](enum.WalkType.html "enum optionstratlib::simulation::WalkType"){.enum} {#impl-clone-for-walktype .code-header}
:::

::::::: impl-items
::: {#method.clone .section .method .trait-impl}
[Source](../../src/optionstratlib/simulation/model.rs.html#8){.src
.rightside}[§](#method.clone){.anchor}

#### fn [clone](https://doc.rust-lang.org/1.86.0/core/clone/trait.Clone.html#tymethod.clone){.fn}(&self) -\> [WalkType](enum.WalkType.html "enum optionstratlib::simulation::WalkType"){.enum} {#fn-cloneself---walktype .code-header}
:::

::: docblock
Returns a copy of the value. [Read
more](https://doc.rust-lang.org/1.86.0/core/clone/trait.Clone.html#tymethod.clone)
:::

::: {#method.clone_from .section .method .trait-impl}
[[1.0.0]{.since title="Stable since Rust version 1.0.0"} ·
[Source](https://doc.rust-lang.org/1.86.0/src/core/clone.rs.html#174){.src}]{.rightside}[§](#method.clone_from){.anchor}

#### fn [clone_from](https://doc.rust-lang.org/1.86.0/core/clone/trait.Clone.html#method.clone_from){.fn}(&mut self, source: &Self) {#fn-clone_frommut-self-source-self .code-header}
:::

::: docblock
Performs copy-assignment from `source`. [Read
more](https://doc.rust-lang.org/1.86.0/core/clone/trait.Clone.html#method.clone_from)
:::
:::::::

::: {#impl-Debug-for-WalkType .section .impl}
[Source](../../src/optionstratlib/simulation/model.rs.html#8){.src
.rightside}[§](#impl-Debug-for-WalkType){.anchor}

### impl [Debug](https://doc.rust-lang.org/1.86.0/core/fmt/trait.Debug.html "trait core::fmt::Debug"){.trait} for [WalkType](enum.WalkType.html "enum optionstratlib::simulation::WalkType"){.enum} {#impl-debug-for-walktype .code-header}
:::

::::: impl-items
::: {#method.fmt .section .method .trait-impl}
[Source](../../src/optionstratlib/simulation/model.rs.html#8){.src
.rightside}[§](#method.fmt){.anchor}

#### fn [fmt](https://doc.rust-lang.org/1.86.0/core/fmt/trait.Debug.html#tymethod.fmt){.fn}(&self, f: &mut [Formatter](https://doc.rust-lang.org/1.86.0/core/fmt/struct.Formatter.html "struct core::fmt::Formatter"){.struct}\<\'\_\>) -\> [Result](https://doc.rust-lang.org/1.86.0/core/fmt/type.Result.html "type core::fmt::Result"){.type} {#fn-fmtself-f-mut-formatter_---result .code-header}
:::

::: docblock
Formats the value using the given formatter. [Read
more](https://doc.rust-lang.org/1.86.0/core/fmt/trait.Debug.html#tymethod.fmt)
:::
:::::

::: {#impl-Deserialize%3C'de%3E-for-WalkType .section .impl}
[Source](../../src/optionstratlib/simulation/model.rs.html#8){.src
.rightside}[§](#impl-Deserialize%3C'de%3E-for-WalkType){.anchor}

### impl\<\'de\> [Deserialize](https://docs.rs/serde/1.0.219/serde/de/trait.Deserialize.html "trait serde::de::Deserialize"){.trait}\<\'de\> for [WalkType](enum.WalkType.html "enum optionstratlib::simulation::WalkType"){.enum} {#implde-deserializede-for-walktype .code-header}
:::

:::::: impl-items
:::: {#method.deserialize .section .method .trait-impl}
[Source](../../src/optionstratlib/simulation/model.rs.html#8){.src
.rightside}[§](#method.deserialize){.anchor}

#### fn [deserialize](https://docs.rs/serde/1.0.219/serde/de/trait.Deserialize.html#tymethod.deserialize){.fn}\<\_\_D\>(\_\_deserializer: \_\_D) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<Self, \_\_D::[Error](https://docs.rs/serde/1.0.219/serde/de/trait.Deserializer.html#associatedtype.Error "type serde::de::Deserializer::Error"){.associatedtype}\> {#fn-deserialize__d__deserializer-__d---resultself-__derror .code-header}

::: where
where \_\_D:
[Deserializer](https://docs.rs/serde/1.0.219/serde/de/trait.Deserializer.html "trait serde::de::Deserializer"){.trait}\<\'de\>,
:::
::::

::: docblock
Deserialize this value from the given Serde deserializer. [Read
more](https://docs.rs/serde/1.0.219/serde/de/trait.Deserialize.html#tymethod.deserialize)
:::
::::::

::: {#impl-Display-for-WalkType .section .impl}
[Source](../../src/optionstratlib/simulation/model.rs.html#139-236){.src
.rightside}[§](#impl-Display-for-WalkType){.anchor}

### impl [Display](https://doc.rust-lang.org/1.86.0/core/fmt/trait.Display.html "trait core::fmt::Display"){.trait} for [WalkType](enum.WalkType.html "enum optionstratlib::simulation::WalkType"){.enum} {#impl-display-for-walktype .code-header}
:::

::::: impl-items
::: {#method.fmt-1 .section .method .trait-impl}
[Source](../../src/optionstratlib/simulation/model.rs.html#140-235){.src
.rightside}[§](#method.fmt-1){.anchor}

#### fn [fmt](https://doc.rust-lang.org/1.86.0/core/fmt/trait.Display.html#tymethod.fmt){.fn}(&self, f: &mut [Formatter](https://doc.rust-lang.org/1.86.0/core/fmt/struct.Formatter.html "struct core::fmt::Formatter"){.struct}\<\'\_\>) -\> [Result](https://doc.rust-lang.org/1.86.0/core/fmt/type.Result.html "type core::fmt::Result"){.type} {#fn-fmtself-f-mut-formatter_---result-1 .code-header}
:::

::: docblock
Formats the value using the given formatter. [Read
more](https://doc.rust-lang.org/1.86.0/core/fmt/trait.Display.html#tymethod.fmt)
:::
:::::

::: {#impl-PartialEq-for-WalkType .section .impl}
[Source](../../src/optionstratlib/simulation/model.rs.html#8){.src
.rightside}[§](#impl-PartialEq-for-WalkType){.anchor}

### impl [PartialEq](https://doc.rust-lang.org/1.86.0/core/cmp/trait.PartialEq.html "trait core::cmp::PartialEq"){.trait} for [WalkType](enum.WalkType.html "enum optionstratlib::simulation::WalkType"){.enum} {#impl-partialeq-for-walktype .code-header}
:::

::::::: impl-items
::: {#method.eq .section .method .trait-impl}
[Source](../../src/optionstratlib/simulation/model.rs.html#8){.src
.rightside}[§](#method.eq){.anchor}

#### fn [eq](https://doc.rust-lang.org/1.86.0/core/cmp/trait.PartialEq.html#tymethod.eq){.fn}(&self, other: &[WalkType](enum.WalkType.html "enum optionstratlib::simulation::WalkType"){.enum}) -\> [bool](https://doc.rust-lang.org/1.86.0/std/primitive.bool.html){.primitive} {#fn-eqself-other-walktype---bool .code-header}
:::

::: docblock
Tests for `self` and `other` values to be equal, and is used by `==`.
:::

::: {#method.ne .section .method .trait-impl}
[[1.0.0]{.since title="Stable since Rust version 1.0.0"} ·
[Source](https://doc.rust-lang.org/1.86.0/src/core/cmp.rs.html#261){.src}]{.rightside}[§](#method.ne){.anchor}

#### fn [ne](https://doc.rust-lang.org/1.86.0/core/cmp/trait.PartialEq.html#method.ne){.fn}(&self, other: [&Rhs](https://doc.rust-lang.org/1.86.0/std/primitive.reference.html){.primitive}) -\> [bool](https://doc.rust-lang.org/1.86.0/std/primitive.bool.html){.primitive} {#fn-neself-other-rhs---bool .code-header}
:::

::: docblock
Tests for `!=`. The default implementation is almost always sufficient,
and should not be overridden without very good reason.
:::
:::::::

::: {#impl-Serialize-for-WalkType .section .impl}
[Source](../../src/optionstratlib/simulation/model.rs.html#8){.src
.rightside}[§](#impl-Serialize-for-WalkType){.anchor}

### impl [Serialize](https://docs.rs/serde/1.0.219/serde/ser/trait.Serialize.html "trait serde::ser::Serialize"){.trait} for [WalkType](enum.WalkType.html "enum optionstratlib::simulation::WalkType"){.enum} {#impl-serialize-for-walktype .code-header}
:::

:::::: impl-items
:::: {#method.serialize .section .method .trait-impl}
[Source](../../src/optionstratlib/simulation/model.rs.html#8){.src
.rightside}[§](#method.serialize){.anchor}

#### fn [serialize](https://docs.rs/serde/1.0.219/serde/ser/trait.Serialize.html#tymethod.serialize){.fn}\<\_\_S\>(&self, \_\_serializer: \_\_S) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<\_\_S::[Ok](https://docs.rs/serde/1.0.219/serde/ser/trait.Serializer.html#associatedtype.Ok "type serde::ser::Serializer::Ok"){.associatedtype}, \_\_S::[Error](https://docs.rs/serde/1.0.219/serde/ser/trait.Serializer.html#associatedtype.Error "type serde::ser::Serializer::Error"){.associatedtype}\> {#fn-serialize__sself-__serializer-__s---result__sok-__serror .code-header}

::: where
where \_\_S:
[Serializer](https://docs.rs/serde/1.0.219/serde/ser/trait.Serializer.html "trait serde::ser::Serializer"){.trait},
:::
::::

::: docblock
Serialize this value into the given Serde serializer. [Read
more](https://docs.rs/serde/1.0.219/serde/ser/trait.Serialize.html#tymethod.serialize)
:::
::::::

::: {#impl-StructuralPartialEq-for-WalkType .section .impl}
[Source](../../src/optionstratlib/simulation/model.rs.html#8){.src
.rightside}[§](#impl-StructuralPartialEq-for-WalkType){.anchor}

### impl [StructuralPartialEq](https://doc.rust-lang.org/1.86.0/core/marker/trait.StructuralPartialEq.html "trait core::marker::StructuralPartialEq"){.trait} for [WalkType](enum.WalkType.html "enum optionstratlib::simulation::WalkType"){.enum} {#impl-structuralpartialeq-for-walktype .code-header}
:::
::::::::::::::::::::::::::::::::::

## Auto Trait Implementations[§](#synthetic-implementations){.anchor} {#synthetic-implementations .section-header}

::::::::: {#synthetic-implementations-list}
::: {#impl-Freeze-for-WalkType .section .impl}
[§](#impl-Freeze-for-WalkType){.anchor}

### impl [Freeze](https://doc.rust-lang.org/1.86.0/core/marker/trait.Freeze.html "trait core::marker::Freeze"){.trait} for [WalkType](enum.WalkType.html "enum optionstratlib::simulation::WalkType"){.enum} {#impl-freeze-for-walktype .code-header}
:::

::: {#impl-RefUnwindSafe-for-WalkType .section .impl}
[§](#impl-RefUnwindSafe-for-WalkType){.anchor}

### impl [RefUnwindSafe](https://doc.rust-lang.org/1.86.0/core/panic/unwind_safe/trait.RefUnwindSafe.html "trait core::panic::unwind_safe::RefUnwindSafe"){.trait} for [WalkType](enum.WalkType.html "enum optionstratlib::simulation::WalkType"){.enum} {#impl-refunwindsafe-for-walktype .code-header}
:::

::: {#impl-Send-for-WalkType .section .impl}
[§](#impl-Send-for-WalkType){.anchor}

### impl [Send](https://doc.rust-lang.org/1.86.0/core/marker/trait.Send.html "trait core::marker::Send"){.trait} for [WalkType](enum.WalkType.html "enum optionstratlib::simulation::WalkType"){.enum} {#impl-send-for-walktype .code-header}
:::

::: {#impl-Sync-for-WalkType .section .impl}
[§](#impl-Sync-for-WalkType){.anchor}

### impl [Sync](https://doc.rust-lang.org/1.86.0/core/marker/trait.Sync.html "trait core::marker::Sync"){.trait} for [WalkType](enum.WalkType.html "enum optionstratlib::simulation::WalkType"){.enum} {#impl-sync-for-walktype .code-header}
:::

::: {#impl-Unpin-for-WalkType .section .impl}
[§](#impl-Unpin-for-WalkType){.anchor}

### impl [Unpin](https://doc.rust-lang.org/1.86.0/core/marker/trait.Unpin.html "trait core::marker::Unpin"){.trait} for [WalkType](enum.WalkType.html "enum optionstratlib::simulation::WalkType"){.enum} {#impl-unpin-for-walktype .code-header}
:::

::: {#impl-UnwindSafe-for-WalkType .section .impl}
[§](#impl-UnwindSafe-for-WalkType){.anchor}

### impl [UnwindSafe](https://doc.rust-lang.org/1.86.0/core/panic/unwind_safe/trait.UnwindSafe.html "trait core::panic::unwind_safe::UnwindSafe"){.trait} for [WalkType](enum.WalkType.html "enum optionstratlib::simulation::WalkType"){.enum} {#impl-unwindsafe-for-walktype .code-header}
:::
:::::::::

## Blanket Implementations[§](#blanket-implementations){.anchor} {#blanket-implementations .section-header}

:::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::: {#blanket-implementations-list}
:::: {#impl-Any-for-T .section .impl}
[Source](https://doc.rust-lang.org/1.86.0/src/core/any.rs.html#138){.src
.rightside}[§](#impl-Any-for-T){.anchor}

### impl\<T\> [Any](https://doc.rust-lang.org/1.86.0/core/any/trait.Any.html "trait core::any::Any"){.trait} for T {#implt-any-for-t .code-header}

::: where
where T: \'static +
?[Sized](https://doc.rust-lang.org/1.86.0/core/marker/trait.Sized.html "trait core::marker::Sized"){.trait},
:::
::::

::::: impl-items
::: {#method.type_id .section .method .trait-impl}
[Source](https://doc.rust-lang.org/1.86.0/src/core/any.rs.html#139){.src
.rightside}[§](#method.type_id){.anchor}

#### fn [type_id](https://doc.rust-lang.org/1.86.0/core/any/trait.Any.html#tymethod.type_id){.fn}(&self) -\> [TypeId](https://doc.rust-lang.org/1.86.0/core/any/struct.TypeId.html "struct core::any::TypeId"){.struct} {#fn-type_idself---typeid .code-header}
:::

::: docblock
Gets the `TypeId` of `self`. [Read
more](https://doc.rust-lang.org/1.86.0/core/any/trait.Any.html#tymethod.type_id)
:::
:::::

:::: {#impl-Borrow%3CT%3E-for-T .section .impl}
[Source](https://doc.rust-lang.org/1.86.0/src/core/borrow.rs.html#209){.src
.rightside}[§](#impl-Borrow%3CT%3E-for-T){.anchor}

### impl\<T\> [Borrow](https://doc.rust-lang.org/1.86.0/core/borrow/trait.Borrow.html "trait core::borrow::Borrow"){.trait}\<T\> for T {#implt-borrowt-for-t .code-header}

::: where
where T:
?[Sized](https://doc.rust-lang.org/1.86.0/core/marker/trait.Sized.html "trait core::marker::Sized"){.trait},
:::
::::

::::: impl-items
::: {#method.borrow .section .method .trait-impl}
[Source](https://doc.rust-lang.org/1.86.0/src/core/borrow.rs.html#211){.src
.rightside}[§](#method.borrow){.anchor}

#### fn [borrow](https://doc.rust-lang.org/1.86.0/core/borrow/trait.Borrow.html#tymethod.borrow){.fn}(&self) -\> [&T](https://doc.rust-lang.org/1.86.0/std/primitive.reference.html){.primitive} {#fn-borrowself---t .code-header}
:::

::: docblock
Immutably borrows from an owned value. [Read
more](https://doc.rust-lang.org/1.86.0/core/borrow/trait.Borrow.html#tymethod.borrow)
:::
:::::

:::: {#impl-BorrowMut%3CT%3E-for-T .section .impl}
[Source](https://doc.rust-lang.org/1.86.0/src/core/borrow.rs.html#217){.src
.rightside}[§](#impl-BorrowMut%3CT%3E-for-T){.anchor}

### impl\<T\> [BorrowMut](https://doc.rust-lang.org/1.86.0/core/borrow/trait.BorrowMut.html "trait core::borrow::BorrowMut"){.trait}\<T\> for T {#implt-borrowmutt-for-t .code-header}

::: where
where T:
?[Sized](https://doc.rust-lang.org/1.86.0/core/marker/trait.Sized.html "trait core::marker::Sized"){.trait},
:::
::::

::::: impl-items
::: {#method.borrow_mut .section .method .trait-impl}
[Source](https://doc.rust-lang.org/1.86.0/src/core/borrow.rs.html#218){.src
.rightside}[§](#method.borrow_mut){.anchor}

#### fn [borrow_mut](https://doc.rust-lang.org/1.86.0/core/borrow/trait.BorrowMut.html#tymethod.borrow_mut){.fn}(&mut self) -\> [&mut T](https://doc.rust-lang.org/1.86.0/std/primitive.reference.html){.primitive} {#fn-borrow_mutmut-self---mut-t .code-header}
:::

::: docblock
Mutably borrows from an owned value. [Read
more](https://doc.rust-lang.org/1.86.0/core/borrow/trait.BorrowMut.html#tymethod.borrow_mut)
:::
:::::

:::: {#impl-CloneToUninit-for-T .section .impl}
[Source](https://doc.rust-lang.org/1.86.0/src/core/clone.rs.html#273){.src
.rightside}[§](#impl-CloneToUninit-for-T){.anchor}

### impl\<T\> [CloneToUninit](https://doc.rust-lang.org/1.86.0/core/clone/trait.CloneToUninit.html "trait core::clone::CloneToUninit"){.trait} for T {#implt-clonetouninit-for-t .code-header}

::: where
where T:
[Clone](https://doc.rust-lang.org/1.86.0/core/clone/trait.Clone.html "trait core::clone::Clone"){.trait},
:::
::::

:::::: impl-items
::: {#method.clone_to_uninit .section .method .trait-impl}
[Source](https://doc.rust-lang.org/1.86.0/src/core/clone.rs.html#275){.src
.rightside}[§](#method.clone_to_uninit){.anchor}

#### unsafe fn [clone_to_uninit](https://doc.rust-lang.org/1.86.0/core/clone/trait.CloneToUninit.html#tymethod.clone_to_uninit){.fn}(&self, dst: [\*mut](https://doc.rust-lang.org/1.86.0/std/primitive.pointer.html){.primitive} [u8](https://doc.rust-lang.org/1.86.0/std/primitive.u8.html){.primitive}) {#unsafe-fn-clone_to_uninitself-dst-mut-u8 .code-header}
:::

[]{.item-info}

::: {.stab .unstable}
🔬This is a nightly-only experimental API. (`clone_to_uninit`)
:::

::: docblock
Performs copy-assignment from `self` to `dst`. [Read
more](https://doc.rust-lang.org/1.86.0/core/clone/trait.CloneToUninit.html#tymethod.clone_to_uninit)
:::
::::::

::: {#impl-From%3CT%3E-for-T .section .impl}
[Source](https://doc.rust-lang.org/1.86.0/src/core/convert/mod.rs.html#767){.src
.rightside}[§](#impl-From%3CT%3E-for-T){.anchor}

### impl\<T\> [From](https://doc.rust-lang.org/1.86.0/core/convert/trait.From.html "trait core::convert::From"){.trait}\<T\> for T {#implt-fromt-for-t .code-header}
:::

::::: impl-items
::: {#method.from .section .method .trait-impl}
[Source](https://doc.rust-lang.org/1.86.0/src/core/convert/mod.rs.html#770){.src
.rightside}[§](#method.from){.anchor}

#### fn [from](https://doc.rust-lang.org/1.86.0/core/convert/trait.From.html#tymethod.from){.fn}(t: T) -\> T {#fn-fromt-t---t .code-header}
:::

::: docblock
Returns the argument unchanged.
:::
:::::

::: {#impl-Instrument-for-T .section .impl}
[§](#impl-Instrument-for-T){.anchor}

### impl\<T\> Instrument for T {#implt-instrument-for-t .code-header}
:::

::::::: impl-items
::: {#method.instrument .section .method .trait-impl}
[§](#method.instrument){.anchor}

#### fn [instrument]{.fn}(self, span: Span) -\> Instrumented\<Self\> {#fn-instrumentself-span-span---instrumentedself .code-header}
:::

::: docblock
Instruments this type with the provided \[`Span`\], returning an
`Instrumented` wrapper. Read more
:::

::: {#method.in_current_span .section .method .trait-impl}
[§](#method.in_current_span){.anchor}

#### fn [in_current_span]{.fn}(self) -\> Instrumented\<Self\> {#fn-in_current_spanself---instrumentedself .code-header}
:::

::: docblock
Instruments this type with the [current](super::Span::current())
[`Span`](crate::Span), returning an `Instrumented` wrapper. Read more
:::
:::::::

:::: {#impl-Into%3CU%3E-for-T .section .impl}
[Source](https://doc.rust-lang.org/1.86.0/src/core/convert/mod.rs.html#750-752){.src
.rightside}[§](#impl-Into%3CU%3E-for-T){.anchor}

### impl\<T, U\> [Into](https://doc.rust-lang.org/1.86.0/core/convert/trait.Into.html "trait core::convert::Into"){.trait}\<U\> for T {#implt-u-intou-for-t .code-header}

::: where
where U:
[From](https://doc.rust-lang.org/1.86.0/core/convert/trait.From.html "trait core::convert::From"){.trait}\<T\>,
:::
::::

::::: impl-items
::: {#method.into .section .method .trait-impl}
[Source](https://doc.rust-lang.org/1.86.0/src/core/convert/mod.rs.html#760){.src
.rightside}[§](#method.into){.anchor}

#### fn [into](https://doc.rust-lang.org/1.86.0/core/convert/trait.Into.html#tymethod.into){.fn}(self) -\> U {#fn-intoself---u .code-header}
:::

::: docblock
Calls `U::from(self)`.

That is, this conversion is whatever the implementation of
[`From`](https://doc.rust-lang.org/1.86.0/core/convert/trait.From.html "trait core::convert::From")`<T> for U`
chooses to do.
:::
:::::

::: {#impl-IntoEither-for-T .section .impl}
[Source](https://docs.rs/either/1/src/either/into_either.rs.html#64){.src
.rightside}[§](#impl-IntoEither-for-T){.anchor}

### impl\<T\> [IntoEither](https://docs.rs/either/1/either/into_either/trait.IntoEither.html "trait either::into_either::IntoEither"){.trait} for T {#implt-intoeither-for-t .code-header}
:::

:::::::: impl-items
::: {#method.into_either .section .method .trait-impl}
[Source](https://docs.rs/either/1/src/either/into_either.rs.html#29){.src
.rightside}[§](#method.into_either){.anchor}

#### fn [into_either](https://docs.rs/either/1/either/into_either/trait.IntoEither.html#method.into_either){.fn}(self, into_left: [bool](https://doc.rust-lang.org/1.86.0/std/primitive.bool.html){.primitive}) -\> [Either](https://docs.rs/either/1/either/enum.Either.html "enum either::Either"){.enum}\<Self, Self\> {#fn-into_eitherself-into_left-bool---eitherself-self .code-header}
:::

::: docblock
Converts `self` into a
[`Left`](https://docs.rs/either/1/either/enum.Either.html#variant.Left "variant either::Either::Left")
variant of
[`Either<Self, Self>`](https://docs.rs/either/1/either/enum.Either.html "enum either::Either")
if `into_left` is `true`. Converts `self` into a
[`Right`](https://docs.rs/either/1/either/enum.Either.html#variant.Right "variant either::Either::Right")
variant of
[`Either<Self, Self>`](https://docs.rs/either/1/either/enum.Either.html "enum either::Either")
otherwise. [Read
more](https://docs.rs/either/1/either/into_either/trait.IntoEither.html#method.into_either)
:::

:::: {#method.into_either_with .section .method .trait-impl}
[Source](https://docs.rs/either/1/src/either/into_either.rs.html#55-57){.src
.rightside}[§](#method.into_either_with){.anchor}

#### fn [into_either_with](https://docs.rs/either/1/either/into_either/trait.IntoEither.html#method.into_either_with){.fn}\<F\>(self, into_left: F) -\> [Either](https://docs.rs/either/1/either/enum.Either.html "enum either::Either"){.enum}\<Self, Self\> {#fn-into_either_withfself-into_left-f---eitherself-self .code-header}

::: where
where F:
[FnOnce](https://doc.rust-lang.org/1.86.0/core/ops/function/trait.FnOnce.html "trait core::ops::function::FnOnce"){.trait}(&Self)
-\>
[bool](https://doc.rust-lang.org/1.86.0/std/primitive.bool.html){.primitive},
:::
::::

::: docblock
Converts `self` into a
[`Left`](https://docs.rs/either/1/either/enum.Either.html#variant.Left "variant either::Either::Left")
variant of
[`Either<Self, Self>`](https://docs.rs/either/1/either/enum.Either.html "enum either::Either")
if `into_left(&self)` returns `true`. Converts `self` into a
[`Right`](https://docs.rs/either/1/either/enum.Either.html#variant.Right "variant either::Either::Right")
variant of
[`Either<Self, Self>`](https://docs.rs/either/1/either/enum.Either.html "enum either::Either")
otherwise. [Read
more](https://docs.rs/either/1/either/into_either/trait.IntoEither.html#method.into_either_with)
:::
::::::::

::: {#impl-Pointable-for-T .section .impl}
[§](#impl-Pointable-for-T){.anchor}

### impl\<T\> Pointable for T {#implt-pointable-for-t .code-header}
:::

::::::::::::::: impl-items
::: {#associatedconstant.ALIGN .section .associatedconstant .trait-impl}
[§](#associatedconstant.ALIGN){.anchor}

#### const [ALIGN]{.constant}: [usize](https://doc.rust-lang.org/1.86.0/std/primitive.usize.html){.primitive} {#const-align-usize .code-header}
:::

::: docblock
The alignment of pointer.
:::

::: {#associatedtype.Init .section .associatedtype .trait-impl}
[§](#associatedtype.Init){.anchor}

#### type [Init]{.associatedtype} = T {#type-init-t .code-header}
:::

::: docblock
The type for initializers.
:::

::: {#method.init .section .method .trait-impl}
[§](#method.init){.anchor}

#### unsafe fn [init]{.fn}(init: \<T as Pointable\>::Init) -\> [usize](https://doc.rust-lang.org/1.86.0/std/primitive.usize.html){.primitive} {#unsafe-fn-initinit-t-as-pointableinit---usize .code-header}
:::

::: docblock
Initializes a with the given initializer. Read more
:::

::: {#method.deref .section .method .trait-impl}
[§](#method.deref){.anchor}

#### unsafe fn [deref]{.fn}\<\'a\>(ptr: [usize](https://doc.rust-lang.org/1.86.0/std/primitive.usize.html){.primitive}) -\> [&\'a T](https://doc.rust-lang.org/1.86.0/std/primitive.reference.html){.primitive} {#unsafe-fn-derefaptr-usize---a-t .code-header}
:::

::: docblock
Dereferences the given pointer. Read more
:::

::: {#method.deref_mut .section .method .trait-impl}
[§](#method.deref_mut){.anchor}

#### unsafe fn [deref_mut]{.fn}\<\'a\>(ptr: [usize](https://doc.rust-lang.org/1.86.0/std/primitive.usize.html){.primitive}) -\> [&\'a mut T](https://doc.rust-lang.org/1.86.0/std/primitive.reference.html){.primitive} {#unsafe-fn-deref_mutaptr-usize---a-mut-t .code-header}
:::

::: docblock
Mutably dereferences the given pointer. Read more
:::

::: {#method.drop .section .method .trait-impl}
[§](#method.drop){.anchor}

#### unsafe fn [drop]{.fn}(ptr: [usize](https://doc.rust-lang.org/1.86.0/std/primitive.usize.html){.primitive}) {#unsafe-fn-dropptr-usize .code-header}
:::

::: docblock
Drops the object pointed to by the given pointer. Read more
:::
:::::::::::::::

::: {#impl-Same-for-T .section .impl}
[Source](https://docs.rs/typenum/1.18.0/src/typenum/type_operators.rs.html#34){.src
.rightside}[§](#impl-Same-for-T){.anchor}

### impl\<T\> [Same](https://docs.rs/typenum/1.18.0/typenum/type_operators/trait.Same.html "trait typenum::type_operators::Same"){.trait} for T {#implt-same-for-t .code-header}
:::

::::: impl-items
::: {#associatedtype.Output .section .associatedtype .trait-impl}
[Source](https://docs.rs/typenum/1.18.0/src/typenum/type_operators.rs.html#35){.src
.rightside}[§](#associatedtype.Output){.anchor}

#### type [Output](https://docs.rs/typenum/1.18.0/typenum/type_operators/trait.Same.html#associatedtype.Output){.associatedtype} = T {#type-output-t .code-header}
:::

::: docblock
Should always be `Self`
:::
:::::

:::: {#impl-SupersetOf%3CSS%3E-for-SP .section .impl}
[§](#impl-SupersetOf%3CSS%3E-for-SP){.anchor}

### impl\<SS, SP\> SupersetOf\<SS\> for SP {#implss-sp-supersetofss-for-sp .code-header}

::: where
where SS: SubsetOf\<SP\>,
:::
::::

::::::::::: impl-items
::: {#method.to_subset .section .method .trait-impl}
[§](#method.to_subset){.anchor}

#### fn [to_subset]{.fn}(&self) -\> [Option](https://doc.rust-lang.org/1.86.0/core/option/enum.Option.html "enum core::option::Option"){.enum}\<SS\> {#fn-to_subsetself---optionss .code-header}
:::

::: docblock
The inverse inclusion map: attempts to construct `self` from the
equivalent element of its superset. Read more
:::

::: {#method.is_in_subset .section .method .trait-impl}
[§](#method.is_in_subset){.anchor}

#### fn [is_in_subset]{.fn}(&self) -\> [bool](https://doc.rust-lang.org/1.86.0/std/primitive.bool.html){.primitive} {#fn-is_in_subsetself---bool .code-header}
:::

::: docblock
Checks if `self` is actually part of its subset `T` (and can be
converted to it).
:::

::: {#method.to_subset_unchecked .section .method .trait-impl}
[§](#method.to_subset_unchecked){.anchor}

#### fn [to_subset_unchecked]{.fn}(&self) -\> SS {#fn-to_subset_uncheckedself---ss .code-header}
:::

::: docblock
Use with care! Same as `self.to_subset` but without any property checks.
Always succeeds.
:::

::: {#method.from_subset .section .method .trait-impl}
[§](#method.from_subset){.anchor}

#### fn [from_subset]{.fn}(element: [&SS](https://doc.rust-lang.org/1.86.0/std/primitive.reference.html){.primitive}) -\> SP {#fn-from_subsetelement-ss---sp .code-header}
:::

::: docblock
The inclusion map: converts `self` to the equivalent element of its
superset.
:::
:::::::::::

:::: {#impl-ToOwned-for-T .section .impl}
[Source](https://doc.rust-lang.org/1.86.0/src/alloc/borrow.rs.html#82-84){.src
.rightside}[§](#impl-ToOwned-for-T){.anchor}

### impl\<T\> [ToOwned](https://doc.rust-lang.org/1.86.0/alloc/borrow/trait.ToOwned.html "trait alloc::borrow::ToOwned"){.trait} for T {#implt-toowned-for-t .code-header}

::: where
where T:
[Clone](https://doc.rust-lang.org/1.86.0/core/clone/trait.Clone.html "trait core::clone::Clone"){.trait},
:::
::::

::::::::: impl-items
::: {#associatedtype.Owned .section .associatedtype .trait-impl}
[Source](https://doc.rust-lang.org/1.86.0/src/alloc/borrow.rs.html#86){.src
.rightside}[§](#associatedtype.Owned){.anchor}

#### type [Owned](https://doc.rust-lang.org/1.86.0/alloc/borrow/trait.ToOwned.html#associatedtype.Owned){.associatedtype} = T {#type-owned-t .code-header}
:::

::: docblock
The resulting type after obtaining ownership.
:::

::: {#method.to_owned .section .method .trait-impl}
[Source](https://doc.rust-lang.org/1.86.0/src/alloc/borrow.rs.html#87){.src
.rightside}[§](#method.to_owned){.anchor}

#### fn [to_owned](https://doc.rust-lang.org/1.86.0/alloc/borrow/trait.ToOwned.html#tymethod.to_owned){.fn}(&self) -\> T {#fn-to_ownedself---t .code-header}
:::

::: docblock
Creates owned data from borrowed data, usually by cloning. [Read
more](https://doc.rust-lang.org/1.86.0/alloc/borrow/trait.ToOwned.html#tymethod.to_owned)
:::

::: {#method.clone_into .section .method .trait-impl}
[Source](https://doc.rust-lang.org/1.86.0/src/alloc/borrow.rs.html#91){.src
.rightside}[§](#method.clone_into){.anchor}

#### fn [clone_into](https://doc.rust-lang.org/1.86.0/alloc/borrow/trait.ToOwned.html#method.clone_into){.fn}(&self, target: [&mut T](https://doc.rust-lang.org/1.86.0/std/primitive.reference.html){.primitive}) {#fn-clone_intoself-target-mut-t .code-header}
:::

::: docblock
Uses borrowed data to replace owned data, usually by cloning. [Read
more](https://doc.rust-lang.org/1.86.0/alloc/borrow/trait.ToOwned.html#method.clone_into)
:::
:::::::::

:::: {#impl-ToString-for-T .section .impl}
[Source](https://doc.rust-lang.org/1.86.0/src/alloc/string.rs.html#2758){.src
.rightside}[§](#impl-ToString-for-T){.anchor}

### impl\<T\> [ToString](https://doc.rust-lang.org/1.86.0/alloc/string/trait.ToString.html "trait alloc::string::ToString"){.trait} for T {#implt-tostring-for-t .code-header}

::: where
where T:
[Display](https://doc.rust-lang.org/1.86.0/core/fmt/trait.Display.html "trait core::fmt::Display"){.trait} +
?[Sized](https://doc.rust-lang.org/1.86.0/core/marker/trait.Sized.html "trait core::marker::Sized"){.trait},
:::
::::

::::: impl-items
::: {#method.to_string .section .method .trait-impl}
[Source](https://doc.rust-lang.org/1.86.0/src/alloc/string.rs.html#2760){.src
.rightside}[§](#method.to_string){.anchor}

#### fn [to_string](https://doc.rust-lang.org/1.86.0/alloc/string/trait.ToString.html#tymethod.to_string){.fn}(&self) -\> [String](https://doc.rust-lang.org/1.86.0/alloc/string/struct.String.html "struct alloc::string::String"){.struct} {#fn-to_stringself---string .code-header}
:::

::: docblock
Converts the given value to a `String`. [Read
more](https://doc.rust-lang.org/1.86.0/alloc/string/trait.ToString.html#tymethod.to_string)
:::
:::::

:::: {#impl-TryFrom%3CU%3E-for-T .section .impl}
[Source](https://doc.rust-lang.org/1.86.0/src/core/convert/mod.rs.html#807-809){.src
.rightside}[§](#impl-TryFrom%3CU%3E-for-T){.anchor}

### impl\<T, U\> [TryFrom](https://doc.rust-lang.org/1.86.0/core/convert/trait.TryFrom.html "trait core::convert::TryFrom"){.trait}\<U\> for T {#implt-u-tryfromu-for-t .code-header}

::: where
where U:
[Into](https://doc.rust-lang.org/1.86.0/core/convert/trait.Into.html "trait core::convert::Into"){.trait}\<T\>,
:::
::::

::::::: impl-items
::: {#associatedtype.Error-1 .section .associatedtype .trait-impl}
[Source](https://doc.rust-lang.org/1.86.0/src/core/convert/mod.rs.html#811){.src
.rightside}[§](#associatedtype.Error-1){.anchor}

#### type [Error](https://doc.rust-lang.org/1.86.0/core/convert/trait.TryFrom.html#associatedtype.Error){.associatedtype} = [Infallible](https://doc.rust-lang.org/1.86.0/core/convert/enum.Infallible.html "enum core::convert::Infallible"){.enum} {#type-error-infallible .code-header}
:::

::: docblock
The type returned in the event of a conversion error.
:::

::: {#method.try_from .section .method .trait-impl}
[Source](https://doc.rust-lang.org/1.86.0/src/core/convert/mod.rs.html#814){.src
.rightside}[§](#method.try_from){.anchor}

#### fn [try_from](https://doc.rust-lang.org/1.86.0/core/convert/trait.TryFrom.html#tymethod.try_from){.fn}(value: U) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<T, \<T as [TryFrom](https://doc.rust-lang.org/1.86.0/core/convert/trait.TryFrom.html "trait core::convert::TryFrom"){.trait}\<U\>\>::[Error](https://doc.rust-lang.org/1.86.0/core/convert/trait.TryFrom.html#associatedtype.Error "type core::convert::TryFrom::Error"){.associatedtype}\> {#fn-try_fromvalue-u---resultt-t-as-tryfromuerror .code-header}
:::

::: docblock
Performs the conversion.
:::
:::::::

:::: {#impl-TryInto%3CU%3E-for-T .section .impl}
[Source](https://doc.rust-lang.org/1.86.0/src/core/convert/mod.rs.html#792-794){.src
.rightside}[§](#impl-TryInto%3CU%3E-for-T){.anchor}

### impl\<T, U\> [TryInto](https://doc.rust-lang.org/1.86.0/core/convert/trait.TryInto.html "trait core::convert::TryInto"){.trait}\<U\> for T {#implt-u-tryintou-for-t .code-header}

::: where
where U:
[TryFrom](https://doc.rust-lang.org/1.86.0/core/convert/trait.TryFrom.html "trait core::convert::TryFrom"){.trait}\<T\>,
:::
::::

::::::: impl-items
::: {#associatedtype.Error .section .associatedtype .trait-impl}
[Source](https://doc.rust-lang.org/1.86.0/src/core/convert/mod.rs.html#796){.src
.rightside}[§](#associatedtype.Error){.anchor}

#### type [Error](https://doc.rust-lang.org/1.86.0/core/convert/trait.TryInto.html#associatedtype.Error){.associatedtype} = \<U as [TryFrom](https://doc.rust-lang.org/1.86.0/core/convert/trait.TryFrom.html "trait core::convert::TryFrom"){.trait}\<T\>\>::[Error](https://doc.rust-lang.org/1.86.0/core/convert/trait.TryFrom.html#associatedtype.Error "type core::convert::TryFrom::Error"){.associatedtype} {#type-error-u-as-tryfromterror .code-header}
:::

::: docblock
The type returned in the event of a conversion error.
:::

::: {#method.try_into .section .method .trait-impl}
[Source](https://doc.rust-lang.org/1.86.0/src/core/convert/mod.rs.html#799){.src
.rightside}[§](#method.try_into){.anchor}

#### fn [try_into](https://doc.rust-lang.org/1.86.0/core/convert/trait.TryInto.html#tymethod.try_into){.fn}(self) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<U, \<U as [TryFrom](https://doc.rust-lang.org/1.86.0/core/convert/trait.TryFrom.html "trait core::convert::TryFrom"){.trait}\<T\>\>::[Error](https://doc.rust-lang.org/1.86.0/core/convert/trait.TryFrom.html#associatedtype.Error "type core::convert::TryFrom::Error"){.associatedtype}\> {#fn-try_intoself---resultu-u-as-tryfromterror .code-header}
:::

::: docblock
Performs the conversion.
:::
:::::::

:::: {#impl-VZip%3CV%3E-for-T .section .impl}
[§](#impl-VZip%3CV%3E-for-T){.anchor}

### impl\<V, T\> VZip\<V\> for T {#implv-t-vzipv-for-t .code-header}

::: where
where V: MultiLane\<T\>,
:::
::::

:::: impl-items
::: {#method.vzip .section .method .trait-impl}
[§](#method.vzip){.anchor}

#### fn [vzip]{.fn}(self) -\> V {#fn-vzipself---v .code-header}
:::
::::

::: {#impl-WithSubscriber-for-T .section .impl}
[§](#impl-WithSubscriber-for-T){.anchor}

### impl\<T\> WithSubscriber for T {#implt-withsubscriber-for-t .code-header}
:::

:::::::: impl-items
:::: {#method.with_subscriber .section .method .trait-impl}
[§](#method.with_subscriber){.anchor}

#### fn [with_subscriber]{.fn}\<S\>(self, subscriber: S) -\> WithDispatch\<Self\> {#fn-with_subscribersself-subscriber-s---withdispatchself .code-header}

::: where
where S:
[Into](https://doc.rust-lang.org/1.86.0/core/convert/trait.Into.html "trait core::convert::Into"){.trait}\<Dispatch\>,
:::
::::

::: docblock
Attaches the provided [`Subscriber`](super::Subscriber) to this type,
returning a \[`WithDispatch`\] wrapper. Read more
:::

::: {#method.with_current_subscriber .section .method .trait-impl}
[§](#method.with_current_subscriber){.anchor}

#### fn [with_current_subscriber]{.fn}(self) -\> WithDispatch\<Self\> {#fn-with_current_subscriberself---withdispatchself .code-header}
:::

::: docblock
Attaches the current
[default](crate::dispatcher#setting-the-default-subscriber)
[`Subscriber`](super::Subscriber) to this type, returning a
\[`WithDispatch`\] wrapper. Read more
:::
::::::::

:::: {#impl-DeserializeOwned-for-T .section .impl}
[Source](https://docs.rs/serde/1.0.219/src/serde/de/mod.rs.html#614){.src
.rightside}[§](#impl-DeserializeOwned-for-T){.anchor}

### impl\<T\> [DeserializeOwned](https://docs.rs/serde/1.0.219/serde/de/trait.DeserializeOwned.html "trait serde::de::DeserializeOwned"){.trait} for T {#implt-deserializeowned-for-t .code-header}

::: where
where T: for\<\'de\>
[Deserialize](https://docs.rs/serde/1.0.219/serde/de/trait.Deserialize.html "trait serde::de::Deserialize"){.trait}\<\'de\>,
:::
::::

:::: {#impl-Scalar-for-T .section .impl}
[Source](https://docs.rs/nalgebra/0.25.0/src/nalgebra/base/scalar.rs.html#8){.src
.rightside}[§](#impl-Scalar-for-T){.anchor}

### impl\<T\> [Scalar](https://docs.rs/nalgebra/0.25.0/nalgebra/base/scalar/trait.Scalar.html "trait nalgebra::base::scalar::Scalar"){.trait} for T {#implt-scalar-for-t .code-header}

::: where
where T: \'static +
[Clone](https://doc.rust-lang.org/1.86.0/core/clone/trait.Clone.html "trait core::clone::Clone"){.trait} +
[PartialEq](https://doc.rust-lang.org/1.86.0/core/cmp/trait.PartialEq.html "trait core::cmp::PartialEq"){.trait} +
[Debug](https://doc.rust-lang.org/1.86.0/core/fmt/trait.Debug.html "trait core::fmt::Debug"){.trait},
:::
::::
::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
:::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
