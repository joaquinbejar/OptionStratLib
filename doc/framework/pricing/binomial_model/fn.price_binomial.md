::::::: width-limiter
:::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../../index.html)::[pricing](../index.html)::[binomial_model](index.html)
:::

# Function [price_binomial]{.fn} Copy item path

[[Source](../../../src/optionstratlib/pricing/binomial_model.rs.html#90-139){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub fn price_binomial(
    params: BinomialPricingParams<'_>,
) -> Result<Decimal, PricingError>
```

Expand description

::: docblock
Calculates the price of an option using the binomial model.

This function implements the binomial model for option pricing, which is
a numerical method for estimating the price of both European and
American options. The model constructs a binomial tree of possible
future underlying asset prices and then recursively calculates the
option value from the leaves to the root of the tree.

## [§](#arguments){.doc-anchor}Arguments

- `params` - A `BinomialPricingParams` struct containing all necessary
  pricing parameters:
  - `asset`: Current price of the underlying asset.
  - `volatility`: Annualized volatility of the underlying asset.
  - `int_rate`: Annualized risk-free interest rate.
  - `strike`: Strike price of the option.
  - `expiry`: Time to expiration in years.
  - `no_steps`: Number of steps in the binomial tree.
  - `option_type`: Type of option (e.g., European, American).
  - `option_style`: Style of the option (Call or Put).
  - `side`: Side of the trade (Long or Short).

## [§](#returns){.doc-anchor}Returns

Returns the calculated price of the option as an `f64`.

## [§](#special-cases){.doc-anchor}Special cases

- If `expiry` is 0, the function returns the intrinsic value of the
  option.
- If `volatility` is 0, the function calculates the option price
  deterministically.

## [§](#notes){.doc-anchor}Notes

- The model's accuracy increases with the number of steps, but so does
  the computation time.
- This model assumes that the underlying asset follows a multiplicative
  binomial process.
- For American options, this model accounts for the possibility of early
  exercise.
:::
::::::
:::::::
