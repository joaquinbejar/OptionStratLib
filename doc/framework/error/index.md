::::::::: width-limiter
:::::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../index.html)
:::

# Module error Copy item path

[[Source](../../src/optionstratlib/error/mod.rs.html#6-242){.src}
]{.sub-heading}
::::

Expand description

::::: docblock
- `error` - Error types and handling functionality for the library.

Defines the error hierarchy used throughout the library, providing
detailed error types for different categories of failures including
validation errors, calculation errors, and input/output errors.

## [§](#error-module){.doc-anchor}Error Module

This module provides a comprehensive error handling system for options
trading and financial calculations. It defines specialized error types
for different aspects of the library, including options trading, pricing
calculations, statistical analysis, and data management.

### [§](#core-modules-overview){.doc-anchor}Core Modules Overview

#### [§](#options-and-pricing){.doc-anchor}Options and Pricing

- `OptionsError` - Core errors for option operations and validations
- `GreeksError` - Errors in Greeks calculations (delta, gamma, etc.)
- `VolatilityError` - Errors in volatility calculations including
  implied volatility

#### [§](#trading-and-analysis){.doc-anchor}Trading and Analysis

- `ChainError` - Option chain operations and data management
- `PositionError` - Position management and trading operations
- `StrategyError` - Trading strategy validation and execution
- `ProbabilityError` - Statistical analysis and probability calculations

#### [§](#mathematical-and-data){.doc-anchor}Mathematical and Data

- `CurveError` - Curve fitting and mathematical operations
- `DecimalError` - Decimal number handling and precision
- `InterpolationError` - Errors in data interpolation operations
- `MetricsError` - Performance and risk metrics calculation errors
- `SurfaceError` - Volatility and pricing surface construction errors

### [§](#usage-example){.doc-anchor}Usage Example

::: example-wrap
``` {.rust .rust-example-rendered}
use optionstratlib::error::{OptionsError, GreeksError, ChainError};

// Options error handling
fn calculate_option_price() -> Result<f64, OptionsError> {
    // Implementation
    Ok(0.0)
}

// Greeks calculation error handling
fn calculate_delta() -> Result<f64, GreeksError> {
    // Implementation
    Ok(0.0)
}

// Chain operation error handling
fn process_option_chain() -> Result<(), ChainError> {
    // Implementation
    Ok(())
}
```
:::

### [§](#error-design-principles){.doc-anchor}Error Design Principles

- All error types implement standard traits (`Error`, `Display`,
  `Debug`)
- Structured error hierarchies for precise error handling
- Detailed error messages for debugging
- Clean error propagation through type conversions
- Context preservation in error chains

### [§](#type-aliases){.doc-anchor}Type Aliases

- `OptionsResult<T>` - Specialized result type for options operations
- `DecimalResult<T>` - Specialized result type for decimal calculations

### [§](#module-structure){.doc-anchor}Module Structure

::: example-wrap
``` language-text
error/
├── chains.rs       - Option chain errors
├── common.rs       - Shared error types
├── curves.rs       - Mathematical curve errors
├── decimal.rs      - Decimal computation errors
├── greeks.rs       - Greeks calculation errors
├── interpolation.rs - Interpolation errors
├── metrics.rs      - Performance metrics errors
├── options.rs      - Core options errors
├── position.rs     - Position management errors
├── probability.rs  - Statistical analysis errors
├── strategies.rs   - Trading strategy errors
├── surfaces.rs     - Surface construction errors
└── volatility.rs   - Volatility calculation errors
```
:::
:::::

## Re-exports[§](#reexports){.anchor} {#reexports .section-header}

`pub use chains::`[`ChainError`](chains/enum.ChainError.html "enum optionstratlib::error::chains::ChainError"){.enum}`;`

`pub use curves::`[`CurveError`](curves/enum.CurveError.html "enum optionstratlib::error::curves::CurveError"){.enum}`;`

`pub use decimal::`[`DecimalError`](decimal/enum.DecimalError.html "enum optionstratlib::error::decimal::DecimalError"){.enum}`;`

`pub use decimal::`[`DecimalResult`](decimal/type.DecimalResult.html "type optionstratlib::error::decimal::DecimalResult"){.type}`;`

`pub use greeks::`[`GreeksError`](greeks/enum.GreeksError.html "enum optionstratlib::error::greeks::GreeksError"){.enum}`;`

`pub use position::`[`PositionError`](position/enum.PositionError.html "enum optionstratlib::error::position::PositionError"){.enum}`;`

`pub use pricing::`[`PricingError`](pricing/enum.PricingError.html "enum optionstratlib::error::pricing::PricingError"){.enum}`;`

`pub use pricing::`[`PricingResult`](pricing/type.PricingResult.html "type optionstratlib::error::pricing::PricingResult"){.type}`;`

`pub use probability::`[`ProbabilityError`](probability/enum.ProbabilityError.html "enum optionstratlib::error::probability::ProbabilityError"){.enum}`;`

`pub use simulation::`[`SimulationError`](simulation/enum.SimulationError.html "enum optionstratlib::error::simulation::SimulationError"){.enum}`;`

`pub use simulation::`[`SimulationResult`](simulation/type.SimulationResult.html "type optionstratlib::error::simulation::SimulationResult"){.type}`;`

`pub use strategies::`[`StrategyError`](strategies/enum.StrategyError.html "enum optionstratlib::error::strategies::StrategyError"){.enum}`;`

`pub use unified::`[`Error`](unified/enum.Error.html "enum optionstratlib::error::unified::Error"){.enum}`;`

## Modules[§](#modules){.anchor} {#modules .section-header}

[chains](chains/index.html "mod optionstratlib::error::chains"){.mod}
:   Chain Errors (`ChainError`)

[curves](curves/index.html "mod optionstratlib::error::curves"){.mod}
:   Curve Errors (`CurveError`)

[decimal](decimal/index.html "mod optionstratlib::error::decimal"){.mod}
:   Decimal Errors (`DecimalError`)

[greeks](greeks/index.html "mod optionstratlib::error::greeks"){.mod}
:   Greeks Errors (`GreeksError`)

[position](position/index.html "mod optionstratlib::error::position"){.mod}
:   Position Errors (`PositionError`)

[pricing](pricing/index.html "mod optionstratlib::error::pricing"){.mod}
:   Pricing Errors (`PricingError`)

[probability](probability/index.html "mod optionstratlib::error::probability"){.mod}
:   Probability Errors (`ProbabilityError`)

[simulation](simulation/index.html "mod optionstratlib::error::simulation"){.mod}
:   Simulation Errors (`SimulationError`)

[strategies](strategies/index.html "mod optionstratlib::error::strategies"){.mod}
:   Strategy Errors (`StrategyError`)

[unified](unified/index.html "mod optionstratlib::error::unified"){.mod}
:   Unified Error Type

## Structs[§](#structs){.anchor} {#structs .section-header}

[TransactionError](struct.TransactionError.html "struct optionstratlib::error::TransactionError"){.struct}
:   Transaction Error

## Enums[§](#enums){.anchor} {#enums .section-header}

[GraphError](enum.GraphError.html "enum optionstratlib::error::GraphError"){.enum}
:   Represents errors that can occur during graph generation and
    rendering operations.

[InterpolationError](enum.InterpolationError.html "enum optionstratlib::error::InterpolationError"){.enum}
:   Represents errors that can occur during different interpolation
    operations.

[MetricsError](enum.MetricsError.html "enum optionstratlib::error::MetricsError"){.enum}
:   Error types specifically related to financial and statistical
    metrics calculations.

[OhlcvError](enum.OhlcvError.html "enum optionstratlib::error::OhlcvError"){.enum}
:   Error type for OHLCV operations

[OperationErrorKind](enum.OperationErrorKind.html "enum optionstratlib::error::OperationErrorKind"){.enum}
:   Represents the types of errors that can occur during operations
    related to trading strategies or other processes.

[OptionsError](enum.OptionsError.html "enum optionstratlib::error::OptionsError"){.enum}
:   Custom errors that can occur during Options operations

[SurfaceError](enum.SurfaceError.html "enum optionstratlib::error::SurfaceError"){.enum}
:   Error variants that can occur when working with surface-related
    operations.

[VolatilityError](enum.VolatilityError.html "enum optionstratlib::error::VolatilityError"){.enum}
:   Represents errors that can occur during volatility-related
    calculations.

## Type Aliases[§](#types){.anchor} {#types .section-header}

[OptionsResult](type.OptionsResult.html "type optionstratlib::error::OptionsResult"){.type}
:   A specialized result type for operations related to Options
    calculations and processing.
::::::::
:::::::::
