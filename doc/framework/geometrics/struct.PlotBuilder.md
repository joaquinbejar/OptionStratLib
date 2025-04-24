::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::: width-limiter
:::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../index.html)::[geometrics](index.html)
:::

# Struct [PlotBuilder]{.struct}Copy item path

[[Source](../../src/optionstratlib/geometrics/visualization/plotters.rs.html#114-128){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub struct PlotBuilder<T: Plottable> { /* private fields */ }
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

:::::::::::::::::::::::::::: {#implementations-list}
::: {#impl-PlotBuilder%3CT%3E .section .impl}
[Source](../../src/optionstratlib/geometrics/visualization/plotters.rs.html#130-310){.src
.rightside}[§](#impl-PlotBuilder%3CT%3E){.anchor}

### impl\<T: [Plottable](trait.Plottable.html "trait optionstratlib::geometrics::Plottable"){.trait}\> [PlotBuilder](struct.PlotBuilder.html "struct optionstratlib::geometrics::PlotBuilder"){.struct}\<T\> {#implt-plottable-plotbuildert .code-header}
:::

:::::::::::::::::::::::::: impl-items
::: {#method.title .section .method}
[Source](../../src/optionstratlib/geometrics/visualization/plotters.rs.html#141-144){.src
.rightside}

#### pub fn [title](#method.title){.fn}(self, title: impl [Into](https://doc.rust-lang.org/1.86.0/core/convert/trait.Into.html "trait core::convert::Into"){.trait}\<[String](https://doc.rust-lang.org/1.86.0/alloc/string/struct.String.html "struct alloc::string::String"){.struct}\>) -\> Self {#pub-fn-titleself-title-impl-intostring---self .code-header}
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
[Source](../../src/optionstratlib/geometrics/visualization/plotters.rs.html#156-159){.src
.rightside}

#### pub fn [x_label](#method.x_label){.fn}(self, label: impl [Into](https://doc.rust-lang.org/1.86.0/core/convert/trait.Into.html "trait core::convert::Into"){.trait}\<[String](https://doc.rust-lang.org/1.86.0/alloc/string/struct.String.html "struct alloc::string::String"){.struct}\>) -\> Self {#pub-fn-x_labelself-label-impl-intostring---self .code-header}
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
[Source](../../src/optionstratlib/geometrics/visualization/plotters.rs.html#171-174){.src
.rightside}

#### pub fn [y_label](#method.y_label){.fn}(self, label: impl [Into](https://doc.rust-lang.org/1.86.0/core/convert/trait.Into.html "trait core::convert::Into"){.trait}\<[String](https://doc.rust-lang.org/1.86.0/alloc/string/struct.String.html "struct alloc::string::String"){.struct}\>) -\> Self {#pub-fn-y_labelself-label-impl-intostring---self .code-header}
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
[Source](../../src/optionstratlib/geometrics/visualization/plotters.rs.html#187-190){.src
.rightside}

#### pub fn [z_label](#method.z_label){.fn}(self, label: impl [Into](https://doc.rust-lang.org/1.86.0/core/convert/trait.Into.html "trait core::convert::Into"){.trait}\<[String](https://doc.rust-lang.org/1.86.0/alloc/string/struct.String.html "struct alloc::string::String"){.struct}\>) -\> Self {#pub-fn-z_labelself-label-impl-intostring---self .code-header}
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

::: {#method.point_size .section .method}
[Source](../../src/optionstratlib/geometrics/visualization/plotters.rs.html#203-206){.src
.rightside}

#### pub fn [point_size](#method.point_size){.fn}(self, size: [u32](https://doc.rust-lang.org/1.86.0/std/primitive.u32.html){.primitive}) -\> Self {#pub-fn-point_sizeself-size-u32---self .code-header}
:::

::: docblock
Sets the size of data points in scatter plots.

This method configures the diameter of individual data points when
rendering scatter plots.

##### [§](#parameters-4){.doc-anchor}Parameters

- `size` - The size of points in pixels

##### [§](#returns-4){.doc-anchor}Returns

The `PlotBuilder` instance with the updated point size setting
:::

::: {#method.label_size .section .method}
[Source](../../src/optionstratlib/geometrics/visualization/plotters.rs.html#219-222){.src
.rightside}

#### pub fn [label_size](#method.label_size){.fn}(self, size: [f64](https://doc.rust-lang.org/1.86.0/std/primitive.f64.html){.primitive}) -\> Self {#pub-fn-label_sizeself-size-f64---self .code-header}
:::

::: docblock
Sets the font size for labels and text elements.

This method configures the font size used for axis labels, titles, and
other textual elements in the visualization.

##### [§](#parameters-5){.doc-anchor}Parameters

- `size` - The font size as a floating point value

##### [§](#returns-5){.doc-anchor}Returns

The `PlotBuilder` instance with the updated label size setting
:::

::: {#method.curve_name .section .method}
[Source](../../src/optionstratlib/geometrics/visualization/plotters.rs.html#235-238){.src
.rightside}

#### pub fn [curve_name](#method.curve_name){.fn}(self, label: [Vec](https://doc.rust-lang.org/1.86.0/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<[String](https://doc.rust-lang.org/1.86.0/alloc/string/struct.String.html "struct alloc::string::String"){.struct}\>) -\> Self {#pub-fn-curve_nameself-label-vecstring---self .code-header}
:::

::: docblock
Sets custom names for each data series/curve in the plot.

This method configures the names displayed in the legend to identify
different data series in the visualization.

##### [§](#parameters-6){.doc-anchor}Parameters

- `label` - A vector of strings, each representing the name of a curve

##### [§](#returns-6){.doc-anchor}Returns

The `PlotBuilder` instance with the updated curve names
:::

::: {#method.line_colors .section .method}
[Source](../../src/optionstratlib/geometrics/visualization/plotters.rs.html#251-254){.src
.rightside}

#### pub fn [line_colors](#method.line_colors){.fn}(self, colors: [Vec](https://doc.rust-lang.org/1.86.0/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<RGBColor\>) -\> Self {#pub-fn-line_colorsself-colors-vecrgbcolor---self .code-header}
:::

::: docblock
Sets the colors for data series lines.

This method configures the colors used to render each data series or
curve in the visualization.

##### [§](#parameters-7){.doc-anchor}Parameters

- `colors` - A vector of RGB colors to use for the plot lines

##### [§](#returns-7){.doc-anchor}Returns

The `PlotBuilder` instance with the updated line colors
:::

::: {#method.line_width .section .method}
[Source](../../src/optionstratlib/geometrics/visualization/plotters.rs.html#267-270){.src
.rightside}

#### pub fn [line_width](#method.line_width){.fn}(self, width: [u32](https://doc.rust-lang.org/1.86.0/std/primitive.u32.html){.primitive}) -\> Self {#pub-fn-line_widthself-width-u32---self .code-header}
:::

::: docblock
Sets the width of plot lines.

This method configures the thickness of lines used to render data series
in the visualization.

##### [§](#parameters-8){.doc-anchor}Parameters

- `width` - The width of lines in pixels

##### [§](#returns-8){.doc-anchor}Returns

The `PlotBuilder` instance with the updated line width setting
:::

::: {#method.dimensions .section .method}
[Source](../../src/optionstratlib/geometrics/visualization/plotters.rs.html#283-287){.src
.rightside}

#### pub fn [dimensions](#method.dimensions){.fn}(self, width: [u32](https://doc.rust-lang.org/1.86.0/std/primitive.u32.html){.primitive}, height: [u32](https://doc.rust-lang.org/1.86.0/std/primitive.u32.html){.primitive}) -\> Self {#pub-fn-dimensionsself-width-u32-height-u32---self .code-header}
:::

::: docblock
Sets the overall dimensions of the plot.

This method configures the width and height of the generated plot image.

##### [§](#parameters-9){.doc-anchor}Parameters

- `width` - The width of the plot in pixels
- `height` - The height of the plot in pixels

##### [§](#returns-9){.doc-anchor}Returns

The `PlotBuilder` instance with the updated dimensions
:::

:::: {#method.save .section .method}
[Source](../../src/optionstratlib/geometrics/visualization/plotters.rs.html#304-309){.src
.rightside}

#### pub fn [save](#method.save){.fn}(self, path: impl [AsRef](https://doc.rust-lang.org/1.86.0/core/convert/trait.AsRef.html "trait core::convert::AsRef"){.trait}\<[Path](https://doc.rust-lang.org/1.86.0/std/path/struct.Path.html "struct std::path::Path"){.struct}\>) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[()](https://doc.rust-lang.org/1.86.0/std/primitive.unit.html){.primitive}, T::[Error](trait.Plottable.html#associatedtype.Error "type optionstratlib::geometrics::Plottable::Error"){.associatedtype}\> {#pub-fn-saveself-path-impl-asrefpath---result-terror .code-header}

::: where
where Self:
[PlotBuilderExt](trait.PlotBuilderExt.html "trait optionstratlib::geometrics::PlotBuilderExt"){.trait}\<T\>,
:::
::::

::: docblock
Saves the configured plot to a file.

This method renders the plot with all configured options and writes the
result to the specified file path.

##### [§](#parameters-10){.doc-anchor}Parameters

- `path` - The file path where the plot should be saved

##### [§](#returns-10){.doc-anchor}Returns

A `Result` indicating success or containing an error if the save
operation failed

##### [§](#errors){.doc-anchor}Errors

This method will return an error if the plot cannot be rendered or
saved, with the specific error type determined by the `Plottable`
implementation.
:::
::::::::::::::::::::::::::
::::::::::::::::::::::::::::

## Trait Implementations[§](#trait-implementations){.anchor} {#trait-implementations .section-header}

::::::::::::::::::: {#trait-implementations-list}
:::: {#impl-PlotBuilderExt%3CCurve%3E-for-PlotBuilder%3CCurve%3E .section .impl}
[Source](../../src/optionstratlib/curves/visualization/plotters.rs.html#153-233){.src
.rightside}[§](#impl-PlotBuilderExt%3CCurve%3E-for-PlotBuilder%3CCurve%3E){.anchor}

### impl [PlotBuilderExt](trait.PlotBuilderExt.html "trait optionstratlib::geometrics::PlotBuilderExt"){.trait}\<[Curve](../curves/struct.Curve.html "struct optionstratlib::curves::Curve"){.struct}\> for [PlotBuilder](struct.PlotBuilder.html "struct optionstratlib::geometrics::PlotBuilder"){.struct}\<[Curve](../curves/struct.Curve.html "struct optionstratlib::curves::Curve"){.struct}\> {#impl-plotbuilderextcurve-for-plotbuildercurve .code-header}

::: docblock
Plotting implementation for single Curve
:::
::::

::::: impl-items
::: {#method.save-1 .section .method .trait-impl}
[Source](../../src/optionstratlib/curves/visualization/plotters.rs.html#154-232){.src
.rightside}[§](#method.save-1){.anchor}

#### fn [save](trait.PlotBuilderExt.html#tymethod.save){.fn}(self, path: impl [AsRef](https://doc.rust-lang.org/1.86.0/core/convert/trait.AsRef.html "trait core::convert::AsRef"){.trait}\<[Path](https://doc.rust-lang.org/1.86.0/std/path/struct.Path.html "struct std::path::Path"){.struct}\>) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[()](https://doc.rust-lang.org/1.86.0/std/primitive.unit.html){.primitive}, [CurveError](../error/curves/enum.CurveError.html "enum optionstratlib::error::curves::CurveError"){.enum}\> {#fn-saveself-path-impl-asrefpath---result-curveerror .code-header}
:::

::: docblock
Saves the configured plot to a file at the specified path. [Read
more](trait.PlotBuilderExt.html#tymethod.save)
:::
:::::

:::: {#impl-PlotBuilderExt%3CSurface%3E-for-PlotBuilder%3CSurface%3E .section .impl}
[Source](../../src/optionstratlib/surfaces/visualization/plotters.rs.html#150-275){.src
.rightside}[§](#impl-PlotBuilderExt%3CSurface%3E-for-PlotBuilder%3CSurface%3E){.anchor}

### impl [PlotBuilderExt](trait.PlotBuilderExt.html "trait optionstratlib::geometrics::PlotBuilderExt"){.trait}\<[Surface](../surfaces/struct.Surface.html "struct optionstratlib::surfaces::Surface"){.struct}\> for [PlotBuilder](struct.PlotBuilder.html "struct optionstratlib::geometrics::PlotBuilder"){.struct}\<[Surface](../surfaces/struct.Surface.html "struct optionstratlib::surfaces::Surface"){.struct}\> {#impl-plotbuilderextsurface-for-plotbuildersurface .code-header}

::: docblock
Plotting implementation for single Surface
:::
::::

::::: impl-items
::: {#method.save-3 .section .method .trait-impl}
[Source](../../src/optionstratlib/surfaces/visualization/plotters.rs.html#151-274){.src
.rightside}[§](#method.save-3){.anchor}

#### fn [save](trait.PlotBuilderExt.html#tymethod.save){.fn}(self, path: impl [AsRef](https://doc.rust-lang.org/1.86.0/core/convert/trait.AsRef.html "trait core::convert::AsRef"){.trait}\<[Path](https://doc.rust-lang.org/1.86.0/std/path/struct.Path.html "struct std::path::Path"){.struct}\>) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[()](https://doc.rust-lang.org/1.86.0/std/primitive.unit.html){.primitive}, [SurfaceError](../error/enum.SurfaceError.html "enum optionstratlib::error::SurfaceError"){.enum}\> {#fn-saveself-path-impl-asrefpath---result-surfaceerror .code-header}
:::

::: docblock
Saves the configured plot to a file at the specified path. [Read
more](trait.PlotBuilderExt.html#tymethod.save)
:::
:::::

:::: {#impl-PlotBuilderExt%3CVec%3CCurve%3E%3E-for-PlotBuilder%3CVec%3CCurve%3E%3E .section .impl}
[Source](../../src/optionstratlib/curves/visualization/plotters.rs.html#307-418){.src
.rightside}[§](#impl-PlotBuilderExt%3CVec%3CCurve%3E%3E-for-PlotBuilder%3CVec%3CCurve%3E%3E){.anchor}

### impl [PlotBuilderExt](trait.PlotBuilderExt.html "trait optionstratlib::geometrics::PlotBuilderExt"){.trait}\<[Vec](https://doc.rust-lang.org/1.86.0/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<[Curve](../curves/struct.Curve.html "struct optionstratlib::curves::Curve"){.struct}\>\> for [PlotBuilder](struct.PlotBuilder.html "struct optionstratlib::geometrics::PlotBuilder"){.struct}\<[Vec](https://doc.rust-lang.org/1.86.0/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<[Curve](../curves/struct.Curve.html "struct optionstratlib::curves::Curve"){.struct}\>\> {#impl-plotbuilderextveccurve-for-plotbuilderveccurve .code-header}

::: docblock
Implementation of the `PlotBuilderExt` trait for
`PlotBuilder<Vec<Curve>>`.
:::
::::

::: docblock
This implementation allows saving a plot to a file by utilizing the
`plotters` library. The `save` method takes a file path as input and
generates a plot based on the data and configuration options provided in
the `PlotBuilder`.

#### [§](#functionality){.doc-anchor}Functionality

- **Curve Points Preparation**: It iterates over the curve data (`data`)
  and transforms the points into a collection of `(f64, f64)` tuples,
  which are compatible with the `plotters` library.
- **Plot Range Calculation**: Determines the plot's x and y axis ranges
  by collecting minimum and maximum values across all the curve points.
- **Plot Rendering**: The method sets up a plot with custom title, axis
  labels, line colors, line widths, and other visual properties defined
  in `PlotOptions`.
- **Curve Drawing**: Each curve is drawn using the `LineSeries` feature
  from `plotters`. A unique color is assigned to each curve, repeated
  cyclically if the number of curves exceeds the number of available
  colors in the palette.
- **Legend Display**: A legend is added to the plot using the series'
  labels.
- **Error Handling**: The method handles unexpected errors during chart
  creation, curve rendering, or plot saving, by propagating them as
  `CurvesError` instances.

#### [§](#parameters-11){.doc-anchor}Parameters

- **`self`**: The `PlotBuilder` instance containing the curve data
  (`data`) and configuration options (`options`).
- **`path`**: A path to the file where the plot will be saved. This path
  can be provided as any value that implements the `AsRef<Path>` trait.

#### [§](#return-value){.doc-anchor}Return Value

- Returns `Ok(())` on success, indicating that the plot was saved
  successfully.
- Returns `Err(CurvesError)` on failure, encapsulating the failure
  reason as a string.

#### [§](#dependencies){.doc-anchor}Dependencies

- Uses the `plotters` library for rendering the plot.
- Leverages utility methods like `.fold()`, `.iter()`, and `.map()` to
  process curve data.
- Relies on `self.options` for plot customization (e.g., width, height,
  colors, etc.).

#### [§](#error-handling){.doc-anchor}Error Handling

Any errors encountered during the plot creation or file save process are
encapsulated as `CurvesError` with a `StdError` variant and a
descriptive error message.

#### [§](#algorithm){.doc-anchor}Algorithm

1.  **Fetch Curve Points**: Convert the curves' `Point2D` instances to
    `(f64, f64)` tuples. Use `to_f64` conversion for high precision.
2.  **Calculate Axis Ranges**: Find minimum (`x_min`, `y_min`) and
    maximum (`x_max`, `y_max`) values for x and y axes across all curve
    points.
3.  **Set Up Plot**: Create the drawing area using `BitMapBackend` with
    the specified dimensions and background color in `options`.
4.  **Configure Chart**: Use `ChartBuilder` to define margins, axis
    labels, and title.
5.  **Draw Axes**: Configure and draw the x and y axes with proper
    labels and formatting.
6.  **Draw Curves**: Iterate through the prepared curve points and draw
    each curve with a distinct color.
7.  **Add Legend**: Add a legend area showing the labels for each curve.
8.  **Save Plot**: Serialize and save the plot to the specified file
    path, returning any errors if encountered.

#### [§](#usage-considerations){.doc-anchor}Usage Considerations

- The `self.options.line_colors` must contain enough colors to
  accommodate all curves. If fewer colors are specified, the colors will
  repeat cyclically.
- The `background_color` and `line_width` options affect the overall
  appearance.
- The success of the plot rendering depends on valid and well-formed
  curve data (`Vec<Curve>`).

#### [§](#examples-of-dependencies){.doc-anchor}Examples of Dependencies

- **Associated Traits**: Must be used with the `PlotBuilder` struct and
  a compatible `Vec<Curve>` data type.
- **Color Palettes**: The `PlotOptions::default_colors` method provides
  a default color palette.

#### [§](#related-types){.doc-anchor}Related Types

- **`PlotBuilder`**: Used to encapsulate curve data and configuration
  options.
- **`PlotOptions`**: Provides visual and layout customization for the
  plot.
- **`CurvesError`**: Represents errors that can occur while saving the
  plot.

#### [§](#remarks){.doc-anchor}Remarks

- The method is tightly integrated with `plotters` and uses its core
  components (`BitMapBackend`, `ChartBuilder`, `LineSeries`, etc.) for
  chart creation.
- The precision of `Point2D::x` and `Point2D::y` values is preserved by
  converting them from `Decimal` to `f64` when plotting.
:::

::::: impl-items
::: {#method.save-2 .section .method .trait-impl}
[Source](../../src/optionstratlib/curves/visualization/plotters.rs.html#308-417){.src
.rightside}[§](#method.save-2){.anchor}

#### fn [save](trait.PlotBuilderExt.html#tymethod.save){.fn}(self, path: impl [AsRef](https://doc.rust-lang.org/1.86.0/core/convert/trait.AsRef.html "trait core::convert::AsRef"){.trait}\<[Path](https://doc.rust-lang.org/1.86.0/std/path/struct.Path.html "struct std::path::Path"){.struct}\>) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[()](https://doc.rust-lang.org/1.86.0/std/primitive.unit.html){.primitive}, [CurveError](../error/curves/enum.CurveError.html "enum optionstratlib::error::curves::CurveError"){.enum}\> {#fn-saveself-path-impl-asrefpath---result-curveerror-1 .code-header}
:::

::: docblock
Saves the configured plot to a file at the specified path. [Read
more](trait.PlotBuilderExt.html#tymethod.save)
:::
:::::
:::::::::::::::::::

## Auto Trait Implementations[§](#synthetic-implementations){.anchor} {#synthetic-implementations .section-header}

::::::::::::::: {#synthetic-implementations-list}
:::: {#impl-Freeze-for-PlotBuilder%3CT%3E .section .impl}
[§](#impl-Freeze-for-PlotBuilder%3CT%3E){.anchor}

### impl\<T\> [Freeze](https://doc.rust-lang.org/1.86.0/core/marker/trait.Freeze.html "trait core::marker::Freeze"){.trait} for [PlotBuilder](struct.PlotBuilder.html "struct optionstratlib::geometrics::PlotBuilder"){.struct}\<T\> {#implt-freeze-for-plotbuildert .code-header}

::: where
where T:
[Freeze](https://doc.rust-lang.org/1.86.0/core/marker/trait.Freeze.html "trait core::marker::Freeze"){.trait},
:::
::::

:::: {#impl-RefUnwindSafe-for-PlotBuilder%3CT%3E .section .impl}
[§](#impl-RefUnwindSafe-for-PlotBuilder%3CT%3E){.anchor}

### impl\<T\> [RefUnwindSafe](https://doc.rust-lang.org/1.86.0/core/panic/unwind_safe/trait.RefUnwindSafe.html "trait core::panic::unwind_safe::RefUnwindSafe"){.trait} for [PlotBuilder](struct.PlotBuilder.html "struct optionstratlib::geometrics::PlotBuilder"){.struct}\<T\> {#implt-refunwindsafe-for-plotbuildert .code-header}

::: where
where T:
[RefUnwindSafe](https://doc.rust-lang.org/1.86.0/core/panic/unwind_safe/trait.RefUnwindSafe.html "trait core::panic::unwind_safe::RefUnwindSafe"){.trait},
:::
::::

:::: {#impl-Send-for-PlotBuilder%3CT%3E .section .impl}
[§](#impl-Send-for-PlotBuilder%3CT%3E){.anchor}

### impl\<T\> [Send](https://doc.rust-lang.org/1.86.0/core/marker/trait.Send.html "trait core::marker::Send"){.trait} for [PlotBuilder](struct.PlotBuilder.html "struct optionstratlib::geometrics::PlotBuilder"){.struct}\<T\> {#implt-send-for-plotbuildert .code-header}

::: where
where T:
[Send](https://doc.rust-lang.org/1.86.0/core/marker/trait.Send.html "trait core::marker::Send"){.trait},
:::
::::

:::: {#impl-Sync-for-PlotBuilder%3CT%3E .section .impl}
[§](#impl-Sync-for-PlotBuilder%3CT%3E){.anchor}

### impl\<T\> [Sync](https://doc.rust-lang.org/1.86.0/core/marker/trait.Sync.html "trait core::marker::Sync"){.trait} for [PlotBuilder](struct.PlotBuilder.html "struct optionstratlib::geometrics::PlotBuilder"){.struct}\<T\> {#implt-sync-for-plotbuildert .code-header}

::: where
where T:
[Sync](https://doc.rust-lang.org/1.86.0/core/marker/trait.Sync.html "trait core::marker::Sync"){.trait},
:::
::::

:::: {#impl-Unpin-for-PlotBuilder%3CT%3E .section .impl}
[§](#impl-Unpin-for-PlotBuilder%3CT%3E){.anchor}

### impl\<T\> [Unpin](https://doc.rust-lang.org/1.86.0/core/marker/trait.Unpin.html "trait core::marker::Unpin"){.trait} for [PlotBuilder](struct.PlotBuilder.html "struct optionstratlib::geometrics::PlotBuilder"){.struct}\<T\> {#implt-unpin-for-plotbuildert .code-header}

::: where
where T:
[Unpin](https://doc.rust-lang.org/1.86.0/core/marker/trait.Unpin.html "trait core::marker::Unpin"){.trait},
:::
::::

:::: {#impl-UnwindSafe-for-PlotBuilder%3CT%3E .section .impl}
[§](#impl-UnwindSafe-for-PlotBuilder%3CT%3E){.anchor}

### impl\<T\> [UnwindSafe](https://doc.rust-lang.org/1.86.0/core/panic/unwind_safe/trait.UnwindSafe.html "trait core::panic::unwind_safe::UnwindSafe"){.trait} for [PlotBuilder](struct.PlotBuilder.html "struct optionstratlib::geometrics::PlotBuilder"){.struct}\<T\> {#implt-unwindsafe-for-plotbuildert .code-header}

::: where
where T:
[UnwindSafe](https://doc.rust-lang.org/1.86.0/core/panic/unwind_safe/trait.UnwindSafe.html "trait core::panic::unwind_safe::UnwindSafe"){.trait},
:::
::::
:::::::::::::::

## Blanket Implementations[§](#blanket-implementations){.anchor} {#blanket-implementations .section-header}

:::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::: {#blanket-implementations-list}
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
::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
:::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
