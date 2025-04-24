::::::::::: width-limiter
:::::::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../index.html)::[surfaces](index.html)
:::

# Trait [Surfacable]{.trait}Copy item path

[[Source](../../src/optionstratlib/surfaces/traits.rs.html#44-49){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub trait Surfacable {
    // Required method
    fn surface(&self) -> Result<Surface, SurfaceError>;
}
```

Expand description

::: docblock
A trait for objects that can generate a mathematical surface in 3D
space.

This trait defines a single method,
[`Surfacable::surface`](trait.Surfacable.html#tymethod.surface "method optionstratlib::surfaces::Surfacable::surface"),
which is responsible for calculating or constructing a
[`Surface`](struct.Surface.html "struct optionstratlib::surfaces::Surface")
representation of the object that implements it. The surface may be
created through direct construction or as the result of some
computational process.

## [§](#errors){.doc-anchor}Errors

The
[`Surfacable::surface`](trait.Surfacable.html#tymethod.surface "method optionstratlib::surfaces::Surfacable::surface")
method returns a
[`Result`](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result")
containing the generated surface on success. If an error occurs during
the surface generation process, a
[`SurfaceError`](../error/enum.SurfaceError.html "enum optionstratlib::error::SurfaceError")
is returned instead. Potential errors could include:

- Invalid inputs or parameters leading to a
  [`SurfaceError::Point3DError`](../error/enum.SurfaceError.html#variant.Point3DError "variant optionstratlib::error::SurfaceError::Point3DError")
  or
  [`SurfaceError::ConstructionError`](../error/enum.SurfaceError.html#variant.ConstructionError "variant optionstratlib::error::SurfaceError::ConstructionError").
- Failures during surface computation due to invalid operations (e.g.,
  [`SurfaceError::OperationError`](../error/enum.SurfaceError.html#variant.OperationError "variant optionstratlib::error::SurfaceError::OperationError")).
- General-purpose errors, such as I/O or analysis issues, represented as
  [`SurfaceError::StdError`](../error/enum.SurfaceError.html#variant.StdError "variant optionstratlib::error::SurfaceError::StdError")
  or
  [`SurfaceError::AnalysisError`](../error/enum.SurfaceError.html#variant.AnalysisError "variant optionstratlib::error::SurfaceError::AnalysisError").

## [§](#implementors-1){.doc-anchor}Implementors {#implementors-1}

of this trait should define how their specific type generates a
[`Surface`](struct.Surface.html "struct optionstratlib::surfaces::Surface").
This could involve:

- Utilizing existing 3D geometry to build the surface.
- Analytical or procedural computations to construct a surface from
  data.
- Interactions with external processes or datasets.

## [§](#see-also){.doc-anchor}See Also

For more details on specific error variants, refer to the
[`SurfaceError`](../error/enum.SurfaceError.html "enum optionstratlib::error::SurfaceError")
enum. For details about the underlying mathematical representation,
refer to the
[`Surface`](struct.Surface.html "struct optionstratlib::surfaces::Surface")
struct.

## [§](#example-use-cases){.doc-anchor}Example Use Cases

This trait can be used in scenarios where multiple types need to provide
a unified interface for surface generation. For instance, different
shapes (spheres, planes, or curves) may implement `Surfacable` so that
they can all produce
[`Surface`](struct.Surface.html "struct optionstratlib::surfaces::Surface")
outputs in a consistent manner.

## [§](#related-modules){.doc-anchor}Related Modules

- **`crate::surfaces::surface`**: Contains the
  [`Surface`](struct.Surface.html "struct optionstratlib::surfaces::Surface")
  structure and its related components.
- **`crate::error::surfaces`**: Contains the
  [`SurfaceError`](../error/enum.SurfaceError.html "enum optionstratlib::error::SurfaceError")
  type and its variants for error representation.
:::

## Required Methods[§](#required-methods){.anchor} {#required-methods .section-header}

::::: methods
::: {#tymethod.surface .section .method}
[Source](../../src/optionstratlib/surfaces/traits.rs.html#48){.src
.rightside}

#### fn [surface](#tymethod.surface){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Surface](struct.Surface.html "struct optionstratlib::surfaces::Surface"){.struct}, [SurfaceError](../error/enum.SurfaceError.html "enum optionstratlib::error::SurfaceError"){.enum}\> {#fn-surfaceself---resultsurface-surfaceerror .code-header}
:::

::: docblock
- `surface()`:
  - Returns: `Result<Surface, SurfaceError>`
  - Description: Generates a surface or returns an error if something
    goes wrong during the process.
:::
:::::

## Implementors[§](#implementors){.anchor} {#implementors .section-header}

::: {#implementors-list}
:::
::::::::::
:::::::::::
