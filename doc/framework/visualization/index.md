::::::: width-limiter
:::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../index.html)
:::

# Module visualizationCopy item path

[[Source](../../src/optionstratlib/visualization/mod.rs.html#1-42){.src}
]{.sub-heading}
::::

Expand description

::: docblock
- `visualization` - Tools for plotting and visual representation of
  options data.

Graphics and visualization utilities for creating charts, graphs, and
interactive plots of options data, strategies, and analytics. Supports
various plot types optimized for different aspects of options analysis.

## [§](#visualization){.doc-anchor}Visualization

This module provides tools for visualizing financial data and option
strategies using charts and diagrams.

### [§](#overview){.doc-anchor}Overview

The visualization module offers a set of utilities for generating visual
representations of financial data, particularly focused on options
pricing and trading strategies. It leverages the `plotters` library for
rendering high-quality charts and diagrams with cross-platform support.

### [§](#module-structure){.doc-anchor}Module Structure

- **binomial_tree**: Tools for visualizing binomial tree models used in
  options pricing.
- **model**: Data structures that represent visual elements like points,
  lines, and styling information.
- **utils**: Common utilities and traits for chart rendering, including
  the `Graph` trait and backend definitions.

### [§](#key-features){.doc-anchor}Key Features

- Platform-agnostic rendering with support for both native applications
  and WebAssembly
- Consistent styling and theming for financial visualizations
- Specialized components for options strategy visualization
- Flexible backend system allowing output to bitmap images or HTML5
  canvas

### [§](#cross-platform-support){.doc-anchor}Cross-Platform Support

The visualization module is designed to work across different platforms:

- **Native Applications**: Charts can be rendered to bitmap images (PNG,
  etc.)
- **WebAssembly**: When compiled to WebAssembly, charts can be rendered
  directly to HTML5 canvas elements

The appropriate backend is selected automatically based on compilation
targets.
:::

## Modules[§](#modules){.anchor} {#modules .section-header}

[binomial_tree](binomial_tree/index.html "mod optionstratlib::visualization::binomial_tree"){.mod}
:   This sub-module contains the implementation of the binomial tree
    model.

[utils](utils/index.html "mod optionstratlib::visualization::utils"){.mod}
:   This sub-module contains various utility functions used throughout
    the crate.
::::::
:::::::
