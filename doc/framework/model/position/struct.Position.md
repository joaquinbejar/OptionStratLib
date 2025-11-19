:::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::: width-limiter
::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../../index.html)::[model](../index.html)::[position](index.html)
:::

# Struct [Position]{.struct} Copy item path

[[Source](../../../src/optionstratlib/model/position.rs.html#59-85){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub struct Position {
    pub option: Options,
    pub premium: Positive,
    pub date: DateTime<Utc>,
    pub open_fee: Positive,
    pub close_fee: Positive,
    pub epic: Option<String>,
    pub extra_fields: Option<Value>,
}
```

Expand description

:::: docblock
The `Position` struct represents a financial position in an options
market.

This structure encapsulates all the necessary information to track an
options position, including the underlying option details, costs
associated with the position, and the date when the position was opened.
It provides methods for analyzing profitability, time metrics, and
position characteristics.

## [§](#examples){.doc-anchor}Examples

::: example-wrap
``` {.rust .rust-example-rendered}
use optionstratlib::{Options, pos, Side, OptionStyle};
use chrono::Utc;
use tracing::info;
use optionstratlib::model::Position;
use optionstratlib::model::utils::create_sample_option_simplest;

let option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
let position = Position::new(
    option,
    pos!(5.25),           // premium per contract
    Utc::now(),           // position open date
    pos!(0.65),           // opening fee per contract
    pos!(0.65),           // closing fee per contract
    None,
    None,
);

let total_cost = position.total_cost().unwrap();
info!("Total position cost: {}", total_cost);
```
:::
::::

## Fields[§](#fields){.anchor} {#fields .fields .section-header}

[[§](#structfield.option){.anchor
.field}`option: `[`Options`](../option/struct.Options.html "struct optionstratlib::model::option::Options"){.struct}]{#structfield.option
.structfield .section-header}

::: docblock
The detailed options contract information, including the type, strike
price, expiration, underlying asset details, and other option-specific
parameters.
:::

[[§](#structfield.premium){.anchor
.field}`premium: `[`Positive`](../positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}]{#structfield.premium
.structfield .section-header}

::: docblock
The premium paid or received per contract. For long positions, this
represents the cost per contract; for short positions, this is the
credit received.
:::

[[§](#structfield.date){.anchor
.field}`date: `[`DateTime`](../../../chrono/datetime/struct.DateTime.html "struct chrono::datetime::DateTime"){.struct}`<`[`Utc`](../../prelude/struct.Utc.html "struct optionstratlib::prelude::Utc"){.struct}`>`]{#structfield.date
.structfield .section-header}

::: docblock
The date and time when the position was opened, used for calculating
time-based metrics like days held and days to expiration.
:::

[[§](#structfield.open_fee){.anchor
.field}`open_fee: `[`Positive`](../positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}]{#structfield.open_fee
.structfield .section-header}

::: docblock
The fee paid to open the position per contract. This typically includes
broker commissions and exchange fees.
:::

[[§](#structfield.close_fee){.anchor
.field}`close_fee: `[`Positive`](../positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}]{#structfield.close_fee
.structfield .section-header}

::: docblock
The fee that will be paid to close the position per contract. This is
used in profit/loss calculations to account for all transaction costs.
:::

[[§](#structfield.epic){.anchor
.field}`epic: `[`Option`](https://doc.rust-lang.org/1.91.1/core/option/enum.Option.html "enum core::option::Option"){.enum}`<`[`String`](https://doc.rust-lang.org/1.91.1/alloc/string/struct.String.html "struct alloc::string::String"){.struct}`>`]{#structfield.epic
.structfield .section-header}

::: docblock
Identifier for the position in an external system or platform
:::

[[§](#structfield.extra_fields){.anchor
.field}`extra_fields: `[`Option`](https://doc.rust-lang.org/1.91.1/core/option/enum.Option.html "enum core::option::Option"){.enum}`<`[`Value`](../../../serde_json/value/enum.Value.html "enum serde_json::value::Value"){.enum}`>`]{#structfield.extra_fields
.structfield .section-header}

::: docblock
Additional custom data fields for the position stored as JSON
:::

## Implementations[§](#implementations){.anchor} {#implementations .section-header}

:::::::::::::::::::::::::::::::::::::: {#implementations-list}
::: {#impl-Position .section .impl}
[Source](../../../src/optionstratlib/model/position.rs.html#87-655){.src
.rightside}[§](#impl-Position){.anchor}

### impl [Position](struct.Position.html "struct optionstratlib::model::position::Position"){.struct} {#impl-position .code-header}
:::

:::::::::::::::::::::::::::::::::::: impl-items
::: {#method.new .section .method}
[Source](../../../src/optionstratlib/model/position.rs.html#133-151){.src
.rightside}

#### pub fn [new](#method.new){.fn}( option: [Options](../option/struct.Options.html "struct optionstratlib::model::option::Options"){.struct}, premium: [Positive](../positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, date: [DateTime](../../../chrono/datetime/struct.DateTime.html "struct chrono::datetime::DateTime"){.struct}\<[Utc](../../prelude/struct.Utc.html "struct optionstratlib::prelude::Utc"){.struct}\>, open_fee: [Positive](../positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, close_fee: [Positive](../positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, epic: [Option](https://doc.rust-lang.org/1.91.1/core/option/enum.Option.html "enum core::option::Option"){.enum}\<[String](https://doc.rust-lang.org/1.91.1/alloc/string/struct.String.html "struct alloc::string::String"){.struct}\>, extra_fields: [Option](https://doc.rust-lang.org/1.91.1/core/option/enum.Option.html "enum core::option::Option"){.enum}\<[Value](../../../serde_json/value/enum.Value.html "enum serde_json::value::Value"){.enum}\>, ) -\> Self {#pub-fn-new-option-options-premium-positive-date-datetimeutc-open_fee-positive-close_fee-positive-epic-optionstring-extra_fields-optionvalue---self .code-header}
:::

:::: docblock
Creates a new options position.

This constructor initializes a new `Position` instance representing an
options trade, capturing all essential information for position tracking
and analysis.

##### [§](#parameters){.doc-anchor}Parameters

- `option` - The options contract details including type (call/put),
  strike price, expiration date, underlying asset information, and other
  option parameters.

- `premium` - The premium paid (for long positions) or received (for
  short positions) per contract, represented as a positive value.

- `date` - The timestamp when the position was opened, used for
  calculating time-based metrics like days to expiration and position
  duration.

- `open_fee` - The transaction costs paid to open the position per
  contract, including broker commissions and exchange fees.

- `close_fee` - The anticipated transaction costs to close the position
  per contract, used for accurate profit/loss calculations.

##### [§](#returns){.doc-anchor}Returns

Returns a new `Position` instance containing the provided information.

##### [§](#examples-1){.doc-anchor}Examples

::: example-wrap
``` {.rust .rust-example-rendered}
use optionstratlib::{Options, pos, Side, OptionStyle};
use chrono::Utc;
use optionstratlib::model::Position;
use optionstratlib::model::utils::create_sample_option_simplest;

let option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
let position = Position::new(
    option,
    pos!(5.25),           // premium per contract
    Utc::now(),           // position open date
    pos!(0.65),           // opening fee per contract
    pos!(0.65),           // closing fee per contract
   None,                  // epic (optional)
  None,                  // extra fields (optional)
);
```
:::
::::

::: {#method.total_cost .section .method}
[Source](../../../src/optionstratlib/model/position.rs.html#213-220){.src
.rightside}

#### pub fn [total_cost](#method.total_cost){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Positive](../positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, [PositionError](../../error/position/enum.PositionError.html "enum optionstratlib::error::position::PositionError"){.enum}\> {#pub-fn-total_costself---resultpositive-positionerror .code-header}
:::

::: docblock
Calculates the total cost of the position based on the option's side and
fees.

Depending on whether the position is long or short, different components
contribute to the total cost calculation:

- For a long position, the total cost includes the premium, open fee,
  and close fee multiplied by the option's quantity.
- For a short position, the total cost includes only the open fee and
  close fee multiplied by the option's quantity.

##### [§](#returns-1){.doc-anchor}Returns

A `f64` representing the total cost of the position. THE VALUE IS ALWAYS
POSITIVE
:::

::: {#method.premium_received .section .method}
[Source](../../../src/optionstratlib/model/position.rs.html#260-265){.src
.rightside}

#### pub fn [premium_received](#method.premium_received){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Positive](../positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, [PositionError](../../error/position/enum.PositionError.html "enum optionstratlib::error::position::PositionError"){.enum}\> {#pub-fn-premium_receivedself---resultpositive-positionerror .code-header}
:::

:::: docblock
Calculates the premium received from an options position.

This method determines the premium amount received based on the
position's side:

- For long positions, it returns zero as the trader pays premium
  (doesn't receive any)
- For short positions, it returns the total premium received (premium
  per contract × quantity)

The result is always returned as a `Positive` value, ensuring
non-negative amounts.

##### [§](#returns-2){.doc-anchor}Returns

- `Result<Positive, PositionError>` - A result containing the premium
  received as a `Positive` value if successful, or a `PositionError` if
  any calculation errors occur.

##### [§](#examples-2){.doc-anchor}Examples

::: example-wrap
``` {.rust .rust-example-rendered}
use optionstratlib::{pos, Side, OptionStyle};
use optionstratlib::model::Position;
use optionstratlib::model::utils::create_sample_option_simplest;
use chrono::Utc;
use tracing::info;

// Create a short position
let option = create_sample_option_simplest(OptionStyle::Call, Side::Short);
let position = Position::new(
    option,
    pos!(5.25),  // premium per contract
    Utc::now(),  // position open date
    pos!(0.65),  // opening fee
    pos!(0.65),  // closing fee
   None,        // epic (optional)
  None,        // extra fields (optional)
);

// Calculate premium received
let received = position.premium_received().unwrap();
info!("Premium received: {}", received);
```
:::
::::

::: {#method.net_premium_received .section .method}
[Source](../../../src/optionstratlib/model/position.rs.html#285-302){.src
.rightside}

#### pub fn [net_premium_received](#method.net_premium_received){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Positive](../positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, [PositionError](../../error/position/enum.PositionError.html "enum optionstratlib::error::position::PositionError"){.enum}\> {#pub-fn-net_premium_receivedself---resultpositive-positionerror .code-header}
:::

::: docblock
Calculates the net premium received for the position.

This method determines the premium amount received after accounting for
costs, which is relevant primarily for short positions. For long
positions, this always returns zero as premium is paid rather than
received.

For short positions, the method calculates the difference between the
premium received and the total costs incurred. If this value is positive
(meaning the premium exceeds the costs), it represents the maximum
potential profit for the position. If negative, the position is
considered invalid as it would represent a guaranteed loss.

##### [§](#returns-3){.doc-anchor}Returns

- `Ok(Positive)` - The net premium received as a non-negative value
- `Err(PositionError)` - If the position is invalid because the premium
  received is less than the costs, resulting in a guaranteed loss
:::

::: {#method.pnl_at_expiration .section .method}
[Source](../../../src/optionstratlib/model/position.rs.html#349-357){.src
.rightside}

#### pub fn [pnl_at_expiration](#method.pnl_at_expiration){.fn}( &self, price: &[Option](https://doc.rust-lang.org/1.91.1/core/option/enum.Option.html "enum core::option::Option"){.enum}\<&[Positive](../positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\>, ) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Decimal](../../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}, [PricingError](../../error/pricing/enum.PricingError.html "enum optionstratlib::error::pricing::PricingError"){.enum}\> {#pub-fn-pnl_at_expiration-self-price-optionpositive---resultdecimal-pricingerror .code-header}
:::

:::: docblock
Calculates the profit and loss (PnL) at the option's expiration.

This function determines the total profit or loss that would be realized
when the option position expires, taking into account the intrinsic
value at expiration, the cost to establish the position, and any
premiums received.

##### [§](#arguments){.doc-anchor}Arguments

- `price` - An optional reference to a positive decimal value
  representing the underlying asset price at expiration. If None is
  provided, the calculation will use the current underlying price stored
  in the option.

##### [§](#returns-4){.doc-anchor}Returns

- `Result<Decimal, PricingError>` - The calculated profit or loss as a
  Decimal value, or an error if the calculation fails.

##### [§](#examples-3){.doc-anchor}Examples

::: example-wrap
``` {.rust .rust-example-rendered}

// Assuming position is a properly initialized Position
use chrono::Utc;
use optionstratlib::model::utils::create_sample_option_simplest;
use optionstratlib::{pos, OptionStyle, Side};
use optionstratlib::model::Position;

let option = create_sample_option_simplest(OptionStyle::Call, Side::Short);
let position = Position::new(
    option,
    pos!(5.25),  // premium per contract
    Utc::now(),  // position open date
    pos!(0.65),  // opening fee
    pos!(0.65),  // closing fee
   None,        // epic (optional)
  None,        // extra fields (optional)
);
let current_price = pos!(105.0);

// Calculate PnL at expiration with specified price
let pnl_specific = position.pnl_at_expiration(&Some(&current_price)).unwrap();

// Calculate PnL at expiration using the option's current underlying price
let pnl_current = position.pnl_at_expiration(&None).unwrap();
```
:::
::::

::: {#method.unrealized_pnl .section .method}
[Source](../../../src/optionstratlib/model/position.rs.html#400-413){.src
.rightside}

#### pub fn [unrealized_pnl](#method.unrealized_pnl){.fn}(&self, price: [Positive](../positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Decimal](../../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}, [PositionError](../../error/position/enum.PositionError.html "enum optionstratlib::error::position::PositionError"){.enum}\> {#pub-fn-unrealized_pnlself-price-positive---resultdecimal-positionerror .code-header}
:::

:::: docblock
Calculates the unrealized profit and loss (PnL) for an options position
at a given price.

This method computes the current theoretical profit or loss of the
position if it were to be closed at the specified price, taking into
account the premium paid/received and all transaction fees (both opening
and closing fees).

The calculation differs based on the position side:

- For long positions: (current_price - premium - open_fee - close_fee)
  \* quantity
- For short positions: (premium - current_price - open_fee - close_fee)
  \* quantity

##### [§](#parameters-1){.doc-anchor}Parameters

- `price` - A `Positive` value representing the current price of the
  option

##### [§](#returns-5){.doc-anchor}Returns

- `Result<Decimal, PositionError>` - The calculated unrealized PnL as a
  `Decimal` if successful, or a `PositionError` if the calculation fails

##### [§](#example){.doc-anchor}Example

::: example-wrap
``` {.rust .rust-example-rendered}
use chrono::Utc;
use tracing::info;
use optionstratlib::model::Position;
use optionstratlib::model::utils::create_sample_option_simplest;
use optionstratlib::{pos, OptionStyle, Side};
let current_price = pos!(6.50);
let option = create_sample_option_simplest(OptionStyle::Call, Side::Short);
let position = Position::new(
    option,
    pos!(5.25),  // premium per contract
    Utc::now(),  // position open date
    pos!(0.65),  // opening fee
    pos!(0.65),  // closing fee
   None,        // epic (optional)
  None,        // extra fields (optional)
);
let unrealized_pnl = position.unrealized_pnl(current_price).unwrap();
info!("Current unrealized PnL: {}", unrealized_pnl);
```
:::
::::

::: {#method.days_held .section .method}
[Source](../../../src/optionstratlib/model/position.rs.html#428-430){.src
.rightside}

#### pub fn [days_held](#method.days_held){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Positive](../positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, [PositionError](../../error/position/enum.PositionError.html "enum optionstratlib::error::position::PositionError"){.enum}\> {#pub-fn-days_heldself---resultpositive-positionerror .code-header}
:::

::: docblock
Calculates the number of days the position has been held.

This method computes the difference between the current UTC date and the
position's opening date, returning the result as a `Positive` value.

The calculation uses Chrono's `num_days` method to determine the precise
number of whole days between the position's date and current time.

##### [§](#returns-6){.doc-anchor}Returns

- `Ok(Positive)` - The number of days the position has been held as a
  positive value
- `Err(PositionError)` - If there's an error during the calculation or
  validation
:::

::: {#method.days_to_expiration .section .method}
[Source](../../../src/optionstratlib/model/position.rs.html#445-452){.src
.rightside}

#### pub fn [days_to_expiration](#method.days_to_expiration){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Positive](../positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, [PositionError](../../error/position/enum.PositionError.html "enum optionstratlib::error::position::PositionError"){.enum}\> {#pub-fn-days_to_expirationself---resultpositive-positionerror .code-header}
:::

::: docblock
Calculates the number of days remaining until the option expires.

This function determines the time to expiration in days based on the
option's expiration date format. It handles both explicit day counts and
datetime-based expiration dates.

##### [§](#returns-7){.doc-anchor}Returns

- `Ok(Positive)` - The number of days to expiration as a positive value
- `Err(PositionError)` - If the calculation fails due to issues with the
  position data

For datetime-based expirations, the function calculates the difference
between the expiration date and the current date, converting the result
to days.
:::

::: {#method.is_long .section .method}
[Source](../../../src/optionstratlib/model/position.rs.html#464-469){.src
.rightside}

#### pub fn [is_long](#method.is_long){.fn}(&self) -\> [bool](https://doc.rust-lang.org/1.91.1/std/primitive.bool.html){.primitive} {#pub-fn-is_longself---bool .code-header}
:::

::: docblock
Determines if the position is a long position.

This method checks the side attribute of the option to determine the
directionality of the position. Long positions profit when the
underlying asset's price increases.

##### [§](#returns-8){.doc-anchor}Returns

- `true` if the position is long
- `false` if the position is short
:::

::: {#method.is_short .section .method}
[Source](../../../src/optionstratlib/model/position.rs.html#481-486){.src
.rightside}

#### pub fn [is_short](#method.is_short){.fn}(&self) -\> [bool](https://doc.rust-lang.org/1.91.1/std/primitive.bool.html){.primitive} {#pub-fn-is_shortself---bool .code-header}
:::

::: docblock
Determines if the position is a short position.

This method checks the side attribute of the option to determine the
directionality of the position. Short positions profit when the
underlying asset's price decreases.

##### [§](#returns-9){.doc-anchor}Returns

- `true` if the position is short
- `false` if the position is long
:::

::: {#method.net_cost .section .method}
[Source](../../../src/optionstratlib/model/position.rs.html#502-511){.src
.rightside}

#### pub fn [net_cost](#method.net_cost){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Decimal](../../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}, [PositionError](../../error/position/enum.PositionError.html "enum optionstratlib::error::position::PositionError"){.enum}\> {#pub-fn-net_costself---resultdecimal-positionerror .code-header}
:::

::: docblock
Calculates the net cost of the position based on the option's side and
fees.

This method calculates the net cost of a position by determining whether
the position is long or short and then computing the respective costs:

- For a long position, the net cost is equivalent to the `total_cost()`
  of the position.
- For a short position, the net cost is calculated by subtracting the
  premium from the sum of the open and close fees, and then multiplying
  the result by the option's quantity.

##### [§](#returns-10){.doc-anchor}Returns

A `Decimal` representing the net cost of the position. The value should
be positive but if the fee is higher than the premium it will be
negative in short positions
:::

::: {#method.break_even .section .method}
[Source](../../../src/optionstratlib/model/position.rs.html#534-553){.src
.rightside}

#### pub fn [break_even](#method.break_even){.fn}(&self) -\> [Option](https://doc.rust-lang.org/1.91.1/core/option/enum.Option.html "enum core::option::Option"){.enum}\<[Positive](../positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\> {#pub-fn-break_evenself---optionpositive .code-header}
:::

::: docblock
Calculates the break-even price for an options position.

This method determines the price of the underlying asset at which the
position will neither make a profit nor a loss. The calculation varies
based on both the side of the position (Long/Short) and the option style
(Call/Put).

The break-even price is an important reference point for options traders
as it represents the threshold price that the underlying asset must
cross for the position to become profitable, accounting for all costs
associated with the position.

##### [§](#formula-by-position-type){.doc-anchor}Formula by position type:

- Long Call: Strike Price + Total Cost per Contract
- Short Call: Strike Price + Premium - Total Cost per Contract
- Long Put: Strike Price - Total Cost per Contract
- Short Put: Strike Price - Premium + Total Cost per Contract

##### [§](#returns-11){.doc-anchor}Returns

- `Some(Positive)` containing the break-even price if the position has
  non-zero quantity
- `None` if the position has zero quantity (no contracts)
:::

::: {#method.fees .section .method}
[Source](../../../src/optionstratlib/model/position.rs.html#605-607){.src
.rightside}

#### pub fn [fees](#method.fees){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Positive](../positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, [PositionError](../../error/position/enum.PositionError.html "enum optionstratlib::error::position::PositionError"){.enum}\> {#pub-fn-feesself---resultpositive-positionerror .code-header}
:::

::: docblock
Calculates the total transaction fees for the position.

This method computes the sum of opening and closing fees for the
position, scaled by the quantity of options contracts. These fees
typically include broker commissions, exchange fees, and other
transaction costs.

##### [§](#returns-12){.doc-anchor}Returns

- `Ok(Positive)` - The total fees as a positive value
- `Err(PositionError)` - If there's an issue calculating the fees
:::

::: {#method.validate .section .method}
[Source](../../../src/optionstratlib/model/position.rs.html#644-654){.src
.rightside}

#### pub fn [validate](#method.validate){.fn}(&self) -\> [bool](https://doc.rust-lang.org/1.91.1/std/primitive.bool.html){.primitive} {#pub-fn-validateself---bool .code-header}
:::

:::: docblock
Validates the position to ensure it meets all necessary conditions for
trading.

This method performs a series of checks to determine if the position is
valid:

1.  For short positions, verifies that:
    - Premium is greater than zero
    - Premium exceeds the sum of opening and closing fees
2.  Validates the underlying option parameters

##### [§](#returns-13){.doc-anchor}Returns

- `true` if the position is valid and meets all conditions
- `false` otherwise, with specific failure reasons logged via debug
  messages

##### [§](#examples-4){.doc-anchor}Examples

::: example-wrap
``` {.rust .rust-example-rendered}
use optionstratlib::model::{Position, Options};
use optionstratlib::{pos, Side, OptionStyle};
use optionstratlib::model::utils::create_sample_option_simplest;
use chrono::Utc;

// Create a valid position
let option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
let position = Position::new(
    option,
    pos!(5.25),
    Utc::now(),
    pos!(0.65),
    pos!(0.65),
    None, // epic (optional)
   None, // extra fields (optional)
);

assert!(position.validate());
```
:::
::::
::::::::::::::::::::::::::::::::::::
::::::::::::::::::::::::::::::::::::::

## Trait Implementations[§](#trait-implementations){.anchor} {#trait-implementations .section-header}

::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::: {#trait-implementations-list}
::: {#impl-BasicAble-for-Position .section .impl}
[Source](../../../src/optionstratlib/model/position.rs.html#997-1072){.src
.rightside}[§](#impl-BasicAble-for-Position){.anchor}

### impl [BasicAble](../../strategies/base/trait.BasicAble.html "trait optionstratlib::strategies::base::BasicAble"){.trait} for [Position](struct.Position.html "struct optionstratlib::model::position::Position"){.struct} {#impl-basicable-for-position .code-header}
:::

::::::::::::::::::::::::::::::::::::::::: impl-items
::: {#method.set_implied_volatility .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/position.rs.html#1069-1071){.src
.rightside}[§](#method.set_implied_volatility){.anchor}

#### fn [set_implied_volatility](../../strategies/base/trait.BasicAble.html#method.set_implied_volatility){.fn}( &mut self, \_volatility: &[Positive](../positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, ) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[()](https://doc.rust-lang.org/1.91.1/std/primitive.unit.html){.primitive}, [StrategyError](../../error/strategies/enum.StrategyError.html "enum optionstratlib::error::strategies::StrategyError"){.enum}\> {#fn-set_implied_volatility-mut-self-_volatility-positive---result-strategyerror .code-header}
:::

::: docblock
Updates the volatility for the strategy.

##### [§](#parameters-3){.doc-anchor}Parameters {#parameters-3}

- `_volatility`: A reference to a `Positive` value representing the new
  volatility to set.

##### [§](#returns-18){.doc-anchor}Returns {#returns-18}

- `Ok(())`: If the update operation succeeds (currently unimplemented).
- `Err(StrategyError)`: If there is an error during the update process
  (place-holder as functionality is not implemented).

##### [§](#notes){.doc-anchor}Notes

This method is currently unimplemented, and calling it will result in
the `unimplemented!` macro being triggered, which causes a panic. This
function is a stub and should be implemented to handle setting the
volatility specific to the strategy.
:::

::: {#method.get_title .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/position.rs.html#998-1000){.src
.rightside}[§](#method.get_title){.anchor}

#### fn [get_title](../../strategies/base/trait.BasicAble.html#method.get_title){.fn}(&self) -\> [String](https://doc.rust-lang.org/1.91.1/alloc/string/struct.String.html "struct alloc::string::String"){.struct} {#fn-get_titleself---string .code-header}
:::

::: docblock
Retrieves the title associated with the current instance of the
strategy. [Read
more](../../strategies/base/trait.BasicAble.html#method.get_title)
:::

::: {#method.get_option_basic_type .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/position.rs.html#1001-1003){.src
.rightside}[§](#method.get_option_basic_type){.anchor}

#### fn [get_option_basic_type](../../strategies/base/trait.BasicAble.html#method.get_option_basic_type){.fn}(&self) -\> [HashSet](https://doc.rust-lang.org/1.91.1/std/collections/hash/set/struct.HashSet.html "struct std::collections::hash::set::HashSet"){.struct}\<[OptionBasicType](../types/struct.OptionBasicType.html "struct optionstratlib::model::types::OptionBasicType"){.struct}\<\'\_\>\> {#fn-get_option_basic_typeself---hashsetoptionbasictype_ .code-header}
:::

::: docblock
Retrieves a `HashSet` of `OptionBasicType` values associated with the
current strategy. [Read
more](../../strategies/base/trait.BasicAble.html#method.get_option_basic_type)
:::

::: {#method.get_symbol .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/position.rs.html#1004-1006){.src
.rightside}[§](#method.get_symbol){.anchor}

#### fn [get_symbol](../../strategies/base/trait.BasicAble.html#method.get_symbol){.fn}(&self) -\> &[str](https://doc.rust-lang.org/1.91.1/std/primitive.str.html){.primitive} {#fn-get_symbolself---str .code-header}
:::

::: docblock
Retrieves the symbol associated with the current instance by delegating
the call to the `get_symbol` method of the `one_option` object. [Read
more](../../strategies/base/trait.BasicAble.html#method.get_symbol)
:::

::: {#method.get_strike .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/position.rs.html#1007-1009){.src
.rightside}[§](#method.get_strike){.anchor}

#### fn [get_strike](../../strategies/base/trait.BasicAble.html#method.get_strike){.fn}(&self) -\> [HashMap](https://doc.rust-lang.org/1.91.1/std/collections/hash/map/struct.HashMap.html "struct std::collections::hash::map::HashMap"){.struct}\<[OptionBasicType](../types/struct.OptionBasicType.html "struct optionstratlib::model::types::OptionBasicType"){.struct}\<\'\_\>, &[Positive](../positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\> {#fn-get_strikeself---hashmapoptionbasictype_-positive .code-header}
:::

::: docblock
Retrieves a mapping of option basic types to their associated positive
strike values. [Read
more](../../strategies/base/trait.BasicAble.html#method.get_strike)
:::

::: {#method.get_strikes .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/position.rs.html#1010-1012){.src
.rightside}[§](#method.get_strikes){.anchor}

#### fn [get_strikes](../../strategies/base/trait.BasicAble.html#method.get_strikes){.fn}(&self) -\> [Vec](https://doc.rust-lang.org/1.91.1/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<&[Positive](../positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\> {#fn-get_strikesself---vecpositive .code-header}
:::

::: docblock
Retrieves a vector of strike prices from the option types. [Read
more](../../strategies/base/trait.BasicAble.html#method.get_strikes)
:::

::: {#method.get_side .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/position.rs.html#1013-1015){.src
.rightside}[§](#method.get_side){.anchor}

#### fn [get_side](../../strategies/base/trait.BasicAble.html#method.get_side){.fn}(&self) -\> [HashMap](https://doc.rust-lang.org/1.91.1/std/collections/hash/map/struct.HashMap.html "struct std::collections::hash::map::HashMap"){.struct}\<[OptionBasicType](../types/struct.OptionBasicType.html "struct optionstratlib::model::types::OptionBasicType"){.struct}\<\'\_\>, &[Side](../types/enum.Side.html "enum optionstratlib::model::types::Side"){.enum}\> {#fn-get_sideself---hashmapoptionbasictype_-side .code-header}
:::

::: docblock
Retrieves a `HashMap` that maps each `OptionBasicType` to its
corresponding `Side`. [Read
more](../../strategies/base/trait.BasicAble.html#method.get_side)
:::

::: {#method.get_type .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/position.rs.html#1016-1018){.src
.rightside}[§](#method.get_type){.anchor}

#### fn [get_type](../../strategies/base/trait.BasicAble.html#method.get_type){.fn}(&self) -\> &[OptionType](../types/enum.OptionType.html "enum optionstratlib::model::types::OptionType"){.enum} {#fn-get_typeself---optiontype .code-header}
:::

::: docblock
Retrieves the type of the option. [Read
more](../../strategies/base/trait.BasicAble.html#method.get_type)
:::

::: {#method.get_style .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/position.rs.html#1019-1021){.src
.rightside}[§](#method.get_style){.anchor}

#### fn [get_style](../../strategies/base/trait.BasicAble.html#method.get_style){.fn}(&self) -\> [HashMap](https://doc.rust-lang.org/1.91.1/std/collections/hash/map/struct.HashMap.html "struct std::collections::hash::map::HashMap"){.struct}\<[OptionBasicType](../types/struct.OptionBasicType.html "struct optionstratlib::model::types::OptionBasicType"){.struct}\<\'\_\>, &[OptionStyle](../types/enum.OptionStyle.html "enum optionstratlib::model::types::OptionStyle"){.enum}\> {#fn-get_styleself---hashmapoptionbasictype_-optionstyle .code-header}
:::

::: docblock
Retrieves a mapping of `OptionBasicType` to their corresponding
`OptionStyle`. [Read
more](../../strategies/base/trait.BasicAble.html#method.get_style)
:::

::: {#method.get_expiration .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/position.rs.html#1022-1024){.src
.rightside}[§](#method.get_expiration){.anchor}

#### fn [get_expiration](../../strategies/base/trait.BasicAble.html#method.get_expiration){.fn}(&self) -\> [HashMap](https://doc.rust-lang.org/1.91.1/std/collections/hash/map/struct.HashMap.html "struct std::collections::hash::map::HashMap"){.struct}\<[OptionBasicType](../types/struct.OptionBasicType.html "struct optionstratlib::model::types::OptionBasicType"){.struct}\<\'\_\>, &[ExpirationDate](../enum.ExpirationDate.html "enum optionstratlib::model::ExpirationDate"){.enum}\> {#fn-get_expirationself---hashmapoptionbasictype_-expirationdate .code-header}
:::

::: docblock
Retrieves a map of option basic types to their corresponding expiration
dates. [Read
more](../../strategies/base/trait.BasicAble.html#method.get_expiration)
:::

::: {#method.get_implied_volatility .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/position.rs.html#1025-1027){.src
.rightside}[§](#method.get_implied_volatility){.anchor}

#### fn [get_implied_volatility](../../strategies/base/trait.BasicAble.html#method.get_implied_volatility){.fn}(&self) -\> [HashMap](https://doc.rust-lang.org/1.91.1/std/collections/hash/map/struct.HashMap.html "struct std::collections::hash::map::HashMap"){.struct}\<[OptionBasicType](../types/struct.OptionBasicType.html "struct optionstratlib::model::types::OptionBasicType"){.struct}\<\'\_\>, &[Positive](../positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\> {#fn-get_implied_volatilityself---hashmapoptionbasictype_-positive .code-header}
:::

::: docblock
Retrieves the implied volatility for the current strategy. [Read
more](../../strategies/base/trait.BasicAble.html#method.get_implied_volatility)
:::

::: {#method.get_quantity .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/position.rs.html#1028-1030){.src
.rightside}[§](#method.get_quantity){.anchor}

#### fn [get_quantity](../../strategies/base/trait.BasicAble.html#method.get_quantity){.fn}(&self) -\> [HashMap](https://doc.rust-lang.org/1.91.1/std/collections/hash/map/struct.HashMap.html "struct std::collections::hash::map::HashMap"){.struct}\<[OptionBasicType](../types/struct.OptionBasicType.html "struct optionstratlib::model::types::OptionBasicType"){.struct}\<\'\_\>, &[Positive](../positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\> {#fn-get_quantityself---hashmapoptionbasictype_-positive .code-header}
:::

::: docblock
Retrieves the quantity information associated with the strategy. [Read
more](../../strategies/base/trait.BasicAble.html#method.get_quantity)
:::

::: {#method.get_underlying_price .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/position.rs.html#1031-1033){.src
.rightside}[§](#method.get_underlying_price){.anchor}

#### fn [get_underlying_price](../../strategies/base/trait.BasicAble.html#method.get_underlying_price){.fn}(&self) -\> &[Positive](../positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#fn-get_underlying_priceself---positive .code-header}
:::

::: docblock
Retrieves the underlying price of the financial instrument (e.g.,
option). [Read
more](../../strategies/base/trait.BasicAble.html#method.get_underlying_price)
:::

::: {#method.get_risk_free_rate .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/position.rs.html#1034-1036){.src
.rightside}[§](#method.get_risk_free_rate){.anchor}

#### fn [get_risk_free_rate](../../strategies/base/trait.BasicAble.html#method.get_risk_free_rate){.fn}(&self) -\> [HashMap](https://doc.rust-lang.org/1.91.1/std/collections/hash/map/struct.HashMap.html "struct std::collections::hash::map::HashMap"){.struct}\<[OptionBasicType](../types/struct.OptionBasicType.html "struct optionstratlib::model::types::OptionBasicType"){.struct}\<\'\_\>, &[Decimal](../../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}\> {#fn-get_risk_free_rateself---hashmapoptionbasictype_-decimal .code-header}
:::

::: docblock
Retrieves the risk-free interest rate associated with a given set of
options. [Read
more](../../strategies/base/trait.BasicAble.html#method.get_risk_free_rate)
:::

::: {#method.get_dividend_yield .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/position.rs.html#1037-1039){.src
.rightside}[§](#method.get_dividend_yield){.anchor}

#### fn [get_dividend_yield](../../strategies/base/trait.BasicAble.html#method.get_dividend_yield){.fn}(&self) -\> [HashMap](https://doc.rust-lang.org/1.91.1/std/collections/hash/map/struct.HashMap.html "struct std::collections::hash::map::HashMap"){.struct}\<[OptionBasicType](../types/struct.OptionBasicType.html "struct optionstratlib::model::types::OptionBasicType"){.struct}\<\'\_\>, &[Positive](../positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\> {#fn-get_dividend_yieldself---hashmapoptionbasictype_-positive .code-header}
:::

::: docblock
Retrieves the dividend yield of a financial option. [Read
more](../../strategies/base/trait.BasicAble.html#method.get_dividend_yield)
:::

::: {#method.one_option .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/position.rs.html#1040-1042){.src
.rightside}[§](#method.one_option){.anchor}

#### fn [one_option](../../strategies/base/trait.BasicAble.html#method.one_option){.fn}(&self) -\> &[Options](../option/struct.Options.html "struct optionstratlib::model::option::Options"){.struct} {#fn-one_optionself---options .code-header}
:::

::: docblock
This method, `one_option`, is designed to retrieve a reference to an
`Options` object. However, in this implementation, the function is not
currently functional, as it explicitly triggers an unimplemented error
when called. [Read
more](../../strategies/base/trait.BasicAble.html#method.one_option)
:::

::: {#method.one_option_mut .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/position.rs.html#1043-1045){.src
.rightside}[§](#method.one_option_mut){.anchor}

#### fn [one_option_mut](../../strategies/base/trait.BasicAble.html#method.one_option_mut){.fn}(&mut self) -\> &mut [Options](../option/struct.Options.html "struct optionstratlib::model::option::Options"){.struct} {#fn-one_option_mutmut-self---mut-options .code-header}
:::

::: docblock
Provides a mutable reference to an `Options` instance. [Read
more](../../strategies/base/trait.BasicAble.html#method.one_option_mut)
:::

::: {#method.set_expiration_date .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/position.rs.html#1047-1052){.src
.rightside}[§](#method.set_expiration_date){.anchor}

#### fn [set_expiration_date](../../strategies/base/trait.BasicAble.html#method.set_expiration_date){.fn}( &mut self, expiration_date: [ExpirationDate](../enum.ExpirationDate.html "enum optionstratlib::model::ExpirationDate"){.enum}, ) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[()](https://doc.rust-lang.org/1.91.1/std/primitive.unit.html){.primitive}, [StrategyError](../../error/strategies/enum.StrategyError.html "enum optionstratlib::error::strategies::StrategyError"){.enum}\> {#fn-set_expiration_date-mut-self-expiration_date-expirationdate---result-strategyerror .code-header}
:::

::: docblock
Sets the expiration date for the strategy. [Read
more](../../strategies/base/trait.BasicAble.html#method.set_expiration_date)
:::

::: {#method.set_underlying_price .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/position.rs.html#1053-1055){.src
.rightside}[§](#method.set_underlying_price){.anchor}

#### fn [set_underlying_price](../../strategies/base/trait.BasicAble.html#method.set_underlying_price){.fn}( &mut self, \_price: &[Positive](../positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, ) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[()](https://doc.rust-lang.org/1.91.1/std/primitive.unit.html){.primitive}, [StrategyError](../../error/strategies/enum.StrategyError.html "enum optionstratlib::error::strategies::StrategyError"){.enum}\> {#fn-set_underlying_price-mut-self-_price-positive---result-strategyerror .code-header}
:::

::: docblock
Sets the underlying price for this strategy. [Read
more](../../strategies/base/trait.BasicAble.html#method.set_underlying_price)
:::
:::::::::::::::::::::::::::::::::::::::::

::: {#impl-Clone-for-Position .section .impl}
[Source](../../../src/optionstratlib/model/position.rs.html#58){.src
.rightside}[§](#impl-Clone-for-Position){.anchor}

### impl [Clone](https://doc.rust-lang.org/1.91.1/core/clone/trait.Clone.html "trait core::clone::Clone"){.trait} for [Position](struct.Position.html "struct optionstratlib::model::position::Position"){.struct} {#impl-clone-for-position .code-header}
:::

::::::: impl-items
::: {#method.clone .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/position.rs.html#58){.src
.rightside}[§](#method.clone){.anchor}

#### fn [clone](https://doc.rust-lang.org/1.91.1/core/clone/trait.Clone.html#tymethod.clone){.fn}(&self) -\> [Position](struct.Position.html "struct optionstratlib::model::position::Position"){.struct} {#fn-cloneself---position .code-header}
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

::: {#impl-ComposeSchema-for-Position .section .impl}
[Source](../../../src/optionstratlib/model/position.rs.html#58){.src
.rightside}[§](#impl-ComposeSchema-for-Position){.anchor}

### impl ComposeSchema for [Position](struct.Position.html "struct optionstratlib::model::position::Position"){.struct} {#impl-composeschema-for-position .code-header}
:::

:::: impl-items
::: {#method.compose .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/position.rs.html#58){.src
.rightside}[§](#method.compose){.anchor}

#### fn [compose](#tymethod.compose){.fn}(generics: [Vec](https://doc.rust-lang.org/1.91.1/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<[RefOr](../../../utoipa/openapi/enum.RefOr.html "enum utoipa::openapi::RefOr"){.enum}\<[Schema](../../../utoipa/openapi/schema/enum.Schema.html "enum utoipa::openapi::schema::Schema"){.enum}\>\>) -\> [RefOr](../../../utoipa/openapi/enum.RefOr.html "enum utoipa::openapi::RefOr"){.enum}\<[Schema](../../../utoipa/openapi/schema/enum.Schema.html "enum utoipa::openapi::schema::Schema"){.enum}\> {#fn-composegenerics-vecreforschema---reforschema .code-header}
:::
::::

::: {#impl-Debug-for-Position .section .impl}
[Source](../../../src/optionstratlib/model/format.rs.html#260-270){.src
.rightside}[§](#impl-Debug-for-Position){.anchor}

### impl [Debug](https://doc.rust-lang.org/1.91.1/core/fmt/trait.Debug.html "trait core::fmt::Debug"){.trait} for [Position](struct.Position.html "struct optionstratlib::model::position::Position"){.struct} {#impl-debug-for-position .code-header}
:::

::::: impl-items
::: {#method.fmt-1 .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/format.rs.html#261-269){.src
.rightside}[§](#method.fmt-1){.anchor}

#### fn [fmt](https://doc.rust-lang.org/1.91.1/core/fmt/trait.Debug.html#tymethod.fmt){.fn}(&self, f: &mut [Formatter](https://doc.rust-lang.org/1.91.1/core/fmt/struct.Formatter.html "struct core::fmt::Formatter"){.struct}\<\'\_\>) -\> [Result](https://doc.rust-lang.org/1.91.1/core/fmt/type.Result.html "type core::fmt::Result"){.type} {#fn-fmtself-f-mut-formatter_---result .code-header}
:::

::: docblock
Formats the value using the given formatter. [Read
more](https://doc.rust-lang.org/1.91.1/core/fmt/trait.Debug.html#tymethod.fmt)
:::
:::::

::: {#impl-Default-for-Position .section .impl}
[Source](../../../src/optionstratlib/model/position.rs.html#657-669){.src
.rightside}[§](#impl-Default-for-Position){.anchor}

### impl [Default](https://doc.rust-lang.org/1.91.1/core/default/trait.Default.html "trait core::default::Default"){.trait} for [Position](struct.Position.html "struct optionstratlib::model::position::Position"){.struct} {#impl-default-for-position .code-header}
:::

::::: impl-items
::: {#method.default .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/position.rs.html#658-668){.src
.rightside}[§](#method.default){.anchor}

#### fn [default](https://doc.rust-lang.org/1.91.1/core/default/trait.Default.html#tymethod.default){.fn}() -\> Self {#fn-default---self .code-header}
:::

::: docblock
Returns the "default value" for a type. [Read
more](https://doc.rust-lang.org/1.91.1/core/default/trait.Default.html#tymethod.default)
:::
:::::

::: {#impl-Deserialize%3C'de%3E-for-Position .section .impl}
[Source](../../../src/optionstratlib/model/position.rs.html#58){.src
.rightside}[§](#impl-Deserialize%3C'de%3E-for-Position){.anchor}

### impl\<\'de\> [Deserialize](../../../serde_core/de/trait.Deserialize.html "trait serde_core::de::Deserialize"){.trait}\<\'de\> for [Position](struct.Position.html "struct optionstratlib::model::position::Position"){.struct} {#implde-deserializede-for-position .code-header}
:::

:::::: impl-items
:::: {#method.deserialize .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/position.rs.html#58){.src
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

::: {#impl-Display-for-Position .section .impl}
[Source](../../../src/optionstratlib/model/format.rs.html#249-258){.src
.rightside}[§](#impl-Display-for-Position){.anchor}

### impl [Display](https://doc.rust-lang.org/1.91.1/core/fmt/trait.Display.html "trait core::fmt::Display"){.trait} for [Position](struct.Position.html "struct optionstratlib::model::position::Position"){.struct} {#impl-display-for-position .code-header}
:::

::::: impl-items
::: {#method.fmt .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/format.rs.html#250-257){.src
.rightside}[§](#method.fmt){.anchor}

#### fn [fmt](https://doc.rust-lang.org/1.91.1/core/fmt/trait.Display.html#tymethod.fmt){.fn}(&self, f: &mut [Formatter](https://doc.rust-lang.org/1.91.1/core/fmt/struct.Formatter.html "struct core::fmt::Formatter"){.struct}\<\'\_\>) -\> [Result](https://doc.rust-lang.org/1.91.1/core/fmt/type.Result.html "type core::fmt::Result"){.type} {#fn-fmtself-f-mut-formatter_---result-1 .code-header}
:::

::: docblock
Formats the value using the given formatter. [Read
more](https://doc.rust-lang.org/1.91.1/core/fmt/trait.Display.html#tymethod.fmt)
:::
:::::

:::: {#impl-Graph-for-Position .section .impl}
[Source](../../../src/optionstratlib/model/position.rs.html#1083-1091){.src
.rightside}[§](#impl-Graph-for-Position){.anchor}

### impl [Graph](../../visualization/trait.Graph.html "trait optionstratlib::visualization::Graph"){.trait} for [Position](struct.Position.html "struct optionstratlib::model::position::Position"){.struct} {#impl-graph-for-position .code-header}

::: docblock
Implementation of the `Graph` trait for the `Position` struct, enabling
graphical representation of financial options positions.
:::
::::

::: docblock
This implementation provides methods to visualize the profit/loss (PnL)
profile of an options position across different price levels of the
underlying asset. It handles the generation of appropriate title, data
values for plotting, and special chart elements like break-even points.

The visualization capabilities allow traders to analyze the potential
outcomes of their options positions at expiration across various price
scenarios.
:::

::::::: impl-items
::: {#method.graph_data .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/position.rs.html#1084-1086){.src
.rightside}[§](#method.graph_data){.anchor}

#### fn [graph_data](../../visualization/trait.Graph.html#tymethod.graph_data){.fn}(&self) -\> [GraphData](../../visualization/enum.GraphData.html "enum optionstratlib::visualization::GraphData"){.enum} {#fn-graph_dataself---graphdata .code-header}
:::

::: docblock
Return the raw data ready for plotting.
:::

::: {#method.graph_config .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/position.rs.html#1088-1090){.src
.rightside}[§](#method.graph_config){.anchor}

#### fn [graph_config](../../visualization/trait.Graph.html#method.graph_config){.fn}(&self) -\> [GraphConfig](../../visualization/struct.GraphConfig.html "struct optionstratlib::visualization::GraphConfig"){.struct} {#fn-graph_configself---graphconfig .code-header}
:::

::: docblock
Optional per‑object configuration overrides.
:::
:::::::

:::: {#impl-Greeks-for-Position .section .impl}
[Source](../../../src/optionstratlib/model/position.rs.html#677-690){.src
.rightside}[§](#impl-Greeks-for-Position){.anchor}

### impl [Greeks](../../greeks/trait.Greeks.html "trait optionstratlib::greeks::Greeks"){.trait} for [Position](struct.Position.html "struct optionstratlib::model::position::Position"){.struct} {#impl-greeks-for-position .code-header}

::: docblock
Implementation of the `Greeks` trait for the `Position` struct.
:::
::::

::: docblock
This implementation allows a `Position` to calculate option Greeks
(delta, gamma, theta, vega, rho, etc.) by accessing its underlying
option contract. The implementation provides a way to expose the
position's option for use in Greek calculations.
:::

::::::::::::::::::::: impl-items
::: {#method.get_options .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/position.rs.html#687-689){.src
.rightside}[§](#method.get_options){.anchor}

#### fn [get_options](../../greeks/trait.Greeks.html#tymethod.get_options){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Vec](https://doc.rust-lang.org/1.91.1/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<&[Options](../option/struct.Options.html "struct optionstratlib::model::option::Options"){.struct}\>, [GreeksError](../../error/greeks/enum.GreeksError.html "enum optionstratlib::error::greeks::GreeksError"){.enum}\> {#fn-get_optionsself---resultvecoptions-greekserror .code-header}
:::

::: docblock
Returns a vector containing a reference to the option contract
associated with this position.

This method satisfies the `Greeks` trait requirement by providing access
to the option contract that will be used for calculating various Greek
values.

##### [§](#returns-14){.doc-anchor}Returns {#returns-14}

- `Ok(Vec<&Options>)` - A vector containing a reference to the
  position's underlying option
- `Err(GreeksError)` - If there is an error accessing the option data
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

::: {#impl-PartialEq-for-Position .section .impl}
[Source](../../../src/optionstratlib/model/position.rs.html#58){.src
.rightside}[§](#impl-PartialEq-for-Position){.anchor}

### impl [PartialEq](https://doc.rust-lang.org/1.91.1/core/cmp/trait.PartialEq.html "trait core::cmp::PartialEq"){.trait} for [Position](struct.Position.html "struct optionstratlib::model::position::Position"){.struct} {#impl-partialeq-for-position .code-header}
:::

::::::: impl-items
::: {#method.eq .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/position.rs.html#58){.src
.rightside}[§](#method.eq){.anchor}

#### fn [eq](https://doc.rust-lang.org/1.91.1/core/cmp/trait.PartialEq.html#tymethod.eq){.fn}(&self, other: &[Position](struct.Position.html "struct optionstratlib::model::position::Position"){.struct}) -\> [bool](https://doc.rust-lang.org/1.91.1/std/primitive.bool.html){.primitive} {#fn-eqself-other-position---bool .code-header}
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

:::: {#impl-PnLCalculator-for-Position .section .impl}
[Source](../../../src/optionstratlib/model/position.rs.html#788-971){.src
.rightside}[§](#impl-PnLCalculator-for-Position){.anchor}

### impl [PnLCalculator](../../pnl/trait.PnLCalculator.html "trait optionstratlib::pnl::PnLCalculator"){.trait} for [Position](struct.Position.html "struct optionstratlib::model::position::Position"){.struct} {#impl-pnlcalculator-for-position .code-header}

::: docblock
#### [§](#position-profit-and-loss-pnl-calculator){.doc-anchor}Position Profit and Loss (PnL) Calculator

This trait implementation provides methods to calculate the profit and
loss (PnL) for option positions under different market scenarios.
:::
::::

::: docblock
The implementation offers two main calculations:

1.  Current PnL based on updated market conditions
2.  PnL at expiration based on a projected underlying price

These calculations are essential for risk management, position
monitoring, and strategy planning in options trading.
:::

::::::::::: impl-items
::: {#method.calculate_pnl .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/position.rs.html#808-832){.src
.rightside}[§](#method.calculate_pnl){.anchor}

#### fn [calculate_pnl](../../pnl/trait.PnLCalculator.html#tymethod.calculate_pnl){.fn}( &self, underlying_price: &[Positive](../positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, expiration_date: [ExpirationDate](../enum.ExpirationDate.html "enum optionstratlib::model::ExpirationDate"){.enum}, implied_volatility: &[Positive](../positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, ) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[PnL](../../pnl/utils/struct.PnL.html "struct optionstratlib::pnl::utils::PnL"){.struct}, [PricingError](../../error/pricing/enum.PricingError.html "enum optionstratlib::error::pricing::PricingError"){.enum}\> {#fn-calculate_pnl-self-underlying_price-positive-expiration_date-expirationdate-implied_volatility-positive---resultpnl-pricingerror .code-header}
:::

::: docblock
Calculates the current unrealized profit and loss for an option position
based on updated market conditions.

This method computes the difference between the option's price at entry
and its current theoretical price using the Black-Scholes model. It
factors in changes to:

- The underlying asset price
- Time to expiration
- Implied volatility

##### [§](#arguments-1){.doc-anchor}Arguments

- `underlying_price` - The current price of the underlying asset
- `expiration_date` - The updated expiration date for the calculation
- `implied_volatility` - The current implied volatility of the option

##### [§](#returns-15){.doc-anchor}Returns {#returns-15}

- `Result<PnL, PricingError>` - A PnL object containing unrealized
  profit/loss and position cost details, or an error if the calculation
  fails
:::

::: {#method.calculate_pnl_at_expiration .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/position.rs.html#849-866){.src
.rightside}[§](#method.calculate_pnl_at_expiration){.anchor}

#### fn [calculate_pnl_at_expiration](../../pnl/trait.PnLCalculator.html#tymethod.calculate_pnl_at_expiration){.fn}( &self, underlying_price: &[Positive](../positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, ) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[PnL](../../pnl/utils/struct.PnL.html "struct optionstratlib::pnl::utils::PnL"){.struct}, [PricingError](../../error/pricing/enum.PricingError.html "enum optionstratlib::error::pricing::PricingError"){.enum}\> {#fn-calculate_pnl_at_expiration-self-underlying_price-positive---resultpnl-pricingerror .code-header}
:::

::: docblock
Calculates the expected profit and loss at option expiration for a given
underlying price.

This method determines the realized profit or loss that would occur if
the option expires with the underlying at the specified price. It uses
intrinsic value calculation at expiration rather than Black-Scholes
pricing.

##### [§](#arguments-2){.doc-anchor}Arguments

- `underlying_price` - The projected price of the underlying asset at
  expiration

##### [§](#returns-16){.doc-anchor}Returns {#returns-16}

- `Result<PnL, PricingError>` - A PnL object containing realized
  profit/loss and position cost details, or an error if the calculation
  fails
:::

::: {#method.diff_position_pnl .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/position.rs.html#868-970){.src
.rightside}[§](#method.diff_position_pnl){.anchor}

#### fn [diff_position_pnl](../../pnl/trait.PnLCalculator.html#method.diff_position_pnl){.fn}(&self, position: &[Position](struct.Position.html "struct optionstratlib::model::position::Position"){.struct}) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[PnL](../../pnl/utils/struct.PnL.html "struct optionstratlib::pnl::utils::PnL"){.struct}, [PricingError](../../error/pricing/enum.PricingError.html "enum optionstratlib::error::pricing::PricingError"){.enum}\> {#fn-diff_position_pnlself-position-position---resultpnl-pricingerror .code-header}
:::

::: docblock
Calculates the profit and loss (PnL) for a given trading position. [Read
more](../../pnl/trait.PnLCalculator.html#method.diff_position_pnl)
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
:::::::::::

:::: {#impl-Profit-for-Position .section .impl}
[Source](../../../src/optionstratlib/model/position.rs.html#978-995){.src
.rightside}[§](#impl-Profit-for-Position){.anchor}

### impl [Profit](../../pricing/trait.Profit.html "trait optionstratlib::pricing::Profit"){.trait} for [Position](struct.Position.html "struct optionstratlib::model::position::Position"){.struct} {#impl-profit-for-position .code-header}

::: docblock
Implementation of the Profit trait for the Position struct.
:::
::::

::: docblock
This allows calculating the profit of a position at a given price by
using the position's profit and loss (PnL) calculation at expiration.
:::

::::::: impl-items
::: {#method.calculate_profit_at .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/position.rs.html#992-994){.src
.rightside}[§](#method.calculate_profit_at){.anchor}

#### fn [calculate_profit_at](../../pricing/trait.Profit.html#tymethod.calculate_profit_at){.fn}(&self, price: &[Positive](../positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Decimal](../../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}, [PricingError](../../error/pricing/enum.PricingError.html "enum optionstratlib::error::pricing::PricingError"){.enum}\> {#fn-calculate_profit_atself-price-positive---resultdecimal-pricingerror .code-header}
:::

::: docblock
Calculates the profit of the position at a specific price.

This method computes the profit or loss that would be realized if the
position were to expire with the underlying asset at the specified
price.

##### [§](#parameters-2){.doc-anchor}Parameters {#parameters-2}

- `price` - The price at which to calculate the profit, represented as a
  Positive value.

##### [§](#returns-17){.doc-anchor}Returns {#returns-17}

- `Result<Decimal, PricingError>` - The calculated profit as a Decimal
  if successful, or an error if the calculation fails.
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

::: {#impl-Serialize-for-Position .section .impl}
[Source](../../../src/optionstratlib/model/position.rs.html#58){.src
.rightside}[§](#impl-Serialize-for-Position){.anchor}

### impl [Serialize](../../../serde_core/ser/trait.Serialize.html "trait serde_core::ser::Serialize"){.trait} for [Position](struct.Position.html "struct optionstratlib::model::position::Position"){.struct} {#impl-serialize-for-position .code-header}
:::

:::::: impl-items
:::: {#method.serialize .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/position.rs.html#58){.src
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

::: {#impl-ToSchema-for-Position .section .impl}
[Source](../../../src/optionstratlib/model/position.rs.html#58){.src
.rightside}[§](#impl-ToSchema-for-Position){.anchor}

### impl [ToSchema](../../../utoipa/trait.ToSchema.html "trait utoipa::ToSchema"){.trait} for [Position](struct.Position.html "struct optionstratlib::model::position::Position"){.struct} {#impl-toschema-for-position .code-header}
:::

::::::: impl-items
::: {#method.name .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/position.rs.html#58){.src
.rightside}[§](#method.name){.anchor}

#### fn [name](../../../utoipa/trait.ToSchema.html#method.name){.fn}() -\> [Cow](https://doc.rust-lang.org/1.91.1/alloc/borrow/enum.Cow.html "enum alloc::borrow::Cow"){.enum}\<\'static, [str](https://doc.rust-lang.org/1.91.1/std/primitive.str.html){.primitive}\> {#fn-name---cowstatic-str .code-header}
:::

::: docblock
Return name of the schema. [Read
more](../../../utoipa/trait.ToSchema.html#method.name)
:::

::: {#method.schemas .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/position.rs.html#58){.src
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

::: {#impl-TradeAble-for-Position .section .impl}
[Source](../../../src/optionstratlib/model/position.rs.html#702-729){.src
.rightside}[§](#impl-TradeAble-for-Position){.anchor}

### impl [TradeAble](../trait.TradeAble.html "trait optionstratlib::model::TradeAble"){.trait} for [Position](struct.Position.html "struct optionstratlib::model::position::Position"){.struct} {#impl-tradeable-for-position .code-header}
:::

::::::::: impl-items
::: {#method.trade .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/position.rs.html#703-720){.src
.rightside}[§](#method.trade){.anchor}

#### fn [trade](../trait.TradeAble.html#tymethod.trade){.fn}(&self) -\> [Trade](../struct.Trade.html "struct optionstratlib::model::Trade"){.struct} {#fn-tradeself---trade .code-header}
:::

::: docblock
Retrieves a reference to a `Trade` instance associated with the current
object. [Read more](../trait.TradeAble.html#tymethod.trade)
:::

::: {#method.trade_ref .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/position.rs.html#722-724){.src
.rightside}[§](#method.trade_ref){.anchor}

#### fn [trade_ref](../trait.TradeAble.html#tymethod.trade_ref){.fn}(&self) -\> &[Trade](../struct.Trade.html "struct optionstratlib::model::Trade"){.struct} {#fn-trade_refself---trade .code-header}
:::

::: docblock
Returns a reference to the `Trade` associated with the current instance.
[Read more](../trait.TradeAble.html#tymethod.trade_ref)
:::

::: {#method.trade_mut .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/position.rs.html#726-728){.src
.rightside}[§](#method.trade_mut){.anchor}

#### fn [trade_mut](../trait.TradeAble.html#tymethod.trade_mut){.fn}(&mut self) -\> &mut [Trade](../struct.Trade.html "struct optionstratlib::model::Trade"){.struct} {#fn-trade_mutmut-self---mut-trade .code-header}
:::

::: docblock
Provides a mutable reference to the `Trade` instance contained within
the current structure. [Read
more](../trait.TradeAble.html#tymethod.trade_mut)
:::
:::::::::

::: {#impl-TradeStatusAble-for-Position .section .impl}
[Source](../../../src/optionstratlib/model/position.rs.html#731-775){.src
.rightside}[§](#impl-TradeStatusAble-for-Position){.anchor}

### impl [TradeStatusAble](../trait.TradeStatusAble.html "trait optionstratlib::model::TradeStatusAble"){.trait} for [Position](struct.Position.html "struct optionstratlib::model::position::Position"){.struct} {#impl-tradestatusable-for-position .code-header}
:::

::::::::::::::: impl-items
::: {#method.open .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/position.rs.html#732-736){.src
.rightside}[§](#method.open){.anchor}

#### fn [open](../trait.TradeStatusAble.html#tymethod.open){.fn}(&self) -\> [Trade](../struct.Trade.html "struct optionstratlib::model::Trade"){.struct} {#fn-openself---trade .code-header}
:::

::: docblock
`open`: Return a `Trade` instance representing the trade in its open
status. [Read more](../trait.TradeStatusAble.html#tymethod.open)
:::

::: {#method.close .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/position.rs.html#738-746){.src
.rightside}[§](#method.close){.anchor}

#### fn [close](../trait.TradeStatusAble.html#tymethod.close){.fn}(&self) -\> [Trade](../struct.Trade.html "struct optionstratlib::model::Trade"){.struct} {#fn-closeself---trade .code-header}
:::

::: docblock
`closed`: Return a `Trade` instance representing the trade in its closed
status. [Read more](../trait.TradeStatusAble.html#tymethod.close)
:::

::: {#method.expired .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/position.rs.html#748-753){.src
.rightside}[§](#method.expired){.anchor}

#### fn [expired](../trait.TradeStatusAble.html#tymethod.expired){.fn}(&self) -\> [Trade](../struct.Trade.html "struct optionstratlib::model::Trade"){.struct} {#fn-expiredself---trade .code-header}
:::

::: docblock
`expired`: Return a `Trade` instance representing the trade in its
expired status. [Read
more](../trait.TradeStatusAble.html#tymethod.expired)
:::

::: {#method.exercised .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/position.rs.html#755-760){.src
.rightside}[§](#method.exercised){.anchor}

#### fn [exercised](../trait.TradeStatusAble.html#tymethod.exercised){.fn}(&self) -\> [Trade](../struct.Trade.html "struct optionstratlib::model::Trade"){.struct} {#fn-exercisedself---trade .code-header}
:::

::: docblock
`exercised`: Return a `Trade` instance representing the trade in its
exercised status. [Read
more](../trait.TradeStatusAble.html#tymethod.exercised)
:::

::: {#method.assigned .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/position.rs.html#762-767){.src
.rightside}[§](#method.assigned){.anchor}

#### fn [assigned](../trait.TradeStatusAble.html#tymethod.assigned){.fn}(&self) -\> [Trade](../struct.Trade.html "struct optionstratlib::model::Trade"){.struct} {#fn-assignedself---trade .code-header}
:::

::: docblock
`assigned`: Return a `Trade` instance representing the trade in its
assigned status. [Read
more](../trait.TradeStatusAble.html#tymethod.assigned)
:::

::: {#method.status_other .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/position.rs.html#769-774){.src
.rightside}[§](#method.status_other){.anchor}

#### fn [status_other](../trait.TradeStatusAble.html#tymethod.status_other){.fn}(&self) -\> [Trade](../struct.Trade.html "struct optionstratlib::model::Trade"){.struct} {#fn-status_otherself---trade .code-header}
:::

::: docblock
`status_other`: Return a `Trade` instance representing undeclared
status. [Read more](../trait.TradeStatusAble.html#tymethod.status_other)
:::
:::::::::::::::

::: {#impl-TransactionAble-for-Position .section .impl}
[Source](../../../src/optionstratlib/model/position.rs.html#692-700){.src
.rightside}[§](#impl-TransactionAble-for-Position){.anchor}

### impl [TransactionAble](../../pnl/trait.TransactionAble.html "trait optionstratlib::pnl::TransactionAble"){.trait} for [Position](struct.Position.html "struct optionstratlib::model::position::Position"){.struct} {#impl-transactionable-for-position .code-header}
:::

::::::: impl-items
::: {#method.add_transaction .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/position.rs.html#693-695){.src
.rightside}[§](#method.add_transaction){.anchor}

#### fn [add_transaction](../../pnl/trait.TransactionAble.html#tymethod.add_transaction){.fn}( &mut self, \_transaction: [Transaction](../../pnl/struct.Transaction.html "struct optionstratlib::pnl::Transaction"){.struct}, ) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[()](https://doc.rust-lang.org/1.91.1/std/primitive.unit.html){.primitive}, [TransactionError](../../error/struct.TransactionError.html "struct optionstratlib::error::TransactionError"){.struct}\> {#fn-add_transaction-mut-self-_transaction-transaction---result-transactionerror .code-header}
:::

::: docblock
Adds a new transaction to the implementing entity.
:::

::: {#method.get_transactions .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/position.rs.html#697-699){.src
.rightside}[§](#method.get_transactions){.anchor}

#### fn [get_transactions](../../pnl/trait.TransactionAble.html#tymethod.get_transactions){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Vec](https://doc.rust-lang.org/1.91.1/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<[Transaction](../../pnl/struct.Transaction.html "struct optionstratlib::pnl::Transaction"){.struct}\>, [TransactionError](../../error/struct.TransactionError.html "struct optionstratlib::error::TransactionError"){.struct}\> {#fn-get_transactionsself---resultvectransaction-transactionerror .code-header}
:::

::: docblock
Retrieves all transactions from the implementing entity.
:::
:::::::

::: {#impl-StructuralPartialEq-for-Position .section .impl}
[Source](../../../src/optionstratlib/model/position.rs.html#58){.src
.rightside}[§](#impl-StructuralPartialEq-for-Position){.anchor}

### impl [StructuralPartialEq](https://doc.rust-lang.org/1.91.1/core/marker/trait.StructuralPartialEq.html "trait core::marker::StructuralPartialEq"){.trait} for [Position](struct.Position.html "struct optionstratlib::model::position::Position"){.struct} {#impl-structuralpartialeq-for-position .code-header}
:::
:::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::

## Auto Trait Implementations[§](#synthetic-implementations){.anchor} {#synthetic-implementations .section-header}

::::::::: {#synthetic-implementations-list}
::: {#impl-Freeze-for-Position .section .impl}
[§](#impl-Freeze-for-Position){.anchor}

### impl [Freeze](https://doc.rust-lang.org/1.91.1/core/marker/trait.Freeze.html "trait core::marker::Freeze"){.trait} for [Position](struct.Position.html "struct optionstratlib::model::position::Position"){.struct} {#impl-freeze-for-position .code-header}
:::

::: {#impl-RefUnwindSafe-for-Position .section .impl}
[§](#impl-RefUnwindSafe-for-Position){.anchor}

### impl [RefUnwindSafe](https://doc.rust-lang.org/1.91.1/core/panic/unwind_safe/trait.RefUnwindSafe.html "trait core::panic::unwind_safe::RefUnwindSafe"){.trait} for [Position](struct.Position.html "struct optionstratlib::model::position::Position"){.struct} {#impl-refunwindsafe-for-position .code-header}
:::

::: {#impl-Send-for-Position .section .impl}
[§](#impl-Send-for-Position){.anchor}

### impl [Send](https://doc.rust-lang.org/1.91.1/core/marker/trait.Send.html "trait core::marker::Send"){.trait} for [Position](struct.Position.html "struct optionstratlib::model::position::Position"){.struct} {#impl-send-for-position .code-header}
:::

::: {#impl-Sync-for-Position .section .impl}
[§](#impl-Sync-for-Position){.anchor}

### impl [Sync](https://doc.rust-lang.org/1.91.1/core/marker/trait.Sync.html "trait core::marker::Sync"){.trait} for [Position](struct.Position.html "struct optionstratlib::model::position::Position"){.struct} {#impl-sync-for-position .code-header}
:::

::: {#impl-Unpin-for-Position .section .impl}
[§](#impl-Unpin-for-Position){.anchor}

### impl [Unpin](https://doc.rust-lang.org/1.91.1/core/marker/trait.Unpin.html "trait core::marker::Unpin"){.trait} for [Position](struct.Position.html "struct optionstratlib::model::position::Position"){.struct} {#impl-unpin-for-position .code-header}
:::

::: {#impl-UnwindSafe-for-Position .section .impl}
[§](#impl-UnwindSafe-for-Position){.anchor}

### impl [UnwindSafe](https://doc.rust-lang.org/1.91.1/core/panic/unwind_safe/trait.UnwindSafe.html "trait core::panic::unwind_safe::UnwindSafe"){.trait} for [Position](struct.Position.html "struct optionstratlib::model::position::Position"){.struct} {#impl-unwindsafe-for-position .code-header}
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
::: {#method.from .section .method .trait-impl}
[Source](https://doc.rust-lang.org/1.91.1/src/core/convert/mod.rs.html#788){.src
.rightside}[§](#method.from){.anchor}

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
:::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
