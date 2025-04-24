::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::: width-limiter
:::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../index.html)::[chains](index.html)
:::

# Struct [OptionData]{.struct}Copy item path

[[Source](../../src/optionstratlib/chains/optiondata.rs.html#69-126){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub struct OptionData {
    pub call_middle: Option<Positive>,
    pub put_middle: Option<Positive>,
    pub options: Option<Box<FourOptions>>,
    /* private fields */
}
```

Expand description

::: docblock
Struct representing a row in an option chain with detailed pricing and
analytics data.

This struct encapsulates the complete market data for an options
contract at a specific strike price, including bid/ask prices for both
call and put options, implied volatility, the Greeks (delta, gamma),
volume, and open interest. It provides all the essential information
needed for options analysis and trading decision-making.

## [§](#fields-1){.doc-anchor}Fields {#fields-1}

- `strike_price` - The strike price of the option, represented as a
  positive floating-point number.
- `call_bid` - The bid price for the call option, represented as an
  optional positive floating-point number. May be `None` if market data
  is unavailable.
- `call_ask` - The ask price for the call option, represented as an
  optional positive floating-point number. May be `None` if market data
  is unavailable.
- `put_bid` - The bid price for the put option, represented as an
  optional positive floating-point number. May be `None` if market data
  is unavailable.
- `put_ask` - The ask price for the put option, represented as an
  optional positive floating-point number. May be `None` if market data
  is unavailable.
- `call_middle` - The mid-price between call bid and ask, represented as
  an optional positive floating-point number. May be `None` if
  underlying bid/ask data is unavailable.
- `put_middle` - The mid-price between put bid and ask, represented as
  an optional positive floating-point number. May be `None` if
  underlying bid/ask data is unavailable.
- `implied_volatility` - The implied volatility of the option,
  represented as an optional positive floating-point number. May be
  `None` if it cannot be calculated from available market data.
- `delta_call` - The delta of the call option, represented as an
  optional decimal number. Measures the rate of change of the option
  price with respect to changes in the underlying asset price.
- `delta_put` - The delta of the put option, represented as an optional
  decimal number. Measures the rate of change of the option price with
  respect to changes in the underlying asset price.
- `gamma` - The gamma of the option, represented as an optional decimal
  number. Measures the rate of change of delta with respect to changes
  in the underlying asset price.
- `volume` - The trading volume of the option, represented as an
  optional positive floating-point number. May be `None` if data is not
  available.
- `open_interest` - The open interest of the option, represented as an
  optional unsigned integer. Represents the total number of outstanding
  option contracts that have not been settled.
- `options` - An optional boxed reference to a `FourOptions` struct that
  may contain the actual option contracts represented by this data. This
  field is not serialized.

## [§](#usage){.doc-anchor}Usage

This struct is typically used to represent a single row in an option
chain table, providing comprehensive market data for options at a
specific strike price. It's useful for option pricing models, strategy
analysis, and trading applications.

## [§](#serialization){.doc-anchor}Serialization

This struct implements Serialize and Deserialize traits, with fields
that are `None` being skipped during serialization to produce more
compact JSON output.
:::

## Fields[§](#fields){.anchor} {#fields .fields .section-header}

[[§](#structfield.call_middle){.anchor
.field}`call_middle: `[`Option`](https://doc.rust-lang.org/1.86.0/core/option/enum.Option.html "enum core::option::Option"){.enum}`<`[`Positive`](../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}`>`]{#structfield.call_middle
.structfield .section-header}

::: docblock
The mid-price between call bid and ask. Calculated as (bid + ask) / 2.
:::

[[§](#structfield.put_middle){.anchor
.field}`put_middle: `[`Option`](https://doc.rust-lang.org/1.86.0/core/option/enum.Option.html "enum core::option::Option"){.enum}`<`[`Positive`](../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}`>`]{#structfield.put_middle
.structfield .section-header}

::: docblock
The mid-price between put bid and ask. Calculated as (bid + ask) / 2.
:::

[[§](#structfield.options){.anchor
.field}`options: `[`Option`](https://doc.rust-lang.org/1.86.0/core/option/enum.Option.html "enum core::option::Option"){.enum}`<`[`Box`](https://doc.rust-lang.org/1.86.0/alloc/boxed/struct.Box.html "struct alloc::boxed::Box"){.struct}`<`[`FourOptions`](struct.FourOptions.html "struct optionstratlib::chains::FourOptions"){.struct}`>>`]{#structfield.options
.structfield .section-header}

::: docblock
Optional reference to the actual option contracts represented by this
data. This field is not serialized.
:::

## Implementations[§](#implementations){.anchor} {#implementations .section-header}

::::::::::::::::::::::::::::::::::::::::::: {#implementations-list}
::: {#impl-OptionData .section .impl}
[Source](../../src/optionstratlib/chains/optiondata.rs.html#128-1180){.src
.rightside}[§](#impl-OptionData){.anchor}

### impl [OptionData](struct.OptionData.html "struct optionstratlib::chains::OptionData"){.struct} {#impl-optiondata .code-header}
:::

::::::::::::::::::::::::::::::::::::::::: impl-items
::: {#method.new .section .method}
[Source](../../src/optionstratlib/chains/optiondata.rs.html#159-188){.src
.rightside}

#### pub fn [new](#method.new){.fn}( strike_price: [Positive](../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, call_bid: [Option](https://doc.rust-lang.org/1.86.0/core/option/enum.Option.html "enum core::option::Option"){.enum}\<[Positive](../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\>, call_ask: [Option](https://doc.rust-lang.org/1.86.0/core/option/enum.Option.html "enum core::option::Option"){.enum}\<[Positive](../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\>, put_bid: [Option](https://doc.rust-lang.org/1.86.0/core/option/enum.Option.html "enum core::option::Option"){.enum}\<[Positive](../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\>, put_ask: [Option](https://doc.rust-lang.org/1.86.0/core/option/enum.Option.html "enum core::option::Option"){.enum}\<[Positive](../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\>, implied_volatility: [Option](https://doc.rust-lang.org/1.86.0/core/option/enum.Option.html "enum core::option::Option"){.enum}\<[Positive](../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\>, delta_call: [Option](https://doc.rust-lang.org/1.86.0/core/option/enum.Option.html "enum core::option::Option"){.enum}\<Decimal\>, delta_put: [Option](https://doc.rust-lang.org/1.86.0/core/option/enum.Option.html "enum core::option::Option"){.enum}\<Decimal\>, gamma: [Option](https://doc.rust-lang.org/1.86.0/core/option/enum.Option.html "enum core::option::Option"){.enum}\<Decimal\>, volume: [Option](https://doc.rust-lang.org/1.86.0/core/option/enum.Option.html "enum core::option::Option"){.enum}\<[Positive](../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\>, open_interest: [Option](https://doc.rust-lang.org/1.86.0/core/option/enum.Option.html "enum core::option::Option"){.enum}\<[u64](https://doc.rust-lang.org/1.86.0/std/primitive.u64.html){.primitive}\>, ) -\> Self {#pub-fn-new-strike_price-positive-call_bid-optionpositive-call_ask-optionpositive-put_bid-optionpositive-put_ask-optionpositive-implied_volatility-optionpositive-delta_call-optiondecimal-delta_put-optiondecimal-gamma-optiondecimal-volume-optionpositive-open_interest-optionu64---self .code-header}
:::

::: docblock
Creates a new instance of `OptionData` with the given option market
parameters.

This constructor creates an `OptionData` structure that represents a
single row in an options chain, containing market data for both call and
put options at a specific strike price. The middle prices for calls and
puts are initially set to `None` and can be calculated later if needed.

##### [§](#parameters){.doc-anchor}Parameters

- `strike_price` - The strike price of the option contract, guaranteed
  to be positive.
- `call_bid` - The bid price for the call option. `None` if market data
  is unavailable.
- `call_ask` - The ask price for the call option. `None` if market data
  is unavailable.
- `put_bid` - The bid price for the put option. `None` if market data is
  unavailable.
- `put_ask` - The ask price for the put option. `None` if market data is
  unavailable.
- `implied_volatility` - The implied volatility derived from option
  prices. `None` if not calculable.
- `delta_call` - The delta of the call option, measuring price
  sensitivity to underlying changes.
- `delta_put` - The delta of the put option, measuring price sensitivity
  to underlying changes.
- `gamma` - The gamma of the option, measuring the rate of change in
  delta.
- `volume` - The trading volume of the option, indicating market
  activity.
- `open_interest` - The number of outstanding option contracts that have
  not been settled.

##### [§](#returns){.doc-anchor}Returns

A new `OptionData` instance with the specified parameters and with
`call_middle`, `put_middle`, and `options` fields initialized to `None`.

##### [§](#note){.doc-anchor}Note

This function allows many optional parameters to accommodate scenarios
where not all market data is available from data providers.
:::

::: {#method.validate .section .method}
[Source](../../src/optionstratlib/chains/optiondata.rs.html#205-223){.src
.rightside}

#### pub fn [validate](#method.validate){.fn}(&self) -\> [bool](https://doc.rust-lang.org/1.86.0/std/primitive.bool.html){.primitive} {#pub-fn-validateself---bool .code-header}
:::

::: docblock
Validates the option data to ensure it meets the required criteria for
calculations.

This method performs a series of validation checks to ensure that the
option data is complete and valid for further processing or analysis. It
verifies:

1.  The strike price is not zero
2.  Implied volatility is present
3.  Call option data is valid (via `valid_call()`)
4.  Put option data is valid (via `valid_put()`)

Each validation failure is logged as an error for debugging and
troubleshooting.

##### [§](#returns-1){.doc-anchor}Returns

- `true` - If all validation checks pass, indicating the option data is
  valid
- `false` - If any validation check fails, indicating the option data is
  incomplete or invalid
:::

::: {#method.get_call_buy_price .section .method}
[Source](../../src/optionstratlib/chains/optiondata.rs.html#267-269){.src
.rightside}

#### pub fn [get_call_buy_price](#method.get_call_buy_price){.fn}(&self) -\> [Option](https://doc.rust-lang.org/1.86.0/core/option/enum.Option.html "enum core::option::Option"){.enum}\<[Positive](../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\> {#pub-fn-get_call_buy_priceself---optionpositive .code-header}
:::

::: docblock
Retrieves the price at which a call option can be purchased.

This method returns the ask price for a call option, which is the price
a buyer would pay to purchase the call option.

##### [§](#returns-2){.doc-anchor}Returns

The call option's ask price as a `Positive` value, or `None` if the
price is unavailable.
:::

::: {#method.get_call_sell_price .section .method}
[Source](../../src/optionstratlib/chains/optiondata.rs.html#279-281){.src
.rightside}

#### pub fn [get_call_sell_price](#method.get_call_sell_price){.fn}(&self) -\> [Option](https://doc.rust-lang.org/1.86.0/core/option/enum.Option.html "enum core::option::Option"){.enum}\<[Positive](../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\> {#pub-fn-get_call_sell_priceself---optionpositive .code-header}
:::

::: docblock
Retrieves the price at which a call option can be sold.

This method returns the bid price for a call option, which is the price
a seller would receive when selling the call option.

##### [§](#returns-3){.doc-anchor}Returns

The call option's bid price as a `Positive` value, or `None` if the
price is unavailable.
:::

::: {#method.get_put_buy_price .section .method}
[Source](../../src/optionstratlib/chains/optiondata.rs.html#291-293){.src
.rightside}

#### pub fn [get_put_buy_price](#method.get_put_buy_price){.fn}(&self) -\> [Option](https://doc.rust-lang.org/1.86.0/core/option/enum.Option.html "enum core::option::Option"){.enum}\<[Positive](../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\> {#pub-fn-get_put_buy_priceself---optionpositive .code-header}
:::

::: docblock
Retrieves the price at which a put option can be purchased.

This method returns the ask price for a put option, which is the price a
buyer would pay to purchase the put option.

##### [§](#returns-4){.doc-anchor}Returns

The put option's ask price as a `Positive` value, or `None` if the price
is unavailable.
:::

::: {#method.get_put_sell_price .section .method}
[Source](../../src/optionstratlib/chains/optiondata.rs.html#303-305){.src
.rightside}

#### pub fn [get_put_sell_price](#method.get_put_sell_price){.fn}(&self) -\> [Option](https://doc.rust-lang.org/1.86.0/core/option/enum.Option.html "enum core::option::Option"){.enum}\<[Positive](../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\> {#pub-fn-get_put_sell_priceself---optionpositive .code-header}
:::

::: docblock
Retrieves the price at which a put option can be sold.

This method returns the bid price for a put option, which is the price a
seller would receive when selling the put option.

##### [§](#returns-5){.doc-anchor}Returns

The put option's bid price as a `Positive` value, or `None` if the price
is unavailable.
:::

::: {#method.some_price_is_none .section .method}
[Source](../../src/optionstratlib/chains/optiondata.rs.html#313-318){.src
.rightside}

#### pub fn [some_price_is_none](#method.some_price_is_none){.fn}(&self) -\> [bool](https://doc.rust-lang.org/1.86.0/std/primitive.bool.html){.primitive} {#pub-fn-some_price_is_noneself---bool .code-header}
:::

::: docblock
Checks if any of the bid or ask prices for call or put options are None.

This function returns `true` if any of the `call_bid`, `call_ask`,
`put_bid`, or `put_ask` fields of the `OptionData` struct are `None`,
indicating missing price information. It returns `false` if all four
fields have valid price data.
:::

::: {#method.get_position .section .method}
[Source](../../src/optionstratlib/chains/optiondata.rs.html#462-502){.src
.rightside}

#### pub fn [get_position](#method.get_position){.fn}( &self, price_params: &[OptionDataPriceParams](utils/struct.OptionDataPriceParams.html "struct optionstratlib::chains::utils::OptionDataPriceParams"){.struct}, side: [Side](../model/types/enum.Side.html "enum optionstratlib::model::types::Side"){.enum}, option_style: [OptionStyle](../model/types/enum.OptionStyle.html "enum optionstratlib::model::types::OptionStyle"){.enum}, date: [Option](https://doc.rust-lang.org/1.86.0/core/option/enum.Option.html "enum core::option::Option"){.enum}\<[DateTime](https://docs.rs/chrono/latest/chrono/datetime/struct.DateTime.html "struct chrono::datetime::DateTime"){.struct}\<[Utc](https://docs.rs/chrono/latest/chrono/offset/utc/struct.Utc.html "struct chrono::offset::utc::Utc"){.struct}\>\>, open_fee: [Option](https://doc.rust-lang.org/1.86.0/core/option/enum.Option.html "enum core::option::Option"){.enum}\<[Positive](../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\>, close_fee: [Option](https://doc.rust-lang.org/1.86.0/core/option/enum.Option.html "enum core::option::Option"){.enum}\<[Positive](../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\>, ) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Position](../model/position/struct.Position.html "struct optionstratlib::model::position::Position"){.struct}, [ChainError](../error/chains/enum.ChainError.html "enum optionstratlib::error::chains::ChainError"){.enum}\> {#pub-fn-get_position-self-price_params-optiondatapriceparams-side-side-option_style-optionstyle-date-optiondatetimeutc-open_fee-optionpositive-close_fee-optionpositive---resultposition-chainerror .code-header}
:::

::: docblock
Retrieves a `Position` based on the provided parameters, calculating the
option premium using the Black-Scholes model.

This method fetches an option based on the provided parameters,
calculates its theoretical premium using the Black-Scholes model, and
constructs a `Position` struct containing the option details, premium,
opening date, and associated fees.

##### [§](#arguments){.doc-anchor}Arguments

- `price_params` - Option pricing parameters encapsulated in
  `OptionDataPriceParams`.
- `side` - The side of the option, either `Side::Long` or `Side::Short`.
- `option_style` - The style of the option, either `OptionStyle::Call`
  or `OptionStyle::Put`.
- `date` - An optional `DateTime<Utc>` representing the opening date of
  the position. If `None`, the current UTC timestamp is used.
- `open_fee` - An optional `Positive` value representing the opening fee
  for the position. If `None`, defaults to `Positive::ZERO`.
- `close_fee` - An optional `Positive` value representing the closing
  fee for the position. If `None`, defaults to `Positive::ZERO`.

##### [§](#returns-6){.doc-anchor}Returns

- `Result<Position, ChainError>` - A `Result` containing the constructed
  `Position` on success, or a `ChainError` if any error occurred during
  option retrieval or premium calculation.

##### [§](#errors){.doc-anchor}Errors

This method can return a `ChainError` if:

- The underlying option cannot be retrieved based on the provided
  parameters.
- The Black-Scholes model fails to calculate a valid option premium.
:::

::: {#method.calculate_prices .section .method}
[Source](../../src/optionstratlib/chains/optiondata.rs.html#594-642){.src
.rightside}

#### pub fn [calculate_prices](#method.calculate_prices){.fn}( &mut self, price_params: &[OptionDataPriceParams](utils/struct.OptionDataPriceParams.html "struct optionstratlib::chains::utils::OptionDataPriceParams"){.struct}, refresh: [bool](https://doc.rust-lang.org/1.86.0/std/primitive.bool.html){.primitive}, ) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[()](https://doc.rust-lang.org/1.86.0/std/primitive.unit.html){.primitive}, [ChainError](../error/chains/enum.ChainError.html "enum optionstratlib::error::chains::ChainError"){.enum}\> {#pub-fn-calculate_prices-mut-self-price_params-optiondatapriceparams-refresh-bool---result-chainerror .code-header}
:::

::: docblock
Calculates and sets the bid and ask prices for call and put options.

This method computes the theoretical prices for both call and put
options using the Black-Scholes pricing model, and then stores these
values in the appropriate fields. After calculating the individual bid
and ask prices, it also computes and sets the mid-prices by calling the
`set_mid_prices` method.

##### [§](#parameters-1){.doc-anchor}Parameters

- `price_params` - A reference to `OptionDataPriceParams` containing the
  necessary parameters for option pricing, such as underlying price,
  volatility, risk-free rate, expiration date, and dividend yield.

- `refresh` - A boolean flag indicating whether to force recalculation
  of option contracts even if they already exist. When set to `true`,
  the method will recreate the option contracts before calculating
  prices.

##### [§](#returns-7){.doc-anchor}Returns

- `Result<(), ChainError>` - Returns `Ok(())` if prices are successfully
  calculated and set, or a `ChainError` if any error occurs during the
  process.

##### [§](#side-effects){.doc-anchor}Side Effects

Sets the following fields in the struct:

- `call_ask` - The ask price for the call option
- `call_bid` - The bid price for the call option
- `put_ask` - The ask price for the put option
- `put_bid` - The bid price for the put option
- `call_middle` and `put_middle` - The mid-prices calculated via
  `set_mid_prices()`

##### [§](#errors-1){.doc-anchor}Errors

May return:

- `ChainError` variants if there are issues creating the options
  contracts
- Errors propagated from the Black-Scholes calculation functions
:::

::: {#method.apply_spread .section .method}
[Source](../../src/optionstratlib/chains/optiondata.rs.html#666-708){.src
.rightside}

#### pub fn [apply_spread](#method.apply_spread){.fn}(&mut self, spread: [Positive](../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, decimal_places: [i32](https://doc.rust-lang.org/1.86.0/std/primitive.i32.html){.primitive}) {#pub-fn-apply_spreadmut-self-spread-positive-decimal_places-i32 .code-header}
:::

::: docblock
Applies a spread to the bid and ask prices of call and put options, then
recalculates mid prices.

This method adjusts the bid and ask prices by half of the specified
spread value, subtracting from bid prices and adding to ask prices. It
also ensures that all prices are rounded to the specified number of
decimal places. If any price becomes negative after applying the spread,
it is set to `None`.

##### [§](#arguments-1){.doc-anchor}Arguments

- `spread` - A positive decimal value representing the total spread to
  apply
- `decimal_places` - The number of decimal places to round the adjusted
  prices to

##### [§](#inner-function){.doc-anchor}Inner Function

The method contains an inner function `round_to_decimal` that handles
the rounding of prices after applying a shift (half the spread).

##### [§](#side-effects-1){.doc-anchor}Side Effects

- Updates `call_ask`, `call_bid`, `put_ask`, and `put_bid` fields with
  adjusted values
- Sets adjusted prices to `None` if they would become negative after
  applying the spread
- Calls `set_mid_prices()` to recalculate the mid prices based on the
  new bid/ask values
:::

::: {#method.calculate_delta .section .method}
[Source](../../src/optionstratlib/chains/optiondata.rs.html#737-782){.src
.rightside}

#### pub fn [calculate_delta](#method.calculate_delta){.fn}(&mut self, price_params: &[OptionDataPriceParams](utils/struct.OptionDataPriceParams.html "struct optionstratlib::chains::utils::OptionDataPriceParams"){.struct}) {#pub-fn-calculate_deltamut-self-price_params-optiondatapriceparams .code-header}
:::

::: docblock
Calculates the delta values for call and put options based on the
provided price parameters.

Delta is a key "Greek" that measures the rate of change of the option's
price with respect to changes in the underlying asset's price. This
method computes and stores delta values for both call and put options.

##### [§](#parameters-2){.doc-anchor}Parameters

- `price_params` - A reference to `OptionDataPriceParams` containing
  essential market data and contract specifications needed for the
  calculation.

##### [§](#behavior){.doc-anchor}Behavior

The function follows these steps:

1.  Ensures implied volatility is available, calculating it if necessary
2.  Creates option objects if they don't exist but implied volatility is
    available
3.  Calculates and stores delta values for call options
4.  Calculates and stores delta values for put options

If any step fails, appropriate error messages are logged and the
corresponding delta values will remain unset.

##### [§](#side-effects-2){.doc-anchor}Side Effects

- Updates the `delta_call` and `delta_put` fields of the struct with
  calculated values
- May update the `implied_volatility` field if it was previously `None`
- May create option objects if they didn't exist but were needed for
  calculations
- Logs errors if calculations fail
:::

::: {#method.calculate_gamma .section .method}
[Source](../../src/optionstratlib/chains/optiondata.rs.html#809-835){.src
.rightside}

#### pub fn [calculate_gamma](#method.calculate_gamma){.fn}(&mut self, price_params: &[OptionDataPriceParams](utils/struct.OptionDataPriceParams.html "struct optionstratlib::chains::utils::OptionDataPriceParams"){.struct}) {#pub-fn-calculate_gammamut-self-price_params-optiondatapriceparams .code-header}
:::

::: docblock
Calculates the gamma value for an option and stores it in the object.

Gamma measures the rate of change in delta with respect to changes in
the underlying price. It represents the second derivative of the option
price with respect to the underlying price.

This method first ensures that implied volatility is available
(calculating it if needed), then creates option structures if they don't
already exist, and finally calculates the gamma value.

##### [§](#parameters-3){.doc-anchor}Parameters

- `price_params` - A reference to the pricing parameters required for
  option calculations, including underlying price, expiration date,
  risk-free rate and other inputs.

##### [§](#behavior-1){.doc-anchor}Behavior

- If implied volatility isn't available, it attempts to calculate it
  first
- If option structures haven't been created yet, it creates them
- On successful calculation, stores the gamma value in `self.gamma`
- On failure, logs an error and sets `self.gamma` to `None`

##### [§](#errors-2){.doc-anchor}Errors

- Does not return errors but logs them through the tracing system
- Common failures include inability to calculate implied volatility or
  issues creating option objects
:::

::: {#method.get_deltas .section .method}
[Source](../../src/optionstratlib/chains/optiondata.rs.html#857-864){.src
.rightside}

#### pub fn [get_deltas](#method.get_deltas){.fn}( &self, price_params: &[OptionDataPriceParams](utils/struct.OptionDataPriceParams.html "struct optionstratlib::chains::utils::OptionDataPriceParams"){.struct}, ) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[DeltasInStrike](struct.DeltasInStrike.html "struct optionstratlib::chains::DeltasInStrike"){.struct}, [ChainError](../error/chains/enum.ChainError.html "enum optionstratlib::error::chains::ChainError"){.enum}\> {#pub-fn-get_deltas-self-price_params-optiondatapriceparams---resultdeltasinstrike-chainerror .code-header}
:::

::: docblock
Retrieves delta values for options at the current strike price.

Delta measures the rate of change of the option price with respect to
changes in the underlying asset's price. This method returns delta
values for options at the specific strike price defined in the price
parameters.

##### [§](#parameters-4){.doc-anchor}Parameters

- `price_params` - A reference to the pricing parameters required for
  option calculations, including underlying price, expiration date,
  risk-free rate and other inputs.

##### [§](#returns-8){.doc-anchor}Returns

- `Result<DeltasInStrike, ChainError>` - On success, returns a structure
  containing delta values for the options at the specified strike. On
  failure, returns a ChainError describing the issue.

##### [§](#errors-3){.doc-anchor}Errors

- Returns a `ChainError` if there's an issue retrieving the options or
  calculating their deltas.
- Possible errors include missing option data, calculation failures, or
  invalid parameters.
:::

::: {#method.is_valid_optimal_side .section .method}
[Source](../../src/optionstratlib/chains/optiondata.rs.html#884-909){.src
.rightside}

#### pub fn [is_valid_optimal_side](#method.is_valid_optimal_side){.fn}( &self, underlying_price: [Positive](../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, side: &[FindOptimalSide](../strategies/utils/enum.FindOptimalSide.html "enum optionstratlib::strategies::utils::FindOptimalSide"){.enum}, ) -\> [bool](https://doc.rust-lang.org/1.86.0/std/primitive.bool.html){.primitive} {#pub-fn-is_valid_optimal_side-self-underlying_price-positive-side-findoptimalside---bool .code-header}
:::

::: docblock
Validates if an option strike price is valid according to the specified
search strategy.

This method checks whether the current option's strike price falls
within the constraints defined by the `FindOptimalSide` parameter,
relative to the given underlying asset price.

##### [§](#parameters-5){.doc-anchor}Parameters

- `underlying_price` - The current market price of the underlying asset
  as a `Positive` value.
- `side` - The strategy to determine valid strike prices, specifying
  whether to consider options with strikes above, below, or within a
  specific range of the underlying price.

##### [§](#returns-9){.doc-anchor}Returns

`bool` - Returns true if the strike price is valid according to the
specified strategy:

- For `Upper`: Strike price must be greater than or equal to underlying
  price
- For `Lower`: Strike price must be less than or equal to underlying
  price
- For `All`: Always returns true (all strike prices are valid)
- For `Range`: Strike price must fall within the specified range
  (inclusive)
:::

::: {#method.set_mid_prices .section .method}
[Source](../../src/optionstratlib/chains/optiondata.rs.html#922-931){.src
.rightside}

#### pub fn [set_mid_prices](#method.set_mid_prices){.fn}(&mut self) {#pub-fn-set_mid_pricesmut-self .code-header}
:::

::: docblock
Calculates and sets the mid-prices for both call and put options.

This method computes the middle price between the bid and ask prices for
both call and put options, when both bid and ask prices are available.
The mid-price is calculated as the simple average: (bid + ask) / 2. If
either bid or ask price is missing for an option type, the corresponding
mid-price will be set to `None`.

##### [§](#side-effects-3){.doc-anchor}Side Effects

Updates the `call_middle` and `put_middle` fields with the calculated
mid-prices.
:::

::: {#method.get_mid_prices .section .method}
[Source](../../src/optionstratlib/chains/optiondata.rs.html#944-946){.src
.rightside}

#### pub fn [get_mid_prices](#method.get_mid_prices){.fn}(&self) -\> ([Option](https://doc.rust-lang.org/1.86.0/core/option/enum.Option.html "enum core::option::Option"){.enum}\<[Positive](../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\>, [Option](https://doc.rust-lang.org/1.86.0/core/option/enum.Option.html "enum core::option::Option"){.enum}\<[Positive](../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\>) {#pub-fn-get_mid_pricesself---optionpositive-optionpositive .code-header}
:::

::: docblock
Retrieves the current mid-prices for call and put options.

This method returns the calculated middle prices for both call and put
options as a tuple. Each price may be `None` if the corresponding
bid/ask prices were not available when `set_mid_prices()` was called.

##### [§](#returns-10){.doc-anchor}Returns

A tuple containing:

- First element: The call option mid-price (bid+ask)/2, or `None` if not
  available
- Second element: The put option mid-price (bid+ask)/2, or `None` if not
  available
:::

::: {#method.calculate_implied_volatility .section .method}
[Source](../../src/optionstratlib/chains/optiondata.rs.html#977-1055){.src
.rightside}

#### pub fn [calculate_implied_volatility](#method.calculate_implied_volatility){.fn}( &mut self, price_params: &[OptionDataPriceParams](utils/struct.OptionDataPriceParams.html "struct optionstratlib::chains::utils::OptionDataPriceParams"){.struct}, ) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[()](https://doc.rust-lang.org/1.86.0/std/primitive.unit.html){.primitive}, [ChainError](../error/chains/enum.ChainError.html "enum optionstratlib::error::chains::ChainError"){.enum}\> {#pub-fn-calculate_implied_volatility-mut-self-price_params-optiondatapriceparams---result-chainerror .code-header}
:::

::: docblock
Calculates the implied volatility for an option based on market prices.

This function attempts to derive the implied volatility from either call
or put option mid-market prices. It first tries to use call options, and
if that fails, it falls back to put options. The calculation uses
different initial volatility guesses based on whether the option is
in-the-money (ITM) or out-of-the-money (OTM).

##### [§](#parameters-6){.doc-anchor}Parameters

- `&mut self` - Mutable reference to the option chain or strike object
- `price_params` - Reference to pricing parameters including underlying
  price and other market data

##### [§](#returns-11){.doc-anchor}Returns

- `Result<(), ChainError>` - Ok if implied volatility was successfully
  calculated, or an error describing why calculation failed

##### [§](#process){.doc-anchor}Process

1.  Ensures middle prices are available, calculating them if necessary
2.  Attempts to calculate IV using call options first
3.  Falls back to put options if call calculation fails
4.  Updates the implied_volatility field if successful
5.  Creates option objects if needed once IV is established

##### [§](#errors-4){.doc-anchor}Errors

Returns a `ChainError::InvalidVolatility` if implied volatility cannot
be calculated from either call or put prices.
:::

::: {#method.create_options .section .method}
[Source](../../src/optionstratlib/chains/optiondata.rs.html#1089-1161){.src
.rightside}

#### pub fn [create_options](#method.create_options){.fn}( &mut self, price_params: &[OptionDataPriceParams](utils/struct.OptionDataPriceParams.html "struct optionstratlib::chains::utils::OptionDataPriceParams"){.struct}, ) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[()](https://doc.rust-lang.org/1.86.0/std/primitive.unit.html){.primitive}, [ChainError](../error/chains/enum.ChainError.html "enum optionstratlib::error::chains::ChainError"){.enum}\> {#pub-fn-create_options-mut-self-price_params-optiondatapriceparams---result-chainerror .code-header}
:::

::: docblock
Creates a complete set of four standard option contracts based on
specified pricing parameters.

This method constructs four option contracts (long call, short call,
long put, short put) with identical strike prices and expiration dates,
all based on the same underlying asset. The resulting options are stored
within the `OptionData` instance for further analysis or trading
strategy evaluation.

##### [§](#parameters-7){.doc-anchor}Parameters

- `price_params` - A reference to `OptionDataPriceParams` containing
  essential pricing inputs including underlying price, expiration date,
  risk-free rate, dividend yield, and optionally the underlying symbol
  and implied volatility.

##### [§](#returns-12){.doc-anchor}Returns

- `Result<(), ChainError>` - Returns `Ok(())` if option creation
  succeeds, or a `ChainError` if any issues occur during creation.
:::

::: {#method.current_deltas .section .method}
[Source](../../src/optionstratlib/chains/optiondata.rs.html#1177-1179){.src
.rightside}

#### pub fn [current_deltas](#method.current_deltas){.fn}(&self) -\> ([Option](https://doc.rust-lang.org/1.86.0/core/option/enum.Option.html "enum core::option::Option"){.enum}\<Decimal\>, [Option](https://doc.rust-lang.org/1.86.0/core/option/enum.Option.html "enum core::option::Option"){.enum}\<Decimal\>) {#pub-fn-current_deltasself---optiondecimal-optiondecimal .code-header}
:::

::: docblock
Returns a tuple containing the current delta values for both call and
put options.

This method provides access to the option's delta values, which measure
the rate of change of the option price with respect to changes in the
underlying asset price. Delta values typically range from -1 to 1 and
are a key metric for understanding option price sensitivity.

##### [§](#returns-13){.doc-anchor}Returns

A tuple containing:

- First element: `Option<Decimal>` - The delta value for the call
  option. May be `None` if the delta value is not available or could not
  be calculated.
- Second element: `Option<Decimal>` - The delta value for the put
  option. May be `None` if the delta value is not available or could not
  be calculated.
:::
:::::::::::::::::::::::::::::::::::::::::
:::::::::::::::::::::::::::::::::::::::::::

## Trait Implementations[§](#trait-implementations){.anchor} {#trait-implementations .section-header}

:::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::: {#trait-implementations-list}
::: {#impl-Clone-for-OptionData .section .impl}
[Source](../../src/optionstratlib/chains/optiondata.rs.html#68){.src
.rightside}[§](#impl-Clone-for-OptionData){.anchor}

### impl [Clone](https://doc.rust-lang.org/1.86.0/core/clone/trait.Clone.html "trait core::clone::Clone"){.trait} for [OptionData](struct.OptionData.html "struct optionstratlib::chains::OptionData"){.struct} {#impl-clone-for-optiondata .code-header}
:::

::::::: impl-items
::: {#method.clone .section .method .trait-impl}
[Source](../../src/optionstratlib/chains/optiondata.rs.html#68){.src
.rightside}[§](#method.clone){.anchor}

#### fn [clone](https://doc.rust-lang.org/1.86.0/core/clone/trait.Clone.html#tymethod.clone){.fn}(&self) -\> [OptionData](struct.OptionData.html "struct optionstratlib::chains::OptionData"){.struct} {#fn-cloneself---optiondata .code-header}
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

::: {#impl-Debug-for-OptionData .section .impl}
[Source](../../src/optionstratlib/chains/optiondata.rs.html#68){.src
.rightside}[§](#impl-Debug-for-OptionData){.anchor}

### impl [Debug](https://doc.rust-lang.org/1.86.0/core/fmt/trait.Debug.html "trait core::fmt::Debug"){.trait} for [OptionData](struct.OptionData.html "struct optionstratlib::chains::OptionData"){.struct} {#impl-debug-for-optiondata .code-header}
:::

::::: impl-items
::: {#method.fmt .section .method .trait-impl}
[Source](../../src/optionstratlib/chains/optiondata.rs.html#68){.src
.rightside}[§](#method.fmt){.anchor}

#### fn [fmt](https://doc.rust-lang.org/1.86.0/core/fmt/trait.Debug.html#tymethod.fmt){.fn}(&self, f: &mut [Formatter](https://doc.rust-lang.org/1.86.0/core/fmt/struct.Formatter.html "struct core::fmt::Formatter"){.struct}\<\'\_\>) -\> [Result](https://doc.rust-lang.org/1.86.0/core/fmt/type.Result.html "type core::fmt::Result"){.type} {#fn-fmtself-f-mut-formatter_---result .code-header}
:::

::: docblock
Formats the value using the given formatter. [Read
more](https://doc.rust-lang.org/1.86.0/core/fmt/trait.Debug.html#tymethod.fmt)
:::
:::::

::: {#impl-Default-for-OptionData .section .impl}
[Source](../../src/optionstratlib/chains/optiondata.rs.html#1182-1201){.src
.rightside}[§](#impl-Default-for-OptionData){.anchor}

### impl [Default](https://doc.rust-lang.org/1.86.0/core/default/trait.Default.html "trait core::default::Default"){.trait} for [OptionData](struct.OptionData.html "struct optionstratlib::chains::OptionData"){.struct} {#impl-default-for-optiondata .code-header}
:::

::::: impl-items
::: {#method.default .section .method .trait-impl}
[Source](../../src/optionstratlib/chains/optiondata.rs.html#1183-1200){.src
.rightside}[§](#method.default){.anchor}

#### fn [default](https://doc.rust-lang.org/1.86.0/core/default/trait.Default.html#tymethod.default){.fn}() -\> Self {#fn-default---self .code-header}
:::

::: docblock
Returns the "default value" for a type. [Read
more](https://doc.rust-lang.org/1.86.0/core/default/trait.Default.html#tymethod.default)
:::
:::::

::: {#impl-Deserialize%3C'de%3E-for-OptionData .section .impl}
[Source](../../src/optionstratlib/chains/optiondata.rs.html#68){.src
.rightside}[§](#impl-Deserialize%3C'de%3E-for-OptionData){.anchor}

### impl\<\'de\> [Deserialize](https://docs.rs/serde/1.0.219/serde/de/trait.Deserialize.html "trait serde::de::Deserialize"){.trait}\<\'de\> for [OptionData](struct.OptionData.html "struct optionstratlib::chains::OptionData"){.struct} {#implde-deserializede-for-optiondata .code-header}
:::

:::::: impl-items
:::: {#method.deserialize .section .method .trait-impl}
[Source](../../src/optionstratlib/chains/optiondata.rs.html#68){.src
.rightside}[§](#method.deserialize){.anchor}

#### fn [deserialize](https://docs.rs/serde/1.0.219/serde/de/trait.Deserialize.html#tymethod.deserialize){.fn}\<\_\_D\>(\_\_deserializer: \_\_D) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<Self, \_\_D::[Error](https://docs.rs/serde/1.0.219/serde/de/trait.Deserializer.html#associatedtype.Error "type serde::de::Deserializer::Error"){.associatedtype}\> {#fn-deserialize__d__deserializer-__d---resultself-__derror .code-header}

::: where
where \_\_D:
[Deserializer](https://docs.rs/serde/1.0.219/serde/de/trait.Deserializer.html "trait serde::de::Deserializer"){.trait}\<\'de\>,
:::
::::

::: docblock
Deserialize this value from the given Serde deserializer. [Read
more](https://docs.rs/serde/1.0.219/serde/de/trait.Deserialize.html#tymethod.deserialize)
:::
::::::

::: {#impl-Display-for-OptionData .section .impl}
[Source](../../src/optionstratlib/chains/optiondata.rs.html#1217-1244){.src
.rightside}[§](#impl-Display-for-OptionData){.anchor}

### impl [Display](https://doc.rust-lang.org/1.86.0/core/fmt/trait.Display.html "trait core::fmt::Display"){.trait} for [OptionData](struct.OptionData.html "struct optionstratlib::chains::OptionData"){.struct} {#impl-display-for-optiondata .code-header}
:::

::::: impl-items
::: {#method.fmt-1 .section .method .trait-impl}
[Source](../../src/optionstratlib/chains/optiondata.rs.html#1218-1243){.src
.rightside}[§](#method.fmt-1){.anchor}

#### fn [fmt](https://doc.rust-lang.org/1.86.0/core/fmt/trait.Display.html#tymethod.fmt){.fn}(&self, f: &mut [Formatter](https://doc.rust-lang.org/1.86.0/core/fmt/struct.Formatter.html "struct core::fmt::Formatter"){.struct}\<\'\_\>) -\> [Result](https://doc.rust-lang.org/1.86.0/core/fmt/type.Result.html "type core::fmt::Result"){.type} {#fn-fmtself-f-mut-formatter_---result-1 .code-header}
:::

::: docblock
Formats the value using the given formatter. [Read
more](https://doc.rust-lang.org/1.86.0/core/fmt/trait.Display.html#tymethod.fmt)
:::
:::::

::: {#impl-Ord-for-OptionData .section .impl}
[Source](../../src/optionstratlib/chains/optiondata.rs.html#1211-1215){.src
.rightside}[§](#impl-Ord-for-OptionData){.anchor}

### impl [Ord](https://doc.rust-lang.org/1.86.0/core/cmp/trait.Ord.html "trait core::cmp::Ord"){.trait} for [OptionData](struct.OptionData.html "struct optionstratlib::chains::OptionData"){.struct} {#impl-ord-for-optiondata .code-header}
:::

:::::::::::::: impl-items
::: {#method.cmp .section .method .trait-impl}
[Source](../../src/optionstratlib/chains/optiondata.rs.html#1212-1214){.src
.rightside}[§](#method.cmp){.anchor}

#### fn [cmp](https://doc.rust-lang.org/1.86.0/core/cmp/trait.Ord.html#tymethod.cmp){.fn}(&self, other: &Self) -\> [Ordering](https://doc.rust-lang.org/1.86.0/core/cmp/enum.Ordering.html "enum core::cmp::Ordering"){.enum} {#fn-cmpself-other-self---ordering .code-header}
:::

::: docblock
This method returns an
[`Ordering`](https://doc.rust-lang.org/1.86.0/core/cmp/enum.Ordering.html "enum core::cmp::Ordering")
between `self` and `other`. [Read
more](https://doc.rust-lang.org/1.86.0/core/cmp/trait.Ord.html#tymethod.cmp)
:::

:::: {#method.max .section .method .trait-impl}
[[1.21.0]{.since title="Stable since Rust version 1.21.0"} ·
[Source](https://doc.rust-lang.org/1.86.0/src/core/cmp.rs.html#998-1000){.src}]{.rightside}[§](#method.max){.anchor}

#### fn [max](https://doc.rust-lang.org/1.86.0/core/cmp/trait.Ord.html#method.max){.fn}(self, other: Self) -\> Self {#fn-maxself-other-self---self .code-header}

::: where
where Self:
[Sized](https://doc.rust-lang.org/1.86.0/core/marker/trait.Sized.html "trait core::marker::Sized"){.trait},
:::
::::

::: docblock
Compares and returns the maximum of two values. [Read
more](https://doc.rust-lang.org/1.86.0/core/cmp/trait.Ord.html#method.max)
:::

:::: {#method.min .section .method .trait-impl}
[[1.21.0]{.since title="Stable since Rust version 1.21.0"} ·
[Source](https://doc.rust-lang.org/1.86.0/src/core/cmp.rs.html#1037-1039){.src}]{.rightside}[§](#method.min){.anchor}

#### fn [min](https://doc.rust-lang.org/1.86.0/core/cmp/trait.Ord.html#method.min){.fn}(self, other: Self) -\> Self {#fn-minself-other-self---self .code-header}

::: where
where Self:
[Sized](https://doc.rust-lang.org/1.86.0/core/marker/trait.Sized.html "trait core::marker::Sized"){.trait},
:::
::::

::: docblock
Compares and returns the minimum of two values. [Read
more](https://doc.rust-lang.org/1.86.0/core/cmp/trait.Ord.html#method.min)
:::

:::: {#method.clamp .section .method .trait-impl}
[[1.50.0]{.since title="Stable since Rust version 1.50.0"} ·
[Source](https://doc.rust-lang.org/1.86.0/src/core/cmp.rs.html#1063-1065){.src}]{.rightside}[§](#method.clamp){.anchor}

#### fn [clamp](https://doc.rust-lang.org/1.86.0/core/cmp/trait.Ord.html#method.clamp){.fn}(self, min: Self, max: Self) -\> Self {#fn-clampself-min-self-max-self---self .code-header}

::: where
where Self:
[Sized](https://doc.rust-lang.org/1.86.0/core/marker/trait.Sized.html "trait core::marker::Sized"){.trait},
:::
::::

::: docblock
Restrict a value to a certain interval. [Read
more](https://doc.rust-lang.org/1.86.0/core/cmp/trait.Ord.html#method.clamp)
:::
::::::::::::::

::: {#impl-PartialEq-for-OptionData .section .impl}
[Source](../../src/optionstratlib/chains/optiondata.rs.html#68){.src
.rightside}[§](#impl-PartialEq-for-OptionData){.anchor}

### impl [PartialEq](https://doc.rust-lang.org/1.86.0/core/cmp/trait.PartialEq.html "trait core::cmp::PartialEq"){.trait} for [OptionData](struct.OptionData.html "struct optionstratlib::chains::OptionData"){.struct} {#impl-partialeq-for-optiondata .code-header}
:::

::::::: impl-items
::: {#method.eq .section .method .trait-impl}
[Source](../../src/optionstratlib/chains/optiondata.rs.html#68){.src
.rightside}[§](#method.eq){.anchor}

#### fn [eq](https://doc.rust-lang.org/1.86.0/core/cmp/trait.PartialEq.html#tymethod.eq){.fn}(&self, other: &[OptionData](struct.OptionData.html "struct optionstratlib::chains::OptionData"){.struct}) -\> [bool](https://doc.rust-lang.org/1.86.0/std/primitive.bool.html){.primitive} {#fn-eqself-other-optiondata---bool .code-header}
:::

::: docblock
Tests for `self` and `other` values to be equal, and is used by `==`.
:::

::: {#method.ne .section .method .trait-impl}
[[1.0.0]{.since title="Stable since Rust version 1.0.0"} ·
[Source](https://doc.rust-lang.org/1.86.0/src/core/cmp.rs.html#261){.src}]{.rightside}[§](#method.ne){.anchor}

#### fn [ne](https://doc.rust-lang.org/1.86.0/core/cmp/trait.PartialEq.html#method.ne){.fn}(&self, other: [&Rhs](https://doc.rust-lang.org/1.86.0/std/primitive.reference.html){.primitive}) -\> [bool](https://doc.rust-lang.org/1.86.0/std/primitive.bool.html){.primitive} {#fn-neself-other-rhs---bool .code-header}
:::

::: docblock
Tests for `!=`. The default implementation is almost always sufficient,
and should not be overridden without very good reason.
:::
:::::::

::: {#impl-PartialOrd-for-OptionData .section .impl}
[Source](../../src/optionstratlib/chains/optiondata.rs.html#1203-1207){.src
.rightside}[§](#impl-PartialOrd-for-OptionData){.anchor}

### impl [PartialOrd](https://doc.rust-lang.org/1.86.0/core/cmp/trait.PartialOrd.html "trait core::cmp::PartialOrd"){.trait} for [OptionData](struct.OptionData.html "struct optionstratlib::chains::OptionData"){.struct} {#impl-partialord-for-optiondata .code-header}
:::

::::::::::::: impl-items
::: {#method.partial_cmp .section .method .trait-impl}
[Source](../../src/optionstratlib/chains/optiondata.rs.html#1204-1206){.src
.rightside}[§](#method.partial_cmp){.anchor}

#### fn [partial_cmp](https://doc.rust-lang.org/1.86.0/core/cmp/trait.PartialOrd.html#tymethod.partial_cmp){.fn}(&self, other: &Self) -\> [Option](https://doc.rust-lang.org/1.86.0/core/option/enum.Option.html "enum core::option::Option"){.enum}\<[Ordering](https://doc.rust-lang.org/1.86.0/core/cmp/enum.Ordering.html "enum core::cmp::Ordering"){.enum}\> {#fn-partial_cmpself-other-self---optionordering .code-header}
:::

::: docblock
This method returns an ordering between `self` and `other` values if one
exists. [Read
more](https://doc.rust-lang.org/1.86.0/core/cmp/trait.PartialOrd.html#tymethod.partial_cmp)
:::

::: {#method.lt .section .method .trait-impl}
[[1.0.0]{.since title="Stable since Rust version 1.0.0"} ·
[Source](https://doc.rust-lang.org/1.86.0/src/core/cmp.rs.html#1371){.src}]{.rightside}[§](#method.lt){.anchor}

#### fn [lt](https://doc.rust-lang.org/1.86.0/core/cmp/trait.PartialOrd.html#method.lt){.fn}(&self, other: [&Rhs](https://doc.rust-lang.org/1.86.0/std/primitive.reference.html){.primitive}) -\> [bool](https://doc.rust-lang.org/1.86.0/std/primitive.bool.html){.primitive} {#fn-ltself-other-rhs---bool .code-header}
:::

::: docblock
Tests less than (for `self` and `other`) and is used by the `<`
operator. [Read
more](https://doc.rust-lang.org/1.86.0/core/cmp/trait.PartialOrd.html#method.lt)
:::

::: {#method.le .section .method .trait-impl}
[[1.0.0]{.since title="Stable since Rust version 1.0.0"} ·
[Source](https://doc.rust-lang.org/1.86.0/src/core/cmp.rs.html#1389){.src}]{.rightside}[§](#method.le){.anchor}

#### fn [le](https://doc.rust-lang.org/1.86.0/core/cmp/trait.PartialOrd.html#method.le){.fn}(&self, other: [&Rhs](https://doc.rust-lang.org/1.86.0/std/primitive.reference.html){.primitive}) -\> [bool](https://doc.rust-lang.org/1.86.0/std/primitive.bool.html){.primitive} {#fn-leself-other-rhs---bool .code-header}
:::

::: docblock
Tests less than or equal to (for `self` and `other`) and is used by the
`<=` operator. [Read
more](https://doc.rust-lang.org/1.86.0/core/cmp/trait.PartialOrd.html#method.le)
:::

::: {#method.gt .section .method .trait-impl}
[[1.0.0]{.since title="Stable since Rust version 1.0.0"} ·
[Source](https://doc.rust-lang.org/1.86.0/src/core/cmp.rs.html#1407){.src}]{.rightside}[§](#method.gt){.anchor}

#### fn [gt](https://doc.rust-lang.org/1.86.0/core/cmp/trait.PartialOrd.html#method.gt){.fn}(&self, other: [&Rhs](https://doc.rust-lang.org/1.86.0/std/primitive.reference.html){.primitive}) -\> [bool](https://doc.rust-lang.org/1.86.0/std/primitive.bool.html){.primitive} {#fn-gtself-other-rhs---bool .code-header}
:::

::: docblock
Tests greater than (for `self` and `other`) and is used by the `>`
operator. [Read
more](https://doc.rust-lang.org/1.86.0/core/cmp/trait.PartialOrd.html#method.gt)
:::

::: {#method.ge .section .method .trait-impl}
[[1.0.0]{.since title="Stable since Rust version 1.0.0"} ·
[Source](https://doc.rust-lang.org/1.86.0/src/core/cmp.rs.html#1425){.src}]{.rightside}[§](#method.ge){.anchor}

#### fn [ge](https://doc.rust-lang.org/1.86.0/core/cmp/trait.PartialOrd.html#method.ge){.fn}(&self, other: [&Rhs](https://doc.rust-lang.org/1.86.0/std/primitive.reference.html){.primitive}) -\> [bool](https://doc.rust-lang.org/1.86.0/std/primitive.bool.html){.primitive} {#fn-geself-other-rhs---bool .code-header}
:::

::: docblock
Tests greater than or equal to (for `self` and `other`) and is used by
the `>=` operator. [Read
more](https://doc.rust-lang.org/1.86.0/core/cmp/trait.PartialOrd.html#method.ge)
:::
:::::::::::::

::: {#impl-Serialize-for-OptionData .section .impl}
[Source](../../src/optionstratlib/chains/optiondata.rs.html#68){.src
.rightside}[§](#impl-Serialize-for-OptionData){.anchor}

### impl [Serialize](https://docs.rs/serde/1.0.219/serde/ser/trait.Serialize.html "trait serde::ser::Serialize"){.trait} for [OptionData](struct.OptionData.html "struct optionstratlib::chains::OptionData"){.struct} {#impl-serialize-for-optiondata .code-header}
:::

:::::: impl-items
:::: {#method.serialize .section .method .trait-impl}
[Source](../../src/optionstratlib/chains/optiondata.rs.html#68){.src
.rightside}[§](#method.serialize){.anchor}

#### fn [serialize](https://docs.rs/serde/1.0.219/serde/ser/trait.Serialize.html#tymethod.serialize){.fn}\<\_\_S\>(&self, \_\_serializer: \_\_S) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<\_\_S::[Ok](https://docs.rs/serde/1.0.219/serde/ser/trait.Serializer.html#associatedtype.Ok "type serde::ser::Serializer::Ok"){.associatedtype}, \_\_S::[Error](https://docs.rs/serde/1.0.219/serde/ser/trait.Serializer.html#associatedtype.Error "type serde::ser::Serializer::Error"){.associatedtype}\> {#fn-serialize__sself-__serializer-__s---result__sok-__serror .code-header}

::: where
where \_\_S:
[Serializer](https://docs.rs/serde/1.0.219/serde/ser/trait.Serializer.html "trait serde::ser::Serializer"){.trait},
:::
::::

::: docblock
Serialize this value into the given Serde serializer. [Read
more](https://docs.rs/serde/1.0.219/serde/ser/trait.Serialize.html#tymethod.serialize)
:::
::::::

::: {#impl-Eq-for-OptionData .section .impl}
[Source](../../src/optionstratlib/chains/optiondata.rs.html#1209){.src
.rightside}[§](#impl-Eq-for-OptionData){.anchor}

### impl [Eq](https://doc.rust-lang.org/1.86.0/core/cmp/trait.Eq.html "trait core::cmp::Eq"){.trait} for [OptionData](struct.OptionData.html "struct optionstratlib::chains::OptionData"){.struct} {#impl-eq-for-optiondata .code-header}
:::

::: {#impl-StructuralPartialEq-for-OptionData .section .impl}
[Source](../../src/optionstratlib/chains/optiondata.rs.html#68){.src
.rightside}[§](#impl-StructuralPartialEq-for-OptionData){.anchor}

### impl [StructuralPartialEq](https://doc.rust-lang.org/1.86.0/core/marker/trait.StructuralPartialEq.html "trait core::marker::StructuralPartialEq"){.trait} for [OptionData](struct.OptionData.html "struct optionstratlib::chains::OptionData"){.struct} {#impl-structuralpartialeq-for-optiondata .code-header}
:::
::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::

## Auto Trait Implementations[§](#synthetic-implementations){.anchor} {#synthetic-implementations .section-header}

::::::::: {#synthetic-implementations-list}
::: {#impl-Freeze-for-OptionData .section .impl}
[§](#impl-Freeze-for-OptionData){.anchor}

### impl [Freeze](https://doc.rust-lang.org/1.86.0/core/marker/trait.Freeze.html "trait core::marker::Freeze"){.trait} for [OptionData](struct.OptionData.html "struct optionstratlib::chains::OptionData"){.struct} {#impl-freeze-for-optiondata .code-header}
:::

::: {#impl-RefUnwindSafe-for-OptionData .section .impl}
[§](#impl-RefUnwindSafe-for-OptionData){.anchor}

### impl [RefUnwindSafe](https://doc.rust-lang.org/1.86.0/core/panic/unwind_safe/trait.RefUnwindSafe.html "trait core::panic::unwind_safe::RefUnwindSafe"){.trait} for [OptionData](struct.OptionData.html "struct optionstratlib::chains::OptionData"){.struct} {#impl-refunwindsafe-for-optiondata .code-header}
:::

::: {#impl-Send-for-OptionData .section .impl}
[§](#impl-Send-for-OptionData){.anchor}

### impl [Send](https://doc.rust-lang.org/1.86.0/core/marker/trait.Send.html "trait core::marker::Send"){.trait} for [OptionData](struct.OptionData.html "struct optionstratlib::chains::OptionData"){.struct} {#impl-send-for-optiondata .code-header}
:::

::: {#impl-Sync-for-OptionData .section .impl}
[§](#impl-Sync-for-OptionData){.anchor}

### impl [Sync](https://doc.rust-lang.org/1.86.0/core/marker/trait.Sync.html "trait core::marker::Sync"){.trait} for [OptionData](struct.OptionData.html "struct optionstratlib::chains::OptionData"){.struct} {#impl-sync-for-optiondata .code-header}
:::

::: {#impl-Unpin-for-OptionData .section .impl}
[§](#impl-Unpin-for-OptionData){.anchor}

### impl [Unpin](https://doc.rust-lang.org/1.86.0/core/marker/trait.Unpin.html "trait core::marker::Unpin"){.trait} for [OptionData](struct.OptionData.html "struct optionstratlib::chains::OptionData"){.struct} {#impl-unpin-for-optiondata .code-header}
:::

::: {#impl-UnwindSafe-for-OptionData .section .impl}
[§](#impl-UnwindSafe-for-OptionData){.anchor}

### impl [UnwindSafe](https://doc.rust-lang.org/1.86.0/core/panic/unwind_safe/trait.UnwindSafe.html "trait core::panic::unwind_safe::UnwindSafe"){.trait} for [OptionData](struct.OptionData.html "struct optionstratlib::chains::OptionData"){.struct} {#impl-unwindsafe-for-optiondata .code-header}
:::
:::::::::

## Blanket Implementations[§](#blanket-implementations){.anchor} {#blanket-implementations .section-header}

::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::: {#blanket-implementations-list}
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

:::: {#impl-Comparable%3CK%3E-for-Q .section .impl}
[§](#impl-Comparable%3CK%3E-for-Q){.anchor}

### impl\<Q, K\> Comparable\<K\> for Q {#implq-k-comparablek-for-q .code-header}

::: where
where Q:
[Ord](https://doc.rust-lang.org/1.86.0/core/cmp/trait.Ord.html "trait core::cmp::Ord"){.trait} +
?[Sized](https://doc.rust-lang.org/1.86.0/core/marker/trait.Sized.html "trait core::marker::Sized"){.trait},
K:
[Borrow](https://doc.rust-lang.org/1.86.0/core/borrow/trait.Borrow.html "trait core::borrow::Borrow"){.trait}\<Q\> +
?[Sized](https://doc.rust-lang.org/1.86.0/core/marker/trait.Sized.html "trait core::marker::Sized"){.trait},
:::
::::

::::: impl-items
::: {#method.compare .section .method .trait-impl}
[§](#method.compare){.anchor}

#### fn [compare]{.fn}(&self, key: [&K](https://doc.rust-lang.org/1.86.0/std/primitive.reference.html){.primitive}) -\> [Ordering](https://doc.rust-lang.org/1.86.0/core/cmp/enum.Ordering.html "enum core::cmp::Ordering"){.enum} {#fn-compareself-key-k---ordering .code-header}
:::

::: docblock
Compare self to `key` and return their ordering.
:::
:::::

:::: {#impl-Equivalent%3CK%3E-for-Q .section .impl}
[§](#impl-Equivalent%3CK%3E-for-Q){.anchor}

### impl\<Q, K\> Equivalent\<K\> for Q {#implq-k-equivalentk-for-q .code-header}

::: where
where Q:
[Eq](https://doc.rust-lang.org/1.86.0/core/cmp/trait.Eq.html "trait core::cmp::Eq"){.trait} +
?[Sized](https://doc.rust-lang.org/1.86.0/core/marker/trait.Sized.html "trait core::marker::Sized"){.trait},
K:
[Borrow](https://doc.rust-lang.org/1.86.0/core/borrow/trait.Borrow.html "trait core::borrow::Borrow"){.trait}\<Q\> +
?[Sized](https://doc.rust-lang.org/1.86.0/core/marker/trait.Sized.html "trait core::marker::Sized"){.trait},
:::
::::

::::: impl-items
::: {#method.equivalent .section .method .trait-impl}
[§](#method.equivalent){.anchor}

#### fn [equivalent]{.fn}(&self, key: [&K](https://doc.rust-lang.org/1.86.0/std/primitive.reference.html){.primitive}) -\> [bool](https://doc.rust-lang.org/1.86.0/std/primitive.bool.html){.primitive} {#fn-equivalentself-key-k---bool .code-header}
:::

::: docblock
Checks if this value is equivalent to the given key. Read more
:::
:::::

:::: {#impl-Equivalent%3CK%3E-for-Q-1 .section .impl}
[§](#impl-Equivalent%3CK%3E-for-Q-1){.anchor}

### impl\<Q, K\> Equivalent\<K\> for Q {#implq-k-equivalentk-for-q-1 .code-header}

::: where
where Q:
[Eq](https://doc.rust-lang.org/1.86.0/core/cmp/trait.Eq.html "trait core::cmp::Eq"){.trait} +
?[Sized](https://doc.rust-lang.org/1.86.0/core/marker/trait.Sized.html "trait core::marker::Sized"){.trait},
K:
[Borrow](https://doc.rust-lang.org/1.86.0/core/borrow/trait.Borrow.html "trait core::borrow::Borrow"){.trait}\<Q\> +
?[Sized](https://doc.rust-lang.org/1.86.0/core/marker/trait.Sized.html "trait core::marker::Sized"){.trait},
:::
::::

::::: impl-items
::: {#method.equivalent-1 .section .method .trait-impl}
[§](#method.equivalent-1){.anchor}

#### fn [equivalent]{.fn}(&self, key: [&K](https://doc.rust-lang.org/1.86.0/std/primitive.reference.html){.primitive}) -\> [bool](https://doc.rust-lang.org/1.86.0/std/primitive.bool.html){.primitive} {#fn-equivalentself-key-k---bool-1 .code-header}
:::

::: docblock
Compare self to `key` and return `true` if they are equal.
:::
:::::

::: {#impl-From%3CT%3E-for-T .section .impl}
[Source](https://doc.rust-lang.org/1.86.0/src/core/convert/mod.rs.html#767){.src
.rightside}[§](#impl-From%3CT%3E-for-T){.anchor}

### impl\<T\> [From](https://doc.rust-lang.org/1.86.0/core/convert/trait.From.html "trait core::convert::From"){.trait}\<T\> for T {#implt-fromt-for-t .code-header}
:::

::::: impl-items
::: {#method.from .section .method .trait-impl}
[Source](https://doc.rust-lang.org/1.86.0/src/core/convert/mod.rs.html#770){.src
.rightside}[§](#method.from){.anchor}

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

:::: {#impl-Scalar-for-T .section .impl}
[Source](https://docs.rs/nalgebra/0.25.0/src/nalgebra/base/scalar.rs.html#8){.src
.rightside}[§](#impl-Scalar-for-T){.anchor}

### impl\<T\> [Scalar](https://docs.rs/nalgebra/0.25.0/nalgebra/base/scalar/trait.Scalar.html "trait nalgebra::base::scalar::Scalar"){.trait} for T {#implt-scalar-for-t .code-header}

::: where
where T: \'static +
[Clone](https://doc.rust-lang.org/1.86.0/core/clone/trait.Clone.html "trait core::clone::Clone"){.trait} +
[PartialEq](https://doc.rust-lang.org/1.86.0/core/cmp/trait.PartialEq.html "trait core::cmp::PartialEq"){.trait} +
[Debug](https://doc.rust-lang.org/1.86.0/core/fmt/trait.Debug.html "trait core::fmt::Debug"){.trait},
:::
::::
:::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
:::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
