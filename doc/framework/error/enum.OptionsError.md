:::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::: width-limiter
::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../index.html)::[error](index.html)
:::

# Enum [OptionsError]{.enum} Copy item path

[[Source](../../src/optionstratlib/error/options.rs.html#100-181){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub enum OptionsError {
    ValidationError {
        field: String,
        reason: String,
    },
    PricingError {
        method: String,
        reason: String,
    },
    GreeksCalculationError {
        greek: String,
        reason: String,
    },
    TimeError {
        operation: String,
        reason: String,
    },
    PayoffError {
        reason: String,
    },
    UpdateError {
        field: String,
        reason: String,
    },
    OtherError {
        reason: String,
    },
    Decimal(DecimalError),
    Greeks(GreeksError),
}
```

Expand description

::: docblock
Custom errors that can occur during Options operations

This enum provides a structured error system for handling various
failure scenarios that may arise during option trading operations,
calculations, and data management. Each variant represents a specific
category of error with contextual information to help with debugging and
error handling.

## [§](#variants-1){.doc-anchor}Variants {#variants-1}

- `ValidationError` - Errors that occur when validating option
  parameters such as strike prices, expiration dates, or option styles.

- `PricingError` - Errors that occur during option price calculation
  using various pricing models like Black-Scholes, Binomial, etc.

- `GreeksCalculationError` - Errors that occur when calculating option
  Greeks (delta, gamma, theta, vega, rho) which measure option price
  sensitivities.

- `TimeError` - Errors related to time calculations, such as determining
  days to expiration, time decay, or handling calendar adjustments.

- `PayoffError` - Errors that occur when calculating potential payoffs
  for options at different price points or expiration scenarios.

- `UpdateError` - Errors that occur when attempting to update option
  data or parameters in an existing option object.

- `OtherError` - A catch-all for errors that don't fit into other
  categories but still need to be represented in the options domain.

## [§](#usage){.doc-anchor}Usage

This error type is typically returned in Result objects from functions
that perform operations on option contracts, pricing calculations, or
option strategy analysis where various error conditions need to be
handled.
:::

## Variants[§](#variants){.anchor} {#variants .variants .section-header}

:::::::::::::::::::::::::::::::::::::::::::::::::::: variants
::: {#variant.ValidationError .section .variant}
[§](#variant.ValidationError){.anchor}

### ValidationError {#validationerror .code-header}
:::

::: docblock
Error when validating option parameters

Used when input parameters for option contracts fail validation.
:::

::::::: {#variant.ValidationError.fields .sub-variant}
#### Fields

:::: sub-variant-field
[[§](#variant.ValidationError.field.field){.anchor
.field}`field: `[`String`](https://doc.rust-lang.org/1.91.1/alloc/string/struct.String.html "struct alloc::string::String"){.struct}]{#variant.ValidationError.field.field
.section-header}

::: docblock
The field name that failed validation
:::
::::

:::: sub-variant-field
[[§](#variant.ValidationError.field.reason){.anchor
.field}`reason: `[`String`](https://doc.rust-lang.org/1.91.1/alloc/string/struct.String.html "struct alloc::string::String"){.struct}]{#variant.ValidationError.field.reason
.section-header}

::: docblock
Detailed explanation of the validation failure
:::
::::
:::::::

::: {#variant.PricingError .section .variant}
[§](#variant.PricingError){.anchor}

### PricingError {#pricingerror .code-header}
:::

::: docblock
Error during price calculation

Used when an option pricing algorithm encounters problems.
:::

::::::: {#variant.PricingError.fields .sub-variant}
#### Fields

:::: sub-variant-field
[[§](#variant.PricingError.field.method){.anchor
.field}`method: `[`String`](https://doc.rust-lang.org/1.91.1/alloc/string/struct.String.html "struct alloc::string::String"){.struct}]{#variant.PricingError.field.method
.section-header}

::: docblock
The pricing method that failed (e.g., "Black-Scholes", "Binomial")
:::
::::

:::: sub-variant-field
[[§](#variant.PricingError.field.reason){.anchor
.field}`reason: `[`String`](https://doc.rust-lang.org/1.91.1/alloc/string/struct.String.html "struct alloc::string::String"){.struct}]{#variant.PricingError.field.reason
.section-header}

::: docblock
Detailed explanation of the pricing calculation failure
:::
::::
:::::::

::: {#variant.GreeksCalculationError .section .variant}
[§](#variant.GreeksCalculationError){.anchor}

### GreeksCalculationError {#greekscalculationerror .code-header}
:::

::: docblock
Error when calculating greeks

Used when calculations for option sensitivities (Greeks) fail.
:::

::::::: {#variant.GreeksCalculationError.fields .sub-variant}
#### Fields

:::: sub-variant-field
[[§](#variant.GreeksCalculationError.field.greek){.anchor
.field}`greek: `[`String`](https://doc.rust-lang.org/1.91.1/alloc/string/struct.String.html "struct alloc::string::String"){.struct}]{#variant.GreeksCalculationError.field.greek
.section-header}

::: docblock
The specific Greek that failed to calculate (delta, gamma, theta, etc.)
:::
::::

:::: sub-variant-field
[[§](#variant.GreeksCalculationError.field.reason){.anchor
.field}`reason: `[`String`](https://doc.rust-lang.org/1.91.1/alloc/string/struct.String.html "struct alloc::string::String"){.struct}]{#variant.GreeksCalculationError.field.reason
.section-header}

::: docblock
Detailed explanation of the Greek calculation failure
:::
::::
:::::::

::: {#variant.TimeError .section .variant}
[§](#variant.TimeError){.anchor}

### TimeError {#timeerror .code-header}
:::

::: docblock
Error when dealing with time calculations

Used for failures in time-related calculations like time to expiry.
:::

::::::: {#variant.TimeError.fields .sub-variant}
#### Fields

:::: sub-variant-field
[[§](#variant.TimeError.field.operation){.anchor
.field}`operation: `[`String`](https://doc.rust-lang.org/1.91.1/alloc/string/struct.String.html "struct alloc::string::String"){.struct}]{#variant.TimeError.field.operation
.section-header}

::: docblock
The time-related operation that failed
:::
::::

:::: sub-variant-field
[[§](#variant.TimeError.field.reason){.anchor
.field}`reason: `[`String`](https://doc.rust-lang.org/1.91.1/alloc/string/struct.String.html "struct alloc::string::String"){.struct}]{#variant.TimeError.field.reason
.section-header}

::: docblock
Detailed explanation of the time calculation failure
:::
::::
:::::::

::: {#variant.PayoffError .section .variant}
[§](#variant.PayoffError){.anchor}

### PayoffError {#payofferror .code-header}
:::

::: docblock
Error when performing payoff calculations

Used when potential profit/loss calculations for options fail.
:::

::::: {#variant.PayoffError.fields .sub-variant}
#### Fields

:::: sub-variant-field
[[§](#variant.PayoffError.field.reason){.anchor
.field}`reason: `[`String`](https://doc.rust-lang.org/1.91.1/alloc/string/struct.String.html "struct alloc::string::String"){.struct}]{#variant.PayoffError.field.reason
.section-header}

::: docblock
Detailed explanation of the payoff calculation failure
:::
::::
:::::

::: {#variant.UpdateError .section .variant}
[§](#variant.UpdateError){.anchor}

### UpdateError {#updateerror .code-header}
:::

::: docblock
Error during option data updates

Used when attempts to update option parameters or data fail.
:::

::::::: {#variant.UpdateError.fields .sub-variant}
#### Fields

:::: sub-variant-field
[[§](#variant.UpdateError.field.field){.anchor
.field}`field: `[`String`](https://doc.rust-lang.org/1.91.1/alloc/string/struct.String.html "struct alloc::string::String"){.struct}]{#variant.UpdateError.field.field
.section-header}

::: docblock
The field that failed to update
:::
::::

:::: sub-variant-field
[[§](#variant.UpdateError.field.reason){.anchor
.field}`reason: `[`String`](https://doc.rust-lang.org/1.91.1/alloc/string/struct.String.html "struct alloc::string::String"){.struct}]{#variant.UpdateError.field.reason
.section-header}

::: docblock
Detailed explanation of the update failure
:::
::::
:::::::

::: {#variant.OtherError .section .variant}
[§](#variant.OtherError){.anchor}

### OtherError {#othererror .code-header}
:::

::: docblock
Error when performing other operations

A general-purpose error for cases not covered by other variants.
:::

::::: {#variant.OtherError.fields .sub-variant}
#### Fields

:::: sub-variant-field
[[§](#variant.OtherError.field.reason){.anchor
.field}`reason: `[`String`](https://doc.rust-lang.org/1.91.1/alloc/string/struct.String.html "struct alloc::string::String"){.struct}]{#variant.OtherError.field.reason
.section-header}

::: docblock
Detailed explanation of the error
:::
::::
:::::

::: {#variant.Decimal .section .variant}
[§](#variant.Decimal){.anchor}

### Decimal([DecimalError](decimal/enum.DecimalError.html "enum optionstratlib::error::decimal::DecimalError"){.enum}) {#decimaldecimalerror .code-header}
:::

::: docblock
Error when DecimalError occurs
:::

::: {#variant.Greeks .section .variant}
[§](#variant.Greeks){.anchor}

### Greeks([GreeksError](greeks/enum.GreeksError.html "enum optionstratlib::error::greeks::GreeksError"){.enum}) {#greeksgreekserror .code-header}
:::

::: docblock
Error when GreeksError occurs
:::
::::::::::::::::::::::::::::::::::::::::::::::::::::

## Implementations[§](#implementations){.anchor} {#implementations .section-header}

:::::::::::::::::::: {#implementations-list}
:::: {#impl-OptionsError .section .impl}
[Source](../../src/optionstratlib/error/options.rs.html#250-365){.src
.rightside}[§](#impl-OptionsError){.anchor}

### impl [OptionsError](enum.OptionsError.html "enum optionstratlib::error::OptionsError"){.enum} {#impl-optionserror .code-header}

::: docblock
Helper methods for creating common options errors.
:::
::::

:::: docblock
This implementation provides convenient factory methods for creating
different variants of `OptionsError` without having to manually
construct the enum variants. Each method corresponds to a specific error
type and properly formats the error fields.

#### [§](#methods){.doc-anchor}Methods

- `validation_error` - Creates an error for parameter validation
  failures
- `pricing_error` - Creates an error for pricing calculation issues
- `greeks_error` - Creates an error for problems with Greeks
  calculations
- `time_error` - Creates an error for time-related calculations
- `payoff_error` - Creates an error for payoff calculation problems
- `update_error` - Creates an error for option data update issues

#### [§](#examples){.doc-anchor}Examples

::: example-wrap
``` {.rust .rust-example-rendered}
use optionstratlib::error::OptionsError;
let error = OptionsError::validation_error("strike_price", "must be positive");

// Create a pricing error
let error = OptionsError::pricing_error("black_scholes", "invalid volatility input");
```
:::
::::

::::::::::::::: impl-items
::: {#method.validation_error .section .method}
[Source](../../src/optionstratlib/error/options.rs.html#263-268){.src
.rightside}

#### pub fn [validation_error](#method.validation_error){.fn}(field: &[str](https://doc.rust-lang.org/1.91.1/std/primitive.str.html){.primitive}, reason: &[str](https://doc.rust-lang.org/1.91.1/std/primitive.str.html){.primitive}) -\> Self {#pub-fn-validation_errorfield-str-reason-str---self .code-header}
:::

::: docblock
Creates a validation error with the specified field name and reason.

This method is used when option parameters fail validation checks.

##### [§](#parameters){.doc-anchor}Parameters

- `field` - The name of the field that failed validation
- `reason` - The reason why validation failed

##### [§](#returns){.doc-anchor}Returns

An `OptionsError::ValidationError` variant with formatted fields
:::

::: {#method.pricing_error .section .method}
[Source](../../src/optionstratlib/error/options.rs.html#282-287){.src
.rightside}

#### pub fn [pricing_error](#method.pricing_error){.fn}(method: &[str](https://doc.rust-lang.org/1.91.1/std/primitive.str.html){.primitive}, reason: &[str](https://doc.rust-lang.org/1.91.1/std/primitive.str.html){.primitive}) -\> Self {#pub-fn-pricing_errormethod-str-reason-str---self .code-header}
:::

::: docblock
Creates a pricing error with the specified pricing method and reason.

This method is used when an error occurs during option price
calculation.

##### [§](#parameters-1){.doc-anchor}Parameters

- `method` - The name of the pricing method that encountered an error
- `reason` - The description of what went wrong

##### [§](#returns-1){.doc-anchor}Returns

An `OptionsError::PricingError` variant with formatted fields
:::

::: {#method.greeks_error .section .method}
[Source](../../src/optionstratlib/error/options.rs.html#302-307){.src
.rightside}

#### pub fn [greeks_error](#method.greeks_error){.fn}(greek: &[str](https://doc.rust-lang.org/1.91.1/std/primitive.str.html){.primitive}, reason: &[str](https://doc.rust-lang.org/1.91.1/std/primitive.str.html){.primitive}) -\> Self {#pub-fn-greeks_errorgreek-str-reason-str---self .code-header}
:::

::: docblock
Creates a Greeks calculation error with the specified Greek name and
reason.

This method is used when an error occurs during the calculation of
option Greeks (delta, gamma, theta, vega, etc.).

##### [§](#parameters-2){.doc-anchor}Parameters

- `greek` - The name of the Greek calculation that failed
- `reason` - The description of what went wrong

##### [§](#returns-2){.doc-anchor}Returns

An `OptionsError::GreeksCalculationError` variant with formatted fields
:::

::: {#method.time_error .section .method}
[Source](../../src/optionstratlib/error/options.rs.html#322-327){.src
.rightside}

#### pub fn [time_error](#method.time_error){.fn}(operation: &[str](https://doc.rust-lang.org/1.91.1/std/primitive.str.html){.primitive}, reason: &[str](https://doc.rust-lang.org/1.91.1/std/primitive.str.html){.primitive}) -\> Self {#pub-fn-time_erroroperation-str-reason-str---self .code-header}
:::

::: docblock
Creates a time calculation error with the specified operation and
reason.

This method is used when an error occurs during time-related
calculations, such as time to expiration, day count conventions, or
calendar adjustments.

##### [§](#parameters-3){.doc-anchor}Parameters

- `operation` - The name of the time operation that failed
- `reason` - The description of what went wrong

##### [§](#returns-3){.doc-anchor}Returns

An `OptionsError::TimeError` variant with formatted fields
:::

::: {#method.payoff_error .section .method}
[Source](../../src/optionstratlib/error/options.rs.html#340-344){.src
.rightside}

#### pub fn [payoff_error](#method.payoff_error){.fn}(reason: &[str](https://doc.rust-lang.org/1.91.1/std/primitive.str.html){.primitive}) -\> Self {#pub-fn-payoff_errorreason-str---self .code-header}
:::

::: docblock
Creates a payoff calculation error with the specified reason.

This method is used when an error occurs during the calculation of
option payoffs.

##### [§](#parameters-4){.doc-anchor}Parameters

- `reason` - The description of what went wrong

##### [§](#returns-4){.doc-anchor}Returns

An `OptionsError::PayoffError` variant with formatted reason
:::

::: {#method.update_error .section .method}
[Source](../../src/optionstratlib/error/options.rs.html#359-364){.src
.rightside}

#### pub fn [update_error](#method.update_error){.fn}(field: &[str](https://doc.rust-lang.org/1.91.1/std/primitive.str.html){.primitive}, reason: &[str](https://doc.rust-lang.org/1.91.1/std/primitive.str.html){.primitive}) -\> Self {#pub-fn-update_errorfield-str-reason-str---self .code-header}
:::

::: docblock
Creates an update error with the specified field and reason.

This method is used when an error occurs during the update of option
parameters or other option data.

##### [§](#parameters-5){.doc-anchor}Parameters

- `field` - The name of the field that failed to update
- `reason` - The description of what went wrong

##### [§](#returns-5){.doc-anchor}Returns

An `OptionsError::UpdateError` variant with formatted fields
:::
:::::::::::::::
::::::::::::::::::::

## Trait Implementations[§](#trait-implementations){.anchor} {#trait-implementations .section-header}

::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::: {#trait-implementations-list}
::: {#impl-Debug-for-OptionsError .section .impl}
[Source](../../src/optionstratlib/error/options.rs.html#99){.src
.rightside}[§](#impl-Debug-for-OptionsError){.anchor}

### impl [Debug](https://doc.rust-lang.org/1.91.1/core/fmt/trait.Debug.html "trait core::fmt::Debug"){.trait} for [OptionsError](enum.OptionsError.html "enum optionstratlib::error::OptionsError"){.enum} {#impl-debug-for-optionserror .code-header}
:::

::::: impl-items
::: {#method.fmt-1 .section .method .trait-impl}
[Source](../../src/optionstratlib/error/options.rs.html#99){.src
.rightside}[§](#method.fmt-1){.anchor}

#### fn [fmt](https://doc.rust-lang.org/1.91.1/core/fmt/trait.Debug.html#tymethod.fmt){.fn}(&self, f: &mut [Formatter](https://doc.rust-lang.org/1.91.1/core/fmt/struct.Formatter.html "struct core::fmt::Formatter"){.struct}\<\'\_\>) -\> [Result](https://doc.rust-lang.org/1.91.1/core/fmt/type.Result.html "type core::fmt::Result"){.type} {#fn-fmtself-f-mut-formatter_---result .code-header}
:::

::: docblock
Formats the value using the given formatter. [Read
more](https://doc.rust-lang.org/1.91.1/core/fmt/trait.Debug.html#tymethod.fmt)
:::
:::::

::: {#impl-Display-for-OptionsError .section .impl}
[Source](../../src/optionstratlib/error/options.rs.html#99){.src
.rightside}[§](#impl-Display-for-OptionsError){.anchor}

### impl [Display](https://doc.rust-lang.org/1.91.1/core/fmt/trait.Display.html "trait core::fmt::Display"){.trait} for [OptionsError](enum.OptionsError.html "enum optionstratlib::error::OptionsError"){.enum} {#impl-display-for-optionserror .code-header}
:::

::::: impl-items
::: {#method.fmt .section .method .trait-impl}
[Source](../../src/optionstratlib/error/options.rs.html#99){.src
.rightside}[§](#method.fmt){.anchor}

#### fn [fmt](https://doc.rust-lang.org/1.91.1/core/fmt/trait.Display.html#tymethod.fmt){.fn}(&self, \_\_formatter: &mut [Formatter](https://doc.rust-lang.org/1.91.1/core/fmt/struct.Formatter.html "struct core::fmt::Formatter"){.struct}\<\'\_\>) -\> [Result](https://doc.rust-lang.org/1.91.1/core/fmt/type.Result.html "type core::fmt::Result"){.type} {#fn-fmtself-__formatter-mut-formatter_---result .code-header}
:::

::: docblock
Formats the value using the given formatter. [Read
more](https://doc.rust-lang.org/1.91.1/core/fmt/trait.Display.html#tymethod.fmt)
:::
:::::

::: {#impl-Error-for-OptionsError .section .impl}
[Source](../../src/optionstratlib/error/options.rs.html#99){.src
.rightside}[§](#impl-Error-for-OptionsError){.anchor}

### impl [Error](https://doc.rust-lang.org/1.91.1/core/error/trait.Error.html "trait core::error::Error"){.trait} for [OptionsError](enum.OptionsError.html "enum optionstratlib::error::OptionsError"){.enum} {#impl-error-for-optionserror .code-header}
:::

::::::::::::: impl-items
::: {#method.source .section .method .trait-impl}
[Source](../../src/optionstratlib/error/options.rs.html#99){.src
.rightside}[§](#method.source){.anchor}

#### fn [source](https://doc.rust-lang.org/1.91.1/core/error/trait.Error.html#method.source){.fn}(&self) -\> [Option](https://doc.rust-lang.org/1.91.1/core/option/enum.Option.html "enum core::option::Option"){.enum}\<&(dyn [Error](https://doc.rust-lang.org/1.91.1/core/error/trait.Error.html "trait core::error::Error"){.trait} + \'static)\> {#fn-sourceself---optiondyn-error-static .code-header}
:::

::: docblock
Returns the lower-level source of this error, if any. [Read
more](https://doc.rust-lang.org/1.91.1/core/error/trait.Error.html#method.source)
:::

::: {#method.description .section .method .trait-impl}
[[1.0.0]{.since title="Stable since Rust version 1.0.0"} ·
[Source](https://doc.rust-lang.org/1.91.1/src/core/error.rs.html#131){.src}]{.rightside}[§](#method.description){.anchor}

#### fn [description](https://doc.rust-lang.org/1.91.1/core/error/trait.Error.html#method.description){.fn}(&self) -\> &[str](https://doc.rust-lang.org/1.91.1/std/primitive.str.html){.primitive} {#fn-descriptionself---str .code-header}
:::

[]{.item-info}

::: {.stab .deprecated}
👎Deprecated since 1.42.0: use the Display impl or to_string()
:::

::: docblock
[Read
more](https://doc.rust-lang.org/1.91.1/core/error/trait.Error.html#method.description)
:::

::: {#method.cause .section .method .trait-impl}
[[1.0.0]{.since title="Stable since Rust version 1.0.0"} ·
[Source](https://doc.rust-lang.org/1.91.1/src/core/error.rs.html#141){.src}]{.rightside}[§](#method.cause){.anchor}

#### fn [cause](https://doc.rust-lang.org/1.91.1/core/error/trait.Error.html#method.cause){.fn}(&self) -\> [Option](https://doc.rust-lang.org/1.91.1/core/option/enum.Option.html "enum core::option::Option"){.enum}\<&dyn [Error](https://doc.rust-lang.org/1.91.1/core/error/trait.Error.html "trait core::error::Error"){.trait}\> {#fn-causeself---optiondyn-error .code-header}
:::

[]{.item-info}

::: {.stab .deprecated}
👎Deprecated since 1.33.0: replaced by Error::source, which can support
downcasting
:::

::: {#method.provide .section .method .trait-impl}
[Source](https://doc.rust-lang.org/1.91.1/src/core/error.rs.html#204){.src
.rightside}[§](#method.provide){.anchor}

#### fn [provide](https://doc.rust-lang.org/1.91.1/core/error/trait.Error.html#method.provide){.fn}\<\'a\>(&\'a self, request: &mut [Request](https://doc.rust-lang.org/1.91.1/core/error/struct.Request.html "struct core::error::Request"){.struct}\<\'a\>) {#fn-provideaa-self-request-mut-requesta .code-header}
:::

[]{.item-info}

::: {.stab .unstable}
🔬This is a nightly-only experimental API.
(`error_generic_member_access`)
:::

::: docblock
Provides type-based access to context intended for error reports. [Read
more](https://doc.rust-lang.org/1.91.1/core/error/trait.Error.html#method.provide)
:::
:::::::::::::

::: {#impl-From%3C%26str%3E-for-OptionsError .section .impl}
[Source](../../src/optionstratlib/error/options.rs.html#375-382){.src
.rightside}[§](#impl-From%3C%26str%3E-for-OptionsError){.anchor}

### impl [From](https://doc.rust-lang.org/1.91.1/core/convert/trait.From.html "trait core::convert::From"){.trait}\<&[str](https://doc.rust-lang.org/1.91.1/std/primitive.str.html){.primitive}\> for [OptionsError](enum.OptionsError.html "enum optionstratlib::error::OptionsError"){.enum} {#impl-fromstr-for-optionserror .code-header}
:::

::::: impl-items
::: {#method.from-5 .section .method .trait-impl}
[Source](../../src/optionstratlib/error/options.rs.html#376-381){.src
.rightside}[§](#method.from-5){.anchor}

#### fn [from](https://doc.rust-lang.org/1.91.1/core/convert/trait.From.html#tymethod.from){.fn}(err: &[str](https://doc.rust-lang.org/1.91.1/std/primitive.str.html){.primitive}) -\> Self {#fn-fromerr-str---self .code-header}
:::

::: docblock
Converts to this type from the input type.
:::
:::::

::: {#impl-From%3CBox%3Cdyn+Error%3E%3E-for-OptionsError .section .impl}
[Source](../../src/optionstratlib/error/options.rs.html#367-373){.src
.rightside}[§](#impl-From%3CBox%3Cdyn+Error%3E%3E-for-OptionsError){.anchor}

### impl [From](https://doc.rust-lang.org/1.91.1/core/convert/trait.From.html "trait core::convert::From"){.trait}\<[Box](https://doc.rust-lang.org/1.91.1/alloc/boxed/struct.Box.html "struct alloc::boxed::Box"){.struct}\<dyn [Error](https://doc.rust-lang.org/1.91.1/core/error/trait.Error.html "trait core::error::Error"){.trait}\>\> for [OptionsError](enum.OptionsError.html "enum optionstratlib::error::OptionsError"){.enum} {#impl-fromboxdyn-error-for-optionserror .code-header}
:::

::::: impl-items
::: {#method.from-4 .section .method .trait-impl}
[Source](../../src/optionstratlib/error/options.rs.html#368-372){.src
.rightside}[§](#method.from-4){.anchor}

#### fn [from](https://doc.rust-lang.org/1.91.1/core/convert/trait.From.html#tymethod.from){.fn}(err: [Box](https://doc.rust-lang.org/1.91.1/alloc/boxed/struct.Box.html "struct alloc::boxed::Box"){.struct}\<dyn [Error](https://doc.rust-lang.org/1.91.1/core/error/trait.Error.html "trait core::error::Error"){.trait}\>) -\> Self {#fn-fromerr-boxdyn-error---self .code-header}
:::

::: docblock
Converts to this type from the input type.
:::
:::::

::: {#impl-From%3CDecimalError%3E-for-OptionsError .section .impl}
[Source](../../src/optionstratlib/error/options.rs.html#176){.src
.rightside}[§](#impl-From%3CDecimalError%3E-for-OptionsError){.anchor}

### impl [From](https://doc.rust-lang.org/1.91.1/core/convert/trait.From.html "trait core::convert::From"){.trait}\<[DecimalError](decimal/enum.DecimalError.html "enum optionstratlib::error::decimal::DecimalError"){.enum}\> for [OptionsError](enum.OptionsError.html "enum optionstratlib::error::OptionsError"){.enum} {#impl-fromdecimalerror-for-optionserror .code-header}
:::

::::: impl-items
::: {#method.from-2 .section .method .trait-impl}
[Source](../../src/optionstratlib/error/options.rs.html#99){.src
.rightside}[§](#method.from-2){.anchor}

#### fn [from](https://doc.rust-lang.org/1.91.1/core/convert/trait.From.html#tymethod.from){.fn}(source: [DecimalError](decimal/enum.DecimalError.html "enum optionstratlib::error::decimal::DecimalError"){.enum}) -\> Self {#fn-fromsource-decimalerror---self .code-header}
:::

::: docblock
Converts to this type from the input type.
:::
:::::

::: {#impl-From%3CGreeksError%3E-for-OptionsError .section .impl}
[Source](../../src/optionstratlib/error/options.rs.html#180){.src
.rightside}[§](#impl-From%3CGreeksError%3E-for-OptionsError){.anchor}

### impl [From](https://doc.rust-lang.org/1.91.1/core/convert/trait.From.html "trait core::convert::From"){.trait}\<[GreeksError](greeks/enum.GreeksError.html "enum optionstratlib::error::greeks::GreeksError"){.enum}\> for [OptionsError](enum.OptionsError.html "enum optionstratlib::error::OptionsError"){.enum} {#impl-fromgreekserror-for-optionserror .code-header}
:::

::::: impl-items
::: {#method.from-3 .section .method .trait-impl}
[Source](../../src/optionstratlib/error/options.rs.html#99){.src
.rightside}[§](#method.from-3){.anchor}

#### fn [from](https://doc.rust-lang.org/1.91.1/core/convert/trait.From.html#tymethod.from){.fn}(source: [GreeksError](greeks/enum.GreeksError.html "enum optionstratlib::error::greeks::GreeksError"){.enum}) -\> Self {#fn-fromsource-greekserror---self .code-header}
:::

::: docblock
Converts to this type from the input type.
:::
:::::

::: {#impl-From%3COptionsError%3E-for-ChainError .section .impl}
[Source](../../src/optionstratlib/error/chains.rs.html#412-418){.src
.rightside}[§](#impl-From%3COptionsError%3E-for-ChainError){.anchor}

### impl [From](https://doc.rust-lang.org/1.91.1/core/convert/trait.From.html "trait core::convert::From"){.trait}\<[OptionsError](enum.OptionsError.html "enum optionstratlib::error::OptionsError"){.enum}\> for [ChainError](chains/enum.ChainError.html "enum optionstratlib::error::chains::ChainError"){.enum} {#impl-fromoptionserror-for-chainerror .code-header}
:::

::::: impl-items
::: {#method.from .section .method .trait-impl}
[Source](../../src/optionstratlib/error/chains.rs.html#413-417){.src
.rightside}[§](#method.from){.anchor}

#### fn [from](https://doc.rust-lang.org/1.91.1/core/convert/trait.From.html#tymethod.from){.fn}(error: [OptionsError](enum.OptionsError.html "enum optionstratlib::error::OptionsError"){.enum}) -\> Self {#fn-fromerror-optionserror---self .code-header}
:::

::: docblock
Converts to this type from the input type.
:::
:::::

::: {#impl-From%3COptionsError%3E-for-CurveError .section .impl}
[Source](../../src/optionstratlib/error/curves.rs.html#150){.src
.rightside}[§](#impl-From%3COptionsError%3E-for-CurveError){.anchor}

### impl [From](https://doc.rust-lang.org/1.91.1/core/convert/trait.From.html "trait core::convert::From"){.trait}\<[OptionsError](enum.OptionsError.html "enum optionstratlib::error::OptionsError"){.enum}\> for [CurveError](curves/enum.CurveError.html "enum optionstratlib::error::curves::CurveError"){.enum} {#impl-fromoptionserror-for-curveerror .code-header}
:::

::::: impl-items
::: {#method.from-1 .section .method .trait-impl}
[Source](../../src/optionstratlib/error/curves.rs.html#93){.src
.rightside}[§](#method.from-1){.anchor}

#### fn [from](https://doc.rust-lang.org/1.91.1/core/convert/trait.From.html#tymethod.from){.fn}(source: [OptionsError](enum.OptionsError.html "enum optionstratlib::error::OptionsError"){.enum}) -\> Self {#fn-fromsource-optionserror---self .code-header}
:::

::: docblock
Converts to this type from the input type.
:::
:::::

::: {#impl-From%3COptionsError%3E-for-Error .section .impl}
[Source](../../src/optionstratlib/error/unified.rs.html#35){.src
.rightside}[§](#impl-From%3COptionsError%3E-for-Error){.anchor}

### impl [From](https://doc.rust-lang.org/1.91.1/core/convert/trait.From.html "trait core::convert::From"){.trait}\<[OptionsError](enum.OptionsError.html "enum optionstratlib::error::OptionsError"){.enum}\> for [Error](unified/enum.Error.html "enum optionstratlib::error::unified::Error"){.enum} {#impl-fromoptionserror-for-error .code-header}
:::

::::: impl-items
::: {#method.from-13 .section .method .trait-impl}
[Source](../../src/optionstratlib/error/unified.rs.html#31){.src
.rightside}[§](#method.from-13){.anchor}

#### fn [from](https://doc.rust-lang.org/1.91.1/core/convert/trait.From.html#tymethod.from){.fn}(source: [OptionsError](enum.OptionsError.html "enum optionstratlib::error::OptionsError"){.enum}) -\> Self {#fn-fromsource-optionserror---self-1 .code-header}
:::

::: docblock
Converts to this type from the input type.
:::
:::::

::: {#impl-From%3COptionsError%3E-for-PricingError .section .impl}
[Source](../../src/optionstratlib/error/pricing.rs.html#39){.src
.rightside}[§](#impl-From%3COptionsError%3E-for-PricingError){.anchor}

### impl [From](https://doc.rust-lang.org/1.91.1/core/convert/trait.From.html "trait core::convert::From"){.trait}\<[OptionsError](enum.OptionsError.html "enum optionstratlib::error::OptionsError"){.enum}\> for [PricingError](pricing/enum.PricingError.html "enum optionstratlib::error::pricing::PricingError"){.enum} {#impl-fromoptionserror-for-pricingerror .code-header}
:::

::::: impl-items
::: {#method.from-11 .section .method .trait-impl}
[Source](../../src/optionstratlib/error/pricing.rs.html#8){.src
.rightside}[§](#method.from-11){.anchor}

#### fn [from](https://doc.rust-lang.org/1.91.1/core/convert/trait.From.html#tymethod.from){.fn}(source: [OptionsError](enum.OptionsError.html "enum optionstratlib::error::OptionsError"){.enum}) -\> Self {#fn-fromsource-optionserror---self-2 .code-header}
:::

::: docblock
Converts to this type from the input type.
:::
:::::

::: {#impl-From%3COptionsError%3E-for-SimulationError .section .impl}
[Source](../../src/optionstratlib/error/simulation.rs.html#116-122){.src
.rightside}[§](#impl-From%3COptionsError%3E-for-SimulationError){.anchor}

### impl [From](https://doc.rust-lang.org/1.91.1/core/convert/trait.From.html "trait core::convert::From"){.trait}\<[OptionsError](enum.OptionsError.html "enum optionstratlib::error::OptionsError"){.enum}\> for [SimulationError](simulation/enum.SimulationError.html "enum optionstratlib::error::simulation::SimulationError"){.enum} {#impl-fromoptionserror-for-simulationerror .code-header}
:::

::::: impl-items
::: {#method.from-10 .section .method .trait-impl}
[Source](../../src/optionstratlib/error/simulation.rs.html#117-121){.src
.rightside}[§](#method.from-10){.anchor}

#### fn [from](https://doc.rust-lang.org/1.91.1/core/convert/trait.From.html#tymethod.from){.fn}(err: [OptionsError](enum.OptionsError.html "enum optionstratlib::error::OptionsError"){.enum}) -\> Self {#fn-fromerr-optionserror---self .code-header}
:::

::: docblock
Converts to this type from the input type.
:::
:::::

::: {#impl-From%3COptionsError%3E-for-StrategyError .section .impl}
[Source](../../src/optionstratlib/error/strategies.rs.html#348-355){.src
.rightside}[§](#impl-From%3COptionsError%3E-for-StrategyError){.anchor}

### impl [From](https://doc.rust-lang.org/1.91.1/core/convert/trait.From.html "trait core::convert::From"){.trait}\<[OptionsError](enum.OptionsError.html "enum optionstratlib::error::OptionsError"){.enum}\> for [StrategyError](strategies/enum.StrategyError.html "enum optionstratlib::error::strategies::StrategyError"){.enum} {#impl-fromoptionserror-for-strategyerror .code-header}
:::

::::: impl-items
::: {#method.from-8 .section .method .trait-impl}
[Source](../../src/optionstratlib/error/strategies.rs.html#349-354){.src
.rightside}[§](#method.from-8){.anchor}

#### fn [from](https://doc.rust-lang.org/1.91.1/core/convert/trait.From.html#tymethod.from){.fn}(err: [OptionsError](enum.OptionsError.html "enum optionstratlib::error::OptionsError"){.enum}) -\> Self {#fn-fromerr-optionserror---self-1 .code-header}
:::

::: docblock
Converts to this type from the input type.
:::
:::::

::: {#impl-From%3COptionsError%3E-for-SurfaceError .section .impl}
[Source](../../src/optionstratlib/error/surfaces.rs.html#79){.src
.rightside}[§](#impl-From%3COptionsError%3E-for-SurfaceError){.anchor}

### impl [From](https://doc.rust-lang.org/1.91.1/core/convert/trait.From.html "trait core::convert::From"){.trait}\<[OptionsError](enum.OptionsError.html "enum optionstratlib::error::OptionsError"){.enum}\> for [SurfaceError](enum.SurfaceError.html "enum optionstratlib::error::SurfaceError"){.enum} {#impl-fromoptionserror-for-surfaceerror .code-header}
:::

::::: impl-items
::: {#method.from-9 .section .method .trait-impl}
[Source](../../src/optionstratlib/error/surfaces.rs.html#28){.src
.rightside}[§](#method.from-9){.anchor}

#### fn [from](https://doc.rust-lang.org/1.91.1/core/convert/trait.From.html#tymethod.from){.fn}(source: [OptionsError](enum.OptionsError.html "enum optionstratlib::error::OptionsError"){.enum}) -\> Self {#fn-fromsource-optionserror---self-3 .code-header}
:::

::: docblock
Converts to this type from the input type.
:::
:::::

::: {#impl-From%3COptionsError%3E-for-VolatilityError .section .impl}
[Source](../../src/optionstratlib/error/volatility.rs.html#59){.src
.rightside}[§](#impl-From%3COptionsError%3E-for-VolatilityError){.anchor}

### impl [From](https://doc.rust-lang.org/1.91.1/core/convert/trait.From.html "trait core::convert::From"){.trait}\<[OptionsError](enum.OptionsError.html "enum optionstratlib::error::OptionsError"){.enum}\> for [VolatilityError](enum.VolatilityError.html "enum optionstratlib::error::VolatilityError"){.enum} {#impl-fromoptionserror-for-volatilityerror .code-header}
:::

::::: impl-items
::: {#method.from-12 .section .method .trait-impl}
[Source](../../src/optionstratlib/error/volatility.rs.html#20){.src
.rightside}[§](#method.from-12){.anchor}

#### fn [from](https://doc.rust-lang.org/1.91.1/core/convert/trait.From.html#tymethod.from){.fn}(source: [OptionsError](enum.OptionsError.html "enum optionstratlib::error::OptionsError"){.enum}) -\> Self {#fn-fromsource-optionserror---self-4 .code-header}
:::

::: docblock
Converts to this type from the input type.
:::
:::::

::: {#impl-From%3CPricingError%3E-for-OptionsError .section .impl}
[Source](../../src/optionstratlib/error/options.rs.html#393-400){.src
.rightside}[§](#impl-From%3CPricingError%3E-for-OptionsError){.anchor}

### impl [From](https://doc.rust-lang.org/1.91.1/core/convert/trait.From.html "trait core::convert::From"){.trait}\<[PricingError](pricing/enum.PricingError.html "enum optionstratlib::error::pricing::PricingError"){.enum}\> for [OptionsError](enum.OptionsError.html "enum optionstratlib::error::OptionsError"){.enum} {#impl-frompricingerror-for-optionserror .code-header}
:::

::::: impl-items
::: {#method.from-7 .section .method .trait-impl}
[Source](../../src/optionstratlib/error/options.rs.html#394-399){.src
.rightside}[§](#method.from-7){.anchor}

#### fn [from](https://doc.rust-lang.org/1.91.1/core/convert/trait.From.html#tymethod.from){.fn}(value: [PricingError](pricing/enum.PricingError.html "enum optionstratlib::error::pricing::PricingError"){.enum}) -\> Self {#fn-fromvalue-pricingerror---self .code-header}
:::

::: docblock
Converts to this type from the input type.
:::
:::::

::: {#impl-From%3CString%3E-for-OptionsError .section .impl}
[Source](../../src/optionstratlib/error/options.rs.html#384-391){.src
.rightside}[§](#impl-From%3CString%3E-for-OptionsError){.anchor}

### impl [From](https://doc.rust-lang.org/1.91.1/core/convert/trait.From.html "trait core::convert::From"){.trait}\<[String](https://doc.rust-lang.org/1.91.1/alloc/string/struct.String.html "struct alloc::string::String"){.struct}\> for [OptionsError](enum.OptionsError.html "enum optionstratlib::error::OptionsError"){.enum} {#impl-fromstring-for-optionserror .code-header}
:::

::::: impl-items
::: {#method.from-6 .section .method .trait-impl}
[Source](../../src/optionstratlib/error/options.rs.html#385-390){.src
.rightside}[§](#method.from-6){.anchor}

#### fn [from](https://doc.rust-lang.org/1.91.1/core/convert/trait.From.html#tymethod.from){.fn}(err: [String](https://doc.rust-lang.org/1.91.1/alloc/string/struct.String.html "struct alloc::string::String"){.struct}) -\> Self {#fn-fromerr-string---self .code-header}
:::

::: docblock
Converts to this type from the input type.
:::
:::::
:::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::

## Auto Trait Implementations[§](#synthetic-implementations){.anchor} {#synthetic-implementations .section-header}

::::::::: {#synthetic-implementations-list}
::: {#impl-Freeze-for-OptionsError .section .impl}
[§](#impl-Freeze-for-OptionsError){.anchor}

### impl [Freeze](https://doc.rust-lang.org/1.91.1/core/marker/trait.Freeze.html "trait core::marker::Freeze"){.trait} for [OptionsError](enum.OptionsError.html "enum optionstratlib::error::OptionsError"){.enum} {#impl-freeze-for-optionserror .code-header}
:::

::: {#impl-RefUnwindSafe-for-OptionsError .section .impl}
[§](#impl-RefUnwindSafe-for-OptionsError){.anchor}

### impl [RefUnwindSafe](https://doc.rust-lang.org/1.91.1/core/panic/unwind_safe/trait.RefUnwindSafe.html "trait core::panic::unwind_safe::RefUnwindSafe"){.trait} for [OptionsError](enum.OptionsError.html "enum optionstratlib::error::OptionsError"){.enum} {#impl-refunwindsafe-for-optionserror .code-header}
:::

::: {#impl-Send-for-OptionsError .section .impl}
[§](#impl-Send-for-OptionsError){.anchor}

### impl [Send](https://doc.rust-lang.org/1.91.1/core/marker/trait.Send.html "trait core::marker::Send"){.trait} for [OptionsError](enum.OptionsError.html "enum optionstratlib::error::OptionsError"){.enum} {#impl-send-for-optionserror .code-header}
:::

::: {#impl-Sync-for-OptionsError .section .impl}
[§](#impl-Sync-for-OptionsError){.anchor}

### impl [Sync](https://doc.rust-lang.org/1.91.1/core/marker/trait.Sync.html "trait core::marker::Sync"){.trait} for [OptionsError](enum.OptionsError.html "enum optionstratlib::error::OptionsError"){.enum} {#impl-sync-for-optionserror .code-header}
:::

::: {#impl-Unpin-for-OptionsError .section .impl}
[§](#impl-Unpin-for-OptionsError){.anchor}

### impl [Unpin](https://doc.rust-lang.org/1.91.1/core/marker/trait.Unpin.html "trait core::marker::Unpin"){.trait} for [OptionsError](enum.OptionsError.html "enum optionstratlib::error::OptionsError"){.enum} {#impl-unpin-for-optionserror .code-header}
:::

::: {#impl-UnwindSafe-for-OptionsError .section .impl}
[§](#impl-UnwindSafe-for-OptionsError){.anchor}

### impl [UnwindSafe](https://doc.rust-lang.org/1.91.1/core/panic/unwind_safe/trait.UnwindSafe.html "trait core::panic::unwind_safe::UnwindSafe"){.trait} for [OptionsError](enum.OptionsError.html "enum optionstratlib::error::OptionsError"){.enum} {#impl-unwindsafe-for-optionserror .code-header}
:::
:::::::::

## Blanket Implementations[§](#blanket-implementations){.anchor} {#blanket-implementations .section-header}

::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::: {#blanket-implementations-list}
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
::: {#method.from-14 .section .method .trait-impl}
[Source](https://doc.rust-lang.org/1.91.1/src/core/convert/mod.rs.html#788){.src
.rightside}[§](#method.from-14){.anchor}

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
:::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
:::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
