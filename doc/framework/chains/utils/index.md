::::::: width-limiter
:::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../../index.html)::[chains](../index.html)
:::

# Module utils Copy item path

[[Source](../../../src/optionstratlib/chains/utils.rs.html#6-1483){.src}
]{.sub-heading}
::::

Expand description

::: docblock
- `utils` - Public module containing utility functions and helpers for
  financial calculations
:::

## Structs[§](#structs){.anchor} {#structs .section-header}

[OptionChainBuildParams](struct.OptionChainBuildParams.html "struct optionstratlib::chains::utils::OptionChainBuildParams"){.struct}
:   Parameters for building an option chain dataset.

[OptionDataPriceParams](struct.OptionDataPriceParams.html "struct optionstratlib::chains::utils::OptionDataPriceParams"){.struct}
:   Parameters required for pricing an option contract.

[RandomPositionsParams](struct.RandomPositionsParams.html "struct optionstratlib::chains::utils::RandomPositionsParams"){.struct}
:   Parameters for generating random positions in an option chain

## Enums[§](#enums){.anchor} {#enums .section-header}

[OptionDataGroup](enum.OptionDataGroup.html "enum optionstratlib::chains::utils::OptionDataGroup"){.enum}
:   Enum representing a grouping of option data references for analysis
    or display purposes.

## Traits[§](#traits){.anchor} {#traits .section-header}

[OptionChainParams](trait.OptionChainParams.html "trait optionstratlib::chains::utils::OptionChainParams"){.trait}
:   A trait for obtaining option pricing parameters based on a strike
    price.

## Functions[§](#functions){.anchor} {#functions .section-header}

[adjust_volatility](fn.adjust_volatility.html "fn optionstratlib::chains::utils::adjust_volatility"){.fn}
:   Adjust vol with skew/smile, using *relative* distance to ATM.

[strike_step](fn.strike_step.html "fn optionstratlib::chains::utils::strike_step"){.fn}
:   Return the strike interval that gives \~`size` strikes around ATM.
    All units are in the same currency.
::::::
:::::::
