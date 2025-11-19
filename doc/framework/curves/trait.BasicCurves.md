::::::::::::::: width-limiter
:::::::::::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../index.html)::[curves](index.html)
:::

# Trait [BasicCurves]{.trait} Copy item path

[[Source](../../src/optionstratlib/curves/basic.rs.html#28-94){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub trait BasicCurves {
    // Required method
    fn curve(
        &self,
        axis: &BasicAxisTypes,
        option_style: &OptionStyle,
        side: &Side,
    ) -> Result<Curve, CurveError>;

    // Provided method
    fn get_curve_strike_versus(
        &self,
        axis: &BasicAxisTypes,
        option: &Arc<Options>,
    ) -> Result<(Decimal, Decimal), CurveError> { ... }
}
```

Expand description

::: docblock
A trait for generating financial option curves based on different
parameters.

This trait provides methods to create and retrieve option curves based
on various financial metrics. It allows for the generation of curves
that plot relationships between option strike prices and different
option Greeks (Delta, Gamma, Theta, Vega), implied volatility, or
prices.

Implementors of this trait can define custom curve generation logic
while using the default implementation for extracting coordinate pairs
for specific option metrics.

## [§](#type-parameters){.doc-anchor}Type Parameters

The trait is designed to work with options data structures and can
generate curves for different visualization and analysis purposes.
:::

## Required Methods[§](#required-methods){.anchor} {#required-methods .section-header}

::::: methods
::: {#tymethod.curve .section .method}
[Source](../../src/optionstratlib/curves/basic.rs.html#44-49){.src
.rightside}

#### fn [curve](#tymethod.curve){.fn}( &self, axis: &[BasicAxisTypes](../model/enum.BasicAxisTypes.html "enum optionstratlib::model::BasicAxisTypes"){.enum}, option_style: &[OptionStyle](../model/types/enum.OptionStyle.html "enum optionstratlib::model::types::OptionStyle"){.enum}, side: &[Side](../model/types/enum.Side.html "enum optionstratlib::model::types::Side"){.enum}, ) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Curve](struct.Curve.html "struct optionstratlib::curves::Curve"){.struct}, [CurveError](../error/curves/enum.CurveError.html "enum optionstratlib::error::curves::CurveError"){.enum}\> {#fn-curve-self-axis-basicaxistypes-option_style-optionstyle-side-side---resultcurve-curveerror .code-header}
:::

::: docblock
Generates a curve for the specified axis type, option style, and market
side.

This method creates a curve that represents the relationship between
strike prices and the selected option metric (as specified by the axis
parameter).

##### [§](#parameters){.doc-anchor}Parameters

- `axis` - The financial metric to be plotted on one of the axes (e.g.,
  Delta, Gamma, Price)
- `option_style` - The style of the option (Call or Put)
- `side` - The market side perspective (Long or Short)

##### [§](#returns){.doc-anchor}Returns

- `Result<Curve, CurveError>` - A curve object containing the plotted
  data points, or an error if the curve could not be generated
:::
:::::

## Provided Methods[§](#provided-methods){.anchor} {#provided-methods .section-header}

::::: methods
::: {#method.get_curve_strike_versus .section .method}
[Source](../../src/optionstratlib/curves/basic.rs.html#67-93){.src
.rightside}

#### fn [get_curve_strike_versus](#method.get_curve_strike_versus){.fn}( &self, axis: &[BasicAxisTypes](../model/enum.BasicAxisTypes.html "enum optionstratlib::model::BasicAxisTypes"){.enum}, option: &[Arc](https://doc.rust-lang.org/1.91.1/alloc/sync/struct.Arc.html "struct alloc::sync::Arc"){.struct}\<[Options](../model/option/struct.Options.html "struct optionstratlib::model::option::Options"){.struct}\>, ) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<([Decimal](../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}, [Decimal](../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}), [CurveError](../error/curves/enum.CurveError.html "enum optionstratlib::error::curves::CurveError"){.enum}\> {#fn-get_curve_strike_versus-self-axis-basicaxistypes-option-arcoptions---resultdecimal-decimal-curveerror .code-header}
:::

::: docblock
Generates coordinate pairs for a specific option and axis type.

This method extracts a pair of values (strike price and the selected
metric) from an option based on the specified axis type. The first value
in the pair is always the strike price, and the second value is
determined by the axis type.

##### [§](#parameters-1){.doc-anchor}Parameters

- `axis` - The financial metric to extract (e.g., Delta, Gamma, Implied
  Volatility)
- `option` - The option contract from which to extract the values

##### [§](#returns-1){.doc-anchor}Returns

- `Result<(Decimal, Decimal), CurveError>` - A tuple containing (strike
  price, metric value), or an error if the values could not be extracted
:::
:::::

## Implementors[§](#implementors){.anchor} {#implementors .section-header}

:::: {#implementors-list}
::: {#impl-BasicCurves-for-OptionChain .section .impl}
[Source](../../src/optionstratlib/chains/chain.rs.html#2665-2709){.src
.rightside}[§](#impl-BasicCurves-for-OptionChain){.anchor}

### impl [BasicCurves](trait.BasicCurves.html "trait optionstratlib::curves::BasicCurves"){.trait} for [OptionChain](../chains/chain/struct.OptionChain.html "struct optionstratlib::chains::chain::OptionChain"){.struct} {#impl-basiccurves-for-optionchain .code-header}
:::
::::
::::::::::::::
:::::::::::::::
