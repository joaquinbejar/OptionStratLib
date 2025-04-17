::::::: width-limiter
:::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../../index.html)::[utils](../index.html)::[others](index.html)
:::

# Function [get_random_element]{.fn}Copy item path

[[Source](../../../src/optionstratlib/utils/others.rs.html#66-73){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub fn get_random_element<T>(set: &BTreeSet<T>) -> Option<&T>
```

Expand description

::: docblock
Gets a random element from a BTreeSet.

This function returns a random element from the provided BTreeSet using
a uniform distribution. If the set is empty, it returns None.

## [§](#type-parameters){.doc-anchor}Type Parameters

- `T` - The type of elements in the BTreeSet

## [§](#arguments){.doc-anchor}Arguments

- `set` - A reference to a BTreeSet containing elements of type T

## [§](#returns){.doc-anchor}Returns

- `Option<&T>` - A reference to a random element from the set, or None
  if the set is empty
:::
::::::
:::::::
