::::::: width-limiter
:::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../../index.html)::[utils](../index.html)::[time](index.html)
:::

# Function [get_x_days_formatted]{.fn}Copy item path

[[Source](../../../src/optionstratlib/utils/time.rs.html#242-245){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub fn get_x_days_formatted(days: i64) -> String
```

Expand description

::: docblock
Formats a date a specified number of days from the current date.

This function calculates the date that is `days` days from the current
date and formats it as a lowercase string in the format "dd-mmm-yyyy".
For example, if the current date is 2024-11-20 and `days` is 1, the
returned string will be "21-nov-2024".

## [§](#arguments){.doc-anchor}Arguments

- `days`: The number of days to offset from the current date. This can
  be positive or negative.

## [§](#returns){.doc-anchor}Returns

A lowercase string representing the calculated date in "dd-mmm-yyyy"
format.
:::
::::::
:::::::
