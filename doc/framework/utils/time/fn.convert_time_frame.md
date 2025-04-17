:::::::: width-limiter
::::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../../index.html)::[utils](../index.html)::[time](index.html)
:::

# Function [convert_time_frame]{.fn}Copy item path

[[Source](../../../src/optionstratlib/utils/time.rs.html#183-209){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub fn convert_time_frame(
    value: Positive,
    from_time_frame: &TimeFrame,
    to_time_frame: &TimeFrame,
) -> Positive
```

Expand description

:::: docblock
Converts a value from one TimeFrame to another.

## [§](#arguments){.doc-anchor}Arguments

- `value` - The value to convert
- `from_time_frame` - The source TimeFrame
- `to_time_frame` - The target TimeFrame

## [§](#returns){.doc-anchor}Returns

A Decimal representing the converted value

## [§](#examples){.doc-anchor}Examples

::: example-wrap
``` {.rust .rust-example-rendered}
use optionstratlib::{assert_pos_relative_eq, pos};
use optionstratlib::utils::time::convert_time_frame;
use optionstratlib::utils::TimeFrame;

// Convert 60 seconds to minutes
let result = convert_time_frame(pos!(60.0), &TimeFrame::Second, &TimeFrame::Minute);
assert_pos_relative_eq!(result, pos!(1.0), pos!(0.0000001));

// Convert 12 hours to days
let result = convert_time_frame(pos!(12.0), &TimeFrame::Hour, &TimeFrame::Day);
assert_pos_relative_eq!(result, pos!(0.5), pos!(0.0000001));
```
:::
::::
:::::::
::::::::
