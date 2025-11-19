:::::::: width-limiter
::::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../../index.html)::[utils](../index.html)::[time](index.html)
:::

# Function [get_today_formatted]{.fn} Copy item path

[[Source](../../../src/optionstratlib/utils/time.rs.html#287-290){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub fn get_today_formatted() -> String
```

Expand description

:::: docblock
Returns the current date formatted as "dd-mmm-yyyy" in lowercase.

## [§](#examples){.doc-anchor}Examples

::: example-wrap
``` {.rust .rust-example-rendered}
use chrono::Local;
use optionstratlib::utils::time::get_today_formatted;

let today_formatted = get_today_formatted();
let expected_format = Local::now().date_naive().format("%d-%b-%Y").to_string().to_lowercase();
assert_eq!(today_formatted, expected_format);
```
:::
::::
:::::::
::::::::
