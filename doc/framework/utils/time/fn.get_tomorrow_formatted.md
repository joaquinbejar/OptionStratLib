:::::::: width-limiter
::::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../../index.html)::[utils](../index.html)::[time](index.html)
:::

# Function [get_tomorrow_formatted]{.fn}Copy item path

[[Source](../../../src/optionstratlib/utils/time.rs.html#221-224){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub fn get_tomorrow_formatted() -> String
```

Expand description

:::: docblock
Returns tomorrow's date in "dd-mmm-yyyy" format (lowercase).

## [§](#examples){.doc-anchor}Examples

::: example-wrap
``` {.rust .rust-example-rendered}
use tracing::info;
use optionstratlib::utils::time::get_tomorrow_formatted;
let tomorrow = get_tomorrow_formatted();
info!("{}", tomorrow); // Output will vary depending on the current date.
```
:::
::::
:::::::
::::::::
