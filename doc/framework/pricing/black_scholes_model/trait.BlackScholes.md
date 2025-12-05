::::::::::::::: width-limiter
:::::::::::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../../index.html)::[pricing](../index.html)::[black_scholes_model](index.html)
:::

# Trait [BlackScholes]{.trait} Copy item path

[[Source](../../../src/optionstratlib/pricing/black_scholes_model.rs.html#247-272){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub trait BlackScholes {
    // Required method
    fn get_option(&self) -> Result<&Options, PricingError>;

    // Provided method
    fn calculate_price_black_scholes(&self) -> Result<Decimal, PricingError> { ... }
}
```

Expand description

:::: docblock
A trait for financial instruments that can be priced using the
Black-Scholes option pricing model.

This trait defines the interface for financial instruments that can have
their price calculated using the Black-Scholes formula. Implementors
must provide access to their underlying option data through the
`get_option` method, which allows the default implementation of
`calculate_price_black_scholes` to perform the calculation.

## [§](#examples){.doc-anchor}Examples

::: example-wrap
``` {.rust .rust-example-rendered}
use std::error::Error;
use optionstratlib::Options;
use optionstratlib::prelude::PricingError;

use optionstratlib::pricing::BlackScholes;

struct MyOption {
    option: Options
}

impl BlackScholes for MyOption {
    fn get_option(&self) -> Result<&Options, PricingError> {
        Ok(&self.option)
    }
}
```
:::
::::

## Required Methods[§](#required-methods){.anchor} {#required-methods .section-header}

::::: methods
::: {#tymethod.get_option .section .method}
[Source](../../../src/optionstratlib/pricing/black_scholes_model.rs.html#257){.src
.rightside}

#### fn [get_option](#tymethod.get_option){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<&[Options](../../model/option/struct.Options.html "struct optionstratlib::model::option::Options"){.struct}, [PricingError](../../error/pricing/enum.PricingError.html "enum optionstratlib::error::pricing::PricingError"){.enum}\> {#fn-get_optionself---resultoptions-pricingerror .code-header}
:::

::: docblock
Retrieves a reference to the options data required for Black-Scholes
calculations.

This method must be implemented by types that implement this trait. It
provides access to the option parameters needed for pricing
calculations.

##### [§](#returns){.doc-anchor}Returns

- `Result<&Options, PricingError>` - A reference to the Options struct
  on success, or an error if the option data cannot be retrieved.
:::
:::::

## Provided Methods[§](#provided-methods){.anchor} {#provided-methods .section-header}

::::: methods
::: {#method.calculate_price_black_scholes .section .method}
[Source](../../../src/optionstratlib/pricing/black_scholes_model.rs.html#268-271){.src
.rightside}

#### fn [calculate_price_black_scholes](#method.calculate_price_black_scholes){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Decimal](../../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}, [PricingError](../../error/pricing/enum.PricingError.html "enum optionstratlib::error::pricing::PricingError"){.enum}\> {#fn-calculate_price_black_scholesself---resultdecimal-pricingerror .code-header}
:::

::: docblock
Calculates the price of the option using the Black-Scholes model.

This default implementation retrieves the option data via `get_option()`
and then passes it to the `black_scholes` function to perform the
calculation.

##### [§](#returns-1){.doc-anchor}Returns

- `Result<Decimal, PricingError>` - The calculated option price as a
  Decimal on success, or an error if the calculation fails.
:::
:::::

## Implementors[§](#implementors){.anchor} {#implementors .section-header}

::: {#implementors-list}
:::
::::::::::::::
:::::::::::::::
