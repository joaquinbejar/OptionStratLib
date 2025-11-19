:::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::: width-limiter
::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../index.html)::[chains](index.html)
:::

# Struct [OptionData]{.struct} Copy item path

[[Source](../../src/optionstratlib/chains/optiondata.rs.html#70-157){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub struct OptionData {Show 20 fields
    pub strike_price: Positive,
    pub call_bid: Option<Positive>,
    pub call_ask: Option<Positive>,
    pub put_bid: Option<Positive>,
    pub put_ask: Option<Positive>,
    pub call_middle: Option<Positive>,
    pub put_middle: Option<Positive>,
    pub implied_volatility: Positive,
    pub delta_call: Option<Decimal>,
    pub delta_put: Option<Decimal>,
    pub gamma: Option<Decimal>,
    pub volume: Option<Positive>,
    pub open_interest: Option<u64>,
    pub symbol: Option<String>,
    pub expiration_date: Option<ExpirationDate>,
    pub underlying_price: Option<Box<Positive>>,
    pub risk_free_rate: Option<Decimal>,
    pub dividend_yield: Option<Positive>,
    pub epic: Option<String>,
    pub extra_fields: Option<Value>,
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

[[§](#structfield.strike_price){.anchor
.field}`strike_price: `[`Positive`](../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}]{#structfield.strike_price
.structfield .section-header}

::: docblock
The strike price of the option, represented as a positive floating-point
number.
:::

[[§](#structfield.call_bid){.anchor
.field}`call_bid: `[`Option`](https://doc.rust-lang.org/1.91.1/core/option/enum.Option.html "enum core::option::Option"){.enum}`<`[`Positive`](../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}`>`]{#structfield.call_bid
.structfield .section-header}

::: docblock
The bid price for the call option. May be `None` if market data is
unavailable.
:::

[[§](#structfield.call_ask){.anchor
.field}`call_ask: `[`Option`](https://doc.rust-lang.org/1.91.1/core/option/enum.Option.html "enum core::option::Option"){.enum}`<`[`Positive`](../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}`>`]{#structfield.call_ask
.structfield .section-header}

::: docblock
The ask price for the call option. May be `None` if market data is
unavailable.
:::

[[§](#structfield.put_bid){.anchor
.field}`put_bid: `[`Option`](https://doc.rust-lang.org/1.91.1/core/option/enum.Option.html "enum core::option::Option"){.enum}`<`[`Positive`](../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}`>`]{#structfield.put_bid
.structfield .section-header}

::: docblock
The bid price for the put option. May be `None` if market data is
unavailable.
:::

[[§](#structfield.put_ask){.anchor
.field}`put_ask: `[`Option`](https://doc.rust-lang.org/1.91.1/core/option/enum.Option.html "enum core::option::Option"){.enum}`<`[`Positive`](../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}`>`]{#structfield.put_ask
.structfield .section-header}

::: docblock
The ask price for the put option. May be `None` if market data is
unavailable.
:::

[[§](#structfield.call_middle){.anchor
.field}`call_middle: `[`Option`](https://doc.rust-lang.org/1.91.1/core/option/enum.Option.html "enum core::option::Option"){.enum}`<`[`Positive`](../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}`>`]{#structfield.call_middle
.structfield .section-header}

::: docblock
The mid-price between call bid and ask. Calculated as (bid + ask) / 2.
:::

[[§](#structfield.put_middle){.anchor
.field}`put_middle: `[`Option`](https://doc.rust-lang.org/1.91.1/core/option/enum.Option.html "enum core::option::Option"){.enum}`<`[`Positive`](../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}`>`]{#structfield.put_middle
.structfield .section-header}

::: docblock
The mid-price between put bid and ask. Calculated as (bid + ask) / 2.
:::

[[§](#structfield.implied_volatility){.anchor
.field}`implied_volatility: `[`Positive`](../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}]{#structfield.implied_volatility
.structfield .section-header}

::: docblock
The `implied_volatility` field represents the implied volatility value,
which is of type `Positive`. This value is intended to store a positive
number, as enforced by the `Positive` type.

### [§](#attributes){.doc-anchor}Attributes

- `#[serde(default)]`: This attribute ensures that the field is given a
  default value during deserialization if the value is not provided. The
  default implementation for the `Positive` type is expected to supply
  an appropriate default positive value.

### [§](#type){.doc-anchor}Type

- `Positive`: A type that ensures the value it holds is strictly
  positive.

### [§](#usage-1){.doc-anchor}Usage

This field is commonly utilized in contexts where implied volatility is
required, such as in financial modeling or derivative pricing
calculations. Deserialization will automatically manage its default
value if it is absent from the data source.
:::

[[§](#structfield.delta_call){.anchor
.field}`delta_call: `[`Option`](https://doc.rust-lang.org/1.91.1/core/option/enum.Option.html "enum core::option::Option"){.enum}`<`[`Decimal`](../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}`>`]{#structfield.delta_call
.structfield .section-header}

::: docblock
The delta of the call option, measuring price sensitivity to underlying
changes.
:::

[[§](#structfield.delta_put){.anchor
.field}`delta_put: `[`Option`](https://doc.rust-lang.org/1.91.1/core/option/enum.Option.html "enum core::option::Option"){.enum}`<`[`Decimal`](../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}`>`]{#structfield.delta_put
.structfield .section-header}

::: docblock
The delta of the put option, measuring price sensitivity to underlying
changes.
:::

[[§](#structfield.gamma){.anchor
.field}`gamma: `[`Option`](https://doc.rust-lang.org/1.91.1/core/option/enum.Option.html "enum core::option::Option"){.enum}`<`[`Decimal`](../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}`>`]{#structfield.gamma
.structfield .section-header}

::: docblock
The gamma of the option, measuring the rate of change in delta.
:::

[[§](#structfield.volume){.anchor
.field}`volume: `[`Option`](https://doc.rust-lang.org/1.91.1/core/option/enum.Option.html "enum core::option::Option"){.enum}`<`[`Positive`](../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}`>`]{#structfield.volume
.structfield .section-header}

::: docblock
The trading volume of the option, indicating market activity.
:::

[[§](#structfield.open_interest){.anchor
.field}`open_interest: `[`Option`](https://doc.rust-lang.org/1.91.1/core/option/enum.Option.html "enum core::option::Option"){.enum}`<`[`u64`](https://doc.rust-lang.org/1.91.1/std/primitive.u64.html){.primitive}`>`]{#structfield.open_interest
.structfield .section-header}

::: docblock
The open interest, representing the number of outstanding contracts.
:::

[[§](#structfield.symbol){.anchor
.field}`symbol: `[`Option`](https://doc.rust-lang.org/1.91.1/core/option/enum.Option.html "enum core::option::Option"){.enum}`<`[`String`](https://doc.rust-lang.org/1.91.1/alloc/string/struct.String.html "struct alloc::string::String"){.struct}`>`]{#structfield.symbol
.structfield .section-header}

::: docblock
The symbol of the underlying asset.
:::

[[§](#structfield.expiration_date){.anchor
.field}`expiration_date: `[`Option`](https://doc.rust-lang.org/1.91.1/core/option/enum.Option.html "enum core::option::Option"){.enum}`<`[`ExpirationDate`](../model/enum.ExpirationDate.html "enum optionstratlib::model::ExpirationDate"){.enum}`>`]{#structfield.expiration_date
.structfield .section-header}

::: docblock
The expiration date of the option contract.
:::

[[§](#structfield.underlying_price){.anchor
.field}`underlying_price: `[`Option`](https://doc.rust-lang.org/1.91.1/core/option/enum.Option.html "enum core::option::Option"){.enum}`<`[`Box`](https://doc.rust-lang.org/1.91.1/alloc/boxed/struct.Box.html "struct alloc::boxed::Box"){.struct}`<`[`Positive`](../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}`>>`]{#structfield.underlying_price
.structfield .section-header}

::: docblock
The price of the underlying asset.
:::

[[§](#structfield.risk_free_rate){.anchor
.field}`risk_free_rate: `[`Option`](https://doc.rust-lang.org/1.91.1/core/option/enum.Option.html "enum core::option::Option"){.enum}`<`[`Decimal`](../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}`>`]{#structfield.risk_free_rate
.structfield .section-header}

::: docblock
The risk-free interest rate used for option pricing.
:::

[[§](#structfield.dividend_yield){.anchor
.field}`dividend_yield: `[`Option`](https://doc.rust-lang.org/1.91.1/core/option/enum.Option.html "enum core::option::Option"){.enum}`<`[`Positive`](../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}`>`]{#structfield.dividend_yield
.structfield .section-header}

::: docblock
The dividend yield of the underlying asset.
:::

[[§](#structfield.epic){.anchor
.field}`epic: `[`Option`](https://doc.rust-lang.org/1.91.1/core/option/enum.Option.html "enum core::option::Option"){.enum}`<`[`String`](https://doc.rust-lang.org/1.91.1/alloc/string/struct.String.html "struct alloc::string::String"){.struct}`>`]{#structfield.epic
.structfield .section-header}

::: docblock
The epic identifier for the option contract, used for trading platforms.
:::

[[§](#structfield.extra_fields){.anchor
.field}`extra_fields: `[`Option`](https://doc.rust-lang.org/1.91.1/core/option/enum.Option.html "enum core::option::Option"){.enum}`<`[`Value`](../../serde_json/value/enum.Value.html "enum serde_json::value::Value"){.enum}`>`]{#structfield.extra_fields
.structfield .section-header}

::: docblock
Additional fields that may be included in the option data.
:::

## Implementations[§](#implementations){.anchor} {#implementations .section-header}

::::::::::::::::::::::::::::::::::::::::::::::::: {#implementations-list}
::: {#impl-OptionData .section .impl}
[Source](../../src/optionstratlib/chains/optiondata.rs.html#159-985){.src
.rightside}[§](#impl-OptionData){.anchor}

### impl [OptionData](struct.OptionData.html "struct optionstratlib::chains::OptionData"){.struct} {#impl-optiondata .code-header}
:::

::::::::::::::::::::::::::::::::::::::::::::::: impl-items
::: {#method.new .section .method}
[Source](../../src/optionstratlib/chains/optiondata.rs.html#190-232){.src
.rightside}

#### pub fn [new](#method.new){.fn}( strike_price: [Positive](../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, call_bid: [Option](https://doc.rust-lang.org/1.91.1/core/option/enum.Option.html "enum core::option::Option"){.enum}\<[Positive](../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\>, call_ask: [Option](https://doc.rust-lang.org/1.91.1/core/option/enum.Option.html "enum core::option::Option"){.enum}\<[Positive](../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\>, put_bid: [Option](https://doc.rust-lang.org/1.91.1/core/option/enum.Option.html "enum core::option::Option"){.enum}\<[Positive](../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\>, put_ask: [Option](https://doc.rust-lang.org/1.91.1/core/option/enum.Option.html "enum core::option::Option"){.enum}\<[Positive](../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\>, implied_volatility: [Positive](../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, delta_call: [Option](https://doc.rust-lang.org/1.91.1/core/option/enum.Option.html "enum core::option::Option"){.enum}\<[Decimal](../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}\>, delta_put: [Option](https://doc.rust-lang.org/1.91.1/core/option/enum.Option.html "enum core::option::Option"){.enum}\<[Decimal](../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}\>, gamma: [Option](https://doc.rust-lang.org/1.91.1/core/option/enum.Option.html "enum core::option::Option"){.enum}\<[Decimal](../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}\>, volume: [Option](https://doc.rust-lang.org/1.91.1/core/option/enum.Option.html "enum core::option::Option"){.enum}\<[Positive](../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\>, open_interest: [Option](https://doc.rust-lang.org/1.91.1/core/option/enum.Option.html "enum core::option::Option"){.enum}\<[u64](https://doc.rust-lang.org/1.91.1/std/primitive.u64.html){.primitive}\>, symbol: [Option](https://doc.rust-lang.org/1.91.1/core/option/enum.Option.html "enum core::option::Option"){.enum}\<[String](https://doc.rust-lang.org/1.91.1/alloc/string/struct.String.html "struct alloc::string::String"){.struct}\>, expiration_date: [Option](https://doc.rust-lang.org/1.91.1/core/option/enum.Option.html "enum core::option::Option"){.enum}\<[ExpirationDate](../model/enum.ExpirationDate.html "enum optionstratlib::model::ExpirationDate"){.enum}\>, underlying_price: [Option](https://doc.rust-lang.org/1.91.1/core/option/enum.Option.html "enum core::option::Option"){.enum}\<[Box](https://doc.rust-lang.org/1.91.1/alloc/boxed/struct.Box.html "struct alloc::boxed::Box"){.struct}\<[Positive](../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\>\>, risk_free_rate: [Option](https://doc.rust-lang.org/1.91.1/core/option/enum.Option.html "enum core::option::Option"){.enum}\<[Decimal](../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}\>, dividend_yield: [Option](https://doc.rust-lang.org/1.91.1/core/option/enum.Option.html "enum core::option::Option"){.enum}\<[Positive](../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\>, epic: [Option](https://doc.rust-lang.org/1.91.1/core/option/enum.Option.html "enum core::option::Option"){.enum}\<[String](https://doc.rust-lang.org/1.91.1/alloc/string/struct.String.html "struct alloc::string::String"){.struct}\>, extra_fields: [Option](https://doc.rust-lang.org/1.91.1/core/option/enum.Option.html "enum core::option::Option"){.enum}\<[Value](../../serde_json/value/enum.Value.html "enum serde_json::value::Value"){.enum}\>, ) -\> Self {#pub-fn-new-strike_price-positive-call_bid-optionpositive-call_ask-optionpositive-put_bid-optionpositive-put_ask-optionpositive-implied_volatility-positive-delta_call-optiondecimal-delta_put-optiondecimal-gamma-optiondecimal-volume-optionpositive-open_interest-optionu64-symbol-optionstring-expiration_date-optionexpirationdate-underlying_price-optionboxpositive-risk_free_rate-optiondecimal-dividend_yield-optionpositive-epic-optionstring-extra_fields-optionvalue---self .code-header}
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

::: {#method.get_volatility .section .method}
[Source](../../src/optionstratlib/chains/optiondata.rs.html#249-251){.src
.rightside}

#### pub fn [get_volatility](#method.get_volatility){.fn}(&self) -\> [Positive](../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#pub-fn-get_volatilityself---positive .code-header}
:::

::: docblock
Retrieves the implied volatility of the underlying asset or option.

##### [§](#returns-1){.doc-anchor}Returns

An `Option<Positive>` where:

- `Some(Positive)` contains the implied volatility if it is available.
- `None` if the implied volatility is not set or available.

##### [§](#notes){.doc-anchor}Notes

The implied volatility represents the market's forecast of a likely
movement in an asset's price and is often used in option pricing models.

Ensure that the `Positive` type enforces constraints to prevent invalid
values such as negative volatility.
:::

::: {#method.set_volatility .section .method}
[Source](../../src/optionstratlib/chains/optiondata.rs.html#257-259){.src
.rightside}

#### pub fn [set_volatility](#method.set_volatility){.fn}(&mut self, volatility: &[Positive](../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}) {#pub-fn-set_volatilitymut-self-volatility-positive .code-header}
:::

::: docblock
Sets the implied volatility for this option contract.

##### [§](#arguments){.doc-anchor}Arguments

- `volatility` - A positive decimal value representing the implied
  volatility.
:::

::: {#method.set_extra_params .section .method}
[Source](../../src/optionstratlib/chains/optiondata.rs.html#268-288){.src
.rightside}

#### pub fn [set_extra_params](#method.set_extra_params){.fn}(&mut self, params: [OptionDataPriceParams](utils/struct.OptionDataPriceParams.html "struct optionstratlib::chains::utils::OptionDataPriceParams"){.struct}) {#pub-fn-set_extra_paramsmut-self-params-optiondatapriceparams .code-header}
:::

::: docblock
Sets additional pricing parameters for this option contract.

This method updates the option data with the provided pricing
parameters, including underlying symbol, price, expiration date,
risk-free rate, and dividend yield.

##### [§](#arguments-1){.doc-anchor}Arguments

- `params` - The pricing parameters to set.
:::

::: {#method.validate .section .method}
[Source](../../src/optionstratlib/chains/optiondata.rs.html#305-315){.src
.rightside}

#### pub fn [validate](#method.validate){.fn}(&self) -\> [bool](https://doc.rust-lang.org/1.91.1/std/primitive.bool.html){.primitive} {#pub-fn-validateself---bool .code-header}
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

##### [§](#returns-2){.doc-anchor}Returns

- `true` - If all validation checks pass, indicating the option data is
  valid
- `false` - If any validation check fails, indicating the option data is
  incomplete or invalid
:::

::: {#method.strike .section .method}
[Source](../../src/optionstratlib/chains/optiondata.rs.html#330-332){.src
.rightside}

#### pub fn [strike](#method.strike){.fn}(&self) -\> [Positive](../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#pub-fn-strikeself---positive .code-header}
:::

::: docblock
Retrieves the strike price.

This method returns the strike price associated with the object. The
strike price is represented as a [`Positive`](struct.Positive.html)
value, ensuring that it is always greater than zero.

##### [§](#returns-3){.doc-anchor}Returns

- [`Positive`](struct.Positive.html) - The strike price of the object.

##### [§](#notes-1){.doc-anchor}Notes

The method assumes that the strike price has been properly initialized
and is a valid positive number.
:::

::: {#method.get_call_buy_price .section .method}
[Source](../../src/optionstratlib/chains/optiondata.rs.html#370-372){.src
.rightside}

#### pub fn [get_call_buy_price](#method.get_call_buy_price){.fn}(&self) -\> [Option](https://doc.rust-lang.org/1.91.1/core/option/enum.Option.html "enum core::option::Option"){.enum}\<[Positive](../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\> {#pub-fn-get_call_buy_priceself---optionpositive .code-header}
:::

::: docblock
Retrieves the price at which a call option can be purchased.

This method returns the ask price for a call option, which is the price
a buyer would pay to purchase the call option.

##### [§](#returns-4){.doc-anchor}Returns

The call option's ask price as a `Positive` value, or `None` if the
price is unavailable.
:::

::: {#method.get_call_sell_price .section .method}
[Source](../../src/optionstratlib/chains/optiondata.rs.html#382-384){.src
.rightside}

#### pub fn [get_call_sell_price](#method.get_call_sell_price){.fn}(&self) -\> [Option](https://doc.rust-lang.org/1.91.1/core/option/enum.Option.html "enum core::option::Option"){.enum}\<[Positive](../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\> {#pub-fn-get_call_sell_priceself---optionpositive .code-header}
:::

::: docblock
Retrieves the price at which a call option can be sold.

This method returns the bid price for a call option, which is the price
a seller would receive when selling the call option.

##### [§](#returns-5){.doc-anchor}Returns

The call option's bid price as a `Positive` value, or `None` if the
price is unavailable.
:::

::: {#method.get_put_buy_price .section .method}
[Source](../../src/optionstratlib/chains/optiondata.rs.html#394-396){.src
.rightside}

#### pub fn [get_put_buy_price](#method.get_put_buy_price){.fn}(&self) -\> [Option](https://doc.rust-lang.org/1.91.1/core/option/enum.Option.html "enum core::option::Option"){.enum}\<[Positive](../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\> {#pub-fn-get_put_buy_priceself---optionpositive .code-header}
:::

::: docblock
Retrieves the price at which a put option can be purchased.

This method returns the ask price for a put option, which is the price a
buyer would pay to purchase the put option.

##### [§](#returns-6){.doc-anchor}Returns

The put option's ask price as a `Positive` value, or `None` if the price
is unavailable.
:::

::: {#method.get_put_sell_price .section .method}
[Source](../../src/optionstratlib/chains/optiondata.rs.html#406-408){.src
.rightside}

#### pub fn [get_put_sell_price](#method.get_put_sell_price){.fn}(&self) -\> [Option](https://doc.rust-lang.org/1.91.1/core/option/enum.Option.html "enum core::option::Option"){.enum}\<[Positive](../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\> {#pub-fn-get_put_sell_priceself---optionpositive .code-header}
:::

::: docblock
Retrieves the price at which a put option can be sold.

This method returns the bid price for a put option, which is the price a
seller would receive when selling the put option.

##### [§](#returns-7){.doc-anchor}Returns

The put option's bid price as a `Positive` value, or `None` if the price
is unavailable.
:::

::: {#method.some_price_is_none .section .method}
[Source](../../src/optionstratlib/chains/optiondata.rs.html#416-421){.src
.rightside}

#### pub fn [some_price_is_none](#method.some_price_is_none){.fn}(&self) -\> [bool](https://doc.rust-lang.org/1.91.1/std/primitive.bool.html){.primitive} {#pub-fn-some_price_is_noneself---bool .code-header}
:::

::: docblock
Checks if any of the bid or ask prices for call or put options are None.

This function returns `true` if any of the `call_bid`, `call_ask`,
`put_bid`, or `put_ask` fields of the `OptionData` struct are `None`,
indicating missing price information. It returns `false` if all four
fields have valid price data.
:::

::: {#method.get_position .section .method}
[Source](../../src/optionstratlib/chains/optiondata.rs.html#527-574){.src
.rightside}

#### pub fn [get_position](#method.get_position){.fn}( &self, side: [Side](../model/types/enum.Side.html "enum optionstratlib::model::types::Side"){.enum}, option_style: [OptionStyle](../model/types/enum.OptionStyle.html "enum optionstratlib::model::types::OptionStyle"){.enum}, date: [Option](https://doc.rust-lang.org/1.91.1/core/option/enum.Option.html "enum core::option::Option"){.enum}\<[DateTime](../../chrono/datetime/struct.DateTime.html "struct chrono::datetime::DateTime"){.struct}\<[Utc](../prelude/struct.Utc.html "struct optionstratlib::prelude::Utc"){.struct}\>\>, open_fee: [Option](https://doc.rust-lang.org/1.91.1/core/option/enum.Option.html "enum core::option::Option"){.enum}\<[Positive](../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\>, close_fee: [Option](https://doc.rust-lang.org/1.91.1/core/option/enum.Option.html "enum core::option::Option"){.enum}\<[Positive](../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\>, ) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Position](../model/position/struct.Position.html "struct optionstratlib::model::position::Position"){.struct}, [ChainError](../error/chains/enum.ChainError.html "enum optionstratlib::error::chains::ChainError"){.enum}\> {#pub-fn-get_position-self-side-side-option_style-optionstyle-date-optiondatetimeutc-open_fee-optionpositive-close_fee-optionpositive---resultposition-chainerror .code-header}
:::

::: docblock
Retrieves a `Position` based on the provided parameters, calculating the
option premium using the Black-Scholes model.

This method fetches an option based on the provided parameters,
calculates its theoretical premium using the Black-Scholes model, and
constructs a `Position` struct containing the option details, premium,
opening date, and associated fees.

##### [§](#arguments-2){.doc-anchor}Arguments

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

##### [§](#returns-8){.doc-anchor}Returns

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
[Source](../../src/optionstratlib/chains/optiondata.rs.html#631-678){.src
.rightside}

#### pub fn [calculate_prices](#method.calculate_prices){.fn}( &mut self, spread: [Option](https://doc.rust-lang.org/1.91.1/core/option/enum.Option.html "enum core::option::Option"){.enum}\<[Positive](../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\>, ) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[()](https://doc.rust-lang.org/1.91.1/std/primitive.unit.html){.primitive}, [ChainError](../error/chains/enum.ChainError.html "enum optionstratlib::error::chains::ChainError"){.enum}\> {#pub-fn-calculate_prices-mut-self-spread-optionpositive---result-chainerror .code-header}
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

##### [§](#returns-9){.doc-anchor}Returns

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
[Source](../../src/optionstratlib/chains/optiondata.rs.html#702-772){.src
.rightside}

#### pub fn [apply_spread](#method.apply_spread){.fn}(&mut self, spread: [Positive](../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, decimal_places: [u32](https://doc.rust-lang.org/1.91.1/std/primitive.u32.html){.primitive}) {#pub-fn-apply_spreadmut-self-spread-positive-decimal_places-u32 .code-header}
:::

::: docblock
Applies a spread to the bid and ask prices of call and put options, then
recalculates mid prices.

This method adjusts the bid and ask prices by half of the specified
spread value, subtracting from bid prices and adding to ask prices. It
also ensures that all prices are rounded to the specified number of
decimal places. If any price becomes negative after applying the spread,
it is set to `None`.

##### [§](#arguments-3){.doc-anchor}Arguments

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
[Source](../../src/optionstratlib/chains/optiondata.rs.html#779-810){.src
.rightside}

#### pub fn [calculate_delta](#method.calculate_delta){.fn}(&mut self) {#pub-fn-calculate_deltamut-self .code-header}
:::

::: docblock
Calculates the delta of the option and stores it in the option data.

Delta measures the rate of change of the option price with respect to
changes in the underlying asset's price. This method creates a Call
option and uses it to calculate the delta value.
:::

::: {#method.calculate_gamma .section .method}
[Source](../../src/optionstratlib/chains/optiondata.rs.html#817-832){.src
.rightside}

#### pub fn [calculate_gamma](#method.calculate_gamma){.fn}(&mut self) {#pub-fn-calculate_gammamut-self .code-header}
:::

::: docblock
Calculates the gamma of the option and stores it in the option data.

Gamma measures the rate of change of delta with respect to changes in
the underlying asset's price. This method creates a Call option and uses
it to calculate the gamma value.
:::

::: {#method.get_deltas .section .method}
[Source](../../src/optionstratlib/chains/optiondata.rs.html#854-857){.src
.rightside}

#### pub fn [get_deltas](#method.get_deltas){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[DeltasInStrike](struct.DeltasInStrike.html "struct optionstratlib::chains::DeltasInStrike"){.struct}, [ChainError](../error/chains/enum.ChainError.html "enum optionstratlib::error::chains::ChainError"){.enum}\> {#pub-fn-get_deltasself---resultdeltasinstrike-chainerror .code-header}
:::

::: docblock
Retrieves delta values for options at the current strike price.

Delta measures the rate of change of the option price with respect to
changes in the underlying asset's price. This method returns delta
values for options at the specific strike price defined in the price
parameters.

##### [§](#parameters-2){.doc-anchor}Parameters

- `price_params` - A reference to the pricing parameters required for
  option calculations, including underlying price, expiration date,
  risk-free rate and other inputs.

##### [§](#returns-10){.doc-anchor}Returns

- `Result<DeltasInStrike, ChainError>` - On success, returns a structure
  containing delta values for the options at the specified strike. On
  failure, returns a ChainError describing the issue.

##### [§](#errors-2){.doc-anchor}Errors

- Returns a `ChainError` if there's an issue retrieving the options or
  calculating their deltas.
- Possible errors include missing option data, calculation failures, or
  invalid parameters.
:::

::: {#method.is_valid_optimal_side .section .method}
[Source](../../src/optionstratlib/chains/optiondata.rs.html#877-902){.src
.rightside}

#### pub fn [is_valid_optimal_side](#method.is_valid_optimal_side){.fn}( &self, underlying_price: &[Positive](../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, side: &[FindOptimalSide](../strategies/utils/enum.FindOptimalSide.html "enum optionstratlib::strategies::utils::FindOptimalSide"){.enum}, ) -\> [bool](https://doc.rust-lang.org/1.91.1/std/primitive.bool.html){.primitive} {#pub-fn-is_valid_optimal_side-self-underlying_price-positive-side-findoptimalside---bool .code-header}
:::

::: docblock
Validates if an option strike price is valid according to the specified
search strategy.

This method checks whether the current option's strike price falls
within the constraints defined by the `FindOptimalSide` parameter,
relative to the given underlying asset price.

##### [§](#parameters-3){.doc-anchor}Parameters

- `underlying_price` - The current market price of the underlying asset
  as a `Positive` value.
- `side` - The strategy to determine valid strike prices, specifying
  whether to consider options with strikes above, below, or within a
  specific range of the underlying price.

##### [§](#returns-11){.doc-anchor}Returns

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
[Source](../../src/optionstratlib/chains/optiondata.rs.html#915-924){.src
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

##### [§](#side-effects-2){.doc-anchor}Side Effects

Updates the `call_middle` and `put_middle` fields with the calculated
mid-prices.
:::

::: {#method.get_mid_prices .section .method}
[Source](../../src/optionstratlib/chains/optiondata.rs.html#937-939){.src
.rightside}

#### pub fn [get_mid_prices](#method.get_mid_prices){.fn}(&self) -\> ([Option](https://doc.rust-lang.org/1.91.1/core/option/enum.Option.html "enum core::option::Option"){.enum}\<[Positive](../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\>, [Option](https://doc.rust-lang.org/1.91.1/core/option/enum.Option.html "enum core::option::Option"){.enum}\<[Positive](../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\>) {#pub-fn-get_mid_pricesself---optionpositive-optionpositive .code-header}
:::

::: docblock
Retrieves the current mid-prices for call and put options.

This method returns the calculated middle prices for both call and put
options as a tuple. Each price may be `None` if the corresponding
bid/ask prices were not available when `set_mid_prices()` was called.

##### [§](#returns-12){.doc-anchor}Returns

A tuple containing:

- First element: The call option mid-price (bid+ask)/2, or `None` if not
  available
- Second element: The put option mid-price (bid+ask)/2, or `None` if not
  available
:::

::: {#method.current_deltas .section .method}
[Source](../../src/optionstratlib/chains/optiondata.rs.html#967-969){.src
.rightside}

#### pub fn [current_deltas](#method.current_deltas){.fn}(&self) -\> ([Option](https://doc.rust-lang.org/1.91.1/core/option/enum.Option.html "enum core::option::Option"){.enum}\<[Decimal](../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}\>, [Option](https://doc.rust-lang.org/1.91.1/core/option/enum.Option.html "enum core::option::Option"){.enum}\<[Decimal](../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}\>) {#pub-fn-current_deltasself---optiondecimal-optiondecimal .code-header}
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

::: {#method.current_gamma .section .method}
[Source](../../src/optionstratlib/chains/optiondata.rs.html#982-984){.src
.rightside}

#### pub fn [current_gamma](#method.current_gamma){.fn}(&self) -\> [Option](https://doc.rust-lang.org/1.91.1/core/option/enum.Option.html "enum core::option::Option"){.enum}\<[Decimal](../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}\> {#pub-fn-current_gammaself---optiondecimal .code-header}
:::

::: docblock
Returns the current gamma value.

This function retrieves the optional `gamma` field of the struct. If the
`gamma` field has been set, it returns a `Some(Decimal)` value;
otherwise, it returns `None`.

##### [§](#returns-14){.doc-anchor}Returns

- `Option<Decimal>` - The current gamma value wrapped in `Some` if it
  exists, or `None` if the gamma value is not set.
:::
:::::::::::::::::::::::::::::::::::::::::::::::
:::::::::::::::::::::::::::::::::::::::::::::::::

## Trait Implementations[§](#trait-implementations){.anchor} {#trait-implementations .section-header}

::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::: {#trait-implementations-list}
::: {#impl-Clone-for-OptionData .section .impl}
[Source](../../src/optionstratlib/chains/optiondata.rs.html#69){.src
.rightside}[§](#impl-Clone-for-OptionData){.anchor}

### impl [Clone](https://doc.rust-lang.org/1.91.1/core/clone/trait.Clone.html "trait core::clone::Clone"){.trait} for [OptionData](struct.OptionData.html "struct optionstratlib::chains::OptionData"){.struct} {#impl-clone-for-optiondata .code-header}
:::

::::::: impl-items
::: {#method.clone .section .method .trait-impl}
[Source](../../src/optionstratlib/chains/optiondata.rs.html#69){.src
.rightside}[§](#method.clone){.anchor}

#### fn [clone](https://doc.rust-lang.org/1.91.1/core/clone/trait.Clone.html#tymethod.clone){.fn}(&self) -\> [OptionData](struct.OptionData.html "struct optionstratlib::chains::OptionData"){.struct} {#fn-cloneself---optiondata .code-header}
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

::: {#impl-ComposeSchema-for-OptionData .section .impl}
[Source](../../src/optionstratlib/chains/optiondata.rs.html#69){.src
.rightside}[§](#impl-ComposeSchema-for-OptionData){.anchor}

### impl ComposeSchema for [OptionData](struct.OptionData.html "struct optionstratlib::chains::OptionData"){.struct} {#impl-composeschema-for-optiondata .code-header}
:::

:::: impl-items
::: {#method.compose .section .method .trait-impl}
[Source](../../src/optionstratlib/chains/optiondata.rs.html#69){.src
.rightside}[§](#method.compose){.anchor}

#### fn [compose](#tymethod.compose){.fn}(generics: [Vec](https://doc.rust-lang.org/1.91.1/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<[RefOr](../../utoipa/openapi/enum.RefOr.html "enum utoipa::openapi::RefOr"){.enum}\<[Schema](../../utoipa/openapi/schema/enum.Schema.html "enum utoipa::openapi::schema::Schema"){.enum}\>\>) -\> [RefOr](../../utoipa/openapi/enum.RefOr.html "enum utoipa::openapi::RefOr"){.enum}\<[Schema](../../utoipa/openapi/schema/enum.Schema.html "enum utoipa::openapi::schema::Schema"){.enum}\> {#fn-composegenerics-vecreforschema---reforschema .code-header}
:::
::::

::: {#impl-Debug-for-OptionData .section .impl}
[Source](../../src/optionstratlib/chains/optiondata.rs.html#69){.src
.rightside}[§](#impl-Debug-for-OptionData){.anchor}

### impl [Debug](https://doc.rust-lang.org/1.91.1/core/fmt/trait.Debug.html "trait core::fmt::Debug"){.trait} for [OptionData](struct.OptionData.html "struct optionstratlib::chains::OptionData"){.struct} {#impl-debug-for-optiondata .code-header}
:::

::::: impl-items
::: {#method.fmt .section .method .trait-impl}
[Source](../../src/optionstratlib/chains/optiondata.rs.html#69){.src
.rightside}[§](#method.fmt){.anchor}

#### fn [fmt](https://doc.rust-lang.org/1.91.1/core/fmt/trait.Debug.html#tymethod.fmt){.fn}(&self, f: &mut [Formatter](https://doc.rust-lang.org/1.91.1/core/fmt/struct.Formatter.html "struct core::fmt::Formatter"){.struct}\<\'\_\>) -\> [Result](https://doc.rust-lang.org/1.91.1/core/fmt/type.Result.html "type core::fmt::Result"){.type} {#fn-fmtself-f-mut-formatter_---result .code-header}
:::

::: docblock
Formats the value using the given formatter. [Read
more](https://doc.rust-lang.org/1.91.1/core/fmt/trait.Debug.html#tymethod.fmt)
:::
:::::

::: {#impl-Default-for-OptionData .section .impl}
[Source](../../src/optionstratlib/chains/optiondata.rs.html#987-1012){.src
.rightside}[§](#impl-Default-for-OptionData){.anchor}

### impl [Default](https://doc.rust-lang.org/1.91.1/core/default/trait.Default.html "trait core::default::Default"){.trait} for [OptionData](struct.OptionData.html "struct optionstratlib::chains::OptionData"){.struct} {#impl-default-for-optiondata .code-header}
:::

::::: impl-items
::: {#method.default .section .method .trait-impl}
[Source](../../src/optionstratlib/chains/optiondata.rs.html#988-1011){.src
.rightside}[§](#method.default){.anchor}

#### fn [default](https://doc.rust-lang.org/1.91.1/core/default/trait.Default.html#tymethod.default){.fn}() -\> Self {#fn-default---self .code-header}
:::

::: docblock
Returns the "default value" for a type. [Read
more](https://doc.rust-lang.org/1.91.1/core/default/trait.Default.html#tymethod.default)
:::
:::::

::: {#impl-Deserialize%3C'de%3E-for-OptionData .section .impl}
[Source](../../src/optionstratlib/chains/optiondata.rs.html#69){.src
.rightside}[§](#impl-Deserialize%3C'de%3E-for-OptionData){.anchor}

### impl\<\'de\> [Deserialize](../../serde_core/de/trait.Deserialize.html "trait serde_core::de::Deserialize"){.trait}\<\'de\> for [OptionData](struct.OptionData.html "struct optionstratlib::chains::OptionData"){.struct} {#implde-deserializede-for-optiondata .code-header}
:::

:::::: impl-items
:::: {#method.deserialize .section .method .trait-impl}
[Source](../../src/optionstratlib/chains/optiondata.rs.html#69){.src
.rightside}[§](#method.deserialize){.anchor}

#### fn [deserialize](../../serde_core/de/trait.Deserialize.html#tymethod.deserialize){.fn}\<\_\_D\>(\_\_deserializer: \_\_D) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<Self, \_\_D::[Error](../../serde_core/de/trait.Deserializer.html#associatedtype.Error "type serde_core::de::Deserializer::Error"){.associatedtype}\> {#fn-deserialize__d__deserializer-__d---resultself-__derror .code-header}

::: where
where \_\_D:
[Deserializer](../../serde_core/de/trait.Deserializer.html "trait serde_core::de::Deserializer"){.trait}\<\'de\>,
:::
::::

::: docblock
Deserialize this value from the given Serde deserializer. [Read
more](../../serde_core/de/trait.Deserialize.html#tymethod.deserialize)
:::
::::::

::: {#impl-Display-for-OptionData .section .impl}
[Source](../../src/optionstratlib/chains/optiondata.rs.html#1028-1053){.src
.rightside}[§](#impl-Display-for-OptionData){.anchor}

### impl [Display](https://doc.rust-lang.org/1.91.1/core/fmt/trait.Display.html "trait core::fmt::Display"){.trait} for [OptionData](struct.OptionData.html "struct optionstratlib::chains::OptionData"){.struct} {#impl-display-for-optiondata .code-header}
:::

::::: impl-items
::: {#method.fmt-1 .section .method .trait-impl}
[Source](../../src/optionstratlib/chains/optiondata.rs.html#1029-1052){.src
.rightside}[§](#method.fmt-1){.anchor}

#### fn [fmt](https://doc.rust-lang.org/1.91.1/core/fmt/trait.Display.html#tymethod.fmt){.fn}(&self, f: &mut [Formatter](https://doc.rust-lang.org/1.91.1/core/fmt/struct.Formatter.html "struct core::fmt::Formatter"){.struct}\<\'\_\>) -\> [Result](https://doc.rust-lang.org/1.91.1/core/fmt/type.Result.html "type core::fmt::Result"){.type} {#fn-fmtself-f-mut-formatter_---result-1 .code-header}
:::

::: docblock
Formats the value using the given formatter. [Read
more](https://doc.rust-lang.org/1.91.1/core/fmt/trait.Display.html#tymethod.fmt)
:::
:::::

::: {#impl-From%3C%26OptionData%3E-for-Options .section .impl}
[Source](../../src/optionstratlib/model/option.rs.html#668-693){.src
.rightside}[§](#impl-From%3C%26OptionData%3E-for-Options){.anchor}

### impl [From](https://doc.rust-lang.org/1.91.1/core/convert/trait.From.html "trait core::convert::From"){.trait}\<&[OptionData](struct.OptionData.html "struct optionstratlib::chains::OptionData"){.struct}\> for [Options](../model/option/struct.Options.html "struct optionstratlib::model::option::Options"){.struct} {#impl-fromoptiondata-for-options .code-header}
:::

::::: impl-items
::: {#method.from .section .method .trait-impl}
[Source](../../src/optionstratlib/model/option.rs.html#669-692){.src
.rightside}[§](#method.from){.anchor}

#### fn [from](https://doc.rust-lang.org/1.91.1/core/convert/trait.From.html#tymethod.from){.fn}(option_data: &[OptionData](struct.OptionData.html "struct optionstratlib::chains::OptionData"){.struct}) -\> Self {#fn-fromoption_data-optiondata---self .code-header}
:::

::: docblock
Converts to this type from the input type.
:::
:::::

::: {#impl-Ord-for-OptionData .section .impl}
[Source](../../src/optionstratlib/chains/optiondata.rs.html#1022-1026){.src
.rightside}[§](#impl-Ord-for-OptionData){.anchor}

### impl [Ord](https://doc.rust-lang.org/1.91.1/core/cmp/trait.Ord.html "trait core::cmp::Ord"){.trait} for [OptionData](struct.OptionData.html "struct optionstratlib::chains::OptionData"){.struct} {#impl-ord-for-optiondata .code-header}
:::

:::::::::::::: impl-items
::: {#method.cmp .section .method .trait-impl}
[Source](../../src/optionstratlib/chains/optiondata.rs.html#1023-1025){.src
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

::: {#impl-PartialEq-for-OptionData .section .impl}
[Source](../../src/optionstratlib/chains/optiondata.rs.html#69){.src
.rightside}[§](#impl-PartialEq-for-OptionData){.anchor}

### impl [PartialEq](https://doc.rust-lang.org/1.91.1/core/cmp/trait.PartialEq.html "trait core::cmp::PartialEq"){.trait} for [OptionData](struct.OptionData.html "struct optionstratlib::chains::OptionData"){.struct} {#impl-partialeq-for-optiondata .code-header}
:::

::::::: impl-items
::: {#method.eq .section .method .trait-impl}
[Source](../../src/optionstratlib/chains/optiondata.rs.html#69){.src
.rightside}[§](#method.eq){.anchor}

#### fn [eq](https://doc.rust-lang.org/1.91.1/core/cmp/trait.PartialEq.html#tymethod.eq){.fn}(&self, other: &[OptionData](struct.OptionData.html "struct optionstratlib::chains::OptionData"){.struct}) -\> [bool](https://doc.rust-lang.org/1.91.1/std/primitive.bool.html){.primitive} {#fn-eqself-other-optiondata---bool .code-header}
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

::: {#impl-PartialOrd-for-OptionData .section .impl}
[Source](../../src/optionstratlib/chains/optiondata.rs.html#1014-1018){.src
.rightside}[§](#impl-PartialOrd-for-OptionData){.anchor}

### impl [PartialOrd](https://doc.rust-lang.org/1.91.1/core/cmp/trait.PartialOrd.html "trait core::cmp::PartialOrd"){.trait} for [OptionData](struct.OptionData.html "struct optionstratlib::chains::OptionData"){.struct} {#impl-partialord-for-optiondata .code-header}
:::

::::::::::::: impl-items
::: {#method.partial_cmp .section .method .trait-impl}
[Source](../../src/optionstratlib/chains/optiondata.rs.html#1015-1017){.src
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

::: {#impl-Serialize-for-OptionData .section .impl}
[Source](../../src/optionstratlib/chains/optiondata.rs.html#69){.src
.rightside}[§](#impl-Serialize-for-OptionData){.anchor}

### impl [Serialize](../../serde_core/ser/trait.Serialize.html "trait serde_core::ser::Serialize"){.trait} for [OptionData](struct.OptionData.html "struct optionstratlib::chains::OptionData"){.struct} {#impl-serialize-for-optiondata .code-header}
:::

:::::: impl-items
:::: {#method.serialize .section .method .trait-impl}
[Source](../../src/optionstratlib/chains/optiondata.rs.html#69){.src
.rightside}[§](#method.serialize){.anchor}

#### fn [serialize](../../serde_core/ser/trait.Serialize.html#tymethod.serialize){.fn}\<\_\_S\>(&self, \_\_serializer: \_\_S) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<\_\_S::[Ok](../../serde_core/ser/trait.Serializer.html#associatedtype.Ok "type serde_core::ser::Serializer::Ok"){.associatedtype}, \_\_S::[Error](../../serde_core/ser/trait.Serializer.html#associatedtype.Error "type serde_core::ser::Serializer::Error"){.associatedtype}\> {#fn-serialize__sself-__serializer-__s---result__sok-__serror .code-header}

::: where
where \_\_S:
[Serializer](../../serde_core/ser/trait.Serializer.html "trait serde_core::ser::Serializer"){.trait},
:::
::::

::: docblock
Serialize this value into the given Serde serializer. [Read
more](../../serde_core/ser/trait.Serialize.html#tymethod.serialize)
:::
::::::

::: {#impl-ToSchema-for-OptionData .section .impl}
[Source](../../src/optionstratlib/chains/optiondata.rs.html#69){.src
.rightside}[§](#impl-ToSchema-for-OptionData){.anchor}

### impl [ToSchema](../../utoipa/trait.ToSchema.html "trait utoipa::ToSchema"){.trait} for [OptionData](struct.OptionData.html "struct optionstratlib::chains::OptionData"){.struct} {#impl-toschema-for-optiondata .code-header}
:::

::::::: impl-items
::: {#method.name .section .method .trait-impl}
[Source](../../src/optionstratlib/chains/optiondata.rs.html#69){.src
.rightside}[§](#method.name){.anchor}

#### fn [name](../../utoipa/trait.ToSchema.html#method.name){.fn}() -\> [Cow](https://doc.rust-lang.org/1.91.1/alloc/borrow/enum.Cow.html "enum alloc::borrow::Cow"){.enum}\<\'static, [str](https://doc.rust-lang.org/1.91.1/std/primitive.str.html){.primitive}\> {#fn-name---cowstatic-str .code-header}
:::

::: docblock
Return name of the schema. [Read
more](../../utoipa/trait.ToSchema.html#method.name)
:::

::: {#method.schemas .section .method .trait-impl}
[Source](../../src/optionstratlib/chains/optiondata.rs.html#69){.src
.rightside}[§](#method.schemas){.anchor}

#### fn [schemas](../../utoipa/trait.ToSchema.html#method.schemas){.fn}(schemas: &mut [Vec](https://doc.rust-lang.org/1.91.1/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<([String](https://doc.rust-lang.org/1.91.1/alloc/string/struct.String.html "struct alloc::string::String"){.struct}, [RefOr](../../utoipa/openapi/enum.RefOr.html "enum utoipa::openapi::RefOr"){.enum}\<[Schema](../../utoipa/openapi/schema/enum.Schema.html "enum utoipa::openapi::schema::Schema"){.enum}\>)\>) {#fn-schemasschemas-mut-vecstring-reforschema .code-header}
:::

::: docblock
Implement reference
[`utoipa::openapi::schema::Schema`](../../utoipa/openapi/schema/enum.Schema.html "enum utoipa::openapi::schema::Schema")s
for this type. [Read
more](../../utoipa/trait.ToSchema.html#method.schemas)
:::
:::::::

::: {#impl-Eq-for-OptionData .section .impl}
[Source](../../src/optionstratlib/chains/optiondata.rs.html#1020){.src
.rightside}[§](#impl-Eq-for-OptionData){.anchor}

### impl [Eq](https://doc.rust-lang.org/1.91.1/core/cmp/trait.Eq.html "trait core::cmp::Eq"){.trait} for [OptionData](struct.OptionData.html "struct optionstratlib::chains::OptionData"){.struct} {#impl-eq-for-optiondata .code-header}
:::

::: {#impl-StructuralPartialEq-for-OptionData .section .impl}
[Source](../../src/optionstratlib/chains/optiondata.rs.html#69){.src
.rightside}[§](#impl-StructuralPartialEq-for-OptionData){.anchor}

### impl [StructuralPartialEq](https://doc.rust-lang.org/1.91.1/core/marker/trait.StructuralPartialEq.html "trait core::marker::StructuralPartialEq"){.trait} for [OptionData](struct.OptionData.html "struct optionstratlib::chains::OptionData"){.struct} {#impl-structuralpartialeq-for-optiondata .code-header}
:::
:::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::

## Auto Trait Implementations[§](#synthetic-implementations){.anchor} {#synthetic-implementations .section-header}

::::::::: {#synthetic-implementations-list}
::: {#impl-Freeze-for-OptionData .section .impl}
[§](#impl-Freeze-for-OptionData){.anchor}

### impl [Freeze](https://doc.rust-lang.org/1.91.1/core/marker/trait.Freeze.html "trait core::marker::Freeze"){.trait} for [OptionData](struct.OptionData.html "struct optionstratlib::chains::OptionData"){.struct} {#impl-freeze-for-optiondata .code-header}
:::

::: {#impl-RefUnwindSafe-for-OptionData .section .impl}
[§](#impl-RefUnwindSafe-for-OptionData){.anchor}

### impl [RefUnwindSafe](https://doc.rust-lang.org/1.91.1/core/panic/unwind_safe/trait.RefUnwindSafe.html "trait core::panic::unwind_safe::RefUnwindSafe"){.trait} for [OptionData](struct.OptionData.html "struct optionstratlib::chains::OptionData"){.struct} {#impl-refunwindsafe-for-optiondata .code-header}
:::

::: {#impl-Send-for-OptionData .section .impl}
[§](#impl-Send-for-OptionData){.anchor}

### impl [Send](https://doc.rust-lang.org/1.91.1/core/marker/trait.Send.html "trait core::marker::Send"){.trait} for [OptionData](struct.OptionData.html "struct optionstratlib::chains::OptionData"){.struct} {#impl-send-for-optiondata .code-header}
:::

::: {#impl-Sync-for-OptionData .section .impl}
[§](#impl-Sync-for-OptionData){.anchor}

### impl [Sync](https://doc.rust-lang.org/1.91.1/core/marker/trait.Sync.html "trait core::marker::Sync"){.trait} for [OptionData](struct.OptionData.html "struct optionstratlib::chains::OptionData"){.struct} {#impl-sync-for-optiondata .code-header}
:::

::: {#impl-Unpin-for-OptionData .section .impl}
[§](#impl-Unpin-for-OptionData){.anchor}

### impl [Unpin](https://doc.rust-lang.org/1.91.1/core/marker/trait.Unpin.html "trait core::marker::Unpin"){.trait} for [OptionData](struct.OptionData.html "struct optionstratlib::chains::OptionData"){.struct} {#impl-unpin-for-optiondata .code-header}
:::

::: {#impl-UnwindSafe-for-OptionData .section .impl}
[§](#impl-UnwindSafe-for-OptionData){.anchor}

### impl [UnwindSafe](https://doc.rust-lang.org/1.91.1/core/panic/unwind_safe/trait.UnwindSafe.html "trait core::panic::unwind_safe::UnwindSafe"){.trait} for [OptionData](struct.OptionData.html "struct optionstratlib::chains::OptionData"){.struct} {#impl-unwindsafe-for-optiondata .code-header}
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
[Source](../../src/equivalent/lib.rs.html#104-107){.src
.rightside}[§](#impl-Comparable%3CK%3E-for-Q){.anchor}

### impl\<Q, K\> [Comparable](../../equivalent/trait.Comparable.html "trait equivalent::Comparable"){.trait}\<K\> for Q {#implq-k-comparablek-for-q .code-header}

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
[Source](../../src/equivalent/lib.rs.html#110){.src
.rightside}[§](#method.compare){.anchor}

#### fn [compare](../../equivalent/trait.Comparable.html#tymethod.compare){.fn}(&self, key: [&K](https://doc.rust-lang.org/1.91.1/std/primitive.reference.html){.primitive}) -\> [Ordering](https://doc.rust-lang.org/1.91.1/core/cmp/enum.Ordering.html "enum core::cmp::Ordering"){.enum} {#fn-compareself-key-k---ordering .code-header}
:::

::: docblock
Compare self to `key` and return their ordering.
:::
:::::

:::: {#impl-Equivalent%3CK%3E-for-Q .section .impl}
[Source](../../src/hashbrown/lib.rs.html#167-170){.src
.rightside}[§](#impl-Equivalent%3CK%3E-for-Q){.anchor}

### impl\<Q, K\> [Equivalent](../../hashbrown/trait.Equivalent.html "trait hashbrown::Equivalent"){.trait}\<K\> for Q {#implq-k-equivalentk-for-q .code-header}

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
[Source](../../src/hashbrown/lib.rs.html#172){.src
.rightside}[§](#method.equivalent){.anchor}

#### fn [equivalent](../../hashbrown/trait.Equivalent.html#tymethod.equivalent){.fn}(&self, key: [&K](https://doc.rust-lang.org/1.91.1/std/primitive.reference.html){.primitive}) -\> [bool](https://doc.rust-lang.org/1.91.1/std/primitive.bool.html){.primitive} {#fn-equivalentself-key-k---bool .code-header}
:::

::: docblock
Checks if this value is equivalent to the given key. [Read
more](../../hashbrown/trait.Equivalent.html#tymethod.equivalent)
:::
:::::

:::: {#impl-Equivalent%3CK%3E-for-Q-1 .section .impl}
[Source](../../src/equivalent/lib.rs.html#82-85){.src
.rightside}[§](#impl-Equivalent%3CK%3E-for-Q-1){.anchor}

### impl\<Q, K\> [Equivalent](../../equivalent/trait.Equivalent.html "trait equivalent::Equivalent"){.trait}\<K\> for Q {#implq-k-equivalentk-for-q-1 .code-header}

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
[Source](../../src/equivalent/lib.rs.html#88){.src
.rightside}[§](#method.equivalent-1){.anchor}

#### fn [equivalent](../../equivalent/trait.Equivalent.html#tymethod.equivalent){.fn}(&self, key: [&K](https://doc.rust-lang.org/1.91.1/std/primitive.reference.html){.primitive}) -\> [bool](https://doc.rust-lang.org/1.91.1/std/primitive.bool.html){.primitive} {#fn-equivalentself-key-k---bool-1 .code-header}
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
::: {#method.from-1 .section .method .trait-impl}
[Source](https://doc.rust-lang.org/1.91.1/src/core/convert/mod.rs.html#788){.src
.rightside}[§](#method.from-1){.anchor}

#### fn [from](https://doc.rust-lang.org/1.91.1/core/convert/trait.From.html#tymethod.from){.fn}(t: T) -\> T {#fn-fromt-t---t .code-header}
:::

::: docblock
Returns the argument unchanged.
:::
:::::

::: {#impl-Instrument-for-T .section .impl}
[Source](../../src/tracing/instrument.rs.html#325){.src
.rightside}[§](#impl-Instrument-for-T){.anchor}

### impl\<T\> [Instrument](../../tracing/instrument/trait.Instrument.html "trait tracing::instrument::Instrument"){.trait} for T {#implt-instrument-for-t .code-header}
:::

::::::: impl-items
::: {#method.instrument .section .method .trait-impl}
[Source](../../src/tracing/instrument.rs.html#86){.src
.rightside}[§](#method.instrument){.anchor}

#### fn [instrument](../../tracing/instrument/trait.Instrument.html#method.instrument){.fn}(self, span: [Span](../../tracing/span/struct.Span.html "struct tracing::span::Span"){.struct}) -\> [Instrumented](../../tracing/instrument/struct.Instrumented.html "struct tracing::instrument::Instrumented"){.struct}\<Self\> {#fn-instrumentself-span-span---instrumentedself .code-header}
:::

::: docblock
Instruments this type with the provided
[`Span`](../../tracing/span/struct.Span.html "struct tracing::span::Span"),
returning an `Instrumented` wrapper. [Read
more](../../tracing/instrument/trait.Instrument.html#method.instrument)
:::

::: {#method.in_current_span .section .method .trait-impl}
[Source](../../src/tracing/instrument.rs.html#128){.src
.rightside}[§](#method.in_current_span){.anchor}

#### fn [in_current_span](../../tracing/instrument/trait.Instrument.html#method.in_current_span){.fn}(self) -\> [Instrumented](../../tracing/instrument/struct.Instrumented.html "struct tracing::instrument::Instrumented"){.struct}\<Self\> {#fn-in_current_spanself---instrumentedself .code-header}
:::

::: docblock
Instruments this type with the
[current](../../tracing/span/struct.Span.html#method.current "associated function tracing::span::Span::current")
[`Span`](../../tracing/span/struct.Span.html "struct tracing::span::Span"),
returning an `Instrumented` wrapper. [Read
more](../../tracing/instrument/trait.Instrument.html#method.in_current_span)
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
[Source](../../src/either/into_either.rs.html#64){.src
.rightside}[§](#impl-IntoEither-for-T){.anchor}

### impl\<T\> [IntoEither](../../either/into_either/trait.IntoEither.html "trait either::into_either::IntoEither"){.trait} for T {#implt-intoeither-for-t .code-header}
:::

:::::::: impl-items
::: {#method.into_either .section .method .trait-impl}
[Source](../../src/either/into_either.rs.html#29){.src
.rightside}[§](#method.into_either){.anchor}

#### fn [into_either](../../either/into_either/trait.IntoEither.html#method.into_either){.fn}(self, into_left: [bool](https://doc.rust-lang.org/1.91.1/std/primitive.bool.html){.primitive}) -\> [Either](../../either/enum.Either.html "enum either::Either"){.enum}\<Self, Self\> {#fn-into_eitherself-into_left-bool---eitherself-self .code-header}
:::

::: docblock
Converts `self` into a
[`Left`](../../either/enum.Either.html#variant.Left "variant either::Either::Left")
variant of
[`Either<Self, Self>`](../../either/enum.Either.html "enum either::Either")
if `into_left` is `true`. Converts `self` into a
[`Right`](../../either/enum.Either.html#variant.Right "variant either::Either::Right")
variant of
[`Either<Self, Self>`](../../either/enum.Either.html "enum either::Either")
otherwise. [Read
more](../../either/into_either/trait.IntoEither.html#method.into_either)
:::

:::: {#method.into_either_with .section .method .trait-impl}
[Source](../../src/either/into_either.rs.html#55-57){.src
.rightside}[§](#method.into_either_with){.anchor}

#### fn [into_either_with](../../either/into_either/trait.IntoEither.html#method.into_either_with){.fn}\<F\>(self, into_left: F) -\> [Either](../../either/enum.Either.html "enum either::Either"){.enum}\<Self, Self\> {#fn-into_either_withfself-into_left-f---eitherself-self .code-header}

::: where
where F:
[FnOnce](https://doc.rust-lang.org/1.91.1/core/ops/function/trait.FnOnce.html "trait core::ops::function::FnOnce"){.trait}(&Self)
-\>
[bool](https://doc.rust-lang.org/1.91.1/std/primitive.bool.html){.primitive},
:::
::::

::: docblock
Converts `self` into a
[`Left`](../../either/enum.Either.html#variant.Left "variant either::Either::Left")
variant of
[`Either<Self, Self>`](../../either/enum.Either.html "enum either::Either")
if `into_left(&self)` returns `true`. Converts `self` into a
[`Right`](../../either/enum.Either.html#variant.Right "variant either::Either::Right")
variant of
[`Either<Self, Self>`](../../either/enum.Either.html "enum either::Either")
otherwise. [Read
more](../../either/into_either/trait.IntoEither.html#method.into_either_with)
:::
::::::::

:::: {#impl-PartialSchema-for-T .section .impl}
[Source](../../src/utoipa/lib.rs.html#1375){.src
.rightside}[§](#impl-PartialSchema-for-T){.anchor}

### impl\<T\> [PartialSchema](../../utoipa/trait.PartialSchema.html "trait utoipa::PartialSchema"){.trait} for T {#implt-partialschema-for-t .code-header}

::: where
where T: ComposeSchema +
?[Sized](https://doc.rust-lang.org/1.91.1/core/marker/trait.Sized.html "trait core::marker::Sized"){.trait},
:::
::::

::::: impl-items
::: {#method.schema .section .method .trait-impl}
[Source](../../src/utoipa/lib.rs.html#1376){.src
.rightside}[§](#method.schema){.anchor}

#### fn [schema](../../utoipa/trait.PartialSchema.html#tymethod.schema){.fn}() -\> [RefOr](../../utoipa/openapi/enum.RefOr.html "enum utoipa::openapi::RefOr"){.enum}\<[Schema](../../utoipa/openapi/schema/enum.Schema.html "enum utoipa::openapi::schema::Schema"){.enum}\> {#fn-schema---reforschema .code-header}
:::

::: docblock
Return ref or schema of implementing type that can then be used to
construct combined schemas.
:::
:::::

::: {#impl-Pointable-for-T .section .impl}
[Source](../../src/crossbeam_epoch/atomic.rs.html#194){.src
.rightside}[§](#impl-Pointable-for-T){.anchor}

### impl\<T\> [Pointable](../../crossbeam_epoch/atomic/trait.Pointable.html "trait crossbeam_epoch::atomic::Pointable"){.trait} for T {#implt-pointable-for-t .code-header}
:::

::::::::::::::: impl-items
::: {#associatedconstant.ALIGN .section .associatedconstant .trait-impl}
[Source](../../src/crossbeam_epoch/atomic.rs.html#195){.src
.rightside}[§](#associatedconstant.ALIGN){.anchor}

#### const [ALIGN](../../crossbeam_epoch/atomic/trait.Pointable.html#associatedconstant.ALIGN){.constant}: [usize](https://doc.rust-lang.org/1.91.1/std/primitive.usize.html){.primitive} {#const-align-usize .code-header}
:::

::: docblock
The alignment of pointer.
:::

::: {#associatedtype.Init .section .associatedtype .trait-impl}
[Source](../../src/crossbeam_epoch/atomic.rs.html#197){.src
.rightside}[§](#associatedtype.Init){.anchor}

#### type [Init](../../crossbeam_epoch/atomic/trait.Pointable.html#associatedtype.Init){.associatedtype} = T {#type-init-t .code-header}
:::

::: docblock
The type for initializers.
:::

::: {#method.init .section .method .trait-impl}
[Source](../../src/crossbeam_epoch/atomic.rs.html#199){.src
.rightside}[§](#method.init){.anchor}

#### unsafe fn [init](../../crossbeam_epoch/atomic/trait.Pointable.html#tymethod.init){.fn}(init: \<T as [Pointable](../../crossbeam_epoch/atomic/trait.Pointable.html "trait crossbeam_epoch::atomic::Pointable"){.trait}\>::[Init](../../crossbeam_epoch/atomic/trait.Pointable.html#associatedtype.Init "type crossbeam_epoch::atomic::Pointable::Init"){.associatedtype}) -\> [usize](https://doc.rust-lang.org/1.91.1/std/primitive.usize.html){.primitive} {#unsafe-fn-initinit-t-as-pointableinit---usize .code-header}
:::

::: docblock
Initializes a with the given initializer. [Read
more](../../crossbeam_epoch/atomic/trait.Pointable.html#tymethod.init)
:::

::: {#method.deref .section .method .trait-impl}
[Source](../../src/crossbeam_epoch/atomic.rs.html#203){.src
.rightside}[§](#method.deref){.anchor}

#### unsafe fn [deref](../../crossbeam_epoch/atomic/trait.Pointable.html#tymethod.deref){.fn}\<\'a\>(ptr: [usize](https://doc.rust-lang.org/1.91.1/std/primitive.usize.html){.primitive}) -\> [&\'a T](https://doc.rust-lang.org/1.91.1/std/primitive.reference.html){.primitive} {#unsafe-fn-derefaptr-usize---a-t .code-header}
:::

::: docblock
Dereferences the given pointer. [Read
more](../../crossbeam_epoch/atomic/trait.Pointable.html#tymethod.deref)
:::

::: {#method.deref_mut .section .method .trait-impl}
[Source](../../src/crossbeam_epoch/atomic.rs.html#207){.src
.rightside}[§](#method.deref_mut){.anchor}

#### unsafe fn [deref_mut](../../crossbeam_epoch/atomic/trait.Pointable.html#tymethod.deref_mut){.fn}\<\'a\>(ptr: [usize](https://doc.rust-lang.org/1.91.1/std/primitive.usize.html){.primitive}) -\> [&\'a mut T](https://doc.rust-lang.org/1.91.1/std/primitive.reference.html){.primitive} {#unsafe-fn-deref_mutaptr-usize---a-mut-t .code-header}
:::

::: docblock
Mutably dereferences the given pointer. [Read
more](../../crossbeam_epoch/atomic/trait.Pointable.html#tymethod.deref_mut)
:::

::: {#method.drop .section .method .trait-impl}
[Source](../../src/crossbeam_epoch/atomic.rs.html#211){.src
.rightside}[§](#method.drop){.anchor}

#### unsafe fn [drop](../../crossbeam_epoch/atomic/trait.Pointable.html#tymethod.drop){.fn}(ptr: [usize](https://doc.rust-lang.org/1.91.1/std/primitive.usize.html){.primitive}) {#unsafe-fn-dropptr-usize .code-header}
:::

::: docblock
Drops the object pointed to by the given pointer. [Read
more](../../crossbeam_epoch/atomic/trait.Pointable.html#tymethod.drop)
:::
:::::::::::::::

::: {#impl-Same-for-T .section .impl}
[Source](../../src/typenum/type_operators.rs.html#34){.src
.rightside}[§](#impl-Same-for-T){.anchor}

### impl\<T\> [Same](../../typenum/type_operators/trait.Same.html "trait typenum::type_operators::Same"){.trait} for T {#implt-same-for-t .code-header}
:::

::::: impl-items
::: {#associatedtype.Output .section .associatedtype .trait-impl}
[Source](../../src/typenum/type_operators.rs.html#35){.src
.rightside}[§](#associatedtype.Output){.anchor}

#### type [Output](../../typenum/type_operators/trait.Same.html#associatedtype.Output){.associatedtype} = T {#type-output-t .code-header}
:::

::: docblock
Should always be `Self`
:::
:::::

:::: {#impl-SupersetOf%3CSS%3E-for-SP .section .impl}
[Source](../../src/simba/scalar/subset.rs.html#90){.src
.rightside}[§](#impl-SupersetOf%3CSS%3E-for-SP){.anchor}

### impl\<SS, SP\> [SupersetOf](../../simba/scalar/subset/trait.SupersetOf.html "trait simba::scalar::subset::SupersetOf"){.trait}\<SS\> for SP {#implss-sp-supersetofss-for-sp .code-header}

::: where
where SS:
[SubsetOf](../../simba/scalar/subset/trait.SubsetOf.html "trait simba::scalar::subset::SubsetOf"){.trait}\<SP\>,
:::
::::

::::::::::: impl-items
::: {#method.to_subset .section .method .trait-impl}
[Source](../../src/simba/scalar/subset.rs.html#92){.src
.rightside}[§](#method.to_subset){.anchor}

#### fn [to_subset](../../simba/scalar/subset/trait.SupersetOf.html#method.to_subset){.fn}(&self) -\> [Option](https://doc.rust-lang.org/1.91.1/core/option/enum.Option.html "enum core::option::Option"){.enum}\<SS\> {#fn-to_subsetself---optionss .code-header}
:::

::: docblock
The inverse inclusion map: attempts to construct `self` from the
equivalent element of its superset. [Read
more](../../simba/scalar/subset/trait.SupersetOf.html#method.to_subset)
:::

::: {#method.is_in_subset .section .method .trait-impl}
[Source](../../src/simba/scalar/subset.rs.html#97){.src
.rightside}[§](#method.is_in_subset){.anchor}

#### fn [is_in_subset](../../simba/scalar/subset/trait.SupersetOf.html#tymethod.is_in_subset){.fn}(&self) -\> [bool](https://doc.rust-lang.org/1.91.1/std/primitive.bool.html){.primitive} {#fn-is_in_subsetself---bool .code-header}
:::

::: docblock
Checks if `self` is actually part of its subset `T` (and can be
converted to it).
:::

::: {#method.to_subset_unchecked .section .method .trait-impl}
[Source](../../src/simba/scalar/subset.rs.html#102){.src
.rightside}[§](#method.to_subset_unchecked){.anchor}

#### fn [to_subset_unchecked](../../simba/scalar/subset/trait.SupersetOf.html#tymethod.to_subset_unchecked){.fn}(&self) -\> SS {#fn-to_subset_uncheckedself---ss .code-header}
:::

::: docblock
Use with care! Same as `self.to_subset` but without any property checks.
Always succeeds.
:::

::: {#method.from_subset .section .method .trait-impl}
[Source](../../src/simba/scalar/subset.rs.html#107){.src
.rightside}[§](#method.from_subset){.anchor}

#### fn [from_subset](../../simba/scalar/subset/trait.SupersetOf.html#tymethod.from_subset){.fn}(element: [&SS](https://doc.rust-lang.org/1.91.1/std/primitive.reference.html){.primitive}) -\> SP {#fn-from_subsetelement-ss---sp .code-header}
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
[Source](../../src/ppv_lite86/types.rs.html#221-223){.src
.rightside}[§](#impl-VZip%3CV%3E-for-T){.anchor}

### impl\<V, T\> [VZip](../../ppv_lite86/types/trait.VZip.html "trait ppv_lite86::types::VZip"){.trait}\<V\> for T {#implv-t-vzipv-for-t .code-header}

::: where
where V:
[MultiLane](../../ppv_lite86/types/trait.MultiLane.html "trait ppv_lite86::types::MultiLane"){.trait}\<T\>,
:::
::::

:::: impl-items
::: {#method.vzip .section .method .trait-impl}
[Source](../../src/ppv_lite86/types.rs.html#226){.src
.rightside}[§](#method.vzip){.anchor}

#### fn [vzip](../../ppv_lite86/types/trait.VZip.html#tymethod.vzip){.fn}(self) -\> V {#fn-vzipself---v .code-header}
:::
::::

::: {#impl-WithSubscriber-for-T .section .impl}
[Source](../../src/tracing/instrument.rs.html#393){.src
.rightside}[§](#impl-WithSubscriber-for-T){.anchor}

### impl\<T\> [WithSubscriber](../../tracing/instrument/trait.WithSubscriber.html "trait tracing::instrument::WithSubscriber"){.trait} for T {#implt-withsubscriber-for-t .code-header}
:::

:::::::: impl-items
:::: {#method.with_subscriber .section .method .trait-impl}
[Source](../../src/tracing/instrument.rs.html#176-178){.src
.rightside}[§](#method.with_subscriber){.anchor}

#### fn [with_subscriber](../../tracing/instrument/trait.WithSubscriber.html#method.with_subscriber){.fn}\<S\>(self, subscriber: S) -\> [WithDispatch](../../tracing/instrument/struct.WithDispatch.html "struct tracing::instrument::WithDispatch"){.struct}\<Self\> {#fn-with_subscribersself-subscriber-s---withdispatchself .code-header}

::: where
where S:
[Into](https://doc.rust-lang.org/1.91.1/core/convert/trait.Into.html "trait core::convert::Into"){.trait}\<[Dispatch](../../tracing_core/dispatcher/struct.Dispatch.html "struct tracing_core::dispatcher::Dispatch"){.struct}\>,
:::
::::

::: docblock
Attaches the provided
[`Subscriber`](../../tracing_core/subscriber/trait.Subscriber.html "trait tracing_core::subscriber::Subscriber")
to this type, returning a
[`WithDispatch`](../../tracing/instrument/struct.WithDispatch.html "struct tracing::instrument::WithDispatch")
wrapper. [Read
more](../../tracing/instrument/trait.WithSubscriber.html#method.with_subscriber)
:::

::: {#method.with_current_subscriber .section .method .trait-impl}
[Source](../../src/tracing/instrument.rs.html#228){.src
.rightside}[§](#method.with_current_subscriber){.anchor}

#### fn [with_current_subscriber](../../tracing/instrument/trait.WithSubscriber.html#method.with_current_subscriber){.fn}(self) -\> [WithDispatch](../../tracing/instrument/struct.WithDispatch.html "struct tracing::instrument::WithDispatch"){.struct}\<Self\> {#fn-with_current_subscriberself---withdispatchself .code-header}
:::

::: docblock
Attaches the current
[default](../../tracing/dispatcher/index.html#setting-the-default-subscriber "mod tracing::dispatcher")
[`Subscriber`](../../tracing_core/subscriber/trait.Subscriber.html "trait tracing_core::subscriber::Subscriber")
to this type, returning a
[`WithDispatch`](../../tracing/instrument/struct.WithDispatch.html "struct tracing::instrument::WithDispatch")
wrapper. [Read
more](../../tracing/instrument/trait.WithSubscriber.html#method.with_current_subscriber)
:::
::::::::

:::: {#impl-DeserializeOwned-for-T .section .impl}
[Source](../../src/serde_core/de/mod.rs.html#633){.src
.rightside}[§](#impl-DeserializeOwned-for-T){.anchor}

### impl\<T\> [DeserializeOwned](../../serde_core/de/trait.DeserializeOwned.html "trait serde_core::de::DeserializeOwned"){.trait} for T {#implt-deserializeowned-for-t .code-header}

::: where
where T: for\<\'de\>
[Deserialize](../../serde_core/de/trait.Deserialize.html "trait serde_core::de::Deserialize"){.trait}\<\'de\>,
:::
::::

:::: {#impl-Scalar-for-T .section .impl}
[Source](../../src/nalgebra/base/scalar.rs.html#8){.src
.rightside}[§](#impl-Scalar-for-T){.anchor}

### impl\<T\> [Scalar](../../nalgebra/base/scalar/trait.Scalar.html "trait nalgebra::base::scalar::Scalar"){.trait} for T {#implt-scalar-for-t .code-header}

::: where
where T: \'static +
[Clone](https://doc.rust-lang.org/1.91.1/core/clone/trait.Clone.html "trait core::clone::Clone"){.trait} +
[PartialEq](https://doc.rust-lang.org/1.91.1/core/cmp/trait.PartialEq.html "trait core::cmp::PartialEq"){.trait} +
[Debug](https://doc.rust-lang.org/1.91.1/core/fmt/trait.Debug.html "trait core::fmt::Debug"){.trait},
:::
::::
::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
:::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
