:::::::: width-limiter
::::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../index.html)::[pnl](index.html)
:::

# Function [save_pnl_metrics_with_document]{.fn}Copy item path

[[Source](../../src/optionstratlib/pnl/metrics.rs.html#373-424){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub fn save_pnl_metrics_with_document(
    document: &PnLMetricsDocument,
    file_path: &str,
) -> Result<()>
```

Expand description

:::: docblock
Saves PnL metrics to a JSON file, handling concurrent access and file
existence.

This function either appends the provided `PnLMetricsDocument` to an
existing JSON file or creates a new file containing the document within
a JSON array. It uses file locking to prevent race conditions when
multiple processes or threads attempt to write to the same file
simultaneously.

## [§](#arguments){.doc-anchor}Arguments

- `document` - A reference to the `PnLMetricsDocument` to be saved. The
  document is cloned before saving.
- `file_path` - A string slice representing the path to the JSON file.

## [§](#errors){.doc-anchor}Errors

Returns an `io::Error` if:

- The file cannot be opened for reading or writing.
- The existing file content cannot be read.
- The existing file content cannot be parsed as a JSON array of
  `PnLMetricsDocument`.
- The updated JSON array cannot be serialized.
- The file cannot be truncated.
- The data cannot be written to the file.

## [§](#thread-safety){.doc-anchor}Thread Safety

This function is thread-safe. It uses a file-based lock to prevent
concurrent writes to the file.

## [§](#example){.doc-anchor}Example

::: example-wrap
``` {.rust .rust-example-rendered}
use rust_decimal::Decimal;
use tracing::{error, info};
use optionstratlib::pnl::{save_pnl_metrics_with_document, PnLMetricsDocument};
use optionstratlib::pos;

// Assume 'document' is a valid PnLMetricsDocument instance
let document = PnLMetricsDocument {
    days: pos!(10.0),
    symbol: "AAPL".to_string(),
    fee: pos!(0.01),
    delta: Decimal::new(5, 1),
    delta_adjustment_at: Decimal::new(0, 0),
    metrics: vec![],
};
let file_path = "pnl_metrics.json";

match save_pnl_metrics_with_document(&document, file_path) {
    Ok(_) => info!("PnL metrics saved successfully."),
    Err(e) => error!("Error saving PnL metrics: {}", e),
}
```
:::
::::
:::::::
::::::::
