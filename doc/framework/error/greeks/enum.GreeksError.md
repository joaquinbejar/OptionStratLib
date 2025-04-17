:::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::: width-limiter
::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../../index.html)::[error](../index.html)::[greeks](index.html)
:::

# Enum [GreeksError]{.enum}Copy item path

[[Source](../../../src/optionstratlib/error/greeks.rs.html#52-68){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub enum GreeksError {
    MathError(MathErrorKind),
    InputError(InputErrorKind),
    CalculationError(CalculationErrorKind),
    StdError(String),
}
```

Expand description

::: docblock
Represents errors that can occur during options Greek calculations.

This enum encapsulates the various types of errors that might arise
during the calculation of option Greeks (delta, gamma, theta, vega, rho)
and related financial computations. It provides a structured approach to
error handling by categorizing errors based on their nature and source.

`GreeksError` serves as the primary error type for the Greek calculation
system, allowing for precise error reporting and handling across
different calculation stages.
:::

## Variants[§](#variants){.anchor} {#variants .variants .section-header}

::::::::::: variants
::: {#variant.MathError .section .variant}
[§](#variant.MathError){.anchor}

### MathError([MathErrorKind](enum.MathErrorKind.html "enum optionstratlib::error::greeks::MathErrorKind"){.enum}) {#matherrormatherrorkind .code-header}
:::

::: docblock
Errors related to mathematical calculations such as division by zero,
overflow, domain errors, or convergence failures.
:::

::: {#variant.InputError .section .variant}
[§](#variant.InputError){.anchor}

### InputError([InputErrorKind](enum.InputErrorKind.html "enum optionstratlib::error::greeks::InputErrorKind"){.enum}) {#inputerrorinputerrorkind .code-header}
:::

::: docblock
Errors related to input validation, including invalid volatility, time,
price, interest rate, or strike price values.
:::

::: {#variant.CalculationError .section .variant}
[§](#variant.CalculationError){.anchor}

### CalculationError([CalculationErrorKind](enum.CalculationErrorKind.html "enum optionstratlib::error::greeks::CalculationErrorKind"){.enum}) {#calculationerrorcalculationerrorkind .code-header}
:::

::: docblock
Errors specific to the calculation of individual Greeks (delta, gamma,
theta, vega, rho) or other option-related computations.
:::

::: {#variant.StdError .section .variant}
[§](#variant.StdError){.anchor}

### StdError([String](https://doc.rust-lang.org/1.86.0/alloc/string/struct.String.html "struct alloc::string::String"){.struct}) {#stderrorstring .code-header}
:::

::: docblock
Errors originating from standard Rust error types, wrapped as strings
for consistent error handling.
:::
:::::::::::

## Implementations[§](#implementations){.anchor} {#implementations .section-header}

::::::::::::: {#implementations-list}
:::: {#impl-GreeksError .section .impl}
[Source](../../../src/optionstratlib/error/greeks.rs.html#416-470){.src
.rightside}[§](#impl-GreeksError){.anchor}

### impl [GreeksError](enum.GreeksError.html "enum optionstratlib::error::greeks::GreeksError"){.enum} {#impl-greekserror .code-header}

::: docblock
Implementation of factory methods for creating specific `GreeksError`
instances.
:::
::::

::: docblock
This implementation provides convenient constructor methods for creating
different types of errors that can occur during options Greek
calculations. These methods make error creation more concise and
readable in the codebase, while ensuring consistent error formatting.
:::

::::::::: impl-items
::: {#method.invalid_volatility .section .method}
[Source](../../../src/optionstratlib/error/greeks.rs.html#429-434){.src
.rightside}

#### pub fn [invalid_volatility](#method.invalid_volatility){.fn}(value: [f64](https://doc.rust-lang.org/1.86.0/std/primitive.f64.html){.primitive}, reason: &[str](https://doc.rust-lang.org/1.86.0/std/primitive.str.html){.primitive}) -\> Self {#pub-fn-invalid_volatilityvalue-f64-reason-str---self .code-header}
:::

::: docblock
Creates an error for invalid volatility values.

Use this method when a volatility input is outside acceptable bounds or
otherwise unsuitable for calculations. Common cases include negative
values or unreasonably large volatilities.

##### [§](#parameters){.doc-anchor}Parameters

- `value` - The invalid volatility value that triggered the error
- `reason` - An explanation of why the volatility value is invalid

##### [§](#returns){.doc-anchor}Returns

A `GreeksError::InputError` with `InvalidVolatility` kind
:::

::: {#method.invalid_time .section .method}
[Source](../../../src/optionstratlib/error/greeks.rs.html#447-452){.src
.rightside}

#### pub fn [invalid_time](#method.invalid_time){.fn}(value: [Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, reason: &[str](https://doc.rust-lang.org/1.86.0/std/primitive.str.html){.primitive}) -\> Self {#pub-fn-invalid_timevalue-positive-reason-str---self .code-header}
:::

::: docblock
Creates an error for invalid time values.

Use this method when a time input (typically representing time to
expiration) is outside acceptable bounds or otherwise invalid for
calculations.

##### [§](#parameters-1){.doc-anchor}Parameters

- `value` - The invalid time value (as a `Positive` type) that triggered
  the error
- `reason` - An explanation of why the time value is invalid

##### [§](#returns-1){.doc-anchor}Returns

A `GreeksError::InputError` with `InvalidTime` kind
:::

::: {#method.delta_error .section .method}
[Source](../../../src/optionstratlib/error/greeks.rs.html#465-469){.src
.rightside}

#### pub fn [delta_error](#method.delta_error){.fn}(reason: &[str](https://doc.rust-lang.org/1.86.0/std/primitive.str.html){.primitive}) -\> Self {#pub-fn-delta_errorreason-str---self .code-header}
:::

::: docblock
Creates an error for delta calculation failures.

Use this method when a calculation of the delta Greek value fails for
any reason. Delta measures the rate of change of option price with
respect to changes in the underlying asset price.

##### [§](#parameters-2){.doc-anchor}Parameters

- `reason` - A detailed explanation of what caused the delta calculation
  to fail

##### [§](#returns-2){.doc-anchor}Returns

A `GreeksError::CalculationError` with `DeltaError` kind
:::
:::::::::
:::::::::::::

## Trait Implementations[§](#trait-implementations){.anchor} {#trait-implementations .section-header}

::::::::::::::::::::::::::::::::::::::::::::::::::::::::: {#trait-implementations-list}
::: {#impl-Debug-for-GreeksError .section .impl}
[Source](../../../src/optionstratlib/error/greeks.rs.html#51){.src
.rightside}[§](#impl-Debug-for-GreeksError){.anchor}

### impl [Debug](https://doc.rust-lang.org/1.86.0/core/fmt/trait.Debug.html "trait core::fmt::Debug"){.trait} for [GreeksError](enum.GreeksError.html "enum optionstratlib::error::greeks::GreeksError"){.enum} {#impl-debug-for-greekserror .code-header}
:::

::::: impl-items
::: {#method.fmt .section .method .trait-impl}
[Source](../../../src/optionstratlib/error/greeks.rs.html#51){.src
.rightside}[§](#method.fmt){.anchor}

#### fn [fmt](https://doc.rust-lang.org/1.86.0/core/fmt/trait.Debug.html#tymethod.fmt){.fn}(&self, f: &mut [Formatter](https://doc.rust-lang.org/1.86.0/core/fmt/struct.Formatter.html "struct core::fmt::Formatter"){.struct}\<\'\_\>) -\> [Result](https://doc.rust-lang.org/1.86.0/core/fmt/type.Result.html "type core::fmt::Result"){.type} {#fn-fmtself-f-mut-formatter_---result .code-header}
:::

::: docblock
Formats the value using the given formatter. [Read
more](https://doc.rust-lang.org/1.86.0/core/fmt/trait.Debug.html#tymethod.fmt)
:::
:::::

::: {#impl-Display-for-GreeksError .section .impl}
[Source](../../../src/optionstratlib/error/greeks.rs.html#310-319){.src
.rightside}[§](#impl-Display-for-GreeksError){.anchor}

### impl [Display](https://doc.rust-lang.org/1.86.0/core/fmt/trait.Display.html "trait core::fmt::Display"){.trait} for [GreeksError](enum.GreeksError.html "enum optionstratlib::error::greeks::GreeksError"){.enum} {#impl-display-for-greekserror .code-header}
:::

::::: impl-items
::: {#method.fmt-1 .section .method .trait-impl}
[Source](../../../src/optionstratlib/error/greeks.rs.html#311-318){.src
.rightside}[§](#method.fmt-1){.anchor}

#### fn [fmt](https://doc.rust-lang.org/1.86.0/core/fmt/trait.Display.html#tymethod.fmt){.fn}(&self, f: &mut [Formatter](https://doc.rust-lang.org/1.86.0/core/fmt/struct.Formatter.html "struct core::fmt::Formatter"){.struct}\<\'\_\>) -\> [Result](https://doc.rust-lang.org/1.86.0/core/fmt/type.Result.html "type core::fmt::Result"){.type} {#fn-fmtself-f-mut-formatter_---result-1 .code-header}
:::

::: docblock
Formats the value using the given formatter. [Read
more](https://doc.rust-lang.org/1.86.0/core/fmt/trait.Display.html#tymethod.fmt)
:::
:::::

::: {#impl-Error-for-GreeksError .section .impl}
[Source](../../../src/optionstratlib/error/greeks.rs.html#388){.src
.rightside}[§](#impl-Error-for-GreeksError){.anchor}

### impl [Error](https://doc.rust-lang.org/1.86.0/core/error/trait.Error.html "trait core::error::Error"){.trait} for [GreeksError](enum.GreeksError.html "enum optionstratlib::error::greeks::GreeksError"){.enum} {#impl-error-for-greekserror .code-header}
:::

::::::::::::: impl-items
::: {#method.source .section .method .trait-impl}
[[1.30.0]{.since title="Stable since Rust version 1.30.0"} ·
[Source](https://doc.rust-lang.org/1.86.0/src/core/error.rs.html#81){.src}]{.rightside}[§](#method.source){.anchor}

#### fn [source](https://doc.rust-lang.org/1.86.0/core/error/trait.Error.html#method.source){.fn}(&self) -\> [Option](https://doc.rust-lang.org/1.86.0/core/option/enum.Option.html "enum core::option::Option"){.enum}\<&(dyn [Error](https://doc.rust-lang.org/1.86.0/core/error/trait.Error.html "trait core::error::Error"){.trait} + \'static)\> {#fn-sourceself---optiondyn-error-static .code-header}
:::

::: docblock
Returns the lower-level source of this error, if any. [Read
more](https://doc.rust-lang.org/1.86.0/core/error/trait.Error.html#method.source)
:::

::: {#method.description .section .method .trait-impl}
[[1.0.0]{.since title="Stable since Rust version 1.0.0"} ·
[Source](https://doc.rust-lang.org/1.86.0/src/core/error.rs.html#107){.src}]{.rightside}[§](#method.description){.anchor}

#### fn [description](https://doc.rust-lang.org/1.86.0/core/error/trait.Error.html#method.description){.fn}(&self) -\> &[str](https://doc.rust-lang.org/1.86.0/std/primitive.str.html){.primitive} {#fn-descriptionself---str .code-header}
:::

[]{.item-info}

::: {.stab .deprecated}
👎Deprecated since 1.42.0: use the Display impl or to_string()
:::

::: docblock
[Read
more](https://doc.rust-lang.org/1.86.0/core/error/trait.Error.html#method.description)
:::

::: {#method.cause .section .method .trait-impl}
[[1.0.0]{.since title="Stable since Rust version 1.0.0"} ·
[Source](https://doc.rust-lang.org/1.86.0/src/core/error.rs.html#117){.src}]{.rightside}[§](#method.cause){.anchor}

#### fn [cause](https://doc.rust-lang.org/1.86.0/core/error/trait.Error.html#method.cause){.fn}(&self) -\> [Option](https://doc.rust-lang.org/1.86.0/core/option/enum.Option.html "enum core::option::Option"){.enum}\<&dyn [Error](https://doc.rust-lang.org/1.86.0/core/error/trait.Error.html "trait core::error::Error"){.trait}\> {#fn-causeself---optiondyn-error .code-header}
:::

[]{.item-info}

::: {.stab .deprecated}
👎Deprecated since 1.33.0: replaced by Error::source, which can support
downcasting
:::

::: {#method.provide .section .method .trait-impl}
[Source](https://doc.rust-lang.org/1.86.0/src/core/error.rs.html#180){.src
.rightside}[§](#method.provide){.anchor}

#### fn [provide](https://doc.rust-lang.org/1.86.0/core/error/trait.Error.html#method.provide){.fn}\<\'a\>(&\'a self, request: &mut [Request](https://doc.rust-lang.org/1.86.0/core/error/struct.Request.html "struct core::error::Request"){.struct}\<\'a\>) {#fn-provideaa-self-request-mut-requesta .code-header}
:::

[]{.item-info}

::: {.stab .unstable}
🔬This is a nightly-only experimental API.
(`error_generic_member_access`)
:::

::: docblock
Provides type-based access to context intended for error reports. [Read
more](https://doc.rust-lang.org/1.86.0/core/error/trait.Error.html#method.provide)
:::
:::::::::::::

:::: {#impl-From%3CBox%3Cdyn+Error%3E%3E-for-GreeksError .section .impl}
[Source](../../../src/optionstratlib/error/greeks.rs.html#505-509){.src
.rightside}[§](#impl-From%3CBox%3Cdyn+Error%3E%3E-for-GreeksError){.anchor}

### impl [From](https://doc.rust-lang.org/1.86.0/core/convert/trait.From.html "trait core::convert::From"){.trait}\<[Box](https://doc.rust-lang.org/1.86.0/alloc/boxed/struct.Box.html "struct alloc::boxed::Box"){.struct}\<dyn [Error](https://doc.rust-lang.org/1.86.0/core/error/trait.Error.html "trait core::error::Error"){.trait}\>\> for [GreeksError](enum.GreeksError.html "enum optionstratlib::error::greeks::GreeksError"){.enum} {#impl-fromboxdyn-error-for-greekserror .code-header}

::: docblock
Implements conversion from `Box<dyn Error>` to `GreeksError`.
:::
::::

::: docblock
This implementation serves as a catch-all for converting any type that
implements the standard Error trait into a `GreeksError`. This is useful
for integrating with libraries or functions that return standard error
types.
:::

::::: impl-items
::: {#method.from-4 .section .method .trait-impl}
[Source](../../../src/optionstratlib/error/greeks.rs.html#506-508){.src
.rightside}[§](#method.from-4){.anchor}

#### fn [from](https://doc.rust-lang.org/1.86.0/core/convert/trait.From.html#tymethod.from){.fn}(error: [Box](https://doc.rust-lang.org/1.86.0/alloc/boxed/struct.Box.html "struct alloc::boxed::Box"){.struct}\<dyn [Error](https://doc.rust-lang.org/1.86.0/core/error/trait.Error.html "trait core::error::Error"){.trait}\>) -\> Self {#fn-fromerror-boxdyn-error---self .code-header}
:::

::: docblock
Converts to this type from the input type.
:::
:::::

:::: {#impl-From%3CDecimalError%3E-for-GreeksError .section .impl}
[Source](../../../src/optionstratlib/error/greeks.rs.html#478-482){.src
.rightside}[§](#impl-From%3CDecimalError%3E-for-GreeksError){.anchor}

### impl [From](https://doc.rust-lang.org/1.86.0/core/convert/trait.From.html "trait core::convert::From"){.trait}\<[DecimalError](../decimal/enum.DecimalError.html "enum optionstratlib::error::decimal::DecimalError"){.enum}\> for [GreeksError](enum.GreeksError.html "enum optionstratlib::error::greeks::GreeksError"){.enum} {#impl-fromdecimalerror-for-greekserror .code-header}

::: docblock
Implements conversion from `decimal::DecimalError` to `GreeksError`.
:::
::::

::: docblock
This implementation allows decimal calculation errors to be
automatically converted into the appropriate `GreeksError` variant,
simplifying error handling when working with decimal operations in
financial calculations.
:::

::::: impl-items
::: {#method.from-2 .section .method .trait-impl}
[Source](../../../src/optionstratlib/error/greeks.rs.html#479-481){.src
.rightside}[§](#method.from-2){.anchor}

#### fn [from](https://doc.rust-lang.org/1.86.0/core/convert/trait.From.html#tymethod.from){.fn}(error: [DecimalError](../decimal/enum.DecimalError.html "enum optionstratlib::error::decimal::DecimalError"){.enum}) -\> Self {#fn-fromerror-decimalerror---self .code-header}
:::

::: docblock
Converts to this type from the input type.
:::
:::::

::: {#impl-From%3CGreeksError%3E-for-ChainError .section .impl}
[Source](../../../src/optionstratlib/error/chains.rs.html#650-654){.src
.rightside}[§](#impl-From%3CGreeksError%3E-for-ChainError){.anchor}

### impl [From](https://doc.rust-lang.org/1.86.0/core/convert/trait.From.html "trait core::convert::From"){.trait}\<[GreeksError](enum.GreeksError.html "enum optionstratlib::error::greeks::GreeksError"){.enum}\> for [ChainError](../chains/enum.ChainError.html "enum optionstratlib::error::chains::ChainError"){.enum} {#impl-fromgreekserror-for-chainerror .code-header}
:::

::::: impl-items
::: {#method.from .section .method .trait-impl}
[Source](../../../src/optionstratlib/error/chains.rs.html#651-653){.src
.rightside}[§](#method.from){.anchor}

#### fn [from](https://doc.rust-lang.org/1.86.0/core/convert/trait.From.html#tymethod.from){.fn}(err: [GreeksError](enum.GreeksError.html "enum optionstratlib::error::greeks::GreeksError"){.enum}) -\> Self {#fn-fromerr-greekserror---self .code-header}
:::

::: docblock
Converts to this type from the input type.
:::
:::::

::: {#impl-From%3CGreeksError%3E-for-CurveError .section .impl}
[Source](../../../src/optionstratlib/error/curves.rs.html#285-292){.src
.rightside}[§](#impl-From%3CGreeksError%3E-for-CurveError){.anchor}

### impl [From](https://doc.rust-lang.org/1.86.0/core/convert/trait.From.html "trait core::convert::From"){.trait}\<[GreeksError](enum.GreeksError.html "enum optionstratlib::error::greeks::GreeksError"){.enum}\> for [CurveError](../curves/enum.CurveError.html "enum optionstratlib::error::curves::CurveError"){.enum} {#impl-fromgreekserror-for-curveerror .code-header}
:::

::::: impl-items
::: {#method.from-1 .section .method .trait-impl}
[Source](../../../src/optionstratlib/error/curves.rs.html#286-291){.src
.rightside}[§](#method.from-1){.anchor}

#### fn [from](https://doc.rust-lang.org/1.86.0/core/convert/trait.From.html#tymethod.from){.fn}(err: [GreeksError](enum.GreeksError.html "enum optionstratlib::error::greeks::GreeksError"){.enum}) -\> Self {#fn-fromerr-greekserror---self-1 .code-header}
:::

::: docblock
Converts to this type from the input type.
:::
:::::

::: {#impl-From%3CGreeksError%3E-for-SurfaceError .section .impl}
[Source](../../../src/optionstratlib/error/surfaces.rs.html#198-205){.src
.rightside}[§](#impl-From%3CGreeksError%3E-for-SurfaceError){.anchor}

### impl [From](https://doc.rust-lang.org/1.86.0/core/convert/trait.From.html "trait core::convert::From"){.trait}\<[GreeksError](enum.GreeksError.html "enum optionstratlib::error::greeks::GreeksError"){.enum}\> for [SurfaceError](../enum.SurfaceError.html "enum optionstratlib::error::SurfaceError"){.enum} {#impl-fromgreekserror-for-surfaceerror .code-header}
:::

::::: impl-items
::: {#method.from-5 .section .method .trait-impl}
[Source](../../../src/optionstratlib/error/surfaces.rs.html#199-204){.src
.rightside}[§](#method.from-5){.anchor}

#### fn [from](https://doc.rust-lang.org/1.86.0/core/convert/trait.From.html#tymethod.from){.fn}(err: [GreeksError](enum.GreeksError.html "enum optionstratlib::error::greeks::GreeksError"){.enum}) -\> Self {#fn-fromerr-greekserror---self-2 .code-header}
:::

::: docblock
Converts to this type from the input type.
:::
:::::

::: {#impl-From%3CGreeksError%3E-for-VolatilityError .section .impl}
[Source](../../../src/optionstratlib/error/volatility.rs.html#110-116){.src
.rightside}[§](#impl-From%3CGreeksError%3E-for-VolatilityError){.anchor}

### impl [From](https://doc.rust-lang.org/1.86.0/core/convert/trait.From.html "trait core::convert::From"){.trait}\<[GreeksError](enum.GreeksError.html "enum optionstratlib::error::greeks::GreeksError"){.enum}\> for [VolatilityError](../enum.VolatilityError.html "enum optionstratlib::error::VolatilityError"){.enum} {#impl-fromgreekserror-for-volatilityerror .code-header}
:::

::::: impl-items
::: {#method.from-6 .section .method .trait-impl}
[Source](../../../src/optionstratlib/error/volatility.rs.html#111-115){.src
.rightside}[§](#method.from-6){.anchor}

#### fn [from](https://doc.rust-lang.org/1.86.0/core/convert/trait.From.html#tymethod.from){.fn}(error: [GreeksError](enum.GreeksError.html "enum optionstratlib::error::greeks::GreeksError"){.enum}) -\> Self {#fn-fromerror-greekserror---self .code-header}
:::

::: docblock
Converts to this type from the input type.
:::
:::::

:::: {#impl-From%3CVolatilityError%3E-for-GreeksError .section .impl}
[Source](../../../src/optionstratlib/error/greeks.rs.html#490-497){.src
.rightside}[§](#impl-From%3CVolatilityError%3E-for-GreeksError){.anchor}

### impl [From](https://doc.rust-lang.org/1.86.0/core/convert/trait.From.html "trait core::convert::From"){.trait}\<[VolatilityError](../enum.VolatilityError.html "enum optionstratlib::error::VolatilityError"){.enum}\> for [GreeksError](enum.GreeksError.html "enum optionstratlib::error::greeks::GreeksError"){.enum} {#impl-fromvolatilityerror-for-greekserror .code-header}

::: docblock
Implements conversion from `VolatilityError` to `GreeksError`.
:::
::::

::: docblock
This implementation allows volatility-related errors to be automatically
converted into appropriate `InputErrorKind::InvalidVolatility` errors,
providing consistent error handling for invalid volatility values.
:::

::::: impl-items
::: {#method.from-3 .section .method .trait-impl}
[Source](../../../src/optionstratlib/error/greeks.rs.html#491-496){.src
.rightside}[§](#method.from-3){.anchor}

#### fn [from](https://doc.rust-lang.org/1.86.0/core/convert/trait.From.html#tymethod.from){.fn}(error: [VolatilityError](../enum.VolatilityError.html "enum optionstratlib::error::VolatilityError"){.enum}) -\> Self {#fn-fromerror-volatilityerror---self .code-header}
:::

::: docblock
Converts to this type from the input type.
:::
:::::
:::::::::::::::::::::::::::::::::::::::::::::::::::::::::

## Auto Trait Implementations[§](#synthetic-implementations){.anchor} {#synthetic-implementations .section-header}

::::::::: {#synthetic-implementations-list}
::: {#impl-Freeze-for-GreeksError .section .impl}
[§](#impl-Freeze-for-GreeksError){.anchor}

### impl [Freeze](https://doc.rust-lang.org/1.86.0/core/marker/trait.Freeze.html "trait core::marker::Freeze"){.trait} for [GreeksError](enum.GreeksError.html "enum optionstratlib::error::greeks::GreeksError"){.enum} {#impl-freeze-for-greekserror .code-header}
:::

::: {#impl-RefUnwindSafe-for-GreeksError .section .impl}
[§](#impl-RefUnwindSafe-for-GreeksError){.anchor}

### impl [RefUnwindSafe](https://doc.rust-lang.org/1.86.0/core/panic/unwind_safe/trait.RefUnwindSafe.html "trait core::panic::unwind_safe::RefUnwindSafe"){.trait} for [GreeksError](enum.GreeksError.html "enum optionstratlib::error::greeks::GreeksError"){.enum} {#impl-refunwindsafe-for-greekserror .code-header}
:::

::: {#impl-Send-for-GreeksError .section .impl}
[§](#impl-Send-for-GreeksError){.anchor}

### impl [Send](https://doc.rust-lang.org/1.86.0/core/marker/trait.Send.html "trait core::marker::Send"){.trait} for [GreeksError](enum.GreeksError.html "enum optionstratlib::error::greeks::GreeksError"){.enum} {#impl-send-for-greekserror .code-header}
:::

::: {#impl-Sync-for-GreeksError .section .impl}
[§](#impl-Sync-for-GreeksError){.anchor}

### impl [Sync](https://doc.rust-lang.org/1.86.0/core/marker/trait.Sync.html "trait core::marker::Sync"){.trait} for [GreeksError](enum.GreeksError.html "enum optionstratlib::error::greeks::GreeksError"){.enum} {#impl-sync-for-greekserror .code-header}
:::

::: {#impl-Unpin-for-GreeksError .section .impl}
[§](#impl-Unpin-for-GreeksError){.anchor}

### impl [Unpin](https://doc.rust-lang.org/1.86.0/core/marker/trait.Unpin.html "trait core::marker::Unpin"){.trait} for [GreeksError](enum.GreeksError.html "enum optionstratlib::error::greeks::GreeksError"){.enum} {#impl-unpin-for-greekserror .code-header}
:::

::: {#impl-UnwindSafe-for-GreeksError .section .impl}
[§](#impl-UnwindSafe-for-GreeksError){.anchor}

### impl [UnwindSafe](https://doc.rust-lang.org/1.86.0/core/panic/unwind_safe/trait.UnwindSafe.html "trait core::panic::unwind_safe::UnwindSafe"){.trait} for [GreeksError](enum.GreeksError.html "enum optionstratlib::error::greeks::GreeksError"){.enum} {#impl-unwindsafe-for-greekserror .code-header}
:::
:::::::::

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
::: {#method.from-7 .section .method .trait-impl}
[Source](https://doc.rust-lang.org/1.86.0/src/core/convert/mod.rs.html#770){.src
.rightside}[§](#method.from-7){.anchor}

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
:::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
