::::::: width-limiter
:::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../index.html)::[pnl](index.html)
:::

# Function [create_pnl_metrics_document]{.fn} Copy item path

[[Source](../../src/optionstratlib/pnl/metrics.rs.html#290-306){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub fn create_pnl_metrics_document(
    metrics: Vec<PnLMetricsStep>,
    days: Positive,
    symbol: String,
    fee: Positive,
    delta: Decimal,
    delta_adjustment_at: Decimal,
) -> PnLMetricsDocument
```

Expand description

::: docblock
Creates a `PnLMetricsDocument` instance.

This function constructs a `PnLMetricsDocument` from provided profit and
loss metrics and associated parameters. It encapsulates the simulation
results and configuration details, providing a structured representation
of the analysis.

## [§](#arguments){.doc-anchor}Arguments

- `metrics`: A vector of `PnLMetricsStep` representing the individual
  steps in the profit and loss simulation.
- `days`: A `Positive` value indicating the number of days over which
  the simulation was run.
- `symbol`: A `String` representing the financial symbol (e.g., stock
  ticker) used in the simulation.
- `fee`: A `Positive` value representing the transaction fee applied
  during the simulation.
- `delta`: A `Decimal` representing the delta value used in the
  simulation.
- `delta_adjustment_at`: A `Decimal` representing the delta adjustment
  threshold used in the simulation.

## [§](#returns){.doc-anchor}Returns

A `PnLMetricsDocument` instance containing the provided metrics and
parameters.
:::
::::::
:::::::
