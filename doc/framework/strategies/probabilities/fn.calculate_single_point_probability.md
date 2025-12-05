::::::: width-limiter
:::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../../index.html)::[strategies](../index.html)::[probabilities](index.html)
:::

# Function [calculate_single_point_probability]{.fn} Copy item path

[[Source](../../../src/optionstratlib/strategies/probabilities/utils.rs.html#64-130){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub fn calculate_single_point_probability(
    current_price: &Positive,
    target_price: &Positive,
    volatility_adj: Option<VolatilityAdjustment>,
    trend: Option<PriceTrend>,
    expiration_date: &ExpirationDate,
    risk_free_rate: Option<Decimal>,
) -> Result<(Positive, Positive), ProbabilityError>
```

Expand description

::: docblock
Calculates the probability of a stock price reaching a target price
within a given timeframe.

This function estimates the probability of a stock following a
log-normal distribution to reach a specified target price before
expiration. It also provides the probability of the stock price being
below or above the target price at the expiration date.

## [§](#parameters){.doc-anchor}Parameters

- `current_price`: The current stock price, represented as a `Positive`.
- `target_price`: The target stock price to evaluate, represented as a
  `Positive`.
- `volatility_adj`: An optional `VolatilityAdjustment` which includes
  base volatility and a standard deviation adjustment.
- `trend`: An optional `PriceTrend` providing the annual drift rate and
  confidence level for the trend.
- `expiration_date`: The date to which the probability is calculated, of
  type `ExpirationDate`.
- `risk_free_rate`: An optional risk-free rate (annual), defaulting to
  zero if not provided.

## [§](#returns){.doc-anchor}Returns

Returns a `Result` containing a tuple of two `Positive` values:

- `prob_below`: The probability of the stock price being below the
  target price at expiry.
- `prob_above`: The probability of the stock price being above the
  target price at expiry.

## [§](#errors){.doc-anchor}Errors

Returns an error string if:

- `time_to_expiry` is not positive, indicating the expiration date has
  passed or is invalid.
- `volatility_adj.base_volatility` is non-positive.
- `trend.confidence` is not between 0 and 1.
:::
::::::
:::::::
