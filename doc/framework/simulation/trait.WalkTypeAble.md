:::::::::::::::::::::::::::::::::::::: width-limiter
::::::::::::::::::::::::::::::::::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../index.html)::[simulation](index.html)
:::

# Trait [WalkTypeAble]{.trait}Copy item path

[[Source](../../src/optionstratlib/simulation/traits.rs.html#37-483){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub trait WalkTypeAble<X, Y>where
    X: Copy + Into<Positive> + AddAssign + Display,
    Y: Into<Positive> + Display + Clone,{
    // Provided methods
    fn brownian(
        &self,
        params: &WalkParams<X, Y>,
    ) -> Result<Vec<Positive>, Box<dyn Error>> { ... }
    fn geometric_brownian(
        &self,
        params: &WalkParams<X, Y>,
    ) -> Result<Vec<Positive>, Box<dyn Error>> { ... }
    fn log_returns(
        &self,
        params: &WalkParams<X, Y>,
    ) -> Result<Vec<Positive>, Box<dyn Error>> { ... }
    fn mean_reverting(
        &self,
        params: &WalkParams<X, Y>,
    ) -> Result<Vec<Positive>, Box<dyn Error>> { ... }
    fn jump_diffusion(
        &self,
        params: &WalkParams<X, Y>,
    ) -> Result<Vec<Positive>, Box<dyn Error>> { ... }
    fn garch(
        &self,
        params: &WalkParams<X, Y>,
    ) -> Result<Vec<Positive>, Box<dyn Error>> { ... }
    fn heston(
        &self,
        params: &WalkParams<X, Y>,
    ) -> Result<Vec<Positive>, Box<dyn Error>> { ... }
    fn custom(
        &self,
        params: &WalkParams<X, Y>,
    ) -> Result<Vec<Positive>, Box<dyn Error>> { ... }
    fn historical(
        &self,
        params: &WalkParams<X, Y>,
    ) -> Result<Vec<Positive>, Box<dyn Error>> { ... }
}
```

Expand description

::: docblock
Trait for implementing various random walk models and stochastic
processes.

This trait provides methods to generate different types of stochastic
processes commonly used in financial modeling, time series analysis, and
simulation studies. Each method implements a specific type of random
walk based on the parameters provided.

The trait is generic over types `X` and `Y`, which represent the x-axis
(typically time) and y-axis (typically price or value) components
respectively.

## [§](#type-parameters){.doc-anchor}Type Parameters

- `X` - The type for the x-axis values (typically time), must be `Copy`,
  convertible to `Positive`, implement `AddAssign`, and implement
  `Display`.
- `Y` - The type for the y-axis values (typically price), must be
  `Copy`, convertible to `Positive`, and implement `Display`.

## [§](#methods){.doc-anchor}Methods

The trait provides methods for generating the following stochastic
processes:

- Brownian motion (standard random walk)
- Geometric Brownian motion
- Log returns process with optional autocorrelation
- Mean reverting (Ornstein-Uhlenbeck) process
- Jump diffusion process
- GARCH (Generalized Autoregressive Conditional Heteroskedasticity)
- Heston stochastic volatility model
- Custom stochastic process with mean-reverting volatility
:::

## Provided Methods[§](#provided-methods){.anchor} {#provided-methods .section-header}

::::::::::::::::::::: methods
::: {#method.brownian .section .method}
[Source](../../src/optionstratlib/simulation/traits.rs.html#55-73){.src
.rightside}

#### fn [brownian](#method.brownian){.fn}( &self, params: &[WalkParams](struct.WalkParams.html "struct optionstratlib::simulation::WalkParams"){.struct}\<X, Y\>, ) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Vec](https://doc.rust-lang.org/1.86.0/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<[Positive](../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\>, [Box](https://doc.rust-lang.org/1.86.0/alloc/boxed/struct.Box.html "struct alloc::boxed::Box"){.struct}\<dyn [Error](https://doc.rust-lang.org/1.86.0/core/error/trait.Error.html "trait core::error::Error"){.trait}\>\> {#fn-brownian-self-params-walkparamsx-y---resultvecpositive-boxdyn-error .code-header}
:::

::: docblock
Generates a Brownian motion (standard random walk) process.

Brownian motion is a continuous-time stochastic process where changes
are normally distributed with a drift term and volatility.

##### [§](#parameters){.doc-anchor}Parameters

- `params` - Walk parameters including initial value, time step, drift,
  and volatility.

##### [§](#returns){.doc-anchor}Returns

- `Result<Vec<Positive>, Box<dyn Error>>` - A vector of positive values
  representing the generated Brownian motion path, or an error if
  parameters are invalid.
:::

::: {#method.geometric_brownian .section .method}
[Source](../../src/optionstratlib/simulation/traits.rs.html#89-110){.src
.rightside}

#### fn [geometric_brownian](#method.geometric_brownian){.fn}( &self, params: &[WalkParams](struct.WalkParams.html "struct optionstratlib::simulation::WalkParams"){.struct}\<X, Y\>, ) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Vec](https://doc.rust-lang.org/1.86.0/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<[Positive](../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\>, [Box](https://doc.rust-lang.org/1.86.0/alloc/boxed/struct.Box.html "struct alloc::boxed::Box"){.struct}\<dyn [Error](https://doc.rust-lang.org/1.86.0/core/error/trait.Error.html "trait core::error::Error"){.trait}\>\> {#fn-geometric_brownian-self-params-walkparamsx-y---resultvecpositive-boxdyn-error .code-header}
:::

::: docblock
Generates a Geometric Brownian motion process.

Geometric Brownian motion is a continuous-time stochastic process where
the logarithm of the randomly varying quantity follows Brownian motion.
It's commonly used to model stock prices in the Black-Scholes options
pricing model.

##### [§](#parameters-1){.doc-anchor}Parameters

- `params` - Walk parameters including initial value, time step, drift,
  and volatility.

##### [§](#returns-1){.doc-anchor}Returns

- `Result<Vec<Positive>, Box<dyn Error>>` - A vector of positive values
  representing the generated Geometric Brownian motion path, or an error
  if parameters are invalid.
:::

::: {#method.log_returns .section .method}
[Source](../../src/optionstratlib/simulation/traits.rs.html#127-153){.src
.rightside}

#### fn [log_returns](#method.log_returns){.fn}( &self, params: &[WalkParams](struct.WalkParams.html "struct optionstratlib::simulation::WalkParams"){.struct}\<X, Y\>, ) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Vec](https://doc.rust-lang.org/1.86.0/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<[Positive](../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\>, [Box](https://doc.rust-lang.org/1.86.0/alloc/boxed/struct.Box.html "struct alloc::boxed::Box"){.struct}\<dyn [Error](https://doc.rust-lang.org/1.86.0/core/error/trait.Error.html "trait core::error::Error"){.trait}\>\> {#fn-log_returns-self-params-walkparamsx-y---resultvecpositive-boxdyn-error .code-header}
:::

::: docblock
Generates a Log Returns process, potentially with autocorrelation.

This process models returns (percentage changes) directly, rather than
absolute values, and can include autocorrelation to capture the tendency
of returns to be influenced by previous returns.

##### [§](#parameters-2){.doc-anchor}Parameters

- `params` - Walk parameters including initial value, time step,
  expected return, volatility, and optional autocorrelation coefficient.

##### [§](#returns-2){.doc-anchor}Returns

- `Result<Vec<Positive>, Box<dyn Error>>` - A vector of positive values
  representing the generated Log Returns path, or an error if parameters
  are invalid.
:::

::: {#method.mean_reverting .section .method}
[Source](../../src/optionstratlib/simulation/traits.rs.html#170-188){.src
.rightside}

#### fn [mean_reverting](#method.mean_reverting){.fn}( &self, params: &[WalkParams](struct.WalkParams.html "struct optionstratlib::simulation::WalkParams"){.struct}\<X, Y\>, ) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Vec](https://doc.rust-lang.org/1.86.0/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<[Positive](../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\>, [Box](https://doc.rust-lang.org/1.86.0/alloc/boxed/struct.Box.html "struct alloc::boxed::Box"){.struct}\<dyn [Error](https://doc.rust-lang.org/1.86.0/core/error/trait.Error.html "trait core::error::Error"){.trait}\>\> {#fn-mean_reverting-self-params-walkparamsx-y---resultvecpositive-boxdyn-error .code-header}
:::

::: docblock
Generates a Mean Reverting (Ornstein-Uhlenbeck) process.

The Ornstein-Uhlenbeck process models a value that tends to drift toward
a long-term mean, with the strength of the reversion proportional to the
distance from the mean. It's commonly used for interest rates,
volatility, and other mean-reverting financial variables.

##### [§](#parameters-3){.doc-anchor}Parameters

- `params` - Walk parameters including initial value, mean level,
  reversion speed, volatility, and time step.

##### [§](#returns-3){.doc-anchor}Returns

- `Result<Vec<Positive>, Box<dyn Error>>` - A vector of positive values
  representing the generated Mean Reverting path, or an error if
  parameters are invalid.
:::

::: {#method.jump_diffusion .section .method}
[Source](../../src/optionstratlib/simulation/traits.rs.html#205-230){.src
.rightside}

#### fn [jump_diffusion](#method.jump_diffusion){.fn}( &self, params: &[WalkParams](struct.WalkParams.html "struct optionstratlib::simulation::WalkParams"){.struct}\<X, Y\>, ) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Vec](https://doc.rust-lang.org/1.86.0/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<[Positive](../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\>, [Box](https://doc.rust-lang.org/1.86.0/alloc/boxed/struct.Box.html "struct alloc::boxed::Box"){.struct}\<dyn [Error](https://doc.rust-lang.org/1.86.0/core/error/trait.Error.html "trait core::error::Error"){.trait}\>\> {#fn-jump_diffusion-self-params-walkparamsx-y---resultvecpositive-boxdyn-error .code-header}
:::

::: docblock
Generates a Jump Diffusion process.

Jump Diffusion combines continuous Brownian motion with discrete jumps
that occur according to a Poisson process. This model is useful for
capturing sudden market movements like crashes or spikes that standard
Brownian motion cannot adequately model.

##### [§](#parameters-4){.doc-anchor}Parameters

- `params` - Walk parameters including initial value, drift, volatility,
  jump intensity, jump mean size, and jump volatility.

##### [§](#returns-4){.doc-anchor}Returns

- `Result<Vec<Positive>, Box<dyn Error>>` - A vector of positive values
  representing the generated Jump Diffusion path, or an error if
  parameters are invalid.
:::

::: {#method.garch .section .method}
[Source](../../src/optionstratlib/simulation/traits.rs.html#249-298){.src
.rightside}

#### fn [garch](#method.garch){.fn}( &self, params: &[WalkParams](struct.WalkParams.html "struct optionstratlib::simulation::WalkParams"){.struct}\<X, Y\>, ) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Vec](https://doc.rust-lang.org/1.86.0/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<[Positive](../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\>, [Box](https://doc.rust-lang.org/1.86.0/alloc/boxed/struct.Box.html "struct alloc::boxed::Box"){.struct}\<dyn [Error](https://doc.rust-lang.org/1.86.0/core/error/trait.Error.html "trait core::error::Error"){.trait}\>\> {#fn-garch-self-params-walkparamsx-y---resultvecpositive-boxdyn-error .code-header}
:::

::: docblock
Generates a GARCH (Generalized Autoregressive Conditional
Heteroskedasticity) process.

GARCH models time-varying volatility clustering, where periods of high
volatility tend to be followed by high volatility, and low volatility by
low volatility.

##### [§](#parameters-5){.doc-anchor}Parameters

- `params` - Walk parameters for the GARCH process.

##### [§](#returns-5){.doc-anchor}Returns

- `Result<Vec<Positive>, Box<dyn Error>>` - A vector of positive values
  representing the generated GARCH path, or an error if parameters are
  invalid.

##### [§](#note){.doc-anchor}Note

This implementation is currently a placeholder and returns an empty
vector.
:::

::: {#method.heston .section .method}
[Source](../../src/optionstratlib/simulation/traits.rs.html#344-398){.src
.rightside}

#### fn [heston](#method.heston){.fn}( &self, params: &[WalkParams](struct.WalkParams.html "struct optionstratlib::simulation::WalkParams"){.struct}\<X, Y\>, ) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Vec](https://doc.rust-lang.org/1.86.0/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<[Positive](../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\>, [Box](https://doc.rust-lang.org/1.86.0/alloc/boxed/struct.Box.html "struct alloc::boxed::Box"){.struct}\<dyn [Error](https://doc.rust-lang.org/1.86.0/core/error/trait.Error.html "trait core::error::Error"){.trait}\>\> {#fn-heston-self-params-walkparamsx-y---resultvecpositive-boxdyn-error .code-header}
:::

::: docblock
Generates a Heston stochastic volatility model.

The Heston model extends Geometric Brownian Motion by allowing the
volatility itself to be a stochastic process, following a mean-reverting
square-root process.

##### [§](#parameters-6){.doc-anchor}Parameters

- `params` - Walk parameters for the Heston process.

##### [§](#returns-6){.doc-anchor}Returns

- `Result<Vec<Positive>, Box<dyn Error>>` - A vector of positive values
  representing the generated Heston model path, or an error if
  parameters are invalid.

##### [§](#note-1){.doc-anchor}Note

This implementation is currently a placeholder and returns an empty
vector. Generates a Heston stochastic volatility model.

The Heston model extends Geometric Brownian Motion by allowing the
volatility itself to be a stochastic process, following a mean-reverting
square-root process.

##### [§](#parameters-7){.doc-anchor}Parameters

- `params` - Walk parameters for the Heston process, including:
  - `dt`: Time step
  - `drift`: Drift coefficient for the price process
  - `v0`: Initial variance
  - `kappa`: Mean reversion speed for variance
  - `theta`: Long-term variance mean level
  - `xi`: Volatility of variance
  - `rho`: Correlation between price and variance Brownian motions

##### [§](#returns-7){.doc-anchor}Returns

- `Result<Vec<Positive>, Box<dyn Error>>` - A vector of positive values
  representing the generated Heston model path, or an error if
  parameters are invalid.

##### [§](#notes){.doc-anchor}Notes

The Heston model is described by the following SDEs: dS_t = μS_t dt +
√v_t S_t dW\^1_t dv_t = κ(θ - v_t) dt + ξ√v_t dW\^2_t with dW\^1_t
dW\^2_t = ρ dt
:::

::: {#method.custom .section .method}
[Source](../../src/optionstratlib/simulation/traits.rs.html#415-441){.src
.rightside}

#### fn [custom](#method.custom){.fn}( &self, params: &[WalkParams](struct.WalkParams.html "struct optionstratlib::simulation::WalkParams"){.struct}\<X, Y\>, ) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Vec](https://doc.rust-lang.org/1.86.0/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<[Positive](../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\>, [Box](https://doc.rust-lang.org/1.86.0/alloc/boxed/struct.Box.html "struct alloc::boxed::Box"){.struct}\<dyn [Error](https://doc.rust-lang.org/1.86.0/core/error/trait.Error.html "trait core::error::Error"){.trait}\>\> {#fn-custom-self-params-walkparamsx-y---resultvecpositive-boxdyn-error .code-header}
:::

::: docblock
Generates a custom stochastic process with mean-reverting volatility.

This implements a process where the underlying value follows Brownian
motion, but with volatility that follows an Ornstein-Uhlenbeck
(mean-reverting) process. This allows for modeling more complex dynamics
than standard models.

##### [§](#parameters-8){.doc-anchor}Parameters

- `params` - Walk parameters including drift, initial volatility,
  volatility of volatility (vov), volatility mean reversion speed,
  volatility mean level, and time step.

##### [§](#returns-8){.doc-anchor}Returns

- `Result<Vec<Positive>, Box<dyn Error>>` - A vector of positive values
  representing the generated custom process path, or an error if
  parameters are invalid.
:::

::: {#method.historical .section .method}
[Source](../../src/optionstratlib/simulation/traits.rs.html#468-482){.src
.rightside}

#### fn [historical](#method.historical){.fn}( &self, params: &[WalkParams](struct.WalkParams.html "struct optionstratlib::simulation::WalkParams"){.struct}\<X, Y\>, ) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Vec](https://doc.rust-lang.org/1.86.0/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<[Positive](../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\>, [Box](https://doc.rust-lang.org/1.86.0/alloc/boxed/struct.Box.html "struct alloc::boxed::Box"){.struct}\<dyn [Error](https://doc.rust-lang.org/1.86.0/core/error/trait.Error.html "trait core::error::Error"){.trait}\>\> {#fn-historical-self-params-walkparamsx-y---resultvecpositive-boxdyn-error .code-header}
:::

::: docblock
Generates a historical walk based on the given parameters.

This function processes the historical walk by extracting a specified
number of elements from the provided price data (`prices`) based on the
`size` defined in `params`.

##### [§](#parameters-9){.doc-anchor}Parameters

- `self`: Reference to the instance of the object.
- `params`: A reference to `WalkParams<X, Y>` containing the
  configuration details for the walk.
  - Expected to have a `walk_type` of `WalkType::Historical` with
    associated timeframe and price data.
  - `params.size` determines the number of historical prices to include
    in the result.

##### [§](#returns-9){.doc-anchor}Returns

- `Ok(Vec<Positive>)`: A vector containing the first `params.size`
  elements from the given price data (`prices`), if there are at least
  `params.size` elements available.
- `Err(Box<dyn Error>)`: If the `walk_type` is not
  `WalkType::Historical` or if the provided price data does not contain
  enough elements to fulfill the requested size (`params.size`).

##### [§](#errors){.doc-anchor}Errors

- Returns an error if:
  - The `walk_type` in `params` is not `WalkType::Historical`.
  - The `prices` do not contain at least `params.size` elements.
:::
:::::::::::::::::::::

## Trait Implementations[§](#trait-implementations){.anchor} {#trait-implementations .section-header}

::::::::::::: {#trait-implementations-list}
::: {#impl-Clone-for-Box%3Cdyn+WalkTypeAble%3CX,+Y%3E%3E .section .impl}
[Source](../../src/optionstratlib/simulation/traits.rs.html#491-495){.src
.rightside}[§](#impl-Clone-for-Box%3Cdyn+WalkTypeAble%3CX,+Y%3E%3E){.anchor}

### impl\<X, Y\> [Clone](https://doc.rust-lang.org/1.86.0/core/clone/trait.Clone.html "trait core::clone::Clone"){.trait} for [Box](https://doc.rust-lang.org/1.86.0/alloc/boxed/struct.Box.html "struct alloc::boxed::Box"){.struct}\<dyn [WalkTypeAble](trait.WalkTypeAble.html "trait optionstratlib::simulation::WalkTypeAble"){.trait}\<X, Y\>\> {#implx-y-clone-for-boxdyn-walktypeablex-y .code-header}
:::

::::::: impl-items
::: {#method.clone .section .method .trait-impl}
[Source](../../src/optionstratlib/simulation/traits.rs.html#492-494){.src
.rightside}[§](#method.clone){.anchor}

#### fn [clone](https://doc.rust-lang.org/1.86.0/core/clone/trait.Clone.html#tymethod.clone){.fn}(&self) -\> Self {#fn-cloneself---self .code-header}
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

::: {#impl-Debug-for-Box%3Cdyn+WalkTypeAble%3CX,+Y%3E%3E .section .impl}
[Source](../../src/optionstratlib/simulation/traits.rs.html#485-489){.src
.rightside}[§](#impl-Debug-for-Box%3Cdyn+WalkTypeAble%3CX,+Y%3E%3E){.anchor}

### impl\<X, Y\> [Debug](https://doc.rust-lang.org/1.86.0/core/fmt/trait.Debug.html "trait core::fmt::Debug"){.trait} for [Box](https://doc.rust-lang.org/1.86.0/alloc/boxed/struct.Box.html "struct alloc::boxed::Box"){.struct}\<dyn [WalkTypeAble](trait.WalkTypeAble.html "trait optionstratlib::simulation::WalkTypeAble"){.trait}\<X, Y\>\> {#implx-y-debug-for-boxdyn-walktypeablex-y .code-header}
:::

::::: impl-items
::: {#method.fmt .section .method .trait-impl}
[Source](../../src/optionstratlib/simulation/traits.rs.html#486-488){.src
.rightside}[§](#method.fmt){.anchor}

#### fn [fmt](https://doc.rust-lang.org/1.86.0/core/fmt/trait.Debug.html#tymethod.fmt){.fn}(&self, f: &mut [Formatter](https://doc.rust-lang.org/1.86.0/core/fmt/struct.Formatter.html "struct core::fmt::Formatter"){.struct}\<\'\_\>) -\> [Result](https://doc.rust-lang.org/1.86.0/core/fmt/type.Result.html "type core::fmt::Result"){.type} {#fn-fmtself-f-mut-formatter_---result .code-header}
:::

::: docblock
Formats the value using the given formatter. [Read
more](https://doc.rust-lang.org/1.86.0/core/fmt/trait.Debug.html#tymethod.fmt)
:::
:::::
:::::::::::::

## Implementors[§](#implementors){.anchor} {#implementors .section-header}

::: {#implementors-list}
:::
:::::::::::::::::::::::::::::::::::::
::::::::::::::::::::::::::::::::::::::
