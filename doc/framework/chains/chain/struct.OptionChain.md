:::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::: width-limiter
::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../../index.html)::[chains](../index.html)::[chain](index.html)
:::

# Struct [OptionChain]{.struct}Copy item path

[[Source](../../../src/optionstratlib/chains/chain.rs.html#73-91){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub struct OptionChain {
    pub symbol: String,
    pub underlying_price: Positive,
    /* private fields */
}
```

Expand description

::: docblock
Represents an option chain for a specific underlying asset and
expiration date.

An option chain contains all available option contracts (calls and puts)
for a given underlying asset at a specific expiration date, along with
current market data and pricing parameters necessary for financial
analysis and valuation.

This struct provides a complete representation of option market data
that can be used for options strategy analysis, risk assessment, and
pricing model calculations.

## [§](#fields-1){.doc-anchor}Fields {#fields-1}

- `symbol` - The ticker symbol for the underlying asset (e.g., "AAPL",
  "SPY").

- `underlying_price` - The current market price of the underlying asset,
  stored as a guaranteed positive value.

- `expiration_date` - The expiration date of the options in the chain,
  typically represented in a standard date format.

- `options` - A sorted collection of option contracts at different
  strike prices, containing detailed market data like bid/ask prices,
  implied volatility, and the Greeks.

- `risk_free_rate` - The risk-free interest rate used for option pricing
  models, typically derived from treasury yields. May be `None` if not
  specified.

- `dividend_yield` - The annual dividend yield of the underlying asset,
  represented as a positive percentage. May be `None` for
  non-dividend-paying assets.

## [§](#usage){.doc-anchor}Usage

This struct is typically used as the primary container for options
market data analysis, serving as input to pricing models, strategy
backtesting, and risk management tools.
:::

## Fields[§](#fields){.anchor} {#fields .fields .section-header}

[[§](#structfield.symbol){.anchor
.field}`symbol: `[`String`](https://doc.rust-lang.org/1.86.0/alloc/string/struct.String.html "struct alloc::string::String"){.struct}]{#structfield.symbol
.structfield .section-header}

::: docblock
The ticker symbol for the underlying asset (e.g., "AAPL", "SPY").
:::

[[§](#structfield.underlying_price){.anchor
.field}`underlying_price: `[`Positive`](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}]{#structfield.underlying_price
.structfield .section-header}

::: docblock
The current market price of the underlying asset.
:::

## Implementations[§](#implementations){.anchor} {#implementations .section-header}

::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::: {#implementations-list}
::: {#impl-OptionChain .section .impl}
[Source](../../../src/optionstratlib/chains/chain.rs.html#227-2047){.src
.rightside}[§](#impl-OptionChain){.anchor}

### impl [OptionChain](struct.OptionChain.html "struct optionstratlib::chains::chain::OptionChain"){.struct} {#impl-optionchain .code-header}
:::

::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::: impl-items
::: {#method.new .section .method}
[Source](../../../src/optionstratlib/chains/chain.rs.html#269-284){.src
.rightside}

#### pub fn [new](#method.new){.fn}( symbol: &[str](https://doc.rust-lang.org/1.86.0/std/primitive.str.html){.primitive}, underlying_price: [Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, expiration_date: [String](https://doc.rust-lang.org/1.86.0/alloc/string/struct.String.html "struct alloc::string::String"){.struct}, risk_free_rate: [Option](https://doc.rust-lang.org/1.86.0/core/option/enum.Option.html "enum core::option::Option"){.enum}\<Decimal\>, dividend_yield: [Option](https://doc.rust-lang.org/1.86.0/core/option/enum.Option.html "enum core::option::Option"){.enum}\<[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\>, ) -\> Self {#pub-fn-new-symbol-str-underlying_price-positive-expiration_date-string-risk_free_rate-optiondecimal-dividend_yield-optionpositive---self .code-header}
:::

:::: docblock
Creates a new `OptionChain` for a specific underlying instrument and
expiration date.

This constructor initializes an `OptionChain` with the fundamental
parameters needed for option calculations and analysis. It creates an
empty collection of options that can be populated later through other
methods.

##### [§](#parameters){.doc-anchor}Parameters

- `symbol` - The ticker symbol of the underlying instrument (e.g.,
  "AAPL" for Apple Inc.).

- `underlying_price` - The current market price of the underlying
  instrument as a `Positive` value, ensuring it's always greater than or
  equal to zero.

- `expiration_date` - The expiration date for the options in this chain,
  provided as a string. The expected format depends on the
  implementation's requirements.

- `risk_free_rate` - The risk-free interest rate used for theoretical
  pricing models. This is optional and can be provided later if not
  available at creation time.

- `dividend_yield` - The dividend yield of the underlying instrument as
  a `Positive` value. This is optional and can be provided later for
  dividend-paying instruments.

##### [§](#returns){.doc-anchor}Returns

A new `OptionChain` instance with the specified parameters and an empty
set of options.

##### [§](#example){.doc-anchor}Example

::: example-wrap
``` {.rust .rust-example-rendered}
use rust_decimal_macros::dec;
use optionstratlib::chains::chain::OptionChain;
use optionstratlib::pos;

let chain = OptionChain::new(
    "AAPL",
    pos!(172.50),
    "2023-12-15".to_string(),
    Some(dec!(0.05)),  // 5% risk-free rate
    Some(pos!(0.0065)) // 0.65% dividend yield
);
```
:::
::::

::: {#method.build_chain .section .method}
[Source](../../../src/optionstratlib/chains/chain.rs.html#330-423){.src
.rightside}

#### pub fn [build_chain](#method.build_chain){.fn}(params: &[OptionChainBuildParams](../utils/struct.OptionChainBuildParams.html "struct optionstratlib::chains::utils::OptionChainBuildParams"){.struct}) -\> Self {#pub-fn-build_chainparams-optionchainbuildparams---self .code-header}
:::

:::: docblock
Builds a complete option chain based on the provided parameters.

This function creates an option chain with strikes generated around the
underlying price, calculates prices and Greeks for each option using the
Black-Scholes model, and applies the specified volatility skew to
reflect market conditions.

##### [§](#arguments){.doc-anchor}Arguments

- `params` - A reference to `OptionChainBuildParams` containing all
  necessary parameters for building the chain, including price
  parameters, chain size, and volatility settings.

##### [§](#returns-1){.doc-anchor}Returns

A fully populated `OptionChain` containing option data for all generated
strikes.

##### [§](#examples){.doc-anchor}Examples

::: example-wrap
``` {.rust .rust-example-rendered}
use rust_decimal_macros::dec;
use optionstratlib::chains::utils::{OptionChainBuildParams, OptionDataPriceParams};
use optionstratlib::{pos, spos, ExpirationDate};
use optionstratlib::chains::chain::OptionChain;
let price_params = OptionDataPriceParams::new(
    pos!(100.0),                         // underlying price
    ExpirationDate::Days(pos!(30.0)),    // expiration date
    Some(pos!(0.2)),                     // implied volatility
    dec!(0.05),                          // risk-free rate
    pos!(0.0),                           // dividend yield
    Some("SPY".to_string())              // underlying symbol
);

let build_params = OptionChainBuildParams::new(
    "SPY".to_string(),
    spos!(1000.0),
    10,
    pos!(5.0),
    dec!(0.1),
    pos!(0.02),
    2,
    price_params,
);

let chain = OptionChain::build_chain(&build_params);
```
:::
::::

::: {#method.to_build_params .section .method}
[Source](../../../src/optionstratlib/chains/chain.rs.html#439-540){.src
.rightside}

#### pub fn [to_build_params](#method.to_build_params){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[OptionChainBuildParams](../utils/struct.OptionChainBuildParams.html "struct optionstratlib::chains::utils::OptionChainBuildParams"){.struct}, [Box](https://doc.rust-lang.org/1.86.0/alloc/boxed/struct.Box.html "struct alloc::boxed::Box"){.struct}\<dyn [Error](https://doc.rust-lang.org/1.86.0/core/error/trait.Error.html "trait core::error::Error"){.trait}\>\> {#pub-fn-to_build_paramsself---resultoptionchainbuildparams-boxdyn-error .code-header}
:::

::: docblock
Generates build parameters that would reproduce the current option
chain.

This method creates an `OptionChainBuildParams` object with
configuration values extracted from the current chain. This is useful
for:

- Recreating a similar chain with modified parameters
- Saving the chain's configuration for later reconstruction
- Generating additional chains with consistent parameters

##### [§](#returns-2){.doc-anchor}Returns

An `OptionChainBuildParams` structure containing the parameters needed
to rebuild this option chain. The method calculates appropriate values
for chain size, strike interval, and estimated spread based on the
current data.
:::

::: {#method.add_option .section .method}
[Source](../../../src/optionstratlib/chains/chain.rs.html#659-711){.src
.rightside}

#### pub fn [add_option](#method.add_option){.fn}( &mut self, strike_price: [Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, call_bid: [Option](https://doc.rust-lang.org/1.86.0/core/option/enum.Option.html "enum core::option::Option"){.enum}\<[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\>, call_ask: [Option](https://doc.rust-lang.org/1.86.0/core/option/enum.Option.html "enum core::option::Option"){.enum}\<[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\>, put_bid: [Option](https://doc.rust-lang.org/1.86.0/core/option/enum.Option.html "enum core::option::Option"){.enum}\<[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\>, put_ask: [Option](https://doc.rust-lang.org/1.86.0/core/option/enum.Option.html "enum core::option::Option"){.enum}\<[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\>, implied_volatility: [Option](https://doc.rust-lang.org/1.86.0/core/option/enum.Option.html "enum core::option::Option"){.enum}\<[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\>, delta_call: [Option](https://doc.rust-lang.org/1.86.0/core/option/enum.Option.html "enum core::option::Option"){.enum}\<Decimal\>, delta_put: [Option](https://doc.rust-lang.org/1.86.0/core/option/enum.Option.html "enum core::option::Option"){.enum}\<Decimal\>, gamma: [Option](https://doc.rust-lang.org/1.86.0/core/option/enum.Option.html "enum core::option::Option"){.enum}\<Decimal\>, volume: [Option](https://doc.rust-lang.org/1.86.0/core/option/enum.Option.html "enum core::option::Option"){.enum}\<[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\>, open_interest: [Option](https://doc.rust-lang.org/1.86.0/core/option/enum.Option.html "enum core::option::Option"){.enum}\<[u64](https://doc.rust-lang.org/1.86.0/std/primitive.u64.html){.primitive}\>, ) {#pub-fn-add_option-mut-self-strike_price-positive-call_bid-optionpositive-call_ask-optionpositive-put_bid-optionpositive-put_ask-optionpositive-implied_volatility-optionpositive-delta_call-optiondecimal-delta_put-optiondecimal-gamma-optiondecimal-volume-optionpositive-open_interest-optionu64 .code-header}
:::

::: docblock
Adds a new option to the chain with the specified parameters.

This method creates and adds a new option at the given strike price to
the chain. It calculates mid prices and attempts to create detailed
option objects with the provided parameters.

##### [§](#arguments-1){.doc-anchor}Arguments

- `strike_price` - The strike price for the new option
- `call_bid` - Optional bid price for the call option
- `call_ask` - Optional ask price for the call option
- `put_bid` - Optional bid price for the put option
- `put_ask` - Optional ask price for the put option
- `implied_volatility` - Optional implied volatility for the option
- `delta_call` - Optional delta value for the call option
- `delta_put` - Optional delta value for the put option
- `gamma` - Optional gamma value for the option
- `volume` - Optional trading volume for the option
- `open_interest` - Optional open interest for the option

##### [§](#panics){.doc-anchor}Panics

Panics if the expiration date in the option chain cannot be parsed.
:::

::: {#method.atm_strike .section .method}
[Source](../../../src/optionstratlib/chains/chain.rs.html#740-743){.src
.rightside}

#### pub fn [atm_strike](#method.atm_strike){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<&[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, [Box](https://doc.rust-lang.org/1.86.0/alloc/boxed/struct.Box.html "struct alloc::boxed::Box"){.struct}\<dyn [Error](https://doc.rust-lang.org/1.86.0/core/error/trait.Error.html "trait core::error::Error"){.trait}\>\> {#pub-fn-atm_strikeself---resultpositive-boxdyn-error .code-header}
:::

:::: docblock
Returns the strike price closest to the underlying price (at-the-money).

This method searches through all available options in the chain to find
the one with a strike price that most closely matches the current
underlying price. This is useful for finding at-the-money (ATM) options
when there isn't an exact match for the underlying price.

##### [§](#returns-3){.doc-anchor}Returns

- `Ok(&Positive)` - Reference to the strike price closest to the
  underlying price
- `Err(Box<dyn Error>)` - Error if the option chain is empty or if the
  operation fails

##### [§](#example-1){.doc-anchor}Example

::: example-wrap
``` {.rust .rust-example-rendered}
use tracing::{error, info};
use optionstratlib::chains::chain::OptionChain;
use optionstratlib::pos;

let chain = OptionChain::new("SPY", pos!(450.75), "2023-12-15".to_string(), None, None);
// Add options to the chain...

match chain.atm_strike() {
    Ok(strike) => info!("Closest strike to underlying: {}", strike),
    Err(e) => error!("Error finding ATM strike: {}", e),
}
```
:::
::::

::: {#method.atm_option_data .section .method}
[Source](../../../src/optionstratlib/chains/chain.rs.html#764-800){.src
.rightside}

#### pub fn [atm_option_data](#method.atm_option_data){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<&[OptionData](../struct.OptionData.html "struct optionstratlib::chains::OptionData"){.struct}, [Box](https://doc.rust-lang.org/1.86.0/alloc/boxed/struct.Box.html "struct alloc::boxed::Box"){.struct}\<dyn [Error](https://doc.rust-lang.org/1.86.0/core/error/trait.Error.html "trait core::error::Error"){.trait}\>\> {#pub-fn-atm_option_dataself---resultoptiondata-boxdyn-error .code-header}
:::

::: docblock
Retrieves the OptionData for the at-the-money (ATM) option.

This function attempts to find the ATM option within the option chain.
First, it checks for an option with a strike price that exactly matches
the underlying asset's price. If an exact match is not found, it
searches for the option with the strike price closest to the underlying
price.

##### [§](#returns-4){.doc-anchor}Returns

- `Ok(&OptionData)` - If a suitable ATM option is found, returns a
  reference to it.
- `Err(Box<dyn Error>)` - If the option chain is empty or no ATM option
  can be found, returns an error describing the failure.

##### [§](#errors){.doc-anchor}Errors

This function returns an error in the following cases:

- The option chain (`self.options`) is empty.
- No option with a strike price close to the underlying price can be
  found.
:::

::: {#method.get_title .section .method}
[Source](../../../src/optionstratlib/chains/chain.rs.html#812-819){.src
.rightside}

#### pub fn [get_title](#method.get_title){.fn}(&self) -\> [String](https://doc.rust-lang.org/1.86.0/alloc/string/struct.String.html "struct alloc::string::String"){.struct} {#pub-fn-get_titleself---string .code-header}
:::

::: docblock
Returns a formatted title string for the option chain.

This method creates a title by combining the option chain's symbol,
expiration date, and underlying price. Spaces in the symbol and
expiration date are replaced with hyphens for better compatibility with
file systems and data representation.

##### [§](#returns-5){.doc-anchor}Returns

A formatted string in the format
"{symbol}-{expiration_date}-{underlying_price}" where spaces have been
replaced with hyphens in the symbol and expiration date.
:::

::: {#method.set_from_title .section .method}
[Source](../../../src/optionstratlib/chains/chain.rs.html#839-858){.src
.rightside}

#### pub fn [set_from_title](#method.set_from_title){.fn}(&mut self, file: &[str](https://doc.rust-lang.org/1.86.0/std/primitive.str.html){.primitive}) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[()](https://doc.rust-lang.org/1.86.0/std/primitive.unit.html){.primitive}, [Box](https://doc.rust-lang.org/1.86.0/alloc/boxed/struct.Box.html "struct alloc::boxed::Box"){.struct}\<dyn [Error](https://doc.rust-lang.org/1.86.0/core/error/trait.Error.html "trait core::error::Error"){.trait}\>\> {#pub-fn-set_from_titlemut-self-file-str---result-boxdyn-error .code-header}
:::

::: docblock
Parses a file name to set the option chain's properties.

This method extracts information from a file name that follows the
format "symbol-day-month-year-price.extension". It sets the symbol,
expiration date, and underlying price of the option chain based on the
parsed values.

##### [§](#arguments-2){.doc-anchor}Arguments

- `file` - A string slice representing the file path or name to parse

##### [§](#returns-6){.doc-anchor}Returns

- `Ok(())` - If the file name was successfully parsed and the properties
  were set
- `Err(...)` - If the file name format is invalid or the underlying
  price cannot be parsed

##### [§](#panics-1){.doc-anchor}Panics

This function will panic if the underlying price in the file name cannot
be parsed as an f64.
:::

::: {#method.update_mid_prices .section .method}
[Source](../../../src/optionstratlib/chains/chain.rs.html#867-878){.src
.rightside}

#### pub fn [update_mid_prices](#method.update_mid_prices){.fn}(&mut self) {#pub-fn-update_mid_pricesmut-self .code-header}
:::

::: docblock
Updates the mid prices for all options in the chain.

This method creates a new collection of options where each option has
its mid price calculated and updated. The mid price is typically the
average of the bid and ask prices.

The original options in the chain are replaced with the updated ones.
:::

::: {#method.update_greeks .section .method}
[Source](../../../src/optionstratlib/chains/chain.rs.html#888-901){.src
.rightside}

#### pub fn [update_greeks](#method.update_greeks){.fn}(&mut self) {#pub-fn-update_greeksmut-self .code-header}
:::

::: docblock
Calculates and updates the delta and gamma Greeks for all options in the
chain.

This method computes the delta and gamma values for each option in the
chain based on the current market parameters. Delta measures the rate of
change of the option price with respect to the underlying asset's price,
while gamma measures the rate of change of delta with respect to the
underlying asset's price.

The original options in the chain are replaced with the ones containing
the updated Greeks.
:::

::: {#method.update_implied_volatilities .section .method}
[Source](../../../src/optionstratlib/chains/chain.rs.html#914-931){.src
.rightside}

#### pub fn [update_implied_volatilities](#method.update_implied_volatilities){.fn}(&mut self) {#pub-fn-update_implied_volatilitiesmut-self .code-header}
:::

::: docblock
Calculates and updates the implied volatility for all options in the
chain.

This method attempts to compute the implied volatility for each option
in the chain. Implied volatility is the market's forecast of a likely
movement in the underlying price and is derived from the option's market
price.

If the calculation fails for any option, a debug message is logged with
the strike price and the error, but the process continues for other
options.

The original options in the chain are replaced with the ones containing
the updated implied volatility values.
:::

::: {#method.save_to_csv .section .method}
[Source](../../../src/optionstratlib/chains/chain.rs.html#952-985){.src
.rightside}

#### pub fn [save_to_csv](#method.save_to_csv){.fn}(&self, file_path: &[str](https://doc.rust-lang.org/1.86.0/std/primitive.str.html){.primitive}) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[()](https://doc.rust-lang.org/1.86.0/std/primitive.unit.html){.primitive}, [Box](https://doc.rust-lang.org/1.86.0/alloc/boxed/struct.Box.html "struct alloc::boxed::Box"){.struct}\<dyn [Error](https://doc.rust-lang.org/1.86.0/core/error/trait.Error.html "trait core::error::Error"){.trait}\>\> {#pub-fn-save_to_csvself-file_path-str---result-boxdyn-error .code-header}
:::

::: docblock
Saves the option chain data to a CSV file.

This method writes the option chain data to a CSV file at the specified
path. The file will be named using the option chain's title (symbol,
expiration date, and price). The CSV includes headers for all option
properties and each option in the chain is written as a row.

##### [§](#arguments-3){.doc-anchor}Arguments

- `file_path` - The directory path where the CSV file will be created

##### [§](#returns-7){.doc-anchor}Returns

- `Result<(), Box<dyn Error>>` - Ok(()) if successful, or an Error if
  the file couldn't be created or written to.

##### [§](#note){.doc-anchor}Note

This method is only available on non-WebAssembly targets.
:::

::: {#method.save_to_json .section .method}
[Source](../../../src/optionstratlib/chains/chain.rs.html#1005-1010){.src
.rightside}

#### pub fn [save_to_json](#method.save_to_json){.fn}(&self, file_path: &[str](https://doc.rust-lang.org/1.86.0/std/primitive.str.html){.primitive}) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[()](https://doc.rust-lang.org/1.86.0/std/primitive.unit.html){.primitive}, [Box](https://doc.rust-lang.org/1.86.0/alloc/boxed/struct.Box.html "struct alloc::boxed::Box"){.struct}\<dyn [Error](https://doc.rust-lang.org/1.86.0/core/error/trait.Error.html "trait core::error::Error"){.trait}\>\> {#pub-fn-save_to_jsonself-file_path-str---result-boxdyn-error .code-header}
:::

::: docblock
Saves the option chain data to a JSON file.

This method serializes the option chain into JSON format and writes it
to a file at the specified path. The file will be named using the option
chain's title (symbol, expiration date, and price).

##### [§](#arguments-4){.doc-anchor}Arguments

- `file_path` - The directory path where the JSON file will be created

##### [§](#returns-8){.doc-anchor}Returns

- `Result<(), Box<dyn Error>>` - Ok(()) if successful, or an Error if
  the file couldn't be created or written to.

##### [§](#note-1){.doc-anchor}Note

This method is only available on non-WebAssembly targets.
:::

::: {#method.load_from_csv .section .method}
[Source](../../../src/optionstratlib/chains/chain.rs.html#1029-1071){.src
.rightside}

#### pub fn [load_from_csv](#method.load_from_csv){.fn}(file_path: &[str](https://doc.rust-lang.org/1.86.0/std/primitive.str.html){.primitive}) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<Self, [Box](https://doc.rust-lang.org/1.86.0/alloc/boxed/struct.Box.html "struct alloc::boxed::Box"){.struct}\<dyn [Error](https://doc.rust-lang.org/1.86.0/core/error/trait.Error.html "trait core::error::Error"){.trait}\>\> {#pub-fn-load_from_csvfile_path-str---resultself-boxdyn-error .code-header}
:::

::: docblock
Loads option chain data from a CSV file.

This function reads option data from a CSV file and constructs an
OptionChain. It attempts to extract the symbol, underlying price, and
expiration date from the file name.

##### [§](#arguments-5){.doc-anchor}Arguments

- `file_path` - The path to the CSV file containing option chain data

##### [§](#returns-9){.doc-anchor}Returns

- `Result<Self, Box<dyn Error>>` - An OptionChain if successful, or an
  Error if the file couldn't be read or the data is invalid.

##### [§](#note-2){.doc-anchor}Note

This method is only available on non-WebAssembly targets.
:::

::: {#method.load_from_json .section .method}
[Source](../../../src/optionstratlib/chains/chain.rs.html#1090-1107){.src
.rightside}

#### pub fn [load_from_json](#method.load_from_json){.fn}(file_path: &[str](https://doc.rust-lang.org/1.86.0/std/primitive.str.html){.primitive}) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<Self, [Box](https://doc.rust-lang.org/1.86.0/alloc/boxed/struct.Box.html "struct alloc::boxed::Box"){.struct}\<dyn [Error](https://doc.rust-lang.org/1.86.0/core/error/trait.Error.html "trait core::error::Error"){.trait}\>\> {#pub-fn-load_from_jsonfile_path-str---resultself-boxdyn-error .code-header}
:::

::: docblock
Loads option chain data from a JSON file.

This function deserializes an OptionChain from a JSON file and updates
the mid prices for all options in the chain.

##### [§](#arguments-6){.doc-anchor}Arguments

- `file_path` - The path to the JSON file containing serialized option
  chain data

##### [§](#returns-10){.doc-anchor}Returns

- `Result<Self, Box<dyn Error>>` - An OptionChain if successful, or an
  Error if the file couldn't be read or the data is invalid.

##### [§](#note-3){.doc-anchor}Note

This method is only available on non-WebAssembly targets.
:::

::: {#method.strike_price_range_vec .section .method}
[Source](../../../src/optionstratlib/chains/chain.rs.html#1139-1153){.src
.rightside}

#### pub fn [strike_price_range_vec](#method.strike_price_range_vec){.fn}(&self, step: [f64](https://doc.rust-lang.org/1.86.0/std/primitive.f64.html){.primitive}) -\> [Option](https://doc.rust-lang.org/1.86.0/core/option/enum.Option.html "enum core::option::Option"){.enum}\<[Vec](https://doc.rust-lang.org/1.86.0/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<[f64](https://doc.rust-lang.org/1.86.0/std/primitive.f64.html){.primitive}\>\> {#pub-fn-strike_price_range_vecself-step-f64---optionvecf64 .code-header}
:::

::: docblock
Generates a vector of strike prices within the range of available
options.

This method creates a vector of strike prices starting from the lowest
strike price in the chain up to the highest, incrementing by the
specified step.

##### [§](#arguments-7){.doc-anchor}Arguments

- `step` - The increment value between consecutive strike prices

##### [§](#returns-11){.doc-anchor}Returns

- `Option<Vec<f64>>` - A vector containing the strike prices if the
  option chain is not empty, or None if there are no options in the
  chain.
:::

::: {#method.get_random_positions .section .method}
[Source](../../../src/optionstratlib/chains/chain.rs.html#1167-1297){.src
.rightside}

#### pub fn [get_random_positions](#method.get_random_positions){.fn}( &self, params: [RandomPositionsParams](../utils/struct.RandomPositionsParams.html "struct optionstratlib::chains::utils::RandomPositionsParams"){.struct}, ) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Vec](https://doc.rust-lang.org/1.86.0/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<[Position](../../model/position/struct.Position.html "struct optionstratlib::model::position::Position"){.struct}\>, [ChainError](../../error/chains/enum.ChainError.html "enum optionstratlib::error::chains::ChainError"){.enum}\> {#pub-fn-get_random_positions-self-params-randompositionsparams---resultvecposition-chainerror .code-header}
:::

::: docblock
Creates random positions based on specified quantities of puts and calls

##### [§](#arguments-8){.doc-anchor}Arguments

- `qty_puts_long` - Number of long put positions to create
- `qty_puts_short` - Number of short put positions to create
- `qty_calls_long` - Number of long call positions to create
- `qty_calls_short` - Number of short call positions to create

##### [§](#returns-12){.doc-anchor}Returns

- `Result<Vec<Position>, ChainError>` - Vector of created positions or
  error message
:::

::: {#method.get_single_iter .section .method}
[Source](../../../src/optionstratlib/chains/chain.rs.html#1310-1314){.src
.rightside}

#### pub fn [get_single_iter](#method.get_single_iter){.fn}(&self) -\> impl [Iterator](https://doc.rust-lang.org/1.86.0/core/iter/traits/iterator/trait.Iterator.html "trait core::iter::traits::iterator::Iterator"){.trait}\<Item = &[OptionData](../struct.OptionData.html "struct optionstratlib::chains::OptionData"){.struct}\> {#pub-fn-get_single_iterself---impl-iteratoritem-optiondata .code-header}
:::

::: docblock
Returns an iterator over the `options` field in the `OptionChain`
structure.

This method provides a mechanism to traverse through the set of options
(`OptionData`) associated with an `OptionChain`.

##### [§](#returns-13){.doc-anchor}Returns

An iterator that yields references to the `OptionData` elements in the
`options` field. Since the `options` field is stored as a `BTreeSet`,
the elements are ordered in ascending order based on the sorting rules
of `BTreeSet` (typically defined by `Ord` implementation).
:::

:::: {#method.mutate_single_options .section .method}
[Source](../../../src/optionstratlib/chains/chain.rs.html#1327-1343){.src
.rightside}

#### pub fn [mutate_single_options](#method.mutate_single_options){.fn}\<F\>(&mut self, f: F) {#pub-fn-mutate_single_optionsfmut-self-f-f .code-header}

::: where
where F:
[FnMut](https://doc.rust-lang.org/1.86.0/core/ops/function/trait.FnMut.html "trait core::ops::function::FnMut"){.trait}(&mut
[OptionData](../struct.OptionData.html "struct optionstratlib::chains::OptionData"){.struct}),
:::
::::

::: docblock
Applies a mutation function to each option in the chain that has an
implied volatility value.

This method filters the option chain to include only options with
defined implied volatility, applies the provided function to each
option, and then updates the chain with these modified options. The
options collection is completely replaced with the new, modified set.

##### [§](#arguments-9){.doc-anchor}Arguments

- `f` - A mutable closure that takes a mutable reference to an
  `OptionData` and applies some transformation or modification to it.
:::

::: {#method.get_single_iter_mut .section .method}
[Source](../../../src/optionstratlib/chains/chain.rs.html#1362-1367){.src
.rightside}

#### pub fn [get_single_iter_mut](#method.get_single_iter_mut){.fn}(&mut self) -\> impl [Iterator](https://doc.rust-lang.org/1.86.0/core/iter/traits/iterator/trait.Iterator.html "trait core::iter::traits::iterator::Iterator"){.trait}\<Item = [OptionData](../struct.OptionData.html "struct optionstratlib::chains::OptionData"){.struct}\> {#pub-fn-get_single_iter_mutmut-self---impl-iteratoritem-optiondata .code-header}
:::

::: docblock
Returns an iterator that provides mutable access to individual options
in the chain.

This method enables modifying options in the chain while maintaining the
collection's integrity. It works by:

1.  Filtering options that have implied volatility
2.  Removing each option from the internal collection
3.  Providing mutable access to each option

The caller is responsible for reinserting modified options back into the
chain. After modifications, options should be reinserted into the chain
using appropriate methods.

##### [§](#returns-14){.doc-anchor}Returns

An iterator yielding mutable references to `OptionData` instances.

##### [§](#examples-1){.doc-anchor}Examples
:::

::: {#method.get_double_iter .section .method}
[Source](../../../src/optionstratlib/chains/chain.rs.html#1390-1396){.src
.rightside}

#### pub fn [get_double_iter](#method.get_double_iter){.fn}( &self, ) -\> impl [Iterator](https://doc.rust-lang.org/1.86.0/core/iter/traits/iterator/trait.Iterator.html "trait core::iter::traits::iterator::Iterator"){.trait}\<Item = (&[OptionData](../struct.OptionData.html "struct optionstratlib::chains::OptionData"){.struct}, &[OptionData](../struct.OptionData.html "struct optionstratlib::chains::OptionData"){.struct})\> {#pub-fn-get_double_iter-self---impl-iteratoritem-optiondata-optiondata .code-header}
:::

:::: docblock
Returns an iterator that generates pairs of distinct option combinations
from the `OptionChain`.

This function iterates over all unique combinations of two options from
the `options` collection without repetition. In mathematical terms, it
generates combinations where order does not matter and an option cannot
combine with itself.

##### [§](#returns-15){.doc-anchor}Returns

An iterator producing tuples of references to two distinct `OptionData`
instances.

##### [§](#example-2){.doc-anchor}Example

::: example-wrap
``` {.rust .rust-example-rendered}
use tracing::info;
use optionstratlib::chains::chain::OptionChain;
use optionstratlib::{pos, Positive};
let mut option_chain = OptionChain::new("TEST", pos!(100.0), "2024-01-01".to_string(), None, None);
for (option1, option2) in option_chain.get_double_iter() {
    info!("{:?}, {:?}", option1, option2);
}
```
:::
::::

::: {#method.get_double_inclusive_iter .section .method}
[Source](../../../src/optionstratlib/chains/chain.rs.html#1420-1426){.src
.rightside}

#### pub fn [get_double_inclusive_iter](#method.get_double_inclusive_iter){.fn}( &self, ) -\> impl [Iterator](https://doc.rust-lang.org/1.86.0/core/iter/traits/iterator/trait.Iterator.html "trait core::iter::traits::iterator::Iterator"){.trait}\<Item = (&[OptionData](../struct.OptionData.html "struct optionstratlib::chains::OptionData"){.struct}, &[OptionData](../struct.OptionData.html "struct optionstratlib::chains::OptionData"){.struct})\> {#pub-fn-get_double_inclusive_iter-self---impl-iteratoritem-optiondata-optiondata .code-header}
:::

:::: docblock
Returns an iterator that generates inclusive pairs of option
combinations from the `OptionChain`.

This function iterates over all combinations of two options from the
`options` collection, including pairing an option with itself.

##### [§](#returns-16){.doc-anchor}Returns

An iterator producing tuples with two references to `OptionData`,
potentially including self-pairs (e.g., `(option, option)`).

##### [§](#example-3){.doc-anchor}Example

::: example-wrap
``` {.rust .rust-example-rendered}
use tracing::info;
use optionstratlib::chains::chain::OptionChain;
use optionstratlib::Positive;
use optionstratlib::pos;
let mut option_chain = OptionChain::new("TEST", pos!(100.0), "2024-01-01".to_string(), None, None);
for (option1, option2) in option_chain.get_double_inclusive_iter() {
    info!("{:?}, {:?}", option1, option2);
}
```
:::
::::

::: {#method.get_triple_iter .section .method}
[Source](../../../src/optionstratlib/chains/chain.rs.html#1449-1462){.src
.rightside}

#### pub fn [get_triple_iter](#method.get_triple_iter){.fn}( &self, ) -\> impl [Iterator](https://doc.rust-lang.org/1.86.0/core/iter/traits/iterator/trait.Iterator.html "trait core::iter::traits::iterator::Iterator"){.trait}\<Item = (&[OptionData](../struct.OptionData.html "struct optionstratlib::chains::OptionData"){.struct}, &[OptionData](../struct.OptionData.html "struct optionstratlib::chains::OptionData"){.struct}, &[OptionData](../struct.OptionData.html "struct optionstratlib::chains::OptionData"){.struct})\> {#pub-fn-get_triple_iter-self---impl-iteratoritem-optiondata-optiondata-optiondata .code-header}
:::

:::: docblock
Returns an iterator that generates unique triplets of distinct option
combinations from the `OptionChain`.

This function iterates over all unique combinations of three options
from the `options` collection without repetition.

##### [§](#returns-17){.doc-anchor}Returns

An iterator producing tuples containing references to three distinct
`OptionData` instances.

##### [§](#example-4){.doc-anchor}Example

::: example-wrap
``` {.rust .rust-example-rendered}
use tracing::info;
use optionstratlib::chains::chain::OptionChain;
use optionstratlib::Positive;
use optionstratlib::pos;
let mut option_chain = OptionChain::new("TEST", pos!(100.0), "2024-01-01".to_string(), None, None);
for (option1, option2, option3) in option_chain.get_triple_iter() {
    info!("{:?}, {:?}, {:?}", option1, option2, option3);
}
```
:::
::::

::: {#method.get_triple_inclusive_iter .section .method}
[Source](../../../src/optionstratlib/chains/chain.rs.html#1486-1501){.src
.rightside}

#### pub fn [get_triple_inclusive_iter](#method.get_triple_inclusive_iter){.fn}( &self, ) -\> impl [Iterator](https://doc.rust-lang.org/1.86.0/core/iter/traits/iterator/trait.Iterator.html "trait core::iter::traits::iterator::Iterator"){.trait}\<Item = (&[OptionData](../struct.OptionData.html "struct optionstratlib::chains::OptionData"){.struct}, &[OptionData](../struct.OptionData.html "struct optionstratlib::chains::OptionData"){.struct}, &[OptionData](../struct.OptionData.html "struct optionstratlib::chains::OptionData"){.struct})\> {#pub-fn-get_triple_inclusive_iter-self---impl-iteratoritem-optiondata-optiondata-optiondata .code-header}
:::

:::: docblock
Returns an iterator that generates inclusive triplets of option
combinations from the `OptionChain`.

This function iterates over all combinations of three options from the
`options` collection, including those where the same option may be
included more than once.

##### [§](#returns-18){.doc-anchor}Returns

An iterator producing tuples with three references to `OptionData`,
potentially including repeated elements (e.g.,
`(option1, option2, option1)`).

##### [§](#example-5){.doc-anchor}Example

::: example-wrap
``` {.rust .rust-example-rendered}
use tracing::info;
use optionstratlib::chains::chain::OptionChain;
use optionstratlib::Positive;
use optionstratlib::pos;
let mut option_chain = OptionChain::new("TEST", pos!(100.0), "2024-01-01".to_string(), None, None);
for (option1, option2, option3) in option_chain.get_triple_inclusive_iter() {
    info!("{:?}, {:?}, {:?}", option1, option2, option3);
}
```
:::
::::

::: {#method.get_quad_iter .section .method}
[Source](../../../src/optionstratlib/chains/chain.rs.html#1524-1543){.src
.rightside}

#### pub fn [get_quad_iter](#method.get_quad_iter){.fn}( &self, ) -\> impl [Iterator](https://doc.rust-lang.org/1.86.0/core/iter/traits/iterator/trait.Iterator.html "trait core::iter::traits::iterator::Iterator"){.trait}\<Item = (&[OptionData](../struct.OptionData.html "struct optionstratlib::chains::OptionData"){.struct}, &[OptionData](../struct.OptionData.html "struct optionstratlib::chains::OptionData"){.struct}, &[OptionData](../struct.OptionData.html "struct optionstratlib::chains::OptionData"){.struct}, &[OptionData](../struct.OptionData.html "struct optionstratlib::chains::OptionData"){.struct})\> {#pub-fn-get_quad_iter-self---impl-iteratoritem-optiondata-optiondata-optiondata-optiondata .code-header}
:::

:::: docblock
Returns an iterator that generates unique quadruples of distinct option
combinations from the `OptionChain`.

This function iterates over all unique combinations of four options from
the `options` collection without repetition.

##### [§](#returns-19){.doc-anchor}Returns

An iterator producing tuples containing references to four distinct
`OptionData` instances.

##### [§](#example-6){.doc-anchor}Example

::: example-wrap
``` {.rust .rust-example-rendered}
use tracing::info;
use optionstratlib::chains::chain::OptionChain;
use optionstratlib::Positive;
use optionstratlib::pos;
let mut option_chain = OptionChain::new("TEST", pos!(100.0), "2024-01-01".to_string(), None, None);
for (option1, option2, option3, option4) in option_chain.get_quad_iter() {
    info!("{:?}, {:?}, {:?}, {:?}", option1, option2, option3, option4);
}
```
:::
::::

::: {#method.get_quad_inclusive_iter .section .method}
[Source](../../../src/optionstratlib/chains/chain.rs.html#1567-1586){.src
.rightside}

#### pub fn [get_quad_inclusive_iter](#method.get_quad_inclusive_iter){.fn}( &self, ) -\> impl [Iterator](https://doc.rust-lang.org/1.86.0/core/iter/traits/iterator/trait.Iterator.html "trait core::iter::traits::iterator::Iterator"){.trait}\<Item = (&[OptionData](../struct.OptionData.html "struct optionstratlib::chains::OptionData"){.struct}, &[OptionData](../struct.OptionData.html "struct optionstratlib::chains::OptionData"){.struct}, &[OptionData](../struct.OptionData.html "struct optionstratlib::chains::OptionData"){.struct}, &[OptionData](../struct.OptionData.html "struct optionstratlib::chains::OptionData"){.struct})\> {#pub-fn-get_quad_inclusive_iter-self---impl-iteratoritem-optiondata-optiondata-optiondata-optiondata .code-header}
:::

:::: docblock
Returns an iterator that generates inclusive quadruples of option
combinations from the `OptionChain`.

This function iterates over all combinations of four options from the
`options` collection, including those where the same option may be
included more than once.

##### [§](#returns-20){.doc-anchor}Returns

An iterator producing tuples with four references to `OptionData`,
potentially including repeated elements (e.g.,
`(option1, option2, option1, option4)`).

##### [§](#example-7){.doc-anchor}Example

::: example-wrap
``` {.rust .rust-example-rendered}
use tracing::info;
use optionstratlib::chains::chain::OptionChain;
use optionstratlib::Positive;
use optionstratlib::pos;
let mut option_chain = OptionChain::new("TEST", pos!(100.0), "2024-01-01".to_string(), None, None);
for (option1, option2, option3, option4) in option_chain.get_quad_inclusive_iter() {
    info!("{:?}, {:?}, {:?}, {:?}", option1, option2, option3, option4);
}
```
:::
::::

::: {#method.get_call_price .section .method}
[Source](../../../src/optionstratlib/chains/chain.rs.html#1603-1609){.src
.rightside}

#### pub fn [get_call_price](#method.get_call_price){.fn}(&self, strike: [Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}) -\> [Option](https://doc.rust-lang.org/1.86.0/core/option/enum.Option.html "enum core::option::Option"){.enum}\<Decimal\> {#pub-fn-get_call_priceself-strike-positive---optiondecimal .code-header}
:::

::: docblock
Retrieves the call option price for a specific strike price

This helper method finds and returns the ask price of a call option at
the specified strike price from the option chain.

##### [§](#arguments-10){.doc-anchor}Arguments

- `strike` - The strike price to look up

##### [§](#returns-21){.doc-anchor}Returns

- `Some(Decimal)` - The call option ask price if found
- `None` - If no option exists at the specified strike or if the price
  is not available

##### [§](#notes){.doc-anchor}Notes

- Uses the ask price as it represents the cost to buy the option
- Converts the price to Decimal for consistency in calculations
:::

::: {#method.get_atm_implied_volatility .section .method}
[Source](../../../src/optionstratlib/chains/chain.rs.html#1636-1645){.src
.rightside}

#### pub fn [get_atm_implied_volatility](#method.get_atm_implied_volatility){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<Decimal, [String](https://doc.rust-lang.org/1.86.0/alloc/string/struct.String.html "struct alloc::string::String"){.struct}\> {#pub-fn-get_atm_implied_volatilityself---resultdecimal-string .code-header}
:::

:::: docblock
Retrieves the implied volatility for the at-the-money (ATM) option

Finds the option with strike price equal to the current underlying price
and returns its implied volatility.

##### [§](#returns-22){.doc-anchor}Returns

- `Ok(Decimal)` - The ATM implied volatility if found
- `Err(String)` - Error message if ATM implied volatility is not
  available

##### [§](#examples-2){.doc-anchor}Examples

::: example-wrap
``` {.rust .rust-example-rendered}
use tracing::info;
use optionstratlib::chains::chain::OptionChain;
use optionstratlib::pos;
let chain = OptionChain::new("TEST", pos!(100.0), "2024-12-31".to_string(), None, None);
match chain.get_atm_implied_volatility() {
    Ok(vol) => info!("ATM volatility: {}", vol),
    Err(e) => info!("Error: {}", e),
}
```
:::

##### [§](#notes-1){.doc-anchor}Notes

- ATM strike is defined as the strike equal to the current underlying
  price
- Important for volatility skew calculations and option pricing
- Returns implied volatility as a decimal for precise calculations
::::

::: {#method.atm_implied_volatility .section .method}
[Source](../../../src/optionstratlib/chains/chain.rs.html#1663-1666){.src
.rightside}

#### pub fn [atm_implied_volatility](#method.atm_implied_volatility){.fn}( &self, ) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<&[Option](https://doc.rust-lang.org/1.86.0/core/option/enum.Option.html "enum core::option::Option"){.enum}\<[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\>, [Box](https://doc.rust-lang.org/1.86.0/alloc/boxed/struct.Box.html "struct alloc::boxed::Box"){.struct}\<dyn [Error](https://doc.rust-lang.org/1.86.0/core/error/trait.Error.html "trait core::error::Error"){.trait}\>\> {#pub-fn-atm_implied_volatility-self---resultoptionpositive-boxdyn-error .code-header}
:::

::: docblock
Retrieves the At-The-Money (ATM) implied volatility.

This function retrieves the implied volatility of the ATM option. It
calls `self.atm_option_data()` to find the ATM option and then returns a
reference to its implied volatility.

##### [§](#returns-23){.doc-anchor}Returns

- `Ok(&Option<Positive>)` - If the ATM option is found, returns a
  reference to its implied volatility, which is an `Option<Positive>`.
- `Err(Box<dyn Error>)` - If the ATM option cannot be found, returns an
  error.

##### [§](#errors-1){.doc-anchor}Errors

This function returns an error if the underlying `atm_option_data()`
call fails, which can happen if the option chain is empty or no suitable
ATM option is found.
:::

::: {#method.gamma_exposure .section .method}
[Source](../../../src/optionstratlib/chains/chain.rs.html#1688-1700){.src
.rightside}

#### pub fn [gamma_exposure](#method.gamma_exposure){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<Decimal, [ChainError](../../error/chains/enum.ChainError.html "enum optionstratlib::error::chains::ChainError"){.enum}\> {#pub-fn-gamma_exposureself---resultdecimal-chainerror .code-header}
:::

::: docblock
Calculates the total gamma exposure for all options in the chain.

Gamma exposure represents the aggregate rate of change in the delta
value with respect to changes in the underlying asset's price across all
options. It measures the second-order price sensitivity and indicates
how the delta will change as the underlying price moves.

##### [§](#returns-24){.doc-anchor}Returns

- `Result<Decimal, ChainError>` - The aggregate gamma value, or an error
  if calculation fails

##### [§](#errors-2){.doc-anchor}Errors

Returns a `ChainError` if:

- Any option's gamma calculation fails
- Options greeks are not initialized

##### [§](#note-4){.doc-anchor}Note

This method requires options greeks to be initialized first by calling
the `update_greeks` method.
:::

::: {#method.delta_exposure .section .method}
[Source](../../../src/optionstratlib/chains/chain.rs.html#1721-1733){.src
.rightside}

#### pub fn [delta_exposure](#method.delta_exposure){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<Decimal, [ChainError](../../error/chains/enum.ChainError.html "enum optionstratlib::error::chains::ChainError"){.enum}\> {#pub-fn-delta_exposureself---resultdecimal-chainerror .code-header}
:::

::: docblock
Calculates the total delta exposure for all options in the chain.

Delta exposure represents the aggregate sensitivity of option prices to
changes in the underlying asset's price. A delta exposure of 1.0 means
that for every \$1 change in the underlying asset, the options portfolio
will change by \$1 in the same direction.

##### [§](#returns-25){.doc-anchor}Returns

- `Result<Decimal, ChainError>` - The aggregate delta value, or an error
  if calculation fails

##### [§](#errors-3){.doc-anchor}Errors

Returns a `ChainError` if:

- Any option's delta calculation fails
- Options greeks are not initialized

##### [§](#note-5){.doc-anchor}Note

This method requires options greeks to be initialized first by calling
the `update_greeks` method.
:::

::: {#method.vega_exposure .section .method}
[Source](../../../src/optionstratlib/chains/chain.rs.html#1754-1766){.src
.rightside}

#### pub fn [vega_exposure](#method.vega_exposure){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<Decimal, [ChainError](../../error/chains/enum.ChainError.html "enum optionstratlib::error::chains::ChainError"){.enum}\> {#pub-fn-vega_exposureself---resultdecimal-chainerror .code-header}
:::

::: docblock
Calculates the total vega exposure for all options in the chain.

Vega exposure represents the aggregate sensitivity of option prices to
changes in the implied volatility of the underlying asset. It measures
how much option prices will change for a 1% change in implied
volatility.

##### [§](#returns-26){.doc-anchor}Returns

- `Result<Decimal, ChainError>` - The aggregate vega value, or an error
  if calculation fails

##### [§](#errors-4){.doc-anchor}Errors

Returns a `ChainError` if:

- Any option's vega calculation fails
- Options greeks are not initialized

##### [§](#note-6){.doc-anchor}Note

This method requires options greeks to be initialized first by calling
the `update_greeks` method.
:::

::: {#method.theta_exposure .section .method}
[Source](../../../src/optionstratlib/chains/chain.rs.html#1787-1799){.src
.rightside}

#### pub fn [theta_exposure](#method.theta_exposure){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<Decimal, [ChainError](../../error/chains/enum.ChainError.html "enum optionstratlib::error::chains::ChainError"){.enum}\> {#pub-fn-theta_exposureself---resultdecimal-chainerror .code-header}
:::

::: docblock
Calculates the total theta exposure for all options in the chain.

Theta exposure represents the aggregate rate of time decay in option
prices as they approach expiration. It measures how much value the
options portfolio will lose per day, holding all other factors constant.

##### [§](#returns-27){.doc-anchor}Returns

- `Result<Decimal, ChainError>` - The aggregate theta value, or an error
  if calculation fails

##### [§](#errors-5){.doc-anchor}Errors

Returns a `ChainError` if:

- Any option's theta calculation fails
- Options greeks are not initialized

##### [§](#note-7){.doc-anchor}Note

This method requires options greeks to be initialized first by calling
the `update_greeks` method.
:::

::: {#method.gamma_curve .section .method}
[Source](../../../src/optionstratlib/chains/chain.rs.html#1815-1817){.src
.rightside}

#### pub fn [gamma_curve](#method.gamma_curve){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Curve](../../curves/struct.Curve.html "struct optionstratlib::curves::Curve"){.struct}, [CurveError](../../error/curves/enum.CurveError.html "enum optionstratlib::error::curves::CurveError"){.enum}\> {#pub-fn-gamma_curveself---resultcurve-curveerror .code-header}
:::

::: docblock
Generates a gamma curve for visualization and analysis.

Creates a curve representing gamma values across different strike prices
or other relevant parameters for long call options in the chain.

##### [§](#returns-28){.doc-anchor}Returns

- `Result<Curve, CurveError>` - A curve object containing gamma data
  points, or an error if curve generation fails

##### [§](#errors-6){.doc-anchor}Errors

Returns a `CurveError` if the curve cannot be generated due to missing
data or calculation errors
:::

::: {#method.delta_curve .section .method}
[Source](../../../src/optionstratlib/chains/chain.rs.html#1833-1835){.src
.rightside}

#### pub fn [delta_curve](#method.delta_curve){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Curve](../../curves/struct.Curve.html "struct optionstratlib::curves::Curve"){.struct}, [CurveError](../../error/curves/enum.CurveError.html "enum optionstratlib::error::curves::CurveError"){.enum}\> {#pub-fn-delta_curveself---resultcurve-curveerror .code-header}
:::

::: docblock
Generates a delta curve for visualization and analysis.

Creates a curve representing delta values across different strike prices
or other relevant parameters for long call options in the chain.

##### [§](#returns-29){.doc-anchor}Returns

- `Result<Curve, CurveError>` - A curve object containing delta data
  points, or an error if curve generation fails

##### [§](#errors-7){.doc-anchor}Errors

Returns a `CurveError` if the curve cannot be generated due to missing
data or calculation errors
:::

::: {#method.vega_curve .section .method}
[Source](../../../src/optionstratlib/chains/chain.rs.html#1851-1853){.src
.rightside}

#### pub fn [vega_curve](#method.vega_curve){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Curve](../../curves/struct.Curve.html "struct optionstratlib::curves::Curve"){.struct}, [CurveError](../../error/curves/enum.CurveError.html "enum optionstratlib::error::curves::CurveError"){.enum}\> {#pub-fn-vega_curveself---resultcurve-curveerror .code-header}
:::

::: docblock
Generates a vega curve for visualization and analysis.

Creates a curve representing vega values across different strike prices
or other relevant parameters for long call options in the chain.

##### [§](#returns-30){.doc-anchor}Returns

- `Result<Curve, CurveError>` - A curve object containing vega data
  points, or an error if curve generation fails

##### [§](#errors-8){.doc-anchor}Errors

Returns a `CurveError` if the curve cannot be generated due to missing
data or calculation errors
:::

::: {#method.theta_curve .section .method}
[Source](../../../src/optionstratlib/chains/chain.rs.html#1869-1871){.src
.rightside}

#### pub fn [theta_curve](#method.theta_curve){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Curve](../../curves/struct.Curve.html "struct optionstratlib::curves::Curve"){.struct}, [CurveError](../../error/curves/enum.CurveError.html "enum optionstratlib::error::curves::CurveError"){.enum}\> {#pub-fn-theta_curveself---resultcurve-curveerror .code-header}
:::

::: docblock
Generates a theta curve for visualization and analysis.

Creates a curve representing theta values across different strike prices
or other relevant parameters for long call options in the chain.

##### [§](#returns-31){.doc-anchor}Returns

- `Result<Curve, CurveError>` - A curve object containing theta data
  points, or an error if curve generation fails

##### [§](#errors-9){.doc-anchor}Errors

Returns a `CurveError` if the curve cannot be generated due to missing
data or calculation errors
:::

::: {#method.update_expiration_date .section .method}
[Source](../../../src/optionstratlib/chains/chain.rs.html#1898-1901){.src
.rightside}

#### pub fn [update_expiration_date](#method.update_expiration_date){.fn}(&mut self, expiration: [String](https://doc.rust-lang.org/1.86.0/alloc/string/struct.String.html "struct alloc::string::String"){.struct}) {#pub-fn-update_expiration_datemut-self-expiration-string .code-header}
:::

:::: docblock
Updates the expiration date for the option chain and recalculates
Greeks.

This method changes the expiration date of the option chain to the
provided value and then triggers a recalculation of all Greek values for
every option in the chain. The Greeks are financial measures that
indicate how option prices are expected to change in response to
different factors.

##### [§](#parameters-1){.doc-anchor}Parameters

- `expiration` - A string representing the new expiration date for the
  option chain. This should be in a standard date format compatible with
  the system.

##### [§](#effects){.doc-anchor}Effects

- Updates the `expiration_date` field of the option chain.
- Calls `update_greeks()` to recalculate and update the Greek values for
  all options in the chain based on the new expiration date.

##### [§](#example-8){.doc-anchor}Example

::: example-wrap
``` {.rust .rust-example-rendered}
use optionstratlib::chains::chain::OptionChain;
let mut chain = OptionChain::new("AAPL", Default::default(), "".to_string(), None, None);
chain.update_expiration_date("2023-12-15".to_string());
```
:::
::::

::: {#method.get_expiration_date .section .method}
[Source](../../../src/optionstratlib/chains/chain.rs.html#1911-1913){.src
.rightside}

#### pub fn [get_expiration_date](#method.get_expiration_date){.fn}(&self) -\> [String](https://doc.rust-lang.org/1.86.0/alloc/string/struct.String.html "struct alloc::string::String"){.struct} {#pub-fn-get_expiration_dateself---string .code-header}
:::

::: docblock
Retrieves the expiration date of the option chain.

This method returns the expiration date associated with the option chain
as a `String`. The expiration date represents the date on which the
options in the chain will expire.

##### [§](#returns-32){.doc-anchor}Returns

A `String` representing the expiration date of the option chain.
:::

::: {#method.get_position_with_delta .section .method}
[Source](../../../src/optionstratlib/chains/chain.rs.html#1983-2046){.src
.rightside}

#### pub fn [get_position_with_delta](#method.get_position_with_delta){.fn}( &self, target_delta: [Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, side: [Side](../../model/types/enum.Side.html "enum optionstratlib::model::types::Side"){.enum}, option_style: [OptionStyle](../../model/types/enum.OptionStyle.html "enum optionstratlib::model::types::OptionStyle"){.enum}, ) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Position](../../model/position/struct.Position.html "struct optionstratlib::model::position::Position"){.struct}, [ChainError](../../error/chains/enum.ChainError.html "enum optionstratlib::error::chains::ChainError"){.enum}\> {#pub-fn-get_position_with_delta-self-target_delta-positive-side-side-option_style-optionstyle---resultposition-chainerror .code-header}
:::

::: docblock
Retrieves a `Position` with a delta closest to the specified
`target_delta`.

This function searches the option chain for an option whose delta is
less than or equal to the `target_delta`. It then selects the option
with the highest delta value (for calls) or the most negative delta
value (for puts) that meets this criteria. A `Position` is constructed
from the selected option.

##### [§](#arguments-11){.doc-anchor}Arguments

- `target_delta` - The target delta value to search for.
- `side` - The side of the position (Long or Short).
- `option_style` - The style of the option (Call or Put).

##### [§](#returns-33){.doc-anchor}Returns

A `Result` containing the `Position` if a suitable option is found, or a
`ChainError` if no option with a delta less than or equal to the
`target_delta` is found.

##### [§](#errors-10){.doc-anchor}Errors

Returns a `ChainError::OptionDataError` with
`OptionDataErrorKind::InvalidDelta` if no option is found with a delta
less than or equal to the specified `target_delta`.
:::
:::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
:::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::

## Trait Implementations[§](#trait-implementations){.anchor} {#trait-implementations .section-header}

::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::: {#trait-implementations-list}
::: {#impl-AtmIvProvider-for-OptionChain .section .impl}
[Source](../../../src/optionstratlib/volatility/traits.rs.html#108-118){.src
.rightside}[§](#impl-AtmIvProvider-for-OptionChain){.anchor}

### impl [AtmIvProvider](../../volatility/trait.AtmIvProvider.html "trait optionstratlib::volatility::AtmIvProvider"){.trait} for [OptionChain](struct.OptionChain.html "struct optionstratlib::chains::chain::OptionChain"){.struct} {#impl-atmivprovider-for-optionchain .code-header}
:::

::::: impl-items
::: {#method.atm_iv .section .method .trait-impl}
[Source](../../../src/optionstratlib/volatility/traits.rs.html#109-117){.src
.rightside}[§](#method.atm_iv){.anchor}

#### fn [atm_iv](../../volatility/trait.AtmIvProvider.html#tymethod.atm_iv){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<&[Option](https://doc.rust-lang.org/1.86.0/core/option/enum.Option.html "enum core::option::Option"){.enum}\<[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\>, [Box](https://doc.rust-lang.org/1.86.0/alloc/boxed/struct.Box.html "struct alloc::boxed::Box"){.struct}\<dyn [Error](https://doc.rust-lang.org/1.86.0/core/error/trait.Error.html "trait core::error::Error"){.trait}\>\> {#fn-atm_ivself---resultoptionpositive-boxdyn-error .code-header}
:::

::: docblock
Get the at-the-money implied volatility [Read
more](../../volatility/trait.AtmIvProvider.html#tymethod.atm_iv)
:::
:::::

::: {#impl-BasicCurves-for-OptionChain .section .impl}
[Source](../../../src/optionstratlib/chains/chain.rs.html#2318-2359){.src
.rightside}[§](#impl-BasicCurves-for-OptionChain){.anchor}

### impl [BasicCurves](../../curves/trait.BasicCurves.html "trait optionstratlib::curves::BasicCurves"){.trait} for [OptionChain](struct.OptionChain.html "struct optionstratlib::chains::chain::OptionChain"){.struct} {#impl-basiccurves-for-optionchain .code-header}
:::

::::::: impl-items
::: {#method.curve .section .method .trait-impl}
[Source](../../../src/optionstratlib/chains/chain.rs.html#2319-2358){.src
.rightside}[§](#method.curve){.anchor}

#### fn [curve](../../curves/trait.BasicCurves.html#tymethod.curve){.fn}( &self, axis: &[BasicAxisTypes](../../model/enum.BasicAxisTypes.html "enum optionstratlib::model::BasicAxisTypes"){.enum}, option_style: &[OptionStyle](../../model/types/enum.OptionStyle.html "enum optionstratlib::model::types::OptionStyle"){.enum}, side: &[Side](../../model/types/enum.Side.html "enum optionstratlib::model::types::Side"){.enum}, ) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Curve](../../curves/struct.Curve.html "struct optionstratlib::curves::Curve"){.struct}, [CurveError](../../error/curves/enum.CurveError.html "enum optionstratlib::error::curves::CurveError"){.enum}\> {#fn-curve-self-axis-basicaxistypes-option_style-optionstyle-side-side---resultcurve-curveerror .code-header}
:::

::: docblock
Generates a curve for the specified axis type, option style, and market
side. [Read more](../../curves/trait.BasicCurves.html#tymethod.curve)
:::

::: {#method.get_curve_strike_versus .section .method .trait-impl}
[Source](../../../src/optionstratlib/curves/basic.rs.html#67-93){.src
.rightside}[§](#method.get_curve_strike_versus){.anchor}

#### fn [get_curve_strike_versus](../../curves/trait.BasicCurves.html#method.get_curve_strike_versus){.fn}( &self, axis: &[BasicAxisTypes](../../model/enum.BasicAxisTypes.html "enum optionstratlib::model::BasicAxisTypes"){.enum}, option: &[Arc](https://doc.rust-lang.org/1.86.0/alloc/sync/struct.Arc.html "struct alloc::sync::Arc"){.struct}\<[Options](../../model/option/struct.Options.html "struct optionstratlib::model::option::Options"){.struct}\>, ) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<(Decimal, Decimal), [CurveError](../../error/curves/enum.CurveError.html "enum optionstratlib::error::curves::CurveError"){.enum}\> {#fn-get_curve_strike_versus-self-axis-basicaxistypes-option-arcoptions---resultdecimal-decimal-curveerror .code-header}
:::

::: docblock
Generates coordinate pairs for a specific option and axis type. [Read
more](../../curves/trait.BasicCurves.html#method.get_curve_strike_versus)
:::
:::::::

::: {#impl-BasicSurfaces-for-OptionChain .section .impl}
[Source](../../../src/optionstratlib/chains/chain.rs.html#2361-2424){.src
.rightside}[§](#impl-BasicSurfaces-for-OptionChain){.anchor}

### impl [BasicSurfaces](../../surfaces/trait.BasicSurfaces.html "trait optionstratlib::surfaces::BasicSurfaces"){.trait} for [OptionChain](struct.OptionChain.html "struct optionstratlib::chains::chain::OptionChain"){.struct} {#impl-basicsurfaces-for-optionchain .code-header}
:::

::::::::: impl-items
::: {#method.surface .section .method .trait-impl}
[Source](../../../src/optionstratlib/chains/chain.rs.html#2362-2423){.src
.rightside}[§](#method.surface){.anchor}

#### fn [surface](../../surfaces/trait.BasicSurfaces.html#tymethod.surface){.fn}( &self, axis: &[BasicAxisTypes](../../model/enum.BasicAxisTypes.html "enum optionstratlib::model::BasicAxisTypes"){.enum}, option_style: &[OptionStyle](../../model/types/enum.OptionStyle.html "enum optionstratlib::model::types::OptionStyle"){.enum}, volatility: [Option](https://doc.rust-lang.org/1.86.0/core/option/enum.Option.html "enum core::option::Option"){.enum}\<[Vec](https://doc.rust-lang.org/1.86.0/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\>\>, side: &[Side](../../model/types/enum.Side.html "enum optionstratlib::model::types::Side"){.enum}, ) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Surface](../../surfaces/struct.Surface.html "struct optionstratlib::surfaces::Surface"){.struct}, [SurfaceError](../../error/enum.SurfaceError.html "enum optionstratlib::error::SurfaceError"){.enum}\> {#fn-surface-self-axis-basicaxistypes-option_style-optionstyle-volatility-optionvecpositive-side-side---resultsurface-surfaceerror .code-header}
:::

::: docblock
Creates a surface visualization based on the specified axis type and
option parameters. [Read
more](../../surfaces/trait.BasicSurfaces.html#tymethod.surface)
:::

::: {#method.get_surface_strike_versus .section .method .trait-impl}
[Source](../../../src/optionstratlib/surfaces/basic.rs.html#64-107){.src
.rightside}[§](#method.get_surface_strike_versus){.anchor}

#### fn [get_surface_strike_versus](../../surfaces/trait.BasicSurfaces.html#method.get_surface_strike_versus){.fn}( &self, axis: &[BasicAxisTypes](../../model/enum.BasicAxisTypes.html "enum optionstratlib::model::BasicAxisTypes"){.enum}, option: &[Arc](https://doc.rust-lang.org/1.86.0/alloc/sync/struct.Arc.html "struct alloc::sync::Arc"){.struct}\<[Options](../../model/option/struct.Options.html "struct optionstratlib::model::option::Options"){.struct}\>, ) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<(Decimal, Decimal, Decimal), [SurfaceError](../../error/enum.SurfaceError.html "enum optionstratlib::error::SurfaceError"){.enum}\> {#fn-get_surface_strike_versus-self-axis-basicaxistypes-option-arcoptions---resultdecimal-decimal-decimal-surfaceerror .code-header}
:::

::: docblock
Calculates the relationship between strike price, implied volatility,
and a selected option metric for a given option. [Read
more](../../surfaces/trait.BasicSurfaces.html#method.get_surface_strike_versus)
:::

::: {#method.get_surface_volatility_versus .section .method .trait-impl}
[Source](../../../src/optionstratlib/surfaces/basic.rs.html#131-175){.src
.rightside}[§](#method.get_surface_volatility_versus){.anchor}

#### fn [get_surface_volatility_versus](../../surfaces/trait.BasicSurfaces.html#method.get_surface_volatility_versus){.fn}( &self, axis: &[BasicAxisTypes](../../model/enum.BasicAxisTypes.html "enum optionstratlib::model::BasicAxisTypes"){.enum}, option: &[Arc](https://doc.rust-lang.org/1.86.0/alloc/sync/struct.Arc.html "struct alloc::sync::Arc"){.struct}\<[Options](../../model/option/struct.Options.html "struct optionstratlib::model::option::Options"){.struct}\>, volatility: [Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, ) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<(Decimal, Decimal, Decimal), [SurfaceError](../../error/enum.SurfaceError.html "enum optionstratlib::error::SurfaceError"){.enum}\> {#fn-get_surface_volatility_versus-self-axis-basicaxistypes-option-arcoptions-volatility-positive---resultdecimal-decimal-decimal-surfaceerror .code-header}
:::

::: docblock
Calculates the relationship between strike price, a specified volatility
value, and a selected option metric for a given option. [Read
more](../../surfaces/trait.BasicSurfaces.html#method.get_surface_volatility_versus)
:::
:::::::::

::: {#impl-Clone-for-OptionChain .section .impl}
[Source](../../../src/optionstratlib/chains/chain.rs.html#72){.src
.rightside}[§](#impl-Clone-for-OptionChain){.anchor}

### impl [Clone](https://doc.rust-lang.org/1.86.0/core/clone/trait.Clone.html "trait core::clone::Clone"){.trait} for [OptionChain](struct.OptionChain.html "struct optionstratlib::chains::chain::OptionChain"){.struct} {#impl-clone-for-optionchain .code-header}
:::

::::::: impl-items
::: {#method.clone .section .method .trait-impl}
[Source](../../../src/optionstratlib/chains/chain.rs.html#72){.src
.rightside}[§](#method.clone){.anchor}

#### fn [clone](https://doc.rust-lang.org/1.86.0/core/clone/trait.Clone.html#tymethod.clone){.fn}(&self) -\> [OptionChain](struct.OptionChain.html "struct optionstratlib::chains::chain::OptionChain"){.struct} {#fn-cloneself---optionchain .code-header}
:::

::: docblock
Returns a copy of the value. [Read
more](https://doc.rust-lang.org/1.86.0/core/clone/trait.Clone.html#tymethod.clone)
:::

::: {#method.clone_from .section .method .trait-impl}
[[1.0.0]{.since title="Stable since Rust version 1.0.0"} ·
[Source](https://doc.rust-lang.org/1.86.0/src/core/clone.rs.html#174){.src}]{.rightside}[§](#method.clone_from){.anchor}

#### fn [clone_from](https://doc.rust-lang.org/1.86.0/core/clone/trait.Clone.html#method.clone_from){.fn}(&mut self, source: &Self) {#fn-clone_frommut-self-source-self .code-header}
:::

::: docblock
Performs copy-assignment from `source`. [Read
more](https://doc.rust-lang.org/1.86.0/core/clone/trait.Clone.html#method.clone_from)
:::
:::::::

::: {#impl-Debug-for-OptionChain .section .impl}
[Source](../../../src/optionstratlib/chains/chain.rs.html#72){.src
.rightside}[§](#impl-Debug-for-OptionChain){.anchor}

### impl [Debug](https://doc.rust-lang.org/1.86.0/core/fmt/trait.Debug.html "trait core::fmt::Debug"){.trait} for [OptionChain](struct.OptionChain.html "struct optionstratlib::chains::chain::OptionChain"){.struct} {#impl-debug-for-optionchain .code-header}
:::

::::: impl-items
::: {#method.fmt .section .method .trait-impl}
[Source](../../../src/optionstratlib/chains/chain.rs.html#72){.src
.rightside}[§](#method.fmt){.anchor}

#### fn [fmt](https://doc.rust-lang.org/1.86.0/core/fmt/trait.Debug.html#tymethod.fmt){.fn}(&self, f: &mut [Formatter](https://doc.rust-lang.org/1.86.0/core/fmt/struct.Formatter.html "struct core::fmt::Formatter"){.struct}\<\'\_\>) -\> [Result](https://doc.rust-lang.org/1.86.0/core/fmt/type.Result.html "type core::fmt::Result"){.type} {#fn-fmtself-f-mut-formatter_---result .code-header}
:::

::: docblock
Formats the value using the given formatter. [Read
more](https://doc.rust-lang.org/1.86.0/core/fmt/trait.Debug.html#tymethod.fmt)
:::
:::::

::: {#impl-Deserialize%3C'de%3E-for-OptionChain .section .impl}
[Source](../../../src/optionstratlib/chains/chain.rs.html#118-225){.src
.rightside}[§](#impl-Deserialize%3C'de%3E-for-OptionChain){.anchor}

### impl\<\'de\> [Deserialize](https://docs.rs/serde/1.0.219/serde/de/trait.Deserialize.html "trait serde::de::Deserialize"){.trait}\<\'de\> for [OptionChain](struct.OptionChain.html "struct optionstratlib::chains::chain::OptionChain"){.struct} {#implde-deserializede-for-optionchain .code-header}
:::

:::::: impl-items
:::: {#method.deserialize .section .method .trait-impl}
[Source](../../../src/optionstratlib/chains/chain.rs.html#119-224){.src
.rightside}[§](#method.deserialize){.anchor}

#### fn [deserialize](https://docs.rs/serde/1.0.219/serde/de/trait.Deserialize.html#tymethod.deserialize){.fn}\<D\>(deserializer: D) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<Self, D::[Error](https://docs.rs/serde/1.0.219/serde/de/trait.Deserializer.html#associatedtype.Error "type serde::de::Deserializer::Error"){.associatedtype}\> {#fn-deserializeddeserializer-d---resultself-derror .code-header}

::: where
where D:
[Deserializer](https://docs.rs/serde/1.0.219/serde/de/trait.Deserializer.html "trait serde::de::Deserializer"){.trait}\<\'de\>,
:::
::::

::: docblock
Deserialize this value from the given Serde deserializer. [Read
more](https://docs.rs/serde/1.0.219/serde/de/trait.Deserialize.html#tymethod.deserialize)
:::
::::::

::: {#impl-Display-for-OptionChain .section .impl}
[Source](../../../src/optionstratlib/chains/chain.rs.html#2233-2271){.src
.rightside}[§](#impl-Display-for-OptionChain){.anchor}

### impl [Display](https://doc.rust-lang.org/1.86.0/core/fmt/trait.Display.html "trait core::fmt::Display"){.trait} for [OptionChain](struct.OptionChain.html "struct optionstratlib::chains::chain::OptionChain"){.struct} {#impl-display-for-optionchain .code-header}
:::

::::: impl-items
::: {#method.fmt-1 .section .method .trait-impl}
[Source](../../../src/optionstratlib/chains/chain.rs.html#2234-2270){.src
.rightside}[§](#method.fmt-1){.anchor}

#### fn [fmt](https://doc.rust-lang.org/1.86.0/core/fmt/trait.Display.html#tymethod.fmt){.fn}(&self, f: &mut [Formatter](https://doc.rust-lang.org/1.86.0/core/fmt/struct.Formatter.html "struct core::fmt::Formatter"){.struct}\<\'\_\>) -\> [Result](https://doc.rust-lang.org/1.86.0/core/fmt/type.Result.html "type core::fmt::Result"){.type} {#fn-fmtself-f-mut-formatter_---result-1 .code-header}
:::

::: docblock
Formats the value using the given formatter. [Read
more](https://doc.rust-lang.org/1.86.0/core/fmt/trait.Display.html#tymethod.fmt)
:::
:::::

::: {#impl-From%3C%26OptionChain%3E-for-Positive .section .impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#609-613){.src
.rightside}[§](#impl-From%3C%26OptionChain%3E-for-Positive){.anchor}

### impl [From](https://doc.rust-lang.org/1.86.0/core/convert/trait.From.html "trait core::convert::From"){.trait}\<&[OptionChain](struct.OptionChain.html "struct optionstratlib::chains::chain::OptionChain"){.struct}\> for [Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#impl-fromoptionchain-for-positive .code-header}
:::

::::: impl-items
::: {#method.from .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#610-612){.src
.rightside}[§](#method.from){.anchor}

#### fn [from](https://doc.rust-lang.org/1.86.0/core/convert/trait.From.html#tymethod.from){.fn}(value: &[OptionChain](struct.OptionChain.html "struct optionstratlib::chains::chain::OptionChain"){.struct}) -\> Self {#fn-fromvalue-optionchain---self .code-header}
:::

::: docblock
Converts to this type from the input type.
:::
:::::

::: {#impl-From%3COptionChain%3E-for-Positive .section .impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#615-619){.src
.rightside}[§](#impl-From%3COptionChain%3E-for-Positive){.anchor}

### impl [From](https://doc.rust-lang.org/1.86.0/core/convert/trait.From.html "trait core::convert::From"){.trait}\<[OptionChain](struct.OptionChain.html "struct optionstratlib::chains::chain::OptionChain"){.struct}\> for [Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#impl-fromoptionchain-for-positive-1 .code-header}
:::

::::: impl-items
::: {#method.from-1 .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#616-618){.src
.rightside}[§](#method.from-1){.anchor}

#### fn [from](https://doc.rust-lang.org/1.86.0/core/convert/trait.From.html#tymethod.from){.fn}(value: [OptionChain](struct.OptionChain.html "struct optionstratlib::chains::chain::OptionChain"){.struct}) -\> Self {#fn-fromvalue-optionchain---self-1 .code-header}
:::

::: docblock
Converts to this type from the input type.
:::
:::::

::: {#impl-Len-for-OptionChain .section .impl}
[Source](../../../src/optionstratlib/chains/chain.rs.html#2049-2057){.src
.rightside}[§](#impl-Len-for-OptionChain){.anchor}

### impl [Len](../../utils/trait.Len.html "trait optionstratlib::utils::Len"){.trait} for [OptionChain](struct.OptionChain.html "struct optionstratlib::chains::chain::OptionChain"){.struct} {#impl-len-for-optionchain .code-header}
:::

::::::: impl-items
::: {#method.len .section .method .trait-impl}
[Source](../../../src/optionstratlib/chains/chain.rs.html#2050-2052){.src
.rightside}[§](#method.len){.anchor}

#### fn [len](../../utils/trait.Len.html#tymethod.len){.fn}(&self) -\> [usize](https://doc.rust-lang.org/1.86.0/std/primitive.usize.html){.primitive} {#fn-lenself---usize .code-header}
:::

::: docblock
Returns the number of elements in the collection or the size of the
object. [Read more](../../utils/trait.Len.html#tymethod.len)
:::

::: {#method.is_empty .section .method .trait-impl}
[Source](../../../src/optionstratlib/chains/chain.rs.html#2054-2056){.src
.rightside}[§](#method.is_empty){.anchor}

#### fn [is_empty](../../utils/trait.Len.html#method.is_empty){.fn}(&self) -\> [bool](https://doc.rust-lang.org/1.86.0/std/primitive.bool.html){.primitive} {#fn-is_emptyself---bool .code-header}
:::

::: docblock
Returns `true` if the collection contains no elements or the object has
zero size. [Read more](../../utils/trait.Len.html#method.is_empty)
:::
:::::::

::: {#impl-OptionChainParams-for-OptionChain .section .impl}
[Source](../../../src/optionstratlib/chains/chain.rs.html#2059-2078){.src
.rightside}[§](#impl-OptionChainParams-for-OptionChain){.anchor}

### impl [OptionChainParams](../utils/trait.OptionChainParams.html "trait optionstratlib::chains::utils::OptionChainParams"){.trait} for [OptionChain](struct.OptionChain.html "struct optionstratlib::chains::chain::OptionChain"){.struct} {#impl-optionchainparams-for-optionchain .code-header}
:::

::::: impl-items
::: {#method.get_params .section .method .trait-impl}
[Source](../../../src/optionstratlib/chains/chain.rs.html#2060-2077){.src
.rightside}[§](#method.get_params){.anchor}

#### fn [get_params](../utils/trait.OptionChainParams.html#tymethod.get_params){.fn}( &self, strike_price: [Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, ) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[OptionDataPriceParams](../utils/struct.OptionDataPriceParams.html "struct optionstratlib::chains::utils::OptionDataPriceParams"){.struct}, [ChainError](../../error/chains/enum.ChainError.html "enum optionstratlib::error::chains::ChainError"){.enum}\> {#fn-get_params-self-strike_price-positive---resultoptiondatapriceparams-chainerror .code-header}
:::

::: docblock
Retrieves the option pricing parameters for a given strike price. [Read
more](../utils/trait.OptionChainParams.html#tymethod.get_params)
:::
:::::

::: {#impl-RNDAnalysis-for-OptionChain .section .impl}
[Source](../../../src/optionstratlib/chains/chain.rs.html#2080-2231){.src
.rightside}[§](#impl-RNDAnalysis-for-OptionChain){.anchor}

### impl [RNDAnalysis](../trait.RNDAnalysis.html "trait optionstratlib::chains::RNDAnalysis"){.trait} for [OptionChain](struct.OptionChain.html "struct optionstratlib::chains::chain::OptionChain"){.struct} {#impl-rndanalysis-for-optionchain .code-header}
:::

::::::: impl-items
::: {#method.calculate_rnd .section .method .trait-impl}
[Source](../../../src/optionstratlib/chains/chain.rs.html#2092-2202){.src
.rightside}[§](#method.calculate_rnd){.anchor}

#### fn [calculate_rnd](../trait.RNDAnalysis.html#tymethod.calculate_rnd){.fn}( &self, params: &[RNDParameters](../struct.RNDParameters.html "struct optionstratlib::chains::RNDParameters"){.struct}, ) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[RNDResult](../struct.RNDResult.html "struct optionstratlib::chains::RNDResult"){.struct}, [Box](https://doc.rust-lang.org/1.86.0/alloc/boxed/struct.Box.html "struct alloc::boxed::Box"){.struct}\<dyn [Error](https://doc.rust-lang.org/1.86.0/core/error/trait.Error.html "trait core::error::Error"){.trait}\>\> {#fn-calculate_rnd-self-params-rndparameters---resultrndresult-boxdyn-error .code-header}
:::

::: docblock
Implementation of RND calculation for option chains

##### [§](#numerical-method){.doc-anchor}Numerical Method

1.  Calculates second derivative of option prices
2.  Applies Breeden-Litzenberger formula
3.  Normalizes resulting densities

##### [§](#error-conditions){.doc-anchor}Error Conditions

- Empty option chain
- Zero derivative tolerance
- Failed density calculations
:::

::: {#method.calculate_skew .section .method .trait-impl}
[Source](../../../src/optionstratlib/chains/chain.rs.html#2212-2230){.src
.rightside}[§](#method.calculate_skew){.anchor}

#### fn [calculate_skew](../trait.RNDAnalysis.html#tymethod.calculate_skew){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Vec](https://doc.rust-lang.org/1.86.0/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<([Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, Decimal)\>, [Box](https://doc.rust-lang.org/1.86.0/alloc/boxed/struct.Box.html "struct alloc::boxed::Box"){.struct}\<dyn [Error](https://doc.rust-lang.org/1.86.0/core/error/trait.Error.html "trait core::error::Error"){.trait}\>\> {#fn-calculate_skewself---resultvecpositive-decimal-boxdyn-error .code-header}
:::

::: docblock
Implementation of volatility skew calculation

Extracts and analyzes the relationship between strike prices and implied
volatilities.

##### [§](#error-conditions-1){.doc-anchor}Error Conditions

- Missing ATM volatility
- Insufficient valid data points
:::
:::::::

::: {#impl-Serialize-for-OptionChain .section .impl}
[Source](../../../src/optionstratlib/chains/chain.rs.html#93-116){.src
.rightside}[§](#impl-Serialize-for-OptionChain){.anchor}

### impl [Serialize](https://docs.rs/serde/1.0.219/serde/ser/trait.Serialize.html "trait serde::ser::Serialize"){.trait} for [OptionChain](struct.OptionChain.html "struct optionstratlib::chains::chain::OptionChain"){.struct} {#impl-serialize-for-optionchain .code-header}
:::

:::::: impl-items
:::: {#method.serialize .section .method .trait-impl}
[Source](../../../src/optionstratlib/chains/chain.rs.html#94-115){.src
.rightside}[§](#method.serialize){.anchor}

#### fn [serialize](https://docs.rs/serde/1.0.219/serde/ser/trait.Serialize.html#tymethod.serialize){.fn}\<S\>(&self, serializer: S) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<S::[Ok](https://docs.rs/serde/1.0.219/serde/ser/trait.Serializer.html#associatedtype.Ok "type serde::ser::Serializer::Ok"){.associatedtype}, S::[Error](https://docs.rs/serde/1.0.219/serde/ser/trait.Serializer.html#associatedtype.Error "type serde::ser::Serializer::Error"){.associatedtype}\> {#fn-serializesself-serializer-s---resultsok-serror .code-header}

::: where
where S:
[Serializer](https://docs.rs/serde/1.0.219/serde/ser/trait.Serializer.html "trait serde::ser::Serializer"){.trait},
:::
::::

::: docblock
Serialize this value into the given Serde serializer. [Read
more](https://docs.rs/serde/1.0.219/serde/ser/trait.Serialize.html#tymethod.serialize)
:::
::::::

::: {#impl-VolatilitySmile-for-OptionChain .section .impl}
[Source](../../../src/optionstratlib/chains/chain.rs.html#2273-2316){.src
.rightside}[§](#impl-VolatilitySmile-for-OptionChain){.anchor}

### impl [VolatilitySmile](../../volatility/trait.VolatilitySmile.html "trait optionstratlib::volatility::VolatilitySmile"){.trait} for [OptionChain](struct.OptionChain.html "struct optionstratlib::chains::chain::OptionChain"){.struct} {#impl-volatilitysmile-for-optionchain .code-header}
:::

::::: impl-items
::: {#method.smile .section .method .trait-impl}
[Source](../../../src/optionstratlib/chains/chain.rs.html#2285-2315){.src
.rightside}[§](#method.smile){.anchor}

#### fn [smile](../../volatility/trait.VolatilitySmile.html#tymethod.smile){.fn}(&self) -\> [Curve](../../curves/struct.Curve.html "struct optionstratlib::curves::Curve"){.struct} {#fn-smileself---curve .code-header}
:::

::: docblock
Computes the volatility smile for the option chain.

This function calculates the volatility smile by interpolating the
implied volatilities for all strike prices in the option chain. It uses
the available implied volatilities from the `options` field and performs
linear interpolation to estimate missing values.

##### [§](#returns-34){.doc-anchor}Returns

A `Curve` object representing the volatility smile. The x-coordinates of
the curve correspond to the strike prices, and the y-coordinates
represent the corresponding implied volatilities.
:::
:::::
:::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::

## Auto Trait Implementations[§](#synthetic-implementations){.anchor} {#synthetic-implementations .section-header}

::::::::: {#synthetic-implementations-list}
::: {#impl-Freeze-for-OptionChain .section .impl}
[§](#impl-Freeze-for-OptionChain){.anchor}

### impl [Freeze](https://doc.rust-lang.org/1.86.0/core/marker/trait.Freeze.html "trait core::marker::Freeze"){.trait} for [OptionChain](struct.OptionChain.html "struct optionstratlib::chains::chain::OptionChain"){.struct} {#impl-freeze-for-optionchain .code-header}
:::

::: {#impl-RefUnwindSafe-for-OptionChain .section .impl}
[§](#impl-RefUnwindSafe-for-OptionChain){.anchor}

### impl [RefUnwindSafe](https://doc.rust-lang.org/1.86.0/core/panic/unwind_safe/trait.RefUnwindSafe.html "trait core::panic::unwind_safe::RefUnwindSafe"){.trait} for [OptionChain](struct.OptionChain.html "struct optionstratlib::chains::chain::OptionChain"){.struct} {#impl-refunwindsafe-for-optionchain .code-header}
:::

::: {#impl-Send-for-OptionChain .section .impl}
[§](#impl-Send-for-OptionChain){.anchor}

### impl [Send](https://doc.rust-lang.org/1.86.0/core/marker/trait.Send.html "trait core::marker::Send"){.trait} for [OptionChain](struct.OptionChain.html "struct optionstratlib::chains::chain::OptionChain"){.struct} {#impl-send-for-optionchain .code-header}
:::

::: {#impl-Sync-for-OptionChain .section .impl}
[§](#impl-Sync-for-OptionChain){.anchor}

### impl [Sync](https://doc.rust-lang.org/1.86.0/core/marker/trait.Sync.html "trait core::marker::Sync"){.trait} for [OptionChain](struct.OptionChain.html "struct optionstratlib::chains::chain::OptionChain"){.struct} {#impl-sync-for-optionchain .code-header}
:::

::: {#impl-Unpin-for-OptionChain .section .impl}
[§](#impl-Unpin-for-OptionChain){.anchor}

### impl [Unpin](https://doc.rust-lang.org/1.86.0/core/marker/trait.Unpin.html "trait core::marker::Unpin"){.trait} for [OptionChain](struct.OptionChain.html "struct optionstratlib::chains::chain::OptionChain"){.struct} {#impl-unpin-for-optionchain .code-header}
:::

::: {#impl-UnwindSafe-for-OptionChain .section .impl}
[§](#impl-UnwindSafe-for-OptionChain){.anchor}

### impl [UnwindSafe](https://doc.rust-lang.org/1.86.0/core/panic/unwind_safe/trait.UnwindSafe.html "trait core::panic::unwind_safe::UnwindSafe"){.trait} for [OptionChain](struct.OptionChain.html "struct optionstratlib::chains::chain::OptionChain"){.struct} {#impl-unwindsafe-for-optionchain .code-header}
:::
:::::::::

## Blanket Implementations[§](#blanket-implementations){.anchor} {#blanket-implementations .section-header}

:::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::: {#blanket-implementations-list}
:::: {#impl-Any-for-T .section .impl}
[Source](https://doc.rust-lang.org/1.86.0/src/core/any.rs.html#138){.src
.rightside}[§](#impl-Any-for-T){.anchor}

### impl\<T\> [Any](https://doc.rust-lang.org/1.86.0/core/any/trait.Any.html "trait core::any::Any"){.trait} for T {#implt-any-for-t .code-header}

::: where
where T: \'static +
?[Sized](https://doc.rust-lang.org/1.86.0/core/marker/trait.Sized.html "trait core::marker::Sized"){.trait},
:::
::::

::::: impl-items
::: {#method.type_id .section .method .trait-impl}
[Source](https://doc.rust-lang.org/1.86.0/src/core/any.rs.html#139){.src
.rightside}[§](#method.type_id){.anchor}

#### fn [type_id](https://doc.rust-lang.org/1.86.0/core/any/trait.Any.html#tymethod.type_id){.fn}(&self) -\> [TypeId](https://doc.rust-lang.org/1.86.0/core/any/struct.TypeId.html "struct core::any::TypeId"){.struct} {#fn-type_idself---typeid .code-header}
:::

::: docblock
Gets the `TypeId` of `self`. [Read
more](https://doc.rust-lang.org/1.86.0/core/any/trait.Any.html#tymethod.type_id)
:::
:::::

:::: {#impl-Borrow%3CT%3E-for-T .section .impl}
[Source](https://doc.rust-lang.org/1.86.0/src/core/borrow.rs.html#209){.src
.rightside}[§](#impl-Borrow%3CT%3E-for-T){.anchor}

### impl\<T\> [Borrow](https://doc.rust-lang.org/1.86.0/core/borrow/trait.Borrow.html "trait core::borrow::Borrow"){.trait}\<T\> for T {#implt-borrowt-for-t .code-header}

::: where
where T:
?[Sized](https://doc.rust-lang.org/1.86.0/core/marker/trait.Sized.html "trait core::marker::Sized"){.trait},
:::
::::

::::: impl-items
::: {#method.borrow .section .method .trait-impl}
[Source](https://doc.rust-lang.org/1.86.0/src/core/borrow.rs.html#211){.src
.rightside}[§](#method.borrow){.anchor}

#### fn [borrow](https://doc.rust-lang.org/1.86.0/core/borrow/trait.Borrow.html#tymethod.borrow){.fn}(&self) -\> [&T](https://doc.rust-lang.org/1.86.0/std/primitive.reference.html){.primitive} {#fn-borrowself---t .code-header}
:::

::: docblock
Immutably borrows from an owned value. [Read
more](https://doc.rust-lang.org/1.86.0/core/borrow/trait.Borrow.html#tymethod.borrow)
:::
:::::

:::: {#impl-BorrowMut%3CT%3E-for-T .section .impl}
[Source](https://doc.rust-lang.org/1.86.0/src/core/borrow.rs.html#217){.src
.rightside}[§](#impl-BorrowMut%3CT%3E-for-T){.anchor}

### impl\<T\> [BorrowMut](https://doc.rust-lang.org/1.86.0/core/borrow/trait.BorrowMut.html "trait core::borrow::BorrowMut"){.trait}\<T\> for T {#implt-borrowmutt-for-t .code-header}

::: where
where T:
?[Sized](https://doc.rust-lang.org/1.86.0/core/marker/trait.Sized.html "trait core::marker::Sized"){.trait},
:::
::::

::::: impl-items
::: {#method.borrow_mut .section .method .trait-impl}
[Source](https://doc.rust-lang.org/1.86.0/src/core/borrow.rs.html#218){.src
.rightside}[§](#method.borrow_mut){.anchor}

#### fn [borrow_mut](https://doc.rust-lang.org/1.86.0/core/borrow/trait.BorrowMut.html#tymethod.borrow_mut){.fn}(&mut self) -\> [&mut T](https://doc.rust-lang.org/1.86.0/std/primitive.reference.html){.primitive} {#fn-borrow_mutmut-self---mut-t .code-header}
:::

::: docblock
Mutably borrows from an owned value. [Read
more](https://doc.rust-lang.org/1.86.0/core/borrow/trait.BorrowMut.html#tymethod.borrow_mut)
:::
:::::

:::: {#impl-CloneToUninit-for-T .section .impl}
[Source](https://doc.rust-lang.org/1.86.0/src/core/clone.rs.html#273){.src
.rightside}[§](#impl-CloneToUninit-for-T){.anchor}

### impl\<T\> [CloneToUninit](https://doc.rust-lang.org/1.86.0/core/clone/trait.CloneToUninit.html "trait core::clone::CloneToUninit"){.trait} for T {#implt-clonetouninit-for-t .code-header}

::: where
where T:
[Clone](https://doc.rust-lang.org/1.86.0/core/clone/trait.Clone.html "trait core::clone::Clone"){.trait},
:::
::::

:::::: impl-items
::: {#method.clone_to_uninit .section .method .trait-impl}
[Source](https://doc.rust-lang.org/1.86.0/src/core/clone.rs.html#275){.src
.rightside}[§](#method.clone_to_uninit){.anchor}

#### unsafe fn [clone_to_uninit](https://doc.rust-lang.org/1.86.0/core/clone/trait.CloneToUninit.html#tymethod.clone_to_uninit){.fn}(&self, dst: [\*mut](https://doc.rust-lang.org/1.86.0/std/primitive.pointer.html){.primitive} [u8](https://doc.rust-lang.org/1.86.0/std/primitive.u8.html){.primitive}) {#unsafe-fn-clone_to_uninitself-dst-mut-u8 .code-header}
:::

[]{.item-info}

::: {.stab .unstable}
🔬This is a nightly-only experimental API. (`clone_to_uninit`)
:::

::: docblock
Performs copy-assignment from `self` to `dst`. [Read
more](https://doc.rust-lang.org/1.86.0/core/clone/trait.CloneToUninit.html#tymethod.clone_to_uninit)
:::
::::::

::: {#impl-From%3CT%3E-for-T .section .impl}
[Source](https://doc.rust-lang.org/1.86.0/src/core/convert/mod.rs.html#767){.src
.rightside}[§](#impl-From%3CT%3E-for-T){.anchor}

### impl\<T\> [From](https://doc.rust-lang.org/1.86.0/core/convert/trait.From.html "trait core::convert::From"){.trait}\<T\> for T {#implt-fromt-for-t .code-header}
:::

::::: impl-items
::: {#method.from-2 .section .method .trait-impl}
[Source](https://doc.rust-lang.org/1.86.0/src/core/convert/mod.rs.html#770){.src
.rightside}[§](#method.from-2){.anchor}

#### fn [from](https://doc.rust-lang.org/1.86.0/core/convert/trait.From.html#tymethod.from){.fn}(t: T) -\> T {#fn-fromt-t---t .code-header}
:::

::: docblock
Returns the argument unchanged.
:::
:::::

::: {#impl-Instrument-for-T .section .impl}
[§](#impl-Instrument-for-T){.anchor}

### impl\<T\> Instrument for T {#implt-instrument-for-t .code-header}
:::

::::::: impl-items
::: {#method.instrument .section .method .trait-impl}
[§](#method.instrument){.anchor}

#### fn [instrument]{.fn}(self, span: Span) -\> Instrumented\<Self\> {#fn-instrumentself-span-span---instrumentedself .code-header}
:::

::: docblock
Instruments this type with the provided \[`Span`\], returning an
`Instrumented` wrapper. Read more
:::

::: {#method.in_current_span .section .method .trait-impl}
[§](#method.in_current_span){.anchor}

#### fn [in_current_span]{.fn}(self) -\> Instrumented\<Self\> {#fn-in_current_spanself---instrumentedself .code-header}
:::

::: docblock
Instruments this type with the [current](super::Span::current())
[`Span`](crate::Span), returning an `Instrumented` wrapper. Read more
:::
:::::::

:::: {#impl-Into%3CU%3E-for-T .section .impl}
[Source](https://doc.rust-lang.org/1.86.0/src/core/convert/mod.rs.html#750-752){.src
.rightside}[§](#impl-Into%3CU%3E-for-T){.anchor}

### impl\<T, U\> [Into](https://doc.rust-lang.org/1.86.0/core/convert/trait.Into.html "trait core::convert::Into"){.trait}\<U\> for T {#implt-u-intou-for-t .code-header}

::: where
where U:
[From](https://doc.rust-lang.org/1.86.0/core/convert/trait.From.html "trait core::convert::From"){.trait}\<T\>,
:::
::::

::::: impl-items
::: {#method.into .section .method .trait-impl}
[Source](https://doc.rust-lang.org/1.86.0/src/core/convert/mod.rs.html#760){.src
.rightside}[§](#method.into){.anchor}

#### fn [into](https://doc.rust-lang.org/1.86.0/core/convert/trait.Into.html#tymethod.into){.fn}(self) -\> U {#fn-intoself---u .code-header}
:::

::: docblock
Calls `U::from(self)`.

That is, this conversion is whatever the implementation of
[`From`](https://doc.rust-lang.org/1.86.0/core/convert/trait.From.html "trait core::convert::From")`<T> for U`
chooses to do.
:::
:::::

::: {#impl-IntoEither-for-T .section .impl}
[Source](https://docs.rs/either/1/src/either/into_either.rs.html#64){.src
.rightside}[§](#impl-IntoEither-for-T){.anchor}

### impl\<T\> [IntoEither](https://docs.rs/either/1/either/into_either/trait.IntoEither.html "trait either::into_either::IntoEither"){.trait} for T {#implt-intoeither-for-t .code-header}
:::

:::::::: impl-items
::: {#method.into_either .section .method .trait-impl}
[Source](https://docs.rs/either/1/src/either/into_either.rs.html#29){.src
.rightside}[§](#method.into_either){.anchor}

#### fn [into_either](https://docs.rs/either/1/either/into_either/trait.IntoEither.html#method.into_either){.fn}(self, into_left: [bool](https://doc.rust-lang.org/1.86.0/std/primitive.bool.html){.primitive}) -\> [Either](https://docs.rs/either/1/either/enum.Either.html "enum either::Either"){.enum}\<Self, Self\> {#fn-into_eitherself-into_left-bool---eitherself-self .code-header}
:::

::: docblock
Converts `self` into a
[`Left`](https://docs.rs/either/1/either/enum.Either.html#variant.Left "variant either::Either::Left")
variant of
[`Either<Self, Self>`](https://docs.rs/either/1/either/enum.Either.html "enum either::Either")
if `into_left` is `true`. Converts `self` into a
[`Right`](https://docs.rs/either/1/either/enum.Either.html#variant.Right "variant either::Either::Right")
variant of
[`Either<Self, Self>`](https://docs.rs/either/1/either/enum.Either.html "enum either::Either")
otherwise. [Read
more](https://docs.rs/either/1/either/into_either/trait.IntoEither.html#method.into_either)
:::

:::: {#method.into_either_with .section .method .trait-impl}
[Source](https://docs.rs/either/1/src/either/into_either.rs.html#55-57){.src
.rightside}[§](#method.into_either_with){.anchor}

#### fn [into_either_with](https://docs.rs/either/1/either/into_either/trait.IntoEither.html#method.into_either_with){.fn}\<F\>(self, into_left: F) -\> [Either](https://docs.rs/either/1/either/enum.Either.html "enum either::Either"){.enum}\<Self, Self\> {#fn-into_either_withfself-into_left-f---eitherself-self .code-header}

::: where
where F:
[FnOnce](https://doc.rust-lang.org/1.86.0/core/ops/function/trait.FnOnce.html "trait core::ops::function::FnOnce"){.trait}(&Self)
-\>
[bool](https://doc.rust-lang.org/1.86.0/std/primitive.bool.html){.primitive},
:::
::::

::: docblock
Converts `self` into a
[`Left`](https://docs.rs/either/1/either/enum.Either.html#variant.Left "variant either::Either::Left")
variant of
[`Either<Self, Self>`](https://docs.rs/either/1/either/enum.Either.html "enum either::Either")
if `into_left(&self)` returns `true`. Converts `self` into a
[`Right`](https://docs.rs/either/1/either/enum.Either.html#variant.Right "variant either::Either::Right")
variant of
[`Either<Self, Self>`](https://docs.rs/either/1/either/enum.Either.html "enum either::Either")
otherwise. [Read
more](https://docs.rs/either/1/either/into_either/trait.IntoEither.html#method.into_either_with)
:::
::::::::

::: {#impl-Pointable-for-T .section .impl}
[§](#impl-Pointable-for-T){.anchor}

### impl\<T\> Pointable for T {#implt-pointable-for-t .code-header}
:::

::::::::::::::: impl-items
::: {#associatedconstant.ALIGN .section .associatedconstant .trait-impl}
[§](#associatedconstant.ALIGN){.anchor}

#### const [ALIGN]{.constant}: [usize](https://doc.rust-lang.org/1.86.0/std/primitive.usize.html){.primitive} {#const-align-usize .code-header}
:::

::: docblock
The alignment of pointer.
:::

::: {#associatedtype.Init .section .associatedtype .trait-impl}
[§](#associatedtype.Init){.anchor}

#### type [Init]{.associatedtype} = T {#type-init-t .code-header}
:::

::: docblock
The type for initializers.
:::

::: {#method.init .section .method .trait-impl}
[§](#method.init){.anchor}

#### unsafe fn [init]{.fn}(init: \<T as Pointable\>::Init) -\> [usize](https://doc.rust-lang.org/1.86.0/std/primitive.usize.html){.primitive} {#unsafe-fn-initinit-t-as-pointableinit---usize .code-header}
:::

::: docblock
Initializes a with the given initializer. Read more
:::

::: {#method.deref .section .method .trait-impl}
[§](#method.deref){.anchor}

#### unsafe fn [deref]{.fn}\<\'a\>(ptr: [usize](https://doc.rust-lang.org/1.86.0/std/primitive.usize.html){.primitive}) -\> [&\'a T](https://doc.rust-lang.org/1.86.0/std/primitive.reference.html){.primitive} {#unsafe-fn-derefaptr-usize---a-t .code-header}
:::

::: docblock
Dereferences the given pointer. Read more
:::

::: {#method.deref_mut .section .method .trait-impl}
[§](#method.deref_mut){.anchor}

#### unsafe fn [deref_mut]{.fn}\<\'a\>(ptr: [usize](https://doc.rust-lang.org/1.86.0/std/primitive.usize.html){.primitive}) -\> [&\'a mut T](https://doc.rust-lang.org/1.86.0/std/primitive.reference.html){.primitive} {#unsafe-fn-deref_mutaptr-usize---a-mut-t .code-header}
:::

::: docblock
Mutably dereferences the given pointer. Read more
:::

::: {#method.drop .section .method .trait-impl}
[§](#method.drop){.anchor}

#### unsafe fn [drop]{.fn}(ptr: [usize](https://doc.rust-lang.org/1.86.0/std/primitive.usize.html){.primitive}) {#unsafe-fn-dropptr-usize .code-header}
:::

::: docblock
Drops the object pointed to by the given pointer. Read more
:::
:::::::::::::::

::: {#impl-Same-for-T .section .impl}
[Source](https://docs.rs/typenum/1.18.0/src/typenum/type_operators.rs.html#34){.src
.rightside}[§](#impl-Same-for-T){.anchor}

### impl\<T\> [Same](https://docs.rs/typenum/1.18.0/typenum/type_operators/trait.Same.html "trait typenum::type_operators::Same"){.trait} for T {#implt-same-for-t .code-header}
:::

::::: impl-items
::: {#associatedtype.Output .section .associatedtype .trait-impl}
[Source](https://docs.rs/typenum/1.18.0/src/typenum/type_operators.rs.html#35){.src
.rightside}[§](#associatedtype.Output){.anchor}

#### type [Output](https://docs.rs/typenum/1.18.0/typenum/type_operators/trait.Same.html#associatedtype.Output){.associatedtype} = T {#type-output-t .code-header}
:::

::: docblock
Should always be `Self`
:::
:::::

:::: {#impl-SupersetOf%3CSS%3E-for-SP .section .impl}
[§](#impl-SupersetOf%3CSS%3E-for-SP){.anchor}

### impl\<SS, SP\> SupersetOf\<SS\> for SP {#implss-sp-supersetofss-for-sp .code-header}

::: where
where SS: SubsetOf\<SP\>,
:::
::::

::::::::::: impl-items
::: {#method.to_subset .section .method .trait-impl}
[§](#method.to_subset){.anchor}

#### fn [to_subset]{.fn}(&self) -\> [Option](https://doc.rust-lang.org/1.86.0/core/option/enum.Option.html "enum core::option::Option"){.enum}\<SS\> {#fn-to_subsetself---optionss .code-header}
:::

::: docblock
The inverse inclusion map: attempts to construct `self` from the
equivalent element of its superset. Read more
:::

::: {#method.is_in_subset .section .method .trait-impl}
[§](#method.is_in_subset){.anchor}

#### fn [is_in_subset]{.fn}(&self) -\> [bool](https://doc.rust-lang.org/1.86.0/std/primitive.bool.html){.primitive} {#fn-is_in_subsetself---bool .code-header}
:::

::: docblock
Checks if `self` is actually part of its subset `T` (and can be
converted to it).
:::

::: {#method.to_subset_unchecked .section .method .trait-impl}
[§](#method.to_subset_unchecked){.anchor}

#### fn [to_subset_unchecked]{.fn}(&self) -\> SS {#fn-to_subset_uncheckedself---ss .code-header}
:::

::: docblock
Use with care! Same as `self.to_subset` but without any property checks.
Always succeeds.
:::

::: {#method.from_subset .section .method .trait-impl}
[§](#method.from_subset){.anchor}

#### fn [from_subset]{.fn}(element: [&SS](https://doc.rust-lang.org/1.86.0/std/primitive.reference.html){.primitive}) -\> SP {#fn-from_subsetelement-ss---sp .code-header}
:::

::: docblock
The inclusion map: converts `self` to the equivalent element of its
superset.
:::
:::::::::::

:::: {#impl-ToOwned-for-T .section .impl}
[Source](https://doc.rust-lang.org/1.86.0/src/alloc/borrow.rs.html#82-84){.src
.rightside}[§](#impl-ToOwned-for-T){.anchor}

### impl\<T\> [ToOwned](https://doc.rust-lang.org/1.86.0/alloc/borrow/trait.ToOwned.html "trait alloc::borrow::ToOwned"){.trait} for T {#implt-toowned-for-t .code-header}

::: where
where T:
[Clone](https://doc.rust-lang.org/1.86.0/core/clone/trait.Clone.html "trait core::clone::Clone"){.trait},
:::
::::

::::::::: impl-items
::: {#associatedtype.Owned .section .associatedtype .trait-impl}
[Source](https://doc.rust-lang.org/1.86.0/src/alloc/borrow.rs.html#86){.src
.rightside}[§](#associatedtype.Owned){.anchor}

#### type [Owned](https://doc.rust-lang.org/1.86.0/alloc/borrow/trait.ToOwned.html#associatedtype.Owned){.associatedtype} = T {#type-owned-t .code-header}
:::

::: docblock
The resulting type after obtaining ownership.
:::

::: {#method.to_owned .section .method .trait-impl}
[Source](https://doc.rust-lang.org/1.86.0/src/alloc/borrow.rs.html#87){.src
.rightside}[§](#method.to_owned){.anchor}

#### fn [to_owned](https://doc.rust-lang.org/1.86.0/alloc/borrow/trait.ToOwned.html#tymethod.to_owned){.fn}(&self) -\> T {#fn-to_ownedself---t .code-header}
:::

::: docblock
Creates owned data from borrowed data, usually by cloning. [Read
more](https://doc.rust-lang.org/1.86.0/alloc/borrow/trait.ToOwned.html#tymethod.to_owned)
:::

::: {#method.clone_into .section .method .trait-impl}
[Source](https://doc.rust-lang.org/1.86.0/src/alloc/borrow.rs.html#91){.src
.rightside}[§](#method.clone_into){.anchor}

#### fn [clone_into](https://doc.rust-lang.org/1.86.0/alloc/borrow/trait.ToOwned.html#method.clone_into){.fn}(&self, target: [&mut T](https://doc.rust-lang.org/1.86.0/std/primitive.reference.html){.primitive}) {#fn-clone_intoself-target-mut-t .code-header}
:::

::: docblock
Uses borrowed data to replace owned data, usually by cloning. [Read
more](https://doc.rust-lang.org/1.86.0/alloc/borrow/trait.ToOwned.html#method.clone_into)
:::
:::::::::

:::: {#impl-ToString-for-T .section .impl}
[Source](https://doc.rust-lang.org/1.86.0/src/alloc/string.rs.html#2758){.src
.rightside}[§](#impl-ToString-for-T){.anchor}

### impl\<T\> [ToString](https://doc.rust-lang.org/1.86.0/alloc/string/trait.ToString.html "trait alloc::string::ToString"){.trait} for T {#implt-tostring-for-t .code-header}

::: where
where T:
[Display](https://doc.rust-lang.org/1.86.0/core/fmt/trait.Display.html "trait core::fmt::Display"){.trait} +
?[Sized](https://doc.rust-lang.org/1.86.0/core/marker/trait.Sized.html "trait core::marker::Sized"){.trait},
:::
::::

::::: impl-items
::: {#method.to_string .section .method .trait-impl}
[Source](https://doc.rust-lang.org/1.86.0/src/alloc/string.rs.html#2760){.src
.rightside}[§](#method.to_string){.anchor}

#### fn [to_string](https://doc.rust-lang.org/1.86.0/alloc/string/trait.ToString.html#tymethod.to_string){.fn}(&self) -\> [String](https://doc.rust-lang.org/1.86.0/alloc/string/struct.String.html "struct alloc::string::String"){.struct} {#fn-to_stringself---string .code-header}
:::

::: docblock
Converts the given value to a `String`. [Read
more](https://doc.rust-lang.org/1.86.0/alloc/string/trait.ToString.html#tymethod.to_string)
:::
:::::

:::: {#impl-TryFrom%3CU%3E-for-T .section .impl}
[Source](https://doc.rust-lang.org/1.86.0/src/core/convert/mod.rs.html#807-809){.src
.rightside}[§](#impl-TryFrom%3CU%3E-for-T){.anchor}

### impl\<T, U\> [TryFrom](https://doc.rust-lang.org/1.86.0/core/convert/trait.TryFrom.html "trait core::convert::TryFrom"){.trait}\<U\> for T {#implt-u-tryfromu-for-t .code-header}

::: where
where U:
[Into](https://doc.rust-lang.org/1.86.0/core/convert/trait.Into.html "trait core::convert::Into"){.trait}\<T\>,
:::
::::

::::::: impl-items
::: {#associatedtype.Error-1 .section .associatedtype .trait-impl}
[Source](https://doc.rust-lang.org/1.86.0/src/core/convert/mod.rs.html#811){.src
.rightside}[§](#associatedtype.Error-1){.anchor}

#### type [Error](https://doc.rust-lang.org/1.86.0/core/convert/trait.TryFrom.html#associatedtype.Error){.associatedtype} = [Infallible](https://doc.rust-lang.org/1.86.0/core/convert/enum.Infallible.html "enum core::convert::Infallible"){.enum} {#type-error-infallible .code-header}
:::

::: docblock
The type returned in the event of a conversion error.
:::

::: {#method.try_from .section .method .trait-impl}
[Source](https://doc.rust-lang.org/1.86.0/src/core/convert/mod.rs.html#814){.src
.rightside}[§](#method.try_from){.anchor}

#### fn [try_from](https://doc.rust-lang.org/1.86.0/core/convert/trait.TryFrom.html#tymethod.try_from){.fn}(value: U) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<T, \<T as [TryFrom](https://doc.rust-lang.org/1.86.0/core/convert/trait.TryFrom.html "trait core::convert::TryFrom"){.trait}\<U\>\>::[Error](https://doc.rust-lang.org/1.86.0/core/convert/trait.TryFrom.html#associatedtype.Error "type core::convert::TryFrom::Error"){.associatedtype}\> {#fn-try_fromvalue-u---resultt-t-as-tryfromuerror .code-header}
:::

::: docblock
Performs the conversion.
:::
:::::::

:::: {#impl-TryInto%3CU%3E-for-T .section .impl}
[Source](https://doc.rust-lang.org/1.86.0/src/core/convert/mod.rs.html#792-794){.src
.rightside}[§](#impl-TryInto%3CU%3E-for-T){.anchor}

### impl\<T, U\> [TryInto](https://doc.rust-lang.org/1.86.0/core/convert/trait.TryInto.html "trait core::convert::TryInto"){.trait}\<U\> for T {#implt-u-tryintou-for-t .code-header}

::: where
where U:
[TryFrom](https://doc.rust-lang.org/1.86.0/core/convert/trait.TryFrom.html "trait core::convert::TryFrom"){.trait}\<T\>,
:::
::::

::::::: impl-items
::: {#associatedtype.Error .section .associatedtype .trait-impl}
[Source](https://doc.rust-lang.org/1.86.0/src/core/convert/mod.rs.html#796){.src
.rightside}[§](#associatedtype.Error){.anchor}

#### type [Error](https://doc.rust-lang.org/1.86.0/core/convert/trait.TryInto.html#associatedtype.Error){.associatedtype} = \<U as [TryFrom](https://doc.rust-lang.org/1.86.0/core/convert/trait.TryFrom.html "trait core::convert::TryFrom"){.trait}\<T\>\>::[Error](https://doc.rust-lang.org/1.86.0/core/convert/trait.TryFrom.html#associatedtype.Error "type core::convert::TryFrom::Error"){.associatedtype} {#type-error-u-as-tryfromterror .code-header}
:::

::: docblock
The type returned in the event of a conversion error.
:::

::: {#method.try_into .section .method .trait-impl}
[Source](https://doc.rust-lang.org/1.86.0/src/core/convert/mod.rs.html#799){.src
.rightside}[§](#method.try_into){.anchor}

#### fn [try_into](https://doc.rust-lang.org/1.86.0/core/convert/trait.TryInto.html#tymethod.try_into){.fn}(self) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<U, \<U as [TryFrom](https://doc.rust-lang.org/1.86.0/core/convert/trait.TryFrom.html "trait core::convert::TryFrom"){.trait}\<T\>\>::[Error](https://doc.rust-lang.org/1.86.0/core/convert/trait.TryFrom.html#associatedtype.Error "type core::convert::TryFrom::Error"){.associatedtype}\> {#fn-try_intoself---resultu-u-as-tryfromterror .code-header}
:::

::: docblock
Performs the conversion.
:::
:::::::

:::: {#impl-VZip%3CV%3E-for-T .section .impl}
[§](#impl-VZip%3CV%3E-for-T){.anchor}

### impl\<V, T\> VZip\<V\> for T {#implv-t-vzipv-for-t .code-header}

::: where
where V: MultiLane\<T\>,
:::
::::

:::: impl-items
::: {#method.vzip .section .method .trait-impl}
[§](#method.vzip){.anchor}

#### fn [vzip]{.fn}(self) -\> V {#fn-vzipself---v .code-header}
:::
::::

::: {#impl-WithSubscriber-for-T .section .impl}
[§](#impl-WithSubscriber-for-T){.anchor}

### impl\<T\> WithSubscriber for T {#implt-withsubscriber-for-t .code-header}
:::

:::::::: impl-items
:::: {#method.with_subscriber .section .method .trait-impl}
[§](#method.with_subscriber){.anchor}

#### fn [with_subscriber]{.fn}\<S\>(self, subscriber: S) -\> WithDispatch\<Self\> {#fn-with_subscribersself-subscriber-s---withdispatchself .code-header}

::: where
where S:
[Into](https://doc.rust-lang.org/1.86.0/core/convert/trait.Into.html "trait core::convert::Into"){.trait}\<Dispatch\>,
:::
::::

::: docblock
Attaches the provided [`Subscriber`](super::Subscriber) to this type,
returning a \[`WithDispatch`\] wrapper. Read more
:::

::: {#method.with_current_subscriber .section .method .trait-impl}
[§](#method.with_current_subscriber){.anchor}

#### fn [with_current_subscriber]{.fn}(self) -\> WithDispatch\<Self\> {#fn-with_current_subscriberself---withdispatchself .code-header}
:::

::: docblock
Attaches the current
[default](crate::dispatcher#setting-the-default-subscriber)
[`Subscriber`](super::Subscriber) to this type, returning a
\[`WithDispatch`\] wrapper. Read more
:::
::::::::

:::: {#impl-DeserializeOwned-for-T .section .impl}
[Source](https://docs.rs/serde/1.0.219/src/serde/de/mod.rs.html#614){.src
.rightside}[§](#impl-DeserializeOwned-for-T){.anchor}

### impl\<T\> [DeserializeOwned](https://docs.rs/serde/1.0.219/serde/de/trait.DeserializeOwned.html "trait serde::de::DeserializeOwned"){.trait} for T {#implt-deserializeowned-for-t .code-header}

::: where
where T: for\<\'de\>
[Deserialize](https://docs.rs/serde/1.0.219/serde/de/trait.Deserialize.html "trait serde::de::Deserialize"){.trait}\<\'de\>,
:::
::::
::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
:::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
