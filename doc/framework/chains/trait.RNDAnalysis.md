:::::::::::::: width-limiter
::::::::::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../index.html)::[chains](index.html)
:::

# Trait [RNDAnalysis]{.trait}Copy item path

[[Source](../../src/optionstratlib/chains/rnd.rs.html#389-410){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub trait RNDAnalysis {
    // Required methods
    fn calculate_rnd(
        &self,
        params: &RNDParameters,
    ) -> Result<RNDResult, Box<dyn Error>>;
    fn calculate_skew(&self) -> Result<Vec<(Positive, Decimal)>, Box<dyn Error>>;
}
```

Expand description

::: docblock
Trait defining Risk-Neutral Density analysis capabilities

This trait provides methods for calculating RND and analyzing volatility
skew from option chain data.
:::

## Required Methods[§](#required-methods){.anchor} {#required-methods .section-header}

::::::: methods
::: {#tymethod.calculate_rnd .section .method}
[Source](../../src/optionstratlib/chains/rnd.rs.html#400){.src
.rightside}

#### fn [calculate_rnd](#tymethod.calculate_rnd){.fn}( &self, params: &[RNDParameters](struct.RNDParameters.html "struct optionstratlib::chains::RNDParameters"){.struct}, ) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[RNDResult](struct.RNDResult.html "struct optionstratlib::chains::RNDResult"){.struct}, [Box](https://doc.rust-lang.org/1.86.0/alloc/boxed/struct.Box.html "struct alloc::boxed::Box"){.struct}\<dyn [Error](https://doc.rust-lang.org/1.86.0/core/error/trait.Error.html "trait core::error::Error"){.trait}\>\> {#fn-calculate_rnd-self-params-rndparameters---resultrndresult-boxdyn-error .code-header}
:::

::: docblock
Calculates the Risk-Neutral Density from the option chain

Uses the Breeden-Litzenberger formula to extract implied probabilities
from option prices.

##### [§](#arguments){.doc-anchor}Arguments

- `params` - Parameters controlling the RND calculation

##### [§](#returns){.doc-anchor}Returns

Result containing either RNDResult or an error
:::

::: {#tymethod.calculate_skew .section .method}
[Source](../../src/optionstratlib/chains/rnd.rs.html#409){.src
.rightside}

#### fn [calculate_skew](#tymethod.calculate_skew){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Vec](https://doc.rust-lang.org/1.86.0/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<([Positive](../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, Decimal)\>, [Box](https://doc.rust-lang.org/1.86.0/alloc/boxed/struct.Box.html "struct alloc::boxed::Box"){.struct}\<dyn [Error](https://doc.rust-lang.org/1.86.0/core/error/trait.Error.html "trait core::error::Error"){.trait}\>\> {#fn-calculate_skewself---resultvecpositive-decimal-boxdyn-error .code-header}
:::

::: docblock
Calculates the implied volatility skew

Analyzes how implied volatility varies across different strike prices,
providing insight into market's price expectations.

##### [§](#returns-1){.doc-anchor}Returns

Result containing vector of (strike_price, volatility) pairs or an error
:::
:::::::

## Implementors[§](#implementors){.anchor} {#implementors .section-header}

:::: {#implementors-list}
::: {#impl-RNDAnalysis-for-OptionChain .section .impl}
[Source](../../src/optionstratlib/chains/chain.rs.html#2080-2231){.src
.rightside}[§](#impl-RNDAnalysis-for-OptionChain){.anchor}

### impl [RNDAnalysis](trait.RNDAnalysis.html "trait optionstratlib::chains::RNDAnalysis"){.trait} for [OptionChain](chain/struct.OptionChain.html "struct optionstratlib::chains::chain::OptionChain"){.struct} {#impl-rndanalysis-for-optionchain .code-header}
:::
::::
:::::::::::::
::::::::::::::
