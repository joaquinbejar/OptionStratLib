::::::: width-limiter
:::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../../index.html)::[utils](../index.html)
:::

# Module logger Copy item path

[[Source](../../../src/optionstratlib/utils/logger.rs.html#1-295){.src}
]{.sub-heading}
::::

Expand description

::: docblock
This module contains the logger setup and configuration. It provides
functionality for initializing the logger, setting log levels, and
formatting log messages. It uses the `tracing` crate for structured
logging and supports various log levels.

## [§](#utils-module){.doc-anchor}Utils Module

This module provides a collection of utility functions, structures, and
tools designed to simplify and support common tasks across the library.
These utilities range from logging, time frame management, testing
helpers, and other general-purpose helpers.

### [§](#key-components){.doc-anchor}Key Components

#### [§](#logger-logger){.doc-anchor}Logger (`logger`)

Handles application logging with configurable log levels. It includes
safe and idempotent initialization to avoid redundant setups. Useful for
debugging, tracing, and monitoring program behavior.

**Log Levels:**

- `DEBUG`: Detailed debugging information.
- `INFO`: General application status information.
- `WARN`: Non-critical issues that require attention.
- `ERROR`: Significant problems causing failures.
- `TRACE`: Fine-grained application execution details.

#### [§](#time-frames-time){.doc-anchor}Time Frames (`time`)

Provides robust structures for managing common time frames used in
financial or other periodic systems. This includes predefined constants
for standard periods and support for custom periods.

**Supported Time Frames:**

- Microsecond
- Millisecond
- Second
- Minute
- Hour
- Day
- Week
- Month
- Quarter
- Year
- Custom periods (customizable to specific needs)

#### [§](#testing-utilities-tests){.doc-anchor}Testing Utilities (`tests`)

A set of functions and macros to simplify testing. These include
utilities for relative equality comparisons and other common test-case
behaviors.

#### [§](#miscellaneous-utilities-others){.doc-anchor}Miscellaneous Utilities (`others`)

General-purpose functions for common operations, such as approximate
equality checks, random selection from a collection, and iterating over
combinations.

### [§](#performance-characteristics){.doc-anchor}Performance Characteristics

- **Logger:** Initialization is thread-safe and happens only once,
  ensuring minimal performance impact.
- **Time Frames:** All operations on time structures are constant-time.
- **Random Selection:** Complexity is O(n), where `n` is the size of the
  collection.
- **Combination Processing:** Complexity depends on the size of each
  combination and the number of combinations processed.

### [§](#design-notes){.doc-anchor}Design Notes

- **Logger:** Leverages the `tracing` crate, enabling structured and
  asynchronous logging.
- **Time Frames:** Focuses on reusable constants while supporting
  flexible customizations.
- **Testing Utilities:** Targets precise and consistent floating-point
  comparisons to prevent test inaccuracies.
- **General Utilities:** Built with error handling, edge case scenarios,
  and performance in mind.
:::

## Functions[§](#functions){.anchor} {#functions .section-header}

[setup_logger](fn.setup_logger.html "fn optionstratlib::utils::logger::setup_logger"){.fn}
:   Sets up a logger for the application

[setup_logger_with_level](fn.setup_logger_with_level.html "fn optionstratlib::utils::logger::setup_logger_with_level"){.fn}
:   Sets up a logger with a user-specified log level for platforms
::::::
:::::::
