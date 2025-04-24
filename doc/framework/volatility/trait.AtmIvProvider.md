::::::::::::: width-limiter
:::::::::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../index.html)::[volatility](index.html)
:::

# Trait [AtmIvProvider]{.trait}Copy item path

[[Source](../../src/optionstratlib/volatility/traits.rs.html#89-100){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub trait AtmIvProvider {
    // Required method
    fn atm_iv(&self) -> Result<&Option<Positive>, Box<dyn Error>>;
}
```

Expand description

::: docblock
Trait for providing at-the-money implied volatility.

This trait defines a method to retrieve the at-the-money (ATM) implied
volatility. Implementations should return a `Positive` value
representing the ATM IV, or an error if the value cannot be determined.
:::

## Required Methods[§](#required-methods){.anchor} {#required-methods .section-header}

::::: methods
::: {#tymethod.atm_iv .section .method}
[Source](../../src/optionstratlib/volatility/traits.rs.html#99){.src
.rightside}

#### fn [atm_iv](#tymethod.atm_iv){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<&[Option](https://doc.rust-lang.org/1.86.0/core/option/enum.Option.html "enum core::option::Option"){.enum}\<[Positive](../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\>, [Box](https://doc.rust-lang.org/1.86.0/alloc/boxed/struct.Box.html "struct alloc::boxed::Box"){.struct}\<dyn [Error](https://doc.rust-lang.org/1.86.0/core/error/trait.Error.html "trait core::error::Error"){.trait}\>\> {#fn-atm_ivself---resultoptionpositive-boxdyn-error .code-header}
:::

::: docblock
Get the at-the-money implied volatility

This method attempts to return the at-the-money implied volatility as an
`Option<Positive>`.

##### [§](#returns){.doc-anchor}Returns

- `Ok(Some(Positive))` - If the ATM implied volatility is successfully
  retrieved.
- `Ok(None)` - If the ATM implied volatility is not available or not
  applicable.
- `Err(Box<dyn Error>)` - If an error occurs during the retrieval
  process.
:::
:::::

## Implementors[§](#implementors){.anchor} {#implementors .section-header}

::::: {#implementors-list}
::: {#impl-AtmIvProvider-for-OptionChain .section .impl}
[Source](../../src/optionstratlib/volatility/traits.rs.html#108-118){.src
.rightside}[§](#impl-AtmIvProvider-for-OptionChain){.anchor}

### impl [AtmIvProvider](trait.AtmIvProvider.html "trait optionstratlib::volatility::AtmIvProvider"){.trait} for [OptionChain](../chains/chain/struct.OptionChain.html "struct optionstratlib::chains::chain::OptionChain"){.struct} {#impl-atmivprovider-for-optionchain .code-header}
:::

::: {#impl-AtmIvProvider-for-Positive .section .impl}
[Source](../../src/optionstratlib/volatility/traits.rs.html#102-106){.src
.rightside}[§](#impl-AtmIvProvider-for-Positive){.anchor}

### impl [AtmIvProvider](trait.AtmIvProvider.html "trait optionstratlib::volatility::AtmIvProvider"){.trait} for [Positive](../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#impl-atmivprovider-for-positive .code-header}
:::
:::::
::::::::::::
:::::::::::::
