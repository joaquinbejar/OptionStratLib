:::::::::::::::::::::::::::::: width-limiter
::::::::::::::::::::::::::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../index.html)::[geometrics](index.html)
:::

# Trait [Plottable]{.trait} Copy item path

[[Source](../../src/optionstratlib/geometrics/visualization/plotters.rs.html#19-33){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub trait Plottable {
    type Error;

    // Required method
    fn plot(&self) -> PlotBuilder<Self>
       where Self: Sized + Graph;
}
```

Expand description

::: docblock
Trait for defining objects that can be visualized as plots.

The `Plottable` trait provides a standardized interface for types that
can be represented graphically. It enables visualization of data
structures through a fluent builder pattern, allowing for customizable
plot creation.

Implementers of this trait can be visualized using the plotting system
with configurable options for appearance, labels, colors, and other
visual attributes.
:::

## Required Associated Types[§](#required-associated-types){.anchor} {#required-associated-types .section-header}

::::: methods
::: {#associatedtype.Error .section .method}
[Source](../../src/optionstratlib/geometrics/visualization/plotters.rs.html#24){.src
.rightside}

#### type [Error](#associatedtype.Error){.associatedtype} {#type-error .code-header}
:::

::: docblock
The error type returned by plotting operations.

This associated type allows implementers to define their specific error
handling approach for plot generation and rendering.
:::
:::::

## Required Methods[§](#required-methods){.anchor} {#required-methods .section-header}

:::::: methods
:::: {#tymethod.plot .section .method}
[Source](../../src/optionstratlib/geometrics/visualization/plotters.rs.html#30-32){.src
.rightside}

#### fn [plot](#tymethod.plot){.fn}(&self) -\> [PlotBuilder](struct.PlotBuilder.html "struct optionstratlib::geometrics::PlotBuilder"){.struct}\<Self\> {#fn-plotself---plotbuilderself .code-header}

::: where
where Self:
[Sized](https://doc.rust-lang.org/1.91.1/core/marker/trait.Sized.html "trait core::marker::Sized"){.trait} +
[Graph](../visualization/trait.Graph.html "trait optionstratlib::visualization::Graph"){.trait},
:::
::::

::: docblock
Creates a plot builder for configuring and generating visualizations.

Returns a `PlotBuilder` instance that provides a fluent interface for
customizing plot appearance and behavior before rendering.
:::
::::::

## Implementations on Foreign Types[§](#foreign-impls){.anchor} {#foreign-impls .section-header}

:::: {#impl-Plottable-for-Vec%3CCurve%3E .section .impl}
[Source](../../src/optionstratlib/curves/visualization/plotters.rs.html#137-149){.src
.rightside}[§](#impl-Plottable-for-Vec%3CCurve%3E){.anchor}

### impl [Plottable](trait.Plottable.html "trait optionstratlib::geometrics::Plottable"){.trait} for [Vec](https://doc.rust-lang.org/1.91.1/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<[Curve](../curves/struct.Curve.html "struct optionstratlib::curves::Curve"){.struct}\> {#impl-plottable-for-veccurve .code-header}

::: docblock
Implementation of the `Plottable` trait for `Vec<Curve>`.
:::
::::

::: docblock
This implementation enables a vector of `Curve` instances to be plotted
using the `plot` method. The method creates a `PlotBuilder` instance,
which allows for flexible and configurable visualization of the curves.

#### [§](#overview){.doc-anchor}Overview

By implementing the `Plottable` trait, a vector of `Curve` objects gains
the ability to leverage plot-building functionality. The `plot` method
clones the data (to ensure immutability of the original input) and pairs
it with default plotting options (`PlotOptions`) for further
configuration and rendering.

The `PlotBuilder` struct, which is returned by this implementation, acts
as a pipeline for customizing and generating the final plot. Once the
plot is fully configured in terms of styling and layout, it can be saved
to a file, rendered in memory, or manipulated further depending on the
builder's available methods.

#### [§](#method-details){.doc-anchor}Method Details

- **`plot`**:
  - Creates a `PlotBuilder` instance containing the data from the
    `Vec<Curve>` and populates it with default plot options.
  - Returns a configurable tool for building curve visualizations.

#### [§](#considerations){.doc-anchor}Considerations

- This implementation assumes that it is appropriate to clone the data
  from the vector of `Curve` instances. If the cloning behavior is
  expensive or not necessary, further optimization may be required.
- `PlotOptions` default values provide a reasonable starting point, but
  most real-world applications will override some of these values for
  more customization.

#### [§](#example-behavior){.doc-anchor}Example Behavior

A vector of `Curve` objects can be passed to the `plot` method to
generate a plot tailored to the desired styling and configuration.
Methods available on `PlotBuilder` can then be chained to adjust plot
dimensions, colors, titles, labels, and more.

#### [§](#returns){.doc-anchor}Returns

- A `PlotBuilder` instance configured with the cloned curve data
  (`self.clone()`) and fully initialized with default `PlotOptions`.

#### [§](#default-settings-1){.doc-anchor}Default Settings {#default-settings-1}

- The default `PlotOptions`, as used in this implementation, include:
  - White background
  - Line width of 2 pixels
  - Default dimensions (800x600 pixels)
  - No title or axis labels
  - No default line colors

#### [§](#errors){.doc-anchor}Errors

- While creating a `PlotBuilder` instance does not directly raise
  errors, subsequent operations (e.g., saving a plot or generating a
  view) may encounter runtime issues related to file I/O, data validity,
  or plot rendering.

#### [§](#see-also){.doc-anchor}See Also

- [`Plottable`](trait.Plottable.html "trait optionstratlib::geometrics::Plottable"):
  The trait allowing generalized plotting functionality.
- [`PlotBuilder`](struct.PlotBuilder.html "struct optionstratlib::geometrics::PlotBuilder"):
  The plot generation and configuration builder.

#### [§](#modules){.doc-anchor}Modules

Code related to this implementation exists within the
`crate::curves::visualization::plotters` module, and it works in
conjunction with the `Curve` struct, `PlotBuilder`, and `PlotOptions`.
These modules provide the functionality required to create, configure,
and render curve plots.
:::

:::::: impl-items
::: {#associatedtype.Error-1 .section .associatedtype .trait-impl}
[Source](../../src/optionstratlib/curves/visualization/plotters.rs.html#138){.src
.rightside}[§](#associatedtype.Error-1){.anchor}

#### type [Error](#associatedtype.Error){.associatedtype} = [CurveError](../error/curves/enum.CurveError.html "enum optionstratlib::error::curves::CurveError"){.enum} {#type-error-curveerror .code-header}
:::

:::: {#method.plot .section .method .trait-impl}
[Source](../../src/optionstratlib/curves/visualization/plotters.rs.html#140-148){.src
.rightside}[§](#method.plot){.anchor}

#### fn [plot](#tymethod.plot){.fn}(&self) -\> [PlotBuilder](struct.PlotBuilder.html "struct optionstratlib::geometrics::PlotBuilder"){.struct}\<Self\> {#fn-plotself---plotbuilderself-1 .code-header}

::: where
where Self:
[Sized](https://doc.rust-lang.org/1.91.1/core/marker/trait.Sized.html "trait core::marker::Sized"){.trait},
:::
::::
::::::

## Implementors[§](#implementors){.anchor} {#implementors .section-header}

::::::::::: {#implementors-list}
:::: {#impl-Plottable-for-Curve .section .impl}
[Source](../../src/optionstratlib/curves/visualization/plotters.rs.html#57-69){.src
.rightside}[§](#impl-Plottable-for-Curve){.anchor}

### impl [Plottable](trait.Plottable.html "trait optionstratlib::geometrics::Plottable"){.trait} for [Curve](../curves/struct.Curve.html "struct optionstratlib::curves::Curve"){.struct} {#impl-plottable-for-curve .code-header}

::: docblock
Plottable implementation for single Curve
:::
::::

:::: impl-items
::: {#associatedtype.Error-2 .section .associatedtype .trait-impl}
[Source](../../src/optionstratlib/curves/visualization/plotters.rs.html#58){.src
.rightside}[§](#associatedtype.Error-2){.anchor}

#### type [Error](#associatedtype.Error){.associatedtype} = [CurveError](../error/curves/enum.CurveError.html "enum optionstratlib::error::curves::CurveError"){.enum} {#type-error-curveerror-1 .code-header}
:::
::::

:::: {#impl-Plottable-for-Surface .section .impl}
[Source](../../src/optionstratlib/surfaces/visualization/plotters.rs.html#56-68){.src
.rightside}[§](#impl-Plottable-for-Surface){.anchor}

### impl [Plottable](trait.Plottable.html "trait optionstratlib::geometrics::Plottable"){.trait} for [Surface](../surfaces/struct.Surface.html "struct optionstratlib::surfaces::Surface"){.struct} {#impl-plottable-for-surface .code-header}

::: docblock
Plottable implementation for single Surface
:::
::::

:::: impl-items
::: {#associatedtype.Error-3 .section .associatedtype .trait-impl}
[Source](../../src/optionstratlib/surfaces/visualization/plotters.rs.html#57){.src
.rightside}[§](#associatedtype.Error-3){.anchor}

#### type [Error](#associatedtype.Error){.associatedtype} = [SurfaceError](../error/enum.SurfaceError.html "enum optionstratlib::error::SurfaceError"){.enum} {#type-error-surfaceerror .code-header}
:::
::::
:::::::::::
:::::::::::::::::::::::::::::
::::::::::::::::::::::::::::::
