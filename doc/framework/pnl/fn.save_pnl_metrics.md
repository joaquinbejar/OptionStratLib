::::::: width-limiter
:::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../index.html)::[pnl](index.html)
:::

# Function [save_pnl_metrics]{.fn} Copy item path

[[Source](../../src/optionstratlib/pnl/metrics.rs.html#216-228){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub fn save_pnl_metrics(
    metrics: &[PnLMetricsStep],
    file_path: &str,
) -> Result<()>
```

Expand description

::: docblock
Serializes a vector of PnLMetricsStep to compact JSON and saves it to a
file

Similar to save_pnl_metrics_to_json but creates a compact representation
without extra whitespace, resulting in smaller file size

## [§](#arguments){.doc-anchor}Arguments

- `metrics` - Vector of PnLMetricsStep to serialize
- `file_path` - Path where the JSON file will be saved

## [§](#returns){.doc-anchor}Returns

- `io::Result<()>` - Success or error result
:::
::::::
:::::::
