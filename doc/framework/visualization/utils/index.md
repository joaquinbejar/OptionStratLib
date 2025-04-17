::::::: width-limiter
:::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../../index.html)::[visualization](../index.html)
:::

# Module utilsCopy item path

[[Source](../../../src/optionstratlib/visualization/utils.rs.html#1-1227){.src}
]{.sub-heading}
::::

Expand description

::: docblock
This sub-module contains various utility functions used throughout the
crate.
:::

## Enums[§](#enums){.anchor} {#enums .section-header}

[GraphBackend](enum.GraphBackend.html "enum optionstratlib::visualization::utils::GraphBackend"){.enum}
:   Defines the backend for rendering graphs. Different backends are
    available depending on the target architecture.

## Traits[§](#traits){.anchor} {#traits .section-header}

[Graph](trait.Graph.html "trait optionstratlib::visualization::utils::Graph"){.trait}
:   Trait for creating graphs of profit calculations. This trait extends
    the `Profit` trait, adding the functionality to visualize profit
    calculations.

## Functions[§](#functions){.anchor} {#functions .section-header}

[apply_shade](fn.apply_shade.html "fn optionstratlib::visualization::utils::apply_shade"){.fn}
:   Applies a color gradient effect by interpolating between a base
    color and a derived color.

[draw_points_on_chart](fn.draw_points_on_chart.html "fn optionstratlib::visualization::utils::draw_points_on_chart"){.fn}
:   Draws chart points and their associated labels on a chart context.

[draw_vertical_lines_on_chart](fn.draw_vertical_lines_on_chart.html "fn optionstratlib::visualization::utils::draw_vertical_lines_on_chart"){.fn}
:   Draws vertical lines with labels on a given chart using the
    specified drawing backend.

[random_color](fn.random_color.html "fn optionstratlib::visualization::utils::random_color"){.fn}
:   Creates a random, visually distinguishable color.
::::::
:::::::
