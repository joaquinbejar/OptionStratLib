:::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::: width-limiter
::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../../index.html)::[model](../index.html)::[option](index.html)
:::

# Struct [Options]{.struct} Copy item path

[[Source](../../../src/optionstratlib/model/option.rs.html#67-110){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub struct Options {
    pub option_type: OptionType,
    pub side: Side,
    pub underlying_symbol: String,
    pub strike_price: Positive,
    pub expiration_date: ExpirationDate,
    pub implied_volatility: Positive,
    pub quantity: Positive,
    pub underlying_price: Positive,
    pub risk_free_rate: Decimal,
    pub option_style: OptionStyle,
    pub dividend_yield: Positive,
    pub exotic_params: Option<ExoticParams>,
}
```

Expand description

::: docblock
Represents a financial option contract with its essential parameters and
characteristics.

This structure contains all the necessary information to define an
options contract, including its type (call/put), market position
(long/short), pricing parameters, and contract specifications. It serves
as the core data model for option pricing, risk analysis, and strategy
development.

The `Options` struct supports both standard option types and exotic
options through the optional `exotic_params` field, making it versatile
for various financial modeling scenarios.
:::

## Fields[§](#fields){.anchor} {#fields .fields .section-header}

[[§](#structfield.option_type){.anchor
.field}`option_type: `[`OptionType`](../types/enum.OptionType.html "enum optionstratlib::model::types::OptionType"){.enum}]{#structfield.option_type
.structfield .section-header}

::: docblock
Specifies whether this is a European or American option
:::

[[§](#structfield.side){.anchor
.field}`side: `[`Side`](../types/enum.Side.html "enum optionstratlib::model::types::Side"){.enum}]{#structfield.side
.structfield .section-header}

::: docblock
Indicates whether the position is Long (purchased) or Short
(sold/written), which determines the profit/loss direction and risk
profile.
:::

[[§](#structfield.underlying_symbol){.anchor
.field}`underlying_symbol: `[`String`](https://doc.rust-lang.org/1.91.1/alloc/string/struct.String.html "struct alloc::string::String"){.struct}]{#structfield.underlying_symbol
.structfield .section-header}

::: docblock
The ticker symbol or identifier of the underlying asset (e.g., "AAPL"
for Apple stock).
:::

[[§](#structfield.strike_price){.anchor
.field}`strike_price: `[`Positive`](../positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}]{#structfield.strike_price
.structfield .section-header}

::: docblock
The price at which the option holder can exercise their right to buy
(for calls) or sell (for puts) the underlying asset.
:::

[[§](#structfield.expiration_date){.anchor
.field}`expiration_date: `[`ExpirationDate`](../enum.ExpirationDate.html "enum optionstratlib::model::ExpirationDate"){.enum}]{#structfield.expiration_date
.structfield .section-header}

::: docblock
When the option contract expires, either as days from now or as a
specific date.
:::

[[§](#structfield.implied_volatility){.anchor
.field}`implied_volatility: `[`Positive`](../positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}]{#structfield.implied_volatility
.structfield .section-header}

::: docblock
The market's expectation for future volatility of the underlying asset,
a key parameter for option pricing models.
:::

[[§](#structfield.quantity){.anchor
.field}`quantity: `[`Positive`](../positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}]{#structfield.quantity
.structfield .section-header}

::: docblock
The number of contracts in this position.
:::

[[§](#structfield.underlying_price){.anchor
.field}`underlying_price: `[`Positive`](../positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}]{#structfield.underlying_price
.structfield .section-header}

::: docblock
The current market price of the underlying asset.
:::

[[§](#structfield.risk_free_rate){.anchor
.field}`risk_free_rate: `[`Decimal`](../../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}]{#structfield.risk_free_rate
.structfield .section-header}

::: docblock
The current risk-free interest rate used in option pricing models,
typically based on treasury yields of similar duration.
:::

[[§](#structfield.option_style){.anchor
.field}`option_style: `[`OptionStyle`](../types/enum.OptionStyle.html "enum optionstratlib::model::types::OptionStyle"){.enum}]{#structfield.option_style
.structfield .section-header}

::: docblock
The option is a Call or Put option, determining the fundamental right
option can be exercised.
:::

[[§](#structfield.dividend_yield){.anchor
.field}`dividend_yield: `[`Positive`](../positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}]{#structfield.dividend_yield
.structfield .section-header}

::: docblock
The annualized dividend yield of the underlying asset, affecting option
pricing particularly for longer-dated contracts.
:::

[[§](#structfield.exotic_params){.anchor
.field}`exotic_params: `[`Option`](https://doc.rust-lang.org/1.91.1/core/option/enum.Option.html "enum core::option::Option"){.enum}`<`[`ExoticParams`](struct.ExoticParams.html "struct optionstratlib::model::option::ExoticParams"){.struct}`>`]{#structfield.exotic_params
.structfield .section-header}

::: docblock
Additional parameters required for exotic option types like Asian or
Lookback options. This field is None for standard (vanilla) options.
:::

## Implementations[§](#implementations){.anchor} {#implementations .section-header}

:::::::::::::::::::::::::::::::::::: {#implementations-list}
::: {#impl-Options .section .impl}
[Source](../../../src/optionstratlib/model/option.rs.html#112-666){.src
.rightside}[§](#impl-Options){.anchor}

### impl [Options](struct.Options.html "struct optionstratlib::model::option::Options"){.struct} {#impl-options .code-header}
:::

:::::::::::::::::::::::::::::::::: impl-items
::: {#method.new .section .method}
[Source](../../../src/optionstratlib/model/option.rs.html#144-172){.src
.rightside}

#### pub fn [new](#method.new){.fn}( option_type: [OptionType](../types/enum.OptionType.html "enum optionstratlib::model::types::OptionType"){.enum}, side: [Side](../types/enum.Side.html "enum optionstratlib::model::types::Side"){.enum}, underlying_symbol: [String](https://doc.rust-lang.org/1.91.1/alloc/string/struct.String.html "struct alloc::string::String"){.struct}, strike_price: [Positive](../positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, expiration_date: [ExpirationDate](../enum.ExpirationDate.html "enum optionstratlib::model::ExpirationDate"){.enum}, implied_volatility: [Positive](../positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, quantity: [Positive](../positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, underlying_price: [Positive](../positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, risk_free_rate: [Decimal](../../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}, option_style: [OptionStyle](../types/enum.OptionStyle.html "enum optionstratlib::model::types::OptionStyle"){.enum}, dividend_yield: [Positive](../positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, exotic_params: [Option](https://doc.rust-lang.org/1.91.1/core/option/enum.Option.html "enum core::option::Option"){.enum}\<[ExoticParams](struct.ExoticParams.html "struct optionstratlib::model::option::ExoticParams"){.struct}\>, ) -\> Self {#pub-fn-new-option_type-optiontype-side-side-underlying_symbol-string-strike_price-positive-expiration_date-expirationdate-implied_volatility-positive-quantity-positive-underlying_price-positive-risk_free_rate-decimal-option_style-optionstyle-dividend_yield-positive-exotic_params-optionexoticparams---self .code-header}
:::

::: docblock
Creates a new options contract with the specified parameters.

This constructor creates an instance of `Options` with all the required
parameters for defining and pricing an option contract. It supports both
standard (vanilla) options and exotic options through the optional
`exotic_params` parameter.

##### [§](#parameters){.doc-anchor}Parameters

- `option_type` - Specifies whether this is a Call or Put option,
  determining the fundamental right the option contract provides.
- `side` - Indicates whether the position is Long (purchased) or Short
  (sold/written), which determines the profit/loss direction.
- `underlying_symbol` - The ticker symbol or identifier of the
  underlying asset (e.g., "AAPL").
- `strike_price` - The price at which the option can be exercised,
  represented as a `Positive` value.
- `expiration_date` - When the option contract expires, either as days
  from now or as a specific date.
- `implied_volatility` - The market's expectation for future volatility
  of the underlying asset, a key parameter for option pricing.
- `quantity` - The number of contracts in this position, represented as
  a `Positive` value.
- `underlying_price` - The current market price of the underlying asset.
- `risk_free_rate` - The current risk-free interest rate used in option
  pricing models.
- `option_style` - The option exercise style (European or American),
  determining when the option can be exercised.
- `dividend_yield` - The annualized dividend yield of the underlying
  asset, affecting option pricing.
- `exotic_params` - Additional parameters required for exotic option
  types. Set to `None` for standard (vanilla) options.

##### [§](#returns){.doc-anchor}Returns

A fully configured `Options` instance with all the specified parameters.
:::

::: {#method.time_to_expiration .section .method}
[Source](../../../src/optionstratlib/model/option.rs.html#201-203){.src
.rightside}

#### pub fn [time_to_expiration](#method.time_to_expiration){.fn}(&self) -\> [OptionsResult](../../error/type.OptionsResult.html "type optionstratlib::error::OptionsResult"){.type}\<[Positive](../positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\> {#pub-fn-time_to_expirationself---optionsresultpositive .code-header}
:::

::: docblock
Calculates the time to expiration of the option in years.

This function computes the time remaining until the option's expiration
date, expressed as a positive decimal value representing years. This is
a key parameter used in option pricing models.

##### [§](#returns-1){.doc-anchor}Returns

- `OptionsResult<Positive>` - A result containing the time to expiration
  in years as a Positive value, or an error if the calculation failed.
:::

::: {#method.is_long .section .method}
[Source](../../../src/optionstratlib/model/option.rs.html#214-216){.src
.rightside}

#### pub fn [is_long](#method.is_long){.fn}(&self) -\> [bool](https://doc.rust-lang.org/1.91.1/std/primitive.bool.html){.primitive} {#pub-fn-is_longself---bool .code-header}
:::

::: docblock
Determines if the option position is long (purchased).

A long position indicates that the option has been bought, meaning the
holder has the right to exercise the option according to its terms.

##### [§](#returns-2){.doc-anchor}Returns

- `bool` - Returns true if the option is held as a long position, false
  otherwise.
:::

::: {#method.is_short .section .method}
[Source](../../../src/optionstratlib/model/option.rs.html#227-229){.src
.rightside}

#### pub fn [is_short](#method.is_short){.fn}(&self) -\> [bool](https://doc.rust-lang.org/1.91.1/std/primitive.bool.html){.primitive} {#pub-fn-is_shortself---bool .code-header}
:::

::: docblock
Determines if the option position is short (written/sold).

A short position indicates that the option has been sold or written,
meaning the holder has the obligation to fulfill the contract terms if
the option is exercised.

##### [§](#returns-3){.doc-anchor}Returns

- `bool` - Returns true if the option is held as a short position, false
  otherwise.
:::

::: {#method.calculate_price_binomial .section .method}
[Source](../../../src/optionstratlib/model/option.rs.html#259-278){.src
.rightside}

#### pub fn [calculate_price_binomial](#method.calculate_price_binomial){.fn}( &self, no_steps: [usize](https://doc.rust-lang.org/1.91.1/std/primitive.usize.html){.primitive}, ) -\> [OptionsResult](../../error/type.OptionsResult.html "type optionstratlib::error::OptionsResult"){.type}\<[Decimal](../../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}\> {#pub-fn-calculate_price_binomial-self-no_steps-usize---optionsresultdecimal .code-header}
:::

::: docblock
Calculates the price of an option using the binomial tree model.

This method implements the binomial option pricing model which
constructs a discrete-time lattice (tree) of possible future underlying
asset prices to determine the option's value. The approach is
particularly valuable for pricing American options and other
early-exercise scenarios.

The calculation divides the time to expiration into a specified number
of steps, creating a binomial tree that represents possible price paths
of the underlying asset. The option's value is then calculated by
working backward from expiration to the present value.

##### [§](#parameters-1){.doc-anchor}Parameters

- `no_steps` - The number of steps to use in the binomial tree
  calculation. Higher values increase accuracy but also computational
  cost.

##### [§](#returns-4){.doc-anchor}Returns

- `OptionsResult<Decimal>` - A result containing the calculated option
  price as a Decimal value, or an OptionsError if the calculation
  failed.

##### [§](#errors){.doc-anchor}Errors

Returns an `OptionsError::OtherError` if:

- The number of steps is zero
- The time to expiration calculation fails
- The binomial price calculation fails
:::

::: {#method.calculate_price_binomial_tree .section .method}
[Source](../../../src/optionstratlib/model/option.rs.html#300-319){.src
.rightside}

#### pub fn [calculate_price_binomial_tree](#method.calculate_price_binomial_tree){.fn}( &self, no_steps: [usize](https://doc.rust-lang.org/1.91.1/std/primitive.usize.html){.primitive}, ) -\> [OptionsResult](../../error/type.OptionsResult.html "type optionstratlib::error::OptionsResult"){.type}\<([Decimal](../../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}, [Vec](https://doc.rust-lang.org/1.91.1/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<[Vec](https://doc.rust-lang.org/1.91.1/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<[Decimal](../../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}\>\>, [Vec](https://doc.rust-lang.org/1.91.1/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<[Vec](https://doc.rust-lang.org/1.91.1/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<[Decimal](../../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}\>\>)\> {#pub-fn-calculate_price_binomial_tree-self-no_steps-usize---optionsresultdecimal-vecvecdecimal-vecvecdecimal .code-header}
:::

::: docblock
Calculates option price using the binomial tree model.

This method implements a binomial tree (lattice) approach to option
pricing, which discretizes the underlying asset's price movement over
time. The model builds a tree of possible future asset prices and works
backwards to determine the current option value.

##### [§](#parameters-2){.doc-anchor}Parameters

- `no_steps` - The number of discrete time steps to use in the model.
  Higher values increase precision but also computational cost.

##### [§](#returns-5){.doc-anchor}Returns

- `PriceBinomialTree` - A result containing:
  - The calculated option price
  - The asset price tree (underlying price evolution)
  - The option value tree (option price at each node)

This method is particularly valuable for pricing American options and
other early-exercise scenarios that cannot be accurately priced using
closed-form solutions.
:::

::: {#method.calculate_price_black_scholes .section .method}
[Source](../../../src/optionstratlib/model/option.rs.html#334-336){.src
.rightside}

#### pub fn [calculate_price_black_scholes](#method.calculate_price_black_scholes){.fn}(&self) -\> [OptionsResult](../../error/type.OptionsResult.html "type optionstratlib::error::OptionsResult"){.type}\<[Decimal](../../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}\> {#pub-fn-calculate_price_black_scholesself---optionsresultdecimal .code-header}
:::

::: docblock
Calculates option price using the Black-Scholes model.

This method implements the Black-Scholes option pricing formula, which
provides a closed-form solution for European-style options. The model
assumes lognormal distribution of underlying asset prices and constant
volatility.

##### [§](#returns-6){.doc-anchor}Returns

- `OptionsResult<Decimal>` - A result containing the calculated option
  price as a Decimal value, or an error if the calculation failed.

This method is computationally efficient but limited to European options
without early exercise capabilities.
:::

::: {#method.calculate_price_montecarlo .section .method}
[Source](../../../src/optionstratlib/model/option.rs.html#356-358){.src
.rightside}

#### pub fn [calculate_price_montecarlo](#method.calculate_price_montecarlo){.fn}( &self, prices: &\[[Positive](../positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\], ) -\> [OptionsResult](../../error/type.OptionsResult.html "type optionstratlib::error::OptionsResult"){.type}\<[Positive](../positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\> {#pub-fn-calculate_price_montecarlo-self-prices-positive---optionsresultpositive .code-header}
:::

::: docblock
Calculates the price of an option using the Monte Carlo simulation
method.

##### [§](#arguments){.doc-anchor}Arguments

- `prices` - A slice of `Positive` values representing the prices used
  in the Monte Carlo simulation.

##### [§](#returns-7){.doc-anchor}Returns

- `OptionsResult<Positive>` - The calculated price of the option wrapped
  in an `OptionsResult`. This will return a valid `Positive` value if
  successful, or an error if the simulation fails.

##### [§](#errors-1){.doc-anchor}Errors

This function will return an error in the `OptionsResult` if the
internal Monte Carlo price computation fails during the execution of
`price_option_monte_carlo`.
:::

::: {#method.calculate_price_telegraph .section .method}
[Source](../../../src/optionstratlib/model/option.rs.html#375-377){.src
.rightside}

#### pub fn [calculate_price_telegraph](#method.calculate_price_telegraph){.fn}( &self, no_steps: [usize](https://doc.rust-lang.org/1.91.1/std/primitive.usize.html){.primitive}, ) -\> [OptionsResult](../../error/type.OptionsResult.html "type optionstratlib::error::OptionsResult"){.type}\<[Decimal](../../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}\> {#pub-fn-calculate_price_telegraph-self-no_steps-usize---optionsresultdecimal .code-header}
:::

::: docblock
Calculates option price using the Telegraph equation approach.

This method implements a finite-difference method based on the Telegraph
equation to price options. This approach can handle a variety of option
styles and types, including path-dependent options.

##### [§](#parameters-3){.doc-anchor}Parameters

- `no_steps` - The number of discrete time steps to use in the model.
  Higher values increase precision but also computational cost.

##### [§](#returns-8){.doc-anchor}Returns

- `Result<Decimal, PricingError>` - A result containing the calculated
  option price as a Decimal value, or a boxed error if the calculation
  failed.
:::

::: {#method.payoff .section .method}
[Source](../../../src/optionstratlib/model/option.rs.html#393-405){.src
.rightside}

#### pub fn [payoff](#method.payoff){.fn}(&self) -\> [OptionsResult](../../error/type.OptionsResult.html "type optionstratlib::error::OptionsResult"){.type}\<[Decimal](../../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}\> {#pub-fn-payoffself---optionsresultdecimal .code-header}
:::

::: docblock
Calculates the intrinsic value (payoff) of the option at the current
underlying price.

The payoff represents what the option would be worth if exercised
immediately, based on the current market conditions. For
out-of-the-money options, the payoff will be zero.

##### [§](#returns-9){.doc-anchor}Returns

- `OptionsResult<Decimal>` - A result containing the calculated payoff
  as a Decimal value, adjusted for the quantity of contracts held, or an
  error if the calculation failed.

This method is useful for determining the exercise value of an option
and for analyzing whether an option has intrinsic value.
:::

::: {#method.payoff_at_price .section .method}
[Source](../../../src/optionstratlib/model/option.rs.html#422-434){.src
.rightside}

#### pub fn [payoff_at_price](#method.payoff_at_price){.fn}(&self, price: &[Positive](../positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}) -\> [OptionsResult](../../error/type.OptionsResult.html "type optionstratlib::error::OptionsResult"){.type}\<[Decimal](../../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}\> {#pub-fn-payoff_at_priceself-price-positive---optionsresultdecimal .code-header}
:::

::: docblock
Calculates the financial payoff value of the option at a specific
underlying price.

This method determines the option's payoff based on its type, strike
price, style, and side (long/short) at the given underlying price. The
result represents the total profit or loss for the option position at
that price, adjusted by the position quantity.

##### [§](#parameters-4){.doc-anchor}Parameters

- `price` - A `Positive` value representing the hypothetical price of
  the underlying asset.

##### [§](#returns-10){.doc-anchor}Returns

- `OptionsResult<Decimal>` - The calculated payoff value as a `Decimal`,
  wrapped in a `Result` type. Returns an `Err` if the payoff calculation
  encounters an error.
:::

::: {#method.intrinsic_value .section .method}
[Source](../../../src/optionstratlib/model/option.rs.html#449-461){.src
.rightside}

#### pub fn [intrinsic_value](#method.intrinsic_value){.fn}( &self, underlying_price: [Positive](../positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, ) -\> [OptionsResult](../../error/type.OptionsResult.html "type optionstratlib::error::OptionsResult"){.type}\<[Decimal](../../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}\> {#pub-fn-intrinsic_value-self-underlying_price-positive---optionsresultdecimal .code-header}
:::

::: docblock
Calculates the intrinsic value of the option.

The intrinsic value is the difference between the underlying asset's
price and the option's strike price. For call options, the intrinsic
value is the maximum of zero and the difference between the underlying
price and the strike price. For put options, the intrinsic value is the
maximum of zero and the difference between the strike price and the
underlying price.

##### [§](#arguments-1){.doc-anchor}Arguments

- `underlying_price` - The current price of the underlying asset.

##### [§](#returns-11){.doc-anchor}Returns

- `OptionsResult<Decimal>` - The intrinsic value of the option, or an
  error if the calculation fails.
:::

::: {#method.is_in_the_money .section .method}
[Source](../../../src/optionstratlib/model/option.rs.html#473-478){.src
.rightside}

#### pub fn [is_in_the_money](#method.is_in_the_money){.fn}(&self) -\> [bool](https://doc.rust-lang.org/1.91.1/std/primitive.bool.html){.primitive} {#pub-fn-is_in_the_moneyself---bool .code-header}
:::

::: docblock
Determines whether an option is "in-the-money" based on its current
price relative to strike price.

An option is considered in-the-money when:

- For Call options: the underlying asset price is greater than or equal
  to the strike price
- For Put options: the underlying asset price is less than or equal to
  the strike price

This status is important for evaluating the option's current value and
potential profitability.

##### [§](#returns-12){.doc-anchor}Returns

`true` if the option is in-the-money, `false` otherwise
:::

::: {#method.time_value .section .method}
[Source](../../../src/optionstratlib/model/option.rs.html#492-496){.src
.rightside}

#### pub fn [time_value](#method.time_value){.fn}(&self) -\> [OptionsResult](../../error/type.OptionsResult.html "type optionstratlib::error::OptionsResult"){.type}\<[Decimal](../../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}\> {#pub-fn-time_valueself---optionsresultdecimal .code-header}
:::

::: docblock
Calculates the time value component of an option's price.

Time value represents the portion of an option's premium that exceeds
its intrinsic value. It reflects the market's expectation that the
option may become more valuable before expiration due to potential
favorable movements in the underlying asset price.

The calculation uses the Black-Scholes model to determine the total
option price, then subtracts the intrinsic value to find the time value
component.

##### [§](#returns-13){.doc-anchor}Returns

- `Ok(Decimal)` containing the time value (never negative, minimum value
  is zero)
- `Err` if the price calculation encounters an error
:::

::: {#method.calculate_implied_volatility .section .method}
[Source](../../../src/optionstratlib/model/option.rs.html#612-665){.src
.rightside}

#### pub fn [calculate_implied_volatility](#method.calculate_implied_volatility){.fn}( &self, market_price: [Decimal](../../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}, ) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Positive](../positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, [VolatilityError](../../error/enum.VolatilityError.html "enum optionstratlib::error::VolatilityError"){.enum}\> {#pub-fn-calculate_implied_volatility-self-market_price-decimal---resultpositive-volatilityerror .code-header}
:::

:::: docblock
**calculate_implied_volatility**:

This function estimates the implied volatility of an option based on its
market price using binary search. Implied volatility is a key metric in
options trading that reflects the market's view of the expected
volatility of the underlying asset.

###### [§](#parameters-5){.doc-anchor}Parameters:

- `market_price`: The market price of the option as a `Decimal`. This
  represents the cost at which the option is traded in the market.

###### [§](#returns-14){.doc-anchor}Returns:

- `Ok(Positive)`: A `Positive` value representing the calculated implied
  volatility as a percentage.
- `Err(ImpliedVolatilityError)`: An error indicating the reason
  calculation failed, such as:
  - No convergence within the maximum number of iterations.
  - Invalid option parameters.

###### [§](#implementation-details){.doc-anchor}Implementation Details:

- **Binary Search**: The function uses a binary search approach to
  iteratively find the implied volatility (`volatility`) that narrows
  the difference between the calculated option price (via Black-Scholes)
  and the target `market_price`.

- **Short Options Adjustment**: For short options, the market price is
  inverted (negated), and this adjustment ensures proper calculation of
  implied volatility.

- **Bounds and Iteration**: The method starts with a maximum bound
  (`5.0`, representing 500% volatility) and a lower bound (`0.0`). It
  adjusts these bounds based on whether the computed price is above or
  below the target and repeats until convergence or the maximum number
  of iterations is reached (`MAX_ITERATIONS_IV`).

- **Convergence Tolerance**: The function stops iterating when the
  computed price is within `IV_TOLERANCE` of the target market price or
  when the difference between the high and low bounds is smaller than a
  threshold (`0.0001`).

###### [§](#error-cases){.doc-anchor}Error Cases:

- **No Convergence**: If the binary search exhausts the allowed number
  of iterations (`MAX_ITERATIONS_IV`) without sufficiently narrowing
  down the implied volatility, the function returns an
  `ImpliedVolatilityError::NoConvergence`.
- **Invalid Parameters**: Bounds violations or invalid market inputs can
  potentially cause other errors during calculations.

###### [§](#example-usage){.doc-anchor}Example Usage:

::: example-wrap
``` {.rust .rust-example-rendered}
use rust_decimal_macros::dec;
use rust_decimal::Decimal;
use tracing::{error, info};
use optionstratlib::{pos, ExpirationDate, OptionStyle, OptionType, Options, Side};

let options = Options::new(
            OptionType::European,
            Side::Short,
            "TEST".to_string(),
            pos!(6050.0), // strike
            ExpirationDate::Days(pos!(60.0)),
            pos!(0.1),     // initial iv
            pos!(1.0),     // qty
            pos!(6032.18), // underlying
            dec!(0.0),     // rate
            OptionStyle::Call,
            pos!(0.0), // div
            None,
        ); // Configure your option parameters
let market_price = dec!(133.5);  

match options.calculate_implied_volatility(market_price) {
    Ok(volatility) => info!("Implied Volatility: {}", volatility.to_dec()),
    Err(e) => error!("Failed to calculate implied volatility: {:?}", e),
}
```
:::
::::
::::::::::::::::::::::::::::::::::
::::::::::::::::::::::::::::::::::::

## Trait Implementations[§](#trait-implementations){.anchor} {#trait-implementations .section-header}

::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::: {#trait-implementations-list}
::: {#impl-BasicAble-for-Options .section .impl}
[Source](../../../src/optionstratlib/model/option.rs.html#785-892){.src
.rightside}[§](#impl-BasicAble-for-Options){.anchor}

### impl [BasicAble](../../strategies/base/trait.BasicAble.html "trait optionstratlib::strategies::base::BasicAble"){.trait} for [Options](struct.Options.html "struct optionstratlib::model::option::Options"){.struct} {#impl-basicable-for-options .code-header}
:::

::::::::::::::::::::::::::::::::::::::::: impl-items
::: {#method.get_title .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/option.rs.html#786-795){.src
.rightside}[§](#method.get_title){.anchor}

#### fn [get_title](../../strategies/base/trait.BasicAble.html#method.get_title){.fn}(&self) -\> [String](https://doc.rust-lang.org/1.91.1/alloc/string/struct.String.html "struct alloc::string::String"){.struct} {#fn-get_titleself---string .code-header}
:::

::: docblock
Retrieves the title associated with the current instance of the
strategy. [Read
more](../../strategies/base/trait.BasicAble.html#method.get_title)
:::

::: {#method.get_option_basic_type .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/option.rs.html#796-805){.src
.rightside}[§](#method.get_option_basic_type){.anchor}

#### fn [get_option_basic_type](../../strategies/base/trait.BasicAble.html#method.get_option_basic_type){.fn}(&self) -\> [HashSet](https://doc.rust-lang.org/1.91.1/std/collections/hash/set/struct.HashSet.html "struct std::collections::hash::set::HashSet"){.struct}\<[OptionBasicType](../types/struct.OptionBasicType.html "struct optionstratlib::model::types::OptionBasicType"){.struct}\<\'\_\>\> {#fn-get_option_basic_typeself---hashsetoptionbasictype_ .code-header}
:::

::: docblock
Retrieves a `HashSet` of `OptionBasicType` values associated with the
current strategy. [Read
more](../../strategies/base/trait.BasicAble.html#method.get_option_basic_type)
:::

::: {#method.get_symbol .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/option.rs.html#806-808){.src
.rightside}[§](#method.get_symbol){.anchor}

#### fn [get_symbol](../../strategies/base/trait.BasicAble.html#method.get_symbol){.fn}(&self) -\> &[str](https://doc.rust-lang.org/1.91.1/std/primitive.str.html){.primitive} {#fn-get_symbolself---str .code-header}
:::

::: docblock
Retrieves the symbol associated with the current instance by delegating
the call to the `get_symbol` method of the `one_option` object. [Read
more](../../strategies/base/trait.BasicAble.html#method.get_symbol)
:::

::: {#method.get_strike .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/option.rs.html#809-815){.src
.rightside}[§](#method.get_strike){.anchor}

#### fn [get_strike](../../strategies/base/trait.BasicAble.html#method.get_strike){.fn}(&self) -\> [HashMap](https://doc.rust-lang.org/1.91.1/std/collections/hash/map/struct.HashMap.html "struct std::collections::hash::map::HashMap"){.struct}\<[OptionBasicType](../types/struct.OptionBasicType.html "struct optionstratlib::model::types::OptionBasicType"){.struct}\<\'\_\>, &[Positive](../positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\> {#fn-get_strikeself---hashmapoptionbasictype_-positive .code-header}
:::

::: docblock
Retrieves a mapping of option basic types to their associated positive
strike values. [Read
more](../../strategies/base/trait.BasicAble.html#method.get_strike)
:::

::: {#method.get_side .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/option.rs.html#816-822){.src
.rightside}[§](#method.get_side){.anchor}

#### fn [get_side](../../strategies/base/trait.BasicAble.html#method.get_side){.fn}(&self) -\> [HashMap](https://doc.rust-lang.org/1.91.1/std/collections/hash/map/struct.HashMap.html "struct std::collections::hash::map::HashMap"){.struct}\<[OptionBasicType](../types/struct.OptionBasicType.html "struct optionstratlib::model::types::OptionBasicType"){.struct}\<\'\_\>, &[Side](../types/enum.Side.html "enum optionstratlib::model::types::Side"){.enum}\> {#fn-get_sideself---hashmapoptionbasictype_-side .code-header}
:::

::: docblock
Retrieves a `HashMap` that maps each `OptionBasicType` to its
corresponding `Side`. [Read
more](../../strategies/base/trait.BasicAble.html#method.get_side)
:::

::: {#method.get_type .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/option.rs.html#823-825){.src
.rightside}[§](#method.get_type){.anchor}

#### fn [get_type](../../strategies/base/trait.BasicAble.html#method.get_type){.fn}(&self) -\> &[OptionType](../types/enum.OptionType.html "enum optionstratlib::model::types::OptionType"){.enum} {#fn-get_typeself---optiontype .code-header}
:::

::: docblock
Retrieves the type of the option. [Read
more](../../strategies/base/trait.BasicAble.html#method.get_type)
:::

::: {#method.get_style .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/option.rs.html#826-832){.src
.rightside}[§](#method.get_style){.anchor}

#### fn [get_style](../../strategies/base/trait.BasicAble.html#method.get_style){.fn}(&self) -\> [HashMap](https://doc.rust-lang.org/1.91.1/std/collections/hash/map/struct.HashMap.html "struct std::collections::hash::map::HashMap"){.struct}\<[OptionBasicType](../types/struct.OptionBasicType.html "struct optionstratlib::model::types::OptionBasicType"){.struct}\<\'\_\>, &[OptionStyle](../types/enum.OptionStyle.html "enum optionstratlib::model::types::OptionStyle"){.enum}\> {#fn-get_styleself---hashmapoptionbasictype_-optionstyle .code-header}
:::

::: docblock
Retrieves a mapping of `OptionBasicType` to their corresponding
`OptionStyle`. [Read
more](../../strategies/base/trait.BasicAble.html#method.get_style)
:::

::: {#method.get_expiration .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/option.rs.html#833-839){.src
.rightside}[§](#method.get_expiration){.anchor}

#### fn [get_expiration](../../strategies/base/trait.BasicAble.html#method.get_expiration){.fn}(&self) -\> [HashMap](https://doc.rust-lang.org/1.91.1/std/collections/hash/map/struct.HashMap.html "struct std::collections::hash::map::HashMap"){.struct}\<[OptionBasicType](../types/struct.OptionBasicType.html "struct optionstratlib::model::types::OptionBasicType"){.struct}\<\'\_\>, &[ExpirationDate](../enum.ExpirationDate.html "enum optionstratlib::model::ExpirationDate"){.enum}\> {#fn-get_expirationself---hashmapoptionbasictype_-expirationdate .code-header}
:::

::: docblock
Retrieves a map of option basic types to their corresponding expiration
dates. [Read
more](../../strategies/base/trait.BasicAble.html#method.get_expiration)
:::

::: {#method.get_implied_volatility .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/option.rs.html#840-846){.src
.rightside}[§](#method.get_implied_volatility){.anchor}

#### fn [get_implied_volatility](../../strategies/base/trait.BasicAble.html#method.get_implied_volatility){.fn}(&self) -\> [HashMap](https://doc.rust-lang.org/1.91.1/std/collections/hash/map/struct.HashMap.html "struct std::collections::hash::map::HashMap"){.struct}\<[OptionBasicType](../types/struct.OptionBasicType.html "struct optionstratlib::model::types::OptionBasicType"){.struct}\<\'\_\>, &[Positive](../positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\> {#fn-get_implied_volatilityself---hashmapoptionbasictype_-positive .code-header}
:::

::: docblock
Retrieves the implied volatility for the current strategy. [Read
more](../../strategies/base/trait.BasicAble.html#method.get_implied_volatility)
:::

::: {#method.get_quantity .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/option.rs.html#847-853){.src
.rightside}[§](#method.get_quantity){.anchor}

#### fn [get_quantity](../../strategies/base/trait.BasicAble.html#method.get_quantity){.fn}(&self) -\> [HashMap](https://doc.rust-lang.org/1.91.1/std/collections/hash/map/struct.HashMap.html "struct std::collections::hash::map::HashMap"){.struct}\<[OptionBasicType](../types/struct.OptionBasicType.html "struct optionstratlib::model::types::OptionBasicType"){.struct}\<\'\_\>, &[Positive](../positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\> {#fn-get_quantityself---hashmapoptionbasictype_-positive .code-header}
:::

::: docblock
Retrieves the quantity information associated with the strategy. [Read
more](../../strategies/base/trait.BasicAble.html#method.get_quantity)
:::

::: {#method.get_underlying_price .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/option.rs.html#854-856){.src
.rightside}[§](#method.get_underlying_price){.anchor}

#### fn [get_underlying_price](../../strategies/base/trait.BasicAble.html#method.get_underlying_price){.fn}(&self) -\> &[Positive](../positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#fn-get_underlying_priceself---positive .code-header}
:::

::: docblock
Retrieves the underlying price of the financial instrument (e.g.,
option). [Read
more](../../strategies/base/trait.BasicAble.html#method.get_underlying_price)
:::

::: {#method.get_risk_free_rate .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/option.rs.html#857-863){.src
.rightside}[§](#method.get_risk_free_rate){.anchor}

#### fn [get_risk_free_rate](../../strategies/base/trait.BasicAble.html#method.get_risk_free_rate){.fn}(&self) -\> [HashMap](https://doc.rust-lang.org/1.91.1/std/collections/hash/map/struct.HashMap.html "struct std::collections::hash::map::HashMap"){.struct}\<[OptionBasicType](../types/struct.OptionBasicType.html "struct optionstratlib::model::types::OptionBasicType"){.struct}\<\'\_\>, &[Decimal](../../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}\> {#fn-get_risk_free_rateself---hashmapoptionbasictype_-decimal .code-header}
:::

::: docblock
Retrieves the risk-free interest rate associated with a given set of
options. [Read
more](../../strategies/base/trait.BasicAble.html#method.get_risk_free_rate)
:::

::: {#method.get_dividend_yield .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/option.rs.html#864-870){.src
.rightside}[§](#method.get_dividend_yield){.anchor}

#### fn [get_dividend_yield](../../strategies/base/trait.BasicAble.html#method.get_dividend_yield){.fn}(&self) -\> [HashMap](https://doc.rust-lang.org/1.91.1/std/collections/hash/map/struct.HashMap.html "struct std::collections::hash::map::HashMap"){.struct}\<[OptionBasicType](../types/struct.OptionBasicType.html "struct optionstratlib::model::types::OptionBasicType"){.struct}\<\'\_\>, &[Positive](../positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\> {#fn-get_dividend_yieldself---hashmapoptionbasictype_-positive .code-header}
:::

::: docblock
Retrieves the dividend yield of a financial option. [Read
more](../../strategies/base/trait.BasicAble.html#method.get_dividend_yield)
:::

::: {#method.one_option .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/option.rs.html#871-873){.src
.rightside}[§](#method.one_option){.anchor}

#### fn [one_option](../../strategies/base/trait.BasicAble.html#method.one_option){.fn}(&self) -\> &[Options](struct.Options.html "struct optionstratlib::model::option::Options"){.struct} {#fn-one_optionself---options .code-header}
:::

::: docblock
This method, `one_option`, is designed to retrieve a reference to an
`Options` object. However, in this implementation, the function is not
currently functional, as it explicitly triggers an unimplemented error
when called. [Read
more](../../strategies/base/trait.BasicAble.html#method.one_option)
:::

::: {#method.one_option_mut .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/option.rs.html#874-876){.src
.rightside}[§](#method.one_option_mut){.anchor}

#### fn [one_option_mut](../../strategies/base/trait.BasicAble.html#method.one_option_mut){.fn}(&mut self) -\> &mut [Options](struct.Options.html "struct optionstratlib::model::option::Options"){.struct} {#fn-one_option_mutmut-self---mut-options .code-header}
:::

::: docblock
Provides a mutable reference to an `Options` instance. [Read
more](../../strategies/base/trait.BasicAble.html#method.one_option_mut)
:::

::: {#method.set_implied_volatility .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/option.rs.html#877-880){.src
.rightside}[§](#method.set_implied_volatility){.anchor}

#### fn [set_implied_volatility](../../strategies/base/trait.BasicAble.html#method.set_implied_volatility){.fn}( &mut self, volatility: &[Positive](../positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, ) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[()](https://doc.rust-lang.org/1.91.1/std/primitive.unit.html){.primitive}, [StrategyError](../../error/strategies/enum.StrategyError.html "enum optionstratlib::error::strategies::StrategyError"){.enum}\> {#fn-set_implied_volatility-mut-self-volatility-positive---result-strategyerror .code-header}
:::

::: docblock
Updates the volatility for the strategy. [Read
more](../../strategies/base/trait.BasicAble.html#method.set_implied_volatility)
:::

::: {#method.set_underlying_price .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/option.rs.html#881-884){.src
.rightside}[§](#method.set_underlying_price){.anchor}

#### fn [set_underlying_price](../../strategies/base/trait.BasicAble.html#method.set_underlying_price){.fn}( &mut self, price: &[Positive](../positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, ) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[()](https://doc.rust-lang.org/1.91.1/std/primitive.unit.html){.primitive}, [StrategyError](../../error/strategies/enum.StrategyError.html "enum optionstratlib::error::strategies::StrategyError"){.enum}\> {#fn-set_underlying_price-mut-self-price-positive---result-strategyerror .code-header}
:::

::: docblock
Sets the underlying price for this strategy. [Read
more](../../strategies/base/trait.BasicAble.html#method.set_underlying_price)
:::

::: {#method.set_expiration_date .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/option.rs.html#885-891){.src
.rightside}[§](#method.set_expiration_date){.anchor}

#### fn [set_expiration_date](../../strategies/base/trait.BasicAble.html#method.set_expiration_date){.fn}( &mut self, expiration_date: [ExpirationDate](../enum.ExpirationDate.html "enum optionstratlib::model::ExpirationDate"){.enum}, ) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[()](https://doc.rust-lang.org/1.91.1/std/primitive.unit.html){.primitive}, [StrategyError](../../error/strategies/enum.StrategyError.html "enum optionstratlib::error::strategies::StrategyError"){.enum}\> {#fn-set_expiration_date-mut-self-expiration_date-expirationdate---result-strategyerror .code-header}
:::

::: docblock
Sets the expiration date for the strategy. [Read
more](../../strategies/base/trait.BasicAble.html#method.set_expiration_date)
:::

::: {#method.get_strikes .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/base.rs.html#448-453){.src
.rightside}[§](#method.get_strikes){.anchor}

#### fn [get_strikes](../../strategies/base/trait.BasicAble.html#method.get_strikes){.fn}(&self) -\> [Vec](https://doc.rust-lang.org/1.91.1/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<&[Positive](../positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\> {#fn-get_strikesself---vecpositive .code-header}
:::

::: docblock
Retrieves a vector of strike prices from the option types. [Read
more](../../strategies/base/trait.BasicAble.html#method.get_strikes)
:::
:::::::::::::::::::::::::::::::::::::::::

::: {#impl-Clone-for-Options .section .impl}
[Source](../../../src/optionstratlib/model/option.rs.html#66){.src
.rightside}[§](#impl-Clone-for-Options){.anchor}

### impl [Clone](https://doc.rust-lang.org/1.91.1/core/clone/trait.Clone.html "trait core::clone::Clone"){.trait} for [Options](struct.Options.html "struct optionstratlib::model::option::Options"){.struct} {#impl-clone-for-options .code-header}
:::

::::::: impl-items
::: {#method.clone .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/option.rs.html#66){.src
.rightside}[§](#method.clone){.anchor}

#### fn [clone](https://doc.rust-lang.org/1.91.1/core/clone/trait.Clone.html#tymethod.clone){.fn}(&self) -\> [Options](struct.Options.html "struct optionstratlib::model::option::Options"){.struct} {#fn-cloneself---options .code-header}
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

::: {#impl-ComposeSchema-for-Options .section .impl}
[Source](../../../src/optionstratlib/model/option.rs.html#66){.src
.rightside}[§](#impl-ComposeSchema-for-Options){.anchor}

### impl ComposeSchema for [Options](struct.Options.html "struct optionstratlib::model::option::Options"){.struct} {#impl-composeschema-for-options .code-header}
:::

:::: impl-items
::: {#method.compose .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/option.rs.html#66){.src
.rightside}[§](#method.compose){.anchor}

#### fn [compose](#tymethod.compose){.fn}(generics: [Vec](https://doc.rust-lang.org/1.91.1/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<[RefOr](../../../utoipa/openapi/enum.RefOr.html "enum utoipa::openapi::RefOr"){.enum}\<[Schema](../../../utoipa/openapi/schema/enum.Schema.html "enum utoipa::openapi::schema::Schema"){.enum}\>\>) -\> [RefOr](../../../utoipa/openapi/enum.RefOr.html "enum utoipa::openapi::RefOr"){.enum}\<[Schema](../../../utoipa/openapi/schema/enum.Schema.html "enum utoipa::openapi::schema::Schema"){.enum}\> {#fn-composegenerics-vecreforschema---reforschema .code-header}
:::
::::

::: {#impl-Debug-for-Options .section .impl}
[Source](../../../src/optionstratlib/model/format.rs.html#48-65){.src
.rightside}[§](#impl-Debug-for-Options){.anchor}

### impl [Debug](https://doc.rust-lang.org/1.91.1/core/fmt/trait.Debug.html "trait core::fmt::Debug"){.trait} for [Options](struct.Options.html "struct optionstratlib::model::option::Options"){.struct} {#impl-debug-for-options .code-header}
:::

::::: impl-items
::: {#method.fmt-1 .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/format.rs.html#49-64){.src
.rightside}[§](#method.fmt-1){.anchor}

#### fn [fmt](https://doc.rust-lang.org/1.91.1/core/fmt/trait.Debug.html#tymethod.fmt){.fn}(&self, f: &mut [Formatter](https://doc.rust-lang.org/1.91.1/core/fmt/struct.Formatter.html "struct core::fmt::Formatter"){.struct}\<\'\_\>) -\> [Result](https://doc.rust-lang.org/1.91.1/core/fmt/type.Result.html "type core::fmt::Result"){.type} {#fn-fmtself-f-mut-formatter_---result .code-header}
:::

::: docblock
Formats the value using the given formatter. [Read
more](https://doc.rust-lang.org/1.91.1/core/fmt/trait.Debug.html#tymethod.fmt)
:::
:::::

::: {#impl-Default-for-Options .section .impl}
[Source](../../../src/optionstratlib/model/option.rs.html#695-712){.src
.rightside}[§](#impl-Default-for-Options){.anchor}

### impl [Default](https://doc.rust-lang.org/1.91.1/core/default/trait.Default.html "trait core::default::Default"){.trait} for [Options](struct.Options.html "struct optionstratlib::model::option::Options"){.struct} {#impl-default-for-options .code-header}
:::

::::: impl-items
::: {#method.default .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/option.rs.html#696-711){.src
.rightside}[§](#method.default){.anchor}

#### fn [default](https://doc.rust-lang.org/1.91.1/core/default/trait.Default.html#tymethod.default){.fn}() -\> Self {#fn-default---self .code-header}
:::

::: docblock
Returns the "default value" for a type. [Read
more](https://doc.rust-lang.org/1.91.1/core/default/trait.Default.html#tymethod.default)
:::
:::::

::: {#impl-Deserialize%3C'de%3E-for-Options .section .impl}
[Source](../../../src/optionstratlib/model/option.rs.html#66){.src
.rightside}[§](#impl-Deserialize%3C'de%3E-for-Options){.anchor}

### impl\<\'de\> [Deserialize](../../../serde_core/de/trait.Deserialize.html "trait serde_core::de::Deserialize"){.trait}\<\'de\> for [Options](struct.Options.html "struct optionstratlib::model::option::Options"){.struct} {#implde-deserializede-for-options .code-header}
:::

:::::: impl-items
:::: {#method.deserialize .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/option.rs.html#66){.src
.rightside}[§](#method.deserialize){.anchor}

#### fn [deserialize](../../../serde_core/de/trait.Deserialize.html#tymethod.deserialize){.fn}\<\_\_D\>(\_\_deserializer: \_\_D) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<Self, \_\_D::[Error](../../../serde_core/de/trait.Deserializer.html#associatedtype.Error "type serde_core::de::Deserializer::Error"){.associatedtype}\> {#fn-deserialize__d__deserializer-__d---resultself-__derror .code-header}

::: where
where \_\_D:
[Deserializer](../../../serde_core/de/trait.Deserializer.html "trait serde_core::de::Deserializer"){.trait}\<\'de\>,
:::
::::

::: docblock
Deserialize this value from the given Serde deserializer. [Read
more](../../../serde_core/de/trait.Deserialize.html#tymethod.deserialize)
:::
::::::

::: {#impl-Display-for-Options .section .impl}
[Source](../../../src/optionstratlib/model/format.rs.html#15-46){.src
.rightside}[§](#impl-Display-for-Options){.anchor}

### impl [Display](https://doc.rust-lang.org/1.91.1/core/fmt/trait.Display.html "trait core::fmt::Display"){.trait} for [Options](struct.Options.html "struct optionstratlib::model::option::Options"){.struct} {#impl-display-for-options .code-header}
:::

::::: impl-items
::: {#method.fmt .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/format.rs.html#16-45){.src
.rightside}[§](#method.fmt){.anchor}

#### fn [fmt](https://doc.rust-lang.org/1.91.1/core/fmt/trait.Display.html#tymethod.fmt){.fn}(&self, f: &mut [Formatter](https://doc.rust-lang.org/1.91.1/core/fmt/struct.Formatter.html "struct core::fmt::Formatter"){.struct}\<\'\_\>) -\> [Result](https://doc.rust-lang.org/1.91.1/core/fmt/type.Result.html "type core::fmt::Result"){.type} {#fn-fmtself-f-mut-formatter_---result-1 .code-header}
:::

::: docblock
Formats the value using the given formatter. [Read
more](https://doc.rust-lang.org/1.91.1/core/fmt/trait.Display.html#tymethod.fmt)
:::
:::::

::: {#impl-From%3C%26OptionData%3E-for-Options .section .impl}
[Source](../../../src/optionstratlib/model/option.rs.html#668-693){.src
.rightside}[§](#impl-From%3C%26OptionData%3E-for-Options){.anchor}

### impl [From](https://doc.rust-lang.org/1.91.1/core/convert/trait.From.html "trait core::convert::From"){.trait}\<&[OptionData](../../chains/struct.OptionData.html "struct optionstratlib::chains::OptionData"){.struct}\> for [Options](struct.Options.html "struct optionstratlib::model::option::Options"){.struct} {#impl-fromoptiondata-for-options .code-header}
:::

::::: impl-items
::: {#method.from .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/option.rs.html#669-692){.src
.rightside}[§](#method.from){.anchor}

#### fn [from](https://doc.rust-lang.org/1.91.1/core/convert/trait.From.html#tymethod.from){.fn}(option_data: &[OptionData](../../chains/struct.OptionData.html "struct optionstratlib::chains::OptionData"){.struct}) -\> Self {#fn-fromoption_data-optiondata---self .code-header}
:::

::: docblock
Converts to this type from the input type.
:::
:::::

::: {#impl-Graph-for-Options .section .impl}
[Source](../../../src/optionstratlib/model/option.rs.html#894-962){.src
.rightside}[§](#impl-Graph-for-Options){.anchor}

### impl [Graph](../../visualization/trait.Graph.html "trait optionstratlib::visualization::Graph"){.trait} for [Options](struct.Options.html "struct optionstratlib::model::option::Options"){.struct} {#impl-graph-for-options .code-header}
:::

::::::: impl-items
::: {#method.graph_data .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/option.rs.html#895-944){.src
.rightside}[§](#method.graph_data){.anchor}

#### fn [graph_data](../../visualization/trait.Graph.html#tymethod.graph_data){.fn}(&self) -\> [GraphData](../../visualization/enum.GraphData.html "enum optionstratlib::visualization::GraphData"){.enum} {#fn-graph_dataself---graphdata .code-header}
:::

::: docblock
Return the raw data ready for plotting.
:::

::: {#method.graph_config .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/option.rs.html#946-961){.src
.rightside}[§](#method.graph_config){.anchor}

#### fn [graph_config](../../visualization/trait.Graph.html#method.graph_config){.fn}(&self) -\> [GraphConfig](../../visualization/struct.GraphConfig.html "struct optionstratlib::visualization::GraphConfig"){.struct} {#fn-graph_configself---graphconfig .code-header}
:::

::: docblock
Optional per‑object configuration overrides.
:::
:::::::

::: {#impl-Greeks-for-Options .section .impl}
[Source](../../../src/optionstratlib/model/option.rs.html#714-718){.src
.rightside}[§](#impl-Greeks-for-Options){.anchor}

### impl [Greeks](../../greeks/trait.Greeks.html "trait optionstratlib::greeks::Greeks"){.trait} for [Options](struct.Options.html "struct optionstratlib::model::option::Options"){.struct} {#impl-greeks-for-options .code-header}
:::

::::::::::::::::::::: impl-items
::: {#method.get_options .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/option.rs.html#715-717){.src
.rightside}[§](#method.get_options){.anchor}

#### fn [get_options](../../greeks/trait.Greeks.html#tymethod.get_options){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Vec](https://doc.rust-lang.org/1.91.1/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<&[Options](struct.Options.html "struct optionstratlib::model::option::Options"){.struct}\>, [GreeksError](../../error/greeks/enum.GreeksError.html "enum optionstratlib::error::greeks::GreeksError"){.enum}\> {#fn-get_optionsself---resultvecoptions-greekserror .code-header}
:::

::: docblock
Returns a vector of references to the option contracts for which Greeks
will be calculated. [Read
more](../../greeks/trait.Greeks.html#tymethod.get_options)
:::

::: {#method.greeks .section .method .trait-impl}
[Source](../../../src/optionstratlib/greeks/equations.rs.html#127-144){.src
.rightside}[§](#method.greeks){.anchor}

#### fn [greeks](../../greeks/trait.Greeks.html#method.greeks){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Greek](../../greeks/struct.Greek.html "struct optionstratlib::greeks::Greek"){.struct}, [GreeksError](../../error/greeks/enum.GreeksError.html "enum optionstratlib::error::greeks::GreeksError"){.enum}\> {#fn-greeksself---resultgreek-greekserror .code-header}
:::

::: docblock
Calculates and returns all Greeks as a single `Greek` struct. [Read
more](../../greeks/trait.Greeks.html#method.greeks)
:::

::: {#method.delta .section .method .trait-impl}
[Source](../../../src/optionstratlib/greeks/equations.rs.html#154-161){.src
.rightside}[§](#method.delta){.anchor}

#### fn [delta](../../greeks/trait.Greeks.html#method.delta){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Decimal](../../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}, [GreeksError](../../error/greeks/enum.GreeksError.html "enum optionstratlib::error::greeks::GreeksError"){.enum}\> {#fn-deltaself---resultdecimal-greekserror .code-header}
:::

::: docblock
Calculates the aggregate delta value for all options. [Read
more](../../greeks/trait.Greeks.html#method.delta)
:::

::: {#method.gamma .section .method .trait-impl}
[Source](../../../src/optionstratlib/greeks/equations.rs.html#171-178){.src
.rightside}[§](#method.gamma){.anchor}

#### fn [gamma](../../greeks/trait.Greeks.html#method.gamma){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Decimal](../../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}, [GreeksError](../../error/greeks/enum.GreeksError.html "enum optionstratlib::error::greeks::GreeksError"){.enum}\> {#fn-gammaself---resultdecimal-greekserror .code-header}
:::

::: docblock
Calculates the aggregate gamma value for all options. [Read
more](../../greeks/trait.Greeks.html#method.gamma)
:::

::: {#method.theta .section .method .trait-impl}
[Source](../../../src/optionstratlib/greeks/equations.rs.html#188-195){.src
.rightside}[§](#method.theta){.anchor}

#### fn [theta](../../greeks/trait.Greeks.html#method.theta){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Decimal](../../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}, [GreeksError](../../error/greeks/enum.GreeksError.html "enum optionstratlib::error::greeks::GreeksError"){.enum}\> {#fn-thetaself---resultdecimal-greekserror .code-header}
:::

::: docblock
Calculates the aggregate theta value for all options. [Read
more](../../greeks/trait.Greeks.html#method.theta)
:::

::: {#method.vega .section .method .trait-impl}
[Source](../../../src/optionstratlib/greeks/equations.rs.html#205-212){.src
.rightside}[§](#method.vega){.anchor}

#### fn [vega](../../greeks/trait.Greeks.html#method.vega){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Decimal](../../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}, [GreeksError](../../error/greeks/enum.GreeksError.html "enum optionstratlib::error::greeks::GreeksError"){.enum}\> {#fn-vegaself---resultdecimal-greekserror .code-header}
:::

::: docblock
Calculates the aggregate vega value for all options. [Read
more](../../greeks/trait.Greeks.html#method.vega)
:::

::: {#method.rho .section .method .trait-impl}
[Source](../../../src/optionstratlib/greeks/equations.rs.html#222-229){.src
.rightside}[§](#method.rho){.anchor}

#### fn [rho](../../greeks/trait.Greeks.html#method.rho){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Decimal](../../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}, [GreeksError](../../error/greeks/enum.GreeksError.html "enum optionstratlib::error::greeks::GreeksError"){.enum}\> {#fn-rhoself---resultdecimal-greekserror .code-header}
:::

::: docblock
Calculates the aggregate rho value for all options. [Read
more](../../greeks/trait.Greeks.html#method.rho)
:::

::: {#method.rho_d .section .method .trait-impl}
[Source](../../../src/optionstratlib/greeks/equations.rs.html#239-246){.src
.rightside}[§](#method.rho_d){.anchor}

#### fn [rho_d](../../greeks/trait.Greeks.html#method.rho_d){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Decimal](../../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}, [GreeksError](../../error/greeks/enum.GreeksError.html "enum optionstratlib::error::greeks::GreeksError"){.enum}\> {#fn-rho_dself---resultdecimal-greekserror .code-header}
:::

::: docblock
Calculates the aggregate rho_d value for all options. [Read
more](../../greeks/trait.Greeks.html#method.rho_d)
:::

::: {#method.alpha .section .method .trait-impl}
[Source](../../../src/optionstratlib/greeks/equations.rs.html#256-263){.src
.rightside}[§](#method.alpha){.anchor}

#### fn [alpha](../../greeks/trait.Greeks.html#method.alpha){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Decimal](../../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}, [GreeksError](../../error/greeks/enum.GreeksError.html "enum optionstratlib::error::greeks::GreeksError"){.enum}\> {#fn-alphaself---resultdecimal-greekserror .code-header}
:::

::: docblock
Calculates the aggregate alpha value for all options. [Read
more](../../greeks/trait.Greeks.html#method.alpha)
:::
:::::::::::::::::::::

::: {#impl-PartialEq-for-Options .section .impl}
[Source](../../../src/optionstratlib/model/option.rs.html#66){.src
.rightside}[§](#impl-PartialEq-for-Options){.anchor}

### impl [PartialEq](https://doc.rust-lang.org/1.91.1/core/cmp/trait.PartialEq.html "trait core::cmp::PartialEq"){.trait} for [Options](struct.Options.html "struct optionstratlib::model::option::Options"){.struct} {#impl-partialeq-for-options .code-header}
:::

::::::: impl-items
::: {#method.eq .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/option.rs.html#66){.src
.rightside}[§](#method.eq){.anchor}

#### fn [eq](https://doc.rust-lang.org/1.91.1/core/cmp/trait.PartialEq.html#tymethod.eq){.fn}(&self, other: &[Options](struct.Options.html "struct optionstratlib::model::option::Options"){.struct}) -\> [bool](https://doc.rust-lang.org/1.91.1/std/primitive.bool.html){.primitive} {#fn-eqself-other-options---bool .code-header}
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

::: {#impl-PnLCalculator-for-Options .section .impl}
[Source](../../../src/optionstratlib/model/option.rs.html#720-777){.src
.rightside}[§](#impl-PnLCalculator-for-Options){.anchor}

### impl [PnLCalculator](../../pnl/trait.PnLCalculator.html "trait optionstratlib::pnl::PnLCalculator"){.trait} for [Options](struct.Options.html "struct optionstratlib::model::option::Options"){.struct} {#impl-pnlcalculator-for-options .code-header}
:::

::::::::::: impl-items
::: {#method.calculate_pnl .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/option.rs.html#721-755){.src
.rightside}[§](#method.calculate_pnl){.anchor}

#### fn [calculate_pnl](../../pnl/trait.PnLCalculator.html#tymethod.calculate_pnl){.fn}( &self, market_price: &[Positive](../positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, expiration_date: [ExpirationDate](../enum.ExpirationDate.html "enum optionstratlib::model::ExpirationDate"){.enum}, implied_volatility: &[Positive](../positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, ) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[PnL](../../pnl/utils/struct.PnL.html "struct optionstratlib::pnl::utils::PnL"){.struct}, [PricingError](../../error/pricing/enum.PricingError.html "enum optionstratlib::error::pricing::PricingError"){.enum}\> {#fn-calculate_pnl-self-market_price-positive-expiration_date-expirationdate-implied_volatility-positive---resultpnl-pricingerror .code-header}
:::

::: docblock
Calculates the current PnL based on market conditions. [Read
more](../../pnl/trait.PnLCalculator.html#tymethod.calculate_pnl)
:::

::: {#method.calculate_pnl_at_expiration .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/option.rs.html#757-776){.src
.rightside}[§](#method.calculate_pnl_at_expiration){.anchor}

#### fn [calculate_pnl_at_expiration](../../pnl/trait.PnLCalculator.html#tymethod.calculate_pnl_at_expiration){.fn}( &self, underlying_price: &[Positive](../positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, ) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[PnL](../../pnl/utils/struct.PnL.html "struct optionstratlib::pnl::utils::PnL"){.struct}, [PricingError](../../error/pricing/enum.PricingError.html "enum optionstratlib::error::pricing::PricingError"){.enum}\> {#fn-calculate_pnl_at_expiration-self-underlying_price-positive---resultpnl-pricingerror .code-header}
:::

::: docblock
Calculates the PnL at the expiration of the instrument. [Read
more](../../pnl/trait.PnLCalculator.html#tymethod.calculate_pnl_at_expiration)
:::

::: {#method.adjustments_pnl .section .method .trait-impl}
[Source](../../../src/optionstratlib/pnl/traits.rs.html#70-72){.src
.rightside}[§](#method.adjustments_pnl){.anchor}

#### fn [adjustments_pnl](../../pnl/trait.PnLCalculator.html#method.adjustments_pnl){.fn}( &self, \_adjustments: &[DeltaAdjustment](../../strategies/delta_neutral/enum.DeltaAdjustment.html "enum optionstratlib::strategies::delta_neutral::DeltaAdjustment"){.enum}, ) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[PnL](../../pnl/utils/struct.PnL.html "struct optionstratlib::pnl::utils::PnL"){.struct}, [PricingError](../../error/pricing/enum.PricingError.html "enum optionstratlib::error::pricing::PricingError"){.enum}\> {#fn-adjustments_pnl-self-_adjustments-deltaadjustment---resultpnl-pricingerror .code-header}
:::

::: docblock
Calculates the Profit and Loss (PnL) for a series of delta adjustments
in a trading strategy. [Read
more](../../pnl/trait.PnLCalculator.html#method.adjustments_pnl)
:::

::: {#method.diff_position_pnl .section .method .trait-impl}
[Source](../../../src/optionstratlib/pnl/traits.rs.html#95-97){.src
.rightside}[§](#method.diff_position_pnl){.anchor}

#### fn [diff_position_pnl](../../pnl/trait.PnLCalculator.html#method.diff_position_pnl){.fn}(&self, \_position: &[Position](../position/struct.Position.html "struct optionstratlib::model::position::Position"){.struct}) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[PnL](../../pnl/utils/struct.PnL.html "struct optionstratlib::pnl::utils::PnL"){.struct}, [PricingError](../../error/pricing/enum.PricingError.html "enum optionstratlib::error::pricing::PricingError"){.enum}\> {#fn-diff_position_pnlself-_position-position---resultpnl-pricingerror .code-header}
:::

::: docblock
Calculates the profit and loss (PnL) for a given trading position. [Read
more](../../pnl/trait.PnLCalculator.html#method.diff_position_pnl)
:::
:::::::::::

:::: {#impl-Priceable-for-Options .section .impl}
[Source](../../../src/optionstratlib/pricing/unified.rs.html#104-108){.src
.rightside}[§](#impl-Priceable-for-Options){.anchor}

### impl [Priceable](../../pricing/unified/trait.Priceable.html "trait optionstratlib::pricing::unified::Priceable"){.trait} for [Options](struct.Options.html "struct optionstratlib::model::option::Options"){.struct} {#impl-priceable-for-options .code-header}

::: docblock
Implementation of `Priceable` for `Options`.
:::
::::

::: docblock
This allows options to be priced using the unified pricing API.
:::

::::: impl-items
::: {#method.price .section .method .trait-impl}
[Source](../../../src/optionstratlib/pricing/unified.rs.html#105-107){.src
.rightside}[§](#method.price){.anchor}

#### fn [price](../../pricing/unified/trait.Priceable.html#tymethod.price){.fn}(&self, engine: &[PricingEngine](../../pricing/unified/enum.PricingEngine.html "enum optionstratlib::pricing::unified::PricingEngine"){.enum}) -\> [PricingResult](../../error/pricing/type.PricingResult.html "type optionstratlib::error::pricing::PricingResult"){.type}\<[Positive](../positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\> {#fn-priceself-engine-pricingengine---pricingresultpositive .code-header}
:::

::: docblock
Prices the instrument using the specified pricing engine. [Read
more](../../pricing/unified/trait.Priceable.html#tymethod.price)
:::
:::::

::: {#impl-Profit-for-Options .section .impl}
[Source](../../../src/optionstratlib/model/option.rs.html#779-783){.src
.rightside}[§](#impl-Profit-for-Options){.anchor}

### impl [Profit](../../pricing/trait.Profit.html "trait optionstratlib::pricing::Profit"){.trait} for [Options](struct.Options.html "struct optionstratlib::model::option::Options"){.struct} {#impl-profit-for-options .code-header}
:::

::::::: impl-items
::: {#method.calculate_profit_at .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/option.rs.html#780-782){.src
.rightside}[§](#method.calculate_profit_at){.anchor}

#### fn [calculate_profit_at](../../pricing/trait.Profit.html#tymethod.calculate_profit_at){.fn}(&self, price: &[Positive](../positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Decimal](../../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}, [PricingError](../../error/pricing/enum.PricingError.html "enum optionstratlib::error::pricing::PricingError"){.enum}\> {#fn-calculate_profit_atself-price-positive---resultdecimal-pricingerror .code-header}
:::

::: docblock
Calculates the profit at a specified price. [Read
more](../../pricing/trait.Profit.html#tymethod.calculate_profit_at)
:::

::: {#method.get_point_at_price .section .method .trait-impl}
[Source](../../../src/optionstratlib/pricing/payoff.rs.html#219-225){.src
.rightside}[§](#method.get_point_at_price){.anchor}

#### fn [get_point_at_price](../../pricing/trait.Profit.html#method.get_point_at_price){.fn}( &self, \_price: &[Positive](../positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, ) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<([Decimal](../../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}, [Decimal](../../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}), [PricingError](../../error/pricing/enum.PricingError.html "enum optionstratlib::error::pricing::PricingError"){.enum}\> {#fn-get_point_at_price-self-_price-positive---resultdecimal-decimal-pricingerror .code-header}
:::

::: docblock
Creates a chart point representation of the profit at the given price.
[Read more](../../pricing/trait.Profit.html#method.get_point_at_price)
:::
:::::::

::: {#impl-Serialize-for-Options .section .impl}
[Source](../../../src/optionstratlib/model/option.rs.html#66){.src
.rightside}[§](#impl-Serialize-for-Options){.anchor}

### impl [Serialize](../../../serde_core/ser/trait.Serialize.html "trait serde_core::ser::Serialize"){.trait} for [Options](struct.Options.html "struct optionstratlib::model::option::Options"){.struct} {#impl-serialize-for-options .code-header}
:::

:::::: impl-items
:::: {#method.serialize .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/option.rs.html#66){.src
.rightside}[§](#method.serialize){.anchor}

#### fn [serialize](../../../serde_core/ser/trait.Serialize.html#tymethod.serialize){.fn}\<\_\_S\>(&self, \_\_serializer: \_\_S) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<\_\_S::[Ok](../../../serde_core/ser/trait.Serializer.html#associatedtype.Ok "type serde_core::ser::Serializer::Ok"){.associatedtype}, \_\_S::[Error](../../../serde_core/ser/trait.Serializer.html#associatedtype.Error "type serde_core::ser::Serializer::Error"){.associatedtype}\> {#fn-serialize__sself-__serializer-__s---result__sok-__serror .code-header}

::: where
where \_\_S:
[Serializer](../../../serde_core/ser/trait.Serializer.html "trait serde_core::ser::Serializer"){.trait},
:::
::::

::: docblock
Serialize this value into the given Serde serializer. [Read
more](../../../serde_core/ser/trait.Serialize.html#tymethod.serialize)
:::
::::::

::: {#impl-ToSchema-for-Options .section .impl}
[Source](../../../src/optionstratlib/model/option.rs.html#66){.src
.rightside}[§](#impl-ToSchema-for-Options){.anchor}

### impl [ToSchema](../../../utoipa/trait.ToSchema.html "trait utoipa::ToSchema"){.trait} for [Options](struct.Options.html "struct optionstratlib::model::option::Options"){.struct} {#impl-toschema-for-options .code-header}
:::

::::::: impl-items
::: {#method.name .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/option.rs.html#66){.src
.rightside}[§](#method.name){.anchor}

#### fn [name](../../../utoipa/trait.ToSchema.html#method.name){.fn}() -\> [Cow](https://doc.rust-lang.org/1.91.1/alloc/borrow/enum.Cow.html "enum alloc::borrow::Cow"){.enum}\<\'static, [str](https://doc.rust-lang.org/1.91.1/std/primitive.str.html){.primitive}\> {#fn-name---cowstatic-str .code-header}
:::

::: docblock
Return name of the schema. [Read
more](../../../utoipa/trait.ToSchema.html#method.name)
:::

::: {#method.schemas .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/option.rs.html#66){.src
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

::: {#impl-StructuralPartialEq-for-Options .section .impl}
[Source](../../../src/optionstratlib/model/option.rs.html#66){.src
.rightside}[§](#impl-StructuralPartialEq-for-Options){.anchor}

### impl [StructuralPartialEq](https://doc.rust-lang.org/1.91.1/core/marker/trait.StructuralPartialEq.html "trait core::marker::StructuralPartialEq"){.trait} for [Options](struct.Options.html "struct optionstratlib::model::option::Options"){.struct} {#impl-structuralpartialeq-for-options .code-header}
:::
:::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::

## Auto Trait Implementations[§](#synthetic-implementations){.anchor} {#synthetic-implementations .section-header}

::::::::: {#synthetic-implementations-list}
::: {#impl-Freeze-for-Options .section .impl}
[§](#impl-Freeze-for-Options){.anchor}

### impl [Freeze](https://doc.rust-lang.org/1.91.1/core/marker/trait.Freeze.html "trait core::marker::Freeze"){.trait} for [Options](struct.Options.html "struct optionstratlib::model::option::Options"){.struct} {#impl-freeze-for-options .code-header}
:::

::: {#impl-RefUnwindSafe-for-Options .section .impl}
[§](#impl-RefUnwindSafe-for-Options){.anchor}

### impl [RefUnwindSafe](https://doc.rust-lang.org/1.91.1/core/panic/unwind_safe/trait.RefUnwindSafe.html "trait core::panic::unwind_safe::RefUnwindSafe"){.trait} for [Options](struct.Options.html "struct optionstratlib::model::option::Options"){.struct} {#impl-refunwindsafe-for-options .code-header}
:::

::: {#impl-Send-for-Options .section .impl}
[§](#impl-Send-for-Options){.anchor}

### impl [Send](https://doc.rust-lang.org/1.91.1/core/marker/trait.Send.html "trait core::marker::Send"){.trait} for [Options](struct.Options.html "struct optionstratlib::model::option::Options"){.struct} {#impl-send-for-options .code-header}
:::

::: {#impl-Sync-for-Options .section .impl}
[§](#impl-Sync-for-Options){.anchor}

### impl [Sync](https://doc.rust-lang.org/1.91.1/core/marker/trait.Sync.html "trait core::marker::Sync"){.trait} for [Options](struct.Options.html "struct optionstratlib::model::option::Options"){.struct} {#impl-sync-for-options .code-header}
:::

::: {#impl-Unpin-for-Options .section .impl}
[§](#impl-Unpin-for-Options){.anchor}

### impl [Unpin](https://doc.rust-lang.org/1.91.1/core/marker/trait.Unpin.html "trait core::marker::Unpin"){.trait} for [Options](struct.Options.html "struct optionstratlib::model::option::Options"){.struct} {#impl-unpin-for-options .code-header}
:::

::: {#impl-UnwindSafe-for-Options .section .impl}
[§](#impl-UnwindSafe-for-Options){.anchor}

### impl [UnwindSafe](https://doc.rust-lang.org/1.91.1/core/panic/unwind_safe/trait.UnwindSafe.html "trait core::panic::unwind_safe::UnwindSafe"){.trait} for [Options](struct.Options.html "struct optionstratlib::model::option::Options"){.struct} {#impl-unwindsafe-for-options .code-header}
:::
:::::::::

## Blanket Implementations[§](#blanket-implementations){.anchor} {#blanket-implementations .section-header}

::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::: {#blanket-implementations-list}
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
:::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
:::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
