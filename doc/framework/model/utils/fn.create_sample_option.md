:::::::: width-limiter
::::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../../index.html)::[model](../index.html)::[utils](index.html)
:::

# Function [create_sample_option]{.fn} Copy item path

[[Source](../../../src/optionstratlib/model/utils.rs.html#67-89){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub fn create_sample_option(
    option_style: OptionStyle,
    side: Side,
    underlying_price: Positive,
    quantity: Positive,
    strike_price: Positive,
    volatility: Positive,
) -> Options
```

Expand description

:::: docblock
Creates a sample option contract with predefined parameters for testing
or demonstration purposes.

This utility function simplifies the creation of option contracts by
providing a standard configuration with reasonable defaults. It creates
a European-style option with a 30-day expiration and a fixed risk-free
rate of 5%.

## [§](#parameters){.doc-anchor}Parameters

- `option_style` - Specifies whether the option is a Call or Put.
- `side` - Determines if the position is Long or Short.
- `underlying_price` - The current market price of the underlying asset.
- `quantity` - The number of contracts in the position.
- `strike_price` - The price at which the option holder can exercise the
  option.
- `volatility` - The implied volatility used for pricing the option.

## [§](#returns){.doc-anchor}Returns

An `Options` struct configured with the specified parameters and
sensible defaults.

## [§](#examples){.doc-anchor}Examples

::: example-wrap
``` {.rust .rust-example-rendered}
use optionstratlib::{pos, OptionStyle, Side};
use optionstratlib::model::utils::create_sample_option;
let option = create_sample_option(
    OptionStyle::Call,
    Side::Long,
    pos!(150.0),  // underlying price
    pos!(10.0),   // quantity
    pos!(155.0),  // strike price
    pos!(0.25),   // volatility (25%)
);
```
:::
::::
:::::::
::::::::
