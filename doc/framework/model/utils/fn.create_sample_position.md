:::::::: width-limiter
::::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../../index.html)::[model](../index.html)::[utils](index.html)
:::

# Function [create_sample_position]{.fn}Copy item path

[[Source](../../../src/optionstratlib/model/utils.rs.html#131-159){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub fn create_sample_position(
    option_style: OptionStyle,
    side: Side,
    underlying_price: Positive,
    quantity: Positive,
    strike_price: Positive,
    implied_volatility: Positive,
) -> Position
```

Expand description

:::: docblock
Creates a sample position for testing and demonstration purposes.

This function generates a `Position` instance with predefined values for
some fields while allowing customization of key option parameters. It's
useful for creating test scenarios, examples, or sample data for option
position analysis.

## [§](#parameters){.doc-anchor}Parameters

- `option_style` - The style of the option (Call or Put)
- `side` - Whether the position is Long or Short
- `underlying_price` - The current price of the underlying asset
- `quantity` - The number of option contracts in the position
- `strike_price` - The price at which the option can be exercised
- `implied_volatility` - The market's forecast of likely movement in the
  underlying asset

## [§](#returns){.doc-anchor}Returns

A `Position` instance with the specified parameters and these default
values:

- European-style option
- "AAPL" as the underlying symbol
- 30-day expiration
- 5% risk-free rate
- 1% dividend yield
- Premium of \$5.00
- Open and close fees of \$0.50 each
- Current date and time

## [§](#example){.doc-anchor}Example

::: example-wrap
``` {.rust .rust-example-rendered}
use optionstratlib::model::utils::create_sample_position;
use optionstratlib::{pos, OptionStyle, Side};
let sample_call = create_sample_position(
    OptionStyle::Call,
    Side::Long,
    pos!(150.0),  // underlying price
    pos!(1.0),    // quantity
    pos!(155.0),  // strike price
    pos!(0.25)    // implied volatility
);
```
:::
::::
:::::::
::::::::
