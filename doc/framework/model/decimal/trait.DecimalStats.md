:::::::::::::::::: width-limiter
::::::::::::::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../../index.html)::[model](../index.html)::[decimal](index.html)
:::

# Trait [DecimalStats]{.trait}Copy item path

[[Source](../../../src/optionstratlib/model/decimal.rs.html#91-105){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub trait DecimalStats {
    // Required methods
    fn mean(&self) -> Decimal;
    fn std_dev(&self) -> Decimal;
}
```

Expand description

:::: docblock
Defines statistical operations for collections of decimal values.

This trait provides methods to calculate common statistical measures for
sequences or collections of `Decimal` values. It allows implementing
types to offer standardized statistical analysis capabilities.

### [§](#key-features){.doc-anchor}Key Features

- Basic statistical calculations for `Decimal` collections
- Consistent interface for various collection types
- Precision-preserving operations using the `Decimal` type

### [§](#available-statistics){.doc-anchor}Available Statistics

- `mean`: Calculates the arithmetic mean (average) of the values
- `std_dev`: Calculates the standard deviation, measuring the dispersion
  from the mean

### [§](#example){.doc-anchor}Example

::: example-wrap
``` {.rust .rust-example-rendered}
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use optionstratlib::model::decimal::DecimalStats;

struct DecimalSeries(Vec<Decimal>);

impl DecimalStats for DecimalSeries {
    fn mean(&self) -> Decimal {
        let sum: Decimal = self.0.iter().sum();
        if self.0.is_empty() {
            dec!(0)
        } else {
            sum / Decimal::from(self.0.len())
        }
    }
     
    fn std_dev(&self) -> Decimal {
        // Implementation of standard deviation calculation
        // ...
        dec!(0) // Placeholder return
    }
}
```
:::
::::

## Required Methods[§](#required-methods){.anchor} {#required-methods .section-header}

::::::: methods
::: {#tymethod.mean .section .method}
[Source](../../../src/optionstratlib/model/decimal.rs.html#96){.src
.rightside}

#### fn [mean](#tymethod.mean){.fn}(&self) -\> Decimal {#fn-meanself---decimal .code-header}
:::

::: docblock
Calculates the arithmetic mean (average) of the collection.

The mean is the sum of all values divided by the count of values. This
method should handle empty collections appropriately.
:::

::: {#tymethod.std_dev .section .method}
[Source](../../../src/optionstratlib/model/decimal.rs.html#104){.src
.rightside}

#### fn [std_dev](#tymethod.std_dev){.fn}(&self) -\> Decimal {#fn-std_devself---decimal .code-header}
:::

::: docblock
Calculates the standard deviation of the collection.

The standard deviation measures the amount of variation or dispersion
from the mean. A low standard deviation indicates that values tend to be
close to the mean, while a high standard deviation indicates values are
spread out over a wider range.
:::
:::::::

## Implementations on Foreign Types[§](#foreign-impls){.anchor} {#foreign-impls .section-header}

::: {#impl-DecimalStats-for-Vec%3CDecimal%3E .section .impl}
[Source](../../../src/optionstratlib/model/decimal.rs.html#126-143){.src
.rightside}[§](#impl-DecimalStats-for-Vec%3CDecimal%3E){.anchor}

### impl [DecimalStats](trait.DecimalStats.html "trait optionstratlib::model::decimal::DecimalStats"){.trait} for [Vec](https://doc.rust-lang.org/1.86.0/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<Decimal\> {#impl-decimalstats-for-vecdecimal .code-header}
:::

::::: impl-items
::: {#method.mean .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/decimal.rs.html#127-133){.src
.rightside}[§](#method.mean){.anchor}

#### fn [mean](#tymethod.mean){.fn}(&self) -\> Decimal {#fn-meanself---decimal-1 .code-header}
:::

::: {#method.std_dev .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/decimal.rs.html#135-142){.src
.rightside}[§](#method.std_dev){.anchor}

#### fn [std_dev](#tymethod.std_dev){.fn}(&self) -\> Decimal {#fn-std_devself---decimal-1 .code-header}
:::
:::::

## Implementors[§](#implementors){.anchor} {#implementors .section-header}

::: {#implementors-list}
:::
:::::::::::::::::
::::::::::::::::::
