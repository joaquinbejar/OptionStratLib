::::::: width-limiter
:::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../../index.html)::[utils](../index.html)::[logger](index.html)
:::

# Function [setup_logger]{.fn} Copy item path

[[Source](../../../src/optionstratlib/utils/logger.rs.html#88-109){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub fn setup_logger()
```

Expand description

::: docblock
Sets up a logger for the application

The logger level is determined by the `LOGLEVEL` environment variable.
Supported log levels are:

- `DEBUG`: Captures detailed debug information.
- `ERROR`: Captures error messages.
- `WARN`: Captures warnings.
- `TRACE`: Captures detailed trace logs.
- All other values default to `INFO`, which captures general
  information.

**Behavior:**

- Concurrent calls to this function result in the logger being
  initialized only once.

## [§](#panics){.doc-anchor}Panics

This function panics if setting the default subscriber fails.
:::
::::::
:::::::
