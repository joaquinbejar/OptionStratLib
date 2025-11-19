::::::: width-limiter
:::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../../index.html)::[simulation](../index.html)
:::

# Module steps Copy item path

[[Source](../../../src/optionstratlib/simulation/steps/mod.rs.html#1-47){.src}
]{.sub-heading}
::::

Expand description

::: docblock
Module containing functionality for stepping through data or
calculations.

This module provides components and utilities for managing step-based
operations, such as iterative calculations, data processing steps, or
any process that requires incremental progression through a series of
operations.

The stepping functionality is particularly useful for scenarios where:

- Operations need to be performed in a specific sequence
- Progress tracking is required through a multi-stage process
- Incremental state changes need to be managed

## [§](#financial-time-series-framework){.doc-anchor}Financial Time Series Framework

This module provides a comprehensive framework for handling time series
data in financial applications. It offers a structured approach to
manage and process time-based financial information with support for
different time units, step sizes, and value representations.

The framework is designed to be flexible, type-safe, and easy to use,
making it suitable for various financial modeling tasks including option
pricing, risk analysis, and time series forecasting.

### [§](#architecture){.doc-anchor}Architecture

The framework consists of three primary components:

- The `Step` structure which combines time and value information
- The `Xstep` structure for managing time-related components
- The `Ystep` structure for managing value-related components
:::

## Structs[§](#structs){.anchor} {#structs .section-header}

[Step](struct.Step.html "struct optionstratlib::simulation::steps::Step"){.struct}
:   Represents a combined x-y step in a two-dimensional simulation or
    analysis.

[Xstep](struct.Xstep.html "struct optionstratlib::simulation::steps::Xstep"){.struct}
:   Represents a step in a time series with an indexed value at a
    specific time point.

[Ystep](struct.Ystep.html "struct optionstratlib::simulation::steps::Ystep"){.struct}
:   A step entity in a Y-axis progression with an associated numeric
    value.
::::::
:::::::
