:::::::: width-limiter
::::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../../index.html)::[model](../index.html)::[utils](index.html)
:::

# Function [create_sample_option_simplest_strike]{.fn}Copy item path

[[Source](../../../src/optionstratlib/model/utils.rs.html#288-307){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub fn create_sample_option_simplest_strike(
    side: Side,
    option_style: OptionStyle,
    strike: Positive,
) -> Options
```

Expand description

:::: docblock
Creates a sample option with specified parameters and default values.

This function provides a convenient way to create an `Options` instance
with common default values while allowing customization of the most
important parameters: side, option style, and strike price. All other
parameters are set to reasonable defaults for testing or demonstration
purposes.

## [§](#parameters){.doc-anchor}Parameters

- `side` - The position side (Long or Short) for the option.
- `option_style` - The style of option (Call or Put).
- `strike` - The strike price of the option as a `Positive` value.

## [§](#returns){.doc-anchor}Returns

An `Options` instance representing a European option on AAPL stock with:

- 30 days until expiration
- 20% implied volatility
- Quantity of 1.0
- Underlying price of \$100.0
- 5% risk-free rate
- 1% dividend yield
- No exotic parameters

## [§](#examples){.doc-anchor}Examples

::: example-wrap
``` {.rust .rust-example-rendered}
use optionstratlib::model::utils::create_sample_option_simplest_strike;
use optionstratlib::{pos, OptionStyle, Side};
let long_call = create_sample_option_simplest_strike(
    Side::Long,
    OptionStyle::Call,
    pos!(105.0)
);
```
:::
::::
:::::::
::::::::
