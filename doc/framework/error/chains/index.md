:::::::: width-limiter
::::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../../index.html)::[error](../index.html)
:::

# Module chainsCopy item path

[[Source](../../../src/optionstratlib/error/chains.rs.html#7-935){.src}
]{.sub-heading}
::::

Expand description

:::: docblock
#### [§](#chain-errors-chainerror){.doc-anchor}Chain Errors (`ChainError`)

Handles:

- Option data validation
- Chain construction
- File operations (CSV/JSON)
- Strategy validation

## [§](#chain-error-module){.doc-anchor}Chain Error Module

This module provides error handling for operations related to option
chains and their data. It defines a comprehensive error hierarchy to
handle various failure scenarios in option chain operations, data
validation, and file handling.

### [§](#error-types){.doc-anchor}Error Types

- `ChainError` - The main error enum that encompasses all possible
  chain-related errors
- `OptionDataErrorKind` - Specific errors related to option data
  validation
- `ChainBuildErrorKind` - Errors that can occur during chain
  construction
- `FileErrorKind` - File operation related errors
- `StrategyErrorKind` - Strategy-specific validation errors

### [§](#usage-example){.doc-anchor}Usage Example

::: example-wrap
``` {.rust .rust-example-rendered}
use optionstratlib::error::chains::ChainError;

fn validate_strike_price(strike: f64) -> Result<(), ChainError> {
    if strike <= 0.0 {
        return Err(ChainError::invalid_strike(
            strike,
            "Strike price must be positive"
        ));
    }
    Ok(())
}
```
:::

### [§](#error-creation-helpers){.doc-anchor}Error Creation Helpers

The module provides several helper methods for creating common errors:

- `invalid_strike` - Creates an error for invalid strike prices
- `invalid_volatility` - Creates an error for invalid volatility values
- `invalid_prices` - Creates an error for invalid bid/ask prices
- `invalid_legs` - Creates an error for invalid strategy legs
- `invalid_parameters` - Creates an error for invalid chain building
  parameters

### [§](#conversions){.doc-anchor}Conversions

The module implements various conversion traits:

- `From<io::Error>` - Converts IO errors to chain errors
- `From<String>` - Converts string messages to price calculation errors

All error types implement `std::error::Error` and `std::fmt::Display`
for proper error handling and formatting.
::::

## Enums[§](#enums){.anchor} {#enums .section-header}

[ChainBuildErrorKind](enum.ChainBuildErrorKind.html "enum optionstratlib::error::chains::ChainBuildErrorKind"){.enum}
:   Enum representing specific errors that can occur during option chain
    building processes.

[ChainError](enum.ChainError.html "enum optionstratlib::error::chains::ChainError"){.enum}
:   ChainError

[FileErrorKind](enum.FileErrorKind.html "enum optionstratlib::error::chains::FileErrorKind"){.enum}
:   Enum representing errors related to file operations in the system.

[OptionDataErrorKind](enum.OptionDataErrorKind.html "enum optionstratlib::error::chains::OptionDataErrorKind"){.enum}
:   Represents specific error types related to option data validation
    and calculations.

[StrategyErrorKind](enum.StrategyErrorKind.html "enum optionstratlib::error::chains::StrategyErrorKind"){.enum}
:   Enum representing specific error types that can occur in options
    trading strategies.
:::::::
::::::::
