:::::::: width-limiter
::::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../../index.html)::[model](../index.html)::[decimal](index.html)
:::

# Function [f64_to_decimal]{.fn}Copy item path

[[Source](../../../src/optionstratlib/model/decimal.rs.html#281-287){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub fn f64_to_decimal(value: f64) -> Result<Decimal, DecimalError>
```

Expand description

:::: docblock
Converts an f64 floating-point number to a Decimal.

This function attempts to convert an f64 floating-point number to a
Decimal value. If the conversion fails (for example, if the f64
represents NaN, infinity, or is otherwise not representable as a
Decimal), it returns a DecimalError with detailed information about the
failure.

## [§](#parameters){.doc-anchor}Parameters

- `value` - The f64 value to convert

## [§](#returns){.doc-anchor}Returns

- `Result<Decimal, DecimalError>` - The converted Decimal value if
  successful, or a DecimalError if the conversion fails

## [§](#example){.doc-anchor}Example

::: example-wrap
``` {.rust .rust-example-rendered}
use rust_decimal::Decimal;
use tracing::info;
use optionstratlib::model::decimal::f64_to_decimal;

let float = std::f64::consts::PI;
match f64_to_decimal(float) {
    Ok(decimal) => info!("Converted to Decimal: {}", decimal),
    Err(e) => info!("Conversion error: {:?}", e)
}
```
:::
::::
:::::::
::::::::
