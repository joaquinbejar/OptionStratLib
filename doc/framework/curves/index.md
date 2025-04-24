::::::: width-limiter
:::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../index.html)
:::

# Module curvesCopy item path

[[Source](../../src/optionstratlib/curves/mod.rs.html#1-13){.src}
]{.sub-heading}
::::

Expand description

::: docblock
- `curves` - Tools for yield curves, term structures, and other
  financial curves.

Implementations of various interest rate curves, forward curves, and
term structures used in options pricing and risk management. Includes
interpolation methods and curve fitting algorithms.
:::

## Modules[§](#modules){.anchor} {#modules .section-header}

[visualization](visualization/index.html "mod optionstratlib::curves::visualization"){.mod}
:   Curve Visualization Module

## Structs[§](#structs){.anchor} {#structs .section-header}

[Curve](struct.Curve.html "struct optionstratlib::curves::Curve"){.struct}
:   Represents a mathematical curve as a collection of 2D points.

[Point2D](struct.Point2D.html "struct optionstratlib::curves::Point2D"){.struct}
:   Represents a point in two-dimensional space with `x` and `y`
    coordinates.

## Traits[§](#traits){.anchor} {#traits .section-header}

[BasicCurves](trait.BasicCurves.html "trait optionstratlib::curves::BasicCurves"){.trait}
:   A trait for generating financial option curves based on different
    parameters.

[Curvable](trait.Curvable.html "trait optionstratlib::curves::Curvable"){.trait}
:   A trait that defines the behavior of any object that can produce a
    curve representation.

[StatisticalCurve](trait.StatisticalCurve.html "trait optionstratlib::curves::StatisticalCurve"){.trait}
:   A trait for generating statistical curves based on metrics

## Functions[§](#functions){.anchor} {#functions .section-header}

[create_constant_curve](fn.create_constant_curve.html "fn optionstratlib::curves::create_constant_curve"){.fn}
:   Creates a constant curve with equidistant points along the x-axis
    and the same constant value for the y-axis.

[create_linear_curve](fn.create_linear_curve.html "fn optionstratlib::curves::create_linear_curve"){.fn}
:   Creates a linear curve defined by a starting point, an ending point,
    and a slope.
::::::
:::::::
