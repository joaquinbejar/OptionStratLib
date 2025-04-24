:::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::: width-limiter
::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../../index.html)::[model](../index.html)::[option](index.html)
:::

# Struct [Options]{.struct}Copy item path

[[Source](../../../src/optionstratlib/model/option.rs.html#61-105){.src}
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
Specifies whether this is a Call or Put option, determining the
fundamental right the option contract provides (buying or selling the
underlying).
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
.field}`underlying_symbol: `[`String`](https://doc.rust-lang.org/1.86.0/alloc/string/struct.String.html "struct alloc::string::String"){.struct}]{#structfield.underlying_symbol
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
.field}`expiration_date: `[`ExpirationDate`](../types/enum.ExpirationDate.html "enum optionstratlib::model::types::ExpirationDate"){.enum}]{#structfield.expiration_date
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
.field}`risk_free_rate: Decimal`]{#structfield.risk_free_rate
.structfield .section-header}

::: docblock
The current risk-free interest rate used in option pricing models,
typically based on treasury yields of similar duration.
:::

[[§](#structfield.option_style){.anchor
.field}`option_style: `[`OptionStyle`](../types/enum.OptionStyle.html "enum optionstratlib::model::types::OptionStyle"){.enum}]{#structfield.option_style
.structfield .section-header}

::: docblock
The option exercise style (European or American), determining when the
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
.field}`exotic_params: `[`Option`](https://doc.rust-lang.org/1.86.0/core/option/enum.Option.html "enum core::option::Option"){.enum}`<`[`ExoticParams`](struct.ExoticParams.html "struct optionstratlib::model::option::ExoticParams"){.struct}`>`]{#structfield.exotic_params
.structfield .section-header}

::: docblock
Additional parameters required for exotic option types like Asian or
Lookback options. This field is None for standard (vanilla) options.
:::

## Implementations[§](#implementations){.anchor} {#implementations .section-header}

