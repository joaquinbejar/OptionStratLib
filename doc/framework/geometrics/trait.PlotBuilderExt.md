::::::::::::::::::: width-limiter
:::::::::::::::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../index.html)::[geometrics](index.html)
:::

# Trait [PlotBuilderExt]{.trait}Copy item path

[[Source](../../src/optionstratlib/geometrics/visualization/plotters.rs.html#322-340){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub trait PlotBuilderExt<T: Plottable> {
    // Required method
    fn save(self, path: impl AsRef<Path>) -> Result<(), T::Error>;
}
```

Expand description

::: docblock
Extension methods for the plot building process.

This trait extends the `PlotBuilder` functionality to provide methods
for outputting and saving plots. It serves as the final step in the plot
creation pipeline after configuring visualization options.

`PlotBuilderExt` complements the builder pattern used in the plotting
system by providing output capabilities that work with any type
implementing the `Plottable` trait. This separation of concerns allows
for a clean interface where plot configuration and rendering/output are
logically separated.
:::

## Required Methods[§](#required-methods){.anchor} {#required-methods .section-header}

::::: methods
::: {#tymethod.save .section .method}
[Source](../../src/optionstratlib/geometrics/visualization/plotters.rs.html#339){.src
.rightside}

#### fn [save](#tymethod.save){.fn}(self, path: impl [AsRef](https://doc.rust-lang.org/1.86.0/core/convert/trait.AsRef.html "trait core::convert::AsRef"){.trait}\<[Path](https://doc.rust-lang.org/1.86.0/std/path/struct.Path.html "struct std::path::Path"){.struct}\>) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[()](https://doc.rust-lang.org/1.86.0/std/primitive.unit.html){.primitive}, T::[Error](trait.Plottable.html#associatedtype.Error "type optionstratlib::geometrics::Plottable::Error"){.associatedtype}\> {#fn-saveself-path-impl-asrefpath---result-terror .code-header}
:::

::: docblock
Saves the configured plot to a file at the specified path.

This method renders the plot with all configured options and writes the
resulting visualization to the given file path. The file format is
determined by the path's extension (e.g., .png, .svg).

##### [§](#arguments){.doc-anchor}Arguments

- `path` - The file path where the plot should be saved. Can be any type
  that can be converted to a `Path`.

##### [§](#returns){.doc-anchor}Returns

- `Ok(())` if the plot was successfully saved
- `Err(T::Error)` if an error occurred during rendering or saving
:::
:::::

## Dyn Compatibility[§](#dyn-compatibility){.anchor} {#dyn-compatibility .section-header}

::: dyn-compatibility-info
This trait is **not** [dyn
compatible](https://doc.rust-lang.org/1.86.0/reference/items/traits.html#dyn-compatibility).

*In older versions of Rust, dyn compatibility was called \"object
safety\", so this trait is not object safe.*
:::

## Implementors[§](#implementors){.anchor} {#implementors .section-header}

:::::::::: {#implementors-list}
:::: {#impl-PlotBuilderExt%3CCurve%3E-for-PlotBuilder%3CCurve%3E .section .impl}
[Source](../../src/optionstratlib/curves/visualization/plotters.rs.html#153-233){.src
.rightside}[§](#impl-PlotBuilderExt%3CCurve%3E-for-PlotBuilder%3CCurve%3E){.anchor}

### impl [PlotBuilderExt](trait.PlotBuilderExt.html "trait optionstratlib::geometrics::PlotBuilderExt"){.trait}\<[Curve](../curves/struct.Curve.html "struct optionstratlib::curves::Curve"){.struct}\> for [PlotBuilder](struct.PlotBuilder.html "struct optionstratlib::geometrics::PlotBuilder"){.struct}\<[Curve](../curves/struct.Curve.html "struct optionstratlib::curves::Curve"){.struct}\> {#impl-plotbuilderextcurve-for-plotbuildercurve .code-header}

::: docblock
Plotting implementation for single Curve
:::
::::

:::: {#impl-PlotBuilderExt%3CSurface%3E-for-PlotBuilder%3CSurface%3E .section .impl}
[Source](../../src/optionstratlib/surfaces/visualization/plotters.rs.html#150-275){.src
.rightside}[§](#impl-PlotBuilderExt%3CSurface%3E-for-PlotBuilder%3CSurface%3E){.anchor}

### impl [PlotBuilderExt](trait.PlotBuilderExt.html "trait optionstratlib::geometrics::PlotBuilderExt"){.trait}\<[Surface](../surfaces/struct.Surface.html "struct optionstratlib::surfaces::Surface"){.struct}\> for [PlotBuilder](struct.PlotBuilder.html "struct optionstratlib::geometrics::PlotBuilder"){.struct}\<[Surface](../surfaces/struct.Surface.html "struct optionstratlib::surfaces::Surface"){.struct}\> {#impl-plotbuilderextsurface-for-plotbuildersurface .code-header}

::: docblock
Plotting implementation for single Surface
:::
::::

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

#### [§](#parameters){.doc-anchor}Parameters

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
::::::::::
::::::::::::::::::
:::::::::::::::::::
