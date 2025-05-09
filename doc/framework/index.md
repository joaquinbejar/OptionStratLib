::::::::::::::::::::::::::: width-limiter
:::::::::::::::::::::::::: {#main-content .section .content}
::: main-heading
# Crate optionstratlibCopy item path

[[Source](../src/optionstratlib/lib.rs.html#1-823){.src} ]{.sub-heading}
:::

Expand description

:::::::::::::::::::::::: docblock
::: {style="text-align: center;"}
![OptionStratLib](https://raw.githubusercontent.com/joaquinbejar/OptionStratLib/refs/heads/main/doc/images/logo.png){style="width: 100%; height: 200px;"}
:::

[![Dual
License](https://img.shields.io/badge/license-MIT%20and%20Apache%202.0-blue)](./LICENSE)
[![Crates.io](https://img.shields.io/crates/v/optionstratlib.svg)](https://crates.io/crates/optionstratlib)
[![Downloads](https://img.shields.io/crates/d/optionstratlib.svg)](https://crates.io/crates/optionstratlib)
[![Stars](https://img.shields.io/github/stars/joaquinbejar/OptionStratLib.svg)](https://github.com/joaquinbejar/OptionStratLib/stargazers)
[![Issues](https://img.shields.io/github/issues/joaquinbejar/OptionStratLib.svg)](https://github.com/joaquinbejar/OptionStratLib/issues)
[![PRs](https://img.shields.io/github/issues-pr/joaquinbejar/OptionStratLib.svg)](https://github.com/joaquinbejar/OptionStratLib/pulls)

[![Build
Status](https://img.shields.io/github/workflow/status/joaquinbejar/OptionStratLib/CI)](https://github.com/joaquinbejar/OptionStratLib/actions)
[![Coverage](https://img.shields.io/codecov/c/github/joaquinbejar/OptionStratLib)](https://codecov.io/gh/joaquinbejar/OptionStratLib)
[![Dependencies](https://img.shields.io/librariesio/github/joaquinbejar/OptionStratLib)](https://libraries.io/github/joaquinbejar/OptionStratLib)
[![Documentation](https://img.shields.io/badge/docs-latest-blue.svg)](https://docs.rs/optionstratlib)

## [§](#optionstratlib-v045-financial-options-library){.doc-anchor}OptionStratLib v0.4.5: Financial Options Library {#optionstratlib-v045-financial-options-library}

### [§](#table-of-contents){.doc-anchor}Table of Contents

1.  [Introduction](#introduction)
2.  [Features](#features)
3.  [Project Structure](#project-structure)
4.  [Setup Instructions](#setup-instructions)
5.  [Library Usage](#library-usage)
6.  [Usage Examples](#usage-examples)
7.  [Testing](#testing)
8.  [Contribution and Contact](#contribution-and-contact)

### [§](#introduction){.doc-anchor}Introduction

OptionStratLib is a comprehensive Rust library for options trading and
strategy development across multiple asset classes. This versatile
toolkit enables traders, quants, and developers to:

### [§](#features){.doc-anchor}Features

1.  **Valuation Models**:

- Black-Scholes model
- Binomial model
- Monte Carlo simulations
- Telegraph process model

2.  **Greeks Calculation**:

- Delta, gamma, theta, vega, and rho
- Custom Greeks implementation
- Greeks visualization

3.  **Option Types**:

- European and American options
- Calls and puts
- Support for exotic options (Asian, Barrier, etc.)

4.  **Volatility Models**:

- Constant volatility
- EWMA (Exponentially Weighted Moving Average)
- GARCH implementation
- Heston stochastic volatility
- Volatility surface interpolation

5.  **Option Chain Management**:

- Chain construction and analysis
- Strike price generation
- Chain data import/export (CSV/JSON)

6.  **Trading Strategies**:

- Bull Call Spread
- Bear Put Spread
- Call Butterfly
- Strategy optimization
- Custom strategy development

7.  **Risk Management**:

- SPAN margin calculation
- Position tracking
- Break-even analysis
- Profit/Loss calculations

8.  **Simulation Tools**:

- Random Walk simulation
- Telegraph process
- Monte Carlo methods
- Custom simulation frameworks

9.  **Visualization**:

- Strategy payoff diagrams
- Greeks visualization
- Binomial trees
- Risk profiles
- Interactive charts

10. **Data Management**:

- CSV/JSON import/export
- Option chain data handling
- Historical data analysis
- Price series management

11. **Backtesting**: **TODO!**
12. **Performance Metrics**: **TODO!**

### [§](#project-structure){.doc-anchor}Project Structure

The project is organized into the following key modules:

1.  **Core Options** (`options/`):

- `option.rs`: Core option structures and methods
- `position.rs`: Position management
- `chain.rs`: Option chain handling

2.  **Pricing Models** (`pricing/`):

- `binomial_model.rs`: Binomial tree implementation
- `black_scholes_model.rs`: Black-Scholes pricing
- `monte_carlo.rs`: Monte Carlo simulations
- `telegraph.rs`: Telegraph process model

3.  **Greeks** (`greeks/`):

- `equations.rs`: Greeks calculations
- `utils.rs`: Greek utilities

4.  **Volatility** (`volatility/`):

- `constant.rs`: Constant volatility model
- `ewma.rs`: EWMA implementation
- `garch.rs`: GARCH model
- `heston.rs`: Heston model
- `surface.rs`: Volatility surface handling

5.  **Strategies** (`strategies/`):

- `base.rs`: Strategy base traits
- `bear_put_spread.rs`:
- `bull_call_spread.rs`:
- `butterfly_spread.rs`:
- `call_butterfly.rs`:
- `collar.rs`:
- `covered_call.rs`:
- `custom.rs`: Custom strategy framework
- `iron_condor.rs`:
- `poor_mans_covered_call.rs`:
- `protective_put.rs`:
- `straddle.rs`:
- `strangle.rs`:
- `utils.rs`: Strategy utilities

6.  **Risk Management** (`risk/`):

- `span.rs`: SPAN margin calculation
- `margin.rs`: Margin requirements
- `position.rs`: Position risk metrics

7.  **Simulation** (`simulation/`):

- `random_walk.rs`
- `telegraph.rs`
- `monte_carlo.rs`

8.  **Visualization** (`visualization/`):

- `binomial_tree.rs`
- `strategy.rs`
- `utils.rs`

9.  **Data Management** (`data/`):

- `chain.rs`: Chain data structures
- `import.rs`: Data import utilities
- `export.rs`: Data export utilities

### [§](#relationships){.doc-anchor}Relationships

#### [§](#base-structure){.doc-anchor}Base Structure

::: example-wrap
``` language-mermaid
classDiagram
class Options {
+option_type: OptionType
+side: Side
+underlying_symbol: String
+strike_price: Positive
+expiration_date: ExpirationDate
+implied_volatility: Positive
+quantity: Positive
+underlying_price: Positive
+risk_free_rate: Decimal
+option_style: OptionStyle
+dividend_yield: Positive
+exotic_params: Option~ExoticParams~
+calculate_price_black_scholes()
+calculate_price_binomial()
+time_to_expiration()
+is_long()
+is_short()
+validate()
}

class OptionType {
<<enumeration>>
European
American
Bermuda
Asian
Barrier
Binary
Lookback
Compound
Chooser
Cliquet
Rainbow
Spread
Quanto
Exchange
Power
}

class Side {
<<enumeration>>
Long
Short
}

class OptionStyle {
<<enumeration>>
Call
Put
}

class Position {
+option: Options
+premium: f64
+date: DateTime
+open_fee: f64
+close_fee: f64
+total_cost()
+unrealized_pnl()
+days_held()
+days_to_expiration()
+is_long()
+is_short()
+break_even()
}

class Strategies {
<<interface>>
+add_leg()
+get_legs()
+break_even()
+max_profit()
+max_loss()
+total_cost()
+net_premium_received()
+fees()
}

class BullCallSpread {
+long_call: Position
+short_call: Position
+break_even()
+max_profit()
+max_loss()
}

class CallButterfly {
+long_call_itm: Position
+short_call: Position
+long_call_otm: Position
+break_even()
+max_profit()
+max_loss()
}

class Greeks {
<<interface>>
+delta()
+gamma()
+theta()
+vega()
+rho()
+rho_d()
}

class Profit {
<<interface>>
+calculate_profit_at()
}

class Graph {
<<interface>>
+graph()
+title()
+get_values()
}

class PnLCalculator {
<<interface>>
+calculate_pnl()
+calculate_pnl_at_expiration()
}

Options --|> Greeks
Options --|> Profit
Options --|> Graph
Position *-- Options
Position --|> Greeks
Position --|> Profit
Position --|> Graph
Position --|> PnLCalculator
BullCallSpread --|> Strategies
BullCallSpread --|> Profit
BullCallSpread --|> Graph
CallButterfly --|> Strategies
CallButterfly --|> Profit
CallButterfly --|> Graph
Options o-- OptionType
Options o-- Side
Options o-- OptionStyle
```
:::

#### [§](#strategy-structure){.doc-anchor}Strategy Structure

::: example-wrap
``` language-mermaid
classDiagram
class Options {
+option_type: OptionType
+side: Side
+strike_price: Positive
+expiration_date: ExpirationDate
+implied_volatility: f64
+calculate_price_black_scholes()
+calculate_price_binomial()
+calculate_delta()
+payoff()
}

class Position {
+option: Options
+premium: f64
+date: DateTime
+open_fee: f64
+close_fee: f64
+total_cost()
+unrealized_pnl()
+days_held()
}

class OptionChain {
+symbol: String
+underlying_price: Positive
+options: BTreeSet<OptionData>
+build_chain()
+add_option()
+save_to_csv()
+load_from_csv()
}

class Strategy {
<<Interface>>
+add_leg()
+get_legs()
+break_even()
+max_profit()
+max_loss()
+total_cost()
}

class BullCallSpread {
+long_call: Position
+short_call: Position
+break_even_points: Vec<f64>
+calculate_profit_at()
}

class CallButterfly {
+long_call_itm: Position
+long_call_otm: Position
+short_call: Position
+break_even_points: Vec<f64>
}

class Graph {
<<Interface>>
+title()
+get_values()
+get_vertical_lines()
+get_points()
}

class Profit {
<<Interface>>
+calculate_profit_at()
}

class Greeks {
<<Interface>>
+delta()
+gamma()
+theta()
+vega()
}

Position o-- Options
Strategy <|.. BullCallSpread
Strategy <|.. CallButterfly
Graph <|.. Options
Graph <|.. Position
Graph <|.. Strategy
Profit <|.. Options
Profit <|.. Position
Profit <|.. Strategy
Greeks <|.. Options
OptionChain o-- Options
BullCallSpread o-- Position
CallButterfly o-- Position
```
:::

### [§](#strategies-classifications){.doc-anchor}Strategies Classifications

::: example-wrap
``` language-mermaid
---
config:
layout: fixed
---
flowchart TD
start["Options Strategies"] --> bullish["Bullish"] & bearish["Bearish"] & neutral["Neutral"]
bullish --> bull_high["High Volatility"] & bull_low["Low Volatility"]
bull_high --> bull_high_lim["Limited Risk"] & bull_high_unlim["Unlimited Risk"]
bull_low --> bull_low_lim["Limited Risk"] & bull_low_unlim["Unlimited Risk"]
bull_high_lim --> bull_high_lim_opt["Options Only"] & bull_high_lim_stock["With Underlying"]
bull_high_unlim --> bull_high_unlim_opt["Options Only"] & bull_high_unlim_stock["With Underlying"]
bull_low_lim --> bull_low_lim_opt["Options Only"] & bull_low_lim_stock["With Underlying"]
bull_low_unlim --> bull_low_unlim_opt["Options Only"] & bull_low_unlim_stock["With Underlying"]
bull_high_lim_opt --> bull_call(("Bull Call Spread"))
bull_high_lim_stock --> protective_put(("Protective Put"))
bull_high_unlim_opt --> long_call(("Long Call"))
bull_high_unlim_stock --> pmcc@{ label: "Poor Man's Covered Call" }
bull_low_lim_opt --> bull_put(("Bull Put Spread"))
bull_low_lim_stock --> collar(("Collar"))
bull_low_unlim_opt --> naked_put(("Naked Put"))
bull_low_unlim_stock --> covered_call(("Covered Call"))
bearish --> bear_high["High Volatility"] & bear_low["Low Volatility"]
bear_high --> bear_high_lim["Limited Risk"] & bear_high_unlim["Unlimited Risk"]
bear_low --> bear_low_lim["Limited Risk"] & bear_low_unlim["Unlimited Risk"]
bear_high_lim --> bear_high_lim_opt["Options Only"] & bear_high_lim_stock["With Underlying"]
bear_high_unlim --> bear_high_unlim_opt["Options Only"] & bear_high_unlim_stock["With Underlying"]
bear_low_lim --> bear_low_lim_opt["Options Only"] & bear_low_lim_stock["With Underlying"]
bear_low_unlim --> bear_low_unlim_opt["Options Only"] & bear_low_unlim_stock["With Underlying"]
bear_high_lim_opt --> bear_put(("Bear Put Spread"))
bear_high_lim_stock --> synthetic_put(("Synthetic Put"))
bear_high_unlim_opt --> long_put(("Long Put"))
bear_high_unlim_stock --> covered_put(("Covered Put"))
bear_low_lim_opt --> bear_call(("Bear Call Spread"))
bear_low_lim_stock --> reverse_collar(("Reverse Collar"))
bear_low_unlim_opt --> naked_call(("Naked Call"))
bear_low_unlim_stock --> protective_call(("Protective Call"))
neutral --> neut_high["High Volatility"] & neut_low["Low Volatility"]
neut_high --> neut_high_lim["Limited Risk"] & neut_high_unlim["Unlimited Risk"]
neut_low --> neut_low_lim["Limited Risk"] & neut_low_unlim["Unlimited Risk"]
neut_high_lim --> neut_high_lim_opt["Options Only"] & neut_high_lim_stock["With Underlying"]
neut_high_unlim --> neut_high_unlim_opt["Options Only"] & neut_high_unlim_stock["With Underlying"]
neut_low_lim --> neut_low_lim_opt["Options Only"] & neut_low_lim_stock["With Underlying"]
neut_low_unlim --> neut_low_unlim_opt["Options Only"] & neut_low_unlim_stock["With Underlying"]
neut_high_lim_opt --> call_butterfly(("Call Butterfly")) & butterfly_spread(("Butterfly Spread")) & long_straddle(("Long Straddle")) & long_strangle(("Long Strangle"))
neut_high_unlim_opt --> short_straddle(("Short Straddle")) & short_strangle(("Short Strangle"))
neut_low_lim_opt --> iron_butterfly(("Iron Butterfly")) & iron_condor(("Iron Condor"))
neut_low_unlim_opt --> calendar_spread(("Calendar Spread")) & box_spread(("Box Spread"))
neut_high_lim_stock --> conversion(("Conversion"))
neut_high_unlim_stock --> reversal(("Reversal"))
neut_low_lim_stock --> married_combo(("Married Combo"))
neut_low_unlim_stock --> ratio_spread(("Ratio Spread"))
pmcc@{ shape: circle}
```
:::

### [§](#setup-instructions){.doc-anchor}Setup Instructions

1.  Clone the repository:

::: example-wrap
``` language-shell
git clone https://github.com/joaquinbejar/OptionStratLib.git
cd OptionStratLib
```
:::

2.  Build the project:

::: example-wrap
``` language-shell
make build
```
:::

3.  Run tests:

::: example-wrap
``` language-shell
make test
```
:::

4.  Format the code:

::: example-wrap
``` language-shell
make fmt
```
:::

5.  Run linting:

::: example-wrap
``` language-shell
make lint
```
:::

6.  Clean the project:

::: example-wrap
``` language-shell
make clean
```
:::

7.  Run the project:

::: example-wrap
``` language-shell
make run
```
:::

8.  Fix issues:

::: example-wrap
``` language-shell
make fix
```
:::

9.  Run pre-push checks:

::: example-wrap
``` language-shell
make pre-push
```
:::

10. Generate documentation:

::: example-wrap
``` language-shell
make doc
```
:::

11. Publish the package:

::: example-wrap
``` language-shell
make publish
```
:::

12. Generate coverage report:

::: example-wrap
``` language-shell
make coverage
```
:::

### [§](#library-usage){.doc-anchor}Library Usage

To use the library in your project, add the following to your
`Cargo.toml`:

::: example-wrap
``` language-toml
[dependencies]
optionstratlib = { git = "https://github.com/joaquinbejar/OptionStratLib.git" }
```
:::

### [§](#usage-examples){.doc-anchor}Usage Examples

Here are some examples of how to use the library for option pricing and
analysis:

::: example-wrap
``` {.rust .rust-example-rendered}
 use optionstratlib::greeks::Greeks;
 use optionstratlib::Options;
 use optionstratlib::Positive;
 use optionstratlib::model::types::{ OptionStyle, OptionType, Side};
 use optionstratlib::pos;
 use optionstratlib::utils::setup_logger;
   use std::error::Error;
 use tracing::info;

 fn create_sample_option() -> Options {
     use rust_decimal_macros::dec;
 use optionstratlib::pos;Options::new(
         OptionType::European,
         Side::Long,
         "AAPL".to_string(),
         pos!(100.0),
         ExpirationDate::Days(pos!(30.0)),
         pos!(0.2),
         pos!(1.0),
         pos!(105.0),
         dec!(0.05),
         OptionStyle::Call,
         Positive::ZERO,
         None,
     )
 }
 fn main() -> Result<(), Box<dyn Error>> {
     setup_logger();
     let option = create_sample_option();
     info!("Title: {}", option.title());
     info!("Greeks: {:?}", option.greeks());

     // Define a range of prices for the graph
     let price_range: Vec<Positive> = (50..150)
         .map(|x| pos!(x as f64))
         .collect();

     // Generate the intrinsic value graph
     option.graph(
         GraphBackend::Bitmap {
             file_path: "Draws/Options/intrinsic_value_chart.png",
             size: (1400, 933),
         },
         25,
     )?;

     Ok(())
 }
```
:::

::: example-wrap
``` {.rust .rust-example-rendered}
 use optionstratlib::Positive;
 use optionstratlib::ExpirationDate;
 use optionstratlib::pos;
 use optionstratlib::strategies::Strategies;
 use optionstratlib::strategies::bull_call_spread::BullCallSpread;
 use optionstratlib::utils::setup_logger;
   use std::error::Error;
 use tracing::info;

 fn main() -> Result<(), Box<dyn Error>> {
     use rust_decimal_macros::dec;
 setup_logger();

     let underlying_price = pos!(5781.88);

     let strategy = BullCallSpread::new(
         "SP500".to_string(),
         underlying_price,   // underlying_price
         pos!(5750.0),   // long_strike_itm
         pos!(5820.0),   // short_strike
         ExpirationDate::Days(pos!(2.0)),
         pos!(0.18),   // implied_volatility
         dec!(0.05),   // risk_free_rate
         Positive::ZERO,   // dividend_yield
         pos!(2.0),   // long quantity
         pos!(85.04),   // premium_long
         pos!(29.85),   // premium_short
         pos!(0.78),   // open_fee_long
         pos!(0.78),   // open_fee_long
         pos!(0.73),   // close_fee_long
         pos!(0.73),   // close_fee_short
     );

     let price_range = strategy.best_range_to_show(pos!(1.0)).unwrap();

     info!("Title: {}", strategy.title());
     info!("Break Even Points: {:?}", strategy.break_even_points);
     info!(
         "Net Premium Received: ${:.2}",
         strategy.net_premium_received()?
     );
     info!("Max Profit: ${:.2}", strategy.max_profit().unwrap_or(Positive::ZERO));
     info!("Max Loss: ${:0.2}", strategy.max_loss().unwrap_or(Positive::ZERO));
     info!("Total Fees: ${:.2}", strategy.fees()?);
     info!("Profit Area: {:.2}%", strategy.profit_area()?);
     info!("Profit Ratio: {:.2}%", strategy.profit_ratio()?);

     // Generate the profit/loss graph
     strategy.graph(
         GraphBackend::Bitmap {
             file_path: "Draws/Strategy/bull_call_spread_profit_loss_chart.png",
             size: (1400, 933),
         },
         20,
     )?;

     Ok(())
 }
```
:::

### [§](#testing){.doc-anchor}Testing

To run unit tests:

::: example-wrap
``` language-shell
make test
```
:::

To run tests with coverage:

::: example-wrap
``` language-shell
make coverage
```
:::

### [§](#contribution-and-contact){.doc-anchor}Contribution and Contact

We welcome contributions to this project! If you would like to
contribute, please follow these steps:

1.  Fork the repository.
2.  Create a new branch for your feature or bug fix.
3.  Make your changes and ensure that the project still builds and all
    tests pass.
4.  Commit your changes and push your branch to your forked repository.
5.  Submit a pull request to the main repository.

If you have any questions, issues, or would like to provide feedback,
please feel free to contact the project maintainer:

**Joaquín Béjar García**

- Email: jb@taunais.com
- GitHub: [joaquinbejar](https://github.com/joaquinbejar)

We appreciate your interest and look forward to your contributions!
::::::::::::::::::::::::

## Re-exports[§](#reexports){.anchor} {#reexports .section-header}

`pub use model::`[`Options`](model/option/struct.Options.html "struct optionstratlib::model::option::Options"){.struct}`;`

`pub use model::positive::`[`Positive`](model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}`;`

`pub use model::types::`[`ExpirationDate`](model/types/enum.ExpirationDate.html "enum optionstratlib::model::types::ExpirationDate"){.enum}`;`

`pub use model::types::`[`OptionStyle`](model/types/enum.OptionStyle.html "enum optionstratlib::model::types::OptionStyle"){.enum}`;`

`pub use model::types::`[`OptionType`](model/types/enum.OptionType.html "enum optionstratlib::model::types::OptionType"){.enum}`;`

`pub use model::types::`[`Side`](model/types/enum.Side.html "enum optionstratlib::model::types::Side"){.enum}`;`

## Modules[§](#modules){.anchor} {#modules .section-header}

[backtesting](backtesting/index.html "mod optionstratlib::backtesting"){.mod}
:   `backtesting` - Tools for historical performance evaluation of
    options strategies.

[chains](chains/index.html "mod optionstratlib::chains"){.mod}
:   `chains` - Functionality for working with options chains and series
    data.

[constants](constants/index.html "mod optionstratlib::constants"){.mod}
:   `constants` - Library-wide mathematical and financial constants.

[curves](curves/index.html "mod optionstratlib::curves"){.mod}
:   `curves` - Tools for yield curves, term structures, and other
    financial curves.

[error](error/index.html "mod optionstratlib::error"){.mod}
:   `error` - Error types and handling functionality for the library.

[geometrics](geometrics/index.html "mod optionstratlib::geometrics"){.mod}
:   `geometrics` - Mathematical utilities for geometric calculations
    relevant to options.

[greeks](greeks/index.html "mod optionstratlib::greeks"){.mod}
:   `greeks` - Calculation and management of option sensitivity metrics
    (Delta, Gamma, etc.).

[model](model/index.html "mod optionstratlib::model"){.mod}
:   `model` - Core data structures and models for options and
    derivatives.

[pnl](pnl/index.html "mod optionstratlib::pnl"){.mod}
:   `pnl` - Profit and loss analysis tools for options positions.

[pricing](pricing/index.html "mod optionstratlib::pricing"){.mod}
:   `pricing` - Option pricing models including Black-Scholes and
    numerical methods.

[risk](risk/index.html "mod optionstratlib::risk"){.mod}
:   `risk` - Risk assessment and management tools for options
    portfolios.

[simulation](simulation/index.html "mod optionstratlib::simulation"){.mod}
:   `simulation` - Simulation techniques for scenario analysis.

[strategies](strategies/index.html "mod optionstratlib::strategies"){.mod}
:   `strategies` - Pre-defined option strategy templates and building
    blocks.

[surfaces](surfaces/index.html "mod optionstratlib::surfaces"){.mod}
:   `surfaces` - Volatility surface and other 3D financial data
    modeling.

[utils](utils/index.html "mod optionstratlib::utils"){.mod}
:   `utils` - General utility functions for data manipulation and
    calculations.

[visualization](visualization/index.html "mod optionstratlib::visualization"){.mod}
:   `visualization` - Tools for plotting and visual representation of
    options data.

[volatility](volatility/index.html "mod optionstratlib::volatility"){.mod}
:   `volatility` - Volatility modeling, forecasting, and analysis
    utilities.

## Macros[§](#macros){.anchor} {#macros .section-header}

[assert_decimal_eq](macro.assert_decimal_eq.html "macro optionstratlib::assert_decimal_eq"){.macro}
:   Asserts that two Decimal values are approximately equal within a
    given epsilon

[assert_pos_relative_eq](macro.assert_pos_relative_eq.html "macro optionstratlib::assert_pos_relative_eq"){.macro}
:   Asserts that two `Positive` values are relatively equal within a
    given epsilon.

[build_chart](macro.build_chart.html "macro optionstratlib::build_chart"){.macro}
:   Builds a chart with a title and specified axis ranges.

[configure_chart_and_draw_mesh](macro.configure_chart_and_draw_mesh.html "macro optionstratlib::configure_chart_and_draw_mesh"){.macro}
:   Configures the chart mesh, labels, and draws a horizontal line at y
    = 0.

[create_drawing_area](macro.create_drawing_area.html "macro optionstratlib::create_drawing_area"){.macro}
:   Creates a drawing area with a white background.

[d2f](macro.d2f.html "macro optionstratlib::d2f"){.macro}
:   Converts a Decimal value to f64 with error propagation.

[d2fu](macro.d2fu.html "macro optionstratlib::d2fu"){.macro}
:   Converts a Decimal value to f64 without error checking.

[draw_line_segments](macro.draw_line_segments.html "macro optionstratlib::draw_line_segments"){.macro}
:   Draws line segments on the chart based on provided data.

[f2d](macro.f2d.html "macro optionstratlib::f2d"){.macro}
:   Converts an f64 value to Decimal with error propagation.

[f2du](macro.f2du.html "macro optionstratlib::f2du"){.macro}
:   Converts an f64 value to Decimal without error checking.

[pos](macro.pos.html "macro optionstratlib::pos"){.macro}
:   Macro for creating a new `Positive` value with simplified syntax.

[spos](macro.spos.html "macro optionstratlib::spos"){.macro}
:   Macro for creating an `Option<Positive>` value with simplified
    syntax.
::::::::::::::::::::::::::
:::::::::::::::::::::::::::
