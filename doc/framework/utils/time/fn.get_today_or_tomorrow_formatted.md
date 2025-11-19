:::::::: width-limiter
::::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../../index.html)::[utils](../index.html)::[time](index.html)
:::

# Function [get_today_or_tomorrow_formatted]{.fn} Copy item path

[[Source](../../../src/optionstratlib/utils/time.rs.html#313-323){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub fn get_today_or_tomorrow_formatted() -> String
```

Expand description

:::: docblock
Formats the current date or the next day's date based on the current UTC
time.

The function checks the current UTC time against a cutoff time of
18:30:00. If the current time is past the cutoff, the date for the next
day is returned. Otherwise, the current date is returned. The returned
date is formatted as `dd-mmm-yyyy` in lowercase. Note that getting the
next day is done safely, handling potential overflow (e.g. the last day
of the year).

Returns:

A lowercase String representing the formatted date.

## [§](#examples){.doc-anchor}Examples

::: example-wrap
``` {.rust .rust-example-rendered}
use chrono::{Utc, NaiveTime, Timelike};
use tracing::info;
use optionstratlib::utils::time::get_today_or_tomorrow_formatted;

info!("{}", get_today_or_tomorrow_formatted());
```
:::
::::
:::::::
::::::::
