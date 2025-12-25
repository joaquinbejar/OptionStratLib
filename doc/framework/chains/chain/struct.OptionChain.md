::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::: width-limiter
:::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../../index.html)::[chains](../index.html)::[chain](index.html)
:::

# Struct [OptionChain]{.struct} Copy item path

[[Source](../../../src/optionstratlib/chains/chain.rs.html#90-108){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub struct OptionChain {
    pub symbol: String,
    pub underlying_price: Positive,
    pub options: BTreeSet<OptionData>,
    pub risk_free_rate: Option<Decimal>,
    pub dividend_yield: Option<Positive>,
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
.field}`symbol: `[`String`](https://doc.rust-lang.org/1.91.1/alloc/string/struct.String.html "struct alloc::string::String"){.struct}]{#structfield.symbol
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

[[§](#structfield.options){.anchor
.field}`options: `[`BTreeSet`](https://doc.rust-lang.org/1.91.1/alloc/collections/btree/set/struct.BTreeSet.html "struct alloc::collections::btree::set::BTreeSet"){.struct}`<`[`OptionData`](../struct.OptionData.html "struct optionstratlib::chains::OptionData"){.struct}`>`]{#structfield.options
.structfield .section-header}

::: docblock
A sorted collection of option contracts at different strike prices.
:::

[[§](#structfield.risk_free_rate){.anchor
.field}`risk_free_rate: `[`Option`](https://doc.rust-lang.org/1.91.1/core/option/enum.Option.html "enum core::option::Option"){.enum}`<`[`Decimal`](../../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}`>`]{#structfield.risk_free_rate
.structfield .section-header}

::: docblock
The risk-free interest rate used for option pricing models.
:::

[[§](#structfield.dividend_yield){.anchor
.field}`dividend_yield: `[`Option`](https://doc.rust-lang.org/1.91.1/core/option/enum.Option.html "enum core::option::Option"){.enum}`<`[`Positive`](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}`>`]{#structfield.dividend_yield
.structfield .section-header}

::: docblock
The annual dividend yield of the underlying asset.
:::

## Implementations[§](#implementations){.anchor} {#implementations .section-header}

:::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::: {#implementations-list}
::: {#impl-OptionChain .section .impl}
[Source](../../../src/optionstratlib/chains/chain.rs.html#244-2229){.src
.rightside}[§](#impl-OptionChain){.anchor}

### impl [OptionChain](struct.OptionChain.html "struct optionstratlib::chains::chain::OptionChain"){.struct} {#impl-optionchain .code-header}
:::

:::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::: impl-items
::: {#method.new .section .method}
[Source](../../../src/optionstratlib/chains/chain.rs.html#286-301){.src
.rightside}

#### pub fn [new](#method.new){.fn}( symbol: &[str](https://doc.rust-lang.org/1.91.1/std/primitive.str.html){.primitive}, underlying_price: [Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, expiration_date: [String](https://doc.rust-lang.org/1.91.1/alloc/string/struct.String.html "struct alloc::string::String"){.struct}, risk_free_rate: [Option](https://doc.rust-lang.org/1.91.1/core/option/enum.Option.html "enum core::option::Option"){.enum}\<[Decimal](../../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}\>, dividend_yield: [Option](https://doc.rust-lang.org/1.91.1/core/option/enum.Option.html "enum core::option::Option"){.enum}\<[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\>, ) -\> Self {#pub-fn-new-symbol-str-underlying_price-positive-expiration_date-string-risk_free_rate-optiondecimal-dividend_yield-optionpositive---self .code-header}
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
use positive::{spos, pos_or_panic};

let chain = OptionChain::new(
    "AAPL",
    pos!(172.50),
    "2023-12-15".to_string(),
    Some(dec!(0.05)),  // 5% risk-free rate
    spos!(0.0065) // 0.65% dividend yield
);
```
:::
::::

::: {#method.build_chain .section .method}
[Source](../../../src/optionstratlib/chains/chain.rs.html#348-486){.src
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
    Some(Box::new(pos!(100.0))),                         // underlying price
    Some(ExpirationDate::Days(pos!(30.0))),    // expiration date
    Some(dec!(0.05)),                          // risk-free rate
    spos!(0.0),                           // dividend yield
    Some("SPY".to_string())              // underlying symbol
);

let build_params = OptionChainBuildParams::new(
    "SPY".to_string(),
    spos!(1000.0),
    10,
    spos!(5.0),
    dec!(-0.2),
    dec!(0.1),
    pos!(0.02),
    2,
    price_params,
    pos!(0.2) // implied volatility
);

let chain = OptionChain::build_chain(&build_params);
```
:::
::::

::: {#method.to_build_params .section .method}
[Source](../../../src/optionstratlib/chains/chain.rs.html#502-605){.src
.rightside}

#### pub fn [to_build_params](#method.to_build_params){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[OptionChainBuildParams](../utils/struct.OptionChainBuildParams.html "struct optionstratlib::chains::utils::OptionChainBuildParams"){.struct}, [ChainError](../../error/chains/enum.ChainError.html "enum optionstratlib::error::chains::ChainError"){.enum}\> {#pub-fn-to_build_paramsself---resultoptionchainbuildparams-chainerror .code-header}
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
[Source](../../../src/optionstratlib/chains/chain.rs.html#724-773){.src
.rightside}

#### pub fn [add_option](#method.add_option){.fn}( &mut self, strike_price: [Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, call_bid: [Option](https://doc.rust-lang.org/1.91.1/core/option/enum.Option.html "enum core::option::Option"){.enum}\<[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\>, call_ask: [Option](https://doc.rust-lang.org/1.91.1/core/option/enum.Option.html "enum core::option::Option"){.enum}\<[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\>, put_bid: [Option](https://doc.rust-lang.org/1.91.1/core/option/enum.Option.html "enum core::option::Option"){.enum}\<[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\>, put_ask: [Option](https://doc.rust-lang.org/1.91.1/core/option/enum.Option.html "enum core::option::Option"){.enum}\<[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\>, implied_volatility: [Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, delta_call: [Option](https://doc.rust-lang.org/1.91.1/core/option/enum.Option.html "enum core::option::Option"){.enum}\<[Decimal](../../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}\>, delta_put: [Option](https://doc.rust-lang.org/1.91.1/core/option/enum.Option.html "enum core::option::Option"){.enum}\<[Decimal](../../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}\>, gamma: [Option](https://doc.rust-lang.org/1.91.1/core/option/enum.Option.html "enum core::option::Option"){.enum}\<[Decimal](../../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}\>, volume: [Option](https://doc.rust-lang.org/1.91.1/core/option/enum.Option.html "enum core::option::Option"){.enum}\<[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\>, open_interest: [Option](https://doc.rust-lang.org/1.91.1/core/option/enum.Option.html "enum core::option::Option"){.enum}\<[u64](https://doc.rust-lang.org/1.91.1/std/primitive.u64.html){.primitive}\>, extra_fields: [Option](https://doc.rust-lang.org/1.91.1/core/option/enum.Option.html "enum core::option::Option"){.enum}\<[Value](../../../serde_json/value/enum.Value.html "enum serde_json::value::Value"){.enum}\>, ) {#pub-fn-add_option-mut-self-strike_price-positive-call_bid-optionpositive-call_ask-optionpositive-put_bid-optionpositive-put_ask-optionpositive-implied_volatility-positive-delta_call-optiondecimal-delta_put-optiondecimal-gamma-optiondecimal-volume-optionpositive-open_interest-optionu64-extra_fields-optionvalue .code-header}
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
[Source](../../../src/optionstratlib/chains/chain.rs.html#802-805){.src
.rightside}

#### pub fn [atm_strike](#method.atm_strike){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<&[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, [ChainError](../../error/chains/enum.ChainError.html "enum optionstratlib::error::chains::ChainError"){.enum}\> {#pub-fn-atm_strikeself---resultpositive-chainerror .code-header}
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
- `Err(ChainError)` - Error if the option chain is empty or if the
  operation fails

##### [§](#example-1){.doc-anchor}Example

::: example-wrap
``` {.rust .rust-example-rendered}
use tracing::{error, info};
use optionstratlib::chains::chain::OptionChain;
use optionstratlib::pos_or_panic;

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
[Source](../../../src/optionstratlib/chains/chain.rs.html#826-862){.src
.rightside}

#### pub fn [atm_option_data](#method.atm_option_data){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<&[OptionData](../struct.OptionData.html "struct optionstratlib::chains::OptionData"){.struct}, [ChainError](../../error/chains/enum.ChainError.html "enum optionstratlib::error::chains::ChainError"){.enum}\> {#pub-fn-atm_option_dataself---resultoptiondata-chainerror .code-header}
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
- `Err(ChainError)` - If the option chain is empty or no ATM option can
  be found, returns an error describing the failure.

##### [§](#errors){.doc-anchor}Errors

This function returns an error in the following cases:

- The option chain (`self.options`) is empty.
- No option with a strike price close to the underlying price can be
  found.
:::

::: {#method.get_title .section .method}
[Source](../../../src/optionstratlib/chains/chain.rs.html#874-881){.src
.rightside}

#### pub fn [get_title](#method.get_title){.fn}(&self) -\> [String](https://doc.rust-lang.org/1.91.1/alloc/string/struct.String.html "struct alloc::string::String"){.struct} {#pub-fn-get_titleself---string .code-header}
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
[Source](../../../src/optionstratlib/chains/chain.rs.html#901-920){.src
.rightside}

#### pub fn [set_from_title](#method.set_from_title){.fn}(&mut self, file: &[str](https://doc.rust-lang.org/1.91.1/std/primitive.str.html){.primitive}) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[()](https://doc.rust-lang.org/1.91.1/std/primitive.unit.html){.primitive}, [ChainError](../../error/chains/enum.ChainError.html "enum optionstratlib::error::chains::ChainError"){.enum}\> {#pub-fn-set_from_titlemut-self-file-str---result-chainerror .code-header}
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
[Source](../../../src/optionstratlib/chains/chain.rs.html#929-940){.src
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
[Source](../../../src/optionstratlib/chains/chain.rs.html#950-962){.src
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

::: {#method.save_to_csv .section .method}
[Source](../../../src/optionstratlib/chains/chain.rs.html#983-1016){.src
.rightside}

#### pub fn [save_to_csv](#method.save_to_csv){.fn}(&self, file_path: &[str](https://doc.rust-lang.org/1.91.1/std/primitive.str.html){.primitive}) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[()](https://doc.rust-lang.org/1.91.1/std/primitive.unit.html){.primitive}, [ChainError](../../error/chains/enum.ChainError.html "enum optionstratlib::error::chains::ChainError"){.enum}\> {#pub-fn-save_to_csvself-file_path-str---result-chainerror .code-header}
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

- `Result<(), ChainError>` - Ok(()) if successful, or an Error if the
  file couldn't be created or written to.

##### [§](#note){.doc-anchor}Note

This method is only available on non-WebAssembly targets.
:::

::: {#method.save_to_json .section .method}
[Source](../../../src/optionstratlib/chains/chain.rs.html#1036-1041){.src
.rightside}

#### pub fn [save_to_json](#method.save_to_json){.fn}(&self, file_path: &[str](https://doc.rust-lang.org/1.91.1/std/primitive.str.html){.primitive}) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[()](https://doc.rust-lang.org/1.91.1/std/primitive.unit.html){.primitive}, [ChainError](../../error/chains/enum.ChainError.html "enum optionstratlib::error::chains::ChainError"){.enum}\> {#pub-fn-save_to_jsonself-file_path-str---result-chainerror .code-header}
:::

::: docblock
Saves the option chain data to a JSON file.

This method serializes the option chain into JSON format and writes it
to a file at the specified path. The file will be named using the option
chain's title (symbol, expiration date, and price).

##### [§](#arguments-4){.doc-anchor}Arguments

- `file_path` - The directory path where the JSON file will be created

##### [§](#returns-8){.doc-anchor}Returns

- `Result<(), ChainError>` - Ok(()) if successful, or an Error if the
  file couldn't be created or written to.

##### [§](#note-1){.doc-anchor}Note

This method is only available on non-WebAssembly targets.
:::

::: {#method.load_from_csv .section .method}
[Source](../../../src/optionstratlib/chains/chain.rs.html#1060-1102){.src
.rightside}

#### pub fn [load_from_csv](#method.load_from_csv){.fn}(file_path: &[str](https://doc.rust-lang.org/1.91.1/std/primitive.str.html){.primitive}) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<Self, [ChainError](../../error/chains/enum.ChainError.html "enum optionstratlib::error::chains::ChainError"){.enum}\> {#pub-fn-load_from_csvfile_path-str---resultself-chainerror .code-header}
:::

::: docblock
Loads option chain data from a CSV file.

This function reads option data from a CSV file and constructs an
OptionChain. It attempts to extract the symbol, underlying price, and
expiration date from the file name.

##### [§](#arguments-5){.doc-anchor}Arguments

- `file_path` - The path to the CSV file containing option chain data

##### [§](#returns-9){.doc-anchor}Returns

- `Result<Self, ChainError>` - An OptionChain if successful, or an Error
  if the file couldn't be read or the data is invalid.

##### [§](#note-2){.doc-anchor}Note

This method is only available on non-WebAssembly targets.
:::

::: {#method.load_from_json .section .method}
[Source](../../../src/optionstratlib/chains/chain.rs.html#1121-1138){.src
.rightside}

#### pub fn [load_from_json](#method.load_from_json){.fn}(file_path: &[str](https://doc.rust-lang.org/1.91.1/std/primitive.str.html){.primitive}) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<Self, [ChainError](../../error/chains/enum.ChainError.html "enum optionstratlib::error::chains::ChainError"){.enum}\> {#pub-fn-load_from_jsonfile_path-str---resultself-chainerror .code-header}
:::

::: docblock
Loads option chain data from a JSON file.

This function deserializes an OptionChain from a JSON file and updates
the mid prices for all options in the chain.

##### [§](#arguments-6){.doc-anchor}Arguments

- `file_path` - The path to the JSON file containing serialized option
  chain data

##### [§](#returns-10){.doc-anchor}Returns

- `Result<Self, ChainError>` - An OptionChain if successful, or an Error
  if the file couldn't be read or the data is invalid.

##### [§](#note-3){.doc-anchor}Note

This method is only available on non-WebAssembly targets.
:::

::: {#method.strike_price_range_vec .section .method}
[Source](../../../src/optionstratlib/chains/chain.rs.html#1168-1182){.src
.rightside}

#### pub fn [strike_price_range_vec](#method.strike_price_range_vec){.fn}(&self, step: [f64](https://doc.rust-lang.org/1.91.1/std/primitive.f64.html){.primitive}) -\> [Option](https://doc.rust-lang.org/1.91.1/core/option/enum.Option.html "enum core::option::Option"){.enum}\<[Vec](https://doc.rust-lang.org/1.91.1/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<[f64](https://doc.rust-lang.org/1.91.1/std/primitive.f64.html){.primitive}\>\> {#pub-fn-strike_price_range_vecself-step-f64---optionvecf64 .code-header}
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
[Source](../../../src/optionstratlib/chains/chain.rs.html#1196-1334){.src
.rightside}

#### pub fn [get_random_positions](#method.get_random_positions){.fn}( &self, params: [RandomPositionsParams](../utils/struct.RandomPositionsParams.html "struct optionstratlib::chains::utils::RandomPositionsParams"){.struct}, ) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Vec](https://doc.rust-lang.org/1.91.1/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<[Position](../../model/position/struct.Position.html "struct optionstratlib::model::position::Position"){.struct}\>, [ChainError](../../error/chains/enum.ChainError.html "enum optionstratlib::error::chains::ChainError"){.enum}\> {#pub-fn-get_random_positions-self-params-randompositionsparams---resultvecposition-chainerror .code-header}
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

::: {#method.iter .section .method}
[Source](../../../src/optionstratlib/chains/chain.rs.html#1344-1346){.src
.rightside}

#### pub fn [iter](#method.iter){.fn}(&self) -\> impl [Iterator](https://doc.rust-lang.org/1.91.1/core/iter/traits/iterator/trait.Iterator.html "trait core::iter::traits::iterator::Iterator"){.trait}\<Item = &[OptionData](../struct.OptionData.html "struct optionstratlib::chains::OptionData"){.struct}\> {#pub-fn-iterself---impl-iteratoritem-optiondata .code-header}
:::

::: docblock
Returns an iterator over the `OptionData` elements.

This method provides an iterator that yields references to the
`OptionData` items contained within the structure.

##### [§](#returns-13){.doc-anchor}Returns

An iterator where each item is a reference to an `OptionData`.
:::

::: {#method.get_single_iter .section .method}
[Source](../../../src/optionstratlib/chains/chain.rs.html#1359-1361){.src
.rightside}

#### pub fn [get_single_iter](#method.get_single_iter){.fn}(&self) -\> impl [Iterator](https://doc.rust-lang.org/1.91.1/core/iter/traits/iterator/trait.Iterator.html "trait core::iter::traits::iterator::Iterator"){.trait}\<Item = &[OptionData](../struct.OptionData.html "struct optionstratlib::chains::OptionData"){.struct}\> {#pub-fn-get_single_iterself---impl-iteratoritem-optiondata .code-header}
:::

::: docblock
Returns an iterator over the `options` field in the `OptionChain`
structure.

This method provides a mechanism to traverse through the set of options
(`OptionData`) associated with an `OptionChain`.

##### [§](#returns-14){.doc-anchor}Returns

An iterator that yields references to the `OptionData` elements in the
`options` field. Since the `options` field is stored as a `BTreeSet`,
the elements are ordered in ascending order based on the sorting rules
of `BTreeSet` (typically defined by `Ord` implementation).
:::

:::: {#method.mutate_single_options .section .method}
[Source](../../../src/optionstratlib/chains/chain.rs.html#1374-1390){.src
.rightside}

#### pub fn [mutate_single_options](#method.mutate_single_options){.fn}\<F\>(&mut self, f: F) {#pub-fn-mutate_single_optionsfmut-self-f-f .code-header}

::: where
where F:
[FnMut](https://doc.rust-lang.org/1.91.1/core/ops/function/trait.FnMut.html "trait core::ops::function::FnMut"){.trait}(&mut
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
[Source](../../../src/optionstratlib/chains/chain.rs.html#1409-1414){.src
.rightside}

#### pub fn [get_single_iter_mut](#method.get_single_iter_mut){.fn}(&mut self) -\> impl [Iterator](https://doc.rust-lang.org/1.91.1/core/iter/traits/iterator/trait.Iterator.html "trait core::iter::traits::iterator::Iterator"){.trait}\<Item = [OptionData](../struct.OptionData.html "struct optionstratlib::chains::OptionData"){.struct}\> {#pub-fn-get_single_iter_mutmut-self---impl-iteratoritem-optiondata .code-header}
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

##### [§](#returns-15){.doc-anchor}Returns

An iterator yielding mutable references to `OptionData` instances.

##### [§](#examples-1){.doc-anchor}Examples
:::

::: {#method.get_double_iter .section .method}
[Source](../../../src/optionstratlib/chains/chain.rs.html#1437-1443){.src
.rightside}

#### pub fn [get_double_iter](#method.get_double_iter){.fn}( &self, ) -\> impl [Iterator](https://doc.rust-lang.org/1.91.1/core/iter/traits/iterator/trait.Iterator.html "trait core::iter::traits::iterator::Iterator"){.trait}\<Item = (&[OptionData](../struct.OptionData.html "struct optionstratlib::chains::OptionData"){.struct}, &[OptionData](../struct.OptionData.html "struct optionstratlib::chains::OptionData"){.struct})\> {#pub-fn-get_double_iter-self---impl-iteratoritem-optiondata-optiondata .code-header}
:::

:::: docblock
Returns an iterator that generates pairs of distinct option combinations
from the `OptionChain`.

This function iterates over all unique combinations of two options from
the `options` collection without repetition. In mathematical terms, it
generates combinations where order does not matter and an option cannot
combine with itself.

##### [§](#returns-16){.doc-anchor}Returns

An iterator producing tuples of references to two distinct `OptionData`
instances.

##### [§](#example-2){.doc-anchor}Example

::: example-wrap
``` {.rust .rust-example-rendered}
use tracing::info;
use optionstratlib::chains::chain::OptionChain;
use optionstratlib::{pos, Positive};
let mut option_chain = OptionChain::new("TEST", pos!(100.0), "2030-01-01".to_string(), None, None);
for (option1, option2) in option_chain.get_double_iter() {
    info!("{:?}, {:?}", option1, option2);
}
```
:::
::::

::: {#method.get_double_inclusive_iter .section .method}
[Source](../../../src/optionstratlib/chains/chain.rs.html#1467-1473){.src
.rightside}

#### pub fn [get_double_inclusive_iter](#method.get_double_inclusive_iter){.fn}( &self, ) -\> impl [Iterator](https://doc.rust-lang.org/1.91.1/core/iter/traits/iterator/trait.Iterator.html "trait core::iter::traits::iterator::Iterator"){.trait}\<Item = (&[OptionData](../struct.OptionData.html "struct optionstratlib::chains::OptionData"){.struct}, &[OptionData](../struct.OptionData.html "struct optionstratlib::chains::OptionData"){.struct})\> {#pub-fn-get_double_inclusive_iter-self---impl-iteratoritem-optiondata-optiondata .code-header}
:::

:::: docblock
Returns an iterator that generates inclusive pairs of option
combinations from the `OptionChain`.

This function iterates over all combinations of two options from the
`options` collection, including pairing an option with itself.

##### [§](#returns-17){.doc-anchor}Returns

An iterator producing tuples with two references to `OptionData`,
potentially including self-pairs (e.g., `(option, option)`).

##### [§](#example-3){.doc-anchor}Example

::: example-wrap
``` {.rust .rust-example-rendered}
use tracing::info;
use optionstratlib::chains::chain::OptionChain;
use optionstratlib::Positive;
use optionstratlib::pos_or_panic;
let mut option_chain = OptionChain::new("TEST", pos!(100.0), "2030-01-01".to_string(), None, None);
for (option1, option2) in option_chain.get_double_inclusive_iter() {
    info!("{:?}, {:?}", option1, option2);
}
```
:::
::::

::: {#method.get_triple_iter .section .method}
[Source](../../../src/optionstratlib/chains/chain.rs.html#1496-1509){.src
.rightside}

#### pub fn [get_triple_iter](#method.get_triple_iter){.fn}( &self, ) -\> impl [Iterator](https://doc.rust-lang.org/1.91.1/core/iter/traits/iterator/trait.Iterator.html "trait core::iter::traits::iterator::Iterator"){.trait}\<Item = (&[OptionData](../struct.OptionData.html "struct optionstratlib::chains::OptionData"){.struct}, &[OptionData](../struct.OptionData.html "struct optionstratlib::chains::OptionData"){.struct}, &[OptionData](../struct.OptionData.html "struct optionstratlib::chains::OptionData"){.struct})\> {#pub-fn-get_triple_iter-self---impl-iteratoritem-optiondata-optiondata-optiondata .code-header}
:::

:::: docblock
Returns an iterator that generates unique triplets of distinct option
combinations from the `OptionChain`.

This function iterates over all unique combinations of three options
from the `options` collection without repetition.

##### [§](#returns-18){.doc-anchor}Returns

An iterator producing tuples containing references to three distinct
`OptionData` instances.

##### [§](#example-4){.doc-anchor}Example

::: example-wrap
``` {.rust .rust-example-rendered}
use tracing::info;
use optionstratlib::chains::chain::OptionChain;
use optionstratlib::Positive;
use optionstratlib::pos_or_panic;
let mut option_chain = OptionChain::new("TEST", pos!(100.0), "2030-01-01".to_string(), None, None);
for (option1, option2, option3) in option_chain.get_triple_iter() {
    info!("{:?}, {:?}, {:?}", option1, option2, option3);
}
```
:::
::::

::: {#method.get_triple_inclusive_iter .section .method}
[Source](../../../src/optionstratlib/chains/chain.rs.html#1533-1548){.src
.rightside}

#### pub fn [get_triple_inclusive_iter](#method.get_triple_inclusive_iter){.fn}( &self, ) -\> impl [Iterator](https://doc.rust-lang.org/1.91.1/core/iter/traits/iterator/trait.Iterator.html "trait core::iter::traits::iterator::Iterator"){.trait}\<Item = (&[OptionData](../struct.OptionData.html "struct optionstratlib::chains::OptionData"){.struct}, &[OptionData](../struct.OptionData.html "struct optionstratlib::chains::OptionData"){.struct}, &[OptionData](../struct.OptionData.html "struct optionstratlib::chains::OptionData"){.struct})\> {#pub-fn-get_triple_inclusive_iter-self---impl-iteratoritem-optiondata-optiondata-optiondata .code-header}
:::

:::: docblock
Returns an iterator that generates inclusive triplets of option
combinations from the `OptionChain`.

This function iterates over all combinations of three options from the
`options` collection, including those where the same option may be
included more than once.

##### [§](#returns-19){.doc-anchor}Returns

An iterator producing tuples with three references to `OptionData`,
potentially including repeated elements (e.g.,
`(option1, option2, option1)`).

##### [§](#example-5){.doc-anchor}Example

::: example-wrap
``` {.rust .rust-example-rendered}
use tracing::info;
use optionstratlib::chains::chain::OptionChain;
use optionstratlib::Positive;
use optionstratlib::pos_or_panic;
let mut option_chain = OptionChain::new("TEST", pos!(100.0), "2030-01-01".to_string(), None, None);
for (option1, option2, option3) in option_chain.get_triple_inclusive_iter() {
    info!("{:?}, {:?}, {:?}", option1, option2, option3);
}
```
:::
::::

::: {#method.get_quad_iter .section .method}
[Source](../../../src/optionstratlib/chains/chain.rs.html#1571-1590){.src
.rightside}

#### pub fn [get_quad_iter](#method.get_quad_iter){.fn}( &self, ) -\> impl [Iterator](https://doc.rust-lang.org/1.91.1/core/iter/traits/iterator/trait.Iterator.html "trait core::iter::traits::iterator::Iterator"){.trait}\<Item = (&[OptionData](../struct.OptionData.html "struct optionstratlib::chains::OptionData"){.struct}, &[OptionData](../struct.OptionData.html "struct optionstratlib::chains::OptionData"){.struct}, &[OptionData](../struct.OptionData.html "struct optionstratlib::chains::OptionData"){.struct}, &[OptionData](../struct.OptionData.html "struct optionstratlib::chains::OptionData"){.struct})\> {#pub-fn-get_quad_iter-self---impl-iteratoritem-optiondata-optiondata-optiondata-optiondata .code-header}
:::

:::: docblock
Returns an iterator that generates unique quadruples of distinct option
combinations from the `OptionChain`.

This function iterates over all unique combinations of four options from
the `options` collection without repetition.

##### [§](#returns-20){.doc-anchor}Returns

An iterator producing tuples containing references to four distinct
`OptionData` instances.

##### [§](#example-6){.doc-anchor}Example

::: example-wrap
``` {.rust .rust-example-rendered}
use tracing::info;
use optionstratlib::chains::chain::OptionChain;
use optionstratlib::Positive;
use optionstratlib::pos_or_panic;
let mut option_chain = OptionChain::new("TEST", pos!(100.0), "2030-01-01".to_string(), None, None);
for (option1, option2, option3, option4) in option_chain.get_quad_iter() {
    info!("{:?}, {:?}, {:?}, {:?}", option1, option2, option3, option4);
}
```
:::
::::

::: {#method.get_quad_inclusive_iter .section .method}
[Source](../../../src/optionstratlib/chains/chain.rs.html#1614-1633){.src
.rightside}

#### pub fn [get_quad_inclusive_iter](#method.get_quad_inclusive_iter){.fn}( &self, ) -\> impl [Iterator](https://doc.rust-lang.org/1.91.1/core/iter/traits/iterator/trait.Iterator.html "trait core::iter::traits::iterator::Iterator"){.trait}\<Item = (&[OptionData](../struct.OptionData.html "struct optionstratlib::chains::OptionData"){.struct}, &[OptionData](../struct.OptionData.html "struct optionstratlib::chains::OptionData"){.struct}, &[OptionData](../struct.OptionData.html "struct optionstratlib::chains::OptionData"){.struct}, &[OptionData](../struct.OptionData.html "struct optionstratlib::chains::OptionData"){.struct})\> {#pub-fn-get_quad_inclusive_iter-self---impl-iteratoritem-optiondata-optiondata-optiondata-optiondata .code-header}
:::

:::: docblock
Returns an iterator that generates inclusive quadruples of option
combinations from the `OptionChain`.

This function iterates over all combinations of four options from the
`options` collection, including those where the same option may be
included more than once.

##### [§](#returns-21){.doc-anchor}Returns

An iterator producing tuples with four references to `OptionData`,
potentially including repeated elements (e.g.,
`(option1, option2, option1, option4)`).

##### [§](#example-7){.doc-anchor}Example

::: example-wrap
``` {.rust .rust-example-rendered}
use tracing::info;
use optionstratlib::chains::chain::OptionChain;
use optionstratlib::Positive;
use optionstratlib::pos_or_panic;
let mut option_chain = OptionChain::new("TEST", pos!(100.0), "2030-01-01".to_string(), None, None);
for (option1, option2, option3, option4) in option_chain.get_quad_inclusive_iter() {
    info!("{:?}, {:?}, {:?}, {:?}", option1, option2, option3, option4);
}
```
:::
::::

::: {#method.get_call_price .section .method}
[Source](../../../src/optionstratlib/chains/chain.rs.html#1650-1656){.src
.rightside}

#### pub fn [get_call_price](#method.get_call_price){.fn}(&self, strike: [Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}) -\> [Option](https://doc.rust-lang.org/1.91.1/core/option/enum.Option.html "enum core::option::Option"){.enum}\<[Decimal](../../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}\> {#pub-fn-get_call_priceself-strike-positive---optiondecimal .code-header}
:::

::: docblock
Retrieves the call option price for a specific strike price

This helper method finds and returns the ask price of a call option at
the specified strike price from the option chain.

##### [§](#arguments-10){.doc-anchor}Arguments

- `strike` - The strike price to look up

##### [§](#returns-22){.doc-anchor}Returns

- `Some(Decimal)` - The call option ask price if found
- `None` - If no option exists at the specified strike or if the price
  is not available

##### [§](#notes){.doc-anchor}Notes

- Uses the ask price as it represents the cost to buy the option
- Converts the price to Decimal for consistency in calculations
:::

::: {#method.get_atm_implied_volatility .section .method}
[Source](../../../src/optionstratlib/chains/chain.rs.html#1674-1677){.src
.rightside}

#### pub fn [get_atm_implied_volatility](#method.get_atm_implied_volatility){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<&[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, [ChainError](../../error/chains/enum.ChainError.html "enum optionstratlib::error::chains::ChainError"){.enum}\> {#pub-fn-get_atm_implied_volatilityself---resultpositive-chainerror .code-header}
:::

::: docblock
Retrieves the At-The-Money (ATM) implied volatility.

This function retrieves the implied volatility of the ATM option. It
calls `self.atm_option_data()` to find the ATM option and then returns a
reference to its implied volatility.

##### [§](#returns-23){.doc-anchor}Returns

- `Ok(&Option<Positive>)` - If the ATM option is found, returns a
  reference to its implied volatility, which is an `Option<Positive>`.
- `Err(ChainError)` - If the ATM option cannot be found, returns an
  error.

##### [§](#errors-1){.doc-anchor}Errors

This function returns an error if the underlying `atm_option_data()`
call fails, which can happen if the option chain is empty or no suitable
ATM option is found.
:::

::: {#method.gamma_exposure .section .method}
[Source](../../../src/optionstratlib/chains/chain.rs.html#1699-1705){.src
.rightside}

#### pub fn [gamma_exposure](#method.gamma_exposure){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Decimal](../../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}, [ChainError](../../error/chains/enum.ChainError.html "enum optionstratlib::error::chains::ChainError"){.enum}\> {#pub-fn-gamma_exposureself---resultdecimal-chainerror .code-header}
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
[Source](../../../src/optionstratlib/chains/chain.rs.html#1726-1733){.src
.rightside}

#### pub fn [delta_exposure](#method.delta_exposure){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Decimal](../../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}, [ChainError](../../error/chains/enum.ChainError.html "enum optionstratlib::error::chains::ChainError"){.enum}\> {#pub-fn-delta_exposureself---resultdecimal-chainerror .code-header}
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
[Source](../../../src/optionstratlib/chains/chain.rs.html#1754-1767){.src
.rightside}

#### pub fn [vega_exposure](#method.vega_exposure){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Decimal](../../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}, [ChainError](../../error/chains/enum.ChainError.html "enum optionstratlib::error::chains::ChainError"){.enum}\> {#pub-fn-vega_exposureself---resultdecimal-chainerror .code-header}
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
[Source](../../../src/optionstratlib/chains/chain.rs.html#1788-1801){.src
.rightside}

#### pub fn [theta_exposure](#method.theta_exposure){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Decimal](../../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}, [ChainError](../../error/chains/enum.ChainError.html "enum optionstratlib::error::chains::ChainError"){.enum}\> {#pub-fn-theta_exposureself---resultdecimal-chainerror .code-header}
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
[Source](../../../src/optionstratlib/chains/chain.rs.html#1817-1819){.src
.rightside}

#### pub fn [gamma_curve](#method.gamma_curve){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Curve](../../curves/struct.Curve.html "struct optionstratlib::curves::Curve"){.struct}, [CurveError](../../error/curves/enum.CurveError.html "enum optionstratlib::error::curves::CurveError"){.enum}\> {#pub-fn-gamma_curveself---resultcurve-curveerror .code-header}
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
[Source](../../../src/optionstratlib/chains/chain.rs.html#1835-1837){.src
.rightside}

#### pub fn [delta_curve](#method.delta_curve){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Curve](../../curves/struct.Curve.html "struct optionstratlib::curves::Curve"){.struct}, [CurveError](../../error/curves/enum.CurveError.html "enum optionstratlib::error::curves::CurveError"){.enum}\> {#pub-fn-delta_curveself---resultcurve-curveerror .code-header}
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
[Source](../../../src/optionstratlib/chains/chain.rs.html#1853-1855){.src
.rightside}

#### pub fn [vega_curve](#method.vega_curve){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Curve](../../curves/struct.Curve.html "struct optionstratlib::curves::Curve"){.struct}, [CurveError](../../error/curves/enum.CurveError.html "enum optionstratlib::error::curves::CurveError"){.enum}\> {#pub-fn-vega_curveself---resultcurve-curveerror .code-header}
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
[Source](../../../src/optionstratlib/chains/chain.rs.html#1871-1873){.src
.rightside}

#### pub fn [theta_curve](#method.theta_curve){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Curve](../../curves/struct.Curve.html "struct optionstratlib::curves::Curve"){.struct}, [CurveError](../../error/curves/enum.CurveError.html "enum optionstratlib::error::curves::CurveError"){.enum}\> {#pub-fn-theta_curveself---resultcurve-curveerror .code-header}
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
[Source](../../../src/optionstratlib/chains/chain.rs.html#1900-1920){.src
.rightside}

#### pub fn [update_expiration_date](#method.update_expiration_date){.fn}(&mut self, expiration: [String](https://doc.rust-lang.org/1.91.1/alloc/string/struct.String.html "struct alloc::string::String"){.struct}) {#pub-fn-update_expiration_datemut-self-expiration-string .code-header}
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
[Source](../../../src/optionstratlib/chains/chain.rs.html#1930-1932){.src
.rightside}

#### pub fn [get_expiration_date](#method.get_expiration_date){.fn}(&self) -\> [String](https://doc.rust-lang.org/1.91.1/alloc/string/struct.String.html "struct alloc::string::String"){.struct} {#pub-fn-get_expiration_dateself---string .code-header}
:::

::: docblock
Retrieves the expiration date of the option chain.

This method returns the expiration date associated with the option chain
as a `String`. The expiration date represents the date on which the
options in the chain will expire.

##### [§](#returns-32){.doc-anchor}Returns

A `String` representing the expiration date of the option chain.
:::

::: {#method.get_expiration .section .method}
[Source](../../../src/optionstratlib/chains/chain.rs.html#1938-1940){.src
.rightside}

#### pub fn [get_expiration](#method.get_expiration){.fn}(&self) -\> [Option](https://doc.rust-lang.org/1.91.1/core/option/enum.Option.html "enum core::option::Option"){.enum}\<[ExpirationDate](../../model/enum.ExpirationDate.html "enum optionstratlib::model::ExpirationDate"){.enum}\> {#pub-fn-get_expirationself---optionexpirationdate .code-header}
:::

::: docblock
Returns the expiration date of the option chain as an `ExpirationDate`
object.

##### [§](#returns-33){.doc-anchor}Returns

- `Option<ExpirationDate>` - The expiration date if it can be parsed, or
  `None` if parsing fails.
:::

::: {#method.get_position_with_delta .section .method}
[Source](../../../src/optionstratlib/chains/chain.rs.html#2010-2105){.src
.rightside}

#### pub fn [get_position_with_delta](#method.get_position_with_delta){.fn}( &self, target_delta: [Decimal](../../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}, side: [Side](../../model/types/enum.Side.html "enum optionstratlib::model::types::Side"){.enum}, option_style: [OptionStyle](../../model/types/enum.OptionStyle.html "enum optionstratlib::model::types::OptionStyle"){.enum}, ) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Position](../../model/position/struct.Position.html "struct optionstratlib::model::position::Position"){.struct}, [ChainError](../../error/chains/enum.ChainError.html "enum optionstratlib::error::chains::ChainError"){.enum}\> {#pub-fn-get_position_with_delta-self-target_delta-decimal-side-side-option_style-optionstyle---resultposition-chainerror .code-header}
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

##### [§](#returns-34){.doc-anchor}Returns

A `Result` containing the `Position` if a suitable option is found, or a
`ChainError` if no option with a delta less than or equal to the
`target_delta` is found.

##### [§](#errors-10){.doc-anchor}Errors

Returns a `ChainError::OptionDataError` with
`OptionDataErrorKind::InvalidDelta` if no option is found with a delta
less than or equal to the specified `target_delta`.
:::

::: {#method.get_strikes .section .method}
[Source](../../../src/optionstratlib/chains/chain.rs.html#2129-2135){.src
.rightside}

#### pub fn [get_strikes](#method.get_strikes){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Vec](https://doc.rust-lang.org/1.91.1/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\>, [ChainError](../../error/chains/enum.ChainError.html "enum optionstratlib::error::chains::ChainError"){.enum}\> {#pub-fn-get_strikesself---resultvecpositive-chainerror .code-header}
:::

::: docblock
Retrieves a collection of strike prices from the chain of options.

This method iterates through the options in the chain, extracts the
`strike_price` of each option, and returns them as a vector of
`Positive` values.

##### [§](#returns-35){.doc-anchor}Returns

This function returns a `Result`:

- On success, it returns an `Ok` variant containing a `Vec<Positive>`,
  where each element is the strike price of a corresponding option in
  the chain.
- If an error occurs, it returns an `Err` variant containing a
  `ChainError`.

##### [§](#errors-11){.doc-anchor}Errors

This function will return an error if there is any issue in processing
the options chain that prevents successful extraction of strike prices.

##### [§](#note-8){.doc-anchor}Note

- The `Positive` type for `strike_price` ensures that only valid
  positive values are included.
- An empty vector will be returned if there are no options in the chain.

##### [§](#dependencies){.doc-anchor}Dependencies

The method depends on `self.iter()` to provide access to the underlying
collection of options. Each option is expected to have a `strike_price`
field.
:::

::: {#method.get_optiondata_with_strike .section .method}
[Source](../../../src/optionstratlib/chains/chain.rs.html#2178-2205){.src
.rightside}

#### pub fn [get_optiondata_with_strike](#method.get_optiondata_with_strike){.fn}( &self, price: &[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, ) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<&[OptionData](../struct.OptionData.html "struct optionstratlib::chains::OptionData"){.struct}, [ChainError](../../error/chains/enum.ChainError.html "enum optionstratlib::error::chains::ChainError"){.enum}\> {#pub-fn-get_optiondata_with_strike-self-price-positive---resultoptiondata-chainerror .code-header}
:::

::: docblock
Retrieves an `OptionData` instance from an option chain that has a
strike price closest to the given price.

##### [§](#arguments-12){.doc-anchor}Arguments

- `price` - A reference to a `Positive`, which represents the price to
  compare against the strike prices in the option chain.

##### [§](#returns-36){.doc-anchor}Returns

- `Ok(&OptionData)` - A reference to the `OptionData` instance with the
  strike price closest to the specified price.
- `Err(ChainError)` - An error indicating the failure to retrieve the
  option data, which could occur due to:
  - The option chain being empty.
  - No matching `OptionData` found for the given price.

##### [§](#errors-12){.doc-anchor}Errors

- `ChainError` - Returned if the option chain is empty or no suitable
  option data can be found that matches the given price.

##### [§](#behavior){.doc-anchor}Behavior

- If the option chain is empty (`self.options.is_empty()`), this
  function will immediately return an error with a message indicating
  that the option data cannot be found for an empty chain.
- The function iterates through the available `OptionData` instances in
  the chain and identifies the one whose `strike_price` is closest to
  the specified `price`.
  - The comparison is done based on the absolute difference between the
    `strike_price` and `price`, with the smallest difference being
    considered the best match.
- If a matching option is found, it is returned as a reference inside an
  `Ok`.
- If no matching option is found, an error will be returned with a
  descriptive message.

##### [§](#notes-1){.doc-anchor}Notes

- The `strike_price` and `price` values are compared as decimal values
  using the `to_dec` method.
- If two or more `OptionData` instances have the same distance to the
  given `price`, the implementation will use the first instance it
  encounters based on the iteration order.
:::

::: {#method.set_optiondata_extra_params .section .method}
[Source](../../../src/optionstratlib/chains/chain.rs.html#2214-2228){.src
.rightside}

#### pub fn [set_optiondata_extra_params](#method.set_optiondata_extra_params){.fn}(&mut self) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[()](https://doc.rust-lang.org/1.91.1/std/primitive.unit.html){.primitive}, [ChainError](../../error/chains/enum.ChainError.html "enum optionstratlib::error::chains::ChainError"){.enum}\> {#pub-fn-set_optiondata_extra_paramsmut-self---result-chainerror .code-header}
:::

::: docblock
Sets additional parameters for all option data objects in the chain.

This method propagates the chain-level parameters (underlying price,
expiration date, risk-free rate, dividend yield, and symbol) to all
individual option contracts.

##### [§](#returns-37){.doc-anchor}Returns

- `Result<(), ChainError>` - Ok if successful, or an error if the
  operation fails.
:::
::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::

::: {#impl-OptionChain-1 .section .impl}
[Source](../../../src/optionstratlib/chains/chain.rs.html#2441-2540){.src
.rightside}[§](#impl-OptionChain-1){.anchor}

### impl [OptionChain](struct.OptionChain.html "struct optionstratlib::chains::chain::OptionChain"){.struct} {#impl-optionchain-1 .code-header}
:::

::::: impl-items
::: {#method.show .section .method}
[Source](../../../src/optionstratlib/chains/chain.rs.html#2447-2539){.src
.rightside}

#### pub fn [show](#method.show){.fn}(&self) {#pub-fn-showself .code-header}
:::

::: docblock
Print the option chain with colored headers to stdout.

This method prints the option chain directly to stdout using
prettytable's `printstd()` method, which properly displays colors in the
terminal. Use this method instead of `info!("{}", chain)` to see colored
headers.
:::
:::::
::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::

## Trait Implementations[§](#trait-implementations){.anchor} {#trait-implementations .section-header}

:::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::: {#trait-implementations-list}
::: {#impl-AtmIvProvider-for-OptionChain .section .impl}
[Source](../../../src/optionstratlib/volatility/traits.rs.html#108-115){.src
.rightside}[§](#impl-AtmIvProvider-for-OptionChain){.anchor}

### impl [AtmIvProvider](../../volatility/trait.AtmIvProvider.html "trait optionstratlib::volatility::AtmIvProvider"){.trait} for [OptionChain](struct.OptionChain.html "struct optionstratlib::chains::chain::OptionChain"){.struct} {#impl-atmivprovider-for-optionchain .code-header}
:::

::::: impl-items
::: {#method.atm_iv .section .method .trait-impl}
[Source](../../../src/optionstratlib/volatility/traits.rs.html#109-114){.src
.rightside}[§](#method.atm_iv){.anchor}

#### fn [atm_iv](../../volatility/trait.AtmIvProvider.html#tymethod.atm_iv){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<&[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, [VolatilityError](../../error/enum.VolatilityError.html "enum optionstratlib::error::VolatilityError"){.enum}\> {#fn-atm_ivself---resultpositive-volatilityerror .code-header}
:::

::: docblock
Get the at-the-money implied volatility [Read
more](../../volatility/trait.AtmIvProvider.html#tymethod.atm_iv)
:::
:::::

::: {#impl-BasicCurves-for-OptionChain .section .impl}
[Source](../../../src/optionstratlib/chains/chain.rs.html#2665-2709){.src
.rightside}[§](#impl-BasicCurves-for-OptionChain){.anchor}

### impl [BasicCurves](../../curves/trait.BasicCurves.html "trait optionstratlib::curves::BasicCurves"){.trait} for [OptionChain](struct.OptionChain.html "struct optionstratlib::chains::chain::OptionChain"){.struct} {#impl-basiccurves-for-optionchain .code-header}
:::

::::::: impl-items
::: {#method.curve .section .method .trait-impl}
[Source](../../../src/optionstratlib/chains/chain.rs.html#2666-2708){.src
.rightside}[§](#method.curve){.anchor}

#### fn [curve](../../curves/trait.BasicCurves.html#tymethod.curve){.fn}( &self, axis: &[BasicAxisTypes](../../model/enum.BasicAxisTypes.html "enum optionstratlib::model::BasicAxisTypes"){.enum}, option_style: &[OptionStyle](../../model/types/enum.OptionStyle.html "enum optionstratlib::model::types::OptionStyle"){.enum}, side: &[Side](../../model/types/enum.Side.html "enum optionstratlib::model::types::Side"){.enum}, ) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Curve](../../curves/struct.Curve.html "struct optionstratlib::curves::Curve"){.struct}, [CurveError](../../error/curves/enum.CurveError.html "enum optionstratlib::error::curves::CurveError"){.enum}\> {#fn-curve-self-axis-basicaxistypes-option_style-optionstyle-side-side---resultcurve-curveerror .code-header}
:::

::: docblock
Generates a curve for the specified axis type, option style, and market
side. [Read more](../../curves/trait.BasicCurves.html#tymethod.curve)
:::

::: {#method.get_curve_strike_versus .section .method .trait-impl}
[Source](../../../src/optionstratlib/curves/basic.rs.html#67-93){.src
.rightside}[§](#method.get_curve_strike_versus){.anchor}

#### fn [get_curve_strike_versus](../../curves/trait.BasicCurves.html#method.get_curve_strike_versus){.fn}( &self, axis: &[BasicAxisTypes](../../model/enum.BasicAxisTypes.html "enum optionstratlib::model::BasicAxisTypes"){.enum}, option: &[Arc](https://doc.rust-lang.org/1.91.1/alloc/sync/struct.Arc.html "struct alloc::sync::Arc"){.struct}\<[Options](../../model/option/struct.Options.html "struct optionstratlib::model::option::Options"){.struct}\>, ) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<([Decimal](../../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}, [Decimal](../../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}), [CurveError](../../error/curves/enum.CurveError.html "enum optionstratlib::error::curves::CurveError"){.enum}\> {#fn-get_curve_strike_versus-self-axis-basicaxistypes-option-arcoptions---resultdecimal-decimal-curveerror .code-header}
:::

::: docblock
Generates coordinate pairs for a specific option and axis type. [Read
more](../../curves/trait.BasicCurves.html#method.get_curve_strike_versus)
:::
:::::::

::: {#impl-BasicSurfaces-for-OptionChain .section .impl}
[Source](../../../src/optionstratlib/chains/chain.rs.html#2711-2777){.src
.rightside}[§](#impl-BasicSurfaces-for-OptionChain){.anchor}

### impl [BasicSurfaces](../../surfaces/trait.BasicSurfaces.html "trait optionstratlib::surfaces::BasicSurfaces"){.trait} for [OptionChain](struct.OptionChain.html "struct optionstratlib::chains::chain::OptionChain"){.struct} {#impl-basicsurfaces-for-optionchain .code-header}
:::

::::::::: impl-items
::: {#method.surface .section .method .trait-impl}
[Source](../../../src/optionstratlib/chains/chain.rs.html#2712-2776){.src
.rightside}[§](#method.surface){.anchor}

#### fn [surface](../../surfaces/trait.BasicSurfaces.html#tymethod.surface){.fn}( &self, axis: &[BasicAxisTypes](../../model/enum.BasicAxisTypes.html "enum optionstratlib::model::BasicAxisTypes"){.enum}, option_style: &[OptionStyle](../../model/types/enum.OptionStyle.html "enum optionstratlib::model::types::OptionStyle"){.enum}, volatility: [Option](https://doc.rust-lang.org/1.91.1/core/option/enum.Option.html "enum core::option::Option"){.enum}\<[Vec](https://doc.rust-lang.org/1.91.1/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\>\>, side: &[Side](../../model/types/enum.Side.html "enum optionstratlib::model::types::Side"){.enum}, ) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Surface](../../surfaces/struct.Surface.html "struct optionstratlib::surfaces::Surface"){.struct}, [SurfaceError](../../error/enum.SurfaceError.html "enum optionstratlib::error::SurfaceError"){.enum}\> {#fn-surface-self-axis-basicaxistypes-option_style-optionstyle-volatility-optionvecpositive-side-side---resultsurface-surfaceerror .code-header}
:::

::: docblock
Creates a surface visualization based on the specified axis type and
option parameters. [Read
more](../../surfaces/trait.BasicSurfaces.html#tymethod.surface)
:::

::: {#method.get_surface_strike_versus .section .method .trait-impl}
[Source](../../../src/optionstratlib/surfaces/basic.rs.html#64-107){.src
.rightside}[§](#method.get_surface_strike_versus){.anchor}

#### fn [get_surface_strike_versus](../../surfaces/trait.BasicSurfaces.html#method.get_surface_strike_versus){.fn}( &self, axis: &[BasicAxisTypes](../../model/enum.BasicAxisTypes.html "enum optionstratlib::model::BasicAxisTypes"){.enum}, option: &[Arc](https://doc.rust-lang.org/1.91.1/alloc/sync/struct.Arc.html "struct alloc::sync::Arc"){.struct}\<[Options](../../model/option/struct.Options.html "struct optionstratlib::model::option::Options"){.struct}\>, ) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<([Decimal](../../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}, [Decimal](../../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}, [Decimal](../../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}), [SurfaceError](../../error/enum.SurfaceError.html "enum optionstratlib::error::SurfaceError"){.enum}\> {#fn-get_surface_strike_versus-self-axis-basicaxistypes-option-arcoptions---resultdecimal-decimal-decimal-surfaceerror .code-header}
:::

::: docblock
Calculates the relationship between strike price, implied volatility,
and a selected option metric for a given option. [Read
more](../../surfaces/trait.BasicSurfaces.html#method.get_surface_strike_versus)
:::

::: {#method.get_surface_volatility_versus .section .method .trait-impl}
[Source](../../../src/optionstratlib/surfaces/basic.rs.html#131-175){.src
.rightside}[§](#method.get_surface_volatility_versus){.anchor}

#### fn [get_surface_volatility_versus](../../surfaces/trait.BasicSurfaces.html#method.get_surface_volatility_versus){.fn}( &self, axis: &[BasicAxisTypes](../../model/enum.BasicAxisTypes.html "enum optionstratlib::model::BasicAxisTypes"){.enum}, option: &[Arc](https://doc.rust-lang.org/1.91.1/alloc/sync/struct.Arc.html "struct alloc::sync::Arc"){.struct}\<[Options](../../model/option/struct.Options.html "struct optionstratlib::model::option::Options"){.struct}\>, volatility: [Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, ) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<([Decimal](../../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}, [Decimal](../../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}, [Decimal](../../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}), [SurfaceError](../../error/enum.SurfaceError.html "enum optionstratlib::error::SurfaceError"){.enum}\> {#fn-get_surface_volatility_versus-self-axis-basicaxistypes-option-arcoptions-volatility-positive---resultdecimal-decimal-decimal-surfaceerror .code-header}
:::

::: docblock
Calculates the relationship between strike price, a specified volatility
value, and a selected option metric for a given option. [Read
more](../../surfaces/trait.BasicSurfaces.html#method.get_surface_volatility_versus)
:::
:::::::::

::: {#impl-Clone-for-OptionChain .section .impl}
[Source](../../../src/optionstratlib/chains/chain.rs.html#89){.src
.rightside}[§](#impl-Clone-for-OptionChain){.anchor}

### impl [Clone](https://doc.rust-lang.org/1.91.1/core/clone/trait.Clone.html "trait core::clone::Clone"){.trait} for [OptionChain](struct.OptionChain.html "struct optionstratlib::chains::chain::OptionChain"){.struct} {#impl-clone-for-optionchain .code-header}
:::

::::::: impl-items
::: {#method.clone .section .method .trait-impl}
[Source](../../../src/optionstratlib/chains/chain.rs.html#89){.src
.rightside}[§](#method.clone){.anchor}

#### fn [clone](https://doc.rust-lang.org/1.91.1/core/clone/trait.Clone.html#tymethod.clone){.fn}(&self) -\> [OptionChain](struct.OptionChain.html "struct optionstratlib::chains::chain::OptionChain"){.struct} {#fn-cloneself---optionchain .code-header}
:::

::: docblock
Returns a duplicate of the value. [Read
more](https://doc.rust-lang.org/1.91.1/core/clone/trait.Clone.html#tymethod.clone)
:::

::: {#method.clone_from .section .method .trait-impl}
[[1.0.0]{.since title="Stable since Rust version 1.0.0"} ·
[Source](https://doc.rust-lang.org/1.91.1/src/core/clone.rs.html#245-247){.src}]{.rightside}[§](#method.clone_from){.anchor}

#### fn [clone_from](https://doc.rust-lang.org/1.91.1/core/clone/trait.Clone.html#method.clone_from){.fn}(&mut self, source: &Self) {#fn-clone_frommut-self-source-self .code-header}
:::

::: docblock
Performs copy-assignment from `source`. [Read
more](https://doc.rust-lang.org/1.91.1/core/clone/trait.Clone.html#method.clone_from)
:::
:::::::

::: {#impl-ComposeSchema-for-OptionChain .section .impl}
[Source](../../../src/optionstratlib/chains/chain.rs.html#89){.src
.rightside}[§](#impl-ComposeSchema-for-OptionChain){.anchor}

### impl ComposeSchema for [OptionChain](struct.OptionChain.html "struct optionstratlib::chains::chain::OptionChain"){.struct} {#impl-composeschema-for-optionchain .code-header}
:::

:::: impl-items
::: {#method.compose .section .method .trait-impl}
[Source](../../../src/optionstratlib/chains/chain.rs.html#89){.src
.rightside}[§](#method.compose){.anchor}

#### fn [compose](#tymethod.compose){.fn}(generics: [Vec](https://doc.rust-lang.org/1.91.1/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<[RefOr](../../../utoipa/openapi/enum.RefOr.html "enum utoipa::openapi::RefOr"){.enum}\<[Schema](../../../utoipa/openapi/schema/enum.Schema.html "enum utoipa::openapi::schema::Schema"){.enum}\>\>) -\> [RefOr](../../../utoipa/openapi/enum.RefOr.html "enum utoipa::openapi::RefOr"){.enum}\<[Schema](../../../utoipa/openapi/schema/enum.Schema.html "enum utoipa::openapi::schema::Schema"){.enum}\> {#fn-composegenerics-vecreforschema---reforschema .code-header}
:::
::::

::: {#impl-Debug-for-OptionChain .section .impl}
[Source](../../../src/optionstratlib/chains/chain.rs.html#89){.src
.rightside}[§](#impl-Debug-for-OptionChain){.anchor}

### impl [Debug](https://doc.rust-lang.org/1.91.1/core/fmt/trait.Debug.html "trait core::fmt::Debug"){.trait} for [OptionChain](struct.OptionChain.html "struct optionstratlib::chains::chain::OptionChain"){.struct} {#impl-debug-for-optionchain .code-header}
:::

::::: impl-items
::: {#method.fmt .section .method .trait-impl}
[Source](../../../src/optionstratlib/chains/chain.rs.html#89){.src
.rightside}[§](#method.fmt){.anchor}

#### fn [fmt](https://doc.rust-lang.org/1.91.1/core/fmt/trait.Debug.html#tymethod.fmt){.fn}(&self, f: &mut [Formatter](https://doc.rust-lang.org/1.91.1/core/fmt/struct.Formatter.html "struct core::fmt::Formatter"){.struct}\<\'\_\>) -\> [Result](https://doc.rust-lang.org/1.91.1/core/fmt/type.Result.html "type core::fmt::Result"){.type} {#fn-fmtself-f-mut-formatter_---result .code-header}
:::

::: docblock
Formats the value using the given formatter. [Read
more](https://doc.rust-lang.org/1.91.1/core/fmt/trait.Debug.html#tymethod.fmt)
:::
:::::

::: {#impl-Default-for-OptionChain .section .impl}
[Source](../../../src/optionstratlib/chains/chain.rs.html#2253-2257){.src
.rightside}[§](#impl-Default-for-OptionChain){.anchor}

### impl [Default](https://doc.rust-lang.org/1.91.1/core/default/trait.Default.html "trait core::default::Default"){.trait} for [OptionChain](struct.OptionChain.html "struct optionstratlib::chains::chain::OptionChain"){.struct} {#impl-default-for-optionchain .code-header}
:::

::::: impl-items
::: {#method.default .section .method .trait-impl}
[Source](../../../src/optionstratlib/chains/chain.rs.html#2254-2256){.src
.rightside}[§](#method.default){.anchor}

#### fn [default](https://doc.rust-lang.org/1.91.1/core/default/trait.Default.html#tymethod.default){.fn}() -\> Self {#fn-default---self .code-header}
:::

::: docblock
Returns the "default value" for a type. [Read
more](https://doc.rust-lang.org/1.91.1/core/default/trait.Default.html#tymethod.default)
:::
:::::

::: {#impl-Deserialize%3C'de%3E-for-OptionChain .section .impl}
[Source](../../../src/optionstratlib/chains/chain.rs.html#135-242){.src
.rightside}[§](#impl-Deserialize%3C'de%3E-for-OptionChain){.anchor}

### impl\<\'de\> [Deserialize](../../../serde_core/de/trait.Deserialize.html "trait serde_core::de::Deserialize"){.trait}\<\'de\> for [OptionChain](struct.OptionChain.html "struct optionstratlib::chains::chain::OptionChain"){.struct} {#implde-deserializede-for-optionchain .code-header}
:::

:::::: impl-items
:::: {#method.deserialize .section .method .trait-impl}
[Source](../../../src/optionstratlib/chains/chain.rs.html#136-241){.src
.rightside}[§](#method.deserialize){.anchor}

#### fn [deserialize](../../../serde_core/de/trait.Deserialize.html#tymethod.deserialize){.fn}\<D\>(deserializer: D) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<Self, D::[Error](../../../serde_core/de/trait.Deserializer.html#associatedtype.Error "type serde_core::de::Deserializer::Error"){.associatedtype}\> {#fn-deserializeddeserializer-d---resultself-derror .code-header}

::: where
where D:
[Deserializer](../../../serde_core/de/trait.Deserializer.html "trait serde_core::de::Deserializer"){.trait}\<\'de\>,
:::
::::

::: docblock
Deserialize this value from the given Serde deserializer. [Read
more](../../../serde_core/de/trait.Deserialize.html#tymethod.deserialize)
:::
::::::

::: {#impl-Display-for-OptionChain .section .impl}
[Source](../../../src/optionstratlib/chains/chain.rs.html#2542-2622){.src
.rightside}[§](#impl-Display-for-OptionChain){.anchor}

### impl [Display](https://doc.rust-lang.org/1.91.1/core/fmt/trait.Display.html "trait core::fmt::Display"){.trait} for [OptionChain](struct.OptionChain.html "struct optionstratlib::chains::chain::OptionChain"){.struct} {#impl-display-for-optionchain .code-header}
:::

::::: impl-items
::: {#method.fmt-1 .section .method .trait-impl}
[Source](../../../src/optionstratlib/chains/chain.rs.html#2543-2621){.src
.rightside}[§](#method.fmt-1){.anchor}

#### fn [fmt](https://doc.rust-lang.org/1.91.1/core/fmt/trait.Display.html#tymethod.fmt){.fn}(&self, f: &mut [Formatter](https://doc.rust-lang.org/1.91.1/core/fmt/struct.Formatter.html "struct core::fmt::Formatter"){.struct}\<\'\_\>) -\> [Result](https://doc.rust-lang.org/1.91.1/core/fmt/type.Result.html "type core::fmt::Result"){.type} {#fn-fmtself-f-mut-formatter_---result-1 .code-header}
:::

::: docblock
Formats the value using the given formatter. [Read
more](https://doc.rust-lang.org/1.91.1/core/fmt/trait.Display.html#tymethod.fmt)
:::
:::::

::: {#impl-From%3C%26OptionChain%3E-for-Positive .section .impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#856-860){.src
.rightside}[§](#impl-From%3C%26OptionChain%3E-for-Positive){.anchor}

### impl [From](https://doc.rust-lang.org/1.91.1/core/convert/trait.From.html "trait core::convert::From"){.trait}\<&[OptionChain](struct.OptionChain.html "struct optionstratlib::chains::chain::OptionChain"){.struct}\> for [Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#impl-fromoptionchain-for-positive .code-header}
:::

::::: impl-items
::: {#method.from-1 .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#857-859){.src
.rightside}[§](#method.from-1){.anchor}

#### fn [from](https://doc.rust-lang.org/1.91.1/core/convert/trait.From.html#tymethod.from){.fn}(value: &[OptionChain](struct.OptionChain.html "struct optionstratlib::chains::chain::OptionChain"){.struct}) -\> Self {#fn-fromvalue-optionchain---self .code-header}
:::

::: docblock
Converts to this type from the input type.
:::
:::::

::: {#impl-From%3C%26Vec%3COptionData%3E%3E-for-OptionChain .section .impl}
[Source](../../../src/optionstratlib/chains/chain.rs.html#2779-2811){.src
.rightside}[§](#impl-From%3C%26Vec%3COptionData%3E%3E-for-OptionChain){.anchor}

### impl [From](https://doc.rust-lang.org/1.91.1/core/convert/trait.From.html "trait core::convert::From"){.trait}\<&[Vec](https://doc.rust-lang.org/1.91.1/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<[OptionData](../struct.OptionData.html "struct optionstratlib::chains::OptionData"){.struct}\>\> for [OptionChain](struct.OptionChain.html "struct optionstratlib::chains::chain::OptionChain"){.struct} {#impl-fromvecoptiondata-for-optionchain .code-header}
:::

::::: impl-items
::: {#method.from .section .method .trait-impl}
[Source](../../../src/optionstratlib/chains/chain.rs.html#2780-2810){.src
.rightside}[§](#method.from){.anchor}

#### fn [from](https://doc.rust-lang.org/1.91.1/core/convert/trait.From.html#tymethod.from){.fn}(options: &[Vec](https://doc.rust-lang.org/1.91.1/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<[OptionData](../struct.OptionData.html "struct optionstratlib::chains::OptionData"){.struct}\>) -\> Self {#fn-fromoptions-vecoptiondata---self .code-header}
:::

::: docblock
Converts to this type from the input type.
:::
:::::

::: {#impl-From%3COptionChain%3E-for-Positive .section .impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#862-866){.src
.rightside}[§](#impl-From%3COptionChain%3E-for-Positive){.anchor}

### impl [From](https://doc.rust-lang.org/1.91.1/core/convert/trait.From.html "trait core::convert::From"){.trait}\<[OptionChain](struct.OptionChain.html "struct optionstratlib::chains::chain::OptionChain"){.struct}\> for [Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#impl-fromoptionchain-for-positive-1 .code-header}
:::

::::: impl-items
::: {#method.from-2 .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#863-865){.src
.rightside}[§](#method.from-2){.anchor}

#### fn [from](https://doc.rust-lang.org/1.91.1/core/convert/trait.From.html#tymethod.from){.fn}(value: [OptionChain](struct.OptionChain.html "struct optionstratlib::chains::chain::OptionChain"){.struct}) -\> Self {#fn-fromvalue-optionchain---self-1 .code-header}
:::

::: docblock
Converts to this type from the input type.
:::
:::::

::: {#impl-Len-for-OptionChain .section .impl}
[Source](../../../src/optionstratlib/chains/chain.rs.html#2259-2267){.src
.rightside}[§](#impl-Len-for-OptionChain){.anchor}

### impl [Len](../../utils/trait.Len.html "trait optionstratlib::utils::Len"){.trait} for [OptionChain](struct.OptionChain.html "struct optionstratlib::chains::chain::OptionChain"){.struct} {#impl-len-for-optionchain .code-header}
:::

::::::: impl-items
::: {#method.len .section .method .trait-impl}
[Source](../../../src/optionstratlib/chains/chain.rs.html#2260-2262){.src
.rightside}[§](#method.len){.anchor}

#### fn [len](../../utils/trait.Len.html#tymethod.len){.fn}(&self) -\> [usize](https://doc.rust-lang.org/1.91.1/std/primitive.usize.html){.primitive} {#fn-lenself---usize .code-header}
:::

::: docblock
Returns the number of elements in the collection or the size of the
object. [Read more](../../utils/trait.Len.html#tymethod.len)
:::

::: {#method.is_empty .section .method .trait-impl}
[Source](../../../src/optionstratlib/chains/chain.rs.html#2264-2266){.src
.rightside}[§](#method.is_empty){.anchor}

#### fn [is_empty](../../utils/trait.Len.html#method.is_empty){.fn}(&self) -\> [bool](https://doc.rust-lang.org/1.91.1/std/primitive.bool.html){.primitive} {#fn-is_emptyself---bool .code-header}
:::

::: docblock
Returns `true` if the collection contains no elements or the object has
zero size. [Read more](../../utils/trait.Len.html#method.is_empty)
:::
:::::::

::: {#impl-OptionChainParams-for-OptionChain .section .impl}
[Source](../../../src/optionstratlib/chains/chain.rs.html#2269-2287){.src
.rightside}[§](#impl-OptionChainParams-for-OptionChain){.anchor}

### impl [OptionChainParams](../utils/trait.OptionChainParams.html "trait optionstratlib::chains::utils::OptionChainParams"){.trait} for [OptionChain](struct.OptionChain.html "struct optionstratlib::chains::chain::OptionChain"){.struct} {#impl-optionchainparams-for-optionchain .code-header}
:::

::::: impl-items
::: {#method.get_params .section .method .trait-impl}
[Source](../../../src/optionstratlib/chains/chain.rs.html#2270-2286){.src
.rightside}[§](#method.get_params){.anchor}

#### fn [get_params](../utils/trait.OptionChainParams.html#tymethod.get_params){.fn}( &self, strike_price: [Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, ) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[OptionDataPriceParams](../utils/struct.OptionDataPriceParams.html "struct optionstratlib::chains::utils::OptionDataPriceParams"){.struct}, [ChainError](../../error/chains/enum.ChainError.html "enum optionstratlib::error::chains::ChainError"){.enum}\> {#fn-get_params-self-strike_price-positive---resultoptiondatapriceparams-chainerror .code-header}
:::

::: docblock
Retrieves the option pricing parameters for a given strike price. [Read
more](../utils/trait.OptionChainParams.html#tymethod.get_params)
:::
:::::

::: {#impl-Ord-for-OptionChain .section .impl}
[Source](../../../src/optionstratlib/chains/chain.rs.html#2245-2251){.src
.rightside}[§](#impl-Ord-for-OptionChain){.anchor}

### impl [Ord](https://doc.rust-lang.org/1.91.1/core/cmp/trait.Ord.html "trait core::cmp::Ord"){.trait} for [OptionChain](struct.OptionChain.html "struct optionstratlib::chains::chain::OptionChain"){.struct} {#impl-ord-for-optionchain .code-header}
:::

:::::::::::::: impl-items
::: {#method.cmp .section .method .trait-impl}
[Source](../../../src/optionstratlib/chains/chain.rs.html#2246-2250){.src
.rightside}[§](#method.cmp){.anchor}

#### fn [cmp](https://doc.rust-lang.org/1.91.1/core/cmp/trait.Ord.html#tymethod.cmp){.fn}(&self, other: &Self) -\> [Ordering](https://doc.rust-lang.org/1.91.1/core/cmp/enum.Ordering.html "enum core::cmp::Ordering"){.enum} {#fn-cmpself-other-self---ordering .code-header}
:::

::: docblock
This method returns an
[`Ordering`](https://doc.rust-lang.org/1.91.1/core/cmp/enum.Ordering.html "enum core::cmp::Ordering")
between `self` and `other`. [Read
more](https://doc.rust-lang.org/1.91.1/core/cmp/trait.Ord.html#tymethod.cmp)
:::

:::: {#method.max .section .method .trait-impl}
[[1.21.0]{.since title="Stable since Rust version 1.21.0"} ·
[Source](https://doc.rust-lang.org/1.91.1/src/core/cmp.rs.html#1023-1025){.src}]{.rightside}[§](#method.max){.anchor}

#### fn [max](https://doc.rust-lang.org/1.91.1/core/cmp/trait.Ord.html#method.max){.fn}(self, other: Self) -\> Self {#fn-maxself-other-self---self .code-header}

::: where
where Self:
[Sized](https://doc.rust-lang.org/1.91.1/core/marker/trait.Sized.html "trait core::marker::Sized"){.trait},
:::
::::

::: docblock
Compares and returns the maximum of two values. [Read
more](https://doc.rust-lang.org/1.91.1/core/cmp/trait.Ord.html#method.max)
:::

:::: {#method.min .section .method .trait-impl}
[[1.21.0]{.since title="Stable since Rust version 1.21.0"} ·
[Source](https://doc.rust-lang.org/1.91.1/src/core/cmp.rs.html#1062-1064){.src}]{.rightside}[§](#method.min){.anchor}

#### fn [min](https://doc.rust-lang.org/1.91.1/core/cmp/trait.Ord.html#method.min){.fn}(self, other: Self) -\> Self {#fn-minself-other-self---self .code-header}

::: where
where Self:
[Sized](https://doc.rust-lang.org/1.91.1/core/marker/trait.Sized.html "trait core::marker::Sized"){.trait},
:::
::::

::: docblock
Compares and returns the minimum of two values. [Read
more](https://doc.rust-lang.org/1.91.1/core/cmp/trait.Ord.html#method.min)
:::

:::: {#method.clamp .section .method .trait-impl}
[[1.50.0]{.since title="Stable since Rust version 1.50.0"} ·
[Source](https://doc.rust-lang.org/1.91.1/src/core/cmp.rs.html#1088-1090){.src}]{.rightside}[§](#method.clamp){.anchor}

#### fn [clamp](https://doc.rust-lang.org/1.91.1/core/cmp/trait.Ord.html#method.clamp){.fn}(self, min: Self, max: Self) -\> Self {#fn-clampself-min-self-max-self---self .code-header}

::: where
where Self:
[Sized](https://doc.rust-lang.org/1.91.1/core/marker/trait.Sized.html "trait core::marker::Sized"){.trait},
:::
::::

::: docblock
Restrict a value to a certain interval. [Read
more](https://doc.rust-lang.org/1.91.1/core/cmp/trait.Ord.html#method.clamp)
:::
::::::::::::::

::: {#impl-PartialEq-for-OptionChain .section .impl}
[Source](../../../src/optionstratlib/chains/chain.rs.html#2231-2235){.src
.rightside}[§](#impl-PartialEq-for-OptionChain){.anchor}

### impl [PartialEq](https://doc.rust-lang.org/1.91.1/core/cmp/trait.PartialEq.html "trait core::cmp::PartialEq"){.trait} for [OptionChain](struct.OptionChain.html "struct optionstratlib::chains::chain::OptionChain"){.struct} {#impl-partialeq-for-optionchain .code-header}
:::

::::::: impl-items
::: {#method.eq .section .method .trait-impl}
[Source](../../../src/optionstratlib/chains/chain.rs.html#2232-2234){.src
.rightside}[§](#method.eq){.anchor}

#### fn [eq](https://doc.rust-lang.org/1.91.1/core/cmp/trait.PartialEq.html#tymethod.eq){.fn}(&self, other: &Self) -\> [bool](https://doc.rust-lang.org/1.91.1/std/primitive.bool.html){.primitive} {#fn-eqself-other-self---bool .code-header}
:::

::: docblock
Tests for `self` and `other` values to be equal, and is used by `==`.
:::

::: {#method.ne .section .method .trait-impl}
[[1.0.0]{.since title="Stable since Rust version 1.0.0"} ·
[Source](https://doc.rust-lang.org/1.91.1/src/core/cmp.rs.html#264){.src}]{.rightside}[§](#method.ne){.anchor}

#### fn [ne](https://doc.rust-lang.org/1.91.1/core/cmp/trait.PartialEq.html#method.ne){.fn}(&self, other: [&Rhs](https://doc.rust-lang.org/1.91.1/std/primitive.reference.html){.primitive}) -\> [bool](https://doc.rust-lang.org/1.91.1/std/primitive.bool.html){.primitive} {#fn-neself-other-rhs---bool .code-header}
:::

::: docblock
Tests for `!=`. The default implementation is almost always sufficient,
and should not be overridden without very good reason.
:::
:::::::

::: {#impl-PartialOrd-for-OptionChain .section .impl}
[Source](../../../src/optionstratlib/chains/chain.rs.html#2239-2243){.src
.rightside}[§](#impl-PartialOrd-for-OptionChain){.anchor}

### impl [PartialOrd](https://doc.rust-lang.org/1.91.1/core/cmp/trait.PartialOrd.html "trait core::cmp::PartialOrd"){.trait} for [OptionChain](struct.OptionChain.html "struct optionstratlib::chains::chain::OptionChain"){.struct} {#impl-partialord-for-optionchain .code-header}
:::

::::::::::::: impl-items
::: {#method.partial_cmp .section .method .trait-impl}
[Source](../../../src/optionstratlib/chains/chain.rs.html#2240-2242){.src
.rightside}[§](#method.partial_cmp){.anchor}

#### fn [partial_cmp](https://doc.rust-lang.org/1.91.1/core/cmp/trait.PartialOrd.html#tymethod.partial_cmp){.fn}(&self, other: &Self) -\> [Option](https://doc.rust-lang.org/1.91.1/core/option/enum.Option.html "enum core::option::Option"){.enum}\<[Ordering](https://doc.rust-lang.org/1.91.1/core/cmp/enum.Ordering.html "enum core::cmp::Ordering"){.enum}\> {#fn-partial_cmpself-other-self---optionordering .code-header}
:::

::: docblock
This method returns an ordering between `self` and `other` values if one
exists. [Read
more](https://doc.rust-lang.org/1.91.1/core/cmp/trait.PartialOrd.html#tymethod.partial_cmp)
:::

::: {#method.lt .section .method .trait-impl}
[[1.0.0]{.since title="Stable since Rust version 1.0.0"} ·
[Source](https://doc.rust-lang.org/1.91.1/src/core/cmp.rs.html#1399){.src}]{.rightside}[§](#method.lt){.anchor}

#### fn [lt](https://doc.rust-lang.org/1.91.1/core/cmp/trait.PartialOrd.html#method.lt){.fn}(&self, other: [&Rhs](https://doc.rust-lang.org/1.91.1/std/primitive.reference.html){.primitive}) -\> [bool](https://doc.rust-lang.org/1.91.1/std/primitive.bool.html){.primitive} {#fn-ltself-other-rhs---bool .code-header}
:::

::: docblock
Tests less than (for `self` and `other`) and is used by the `<`
operator. [Read
more](https://doc.rust-lang.org/1.91.1/core/cmp/trait.PartialOrd.html#method.lt)
:::

::: {#method.le .section .method .trait-impl}
[[1.0.0]{.since title="Stable since Rust version 1.0.0"} ·
[Source](https://doc.rust-lang.org/1.91.1/src/core/cmp.rs.html#1417){.src}]{.rightside}[§](#method.le){.anchor}

#### fn [le](https://doc.rust-lang.org/1.91.1/core/cmp/trait.PartialOrd.html#method.le){.fn}(&self, other: [&Rhs](https://doc.rust-lang.org/1.91.1/std/primitive.reference.html){.primitive}) -\> [bool](https://doc.rust-lang.org/1.91.1/std/primitive.bool.html){.primitive} {#fn-leself-other-rhs---bool .code-header}
:::

::: docblock
Tests less than or equal to (for `self` and `other`) and is used by the
`<=` operator. [Read
more](https://doc.rust-lang.org/1.91.1/core/cmp/trait.PartialOrd.html#method.le)
:::

::: {#method.gt .section .method .trait-impl}
[[1.0.0]{.since title="Stable since Rust version 1.0.0"} ·
[Source](https://doc.rust-lang.org/1.91.1/src/core/cmp.rs.html#1435){.src}]{.rightside}[§](#method.gt){.anchor}

#### fn [gt](https://doc.rust-lang.org/1.91.1/core/cmp/trait.PartialOrd.html#method.gt){.fn}(&self, other: [&Rhs](https://doc.rust-lang.org/1.91.1/std/primitive.reference.html){.primitive}) -\> [bool](https://doc.rust-lang.org/1.91.1/std/primitive.bool.html){.primitive} {#fn-gtself-other-rhs---bool .code-header}
:::

::: docblock
Tests greater than (for `self` and `other`) and is used by the `>`
operator. [Read
more](https://doc.rust-lang.org/1.91.1/core/cmp/trait.PartialOrd.html#method.gt)
:::

::: {#method.ge .section .method .trait-impl}
[[1.0.0]{.since title="Stable since Rust version 1.0.0"} ·
[Source](https://doc.rust-lang.org/1.91.1/src/core/cmp.rs.html#1453){.src}]{.rightside}[§](#method.ge){.anchor}

#### fn [ge](https://doc.rust-lang.org/1.91.1/core/cmp/trait.PartialOrd.html#method.ge){.fn}(&self, other: [&Rhs](https://doc.rust-lang.org/1.91.1/std/primitive.reference.html){.primitive}) -\> [bool](https://doc.rust-lang.org/1.91.1/std/primitive.bool.html){.primitive} {#fn-geself-other-rhs---bool .code-header}
:::

::: docblock
Tests greater than or equal to (for `self` and `other`) and is used by
the `>=` operator. [Read
more](https://doc.rust-lang.org/1.91.1/core/cmp/trait.PartialOrd.html#method.ge)
:::
:::::::::::::

::: {#impl-RNDAnalysis-for-OptionChain .section .impl}
[Source](../../../src/optionstratlib/chains/chain.rs.html#2289-2439){.src
.rightside}[§](#impl-RNDAnalysis-for-OptionChain){.anchor}

### impl [RNDAnalysis](../trait.RNDAnalysis.html "trait optionstratlib::chains::RNDAnalysis"){.trait} for [OptionChain](struct.OptionChain.html "struct optionstratlib::chains::chain::OptionChain"){.struct} {#impl-rndanalysis-for-optionchain .code-header}
:::

::::::: impl-items
::: {#method.calculate_rnd .section .method .trait-impl}
[Source](../../../src/optionstratlib/chains/chain.rs.html#2301-2412){.src
.rightside}[§](#method.calculate_rnd){.anchor}

#### fn [calculate_rnd](../trait.RNDAnalysis.html#tymethod.calculate_rnd){.fn}(&self, params: &[RNDParameters](../struct.RNDParameters.html "struct optionstratlib::chains::RNDParameters"){.struct}) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[RNDResult](../struct.RNDResult.html "struct optionstratlib::chains::RNDResult"){.struct}, [ChainError](../../error/chains/enum.ChainError.html "enum optionstratlib::error::chains::ChainError"){.enum}\> {#fn-calculate_rndself-params-rndparameters---resultrndresult-chainerror .code-header}
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
[Source](../../../src/optionstratlib/chains/chain.rs.html#2422-2438){.src
.rightside}[§](#method.calculate_skew){.anchor}

#### fn [calculate_skew](../trait.RNDAnalysis.html#tymethod.calculate_skew){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Vec](https://doc.rust-lang.org/1.91.1/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<([Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, [Decimal](../../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct})\>, [ChainError](../../error/chains/enum.ChainError.html "enum optionstratlib::error::chains::ChainError"){.enum}\> {#fn-calculate_skewself---resultvecpositive-decimal-chainerror .code-header}
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
[Source](../../../src/optionstratlib/chains/chain.rs.html#110-133){.src
.rightside}[§](#impl-Serialize-for-OptionChain){.anchor}

### impl [Serialize](../../../serde_core/ser/trait.Serialize.html "trait serde_core::ser::Serialize"){.trait} for [OptionChain](struct.OptionChain.html "struct optionstratlib::chains::chain::OptionChain"){.struct} {#impl-serialize-for-optionchain .code-header}
:::

:::::: impl-items
:::: {#method.serialize .section .method .trait-impl}
[Source](../../../src/optionstratlib/chains/chain.rs.html#111-132){.src
.rightside}[§](#method.serialize){.anchor}

#### fn [serialize](../../../serde_core/ser/trait.Serialize.html#tymethod.serialize){.fn}\<S\>(&self, serializer: S) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<S::[Ok](../../../serde_core/ser/trait.Serializer.html#associatedtype.Ok "type serde_core::ser::Serializer::Ok"){.associatedtype}, S::[Error](../../../serde_core/ser/trait.Serializer.html#associatedtype.Error "type serde_core::ser::Serializer::Error"){.associatedtype}\> {#fn-serializesself-serializer-s---resultsok-serror .code-header}

::: where
where S:
[Serializer](../../../serde_core/ser/trait.Serializer.html "trait serde_core::ser::Serializer"){.trait},
:::
::::

::: docblock
Serialize this value into the given Serde serializer. [Read
more](../../../serde_core/ser/trait.Serialize.html#tymethod.serialize)
:::
::::::

::: {#impl-ToSchema-for-OptionChain .section .impl}
[Source](../../../src/optionstratlib/chains/chain.rs.html#89){.src
.rightside}[§](#impl-ToSchema-for-OptionChain){.anchor}

### impl [ToSchema](../../../utoipa/trait.ToSchema.html "trait utoipa::ToSchema"){.trait} for [OptionChain](struct.OptionChain.html "struct optionstratlib::chains::chain::OptionChain"){.struct} {#impl-toschema-for-optionchain .code-header}
:::

::::::: impl-items
::: {#method.name .section .method .trait-impl}
[Source](../../../src/optionstratlib/chains/chain.rs.html#89){.src
.rightside}[§](#method.name){.anchor}

#### fn [name](../../../utoipa/trait.ToSchema.html#method.name){.fn}() -\> [Cow](https://doc.rust-lang.org/1.91.1/alloc/borrow/enum.Cow.html "enum alloc::borrow::Cow"){.enum}\<\'static, [str](https://doc.rust-lang.org/1.91.1/std/primitive.str.html){.primitive}\> {#fn-name---cowstatic-str .code-header}
:::

::: docblock
Return name of the schema. [Read
more](../../../utoipa/trait.ToSchema.html#method.name)
:::

::: {#method.schemas .section .method .trait-impl}
[Source](../../../src/optionstratlib/chains/chain.rs.html#89){.src
.rightside}[§](#method.schemas){.anchor}

#### fn [schemas](../../../utoipa/trait.ToSchema.html#method.schemas){.fn}(schemas: &mut [Vec](https://doc.rust-lang.org/1.91.1/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<([String](https://doc.rust-lang.org/1.91.1/alloc/string/struct.String.html "struct alloc::string::String"){.struct}, [RefOr](../../../utoipa/openapi/enum.RefOr.html "enum utoipa::openapi::RefOr"){.enum}\<[Schema](../../../utoipa/openapi/schema/enum.Schema.html "enum utoipa::openapi::schema::Schema"){.enum}\>)\>) {#fn-schemasschemas-mut-vecstring-reforschema .code-header}
:::

::: docblock
Implement reference
[`utoipa::openapi::schema::Schema`](../../../utoipa/openapi/schema/enum.Schema.html "enum utoipa::openapi::schema::Schema")s
for this type. [Read
more](../../../utoipa/trait.ToSchema.html#method.schemas)
:::
:::::::

::: {#impl-VolatilitySmile-for-OptionChain .section .impl}
[Source](../../../src/optionstratlib/chains/chain.rs.html#2624-2663){.src
.rightside}[§](#impl-VolatilitySmile-for-OptionChain){.anchor}

### impl [VolatilitySmile](../../volatility/trait.VolatilitySmile.html "trait optionstratlib::volatility::VolatilitySmile"){.trait} for [OptionChain](struct.OptionChain.html "struct optionstratlib::chains::chain::OptionChain"){.struct} {#impl-volatilitysmile-for-optionchain .code-header}
:::

::::: impl-items
::: {#method.smile .section .method .trait-impl}
[Source](../../../src/optionstratlib/chains/chain.rs.html#2636-2662){.src
.rightside}[§](#method.smile){.anchor}

#### fn [smile](../../volatility/trait.VolatilitySmile.html#tymethod.smile){.fn}(&self) -\> [Curve](../../curves/struct.Curve.html "struct optionstratlib::curves::Curve"){.struct} {#fn-smileself---curve .code-header}
:::

::: docblock
Computes the volatility smile for the option chain.

This function calculates the volatility smile by interpolating the
implied volatilities for all strike prices in the option chain. It uses
the available implied volatilities from the `options` field and performs
linear interpolation to estimate missing values.

##### [§](#returns-38){.doc-anchor}Returns

A `Curve` object representing the volatility smile. The x-coordinates of
the curve correspond to the strike prices, and the y-coordinates
represent the corresponding implied volatilities.
:::
:::::

::: {#impl-Eq-for-OptionChain .section .impl}
[Source](../../../src/optionstratlib/chains/chain.rs.html#2237){.src
.rightside}[§](#impl-Eq-for-OptionChain){.anchor}

### impl [Eq](https://doc.rust-lang.org/1.91.1/core/cmp/trait.Eq.html "trait core::cmp::Eq"){.trait} for [OptionChain](struct.OptionChain.html "struct optionstratlib::chains::chain::OptionChain"){.struct} {#impl-eq-for-optionchain .code-header}
:::
::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::

## Auto Trait Implementations[§](#synthetic-implementations){.anchor} {#synthetic-implementations .section-header}

::::::::: {#synthetic-implementations-list}
::: {#impl-Freeze-for-OptionChain .section .impl}
[§](#impl-Freeze-for-OptionChain){.anchor}

### impl [Freeze](https://doc.rust-lang.org/1.91.1/core/marker/trait.Freeze.html "trait core::marker::Freeze"){.trait} for [OptionChain](struct.OptionChain.html "struct optionstratlib::chains::chain::OptionChain"){.struct} {#impl-freeze-for-optionchain .code-header}
:::

::: {#impl-RefUnwindSafe-for-OptionChain .section .impl}
[§](#impl-RefUnwindSafe-for-OptionChain){.anchor}

### impl [RefUnwindSafe](https://doc.rust-lang.org/1.91.1/core/panic/unwind_safe/trait.RefUnwindSafe.html "trait core::panic::unwind_safe::RefUnwindSafe"){.trait} for [OptionChain](struct.OptionChain.html "struct optionstratlib::chains::chain::OptionChain"){.struct} {#impl-refunwindsafe-for-optionchain .code-header}
:::

::: {#impl-Send-for-OptionChain .section .impl}
[§](#impl-Send-for-OptionChain){.anchor}

### impl [Send](https://doc.rust-lang.org/1.91.1/core/marker/trait.Send.html "trait core::marker::Send"){.trait} for [OptionChain](struct.OptionChain.html "struct optionstratlib::chains::chain::OptionChain"){.struct} {#impl-send-for-optionchain .code-header}
:::

::: {#impl-Sync-for-OptionChain .section .impl}
[§](#impl-Sync-for-OptionChain){.anchor}

### impl [Sync](https://doc.rust-lang.org/1.91.1/core/marker/trait.Sync.html "trait core::marker::Sync"){.trait} for [OptionChain](struct.OptionChain.html "struct optionstratlib::chains::chain::OptionChain"){.struct} {#impl-sync-for-optionchain .code-header}
:::

::: {#impl-Unpin-for-OptionChain .section .impl}
[§](#impl-Unpin-for-OptionChain){.anchor}

### impl [Unpin](https://doc.rust-lang.org/1.91.1/core/marker/trait.Unpin.html "trait core::marker::Unpin"){.trait} for [OptionChain](struct.OptionChain.html "struct optionstratlib::chains::chain::OptionChain"){.struct} {#impl-unpin-for-optionchain .code-header}
:::

::: {#impl-UnwindSafe-for-OptionChain .section .impl}
[§](#impl-UnwindSafe-for-OptionChain){.anchor}

### impl [UnwindSafe](https://doc.rust-lang.org/1.91.1/core/panic/unwind_safe/trait.UnwindSafe.html "trait core::panic::unwind_safe::UnwindSafe"){.trait} for [OptionChain](struct.OptionChain.html "struct optionstratlib::chains::chain::OptionChain"){.struct} {#impl-unwindsafe-for-optionchain .code-header}
:::
:::::::::

## Blanket Implementations[§](#blanket-implementations){.anchor} {#blanket-implementations .section-header}

:::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::: {#blanket-implementations-list}
:::: {#impl-Any-for-T .section .impl}
[Source](https://doc.rust-lang.org/1.91.1/src/core/any.rs.html#138){.src
.rightside}[§](#impl-Any-for-T){.anchor}

### impl\<T\> [Any](https://doc.rust-lang.org/1.91.1/core/any/trait.Any.html "trait core::any::Any"){.trait} for T {#implt-any-for-t .code-header}

::: where
where T: \'static +
?[Sized](https://doc.rust-lang.org/1.91.1/core/marker/trait.Sized.html "trait core::marker::Sized"){.trait},
:::
::::

::::: impl-items
::: {#method.type_id .section .method .trait-impl}
[Source](https://doc.rust-lang.org/1.91.1/src/core/any.rs.html#139){.src
.rightside}[§](#method.type_id){.anchor}

#### fn [type_id](https://doc.rust-lang.org/1.91.1/core/any/trait.Any.html#tymethod.type_id){.fn}(&self) -\> [TypeId](https://doc.rust-lang.org/1.91.1/core/any/struct.TypeId.html "struct core::any::TypeId"){.struct} {#fn-type_idself---typeid .code-header}
:::

::: docblock
Gets the `TypeId` of `self`. [Read
more](https://doc.rust-lang.org/1.91.1/core/any/trait.Any.html#tymethod.type_id)
:::
:::::

:::: {#impl-Borrow%3CT%3E-for-T .section .impl}
[Source](https://doc.rust-lang.org/1.91.1/src/core/borrow.rs.html#212){.src
.rightside}[§](#impl-Borrow%3CT%3E-for-T){.anchor}

### impl\<T\> [Borrow](https://doc.rust-lang.org/1.91.1/core/borrow/trait.Borrow.html "trait core::borrow::Borrow"){.trait}\<T\> for T {#implt-borrowt-for-t .code-header}

::: where
where T:
?[Sized](https://doc.rust-lang.org/1.91.1/core/marker/trait.Sized.html "trait core::marker::Sized"){.trait},
:::
::::

::::: impl-items
::: {#method.borrow .section .method .trait-impl}
[Source](https://doc.rust-lang.org/1.91.1/src/core/borrow.rs.html#214){.src
.rightside}[§](#method.borrow){.anchor}

#### fn [borrow](https://doc.rust-lang.org/1.91.1/core/borrow/trait.Borrow.html#tymethod.borrow){.fn}(&self) -\> [&T](https://doc.rust-lang.org/1.91.1/std/primitive.reference.html){.primitive} {#fn-borrowself---t .code-header}
:::

::: docblock
Immutably borrows from an owned value. [Read
more](https://doc.rust-lang.org/1.91.1/core/borrow/trait.Borrow.html#tymethod.borrow)
:::
:::::

:::: {#impl-BorrowMut%3CT%3E-for-T .section .impl}
[Source](https://doc.rust-lang.org/1.91.1/src/core/borrow.rs.html#221){.src
.rightside}[§](#impl-BorrowMut%3CT%3E-for-T){.anchor}

### impl\<T\> [BorrowMut](https://doc.rust-lang.org/1.91.1/core/borrow/trait.BorrowMut.html "trait core::borrow::BorrowMut"){.trait}\<T\> for T {#implt-borrowmutt-for-t .code-header}

::: where
where T:
?[Sized](https://doc.rust-lang.org/1.91.1/core/marker/trait.Sized.html "trait core::marker::Sized"){.trait},
:::
::::

::::: impl-items
::: {#method.borrow_mut .section .method .trait-impl}
[Source](https://doc.rust-lang.org/1.91.1/src/core/borrow.rs.html#222){.src
.rightside}[§](#method.borrow_mut){.anchor}

#### fn [borrow_mut](https://doc.rust-lang.org/1.91.1/core/borrow/trait.BorrowMut.html#tymethod.borrow_mut){.fn}(&mut self) -\> [&mut T](https://doc.rust-lang.org/1.91.1/std/primitive.reference.html){.primitive} {#fn-borrow_mutmut-self---mut-t .code-header}
:::

::: docblock
Mutably borrows from an owned value. [Read
more](https://doc.rust-lang.org/1.91.1/core/borrow/trait.BorrowMut.html#tymethod.borrow_mut)
:::
:::::

:::: {#impl-CloneToUninit-for-T .section .impl}
[Source](https://doc.rust-lang.org/1.91.1/src/core/clone.rs.html#515){.src
.rightside}[§](#impl-CloneToUninit-for-T){.anchor}

### impl\<T\> [CloneToUninit](https://doc.rust-lang.org/1.91.1/core/clone/trait.CloneToUninit.html "trait core::clone::CloneToUninit"){.trait} for T {#implt-clonetouninit-for-t .code-header}

::: where
where T:
[Clone](https://doc.rust-lang.org/1.91.1/core/clone/trait.Clone.html "trait core::clone::Clone"){.trait},
:::
::::

:::::: impl-items
::: {#method.clone_to_uninit .section .method .trait-impl}
[Source](https://doc.rust-lang.org/1.91.1/src/core/clone.rs.html#517){.src
.rightside}[§](#method.clone_to_uninit){.anchor}

#### unsafe fn [clone_to_uninit](https://doc.rust-lang.org/1.91.1/core/clone/trait.CloneToUninit.html#tymethod.clone_to_uninit){.fn}(&self, dest: [\*mut](https://doc.rust-lang.org/1.91.1/std/primitive.pointer.html){.primitive} [u8](https://doc.rust-lang.org/1.91.1/std/primitive.u8.html){.primitive}) {#unsafe-fn-clone_to_uninitself-dest-mut-u8 .code-header}
:::

[]{.item-info}

::: {.stab .unstable}
🔬This is a nightly-only experimental API. (`clone_to_uninit`)
:::

::: docblock
Performs copy-assignment from `self` to `dest`. [Read
more](https://doc.rust-lang.org/1.91.1/core/clone/trait.CloneToUninit.html#tymethod.clone_to_uninit)
:::
::::::

:::: {#impl-Comparable%3CK%3E-for-Q .section .impl}
[Source](../../../src/equivalent/lib.rs.html#104-107){.src
.rightside}[§](#impl-Comparable%3CK%3E-for-Q){.anchor}

### impl\<Q, K\> [Comparable](../../../equivalent/trait.Comparable.html "trait equivalent::Comparable"){.trait}\<K\> for Q {#implq-k-comparablek-for-q .code-header}

::: where
where Q:
[Ord](https://doc.rust-lang.org/1.91.1/core/cmp/trait.Ord.html "trait core::cmp::Ord"){.trait} +
?[Sized](https://doc.rust-lang.org/1.91.1/core/marker/trait.Sized.html "trait core::marker::Sized"){.trait},
K:
[Borrow](https://doc.rust-lang.org/1.91.1/core/borrow/trait.Borrow.html "trait core::borrow::Borrow"){.trait}\<Q\> +
?[Sized](https://doc.rust-lang.org/1.91.1/core/marker/trait.Sized.html "trait core::marker::Sized"){.trait},
:::
::::

::::: impl-items
::: {#method.compare .section .method .trait-impl}
[Source](../../../src/equivalent/lib.rs.html#110){.src
.rightside}[§](#method.compare){.anchor}

#### fn [compare](../../../equivalent/trait.Comparable.html#tymethod.compare){.fn}(&self, key: [&K](https://doc.rust-lang.org/1.91.1/std/primitive.reference.html){.primitive}) -\> [Ordering](https://doc.rust-lang.org/1.91.1/core/cmp/enum.Ordering.html "enum core::cmp::Ordering"){.enum} {#fn-compareself-key-k---ordering .code-header}
:::

::: docblock
Compare self to `key` and return their ordering.
:::
:::::

:::: {#impl-Equivalent%3CK%3E-for-Q .section .impl}
[Source](../../../src/hashbrown/lib.rs.html#167-170){.src
.rightside}[§](#impl-Equivalent%3CK%3E-for-Q){.anchor}

### impl\<Q, K\> [Equivalent](../../../hashbrown/trait.Equivalent.html "trait hashbrown::Equivalent"){.trait}\<K\> for Q {#implq-k-equivalentk-for-q .code-header}

::: where
where Q:
[Eq](https://doc.rust-lang.org/1.91.1/core/cmp/trait.Eq.html "trait core::cmp::Eq"){.trait} +
?[Sized](https://doc.rust-lang.org/1.91.1/core/marker/trait.Sized.html "trait core::marker::Sized"){.trait},
K:
[Borrow](https://doc.rust-lang.org/1.91.1/core/borrow/trait.Borrow.html "trait core::borrow::Borrow"){.trait}\<Q\> +
?[Sized](https://doc.rust-lang.org/1.91.1/core/marker/trait.Sized.html "trait core::marker::Sized"){.trait},
:::
::::

::::: impl-items
::: {#method.equivalent .section .method .trait-impl}
[Source](../../../src/hashbrown/lib.rs.html#172){.src
.rightside}[§](#method.equivalent){.anchor}

#### fn [equivalent](../../../hashbrown/trait.Equivalent.html#tymethod.equivalent){.fn}(&self, key: [&K](https://doc.rust-lang.org/1.91.1/std/primitive.reference.html){.primitive}) -\> [bool](https://doc.rust-lang.org/1.91.1/std/primitive.bool.html){.primitive} {#fn-equivalentself-key-k---bool .code-header}
:::

::: docblock
Checks if this value is equivalent to the given key. [Read
more](../../../hashbrown/trait.Equivalent.html#tymethod.equivalent)
:::
:::::

:::: {#impl-Equivalent%3CK%3E-for-Q-1 .section .impl}
[Source](../../../src/equivalent/lib.rs.html#82-85){.src
.rightside}[§](#impl-Equivalent%3CK%3E-for-Q-1){.anchor}

### impl\<Q, K\> [Equivalent](../../../equivalent/trait.Equivalent.html "trait equivalent::Equivalent"){.trait}\<K\> for Q {#implq-k-equivalentk-for-q-1 .code-header}

::: where
where Q:
[Eq](https://doc.rust-lang.org/1.91.1/core/cmp/trait.Eq.html "trait core::cmp::Eq"){.trait} +
?[Sized](https://doc.rust-lang.org/1.91.1/core/marker/trait.Sized.html "trait core::marker::Sized"){.trait},
K:
[Borrow](https://doc.rust-lang.org/1.91.1/core/borrow/trait.Borrow.html "trait core::borrow::Borrow"){.trait}\<Q\> +
?[Sized](https://doc.rust-lang.org/1.91.1/core/marker/trait.Sized.html "trait core::marker::Sized"){.trait},
:::
::::

::::: impl-items
::: {#method.equivalent-1 .section .method .trait-impl}
[Source](../../../src/equivalent/lib.rs.html#88){.src
.rightside}[§](#method.equivalent-1){.anchor}

#### fn [equivalent](../../../equivalent/trait.Equivalent.html#tymethod.equivalent){.fn}(&self, key: [&K](https://doc.rust-lang.org/1.91.1/std/primitive.reference.html){.primitive}) -\> [bool](https://doc.rust-lang.org/1.91.1/std/primitive.bool.html){.primitive} {#fn-equivalentself-key-k---bool-1 .code-header}
:::

::: docblock
Compare self to `key` and return `true` if they are equal.
:::
:::::

::: {#impl-From%3CT%3E-for-T .section .impl}
[Source](https://doc.rust-lang.org/1.91.1/src/core/convert/mod.rs.html#785){.src
.rightside}[§](#impl-From%3CT%3E-for-T){.anchor}

### impl\<T\> [From](https://doc.rust-lang.org/1.91.1/core/convert/trait.From.html "trait core::convert::From"){.trait}\<T\> for T {#implt-fromt-for-t .code-header}
:::

::::: impl-items
::: {#method.from-3 .section .method .trait-impl}
[Source](https://doc.rust-lang.org/1.91.1/src/core/convert/mod.rs.html#788){.src
.rightside}[§](#method.from-3){.anchor}

#### fn [from](https://doc.rust-lang.org/1.91.1/core/convert/trait.From.html#tymethod.from){.fn}(t: T) -\> T {#fn-fromt-t---t .code-header}
:::

::: docblock
Returns the argument unchanged.
:::
:::::

::: {#impl-Instrument-for-T .section .impl}
[Source](../../../src/tracing/instrument.rs.html#325){.src
.rightside}[§](#impl-Instrument-for-T){.anchor}

### impl\<T\> [Instrument](../../../tracing/instrument/trait.Instrument.html "trait tracing::instrument::Instrument"){.trait} for T {#implt-instrument-for-t .code-header}
:::

::::::: impl-items
::: {#method.instrument .section .method .trait-impl}
[Source](../../../src/tracing/instrument.rs.html#86){.src
.rightside}[§](#method.instrument){.anchor}

#### fn [instrument](../../../tracing/instrument/trait.Instrument.html#method.instrument){.fn}(self, span: [Span](../../../tracing/span/struct.Span.html "struct tracing::span::Span"){.struct}) -\> [Instrumented](../../../tracing/instrument/struct.Instrumented.html "struct tracing::instrument::Instrumented"){.struct}\<Self\> {#fn-instrumentself-span-span---instrumentedself .code-header}
:::

::: docblock
Instruments this type with the provided
[`Span`](../../../tracing/span/struct.Span.html "struct tracing::span::Span"),
returning an `Instrumented` wrapper. [Read
more](../../../tracing/instrument/trait.Instrument.html#method.instrument)
:::

::: {#method.in_current_span .section .method .trait-impl}
[Source](../../../src/tracing/instrument.rs.html#128){.src
.rightside}[§](#method.in_current_span){.anchor}

#### fn [in_current_span](../../../tracing/instrument/trait.Instrument.html#method.in_current_span){.fn}(self) -\> [Instrumented](../../../tracing/instrument/struct.Instrumented.html "struct tracing::instrument::Instrumented"){.struct}\<Self\> {#fn-in_current_spanself---instrumentedself .code-header}
:::

::: docblock
Instruments this type with the
[current](../../../tracing/span/struct.Span.html#method.current "associated function tracing::span::Span::current")
[`Span`](../../../tracing/span/struct.Span.html "struct tracing::span::Span"),
returning an `Instrumented` wrapper. [Read
more](../../../tracing/instrument/trait.Instrument.html#method.in_current_span)
:::
:::::::

:::: {#impl-Into%3CU%3E-for-T .section .impl}
[Source](https://doc.rust-lang.org/1.91.1/src/core/convert/mod.rs.html#767-769){.src
.rightside}[§](#impl-Into%3CU%3E-for-T){.anchor}

### impl\<T, U\> [Into](https://doc.rust-lang.org/1.91.1/core/convert/trait.Into.html "trait core::convert::Into"){.trait}\<U\> for T {#implt-u-intou-for-t .code-header}

::: where
where U:
[From](https://doc.rust-lang.org/1.91.1/core/convert/trait.From.html "trait core::convert::From"){.trait}\<T\>,
:::
::::

::::: impl-items
::: {#method.into .section .method .trait-impl}
[Source](https://doc.rust-lang.org/1.91.1/src/core/convert/mod.rs.html#777){.src
.rightside}[§](#method.into){.anchor}

#### fn [into](https://doc.rust-lang.org/1.91.1/core/convert/trait.Into.html#tymethod.into){.fn}(self) -\> U {#fn-intoself---u .code-header}
:::

::: docblock
Calls `U::from(self)`.

That is, this conversion is whatever the implementation of
[`From`](https://doc.rust-lang.org/1.91.1/core/convert/trait.From.html "trait core::convert::From")`<T> for U`
chooses to do.
:::
:::::

::: {#impl-IntoEither-for-T .section .impl}
[Source](../../../src/either/into_either.rs.html#64){.src
.rightside}[§](#impl-IntoEither-for-T){.anchor}

### impl\<T\> [IntoEither](../../../either/into_either/trait.IntoEither.html "trait either::into_either::IntoEither"){.trait} for T {#implt-intoeither-for-t .code-header}
:::

:::::::: impl-items
::: {#method.into_either .section .method .trait-impl}
[Source](../../../src/either/into_either.rs.html#29){.src
.rightside}[§](#method.into_either){.anchor}

#### fn [into_either](../../../either/into_either/trait.IntoEither.html#method.into_either){.fn}(self, into_left: [bool](https://doc.rust-lang.org/1.91.1/std/primitive.bool.html){.primitive}) -\> [Either](../../../either/enum.Either.html "enum either::Either"){.enum}\<Self, Self\> {#fn-into_eitherself-into_left-bool---eitherself-self .code-header}
:::

::: docblock
Converts `self` into a
[`Left`](../../../either/enum.Either.html#variant.Left "variant either::Either::Left")
variant of
[`Either<Self, Self>`](../../../either/enum.Either.html "enum either::Either")
if `into_left` is `true`. Converts `self` into a
[`Right`](../../../either/enum.Either.html#variant.Right "variant either::Either::Right")
variant of
[`Either<Self, Self>`](../../../either/enum.Either.html "enum either::Either")
otherwise. [Read
more](../../../either/into_either/trait.IntoEither.html#method.into_either)
:::

:::: {#method.into_either_with .section .method .trait-impl}
[Source](../../../src/either/into_either.rs.html#55-57){.src
.rightside}[§](#method.into_either_with){.anchor}

#### fn [into_either_with](../../../either/into_either/trait.IntoEither.html#method.into_either_with){.fn}\<F\>(self, into_left: F) -\> [Either](../../../either/enum.Either.html "enum either::Either"){.enum}\<Self, Self\> {#fn-into_either_withfself-into_left-f---eitherself-self .code-header}

::: where
where F:
[FnOnce](https://doc.rust-lang.org/1.91.1/core/ops/function/trait.FnOnce.html "trait core::ops::function::FnOnce"){.trait}(&Self)
-\>
[bool](https://doc.rust-lang.org/1.91.1/std/primitive.bool.html){.primitive},
:::
::::

::: docblock
Converts `self` into a
[`Left`](../../../either/enum.Either.html#variant.Left "variant either::Either::Left")
variant of
[`Either<Self, Self>`](../../../either/enum.Either.html "enum either::Either")
if `into_left(&self)` returns `true`. Converts `self` into a
[`Right`](../../../either/enum.Either.html#variant.Right "variant either::Either::Right")
variant of
[`Either<Self, Self>`](../../../either/enum.Either.html "enum either::Either")
otherwise. [Read
more](../../../either/into_either/trait.IntoEither.html#method.into_either_with)
:::
::::::::

:::: {#impl-PartialSchema-for-T .section .impl}
[Source](../../../src/utoipa/lib.rs.html#1375){.src
.rightside}[§](#impl-PartialSchema-for-T){.anchor}

### impl\<T\> [PartialSchema](../../../utoipa/trait.PartialSchema.html "trait utoipa::PartialSchema"){.trait} for T {#implt-partialschema-for-t .code-header}

::: where
where T: ComposeSchema +
?[Sized](https://doc.rust-lang.org/1.91.1/core/marker/trait.Sized.html "trait core::marker::Sized"){.trait},
:::
::::

::::: impl-items
::: {#method.schema .section .method .trait-impl}
[Source](../../../src/utoipa/lib.rs.html#1376){.src
.rightside}[§](#method.schema){.anchor}

#### fn [schema](../../../utoipa/trait.PartialSchema.html#tymethod.schema){.fn}() -\> [RefOr](../../../utoipa/openapi/enum.RefOr.html "enum utoipa::openapi::RefOr"){.enum}\<[Schema](../../../utoipa/openapi/schema/enum.Schema.html "enum utoipa::openapi::schema::Schema"){.enum}\> {#fn-schema---reforschema .code-header}
:::

::: docblock
Return ref or schema of implementing type that can then be used to
construct combined schemas.
:::
:::::

::: {#impl-Pointable-for-T .section .impl}
[Source](../../../src/crossbeam_epoch/atomic.rs.html#194){.src
.rightside}[§](#impl-Pointable-for-T){.anchor}

### impl\<T\> [Pointable](../../../crossbeam_epoch/atomic/trait.Pointable.html "trait crossbeam_epoch::atomic::Pointable"){.trait} for T {#implt-pointable-for-t .code-header}
:::

::::::::::::::: impl-items
::: {#associatedconstant.ALIGN .section .associatedconstant .trait-impl}
[Source](../../../src/crossbeam_epoch/atomic.rs.html#195){.src
.rightside}[§](#associatedconstant.ALIGN){.anchor}

#### const [ALIGN](../../../crossbeam_epoch/atomic/trait.Pointable.html#associatedconstant.ALIGN){.constant}: [usize](https://doc.rust-lang.org/1.91.1/std/primitive.usize.html){.primitive} {#const-align-usize .code-header}
:::

::: docblock
The alignment of pointer.
:::

::: {#associatedtype.Init .section .associatedtype .trait-impl}
[Source](../../../src/crossbeam_epoch/atomic.rs.html#197){.src
.rightside}[§](#associatedtype.Init){.anchor}

#### type [Init](../../../crossbeam_epoch/atomic/trait.Pointable.html#associatedtype.Init){.associatedtype} = T {#type-init-t .code-header}
:::

::: docblock
The type for initializers.
:::

::: {#method.init .section .method .trait-impl}
[Source](../../../src/crossbeam_epoch/atomic.rs.html#199){.src
.rightside}[§](#method.init){.anchor}

#### unsafe fn [init](../../../crossbeam_epoch/atomic/trait.Pointable.html#tymethod.init){.fn}(init: \<T as [Pointable](../../../crossbeam_epoch/atomic/trait.Pointable.html "trait crossbeam_epoch::atomic::Pointable"){.trait}\>::[Init](../../../crossbeam_epoch/atomic/trait.Pointable.html#associatedtype.Init "type crossbeam_epoch::atomic::Pointable::Init"){.associatedtype}) -\> [usize](https://doc.rust-lang.org/1.91.1/std/primitive.usize.html){.primitive} {#unsafe-fn-initinit-t-as-pointableinit---usize .code-header}
:::

::: docblock
Initializes a with the given initializer. [Read
more](../../../crossbeam_epoch/atomic/trait.Pointable.html#tymethod.init)
:::

::: {#method.deref .section .method .trait-impl}
[Source](../../../src/crossbeam_epoch/atomic.rs.html#203){.src
.rightside}[§](#method.deref){.anchor}

#### unsafe fn [deref](../../../crossbeam_epoch/atomic/trait.Pointable.html#tymethod.deref){.fn}\<\'a\>(ptr: [usize](https://doc.rust-lang.org/1.91.1/std/primitive.usize.html){.primitive}) -\> [&\'a T](https://doc.rust-lang.org/1.91.1/std/primitive.reference.html){.primitive} {#unsafe-fn-derefaptr-usize---a-t .code-header}
:::

::: docblock
Dereferences the given pointer. [Read
more](../../../crossbeam_epoch/atomic/trait.Pointable.html#tymethod.deref)
:::

::: {#method.deref_mut .section .method .trait-impl}
[Source](../../../src/crossbeam_epoch/atomic.rs.html#207){.src
.rightside}[§](#method.deref_mut){.anchor}

#### unsafe fn [deref_mut](../../../crossbeam_epoch/atomic/trait.Pointable.html#tymethod.deref_mut){.fn}\<\'a\>(ptr: [usize](https://doc.rust-lang.org/1.91.1/std/primitive.usize.html){.primitive}) -\> [&\'a mut T](https://doc.rust-lang.org/1.91.1/std/primitive.reference.html){.primitive} {#unsafe-fn-deref_mutaptr-usize---a-mut-t .code-header}
:::

::: docblock
Mutably dereferences the given pointer. [Read
more](../../../crossbeam_epoch/atomic/trait.Pointable.html#tymethod.deref_mut)
:::

::: {#method.drop .section .method .trait-impl}
[Source](../../../src/crossbeam_epoch/atomic.rs.html#211){.src
.rightside}[§](#method.drop){.anchor}

#### unsafe fn [drop](../../../crossbeam_epoch/atomic/trait.Pointable.html#tymethod.drop){.fn}(ptr: [usize](https://doc.rust-lang.org/1.91.1/std/primitive.usize.html){.primitive}) {#unsafe-fn-dropptr-usize .code-header}
:::

::: docblock
Drops the object pointed to by the given pointer. [Read
more](../../../crossbeam_epoch/atomic/trait.Pointable.html#tymethod.drop)
:::
:::::::::::::::

::: {#impl-Same-for-T .section .impl}
[Source](../../../src/typenum/type_operators.rs.html#34){.src
.rightside}[§](#impl-Same-for-T){.anchor}

### impl\<T\> [Same](../../../typenum/type_operators/trait.Same.html "trait typenum::type_operators::Same"){.trait} for T {#implt-same-for-t .code-header}
:::

::::: impl-items
::: {#associatedtype.Output .section .associatedtype .trait-impl}
[Source](../../../src/typenum/type_operators.rs.html#35){.src
.rightside}[§](#associatedtype.Output){.anchor}

#### type [Output](../../../typenum/type_operators/trait.Same.html#associatedtype.Output){.associatedtype} = T {#type-output-t .code-header}
:::

::: docblock
Should always be `Self`
:::
:::::

:::: {#impl-SupersetOf%3CSS%3E-for-SP .section .impl}
[Source](../../../src/simba/scalar/subset.rs.html#90){.src
.rightside}[§](#impl-SupersetOf%3CSS%3E-for-SP){.anchor}

### impl\<SS, SP\> [SupersetOf](../../../simba/scalar/subset/trait.SupersetOf.html "trait simba::scalar::subset::SupersetOf"){.trait}\<SS\> for SP {#implss-sp-supersetofss-for-sp .code-header}

::: where
where SS:
[SubsetOf](../../../simba/scalar/subset/trait.SubsetOf.html "trait simba::scalar::subset::SubsetOf"){.trait}\<SP\>,
:::
::::

::::::::::: impl-items
::: {#method.to_subset .section .method .trait-impl}
[Source](../../../src/simba/scalar/subset.rs.html#92){.src
.rightside}[§](#method.to_subset){.anchor}

#### fn [to_subset](../../../simba/scalar/subset/trait.SupersetOf.html#method.to_subset){.fn}(&self) -\> [Option](https://doc.rust-lang.org/1.91.1/core/option/enum.Option.html "enum core::option::Option"){.enum}\<SS\> {#fn-to_subsetself---optionss .code-header}
:::

::: docblock
The inverse inclusion map: attempts to construct `self` from the
equivalent element of its superset. [Read
more](../../../simba/scalar/subset/trait.SupersetOf.html#method.to_subset)
:::

::: {#method.is_in_subset .section .method .trait-impl}
[Source](../../../src/simba/scalar/subset.rs.html#97){.src
.rightside}[§](#method.is_in_subset){.anchor}

#### fn [is_in_subset](../../../simba/scalar/subset/trait.SupersetOf.html#tymethod.is_in_subset){.fn}(&self) -\> [bool](https://doc.rust-lang.org/1.91.1/std/primitive.bool.html){.primitive} {#fn-is_in_subsetself---bool .code-header}
:::

::: docblock
Checks if `self` is actually part of its subset `T` (and can be
converted to it).
:::

::: {#method.to_subset_unchecked .section .method .trait-impl}
[Source](../../../src/simba/scalar/subset.rs.html#102){.src
.rightside}[§](#method.to_subset_unchecked){.anchor}

#### fn [to_subset_unchecked](../../../simba/scalar/subset/trait.SupersetOf.html#tymethod.to_subset_unchecked){.fn}(&self) -\> SS {#fn-to_subset_uncheckedself---ss .code-header}
:::

::: docblock
Use with care! Same as `self.to_subset` but without any property checks.
Always succeeds.
:::

::: {#method.from_subset .section .method .trait-impl}
[Source](../../../src/simba/scalar/subset.rs.html#107){.src
.rightside}[§](#method.from_subset){.anchor}

#### fn [from_subset](../../../simba/scalar/subset/trait.SupersetOf.html#tymethod.from_subset){.fn}(element: [&SS](https://doc.rust-lang.org/1.91.1/std/primitive.reference.html){.primitive}) -\> SP {#fn-from_subsetelement-ss---sp .code-header}
:::

::: docblock
The inclusion map: converts `self` to the equivalent element of its
superset.
:::
:::::::::::

:::: {#impl-ToOwned-for-T .section .impl}
[Source](https://doc.rust-lang.org/1.91.1/src/alloc/borrow.rs.html#85-87){.src
.rightside}[§](#impl-ToOwned-for-T){.anchor}

### impl\<T\> [ToOwned](https://doc.rust-lang.org/1.91.1/alloc/borrow/trait.ToOwned.html "trait alloc::borrow::ToOwned"){.trait} for T {#implt-toowned-for-t .code-header}

::: where
where T:
[Clone](https://doc.rust-lang.org/1.91.1/core/clone/trait.Clone.html "trait core::clone::Clone"){.trait},
:::
::::

::::::::: impl-items
::: {#associatedtype.Owned .section .associatedtype .trait-impl}
[Source](https://doc.rust-lang.org/1.91.1/src/alloc/borrow.rs.html#89){.src
.rightside}[§](#associatedtype.Owned){.anchor}

#### type [Owned](https://doc.rust-lang.org/1.91.1/alloc/borrow/trait.ToOwned.html#associatedtype.Owned){.associatedtype} = T {#type-owned-t .code-header}
:::

::: docblock
The resulting type after obtaining ownership.
:::

::: {#method.to_owned .section .method .trait-impl}
[Source](https://doc.rust-lang.org/1.91.1/src/alloc/borrow.rs.html#90){.src
.rightside}[§](#method.to_owned){.anchor}

#### fn [to_owned](https://doc.rust-lang.org/1.91.1/alloc/borrow/trait.ToOwned.html#tymethod.to_owned){.fn}(&self) -\> T {#fn-to_ownedself---t .code-header}
:::

::: docblock
Creates owned data from borrowed data, usually by cloning. [Read
more](https://doc.rust-lang.org/1.91.1/alloc/borrow/trait.ToOwned.html#tymethod.to_owned)
:::

::: {#method.clone_into .section .method .trait-impl}
[Source](https://doc.rust-lang.org/1.91.1/src/alloc/borrow.rs.html#94){.src
.rightside}[§](#method.clone_into){.anchor}

#### fn [clone_into](https://doc.rust-lang.org/1.91.1/alloc/borrow/trait.ToOwned.html#method.clone_into){.fn}(&self, target: [&mut T](https://doc.rust-lang.org/1.91.1/std/primitive.reference.html){.primitive}) {#fn-clone_intoself-target-mut-t .code-header}
:::

::: docblock
Uses borrowed data to replace owned data, usually by cloning. [Read
more](https://doc.rust-lang.org/1.91.1/alloc/borrow/trait.ToOwned.html#method.clone_into)
:::
:::::::::

:::: {#impl-ToString-for-T .section .impl}
[Source](https://doc.rust-lang.org/1.91.1/src/alloc/string.rs.html#2796){.src
.rightside}[§](#impl-ToString-for-T){.anchor}

### impl\<T\> [ToString](https://doc.rust-lang.org/1.91.1/alloc/string/trait.ToString.html "trait alloc::string::ToString"){.trait} for T {#implt-tostring-for-t .code-header}

::: where
where T:
[Display](https://doc.rust-lang.org/1.91.1/core/fmt/trait.Display.html "trait core::fmt::Display"){.trait} +
?[Sized](https://doc.rust-lang.org/1.91.1/core/marker/trait.Sized.html "trait core::marker::Sized"){.trait},
:::
::::

::::: impl-items
::: {#method.to_string .section .method .trait-impl}
[Source](https://doc.rust-lang.org/1.91.1/src/alloc/string.rs.html#2798){.src
.rightside}[§](#method.to_string){.anchor}

#### fn [to_string](https://doc.rust-lang.org/1.91.1/alloc/string/trait.ToString.html#tymethod.to_string){.fn}(&self) -\> [String](https://doc.rust-lang.org/1.91.1/alloc/string/struct.String.html "struct alloc::string::String"){.struct} {#fn-to_stringself---string .code-header}
:::

::: docblock
Converts the given value to a `String`. [Read
more](https://doc.rust-lang.org/1.91.1/alloc/string/trait.ToString.html#tymethod.to_string)
:::
:::::

:::: {#impl-TryFrom%3CU%3E-for-T .section .impl}
[Source](https://doc.rust-lang.org/1.91.1/src/core/convert/mod.rs.html#827-829){.src
.rightside}[§](#impl-TryFrom%3CU%3E-for-T){.anchor}

### impl\<T, U\> [TryFrom](https://doc.rust-lang.org/1.91.1/core/convert/trait.TryFrom.html "trait core::convert::TryFrom"){.trait}\<U\> for T {#implt-u-tryfromu-for-t .code-header}

::: where
where U:
[Into](https://doc.rust-lang.org/1.91.1/core/convert/trait.Into.html "trait core::convert::Into"){.trait}\<T\>,
:::
::::

::::::: impl-items
::: {#associatedtype.Error-1 .section .associatedtype .trait-impl}
[Source](https://doc.rust-lang.org/1.91.1/src/core/convert/mod.rs.html#831){.src
.rightside}[§](#associatedtype.Error-1){.anchor}

#### type [Error](https://doc.rust-lang.org/1.91.1/core/convert/trait.TryFrom.html#associatedtype.Error){.associatedtype} = [Infallible](https://doc.rust-lang.org/1.91.1/core/convert/enum.Infallible.html "enum core::convert::Infallible"){.enum} {#type-error-infallible .code-header}
:::

::: docblock
The type returned in the event of a conversion error.
:::

::: {#method.try_from .section .method .trait-impl}
[Source](https://doc.rust-lang.org/1.91.1/src/core/convert/mod.rs.html#834){.src
.rightside}[§](#method.try_from){.anchor}

#### fn [try_from](https://doc.rust-lang.org/1.91.1/core/convert/trait.TryFrom.html#tymethod.try_from){.fn}(value: U) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<T, \<T as [TryFrom](https://doc.rust-lang.org/1.91.1/core/convert/trait.TryFrom.html "trait core::convert::TryFrom"){.trait}\<U\>\>::[Error](https://doc.rust-lang.org/1.91.1/core/convert/trait.TryFrom.html#associatedtype.Error "type core::convert::TryFrom::Error"){.associatedtype}\> {#fn-try_fromvalue-u---resultt-t-as-tryfromuerror .code-header}
:::

::: docblock
Performs the conversion.
:::
:::::::

:::: {#impl-TryInto%3CU%3E-for-T .section .impl}
[Source](https://doc.rust-lang.org/1.91.1/src/core/convert/mod.rs.html#811-813){.src
.rightside}[§](#impl-TryInto%3CU%3E-for-T){.anchor}

### impl\<T, U\> [TryInto](https://doc.rust-lang.org/1.91.1/core/convert/trait.TryInto.html "trait core::convert::TryInto"){.trait}\<U\> for T {#implt-u-tryintou-for-t .code-header}

::: where
where U:
[TryFrom](https://doc.rust-lang.org/1.91.1/core/convert/trait.TryFrom.html "trait core::convert::TryFrom"){.trait}\<T\>,
:::
::::

::::::: impl-items
::: {#associatedtype.Error .section .associatedtype .trait-impl}
[Source](https://doc.rust-lang.org/1.91.1/src/core/convert/mod.rs.html#815){.src
.rightside}[§](#associatedtype.Error){.anchor}

#### type [Error](https://doc.rust-lang.org/1.91.1/core/convert/trait.TryInto.html#associatedtype.Error){.associatedtype} = \<U as [TryFrom](https://doc.rust-lang.org/1.91.1/core/convert/trait.TryFrom.html "trait core::convert::TryFrom"){.trait}\<T\>\>::[Error](https://doc.rust-lang.org/1.91.1/core/convert/trait.TryFrom.html#associatedtype.Error "type core::convert::TryFrom::Error"){.associatedtype} {#type-error-u-as-tryfromterror .code-header}
:::

::: docblock
The type returned in the event of a conversion error.
:::

::: {#method.try_into .section .method .trait-impl}
[Source](https://doc.rust-lang.org/1.91.1/src/core/convert/mod.rs.html#818){.src
.rightside}[§](#method.try_into){.anchor}

#### fn [try_into](https://doc.rust-lang.org/1.91.1/core/convert/trait.TryInto.html#tymethod.try_into){.fn}(self) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<U, \<U as [TryFrom](https://doc.rust-lang.org/1.91.1/core/convert/trait.TryFrom.html "trait core::convert::TryFrom"){.trait}\<T\>\>::[Error](https://doc.rust-lang.org/1.91.1/core/convert/trait.TryFrom.html#associatedtype.Error "type core::convert::TryFrom::Error"){.associatedtype}\> {#fn-try_intoself---resultu-u-as-tryfromterror .code-header}
:::

::: docblock
Performs the conversion.
:::
:::::::

:::: {#impl-VZip%3CV%3E-for-T .section .impl}
[Source](../../../src/ppv_lite86/types.rs.html#221-223){.src
.rightside}[§](#impl-VZip%3CV%3E-for-T){.anchor}

### impl\<V, T\> [VZip](../../../ppv_lite86/types/trait.VZip.html "trait ppv_lite86::types::VZip"){.trait}\<V\> for T {#implv-t-vzipv-for-t .code-header}

::: where
where V:
[MultiLane](../../../ppv_lite86/types/trait.MultiLane.html "trait ppv_lite86::types::MultiLane"){.trait}\<T\>,
:::
::::

:::: impl-items
::: {#method.vzip .section .method .trait-impl}
[Source](../../../src/ppv_lite86/types.rs.html#226){.src
.rightside}[§](#method.vzip){.anchor}

#### fn [vzip](../../../ppv_lite86/types/trait.VZip.html#tymethod.vzip){.fn}(self) -\> V {#fn-vzipself---v .code-header}
:::
::::

::: {#impl-WithSubscriber-for-T .section .impl}
[Source](../../../src/tracing/instrument.rs.html#393){.src
.rightside}[§](#impl-WithSubscriber-for-T){.anchor}

### impl\<T\> [WithSubscriber](../../../tracing/instrument/trait.WithSubscriber.html "trait tracing::instrument::WithSubscriber"){.trait} for T {#implt-withsubscriber-for-t .code-header}
:::

:::::::: impl-items
:::: {#method.with_subscriber .section .method .trait-impl}
[Source](../../../src/tracing/instrument.rs.html#176-178){.src
.rightside}[§](#method.with_subscriber){.anchor}

#### fn [with_subscriber](../../../tracing/instrument/trait.WithSubscriber.html#method.with_subscriber){.fn}\<S\>(self, subscriber: S) -\> [WithDispatch](../../../tracing/instrument/struct.WithDispatch.html "struct tracing::instrument::WithDispatch"){.struct}\<Self\> {#fn-with_subscribersself-subscriber-s---withdispatchself .code-header}

::: where
where S:
[Into](https://doc.rust-lang.org/1.91.1/core/convert/trait.Into.html "trait core::convert::Into"){.trait}\<[Dispatch](../../../tracing_core/dispatcher/struct.Dispatch.html "struct tracing_core::dispatcher::Dispatch"){.struct}\>,
:::
::::

::: docblock
Attaches the provided
[`Subscriber`](../../../tracing_core/subscriber/trait.Subscriber.html "trait tracing_core::subscriber::Subscriber")
to this type, returning a
[`WithDispatch`](../../../tracing/instrument/struct.WithDispatch.html "struct tracing::instrument::WithDispatch")
wrapper. [Read
more](../../../tracing/instrument/trait.WithSubscriber.html#method.with_subscriber)
:::

::: {#method.with_current_subscriber .section .method .trait-impl}
[Source](../../../src/tracing/instrument.rs.html#228){.src
.rightside}[§](#method.with_current_subscriber){.anchor}

#### fn [with_current_subscriber](../../../tracing/instrument/trait.WithSubscriber.html#method.with_current_subscriber){.fn}(self) -\> [WithDispatch](../../../tracing/instrument/struct.WithDispatch.html "struct tracing::instrument::WithDispatch"){.struct}\<Self\> {#fn-with_current_subscriberself---withdispatchself .code-header}
:::

::: docblock
Attaches the current
[default](../../../tracing/dispatcher/index.html#setting-the-default-subscriber "mod tracing::dispatcher")
[`Subscriber`](../../../tracing_core/subscriber/trait.Subscriber.html "trait tracing_core::subscriber::Subscriber")
to this type, returning a
[`WithDispatch`](../../../tracing/instrument/struct.WithDispatch.html "struct tracing::instrument::WithDispatch")
wrapper. [Read
more](../../../tracing/instrument/trait.WithSubscriber.html#method.with_current_subscriber)
:::
::::::::

:::: {#impl-DeserializeOwned-for-T .section .impl}
[Source](../../../src/serde_core/de/mod.rs.html#633){.src
.rightside}[§](#impl-DeserializeOwned-for-T){.anchor}

### impl\<T\> [DeserializeOwned](../../../serde_core/de/trait.DeserializeOwned.html "trait serde_core::de::DeserializeOwned"){.trait} for T {#implt-deserializeowned-for-t .code-header}

::: where
where T: for\<\'de\>
[Deserialize](../../../serde_core/de/trait.Deserialize.html "trait serde_core::de::Deserialize"){.trait}\<\'de\>,
:::
::::

:::: {#impl-Scalar-for-T .section .impl}
[Source](../../../src/nalgebra/base/scalar.rs.html#8){.src
.rightside}[§](#impl-Scalar-for-T){.anchor}

### impl\<T\> [Scalar](../../../nalgebra/base/scalar/trait.Scalar.html "trait nalgebra::base::scalar::Scalar"){.trait} for T {#implt-scalar-for-t .code-header}

::: where
where T: \'static +
[Clone](https://doc.rust-lang.org/1.91.1/core/clone/trait.Clone.html "trait core::clone::Clone"){.trait} +
[PartialEq](https://doc.rust-lang.org/1.91.1/core/cmp/trait.PartialEq.html "trait core::cmp::PartialEq"){.trait} +
[Debug](https://doc.rust-lang.org/1.91.1/core/fmt/trait.Debug.html "trait core::fmt::Debug"){.trait},
:::
::::
::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
:::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
