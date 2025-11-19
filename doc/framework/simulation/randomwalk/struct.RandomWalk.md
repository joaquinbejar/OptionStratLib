::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::: width-limiter
:::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../../index.html)::[simulation](../index.html)::[randomwalk](index.html)
:::

# Struct [RandomWalk]{.struct} Copy item path

[[Source](../../../src/optionstratlib/simulation/randomwalk.rs.html#35-45){.src}
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
[Source](../../../src/optionstratlib/simulation/randomwalk.rs.html#47-157){.src
.rightside}[§](#impl-RandomWalk%3CX,+Y%3E){.anchor}

### impl\<X, Y\> [RandomWalk](struct.RandomWalk.html "struct optionstratlib::simulation::randomwalk::RandomWalk"){.struct}\<X, Y\> {#implx-y-randomwalkx-y .code-header}

::: where
where X:
[Copy](https://doc.rust-lang.org/1.91.1/core/marker/trait.Copy.html "trait core::marker::Copy"){.trait} +
[Into](https://doc.rust-lang.org/1.91.1/core/convert/trait.Into.html "trait core::convert::Into"){.trait}\<[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\> +
[AddAssign](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.AddAssign.html "trait core::ops::arith::AddAssign"){.trait} +
[Display](https://doc.rust-lang.org/1.91.1/core/fmt/trait.Display.html "trait core::fmt::Display"){.trait},
Y:
[Into](https://doc.rust-lang.org/1.91.1/core/convert/trait.Into.html "trait core::convert::Into"){.trait}\<[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\> +
[Display](https://doc.rust-lang.org/1.91.1/core/fmt/trait.Display.html "trait core::fmt::Display"){.trait} +
[Clone](https://doc.rust-lang.org/1.91.1/core/clone/trait.Clone.html "trait core::clone::Clone"){.trait},
:::
::::

:::::::::::::::::::: impl-items
:::: {#method.new .section .method}
[Source](../../../src/optionstratlib/simulation/randomwalk.rs.html#67-75){.src
.rightside}

#### pub fn [new](#method.new){.fn}\<F\>(title: [String](https://doc.rust-lang.org/1.91.1/alloc/string/struct.String.html "struct alloc::string::String"){.struct}, params: &[WalkParams](../struct.WalkParams.html "struct optionstratlib::simulation::WalkParams"){.struct}\<X, Y\>, generator: F) -\> Self {#pub-fn-newftitle-string-params-walkparamsx-y-generator-f---self .code-header}

::: where
where F:
[FnOnce](https://doc.rust-lang.org/1.91.1/core/ops/function/trait.FnOnce.html "trait core::ops::function::FnOnce"){.trait}(&[WalkParams](../struct.WalkParams.html "struct optionstratlib::simulation::WalkParams"){.struct}\<X,
Y\>) -\>
[Vec](https://doc.rust-lang.org/1.91.1/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<[Step](../steps/struct.Step.html "struct optionstratlib::simulation::steps::Step"){.struct}\<X,
Y\>\>, X:
[Copy](https://doc.rust-lang.org/1.91.1/core/marker/trait.Copy.html "trait core::marker::Copy"){.trait} +
[Into](https://doc.rust-lang.org/1.91.1/core/convert/trait.Into.html "trait core::convert::Into"){.trait}\<[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\> +
[AddAssign](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.AddAssign.html "trait core::ops::arith::AddAssign"){.trait} +
[Display](https://doc.rust-lang.org/1.91.1/core/fmt/trait.Display.html "trait core::fmt::Display"){.trait},
Y:
[Into](https://doc.rust-lang.org/1.91.1/core/convert/trait.Into.html "trait core::convert::Into"){.trait}\<[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\> +
[Display](https://doc.rust-lang.org/1.91.1/core/fmt/trait.Display.html "trait core::fmt::Display"){.trait} +
[Clone](https://doc.rust-lang.org/1.91.1/core/clone/trait.Clone.html "trait core::clone::Clone"){.trait},
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
[Source](../../../src/optionstratlib/simulation/randomwalk.rs.html#82-84){.src
.rightside}

#### pub fn [get_title](#method.get_title){.fn}(&self) -\> &[str](https://doc.rust-lang.org/1.91.1/std/primitive.str.html){.primitive} {#pub-fn-get_titleself---str .code-header}
:::

::: docblock
Returns the title of the random walk.

##### [§](#returns-1){.doc-anchor}Returns

A string slice containing the title of the random walk.
:::

::: {#method.set_title .section .method}
[Source](../../../src/optionstratlib/simulation/randomwalk.rs.html#91-93){.src
.rightside}

#### pub fn [set_title](#method.set_title){.fn}(&mut self, title: [String](https://doc.rust-lang.org/1.91.1/alloc/string/struct.String.html "struct alloc::string::String"){.struct}) {#pub-fn-set_titlemut-self-title-string .code-header}
:::

::: docblock
Updates the title of the random walk.

##### [§](#parameters-1){.doc-anchor}Parameters

- `title` - The new title to set
:::

::: {#method.get_steps .section .method}
[Source](../../../src/optionstratlib/simulation/randomwalk.rs.html#100-102){.src
.rightside}

#### pub fn [get_steps](#method.get_steps){.fn}(&self) -\> [Vec](https://doc.rust-lang.org/1.91.1/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<&[Step](../steps/struct.Step.html "struct optionstratlib::simulation::steps::Step"){.struct}\<X, Y\>\> {#pub-fn-get_stepsself---vecstepx-y .code-header}
:::

::: docblock
Returns a vector of references to all steps in the random walk.

##### [§](#returns-2){.doc-anchor}Returns

A vector containing references to all steps in the random walk.
:::

::: {#method.get_step .section .method}
[Source](../../../src/optionstratlib/simulation/randomwalk.rs.html#117-119){.src
.rightside}

#### pub fn [get_step](#method.get_step){.fn}(&self, index: [usize](https://doc.rust-lang.org/1.91.1/std/primitive.usize.html){.primitive}) -\> &[Step](../steps/struct.Step.html "struct optionstratlib::simulation::steps::Step"){.struct}\<X, Y\> {#pub-fn-get_stepself-index-usize---stepx-y .code-header}
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
[Source](../../../src/optionstratlib/simulation/randomwalk.rs.html#134-136){.src
.rightside}

#### pub fn [get_step_mut](#method.get_step_mut){.fn}(&mut self, index: [usize](https://doc.rust-lang.org/1.91.1/std/primitive.usize.html){.primitive}) -\> &mut [Step](../steps/struct.Step.html "struct optionstratlib::simulation::steps::Step"){.struct}\<X, Y\> {#pub-fn-get_step_mutmut-self-index-usize---mut-stepx-y .code-header}
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
[Source](../../../src/optionstratlib/simulation/randomwalk.rs.html#144-146){.src
.rightside}

#### pub fn [first](#method.first){.fn}(&self) -\> [Option](https://doc.rust-lang.org/1.91.1/core/option/enum.Option.html "enum core::option::Option"){.enum}\<&[Step](../steps/struct.Step.html "struct optionstratlib::simulation::steps::Step"){.struct}\<X, Y\>\> {#pub-fn-firstself---optionstepx-y .code-header}
:::

::: docblock
Returns a reference to the first step in the random walk, if any.

##### [§](#returns-5){.doc-anchor}Returns

- `Some(&Step<X, Y>)` - A reference to the first step if the random walk
  is not empty
- `None` - If the random walk has no steps
:::

::: {#method.last .section .method}
[Source](../../../src/optionstratlib/simulation/randomwalk.rs.html#154-156){.src
.rightside}

#### pub fn [last](#method.last){.fn}(&self) -\> [Option](https://doc.rust-lang.org/1.91.1/core/option/enum.Option.html "enum core::option::Option"){.enum}\<&[Step](../steps/struct.Step.html "struct optionstratlib::simulation::steps::Step"){.struct}\<X, Y\>\> {#pub-fn-lastself---optionstepx-y .code-header}
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

::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::: {#trait-implementations-list}
:::: {#impl-BasicAble-for-RandomWalk%3CX,+Y%3E .section .impl}
[Source](../../../src/optionstratlib/simulation/randomwalk.rs.html#289-297){.src
.rightside}[§](#impl-BasicAble-for-RandomWalk%3CX,+Y%3E){.anchor}

### impl\<X, Y\> [BasicAble](../../strategies/base/trait.BasicAble.html "trait optionstratlib::strategies::base::BasicAble"){.trait} for [RandomWalk](struct.RandomWalk.html "struct optionstratlib::simulation::randomwalk::RandomWalk"){.struct}\<X, Y\> {#implx-y-basicable-for-randomwalkx-y .code-header}

::: where
where X:
[AddAssign](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.AddAssign.html "trait core::ops::arith::AddAssign"){.trait} +
[Copy](https://doc.rust-lang.org/1.91.1/core/marker/trait.Copy.html "trait core::marker::Copy"){.trait} +
[Display](https://doc.rust-lang.org/1.91.1/core/fmt/trait.Display.html "trait core::fmt::Display"){.trait} +
[Into](https://doc.rust-lang.org/1.91.1/core/convert/trait.Into.html "trait core::convert::Into"){.trait}\<[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\>,
Y:
[Clone](https://doc.rust-lang.org/1.91.1/core/clone/trait.Clone.html "trait core::clone::Clone"){.trait} +
[Display](https://doc.rust-lang.org/1.91.1/core/fmt/trait.Display.html "trait core::fmt::Display"){.trait} +
[Into](https://doc.rust-lang.org/1.91.1/core/convert/trait.Into.html "trait core::convert::Into"){.trait}\<[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\>,
:::
::::

::::::::::::::::::::::::::::::::::::::::: impl-items
::: {#method.get_title-1 .section .method .trait-impl}
[Source](../../../src/optionstratlib/simulation/randomwalk.rs.html#294-296){.src
.rightside}[§](#method.get_title-1){.anchor}

#### fn [get_title](../../strategies/base/trait.BasicAble.html#method.get_title){.fn}(&self) -\> [String](https://doc.rust-lang.org/1.91.1/alloc/string/struct.String.html "struct alloc::string::String"){.struct} {#fn-get_titleself---string .code-header}
:::

::: docblock
Retrieves the title associated with the current instance of the
strategy. [Read
more](../../strategies/base/trait.BasicAble.html#method.get_title)
:::

::: {#method.get_option_basic_type .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/base.rs.html#399-401){.src
.rightside}[§](#method.get_option_basic_type){.anchor}

#### fn [get_option_basic_type](../../strategies/base/trait.BasicAble.html#method.get_option_basic_type){.fn}(&self) -\> [HashSet](https://doc.rust-lang.org/1.91.1/std/collections/hash/set/struct.HashSet.html "struct std::collections::hash::set::HashSet"){.struct}\<[OptionBasicType](../../model/types/struct.OptionBasicType.html "struct optionstratlib::model::types::OptionBasicType"){.struct}\<\'\_\>\> {#fn-get_option_basic_typeself---hashsetoptionbasictype_ .code-header}
:::

::: docblock
Retrieves a `HashSet` of `OptionBasicType` values associated with the
current strategy. [Read
more](../../strategies/base/trait.BasicAble.html#method.get_option_basic_type)
:::

::: {#method.get_symbol .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/base.rs.html#412-414){.src
.rightside}[§](#method.get_symbol){.anchor}

#### fn [get_symbol](../../strategies/base/trait.BasicAble.html#method.get_symbol){.fn}(&self) -\> &[str](https://doc.rust-lang.org/1.91.1/std/primitive.str.html){.primitive} {#fn-get_symbolself---str .code-header}
:::

::: docblock
Retrieves the symbol associated with the current instance by delegating
the call to the `get_symbol` method of the `one_option` object. [Read
more](../../strategies/base/trait.BasicAble.html#method.get_symbol)
:::

::: {#method.get_strike .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/base.rs.html#430-432){.src
.rightside}[§](#method.get_strike){.anchor}

#### fn [get_strike](../../strategies/base/trait.BasicAble.html#method.get_strike){.fn}(&self) -\> [HashMap](https://doc.rust-lang.org/1.91.1/std/collections/hash/map/struct.HashMap.html "struct std::collections::hash::map::HashMap"){.struct}\<[OptionBasicType](../../model/types/struct.OptionBasicType.html "struct optionstratlib::model::types::OptionBasicType"){.struct}\<\'\_\>, &[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\> {#fn-get_strikeself---hashmapoptionbasictype_-positive .code-header}
:::

::: docblock
Retrieves a mapping of option basic types to their associated positive
strike values. [Read
more](../../strategies/base/trait.BasicAble.html#method.get_strike)
:::

::: {#method.get_strikes .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/base.rs.html#448-453){.src
.rightside}[§](#method.get_strikes){.anchor}

#### fn [get_strikes](../../strategies/base/trait.BasicAble.html#method.get_strikes){.fn}(&self) -\> [Vec](https://doc.rust-lang.org/1.91.1/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<&[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\> {#fn-get_strikesself---vecpositive .code-header}
:::

::: docblock
Retrieves a vector of strike prices from the option types. [Read
more](../../strategies/base/trait.BasicAble.html#method.get_strikes)
:::

::: {#method.get_side .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/base.rs.html#475-480){.src
.rightside}[§](#method.get_side){.anchor}

#### fn [get_side](../../strategies/base/trait.BasicAble.html#method.get_side){.fn}(&self) -\> [HashMap](https://doc.rust-lang.org/1.91.1/std/collections/hash/map/struct.HashMap.html "struct std::collections::hash::map::HashMap"){.struct}\<[OptionBasicType](../../model/types/struct.OptionBasicType.html "struct optionstratlib::model::types::OptionBasicType"){.struct}\<\'\_\>, &[Side](../../model/types/enum.Side.html "enum optionstratlib::model::types::Side"){.enum}\> {#fn-get_sideself---hashmapoptionbasictype_-side .code-header}
:::

::: docblock
Retrieves a `HashMap` that maps each `OptionBasicType` to its
corresponding `Side`. [Read
more](../../strategies/base/trait.BasicAble.html#method.get_side)
:::

::: {#method.get_type .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/base.rs.html#492-494){.src
.rightside}[§](#method.get_type){.anchor}

#### fn [get_type](../../strategies/base/trait.BasicAble.html#method.get_type){.fn}(&self) -\> &[OptionType](../../model/types/enum.OptionType.html "enum optionstratlib::model::types::OptionType"){.enum} {#fn-get_typeself---optiontype .code-header}
:::

::: docblock
Retrieves the type of the option. [Read
more](../../strategies/base/trait.BasicAble.html#method.get_type)
:::

::: {#method.get_style .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/base.rs.html#512-517){.src
.rightside}[§](#method.get_style){.anchor}

#### fn [get_style](../../strategies/base/trait.BasicAble.html#method.get_style){.fn}(&self) -\> [HashMap](https://doc.rust-lang.org/1.91.1/std/collections/hash/map/struct.HashMap.html "struct std::collections::hash::map::HashMap"){.struct}\<[OptionBasicType](../../model/types/struct.OptionBasicType.html "struct optionstratlib::model::types::OptionBasicType"){.struct}\<\'\_\>, &[OptionStyle](../../model/types/enum.OptionStyle.html "enum optionstratlib::model::types::OptionStyle"){.enum}\> {#fn-get_styleself---hashmapoptionbasictype_-optionstyle .code-header}
:::

::: docblock
Retrieves a mapping of `OptionBasicType` to their corresponding
`OptionStyle`. [Read
more](../../strategies/base/trait.BasicAble.html#method.get_style)
:::

::: {#method.get_expiration .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/base.rs.html#536-541){.src
.rightside}[§](#method.get_expiration){.anchor}

#### fn [get_expiration](../../strategies/base/trait.BasicAble.html#method.get_expiration){.fn}(&self) -\> [HashMap](https://doc.rust-lang.org/1.91.1/std/collections/hash/map/struct.HashMap.html "struct std::collections::hash::map::HashMap"){.struct}\<[OptionBasicType](../../model/types/struct.OptionBasicType.html "struct optionstratlib::model::types::OptionBasicType"){.struct}\<\'\_\>, &[ExpirationDate](../../model/enum.ExpirationDate.html "enum optionstratlib::model::ExpirationDate"){.enum}\> {#fn-get_expirationself---hashmapoptionbasictype_-expirationdate .code-header}
:::

::: docblock
Retrieves a map of option basic types to their corresponding expiration
dates. [Read
more](../../strategies/base/trait.BasicAble.html#method.get_expiration)
:::

::: {#method.get_implied_volatility .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/base.rs.html#560-562){.src
.rightside}[§](#method.get_implied_volatility){.anchor}

#### fn [get_implied_volatility](../../strategies/base/trait.BasicAble.html#method.get_implied_volatility){.fn}(&self) -\> [HashMap](https://doc.rust-lang.org/1.91.1/std/collections/hash/map/struct.HashMap.html "struct std::collections::hash::map::HashMap"){.struct}\<[OptionBasicType](../../model/types/struct.OptionBasicType.html "struct optionstratlib::model::types::OptionBasicType"){.struct}\<\'\_\>, &[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\> {#fn-get_implied_volatilityself---hashmapoptionbasictype_-positive .code-header}
:::

::: docblock
Retrieves the implied volatility for the current strategy. [Read
more](../../strategies/base/trait.BasicAble.html#method.get_implied_volatility)
:::

::: {#method.get_quantity .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/base.rs.html#580-582){.src
.rightside}[§](#method.get_quantity){.anchor}

#### fn [get_quantity](../../strategies/base/trait.BasicAble.html#method.get_quantity){.fn}(&self) -\> [HashMap](https://doc.rust-lang.org/1.91.1/std/collections/hash/map/struct.HashMap.html "struct std::collections::hash::map::HashMap"){.struct}\<[OptionBasicType](../../model/types/struct.OptionBasicType.html "struct optionstratlib::model::types::OptionBasicType"){.struct}\<\'\_\>, &[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\> {#fn-get_quantityself---hashmapoptionbasictype_-positive .code-header}
:::

::: docblock
Retrieves the quantity information associated with the strategy. [Read
more](../../strategies/base/trait.BasicAble.html#method.get_quantity)
:::

::: {#method.get_underlying_price .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/base.rs.html#595-597){.src
.rightside}[§](#method.get_underlying_price){.anchor}

#### fn [get_underlying_price](../../strategies/base/trait.BasicAble.html#method.get_underlying_price){.fn}(&self) -\> &[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#fn-get_underlying_priceself---positive .code-header}
:::

::: docblock
Retrieves the underlying price of the financial instrument (e.g.,
option). [Read
more](../../strategies/base/trait.BasicAble.html#method.get_underlying_price)
:::

::: {#method.get_risk_free_rate .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/base.rs.html#619-621){.src
.rightside}[§](#method.get_risk_free_rate){.anchor}

#### fn [get_risk_free_rate](../../strategies/base/trait.BasicAble.html#method.get_risk_free_rate){.fn}(&self) -\> [HashMap](https://doc.rust-lang.org/1.91.1/std/collections/hash/map/struct.HashMap.html "struct std::collections::hash::map::HashMap"){.struct}\<[OptionBasicType](../../model/types/struct.OptionBasicType.html "struct optionstratlib::model::types::OptionBasicType"){.struct}\<\'\_\>, &[Decimal](../../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}\> {#fn-get_risk_free_rateself---hashmapoptionbasictype_-decimal .code-header}
:::

::: docblock
Retrieves the risk-free interest rate associated with a given set of
options. [Read
more](../../strategies/base/trait.BasicAble.html#method.get_risk_free_rate)
:::

::: {#method.get_dividend_yield .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/base.rs.html#636-638){.src
.rightside}[§](#method.get_dividend_yield){.anchor}

#### fn [get_dividend_yield](../../strategies/base/trait.BasicAble.html#method.get_dividend_yield){.fn}(&self) -\> [HashMap](https://doc.rust-lang.org/1.91.1/std/collections/hash/map/struct.HashMap.html "struct std::collections::hash::map::HashMap"){.struct}\<[OptionBasicType](../../model/types/struct.OptionBasicType.html "struct optionstratlib::model::types::OptionBasicType"){.struct}\<\'\_\>, &[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\> {#fn-get_dividend_yieldself---hashmapoptionbasictype_-positive .code-header}
:::

::: docblock
Retrieves the dividend yield of a financial option. [Read
more](../../strategies/base/trait.BasicAble.html#method.get_dividend_yield)
:::

::: {#method.one_option .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/base.rs.html#654-656){.src
.rightside}[§](#method.one_option){.anchor}

#### fn [one_option](../../strategies/base/trait.BasicAble.html#method.one_option){.fn}(&self) -\> &[Options](../../model/option/struct.Options.html "struct optionstratlib::model::option::Options"){.struct} {#fn-one_optionself---options .code-header}
:::

::: docblock
This method, `one_option`, is designed to retrieve a reference to an
`Options` object. However, in this implementation, the function is not
currently functional, as it explicitly triggers an unimplemented error
when called. [Read
more](../../strategies/base/trait.BasicAble.html#method.one_option)
:::

::: {#method.one_option_mut .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/base.rs.html#674-676){.src
.rightside}[§](#method.one_option_mut){.anchor}

#### fn [one_option_mut](../../strategies/base/trait.BasicAble.html#method.one_option_mut){.fn}(&mut self) -\> &mut [Options](../../model/option/struct.Options.html "struct optionstratlib::model::option::Options"){.struct} {#fn-one_option_mutmut-self---mut-options .code-header}
:::

::: docblock
Provides a mutable reference to an `Options` instance. [Read
more](../../strategies/base/trait.BasicAble.html#method.one_option_mut)
:::

::: {#method.set_expiration_date .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/base.rs.html#712-717){.src
.rightside}[§](#method.set_expiration_date){.anchor}

#### fn [set_expiration_date](../../strategies/base/trait.BasicAble.html#method.set_expiration_date){.fn}( &mut self, \_expiration_date: [ExpirationDate](../../model/enum.ExpirationDate.html "enum optionstratlib::model::ExpirationDate"){.enum}, ) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[()](https://doc.rust-lang.org/1.91.1/std/primitive.unit.html){.primitive}, [StrategyError](../../error/strategies/enum.StrategyError.html "enum optionstratlib::error::strategies::StrategyError"){.enum}\> {#fn-set_expiration_date-mut-self-_expiration_date-expirationdate---result-strategyerror .code-header}
:::

::: docblock
Sets the expiration date for the strategy. [Read
more](../../strategies/base/trait.BasicAble.html#method.set_expiration_date)
:::

::: {#method.set_underlying_price .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/base.rs.html#736-738){.src
.rightside}[§](#method.set_underlying_price){.anchor}

#### fn [set_underlying_price](../../strategies/base/trait.BasicAble.html#method.set_underlying_price){.fn}( &mut self, \_price: &[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, ) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[()](https://doc.rust-lang.org/1.91.1/std/primitive.unit.html){.primitive}, [StrategyError](../../error/strategies/enum.StrategyError.html "enum optionstratlib::error::strategies::StrategyError"){.enum}\> {#fn-set_underlying_price-mut-self-_price-positive---result-strategyerror .code-header}
:::

::: docblock
Sets the underlying price for this strategy. [Read
more](../../strategies/base/trait.BasicAble.html#method.set_underlying_price)
:::

::: {#method.set_implied_volatility .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/base.rs.html#752-754){.src
.rightside}[§](#method.set_implied_volatility){.anchor}

#### fn [set_implied_volatility](../../strategies/base/trait.BasicAble.html#method.set_implied_volatility){.fn}( &mut self, \_volatility: &[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, ) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[()](https://doc.rust-lang.org/1.91.1/std/primitive.unit.html){.primitive}, [StrategyError](../../error/strategies/enum.StrategyError.html "enum optionstratlib::error::strategies::StrategyError"){.enum}\> {#fn-set_implied_volatility-mut-self-_volatility-positive---result-strategyerror .code-header}
:::

::: docblock
Updates the volatility for the strategy. [Read
more](../../strategies/base/trait.BasicAble.html#method.set_implied_volatility)
:::
:::::::::::::::::::::::::::::::::::::::::

:::: {#impl-Clone-for-RandomWalk%3CX,+Y%3E .section .impl}
[Source](../../../src/optionstratlib/simulation/randomwalk.rs.html#34){.src
.rightside}[§](#impl-Clone-for-RandomWalk%3CX,+Y%3E){.anchor}

### impl\<X, Y\> [Clone](https://doc.rust-lang.org/1.91.1/core/clone/trait.Clone.html "trait core::clone::Clone"){.trait} for [RandomWalk](struct.RandomWalk.html "struct optionstratlib::simulation::randomwalk::RandomWalk"){.struct}\<X, Y\> {#implx-y-clone-for-randomwalkx-y .code-header}

::: where
where X:
[Copy](https://doc.rust-lang.org/1.91.1/core/marker/trait.Copy.html "trait core::marker::Copy"){.trait} +
[Into](https://doc.rust-lang.org/1.91.1/core/convert/trait.Into.html "trait core::convert::Into"){.trait}\<[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\> +
[AddAssign](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.AddAssign.html "trait core::ops::arith::AddAssign"){.trait} +
[Display](https://doc.rust-lang.org/1.91.1/core/fmt/trait.Display.html "trait core::fmt::Display"){.trait} +
[Clone](https://doc.rust-lang.org/1.91.1/core/clone/trait.Clone.html "trait core::clone::Clone"){.trait},
Y:
[Into](https://doc.rust-lang.org/1.91.1/core/convert/trait.Into.html "trait core::convert::Into"){.trait}\<[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\> +
[Display](https://doc.rust-lang.org/1.91.1/core/fmt/trait.Display.html "trait core::fmt::Display"){.trait} +
[Clone](https://doc.rust-lang.org/1.91.1/core/clone/trait.Clone.html "trait core::clone::Clone"){.trait} +
[Clone](https://doc.rust-lang.org/1.91.1/core/clone/trait.Clone.html "trait core::clone::Clone"){.trait},
:::
::::

::::::: impl-items
::: {#method.clone .section .method .trait-impl}
[Source](../../../src/optionstratlib/simulation/randomwalk.rs.html#34){.src
.rightside}[§](#method.clone){.anchor}

#### fn [clone](https://doc.rust-lang.org/1.91.1/core/clone/trait.Clone.html#tymethod.clone){.fn}(&self) -\> [RandomWalk](struct.RandomWalk.html "struct optionstratlib::simulation::randomwalk::RandomWalk"){.struct}\<X, Y\> {#fn-cloneself---randomwalkx-y .code-header}
:::

::: docblock
Returns a duplicate of the value. [Read
more](https://doc.rust-lang.org/1.91.1/core/clone/trait.Clone.html#tymethod.clone)
:::

::: {#method.clone_from .section .method .trait-impl}
[[1.0.0]{.since title="Stable since Rust version 1.0.0"} ·
[Source](https://doc.rust-lang.org/1.91.1/src/core/clone.rs.html#245-247){.src}]{.rightside}[§](#method.clone_from){.anchor}

#### fn [clone_from](https://doc.rust-lang.org/1.91.1/core/clone/trait.Clone.html#method.clone_from){.fn}(&mut self, source: &Self) {#fn-clone_frommut-self-source-self .code-header}
:::

::: docblock
Performs copy-assignment from `source`. [Read
more](https://doc.rust-lang.org/1.91.1/core/clone/trait.Clone.html#method.clone_from)
:::
:::::::

:::: {#impl-Debug-for-RandomWalk%3CX,+Y%3E .section .impl}
[Source](../../../src/optionstratlib/simulation/randomwalk.rs.html#34){.src
.rightside}[§](#impl-Debug-for-RandomWalk%3CX,+Y%3E){.anchor}

### impl\<X, Y\> [Debug](https://doc.rust-lang.org/1.91.1/core/fmt/trait.Debug.html "trait core::fmt::Debug"){.trait} for [RandomWalk](struct.RandomWalk.html "struct optionstratlib::simulation::randomwalk::RandomWalk"){.struct}\<X, Y\> {#implx-y-debug-for-randomwalkx-y .code-header}

::: where
where X:
[Copy](https://doc.rust-lang.org/1.91.1/core/marker/trait.Copy.html "trait core::marker::Copy"){.trait} +
[Into](https://doc.rust-lang.org/1.91.1/core/convert/trait.Into.html "trait core::convert::Into"){.trait}\<[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\> +
[AddAssign](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.AddAssign.html "trait core::ops::arith::AddAssign"){.trait} +
[Display](https://doc.rust-lang.org/1.91.1/core/fmt/trait.Display.html "trait core::fmt::Display"){.trait} +
[Debug](https://doc.rust-lang.org/1.91.1/core/fmt/trait.Debug.html "trait core::fmt::Debug"){.trait},
Y:
[Into](https://doc.rust-lang.org/1.91.1/core/convert/trait.Into.html "trait core::convert::Into"){.trait}\<[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\> +
[Display](https://doc.rust-lang.org/1.91.1/core/fmt/trait.Display.html "trait core::fmt::Display"){.trait} +
[Clone](https://doc.rust-lang.org/1.91.1/core/clone/trait.Clone.html "trait core::clone::Clone"){.trait} +
[Debug](https://doc.rust-lang.org/1.91.1/core/fmt/trait.Debug.html "trait core::fmt::Debug"){.trait},
:::
::::

::::: impl-items
::: {#method.fmt .section .method .trait-impl}
[Source](../../../src/optionstratlib/simulation/randomwalk.rs.html#34){.src
.rightside}[§](#method.fmt){.anchor}

#### fn [fmt](https://doc.rust-lang.org/1.91.1/core/fmt/trait.Debug.html#tymethod.fmt){.fn}(&self, f: &mut [Formatter](https://doc.rust-lang.org/1.91.1/core/fmt/struct.Formatter.html "struct core::fmt::Formatter"){.struct}\<\'\_\>) -\> [Result](https://doc.rust-lang.org/1.91.1/core/fmt/type.Result.html "type core::fmt::Result"){.type} {#fn-fmtself-f-mut-formatter_---result .code-header}
:::

::: docblock
Formats the value using the given formatter. [Read
more](https://doc.rust-lang.org/1.91.1/core/fmt/trait.Debug.html#tymethod.fmt)
:::
:::::

:::: {#impl-Default-for-RandomWalk%3CX,+Y%3E .section .impl}
[Source](../../../src/optionstratlib/simulation/randomwalk.rs.html#34){.src
.rightside}[§](#impl-Default-for-RandomWalk%3CX,+Y%3E){.anchor}

### impl\<X, Y\> [Default](https://doc.rust-lang.org/1.91.1/core/default/trait.Default.html "trait core::default::Default"){.trait} for [RandomWalk](struct.RandomWalk.html "struct optionstratlib::simulation::randomwalk::RandomWalk"){.struct}\<X, Y\> {#implx-y-default-for-randomwalkx-y .code-header}

::: where
where X:
[Copy](https://doc.rust-lang.org/1.91.1/core/marker/trait.Copy.html "trait core::marker::Copy"){.trait} +
[Into](https://doc.rust-lang.org/1.91.1/core/convert/trait.Into.html "trait core::convert::Into"){.trait}\<[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\> +
[AddAssign](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.AddAssign.html "trait core::ops::arith::AddAssign"){.trait} +
[Display](https://doc.rust-lang.org/1.91.1/core/fmt/trait.Display.html "trait core::fmt::Display"){.trait} +
[Default](https://doc.rust-lang.org/1.91.1/core/default/trait.Default.html "trait core::default::Default"){.trait},
Y:
[Into](https://doc.rust-lang.org/1.91.1/core/convert/trait.Into.html "trait core::convert::Into"){.trait}\<[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\> +
[Display](https://doc.rust-lang.org/1.91.1/core/fmt/trait.Display.html "trait core::fmt::Display"){.trait} +
[Clone](https://doc.rust-lang.org/1.91.1/core/clone/trait.Clone.html "trait core::clone::Clone"){.trait} +
[Default](https://doc.rust-lang.org/1.91.1/core/default/trait.Default.html "trait core::default::Default"){.trait},
:::
::::

::::: impl-items
::: {#method.default .section .method .trait-impl}
[Source](../../../src/optionstratlib/simulation/randomwalk.rs.html#34){.src
.rightside}[§](#method.default){.anchor}

#### fn [default](https://doc.rust-lang.org/1.91.1/core/default/trait.Default.html#tymethod.default){.fn}() -\> [RandomWalk](struct.RandomWalk.html "struct optionstratlib::simulation::randomwalk::RandomWalk"){.struct}\<X, Y\> {#fn-default---randomwalkx-y .code-header}
:::

::: docblock
Returns the "default value" for a type. [Read
more](https://doc.rust-lang.org/1.91.1/core/default/trait.Default.html#tymethod.default)
:::
:::::

:::: {#impl-Display-for-RandomWalk%3CX,+Y%3E .section .impl}
[Source](../../../src/optionstratlib/simulation/randomwalk.rs.html#263-275){.src
.rightside}[§](#impl-Display-for-RandomWalk%3CX,+Y%3E){.anchor}

### impl\<X, Y\> [Display](https://doc.rust-lang.org/1.91.1/core/fmt/trait.Display.html "trait core::fmt::Display"){.trait} for [RandomWalk](struct.RandomWalk.html "struct optionstratlib::simulation::randomwalk::RandomWalk"){.struct}\<X, Y\> {#implx-y-display-for-randomwalkx-y .code-header}

::: where
where X:
[Copy](https://doc.rust-lang.org/1.91.1/core/marker/trait.Copy.html "trait core::marker::Copy"){.trait} +
[Into](https://doc.rust-lang.org/1.91.1/core/convert/trait.Into.html "trait core::convert::Into"){.trait}\<[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\> +
[AddAssign](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.AddAssign.html "trait core::ops::arith::AddAssign"){.trait} +
[Display](https://doc.rust-lang.org/1.91.1/core/fmt/trait.Display.html "trait core::fmt::Display"){.trait},
Y:
[Into](https://doc.rust-lang.org/1.91.1/core/convert/trait.Into.html "trait core::convert::Into"){.trait}\<[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\> +
[Display](https://doc.rust-lang.org/1.91.1/core/fmt/trait.Display.html "trait core::fmt::Display"){.trait} +
[Clone](https://doc.rust-lang.org/1.91.1/core/clone/trait.Clone.html "trait core::clone::Clone"){.trait},
:::
::::

::::: impl-items
::: {#method.fmt-1 .section .method .trait-impl}
[Source](../../../src/optionstratlib/simulation/randomwalk.rs.html#268-274){.src
.rightside}[§](#method.fmt-1){.anchor}

#### fn [fmt](https://doc.rust-lang.org/1.91.1/core/fmt/trait.Display.html#tymethod.fmt){.fn}(&self, f: &mut [Formatter](https://doc.rust-lang.org/1.91.1/core/fmt/struct.Formatter.html "struct core::fmt::Formatter"){.struct}\<\'\_\>) -\> [Result](https://doc.rust-lang.org/1.91.1/core/fmt/type.Result.html "type core::fmt::Result"){.type} {#fn-fmtself-f-mut-formatter_---result-1 .code-header}
:::

::: docblock
Formats the value using the given formatter. [Read
more](https://doc.rust-lang.org/1.91.1/core/fmt/trait.Display.html#tymethod.fmt)
:::
:::::

:::: {#impl-Graph-for-RandomWalk%3CX,+Y%3E .section .impl}
[Source](../../../src/optionstratlib/simulation/randomwalk.rs.html#299-339){.src
.rightside}[§](#impl-Graph-for-RandomWalk%3CX,+Y%3E){.anchor}

### impl\<X, Y\> [Graph](../../visualization/trait.Graph.html "trait optionstratlib::visualization::Graph"){.trait} for [RandomWalk](struct.RandomWalk.html "struct optionstratlib::simulation::randomwalk::RandomWalk"){.struct}\<X, Y\> {#implx-y-graph-for-randomwalkx-y .code-header}

::: where
where X:
[Copy](https://doc.rust-lang.org/1.91.1/core/marker/trait.Copy.html "trait core::marker::Copy"){.trait} +
[Into](https://doc.rust-lang.org/1.91.1/core/convert/trait.Into.html "trait core::convert::Into"){.trait}\<[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\> +
[AddAssign](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.AddAssign.html "trait core::ops::arith::AddAssign"){.trait} +
[Display](https://doc.rust-lang.org/1.91.1/core/fmt/trait.Display.html "trait core::fmt::Display"){.trait},
Y:
[Into](https://doc.rust-lang.org/1.91.1/core/convert/trait.Into.html "trait core::convert::Into"){.trait}\<[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\> +
[Display](https://doc.rust-lang.org/1.91.1/core/fmt/trait.Display.html "trait core::fmt::Display"){.trait} +
[Clone](https://doc.rust-lang.org/1.91.1/core/clone/trait.Clone.html "trait core::clone::Clone"){.trait},
:::
::::

::::::: impl-items
::: {#method.graph_data .section .method .trait-impl}
[Source](../../../src/optionstratlib/simulation/randomwalk.rs.html#304-323){.src
.rightside}[§](#method.graph_data){.anchor}

#### fn [graph_data](../../visualization/trait.Graph.html#tymethod.graph_data){.fn}(&self) -\> [GraphData](../../visualization/enum.GraphData.html "enum optionstratlib::visualization::GraphData"){.enum} {#fn-graph_dataself---graphdata .code-header}
:::

::: docblock
Return the raw data ready for plotting.
:::

::: {#method.graph_config .section .method .trait-impl}
[Source](../../../src/optionstratlib/simulation/randomwalk.rs.html#325-338){.src
.rightside}[§](#method.graph_config){.anchor}

#### fn [graph_config](../../visualization/trait.Graph.html#method.graph_config){.fn}(&self) -\> [GraphConfig](../../visualization/struct.GraphConfig.html "struct optionstratlib::visualization::GraphConfig"){.struct} {#fn-graph_configself---graphconfig .code-header}
:::

::: docblock
Optional per‑object configuration overrides.
:::
:::::::

::::: {#impl-Index%3Cusize%3E-for-RandomWalk%3CX,+Y%3E .section .impl}
[Source](../../../src/optionstratlib/simulation/randomwalk.rs.html#205-229){.src
.rightside}[§](#impl-Index%3Cusize%3E-for-RandomWalk%3CX,+Y%3E){.anchor}

### impl\<X, Y\> [Index](https://doc.rust-lang.org/1.91.1/core/ops/index/trait.Index.html "trait core::ops::index::Index"){.trait}\<[usize](https://doc.rust-lang.org/1.91.1/std/primitive.usize.html){.primitive}\> for [RandomWalk](struct.RandomWalk.html "struct optionstratlib::simulation::randomwalk::RandomWalk"){.struct}\<X, Y\> {#implx-y-indexusize-for-randomwalkx-y .code-header}

::: where
where X:
[Copy](https://doc.rust-lang.org/1.91.1/core/marker/trait.Copy.html "trait core::marker::Copy"){.trait} +
[Into](https://doc.rust-lang.org/1.91.1/core/convert/trait.Into.html "trait core::convert::Into"){.trait}\<[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\> +
[AddAssign](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.AddAssign.html "trait core::ops::arith::AddAssign"){.trait} +
[Display](https://doc.rust-lang.org/1.91.1/core/fmt/trait.Display.html "trait core::fmt::Display"){.trait},
Y:
[Into](https://doc.rust-lang.org/1.91.1/core/convert/trait.Into.html "trait core::convert::Into"){.trait}\<[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\> +
[Display](https://doc.rust-lang.org/1.91.1/core/fmt/trait.Display.html "trait core::fmt::Display"){.trait} +
[Clone](https://doc.rust-lang.org/1.91.1/core/clone/trait.Clone.html "trait core::clone::Clone"){.trait},
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
[Source](../../../src/optionstratlib/simulation/randomwalk.rs.html#211){.src
.rightside}[§](#associatedtype.Output){.anchor}

#### type [Output](https://doc.rust-lang.org/1.91.1/core/ops/index/trait.Index.html#associatedtype.Output){.associatedtype} = [Step](../steps/struct.Step.html "struct optionstratlib::simulation::steps::Step"){.struct}\<X, Y\> {#type-output-stepx-y .code-header}
:::

::: docblock
The type returned when indexing the random walk.
:::

::: {#method.index .section .method .trait-impl}
[Source](../../../src/optionstratlib/simulation/randomwalk.rs.html#226-228){.src
.rightside}[§](#method.index){.anchor}

#### fn [index](https://doc.rust-lang.org/1.91.1/core/ops/index/trait.Index.html#tymethod.index){.fn}(&self, index: [usize](https://doc.rust-lang.org/1.91.1/std/primitive.usize.html){.primitive}) -\> &Self::[Output](https://doc.rust-lang.org/1.91.1/core/ops/index/trait.Index.html#associatedtype.Output "type core::ops::index::Index::Output"){.associatedtype} {#fn-indexself-index-usize---selfoutput .code-header}
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
[Source](../../../src/optionstratlib/simulation/randomwalk.rs.html#240-261){.src
.rightside}[§](#impl-IndexMut%3Cusize%3E-for-RandomWalk%3CX,+Y%3E){.anchor}

### impl\<X, Y\> [IndexMut](https://doc.rust-lang.org/1.91.1/core/ops/index/trait.IndexMut.html "trait core::ops::index::IndexMut"){.trait}\<[usize](https://doc.rust-lang.org/1.91.1/std/primitive.usize.html){.primitive}\> for [RandomWalk](struct.RandomWalk.html "struct optionstratlib::simulation::randomwalk::RandomWalk"){.struct}\<X, Y\> {#implx-y-indexmutusize-for-randomwalkx-y .code-header}

::: where
where X:
[Copy](https://doc.rust-lang.org/1.91.1/core/marker/trait.Copy.html "trait core::marker::Copy"){.trait} +
[Into](https://doc.rust-lang.org/1.91.1/core/convert/trait.Into.html "trait core::convert::Into"){.trait}\<[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\> +
[AddAssign](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.AddAssign.html "trait core::ops::arith::AddAssign"){.trait} +
[Display](https://doc.rust-lang.org/1.91.1/core/fmt/trait.Display.html "trait core::fmt::Display"){.trait},
Y:
[Into](https://doc.rust-lang.org/1.91.1/core/convert/trait.Into.html "trait core::convert::Into"){.trait}\<[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\> +
[Display](https://doc.rust-lang.org/1.91.1/core/fmt/trait.Display.html "trait core::fmt::Display"){.trait} +
[Clone](https://doc.rust-lang.org/1.91.1/core/clone/trait.Clone.html "trait core::clone::Clone"){.trait},
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
[Source](../../../src/optionstratlib/simulation/randomwalk.rs.html#258-260){.src
.rightside}[§](#method.index_mut){.anchor}

#### fn [index_mut](https://doc.rust-lang.org/1.91.1/core/ops/index/trait.IndexMut.html#tymethod.index_mut){.fn}(&mut self, index: [usize](https://doc.rust-lang.org/1.91.1/std/primitive.usize.html){.primitive}) -\> &mut Self::[Output](https://doc.rust-lang.org/1.91.1/core/ops/index/trait.Index.html#associatedtype.Output "type core::ops::index::Index::Output"){.associatedtype} {#fn-index_mutmut-self-index-usize---mut-selfoutput .code-header}
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
[Source](../../../src/optionstratlib/simulation/randomwalk.rs.html#172-194){.src
.rightside}[§](#impl-Len-for-RandomWalk%3CX,+Y%3E){.anchor}

### impl\<X, Y\> [Len](../../utils/trait.Len.html "trait optionstratlib::utils::Len"){.trait} for [RandomWalk](struct.RandomWalk.html "struct optionstratlib::simulation::randomwalk::RandomWalk"){.struct}\<X, Y\> {#implx-y-len-for-randomwalkx-y .code-header}

::: where
where X:
[Copy](https://doc.rust-lang.org/1.91.1/core/marker/trait.Copy.html "trait core::marker::Copy"){.trait} +
[Into](https://doc.rust-lang.org/1.91.1/core/convert/trait.Into.html "trait core::convert::Into"){.trait}\<[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\> +
[AddAssign](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.AddAssign.html "trait core::ops::arith::AddAssign"){.trait} +
[Display](https://doc.rust-lang.org/1.91.1/core/fmt/trait.Display.html "trait core::fmt::Display"){.trait},
Y:
[Into](https://doc.rust-lang.org/1.91.1/core/convert/trait.Into.html "trait core::convert::Into"){.trait}\<[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\> +
[Display](https://doc.rust-lang.org/1.91.1/core/fmt/trait.Display.html "trait core::fmt::Display"){.trait} +
[Clone](https://doc.rust-lang.org/1.91.1/core/clone/trait.Clone.html "trait core::clone::Clone"){.trait},
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
[Source](../../../src/optionstratlib/simulation/randomwalk.rs.html#182-184){.src
.rightside}[§](#method.len){.anchor}

#### fn [len](../../utils/trait.Len.html#tymethod.len){.fn}(&self) -\> [usize](https://doc.rust-lang.org/1.91.1/std/primitive.usize.html){.primitive} {#fn-lenself---usize .code-header}
:::

::: docblock
Returns the number of steps in the random walk.

##### [§](#returns-7){.doc-anchor}Returns {#returns-7}

A `usize` representing the number of steps.
:::

::: {#method.is_empty .section .method .trait-impl}
[Source](../../../src/optionstratlib/simulation/randomwalk.rs.html#191-193){.src
.rightside}[§](#method.is_empty){.anchor}

#### fn [is_empty](../../utils/trait.Len.html#method.is_empty){.fn}(&self) -\> [bool](https://doc.rust-lang.org/1.91.1/std/primitive.bool.html){.primitive} {#fn-is_emptyself---bool .code-header}
:::

::: docblock
Determines whether the random walk contains any steps.

##### [§](#returns-8){.doc-anchor}Returns {#returns-8}

`true` if the random walk has no steps, `false` otherwise.
:::
:::::::

:::: {#impl-Profit-for-RandomWalk%3CX,+Y%3E .section .impl}
[Source](../../../src/optionstratlib/simulation/randomwalk.rs.html#277-287){.src
.rightside}[§](#impl-Profit-for-RandomWalk%3CX,+Y%3E){.anchor}

### impl\<X, Y\> [Profit](../../pricing/trait.Profit.html "trait optionstratlib::pricing::Profit"){.trait} for [RandomWalk](struct.RandomWalk.html "struct optionstratlib::simulation::randomwalk::RandomWalk"){.struct}\<X, Y\> {#implx-y-profit-for-randomwalkx-y .code-header}

::: where
where X:
[AddAssign](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.AddAssign.html "trait core::ops::arith::AddAssign"){.trait} +
[Copy](https://doc.rust-lang.org/1.91.1/core/marker/trait.Copy.html "trait core::marker::Copy"){.trait} +
[Display](https://doc.rust-lang.org/1.91.1/core/fmt/trait.Display.html "trait core::fmt::Display"){.trait} +
[Into](https://doc.rust-lang.org/1.91.1/core/convert/trait.Into.html "trait core::convert::Into"){.trait}\<[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\>,
Y:
[Into](https://doc.rust-lang.org/1.91.1/core/convert/trait.Into.html "trait core::convert::Into"){.trait}\<[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\> +
[Display](https://doc.rust-lang.org/1.91.1/core/fmt/trait.Display.html "trait core::fmt::Display"){.trait} +
[Clone](https://doc.rust-lang.org/1.91.1/core/clone/trait.Clone.html "trait core::clone::Clone"){.trait},
:::
::::

::::::: impl-items
::: {#method.calculate_profit_at .section .method .trait-impl}
[Source](../../../src/optionstratlib/simulation/randomwalk.rs.html#282-286){.src
.rightside}[§](#method.calculate_profit_at){.anchor}

#### fn [calculate_profit_at](../../pricing/trait.Profit.html#tymethod.calculate_profit_at){.fn}( &self, \_price: &[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, ) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Decimal](../../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}, [PricingError](../../error/pricing/enum.PricingError.html "enum optionstratlib::error::pricing::PricingError"){.enum}\> {#fn-calculate_profit_at-self-_price-positive---resultdecimal-pricingerror .code-header}
:::

::: docblock
Calculates the profit at a specified price. [Read
more](../../pricing/trait.Profit.html#tymethod.calculate_profit_at)
:::

::: {#method.get_point_at_price .section .method .trait-impl}
[Source](../../../src/optionstratlib/pricing/payoff.rs.html#219-225){.src
.rightside}[§](#method.get_point_at_price){.anchor}

#### fn [get_point_at_price](../../pricing/trait.Profit.html#method.get_point_at_price){.fn}( &self, \_price: &[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, ) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<([Decimal](../../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}, [Decimal](../../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}), [PricingError](../../error/pricing/enum.PricingError.html "enum optionstratlib::error::pricing::PricingError"){.enum}\> {#fn-get_point_at_price-self-_price-positive---resultdecimal-decimal-pricingerror .code-header}
:::

::: docblock
Creates a chart point representation of the profit at the given price.
[Read more](../../pricing/trait.Profit.html#method.get_point_at_price)
:::
:::::::
:::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::

## Auto Trait Implementations[§](#synthetic-implementations){.anchor} {#synthetic-implementations .section-header}

:::::::::::::: {#synthetic-implementations-list}
::: {#impl-Freeze-for-RandomWalk%3CX,+Y%3E .section .impl}
[§](#impl-Freeze-for-RandomWalk%3CX,+Y%3E){.anchor}

### impl\<X, Y\> [Freeze](https://doc.rust-lang.org/1.91.1/core/marker/trait.Freeze.html "trait core::marker::Freeze"){.trait} for [RandomWalk](struct.RandomWalk.html "struct optionstratlib::simulation::randomwalk::RandomWalk"){.struct}\<X, Y\> {#implx-y-freeze-for-randomwalkx-y .code-header}
:::

:::: {#impl-RefUnwindSafe-for-RandomWalk%3CX,+Y%3E .section .impl}
[§](#impl-RefUnwindSafe-for-RandomWalk%3CX,+Y%3E){.anchor}

### impl\<X, Y\> [RefUnwindSafe](https://doc.rust-lang.org/1.91.1/core/panic/unwind_safe/trait.RefUnwindSafe.html "trait core::panic::unwind_safe::RefUnwindSafe"){.trait} for [RandomWalk](struct.RandomWalk.html "struct optionstratlib::simulation::randomwalk::RandomWalk"){.struct}\<X, Y\> {#implx-y-refunwindsafe-for-randomwalkx-y .code-header}

::: where
where X:
[RefUnwindSafe](https://doc.rust-lang.org/1.91.1/core/panic/unwind_safe/trait.RefUnwindSafe.html "trait core::panic::unwind_safe::RefUnwindSafe"){.trait},
Y:
[RefUnwindSafe](https://doc.rust-lang.org/1.91.1/core/panic/unwind_safe/trait.RefUnwindSafe.html "trait core::panic::unwind_safe::RefUnwindSafe"){.trait},
:::
::::

:::: {#impl-Send-for-RandomWalk%3CX,+Y%3E .section .impl}
[§](#impl-Send-for-RandomWalk%3CX,+Y%3E){.anchor}

### impl\<X, Y\> [Send](https://doc.rust-lang.org/1.91.1/core/marker/trait.Send.html "trait core::marker::Send"){.trait} for [RandomWalk](struct.RandomWalk.html "struct optionstratlib::simulation::randomwalk::RandomWalk"){.struct}\<X, Y\> {#implx-y-send-for-randomwalkx-y .code-header}

::: where
where X:
[Send](https://doc.rust-lang.org/1.91.1/core/marker/trait.Send.html "trait core::marker::Send"){.trait},
Y:
[Send](https://doc.rust-lang.org/1.91.1/core/marker/trait.Send.html "trait core::marker::Send"){.trait},
:::
::::

:::: {#impl-Sync-for-RandomWalk%3CX,+Y%3E .section .impl}
[§](#impl-Sync-for-RandomWalk%3CX,+Y%3E){.anchor}

### impl\<X, Y\> [Sync](https://doc.rust-lang.org/1.91.1/core/marker/trait.Sync.html "trait core::marker::Sync"){.trait} for [RandomWalk](struct.RandomWalk.html "struct optionstratlib::simulation::randomwalk::RandomWalk"){.struct}\<X, Y\> {#implx-y-sync-for-randomwalkx-y .code-header}

::: where
where X:
[Sync](https://doc.rust-lang.org/1.91.1/core/marker/trait.Sync.html "trait core::marker::Sync"){.trait},
Y:
[Sync](https://doc.rust-lang.org/1.91.1/core/marker/trait.Sync.html "trait core::marker::Sync"){.trait},
:::
::::

:::: {#impl-Unpin-for-RandomWalk%3CX,+Y%3E .section .impl}
[§](#impl-Unpin-for-RandomWalk%3CX,+Y%3E){.anchor}

### impl\<X, Y\> [Unpin](https://doc.rust-lang.org/1.91.1/core/marker/trait.Unpin.html "trait core::marker::Unpin"){.trait} for [RandomWalk](struct.RandomWalk.html "struct optionstratlib::simulation::randomwalk::RandomWalk"){.struct}\<X, Y\> {#implx-y-unpin-for-randomwalkx-y .code-header}

::: where
where X:
[Unpin](https://doc.rust-lang.org/1.91.1/core/marker/trait.Unpin.html "trait core::marker::Unpin"){.trait},
Y:
[Unpin](https://doc.rust-lang.org/1.91.1/core/marker/trait.Unpin.html "trait core::marker::Unpin"){.trait},
:::
::::

:::: {#impl-UnwindSafe-for-RandomWalk%3CX,+Y%3E .section .impl}
[§](#impl-UnwindSafe-for-RandomWalk%3CX,+Y%3E){.anchor}

### impl\<X, Y\> [UnwindSafe](https://doc.rust-lang.org/1.91.1/core/panic/unwind_safe/trait.UnwindSafe.html "trait core::panic::unwind_safe::UnwindSafe"){.trait} for [RandomWalk](struct.RandomWalk.html "struct optionstratlib::simulation::randomwalk::RandomWalk"){.struct}\<X, Y\> {#implx-y-unwindsafe-for-randomwalkx-y .code-header}

::: where
where X:
[UnwindSafe](https://doc.rust-lang.org/1.91.1/core/panic/unwind_safe/trait.UnwindSafe.html "trait core::panic::unwind_safe::UnwindSafe"){.trait},
Y:
[UnwindSafe](https://doc.rust-lang.org/1.91.1/core/panic/unwind_safe/trait.UnwindSafe.html "trait core::panic::unwind_safe::UnwindSafe"){.trait},
:::
::::
::::::::::::::

## Blanket Implementations[§](#blanket-implementations){.anchor} {#blanket-implementations .section-header}

:::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::: {#blanket-implementations-list}
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

:::: {#impl-CloneToUninit-for-T .section .impl}
[Source](https://doc.rust-lang.org/1.91.1/src/core/clone.rs.html#515){.src
.rightside}[§](#impl-CloneToUninit-for-T){.anchor}

### impl\<T\> [CloneToUninit](https://doc.rust-lang.org/1.91.1/core/clone/trait.CloneToUninit.html "trait core::clone::CloneToUninit"){.trait} for T {#implt-clonetouninit-for-t .code-header}

::: where
where T:
[Clone](https://doc.rust-lang.org/1.91.1/core/clone/trait.Clone.html "trait core::clone::Clone"){.trait},
:::
::::

:::::: impl-items
::: {#method.clone_to_uninit .section .method .trait-impl}
[Source](https://doc.rust-lang.org/1.91.1/src/core/clone.rs.html#517){.src
.rightside}[§](#method.clone_to_uninit){.anchor}

#### unsafe fn [clone_to_uninit](https://doc.rust-lang.org/1.91.1/core/clone/trait.CloneToUninit.html#tymethod.clone_to_uninit){.fn}(&self, dest: [\*mut](https://doc.rust-lang.org/1.91.1/std/primitive.pointer.html){.primitive} [u8](https://doc.rust-lang.org/1.91.1/std/primitive.u8.html){.primitive}) {#unsafe-fn-clone_to_uninitself-dest-mut-u8 .code-header}
:::

[]{.item-info}

::: {.stab .unstable}
🔬This is a nightly-only experimental API. (`clone_to_uninit`)
:::

::: docblock
Performs copy-assignment from `self` to `dest`. [Read
more](https://doc.rust-lang.org/1.91.1/core/clone/trait.CloneToUninit.html#tymethod.clone_to_uninit)
:::
::::::

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
[Source](../../../src/tracing/instrument.rs.html#325){.src
.rightside}[§](#impl-Instrument-for-T){.anchor}

### impl\<T\> [Instrument](../../../tracing/instrument/trait.Instrument.html "trait tracing::instrument::Instrument"){.trait} for T {#implt-instrument-for-t .code-header}
:::

::::::: impl-items
::: {#method.instrument .section .method .trait-impl}
[Source](../../../src/tracing/instrument.rs.html#86){.src
.rightside}[§](#method.instrument){.anchor}

#### fn [instrument](../../../tracing/instrument/trait.Instrument.html#method.instrument){.fn}(self, span: [Span](../../../tracing/span/struct.Span.html "struct tracing::span::Span"){.struct}) -\> [Instrumented](../../../tracing/instrument/struct.Instrumented.html "struct tracing::instrument::Instrumented"){.struct}\<Self\> {#fn-instrumentself-span-span---instrumentedself .code-header}
:::

::: docblock
Instruments this type with the provided
[`Span`](../../../tracing/span/struct.Span.html "struct tracing::span::Span"),
returning an `Instrumented` wrapper. [Read
more](../../../tracing/instrument/trait.Instrument.html#method.instrument)
:::

::: {#method.in_current_span .section .method .trait-impl}
[Source](../../../src/tracing/instrument.rs.html#128){.src
.rightside}[§](#method.in_current_span){.anchor}

#### fn [in_current_span](../../../tracing/instrument/trait.Instrument.html#method.in_current_span){.fn}(self) -\> [Instrumented](../../../tracing/instrument/struct.Instrumented.html "struct tracing::instrument::Instrumented"){.struct}\<Self\> {#fn-in_current_spanself---instrumentedself .code-header}
:::

::: docblock
Instruments this type with the
[current](../../../tracing/span/struct.Span.html#method.current "associated function tracing::span::Span::current")
[`Span`](../../../tracing/span/struct.Span.html "struct tracing::span::Span"),
returning an `Instrumented` wrapper. [Read
more](../../../tracing/instrument/trait.Instrument.html#method.in_current_span)
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
[Source](../../../src/either/into_either.rs.html#64){.src
.rightside}[§](#impl-IntoEither-for-T){.anchor}

### impl\<T\> [IntoEither](../../../either/into_either/trait.IntoEither.html "trait either::into_either::IntoEither"){.trait} for T {#implt-intoeither-for-t .code-header}
:::

:::::::: impl-items
::: {#method.into_either .section .method .trait-impl}
[Source](../../../src/either/into_either.rs.html#29){.src
.rightside}[§](#method.into_either){.anchor}

#### fn [into_either](../../../either/into_either/trait.IntoEither.html#method.into_either){.fn}(self, into_left: [bool](https://doc.rust-lang.org/1.91.1/std/primitive.bool.html){.primitive}) -\> [Either](../../../either/enum.Either.html "enum either::Either"){.enum}\<Self, Self\> {#fn-into_eitherself-into_left-bool---eitherself-self .code-header}
:::

::: docblock
Converts `self` into a
[`Left`](../../../either/enum.Either.html#variant.Left "variant either::Either::Left")
variant of
[`Either<Self, Self>`](../../../either/enum.Either.html "enum either::Either")
if `into_left` is `true`. Converts `self` into a
[`Right`](../../../either/enum.Either.html#variant.Right "variant either::Either::Right")
variant of
[`Either<Self, Self>`](../../../either/enum.Either.html "enum either::Either")
otherwise. [Read
more](../../../either/into_either/trait.IntoEither.html#method.into_either)
:::

:::: {#method.into_either_with .section .method .trait-impl}
[Source](../../../src/either/into_either.rs.html#55-57){.src
.rightside}[§](#method.into_either_with){.anchor}

#### fn [into_either_with](../../../either/into_either/trait.IntoEither.html#method.into_either_with){.fn}\<F\>(self, into_left: F) -\> [Either](../../../either/enum.Either.html "enum either::Either"){.enum}\<Self, Self\> {#fn-into_either_withfself-into_left-f---eitherself-self .code-header}

::: where
where F:
[FnOnce](https://doc.rust-lang.org/1.91.1/core/ops/function/trait.FnOnce.html "trait core::ops::function::FnOnce"){.trait}(&Self)
-\>
[bool](https://doc.rust-lang.org/1.91.1/std/primitive.bool.html){.primitive},
:::
::::

::: docblock
Converts `self` into a
[`Left`](../../../either/enum.Either.html#variant.Left "variant either::Either::Left")
variant of
[`Either<Self, Self>`](../../../either/enum.Either.html "enum either::Either")
if `into_left(&self)` returns `true`. Converts `self` into a
[`Right`](../../../either/enum.Either.html#variant.Right "variant either::Either::Right")
variant of
[`Either<Self, Self>`](../../../either/enum.Either.html "enum either::Either")
otherwise. [Read
more](../../../either/into_either/trait.IntoEither.html#method.into_either_with)
:::
::::::::

::: {#impl-Pointable-for-T .section .impl}
[Source](../../../src/crossbeam_epoch/atomic.rs.html#194){.src
.rightside}[§](#impl-Pointable-for-T){.anchor}

### impl\<T\> [Pointable](../../../crossbeam_epoch/atomic/trait.Pointable.html "trait crossbeam_epoch::atomic::Pointable"){.trait} for T {#implt-pointable-for-t .code-header}
:::

::::::::::::::: impl-items
::: {#associatedconstant.ALIGN .section .associatedconstant .trait-impl}
[Source](../../../src/crossbeam_epoch/atomic.rs.html#195){.src
.rightside}[§](#associatedconstant.ALIGN){.anchor}

#### const [ALIGN](../../../crossbeam_epoch/atomic/trait.Pointable.html#associatedconstant.ALIGN){.constant}: [usize](https://doc.rust-lang.org/1.91.1/std/primitive.usize.html){.primitive} {#const-align-usize .code-header}
:::

::: docblock
The alignment of pointer.
:::

::: {#associatedtype.Init .section .associatedtype .trait-impl}
[Source](../../../src/crossbeam_epoch/atomic.rs.html#197){.src
.rightside}[§](#associatedtype.Init){.anchor}

#### type [Init](../../../crossbeam_epoch/atomic/trait.Pointable.html#associatedtype.Init){.associatedtype} = T {#type-init-t .code-header}
:::

::: docblock
The type for initializers.
:::

::: {#method.init .section .method .trait-impl}
[Source](../../../src/crossbeam_epoch/atomic.rs.html#199){.src
.rightside}[§](#method.init){.anchor}

#### unsafe fn [init](../../../crossbeam_epoch/atomic/trait.Pointable.html#tymethod.init){.fn}(init: \<T as [Pointable](../../../crossbeam_epoch/atomic/trait.Pointable.html "trait crossbeam_epoch::atomic::Pointable"){.trait}\>::[Init](../../../crossbeam_epoch/atomic/trait.Pointable.html#associatedtype.Init "type crossbeam_epoch::atomic::Pointable::Init"){.associatedtype}) -\> [usize](https://doc.rust-lang.org/1.91.1/std/primitive.usize.html){.primitive} {#unsafe-fn-initinit-t-as-pointableinit---usize .code-header}
:::

::: docblock
Initializes a with the given initializer. [Read
more](../../../crossbeam_epoch/atomic/trait.Pointable.html#tymethod.init)
:::

::: {#method.deref .section .method .trait-impl}
[Source](../../../src/crossbeam_epoch/atomic.rs.html#203){.src
.rightside}[§](#method.deref){.anchor}

#### unsafe fn [deref](../../../crossbeam_epoch/atomic/trait.Pointable.html#tymethod.deref){.fn}\<\'a\>(ptr: [usize](https://doc.rust-lang.org/1.91.1/std/primitive.usize.html){.primitive}) -\> [&\'a T](https://doc.rust-lang.org/1.91.1/std/primitive.reference.html){.primitive} {#unsafe-fn-derefaptr-usize---a-t .code-header}
:::

::: docblock
Dereferences the given pointer. [Read
more](../../../crossbeam_epoch/atomic/trait.Pointable.html#tymethod.deref)
:::

::: {#method.deref_mut .section .method .trait-impl}
[Source](../../../src/crossbeam_epoch/atomic.rs.html#207){.src
.rightside}[§](#method.deref_mut){.anchor}

#### unsafe fn [deref_mut](../../../crossbeam_epoch/atomic/trait.Pointable.html#tymethod.deref_mut){.fn}\<\'a\>(ptr: [usize](https://doc.rust-lang.org/1.91.1/std/primitive.usize.html){.primitive}) -\> [&\'a mut T](https://doc.rust-lang.org/1.91.1/std/primitive.reference.html){.primitive} {#unsafe-fn-deref_mutaptr-usize---a-mut-t .code-header}
:::

::: docblock
Mutably dereferences the given pointer. [Read
more](../../../crossbeam_epoch/atomic/trait.Pointable.html#tymethod.deref_mut)
:::

::: {#method.drop .section .method .trait-impl}
[Source](../../../src/crossbeam_epoch/atomic.rs.html#211){.src
.rightside}[§](#method.drop){.anchor}

#### unsafe fn [drop](../../../crossbeam_epoch/atomic/trait.Pointable.html#tymethod.drop){.fn}(ptr: [usize](https://doc.rust-lang.org/1.91.1/std/primitive.usize.html){.primitive}) {#unsafe-fn-dropptr-usize .code-header}
:::

::: docblock
Drops the object pointed to by the given pointer. [Read
more](../../../crossbeam_epoch/atomic/trait.Pointable.html#tymethod.drop)
:::
:::::::::::::::

::: {#impl-Same-for-T .section .impl}
[Source](../../../src/typenum/type_operators.rs.html#34){.src
.rightside}[§](#impl-Same-for-T){.anchor}

### impl\<T\> [Same](../../../typenum/type_operators/trait.Same.html "trait typenum::type_operators::Same"){.trait} for T {#implt-same-for-t .code-header}
:::

::::: impl-items
::: {#associatedtype.Output-1 .section .associatedtype .trait-impl}
[Source](../../../src/typenum/type_operators.rs.html#35){.src
.rightside}[§](#associatedtype.Output-1){.anchor}

#### type [Output](../../../typenum/type_operators/trait.Same.html#associatedtype.Output){.associatedtype} = T {#type-output-t .code-header}
:::

::: docblock
Should always be `Self`
:::
:::::

:::: {#impl-SupersetOf%3CSS%3E-for-SP .section .impl}
[Source](../../../src/simba/scalar/subset.rs.html#90){.src
.rightside}[§](#impl-SupersetOf%3CSS%3E-for-SP){.anchor}

### impl\<SS, SP\> [SupersetOf](../../../simba/scalar/subset/trait.SupersetOf.html "trait simba::scalar::subset::SupersetOf"){.trait}\<SS\> for SP {#implss-sp-supersetofss-for-sp .code-header}

::: where
where SS:
[SubsetOf](../../../simba/scalar/subset/trait.SubsetOf.html "trait simba::scalar::subset::SubsetOf"){.trait}\<SP\>,
:::
::::

::::::::::: impl-items
::: {#method.to_subset .section .method .trait-impl}
[Source](../../../src/simba/scalar/subset.rs.html#92){.src
.rightside}[§](#method.to_subset){.anchor}

#### fn [to_subset](../../../simba/scalar/subset/trait.SupersetOf.html#method.to_subset){.fn}(&self) -\> [Option](https://doc.rust-lang.org/1.91.1/core/option/enum.Option.html "enum core::option::Option"){.enum}\<SS\> {#fn-to_subsetself---optionss .code-header}
:::

::: docblock
The inverse inclusion map: attempts to construct `self` from the
equivalent element of its superset. [Read
more](../../../simba/scalar/subset/trait.SupersetOf.html#method.to_subset)
:::

::: {#method.is_in_subset .section .method .trait-impl}
[Source](../../../src/simba/scalar/subset.rs.html#97){.src
.rightside}[§](#method.is_in_subset){.anchor}

#### fn [is_in_subset](../../../simba/scalar/subset/trait.SupersetOf.html#tymethod.is_in_subset){.fn}(&self) -\> [bool](https://doc.rust-lang.org/1.91.1/std/primitive.bool.html){.primitive} {#fn-is_in_subsetself---bool .code-header}
:::

::: docblock
Checks if `self` is actually part of its subset `T` (and can be
converted to it).
:::

::: {#method.to_subset_unchecked .section .method .trait-impl}
[Source](../../../src/simba/scalar/subset.rs.html#102){.src
.rightside}[§](#method.to_subset_unchecked){.anchor}

#### fn [to_subset_unchecked](../../../simba/scalar/subset/trait.SupersetOf.html#tymethod.to_subset_unchecked){.fn}(&self) -\> SS {#fn-to_subset_uncheckedself---ss .code-header}
:::

::: docblock
Use with care! Same as `self.to_subset` but without any property checks.
Always succeeds.
:::

::: {#method.from_subset .section .method .trait-impl}
[Source](../../../src/simba/scalar/subset.rs.html#107){.src
.rightside}[§](#method.from_subset){.anchor}

#### fn [from_subset](../../../simba/scalar/subset/trait.SupersetOf.html#tymethod.from_subset){.fn}(element: [&SS](https://doc.rust-lang.org/1.91.1/std/primitive.reference.html){.primitive}) -\> SP {#fn-from_subsetelement-ss---sp .code-header}
:::

::: docblock
The inclusion map: converts `self` to the equivalent element of its
superset.
:::
:::::::::::

:::: {#impl-ToOwned-for-T .section .impl}
[Source](https://doc.rust-lang.org/1.91.1/src/alloc/borrow.rs.html#85-87){.src
.rightside}[§](#impl-ToOwned-for-T){.anchor}

### impl\<T\> [ToOwned](https://doc.rust-lang.org/1.91.1/alloc/borrow/trait.ToOwned.html "trait alloc::borrow::ToOwned"){.trait} for T {#implt-toowned-for-t .code-header}

::: where
where T:
[Clone](https://doc.rust-lang.org/1.91.1/core/clone/trait.Clone.html "trait core::clone::Clone"){.trait},
:::
::::

::::::::: impl-items
::: {#associatedtype.Owned .section .associatedtype .trait-impl}
[Source](https://doc.rust-lang.org/1.91.1/src/alloc/borrow.rs.html#89){.src
.rightside}[§](#associatedtype.Owned){.anchor}

#### type [Owned](https://doc.rust-lang.org/1.91.1/alloc/borrow/trait.ToOwned.html#associatedtype.Owned){.associatedtype} = T {#type-owned-t .code-header}
:::

::: docblock
The resulting type after obtaining ownership.
:::

::: {#method.to_owned .section .method .trait-impl}
[Source](https://doc.rust-lang.org/1.91.1/src/alloc/borrow.rs.html#90){.src
.rightside}[§](#method.to_owned){.anchor}

#### fn [to_owned](https://doc.rust-lang.org/1.91.1/alloc/borrow/trait.ToOwned.html#tymethod.to_owned){.fn}(&self) -\> T {#fn-to_ownedself---t .code-header}
:::

::: docblock
Creates owned data from borrowed data, usually by cloning. [Read
more](https://doc.rust-lang.org/1.91.1/alloc/borrow/trait.ToOwned.html#tymethod.to_owned)
:::

::: {#method.clone_into .section .method .trait-impl}
[Source](https://doc.rust-lang.org/1.91.1/src/alloc/borrow.rs.html#94){.src
.rightside}[§](#method.clone_into){.anchor}

#### fn [clone_into](https://doc.rust-lang.org/1.91.1/alloc/borrow/trait.ToOwned.html#method.clone_into){.fn}(&self, target: [&mut T](https://doc.rust-lang.org/1.91.1/std/primitive.reference.html){.primitive}) {#fn-clone_intoself-target-mut-t .code-header}
:::

::: docblock
Uses borrowed data to replace owned data, usually by cloning. [Read
more](https://doc.rust-lang.org/1.91.1/alloc/borrow/trait.ToOwned.html#method.clone_into)
:::
:::::::::

:::: {#impl-ToString-for-T .section .impl}
[Source](https://doc.rust-lang.org/1.91.1/src/alloc/string.rs.html#2796){.src
.rightside}[§](#impl-ToString-for-T){.anchor}

### impl\<T\> [ToString](https://doc.rust-lang.org/1.91.1/alloc/string/trait.ToString.html "trait alloc::string::ToString"){.trait} for T {#implt-tostring-for-t .code-header}

::: where
where T:
[Display](https://doc.rust-lang.org/1.91.1/core/fmt/trait.Display.html "trait core::fmt::Display"){.trait} +
?[Sized](https://doc.rust-lang.org/1.91.1/core/marker/trait.Sized.html "trait core::marker::Sized"){.trait},
:::
::::

::::: impl-items
::: {#method.to_string .section .method .trait-impl}
[Source](https://doc.rust-lang.org/1.91.1/src/alloc/string.rs.html#2798){.src
.rightside}[§](#method.to_string){.anchor}

#### fn [to_string](https://doc.rust-lang.org/1.91.1/alloc/string/trait.ToString.html#tymethod.to_string){.fn}(&self) -\> [String](https://doc.rust-lang.org/1.91.1/alloc/string/struct.String.html "struct alloc::string::String"){.struct} {#fn-to_stringself---string .code-header}
:::

::: docblock
Converts the given value to a `String`. [Read
more](https://doc.rust-lang.org/1.91.1/alloc/string/trait.ToString.html#tymethod.to_string)
:::
:::::

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
[Source](../../../src/ppv_lite86/types.rs.html#221-223){.src
.rightside}[§](#impl-VZip%3CV%3E-for-T){.anchor}

### impl\<V, T\> [VZip](../../../ppv_lite86/types/trait.VZip.html "trait ppv_lite86::types::VZip"){.trait}\<V\> for T {#implv-t-vzipv-for-t .code-header}

::: where
where V:
[MultiLane](../../../ppv_lite86/types/trait.MultiLane.html "trait ppv_lite86::types::MultiLane"){.trait}\<T\>,
:::
::::

:::: impl-items
::: {#method.vzip .section .method .trait-impl}
[Source](../../../src/ppv_lite86/types.rs.html#226){.src
.rightside}[§](#method.vzip){.anchor}

#### fn [vzip](../../../ppv_lite86/types/trait.VZip.html#tymethod.vzip){.fn}(self) -\> V {#fn-vzipself---v .code-header}
:::
::::

::: {#impl-WithSubscriber-for-T .section .impl}
[Source](../../../src/tracing/instrument.rs.html#393){.src
.rightside}[§](#impl-WithSubscriber-for-T){.anchor}

### impl\<T\> [WithSubscriber](../../../tracing/instrument/trait.WithSubscriber.html "trait tracing::instrument::WithSubscriber"){.trait} for T {#implt-withsubscriber-for-t .code-header}
:::

:::::::: impl-items
:::: {#method.with_subscriber .section .method .trait-impl}
[Source](../../../src/tracing/instrument.rs.html#176-178){.src
.rightside}[§](#method.with_subscriber){.anchor}

#### fn [with_subscriber](../../../tracing/instrument/trait.WithSubscriber.html#method.with_subscriber){.fn}\<S\>(self, subscriber: S) -\> [WithDispatch](../../../tracing/instrument/struct.WithDispatch.html "struct tracing::instrument::WithDispatch"){.struct}\<Self\> {#fn-with_subscribersself-subscriber-s---withdispatchself .code-header}

::: where
where S:
[Into](https://doc.rust-lang.org/1.91.1/core/convert/trait.Into.html "trait core::convert::Into"){.trait}\<[Dispatch](../../../tracing_core/dispatcher/struct.Dispatch.html "struct tracing_core::dispatcher::Dispatch"){.struct}\>,
:::
::::

::: docblock
Attaches the provided
[`Subscriber`](../../../tracing_core/subscriber/trait.Subscriber.html "trait tracing_core::subscriber::Subscriber")
to this type, returning a
[`WithDispatch`](../../../tracing/instrument/struct.WithDispatch.html "struct tracing::instrument::WithDispatch")
wrapper. [Read
more](../../../tracing/instrument/trait.WithSubscriber.html#method.with_subscriber)
:::

::: {#method.with_current_subscriber .section .method .trait-impl}
[Source](../../../src/tracing/instrument.rs.html#228){.src
.rightside}[§](#method.with_current_subscriber){.anchor}

#### fn [with_current_subscriber](../../../tracing/instrument/trait.WithSubscriber.html#method.with_current_subscriber){.fn}(self) -\> [WithDispatch](../../../tracing/instrument/struct.WithDispatch.html "struct tracing::instrument::WithDispatch"){.struct}\<Self\> {#fn-with_current_subscriberself---withdispatchself .code-header}
:::

::: docblock
Attaches the current
[default](../../../tracing/dispatcher/index.html#setting-the-default-subscriber "mod tracing::dispatcher")
[`Subscriber`](../../../tracing_core/subscriber/trait.Subscriber.html "trait tracing_core::subscriber::Subscriber")
to this type, returning a
[`WithDispatch`](../../../tracing/instrument/struct.WithDispatch.html "struct tracing::instrument::WithDispatch")
wrapper. [Read
more](../../../tracing/instrument/trait.WithSubscriber.html#method.with_current_subscriber)
:::
::::::::
::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
:::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
