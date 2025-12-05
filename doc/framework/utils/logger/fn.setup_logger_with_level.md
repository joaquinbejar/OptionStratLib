::::::: width-limiter
:::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../../index.html)::[utils](../index.html)::[logger](index.html)
:::

# Function [setup_logger_with_level]{.fn} Copy item path

[[Source](../../../src/optionstratlib/utils/logger.rs.html#122-141){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub fn setup_logger_with_level(log_level: &str)
```

Expand description

::: docblock
Sets up a logger with a user-specified log level for platforms

**Parameters:**

- `log_level`: The desired log level as a string. Supported levels are
  the same as for `setup_logger`.

**Behavior:**

- Concurrent calls to this function result in the logger being
  initialized only once.

## [§](#panics){.doc-anchor}Panics

This function panics if setting the default subscriber fails.
:::
::::::
:::::::
