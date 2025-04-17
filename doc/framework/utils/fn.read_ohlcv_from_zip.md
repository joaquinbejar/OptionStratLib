::::::: width-limiter
:::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../index.html)::[utils](index.html)
:::

# Function [read_ohlcv_from_zip]{.fn}Copy item path

[[Source](../../src/optionstratlib/utils/csv.rs.html#142-240){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub fn read_ohlcv_from_zip(
    zip_path: &str,
    start_date: Option<&str>,
    end_date: Option<&str>,
) -> Result<Vec<OhlcvCandle>, OhlcvError>
```

Expand description

::: docblock
Reads OHLCV data from a zipped CSV file and filters it by date range

## [§](#arguments){.doc-anchor}Arguments

- `zip_path` - Path to the ZIP file containing the CSV
- `start_date` - Optional start date in DD/MM/YYYY format (inclusive)
- `end_date` - Optional end date in DD/MM/YYYY format (inclusive)

## [§](#returns){.doc-anchor}Returns

A vector of OhlcvCandle structs containing the filtered data

## [§](#errors){.doc-anchor}Errors

Returns an OhlcvError if:

- The ZIP file cannot be opened
- The CSV file within the ZIP cannot be read
- The date range is invalid
- Data parsing fails
:::
::::::
:::::::
