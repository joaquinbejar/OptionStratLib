:::::::: width-limiter
::::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../../index.html)::[pricing](../index.html)::[binomial_model](index.html)
:::

# Function [generate_binomial_tree]{.fn}Copy item path

[[Source](../../../src/optionstratlib/pricing/binomial_model.rs.html#188-251){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub fn generate_binomial_tree(
    params: &BinomialPricingParams<'_>,
) -> Result<(Vec<Vec<Decimal>>, Vec<Vec<Decimal>>), Box<dyn Error>>
```

Expand description

:::: docblock
Generates a binomial tree for option pricing.

## [§](#parameters){.doc-anchor}Parameters

- `params`: A reference to `BinomialPricingParams` which contains the
  parameters required for generating the binomial tree including
  expiration time, number of steps, volatility, interest rate, asset
  price, strike price, option type, and option style.

## [§](#returns){.doc-anchor}Returns

A tuple containing two vectors of vectors:

- `asset_tree`: The tree representing the possible future values of the
  asset at each step.
- `option_tree`: The tree representing the values of the option at each
  step.

The `generate_binomial_tree` function calculates the possible asset
prices and option prices at each node in a binomial tree based on the
input parameters.

1.  It calculates the time interval `dt` for each step.
2.  `u` and `d` are the factors by which the price increases or
    decreases.
3.  `p` is the risk-neutral probability.
4.  It initializes the `asset_tree` and `option_tree` with the
    appropriate dimensions.
5.  The asset prices are computed for all nodes.
6.  The option values are computed at maturity based on the payoff
    function.
7.  The option values are then back-propagated to compute the option
    value at the current time.

## [§](#example){.doc-anchor}Example

::: example-wrap
``` {.rust .rust-example-rendered}
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use optionstratlib::model::types::{OptionStyle, OptionType, Side};
use optionstratlib::pos;
use optionstratlib::pricing::binomial_model::{BinomialPricingParams, generate_binomial_tree};
use optionstratlib::Positive;
let params = BinomialPricingParams {
            asset: pos!(100.0),
            volatility: pos!(0.2),
            int_rate: dec!(0.05),
            strike: pos!(100.0),
            expiry: Positive::ONE,
            no_steps: 1000,
            option_type: &OptionType::European,
            option_style: &OptionStyle::Call,
            side: &Side::Long,
        };
let (asset_tree, option_tree) = generate_binomial_tree(&params).unwrap();
```
:::
::::
:::::::
::::::::
