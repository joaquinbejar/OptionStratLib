:::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::: width-limiter
::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../../index.html)::[simulation](../index.html)::[randomwalk](index.html)
:::

# Struct [RandomWalk]{.struct}Copy item path

[[Source](../../../src/optionstratlib/simulation/randomwalk.rs.html#33-43){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub struct RandomWalk<X, Y>where
    X: Copy + Into<Positive> + AddAssign + Display,
    Y: Into<Positive> + Display + Clone,{ /* private fields */ }
```

Expand description

::: docblock
A struct that represents a two-dimensional random walk simulation.

`RandomWalk` stores a sequence of steps that describe a path in a
two-dimensional space, typically used for financial modeling, time
series analysis, or statistical simulations. It maintains both the steps
of the random walk and a descriptive title.

## [§](#type-parameters){.doc-anchor}Type Parameters

- `X` - The type for x-axis values (typically representing time or
  sequence position), which must implement `AddAssign` (allowing values
  to be accumulated), be convertible to `Positive`, and be `Copy`.

- `Y` - The type for y-axis values (typically representing price, value,
  or position), which must implement `AddAssign`, be convertible to
  `Positive`, be `Copy`, and implement the `Walktypable` trait for
  additional functionality.
:::

## Implementations[§](#implementations){.anchor} {#implementations .section-header}

::::::::::::::::::::::: {#implementations-list}
:::: {#impl-RandomWalk%3CX,+Y%3E .section .impl}
[Source](../../../src/optionstratlib/simulation/randomwalk.rs.html#45-155){.src
.rightside}[§](#impl-RandomWalk%3CX,+Y%3E){.anchor}

### impl\<X, Y\> [RandomWalk](struct.RandomWalk.html "struct optionstratlib::simulation::randomwalk::RandomWalk"){.struct}\<X, Y\> {#implx-y-randomwalkx-y .code-header}

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

:::::::::::::::::::: impl-items
:::: {#method.new .section .method}
[Source](../../../src/optionstratlib/simulation/randomwalk.rs.html#65-73){.src
.rightside}

#### pub fn [new](#method.new){.fn}\<F\>(title: [String](https://doc.rust-lang.org/1.86.0/alloc/string/struct.String.html "struct alloc::string::String"){.struct}, params: &[WalkParams](../struct.WalkParams.html "struct optionstratlib::simulation::WalkParams"){.struct}\<X, Y\>, generator: F) -\> Self {#pub-fn-newftitle-string-params-walkparamsx-y-generator-f---self .code-header}

::: where
where F:
[FnOnce](https://doc.rust-lang.org/1.86.0/core/ops/function/trait.FnOnce.html "trait core::ops::function::FnOnce"){.trait}(&[WalkParams](../struct.WalkParams.html "struct optionstratlib::simulation::WalkParams"){.struct}\<X,
Y\>) -\>
[Vec](https://doc.rust-lang.org/1.86.0/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<[Step](../steps/struct.Step.html "struct optionstratlib::simulation::steps::Step"){.struct}\<X,
Y\>\>, X:
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

::: docblock
Creates a new random walk instance with the given title and steps.

This constructor takes a title, walk parameters, and a generator
function that produces the actual steps of the random walk based on the
provided parameters.

##### [§](#parameters){.doc-anchor}Parameters

- `title` - A descriptive title for the random walk
- `params` - Parameters that define the properties of the random walk
- `generator` - A function that generates the steps of the random walk

##### [§](#returns){.doc-anchor}Returns

A new `RandomWalk` instance with the generated steps.
:::

::: {#method.get_title .section .method}
[Source](../../../src/optionstratlib/simulation/randomwalk.rs.html#80-82){.src
.rightside}

#### pub fn [get_title](#method.get_title){.fn}(&self) -\> &[str](https://doc.rust-lang.org/1.86.0/std/primitive.str.html){.primitive} {#pub-fn-get_titleself---str .code-header}
:::

::: docblock
Returns the title of the random walk.

##### [§](#returns-1){.doc-anchor}Returns

A string slice containing the title of the random walk.
:::

::: {#method.set_title .section .method}
[Source](../../../src/optionstratlib/simulation/randomwalk.rs.html#89-91){.src
.rightside}

#### pub fn [set_title](#method.set_title){.fn}(&mut self, title: [String](https://doc.rust-lang.org/1.86.0/alloc/string/struct.String.html "struct alloc::string::String"){.struct}) {#pub-fn-set_titlemut-self-title-string .code-header}
:::

::: docblock
Updates the title of the random walk.

##### [§](#parameters-1){.doc-anchor}Parameters

- `title` - The new title to set
:::

::: {#method.get_steps .section .method}
[Source](../../../src/optionstratlib/simulation/randomwalk.rs.html#98-100){.src
.rightside}

#### pub fn [get_steps](#method.get_steps){.fn}(&self) -\> [Vec](https://doc.rust-lang.org/1.86.0/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<&[Step](../steps/struct.Step.html "struct optionstratlib::simulation::steps::Step"){.struct}\<X, Y\>\> {#pub-fn-get_stepsself---vecstepx-y .code-header}
:::

::: docblock
Returns a vector of references to all steps in the random walk.

##### [§](#returns-2){.doc-anchor}Returns

A vector containing references to all steps in the random walk.
:::

::: {#method.get_step .section .method}
[Source](../../../src/optionstratlib/simulation/randomwalk.rs.html#115-117){.src
.rightside}

#### pub fn [get_step](#method.get_step){.fn}(&self, index: [usize](https://doc.rust-lang.org/1.86.0/std/primitive.usize.html){.primitive}) -\> &[Step](../steps/struct.Step.html "struct optionstratlib::simulation::steps::Step"){.struct}\<X, Y\> {#pub-fn-get_stepself-index-usize---stepx-y .code-header}
:::

::: docblock
Returns a reference to the step at the specified index.

##### [§](#parameters-2){.doc-anchor}Parameters

- `index` - The zero-based index of the step to retrieve

##### [§](#returns-3){.doc-anchor}Returns

A reference to the step at the specified index.

##### [§](#panics){.doc-anchor}Panics

Panics if the index is out of bounds.
:::

::: {#method.get_step_mut .section .method}
[Source](../../../src/optionstratlib/simulation/randomwalk.rs.html#132-134){.src
.rightside}

#### pub fn [get_step_mut](#method.get_step_mut){.fn}(&mut self, index: [usize](https://doc.rust-lang.org/1.86.0/std/primitive.usize.html){.primitive}) -\> &mut [Step](../steps/struct.Step.html "struct optionstratlib::simulation::steps::Step"){.struct}\<X, Y\> {#pub-fn-get_step_mutmut-self-index-usize---mut-stepx-y .code-header}
:::

::: docblock
Returns a mutable reference to the step at the specified index.

##### [§](#parameters-3){.doc-anchor}Parameters

- `index` - The zero-based index of the step to retrieve

##### [§](#returns-4){.doc-anchor}Returns

A mutable reference to the step at the specified index.

##### [§](#panics-1){.doc-anchor}Panics

Panics if the index is out of bounds.
:::

::: {#method.first .section .method}
[Source](../../../src/optionstratlib/simulation/randomwalk.rs.html#142-144){.src
.rightside}

#### pub fn [first](#method.first){.fn}(&self) -\> [Option](https://doc.rust-lang.org/1.86.0/core/option/enum.Option.html "enum core::option::Option"){.enum}\<&[Step](../steps/struct.Step.html "struct optionstratlib::simulation::steps::Step"){.struct}\<X, Y\>\> {#pub-fn-firstself---optionstepx-y .code-header}
:::

::: docblock
Returns a reference to the first step in the random walk, if any.

##### [§](#returns-5){.doc-anchor}Returns

- `Some(&Step<X, Y>)` - A reference to the first step if the random walk
  is not empty
- `None` - If the random walk has no steps
:::

::: {#method.last .section .method}
[Source](../../../src/optionstratlib/simulation/randomwalk.rs.html#152-154){.src
.rightside}

#### pub fn [last](#method.last){.fn}(&self) -\> [Option](https://doc.rust-lang.org/1.86.0/core/option/enum.Option.html "enum core::option::Option"){.enum}\<&[Step](../steps/struct.Step.html "struct optionstratlib::simulation::steps::Step"){.struct}\<X, Y\>\> {#pub-fn-lastself---optionstepx-y .code-header}
:::

::: docblock
Returns a reference to the last step in the random walk, if any.

##### [§](#returns-6){.doc-anchor}Returns

- `Some(&Step<X, Y>)` - A reference to the last step if the random walk
  is not empty
- `None` - If the random walk has no steps
:::
::::::::::::::::::::
:::::::::::::::::::::::

## Trait Implementations[§](#trait-implementations){.anchor} {#trait-implementations .section-header}

::::::::::::::::::::::::::::::::::::::::::::::::::::::: {#trait-implementations-list}
:::: {#impl-Display-for-RandomWalk%3CX,+Y%3E .section .impl}
[Source](../../../src/optionstratlib/simulation/randomwalk.rs.html#261-273){.src
.rightside}[§](#impl-Display-for-RandomWalk%3CX,+Y%3E){.anchor}

### impl\<X, Y\> [Display](https://doc.rust-lang.org/1.86.0/core/fmt/trait.Display.html "trait core::fmt::Display"){.trait} for [RandomWalk](struct.RandomWalk.html "struct optionstratlib::simulation::randomwalk::RandomWalk"){.struct}\<X, Y\> {#implx-y-display-for-randomwalkx-y .code-header}

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

::::: impl-items
::: {#method.fmt .section .method .trait-impl}
[Source](../../../src/optionstratlib/simulation/randomwalk.rs.html#266-272){.src
.rightside}[§](#method.fmt){.anchor}

#### fn [fmt](https://doc.rust-lang.org/1.86.0/core/fmt/trait.Display.html#tymethod.fmt){.fn}(&self, f: &mut [Formatter](https://doc.rust-lang.org/1.86.0/core/fmt/struct.Formatter.html "struct core::fmt::Formatter"){.struct}\<\'\_\>) -\> [Result](https://doc.rust-lang.org/1.86.0/core/fmt/type.Result.html "type core::fmt::Result"){.type} {#fn-fmtself-f-mut-formatter_---result .code-header}
:::

::: docblock
Formats the value using the given formatter. [Read
more](https://doc.rust-lang.org/1.86.0/core/fmt/trait.Display.html#tymethod.fmt)
:::
:::::

:::: {#impl-Graph-for-RandomWalk%3CX,+Y%3E .section .impl}
[Source](../../../src/optionstratlib/simulation/randomwalk.rs.html#285-308){.src
.rightside}[§](#impl-Graph-for-RandomWalk%3CX,+Y%3E){.anchor}

### impl\<X, Y\> [Graph](../../visualization/utils/trait.Graph.html "trait optionstratlib::visualization::utils::Graph"){.trait} for [RandomWalk](struct.RandomWalk.html "struct optionstratlib::simulation::randomwalk::RandomWalk"){.struct}\<X, Y\> {#implx-y-graph-for-randomwalkx-y .code-header}

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

::::::::::::::: impl-items
::: {#method.title .section .method .trait-impl}
[Source](../../../src/optionstratlib/simulation/randomwalk.rs.html#290-292){.src
.rightside}[§](#method.title){.anchor}

#### fn [title](../../visualization/utils/trait.Graph.html#tymethod.title){.fn}(&self) -\> [String](https://doc.rust-lang.org/1.86.0/alloc/string/struct.String.html "struct alloc::string::String"){.struct} {#fn-titleself---string .code-header}
:::

::: docblock
Returns the title of the graph.
:::

::: {#method.get_x_values .section .method .trait-impl}
[Source](../../../src/optionstratlib/simulation/randomwalk.rs.html#294-300){.src
.rightside}[§](#method.get_x_values){.anchor}

#### fn [get_x_values](../../visualization/utils/trait.Graph.html#tymethod.get_x_values){.fn}(&self) -\> [Vec](https://doc.rust-lang.org/1.86.0/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\> {#fn-get_x_valuesself---vecpositive .code-header}
:::

::: docblock
Returns a collection of positive X values for visualization. [Read
more](../../visualization/utils/trait.Graph.html#tymethod.get_x_values)
:::

::: {#method.get_y_values .section .method .trait-impl}
[Source](../../../src/optionstratlib/simulation/randomwalk.rs.html#302-307){.src
.rightside}[§](#method.get_y_values){.anchor}

#### fn [get_y_values](../../visualization/utils/trait.Graph.html#method.get_y_values){.fn}(&self) -\> [Vec](https://doc.rust-lang.org/1.86.0/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<[f64](https://doc.rust-lang.org/1.86.0/std/primitive.f64.html){.primitive}\> {#fn-get_y_valuesself---vecf64 .code-header}
:::

::: docblock
Calculates the y-axis values (profit) corresponding to the provided
x-axis data. [Read
more](../../visualization/utils/trait.Graph.html#method.get_y_values)
:::

::: {#method.graph .section .method .trait-impl}
[Source](../../../src/optionstratlib/visualization/utils.rs.html#198-240){.src
.rightside}[§](#method.graph){.anchor}

#### fn [graph](../../visualization/utils/trait.Graph.html#method.graph){.fn}( &self, backend: [GraphBackend](../../visualization/utils/enum.GraphBackend.html "enum optionstratlib::visualization::utils::GraphBackend"){.enum}\<\'\_\>, title_size: [u32](https://doc.rust-lang.org/1.86.0/std/primitive.u32.html){.primitive}, ) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[()](https://doc.rust-lang.org/1.86.0/std/primitive.unit.html){.primitive}, [Box](https://doc.rust-lang.org/1.86.0/alloc/boxed/struct.Box.html "struct alloc::boxed::Box"){.struct}\<dyn [Error](https://doc.rust-lang.org/1.86.0/core/error/trait.Error.html "trait core::error::Error"){.trait}\>\> {#fn-graph-self-backend-graphbackend_-title_size-u32---result-boxdyn-error .code-header}
:::

::: docblock
Generates a graph of profit calculations. [Read
more](../../visualization/utils/trait.Graph.html#method.graph)
:::

::: {#method.get_vertical_lines .section .method .trait-impl}
[Source](../../../src/optionstratlib/visualization/utils.rs.html#282-284){.src
.rightside}[§](#method.get_vertical_lines){.anchor}

#### fn [get_vertical_lines](../../visualization/utils/trait.Graph.html#method.get_vertical_lines){.fn}(&self) -\> [Vec](https://doc.rust-lang.org/1.86.0/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<ChartVerticalLine\<[f64](https://doc.rust-lang.org/1.86.0/std/primitive.f64.html){.primitive}, [f64](https://doc.rust-lang.org/1.86.0/std/primitive.f64.html){.primitive}\>\> {#fn-get_vertical_linesself---vecchartverticallinef64-f64 .code-header}
:::

::: docblock
Returns a vector of vertical lines to draw on the chart. Default
implementation returns an empty vector.
:::

::: {#method.get_points .section .method .trait-impl}
[Source](../../../src/optionstratlib/visualization/utils.rs.html#287-289){.src
.rightside}[§](#method.get_points){.anchor}

#### fn [get_points](../../visualization/utils/trait.Graph.html#method.get_points){.fn}(&self) -\> [Vec](https://doc.rust-lang.org/1.86.0/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<ChartPoint\<([f64](https://doc.rust-lang.org/1.86.0/std/primitive.f64.html){.primitive}, [f64](https://doc.rust-lang.org/1.86.0/std/primitive.f64.html){.primitive})\>\> {#fn-get_pointsself---vecchartpointf64-f64 .code-header}
:::

::: docblock
Returns a vector of points to draw on the chart. Default implementation
returns an empty vector.
:::
:::::::::::::::

::::: {#impl-Index%3Cusize%3E-for-RandomWalk%3CX,+Y%3E .section .impl}
[Source](../../../src/optionstratlib/simulation/randomwalk.rs.html#203-227){.src
.rightside}[§](#impl-Index%3Cusize%3E-for-RandomWalk%3CX,+Y%3E){.anchor}

### impl\<X, Y\> [Index](https://doc.rust-lang.org/1.86.0/core/ops/index/trait.Index.html "trait core::ops::index::Index"){.trait}\<[usize](https://doc.rust-lang.org/1.86.0/std/primitive.usize.html){.primitive}\> for [RandomWalk](struct.RandomWalk.html "struct optionstratlib::simulation::randomwalk::RandomWalk"){.struct}\<X, Y\> {#implx-y-indexusize-for-randomwalkx-y .code-header}

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

::: docblock
Implementation of the `Index` trait for `RandomWalk<X, Y>`.
:::
:::::

::: docblock
This allows accessing the steps of a random walk using array indexing
notation: `walk[index]`.

#### [§](#type-parameters-2){.doc-anchor}Type Parameters {#type-parameters-2}

- `X` - The type for x-axis values, with constraints as described above.
- `Y` - The type for y-axis values, with constraints as described above.
:::

::::::: impl-items
::: {#associatedtype.Output .section .associatedtype .trait-impl}
[Source](../../../src/optionstratlib/simulation/randomwalk.rs.html#209){.src
.rightside}[§](#associatedtype.Output){.anchor}

#### type [Output](https://doc.rust-lang.org/1.86.0/core/ops/index/trait.Index.html#associatedtype.Output){.associatedtype} = [Step](../steps/struct.Step.html "struct optionstratlib::simulation::steps::Step"){.struct}\<X, Y\> {#type-output-stepx-y .code-header}
:::

::: docblock
The type returned when indexing the random walk.
:::

::: {#method.index .section .method .trait-impl}
[Source](../../../src/optionstratlib/simulation/randomwalk.rs.html#224-226){.src
.rightside}[§](#method.index){.anchor}

#### fn [index](https://doc.rust-lang.org/1.86.0/core/ops/index/trait.Index.html#tymethod.index){.fn}(&self, index: [usize](https://doc.rust-lang.org/1.86.0/std/primitive.usize.html){.primitive}) -\> &Self::[Output](https://doc.rust-lang.org/1.86.0/core/ops/index/trait.Index.html#associatedtype.Output "type core::ops::index::Index::Output"){.associatedtype} {#fn-indexself-index-usize---selfoutput .code-header}
:::

::: docblock
Provides read access to a specific step in the random walk by index.

##### [§](#parameters-4){.doc-anchor}Parameters

- `index` - The zero-based index of the step to access.

##### [§](#returns-9){.doc-anchor}Returns {#returns-9}

A reference to the `Step<X, Y>` at the specified index.

##### [§](#panics-2){.doc-anchor}Panics

Panics if the index is out of bounds.
:::
:::::::

::::: {#impl-IndexMut%3Cusize%3E-for-RandomWalk%3CX,+Y%3E .section .impl}
[Source](../../../src/optionstratlib/simulation/randomwalk.rs.html#238-259){.src
.rightside}[§](#impl-IndexMut%3Cusize%3E-for-RandomWalk%3CX,+Y%3E){.anchor}

### impl\<X, Y\> [IndexMut](https://doc.rust-lang.org/1.86.0/core/ops/index/trait.IndexMut.html "trait core::ops::index::IndexMut"){.trait}\<[usize](https://doc.rust-lang.org/1.86.0/std/primitive.usize.html){.primitive}\> for [RandomWalk](struct.RandomWalk.html "struct optionstratlib::simulation::randomwalk::RandomWalk"){.struct}\<X, Y\> {#implx-y-indexmutusize-for-randomwalkx-y .code-header}

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

::: docblock
Implementation of the `IndexMut` trait for `RandomWalk<X, Y>`.
:::
:::::

::: docblock
This allows modifying steps in a random walk using array indexing
notation: `walk[index] = new_step`.

#### [§](#type-parameters-3){.doc-anchor}Type Parameters {#type-parameters-3}

- `X` - The type for x-axis values, with constraints as described above.
- `Y` - The type for y-axis values, with constraints as described above.
:::

::::: impl-items
::: {#method.index_mut .section .method .trait-impl}
[Source](../../../src/optionstratlib/simulation/randomwalk.rs.html#256-258){.src
.rightside}[§](#method.index_mut){.anchor}

#### fn [index_mut](https://doc.rust-lang.org/1.86.0/core/ops/index/trait.IndexMut.html#tymethod.index_mut){.fn}(&mut self, index: [usize](https://doc.rust-lang.org/1.86.0/std/primitive.usize.html){.primitive}) -\> &mut Self::[Output](https://doc.rust-lang.org/1.86.0/core/ops/index/trait.Index.html#associatedtype.Output "type core::ops::index::Index::Output"){.associatedtype} {#fn-index_mutmut-self-index-usize---mut-selfoutput .code-header}
:::

::: docblock
Provides mutable access to a specific step in the random walk by index.

##### [§](#parameters-5){.doc-anchor}Parameters

- `index` - The zero-based index of the step to modify.

##### [§](#returns-10){.doc-anchor}Returns {#returns-10}

A mutable reference to the `Step<X, Y>` at the specified index.

##### [§](#panics-3){.doc-anchor}Panics

Panics if the index is out of bounds.
:::
:::::

::::: {#impl-Len-for-RandomWalk%3CX,+Y%3E .section .impl}
[Source](../../../src/optionstratlib/simulation/randomwalk.rs.html#170-192){.src
.rightside}[§](#impl-Len-for-RandomWalk%3CX,+Y%3E){.anchor}

### impl\<X, Y\> [Len](../../utils/trait.Len.html "trait optionstratlib::utils::Len"){.trait} for [RandomWalk](struct.RandomWalk.html "struct optionstratlib::simulation::randomwalk::RandomWalk"){.struct}\<X, Y\> {#implx-y-len-for-randomwalkx-y .code-header}

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

::: docblock
Implementation of the `Len` trait for `RandomWalk<X, Y>`.
:::
:::::

::: docblock
This implementation provides methods to determine the length and
emptiness of a random walk by delegating to the underlying `steps`
collection.

#### [§](#type-parameters-1){.doc-anchor}Type Parameters {#type-parameters-1}

- `X` - The type for x-axis values (typically time or sequence
  position), which must implement `AddAssign`, be convertible to
  `Positive`, and be `Copy`.

- `Y` - The type for y-axis values (typically price or value), which
  must implement `AddAssign`, be convertible to `Positive`, be `Copy`,
  and implement the `Walktypable` trait.
:::

::::::: impl-items
::: {#method.len .section .method .trait-impl}
[Source](../../../src/optionstratlib/simulation/randomwalk.rs.html#180-182){.src
.rightside}[§](#method.len){.anchor}

#### fn [len](../../utils/trait.Len.html#tymethod.len){.fn}(&self) -\> [usize](https://doc.rust-lang.org/1.86.0/std/primitive.usize.html){.primitive} {#fn-lenself---usize .code-header}
:::

::: docblock
Returns the number of steps in the random walk.

##### [§](#returns-7){.doc-anchor}Returns {#returns-7}

A `usize` representing the number of steps.
:::

::: {#method.is_empty .section .method .trait-impl}
[Source](../../../src/optionstratlib/simulation/randomwalk.rs.html#189-191){.src
.rightside}[§](#method.is_empty){.anchor}

#### fn [is_empty](../../utils/trait.Len.html#method.is_empty){.fn}(&self) -\> [bool](https://doc.rust-lang.org/1.86.0/std/primitive.bool.html){.primitive} {#fn-is_emptyself---bool .code-header}
:::

::: docblock
Determines whether the random walk contains any steps.

##### [§](#returns-8){.doc-anchor}Returns {#returns-8}

`true` if the random walk has no steps, `false` otherwise.
:::
:::::::

:::: {#impl-Profit-for-RandomWalk%3CX,+Y%3E .section .impl}
[Source](../../../src/optionstratlib/simulation/randomwalk.rs.html#275-283){.src
.rightside}[§](#impl-Profit-for-RandomWalk%3CX,+Y%3E){.anchor}

### impl\<X, Y\> [Profit](../../pricing/trait.Profit.html "trait optionstratlib::pricing::Profit"){.trait} for [RandomWalk](struct.RandomWalk.html "struct optionstratlib::simulation::randomwalk::RandomWalk"){.struct}\<X, Y\> {#implx-y-profit-for-randomwalkx-y .code-header}

::: where
where X:
[AddAssign](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.AddAssign.html "trait core::ops::arith::AddAssign"){.trait} +
[Copy](https://doc.rust-lang.org/1.86.0/core/marker/trait.Copy.html "trait core::marker::Copy"){.trait} +
[Display](https://doc.rust-lang.org/1.86.0/core/fmt/trait.Display.html "trait core::fmt::Display"){.trait} +
[Into](https://doc.rust-lang.org/1.86.0/core/convert/trait.Into.html "trait core::convert::Into"){.trait}\<[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\>,
Y:
[Into](https://doc.rust-lang.org/1.86.0/core/convert/trait.Into.html "trait core::convert::Into"){.trait}\<[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\> +
[Display](https://doc.rust-lang.org/1.86.0/core/fmt/trait.Display.html "trait core::fmt::Display"){.trait} +
[Clone](https://doc.rust-lang.org/1.86.0/core/clone/trait.Clone.html "trait core::clone::Clone"){.trait},
:::
::::

::::::: impl-items
::: {#method.calculate_profit_at .section .method .trait-impl}
[Source](../../../src/optionstratlib/simulation/randomwalk.rs.html#280-282){.src
.rightside}[§](#method.calculate_profit_at){.anchor}

#### fn [calculate_profit_at](../../pricing/trait.Profit.html#tymethod.calculate_profit_at){.fn}( &self, \_price: [Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, ) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<Decimal, [Box](https://doc.rust-lang.org/1.86.0/alloc/boxed/struct.Box.html "struct alloc::boxed::Box"){.struct}\<dyn [Error](https://doc.rust-lang.org/1.86.0/core/error/trait.Error.html "trait core::error::Error"){.trait}\>\> {#fn-calculate_profit_at-self-_price-positive---resultdecimal-boxdyn-error .code-header}
:::

::: docblock
Calculates the profit at a specified price. [Read
more](../../pricing/trait.Profit.html#tymethod.calculate_profit_at)
:::

::: {#method.get_point_at_price .section .method .trait-impl}
[Source](../../../src/optionstratlib/pricing/payoff.rs.html#222-238){.src
.rightside}[§](#method.get_point_at_price){.anchor}

#### fn [get_point_at_price](../../pricing/trait.Profit.html#method.get_point_at_price){.fn}(&self, price: [Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}) -\> ChartPoint\<([f64](https://doc.rust-lang.org/1.86.0/std/primitive.f64.html){.primitive}, [f64](https://doc.rust-lang.org/1.86.0/std/primitive.f64.html){.primitive})\> {#fn-get_point_at_priceself-price-positive---chartpointf64-f64 .code-header}
:::

::: docblock
Creates a chart point representation of the profit at the given price.
[Read more](../../pricing/trait.Profit.html#method.get_point_at_price)
:::
:::::::
:::::::::::::::::::::::::::::::::::::::::::::::::::::::

## Auto Trait Implementations[§](#synthetic-implementations){.anchor} {#synthetic-implementations .section-header}

:::::::::::::: {#synthetic-implementations-list}
::: {#impl-Freeze-for-RandomWalk%3CX,+Y%3E .section .impl}
[§](#impl-Freeze-for-RandomWalk%3CX,+Y%3E){.anchor}

### impl\<X, Y\> [Freeze](https://doc.rust-lang.org/1.86.0/core/marker/trait.Freeze.html "trait core::marker::Freeze"){.trait} for [RandomWalk](struct.RandomWalk.html "struct optionstratlib::simulation::randomwalk::RandomWalk"){.struct}\<X, Y\> {#implx-y-freeze-for-randomwalkx-y .code-header}
:::

:::: {#impl-RefUnwindSafe-for-RandomWalk%3CX,+Y%3E .section .impl}
[§](#impl-RefUnwindSafe-for-RandomWalk%3CX,+Y%3E){.anchor}

### impl\<X, Y\> [RefUnwindSafe](https://doc.rust-lang.org/1.86.0/core/panic/unwind_safe/trait.RefUnwindSafe.html "trait core::panic::unwind_safe::RefUnwindSafe"){.trait} for [RandomWalk](struct.RandomWalk.html "struct optionstratlib::simulation::randomwalk::RandomWalk"){.struct}\<X, Y\> {#implx-y-refunwindsafe-for-randomwalkx-y .code-header}

::: where
where X:
[RefUnwindSafe](https://doc.rust-lang.org/1.86.0/core/panic/unwind_safe/trait.RefUnwindSafe.html "trait core::panic::unwind_safe::RefUnwindSafe"){.trait},
Y:
[RefUnwindSafe](https://doc.rust-lang.org/1.86.0/core/panic/unwind_safe/trait.RefUnwindSafe.html "trait core::panic::unwind_safe::RefUnwindSafe"){.trait},
:::
::::

:::: {#impl-Send-for-RandomWalk%3CX,+Y%3E .section .impl}
[§](#impl-Send-for-RandomWalk%3CX,+Y%3E){.anchor}

### impl\<X, Y\> [Send](https://doc.rust-lang.org/1.86.0/core/marker/trait.Send.html "trait core::marker::Send"){.trait} for [RandomWalk](struct.RandomWalk.html "struct optionstratlib::simulation::randomwalk::RandomWalk"){.struct}\<X, Y\> {#implx-y-send-for-randomwalkx-y .code-header}

::: where
where X:
[Send](https://doc.rust-lang.org/1.86.0/core/marker/trait.Send.html "trait core::marker::Send"){.trait},
Y:
[Send](https://doc.rust-lang.org/1.86.0/core/marker/trait.Send.html "trait core::marker::Send"){.trait},
:::
::::

:::: {#impl-Sync-for-RandomWalk%3CX,+Y%3E .section .impl}
[§](#impl-Sync-for-RandomWalk%3CX,+Y%3E){.anchor}

### impl\<X, Y\> [Sync](https://doc.rust-lang.org/1.86.0/core/marker/trait.Sync.html "trait core::marker::Sync"){.trait} for [RandomWalk](struct.RandomWalk.html "struct optionstratlib::simulation::randomwalk::RandomWalk"){.struct}\<X, Y\> {#implx-y-sync-for-randomwalkx-y .code-header}

::: where
where X:
[Sync](https://doc.rust-lang.org/1.86.0/core/marker/trait.Sync.html "trait core::marker::Sync"){.trait},
Y:
[Sync](https://doc.rust-lang.org/1.86.0/core/marker/trait.Sync.html "trait core::marker::Sync"){.trait},
:::
::::

:::: {#impl-Unpin-for-RandomWalk%3CX,+Y%3E .section .impl}
[§](#impl-Unpin-for-RandomWalk%3CX,+Y%3E){.anchor}

### impl\<X, Y\> [Unpin](https://doc.rust-lang.org/1.86.0/core/marker/trait.Unpin.html "trait core::marker::Unpin"){.trait} for [RandomWalk](struct.RandomWalk.html "struct optionstratlib::simulation::randomwalk::RandomWalk"){.struct}\<X, Y\> {#implx-y-unpin-for-randomwalkx-y .code-header}

::: where
where X:
[Unpin](https://doc.rust-lang.org/1.86.0/core/marker/trait.Unpin.html "trait core::marker::Unpin"){.trait},
Y:
[Unpin](https://doc.rust-lang.org/1.86.0/core/marker/trait.Unpin.html "trait core::marker::Unpin"){.trait},
:::
::::

:::: {#impl-UnwindSafe-for-RandomWalk%3CX,+Y%3E .section .impl}
[§](#impl-UnwindSafe-for-RandomWalk%3CX,+Y%3E){.anchor}

### impl\<X, Y\> [UnwindSafe](https://doc.rust-lang.org/1.86.0/core/panic/unwind_safe/trait.UnwindSafe.html "trait core::panic::unwind_safe::UnwindSafe"){.trait} for [RandomWalk](struct.RandomWalk.html "struct optionstratlib::simulation::randomwalk::RandomWalk"){.struct}\<X, Y\> {#implx-y-unwindsafe-for-randomwalkx-y .code-header}

::: where
where X:
[UnwindSafe](https://doc.rust-lang.org/1.86.0/core/panic/unwind_safe/trait.UnwindSafe.html "trait core::panic::unwind_safe::UnwindSafe"){.trait},
Y:
[UnwindSafe](https://doc.rust-lang.org/1.86.0/core/panic/unwind_safe/trait.UnwindSafe.html "trait core::panic::unwind_safe::UnwindSafe"){.trait},
:::
::::
::::::::::::::

## Blanket Implementations[§](#blanket-implementations){.anchor} {#blanket-implementations .section-header}

::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::: {#blanket-implementations-list}
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
::: {#associatedtype.Output-1 .section .associatedtype .trait-impl}
[Source](https://docs.rs/typenum/1.18.0/src/typenum/type_operators.rs.html#35){.src
.rightside}[§](#associatedtype.Output-1){.anchor}

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
:::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
:::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
