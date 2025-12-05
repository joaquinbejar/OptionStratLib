::::::: width-limiter
:::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../../index.html)::[error](../index.html)
:::

# Module greeks Copy item path

[[Source](../../../src/optionstratlib/error/greeks.rs.html#7-693){.src}
]{.sub-heading}
::::

Expand description

::: docblock
#### [§](#greeks-errors-greekserror){.doc-anchor}Greeks Errors (`GreeksError`)

Handles:

- Greeks calculations
- Mathematical validation
- Input parameter validation
- Numerical computations

## [§](#greeks-error-module){.doc-anchor}Greeks Error Module

This module provides error handling for Greek calculations and equations
in option pricing. It defines error types for various mathematical
calculations and validations used in financial derivatives analysis.

### [§](#error-types){.doc-anchor}Error Types

#### [§](#greeks-error-greekserror){.doc-anchor}Greeks Error (`GreeksError`)

Main error enum that encompasses:

- Calculation errors in Greek values
- Input validation errors
- Mathematical operation errors
- Boundary condition errors

#### [§](#mathematical-error-matherrorkind){.doc-anchor}Mathematical Error (`MathErrorKind`)

Handles specific mathematical errors:

- Division by zero
- Overflow conditions
- Invalid domain errors
- Convergence failures

#### [§](#input-validation-error-inputerrorkind){.doc-anchor}Input Validation Error (`InputErrorKind`)

Manages validation of input parameters:

- Invalid volatility values
- Invalid time values
- Invalid price values
- Invalid rate values
:::

## Enums[§](#enums){.anchor} {#enums .section-header}

[CalculationErrorKind](enum.CalculationErrorKind.html "enum optionstratlib::error::greeks::CalculationErrorKind"){.enum}
:   Represents specific error types that can occur during financial
    derivative calculations.

[GreeksError](enum.GreeksError.html "enum optionstratlib::error::greeks::GreeksError"){.enum}
:   Represents errors that can occur during options Greek calculations.

[InputErrorKind](enum.InputErrorKind.html "enum optionstratlib::error::greeks::InputErrorKind"){.enum}
:   Represents different types of input validation errors that can occur
    during financial calculations.

[MathErrorKind](enum.MathErrorKind.html "enum optionstratlib::error::greeks::MathErrorKind"){.enum}
:   Represents various types of mathematical errors that can occur
    during calculations.

## Type Aliases[§](#types){.anchor} {#types .section-header}

[GreeksResult](type.GreeksResult.html "type optionstratlib::error::greeks::GreeksResult"){.type}
:   Type alias for Results returned from Greek calculation functions.
::::::
:::::::
