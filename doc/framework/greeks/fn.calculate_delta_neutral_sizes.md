:::::::: width-limiter
::::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../index.html)::[greeks](index.html)
:::

# Function [calculate_delta_neutral_sizes]{.fn} Copy item path

[[Source](../../src/optionstratlib/greeks/utils.rs.html#449-501){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub fn calculate_delta_neutral_sizes(
    delta1: Decimal,
    delta2: Decimal,
    total_size: Positive,
) -> Result<(Positive, Positive), GreeksError>
```

Expand description

:::: docblock
Calculates the optimal position sizes for two positions to achieve delta
neutrality while maintaining a specified total position size.

## [§](#arguments){.doc-anchor}Arguments

- `delta1` - Delta of the first position (e.g., short call delta)
- `delta2` - Delta of the second position (e.g., short put delta)
- `total_size` - Desired total position size (sum of both positions)

## [§](#returns){.doc-anchor}Returns

- `Ok((size1, size2))` - Tuple containing the calculated sizes for each
  position
- `Err(String)` - Error message if calculation is not possible

## [§](#example){.doc-anchor}Example

::: example-wrap
``` {.rust .rust-example-rendered}
use rust_decimal_macros::dec;
use optionstratlib::greeks::calculate_delta_neutral_sizes;
use optionstratlib::pos;
let (call_size, put_size) = calculate_delta_neutral_sizes(
    dec!(-0.30),  // Short call delta
    dec!(0.20),   // Short put delta
    pos!(7.0)     // Total desired position size
).unwrap();
```
:::
::::
:::::::
::::::::
