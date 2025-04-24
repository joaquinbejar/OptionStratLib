::::::::::::::::::::: width-limiter
:::::::::::::::::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../index.html)::[utils](index.html)
:::

# Trait [Len]{.trait}Copy item path

[[Source](../../src/optionstratlib/utils/traits.rs.html#11-27){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub trait Len {
    // Required method
    fn len(&self) -> usize;

    // Provided method
    fn is_empty(&self) -> bool { ... }
}
```

Expand description

::: docblock
A trait for types that have a notion of length or size.

This trait provides a standard interface for determining the number of
elements in a collection or the size of an object. It defines both a
required `len()` method and a default implementation of `is_empty()`
which relies on `len()`.

Types implementing this trait can be checked for emptiness using the
`is_empty()` method without requiring a separate implementation, as long
as they provide a way to determine their length.
:::

## Required Methods[§](#required-methods){.anchor} {#required-methods .section-header}

::::: methods
::: {#tymethod.len .section .method}
[Source](../../src/optionstratlib/utils/traits.rs.html#17){.src
.rightside}

#### fn [len](#tymethod.len){.fn}(&self) -\> [usize](https://doc.rust-lang.org/1.86.0/std/primitive.usize.html){.primitive} {#fn-lenself---usize .code-header}
:::

::: docblock
Returns the number of elements in the collection or the size of the
object.

##### [§](#returns){.doc-anchor}Returns

A `usize` representing the length or size.
:::
:::::

## Provided Methods[§](#provided-methods){.anchor} {#provided-methods .section-header}

::::: methods
::: {#method.is_empty .section .method}
[Source](../../src/optionstratlib/utils/traits.rs.html#24-26){.src
.rightside}

#### fn [is_empty](#method.is_empty){.fn}(&self) -\> [bool](https://doc.rust-lang.org/1.86.0/std/primitive.bool.html){.primitive} {#fn-is_emptyself---bool .code-header}
:::

::: docblock
Returns `true` if the collection contains no elements or the object has
zero size.

##### [§](#returns-1){.doc-anchor}Returns

A boolean indicating whether the object is empty.
:::
:::::

## Implementors[§](#implementors){.anchor} {#implementors .section-header}

:::::::::: {#implementors-list}
::: {#impl-Len-for-OptionChain .section .impl}
[Source](../../src/optionstratlib/chains/chain.rs.html#2049-2057){.src
.rightside}[§](#impl-Len-for-OptionChain){.anchor}

### impl [Len](trait.Len.html "trait optionstratlib::utils::Len"){.trait} for [OptionChain](../chains/chain/struct.OptionChain.html "struct optionstratlib::chains::chain::OptionChain"){.struct} {#impl-len-for-optionchain .code-header}
:::

::: {#impl-Len-for-Curve .section .impl}
[Source](../../src/optionstratlib/curves/curve.rs.html#124-132){.src
.rightside}[§](#impl-Len-for-Curve){.anchor}

### impl [Len](trait.Len.html "trait optionstratlib::utils::Len"){.trait} for [Curve](../curves/struct.Curve.html "struct optionstratlib::curves::Curve"){.struct} {#impl-len-for-curve .code-header}
:::

::: {#impl-Len-for-Surface .section .impl}
[Source](../../src/optionstratlib/surfaces/surface.rs.html#936-944){.src
.rightside}[§](#impl-Len-for-Surface){.anchor}

### impl [Len](trait.Len.html "trait optionstratlib::utils::Len"){.trait} for [Surface](../surfaces/struct.Surface.html "struct optionstratlib::surfaces::Surface"){.struct} {#impl-len-for-surface .code-header}
:::

::::: {#impl-Len-for-RandomWalk%3CX,+Y%3E .section .impl}
[Source](../../src/optionstratlib/simulation/randomwalk.rs.html#170-192){.src
.rightside}[§](#impl-Len-for-RandomWalk%3CX,+Y%3E){.anchor}

### impl\<X, Y\> [Len](trait.Len.html "trait optionstratlib::utils::Len"){.trait} for [RandomWalk](../simulation/randomwalk/struct.RandomWalk.html "struct optionstratlib::simulation::randomwalk::RandomWalk"){.struct}\<X, Y\> {#implx-y-len-for-randomwalkx-y .code-header}

::: where
where X:
[Copy](https://doc.rust-lang.org/1.86.0/core/marker/trait.Copy.html "trait core::marker::Copy"){.trait} +
[Into](https://doc.rust-lang.org/1.86.0/core/convert/trait.Into.html "trait core::convert::Into"){.trait}\<[Positive](../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\> +
[AddAssign](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.AddAssign.html "trait core::ops::arith::AddAssign"){.trait} +
[Display](https://doc.rust-lang.org/1.86.0/core/fmt/trait.Display.html "trait core::fmt::Display"){.trait},
Y:
[Into](https://doc.rust-lang.org/1.86.0/core/convert/trait.Into.html "trait core::convert::Into"){.trait}\<[Positive](../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\> +
[Display](https://doc.rust-lang.org/1.86.0/core/fmt/trait.Display.html "trait core::fmt::Display"){.trait} +
[Clone](https://doc.rust-lang.org/1.86.0/core/clone/trait.Clone.html "trait core::clone::Clone"){.trait},
:::

::: docblock
Implementation of the `Len` trait for `RandomWalk<X, Y>`.
:::
:::::

::: docblock
This implementation provides methods to determine the length and
emptiness of a random walk by delegating to the underlying `steps`
collection.

#### [§](#type-parameters){.doc-anchor}Type Parameters

- `X` - The type for x-axis values (typically time or sequence
  position), which must implement `AddAssign`, be convertible to
  `Positive`, and be `Copy`.

- `Y` - The type for y-axis values (typically price or value), which
  must implement `AddAssign`, be convertible to `Positive`, be `Copy`,
  and implement the `Walktypable` trait.
:::
::::::::::
::::::::::::::::::::
:::::::::::::::::::::
