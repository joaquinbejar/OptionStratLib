:::::::: width-limiter
::::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../index.html)
:::

# Module pnlCopy item path

[[Source](../../src/optionstratlib/pnl/mod.rs.html#7-120){.src}
]{.sub-heading}
::::

Expand description

:::: docblock
- `pnl` - Profit and loss analysis tools for options positions.

Utilities for calculating, projecting, and visualizing profit and loss
(P&L) profiles for individual options and complex strategies. Includes
time-based P&L evolution and scenario analysis.

## [§](#pnl-profit-and-loss-module){.doc-anchor}PnL (Profit and Loss) Module

This module provides structures and traits for calculating and managing
profit and loss (PnL) metrics in financial instruments, particularly
options.

### [§](#core-components){.doc-anchor}Core Components

- `PnL` - Structure representing profit and loss information
- `PnLCalculator` - Trait for implementing PnL calculation logic

### [§](#key-features){.doc-anchor}Key Features

#### [§](#pnl-structure){.doc-anchor}PnL Structure

The `PnL` structure captures:

- Realized profits or losses
- Unrealized profits or losses
- Initial costs and income
- Timestamp of calculation

#### [§](#pnl-calculator){.doc-anchor}PnL Calculator

The `PnLCalculator` trait enables:

- Real-time PnL calculations based on market prices
- PnL calculations at expiration
- Custom PnL calculation strategies

### [§](#example-usage){.doc-anchor}Example Usage

::: example-wrap
``` {.rust .rust-example-rendered}
use std::error::Error;
use optionstratlib::pnl::utils::{PnL, PnLCalculator};
use chrono::{DateTime, Utc};
use rust_decimal_macros::dec;
use optionstratlib::{ExpirationDate, Positive};
use optionstratlib::pos;

// Create a new PnL instance
let pnl = PnL::new(
    Some(dec!(100.0)),   // Realized PnL
    Some(dec!(50.0)),   // Unrealized PnL
    pos!(25.0),   // Initial costs
    pos!(75.0),   // Initial income
    Utc::now(),   // Calculation timestamp
);

// Example implementation of PnLCalculator
struct MyOption;

impl PnLCalculator for MyOption {

 fn calculate_pnl(
     &self,
     market_price: &Positive,
     expiration_date: ExpirationDate,
     _implied_volatility: &Positive,
 ) -> Result<PnL, Box<dyn Error>> {
     Ok(PnL::new(
         Some(market_price.into()),
         None,
         pos!(10.0),
         pos!(20.0),
         expiration_date.get_date()?,
     ))
 }
  
 fn calculate_pnl_at_expiration(
     &self,
     underlying_price: &Positive,
 ) -> Result<PnL, Box<dyn Error>> {
     let underlying_price = underlying_price.to_dec();
     Ok(PnL::new(
         Some(underlying_price),
         None,
         pos!(10.0),
         pos!(20.0),
         Utc::now(),
     ))
 }
}
```
:::

### [§](#applications){.doc-anchor}Applications

The PnL module is particularly useful for:

- Option position tracking
- Portfolio management
- Risk assessment
- Performance monitoring

### [§](#features){.doc-anchor}Features

- Real-time PnL tracking
- Expiration value calculations
- Cost basis tracking
- Income tracking
- Timestamp-based calculations
::::

## Re-exports[§](#reexports){.anchor} {#reexports .section-header}

`pub use utils::`[`PnL`](utils/struct.PnL.html "struct optionstratlib::pnl::utils::PnL"){.struct}`;`

## Modules[§](#modules){.anchor} {#modules .section-header}

[model](model/index.html "mod optionstratlib::pnl::model"){.mod}
:   [`model`](model/index.html "mod optionstratlib::pnl::model") - Core
    data structures for financial analysis and PnL modeling

[utils](utils/index.html "mod optionstratlib::pnl::utils"){.mod}
:   [`utils`](utils/index.html "mod optionstratlib::pnl::utils") -
    Utility functions for data manipulation and calculations

## Structs[§](#structs){.anchor} {#structs .section-header}

[PnLMetrics](struct.PnLMetrics.html "struct optionstratlib::pnl::PnLMetrics"){.struct}
:   `PnLMetrics` struct holds various metrics related to Profit and Loss
    (PnL) analysis.

[PnLMetricsDocument](struct.PnLMetricsDocument.html "struct optionstratlib::pnl::PnLMetricsDocument"){.struct}
:   Represents a document containing profit and loss (PnL) metrics for a
    specific asset over a period.

[PnLMetricsStep](struct.PnLMetricsStep.html "struct optionstratlib::pnl::PnLMetricsStep"){.struct}
:   PnLMetricsStep

[Transaction](struct.Transaction.html "struct optionstratlib::pnl::Transaction"){.struct}
:   Transaction

## Enums[§](#enums){.anchor} {#enums .section-header}

[TransactionStatus](enum.TransactionStatus.html "enum optionstratlib::pnl::TransactionStatus"){.enum}
:   Transaction Status

## Traits[§](#traits){.anchor} {#traits .section-header}

[PnLCalculator](trait.PnLCalculator.html "trait optionstratlib::pnl::PnLCalculator"){.trait}
:   Defines the interface for profit and loss (PnL) calculation on
    financial instruments.

[TransactionAble](trait.TransactionAble.html "trait optionstratlib::pnl::TransactionAble"){.trait}
:   TransactionAble

## Functions[§](#functions){.anchor} {#functions .section-header}

[create_pnl_metrics_document](fn.create_pnl_metrics_document.html "fn optionstratlib::pnl::create_pnl_metrics_document"){.fn}
:   Creates a `PnLMetricsDocument` instance.

[load_pnl_metrics](fn.load_pnl_metrics.html "fn optionstratlib::pnl::load_pnl_metrics"){.fn}
:   Loads a vector of PnLMetricsStep from a JSON file

[save_pnl_metrics](fn.save_pnl_metrics.html "fn optionstratlib::pnl::save_pnl_metrics"){.fn}
:   Serializes a vector of PnLMetricsStep to compact JSON and saves it
    to a file

[save_pnl_metrics_with_document](fn.save_pnl_metrics_with_document.html "fn optionstratlib::pnl::save_pnl_metrics_with_document"){.fn}
:   Saves PnL metrics to a JSON file, handling concurrent access and
    file existence.
:::::::
::::::::
