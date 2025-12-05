::::::: width-limiter
:::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../../index.html)::[model](../index.html)::[utils](index.html)
:::

# Function [create_sample_option_with_date]{.fn} Copy item path

[[Source](../../../src/optionstratlib/model/utils.rs.html#185-208){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub fn create_sample_option_with_date(
    option_style: OptionStyle,
    side: Side,
    underlying_price: Positive,
    quantity: Positive,
    strike_price: Positive,
    volatility: Positive,
    naive_date: NaiveDateTime,
) -> Options
```

Expand description

::: docblock
Creates a sample Options object with a specific expiration date.

This utility function simplifies the creation of option contracts for
testing or demonstration purposes by providing a specific expiration
date using a NaiveDateTime. It creates a European-style option with
predefined parameters and a fixed risk-free rate of 5% and dividend
yield of 1%.

## [§](#parameters){.doc-anchor}Parameters

- `option_style` - The style of the option (Call or Put)
- `side` - The position side (Long or Short)
- `underlying_price` - The current price of the underlying asset
- `quantity` - The number of option contracts
- `strike_price` - The strike price of the option
- `volatility` - The implied volatility for pricing the option
- `naive_date` - The expiration date and time in naive format (will be
  converted to UTC)

## [§](#returns){.doc-anchor}Returns

Returns a fully configured Options instance with "AAPL" as the
underlying symbol.
:::
::::::
:::::::