:::::::::::::::::::::::::::::::::: {#implementations-list}
::: {#impl-Options .section .impl}
[Source](../../../src/optionstratlib/model/option.rs.html#106-638){.src
.rightside}[§](#impl-Options){.anchor}

### impl [Options](struct.Options.html "struct optionstratlib::model::option::Options"){.struct} {#impl-options .code-header}
:::

:::::::::::::::::::::::::::::::: impl-items
::: {#method.new .section .method}
[Source](../../../src/optionstratlib/model/option.rs.html#138-166){.src
.rightside}

#### pub fn [new](#method.new){.fn}( option_type: [OptionType](../types/enum.OptionType.html "enum optionstratlib::model::types::OptionType"){.enum}, side: [Side](../types/enum.Side.html "enum optionstratlib::model::types::Side"){.enum}, underlying_symbol: [String](https://doc.rust-lang.org/1.86.0/alloc/string/struct.String.html "struct alloc::string::String"){.struct}, strike_price: [Positive](../positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, expiration_date: [ExpirationDate](../types/enum.ExpirationDate.html "enum optionstratlib::model::types::ExpirationDate"){.enum}, implied_volatility: [Positive](../positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, quantity: [Positive](../positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, underlying_price: [Positive](../positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, risk_free_rate: Decimal, option_style: [OptionStyle](../types/enum.OptionStyle.html "enum optionstratlib::model::types::OptionStyle"){.enum}, dividend_yield: [Positive](../positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, exotic_params: [Option](https://doc.rust-lang.org/1.86.0/core/option/enum.Option.html "enum core::option::Option"){.enum}\<[ExoticParams](struct.ExoticParams.html "struct optionstratlib::model::option::ExoticParams"){.struct}\>, ) -\> Self {#pub-fn-new-option_type-optiontype-side-side-underlying_symbol-string-strike_price-positive-expiration_date-expirationdate-implied_volatility-positive-quantity-positive-underlying_price-positive-risk_free_rate-decimal-option_style-optionstyle-dividend_yield-positive-exotic_params-optionexoticparams---self .code-header}
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
[Source](../../../src/optionstratlib/model/option.rs.html#195-197){.src
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
[Source](../../../src/optionstratlib/model/option.rs.html#208-210){.src
.rightside}

#### pub fn [is_long](#method.is_long){.fn}(&self) -\> [bool](https://doc.rust-lang.org/1.86.0/std/primitive.bool.html){.primitive} {#pub-fn-is_longself---bool .code-header}
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
[Source](../../../src/optionstratlib/model/option.rs.html#221-223){.src
.rightside}

#### pub fn [is_short](#method.is_short){.fn}(&self) -\> [bool](https://doc.rust-lang.org/1.86.0/std/primitive.bool.html){.primitive} {#pub-fn-is_shortself---bool .code-header}
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
[Source](../../../src/optionstratlib/model/option.rs.html#253-272){.src
.rightside}

#### pub fn [calculate_price_binomial](#method.calculate_price_binomial){.fn}( &self, no_steps: [usize](https://doc.rust-lang.org/1.86.0/std/primitive.usize.html){.primitive}, ) -\> [OptionsResult](../../error/type.OptionsResult.html "type optionstratlib::error::OptionsResult"){.type}\<Decimal\> {#pub-fn-calculate_price_binomial-self-no_steps-usize---optionsresultdecimal .code-header}
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
[Source](../../../src/optionstratlib/model/option.rs.html#294-313){.src
.rightside}

#### pub fn [calculate_price_binomial_tree](#method.calculate_price_binomial_tree){.fn}( &self, no_steps: [usize](https://doc.rust-lang.org/1.86.0/std/primitive.usize.html){.primitive}, ) -\> [OptionsResult](../../error/type.OptionsResult.html "type optionstratlib::error::OptionsResult"){.type}\<(Decimal, [Vec](https://doc.rust-lang.org/1.86.0/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<[Vec](https://doc.rust-lang.org/1.86.0/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<Decimal\>\>, [Vec](https://doc.rust-lang.org/1.86.0/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<[Vec](https://doc.rust-lang.org/1.86.0/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<Decimal\>\>)\> {#pub-fn-calculate_price_binomial_tree-self-no_steps-usize---optionsresultdecimal-vecvecdecimal-vecvecdecimal .code-header}
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
[Source](../../../src/optionstratlib/model/option.rs.html#328-330){.src
.rightside}

#### pub fn [calculate_price_black_scholes](#method.calculate_price_black_scholes){.fn}(&self) -\> [OptionsResult](../../error/type.OptionsResult.html "type optionstratlib::error::OptionsResult"){.type}\<Decimal\> {#pub-fn-calculate_price_black_scholesself---optionsresultdecimal .code-header}
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

::: {#method.calculate_price_telegraph .section .method}
[Source](../../../src/optionstratlib/model/option.rs.html#347-349){.src
.rightside}

#### pub fn [calculate_price_telegraph](#method.calculate_price_telegraph){.fn}( &self, no_steps: [usize](https://doc.rust-lang.org/1.86.0/std/primitive.usize.html){.primitive}, ) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<Decimal, [Box](https://doc.rust-lang.org/1.86.0/alloc/boxed/struct.Box.html "struct alloc::boxed::Box"){.struct}\<dyn [Error](https://doc.rust-lang.org/1.86.0/core/error/trait.Error.html "trait core::error::Error"){.trait}\>\> {#pub-fn-calculate_price_telegraph-self-no_steps-usize---resultdecimal-boxdyn-error .code-header}
:::

::: docblock
Calculates option price using the Telegraph equation approach.

This method implements a finite-difference method based on the Telegraph
equation to price options. This approach can handle a variety of option
styles and types, including path-dependent options.

##### [§](#parameters-3){.doc-anchor}Parameters

- `no_steps` - The number of discrete time steps to use in the model.
  Higher values increase precision but also computational cost.

##### [§](#returns-7){.doc-anchor}Returns

- `Result<Decimal, Box<dyn Error>>` - A result containing the calculated
  option price as a Decimal value, or a boxed error if the calculation
  failed.
:::

::: {#method.payoff .section .method}
[Source](../../../src/optionstratlib/model/option.rs.html#365-377){.src
.rightside}

#### pub fn [payoff](#method.payoff){.fn}(&self) -\> [OptionsResult](../../error/type.OptionsResult.html "type optionstratlib::error::OptionsResult"){.type}\<Decimal\> {#pub-fn-payoffself---optionsresultdecimal .code-header}
:::

::: docblock
Calculates the intrinsic value (payoff) of the option at the current
underlying price.

The payoff represents what the option would be worth if exercised
immediately, based on the current market conditions. For
out-of-the-money options, the payoff will be zero.

##### [§](#returns-8){.doc-anchor}Returns

- `OptionsResult<Decimal>` - A result containing the calculated payoff
  as a Decimal value, adjusted for the quantity of contracts held, or an
  error if the calculation failed.

This method is useful for determining the exercise value of an option
and for analyzing whether an option has intrinsic value.
:::

::: {#method.payoff_at_price .section .method}
[Source](../../../src/optionstratlib/model/option.rs.html#394-406){.src
.rightside}

#### pub fn [payoff_at_price](#method.payoff_at_price){.fn}(&self, price: [Positive](../positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}) -\> [OptionsResult](../../error/type.OptionsResult.html "type optionstratlib::error::OptionsResult"){.type}\<Decimal\> {#pub-fn-payoff_at_priceself-price-positive---optionsresultdecimal .code-header}
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

##### [§](#returns-9){.doc-anchor}Returns

- `OptionsResult<Decimal>` - The calculated payoff value as a `Decimal`,
  wrapped in a `Result` type. Returns an `Err` if the payoff calculation
  encounters an error.
:::

::: {#method.intrinsic_value .section .method}
[Source](../../../src/optionstratlib/model/option.rs.html#421-433){.src
.rightside}

#### pub fn [intrinsic_value](#method.intrinsic_value){.fn}( &self, underlying_price: [Positive](../positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, ) -\> [OptionsResult](../../error/type.OptionsResult.html "type optionstratlib::error::OptionsResult"){.type}\<Decimal\> {#pub-fn-intrinsic_value-self-underlying_price-positive---optionsresultdecimal .code-header}
:::

::: docblock
Calculates the intrinsic value of the option.

The intrinsic value is the difference between the underlying asset's
price and the option's strike price. For call options, the intrinsic
value is the maximum of zero and the difference between the underlying
price and the strike price. For put options, the intrinsic value is the
maximum of zero and the difference between the strike price and the
underlying price.

##### [§](#arguments){.doc-anchor}Arguments

- `underlying_price` - The current price of the underlying asset.

##### [§](#returns-10){.doc-anchor}Returns

- `OptionsResult<Decimal>` - The intrinsic value of the option, or an
  error if the calculation fails.
:::

::: {#method.is_in_the_money .section .method}
[Source](../../../src/optionstratlib/model/option.rs.html#445-450){.src
.rightside}

#### pub fn [is_in_the_money](#method.is_in_the_money){.fn}(&self) -\> [bool](https://doc.rust-lang.org/1.86.0/std/primitive.bool.html){.primitive} {#pub-fn-is_in_the_moneyself---bool .code-header}
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

##### [§](#returns-11){.doc-anchor}Returns

`true` if the option is in-the-money, `false` otherwise
:::

::: {#method.time_value .section .method}
[Source](../../../src/optionstratlib/model/option.rs.html#464-468){.src
.rightside}

#### pub fn [time_value](#method.time_value){.fn}(&self) -\> [OptionsResult](../../error/type.OptionsResult.html "type optionstratlib::error::OptionsResult"){.type}\<Decimal\> {#pub-fn-time_valueself---optionsresultdecimal .code-header}
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

##### [§](#returns-12){.doc-anchor}Returns

- `Ok(Decimal)` containing the time value (never negative, minimum value
  is zero)
- `Err` if the price calculation encounters an error
:::

::: {#method.calculate_implied_volatility .section .method}
[Source](../../../src/optionstratlib/model/option.rs.html#584-637){.src
.rightside}

#### pub fn [calculate_implied_volatility](#method.calculate_implied_volatility){.fn}( &self, market_price: Decimal, ) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Positive](../positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, [VolatilityError](../../error/enum.VolatilityError.html "enum optionstratlib::error::VolatilityError"){.enum}\> {#pub-fn-calculate_implied_volatility-self-market_price-decimal---resultpositive-volatilityerror .code-header}
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

###### [§](#returns-13){.doc-anchor}Returns:

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
::::::::::::::::::::::::::::::::
::::::::::::::::::::::::::::::::::

## Trait Implementations[§](#trait-implementations){.anchor} {#trait-implementations .section-header}

:::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::: {#trait-implementations-list}
::: {#impl-Clone-for-Options .section .impl}
[Source](../../../src/optionstratlib/model/option.rs.html#60){.src
.rightside}[§](#impl-Clone-for-Options){.anchor}

### impl [Clone](https://doc.rust-lang.org/1.86.0/core/clone/trait.Clone.html "trait core::clone::Clone"){.trait} for [Options](struct.Options.html "struct optionstratlib::model::option::Options"){.struct} {#impl-clone-for-options .code-header}
:::

::::::: impl-items
::: {#method.clone .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/option.rs.html#60){.src
.rightside}[§](#method.clone){.anchor}

#### fn [clone](https://doc.rust-lang.org/1.86.0/core/clone/trait.Clone.html#tymethod.clone){.fn}(&self) -\> [Options](struct.Options.html "struct optionstratlib::model::option::Options"){.struct} {#fn-cloneself---options .code-header}
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

::: {#impl-Debug-for-Options .section .impl}
[Source](../../../src/optionstratlib/model/format.rs.html#50-67){.src
.rightside}[§](#impl-Debug-for-Options){.anchor}

### impl [Debug](https://doc.rust-lang.org/1.86.0/core/fmt/trait.Debug.html "trait core::fmt::Debug"){.trait} for [Options](struct.Options.html "struct optionstratlib::model::option::Options"){.struct} {#impl-debug-for-options .code-header}
:::

::::: impl-items
::: {#method.fmt-1 .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/format.rs.html#51-66){.src
.rightside}[§](#method.fmt-1){.anchor}

#### fn [fmt](https://doc.rust-lang.org/1.86.0/core/fmt/trait.Debug.html#tymethod.fmt){.fn}(&self, f: &mut [Formatter](https://doc.rust-lang.org/1.86.0/core/fmt/struct.Formatter.html "struct core::fmt::Formatter"){.struct}\<\'\_\>) -\> [Result](https://doc.rust-lang.org/1.86.0/core/fmt/type.Result.html "type core::fmt::Result"){.type} {#fn-fmtself-f-mut-formatter_---result .code-header}
:::

::: docblock
Formats the value using the given formatter. [Read
more](https://doc.rust-lang.org/1.86.0/core/fmt/trait.Debug.html#tymethod.fmt)
:::
:::::

::: {#impl-Default-for-Options .section .impl}
[Source](../../../src/optionstratlib/model/option.rs.html#640-657){.src
.rightside}[§](#impl-Default-for-Options){.anchor}

### impl [Default](https://doc.rust-lang.org/1.86.0/core/default/trait.Default.html "trait core::default::Default"){.trait} for [Options](struct.Options.html "struct optionstratlib::model::option::Options"){.struct} {#impl-default-for-options .code-header}
:::

::::: impl-items
::: {#method.default .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/option.rs.html#641-656){.src
.rightside}[§](#method.default){.anchor}

#### fn [default](https://doc.rust-lang.org/1.86.0/core/default/trait.Default.html#tymethod.default){.fn}() -\> Self {#fn-default---self .code-header}
:::

::: docblock
Returns the "default value" for a type. [Read
more](https://doc.rust-lang.org/1.86.0/core/default/trait.Default.html#tymethod.default)
:::
:::::

::: {#impl-Deserialize%3C'de%3E-for-Options .section .impl}
[Source](../../../src/optionstratlib/model/option.rs.html#60){.src
.rightside}[§](#impl-Deserialize%3C'de%3E-for-Options){.anchor}

### impl\<\'de\> [Deserialize](https://docs.rs/serde/1.0.219/serde/de/trait.Deserialize.html "trait serde::de::Deserialize"){.trait}\<\'de\> for [Options](struct.Options.html "struct optionstratlib::model::option::Options"){.struct} {#implde-deserializede-for-options .code-header}
:::

:::::: impl-items
:::: {#method.deserialize .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/option.rs.html#60){.src
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

::: {#impl-Display-for-Options .section .impl}
[Source](../../../src/optionstratlib/model/format.rs.html#17-48){.src
.rightside}[§](#impl-Display-for-Options){.anchor}

### impl [Display](https://doc.rust-lang.org/1.86.0/core/fmt/trait.Display.html "trait core::fmt::Display"){.trait} for [Options](struct.Options.html "struct optionstratlib::model::option::Options"){.struct} {#impl-display-for-options .code-header}
:::

::::: impl-items
::: {#method.fmt .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/format.rs.html#18-47){.src
.rightside}[§](#method.fmt){.anchor}

#### fn [fmt](https://doc.rust-lang.org/1.86.0/core/fmt/trait.Display.html#tymethod.fmt){.fn}(&self, f: &mut [Formatter](https://doc.rust-lang.org/1.86.0/core/fmt/struct.Formatter.html "struct core::fmt::Formatter"){.struct}\<\'\_\>) -\> [Result](https://doc.rust-lang.org/1.86.0/core/fmt/type.Result.html "type core::fmt::Result"){.type} {#fn-fmtself-f-mut-formatter_---result-1 .code-header}
:::

::: docblock
Formats the value using the given formatter. [Read
more](https://doc.rust-lang.org/1.86.0/core/fmt/trait.Display.html#tymethod.fmt)
:::
:::::

::: {#impl-Graph-for-Options .section .impl}
[Source](../../../src/optionstratlib/model/option.rs.html#730-792){.src
.rightside}[§](#impl-Graph-for-Options){.anchor}

### impl [Graph](../../visualization/utils/trait.Graph.html "trait optionstratlib::visualization::utils::Graph"){.trait} for [Options](struct.Options.html "struct optionstratlib::model::option::Options"){.struct} {#impl-graph-for-options .code-header}
:::

::::::::::::::: impl-items
::: {#method.get_x_values .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/option.rs.html#759-769){.src
.rightside}[§](#method.get_x_values){.anchor}

#### fn [get_x_values](../../visualization/utils/trait.Graph.html#tymethod.get_x_values){.fn}(&self) -\> [Vec](https://doc.rust-lang.org/1.86.0/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<[Positive](../positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\> {#fn-get_x_valuesself---vecpositive .code-header}
:::

::: docblock
Generates a vector of evenly spaced x-values for option
pricing/plotting.

This method creates a range of x-values (potential stock prices)
centered around the strike price and spanning 5 standard deviations in
each direction. The standard deviation is calculated as the product of
strike price and implied volatility.

##### [§](#returns-14){.doc-anchor}Returns

A vector of `Positive` values representing potential stock prices, with
1000 total points (999 steps plus endpoints) evenly distributed across
the range.

##### [§](#implementation-details-1){.doc-anchor}Implementation Details

- The range extends 5 standard deviations above and below the strike
  price
- Uses 1000 total points (steps + 1) for smooth visualization
- All returned values are guaranteed positive through the use of the
  `pos!` macro
:::

::: {#method.title .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/option.rs.html#731-740){.src
.rightside}[§](#method.title){.anchor}

#### fn [title](../../visualization/utils/trait.Graph.html#tymethod.title){.fn}(&self) -\> [String](https://doc.rust-lang.org/1.86.0/alloc/string/struct.String.html "struct alloc::string::String"){.struct} {#fn-titleself---string .code-header}
:::

::: docblock
Returns the title of the graph.
:::

::: {#method.get_y_values .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/option.rs.html#771-776){.src
.rightside}[§](#method.get_y_values){.anchor}

#### fn [get_y_values](../../visualization/utils/trait.Graph.html#method.get_y_values){.fn}(&self) -\> [Vec](https://doc.rust-lang.org/1.86.0/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<[f64](https://doc.rust-lang.org/1.86.0/std/primitive.f64.html){.primitive}\> {#fn-get_y_valuesself---vecf64 .code-header}
:::

::: docblock
Calculates the y-axis values (profit) corresponding to the provided
x-axis data. [Read
more](../../visualization/utils/trait.Graph.html#method.get_y_values)
:::

::: {#method.get_vertical_lines .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/option.rs.html#778-791){.src
.rightside}[§](#method.get_vertical_lines){.anchor}

#### fn [get_vertical_lines](../../visualization/utils/trait.Graph.html#method.get_vertical_lines){.fn}(&self) -\> [Vec](https://doc.rust-lang.org/1.86.0/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<ChartVerticalLine\<[f64](https://doc.rust-lang.org/1.86.0/std/primitive.f64.html){.primitive}, [f64](https://doc.rust-lang.org/1.86.0/std/primitive.f64.html){.primitive}\>\> {#fn-get_vertical_linesself---vecchartverticallinef64-f64 .code-header}
:::

::: docblock
Returns a vector of vertical lines to draw on the chart. Default
implementation returns an empty vector.
:::

::: {#method.graph .section .method .trait-impl}
[Source](../../../src/optionstratlib/visualization/utils.rs.html#198-240){.src
.rightside}[§](#method.graph){.anchor}

#### fn [graph](../../visualization/utils/trait.Graph.html#method.graph){.fn}( &self, backend: [GraphBackend](../../visualization/utils/enum.GraphBackend.html "enum optionstratlib::visualization::utils::GraphBackend"){.enum}\<\'\_\>, title_size: [u32](https://doc.rust-lang.org/1.86.0/std/primitive.u32.html){.primitive}, ) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[()](https://doc.rust-lang.org/1.86.0/std/primitive.unit.html){.primitive}, [Box](https://doc.rust-lang.org/1.86.0/alloc/boxed/struct.Box.html "struct alloc::boxed::Box"){.struct}\<dyn [Error](https://doc.rust-lang.org/1.86.0/core/error/trait.Error.html "trait core::error::Error"){.trait}\>\> {#fn-graph-self-backend-graphbackend_-title_size-u32---result-boxdyn-error .code-header}
:::

::: docblock
Generates a graph of profit calculations. [Read
more](../../visualization/utils/trait.Graph.html#method.graph)
:::

::: {#method.get_points .section .method .trait-impl}
[Source](../../../src/optionstratlib/visualization/utils.rs.html#287-289){.src
.rightside}[§](#method.get_points){.anchor}

#### fn [get_points](../../visualization/utils/trait.Graph.html#method.get_points){.fn}(&self) -\> [Vec](https://doc.rust-lang.org/1.86.0/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<ChartPoint\<([f64](https://doc.rust-lang.org/1.86.0/std/primitive.f64.html){.primitive}, [f64](https://doc.rust-lang.org/1.86.0/std/primitive.f64.html){.primitive})\>\> {#fn-get_pointsself---vecchartpointf64-f64 .code-header}
:::

::: docblock
Returns a vector of points to draw on the chart. Default implementation
returns an empty vector.
:::
:::::::::::::::

::: {#impl-Greeks-for-Options .section .impl}
[Source](../../../src/optionstratlib/model/option.rs.html#659-663){.src
.rightside}[§](#impl-Greeks-for-Options){.anchor}

### impl [Greeks](../../greeks/trait.Greeks.html "trait optionstratlib::greeks::Greeks"){.trait} for [Options](struct.Options.html "struct optionstratlib::model::option::Options"){.struct} {#impl-greeks-for-options .code-header}
:::

::::::::::::::::::::: impl-items
::: {#method.get_options .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/option.rs.html#660-662){.src
.rightside}[§](#method.get_options){.anchor}

#### fn [get_options](../../greeks/trait.Greeks.html#tymethod.get_options){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Vec](https://doc.rust-lang.org/1.86.0/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<&[Options](struct.Options.html "struct optionstratlib::model::option::Options"){.struct}\>, [GreeksError](../../error/greeks/enum.GreeksError.html "enum optionstratlib::error::greeks::GreeksError"){.enum}\> {#fn-get_optionsself---resultvecoptions-greekserror .code-header}
:::

::: docblock
Returns a vector of references to the option contracts for which Greeks
will be calculated. [Read
more](../../greeks/trait.Greeks.html#tymethod.get_options)
:::

::: {#method.greeks .section .method .trait-impl}
[Source](../../../src/optionstratlib/greeks/equations.rs.html#99-116){.src
.rightside}[§](#method.greeks){.anchor}

#### fn [greeks](../../greeks/trait.Greeks.html#method.greeks){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Greek](../../greeks/struct.Greek.html "struct optionstratlib::greeks::Greek"){.struct}, [GreeksError](../../error/greeks/enum.GreeksError.html "enum optionstratlib::error::greeks::GreeksError"){.enum}\> {#fn-greeksself---resultgreek-greekserror .code-header}
:::

::: docblock
Calculates and returns all Greeks as a single `Greek` struct. [Read
more](../../greeks/trait.Greeks.html#method.greeks)
:::

::: {#method.delta .section .method .trait-impl}
[Source](../../../src/optionstratlib/greeks/equations.rs.html#126-133){.src
.rightside}[§](#method.delta){.anchor}

#### fn [delta](../../greeks/trait.Greeks.html#method.delta){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<Decimal, [GreeksError](../../error/greeks/enum.GreeksError.html "enum optionstratlib::error::greeks::GreeksError"){.enum}\> {#fn-deltaself---resultdecimal-greekserror .code-header}
:::

::: docblock
Calculates the aggregate delta value for all options. [Read
more](../../greeks/trait.Greeks.html#method.delta)
:::

::: {#method.gamma .section .method .trait-impl}
[Source](../../../src/optionstratlib/greeks/equations.rs.html#143-150){.src
.rightside}[§](#method.gamma){.anchor}

#### fn [gamma](../../greeks/trait.Greeks.html#method.gamma){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<Decimal, [GreeksError](../../error/greeks/enum.GreeksError.html "enum optionstratlib::error::greeks::GreeksError"){.enum}\> {#fn-gammaself---resultdecimal-greekserror .code-header}
:::

::: docblock
Calculates the aggregate gamma value for all options. [Read
more](../../greeks/trait.Greeks.html#method.gamma)
:::

::: {#method.theta .section .method .trait-impl}
[Source](../../../src/optionstratlib/greeks/equations.rs.html#160-167){.src
.rightside}[§](#method.theta){.anchor}

#### fn [theta](../../greeks/trait.Greeks.html#method.theta){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<Decimal, [GreeksError](../../error/greeks/enum.GreeksError.html "enum optionstratlib::error::greeks::GreeksError"){.enum}\> {#fn-thetaself---resultdecimal-greekserror .code-header}
:::

::: docblock
Calculates the aggregate theta value for all options. [Read
more](../../greeks/trait.Greeks.html#method.theta)
:::

::: {#method.vega .section .method .trait-impl}
[Source](../../../src/optionstratlib/greeks/equations.rs.html#177-184){.src
.rightside}[§](#method.vega){.anchor}

#### fn [vega](../../greeks/trait.Greeks.html#method.vega){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<Decimal, [GreeksError](../../error/greeks/enum.GreeksError.html "enum optionstratlib::error::greeks::GreeksError"){.enum}\> {#fn-vegaself---resultdecimal-greekserror .code-header}
:::

::: docblock
Calculates the aggregate vega value for all options. [Read
more](../../greeks/trait.Greeks.html#method.vega)
:::

::: {#method.rho .section .method .trait-impl}
[Source](../../../src/optionstratlib/greeks/equations.rs.html#194-201){.src
.rightside}[§](#method.rho){.anchor}

#### fn [rho](../../greeks/trait.Greeks.html#method.rho){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<Decimal, [GreeksError](../../error/greeks/enum.GreeksError.html "enum optionstratlib::error::greeks::GreeksError"){.enum}\> {#fn-rhoself---resultdecimal-greekserror .code-header}
:::

::: docblock
Calculates the aggregate rho value for all options. [Read
more](../../greeks/trait.Greeks.html#method.rho)
:::

::: {#method.rho_d .section .method .trait-impl}
[Source](../../../src/optionstratlib/greeks/equations.rs.html#211-218){.src
.rightside}[§](#method.rho_d){.anchor}

#### fn [rho_d](../../greeks/trait.Greeks.html#method.rho_d){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<Decimal, [GreeksError](../../error/greeks/enum.GreeksError.html "enum optionstratlib::error::greeks::GreeksError"){.enum}\> {#fn-rho_dself---resultdecimal-greekserror .code-header}
:::

::: docblock
Calculates the aggregate rho_d value for all options. [Read
more](../../greeks/trait.Greeks.html#method.rho_d)
:::

::: {#method.alpha .section .method .trait-impl}
[Source](../../../src/optionstratlib/greeks/equations.rs.html#228-235){.src
.rightside}[§](#method.alpha){.anchor}

#### fn [alpha](../../greeks/trait.Greeks.html#method.alpha){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<Decimal, [GreeksError](../../error/greeks/enum.GreeksError.html "enum optionstratlib::error::greeks::GreeksError"){.enum}\> {#fn-alphaself---resultdecimal-greekserror .code-header}
:::

::: docblock
Calculates the aggregate alpha value for all options. [Read
more](../../greeks/trait.Greeks.html#method.alpha)
:::
:::::::::::::::::::::

::: {#impl-PartialEq-for-Options .section .impl}
[Source](../../../src/optionstratlib/model/option.rs.html#60){.src
.rightside}[§](#impl-PartialEq-for-Options){.anchor}

### impl [PartialEq](https://doc.rust-lang.org/1.86.0/core/cmp/trait.PartialEq.html "trait core::cmp::PartialEq"){.trait} for [Options](struct.Options.html "struct optionstratlib::model::option::Options"){.struct} {#impl-partialeq-for-options .code-header}
:::

::::::: impl-items
::: {#method.eq .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/option.rs.html#60){.src
.rightside}[§](#method.eq){.anchor}

#### fn [eq](https://doc.rust-lang.org/1.86.0/core/cmp/trait.PartialEq.html#tymethod.eq){.fn}(&self, other: &[Options](struct.Options.html "struct optionstratlib::model::option::Options"){.struct}) -\> [bool](https://doc.rust-lang.org/1.86.0/std/primitive.bool.html){.primitive} {#fn-eqself-other-options---bool .code-header}
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

::: {#impl-PnLCalculator-for-Options .section .impl}
[Source](../../../src/optionstratlib/model/option.rs.html#665-722){.src
.rightside}[§](#impl-PnLCalculator-for-Options){.anchor}

### impl [PnLCalculator](../../pnl/trait.PnLCalculator.html "trait optionstratlib::pnl::PnLCalculator"){.trait} for [Options](struct.Options.html "struct optionstratlib::model::option::Options"){.struct} {#impl-pnlcalculator-for-options .code-header}
:::

::::::::: impl-items
::: {#method.calculate_pnl .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/option.rs.html#666-700){.src
.rightside}[§](#method.calculate_pnl){.anchor}

#### fn [calculate_pnl](../../pnl/trait.PnLCalculator.html#tymethod.calculate_pnl){.fn}( &self, market_price: &[Positive](../positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, expiration_date: [ExpirationDate](../types/enum.ExpirationDate.html "enum optionstratlib::model::types::ExpirationDate"){.enum}, implied_volatility: &[Positive](../positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, ) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[PnL](../../pnl/utils/struct.PnL.html "struct optionstratlib::pnl::utils::PnL"){.struct}, [Box](https://doc.rust-lang.org/1.86.0/alloc/boxed/struct.Box.html "struct alloc::boxed::Box"){.struct}\<dyn [Error](https://doc.rust-lang.org/1.86.0/core/error/trait.Error.html "trait core::error::Error"){.trait}\>\> {#fn-calculate_pnl-self-market_price-positive-expiration_date-expirationdate-implied_volatility-positive---resultpnl-boxdyn-error .code-header}
:::

::: docblock
Calculates the current PnL based on market conditions. [Read
more](../../pnl/trait.PnLCalculator.html#tymethod.calculate_pnl)
:::

::: {#method.calculate_pnl_at_expiration .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/option.rs.html#702-721){.src
.rightside}[§](#method.calculate_pnl_at_expiration){.anchor}

#### fn [calculate_pnl_at_expiration](../../pnl/trait.PnLCalculator.html#tymethod.calculate_pnl_at_expiration){.fn}( &self, underlying_price: &[Positive](../positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, ) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[PnL](../../pnl/utils/struct.PnL.html "struct optionstratlib::pnl::utils::PnL"){.struct}, [Box](https://doc.rust-lang.org/1.86.0/alloc/boxed/struct.Box.html "struct alloc::boxed::Box"){.struct}\<dyn [Error](https://doc.rust-lang.org/1.86.0/core/error/trait.Error.html "trait core::error::Error"){.trait}\>\> {#fn-calculate_pnl_at_expiration-self-underlying_price-positive---resultpnl-boxdyn-error .code-header}
:::

::: docblock
Calculates the PnL at the expiration of the instrument. [Read
more](../../pnl/trait.PnLCalculator.html#tymethod.calculate_pnl_at_expiration)
:::

::: {#method.adjustments_pnl .section .method .trait-impl}
[Source](../../../src/optionstratlib/pnl/traits.rs.html#70-72){.src
.rightside}[§](#method.adjustments_pnl){.anchor}

#### fn [adjustments_pnl](../../pnl/trait.PnLCalculator.html#method.adjustments_pnl){.fn}( &self, \_adjustments: &[DeltaAdjustment](../../strategies/delta_neutral/enum.DeltaAdjustment.html "enum optionstratlib::strategies::delta_neutral::DeltaAdjustment"){.enum}, ) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[PnL](../../pnl/utils/struct.PnL.html "struct optionstratlib::pnl::utils::PnL"){.struct}, [Box](https://doc.rust-lang.org/1.86.0/alloc/boxed/struct.Box.html "struct alloc::boxed::Box"){.struct}\<dyn [Error](https://doc.rust-lang.org/1.86.0/core/error/trait.Error.html "trait core::error::Error"){.trait}\>\> {#fn-adjustments_pnl-self-_adjustments-deltaadjustment---resultpnl-boxdyn-error .code-header}
:::

::: docblock
Calculates the Profit and Loss (PnL) for a series of delta adjustments
in a trading strategy. [Read
more](../../pnl/trait.PnLCalculator.html#method.adjustments_pnl)
:::
:::::::::

::: {#impl-Profit-for-Options .section .impl}
[Source](../../../src/optionstratlib/model/option.rs.html#724-728){.src
.rightside}[§](#impl-Profit-for-Options){.anchor}

### impl [Profit](../../pricing/trait.Profit.html "trait optionstratlib::pricing::Profit"){.trait} for [Options](struct.Options.html "struct optionstratlib::model::option::Options"){.struct} {#impl-profit-for-options .code-header}
:::

::::::: impl-items
::: {#method.calculate_profit_at .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/option.rs.html#725-727){.src
.rightside}[§](#method.calculate_profit_at){.anchor}

#### fn [calculate_profit_at](../../pricing/trait.Profit.html#tymethod.calculate_profit_at){.fn}( &self, price: [Positive](../positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, ) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<Decimal, [Box](https://doc.rust-lang.org/1.86.0/alloc/boxed/struct.Box.html "struct alloc::boxed::Box"){.struct}\<dyn [Error](https://doc.rust-lang.org/1.86.0/core/error/trait.Error.html "trait core::error::Error"){.trait}\>\> {#fn-calculate_profit_at-self-price-positive---resultdecimal-boxdyn-error .code-header}
:::

::: docblock
Calculates the profit at a specified price. [Read
more](../../pricing/trait.Profit.html#tymethod.calculate_profit_at)
:::

::: {#method.get_point_at_price .section .method .trait-impl}
[Source](../../../src/optionstratlib/pricing/payoff.rs.html#222-238){.src
.rightside}[§](#method.get_point_at_price){.anchor}

#### fn [get_point_at_price](../../pricing/trait.Profit.html#method.get_point_at_price){.fn}(&self, price: [Positive](../positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}) -\> ChartPoint\<([f64](https://doc.rust-lang.org/1.86.0/std/primitive.f64.html){.primitive}, [f64](https://doc.rust-lang.org/1.86.0/std/primitive.f64.html){.primitive})\> {#fn-get_point_at_priceself-price-positive---chartpointf64-f64 .code-header}
:::

::: docblock
Creates a chart point representation of the profit at the given price.
[Read more](../../pricing/trait.Profit.html#method.get_point_at_price)
:::
:::::::

::: {#impl-Serialize-for-Options .section .impl}
[Source](../../../src/optionstratlib/model/option.rs.html#60){.src
.rightside}[§](#impl-Serialize-for-Options){.anchor}

### impl [Serialize](https://docs.rs/serde/1.0.219/serde/ser/trait.Serialize.html "trait serde::ser::Serialize"){.trait} for [Options](struct.Options.html "struct optionstratlib::model::option::Options"){.struct} {#impl-serialize-for-options .code-header}
:::

:::::: impl-items
:::: {#method.serialize .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/option.rs.html#60){.src
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

::: {#impl-StructuralPartialEq-for-Options .section .impl}
[Source](../../../src/optionstratlib/model/option.rs.html#60){.src
.rightside}[§](#impl-StructuralPartialEq-for-Options){.anchor}

### impl [StructuralPartialEq](https://doc.rust-lang.org/1.86.0/core/marker/trait.StructuralPartialEq.html "trait core::marker::StructuralPartialEq"){.trait} for [Options](struct.Options.html "struct optionstratlib::model::option::Options"){.struct} {#impl-structuralpartialeq-for-options .code-header}
:::
::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::

## Auto Trait Implementations[§](#synthetic-implementations){.anchor} {#synthetic-implementations .section-header}

::::::::: {#synthetic-implementations-list}
::: {#impl-Freeze-for-Options .section .impl}
[§](#impl-Freeze-for-Options){.anchor}

### impl [Freeze](https://doc.rust-lang.org/1.86.0/core/marker/trait.Freeze.html "trait core::marker::Freeze"){.trait} for [Options](struct.Options.html "struct optionstratlib::model::option::Options"){.struct} {#impl-freeze-for-options .code-header}
:::

::: {#impl-RefUnwindSafe-for-Options .section .impl}
[§](#impl-RefUnwindSafe-for-Options){.anchor}

### impl [RefUnwindSafe](https://doc.rust-lang.org/1.86.0/core/panic/unwind_safe/trait.RefUnwindSafe.html "trait core::panic::unwind_safe::RefUnwindSafe"){.trait} for [Options](struct.Options.html "struct optionstratlib::model::option::Options"){.struct} {#impl-refunwindsafe-for-options .code-header}
:::

::: {#impl-Send-for-Options .section .impl}
[§](#impl-Send-for-Options){.anchor}

### impl [Send](https://doc.rust-lang.org/1.86.0/core/marker/trait.Send.html "trait core::marker::Send"){.trait} for [Options](struct.Options.html "struct optionstratlib::model::option::Options"){.struct} {#impl-send-for-options .code-header}
:::

::: {#impl-Sync-for-Options .section .impl}
[§](#impl-Sync-for-Options){.anchor}

### impl [Sync](https://doc.rust-lang.org/1.86.0/core/marker/trait.Sync.html "trait core::marker::Sync"){.trait} for [Options](struct.Options.html "struct optionstratlib::model::option::Options"){.struct} {#impl-sync-for-options .code-header}
:::

::: {#impl-Unpin-for-Options .section .impl}
[§](#impl-Unpin-for-Options){.anchor}

### impl [Unpin](https://doc.rust-lang.org/1.86.0/core/marker/trait.Unpin.html "trait core::marker::Unpin"){.trait} for [Options](struct.Options.html "struct optionstratlib::model::option::Options"){.struct} {#impl-unpin-for-options .code-header}
:::

::: {#impl-UnwindSafe-for-Options .section .impl}
[§](#impl-UnwindSafe-for-Options){.anchor}

### impl [UnwindSafe](https://doc.rust-lang.org/1.86.0/core/panic/unwind_safe/trait.UnwindSafe.html "trait core::panic::unwind_safe::UnwindSafe"){.trait} for [Options](struct.Options.html "struct optionstratlib::model::option::Options"){.struct} {#impl-unwindsafe-for-options .code-header}
:::
:::::::::

## Blanket Implementations[§](#blanket-implementations){.anchor} {#blanket-implementations .section-header}

:::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::: {#blanket-implementations-list}
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
::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
:::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
