::::::::::::::::::::::::::::::::::::::::::: width-limiter
:::::::::::::::::::::::::::::::::::::::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../../index.html)::[visualization](../index.html)::[utils](index.html)
:::

# Trait [Graph]{.trait}Copy item path

[[Source](../../../src/optionstratlib/visualization/utils.rs.html#182-290){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub trait Graph: Profit {
    // Required methods
    fn title(&self) -> String;
    fn get_x_values(&self) -> Vec<Positive>;

    // Provided methods
    fn graph(
        &self,
        backend: GraphBackend<'_>,
        title_size: u32,
    ) -> Result<(), Box<dyn Error>> { ... }
    fn get_y_values(&self) -> Vec<f64> { ... }
    fn get_vertical_lines(&self) -> Vec<ChartVerticalLine<f64, f64>> { ... }
    fn get_points(&self) -> Vec<ChartPoint<(f64, f64)>> { ... }
}
```

Expand description

::: docblock
Trait for creating graphs of profit calculations. This trait extends the
`Profit` trait, adding the functionality to visualize profit
calculations.
:::

## Required Methods[§](#required-methods){.anchor} {#required-methods .section-header}

::::::: methods
::: {#tymethod.title .section .method}
[Source](../../../src/optionstratlib/visualization/utils.rs.html#243){.src
.rightside}

#### fn [title](#tymethod.title){.fn}(&self) -\> [String](https://doc.rust-lang.org/1.86.0/alloc/string/struct.String.html "struct alloc::string::String"){.struct} {#fn-titleself---string .code-header}
:::

::: docblock
Returns the title of the graph.
:::

::: {#tymethod.get_x_values .section .method}
[Source](../../../src/optionstratlib/visualization/utils.rs.html#258){.src
.rightside}

#### fn [get_x_values](#tymethod.get_x_values){.fn}(&self) -\> [Vec](https://doc.rust-lang.org/1.86.0/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\> {#fn-get_x_valuesself---vecpositive .code-header}
:::

::: docblock
Returns a collection of positive X values for visualization.

This method extracts the X-axis values that will be used for graphing or
plotting financial data. The returned values are guaranteed to be
positive through the `Positive` type wrapper, making them suitable for
financial visualizations where negative X values would be meaningless or
invalid.

##### [§](#returns){.doc-anchor}Returns

A `Vec<Positive>` containing all X-coordinate values to be used in
visualization. These values might represent time points, strike prices,
or other relevant financial metrics depending on the context.
:::
:::::::

## Provided Methods[§](#provided-methods){.anchor} {#provided-methods .section-header}

::::::::::: methods
::: {#method.graph .section .method}
[Source](../../../src/optionstratlib/visualization/utils.rs.html#198-240){.src
.rightside}

#### fn [graph](#method.graph){.fn}( &self, backend: [GraphBackend](enum.GraphBackend.html "enum optionstratlib::visualization::utils::GraphBackend"){.enum}\<\'\_\>, title_size: [u32](https://doc.rust-lang.org/1.86.0/std/primitive.u32.html){.primitive}, ) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[()](https://doc.rust-lang.org/1.86.0/std/primitive.unit.html){.primitive}, [Box](https://doc.rust-lang.org/1.86.0/alloc/boxed/struct.Box.html "struct alloc::boxed::Box"){.struct}\<dyn [Error](https://doc.rust-lang.org/1.86.0/core/error/trait.Error.html "trait core::error::Error"){.trait}\>\> {#fn-graph-self-backend-graphbackend_-title_size-u32---result-boxdyn-error .code-header}
:::

::: docblock
Generates a graph of profit calculations.

##### [§](#arguments){.doc-anchor}Arguments

- `x_axis_data` - A slice of `Positive` values representing the x-axis
  data points (e.g., prices).
- `backend` - The `GraphBackend` to use for rendering. This determines
  whether the graph is rendered to a bitmap file or a canvas element.
- `title_size` - The font size for the graph title.

##### [§](#errors){.doc-anchor}Errors

Returns an error if:

- `x_axis_data` is empty.
- No valid y-axis values could be calculated (e.g., all calculations
  resulted in errors).
- There is an issue during graph creation or rendering with the chosen
  backend.
:::

::: {#method.get_y_values .section .method}
[Source](../../../src/optionstratlib/visualization/utils.rs.html#269-279){.src
.rightside}

#### fn [get_y_values](#method.get_y_values){.fn}(&self) -\> [Vec](https://doc.rust-lang.org/1.86.0/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<[f64](https://doc.rust-lang.org/1.86.0/std/primitive.f64.html){.primitive}\> {#fn-get_y_valuesself---vecf64 .code-header}
:::

::: docblock
Calculates the y-axis values (profit) corresponding to the provided
x-axis data.

##### [§](#arguments-1){.doc-anchor}Arguments

- `data` - A slice of `Positive` values representing the x-axis data
  points.

##### [§](#returns-1){.doc-anchor}Returns

A vector of `f64` representing the calculated profit values.
:::

::: {#method.get_vertical_lines .section .method}
[Source](../../../src/optionstratlib/visualization/utils.rs.html#282-284){.src
.rightside}

#### fn [get_vertical_lines](#method.get_vertical_lines){.fn}(&self) -\> [Vec](https://doc.rust-lang.org/1.86.0/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<ChartVerticalLine\<[f64](https://doc.rust-lang.org/1.86.0/std/primitive.f64.html){.primitive}, [f64](https://doc.rust-lang.org/1.86.0/std/primitive.f64.html){.primitive}\>\> {#fn-get_vertical_linesself---vecchartverticallinef64-f64 .code-header}
:::

::: docblock
Returns a vector of vertical lines to draw on the chart. Default
implementation returns an empty vector.
:::

::: {#method.get_points .section .method}
[Source](../../../src/optionstratlib/visualization/utils.rs.html#287-289){.src
.rightside}

#### fn [get_points](#method.get_points){.fn}(&self) -\> [Vec](https://doc.rust-lang.org/1.86.0/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<ChartPoint\<([f64](https://doc.rust-lang.org/1.86.0/std/primitive.f64.html){.primitive}, [f64](https://doc.rust-lang.org/1.86.0/std/primitive.f64.html){.primitive})\>\> {#fn-get_pointsself---vecchartpointf64-f64 .code-header}
:::

::: docblock
Returns a vector of points to draw on the chart. Default implementation
returns an empty vector.
:::
:::::::::::

## Implementors[§](#implementors){.anchor} {#implementors .section-header}

:::::::::::::::::::::::: {#implementors-list}
::: {#impl-Graph-for-Options .section .impl}
[Source](../../../src/optionstratlib/model/option.rs.html#730-792){.src
.rightside}[§](#impl-Graph-for-Options){.anchor}

### impl [Graph](trait.Graph.html "trait optionstratlib::visualization::utils::Graph"){.trait} for [Options](../../model/option/struct.Options.html "struct optionstratlib::model::option::Options"){.struct} {#impl-graph-for-options .code-header}
:::

:::: {#impl-Graph-for-Position .section .impl}
[Source](../../../src/optionstratlib/model/position.rs.html#800-877){.src
.rightside}[§](#impl-Graph-for-Position){.anchor}

### impl [Graph](trait.Graph.html "trait optionstratlib::visualization::utils::Graph"){.trait} for [Position](../../model/position/struct.Position.html "struct optionstratlib::model::position::Position"){.struct} {#impl-graph-for-position .code-header}

::: docblock
Implementation of the `Graph` trait for the `Position` struct, enabling
graphical representation of financial options positions.
:::
::::

::: docblock
This implementation provides methods to visualize the profit/loss (PnL)
profile of an options position across different price levels of the
underlying asset. It handles the generation of appropriate title, data
values for plotting, and special chart elements like break-even points.

The visualization capabilities allow traders to analyze the potential
outcomes of their options positions at expiration across various price
scenarios.
:::

::: {#impl-Graph-for-BearCallSpread .section .impl}
[Source](../../../src/optionstratlib/strategies/bear_call_spread.rs.html#640-715){.src
.rightside}[§](#impl-Graph-for-BearCallSpread){.anchor}

### impl [Graph](trait.Graph.html "trait optionstratlib::visualization::utils::Graph"){.trait} for [BearCallSpread](../../strategies/bear_call_spread/struct.BearCallSpread.html "struct optionstratlib::strategies::bear_call_spread::BearCallSpread"){.struct} {#impl-graph-for-bearcallspread .code-header}
:::

::: {#impl-Graph-for-BearPutSpread .section .impl}
[Source](../../../src/optionstratlib/strategies/bear_put_spread.rs.html#618-697){.src
.rightside}[§](#impl-Graph-for-BearPutSpread){.anchor}

### impl [Graph](trait.Graph.html "trait optionstratlib::visualization::utils::Graph"){.trait} for [BearPutSpread](../../strategies/bear_put_spread/struct.BearPutSpread.html "struct optionstratlib::strategies::bear_put_spread::BearPutSpread"){.struct} {#impl-graph-for-bearputspread .code-header}
:::

::: {#impl-Graph-for-BullCallSpread .section .impl}
[Source](../../../src/optionstratlib/strategies/bull_call_spread.rs.html#631-710){.src
.rightside}[§](#impl-Graph-for-BullCallSpread){.anchor}

### impl [Graph](trait.Graph.html "trait optionstratlib::visualization::utils::Graph"){.trait} for [BullCallSpread](../../strategies/bull_call_spread/struct.BullCallSpread.html "struct optionstratlib::strategies::bull_call_spread::BullCallSpread"){.struct} {#impl-graph-for-bullcallspread .code-header}
:::

::: {#impl-Graph-for-BullPutSpread .section .impl}
[Source](../../../src/optionstratlib/strategies/bull_put_spread.rs.html#732-811){.src
.rightside}[§](#impl-Graph-for-BullPutSpread){.anchor}

### impl [Graph](trait.Graph.html "trait optionstratlib::visualization::utils::Graph"){.trait} for [BullPutSpread](../../strategies/bull_put_spread/struct.BullPutSpread.html "struct optionstratlib::strategies::bull_put_spread::BullPutSpread"){.struct} {#impl-graph-for-bullputspread .code-header}
:::

::: {#impl-Graph-for-LongButterflySpread .section .impl}
[Source](../../../src/optionstratlib/strategies/butterfly_spread.rs.html#739-875){.src
.rightside}[§](#impl-Graph-for-LongButterflySpread){.anchor}

### impl [Graph](trait.Graph.html "trait optionstratlib::visualization::utils::Graph"){.trait} for [LongButterflySpread](../../strategies/butterfly_spread/struct.LongButterflySpread.html "struct optionstratlib::strategies::butterfly_spread::LongButterflySpread"){.struct} {#impl-graph-for-longbutterflyspread .code-header}
:::

::: {#impl-Graph-for-ShortButterflySpread .section .impl}
[Source](../../../src/optionstratlib/strategies/butterfly_spread.rs.html#1708-1842){.src
.rightside}[§](#impl-Graph-for-ShortButterflySpread){.anchor}

### impl [Graph](trait.Graph.html "trait optionstratlib::visualization::utils::Graph"){.trait} for [ShortButterflySpread](../../strategies/butterfly_spread/struct.ShortButterflySpread.html "struct optionstratlib::strategies::butterfly_spread::ShortButterflySpread"){.struct} {#impl-graph-for-shortbutterflyspread .code-header}
:::

::: {#impl-Graph-for-CallButterfly .section .impl}
[Source](../../../src/optionstratlib/strategies/call_butterfly.rs.html#743-861){.src
.rightside}[§](#impl-Graph-for-CallButterfly){.anchor}

### impl [Graph](trait.Graph.html "trait optionstratlib::visualization::utils::Graph"){.trait} for [CallButterfly](../../strategies/call_butterfly/struct.CallButterfly.html "struct optionstratlib::strategies::call_butterfly::CallButterfly"){.struct} {#impl-graph-for-callbutterfly .code-header}
:::

::: {#impl-Graph-for-CustomStrategy .section .impl}
[Source](../../../src/optionstratlib/strategies/custom.rs.html#587-666){.src
.rightside}[§](#impl-Graph-for-CustomStrategy){.anchor}

### impl [Graph](trait.Graph.html "trait optionstratlib::visualization::utils::Graph"){.trait} for [CustomStrategy](../../strategies/custom/struct.CustomStrategy.html "struct optionstratlib::strategies::custom::CustomStrategy"){.struct} {#impl-graph-for-customstrategy .code-header}
:::

::: {#impl-Graph-for-IronButterfly .section .impl}
[Source](../../../src/optionstratlib/strategies/iron_butterfly.rs.html#787-924){.src
.rightside}[§](#impl-Graph-for-IronButterfly){.anchor}

### impl [Graph](trait.Graph.html "trait optionstratlib::visualization::utils::Graph"){.trait} for [IronButterfly](../../strategies/iron_butterfly/struct.IronButterfly.html "struct optionstratlib::strategies::iron_butterfly::IronButterfly"){.struct} {#impl-graph-for-ironbutterfly .code-header}
:::

::: {#impl-Graph-for-IronCondor .section .impl}
[Source](../../../src/optionstratlib/strategies/iron_condor.rs.html#802-959){.src
.rightside}[§](#impl-Graph-for-IronCondor){.anchor}

### impl [Graph](trait.Graph.html "trait optionstratlib::visualization::utils::Graph"){.trait} for [IronCondor](../../strategies/iron_condor/struct.IronCondor.html "struct optionstratlib::strategies::iron_condor::IronCondor"){.struct} {#impl-graph-for-ironcondor .code-header}
:::

::: {#impl-Graph-for-PoorMansCoveredCall .section .impl}
[Source](../../../src/optionstratlib/strategies/poor_mans_covered_call.rs.html#631-743){.src
.rightside}[§](#impl-Graph-for-PoorMansCoveredCall){.anchor}

### impl [Graph](trait.Graph.html "trait optionstratlib::visualization::utils::Graph"){.trait} for [PoorMansCoveredCall](../../strategies/poor_mans_covered_call/struct.PoorMansCoveredCall.html "struct optionstratlib::strategies::poor_mans_covered_call::PoorMansCoveredCall"){.struct} {#impl-graph-for-poormanscoveredcall .code-header}
:::

::: {#impl-Graph-for-LongStraddle .section .impl}
[Source](../../../src/optionstratlib/strategies/straddle.rs.html#1443-1528){.src
.rightside}[§](#impl-Graph-for-LongStraddle){.anchor}

### impl [Graph](trait.Graph.html "trait optionstratlib::visualization::utils::Graph"){.trait} for [LongStraddle](../../strategies/straddle/struct.LongStraddle.html "struct optionstratlib::strategies::straddle::LongStraddle"){.struct} {#impl-graph-for-longstraddle .code-header}
:::

::: {#impl-Graph-for-ShortStraddle .section .impl}
[Source](../../../src/optionstratlib/strategies/straddle.rs.html#660-748){.src
.rightside}[§](#impl-Graph-for-ShortStraddle){.anchor}

### impl [Graph](trait.Graph.html "trait optionstratlib::visualization::utils::Graph"){.trait} for [ShortStraddle](../../strategies/straddle/struct.ShortStraddle.html "struct optionstratlib::strategies::straddle::ShortStraddle"){.struct} {#impl-graph-for-shortstraddle .code-header}
:::

::: {#impl-Graph-for-LongStrangle .section .impl}
[Source](../../../src/optionstratlib/strategies/strangle.rs.html#1711-1832){.src
.rightside}[§](#impl-Graph-for-LongStrangle){.anchor}

### impl [Graph](trait.Graph.html "trait optionstratlib::visualization::utils::Graph"){.trait} for [LongStrangle](../../strategies/strangle/struct.LongStrangle.html "struct optionstratlib::strategies::strangle::LongStrangle"){.struct} {#impl-graph-for-longstrangle .code-header}
:::

::: {#impl-Graph-for-ShortStrangle .section .impl}
[Source](../../../src/optionstratlib/strategies/strangle.rs.html#725-844){.src
.rightside}[§](#impl-Graph-for-ShortStrangle){.anchor}

### impl [Graph](trait.Graph.html "trait optionstratlib::visualization::utils::Graph"){.trait} for [ShortStrangle](../../strategies/strangle/struct.ShortStrangle.html "struct optionstratlib::strategies::strangle::ShortStrangle"){.struct} {#impl-graph-for-shortstrangle .code-header}
:::

:::: {#impl-Graph-for-RandomWalk%3CX,+Y%3E .section .impl}
[Source](../../../src/optionstratlib/simulation/randomwalk.rs.html#285-308){.src
.rightside}[§](#impl-Graph-for-RandomWalk%3CX,+Y%3E){.anchor}

### impl\<X, Y\> [Graph](trait.Graph.html "trait optionstratlib::visualization::utils::Graph"){.trait} for [RandomWalk](../../simulation/randomwalk/struct.RandomWalk.html "struct optionstratlib::simulation::randomwalk::RandomWalk"){.struct}\<X, Y\> {#implx-y-graph-for-randomwalkx-y .code-header}

::: where
where X:
[Copy](https://doc.rust-lang.org/1.86.0/core/marker/trait.Copy.html "trait core::marker::Copy"){.trait} +
[Into](https://doc.rust-lang.org/1.86.0/core/convert/trait.Into.html "trait core::convert::Into"){.trait}\<[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\> +
[AddAssign](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.AddAssign.html "trait core::ops::arith::AddAssign"){.trait} +
[Display](https://doc.rust-lang.org/1.86.0/core/fmt/trait.Display.html "trait core::fmt::Display"){.trait},
Y:
[Into](https://doc.rust-lang.org/1.86.0/core/convert/trait.Into.html "trait core::convert::Into"){.trait}\<[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\> +
[Display](https://doc.rust-lang.org/1.86.0/core/fmt/trait.Display.html "trait core::fmt::Display"){.trait} +
[Clone](https://doc.rust-lang.org/1.86.0/core/clone/trait.Clone.html "trait core::clone::Clone"){.trait},
:::
::::
::::::::::::::::::::::::
::::::::::::::::::::::::::::::::::::::::::
:::::::::::::::::::::::::::::::::::::::::::
