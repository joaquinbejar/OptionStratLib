:::::::::::: width-limiter
::::::::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../../index.html)::[chains](../index.html)::[utils](index.html)
:::

# Trait [OptionChainParams]{.trait}Copy item path

[[Source](../../../src/optionstratlib/chains/utils.rs.html#482-509){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub trait OptionChainParams {
    // Required method
    fn get_params(
        &self,
        strike_price: Positive,
    ) -> Result<OptionDataPriceParams, ChainError>;
}
```

Expand description

::: docblock
A trait for obtaining option pricing parameters based on a strike price.

This trait defines an interface for types that can provide the necessary
parameters for pricing options at a specific strike price.
Implementations of this trait handle the logic of determining
appropriate pricing parameters such as underlying price, expiration
date, implied volatility, risk-free rate, dividend yield, and other
relevant values required for option pricing models.

## [§](#type-parameters){.doc-anchor}Type Parameters

The trait is generic over the implementing type, allowing various
sources of option parameters to conform to a single interface.

## [§](#methods){.doc-anchor}Methods

- `get_params` - Retrieves the option pricing parameters for a given
  strike price.

## [§](#errors){.doc-anchor}Errors

Returns a `ChainError` if the parameters cannot be determined or are
invalid for the specified strike price.

## [§](#usage){.doc-anchor}Usage

This trait is typically implemented by types that represent sources of
option chain data, such as market data providers, model-based
generators, or historical data repositories. It provides a uniform way
to access option pricing parameters regardless of their source.
:::

## Required Methods[§](#required-methods){.anchor} {#required-methods .section-header}

::::: methods
::: {#tymethod.get_params .section .method}
[Source](../../../src/optionstratlib/chains/utils.rs.html#508){.src
.rightside}

#### fn [get_params](#tymethod.get_params){.fn}( &self, strike_price: [Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, ) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[OptionDataPriceParams](struct.OptionDataPriceParams.html "struct optionstratlib::chains::utils::OptionDataPriceParams"){.struct}, [ChainError](../../error/chains/enum.ChainError.html "enum optionstratlib::error::chains::ChainError"){.enum}\> {#fn-get_params-self-strike_price-positive---resultoptiondatapriceparams-chainerror .code-header}
:::

::: docblock
Retrieves the option pricing parameters for a given strike price.

This method calculates or retrieves all parameters necessary for pricing
an option at the specified strike price, including the underlying price,
expiration date, implied volatility (if available), risk-free rate,
dividend yield, and underlying symbol.

##### [§](#parameters){.doc-anchor}Parameters

- `strike_price` - A positive decimal value representing the strike
  price of the option for which parameters are being requested.

##### [§](#returns){.doc-anchor}Returns

- `Ok(OptionDataPriceParams)` - A structure containing all necessary
  parameters for option pricing calculations if the parameters could be
  successfully determined.
- `Err(ChainError)` - An error if the parameters cannot be determined or
  are invalid for the given strike price.

##### [§](#errors-1){.doc-anchor}Errors

This method may return various `ChainError` variants depending on the
implementation, such as:

- `ChainError::OptionDataError` for invalid option data
- `ChainError::ChainBuildError` for problems constructing chain
  parameters
- Other error types as appropriate for the specific implementation
:::
:::::

## Implementors[§](#implementors){.anchor} {#implementors .section-header}

:::: {#implementors-list}
::: {#impl-OptionChainParams-for-OptionChain .section .impl}
[Source](../../../src/optionstratlib/chains/chain.rs.html#2059-2078){.src
.rightside}[§](#impl-OptionChainParams-for-OptionChain){.anchor}

### impl [OptionChainParams](trait.OptionChainParams.html "trait optionstratlib::chains::utils::OptionChainParams"){.trait} for [OptionChain](../chain/struct.OptionChain.html "struct optionstratlib::chains::chain::OptionChain"){.struct} {#impl-optionchainparams-for-optionchain .code-header}
:::
::::
:::::::::::
::::::::::::
