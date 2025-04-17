::::::: width-limiter
:::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../../index.html)::[model](../index.html)::[decimal](index.html)
:::

# Constant [ONE_DAY]{.constant}Copy item path

[[Source](../../../src/optionstratlib/model/decimal.rs.html#30){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub const ONE_DAY: Decimal;
```

Expand description

::: docblock
Represents the daily interest rate factor used for financial
calculations, approximately equivalent to 1/252 (a standard value for
the number of trading days in a year).

This constant converts annual interest rates to daily rates by providing
a division factor. The value 0.00396825397 corresponds to 1/252, where
252 is the typical number of trading days in a financial year.

## [§](#usage){.doc-anchor}Usage

This constant is commonly used in financial calculations such as:

- Converting annual interest rates to daily rates
- Time value calculations for options pricing
- Discounting cash flows on a daily basis
- Interest accrual calculations
:::
::::::
:::::::
