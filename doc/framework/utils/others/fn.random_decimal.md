::::::: width-limiter
:::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../../index.html)::[utils](../index.html)::[others](index.html)
:::

# Function [random_decimal]{.fn}Copy item path

[[Source](../../../src/optionstratlib/utils/others.rs.html#96-105){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub fn random_decimal(rng: &mut impl Rng) -> Result<Decimal, DecimalError>
```

Expand description

::: docblock
Generates a random `Decimal` value using the provided random number
generator.

This function takes a mutable reference to a random number generator
(`rand::Rng`) and uses it to generate a random `f64` value, which is
then converted to a `Decimal`.

## [§](#arguments){.doc-anchor}Arguments

- `rng` - A mutable reference to a random number generator. This allows
  the function to generate different random numbers on each call.

## [§](#returns){.doc-anchor}Returns

A `Result` containing either the generated `Decimal` or a `DecimalError`
if the conversion from `f64` to `Decimal` fails.

## [§](#errors){.doc-anchor}Errors

Returns a `DecimalError::ConversionError` if the `f64` value generated
by the random number generator cannot be converted to a `Decimal`. This
can occur if the `f64` value is NaN or infinite.
:::
::::::
:::::::
