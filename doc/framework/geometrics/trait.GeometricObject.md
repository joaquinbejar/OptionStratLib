::::::::::::::::::::::::::::::::::::: width-limiter
:::::::::::::::::::::::::::::::::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../index.html)::[geometrics](index.html)
:::

# Trait [GeometricObject]{.trait} Copy item path

[[Source](../../src/optionstratlib/geometrics/utils.rs.html#12-65){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub trait GeometricObject<Point: Clone, Input> {
    type Error;

    // Required methods
    fn get_points(&self) -> BTreeSet<&Point>;
    fn from_vector<T>(points: Vec<T>) -> Self
       where Self: Sized,
             T: Into<Point> + Clone;
    fn construct<T>(method: T) -> Result<Self, Self::Error>
       where Self: Sized,
             T: Into<ConstructionMethod<Point, Input>>;

    // Provided methods
    fn vector(&self) -> Vec<&Point> { ... }
    fn to_vector(&self) -> Vec<&Point> { ... }
    fn calculate_range<I>(iter: I) -> (Decimal, Decimal)
       where I: Iterator<Item = Decimal> { ... }
}
```

Expand description

::: docblock
Defines a geometric object constructed from a set of points. Provides
methods for creating, accessing, and manipulating these objects.
:::

## Required Associated Types[§](#required-associated-types){.anchor} {#required-associated-types .section-header}

::::: methods
::: {#associatedtype.Error .section .method}
[Source](../../src/optionstratlib/geometrics/utils.rs.html#14){.src
.rightside}

#### type [Error](#associatedtype.Error){.associatedtype} {#type-error .code-header}
:::

::: docblock
Type alias for any errors that might occur during the construction of
the geometric object.
:::
:::::

## Required Methods[§](#required-methods){.anchor} {#required-methods .section-header}

::::::::::: methods
::: {#tymethod.get_points .section .method}
[Source](../../src/optionstratlib/geometrics/utils.rs.html#18){.src
.rightside}

#### fn [get_points](#tymethod.get_points){.fn}(&self) -\> [BTreeSet](https://doc.rust-lang.org/1.91.1/alloc/collections/btree/set/struct.BTreeSet.html "struct alloc::collections::btree::set::BTreeSet"){.struct}\<[&Point](https://doc.rust-lang.org/1.91.1/std/primitive.reference.html){.primitive}\> {#fn-get_pointsself---btreesetpoint .code-header}
:::

::: docblock
Returns a `BTreeSet` containing references to the points that constitute
the geometric object. The `BTreeSet` ensures that the points are ordered
and unique.
:::

:::: {#tymethod.from_vector .section .method}
[Source](../../src/optionstratlib/geometrics/utils.rs.html#31-34){.src
.rightside}

#### fn [from_vector](#tymethod.from_vector){.fn}\<T\>(points: [Vec](https://doc.rust-lang.org/1.91.1/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<T\>) -\> Self {#fn-from_vectortpoints-vect---self .code-header}

::: where
where Self:
[Sized](https://doc.rust-lang.org/1.91.1/core/marker/trait.Sized.html "trait core::marker::Sized"){.trait},
T:
[Into](https://doc.rust-lang.org/1.91.1/core/convert/trait.Into.html "trait core::convert::Into"){.trait}\<Point\> +
[Clone](https://doc.rust-lang.org/1.91.1/core/clone/trait.Clone.html "trait core::clone::Clone"){.trait},
:::
::::

::: docblock
Creates a new geometric object from a `Vec` of points.

The generic type `T` represents the input point type, which can be
converted into the `Point` type associated with the geometric object.
:::

:::: {#tymethod.construct .section .method}
[Source](../../src/optionstratlib/geometrics/utils.rs.html#43-46){.src
.rightside}

#### fn [construct](#tymethod.construct){.fn}\<T\>(method: T) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<Self, Self::[Error](trait.GeometricObject.html#associatedtype.Error "type optionstratlib::geometrics::GeometricObject::Error"){.associatedtype}\> {#fn-constructtmethod-t---resultself-selferror .code-header}

::: where
where Self:
[Sized](https://doc.rust-lang.org/1.91.1/core/marker/trait.Sized.html "trait core::marker::Sized"){.trait},
T:
[Into](https://doc.rust-lang.org/1.91.1/core/convert/trait.Into.html "trait core::convert::Into"){.trait}\<[ConstructionMethod](enum.ConstructionMethod.html "enum optionstratlib::geometrics::ConstructionMethod"){.enum}\<Point,
Input\>\>,
:::
::::

::: docblock
Constructs a geometric object using a specific construction method.

The generic type `T` represents a type that can be converted into a
`ConstructionMethod`. The `ConstructionMethod` enum provides different
strategies for building geometric objects, such as constructing from a
set of data points or from a parametric function.

This method returns a `Result` to handle potential errors during
construction.
:::
:::::::::::

## Provided Methods[§](#provided-methods){.anchor} {#provided-methods .section-header}

:::::::::: methods
::: {#method.vector .section .method}
[Source](../../src/optionstratlib/geometrics/utils.rs.html#22-24){.src
.rightside}

#### fn [vector](#method.vector){.fn}(&self) -\> [Vec](https://doc.rust-lang.org/1.91.1/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<[&Point](https://doc.rust-lang.org/1.91.1/std/primitive.reference.html){.primitive}\> {#fn-vectorself---vecpoint .code-header}
:::

::: docblock
Returns a `Vec` containing references to the points that constitute the
geometric object. This method simply converts the `BTreeSet` from
`get_points` into a `Vec`.
:::

::: {#method.to_vector .section .method}
[Source](../../src/optionstratlib/geometrics/utils.rs.html#50-52){.src
.rightside}

#### fn [to_vector](#method.to_vector){.fn}(&self) -\> [Vec](https://doc.rust-lang.org/1.91.1/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<[&Point](https://doc.rust-lang.org/1.91.1/std/primitive.reference.html){.primitive}\> {#fn-to_vectorself---vecpoint .code-header}
:::

::: docblock
Returns the points of the geometric object as a `Vec` of references.
Equivalent to calling the `vector()` method.
:::

:::: {#method.calculate_range .section .method}
[Source](../../src/optionstratlib/geometrics/utils.rs.html#57-64){.src
.rightside}

#### fn [calculate_range](#method.calculate_range){.fn}\<I\>(iter: I) -\> ([Decimal](../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}, [Decimal](../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}) {#fn-calculate_rangeiiter-i---decimal-decimal .code-header}

::: where
where I:
[Iterator](https://doc.rust-lang.org/1.91.1/core/iter/traits/iterator/trait.Iterator.html "trait core::iter::traits::iterator::Iterator"){.trait}\<Item
=
[Decimal](../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}\>,
:::
::::

::: docblock
Calculates the minimum and maximum decimal values from an iterator of
decimals.

This is a utility function that can be used to find the range of values
in a set of points.
:::
::::::::::

## Dyn Compatibility[§](#dyn-compatibility){.anchor} {#dyn-compatibility .section-header}

::: dyn-compatibility-info
This trait is **not** [dyn
compatible](https://doc.rust-lang.org/1.91.1/reference/items/traits.html#dyn-compatibility).

*In older versions of Rust, dyn compatibility was called \"object
safety\", so this trait is not object safe.*
:::

## Implementors[§](#implementors){.anchor} {#implementors .section-header}

::::::::::: {#implementors-list}
::: {#impl-GeometricObject%3CPoint2D,+Decimal%3E-for-Curve .section .impl}
[Source](../../src/optionstratlib/curves/curve.rs.html#148-207){.src
.rightside}[§](#impl-GeometricObject%3CPoint2D,+Decimal%3E-for-Curve){.anchor}

### impl [GeometricObject](trait.GeometricObject.html "trait optionstratlib::geometrics::GeometricObject"){.trait}\<[Point2D](../curves/struct.Point2D.html "struct optionstratlib::curves::Point2D"){.struct}, [Decimal](../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}\> for [Curve](../curves/struct.Curve.html "struct optionstratlib::curves::Curve"){.struct} {#impl-geometricobjectpoint2d-decimal-for-curve .code-header}
:::

:::: impl-items
::: {#associatedtype.Error-1 .section .associatedtype .trait-impl}
[Source](../../src/optionstratlib/curves/curve.rs.html#149){.src
.rightside}[§](#associatedtype.Error-1){.anchor}

#### type [Error](#associatedtype.Error){.associatedtype} = [CurveError](../error/curves/enum.CurveError.html "enum optionstratlib::error::curves::CurveError"){.enum} {#type-error-curveerror .code-header}
:::
::::

:::: {#impl-GeometricObject%3CPoint3D,+Point2D%3E-for-Surface .section .impl}
[Source](../../src/optionstratlib/surfaces/surface.rs.html#335-535){.src
.rightside}[§](#impl-GeometricObject%3CPoint3D,+Point2D%3E-for-Surface){.anchor}

### impl [GeometricObject](trait.GeometricObject.html "trait optionstratlib::geometrics::GeometricObject"){.trait}\<[Point3D](../surfaces/struct.Point3D.html "struct optionstratlib::surfaces::Point3D"){.struct}, [Point2D](../curves/struct.Point2D.html "struct optionstratlib::curves::Point2D"){.struct}\> for [Surface](../surfaces/struct.Surface.html "struct optionstratlib::surfaces::Surface"){.struct} {#impl-geometricobjectpoint3d-point2d-for-surface .code-header}

::: docblock
Implementation of the `GeometricObject` trait for the `Surface` struct.
:::
::::

::: docblock
This implementation provides functionality to create and manipulate 3D
surfaces using points in three-dimensional space. It supports
construction from explicit point collections or through parametric
functions.

#### [§](#type-parameters){.doc-anchor}Type Parameters

- Uses `Point3D` as the points that form the surface
- Uses `Point2D` as the parametric input for surface generation

#### [§](#methods){.doc-anchor}Methods

- `get_points()`: Retrieves all points in the surface
- `from_vector()`: Constructs a surface from a vector of points
- `construct()`: Creates a surface using different construction methods

#### [§](#error-handling){.doc-anchor}Error Handling

Uses `SurfaceError` for various error conditions, including:

- Empty point collections
- Invalid construction parameters
- Errors during parametric function evaluation
:::

:::: impl-items
::: {#associatedtype.Error-2 .section .associatedtype .trait-impl}
[Source](../../src/optionstratlib/surfaces/surface.rs.html#336){.src
.rightside}[§](#associatedtype.Error-2){.anchor}

#### type [Error](#associatedtype.Error){.associatedtype} = [SurfaceError](../error/enum.SurfaceError.html "enum optionstratlib::error::SurfaceError"){.enum} {#type-error-surfaceerror .code-header}
:::
::::
:::::::::::
::::::::::::::::::::::::::::::::::::
:::::::::::::::::::::::::::::::::::::
