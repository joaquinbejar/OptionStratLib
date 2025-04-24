:::::::: width-limiter
::::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../../index.html)::[model](../index.html)::[decimal](index.html)
:::

# Function [decimal_to_f64]{.fn}Copy item path

[[Source](../../../src/optionstratlib/model/decimal.rs.html#244-250){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub fn decimal_to_f64(value: Decimal) -> Result<f64, DecimalError>
```

Expand description

:::: docblock
Converts a Decimal value to an f64.

This function attempts to convert a Decimal value to an f64
floating-point number. If the conversion fails, it returns a
DecimalError with detailed information about the failure.

## [§](#parameters){.doc-anchor}Parameters

- `value` - The Decimal value to convert

## [§](#returns){.doc-anchor}Returns

- `Result<f64, DecimalError>` - The converted f64 value if successful,
  or a DecimalError if the conversion fails

## [§](#example){.doc-anchor}Example

::: example-wrap
``` {.rust .rust-example-rendered}
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use tracing::info;
use optionstratlib::model::decimal::decimal_to_f64;

let decimal = dec!(3.14159);
match decimal_to_f64(decimal) {
    Ok(float) => info!("Converted to f64: {}", float),
    Err(e) => info!("Conversion error: {:?}", e)
}
```
:::
::::
:::::::
::::::::
