::::::: width-limiter
:::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../../index.html)::[model](../index.html)
:::

# Module decimalCopy item path

[[Source](../../../src/optionstratlib/model/decimal.rs.html#6-512){.src}
]{.sub-heading}
::::

Expand description

::: docblock
Core utilities for handling decimal numbers in financial calculations.
:::

## Constants[§](#constants){.anchor} {#constants .section-header}

[ONE_DAY](constant.ONE_DAY.html "constant optionstratlib::model::decimal::ONE_DAY"){.constant}
:   Represents the daily interest rate factor used for financial
    calculations, approximately equivalent to 1/252 (a standard value
    for the number of trading days in a year).

## Traits[§](#traits){.anchor} {#traits .section-header}

[DecimalStats](trait.DecimalStats.html "trait optionstratlib::model::decimal::DecimalStats"){.trait}
:   Defines statistical operations for collections of decimal values.

## Functions[§](#functions){.anchor} {#functions .section-header}

[decimal_normal_sample](fn.decimal_normal_sample.html "fn optionstratlib::model::decimal::decimal_normal_sample"){.fn}
:   Generates a random positive value from a standard normal
    distribution.

[decimal_to_f64](fn.decimal_to_f64.html "fn optionstratlib::model::decimal::decimal_to_f64"){.fn}
:   Converts a Decimal value to an f64.

[f64_to_decimal](fn.f64_to_decimal.html "fn optionstratlib::model::decimal::f64_to_decimal"){.fn}
:   Converts an f64 floating-point number to a Decimal.
::::::
:::::::
