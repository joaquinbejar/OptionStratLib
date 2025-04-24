::::::::::::::::: width-limiter
:::::::::::::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../index.html)::[surfaces](index.html)
:::

# Trait [BasicSurfaces]{.trait}Copy item path

[[Source](../../src/optionstratlib/surfaces/basic.rs.html#22-176){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub trait BasicSurfaces {
    // Required method
    fn surface(
        &self,
        axis: &BasicAxisTypes,
        option_style: &OptionStyle,
        volatility: Option<Vec<Positive>>,
        side: &Side,
    ) -> Result<Surface, SurfaceError>;

    // Provided methods
    fn get_surface_strike_versus(
        &self,
        axis: &BasicAxisTypes,
        option: &Arc<Options>,
    ) -> Result<(Decimal, Decimal, Decimal), SurfaceError> { ... }
    fn get_surface_volatility_versus(
        &self,
        axis: &BasicAxisTypes,
        option: &Arc<Options>,
        volatility: Positive,
    ) -> Result<(Decimal, Decimal, Decimal), SurfaceError> { ... }
}
```

Expand description

::: docblock
## [§](#basicsurfaces-trait){.doc-anchor}BasicSurfaces Trait

This trait defines operations for creating and analyzing option pricing
surfaces, which are three-dimensional representations of option metrics
across different parameters.

A surface typically maps option strike prices and volatilities to
various option metrics like delta, gamma, theta, vega, or price.
:::

## Required Methods[§](#required-methods){.anchor} {#required-methods .section-header}

::::: methods
::: {#tymethod.surface .section .method}
[Source](../../src/optionstratlib/surfaces/basic.rs.html#35-41){.src
.rightside}

#### fn [surface](#tymethod.surface){.fn}( &self, axis: &[BasicAxisTypes](../model/enum.BasicAxisTypes.html "enum optionstratlib::model::BasicAxisTypes"){.enum}, option_style: &[OptionStyle](../model/types/enum.OptionStyle.html "enum optionstratlib::model::types::OptionStyle"){.enum}, volatility: [Option](https://doc.rust-lang.org/1.86.0/core/option/enum.Option.html "enum core::option::Option"){.enum}\<[Vec](https://doc.rust-lang.org/1.86.0/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<[Positive](../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\>\>, side: &[Side](../model/types/enum.Side.html "enum optionstratlib::model::types::Side"){.enum}, ) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Surface](struct.Surface.html "struct optionstratlib::surfaces::Surface"){.struct}, [SurfaceError](../error/enum.SurfaceError.html "enum optionstratlib::error::SurfaceError"){.enum}\> {#fn-surface-self-axis-basicaxistypes-option_style-optionstyle-volatility-optionvecpositive-side-side---resultsurface-surfaceerror .code-header}
:::

::: docblock
Creates a surface visualization based on the specified axis type and
option parameters.

##### [§](#parameters){.doc-anchor}Parameters

- `axis` - The option metric to calculate and display on the surface
  (e.g., Delta, Gamma)
- `option_style` - Whether the options are Calls or Puts
- `volatility` - Optional vector of volatility values to use for surface
  calculations
- `side` - Whether the options are Long or Short positions

##### [§](#returns){.doc-anchor}Returns

- `Result<Surface, SurfaceError>` - A constructed surface or an error if
  creation fails
:::
:::::

## Provided Methods[§](#provided-methods){.anchor} {#provided-methods .section-header}

::::::: methods
::: {#method.get_surface_strike_versus .section .method}
[Source](../../src/optionstratlib/surfaces/basic.rs.html#64-107){.src
.rightside}

#### fn [get_surface_strike_versus](#method.get_surface_strike_versus){.fn}( &self, axis: &[BasicAxisTypes](../model/enum.BasicAxisTypes.html "enum optionstratlib::model::BasicAxisTypes"){.enum}, option: &[Arc](https://doc.rust-lang.org/1.86.0/alloc/sync/struct.Arc.html "struct alloc::sync::Arc"){.struct}\<[Options](../model/option/struct.Options.html "struct optionstratlib::model::option::Options"){.struct}\>, ) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<(Decimal, Decimal, Decimal), [SurfaceError](../error/enum.SurfaceError.html "enum optionstratlib::error::SurfaceError"){.enum}\> {#fn-get_surface_strike_versus-self-axis-basicaxistypes-option-arcoptions---resultdecimal-decimal-decimal-surfaceerror .code-header}
:::

::: docblock
Calculates the relationship between strike price, implied volatility,
and a selected option metric for a given option.

This method uses the option's existing implied volatility value to
calculate the desired metric (delta, gamma, theta, vega, or price).

##### [§](#parameters-1){.doc-anchor}Parameters

- `axis` - The option metric to calculate (e.g., Delta, Gamma)
- `option` - Reference to the option contract to analyze

##### [§](#returns-1){.doc-anchor}Returns

- `Result<(Decimal, Decimal, Decimal), SurfaceError>` - A tuple
  containing:
  - Strike price
  - Implied volatility
  - Calculated metric value

##### [§](#errors){.doc-anchor}Errors

Returns a `SurfaceError` if the selected axis is not supported or if any
calculation fails.
:::

::: {#method.get_surface_volatility_versus .section .method}
[Source](../../src/optionstratlib/surfaces/basic.rs.html#131-175){.src
.rightside}

#### fn [get_surface_volatility_versus](#method.get_surface_volatility_versus){.fn}( &self, axis: &[BasicAxisTypes](../model/enum.BasicAxisTypes.html "enum optionstratlib::model::BasicAxisTypes"){.enum}, option: &[Arc](https://doc.rust-lang.org/1.86.0/alloc/sync/struct.Arc.html "struct alloc::sync::Arc"){.struct}\<[Options](../model/option/struct.Options.html "struct optionstratlib::model::option::Options"){.struct}\>, volatility: [Positive](../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, ) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<(Decimal, Decimal, Decimal), [SurfaceError](../error/enum.SurfaceError.html "enum optionstratlib::error::SurfaceError"){.enum}\> {#fn-get_surface_volatility_versus-self-axis-basicaxistypes-option-arcoptions-volatility-positive---resultdecimal-decimal-decimal-surfaceerror .code-header}
:::

::: docblock
Calculates the relationship between strike price, a specified volatility
value, and a selected option metric for a given option.

This method uses a custom volatility value (different from the option's
current implied volatility) to calculate the desired metric (delta,
gamma, theta, vega, or price).

##### [§](#parameters-2){.doc-anchor}Parameters

- `axis` - The option metric to calculate (e.g., Delta, Gamma)
- `option` - Reference to the option contract to analyze
- `volatility` - The specific volatility value to use for the
  calculation

##### [§](#returns-2){.doc-anchor}Returns

- `Result<(Decimal, Decimal, Decimal), SurfaceError>` - A tuple
  containing:
  - Strike price
  - The provided volatility value
  - Calculated metric value

##### [§](#errors-1){.doc-anchor}Errors

Returns a `SurfaceError` if the selected axis is not supported or if any
calculation fails.
:::
:::::::

## Implementors[§](#implementors){.anchor} {#implementors .section-header}

:::: {#implementors-list}
::: {#impl-BasicSurfaces-for-OptionChain .section .impl}
[Source](../../src/optionstratlib/chains/chain.rs.html#2361-2424){.src
.rightside}[§](#impl-BasicSurfaces-for-OptionChain){.anchor}

### impl [BasicSurfaces](trait.BasicSurfaces.html "trait optionstratlib::surfaces::BasicSurfaces"){.trait} for [OptionChain](../chains/chain/struct.OptionChain.html "struct optionstratlib::chains::chain::OptionChain"){.struct} {#impl-basicsurfaces-for-optionchain .code-header}
:::
::::
::::::::::::::::
:::::::::::::::::
