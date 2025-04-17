::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::: width-limiter
:::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../../index.html)::[error](../index.html)::[decimal](index.html)
:::

# Enum [DecimalError]{.enum}Copy item path

[[Source](../../../src/optionstratlib/error/decimal.rs.html#61-123){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub enum DecimalError {
    InvalidValue {
        value: f64,
        reason: String,
    },
    ArithmeticError {
        operation: String,
        reason: String,
    },
    ConversionError {
        from_type: String,
        to_type: String,
        reason: String,
    },
    OutOfBounds {
        value: f64,
        min: f64,
        max: f64,
    },
    InvalidPrecision {
        precision: i32,
        reason: String,
    },
}
```

Expand description

:::: docblock
## [§](#decimal-error-management){.doc-anchor}Decimal Error Management

Represents errors that can occur during decimal operations in financial
calculations. This enum provides a structured way to handle various
error conditions that may arise when working with decimal values,
including validation, arithmetic operations, conversions, and precision
issues.

## [§](#use-cases){.doc-anchor}Use Cases

- Financial calculations requiring strict decimal precision
- Currency and monetary value operations
- Option pricing models where precision is critical
- Risk management calculations

## [§](#error-propagation){.doc-anchor}Error Propagation

These errors are typically wrapped in `DecimalResult` and propagated
through the application's calculation pipeline.

## [§](#variants-1){.doc-anchor}Variants {#variants-1}

- `InvalidValue` - Handles errors when a value cannot be represented as
  a valid decimal
- `ArithmeticError` - Handles errors during mathematical operations
- `ConversionError` - Handles errors when converting between different
  decimal representations
- `OutOfBounds` - Handles errors when a value exceeds defined limits
- `InvalidPrecision` - Handles errors related to decimal precision
  settings

## [§](#example-usage){.doc-anchor}Example Usage

::: example-wrap
``` {.rust .rust-example-rendered}
use optionstratlib::error::DecimalError;

fn validate_decimal(value: f64) -> Result<(), DecimalError> {
    if value.is_nan() {
        return Err(DecimalError::InvalidValue {
            value,
            reason: "Value cannot be NaN".to_string(),
        });
    }
     
    if value < 0.0 || value > 100.0 {
        return Err(DecimalError::OutOfBounds {
            value,
            min: 0.0,
            max: 100.0,
        });
    }
     
    Ok(())
}
```
:::
::::

## Variants[§](#variants){.anchor} {#variants .variants .section-header}

:::::::::::::::::::::::::::::::::::::::::: variants
::: {#variant.InvalidValue .section .variant}
[§](#variant.InvalidValue){.anchor}

### InvalidValue {#invalidvalue .code-header}
:::

::: docblock
Error when attempting to create a decimal from an invalid value

Occurs when a value cannot be properly represented as a decimal, such as
when it's NaN, infinity, or otherwise unsuitable for financial
calculations.
:::

::::::: {#variant.InvalidValue.fields .sub-variant}
#### Fields

:::: sub-variant-field
[[§](#variant.InvalidValue.field.value){.anchor
.field}`value: `[`f64`](https://doc.rust-lang.org/1.86.0/std/primitive.f64.html){.primitive}]{#variant.InvalidValue.field.value
.section-header}

::: docblock
The problematic value that caused the error
:::
::::

:::: sub-variant-field
[[§](#variant.InvalidValue.field.reason){.anchor
.field}`reason: `[`String`](https://doc.rust-lang.org/1.86.0/alloc/string/struct.String.html "struct alloc::string::String"){.struct}]{#variant.InvalidValue.field.reason
.section-header}

::: docblock
Detailed explanation of why the value is invalid
:::
::::
:::::::

::: {#variant.ArithmeticError .section .variant}
[§](#variant.ArithmeticError){.anchor}

### ArithmeticError {#arithmeticerror .code-header}
:::

::: docblock
Error when performing decimal arithmetic operations

Occurs during mathematical operations such as addition, subtraction,
multiplication, or division when the operation cannot be completed
correctly (e.g., division by zero, overflow).
:::

::::::: {#variant.ArithmeticError.fields .sub-variant}
#### Fields

:::: sub-variant-field
[[§](#variant.ArithmeticError.field.operation){.anchor
.field}`operation: `[`String`](https://doc.rust-lang.org/1.86.0/alloc/string/struct.String.html "struct alloc::string::String"){.struct}]{#variant.ArithmeticError.field.operation
.section-header}

::: docblock
The operation that failed (e.g., "addition", "division")
:::
::::

:::: sub-variant-field
[[§](#variant.ArithmeticError.field.reason){.anchor
.field}`reason: `[`String`](https://doc.rust-lang.org/1.86.0/alloc/string/struct.String.html "struct alloc::string::String"){.struct}]{#variant.ArithmeticError.field.reason
.section-header}

::: docblock
Detailed explanation of why the operation failed
:::
::::
:::::::

::: {#variant.ConversionError .section .variant}
[§](#variant.ConversionError){.anchor}

### ConversionError {#conversionerror .code-header}
:::

::: docblock
Error when converting between decimal types

Occurs when a decimal value cannot be correctly converted from one
representation to another, such as between different precision levels or
between different decimal formats.
:::

::::::::: {#variant.ConversionError.fields .sub-variant}
#### Fields

:::: sub-variant-field
[[§](#variant.ConversionError.field.from_type){.anchor
.field}`from_type: `[`String`](https://doc.rust-lang.org/1.86.0/alloc/string/struct.String.html "struct alloc::string::String"){.struct}]{#variant.ConversionError.field.from_type
.section-header}

::: docblock
The source type being converted from
:::
::::

:::: sub-variant-field
[[§](#variant.ConversionError.field.to_type){.anchor
.field}`to_type: `[`String`](https://doc.rust-lang.org/1.86.0/alloc/string/struct.String.html "struct alloc::string::String"){.struct}]{#variant.ConversionError.field.to_type
.section-header}

::: docblock
The destination type being converted to
:::
::::

:::: sub-variant-field
[[§](#variant.ConversionError.field.reason){.anchor
.field}`reason: `[`String`](https://doc.rust-lang.org/1.86.0/alloc/string/struct.String.html "struct alloc::string::String"){.struct}]{#variant.ConversionError.field.reason
.section-header}

::: docblock
Detailed explanation of why the conversion failed
:::
::::
:::::::::

::: {#variant.OutOfBounds .section .variant}
[§](#variant.OutOfBounds){.anchor}

### OutOfBounds {#outofbounds .code-header}
:::

::: docblock
Error when a decimal value exceeds its bounds

Occurs when a decimal value falls outside of acceptable minimum or
maximum values for a specific calculation context.
:::

::::::::: {#variant.OutOfBounds.fields .sub-variant}
#### Fields

:::: sub-variant-field
[[§](#variant.OutOfBounds.field.value){.anchor
.field}`value: `[`f64`](https://doc.rust-lang.org/1.86.0/std/primitive.f64.html){.primitive}]{#variant.OutOfBounds.field.value
.section-header}

::: docblock
The value that is out of bounds
:::
::::

:::: sub-variant-field
[[§](#variant.OutOfBounds.field.min){.anchor
.field}`min: `[`f64`](https://doc.rust-lang.org/1.86.0/std/primitive.f64.html){.primitive}]{#variant.OutOfBounds.field.min
.section-header}

::: docblock
The minimum acceptable value
:::
::::

:::: sub-variant-field
[[§](#variant.OutOfBounds.field.max){.anchor
.field}`max: `[`f64`](https://doc.rust-lang.org/1.86.0/std/primitive.f64.html){.primitive}]{#variant.OutOfBounds.field.max
.section-header}

::: docblock
The maximum acceptable value
:::
::::
:::::::::

::: {#variant.InvalidPrecision .section .variant}
[§](#variant.InvalidPrecision){.anchor}

### InvalidPrecision {#invalidprecision .code-header}
:::

::: docblock
Error when decimal precision is invalid

Occurs when an operation specifies or results in an invalid precision
level that cannot be properly handled.
:::

::::::: {#variant.InvalidPrecision.fields .sub-variant}
#### Fields

:::: sub-variant-field
[[§](#variant.InvalidPrecision.field.precision){.anchor
.field}`precision: `[`i32`](https://doc.rust-lang.org/1.86.0/std/primitive.i32.html){.primitive}]{#variant.InvalidPrecision.field.precision
.section-header}

::: docblock
The problematic precision value
:::
::::

:::: sub-variant-field
[[§](#variant.InvalidPrecision.field.reason){.anchor
.field}`reason: `[`String`](https://doc.rust-lang.org/1.86.0/alloc/string/struct.String.html "struct alloc::string::String"){.struct}]{#variant.InvalidPrecision.field.reason
.section-header}

::: docblock
Detailed explanation of why the precision is invalid
:::
::::
:::::::
::::::::::::::::::::::::::::::::::::::::::

## Implementations[§](#implementations){.anchor} {#implementations .section-header}

:::::::::::::::::: {#implementations-list}
:::: {#impl-DecimalError .section .impl}
[Source](../../../src/optionstratlib/error/decimal.rs.html#221-320){.src
.rightside}[§](#impl-DecimalError){.anchor}

### impl [DecimalError](enum.DecimalError.html "enum optionstratlib::error::decimal::DecimalError"){.enum} {#impl-decimalerror .code-header}

::: docblock
Helper methods for creating common decimal errors
:::
::::

:::: docblock
This implementation provides convenient factory methods to create
standardized instances of `DecimalError` for common error scenarios in
decimal operations. These methods help maintain consistency in error
creation across the codebase and simplify the construction of
descriptive error instances.

#### [§](#example){.doc-anchor}Example

::: example-wrap
``` {.rust .rust-example-rendered}
use optionstratlib::error::DecimalError;
// Creating an invalid value error
let err = DecimalError::invalid_value(12.34, "Value exceeds maximum allowed");

// Creating an arithmetic error
let div_err = DecimalError::arithmetic_error("division", "Division by zero");
```
:::
::::

::::::::::::: impl-items
::: {#method.invalid_value .section .method}
[Source](../../../src/optionstratlib/error/decimal.rs.html#235-240){.src
.rightside}

#### pub fn [invalid_value](#method.invalid_value){.fn}(value: [f64](https://doc.rust-lang.org/1.86.0/std/primitive.f64.html){.primitive}, reason: &[str](https://doc.rust-lang.org/1.86.0/std/primitive.str.html){.primitive}) -\> Self {#pub-fn-invalid_valuevalue-f64-reason-str---self .code-header}
:::

::: docblock
Creates a new `InvalidValue` error

Used when a decimal value fails validation due to being outside accepted
ranges or otherwise inappropriate for the context.

##### [§](#parameters){.doc-anchor}Parameters

- `value` - The problematic floating-point value
- `reason` - Explanation of why the value is invalid

##### [§](#returns){.doc-anchor}Returns

A new `DecimalError::InvalidValue` instance
:::

::: {#method.arithmetic_error .section .method}
[Source](../../../src/optionstratlib/error/decimal.rs.html#255-260){.src
.rightside}

#### pub fn [arithmetic_error](#method.arithmetic_error){.fn}(operation: &[str](https://doc.rust-lang.org/1.86.0/std/primitive.str.html){.primitive}, reason: &[str](https://doc.rust-lang.org/1.86.0/std/primitive.str.html){.primitive}) -\> Self {#pub-fn-arithmetic_erroroperation-str-reason-str---self .code-header}
:::

::: docblock
Creates a new `ArithmeticError` error

Used when a mathematical operation on decimal values fails, such as
division by zero, overflow, or underflow.

##### [§](#parameters-1){.doc-anchor}Parameters

- `operation` - The name of the operation that failed (e.g., "addition",
  "division")
- `reason` - Explanation of why the operation failed

##### [§](#returns-1){.doc-anchor}Returns

A new `DecimalError::ArithmeticError` instance
:::

::: {#method.conversion_error .section .method}
[Source](../../../src/optionstratlib/error/decimal.rs.html#276-282){.src
.rightside}

#### pub fn [conversion_error](#method.conversion_error){.fn}(from_type: &[str](https://doc.rust-lang.org/1.86.0/std/primitive.str.html){.primitive}, to_type: &[str](https://doc.rust-lang.org/1.86.0/std/primitive.str.html){.primitive}, reason: &[str](https://doc.rust-lang.org/1.86.0/std/primitive.str.html){.primitive}) -\> Self {#pub-fn-conversion_errorfrom_type-str-to_type-str-reason-str---self .code-header}
:::

::: docblock
Creates a new `ConversionError` error

Used when conversion between decimal types or from/to other number types
fails due to compatibility or range issues.

##### [§](#parameters-2){.doc-anchor}Parameters

- `from_type` - The source type being converted from
- `to_type` - The destination type being converted to
- `reason` - Explanation of why the conversion failed

##### [§](#returns-2){.doc-anchor}Returns

A new `DecimalError::ConversionError` instance
:::

::: {#method.out_of_bounds .section .method}
[Source](../../../src/optionstratlib/error/decimal.rs.html#297-299){.src
.rightside}

#### pub fn [out_of_bounds](#method.out_of_bounds){.fn}(value: [f64](https://doc.rust-lang.org/1.86.0/std/primitive.f64.html){.primitive}, min: [f64](https://doc.rust-lang.org/1.86.0/std/primitive.f64.html){.primitive}, max: [f64](https://doc.rust-lang.org/1.86.0/std/primitive.f64.html){.primitive}) -\> Self {#pub-fn-out_of_boundsvalue-f64-min-f64-max-f64---self .code-header}
:::

::: docblock
Creates a new `OutOfBounds` error

Used when a decimal value falls outside of specified minimum and maximum
bounds.

##### [§](#parameters-3){.doc-anchor}Parameters

- `value` - The out-of-bounds floating-point value
- `min` - The lower bound (inclusive) of the valid range
- `max` - The upper bound (inclusive) of the valid range

##### [§](#returns-3){.doc-anchor}Returns

A new `DecimalError::OutOfBounds` instance
:::

::: {#method.invalid_precision .section .method}
[Source](../../../src/optionstratlib/error/decimal.rs.html#314-319){.src
.rightside}

#### pub fn [invalid_precision](#method.invalid_precision){.fn}(precision: [i32](https://doc.rust-lang.org/1.86.0/std/primitive.i32.html){.primitive}, reason: &[str](https://doc.rust-lang.org/1.86.0/std/primitive.str.html){.primitive}) -\> Self {#pub-fn-invalid_precisionprecision-i32-reason-str---self .code-header}
:::

::: docblock
Creates a new `InvalidPrecision` error

Used when a specified decimal precision is invalid, such as being
negative, too large, or otherwise inappropriate for the context.

##### [§](#parameters-4){.doc-anchor}Parameters

- `precision` - The problematic precision value
- `reason` - Explanation of why the precision is invalid

##### [§](#returns-4){.doc-anchor}Returns

A new `DecimalError::InvalidPrecision` instance
:::
:::::::::::::
::::::::::::::::::

## Trait Implementations[§](#trait-implementations){.anchor} {#trait-implementations .section-header}

::::::::::::::::::::::::::::: {#trait-implementations-list}
::: {#impl-Debug-for-DecimalError .section .impl}
[Source](../../../src/optionstratlib/error/decimal.rs.html#60){.src
.rightside}[§](#impl-Debug-for-DecimalError){.anchor}

### impl [Debug](https://doc.rust-lang.org/1.86.0/core/fmt/trait.Debug.html "trait core::fmt::Debug"){.trait} for [DecimalError](enum.DecimalError.html "enum optionstratlib::error::decimal::DecimalError"){.enum} {#impl-debug-for-decimalerror .code-header}
:::

::::: impl-items
::: {#method.fmt .section .method .trait-impl}
[Source](../../../src/optionstratlib/error/decimal.rs.html#60){.src
.rightside}[§](#method.fmt){.anchor}

#### fn [fmt](https://doc.rust-lang.org/1.86.0/core/fmt/trait.Debug.html#tymethod.fmt){.fn}(&self, f: &mut [Formatter](https://doc.rust-lang.org/1.86.0/core/fmt/struct.Formatter.html "struct core::fmt::Formatter"){.struct}\<\'\_\>) -\> [Result](https://doc.rust-lang.org/1.86.0/core/fmt/type.Result.html "type core::fmt::Result"){.type} {#fn-fmtself-f-mut-formatter_---result .code-header}
:::

::: docblock
Formats the value using the given formatter. [Read
more](https://doc.rust-lang.org/1.86.0/core/fmt/trait.Debug.html#tymethod.fmt)
:::
:::::

::: {#impl-Display-for-DecimalError .section .impl}
[Source](../../../src/optionstratlib/error/decimal.rs.html#125-161){.src
.rightside}[§](#impl-Display-for-DecimalError){.anchor}

### impl [Display](https://doc.rust-lang.org/1.86.0/core/fmt/trait.Display.html "trait core::fmt::Display"){.trait} for [DecimalError](enum.DecimalError.html "enum optionstratlib::error::decimal::DecimalError"){.enum} {#impl-display-for-decimalerror .code-header}
:::

::::: impl-items
::: {#method.fmt-1 .section .method .trait-impl}
[Source](../../../src/optionstratlib/error/decimal.rs.html#126-160){.src
.rightside}[§](#method.fmt-1){.anchor}

#### fn [fmt](https://doc.rust-lang.org/1.86.0/core/fmt/trait.Display.html#tymethod.fmt){.fn}(&self, f: &mut [Formatter](https://doc.rust-lang.org/1.86.0/core/fmt/struct.Formatter.html "struct core::fmt::Formatter"){.struct}\<\'\_\>) -\> [Result](https://doc.rust-lang.org/1.86.0/core/fmt/type.Result.html "type core::fmt::Result"){.type} {#fn-fmtself-f-mut-formatter_---result-1 .code-header}
:::

::: docblock
Formats the value using the given formatter. [Read
more](https://doc.rust-lang.org/1.86.0/core/fmt/trait.Display.html#tymethod.fmt)
:::
:::::

::: {#impl-Error-for-DecimalError .section .impl}
[Source](../../../src/optionstratlib/error/decimal.rs.html#163){.src
.rightside}[§](#impl-Error-for-DecimalError){.anchor}

### impl [Error](https://doc.rust-lang.org/1.86.0/core/error/trait.Error.html "trait core::error::Error"){.trait} for [DecimalError](enum.DecimalError.html "enum optionstratlib::error::decimal::DecimalError"){.enum} {#impl-error-for-decimalerror .code-header}
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

:::: {#impl-From%3CDecimalError%3E-for-GreeksError .section .impl}
[Source](../../../src/optionstratlib/error/greeks.rs.html#478-482){.src
.rightside}[§](#impl-From%3CDecimalError%3E-for-GreeksError){.anchor}

### impl [From](https://doc.rust-lang.org/1.86.0/core/convert/trait.From.html "trait core::convert::From"){.trait}\<[DecimalError](enum.DecimalError.html "enum optionstratlib::error::decimal::DecimalError"){.enum}\> for [GreeksError](../greeks/enum.GreeksError.html "enum optionstratlib::error::greeks::GreeksError"){.enum} {#impl-fromdecimalerror-for-greekserror .code-header}

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
::: {#method.from .section .method .trait-impl}
[Source](../../../src/optionstratlib/error/greeks.rs.html#479-481){.src
.rightside}[§](#method.from){.anchor}

#### fn [from](https://doc.rust-lang.org/1.86.0/core/convert/trait.From.html#tymethod.from){.fn}(error: [DecimalError](enum.DecimalError.html "enum optionstratlib::error::decimal::DecimalError"){.enum}) -\> Self {#fn-fromerror-decimalerror---self .code-header}
:::

::: docblock
Converts to this type from the input type.
:::
:::::
:::::::::::::::::::::::::::::

## Auto Trait Implementations[§](#synthetic-implementations){.anchor} {#synthetic-implementations .section-header}

::::::::: {#synthetic-implementations-list}
::: {#impl-Freeze-for-DecimalError .section .impl}
[§](#impl-Freeze-for-DecimalError){.anchor}

### impl [Freeze](https://doc.rust-lang.org/1.86.0/core/marker/trait.Freeze.html "trait core::marker::Freeze"){.trait} for [DecimalError](enum.DecimalError.html "enum optionstratlib::error::decimal::DecimalError"){.enum} {#impl-freeze-for-decimalerror .code-header}
:::

::: {#impl-RefUnwindSafe-for-DecimalError .section .impl}
[§](#impl-RefUnwindSafe-for-DecimalError){.anchor}

### impl [RefUnwindSafe](https://doc.rust-lang.org/1.86.0/core/panic/unwind_safe/trait.RefUnwindSafe.html "trait core::panic::unwind_safe::RefUnwindSafe"){.trait} for [DecimalError](enum.DecimalError.html "enum optionstratlib::error::decimal::DecimalError"){.enum} {#impl-refunwindsafe-for-decimalerror .code-header}
:::

::: {#impl-Send-for-DecimalError .section .impl}
[§](#impl-Send-for-DecimalError){.anchor}

### impl [Send](https://doc.rust-lang.org/1.86.0/core/marker/trait.Send.html "trait core::marker::Send"){.trait} for [DecimalError](enum.DecimalError.html "enum optionstratlib::error::decimal::DecimalError"){.enum} {#impl-send-for-decimalerror .code-header}
:::

::: {#impl-Sync-for-DecimalError .section .impl}
[§](#impl-Sync-for-DecimalError){.anchor}

### impl [Sync](https://doc.rust-lang.org/1.86.0/core/marker/trait.Sync.html "trait core::marker::Sync"){.trait} for [DecimalError](enum.DecimalError.html "enum optionstratlib::error::decimal::DecimalError"){.enum} {#impl-sync-for-decimalerror .code-header}
:::

::: {#impl-Unpin-for-DecimalError .section .impl}
[§](#impl-Unpin-for-DecimalError){.anchor}

### impl [Unpin](https://doc.rust-lang.org/1.86.0/core/marker/trait.Unpin.html "trait core::marker::Unpin"){.trait} for [DecimalError](enum.DecimalError.html "enum optionstratlib::error::decimal::DecimalError"){.enum} {#impl-unpin-for-decimalerror .code-header}
:::

::: {#impl-UnwindSafe-for-DecimalError .section .impl}
[§](#impl-UnwindSafe-for-DecimalError){.anchor}

### impl [UnwindSafe](https://doc.rust-lang.org/1.86.0/core/panic/unwind_safe/trait.UnwindSafe.html "trait core::panic::unwind_safe::UnwindSafe"){.trait} for [DecimalError](enum.DecimalError.html "enum optionstratlib::error::decimal::DecimalError"){.enum} {#impl-unwindsafe-for-decimalerror .code-header}
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
::: {#method.from-1 .section .method .trait-impl}
[Source](https://doc.rust-lang.org/1.86.0/src/core/convert/mod.rs.html#770){.src
.rightside}[§](#method.from-1){.anchor}

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
::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
:::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
