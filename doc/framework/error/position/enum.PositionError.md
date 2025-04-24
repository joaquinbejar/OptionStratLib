:::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::: width-limiter
::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../../index.html)::[error](../index.html)::[position](index.html)
:::

# Enum [PositionError]{.enum}Copy item path

[[Source](../../../src/optionstratlib/error/position.rs.html#95-104){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub enum PositionError {
    StrategyError(StrategyErrorKind),
    ValidationError(PositionValidationErrorKind),
    LimitError(PositionLimitErrorKind),
}
```

Expand description

::: docblock
Represents errors that can occur when managing positions in strategies

This enum provides a top-level categorization of position-related
errors, grouping them by their source or nature. It helps with routing
errors to appropriate handlers and providing context-aware error
messages.

## [§](#variants-1){.doc-anchor}Variants {#variants-1}

- `StrategyError` - Errors related to strategy operations such as
  configuration issues or capacity limitations.

- `ValidationError` - Errors related to position validation including
  issues with size, price, or compatibility with strategy requirements.

- `LimitError` - Errors related to position limits such as maximum
  number of positions or maximum exposure thresholds.

## [§](#usage){.doc-anchor}Usage

This error type is typically used in trading systems where positions
need to be validated, managed, and executed within the context of
trading strategies.
:::

## Variants[§](#variants){.anchor} {#variants .variants .section-header}

::::::::: variants
::: {#variant.StrategyError .section .variant}
[§](#variant.StrategyError){.anchor}

### StrategyError([StrategyErrorKind](enum.StrategyErrorKind.html "enum optionstratlib::error::position::StrategyErrorKind"){.enum}) {#strategyerrorstrategyerrorkind .code-header}
:::

::: docblock
Errors related to strategy operations
:::

::: {#variant.ValidationError .section .variant}
[§](#variant.ValidationError){.anchor}

### ValidationError([PositionValidationErrorKind](enum.PositionValidationErrorKind.html "enum optionstratlib::error::position::PositionValidationErrorKind"){.enum}) {#validationerrorpositionvalidationerrorkind .code-header}
:::

::: docblock
Errors related to position validation
:::

::: {#variant.LimitError .section .variant}
[§](#variant.LimitError){.anchor}

### LimitError([PositionLimitErrorKind](enum.PositionLimitErrorKind.html "enum optionstratlib::error::position::PositionLimitErrorKind"){.enum}) {#limiterrorpositionlimiterrorkind .code-header}
:::

::: docblock
Errors related to position limits
:::
:::::::::

## Implementations[§](#implementations){.anchor} {#implementations .section-header}

::::::::::::::::::: {#implementations-list}
:::: {#impl-PositionError .section .impl}
[Source](../../../src/optionstratlib/error/position.rs.html#407-507){.src
.rightside}[§](#impl-PositionError){.anchor}

### impl [PositionError](enum.PositionError.html "enum optionstratlib::error::position::PositionError"){.enum} {#impl-positionerror .code-header}

::: docblock
Factory methods for creating position-related errors
:::
::::

::: docblock
This implementation provides a set of convenience factory methods for
creating different types of position errors. These methods create
properly structured error instances with clear, descriptive information
about what went wrong.

#### [§](#methods){.doc-anchor}Methods

These factory methods simplify error creation throughout the codebase
and ensure that errors have consistent formatting and information.
:::

::::::::::::::: impl-items
::: {#method.unsupported_operation .section .method}
[Source](../../../src/optionstratlib/error/position.rs.html#418-423){.src
.rightside}

#### pub fn [unsupported_operation](#method.unsupported_operation){.fn}(strategy_type: &[str](https://doc.rust-lang.org/1.86.0/std/primitive.str.html){.primitive}, operation: &[str](https://doc.rust-lang.org/1.86.0/std/primitive.str.html){.primitive}) -\> Self {#pub-fn-unsupported_operationstrategy_type-str-operation-str---self .code-header}
:::

::: docblock
Creates an error for operations not supported by a specific strategy
type

##### [§](#parameters){.doc-anchor}Parameters

- `strategy_type` - The name or identifier of the strategy that doesn't
  support the operation
- `operation` - The name of the unsupported operation that was attempted

##### [§](#returns){.doc-anchor}Returns

A `PositionError::StrategyError` variant with UnsupportedOperation
details
:::

::: {#method.strategy_full .section .method}
[Source](../../../src/optionstratlib/error/position.rs.html#435-440){.src
.rightside}

#### pub fn [strategy_full](#method.strategy_full){.fn}(strategy_type: &[str](https://doc.rust-lang.org/1.86.0/std/primitive.str.html){.primitive}, max_positions: [usize](https://doc.rust-lang.org/1.86.0/std/primitive.usize.html){.primitive}) -\> Self {#pub-fn-strategy_fullstrategy_type-str-max_positions-usize---self .code-header}
:::

::: docblock
Creates an error when a strategy has reached its maximum position
capacity

##### [§](#parameters-1){.doc-anchor}Parameters

- `strategy_type` - The name or identifier of the strategy that is at
  capacity
- `max_positions` - The maximum number of positions the strategy can
  hold

##### [§](#returns-1){.doc-anchor}Returns

A `PositionError::StrategyError` variant with StrategyFull details
:::

::: {#method.invalid_position_size .section .method}
[Source](../../../src/optionstratlib/error/position.rs.html#452-457){.src
.rightside}

#### pub fn [invalid_position_size](#method.invalid_position_size){.fn}(size: [f64](https://doc.rust-lang.org/1.86.0/std/primitive.f64.html){.primitive}, reason: &[str](https://doc.rust-lang.org/1.86.0/std/primitive.str.html){.primitive}) -\> Self {#pub-fn-invalid_position_sizesize-f64-reason-str---self .code-header}
:::

::: docblock
Creates an error for invalid position size values

##### [§](#parameters-2){.doc-anchor}Parameters

- `size` - The invalid position size value
- `reason` - A description of why the size is invalid

##### [§](#returns-2){.doc-anchor}Returns

A `PositionError::ValidationError` variant with InvalidSize details
:::

::: {#method.invalid_position_type .section .method}
[Source](../../../src/optionstratlib/error/position.rs.html#469-474){.src
.rightside}

#### pub fn [invalid_position_type](#method.invalid_position_type){.fn}(position_side: [Side](../../model/types/enum.Side.html "enum optionstratlib::model::types::Side"){.enum}, reason: [String](https://doc.rust-lang.org/1.86.0/alloc/string/struct.String.html "struct alloc::string::String"){.struct}) -\> Self {#pub-fn-invalid_position_typeposition_side-side-reason-string---self .code-header}
:::

::: docblock
Creates an error for incompatible position side/direction

##### [§](#parameters-3){.doc-anchor}Parameters

- `position_side` - The position side (Long or Short) that is
  incompatible
- `reason` - A description of why the position side is incompatible

##### [§](#returns-3){.doc-anchor}Returns

A `PositionError::ValidationError` variant with IncompatibleSide details
:::

::: {#method.invalid_position_style .section .method}
[Source](../../../src/optionstratlib/error/position.rs.html#486-491){.src
.rightside}

#### pub fn [invalid_position_style](#method.invalid_position_style){.fn}(style: [OptionStyle](../../model/types/enum.OptionStyle.html "enum optionstratlib::model::types::OptionStyle"){.enum}, reason: [String](https://doc.rust-lang.org/1.86.0/alloc/string/struct.String.html "struct alloc::string::String"){.struct}) -\> Self {#pub-fn-invalid_position_stylestyle-optionstyle-reason-string---self .code-header}
:::

::: docblock
Creates an error for incompatible option style

##### [§](#parameters-4){.doc-anchor}Parameters

- `style` - The option style (Call or Put) that is incompatible
- `reason` - A description of why the option style is incompatible

##### [§](#returns-4){.doc-anchor}Returns

A `PositionError::ValidationError` variant with IncompatibleStyle
details
:::

::: {#method.invalid_position .section .method}
[Source](../../../src/optionstratlib/error/position.rs.html#502-506){.src
.rightside}

#### pub fn [invalid_position](#method.invalid_position){.fn}(reason: &[str](https://doc.rust-lang.org/1.86.0/std/primitive.str.html){.primitive}) -\> Self {#pub-fn-invalid_positionreason-str---self .code-header}
:::

::: docblock
Creates a generic invalid position error

##### [§](#parameters-5){.doc-anchor}Parameters

- `reason` - A description of why the position is invalid

##### [§](#returns-5){.doc-anchor}Returns

A `PositionError::ValidationError` variant with InvalidPosition details
:::
:::::::::::::::
:::::::::::::::::::

## Trait Implementations[§](#trait-implementations){.anchor} {#trait-implementations .section-header}

::::::::::::::::::::::::::::::::::::::::::::::::::::::: {#trait-implementations-list}
::: {#impl-Debug-for-PositionError .section .impl}
[Source](../../../src/optionstratlib/error/position.rs.html#94){.src
.rightside}[§](#impl-Debug-for-PositionError){.anchor}

### impl [Debug](https://doc.rust-lang.org/1.86.0/core/fmt/trait.Debug.html "trait core::fmt::Debug"){.trait} for [PositionError](enum.PositionError.html "enum optionstratlib::error::position::PositionError"){.enum} {#impl-debug-for-positionerror .code-header}
:::

::::: impl-items
::: {#method.fmt .section .method .trait-impl}
[Source](../../../src/optionstratlib/error/position.rs.html#94){.src
.rightside}[§](#method.fmt){.anchor}

#### fn [fmt](https://doc.rust-lang.org/1.86.0/core/fmt/trait.Debug.html#tymethod.fmt){.fn}(&self, f: &mut [Formatter](https://doc.rust-lang.org/1.86.0/core/fmt/struct.Formatter.html "struct core::fmt::Formatter"){.struct}\<\'\_\>) -\> [Result](https://doc.rust-lang.org/1.86.0/core/fmt/type.Result.html "type core::fmt::Result"){.type} {#fn-fmtself-f-mut-formatter_---result .code-header}
:::

::: docblock
Formats the value using the given formatter. [Read
more](https://doc.rust-lang.org/1.86.0/core/fmt/trait.Debug.html#tymethod.fmt)
:::
:::::

::: {#impl-Display-for-PositionError .section .impl}
[Source](../../../src/optionstratlib/error/position.rs.html#293-303){.src
.rightside}[§](#impl-Display-for-PositionError){.anchor}

### impl [Display](https://doc.rust-lang.org/1.86.0/core/fmt/trait.Display.html "trait core::fmt::Display"){.trait} for [PositionError](enum.PositionError.html "enum optionstratlib::error::position::PositionError"){.enum} {#impl-display-for-positionerror .code-header}
:::

::::: impl-items
::: {#method.fmt-1 .section .method .trait-impl}
[Source](../../../src/optionstratlib/error/position.rs.html#294-302){.src
.rightside}[§](#method.fmt-1){.anchor}

#### fn [fmt](https://doc.rust-lang.org/1.86.0/core/fmt/trait.Display.html#tymethod.fmt){.fn}(&self, f: &mut [Formatter](https://doc.rust-lang.org/1.86.0/core/fmt/struct.Formatter.html "struct core::fmt::Formatter"){.struct}\<\'\_\>) -\> [Result](https://doc.rust-lang.org/1.86.0/core/fmt/type.Result.html "type core::fmt::Result"){.type} {#fn-fmtself-f-mut-formatter_---result-1 .code-header}
:::

::: docblock
Formats the value using the given formatter. [Read
more](https://doc.rust-lang.org/1.86.0/core/fmt/trait.Display.html#tymethod.fmt)
:::
:::::

::: {#impl-Error-for-PositionError .section .impl}
[Source](../../../src/optionstratlib/error/position.rs.html#395){.src
.rightside}[§](#impl-Error-for-PositionError){.anchor}

### impl [Error](https://doc.rust-lang.org/1.86.0/core/error/trait.Error.html "trait core::error::Error"){.trait} for [PositionError](enum.PositionError.html "enum optionstratlib::error::position::PositionError"){.enum} {#impl-error-for-positionerror .code-header}
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

::: {#impl-From%3C%26str%3E-for-PositionError .section .impl}
[Source](../../../src/optionstratlib/error/position.rs.html#517-523){.src
.rightside}[§](#impl-From%3C%26str%3E-for-PositionError){.anchor}

### impl [From](https://doc.rust-lang.org/1.86.0/core/convert/trait.From.html "trait core::convert::From"){.trait}\<&[str](https://doc.rust-lang.org/1.86.0/std/primitive.str.html){.primitive}\> for [PositionError](enum.PositionError.html "enum optionstratlib::error::position::PositionError"){.enum} {#impl-fromstr-for-positionerror .code-header}
:::

::::: impl-items
::: {#method.from-2 .section .method .trait-impl}
[Source](../../../src/optionstratlib/error/position.rs.html#518-522){.src
.rightside}[§](#method.from-2){.anchor}

#### fn [from](https://doc.rust-lang.org/1.86.0/core/convert/trait.From.html#tymethod.from){.fn}(err: &[str](https://doc.rust-lang.org/1.86.0/std/primitive.str.html){.primitive}) -\> Self {#fn-fromerr-str---self .code-header}
:::

::: docblock
Converts to this type from the input type.
:::
:::::

::: {#impl-From%3CBox%3Cdyn+Error%3E%3E-for-PositionError .section .impl}
[Source](../../../src/optionstratlib/error/position.rs.html#509-515){.src
.rightside}[§](#impl-From%3CBox%3Cdyn+Error%3E%3E-for-PositionError){.anchor}

### impl [From](https://doc.rust-lang.org/1.86.0/core/convert/trait.From.html "trait core::convert::From"){.trait}\<[Box](https://doc.rust-lang.org/1.86.0/alloc/boxed/struct.Box.html "struct alloc::boxed::Box"){.struct}\<dyn [Error](https://doc.rust-lang.org/1.86.0/core/error/trait.Error.html "trait core::error::Error"){.trait}\>\> for [PositionError](enum.PositionError.html "enum optionstratlib::error::position::PositionError"){.enum} {#impl-fromboxdyn-error-for-positionerror .code-header}
:::

::::: impl-items
::: {#method.from-1 .section .method .trait-impl}
[Source](../../../src/optionstratlib/error/position.rs.html#510-514){.src
.rightside}[§](#method.from-1){.anchor}

#### fn [from](https://doc.rust-lang.org/1.86.0/core/convert/trait.From.html#tymethod.from){.fn}(err: [Box](https://doc.rust-lang.org/1.86.0/alloc/boxed/struct.Box.html "struct alloc::boxed::Box"){.struct}\<dyn [Error](https://doc.rust-lang.org/1.86.0/core/error/trait.Error.html "trait core::error::Error"){.trait}\>) -\> Self {#fn-fromerr-boxdyn-error---self .code-header}
:::

::: docblock
Converts to this type from the input type.
:::
:::::

:::: {#impl-From%3CPositionError%3E-for-CurveError .section .impl}
[Source](../../../src/optionstratlib/error/curves.rs.html#267-274){.src
.rightside}[§](#impl-From%3CPositionError%3E-for-CurveError){.anchor}

### impl [From](https://doc.rust-lang.org/1.86.0/core/convert/trait.From.html "trait core::convert::From"){.trait}\<[PositionError](enum.PositionError.html "enum optionstratlib::error::position::PositionError"){.enum}\> for [CurveError](../curves/enum.CurveError.html "enum optionstratlib::error::curves::CurveError"){.enum} {#impl-frompositionerror-for-curveerror .code-header}

::: docblock
Converts a `PositionError` into a `CurvesError` by mapping it to an
`OperationError` with the `InvalidParameters` variant.
:::
::::

::: docblock
This implementation ensures a smooth transition between error types when
a `PositionError` is encountered within a context that operates on the
`curves` module. The `InvalidParameters` variant is used to provide
detailed information about the failed operation and the reason for its
failure.

##### [§](#details){.doc-anchor}Details:

- The `operation` field is hardcoded as `"Position"` to indicate the
  context of the error (i.e., relating to position management).
- The `reason` field is derived from the `to_string` representation of
  the `PositionError`, ensuring a human-readable explanation.

##### [§](#example-integration){.doc-anchor}Example Integration:

1.  If a `PositionError` is encountered during curve calculations, this
    implementation converts it into a `CurvesError` for consistent error
    handling within the `curves` module.
2.  The generated `CurvesError` provides detailed diagnostic information
    about the reason for the failure, enabling effective debugging.

##### [§](#implementation-notes){.doc-anchor}Implementation Notes:

- This conversion leverages the `OperationErrorKind::InvalidParameters`
  variant to communicate that invalid parameters (or settings) were the
  root cause of failure.
- Use this implementation to handle interoperability between error types
  in modular design contexts.

##### [§](#example-use-case){.doc-anchor}Example Use Case:

This conversion is frequently used in scenarios where:

- A position-related error (e.g., from validation or limits) occurs
  during a curve operation.
- Such errors need to be mapped into the `CurvesError` domain to
  maintain consistent error handling across the library.

##### [§](#debugging){.doc-anchor}Debugging:

The resulting `CurvesError` will include contextual details, making it
straightforward to trace and debug the underlying issue.
:::

::::: impl-items
::: {#method.from .section .method .trait-impl}
[Source](../../../src/optionstratlib/error/curves.rs.html#268-273){.src
.rightside}[§](#method.from){.anchor}

#### fn [from](https://doc.rust-lang.org/1.86.0/core/convert/trait.From.html#tymethod.from){.fn}(err: [PositionError](enum.PositionError.html "enum optionstratlib::error::position::PositionError"){.enum}) -\> Self {#fn-fromerr-positionerror---self .code-header}
:::

::: docblock
Converts to this type from the input type.
:::
:::::

::: {#impl-From%3CPositionError%3E-for-InterpolationError .section .impl}
[Source](../../../src/optionstratlib/error/interpolation.rs.html#65-69){.src
.rightside}[§](#impl-From%3CPositionError%3E-for-InterpolationError){.anchor}

### impl [From](https://doc.rust-lang.org/1.86.0/core/convert/trait.From.html "trait core::convert::From"){.trait}\<[PositionError](enum.PositionError.html "enum optionstratlib::error::position::PositionError"){.enum}\> for [InterpolationError](../enum.InterpolationError.html "enum optionstratlib::error::InterpolationError"){.enum} {#impl-frompositionerror-for-interpolationerror .code-header}
:::

::::: impl-items
::: {#method.from-5 .section .method .trait-impl}
[Source](../../../src/optionstratlib/error/interpolation.rs.html#66-68){.src
.rightside}[§](#method.from-5){.anchor}

#### fn [from](https://doc.rust-lang.org/1.86.0/core/convert/trait.From.html#tymethod.from){.fn}(err: [PositionError](enum.PositionError.html "enum optionstratlib::error::position::PositionError"){.enum}) -\> Self {#fn-fromerr-positionerror---self-1 .code-header}
:::

::: docblock
Converts to this type from the input type.
:::
:::::

::: {#impl-From%3CPositionError%3E-for-StrategyError .section .impl}
[Source](../../../src/optionstratlib/error/strategies.rs.html#383-390){.src
.rightside}[§](#impl-From%3CPositionError%3E-for-StrategyError){.anchor}

### impl [From](https://doc.rust-lang.org/1.86.0/core/convert/trait.From.html "trait core::convert::From"){.trait}\<[PositionError](enum.PositionError.html "enum optionstratlib::error::position::PositionError"){.enum}\> for [StrategyError](../strategies/enum.StrategyError.html "enum optionstratlib::error::strategies::StrategyError"){.enum} {#impl-frompositionerror-for-strategyerror .code-header}
:::

::::: impl-items
::: {#method.from-4 .section .method .trait-impl}
[Source](../../../src/optionstratlib/error/strategies.rs.html#384-389){.src
.rightside}[§](#method.from-4){.anchor}

#### fn [from](https://doc.rust-lang.org/1.86.0/core/convert/trait.From.html#tymethod.from){.fn}(err: [PositionError](enum.PositionError.html "enum optionstratlib::error::position::PositionError"){.enum}) -\> Self {#fn-fromerr-positionerror---self-2 .code-header}
:::

::: docblock
Converts to this type from the input type.
:::
:::::

:::: {#impl-From%3CPositionError%3E-for-SurfaceError .section .impl}
[Source](../../../src/optionstratlib/error/surfaces.rs.html#172-179){.src
.rightside}[§](#impl-From%3CPositionError%3E-for-SurfaceError){.anchor}

### impl [From](https://doc.rust-lang.org/1.86.0/core/convert/trait.From.html "trait core::convert::From"){.trait}\<[PositionError](enum.PositionError.html "enum optionstratlib::error::position::PositionError"){.enum}\> for [SurfaceError](../enum.SurfaceError.html "enum optionstratlib::error::SurfaceError"){.enum} {#impl-frompositionerror-for-surfaceerror .code-header}

::: docblock
Converts a `PositionError` into a `SurfaceError` by mapping it to an
`OperationError` with the `InvalidParameters` variant.
:::
::::

::: docblock
This implementation ensures a smooth transition between error types when
a `PositionError` is encountered within a context that operates on the
`curves` module. The `InvalidParameters` variant is used to provide
detailed information about the failed operation and the reason for its
failure.

##### [§](#details-1){.doc-anchor}Details:

- The `operation` field is hardcoded as `"Position"` to indicate the
  context of the error (i.e., relating to position management).
- The `reason` field is derived from the `to_string` representation of
  the `PositionError`, ensuring a human-readable explanation.

##### [§](#example-integration-1){.doc-anchor}Example Integration:

1.  If a `PositionError` is encountered during curve calculations, this
    implementation converts it into a `SurfaceError` for consistent
    error handling within the `curves` module.
2.  The generated `SurfaceError` provides detailed diagnostic
    information about the reason for the failure, enabling effective
    debugging.

##### [§](#implementation-notes-1){.doc-anchor}Implementation Notes:

- This conversion leverages the `OperationErrorKind::InvalidParameters`
  variant to communicate that invalid parameters (or settings) were the
  root cause of failure.
- Use this implementation to handle interoperability between error types
  in modular design contexts.

##### [§](#example-use-case-1){.doc-anchor}Example Use Case:

This conversion is frequently used in scenarios where:

- A position-related error (e.g., from validation or limits) occurs
  during a curve operation.
- Such errors need to be mapped into the `SurfaceError` domain to
  maintain consistent error handling across the library.

##### [§](#debugging-1){.doc-anchor}Debugging:

The resulting `SurfaceError` will include contextual details, making it
straightforward to trace and debug the underlying issue.
:::

::::: impl-items
::: {#method.from-6 .section .method .trait-impl}
[Source](../../../src/optionstratlib/error/surfaces.rs.html#173-178){.src
.rightside}[§](#method.from-6){.anchor}

#### fn [from](https://doc.rust-lang.org/1.86.0/core/convert/trait.From.html#tymethod.from){.fn}(err: [PositionError](enum.PositionError.html "enum optionstratlib::error::position::PositionError"){.enum}) -\> Self {#fn-fromerr-positionerror---self-3 .code-header}
:::

::: docblock
Converts to this type from the input type.
:::
:::::

::: {#impl-From%3CString%3E-for-PositionError .section .impl}
[Source](../../../src/optionstratlib/error/position.rs.html#525-531){.src
.rightside}[§](#impl-From%3CString%3E-for-PositionError){.anchor}

### impl [From](https://doc.rust-lang.org/1.86.0/core/convert/trait.From.html "trait core::convert::From"){.trait}\<[String](https://doc.rust-lang.org/1.86.0/alloc/string/struct.String.html "struct alloc::string::String"){.struct}\> for [PositionError](enum.PositionError.html "enum optionstratlib::error::position::PositionError"){.enum} {#impl-fromstring-for-positionerror .code-header}
:::

::::: impl-items
::: {#method.from-3 .section .method .trait-impl}
[Source](../../../src/optionstratlib/error/position.rs.html#526-530){.src
.rightside}[§](#method.from-3){.anchor}

#### fn [from](https://doc.rust-lang.org/1.86.0/core/convert/trait.From.html#tymethod.from){.fn}(err: [String](https://doc.rust-lang.org/1.86.0/alloc/string/struct.String.html "struct alloc::string::String"){.struct}) -\> Self {#fn-fromerr-string---self .code-header}
:::

::: docblock
Converts to this type from the input type.
:::
:::::
:::::::::::::::::::::::::::::::::::::::::::::::::::::::

## Auto Trait Implementations[§](#synthetic-implementations){.anchor} {#synthetic-implementations .section-header}

::::::::: {#synthetic-implementations-list}
::: {#impl-Freeze-for-PositionError .section .impl}
[§](#impl-Freeze-for-PositionError){.anchor}

### impl [Freeze](https://doc.rust-lang.org/1.86.0/core/marker/trait.Freeze.html "trait core::marker::Freeze"){.trait} for [PositionError](enum.PositionError.html "enum optionstratlib::error::position::PositionError"){.enum} {#impl-freeze-for-positionerror .code-header}
:::

::: {#impl-RefUnwindSafe-for-PositionError .section .impl}
[§](#impl-RefUnwindSafe-for-PositionError){.anchor}

### impl [RefUnwindSafe](https://doc.rust-lang.org/1.86.0/core/panic/unwind_safe/trait.RefUnwindSafe.html "trait core::panic::unwind_safe::RefUnwindSafe"){.trait} for [PositionError](enum.PositionError.html "enum optionstratlib::error::position::PositionError"){.enum} {#impl-refunwindsafe-for-positionerror .code-header}
:::

::: {#impl-Send-for-PositionError .section .impl}
[§](#impl-Send-for-PositionError){.anchor}

### impl [Send](https://doc.rust-lang.org/1.86.0/core/marker/trait.Send.html "trait core::marker::Send"){.trait} for [PositionError](enum.PositionError.html "enum optionstratlib::error::position::PositionError"){.enum} {#impl-send-for-positionerror .code-header}
:::

::: {#impl-Sync-for-PositionError .section .impl}
[§](#impl-Sync-for-PositionError){.anchor}

### impl [Sync](https://doc.rust-lang.org/1.86.0/core/marker/trait.Sync.html "trait core::marker::Sync"){.trait} for [PositionError](enum.PositionError.html "enum optionstratlib::error::position::PositionError"){.enum} {#impl-sync-for-positionerror .code-header}
:::

::: {#impl-Unpin-for-PositionError .section .impl}
[§](#impl-Unpin-for-PositionError){.anchor}

### impl [Unpin](https://doc.rust-lang.org/1.86.0/core/marker/trait.Unpin.html "trait core::marker::Unpin"){.trait} for [PositionError](enum.PositionError.html "enum optionstratlib::error::position::PositionError"){.enum} {#impl-unpin-for-positionerror .code-header}
:::

::: {#impl-UnwindSafe-for-PositionError .section .impl}
[§](#impl-UnwindSafe-for-PositionError){.anchor}

### impl [UnwindSafe](https://doc.rust-lang.org/1.86.0/core/panic/unwind_safe/trait.UnwindSafe.html "trait core::panic::unwind_safe::UnwindSafe"){.trait} for [PositionError](enum.PositionError.html "enum optionstratlib::error::position::PositionError"){.enum} {#impl-unwindsafe-for-positionerror .code-header}
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
:::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
