:::::::: width-limiter
::::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../../index.html)::[model](../index.html)::[utils](index.html)
:::

# Function [mean_and_std]{.fn}Copy item path

[[Source](../../../src/optionstratlib/model/utils.rs.html#343-352){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub fn mean_and_std(vec: Vec<Positive>) -> (Positive, Positive)
```

Expand description

:::: docblock
Computes the mean and standard deviation of a vector containing
`Positive` values.

## [§](#arguments){.doc-anchor}Arguments

- `vec` - A `Vec<Positive>` containing the data for which the mean and
  standard deviation are to be calculated.

## [§](#returns){.doc-anchor}Returns

A tuple containing:

- `Positive` - The mean of the provided vector.
- `Positive` - The standard deviation of the provided vector.

## [§](#example){.doc-anchor}Example

::: example-wrap
``` {.rust .rust-example-rendered}
use optionstratlib::Positive;
use optionstratlib::model::utils::mean_and_std;

let data = vec![Positive::new(2.0).unwrap(), Positive::new(4.0).unwrap(), Positive::new(4.0).unwrap(), Positive::new(4.0).unwrap(), Positive::new(5.0).unwrap(), Positive::new(5.0).unwrap(), Positive::new(7.0).unwrap(), Positive::new(9.0).unwrap()];
let (mean, std) = mean_and_std(data);

assert_eq!(mean.to_f64(), 5.0);
assert_eq!(std.to_f64(), 4.0_f64.sqrt());
```
:::

## [§](#details){.doc-anchor}Details

- The mean is computed by summing the `Positive` values and dividing by
  the count of elements.
- The standard deviation is derived from the variance, which is the
  average of the squared differences from the mean. The variance is then
  converted into standard deviation by taking its square root.
- This function assumes the vector is non-empty and filled with valid
  `Positive` values.

Note: The `Positive` type and associated operations are defined in the
`crate::model::types` module.
::::
:::::::
::::::::
