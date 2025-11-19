:::::::: width-limiter
::::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../../index.html)::[error](../index.html)
:::

# Module probability Copy item path

[[Source](../../../src/optionstratlib/error/probability.rs.html#7-908){.src}
]{.sub-heading}
::::

Expand description

:::: docblock
#### [§](#probability-errors-probabilityerror){.doc-anchor}Probability Errors (`ProbabilityError`)

Manages:

- Statistical calculations
- Range analysis
- Probability distributions
- Market scenarios

## [§](#probability-error-module){.doc-anchor}Probability Error Module

This module provides error handling for probability analysis and
calculations in option trading. It defines a comprehensive error system
to handle various scenarios in probability calculations, profit/loss
analysis, and option pricing.

### [§](#main-error-types){.doc-anchor}Main Error Types

#### [§](#probability-error-probabilityerror){.doc-anchor}Probability Error (`ProbabilityError`)

Main error enum with four variants:

- `CalculationError` - For probability calculation failures
- `RangeError` - For profit/loss range analysis errors
- `ExpirationError` - For expiration date related errors
- `PriceError` - For price calculation and validation errors

#### [§](#calculation-errors-probabilitycalculationerrorkind){.doc-anchor}Calculation Errors (`ProbabilityCalculationErrorKind`)

Handles specific calculation failures:

- Invalid probability values
- Expected value calculation errors
- Volatility adjustment errors
- Price trend errors

#### [§](#range-errors-profitlossrangeerrorkind){.doc-anchor}Range Errors (`ProfitLossRangeErrorKind`)

Manages profit/loss analysis errors:

- Invalid profit ranges
- Invalid loss ranges
- Break-even point errors

#### [§](#price-errors-priceerrorkind){.doc-anchor}Price Errors (`PriceErrorKind`)

Handles pricing-related errors:

- Invalid underlying prices
- Invalid price ranges

### [§](#usage-example){.doc-anchor}Usage Example

::: example-wrap
``` {.rust .rust-example-rendered}
use optionstratlib::error::probability::{ProbabilityError, ProbabilityResult};

fn calculate_probability(value: f64) -> ProbabilityResult<f64> {
    if value < 0.0 || value > 1.0 {
        return Err(ProbabilityError::invalid_probability(
            value,
            "Probability must be between 0 and 1"
        ));
    }
    Ok(value)
}
```
:::

### [§](#error-creation-helpers){.doc-anchor}Error Creation Helpers

The module provides helper methods for creating common errors:

- `invalid_probability` - Creates an error for invalid probability
  values
- `invalid_profit_range` - Creates an error for invalid profit ranges
- `invalid_expiration` - Creates an error for invalid expiration dates

### [§](#type-conversions){.doc-anchor}Type Conversions

Implements conversions from:

- `String` to `ProbabilityError`
- `&str` to `ProbabilityError`

A type alias `ProbabilityResult<T>` is provided for convenience when
working with Results that may contain probability errors.
::::

## Enums[§](#enums){.anchor} {#enums .section-header}

[ExpirationErrorKind](enum.ExpirationErrorKind.html "enum optionstratlib::error::probability::ExpirationErrorKind"){.enum}
:   Enum representing errors related to expiration dates and interest
    rates in options calculations.

[PriceErrorKind](enum.PriceErrorKind.html "enum optionstratlib::error::probability::PriceErrorKind"){.enum}
:   Enum that represents various errors that can occur during price
    calculations and validations.

[ProbabilityCalculationErrorKind](enum.ProbabilityCalculationErrorKind.html "enum optionstratlib::error::probability::ProbabilityCalculationErrorKind"){.enum}
:   Error types that can occur during financial probability
    calculations.

[ProbabilityError](enum.ProbabilityError.html "enum optionstratlib::error::probability::ProbabilityError"){.enum}
:   Represents all possible errors that can occur during probability
    analysis calculations

[ProfitLossRangeErrorKind](enum.ProfitLossRangeErrorKind.html "enum optionstratlib::error::probability::ProfitLossRangeErrorKind"){.enum}
:   Enum representing errors that occur during profit and loss range
    calculations in options strategies.

## Type Aliases[§](#types){.anchor} {#types .section-header}

[ProbabilityResult](type.ProbabilityResult.html "type optionstratlib::error::probability::ProbabilityResult"){.type}
:   Convenient type alias for Results with ProbabilityError
:::::::
::::::::
