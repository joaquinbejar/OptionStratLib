:::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::: width-limiter
::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../index.html)::[geometrics](index.html)
:::

# Struct [PlotBuilder]{.struct} Copy item path

[[Source](../../src/optionstratlib/geometrics/visualization/plotters.rs.html#46-59){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub struct PlotBuilder<T: Plottable + Graph> { /* private fields */ }
```

Expand description

::: docblock
A builder for creating and configuring data visualizations.

`PlotBuilder` provides a fluent interface for customizing plots with
various styling and labeling options. It works with any type that
implements the `Plottable` trait, allowing for consistent visualization
capabilities across different data structures.

This builder is typically created via the `plot()` method on types that
implement the `Plottable` trait. After configuring the plot with the
desired options, it can be rendered and saved using the methods from
`PlotBuilderExt`.
:::

## Implementations[§](#implementations){.anchor} {#implementations .section-header}

::::::::::::::::::::::::: {#implementations-list}
::: {#impl-PlotBuilder%3CT%3E .section .impl}
[Source](../../src/optionstratlib/geometrics/visualization/plotters.rs.html#61-265){.src
.rightside}[§](#impl-PlotBuilder%3CT%3E){.anchor}

### impl\<T: [Plottable](trait.Plottable.html "trait optionstratlib::geometrics::Plottable"){.trait} + [Graph](../visualization/trait.Graph.html "trait optionstratlib::visualization::Graph"){.trait}\> [PlotBuilder](struct.PlotBuilder.html "struct optionstratlib::geometrics::PlotBuilder"){.struct}\<T\> {#implt-plottable-graph-plotbuildert .code-header}
:::

::::::::::::::::::::::: impl-items
::: {#method.title .section .method}
[Source](../../src/optionstratlib/geometrics/visualization/plotters.rs.html#72-78){.src
.rightside}

#### pub fn [title](#method.title){.fn}(self, title: impl [Into](https://doc.rust-lang.org/1.91.1/core/convert/trait.Into.html "trait core::convert::Into"){.trait}\<[String](https://doc.rust-lang.org/1.91.1/alloc/string/struct.String.html "struct alloc::string::String"){.struct}\>) -\> Self {#pub-fn-titleself-title-impl-intostring---self .code-header}
:::

::: docblock
Sets the title of the plot.

This method configures the main title that appears at the top of the
visualization.

##### [§](#parameters){.doc-anchor}Parameters

- `title` - The text to display as the plot title

##### [§](#returns){.doc-anchor}Returns

The `PlotBuilder` instance with the updated title setting
:::

::: {#method.x_label .section .method}
[Source](../../src/optionstratlib/geometrics/visualization/plotters.rs.html#90-93){.src
.rightside}

#### pub fn [x_label](#method.x_label){.fn}(self, label: impl [Into](https://doc.rust-lang.org/1.91.1/core/convert/trait.Into.html "trait core::convert::Into"){.trait}\<[String](https://doc.rust-lang.org/1.91.1/alloc/string/struct.String.html "struct alloc::string::String"){.struct}\>) -\> Self {#pub-fn-x_labelself-label-impl-intostring---self .code-header}
:::

::: docblock
Sets the label for the x-axis.

This method configures the descriptive text displayed along the
horizontal axis.

##### [§](#parameters-1){.doc-anchor}Parameters

- `label` - The text to display as the x-axis label

##### [§](#returns-1){.doc-anchor}Returns

The `PlotBuilder` instance with the updated x-axis label
:::

::: {#method.y_label .section .method}
[Source](../../src/optionstratlib/geometrics/visualization/plotters.rs.html#105-108){.src
.rightside}

#### pub fn [y_label](#method.y_label){.fn}(self, label: impl [Into](https://doc.rust-lang.org/1.91.1/core/convert/trait.Into.html "trait core::convert::Into"){.trait}\<[String](https://doc.rust-lang.org/1.91.1/alloc/string/struct.String.html "struct alloc::string::String"){.struct}\>) -\> Self {#pub-fn-y_labelself-label-impl-intostring---self .code-header}
:::

::: docblock
Sets the label for the y-axis.

This method configures the descriptive text displayed along the vertical
axis.

##### [§](#parameters-2){.doc-anchor}Parameters

- `label` - The text to display as the y-axis label

##### [§](#returns-2){.doc-anchor}Returns

The `PlotBuilder` instance with the updated y-axis label
:::

::: {#method.z_label .section .method}
[Source](../../src/optionstratlib/geometrics/visualization/plotters.rs.html#121-124){.src
.rightside}

#### pub fn [z_label](#method.z_label){.fn}(self, label: impl [Into](https://doc.rust-lang.org/1.91.1/core/convert/trait.Into.html "trait core::convert::Into"){.trait}\<[String](https://doc.rust-lang.org/1.91.1/alloc/string/struct.String.html "struct alloc::string::String"){.struct}\>) -\> Self {#pub-fn-z_labelself-label-impl-intostring---self .code-header}
:::

::: docblock
Sets the label for the z-axis.

This method configures the descriptive text displayed along the z-axis
in three-dimensional plots.

##### [§](#parameters-3){.doc-anchor}Parameters

- `label` - The text to display as the z-axis label

##### [§](#returns-3){.doc-anchor}Returns

The `PlotBuilder` instance with the updated z-axis label
:::

::: {#method.line_style .section .method}
[Source](../../src/optionstratlib/geometrics/visualization/plotters.rs.html#137-140){.src
.rightside}

#### pub fn [line_style](#method.line_style){.fn}(self, line_style: [LineStyle](../visualization/enum.LineStyle.html "enum optionstratlib::visualization::LineStyle"){.enum}) -\> Self {#pub-fn-line_styleself-line_style-linestyle---self .code-header}
:::

::: docblock
Sets the colors for data series lines.

This method configures the colors used to render each data series or
curve in the visualization.

##### [§](#parameters-4){.doc-anchor}Parameters

- `colors` - A vector of RGB colors to use for the plot lines

##### [§](#returns-4){.doc-anchor}Returns

The `PlotBuilder` instance with the updated line colors
:::

::: {#method.legend .section .method}
[Source](../../src/optionstratlib/geometrics/visualization/plotters.rs.html#158-162){.src
.rightside}

#### pub fn [legend](#method.legend){.fn}(self, legend: [Vec](https://doc.rust-lang.org/1.91.1/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<impl [Into](https://doc.rust-lang.org/1.91.1/core/convert/trait.Into.html "trait core::convert::Into"){.trait}\<[String](https://doc.rust-lang.org/1.91.1/alloc/string/struct.String.html "struct alloc::string::String"){.struct}\>\>) -\> Self {#pub-fn-legendself-legend-vecimpl-intostring---self .code-header}
:::

::: docblock
Sets the legend for the current instance.

This method accepts a vector of items that can be converted into strings
and sets the `legend` field of the current instance's options. It
returns the modified instance for further chaining.

##### [§](#arguments){.doc-anchor}Arguments

- `legend` - A `Vec` containing items that implement the `Into<String>`
  trait. Each element will be converted into a `String` and assigned to
  the legend.

##### [§](#returns-5){.doc-anchor}Returns

Returns the modified instance with the legend updated.

In this example, the legend is updated to include "Item 1", "Item 2",
and "Item 3".
:::

::: {#method.add_legend .section .method}
[Source](../../src/optionstratlib/geometrics/visualization/plotters.rs.html#183-190){.src
.rightside}

#### pub fn [add_legend](#method.add_legend){.fn}(self, legend: impl [Into](https://doc.rust-lang.org/1.91.1/core/convert/trait.Into.html "trait core::convert::Into"){.trait}\<[String](https://doc.rust-lang.org/1.91.1/alloc/string/struct.String.html "struct alloc::string::String"){.struct}\>) -\> Self {#pub-fn-add_legendself-legend-impl-intostring---self .code-header}
:::

::: docblock
Adds a legend entry to the existing legend configuration or initializes
a new legend with the provided entry if none exists.

This method allows chaining, modifying the `legend` configuration within
the `options` of the current object. If a `legend` already exists, the
provided legend entry will be appended to it. If no `legend` exists, a
new legend will be created containing the provided entry.

##### [§](#arguments-1){.doc-anchor}Arguments

- `legend` - An item implementing `Into<String>` that represents the
  legend entry to add.

##### [§](#returns-6){.doc-anchor}Returns

Returns `Self` (the modified object) to allow method chaining.

In this example, two legend entries, `"Legend 1"` and `"Legend 2"`, are
added to the chart's legend configuration.
:::

::: {#method.dimensions .section .method}
[Source](../../src/optionstratlib/geometrics/visualization/plotters.rs.html#203-207){.src
.rightside}

#### pub fn [dimensions](#method.dimensions){.fn}(self, width: [u32](https://doc.rust-lang.org/1.91.1/std/primitive.u32.html){.primitive}, height: [u32](https://doc.rust-lang.org/1.91.1/std/primitive.u32.html){.primitive}) -\> Self {#pub-fn-dimensionsself-width-u32-height-u32---self .code-header}
:::

::: docblock
Sets the overall dimensions of the plot.

This method configures the width and height of the generated plot image.

##### [§](#parameters-5){.doc-anchor}Parameters

- `width` - The width of the plot in pixels
- `height` - The height of the plot in pixels

##### [§](#returns-7){.doc-anchor}Returns

The `PlotBuilder` instance with the updated dimensions
:::

::: {#method.color_scheme .section .method}
[Source](../../src/optionstratlib/geometrics/visualization/plotters.rs.html#222-225){.src
.rightside}

#### pub fn [color_scheme](#method.color_scheme){.fn}(self, color_scheme: [ColorScheme](../visualization/enum.ColorScheme.html "enum optionstratlib::visualization::ColorScheme"){.enum}) -\> Self {#pub-fn-color_schemeself-color_scheme-colorscheme---self .code-header}
:::

::: docblock
Sets the color scheme for the current instance.

This method allows you to specify a `ColorScheme` to customize the
appearance or theme of the associated object. The method updates the
`color_scheme` field in the `options` struct with the provided value and
returns an updated instance of `self`.

##### [§](#parameters-6){.doc-anchor}Parameters

- `color_scheme`: The desired `ColorScheme` to be applied. This defines
  the visual style or theme to be used by the object.

##### [§](#returns-8){.doc-anchor}Returns

An updated instance of `Self` with the new color scheme applied.
:::

::: {#method.show_legend .section .method}
[Source](../../src/optionstratlib/geometrics/visualization/plotters.rs.html#238-241){.src
.rightside}

#### pub fn [show_legend](#method.show_legend){.fn}(self, show_legend: [bool](https://doc.rust-lang.org/1.91.1/std/primitive.bool.html){.primitive}) -\> Self {#pub-fn-show_legendself-show_legend-bool---self .code-header}
:::

::: docblock
Sets the visibility of the legend in the chart or visualization.

##### [§](#parameters-7){.doc-anchor}Parameters

- `show_legend`: A boolean specifying whether the legend should be
  displayed.
  - `true`: The legend will be displayed.
  - `false`: The legend will be hidden.

##### [§](#returns-9){.doc-anchor}Returns

- Returns an updated instance of `Self` with the `show_legend` option
  set according to the provided parameter.
:::
:::::::::::::::::::::::
:::::::::::::::::::::::::

## Trait Implementations[§](#trait-implementations){.anchor} {#trait-implementations .section-header}

::::::::: {#trait-implementations-list}
::: {#impl-Graph-for-PlotBuilder%3CT%3E .section .impl}
[Source](../../src/optionstratlib/geometrics/visualization/plotters.rs.html#267-286){.src
.rightside}[§](#impl-Graph-for-PlotBuilder%3CT%3E){.anchor}

### impl\<T: [Plottable](trait.Plottable.html "trait optionstratlib::geometrics::Plottable"){.trait} + [Graph](../visualization/trait.Graph.html "trait optionstratlib::visualization::Graph"){.trait}\> [Graph](../visualization/trait.Graph.html "trait optionstratlib::visualization::Graph"){.trait} for [PlotBuilder](struct.PlotBuilder.html "struct optionstratlib::geometrics::PlotBuilder"){.struct}\<T\> {#implt-plottable-graph-graph-for-plotbuildert .code-header}
:::

::::::: impl-items
::: {#method.graph_data .section .method .trait-impl}
[Source](../../src/optionstratlib/geometrics/visualization/plotters.rs.html#268-270){.src
.rightside}[§](#method.graph_data){.anchor}

#### fn [graph_data](../visualization/trait.Graph.html#tymethod.graph_data){.fn}(&self) -\> [GraphData](../visualization/enum.GraphData.html "enum optionstratlib::visualization::GraphData"){.enum} {#fn-graph_dataself---graphdata .code-header}
:::

::: docblock
Return the raw data ready for plotting.
:::

::: {#method.graph_config .section .method .trait-impl}
[Source](../../src/optionstratlib/geometrics/visualization/plotters.rs.html#272-285){.src
.rightside}[§](#method.graph_config){.anchor}

#### fn [graph_config](../visualization/trait.Graph.html#method.graph_config){.fn}(&self) -\> [GraphConfig](../visualization/struct.GraphConfig.html "struct optionstratlib::visualization::GraphConfig"){.struct} {#fn-graph_configself---graphconfig .code-header}
:::

::: docblock
Optional per‑object configuration overrides.
:::
:::::::
:::::::::

## Auto Trait Implementations[§](#synthetic-implementations){.anchor} {#synthetic-implementations .section-header}

::::::::::::::: {#synthetic-implementations-list}
:::: {#impl-Freeze-for-PlotBuilder%3CT%3E .section .impl}
[§](#impl-Freeze-for-PlotBuilder%3CT%3E){.anchor}

### impl\<T\> [Freeze](https://doc.rust-lang.org/1.91.1/core/marker/trait.Freeze.html "trait core::marker::Freeze"){.trait} for [PlotBuilder](struct.PlotBuilder.html "struct optionstratlib::geometrics::PlotBuilder"){.struct}\<T\> {#implt-freeze-for-plotbuildert .code-header}

::: where
where T:
[Freeze](https://doc.rust-lang.org/1.91.1/core/marker/trait.Freeze.html "trait core::marker::Freeze"){.trait},
:::
::::

:::: {#impl-RefUnwindSafe-for-PlotBuilder%3CT%3E .section .impl}
[§](#impl-RefUnwindSafe-for-PlotBuilder%3CT%3E){.anchor}

### impl\<T\> [RefUnwindSafe](https://doc.rust-lang.org/1.91.1/core/panic/unwind_safe/trait.RefUnwindSafe.html "trait core::panic::unwind_safe::RefUnwindSafe"){.trait} for [PlotBuilder](struct.PlotBuilder.html "struct optionstratlib::geometrics::PlotBuilder"){.struct}\<T\> {#implt-refunwindsafe-for-plotbuildert .code-header}

::: where
where T:
[RefUnwindSafe](https://doc.rust-lang.org/1.91.1/core/panic/unwind_safe/trait.RefUnwindSafe.html "trait core::panic::unwind_safe::RefUnwindSafe"){.trait},
:::
::::

:::: {#impl-Send-for-PlotBuilder%3CT%3E .section .impl}
[§](#impl-Send-for-PlotBuilder%3CT%3E){.anchor}

### impl\<T\> [Send](https://doc.rust-lang.org/1.91.1/core/marker/trait.Send.html "trait core::marker::Send"){.trait} for [PlotBuilder](struct.PlotBuilder.html "struct optionstratlib::geometrics::PlotBuilder"){.struct}\<T\> {#implt-send-for-plotbuildert .code-header}

::: where
where T:
[Send](https://doc.rust-lang.org/1.91.1/core/marker/trait.Send.html "trait core::marker::Send"){.trait},
:::
::::

:::: {#impl-Sync-for-PlotBuilder%3CT%3E .section .impl}
[§](#impl-Sync-for-PlotBuilder%3CT%3E){.anchor}

### impl\<T\> [Sync](https://doc.rust-lang.org/1.91.1/core/marker/trait.Sync.html "trait core::marker::Sync"){.trait} for [PlotBuilder](struct.PlotBuilder.html "struct optionstratlib::geometrics::PlotBuilder"){.struct}\<T\> {#implt-sync-for-plotbuildert .code-header}

::: where
where T:
[Sync](https://doc.rust-lang.org/1.91.1/core/marker/trait.Sync.html "trait core::marker::Sync"){.trait},
:::
::::

:::: {#impl-Unpin-for-PlotBuilder%3CT%3E .section .impl}
[§](#impl-Unpin-for-PlotBuilder%3CT%3E){.anchor}

### impl\<T\> [Unpin](https://doc.rust-lang.org/1.91.1/core/marker/trait.Unpin.html "trait core::marker::Unpin"){.trait} for [PlotBuilder](struct.PlotBuilder.html "struct optionstratlib::geometrics::PlotBuilder"){.struct}\<T\> {#implt-unpin-for-plotbuildert .code-header}

::: where
where T:
[Unpin](https://doc.rust-lang.org/1.91.1/core/marker/trait.Unpin.html "trait core::marker::Unpin"){.trait},
:::
::::

:::: {#impl-UnwindSafe-for-PlotBuilder%3CT%3E .section .impl}
[§](#impl-UnwindSafe-for-PlotBuilder%3CT%3E){.anchor}

### impl\<T\> [UnwindSafe](https://doc.rust-lang.org/1.91.1/core/panic/unwind_safe/trait.UnwindSafe.html "trait core::panic::unwind_safe::UnwindSafe"){.trait} for [PlotBuilder](struct.PlotBuilder.html "struct optionstratlib::geometrics::PlotBuilder"){.struct}\<T\> {#implt-unwindsafe-for-plotbuildert .code-header}

::: where
where T:
[UnwindSafe](https://doc.rust-lang.org/1.91.1/core/panic/unwind_safe/trait.UnwindSafe.html "trait core::panic::unwind_safe::UnwindSafe"){.trait},
:::
::::
:::::::::::::::

## Blanket Implementations[§](#blanket-implementations){.anchor} {#blanket-implementations .section-header}

:::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::: {#blanket-implementations-list}
:::: {#impl-Any-for-T .section .impl}
[Source](https://doc.rust-lang.org/1.91.1/src/core/any.rs.html#138){.src
.rightside}[§](#impl-Any-for-T){.anchor}

### impl\<T\> [Any](https://doc.rust-lang.org/1.91.1/core/any/trait.Any.html "trait core::any::Any"){.trait} for T {#implt-any-for-t .code-header}

::: where
where T: \'static +
?[Sized](https://doc.rust-lang.org/1.91.1/core/marker/trait.Sized.html "trait core::marker::Sized"){.trait},
:::
::::

::::: impl-items
::: {#method.type_id .section .method .trait-impl}
[Source](https://doc.rust-lang.org/1.91.1/src/core/any.rs.html#139){.src
.rightside}[§](#method.type_id){.anchor}

#### fn [type_id](https://doc.rust-lang.org/1.91.1/core/any/trait.Any.html#tymethod.type_id){.fn}(&self) -\> [TypeId](https://doc.rust-lang.org/1.91.1/core/any/struct.TypeId.html "struct core::any::TypeId"){.struct} {#fn-type_idself---typeid .code-header}
:::

::: docblock
Gets the `TypeId` of `self`. [Read
more](https://doc.rust-lang.org/1.91.1/core/any/trait.Any.html#tymethod.type_id)
:::
:::::

:::: {#impl-Borrow%3CT%3E-for-T .section .impl}
[Source](https://doc.rust-lang.org/1.91.1/src/core/borrow.rs.html#212){.src
.rightside}[§](#impl-Borrow%3CT%3E-for-T){.anchor}

### impl\<T\> [Borrow](https://doc.rust-lang.org/1.91.1/core/borrow/trait.Borrow.html "trait core::borrow::Borrow"){.trait}\<T\> for T {#implt-borrowt-for-t .code-header}

::: where
where T:
?[Sized](https://doc.rust-lang.org/1.91.1/core/marker/trait.Sized.html "trait core::marker::Sized"){.trait},
:::
::::

::::: impl-items
::: {#method.borrow .section .method .trait-impl}
[Source](https://doc.rust-lang.org/1.91.1/src/core/borrow.rs.html#214){.src
.rightside}[§](#method.borrow){.anchor}

#### fn [borrow](https://doc.rust-lang.org/1.91.1/core/borrow/trait.Borrow.html#tymethod.borrow){.fn}(&self) -\> [&T](https://doc.rust-lang.org/1.91.1/std/primitive.reference.html){.primitive} {#fn-borrowself---t .code-header}
:::

::: docblock
Immutably borrows from an owned value. [Read
more](https://doc.rust-lang.org/1.91.1/core/borrow/trait.Borrow.html#tymethod.borrow)
:::
:::::

:::: {#impl-BorrowMut%3CT%3E-for-T .section .impl}
[Source](https://doc.rust-lang.org/1.91.1/src/core/borrow.rs.html#221){.src
.rightside}[§](#impl-BorrowMut%3CT%3E-for-T){.anchor}

### impl\<T\> [BorrowMut](https://doc.rust-lang.org/1.91.1/core/borrow/trait.BorrowMut.html "trait core::borrow::BorrowMut"){.trait}\<T\> for T {#implt-borrowmutt-for-t .code-header}

::: where
where T:
?[Sized](https://doc.rust-lang.org/1.91.1/core/marker/trait.Sized.html "trait core::marker::Sized"){.trait},
:::
::::

::::: impl-items
::: {#method.borrow_mut .section .method .trait-impl}
[Source](https://doc.rust-lang.org/1.91.1/src/core/borrow.rs.html#222){.src
.rightside}[§](#method.borrow_mut){.anchor}

#### fn [borrow_mut](https://doc.rust-lang.org/1.91.1/core/borrow/trait.BorrowMut.html#tymethod.borrow_mut){.fn}(&mut self) -\> [&mut T](https://doc.rust-lang.org/1.91.1/std/primitive.reference.html){.primitive} {#fn-borrow_mutmut-self---mut-t .code-header}
:::

::: docblock
Mutably borrows from an owned value. [Read
more](https://doc.rust-lang.org/1.91.1/core/borrow/trait.BorrowMut.html#tymethod.borrow_mut)
:::
:::::

::: {#impl-From%3CT%3E-for-T .section .impl}
[Source](https://doc.rust-lang.org/1.91.1/src/core/convert/mod.rs.html#785){.src
.rightside}[§](#impl-From%3CT%3E-for-T){.anchor}

### impl\<T\> [From](https://doc.rust-lang.org/1.91.1/core/convert/trait.From.html "trait core::convert::From"){.trait}\<T\> for T {#implt-fromt-for-t .code-header}
:::

::::: impl-items
::: {#method.from .section .method .trait-impl}
[Source](https://doc.rust-lang.org/1.91.1/src/core/convert/mod.rs.html#788){.src
.rightside}[§](#method.from){.anchor}

#### fn [from](https://doc.rust-lang.org/1.91.1/core/convert/trait.From.html#tymethod.from){.fn}(t: T) -\> T {#fn-fromt-t---t .code-header}
:::

::: docblock
Returns the argument unchanged.
:::
:::::

::: {#impl-Instrument-for-T .section .impl}
[Source](../../src/tracing/instrument.rs.html#325){.src
.rightside}[§](#impl-Instrument-for-T){.anchor}

### impl\<T\> [Instrument](../../tracing/instrument/trait.Instrument.html "trait tracing::instrument::Instrument"){.trait} for T {#implt-instrument-for-t .code-header}
:::

::::::: impl-items
::: {#method.instrument .section .method .trait-impl}
[Source](../../src/tracing/instrument.rs.html#86){.src
.rightside}[§](#method.instrument){.anchor}

#### fn [instrument](../../tracing/instrument/trait.Instrument.html#method.instrument){.fn}(self, span: [Span](../../tracing/span/struct.Span.html "struct tracing::span::Span"){.struct}) -\> [Instrumented](../../tracing/instrument/struct.Instrumented.html "struct tracing::instrument::Instrumented"){.struct}\<Self\> {#fn-instrumentself-span-span---instrumentedself .code-header}
:::

::: docblock
Instruments this type with the provided
[`Span`](../../tracing/span/struct.Span.html "struct tracing::span::Span"),
returning an `Instrumented` wrapper. [Read
more](../../tracing/instrument/trait.Instrument.html#method.instrument)
:::

::: {#method.in_current_span .section .method .trait-impl}
[Source](../../src/tracing/instrument.rs.html#128){.src
.rightside}[§](#method.in_current_span){.anchor}

#### fn [in_current_span](../../tracing/instrument/trait.Instrument.html#method.in_current_span){.fn}(self) -\> [Instrumented](../../tracing/instrument/struct.Instrumented.html "struct tracing::instrument::Instrumented"){.struct}\<Self\> {#fn-in_current_spanself---instrumentedself .code-header}
:::

::: docblock
Instruments this type with the
[current](../../tracing/span/struct.Span.html#method.current "associated function tracing::span::Span::current")
[`Span`](../../tracing/span/struct.Span.html "struct tracing::span::Span"),
returning an `Instrumented` wrapper. [Read
more](../../tracing/instrument/trait.Instrument.html#method.in_current_span)
:::
:::::::

:::: {#impl-Into%3CU%3E-for-T .section .impl}
[Source](https://doc.rust-lang.org/1.91.1/src/core/convert/mod.rs.html#767-769){.src
.rightside}[§](#impl-Into%3CU%3E-for-T){.anchor}

### impl\<T, U\> [Into](https://doc.rust-lang.org/1.91.1/core/convert/trait.Into.html "trait core::convert::Into"){.trait}\<U\> for T {#implt-u-intou-for-t .code-header}

::: where
where U:
[From](https://doc.rust-lang.org/1.91.1/core/convert/trait.From.html "trait core::convert::From"){.trait}\<T\>,
:::
::::

::::: impl-items
::: {#method.into .section .method .trait-impl}
[Source](https://doc.rust-lang.org/1.91.1/src/core/convert/mod.rs.html#777){.src
.rightside}[§](#method.into){.anchor}

#### fn [into](https://doc.rust-lang.org/1.91.1/core/convert/trait.Into.html#tymethod.into){.fn}(self) -\> U {#fn-intoself---u .code-header}
:::

::: docblock
Calls `U::from(self)`.

That is, this conversion is whatever the implementation of
[`From`](https://doc.rust-lang.org/1.91.1/core/convert/trait.From.html "trait core::convert::From")`<T> for U`
chooses to do.
:::
:::::

::: {#impl-IntoEither-for-T .section .impl}
[Source](../../src/either/into_either.rs.html#64){.src
.rightside}[§](#impl-IntoEither-for-T){.anchor}

### impl\<T\> [IntoEither](../../either/into_either/trait.IntoEither.html "trait either::into_either::IntoEither"){.trait} for T {#implt-intoeither-for-t .code-header}
:::

:::::::: impl-items
::: {#method.into_either .section .method .trait-impl}
[Source](../../src/either/into_either.rs.html#29){.src
.rightside}[§](#method.into_either){.anchor}

#### fn [into_either](../../either/into_either/trait.IntoEither.html#method.into_either){.fn}(self, into_left: [bool](https://doc.rust-lang.org/1.91.1/std/primitive.bool.html){.primitive}) -\> [Either](../../either/enum.Either.html "enum either::Either"){.enum}\<Self, Self\> {#fn-into_eitherself-into_left-bool---eitherself-self .code-header}
:::

::: docblock
Converts `self` into a
[`Left`](../../either/enum.Either.html#variant.Left "variant either::Either::Left")
variant of
[`Either<Self, Self>`](../../either/enum.Either.html "enum either::Either")
if `into_left` is `true`. Converts `self` into a
[`Right`](../../either/enum.Either.html#variant.Right "variant either::Either::Right")
variant of
[`Either<Self, Self>`](../../either/enum.Either.html "enum either::Either")
otherwise. [Read
more](../../either/into_either/trait.IntoEither.html#method.into_either)
:::

:::: {#method.into_either_with .section .method .trait-impl}
[Source](../../src/either/into_either.rs.html#55-57){.src
.rightside}[§](#method.into_either_with){.anchor}

#### fn [into_either_with](../../either/into_either/trait.IntoEither.html#method.into_either_with){.fn}\<F\>(self, into_left: F) -\> [Either](../../either/enum.Either.html "enum either::Either"){.enum}\<Self, Self\> {#fn-into_either_withfself-into_left-f---eitherself-self .code-header}

::: where
where F:
[FnOnce](https://doc.rust-lang.org/1.91.1/core/ops/function/trait.FnOnce.html "trait core::ops::function::FnOnce"){.trait}(&Self)
-\>
[bool](https://doc.rust-lang.org/1.91.1/std/primitive.bool.html){.primitive},
:::
::::

::: docblock
Converts `self` into a
[`Left`](../../either/enum.Either.html#variant.Left "variant either::Either::Left")
variant of
[`Either<Self, Self>`](../../either/enum.Either.html "enum either::Either")
if `into_left(&self)` returns `true`. Converts `self` into a
[`Right`](../../either/enum.Either.html#variant.Right "variant either::Either::Right")
variant of
[`Either<Self, Self>`](../../either/enum.Either.html "enum either::Either")
otherwise. [Read
more](../../either/into_either/trait.IntoEither.html#method.into_either_with)
:::
::::::::

::: {#impl-Pointable-for-T .section .impl}
[Source](../../src/crossbeam_epoch/atomic.rs.html#194){.src
.rightside}[§](#impl-Pointable-for-T){.anchor}

### impl\<T\> [Pointable](../../crossbeam_epoch/atomic/trait.Pointable.html "trait crossbeam_epoch::atomic::Pointable"){.trait} for T {#implt-pointable-for-t .code-header}
:::

::::::::::::::: impl-items
::: {#associatedconstant.ALIGN .section .associatedconstant .trait-impl}
[Source](../../src/crossbeam_epoch/atomic.rs.html#195){.src
.rightside}[§](#associatedconstant.ALIGN){.anchor}

#### const [ALIGN](../../crossbeam_epoch/atomic/trait.Pointable.html#associatedconstant.ALIGN){.constant}: [usize](https://doc.rust-lang.org/1.91.1/std/primitive.usize.html){.primitive} {#const-align-usize .code-header}
:::

::: docblock
The alignment of pointer.
:::

::: {#associatedtype.Init .section .associatedtype .trait-impl}
[Source](../../src/crossbeam_epoch/atomic.rs.html#197){.src
.rightside}[§](#associatedtype.Init){.anchor}

#### type [Init](../../crossbeam_epoch/atomic/trait.Pointable.html#associatedtype.Init){.associatedtype} = T {#type-init-t .code-header}
:::

::: docblock
The type for initializers.
:::

::: {#method.init .section .method .trait-impl}
[Source](../../src/crossbeam_epoch/atomic.rs.html#199){.src
.rightside}[§](#method.init){.anchor}

#### unsafe fn [init](../../crossbeam_epoch/atomic/trait.Pointable.html#tymethod.init){.fn}(init: \<T as [Pointable](../../crossbeam_epoch/atomic/trait.Pointable.html "trait crossbeam_epoch::atomic::Pointable"){.trait}\>::[Init](../../crossbeam_epoch/atomic/trait.Pointable.html#associatedtype.Init "type crossbeam_epoch::atomic::Pointable::Init"){.associatedtype}) -\> [usize](https://doc.rust-lang.org/1.91.1/std/primitive.usize.html){.primitive} {#unsafe-fn-initinit-t-as-pointableinit---usize .code-header}
:::

::: docblock
Initializes a with the given initializer. [Read
more](../../crossbeam_epoch/atomic/trait.Pointable.html#tymethod.init)
:::

::: {#method.deref .section .method .trait-impl}
[Source](../../src/crossbeam_epoch/atomic.rs.html#203){.src
.rightside}[§](#method.deref){.anchor}

#### unsafe fn [deref](../../crossbeam_epoch/atomic/trait.Pointable.html#tymethod.deref){.fn}\<\'a\>(ptr: [usize](https://doc.rust-lang.org/1.91.1/std/primitive.usize.html){.primitive}) -\> [&\'a T](https://doc.rust-lang.org/1.91.1/std/primitive.reference.html){.primitive} {#unsafe-fn-derefaptr-usize---a-t .code-header}
:::

::: docblock
Dereferences the given pointer. [Read
more](../../crossbeam_epoch/atomic/trait.Pointable.html#tymethod.deref)
:::

::: {#method.deref_mut .section .method .trait-impl}
[Source](../../src/crossbeam_epoch/atomic.rs.html#207){.src
.rightside}[§](#method.deref_mut){.anchor}

#### unsafe fn [deref_mut](../../crossbeam_epoch/atomic/trait.Pointable.html#tymethod.deref_mut){.fn}\<\'a\>(ptr: [usize](https://doc.rust-lang.org/1.91.1/std/primitive.usize.html){.primitive}) -\> [&\'a mut T](https://doc.rust-lang.org/1.91.1/std/primitive.reference.html){.primitive} {#unsafe-fn-deref_mutaptr-usize---a-mut-t .code-header}
:::

::: docblock
Mutably dereferences the given pointer. [Read
more](../../crossbeam_epoch/atomic/trait.Pointable.html#tymethod.deref_mut)
:::

::: {#method.drop .section .method .trait-impl}
[Source](../../src/crossbeam_epoch/atomic.rs.html#211){.src
.rightside}[§](#method.drop){.anchor}

#### unsafe fn [drop](../../crossbeam_epoch/atomic/trait.Pointable.html#tymethod.drop){.fn}(ptr: [usize](https://doc.rust-lang.org/1.91.1/std/primitive.usize.html){.primitive}) {#unsafe-fn-dropptr-usize .code-header}
:::

::: docblock
Drops the object pointed to by the given pointer. [Read
more](../../crossbeam_epoch/atomic/trait.Pointable.html#tymethod.drop)
:::
:::::::::::::::

::: {#impl-Same-for-T .section .impl}
[Source](../../src/typenum/type_operators.rs.html#34){.src
.rightside}[§](#impl-Same-for-T){.anchor}

### impl\<T\> [Same](../../typenum/type_operators/trait.Same.html "trait typenum::type_operators::Same"){.trait} for T {#implt-same-for-t .code-header}
:::

::::: impl-items
::: {#associatedtype.Output .section .associatedtype .trait-impl}
[Source](../../src/typenum/type_operators.rs.html#35){.src
.rightside}[§](#associatedtype.Output){.anchor}

#### type [Output](../../typenum/type_operators/trait.Same.html#associatedtype.Output){.associatedtype} = T {#type-output-t .code-header}
:::

::: docblock
Should always be `Self`
:::
:::::

:::: {#impl-SupersetOf%3CSS%3E-for-SP .section .impl}
[Source](../../src/simba/scalar/subset.rs.html#90){.src
.rightside}[§](#impl-SupersetOf%3CSS%3E-for-SP){.anchor}

### impl\<SS, SP\> [SupersetOf](../../simba/scalar/subset/trait.SupersetOf.html "trait simba::scalar::subset::SupersetOf"){.trait}\<SS\> for SP {#implss-sp-supersetofss-for-sp .code-header}

::: where
where SS:
[SubsetOf](../../simba/scalar/subset/trait.SubsetOf.html "trait simba::scalar::subset::SubsetOf"){.trait}\<SP\>,
:::
::::

::::::::::: impl-items
::: {#method.to_subset .section .method .trait-impl}
[Source](../../src/simba/scalar/subset.rs.html#92){.src
.rightside}[§](#method.to_subset){.anchor}

#### fn [to_subset](../../simba/scalar/subset/trait.SupersetOf.html#method.to_subset){.fn}(&self) -\> [Option](https://doc.rust-lang.org/1.91.1/core/option/enum.Option.html "enum core::option::Option"){.enum}\<SS\> {#fn-to_subsetself---optionss .code-header}
:::

::: docblock
The inverse inclusion map: attempts to construct `self` from the
equivalent element of its superset. [Read
more](../../simba/scalar/subset/trait.SupersetOf.html#method.to_subset)
:::

::: {#method.is_in_subset .section .method .trait-impl}
[Source](../../src/simba/scalar/subset.rs.html#97){.src
.rightside}[§](#method.is_in_subset){.anchor}

#### fn [is_in_subset](../../simba/scalar/subset/trait.SupersetOf.html#tymethod.is_in_subset){.fn}(&self) -\> [bool](https://doc.rust-lang.org/1.91.1/std/primitive.bool.html){.primitive} {#fn-is_in_subsetself---bool .code-header}
:::

::: docblock
Checks if `self` is actually part of its subset `T` (and can be
converted to it).
:::

::: {#method.to_subset_unchecked .section .method .trait-impl}
[Source](../../src/simba/scalar/subset.rs.html#102){.src
.rightside}[§](#method.to_subset_unchecked){.anchor}

#### fn [to_subset_unchecked](../../simba/scalar/subset/trait.SupersetOf.html#tymethod.to_subset_unchecked){.fn}(&self) -\> SS {#fn-to_subset_uncheckedself---ss .code-header}
:::

::: docblock
Use with care! Same as `self.to_subset` but without any property checks.
Always succeeds.
:::

::: {#method.from_subset .section .method .trait-impl}
[Source](../../src/simba/scalar/subset.rs.html#107){.src
.rightside}[§](#method.from_subset){.anchor}

#### fn [from_subset](../../simba/scalar/subset/trait.SupersetOf.html#tymethod.from_subset){.fn}(element: [&SS](https://doc.rust-lang.org/1.91.1/std/primitive.reference.html){.primitive}) -\> SP {#fn-from_subsetelement-ss---sp .code-header}
:::

::: docblock
The inclusion map: converts `self` to the equivalent element of its
superset.
:::
:::::::::::

:::: {#impl-TryFrom%3CU%3E-for-T .section .impl}
[Source](https://doc.rust-lang.org/1.91.1/src/core/convert/mod.rs.html#827-829){.src
.rightside}[§](#impl-TryFrom%3CU%3E-for-T){.anchor}

### impl\<T, U\> [TryFrom](https://doc.rust-lang.org/1.91.1/core/convert/trait.TryFrom.html "trait core::convert::TryFrom"){.trait}\<U\> for T {#implt-u-tryfromu-for-t .code-header}

::: where
where U:
[Into](https://doc.rust-lang.org/1.91.1/core/convert/trait.Into.html "trait core::convert::Into"){.trait}\<T\>,
:::
::::

::::::: impl-items
::: {#associatedtype.Error-1 .section .associatedtype .trait-impl}
[Source](https://doc.rust-lang.org/1.91.1/src/core/convert/mod.rs.html#831){.src
.rightside}[§](#associatedtype.Error-1){.anchor}

#### type [Error](https://doc.rust-lang.org/1.91.1/core/convert/trait.TryFrom.html#associatedtype.Error){.associatedtype} = [Infallible](https://doc.rust-lang.org/1.91.1/core/convert/enum.Infallible.html "enum core::convert::Infallible"){.enum} {#type-error-infallible .code-header}
:::

::: docblock
The type returned in the event of a conversion error.
:::

::: {#method.try_from .section .method .trait-impl}
[Source](https://doc.rust-lang.org/1.91.1/src/core/convert/mod.rs.html#834){.src
.rightside}[§](#method.try_from){.anchor}

#### fn [try_from](https://doc.rust-lang.org/1.91.1/core/convert/trait.TryFrom.html#tymethod.try_from){.fn}(value: U) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<T, \<T as [TryFrom](https://doc.rust-lang.org/1.91.1/core/convert/trait.TryFrom.html "trait core::convert::TryFrom"){.trait}\<U\>\>::[Error](https://doc.rust-lang.org/1.91.1/core/convert/trait.TryFrom.html#associatedtype.Error "type core::convert::TryFrom::Error"){.associatedtype}\> {#fn-try_fromvalue-u---resultt-t-as-tryfromuerror .code-header}
:::

::: docblock
Performs the conversion.
:::
:::::::

:::: {#impl-TryInto%3CU%3E-for-T .section .impl}
[Source](https://doc.rust-lang.org/1.91.1/src/core/convert/mod.rs.html#811-813){.src
.rightside}[§](#impl-TryInto%3CU%3E-for-T){.anchor}

### impl\<T, U\> [TryInto](https://doc.rust-lang.org/1.91.1/core/convert/trait.TryInto.html "trait core::convert::TryInto"){.trait}\<U\> for T {#implt-u-tryintou-for-t .code-header}

::: where
where U:
[TryFrom](https://doc.rust-lang.org/1.91.1/core/convert/trait.TryFrom.html "trait core::convert::TryFrom"){.trait}\<T\>,
:::
::::

::::::: impl-items
::: {#associatedtype.Error .section .associatedtype .trait-impl}
[Source](https://doc.rust-lang.org/1.91.1/src/core/convert/mod.rs.html#815){.src
.rightside}[§](#associatedtype.Error){.anchor}

#### type [Error](https://doc.rust-lang.org/1.91.1/core/convert/trait.TryInto.html#associatedtype.Error){.associatedtype} = \<U as [TryFrom](https://doc.rust-lang.org/1.91.1/core/convert/trait.TryFrom.html "trait core::convert::TryFrom"){.trait}\<T\>\>::[Error](https://doc.rust-lang.org/1.91.1/core/convert/trait.TryFrom.html#associatedtype.Error "type core::convert::TryFrom::Error"){.associatedtype} {#type-error-u-as-tryfromterror .code-header}
:::

::: docblock
The type returned in the event of a conversion error.
:::

::: {#method.try_into .section .method .trait-impl}
[Source](https://doc.rust-lang.org/1.91.1/src/core/convert/mod.rs.html#818){.src
.rightside}[§](#method.try_into){.anchor}

#### fn [try_into](https://doc.rust-lang.org/1.91.1/core/convert/trait.TryInto.html#tymethod.try_into){.fn}(self) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<U, \<U as [TryFrom](https://doc.rust-lang.org/1.91.1/core/convert/trait.TryFrom.html "trait core::convert::TryFrom"){.trait}\<T\>\>::[Error](https://doc.rust-lang.org/1.91.1/core/convert/trait.TryFrom.html#associatedtype.Error "type core::convert::TryFrom::Error"){.associatedtype}\> {#fn-try_intoself---resultu-u-as-tryfromterror .code-header}
:::

::: docblock
Performs the conversion.
:::
:::::::

:::: {#impl-VZip%3CV%3E-for-T .section .impl}
[Source](../../src/ppv_lite86/types.rs.html#221-223){.src
.rightside}[§](#impl-VZip%3CV%3E-for-T){.anchor}

### impl\<V, T\> [VZip](../../ppv_lite86/types/trait.VZip.html "trait ppv_lite86::types::VZip"){.trait}\<V\> for T {#implv-t-vzipv-for-t .code-header}

::: where
where V:
[MultiLane](../../ppv_lite86/types/trait.MultiLane.html "trait ppv_lite86::types::MultiLane"){.trait}\<T\>,
:::
::::

:::: impl-items
::: {#method.vzip .section .method .trait-impl}
[Source](../../src/ppv_lite86/types.rs.html#226){.src
.rightside}[§](#method.vzip){.anchor}

#### fn [vzip](../../ppv_lite86/types/trait.VZip.html#tymethod.vzip){.fn}(self) -\> V {#fn-vzipself---v .code-header}
:::
::::

::: {#impl-WithSubscriber-for-T .section .impl}
[Source](../../src/tracing/instrument.rs.html#393){.src
.rightside}[§](#impl-WithSubscriber-for-T){.anchor}

### impl\<T\> [WithSubscriber](../../tracing/instrument/trait.WithSubscriber.html "trait tracing::instrument::WithSubscriber"){.trait} for T {#implt-withsubscriber-for-t .code-header}
:::

:::::::: impl-items
:::: {#method.with_subscriber .section .method .trait-impl}
[Source](../../src/tracing/instrument.rs.html#176-178){.src
.rightside}[§](#method.with_subscriber){.anchor}

#### fn [with_subscriber](../../tracing/instrument/trait.WithSubscriber.html#method.with_subscriber){.fn}\<S\>(self, subscriber: S) -\> [WithDispatch](../../tracing/instrument/struct.WithDispatch.html "struct tracing::instrument::WithDispatch"){.struct}\<Self\> {#fn-with_subscribersself-subscriber-s---withdispatchself .code-header}

::: where
where S:
[Into](https://doc.rust-lang.org/1.91.1/core/convert/trait.Into.html "trait core::convert::Into"){.trait}\<[Dispatch](../../tracing_core/dispatcher/struct.Dispatch.html "struct tracing_core::dispatcher::Dispatch"){.struct}\>,
:::
::::

::: docblock
Attaches the provided
[`Subscriber`](../../tracing_core/subscriber/trait.Subscriber.html "trait tracing_core::subscriber::Subscriber")
to this type, returning a
[`WithDispatch`](../../tracing/instrument/struct.WithDispatch.html "struct tracing::instrument::WithDispatch")
wrapper. [Read
more](../../tracing/instrument/trait.WithSubscriber.html#method.with_subscriber)
:::

::: {#method.with_current_subscriber .section .method .trait-impl}
[Source](../../src/tracing/instrument.rs.html#228){.src
.rightside}[§](#method.with_current_subscriber){.anchor}

#### fn [with_current_subscriber](../../tracing/instrument/trait.WithSubscriber.html#method.with_current_subscriber){.fn}(self) -\> [WithDispatch](../../tracing/instrument/struct.WithDispatch.html "struct tracing::instrument::WithDispatch"){.struct}\<Self\> {#fn-with_current_subscriberself---withdispatchself .code-header}
:::

::: docblock
Attaches the current
[default](../../tracing/dispatcher/index.html#setting-the-default-subscriber "mod tracing::dispatcher")
[`Subscriber`](../../tracing_core/subscriber/trait.Subscriber.html "trait tracing_core::subscriber::Subscriber")
to this type, returning a
[`WithDispatch`](../../tracing/instrument/struct.WithDispatch.html "struct tracing::instrument::WithDispatch")
wrapper. [Read
more](../../tracing/instrument/trait.WithSubscriber.html#method.with_current_subscriber)
:::
::::::::
::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
:::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
