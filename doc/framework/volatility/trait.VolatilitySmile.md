::::::::::::: width-limiter
:::::::::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../index.html)::[volatility](index.html)
:::

# Trait [VolatilitySmile]{.trait} Copy item path

[[Source](../../src/optionstratlib/volatility/traits.rs.html#70-82){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub trait VolatilitySmile {
    // Required method
    fn smile(&self) -> Curve;
}
```

Expand description

:::: docblock
A trait defining a volatility smile representation.

The `VolatilitySmile` trait is designed to encapsulate the concept of a
volatility smile, a key phenomenon in derivatives pricing and financial
modeling. A volatility smile occurs when implied volatility varies as a
function of strike price, often depicted as a curved graph resembling a
smile. This trait establishes the foundation for representing and
retrieving these smiles in the form of a mathematical curve.

## [§](#overview){.doc-anchor}Overview

Implementors of this trait are required to provide the `smile` method,
which computes and returns a `Curve` object representing the volatility
smile. The `Curve` struct is a mathematical representation of the smile,
where the x-axis typically corresponds to strike prices (or some other
independent variable), and the y-axis corresponds to implied volatility.

## [§](#usage){.doc-anchor}Usage

This trait serves as the basis for constructing and analyzing volatility
smiles in applications such as:

- Financial derivatives modeling
- Options pricing engines
- Quantitative analysis of market data

## [§](#required-methods-1){.doc-anchor}Required Methods {#required-methods-1}

- **`smile(&self) -> Curve`**
  - Computes and returns the volatility smile as a `Curve`.
  - The returned `Curve` can be used for graphical representation,
    numerical analysis, or further mathematical operations, such as
    interpolation or transformations.

## [§](#integration-with-other-modules){.doc-anchor}Integration with Other Modules

The `VolatilitySmile` trait makes use of the `Curve` struct, defined in
the `crate::curves` module. The `Curve` provides the mathematical
framework necessary for representing and manipulating the smile data.
High-quality precision (via the use of `Decimal` and ordered points)
ensures that the output from the `smile` method is reliable and suitable
for scientific or financial applications.

## [§](#see-also){.doc-anchor}See Also

- [`crate::curves::Curve`](../curves/struct.Curve.html "struct optionstratlib::curves::Curve"):
  The fundamental mathematical representation of the volatility smile.
- [`crate::curves::Point2D`](../curves/struct.Point2D.html "struct optionstratlib::curves::Point2D"):
  The structure representing individual points in the `Curve`.

## [§](#examples){.doc-anchor}Examples

To define a custom volatility model, users can implement this trait and
provide their specific logic for generating a `Curve` corresponding to
the smile.

::: example-wrap
``` {.rust .rust-example-rendered}
use std::collections::BTreeSet;
use rust_decimal::Decimal;
use optionstratlib::curves::Curve;
use optionstratlib::error::greeks::CalculationErrorKind::DecimalError;
use optionstratlib::volatility::VolatilitySmile;

struct MySmile;

impl VolatilitySmile for MySmile {
    fn smile(&self) -> Curve {
        // Custom logic to build and return a Curve representing the smile
        Curve { points: BTreeSet::new(), x_range: (Decimal::ZERO, Decimal::ZERO) }
    }
}
```
:::

This enables integration of user-defined volatility models with the
broader ecosystem of mathematical and financial tools that utilize the
`Curve` data type.
::::

## Required Methods[§](#required-methods){.anchor} {#required-methods .section-header}

::::: methods
::: {#tymethod.smile .section .method}
[Source](../../src/optionstratlib/volatility/traits.rs.html#81){.src
.rightside}

#### fn [smile](#tymethod.smile){.fn}(&self) -\> [Curve](../curves/struct.Curve.html "struct optionstratlib::curves::Curve"){.struct} {#fn-smileself---curve .code-header}
:::

::: docblock
Computes and returns a curve representing the volatility smile.

##### [§](#returns){.doc-anchor}Returns

- A
  [`Curve`](../curves/struct.Curve.html "struct optionstratlib::curves::Curve")
  object that models the volatility smile. The x-axis typically
  represents strike prices (or another independent variable), while the
  y-axis represents implied volatility.

##### [§](#note){.doc-anchor}Note

- The `Curve` returned should ideally conform to the constraints and
  ordering requirements specified in the `Curve` documentation.
:::
:::::

## Implementors[§](#implementors){.anchor} {#implementors .section-header}

:::: {#implementors-list}
::: {#impl-VolatilitySmile-for-OptionChain .section .impl}
[Source](../../src/optionstratlib/chains/chain.rs.html#2624-2663){.src
.rightside}[§](#impl-VolatilitySmile-for-OptionChain){.anchor}

### impl [VolatilitySmile](trait.VolatilitySmile.html "trait optionstratlib::volatility::VolatilitySmile"){.trait} for [OptionChain](../chains/chain/struct.OptionChain.html "struct optionstratlib::chains::chain::OptionChain"){.struct} {#impl-volatilitysmile-for-optionchain .code-header}
:::
::::
::::::::::::
:::::::::::::
