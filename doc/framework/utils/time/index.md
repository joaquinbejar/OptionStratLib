::::::: width-limiter
:::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../../index.html)::[utils](../index.html)
:::

# Module time Copy item path

[[Source](../../../src/optionstratlib/utils/time.rs.html#6-555){.src}
]{.sub-heading}
::::

Expand description

::: docblock
Module for time-related utilities.
:::

## Enums[§](#enums){.anchor} {#enums .section-header}

[TimeFrame](enum.TimeFrame.html "enum optionstratlib::utils::time::TimeFrame"){.enum}
:   Represents different timeframes for volatility calculations.

## Functions[§](#functions){.anchor} {#functions .section-header}

[convert_time_frame](fn.convert_time_frame.html "fn optionstratlib::utils::time::convert_time_frame"){.fn}
:   Converts a value from one TimeFrame to another.

[get_today_formatted](fn.get_today_formatted.html "fn optionstratlib::utils::time::get_today_formatted"){.fn}
:   Returns the current date formatted as "dd-mmm-yyyy" in lowercase.

[get_today_or_tomorrow_formatted](fn.get_today_or_tomorrow_formatted.html "fn optionstratlib::utils::time::get_today_or_tomorrow_formatted"){.fn}
:   Formats the current date or the next day's date based on the current
    UTC time.

[get_tomorrow_formatted](fn.get_tomorrow_formatted.html "fn optionstratlib::utils::time::get_tomorrow_formatted"){.fn}
:   Returns tomorrow's date in "dd-mmm-yyyy" format (lowercase).

[get_x_days_formatted](fn.get_x_days_formatted.html "fn optionstratlib::utils::time::get_x_days_formatted"){.fn}
:   Formats a date a specified number of days from the current date.

[get_x_days_formatted_pos](fn.get_x_days_formatted_pos.html "fn optionstratlib::utils::time::get_x_days_formatted_pos"){.fn}
:   Returns a formatted date string representing the date `x` days in
    the future.

[units_per_year](fn.units_per_year.html "fn optionstratlib::utils::time::units_per_year"){.fn}
:   Returns the number of units per year for each TimeFrame.
::::::
:::::::
