:::::::: width-limiter
::::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../../index.html)::[error](../index.html)
:::

# Module strategies Copy item path

[[Source](../../../src/optionstratlib/error/strategies.rs.html#7-533){.src}
]{.sub-heading}
::::

Expand description

:::: docblock
#### [§](#strategy-errors-strategyerror){.doc-anchor}Strategy Errors (`StrategyError`)

Covers:

- Price calculations
- Break-even analysis
- Profit/Loss calculations
- Operation validation

## [§](#strategy-error-module){.doc-anchor}Strategy Error Module

This module provides error handling for option trading strategies and
their operations. It defines error types for strategy calculations,
validations, and operations, with integration with the probability
analysis system.

### [§](#error-types){.doc-anchor}Error Types

#### [§](#strategy-error-strategyerror){.doc-anchor}Strategy Error (`StrategyError`)

The main error enum with four categories:

- `PriceError` - For price calculation failures
- `BreakEvenError` - For break-even point calculation errors
- `ProfitLossError` - For profit/loss calculation failures
- `OperationError` - For strategy operation errors

#### [§](#price-errors-priceerrorkind){.doc-anchor}Price Errors (`PriceErrorKind`)

Specific errors for price-related operations:

- Invalid underlying prices
- Invalid price ranges with start and end points

#### [§](#break-even-errors-breakevenerrorkind){.doc-anchor}Break-Even Errors (`BreakEvenErrorKind`)

Handles break-even point calculations:

- Calculation failures
- Missing break-even points

#### [§](#profitloss-errors-profitlosserrorkind){.doc-anchor}Profit/Loss Errors (`ProfitLossErrorKind`)

Manages profit and loss calculations:

- Maximum profit calculation errors
- Maximum loss calculation errors
- Profit range calculation errors

### [§](#integration-with-probability-analysis){.doc-anchor}Integration with Probability Analysis

Implements conversion from `StrategyError` to `ProbabilityError` for
seamless error handling between strategy and probability calculations.

### [§](#usage-example){.doc-anchor}Usage Example

::: example-wrap
``` {.rust .rust-example-rendered}

use optionstratlib::error::strategies::{StrategyError, StrategyResult};

fn validate_strategy_operation(operation: &str, strategy: &str) -> StrategyResult<()> {
    if !is_supported_operation(operation) {
        return Err(StrategyError::operation_not_supported(operation, strategy));
    }
    Ok(())
}

fn is_supported_operation(p0: &str) -> bool  {
    false
}
```
:::

### [§](#helper-methods){.doc-anchor}Helper Methods

The module provides convenient methods for creating common errors:

- `operation_not_supported` - Creates an error for unsupported
  operations
- `invalid_parameters` - Creates an error for invalid operation
  parameters

### [§](#type-alias){.doc-anchor}Type Alias

Provides `StrategyResult<T>` for convenient error handling in strategy
operations.
::::

## Enums[§](#enums){.anchor} {#enums .section-header}

[BreakEvenErrorKind](enum.BreakEvenErrorKind.html "enum optionstratlib::error::strategies::BreakEvenErrorKind"){.enum}
:   Represents the type of errors that can occur during break-even point
    calculations.

[PriceErrorKind](enum.PriceErrorKind.html "enum optionstratlib::error::strategies::PriceErrorKind"){.enum}
:   Represents different types of errors that can occur during
    price-related operations.

[ProfitLossErrorKind](enum.ProfitLossErrorKind.html "enum optionstratlib::error::strategies::ProfitLossErrorKind"){.enum}
:   Represents error types that can occur during profit and loss
    calculations.

[StrategyError](enum.StrategyError.html "enum optionstratlib::error::strategies::StrategyError"){.enum}
:   Represents the different types of errors that can occur in options
    trading strategies.

## Type Aliases[§](#types){.anchor} {#types .section-header}

[StrategyResult](type.StrategyResult.html "type optionstratlib::error::strategies::StrategyResult"){.type}
:   A specialized result type for strategy operations.
:::::::
::::::::
