:::::::: width-limiter
::::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../../index.html)::[visualization](../index.html)::[binomial_tree](index.html)
:::

# Function [draw_binomial_tree]{.fn}Copy item path

[[Source](../../../src/optionstratlib/visualization/binomial_tree.rs.html#48-153){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub fn draw_binomial_tree(
    asset_tree: &[Vec<f64>],
    option_tree: &[Vec<f64>],
    backend: GraphBackend<'_>,
) -> Result<(), Box<dyn Error>>
```

Expand description

:::: docblock
This function draws a binomial tree of asset prices and option prices
using the plotters library and saves it to a specified file.

## [§](#arguments){.doc-anchor}Arguments

- `asset_tree` - A reference to a 2D vector containing the asset prices
  at each node of the binomial tree.
- `option_tree` - A reference to a 2D vector containing the option
  prices at each node of the binomial tree.
- `backend` - A GraphBackend object that represents the backend where
  the chart will be saved.

## [§](#returns){.doc-anchor}Returns

This function returns a `Result` which is `Ok` if the drawing and saving
process goes fine, otherwise it will return an error inside a `Box`.

## [§](#errors){.doc-anchor}Errors

This function may return an error if the drawing or saving process
fails.

## [§](#examples){.doc-anchor}Examples

::: example-wrap
``` {.rust .rust-example-rendered}
use optionstratlib::visualization::binomial_tree::draw_binomial_tree;
let asset_tree = vec![
    vec![100.0],
    vec![105.0, 95.0],
    vec![110.25, 99.75, 90.25],
    vec![115.25, 105.0, 95.0, 85.0],
];

let option_tree = vec![
    vec![5.0],
    vec![10.0, 0.0],
    vec![15.0, 5.0, 0.0],
    vec![20.0, 10.0, 0.0, 0.0],
];

let backend = GraphBackend::Bitmap { file_path: "./Draws/Binomial Tree/binomial_tree.png", size: (1200, 800) };
draw_binomial_tree(&asset_tree, &option_tree, backend).unwrap();
```
:::
::::
:::::::
::::::::
