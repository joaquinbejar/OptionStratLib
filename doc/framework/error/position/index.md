:::::::: width-limiter
::::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../../index.html)::[error](../index.html)
:::

# Module positionCopy item path

[[Source](../../../src/optionstratlib/error/position.rs.html#7-767){.src}
]{.sub-heading}
::::

Expand description

:::: docblock
#### [§](#position-errors-positionerror){.doc-anchor}Position Errors (`PositionError`)

Manages:

- Position validation
- Strategy operations
- Position limits
- Option style/side compatibility

## [§](#position-error-module){.doc-anchor}Position Error Module

This module provides error handling for position-related operations in
option trading strategies. It defines error types to handle various
scenarios related to position validation, strategy operations, and
position limits.

### [§](#error-types){.doc-anchor}Error Types

#### [§](#position-error-positionerror){.doc-anchor}Position Error (`PositionError`)

The main error type with three variants:

- `StrategyError` - For strategy operation failures
- `ValidationError` - For position validation failures
- `LimitError` - For position limit violations

#### [§](#strategy-errors-strategyerrorkind){.doc-anchor}Strategy Errors (`StrategyErrorKind`)

Handles specific strategy-related errors:

- Unsupported operations
- Strategy capacity limits
- Invalid configurations

#### [§](#validation-errors-positionvalidationerrorkind){.doc-anchor}Validation Errors (`PositionValidationErrorKind`)

Handles position validation failures:

- Invalid position sizes
- Invalid prices
- Incompatible sides (Long/Short)
- Incompatible styles (Call/Put)
- General position invalidity

#### [§](#limit-errors-positionlimiterrorkind){.doc-anchor}Limit Errors (`PositionLimitErrorKind`)

Handles position limit violations:

- Maximum positions reached
- Maximum exposure reached

### [§](#usage-example){.doc-anchor}Usage Example

::: example-wrap
``` {.rust .rust-example-rendered}
use optionstratlib::error::position::PositionError;

fn validate_position_size(size: f64) -> Result<(), PositionError> {
    if size <= 0.0 {
        return Err(PositionError::invalid_position_size(
            size,
            "Position size must be positive"
        ));
    }
    Ok(())
}
```
:::

### [§](#helper-methods){.doc-anchor}Helper Methods

The module provides several helper methods for creating common errors:

- `unsupported_operation` - Creates an error for unsupported strategy
  operations
- `strategy_full` - Creates an error when strategy capacity is reached
- `invalid_position_size` - Creates an error for invalid position sizes
- `invalid_position_type` - Creates an error for incompatible position
  sides
- `invalid_position_style` - Creates an error for incompatible option
  styles
- `invalid_position` - Creates a general position validation error

All error types implement `std::error::Error` and `std::fmt::Display`
for proper error handling and formatting capabilities.
::::

## Enums[§](#enums){.anchor} {#enums .section-header}

[PositionError](enum.PositionError.html "enum optionstratlib::error::position::PositionError"){.enum}
:   Represents errors that can occur when managing positions in
    strategies

[PositionLimitErrorKind](enum.PositionLimitErrorKind.html "enum optionstratlib::error::position::PositionLimitErrorKind"){.enum}
:   Represents errors related to position limits in trading operations.

[PositionValidationErrorKind](enum.PositionValidationErrorKind.html "enum optionstratlib::error::position::PositionValidationErrorKind"){.enum}
:   Errors related to position validation

[StrategyErrorKind](enum.StrategyErrorKind.html "enum optionstratlib::error::position::StrategyErrorKind"){.enum}
:   Specific errors that can occur in strategy operations
:::::::
::::::::
