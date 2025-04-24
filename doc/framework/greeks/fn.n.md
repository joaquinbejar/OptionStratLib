::::::::: width-limiter
:::::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../index.html)::[greeks](index.html)
:::

# Function [n]{.fn}Copy item path

[[Source](../../src/optionstratlib/greeks/utils.rs.html#283-294){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub fn n(x: Decimal) -> Result<Decimal, GreeksError>
```

Expand description

::::: docblock
Computes the probability density function (PDF) of the standard normal
distribution for a given input `x`.

The PDF of the standard normal distribution is defined as:

::: example-wrap
``` language-math
N(x) = \frac{1}{\sqrt{2 \pi}} \cdot e^{-\frac{x^2}{2}}
```
:::

Where:

- (x): The input value for which the PDF is computed.

## [§](#parameters){.doc-anchor}Parameters

- `x: Decimal` The input value for which the standard normal PDF is
  calculated.

## [§](#returns){.doc-anchor}Returns

- `Ok(Decimal)`: The computed PDF value as a `Decimal`.
- `Err(GreeksError)`: Returns an error if the computation fails.

## [§](#calculation-details){.doc-anchor}Calculation Details

- The denominator is computed as (\\sqrt{2 \\pi}), where ( \\pi ) is
  approximated.
- The exponent is computed as (-\\frac{x\^2}{2}).
- The PDF value is the product of the reciprocal of the denominator and
  the exponential term.

## [§](#errors){.doc-anchor}Errors

- `GreeksError`: This function will return an error if any part of the
  calculation fails, though this is unlikely as the operations are
  well-defined for all finite inputs.

## [§](#example){.doc-anchor}Example

::: example-wrap
``` {.rust .rust-example-rendered}
use rust_decimal::Decimal;
use tracing::{error, info};
use optionstratlib::greeks::n;

let x = Decimal::new(100, 2); // 1.00

match n(x) {
    Ok(result) => info!("N(x): {}", result),
    Err(e) => error!("Error calculating N(x): {:?}", e),
}
```
:::

## [§](#notes){.doc-anchor}Notes

This function assumes that the constant `PI` is pre-defined as a
`Decimal` representing the value of (\\pi) to a sufficient precision for
the application.

The function uses the `Decimal` type for precision and error handling.
The result is returned
:::::
::::::::
:::::::::
