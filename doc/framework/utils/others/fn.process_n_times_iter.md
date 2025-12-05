:::::::: width-limiter
::::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../../index.html)::[utils](../index.html)::[others](index.html)
:::

# Function [process_n_times_iter]{.fn} Copy item path

[[Source](../../../src/optionstratlib/utils/others.rs.html#144-168){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub fn process_n_times_iter<T, Y, F>(
    positions: &[T],
    n: usize,
    process_combination: F,
) -> Result<Vec<Y>, Error>where
    F: FnMut(&[&T]) -> Vec<Y> + Send + Sync,
    T: Clone + Send + Sync,
    Y: Send,
```

Expand description

:::: docblock
Processes combinations of elements from a slice in parallel.

This function takes a slice of elements, a combination size `n`, and a
closure `process_combination`. It generates all combinations with
replacement of size `n` from the input slice and processes each
combination using the provided closure. The results from each
combination processing are collected into a single vector.

The processing is done in parallel using Rayon's parallel iterators for
improved performance.

## [§](#arguments){.doc-anchor}Arguments

- `positions` - A slice of elements to generate combinations from.
- `n` - The size of the combinations to generate.
- `process_combination` - A closure that takes a slice of references to
  elements from `positions` and returns a vector of results. This
  closure should implement `Send + Sync` since it's used in a
  multithreaded environment.

## [§](#returns){.doc-anchor}Returns

- `Result<Vec<Y>, Error>` - A `Result` containing a vector of the
  combined results from the closure or an error if the input slice is
  empty.

## [§](#errors){.doc-anchor}Errors

Returns an error if the input `positions` slice is empty.

## [§](#examples){.doc-anchor}Examples

::: example-wrap
``` {.rust .rust-example-rendered}
use optionstratlib::utils::others::process_n_times_iter;

let numbers = vec![1, 2, 3];
let n = 2;
let result = process_n_times_iter(&numbers, n, |combination| {
    vec![combination[0] + combination[1]]
}).unwrap();

assert_eq!(result, vec![2, 3, 4, 4, 5, 6]);
```
:::
::::
:::::::
::::::::
