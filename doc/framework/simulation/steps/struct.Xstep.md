::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::: width-limiter
:::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../../index.html)::[simulation](../index.html)::[steps](index.html)
:::

# Struct [Xstep]{.struct}Copy item path

[[Source](../../../src/optionstratlib/simulation/steps/x.rs.html#43-51){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub struct Xstep<T>where
    T: Copy + Into<Positive> + AddAssign + Display,{ /* private fields */ }
```

Expand description

::: docblock
Represents a step in a time series with an indexed value at a specific
time point.

This struct encapsulates a value at a specific point in a time series,
tracking not only the value itself but also its position (index), the
time unit it belongs to, and its associated datetime. It's designed to
be used in financial and statistical analysis where tracking values
across time is essential.

The generic parameter `T` allows for different value types while
ensuring they can be incrementally accumulated (via `AddAssign`) and
converted to a `Positive` type for calculations that require
non-negative values.

## [§](#type-parameters){.doc-anchor}Type Parameters

- `T` - The type of the value being stored, which must implement
  `AddAssign`, be convertible to `Positive`, and be `Copy`.

## [§](#fields-1){.doc-anchor}Fields {#fields-1}

- `index` - An integer representing the position of this step in a
  sequence.

- `value` - The actual data value at this time step, of generic type
  `T`.

- `time_unit` - Defines the time granularity (e.g., day, hour, minute)
  for this step.

- `datetime` - The specific expiration date associated with this time
  step, which can be either a relative number of days or an absolute
  datetime.
:::

## Implementations[§](#implementations){.anchor} {#implementations .section-header}

::::::::::::::::::::::::: {#implementations-list}
::::: {#impl-Xstep%3CT%3E .section .impl}
[Source](../../../src/optionstratlib/simulation/steps/x.rs.html#80-234){.src
.rightside}[§](#impl-Xstep%3CT%3E){.anchor}

### impl\<T\> [Xstep](struct.Xstep.html "struct optionstratlib::simulation::steps::Xstep"){.struct}\<T\> {#implt-xstept .code-header}

::: where
where T:
[Copy](https://doc.rust-lang.org/1.86.0/core/marker/trait.Copy.html "trait core::marker::Copy"){.trait} +
[Into](https://doc.rust-lang.org/1.86.0/core/convert/trait.Into.html "trait core::convert::Into"){.trait}\<[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\> +
[AddAssign](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.AddAssign.html "trait core::ops::arith::AddAssign"){.trait} +
[Display](https://doc.rust-lang.org/1.86.0/core/fmt/trait.Display.html "trait core::fmt::Display"){.trait},
:::

::: docblock
Implementation for the `Xstep<T>` struct, which represents a time step
in a simulation.
:::
:::::

:::: docblock
The `Xstep<T>` struct is designed to handle sequential time steps in
financial simulations, allowing for forward and backward movement in
time. The struct maintains an index counter, a value of type T, a time
unit, and an expiration date.

#### [§](#type-parameters-1){.doc-anchor}Type Parameters

- `T` - A type that implements `AddAssign`, can be converted into
  `Positive`, and is `Copy`. This typically represents a numeric value
  used for calculating time differences.

#### [§](#examples){.doc-anchor}Examples

::: example-wrap
``` {.rust .rust-example-rendered}
// Create a step with 7 days as the value, using days as the time unit
use optionstratlib::{pos, ExpirationDate};
use optionstratlib::simulation::steps::Xstep;
use optionstratlib::utils::TimeFrame;
let step = Xstep::new(7, TimeFrame::Day, ExpirationDate::Days(pos!(30.0)));

// Move to the next step (forward in time)
let next_step = step.next();

// Move to the previous step (backward in time)
let prev_step = step.previous();
```
:::
::::

::::::::::::::::::: impl-items
::: {#method.new .section .method}
[Source](../../../src/optionstratlib/simulation/steps/x.rs.html#100-113){.src
.rightside}

#### pub fn [new](#method.new){.fn}(value: T, time_unit: [TimeFrame](../../utils/time/enum.TimeFrame.html "enum optionstratlib::utils::time::TimeFrame"){.enum}, datetime: [ExpirationDate](../../model/types/enum.ExpirationDate.html "enum optionstratlib::model::types::ExpirationDate"){.enum}) -\> Self {#pub-fn-newvalue-t-time_unit-timeframe-datetime-expirationdate---self .code-header}
:::

::: docblock
Creates a new `Xstep` with the specified value, time unit, and datetime.

##### [§](#parameters){.doc-anchor}Parameters

- `value` - The step size value
- `time_unit` - The time unit for this step (e.g., Day, Week, Month)
- `datetime` - The expiration date, which must be of type
  `ExpirationDate::Days`

##### [§](#panics){.doc-anchor}Panics

This method will panic if `datetime` is of type
`ExpirationDate::DateTime` as only the `ExpirationDate::Days` variant is
currently supported.

##### [§](#returns){.doc-anchor}Returns

A new `Xstep<T>` instance initialized with the provided parameters and
index set to 0.
:::

::: {#method.index .section .method}
[Source](../../../src/optionstratlib/simulation/steps/x.rs.html#122-124){.src
.rightside}

#### pub fn [index](#method.index){.fn}(&self) -\> &[i32](https://doc.rust-lang.org/1.86.0/std/primitive.i32.html){.primitive} {#pub-fn-indexself---i32 .code-header}
:::

::: docblock
Returns a reference to the index of the time step.

The index represents the position of this step in a sequence of time
steps.

##### [§](#returns-1){.doc-anchor}Returns

- `&i32` - A reference to the index value.
:::

::: {#method.step_size_in_time .section .method}
[Source](../../../src/optionstratlib/simulation/steps/x.rs.html#134-136){.src
.rightside}

#### pub fn [step_size_in_time](#method.step_size_in_time){.fn}(&self) -\> [&T](https://doc.rust-lang.org/1.86.0/std/primitive.reference.html){.primitive} {#pub-fn-step_size_in_timeself---t .code-header}
:::

::: docblock
Returns a reference to the step size in time units.

This represents the magnitude of this time step in the context of its
time frame, such as the number of days, hours, or other time units.

##### [§](#returns-2){.doc-anchor}Returns

- `&T` - A reference to the step size value.
:::

::: {#method.time_unit .section .method}
[Source](../../../src/optionstratlib/simulation/steps/x.rs.html#145-147){.src
.rightside}

#### pub fn [time_unit](#method.time_unit){.fn}(&self) -\> &[TimeFrame](../../utils/time/enum.TimeFrame.html "enum optionstratlib::utils::time::TimeFrame"){.enum} {#pub-fn-time_unitself---timeframe .code-header}
:::

::: docblock
Returns a reference to the time unit associated with this step.

The time unit defines the granularity of the time measurement (e.g.,
day, hour, minute).

##### [§](#returns-3){.doc-anchor}Returns

- `&TimeFrame` - A reference to the time unit enumeration.
:::

::: {#method.datetime .section .method}
[Source](../../../src/optionstratlib/simulation/steps/x.rs.html#157-159){.src
.rightside}

#### pub fn [datetime](#method.datetime){.fn}(&self) -\> &[ExpirationDate](../../model/types/enum.ExpirationDate.html "enum optionstratlib::model::types::ExpirationDate"){.enum} {#pub-fn-datetimeself---expirationdate .code-header}
:::

::: docblock
Returns a reference to the datetime associated with this step.

This represents the specific point in time (as an expiration date) that
this step corresponds to, which can be either a relative value or an
absolute datetime.

##### [§](#returns-4){.doc-anchor}Returns

- `&ExpirationDate` - A reference to the expiration date.
:::

::: {#method.days_left .section .method}
[Source](../../../src/optionstratlib/simulation/steps/x.rs.html#172-174){.src
.rightside}

#### pub fn [days_left](#method.days_left){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, [Box](https://doc.rust-lang.org/1.86.0/alloc/boxed/struct.Box.html "struct alloc::boxed::Box"){.struct}\<dyn [Error](https://doc.rust-lang.org/1.86.0/core/error/trait.Error.html "trait core::error::Error"){.trait}\>\> {#pub-fn-days_leftself---resultpositive-boxdyn-error .code-header}
:::

::: docblock
Calculates the number of days left until the expiry date associated with
this step.

This method delegates to the underlying `ExpirationDate` component to
determine the days remaining until expiration. It provides a consistent
way to access time-to-expiry regardless of how the date was originally
specified (as days or as an absolute datetime).

##### [§](#returns-5){.doc-anchor}Returns

- `Result<Positive, Box<dyn Error>>` - A positive decimal value
  representing the number of days until expiration, or an error if the
  calculation cannot be performed.
:::

::: {#method.next .section .method}
[Source](../../../src/optionstratlib/simulation/steps/x.rs.html#184-209){.src
.rightside}

#### pub fn [next](#method.next){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<Self, [Box](https://doc.rust-lang.org/1.86.0/alloc/boxed/struct.Box.html "struct alloc::boxed::Box"){.struct}\<dyn [Error](https://doc.rust-lang.org/1.86.0/core/error/trait.Error.html "trait core::error::Error"){.trait}\>\> {#pub-fn-nextself---resultself-boxdyn-error .code-header}
:::

::: docblock
Generates the next step by reducing the expiration days by the step
value.

This method calculates a new `Xstep` instance with its index incremented
by 1, and the expiration date reduced by the equivalent of `value` in
days.

##### [§](#returns-6){.doc-anchor}Returns

A new `Xstep<T>` instance with updated index and datetime values.
:::

::: {#method.previous .section .method}
[Source](../../../src/optionstratlib/simulation/steps/x.rs.html#219-233){.src
.rightside}

#### pub fn [previous](#method.previous){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<Self, [Box](https://doc.rust-lang.org/1.86.0/alloc/boxed/struct.Box.html "struct alloc::boxed::Box"){.struct}\<dyn [Error](https://doc.rust-lang.org/1.86.0/core/error/trait.Error.html "trait core::error::Error"){.trait}\>\> {#pub-fn-previousself---resultself-boxdyn-error .code-header}
:::

::: docblock
Generates the previous step by increasing the expiration days by the
step value.

This method calculates a new `Xstep` instance with its index decremented
by 1, and the expiration date increased by the equivalent of `value` in
days.

##### [§](#returns-7){.doc-anchor}Returns

A new `Xstep<T>` instance with updated index and datetime values.
:::
:::::::::::::::::::
:::::::::::::::::::::::::

## Trait Implementations[§](#trait-implementations){.anchor} {#trait-implementations .section-header}

:::::::::::::::::::::::::::: {#trait-implementations-list}
:::: {#impl-Clone-for-Xstep%3CT%3E .section .impl}
[Source](../../../src/optionstratlib/simulation/steps/x.rs.html#42){.src
.rightside}[§](#impl-Clone-for-Xstep%3CT%3E){.anchor}

### impl\<T\> [Clone](https://doc.rust-lang.org/1.86.0/core/clone/trait.Clone.html "trait core::clone::Clone"){.trait} for [Xstep](struct.Xstep.html "struct optionstratlib::simulation::steps::Xstep"){.struct}\<T\> {#implt-clone-for-xstept .code-header}

::: where
where T:
[Copy](https://doc.rust-lang.org/1.86.0/core/marker/trait.Copy.html "trait core::marker::Copy"){.trait} +
[Into](https://doc.rust-lang.org/1.86.0/core/convert/trait.Into.html "trait core::convert::Into"){.trait}\<[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\> +
[AddAssign](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.AddAssign.html "trait core::ops::arith::AddAssign"){.trait} +
[Display](https://doc.rust-lang.org/1.86.0/core/fmt/trait.Display.html "trait core::fmt::Display"){.trait} +
[Clone](https://doc.rust-lang.org/1.86.0/core/clone/trait.Clone.html "trait core::clone::Clone"){.trait},
:::
::::

::::::: impl-items
::: {#method.clone .section .method .trait-impl}
[Source](../../../src/optionstratlib/simulation/steps/x.rs.html#42){.src
.rightside}[§](#method.clone){.anchor}

#### fn [clone](https://doc.rust-lang.org/1.86.0/core/clone/trait.Clone.html#tymethod.clone){.fn}(&self) -\> [Xstep](struct.Xstep.html "struct optionstratlib::simulation::steps::Xstep"){.struct}\<T\> {#fn-cloneself---xstept .code-header}
:::

::: docblock
Returns a copy of the value. [Read
more](https://doc.rust-lang.org/1.86.0/core/clone/trait.Clone.html#tymethod.clone)
:::

::: {#method.clone_from .section .method .trait-impl}
[[1.0.0]{.since title="Stable since Rust version 1.0.0"} ·
[Source](https://doc.rust-lang.org/1.86.0/src/core/clone.rs.html#174){.src}]{.rightside}[§](#method.clone_from){.anchor}

#### fn [clone_from](https://doc.rust-lang.org/1.86.0/core/clone/trait.Clone.html#method.clone_from){.fn}(&mut self, source: &Self) {#fn-clone_frommut-self-source-self .code-header}
:::

::: docblock
Performs copy-assignment from `source`. [Read
more](https://doc.rust-lang.org/1.86.0/core/clone/trait.Clone.html#method.clone_from)
:::
:::::::

:::: {#impl-Debug-for-Xstep%3CT%3E .section .impl}
[Source](../../../src/optionstratlib/simulation/steps/x.rs.html#42){.src
.rightside}[§](#impl-Debug-for-Xstep%3CT%3E){.anchor}

### impl\<T\> [Debug](https://doc.rust-lang.org/1.86.0/core/fmt/trait.Debug.html "trait core::fmt::Debug"){.trait} for [Xstep](struct.Xstep.html "struct optionstratlib::simulation::steps::Xstep"){.struct}\<T\> {#implt-debug-for-xstept .code-header}

::: where
where T:
[Copy](https://doc.rust-lang.org/1.86.0/core/marker/trait.Copy.html "trait core::marker::Copy"){.trait} +
[Into](https://doc.rust-lang.org/1.86.0/core/convert/trait.Into.html "trait core::convert::Into"){.trait}\<[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\> +
[AddAssign](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.AddAssign.html "trait core::ops::arith::AddAssign"){.trait} +
[Display](https://doc.rust-lang.org/1.86.0/core/fmt/trait.Display.html "trait core::fmt::Display"){.trait} +
[Debug](https://doc.rust-lang.org/1.86.0/core/fmt/trait.Debug.html "trait core::fmt::Debug"){.trait},
:::
::::

::::: impl-items
::: {#method.fmt .section .method .trait-impl}
[Source](../../../src/optionstratlib/simulation/steps/x.rs.html#42){.src
.rightside}[§](#method.fmt){.anchor}

#### fn [fmt](https://doc.rust-lang.org/1.86.0/core/fmt/trait.Debug.html#tymethod.fmt){.fn}(&self, f: &mut [Formatter](https://doc.rust-lang.org/1.86.0/core/fmt/struct.Formatter.html "struct core::fmt::Formatter"){.struct}\<\'\_\>) -\> [Result](https://doc.rust-lang.org/1.86.0/core/fmt/type.Result.html "type core::fmt::Result"){.type} {#fn-fmtself-f-mut-formatter_---result .code-header}
:::

::: docblock
Formats the value using the given formatter. [Read
more](https://doc.rust-lang.org/1.86.0/core/fmt/trait.Debug.html#tymethod.fmt)
:::
:::::

:::: {#impl-Display-for-Xstep%3CT%3E .section .impl}
[Source](../../../src/optionstratlib/simulation/steps/x.rs.html#236-247){.src
.rightside}[§](#impl-Display-for-Xstep%3CT%3E){.anchor}

### impl\<T\> [Display](https://doc.rust-lang.org/1.86.0/core/fmt/trait.Display.html "trait core::fmt::Display"){.trait} for [Xstep](struct.Xstep.html "struct optionstratlib::simulation::steps::Xstep"){.struct}\<T\> {#implt-display-for-xstept .code-header}

::: where
where T:
[Copy](https://doc.rust-lang.org/1.86.0/core/marker/trait.Copy.html "trait core::marker::Copy"){.trait} +
[Into](https://doc.rust-lang.org/1.86.0/core/convert/trait.Into.html "trait core::convert::Into"){.trait}\<[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\> +
[AddAssign](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.AddAssign.html "trait core::ops::arith::AddAssign"){.trait} +
[Display](https://doc.rust-lang.org/1.86.0/core/fmt/trait.Display.html "trait core::fmt::Display"){.trait},
:::
::::

::::: impl-items
::: {#method.fmt-1 .section .method .trait-impl}
[Source](../../../src/optionstratlib/simulation/steps/x.rs.html#240-246){.src
.rightside}[§](#method.fmt-1){.anchor}

#### fn [fmt](https://doc.rust-lang.org/1.86.0/core/fmt/trait.Display.html#tymethod.fmt){.fn}(&self, f: &mut [Formatter](https://doc.rust-lang.org/1.86.0/core/fmt/struct.Formatter.html "struct core::fmt::Formatter"){.struct}\<\'\_\>) -\> [Result](https://doc.rust-lang.org/1.86.0/core/fmt/type.Result.html "type core::fmt::Result"){.type} {#fn-fmtself-f-mut-formatter_---result-1 .code-header}
:::

::: docblock
Formats the value using the given formatter. [Read
more](https://doc.rust-lang.org/1.86.0/core/fmt/trait.Display.html#tymethod.fmt)
:::
:::::

:::: {#impl-Serialize-for-Xstep%3CT%3E .section .impl}
[Source](../../../src/optionstratlib/simulation/steps/x.rs.html#249-267){.src
.rightside}[§](#impl-Serialize-for-Xstep%3CT%3E){.anchor}

### impl\<T\> [Serialize](https://docs.rs/serde/1.0.219/serde/ser/trait.Serialize.html "trait serde::ser::Serialize"){.trait} for [Xstep](struct.Xstep.html "struct optionstratlib::simulation::steps::Xstep"){.struct}\<T\> {#implt-serialize-for-xstept .code-header}

::: where
where T:
[Copy](https://doc.rust-lang.org/1.86.0/core/marker/trait.Copy.html "trait core::marker::Copy"){.trait} +
[Into](https://doc.rust-lang.org/1.86.0/core/convert/trait.Into.html "trait core::convert::Into"){.trait}\<[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\> +
[AddAssign](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.AddAssign.html "trait core::ops::arith::AddAssign"){.trait} +
[Display](https://doc.rust-lang.org/1.86.0/core/fmt/trait.Display.html "trait core::fmt::Display"){.trait} +
[Serialize](https://docs.rs/serde/1.0.219/serde/ser/trait.Serialize.html "trait serde::ser::Serialize"){.trait},
:::
::::

:::::: impl-items
:::: {#method.serialize .section .method .trait-impl}
[Source](../../../src/optionstratlib/simulation/steps/x.rs.html#253-266){.src
.rightside}[§](#method.serialize){.anchor}

#### fn [serialize](https://docs.rs/serde/1.0.219/serde/ser/trait.Serialize.html#tymethod.serialize){.fn}\<S\>(&self, serializer: S) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<S::[Ok](https://docs.rs/serde/1.0.219/serde/ser/trait.Serializer.html#associatedtype.Ok "type serde::ser::Serializer::Ok"){.associatedtype}, S::[Error](https://docs.rs/serde/1.0.219/serde/ser/trait.Serializer.html#associatedtype.Error "type serde::ser::Serializer::Error"){.associatedtype}\> {#fn-serializesself-serializer-s---resultsok-serror .code-header}

::: where
where S:
[Serializer](https://docs.rs/serde/1.0.219/serde/ser/trait.Serializer.html "trait serde::ser::Serializer"){.trait},
:::
::::

::: docblock
Serialize this value into the given Serde serializer. [Read
more](https://docs.rs/serde/1.0.219/serde/ser/trait.Serialize.html#tymethod.serialize)
:::
::::::

:::: {#impl-Copy-for-Xstep%3CT%3E .section .impl}
[Source](../../../src/optionstratlib/simulation/steps/x.rs.html#42){.src
.rightside}[§](#impl-Copy-for-Xstep%3CT%3E){.anchor}

### impl\<T\> [Copy](https://doc.rust-lang.org/1.86.0/core/marker/trait.Copy.html "trait core::marker::Copy"){.trait} for [Xstep](struct.Xstep.html "struct optionstratlib::simulation::steps::Xstep"){.struct}\<T\> {#implt-copy-for-xstept .code-header}

::: where
where T:
[Copy](https://doc.rust-lang.org/1.86.0/core/marker/trait.Copy.html "trait core::marker::Copy"){.trait} +
[Into](https://doc.rust-lang.org/1.86.0/core/convert/trait.Into.html "trait core::convert::Into"){.trait}\<[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\> +
[AddAssign](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.AddAssign.html "trait core::ops::arith::AddAssign"){.trait} +
[Display](https://doc.rust-lang.org/1.86.0/core/fmt/trait.Display.html "trait core::fmt::Display"){.trait} +
[Copy](https://doc.rust-lang.org/1.86.0/core/marker/trait.Copy.html "trait core::marker::Copy"){.trait},
:::
::::
::::::::::::::::::::::::::::

## Auto Trait Implementations[§](#synthetic-implementations){.anchor} {#synthetic-implementations .section-header}

::::::::::::::: {#synthetic-implementations-list}
:::: {#impl-Freeze-for-Xstep%3CT%3E .section .impl}
[§](#impl-Freeze-for-Xstep%3CT%3E){.anchor}

### impl\<T\> [Freeze](https://doc.rust-lang.org/1.86.0/core/marker/trait.Freeze.html "trait core::marker::Freeze"){.trait} for [Xstep](struct.Xstep.html "struct optionstratlib::simulation::steps::Xstep"){.struct}\<T\> {#implt-freeze-for-xstept .code-header}

::: where
where T:
[Freeze](https://doc.rust-lang.org/1.86.0/core/marker/trait.Freeze.html "trait core::marker::Freeze"){.trait},
:::
::::

:::: {#impl-RefUnwindSafe-for-Xstep%3CT%3E .section .impl}
[§](#impl-RefUnwindSafe-for-Xstep%3CT%3E){.anchor}

### impl\<T\> [RefUnwindSafe](https://doc.rust-lang.org/1.86.0/core/panic/unwind_safe/trait.RefUnwindSafe.html "trait core::panic::unwind_safe::RefUnwindSafe"){.trait} for [Xstep](struct.Xstep.html "struct optionstratlib::simulation::steps::Xstep"){.struct}\<T\> {#implt-refunwindsafe-for-xstept .code-header}

::: where
where T:
[RefUnwindSafe](https://doc.rust-lang.org/1.86.0/core/panic/unwind_safe/trait.RefUnwindSafe.html "trait core::panic::unwind_safe::RefUnwindSafe"){.trait},
:::
::::

:::: {#impl-Send-for-Xstep%3CT%3E .section .impl}
[§](#impl-Send-for-Xstep%3CT%3E){.anchor}

### impl\<T\> [Send](https://doc.rust-lang.org/1.86.0/core/marker/trait.Send.html "trait core::marker::Send"){.trait} for [Xstep](struct.Xstep.html "struct optionstratlib::simulation::steps::Xstep"){.struct}\<T\> {#implt-send-for-xstept .code-header}

::: where
where T:
[Send](https://doc.rust-lang.org/1.86.0/core/marker/trait.Send.html "trait core::marker::Send"){.trait},
:::
::::

:::: {#impl-Sync-for-Xstep%3CT%3E .section .impl}
[§](#impl-Sync-for-Xstep%3CT%3E){.anchor}

### impl\<T\> [Sync](https://doc.rust-lang.org/1.86.0/core/marker/trait.Sync.html "trait core::marker::Sync"){.trait} for [Xstep](struct.Xstep.html "struct optionstratlib::simulation::steps::Xstep"){.struct}\<T\> {#implt-sync-for-xstept .code-header}

::: where
where T:
[Sync](https://doc.rust-lang.org/1.86.0/core/marker/trait.Sync.html "trait core::marker::Sync"){.trait},
:::
::::

:::: {#impl-Unpin-for-Xstep%3CT%3E .section .impl}
[§](#impl-Unpin-for-Xstep%3CT%3E){.anchor}

### impl\<T\> [Unpin](https://doc.rust-lang.org/1.86.0/core/marker/trait.Unpin.html "trait core::marker::Unpin"){.trait} for [Xstep](struct.Xstep.html "struct optionstratlib::simulation::steps::Xstep"){.struct}\<T\> {#implt-unpin-for-xstept .code-header}

::: where
where T:
[Unpin](https://doc.rust-lang.org/1.86.0/core/marker/trait.Unpin.html "trait core::marker::Unpin"){.trait},
:::
::::

:::: {#impl-UnwindSafe-for-Xstep%3CT%3E .section .impl}
[§](#impl-UnwindSafe-for-Xstep%3CT%3E){.anchor}

### impl\<T\> [UnwindSafe](https://doc.rust-lang.org/1.86.0/core/panic/unwind_safe/trait.UnwindSafe.html "trait core::panic::unwind_safe::UnwindSafe"){.trait} for [Xstep](struct.Xstep.html "struct optionstratlib::simulation::steps::Xstep"){.struct}\<T\> {#implt-unwindsafe-for-xstept .code-header}

::: where
where T:
[UnwindSafe](https://doc.rust-lang.org/1.86.0/core/panic/unwind_safe/trait.UnwindSafe.html "trait core::panic::unwind_safe::UnwindSafe"){.trait},
:::
::::
:::::::::::::::

## Blanket Implementations[§](#blanket-implementations){.anchor} {#blanket-implementations .section-header}

:::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::: {#blanket-implementations-list}
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

:::: {#impl-CloneToUninit-for-T .section .impl}
[Source](https://doc.rust-lang.org/1.86.0/src/core/clone.rs.html#273){.src
.rightside}[§](#impl-CloneToUninit-for-T){.anchor}

### impl\<T\> [CloneToUninit](https://doc.rust-lang.org/1.86.0/core/clone/trait.CloneToUninit.html "trait core::clone::CloneToUninit"){.trait} for T {#implt-clonetouninit-for-t .code-header}

::: where
where T:
[Clone](https://doc.rust-lang.org/1.86.0/core/clone/trait.Clone.html "trait core::clone::Clone"){.trait},
:::
::::

:::::: impl-items
::: {#method.clone_to_uninit .section .method .trait-impl}
[Source](https://doc.rust-lang.org/1.86.0/src/core/clone.rs.html#275){.src
.rightside}[§](#method.clone_to_uninit){.anchor}

#### unsafe fn [clone_to_uninit](https://doc.rust-lang.org/1.86.0/core/clone/trait.CloneToUninit.html#tymethod.clone_to_uninit){.fn}(&self, dst: [\*mut](https://doc.rust-lang.org/1.86.0/std/primitive.pointer.html){.primitive} [u8](https://doc.rust-lang.org/1.86.0/std/primitive.u8.html){.primitive}) {#unsafe-fn-clone_to_uninitself-dst-mut-u8 .code-header}
:::

[]{.item-info}

::: {.stab .unstable}
🔬This is a nightly-only experimental API. (`clone_to_uninit`)
:::

::: docblock
Performs copy-assignment from `self` to `dst`. [Read
more](https://doc.rust-lang.org/1.86.0/core/clone/trait.CloneToUninit.html#tymethod.clone_to_uninit)
:::
::::::

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

:::: {#impl-ToOwned-for-T .section .impl}
[Source](https://doc.rust-lang.org/1.86.0/src/alloc/borrow.rs.html#82-84){.src
.rightside}[§](#impl-ToOwned-for-T){.anchor}

### impl\<T\> [ToOwned](https://doc.rust-lang.org/1.86.0/alloc/borrow/trait.ToOwned.html "trait alloc::borrow::ToOwned"){.trait} for T {#implt-toowned-for-t .code-header}

::: where
where T:
[Clone](https://doc.rust-lang.org/1.86.0/core/clone/trait.Clone.html "trait core::clone::Clone"){.trait},
:::
::::

::::::::: impl-items
::: {#associatedtype.Owned .section .associatedtype .trait-impl}
[Source](https://doc.rust-lang.org/1.86.0/src/alloc/borrow.rs.html#86){.src
.rightside}[§](#associatedtype.Owned){.anchor}

#### type [Owned](https://doc.rust-lang.org/1.86.0/alloc/borrow/trait.ToOwned.html#associatedtype.Owned){.associatedtype} = T {#type-owned-t .code-header}
:::

::: docblock
The resulting type after obtaining ownership.
:::

::: {#method.to_owned .section .method .trait-impl}
[Source](https://doc.rust-lang.org/1.86.0/src/alloc/borrow.rs.html#87){.src
.rightside}[§](#method.to_owned){.anchor}

#### fn [to_owned](https://doc.rust-lang.org/1.86.0/alloc/borrow/trait.ToOwned.html#tymethod.to_owned){.fn}(&self) -\> T {#fn-to_ownedself---t .code-header}
:::

::: docblock
Creates owned data from borrowed data, usually by cloning. [Read
more](https://doc.rust-lang.org/1.86.0/alloc/borrow/trait.ToOwned.html#tymethod.to_owned)
:::

::: {#method.clone_into .section .method .trait-impl}
[Source](https://doc.rust-lang.org/1.86.0/src/alloc/borrow.rs.html#91){.src
.rightside}[§](#method.clone_into){.anchor}

#### fn [clone_into](https://doc.rust-lang.org/1.86.0/alloc/borrow/trait.ToOwned.html#method.clone_into){.fn}(&self, target: [&mut T](https://doc.rust-lang.org/1.86.0/std/primitive.reference.html){.primitive}) {#fn-clone_intoself-target-mut-t .code-header}
:::

::: docblock
Uses borrowed data to replace owned data, usually by cloning. [Read
more](https://doc.rust-lang.org/1.86.0/alloc/borrow/trait.ToOwned.html#method.clone_into)
:::
:::::::::

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
::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
:::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
