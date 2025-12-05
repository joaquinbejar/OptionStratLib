::::::: width-limiter
:::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../index.html)::[pnl](index.html)
:::

# Function [load_pnl_metrics]{.fn} Copy item path

[[Source](../../src/optionstratlib/pnl/metrics.rs.html#240-249){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub fn load_pnl_metrics(file_path: &str) -> Result<Vec<PnLMetricsStep>>
```

Expand description

::: docblock
Loads a vector of PnLMetricsStep from a JSON file

## [§](#arguments){.doc-anchor}Arguments

- `file_path` - Path to the JSON file

## [§](#returns){.doc-anchor}Returns

- `io::Result<Vec<PnLMetricsStep>>` - Vector of deserialized metrics or
  error
:::
::::::
:::::::
