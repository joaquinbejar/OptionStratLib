::::::: width-limiter
:::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../../index.html)::[strategies](../index.html)::[probabilities](index.html)
:::

# Function [calculate_price_probability]{.fn} Copy item path

[[Source](../../../src/optionstratlib/strategies/probabilities/utils.rs.html#158-202){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub fn calculate_price_probability(
    current_price: &Positive,
    lower_bound: &Positive,
    upper_bound: &Positive,
    volatility_adj: Option<VolatilityAdjustment>,
    trend: Option<PriceTrend>,
    expiration_date: &ExpirationDate,
    risk_free_rate: Option<Decimal>,
) -> Result<(Positive, Positive), ProbabilityError>
```

Expand description

::: docblock
Calculate the probability of the underlying price being in different
ranges at expiration

## [§](#arguments){.doc-anchor}Arguments

- `current_price` - Current price of the underlying asset
- `lower_bound` - Lower boundary of the target price range
- `upper_bound` - Upper boundary of the target price range
- `volatility_adj` - Optional volatility adjustment parameters
- `trend` - Optional price trend parameters
- `expiration_date` - Expiration date of the analysis
- `risk_free_rate` - Optional risk-free rate

## [§](#returns){.doc-anchor}Returns

Returns a tuple containing:

- Probability of price being below the range
- Probability of price being within the range
- Probability of price being above the range

## [§](#errors){.doc-anchor}Errors

Returns an error if:

- Lower bound is greater than upper bound
- Time to expiry is not positive
- Volatility parameters are invalid
- Trend confidence is not between 0 and 1
:::
::::::
:::::::
