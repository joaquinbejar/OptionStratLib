::::::: width-limiter
:::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](index.html)
:::

# Macro [create_drawing_area]{.macro}Copy item path

[[Source](../src/optionstratlib/visualization/utils.rs.html#76-82){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
macro_rules! create_drawing_area {
    ($file_path:expr, $width:expr, $height:expr) => { ... };
}
```

Expand description

::: docblock
Creates a drawing area with a white background.

## [§](#arguments){.doc-anchor}Arguments

- `$file_path` - The path to the output image file.
- `$width` - The width of the drawing area.
- `$height` - The height of the drawing area.

## [§](#returns){.doc-anchor}Returns

A `DrawingArea` object.

## [§](#errors){.doc-anchor}Errors

Returns an error if the drawing area cannot be created.
:::
::::::
:::::::
