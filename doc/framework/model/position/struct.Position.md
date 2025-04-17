:::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::: width-limiter
::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../../index.html)::[model](../index.html)::[position](index.html)
:::

# Struct [Position]{.struct}Copy item path

[[Source](../../../src/optionstratlib/model/position.rs.html#55-75){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub struct Position {
    pub option: Options,
    pub premium: Positive,
    pub date: DateTime<Utc>,
    pub open_fee: Positive,
    pub close_fee: Positive,
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
.field}`date: `[`DateTime`](https://docs.rs/chrono/latest/chrono/datetime/struct.DateTime.html "struct chrono::datetime::DateTime"){.struct}`<`[`Utc`](https://docs.rs/chrono/latest/chrono/offset/utc/struct.Utc.html "struct chrono::offset::utc::Utc"){.struct}`>`]{#structfield.date
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

## Implementations[§](#implementations){.anchor} {#implementations .section-header}

:::::::::::::::::::::::::::::::::::::: {#implementations-list}
::: {#impl-Position .section .impl}
[Source](../../../src/optionstratlib/model/position.rs.html#77-630){.src
.rightside}[§](#impl-Position){.anchor}

### impl [Position](struct.Position.html "struct optionstratlib::model::position::Position"){.struct} {#impl-position .code-header}
:::

:::::::::::::::::::::::::::::::::::: impl-items
::: {#method.new .section .method}
[Source](../../../src/optionstratlib/model/position.rs.html#121-135){.src
.rightside}

#### pub fn [new](#method.new){.fn}( option: [Options](../option/struct.Options.html "struct optionstratlib::model::option::Options"){.struct}, premium: [Positive](../positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, date: [DateTime](https://docs.rs/chrono/latest/chrono/datetime/struct.DateTime.html "struct chrono::datetime::DateTime"){.struct}\<[Utc](https://docs.rs/chrono/latest/chrono/offset/utc/struct.Utc.html "struct chrono::offset::utc::Utc"){.struct}\>, open_fee: [Positive](../positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, close_fee: [Positive](../positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, ) -\> Self {#pub-fn-new-option-options-premium-positive-date-datetimeutc-open_fee-positive-close_fee-positive---self .code-header}
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
);
```
:::
::::

::: {#method.total_cost .section .method}
[Source](../../../src/optionstratlib/model/position.rs.html#196-203){.src
.rightside}

#### pub fn [total_cost](#method.total_cost){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Positive](../positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, [PositionError](../../error/position/enum.PositionError.html "enum optionstratlib::error::position::PositionError"){.enum}\> {#pub-fn-total_costself---resultpositive-positionerror .code-header}
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
[Source](../../../src/optionstratlib/model/position.rs.html#241-246){.src
.rightside}

#### pub fn [premium_received](#method.premium_received){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Positive](../positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, [PositionError](../../error/position/enum.PositionError.html "enum optionstratlib::error::position::PositionError"){.enum}\> {#pub-fn-premium_receivedself---resultpositive-positionerror .code-header}
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
);

// Calculate premium received
let received = position.premium_received().unwrap();
info!("Premium received: {}", received);
```
:::
::::

::: {#method.net_premium_received .section .method}
[Source](../../../src/optionstratlib/model/position.rs.html#266-283){.src
.rightside}

#### pub fn [net_premium_received](#method.net_premium_received){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Positive](../positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, [PositionError](../../error/position/enum.PositionError.html "enum optionstratlib::error::position::PositionError"){.enum}\> {#pub-fn-net_premium_receivedself---resultpositive-positionerror .code-header}
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
[Source](../../../src/optionstratlib/model/position.rs.html#328-336){.src
.rightside}

#### pub fn [pnl_at_expiration](#method.pnl_at_expiration){.fn}( &self, price: &[Option](https://doc.rust-lang.org/1.86.0/core/option/enum.Option.html "enum core::option::Option"){.enum}\<&[Positive](../positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\>, ) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<Decimal, [Box](https://doc.rust-lang.org/1.86.0/alloc/boxed/struct.Box.html "struct alloc::boxed::Box"){.struct}\<dyn [Error](https://doc.rust-lang.org/1.86.0/core/error/trait.Error.html "trait core::error::Error"){.trait}\>\> {#pub-fn-pnl_at_expiration-self-price-optionpositive---resultdecimal-boxdyn-error .code-header}
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

- `Result<Decimal, Box<dyn Error>>` - The calculated profit or loss as a
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
[Source](../../../src/optionstratlib/model/position.rs.html#377-390){.src
.rightside}

#### pub fn [unrealized_pnl](#method.unrealized_pnl){.fn}(&self, price: [Positive](../positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<Decimal, [PositionError](../../error/position/enum.PositionError.html "enum optionstratlib::error::position::PositionError"){.enum}\> {#pub-fn-unrealized_pnlself-price-positive---resultdecimal-positionerror .code-header}
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
);
let unrealized_pnl = position.unrealized_pnl(current_price).unwrap();
info!("Current unrealized PnL: {}", unrealized_pnl);
```
:::
::::

::: {#method.days_held .section .method}
[Source](../../../src/optionstratlib/model/position.rs.html#405-407){.src
.rightside}

#### pub fn [days_held](#method.days_held){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Positive](../positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, [PositionError](../../error/position/enum.PositionError.html "enum optionstratlib::error::position::PositionError"){.enum}\> {#pub-fn-days_heldself---resultpositive-positionerror .code-header}
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
[Source](../../../src/optionstratlib/model/position.rs.html#422-429){.src
.rightside}

#### pub fn [days_to_expiration](#method.days_to_expiration){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Positive](../positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, [PositionError](../../error/position/enum.PositionError.html "enum optionstratlib::error::position::PositionError"){.enum}\> {#pub-fn-days_to_expirationself---resultpositive-positionerror .code-header}
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
[Source](../../../src/optionstratlib/model/position.rs.html#441-446){.src
.rightside}

#### pub fn [is_long](#method.is_long){.fn}(&self) -\> [bool](https://doc.rust-lang.org/1.86.0/std/primitive.bool.html){.primitive} {#pub-fn-is_longself---bool .code-header}
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
[Source](../../../src/optionstratlib/model/position.rs.html#458-463){.src
.rightside}

#### pub fn [is_short](#method.is_short){.fn}(&self) -\> [bool](https://doc.rust-lang.org/1.86.0/std/primitive.bool.html){.primitive} {#pub-fn-is_shortself---bool .code-header}
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
[Source](../../../src/optionstratlib/model/position.rs.html#479-488){.src
.rightside}

#### pub fn [net_cost](#method.net_cost){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<Decimal, [PositionError](../../error/position/enum.PositionError.html "enum optionstratlib::error::position::PositionError"){.enum}\> {#pub-fn-net_costself---resultdecimal-positionerror .code-header}
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
[Source](../../../src/optionstratlib/model/position.rs.html#511-530){.src
.rightside}

#### pub fn [break_even](#method.break_even){.fn}(&self) -\> [Option](https://doc.rust-lang.org/1.86.0/core/option/enum.Option.html "enum core::option::Option"){.enum}\<[Positive](../positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\> {#pub-fn-break_evenself---optionpositive .code-header}
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
[Source](../../../src/optionstratlib/model/position.rs.html#582-584){.src
.rightside}

#### pub fn [fees](#method.fees){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Positive](../positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, [PositionError](../../error/position/enum.PositionError.html "enum optionstratlib::error::position::PositionError"){.enum}\> {#pub-fn-feesself---resultpositive-positionerror .code-header}
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
[Source](../../../src/optionstratlib/model/position.rs.html#619-629){.src
.rightside}

#### pub fn [validate](#method.validate){.fn}(&self) -\> [bool](https://doc.rust-lang.org/1.86.0/std/primitive.bool.html){.primitive} {#pub-fn-validateself---bool .code-header}
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
    pos!(0.65)
);

assert!(position.validate());
```
:::
::::
::::::::::::::::::::::::::::::::::::
::::::::::::::::::::::::::::::::::::::

## Trait Implementations[§](#trait-implementations){.anchor} {#trait-implementations .section-header}

:::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::: {#trait-implementations-list}
::: {#impl-Clone-for-Position .section .impl}
[Source](../../../src/optionstratlib/model/position.rs.html#54){.src
.rightside}[§](#impl-Clone-for-Position){.anchor}

### impl [Clone](https://doc.rust-lang.org/1.86.0/core/clone/trait.Clone.html "trait core::clone::Clone"){.trait} for [Position](struct.Position.html "struct optionstratlib::model::position::Position"){.struct} {#impl-clone-for-position .code-header}
:::

::::::: impl-items
::: {#method.clone .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/position.rs.html#54){.src
.rightside}[§](#method.clone){.anchor}

#### fn [clone](https://doc.rust-lang.org/1.86.0/core/clone/trait.Clone.html#tymethod.clone){.fn}(&self) -\> [Position](struct.Position.html "struct optionstratlib::model::position::Position"){.struct} {#fn-cloneself---position .code-header}
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

::: {#impl-Debug-for-Position .section .impl}
[Source](../../../src/optionstratlib/model/format.rs.html#251-261){.src
.rightside}[§](#impl-Debug-for-Position){.anchor}

### impl [Debug](https://doc.rust-lang.org/1.86.0/core/fmt/trait.Debug.html "trait core::fmt::Debug"){.trait} for [Position](struct.Position.html "struct optionstratlib::model::position::Position"){.struct} {#impl-debug-for-position .code-header}
:::

::::: impl-items
::: {#method.fmt-1 .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/format.rs.html#252-260){.src
.rightside}[§](#method.fmt-1){.anchor}

#### fn [fmt](https://doc.rust-lang.org/1.86.0/core/fmt/trait.Debug.html#tymethod.fmt){.fn}(&self, f: &mut [Formatter](https://doc.rust-lang.org/1.86.0/core/fmt/struct.Formatter.html "struct core::fmt::Formatter"){.struct}\<\'\_\>) -\> [Result](https://doc.rust-lang.org/1.86.0/core/fmt/type.Result.html "type core::fmt::Result"){.type} {#fn-fmtself-f-mut-formatter_---result .code-header}
:::

::: docblock
Formats the value using the given formatter. [Read
more](https://doc.rust-lang.org/1.86.0/core/fmt/trait.Debug.html#tymethod.fmt)
:::
:::::

::: {#impl-Default-for-Position .section .impl}
[Source](../../../src/optionstratlib/model/position.rs.html#632-642){.src
.rightside}[§](#impl-Default-for-Position){.anchor}

### impl [Default](https://doc.rust-lang.org/1.86.0/core/default/trait.Default.html "trait core::default::Default"){.trait} for [Position](struct.Position.html "struct optionstratlib::model::position::Position"){.struct} {#impl-default-for-position .code-header}
:::

::::: impl-items
::: {#method.default .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/position.rs.html#633-641){.src
.rightside}[§](#method.default){.anchor}

#### fn [default](https://doc.rust-lang.org/1.86.0/core/default/trait.Default.html#tymethod.default){.fn}() -\> Self {#fn-default---self .code-header}
:::

::: docblock
Returns the "default value" for a type. [Read
more](https://doc.rust-lang.org/1.86.0/core/default/trait.Default.html#tymethod.default)
:::
:::::

::: {#impl-Deserialize%3C'de%3E-for-Position .section .impl}
[Source](../../../src/optionstratlib/model/position.rs.html#54){.src
.rightside}[§](#impl-Deserialize%3C'de%3E-for-Position){.anchor}

### impl\<\'de\> [Deserialize](https://docs.rs/serde/1.0.219/serde/de/trait.Deserialize.html "trait serde::de::Deserialize"){.trait}\<\'de\> for [Position](struct.Position.html "struct optionstratlib::model::position::Position"){.struct} {#implde-deserializede-for-position .code-header}
:::

:::::: impl-items
:::: {#method.deserialize .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/position.rs.html#54){.src
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

::: {#impl-Display-for-Position .section .impl}
[Source](../../../src/optionstratlib/model/format.rs.html#240-249){.src
.rightside}[§](#impl-Display-for-Position){.anchor}

### impl [Display](https://doc.rust-lang.org/1.86.0/core/fmt/trait.Display.html "trait core::fmt::Display"){.trait} for [Position](struct.Position.html "struct optionstratlib::model::position::Position"){.struct} {#impl-display-for-position .code-header}
:::

::::: impl-items
::: {#method.fmt .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/format.rs.html#241-248){.src
.rightside}[§](#method.fmt){.anchor}

#### fn [fmt](https://doc.rust-lang.org/1.86.0/core/fmt/trait.Display.html#tymethod.fmt){.fn}(&self, f: &mut [Formatter](https://doc.rust-lang.org/1.86.0/core/fmt/struct.Formatter.html "struct core::fmt::Formatter"){.struct}\<\'\_\>) -\> [Result](https://doc.rust-lang.org/1.86.0/core/fmt/type.Result.html "type core::fmt::Result"){.type} {#fn-fmtself-f-mut-formatter_---result-1 .code-header}
:::

::: docblock
Formats the value using the given formatter. [Read
more](https://doc.rust-lang.org/1.86.0/core/fmt/trait.Display.html#tymethod.fmt)
:::
:::::

:::: {#impl-Graph-for-Position .section .impl}
[Source](../../../src/optionstratlib/model/position.rs.html#800-877){.src
.rightside}[§](#impl-Graph-for-Position){.anchor}

### impl [Graph](../../visualization/utils/trait.Graph.html "trait optionstratlib::visualization::utils::Graph"){.trait} for [Position](struct.Position.html "struct optionstratlib::model::position::Position"){.struct} {#impl-graph-for-position .code-header}

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

::::::::::::::: impl-items
::: {#method.title .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/position.rs.html#805-807){.src
.rightside}[§](#method.title){.anchor}

#### fn [title](../../visualization/utils/trait.Graph.html#tymethod.title){.fn}(&self) -\> [String](https://doc.rust-lang.org/1.86.0/alloc/string/struct.String.html "struct alloc::string::String"){.struct} {#fn-titleself---string .code-header}
:::

::: docblock
Generates a title for the graph based on the option's characteristics.

##### [§](#returns-18){.doc-anchor}Returns {#returns-18}

A `String` containing the formatted title that describes the position.
:::

::: {#method.get_x_values .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/position.rs.html#826-828){.src
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

##### [§](#returns-19){.doc-anchor}Returns {#returns-19}

A vector of `Positive` values representing potential stock prices, with
1000 total points (999 steps plus endpoints) evenly distributed across
the range.

##### [§](#implementation-details){.doc-anchor}Implementation Details

- The range extends 5 standard deviations above and below the strike
  price
- Uses 1000 total points (steps + 1) for smooth visualization
- All returned values are guaranteed positive through the use of the
  `pos!` macro
:::

::: {#method.get_y_values .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/position.rs.html#840-850){.src
.rightside}[§](#method.get_y_values){.anchor}

#### fn [get_y_values](../../visualization/utils/trait.Graph.html#method.get_y_values){.fn}(&self) -\> [Vec](https://doc.rust-lang.org/1.86.0/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<[f64](https://doc.rust-lang.org/1.86.0/std/primitive.f64.html){.primitive}\> {#fn-get_y_valuesself---vecf64 .code-header}
:::

::: docblock
Calculates position profit/loss values at expiration for a range of
underlying prices.

This method transforms a slice of potential underlying prices into their
corresponding profit/loss values at expiration for this position.

##### [§](#parameters-3){.doc-anchor}Parameters {#parameters-3}

- `data` - A slice of `Positive` values representing potential prices of
  the underlying asset

##### [§](#returns-20){.doc-anchor}Returns {#returns-20}

A `Vec<f64>` containing the calculated profit/loss values for each input
price
:::

::: {#method.get_vertical_lines .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/position.rs.html#859-876){.src
.rightside}[§](#method.get_vertical_lines){.anchor}

#### fn [get_vertical_lines](../../visualization/utils/trait.Graph.html#method.get_vertical_lines){.fn}(&self) -\> [Vec](https://doc.rust-lang.org/1.86.0/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<ChartVerticalLine\<[f64](https://doc.rust-lang.org/1.86.0/std/primitive.f64.html){.primitive}, [f64](https://doc.rust-lang.org/1.86.0/std/primitive.f64.html){.primitive}\>\> {#fn-get_vertical_linesself---vecchartverticallinef64-f64 .code-header}
:::

::: docblock
Generates vertical lines for the graph to highlight significant price
levels.

This method creates vertical line indicators for important price points
in the position analysis, specifically the break-even price level where
the position transitions between profit and loss.

##### [§](#returns-21){.doc-anchor}Returns {#returns-21}

A `Vec<ChartVerticalLine<f64, f64>>` containing vertical line
definitions to be displayed on the chart
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

:::: {#impl-Greeks-for-Position .section .impl}
[Source](../../../src/optionstratlib/model/position.rs.html#650-663){.src
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
[Source](../../../src/optionstratlib/model/position.rs.html#660-662){.src
.rightside}[§](#method.get_options){.anchor}

#### fn [get_options](../../greeks/trait.Greeks.html#tymethod.get_options){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Vec](https://doc.rust-lang.org/1.86.0/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<&[Options](../option/struct.Options.html "struct optionstratlib::model::option::Options"){.struct}\>, [GreeksError](../../error/greeks/enum.GreeksError.html "enum optionstratlib::error::greeks::GreeksError"){.enum}\> {#fn-get_optionsself---resultvecoptions-greekserror .code-header}
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

::: {#impl-PartialEq-for-Position .section .impl}
[Source](../../../src/optionstratlib/model/position.rs.html#54){.src
.rightside}[§](#impl-PartialEq-for-Position){.anchor}

### impl [PartialEq](https://doc.rust-lang.org/1.86.0/core/cmp/trait.PartialEq.html "trait core::cmp::PartialEq"){.trait} for [Position](struct.Position.html "struct optionstratlib::model::position::Position"){.struct} {#impl-partialeq-for-position .code-header}
:::

::::::: impl-items
::: {#method.eq .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/position.rs.html#54){.src
.rightside}[§](#method.eq){.anchor}

#### fn [eq](https://doc.rust-lang.org/1.86.0/core/cmp/trait.PartialEq.html#tymethod.eq){.fn}(&self, other: &[Position](struct.Position.html "struct optionstratlib::model::position::Position"){.struct}) -\> [bool](https://doc.rust-lang.org/1.86.0/std/primitive.bool.html){.primitive} {#fn-eqself-other-position---bool .code-header}
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

:::: {#impl-PnLCalculator-for-Position .section .impl}
[Source](../../../src/optionstratlib/model/position.rs.html#686-765){.src
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

::::::::: impl-items
::: {#method.calculate_pnl .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/position.rs.html#706-730){.src
.rightside}[§](#method.calculate_pnl){.anchor}

#### fn [calculate_pnl](../../pnl/trait.PnLCalculator.html#tymethod.calculate_pnl){.fn}( &self, underlying_price: &[Positive](../positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, expiration_date: [ExpirationDate](../types/enum.ExpirationDate.html "enum optionstratlib::model::types::ExpirationDate"){.enum}, implied_volatility: &[Positive](../positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, ) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[PnL](../../pnl/utils/struct.PnL.html "struct optionstratlib::pnl::utils::PnL"){.struct}, [Box](https://doc.rust-lang.org/1.86.0/alloc/boxed/struct.Box.html "struct alloc::boxed::Box"){.struct}\<dyn [Error](https://doc.rust-lang.org/1.86.0/core/error/trait.Error.html "trait core::error::Error"){.trait}\>\> {#fn-calculate_pnl-self-underlying_price-positive-expiration_date-expirationdate-implied_volatility-positive---resultpnl-boxdyn-error .code-header}
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

- `Result<PnL, Box<dyn Error>>` - A PnL object containing unrealized
  profit/loss and position cost details, or an error if the calculation
  fails
:::

::: {#method.calculate_pnl_at_expiration .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/position.rs.html#747-764){.src
.rightside}[§](#method.calculate_pnl_at_expiration){.anchor}

#### fn [calculate_pnl_at_expiration](../../pnl/trait.PnLCalculator.html#tymethod.calculate_pnl_at_expiration){.fn}( &self, underlying_price: &[Positive](../positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, ) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[PnL](../../pnl/utils/struct.PnL.html "struct optionstratlib::pnl::utils::PnL"){.struct}, [Box](https://doc.rust-lang.org/1.86.0/alloc/boxed/struct.Box.html "struct alloc::boxed::Box"){.struct}\<dyn [Error](https://doc.rust-lang.org/1.86.0/core/error/trait.Error.html "trait core::error::Error"){.trait}\>\> {#fn-calculate_pnl_at_expiration-self-underlying_price-positive---resultpnl-boxdyn-error .code-header}
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

- `Result<PnL, Box<dyn Error>>` - A PnL object containing realized
  profit/loss and position cost details, or an error if the calculation
  fails
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

:::: {#impl-Profit-for-Position .section .impl}
[Source](../../../src/optionstratlib/model/position.rs.html#772-789){.src
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
[Source](../../../src/optionstratlib/model/position.rs.html#786-788){.src
.rightside}[§](#method.calculate_profit_at){.anchor}

#### fn [calculate_profit_at](../../pricing/trait.Profit.html#tymethod.calculate_profit_at){.fn}( &self, price: [Positive](../positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, ) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<Decimal, [Box](https://doc.rust-lang.org/1.86.0/alloc/boxed/struct.Box.html "struct alloc::boxed::Box"){.struct}\<dyn [Error](https://doc.rust-lang.org/1.86.0/core/error/trait.Error.html "trait core::error::Error"){.trait}\>\> {#fn-calculate_profit_at-self-price-positive---resultdecimal-boxdyn-error .code-header}
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

- `Result<Decimal, Box<dyn Error>>` - The calculated profit as a Decimal
  if successful, or an error if the calculation fails.
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

::: {#impl-Serialize-for-Position .section .impl}
[Source](../../../src/optionstratlib/model/position.rs.html#54){.src
.rightside}[§](#impl-Serialize-for-Position){.anchor}

### impl [Serialize](https://docs.rs/serde/1.0.219/serde/ser/trait.Serialize.html "trait serde::ser::Serialize"){.trait} for [Position](struct.Position.html "struct optionstratlib::model::position::Position"){.struct} {#impl-serialize-for-position .code-header}
:::

:::::: impl-items
:::: {#method.serialize .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/position.rs.html#54){.src
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

::: {#impl-TransactionAble-for-Position .section .impl}
[Source](../../../src/optionstratlib/model/position.rs.html#665-673){.src
.rightside}[§](#impl-TransactionAble-for-Position){.anchor}

### impl [TransactionAble](../../pnl/trait.TransactionAble.html "trait optionstratlib::pnl::TransactionAble"){.trait} for [Position](struct.Position.html "struct optionstratlib::model::position::Position"){.struct} {#impl-transactionable-for-position .code-header}
:::

::::::: impl-items
::: {#method.add_transaction .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/position.rs.html#666-668){.src
.rightside}[§](#method.add_transaction){.anchor}

#### fn [add_transaction](../../pnl/trait.TransactionAble.html#tymethod.add_transaction){.fn}( &mut self, \_transaction: [Transaction](../../pnl/struct.Transaction.html "struct optionstratlib::pnl::Transaction"){.struct}, ) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[()](https://doc.rust-lang.org/1.86.0/std/primitive.unit.html){.primitive}, [TransactionError](../../error/struct.TransactionError.html "struct optionstratlib::error::TransactionError"){.struct}\> {#fn-add_transaction-mut-self-_transaction-transaction---result-transactionerror .code-header}
:::

::: docblock
Adds a new transaction to the implementing entity.
:::

::: {#method.get_transactions .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/position.rs.html#670-672){.src
.rightside}[§](#method.get_transactions){.anchor}

#### fn [get_transactions](../../pnl/trait.TransactionAble.html#tymethod.get_transactions){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Vec](https://doc.rust-lang.org/1.86.0/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<[Transaction](../../pnl/struct.Transaction.html "struct optionstratlib::pnl::Transaction"){.struct}\>, [TransactionError](../../error/struct.TransactionError.html "struct optionstratlib::error::TransactionError"){.struct}\> {#fn-get_transactionsself---resultvectransaction-transactionerror .code-header}
:::

::: docblock
Retrieves all transactions from the implementing entity.
:::
:::::::

::: {#impl-StructuralPartialEq-for-Position .section .impl}
[Source](../../../src/optionstratlib/model/position.rs.html#54){.src
.rightside}[§](#impl-StructuralPartialEq-for-Position){.anchor}

### impl [StructuralPartialEq](https://doc.rust-lang.org/1.86.0/core/marker/trait.StructuralPartialEq.html "trait core::marker::StructuralPartialEq"){.trait} for [Position](struct.Position.html "struct optionstratlib::model::position::Position"){.struct} {#impl-structuralpartialeq-for-position .code-header}
:::
::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::

## Auto Trait Implementations[§](#synthetic-implementations){.anchor} {#synthetic-implementations .section-header}

::::::::: {#synthetic-implementations-list}
::: {#impl-Freeze-for-Position .section .impl}
[§](#impl-Freeze-for-Position){.anchor}

### impl [Freeze](https://doc.rust-lang.org/1.86.0/core/marker/trait.Freeze.html "trait core::marker::Freeze"){.trait} for [Position](struct.Position.html "struct optionstratlib::model::position::Position"){.struct} {#impl-freeze-for-position .code-header}
:::

::: {#impl-RefUnwindSafe-for-Position .section .impl}
[§](#impl-RefUnwindSafe-for-Position){.anchor}

### impl [RefUnwindSafe](https://doc.rust-lang.org/1.86.0/core/panic/unwind_safe/trait.RefUnwindSafe.html "trait core::panic::unwind_safe::RefUnwindSafe"){.trait} for [Position](struct.Position.html "struct optionstratlib::model::position::Position"){.struct} {#impl-refunwindsafe-for-position .code-header}
:::

::: {#impl-Send-for-Position .section .impl}
[§](#impl-Send-for-Position){.anchor}

### impl [Send](https://doc.rust-lang.org/1.86.0/core/marker/trait.Send.html "trait core::marker::Send"){.trait} for [Position](struct.Position.html "struct optionstratlib::model::position::Position"){.struct} {#impl-send-for-position .code-header}
:::

::: {#impl-Sync-for-Position .section .impl}
[§](#impl-Sync-for-Position){.anchor}

### impl [Sync](https://doc.rust-lang.org/1.86.0/core/marker/trait.Sync.html "trait core::marker::Sync"){.trait} for [Position](struct.Position.html "struct optionstratlib::model::position::Position"){.struct} {#impl-sync-for-position .code-header}
:::

::: {#impl-Unpin-for-Position .section .impl}
[§](#impl-Unpin-for-Position){.anchor}

### impl [Unpin](https://doc.rust-lang.org/1.86.0/core/marker/trait.Unpin.html "trait core::marker::Unpin"){.trait} for [Position](struct.Position.html "struct optionstratlib::model::position::Position"){.struct} {#impl-unpin-for-position .code-header}
:::

::: {#impl-UnwindSafe-for-Position .section .impl}
[§](#impl-UnwindSafe-for-Position){.anchor}

### impl [UnwindSafe](https://doc.rust-lang.org/1.86.0/core/panic/unwind_safe/trait.UnwindSafe.html "trait core::panic::unwind_safe::UnwindSafe"){.trait} for [Position](struct.Position.html "struct optionstratlib::model::position::Position"){.struct} {#impl-unwindsafe-for-position .code-header}
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
:::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
