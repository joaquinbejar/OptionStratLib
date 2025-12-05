::::::::::::: width-limiter
:::::::::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../index.html)::[pricing](index.html)
:::

# Trait [Payoff]{.trait} Copy item path

[[Source](../../src/optionstratlib/pricing/payoff.rs.html#42-55){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub trait Payoff {
    // Required method
    fn payoff(&self, info: &PayoffInfo) -> f64;
}
```

Expand description

:::: docblock
Defines a contract for calculating the payoff value of an option.

The `Payoff` trait establishes a standard interface for implementing
different option payoff calculations. Classes that implement this trait
can define specific payoff formulas for various option types (standard
calls/puts, exotic options, etc.).

## [§](#examples){.doc-anchor}Examples

Implementing the trait for a standard call option:

::: example-wrap
``` {.rust .rust-example-rendered}
use num_traits::ToPrimitive;
use optionstratlib::pricing::{Payoff, PayoffInfo};
use optionstratlib::Side;
struct CallOption;

impl Payoff for CallOption {
    fn payoff(&self, info: &PayoffInfo) -> f64 {
        let spot = info.spot.value().to_f64().unwrap();
        let strike = info.strike.value().to_f64().unwrap();
        match info.side {
            Side::Long => (spot - strike).max(0.0),
            Side::Short => -1.0 * (spot - strike).max(0.0),
        }
    }
}
```
:::

## [§](#usage){.doc-anchor}Usage

This trait is typically used within the options pricing module to:

- Create standardized payoff calculations for different option types
- Enable polymorphic handling of various option payoff strategies
- Support both standard and exotic option payoffs through a unified
  interface
::::

## Required Methods[§](#required-methods){.anchor} {#required-methods .section-header}

::::: methods
::: {#tymethod.payoff .section .method}
[Source](../../src/optionstratlib/pricing/payoff.rs.html#54){.src
.rightside}

#### fn [payoff](#tymethod.payoff){.fn}(&self, info: &[PayoffInfo](struct.PayoffInfo.html "struct optionstratlib::pricing::PayoffInfo"){.struct}) -\> [f64](https://doc.rust-lang.org/1.91.1/std/primitive.f64.html){.primitive} {#fn-payoffself-info-payoffinfo---f64 .code-header}
:::

::: docblock
Calculates the payoff value of an option based on the provided
information.

##### [§](#parameters){.doc-anchor}Parameters

- `info` - A reference to a `PayoffInfo` struct containing all necessary
  data for calculating the option's payoff, including spot price, strike
  price, option style, position side, and additional parameters for
  exotic options.

##### [§](#returns){.doc-anchor}Returns

Returns the calculated payoff value as a `f64`.
:::
:::::

## Implementors[§](#implementors){.anchor} {#implementors .section-header}

:::: {#implementors-list}
::: {#impl-Payoff-for-OptionType .section .impl}
[Source](../../src/optionstratlib/model/types.rs.html#255-292){.src
.rightside}[§](#impl-Payoff-for-OptionType){.anchor}

### impl [Payoff](trait.Payoff.html "trait optionstratlib::pricing::Payoff"){.trait} for [OptionType](../model/types/enum.OptionType.html "enum optionstratlib::model::types::OptionType"){.enum} {#impl-payoff-for-optiontype .code-header}
:::
::::
::::::::::::
:::::::::::::
