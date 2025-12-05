::::::: width-limiter
:::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../index.html)
:::

# Module surfaces Copy item path

[[Source](../../src/optionstratlib/surfaces/mod.rs.html#1-22){.src}
]{.sub-heading}
::::

Expand description

::: docblock
- `surfaces` - Volatility surface and other 3D financial data modeling.

Tools for constructing, manipulating, and analyzing volatility surfaces
and other three-dimensional financial data structures. Includes
interpolation methods, fitting algorithms, and visualization utilities.
This module provides tools for working with 3D surfaces.

It includes functionalities for defining surfaces, performing operations
on them, and visualizing them. The core components are:

- `Surface`: Represents a 3D surface. See the `surface` module for more
  details.
- `Point3D`: Represents a point in 3D space. See the `types` module for
  more details.
- `utils`: Contains utility functions for working with surfaces. See the
  `utils` module for more details.
- `visualization`: Provides tools for visualizing surfaces. See the
  `visualization` module for more details.
:::

## Structs[§](#structs){.anchor} {#structs .section-header}

[Point3D](struct.Point3D.html "struct optionstratlib::surfaces::Point3D"){.struct}
:   Represents a point in three-dimensional space with `x`, `y` and `z`
    coordinates.

[Surface](struct.Surface.html "struct optionstratlib::surfaces::Surface"){.struct}
:   Represents a mathematical surface in 3D space.

## Traits[§](#traits){.anchor} {#traits .section-header}

[BasicSurfaces](trait.BasicSurfaces.html "trait optionstratlib::surfaces::BasicSurfaces"){.trait}
:   BasicSurfaces Trait

[Surfacable](trait.Surfacable.html "trait optionstratlib::surfaces::Surfacable"){.trait}
:   A trait for objects that can generate a mathematical surface in 3D
    space.
::::::
:::::::
