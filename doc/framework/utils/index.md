:::::::::::::: width-limiter
::::::::::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../index.html)
:::

# Module utils Copy item path

[[Source](../../src/optionstratlib/utils/mod.rs.html#1-209){.src}
]{.sub-heading}
::::

Expand description

:::::::::: docblock
- `utils` - General utility functions for data manipulation and
  calculations.

Collection of helper functions and utilities used across the library for
data manipulation, mathematical operations, date handling, and other
common tasks in financial calculations.

## [§](#utils-module){.doc-anchor}Utils Module

This module provides various utility functions, types, and tools for
common tasks across the library, including logging, time handling,
testing, and general-purpose utilities.

### [§](#core-components){.doc-anchor}Core Components

#### [§](#logger-loggerrs){.doc-anchor}Logger (`logger.rs`) {#logger-loggerrs}

Provides logging functionality with configurable log levels:

::: example-wrap
``` {.rust .rust-example-rendered}
use optionstratlib::utils::logger::{setup_logger, setup_logger_with_level};

// Initialize logger with environment variable


// Initialize logger with specific level
```
:::

#### [§](#time-timers){.doc-anchor}Time (`time.rs`) {#time-timers}

Handles various time frames for financial calculations:

::: example-wrap
``` {.rust .rust-example-rendered}
use positive::pos_or_panic;
use optionstratlib::utils::time::TimeFrame;

let daily = TimeFrame::Day;
let trading_days_per_year = daily.periods_per_year(); // Returns 252.0

let custom = TimeFrame::Custom(pos!(365.0));
let periods = custom.periods_per_year(); // Returns 365.0
```
:::

#### [§](#testing-testsrs){.doc-anchor}Testing (`tests.rs`) {#testing-testsrs}

Provides testing utilities and macros for relative equality assertions:

::: example-wrap
``` {.rust .rust-example-rendered}
use positive::Positive;
use optionstratlib::{assert_pos_relative_eq, pos};

let a = pos!(1.0);
let b = pos!(1.0001);
let epsilon = pos!(0.001);
assert_pos_relative_eq!(a, b, epsilon);
```
:::

#### [§](#other-utilities-othersrs){.doc-anchor}Other Utilities (`others.rs`) {#other-utilities-othersrs}

General-purpose utility functions:

::: example-wrap
``` {.rust .rust-example-rendered}
use optionstratlib::utils::others::{approx_equal, get_random_element, process_n_times_iter};
use std::collections::BTreeSet;

// Approximate equality comparison
let equal = approx_equal(1.0, 1.0001);

// Get random element from a set
let mut set = BTreeSet::new();
set.insert(1);
set.insert(2);
let random = get_random_element(&set);

// Process combinations
let numbers = vec![1, 2, 3];
let result = process_n_times_iter(&numbers, 2, |combination| {
    vec![combination[0] + combination[1]]
});
```
:::

### [§](#time-frame-support){.doc-anchor}Time Frame Support

The module supports various time frames for financial calculations:

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
- Custom periods

#### [§](#example-time-frame-usage){.doc-anchor}Example: Time Frame Usage

::: example-wrap
``` {.rust .rust-example-rendered}
use tracing::info;
use positive::pos_or_panic;
use optionstratlib::utils::time::TimeFrame;

let timeframes = vec![
    TimeFrame::Day,
    TimeFrame::Week,
    TimeFrame::Month,
    TimeFrame::Custom(pos!(360.0))
];

for tf in timeframes {
    info!("Periods per year: {}", tf.periods_per_year());
}
```
:::

### [§](#logging-configuration){.doc-anchor}Logging Configuration

Log levels can be configured through:

- Environment variable `LOGLEVEL`
- Direct specification in code

Supported levels:

- DEBUG
- INFO
- WARN
- ERROR
- TRACE

#### [§](#example-logging-setup){.doc-anchor}Example: Logging Setup

::: example-wrap
``` {.rust .rust-example-rendered}
use optionstratlib::utils::logger::setup_logger_with_level;
use tracing::{debug, info, warn};

// Setup with specific level


// Log messages
debug!("Detailed information for debugging");
info!("General information about program execution");
warn!("Warning messages for potentially harmful situations");
```
:::

### [§](#testing-utilities){.doc-anchor}Testing Utilities

The module provides testing utilities for:

- Relative equality comparisons for Positive
- Approximate floating-point comparisons
- Random element selection testing

#### [§](#example-testing-positive-values){.doc-anchor}Example: Testing Positive Values

::: example-wrap
``` {.rust .rust-example-rendered}
use positive::Positive;
use positive::pos_or_panic;
use optionstratlib::assert_pos_relative_eq;


fn test_values() {
    let a = pos!(1.0);
    let b = pos!(1.0001);
    let epsilon = pos!(0.001);
    assert_pos_relative_eq!(a, b, epsilon);
}
```
:::

### [§](#performance-considerations){.doc-anchor}Performance Considerations

- Logger initialization is thread-safe and happens only once
- Time frame calculations are constant-time operations
- Random element selection is O(n) where n is the set size
- Process combinations has complexity based on combination size

### [§](#implementation-notes){.doc-anchor}Implementation Notes

- Logger uses the `tracing` crate for structured logging
- Time frames use predefined constants for standard periods
- Testing utilities provide accurate floating-point comparisons
- Utility functions handle edge cases and error conditions
::::::::::

## Re-exports[§](#reexports){.anchor} {#reexports .section-header}

`pub use logger::`[`setup_logger`](logger/fn.setup_logger.html "fn optionstratlib::utils::logger::setup_logger"){.fn}`;`

`pub use logger::`[`setup_logger_with_level`](logger/fn.setup_logger_with_level.html "fn optionstratlib::utils::logger::setup_logger_with_level"){.fn}`;`

`pub use others::`[`approx_equal`](others/fn.approx_equal.html "fn optionstratlib::utils::others::approx_equal"){.fn}`;`

`pub use others::`[`get_random_element`](others/fn.get_random_element.html "fn optionstratlib::utils::others::get_random_element"){.fn}`;`

`pub use others::`[`process_n_times_iter`](others/fn.process_n_times_iter.html "fn optionstratlib::utils::others::process_n_times_iter"){.fn}`;`

`pub use others::`[`random_decimal`](others/fn.random_decimal.html "fn optionstratlib::utils::others::random_decimal"){.fn}`;`

`pub use time::`[`TimeFrame`](time/enum.TimeFrame.html "enum optionstratlib::utils::time::TimeFrame"){.enum}`;`

## Modules[§](#modules){.anchor} {#modules .section-header}

[file](file/index.html "mod optionstratlib::utils::file"){.mod}
:   This module contains the file reader and writer for OHLCV data. It
    provides functionality for reading and writing OHLCV data in various
    file formats, including CSV and JSON.

[logger](logger/index.html "mod optionstratlib::utils::logger"){.mod}
:   This module contains the logger setup and configuration. It provides
    functionality for initializing the logger, setting log levels, and
    formatting log messages. It uses the `tracing` crate for structured
    logging and supports various log levels.

[others](others/index.html "mod optionstratlib::utils::others"){.mod}
:   This module contains other miscellaneous modules and functions. It
    acts as a container for functionality that doesn't fit neatly into
    the main project structure. More specific documentation can be found
    within each sub-module.

[time](time/index.html "mod optionstratlib::utils::time"){.mod}
:   Module for time-related utilities.

## Structs[§](#structs){.anchor} {#structs .section-header}

[OhlcvCandle](struct.OhlcvCandle.html "struct optionstratlib::utils::OhlcvCandle"){.struct}
:   Represents an OHLC+V candlestick with timestamp

## Traits[§](#traits){.anchor} {#traits .section-header}

[Len](trait.Len.html "trait optionstratlib::utils::Len"){.trait}
:   A trait for types that have a notion of length or size.

## Functions[§](#functions){.anchor} {#functions .section-header}

[read_ohlcv_from_zip](fn.read_ohlcv_from_zip.html "fn optionstratlib::utils::read_ohlcv_from_zip"){.fn}
:   Reads OHLCV data from a zipped CSV file and filters it by date range
:::::::::::::
::::::::::::::
