:::::::: width-limiter
::::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../index.html)::[greeks](index.html)
:::

# Function [big_n]{.fn} Copy item path

[[Source](../../src/optionstratlib/greeks/utils.rs.html#377-392){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub fn big_n(x: Decimal) -> Result<Decimal, DecimalError>
```

Expand description

:::: docblock
Computes the cumulative distribution function (CDF) of the standard
normal distribution for a given input `x`.

The function uses the standard normal distribution (mean = 0, standard
deviation = 1) to calculate the probability that a normally distributed
random variable is less than or equal to `x`. This is commonly referred
to as `N(x)` in financial and statistical contexts.

## [§](#parameters){.doc-anchor}Parameters

- `x: Decimal` The input value for which the CDF is computed. Must be
  convertible to `f64`.

## [§](#returns){.doc-anchor}Returns

- `Ok(Decimal)`: The CDF value corresponding to the input `x`.
- `Err(DecimalError)`: Returns an error if the conversion from `Decimal`
  to `f64` fails.

## [§](#errors){.doc-anchor}Errors

Returns a `DecimalError::ConversionError` if:

- The input `x` cannot be converted to an `f64`.

## [§](#notes){.doc-anchor}Notes

This function uses the [`statrs`](https://docs.rs/statrs/latest/statrs/)
crate to model the standard normal distribution and compute the CDF. The
result is returned as a `Decimal` for precision.

## [§](#example){.doc-anchor}Example

::: example-wrap
``` {.rust .rust-example-rendered}
use rust_decimal::Decimal;
use tracing::{error, info};
use optionstratlib::greeks::big_n;

let x = Decimal::new(100, 2);

match big_n(x) {
    Ok(result) => info!("N(x): {}", result),
    Err(e) => error!("Error: {:?}", e),
}
```
:::
::::
:::::::
::::::::
