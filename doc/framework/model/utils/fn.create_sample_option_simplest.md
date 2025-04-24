:::::::: width-limiter
::::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../../index.html)::[model](../index.html)::[utils](index.html)
:::

# Function [create_sample_option_simplest]{.fn}Copy item path

[[Source](../../../src/optionstratlib/model/utils.rs.html#240-255){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub fn create_sample_option_simplest(
    option_style: OptionStyle,
    side: Side,
) -> Options
```

Expand description

:::: docblock
Creates a simplified sample option contract for testing or demonstration
purposes.

This function generates an Options instance with pre-defined values,
requiring only the specification of the option style (Call or Put) and
market position (Long or Short). It uses Apple Inc. (AAPL) as the
underlying security with standard parameters suitable for basic examples
or testing scenarios.

## [§](#parameters){.doc-anchor}Parameters

- `option_style` - Specifies whether the option is a Call or Put
- `side` - Indicates whether the position is Long or Short

## [§](#returns){.doc-anchor}Returns

Returns an `Options` instance with the following predefined values:

- European-style option
- AAPL as the underlying symbol
- Strike price of \$100.0
- 30 days until expiration
- 20% implied volatility (0.2)
- Quantity of 1.0 contracts
- Underlying price of \$100.0
- 5% risk-free rate
- 1% dividend yield
- No exotic parameters

## [§](#examples){.doc-anchor}Examples

::: example-wrap
``` {.rust .rust-example-rendered}
use optionstratlib::model::utils::create_sample_option_simplest;
use optionstratlib::{OptionStyle, Side};
let long_call = create_sample_option_simplest(OptionStyle::Call, Side::Long);
let short_put = create_sample_option_simplest(OptionStyle::Put, Side::Short);
```
:::
::::
:::::::
::::::::
