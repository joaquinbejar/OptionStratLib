:::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::: width-limiter
::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../../index.html)::[strategies](../index.html)::[base](index.html)
:::

# Trait [Optimizable]{.trait} Copy item path

[[Source](../../../src/optionstratlib/strategies/base.rs.html#1159-1285){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub trait Optimizable: Validable + Strategies {
    type Strategy: Strategies;

    // Provided methods
    fn get_best_ratio(
        &mut self,
        option_chain: &OptionChain,
        side: FindOptimalSide,
    ) { ... }
    fn get_best_area(
        &mut self,
        option_chain: &OptionChain,
        side: FindOptimalSide,
    ) { ... }
    fn filter_combinations<'a>(
        &'a self,
        _option_chain: &'a OptionChain,
        _side: FindOptimalSide,
    ) -> impl Iterator<Item = OptionDataGroup<'a>> { ... }
    fn find_optimal(
        &mut self,
        _option_chain: &OptionChain,
        _side: FindOptimalSide,
        _criteria: OptimizationCriteria,
    ) { ... }
    fn is_valid_optimal_option(
        &self,
        option: &OptionData,
        side: &FindOptimalSide,
    ) -> bool { ... }
    fn are_valid_legs(&self, legs: &StrategyLegs<'_>) -> bool { ... }
    fn create_strategy(
        &self,
        _chain: &OptionChain,
        _legs: &StrategyLegs<'_>,
    ) -> Self::Strategy { ... }
}
```

Expand description

::: docblock
This trait defines methods for optimizing and validating option
strategies. It combines the `Validable` and `Strategies` traits,
requiring implementors to provide functionality for both validation and
strategy generation.
:::

## Required Associated Types[§](#required-associated-types){.anchor} {#required-associated-types .section-header}

::::: methods
::: {#associatedtype.Strategy .section .method}
[Source](../../../src/optionstratlib/strategies/base.rs.html#1161){.src
.rightside}

#### type [Strategy](#associatedtype.Strategy){.associatedtype}: [Strategies](trait.Strategies.html "trait optionstratlib::strategies::base::Strategies"){.trait} {#type-strategy-strategies .code-header}
:::

::: docblock
The type of strategy associated with this optimization.
:::
:::::

## Provided Methods[§](#provided-methods){.anchor} {#provided-methods .section-header}

::::::::::::::::: methods
::: {#method.get_best_ratio .section .method}
[Source](../../../src/optionstratlib/strategies/base.rs.html#1168-1170){.src
.rightside}

#### fn [get_best_ratio](#method.get_best_ratio){.fn}(&mut self, option_chain: &[OptionChain](../../chains/chain/struct.OptionChain.html "struct optionstratlib::chains::chain::OptionChain"){.struct}, side: [FindOptimalSide](../utils/enum.FindOptimalSide.html "enum optionstratlib::strategies::utils::FindOptimalSide"){.enum}) {#fn-get_best_ratiomut-self-option_chain-optionchain-side-findoptimalside .code-header}
:::

::: docblock
Finds the best ratio-based strategy within the given `OptionChain`.

##### [§](#arguments){.doc-anchor}Arguments

- `option_chain` - A reference to the `OptionChain` containing option
  data.
- `side` - A `FindOptimalSide` value specifying the filtering strategy.
:::

::: {#method.get_best_area .section .method}
[Source](../../../src/optionstratlib/strategies/base.rs.html#1177-1179){.src
.rightside}

#### fn [get_best_area](#method.get_best_area){.fn}(&mut self, option_chain: &[OptionChain](../../chains/chain/struct.OptionChain.html "struct optionstratlib::chains::chain::OptionChain"){.struct}, side: [FindOptimalSide](../utils/enum.FindOptimalSide.html "enum optionstratlib::strategies::utils::FindOptimalSide"){.enum}) {#fn-get_best_areamut-self-option_chain-optionchain-side-findoptimalside .code-header}
:::

::: docblock
Finds the best area-based strategy within the given `OptionChain`.

##### [§](#arguments-1){.doc-anchor}Arguments

- `option_chain` - A reference to the `OptionChain` containing option
  data.
- `side` - A `FindOptimalSide` value specifying the filtering strategy.
:::

::: {#method.filter_combinations .section .method}
[Source](../../../src/optionstratlib/strategies/base.rs.html#1208-1215){.src
.rightside}

#### fn [filter_combinations](#method.filter_combinations){.fn}\<\'a\>( &\'a self, \_option_chain: &\'a [OptionChain](../../chains/chain/struct.OptionChain.html "struct optionstratlib::chains::chain::OptionChain"){.struct}, \_side: [FindOptimalSide](../utils/enum.FindOptimalSide.html "enum optionstratlib::strategies::utils::FindOptimalSide"){.enum}, ) -\> impl [Iterator](https://doc.rust-lang.org/1.91.1/core/iter/traits/iterator/trait.Iterator.html "trait core::iter::traits::iterator::Iterator"){.trait}\<Item = [OptionDataGroup](../../chains/utils/enum.OptionDataGroup.html "enum optionstratlib::chains::utils::OptionDataGroup"){.enum}\<\'a\>\> {#fn-filter_combinationsa-a-self-_option_chain-a-optionchain-_side-findoptimalside---impl-iteratoritem-optiondatagroupa .code-header}
:::

::: docblock
Filters and generates combinations of options data from the given
`OptionChain`.

##### [§](#parameters){.doc-anchor}Parameters

- `&self`: A reference to the current object/context that holds the
  filtering logic or required data.
- `_option_chain`: A reference to an `OptionChain` object that contains
  relevant financial information such as options data, underlying price,
  and expiration date.
- `_side`: A `FindOptimalSide` value that specifies the filtering
  strategy for finding combinations of options. It can specify:
  - `Upper`: Consider options higher than a certain threshold.
  - `Lower`: Consider options lower than a certain threshold.
  - `All`: Include all options.
  - `Range(start, end)`: Consider options within a specified range.

##### [§](#returns){.doc-anchor}Returns

- An iterator that yields `OptionDataGroup` items. These items represent
  combinations of options data filtered based on the given criteria. The
  `OptionDataGroup` can represent combinations of 2, 3, 4, or any number
  of options depending on the grouping logic.

**Note**:

- The current implementation returns an empty iterator
  (`std::iter::empty()`) as a placeholder.
- You may modify this method to implement the actual filtering and
  combination logic based on the provided `OptionChain` and
  `FindOptimalSide` criteria.

##### [§](#see-also){.doc-anchor}See Also

- `FindOptimalSide` for the strategy enumeration.
- `OptionDataGroup` for the structure of grouped combinations.
- `OptionChain` for the full structure being processed.
:::

::: {#method.find_optimal .section .method}
[Source](../../../src/optionstratlib/strategies/base.rs.html#1225-1232){.src
.rightside}

#### fn [find_optimal](#method.find_optimal){.fn}( &mut self, \_option_chain: &[OptionChain](../../chains/chain/struct.OptionChain.html "struct optionstratlib::chains::chain::OptionChain"){.struct}, \_side: [FindOptimalSide](../utils/enum.FindOptimalSide.html "enum optionstratlib::strategies::utils::FindOptimalSide"){.enum}, \_criteria: [OptimizationCriteria](../utils/enum.OptimizationCriteria.html "enum optionstratlib::strategies::utils::OptimizationCriteria"){.enum}, ) {#fn-find_optimal-mut-self-_option_chain-optionchain-_side-findoptimalside-_criteria-optimizationcriteria .code-header}
:::

::: docblock
Finds the optimal strategy based on the given criteria. The default
implementation panics. Specific strategies should override this method
to provide their own optimization logic.

##### [§](#arguments-2){.doc-anchor}Arguments

- `_option_chain` - A reference to the `OptionChain` containing option
  data.
- `_side` - A `FindOptimalSide` value specifying the filtering strategy.
- `_criteria` - An `OptimizationCriteria` value indicating the
  optimization goal (e.g., ratio, area).
:::

::: {#method.is_valid_optimal_option .section .method}
[Source](../../../src/optionstratlib/strategies/base.rs.html#1239-1259){.src
.rightside}

#### fn [is_valid_optimal_option](#method.is_valid_optimal_option){.fn}( &self, option: &[OptionData](../../chains/struct.OptionData.html "struct optionstratlib::chains::OptionData"){.struct}, side: &[FindOptimalSide](../utils/enum.FindOptimalSide.html "enum optionstratlib::strategies::utils::FindOptimalSide"){.enum}, ) -\> [bool](https://doc.rust-lang.org/1.91.1/std/primitive.bool.html){.primitive} {#fn-is_valid_optimal_option-self-option-optiondata-side-findoptimalside---bool .code-header}
:::

::: docblock
Checks if a long option is valid based on the given criteria.

##### [§](#arguments-3){.doc-anchor}Arguments

- `option` - A reference to the `OptionData` to validate.
- `side` - A reference to the `FindOptimalSide` specifying the filtering
  strategy.
:::

::: {#method.are_valid_legs .section .method}
[Source](../../../src/optionstratlib/strategies/base.rs.html#1266-1274){.src
.rightside}

#### fn [are_valid_legs](#method.are_valid_legs){.fn}(&self, legs: &[StrategyLegs](../../chains/enum.StrategyLegs.html "enum optionstratlib::chains::StrategyLegs"){.enum}\<\'\_\>) -\> [bool](https://doc.rust-lang.org/1.91.1/std/primitive.bool.html){.primitive} {#fn-are_valid_legsself-legs-strategylegs_---bool .code-header}
:::

::: docblock
Checks if the prices in the given `StrategyLegs` are valid. Assumes the
strategy consists of one long call and one short call by default.

##### [§](#arguments-4){.doc-anchor}Arguments

- `legs` - A reference to the `StrategyLegs` containing the option data.
:::

::: {#method.create_strategy .section .method}
[Source](../../../src/optionstratlib/strategies/base.rs.html#1282-1284){.src
.rightside}

#### fn [create_strategy](#method.create_strategy){.fn}( &self, \_chain: &[OptionChain](../../chains/chain/struct.OptionChain.html "struct optionstratlib::chains::chain::OptionChain"){.struct}, \_legs: &[StrategyLegs](../../chains/enum.StrategyLegs.html "enum optionstratlib::chains::StrategyLegs"){.enum}\<\'\_\>, ) -\> Self::[Strategy](trait.Optimizable.html#associatedtype.Strategy "type optionstratlib::strategies::base::Optimizable::Strategy"){.associatedtype} {#fn-create_strategy-self-_chain-optionchain-_legs-strategylegs_---selfstrategy .code-header}
:::

::: docblock
Creates a new strategy from the given `OptionChain` and `StrategyLegs`.
The default implementation panics. Specific strategies must override
this.

##### [§](#arguments-5){.doc-anchor}Arguments

- `_chain` - A reference to the `OptionChain` providing option data.
- `_legs` - A reference to the `StrategyLegs` defining the strategy's
  components.
:::
:::::::::::::::::

## Dyn Compatibility[§](#dyn-compatibility){.anchor} {#dyn-compatibility .section-header}

::: dyn-compatibility-info
This trait is **not** [dyn
compatible](https://doc.rust-lang.org/1.91.1/reference/items/traits.html#dyn-compatibility).

*In older versions of Rust, dyn compatibility was called \"object
safety\", so this trait is not object safe.*
:::

## Implementors[§](#implementors){.anchor} {#implementors .section-header}

:::::::::::::::::::::::::::::::::::::::::::::::::::::::::::: {#implementors-list}
::: {#impl-Optimizable-for-BearCallSpread .section .impl}
[Source](../../../src/optionstratlib/strategies/bear_call_spread.rs.html#643-747){.src
.rightside}[§](#impl-Optimizable-for-BearCallSpread){.anchor}

### impl [Optimizable](trait.Optimizable.html "trait optionstratlib::strategies::base::Optimizable"){.trait} for [BearCallSpread](../bear_call_spread/struct.BearCallSpread.html "struct optionstratlib::strategies::bear_call_spread::BearCallSpread"){.struct} {#impl-optimizable-for-bearcallspread .code-header}
:::

:::: impl-items
::: {#associatedtype.Strategy-1 .section .associatedtype .trait-impl}
[Source](../../../src/optionstratlib/strategies/bear_call_spread.rs.html#644){.src
.rightside}[§](#associatedtype.Strategy-1){.anchor}

#### type [Strategy](#associatedtype.Strategy){.associatedtype} = [BearCallSpread](../bear_call_spread/struct.BearCallSpread.html "struct optionstratlib::strategies::bear_call_spread::BearCallSpread"){.struct} {#type-strategy-bearcallspread .code-header}
:::
::::

::: {#impl-Optimizable-for-BearPutSpread .section .impl}
[Source](../../../src/optionstratlib/strategies/bear_put_spread.rs.html#634-738){.src
.rightside}[§](#impl-Optimizable-for-BearPutSpread){.anchor}

### impl [Optimizable](trait.Optimizable.html "trait optionstratlib::strategies::base::Optimizable"){.trait} for [BearPutSpread](../bear_put_spread/struct.BearPutSpread.html "struct optionstratlib::strategies::bear_put_spread::BearPutSpread"){.struct} {#impl-optimizable-for-bearputspread .code-header}
:::

:::: impl-items
::: {#associatedtype.Strategy-2 .section .associatedtype .trait-impl}
[Source](../../../src/optionstratlib/strategies/bear_put_spread.rs.html#635){.src
.rightside}[§](#associatedtype.Strategy-2){.anchor}

#### type [Strategy](#associatedtype.Strategy){.associatedtype} = [BearPutSpread](../bear_put_spread/struct.BearPutSpread.html "struct optionstratlib::strategies::bear_put_spread::BearPutSpread"){.struct} {#type-strategy-bearputspread .code-header}
:::
::::

::: {#impl-Optimizable-for-BullCallSpread .section .impl}
[Source](../../../src/optionstratlib/strategies/bull_call_spread.rs.html#647-751){.src
.rightside}[§](#impl-Optimizable-for-BullCallSpread){.anchor}

### impl [Optimizable](trait.Optimizable.html "trait optionstratlib::strategies::base::Optimizable"){.trait} for [BullCallSpread](../bull_call_spread/struct.BullCallSpread.html "struct optionstratlib::strategies::bull_call_spread::BullCallSpread"){.struct} {#impl-optimizable-for-bullcallspread .code-header}
:::

:::: impl-items
::: {#associatedtype.Strategy-3 .section .associatedtype .trait-impl}
[Source](../../../src/optionstratlib/strategies/bull_call_spread.rs.html#648){.src
.rightside}[§](#associatedtype.Strategy-3){.anchor}

#### type [Strategy](#associatedtype.Strategy){.associatedtype} = [BullCallSpread](../bull_call_spread/struct.BullCallSpread.html "struct optionstratlib::strategies::bull_call_spread::BullCallSpread"){.struct} {#type-strategy-bullcallspread .code-header}
:::
::::

::: {#impl-Optimizable-for-BullPutSpread .section .impl}
[Source](../../../src/optionstratlib/strategies/bull_put_spread.rs.html#647-848){.src
.rightside}[§](#impl-Optimizable-for-BullPutSpread){.anchor}

### impl [Optimizable](trait.Optimizable.html "trait optionstratlib::strategies::base::Optimizable"){.trait} for [BullPutSpread](../bull_put_spread/struct.BullPutSpread.html "struct optionstratlib::strategies::bull_put_spread::BullPutSpread"){.struct} {#impl-optimizable-for-bullputspread .code-header}
:::

:::: impl-items
::: {#associatedtype.Strategy-4 .section .associatedtype .trait-impl}
[Source](../../../src/optionstratlib/strategies/bull_put_spread.rs.html#648){.src
.rightside}[§](#associatedtype.Strategy-4){.anchor}

#### type [Strategy](#associatedtype.Strategy){.associatedtype} = [BullPutSpread](../bull_put_spread/struct.BullPutSpread.html "struct optionstratlib::strategies::bull_put_spread::BullPutSpread"){.struct} {#type-strategy-bullputspread .code-header}
:::
::::

::: {#impl-Optimizable-for-CallButterfly .section .impl}
[Source](../../../src/optionstratlib/strategies/call_butterfly.rs.html#756-881){.src
.rightside}[§](#impl-Optimizable-for-CallButterfly){.anchor}

### impl [Optimizable](trait.Optimizable.html "trait optionstratlib::strategies::base::Optimizable"){.trait} for [CallButterfly](../call_butterfly/struct.CallButterfly.html "struct optionstratlib::strategies::call_butterfly::CallButterfly"){.struct} {#impl-optimizable-for-callbutterfly .code-header}
:::

:::: impl-items
::: {#associatedtype.Strategy-5 .section .associatedtype .trait-impl}
[Source](../../../src/optionstratlib/strategies/call_butterfly.rs.html#757){.src
.rightside}[§](#associatedtype.Strategy-5){.anchor}

#### type [Strategy](#associatedtype.Strategy){.associatedtype} = [CallButterfly](../call_butterfly/struct.CallButterfly.html "struct optionstratlib::strategies::call_butterfly::CallButterfly"){.struct} {#type-strategy-callbutterfly .code-header}
:::
::::

::: {#impl-Optimizable-for-CustomStrategy .section .impl}
[Source](../../../src/optionstratlib/strategies/custom.rs.html#767-824){.src
.rightside}[§](#impl-Optimizable-for-CustomStrategy){.anchor}

### impl [Optimizable](trait.Optimizable.html "trait optionstratlib::strategies::base::Optimizable"){.trait} for [CustomStrategy](../custom/struct.CustomStrategy.html "struct optionstratlib::strategies::custom::CustomStrategy"){.struct} {#impl-optimizable-for-customstrategy .code-header}
:::

:::: impl-items
::: {#associatedtype.Strategy-6 .section .associatedtype .trait-impl}
[Source](../../../src/optionstratlib/strategies/custom.rs.html#768){.src
.rightside}[§](#associatedtype.Strategy-6){.anchor}

#### type [Strategy](#associatedtype.Strategy){.associatedtype} = [CustomStrategy](../custom/struct.CustomStrategy.html "struct optionstratlib::strategies::custom::CustomStrategy"){.struct} {#type-strategy-customstrategy .code-header}
:::
::::

::: {#impl-Optimizable-for-IronButterfly .section .impl}
[Source](../../../src/optionstratlib/strategies/iron_butterfly.rs.html#827-952){.src
.rightside}[§](#impl-Optimizable-for-IronButterfly){.anchor}

### impl [Optimizable](trait.Optimizable.html "trait optionstratlib::strategies::base::Optimizable"){.trait} for [IronButterfly](../iron_butterfly/struct.IronButterfly.html "struct optionstratlib::strategies::iron_butterfly::IronButterfly"){.struct} {#impl-optimizable-for-ironbutterfly .code-header}
:::

:::: impl-items
::: {#associatedtype.Strategy-7 .section .associatedtype .trait-impl}
[Source](../../../src/optionstratlib/strategies/iron_butterfly.rs.html#828){.src
.rightside}[§](#associatedtype.Strategy-7){.anchor}

#### type [Strategy](#associatedtype.Strategy){.associatedtype} = [IronButterfly](../iron_butterfly/struct.IronButterfly.html "struct optionstratlib::strategies::iron_butterfly::IronButterfly"){.struct} {#type-strategy-ironbutterfly .code-header}
:::
::::

::: {#impl-Optimizable-for-IronCondor .section .impl}
[Source](../../../src/optionstratlib/strategies/iron_condor.rs.html#850-980){.src
.rightside}[§](#impl-Optimizable-for-IronCondor){.anchor}

### impl [Optimizable](trait.Optimizable.html "trait optionstratlib::strategies::base::Optimizable"){.trait} for [IronCondor](../iron_condor/struct.IronCondor.html "struct optionstratlib::strategies::iron_condor::IronCondor"){.struct} {#impl-optimizable-for-ironcondor .code-header}
:::

:::: impl-items
::: {#associatedtype.Strategy-8 .section .associatedtype .trait-impl}
[Source](../../../src/optionstratlib/strategies/iron_condor.rs.html#851){.src
.rightside}[§](#associatedtype.Strategy-8){.anchor}

#### type [Strategy](#associatedtype.Strategy){.associatedtype} = [IronCondor](../iron_condor/struct.IronCondor.html "struct optionstratlib::strategies::iron_condor::IronCondor"){.struct} {#type-strategy-ironcondor .code-header}
:::
::::

::: {#impl-Optimizable-for-LongButterflySpread .section .impl}
[Source](../../../src/optionstratlib/strategies/long_butterfly_spread.rs.html#780-913){.src
.rightside}[§](#impl-Optimizable-for-LongButterflySpread){.anchor}

### impl [Optimizable](trait.Optimizable.html "trait optionstratlib::strategies::base::Optimizable"){.trait} for [LongButterflySpread](../long_butterfly_spread/struct.LongButterflySpread.html "struct optionstratlib::strategies::long_butterfly_spread::LongButterflySpread"){.struct} {#impl-optimizable-for-longbutterflyspread .code-header}
:::

:::: impl-items
::: {#associatedtype.Strategy-9 .section .associatedtype .trait-impl}
[Source](../../../src/optionstratlib/strategies/long_butterfly_spread.rs.html#781){.src
.rightside}[§](#associatedtype.Strategy-9){.anchor}

#### type [Strategy](#associatedtype.Strategy){.associatedtype} = [LongButterflySpread](../long_butterfly_spread/struct.LongButterflySpread.html "struct optionstratlib::strategies::long_butterfly_spread::LongButterflySpread"){.struct} {#type-strategy-longbutterflyspread .code-header}
:::
::::

::: {#impl-Optimizable-for-LongCall .section .impl}
[Source](../../../src/optionstratlib/strategies/long_call.rs.html#414-425){.src
.rightside}[§](#impl-Optimizable-for-LongCall){.anchor}

### impl [Optimizable](trait.Optimizable.html "trait optionstratlib::strategies::base::Optimizable"){.trait} for [LongCall](../long_call/struct.LongCall.html "struct optionstratlib::strategies::long_call::LongCall"){.struct} {#impl-optimizable-for-longcall .code-header}
:::

:::: impl-items
::: {#associatedtype.Strategy-10 .section .associatedtype .trait-impl}
[Source](../../../src/optionstratlib/strategies/long_call.rs.html#415){.src
.rightside}[§](#associatedtype.Strategy-10){.anchor}

#### type [Strategy](#associatedtype.Strategy){.associatedtype} = [LongCall](../long_call/struct.LongCall.html "struct optionstratlib::strategies::long_call::LongCall"){.struct} {#type-strategy-longcall .code-header}
:::
::::

::: {#impl-Optimizable-for-LongPut .section .impl}
[Source](../../../src/optionstratlib/strategies/long_put.rs.html#411-422){.src
.rightside}[§](#impl-Optimizable-for-LongPut){.anchor}

### impl [Optimizable](trait.Optimizable.html "trait optionstratlib::strategies::base::Optimizable"){.trait} for [LongPut](../long_put/struct.LongPut.html "struct optionstratlib::strategies::long_put::LongPut"){.struct} {#impl-optimizable-for-longput .code-header}
:::

:::: impl-items
::: {#associatedtype.Strategy-11 .section .associatedtype .trait-impl}
[Source](../../../src/optionstratlib/strategies/long_put.rs.html#412){.src
.rightside}[§](#associatedtype.Strategy-11){.anchor}

#### type [Strategy](#associatedtype.Strategy){.associatedtype} = [LongPut](../long_put/struct.LongPut.html "struct optionstratlib::strategies::long_put::LongPut"){.struct} {#type-strategy-longput .code-header}
:::
::::

::: {#impl-Optimizable-for-LongStraddle .section .impl}
[Source](../../../src/optionstratlib/strategies/long_straddle.rs.html#633-740){.src
.rightside}[§](#impl-Optimizable-for-LongStraddle){.anchor}

### impl [Optimizable](trait.Optimizable.html "trait optionstratlib::strategies::base::Optimizable"){.trait} for [LongStraddle](../long_straddle/struct.LongStraddle.html "struct optionstratlib::strategies::long_straddle::LongStraddle"){.struct} {#impl-optimizable-for-longstraddle .code-header}
:::

:::: impl-items
::: {#associatedtype.Strategy-12 .section .associatedtype .trait-impl}
[Source](../../../src/optionstratlib/strategies/long_straddle.rs.html#634){.src
.rightside}[§](#associatedtype.Strategy-12){.anchor}

#### type [Strategy](#associatedtype.Strategy){.associatedtype} = [LongStraddle](../long_straddle/struct.LongStraddle.html "struct optionstratlib::strategies::long_straddle::LongStraddle"){.struct} {#type-strategy-longstraddle .code-header}
:::
::::

::: {#impl-Optimizable-for-LongStrangle .section .impl}
[Source](../../../src/optionstratlib/strategies/long_strangle.rs.html#675-799){.src
.rightside}[§](#impl-Optimizable-for-LongStrangle){.anchor}

### impl [Optimizable](trait.Optimizable.html "trait optionstratlib::strategies::base::Optimizable"){.trait} for [LongStrangle](../long_strangle/struct.LongStrangle.html "struct optionstratlib::strategies::long_strangle::LongStrangle"){.struct} {#impl-optimizable-for-longstrangle .code-header}
:::

:::: impl-items
::: {#associatedtype.Strategy-13 .section .associatedtype .trait-impl}
[Source](../../../src/optionstratlib/strategies/long_strangle.rs.html#676){.src
.rightside}[§](#associatedtype.Strategy-13){.anchor}

#### type [Strategy](#associatedtype.Strategy){.associatedtype} = [LongStrangle](../long_strangle/struct.LongStrangle.html "struct optionstratlib::strategies::long_strangle::LongStrangle"){.struct} {#type-strategy-longstrangle .code-header}
:::
::::

::: {#impl-Optimizable-for-PoorMansCoveredCall .section .impl}
[Source](../../../src/optionstratlib/strategies/poor_mans_covered_call.rs.html#657-749){.src
.rightside}[§](#impl-Optimizable-for-PoorMansCoveredCall){.anchor}

### impl [Optimizable](trait.Optimizable.html "trait optionstratlib::strategies::base::Optimizable"){.trait} for [PoorMansCoveredCall](../poor_mans_covered_call/struct.PoorMansCoveredCall.html "struct optionstratlib::strategies::poor_mans_covered_call::PoorMansCoveredCall"){.struct} {#impl-optimizable-for-poormanscoveredcall .code-header}
:::

:::: impl-items
::: {#associatedtype.Strategy-14 .section .associatedtype .trait-impl}
[Source](../../../src/optionstratlib/strategies/poor_mans_covered_call.rs.html#658){.src
.rightside}[§](#associatedtype.Strategy-14){.anchor}

#### type [Strategy](#associatedtype.Strategy){.associatedtype} = [PoorMansCoveredCall](../poor_mans_covered_call/struct.PoorMansCoveredCall.html "struct optionstratlib::strategies::poor_mans_covered_call::PoorMansCoveredCall"){.struct} {#type-strategy-poormanscoveredcall .code-header}
:::
::::

::: {#impl-Optimizable-for-ShortButterflySpread .section .impl}
[Source](../../../src/optionstratlib/strategies/short_butterfly_spread.rs.html#758-891){.src
.rightside}[§](#impl-Optimizable-for-ShortButterflySpread){.anchor}

### impl [Optimizable](trait.Optimizable.html "trait optionstratlib::strategies::base::Optimizable"){.trait} for [ShortButterflySpread](../short_butterfly_spread/struct.ShortButterflySpread.html "struct optionstratlib::strategies::short_butterfly_spread::ShortButterflySpread"){.struct} {#impl-optimizable-for-shortbutterflyspread .code-header}
:::

:::: impl-items
::: {#associatedtype.Strategy-15 .section .associatedtype .trait-impl}
[Source](../../../src/optionstratlib/strategies/short_butterfly_spread.rs.html#759){.src
.rightside}[§](#associatedtype.Strategy-15){.anchor}

#### type [Strategy](#associatedtype.Strategy){.associatedtype} = [ShortButterflySpread](../short_butterfly_spread/struct.ShortButterflySpread.html "struct optionstratlib::strategies::short_butterfly_spread::ShortButterflySpread"){.struct} {#type-strategy-shortbutterflyspread .code-header}
:::
::::

::: {#impl-Optimizable-for-ShortCall .section .impl}
[Source](../../../src/optionstratlib/strategies/short_call.rs.html#422-433){.src
.rightside}[§](#impl-Optimizable-for-ShortCall){.anchor}

### impl [Optimizable](trait.Optimizable.html "trait optionstratlib::strategies::base::Optimizable"){.trait} for [ShortCall](../short_call/struct.ShortCall.html "struct optionstratlib::strategies::short_call::ShortCall"){.struct} {#impl-optimizable-for-shortcall .code-header}
:::

:::: impl-items
::: {#associatedtype.Strategy-16 .section .associatedtype .trait-impl}
[Source](../../../src/optionstratlib/strategies/short_call.rs.html#423){.src
.rightside}[§](#associatedtype.Strategy-16){.anchor}

#### type [Strategy](#associatedtype.Strategy){.associatedtype} = [ShortCall](../short_call/struct.ShortCall.html "struct optionstratlib::strategies::short_call::ShortCall"){.struct} {#type-strategy-shortcall .code-header}
:::
::::

::: {#impl-Optimizable-for-ShortPut .section .impl}
[Source](../../../src/optionstratlib/strategies/short_put.rs.html#416-427){.src
.rightside}[§](#impl-Optimizable-for-ShortPut){.anchor}

### impl [Optimizable](trait.Optimizable.html "trait optionstratlib::strategies::base::Optimizable"){.trait} for [ShortPut](../short_put/struct.ShortPut.html "struct optionstratlib::strategies::short_put::ShortPut"){.struct} {#impl-optimizable-for-shortput .code-header}
:::

:::: impl-items
::: {#associatedtype.Strategy-17 .section .associatedtype .trait-impl}
[Source](../../../src/optionstratlib/strategies/short_put.rs.html#417){.src
.rightside}[§](#associatedtype.Strategy-17){.anchor}

#### type [Strategy](#associatedtype.Strategy){.associatedtype} = [ShortPut](../short_put/struct.ShortPut.html "struct optionstratlib::strategies::short_put::ShortPut"){.struct} {#type-strategy-shortput .code-header}
:::
::::

::: {#impl-Optimizable-for-ShortStraddle .section .impl}
[Source](../../../src/optionstratlib/strategies/short_straddle.rs.html#658-774){.src
.rightside}[§](#impl-Optimizable-for-ShortStraddle){.anchor}

### impl [Optimizable](trait.Optimizable.html "trait optionstratlib::strategies::base::Optimizable"){.trait} for [ShortStraddle](../short_straddle/struct.ShortStraddle.html "struct optionstratlib::strategies::short_straddle::ShortStraddle"){.struct} {#impl-optimizable-for-shortstraddle .code-header}
:::

:::: impl-items
::: {#associatedtype.Strategy-18 .section .associatedtype .trait-impl}
[Source](../../../src/optionstratlib/strategies/short_straddle.rs.html#659){.src
.rightside}[§](#associatedtype.Strategy-18){.anchor}

#### type [Strategy](#associatedtype.Strategy){.associatedtype} = [ShortStraddle](../short_straddle/struct.ShortStraddle.html "struct optionstratlib::strategies::short_straddle::ShortStraddle"){.struct} {#type-strategy-shortstraddle .code-header}
:::
::::

::: {#impl-Optimizable-for-ShortStrangle .section .impl}
[Source](../../../src/optionstratlib/strategies/short_strangle.rs.html#861-1037){.src
.rightside}[§](#impl-Optimizable-for-ShortStrangle){.anchor}

### impl [Optimizable](trait.Optimizable.html "trait optionstratlib::strategies::base::Optimizable"){.trait} for [ShortStrangle](../short_strangle/struct.ShortStrangle.html "struct optionstratlib::strategies::short_strangle::ShortStrangle"){.struct} {#impl-optimizable-for-shortstrangle .code-header}
:::

:::: impl-items
::: {#associatedtype.Strategy-19 .section .associatedtype .trait-impl}
[Source](../../../src/optionstratlib/strategies/short_strangle.rs.html#862){.src
.rightside}[§](#associatedtype.Strategy-19){.anchor}

#### type [Strategy](#associatedtype.Strategy){.associatedtype} = [ShortStrangle](../short_strangle/struct.ShortStrangle.html "struct optionstratlib::strategies::short_strangle::ShortStrangle"){.struct} {#type-strategy-shortstrangle .code-header}
:::
::::
::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
:::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
