::::::: width-limiter
:::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../../index.html)::[pricing](../index.html)
:::

# Module binomial_modelCopy item path

[[Source](../../../src/optionstratlib/pricing/binomial_model.rs.html#1-655){.src}
]{.sub-heading}
::::

Expand description

::: docblock
Binomial tree model implementation for option pricing.

This module provides functionality to price options using binomial tree
methods, which discretize time and price movements to create a lattice
of possible future asset prices.

The binomial model is particularly useful for pricing American options
and other derivatives with early exercise features.
:::

## Structs[§](#structs){.anchor} {#structs .section-header}

[BinomialPricingParams](struct.BinomialPricingParams.html "struct optionstratlib::pricing::binomial_model::BinomialPricingParams"){.struct}
:   Parameters for pricing options using the Binomial Tree model.

## Functions[§](#functions){.anchor} {#functions .section-header}

[generate_binomial_tree](fn.generate_binomial_tree.html "fn optionstratlib::pricing::binomial_model::generate_binomial_tree"){.fn}
:   Generates a binomial tree for option pricing.

[price_binomial](fn.price_binomial.html "fn optionstratlib::pricing::binomial_model::price_binomial"){.fn}
:   Calculates the price of an option using the binomial model.
::::::
:::::::
