::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::: width-limiter
:::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../../index.html)::[strategies](../index.html)::[bear_put_spread](index.html)
:::

# Struct [BearPutSpread]{.struct}Copy item path

[[Source](../../../src/optionstratlib/strategies/bear_put_spread.rs.html#65-78){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub struct BearPutSpread {
    pub name: String,
    pub kind: StrategyType,
    pub description: String,
    pub break_even_points: Vec<Positive>,
    /* private fields */
}
```

Expand description

::: docblock
Represents a Bear Put Spread options trading strategy.

A Bear Put Spread is a bearish options strategy that involves buying a
put option at a higher strike price and simultaneously selling another
put option at a lower strike price, both with the same expiration date.
This strategy is used when expecting a moderate decline in the price of
the underlying asset.

The strategy benefits from limited risk (the net premium paid) and
limited profit potential (the difference between strike prices minus the
net premium paid). It is less expensive than buying a single put
outright due to premium received from the short put.

## [§](#attributes){.doc-anchor}Attributes
:::

## Fields[§](#fields){.anchor} {#fields .fields .section-header}

[[§](#structfield.name){.anchor
.field}`name: `[`String`](https://doc.rust-lang.org/1.86.0/alloc/string/struct.String.html "struct alloc::string::String"){.struct}]{#structfield.name
.structfield .section-header}

::: docblock
The name identifier for this specific strategy instance.
:::

[[§](#structfield.kind){.anchor
.field}`kind: `[`StrategyType`](../base/enum.StrategyType.html "enum optionstratlib::strategies::base::StrategyType"){.enum}]{#structfield.kind
.structfield .section-header}

::: docblock
The type of strategy, should be StrategyType::BearPutSpread.
:::

[[§](#structfield.description){.anchor
.field}`description: `[`String`](https://doc.rust-lang.org/1.86.0/alloc/string/struct.String.html "struct alloc::string::String"){.struct}]{#structfield.description
.structfield .section-header}

::: docblock
A detailed description of this specific strategy implementation.
:::

[[§](#structfield.break_even_points){.anchor
.field}`break_even_points: `[`Vec`](https://doc.rust-lang.org/1.86.0/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}`<`[`Positive`](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}`>`]{#structfield.break_even_points
.structfield .section-header}

::: docblock
The price points at which the strategy breaks even (neither profit nor
loss).
:::

## Implementations[§](#implementations){.anchor} {#implementations .section-header}

::::::: {#implementations-list}
::: {#impl-BearPutSpread .section .impl}
[Source](../../../src/optionstratlib/strategies/bear_put_spread.rs.html#80-211){.src
.rightside}[§](#impl-BearPutSpread){.anchor}

### impl [BearPutSpread](struct.BearPutSpread.html "struct optionstratlib::strategies::bear_put_spread::BearPutSpread"){.struct} {#impl-bearputspread .code-header}
:::

::::: impl-items
::: {#method.new .section .method}
[Source](../../../src/optionstratlib/strategies/bear_put_spread.rs.html#121-210){.src
.rightside}

#### pub fn [new](#method.new){.fn}( underlying_symbol: [String](https://doc.rust-lang.org/1.86.0/alloc/string/struct.String.html "struct alloc::string::String"){.struct}, underlying_price: [Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, long_strike: [Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, short_strike: [Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, expiration: [ExpirationDate](../../model/types/enum.ExpirationDate.html "enum optionstratlib::model::types::ExpirationDate"){.enum}, implied_volatility: [Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, risk_free_rate: Decimal, dividend_yield: [Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, quantity: [Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, premium_long_put: [Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, premium_short_put: [Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, open_fee_long_put: [Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, close_fee_long_put: [Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, open_fee_short_put: [Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, close_fee_short_put: [Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, ) -\> Self {#pub-fn-new-underlying_symbol-string-underlying_price-positive-long_strike-positive-short_strike-positive-expiration-expirationdate-implied_volatility-positive-risk_free_rate-decimal-dividend_yield-positive-quantity-positive-premium_long_put-positive-premium_short_put-positive-open_fee_long_put-positive-close_fee_long_put-positive-open_fee_short_put-positive-close_fee_short_put-positive---self .code-header}
:::

::: docblock
Creates a new Bear Put Spread options strategy.

A bear put spread is created by buying a put option with a higher strike
price and simultaneously selling a put option with a lower strike price,
both with the same expiration date. This strategy is used when you
expect a moderate decrease in the underlying asset's price.

##### [§](#parameters){.doc-anchor}Parameters

- `underlying_symbol` - The symbol of the underlying asset.
- `underlying_price` - The current price of the underlying asset.
- `long_strike` - Strike price for the long put position. If set to
  zero, defaults to the underlying price.
- `short_strike` - Strike price for the short put position. If set to
  zero, defaults to the underlying price.
- `expiration` - The expiration date of the options contracts.
- `implied_volatility` - The implied volatility used for option pricing
  calculations.
- `risk_free_rate` - The risk-free interest rate used in option pricing
  models.
- `dividend_yield` - The dividend yield of the underlying asset.
- `quantity` - The number of option contracts in the strategy.
- `premium_long_put` - The premium paid for the long put option.
- `premium_short_put` - The premium received for the short put option.
- `open_fee_long_put` - The fee paid when opening the long put position.
- `close_fee_long_put` - The fee paid when closing the long put
  position.
- `open_fee_short_put` - The fee paid when opening the short put
  position.
- `close_fee_short_put` - The fee paid when closing the short put
  position.

##### [§](#returns){.doc-anchor}Returns

A validated `BearPutSpread` strategy instance with calculated break-even
points.

##### [§](#validation){.doc-anchor}Validation

The function performs validation to ensure:

- Both put positions are valid
- The long put strike price is higher than the short put strike price

##### [§](#note){.doc-anchor}Note

The maximum profit is limited to the difference between strike prices
minus the net premium paid, while the maximum loss is limited to the net
premium paid.
:::
:::::
:::::::

## Trait Implementations[§](#trait-implementations){.anchor} {#trait-implementations .section-header}

::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::: {#trait-implementations-list}
::: {#impl-BreakEvenable-for-BearPutSpread .section .impl}
[Source](../../../src/optionstratlib/strategies/bear_put_spread.rs.html#305-320){.src
.rightside}[§](#impl-BreakEvenable-for-BearPutSpread){.anchor}

### impl [BreakEvenable](../base/trait.BreakEvenable.html "trait optionstratlib::strategies::base::BreakEvenable"){.trait} for [BearPutSpread](struct.BearPutSpread.html "struct optionstratlib::strategies::bear_put_spread::BearPutSpread"){.struct} {#impl-breakevenable-for-bearputspread .code-header}
:::

::::::: impl-items
::: {#method.get_break_even_points .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/bear_put_spread.rs.html#306-308){.src
.rightside}[§](#method.get_break_even_points){.anchor}

#### fn [get_break_even_points](../base/trait.BreakEvenable.html#method.get_break_even_points){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<&[Vec](https://doc.rust-lang.org/1.86.0/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\>, [StrategyError](../../error/strategies/enum.StrategyError.html "enum optionstratlib::error::strategies::StrategyError"){.enum}\> {#fn-get_break_even_pointsself---resultvecpositive-strategyerror .code-header}
:::

::: docblock
Retrieves the break-even points for the strategy. [Read
more](../base/trait.BreakEvenable.html#method.get_break_even_points)
:::

::: {#method.update_break_even_points .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/bear_put_spread.rs.html#310-319){.src
.rightside}[§](#method.update_break_even_points){.anchor}

#### fn [update_break_even_points](../base/trait.BreakEvenable.html#method.update_break_even_points){.fn}(&mut self) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[()](https://doc.rust-lang.org/1.86.0/std/primitive.unit.html){.primitive}, [StrategyError](../../error/strategies/enum.StrategyError.html "enum optionstratlib::error::strategies::StrategyError"){.enum}\> {#fn-update_break_even_pointsmut-self---result-strategyerror .code-header}
:::

::: docblock
Updates the break-even points for the strategy. [Read
more](../base/trait.BreakEvenable.html#method.update_break_even_points)
:::
:::::::

::: {#impl-Clone-for-BearPutSpread .section .impl}
[Source](../../../src/optionstratlib/strategies/bear_put_spread.rs.html#64){.src
.rightside}[§](#impl-Clone-for-BearPutSpread){.anchor}

### impl [Clone](https://doc.rust-lang.org/1.86.0/core/clone/trait.Clone.html "trait core::clone::Clone"){.trait} for [BearPutSpread](struct.BearPutSpread.html "struct optionstratlib::strategies::bear_put_spread::BearPutSpread"){.struct} {#impl-clone-for-bearputspread .code-header}
:::

::::::: impl-items
::: {#method.clone .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/bear_put_spread.rs.html#64){.src
.rightside}[§](#method.clone){.anchor}

#### fn [clone](https://doc.rust-lang.org/1.86.0/core/clone/trait.Clone.html#tymethod.clone){.fn}(&self) -\> [BearPutSpread](struct.BearPutSpread.html "struct optionstratlib::strategies::bear_put_spread::BearPutSpread"){.struct} {#fn-cloneself---bearputspread .code-header}
:::

::: docblock
Returns a copy of the value. [Read
more](https://doc.rust-lang.org/1.86.0/core/clone/trait.Clone.html#tymethod.clone)
:::

::: {#method.clone_from .section .method .trait-impl}
[[1.0.0]{.since title="Stable since Rust version 1.0.0"} ·
[Source](https://doc.rust-lang.org/1.86.0/src/core/clone.rs.html#174){.src}]{.rightside}[§](#method.clone_from){.anchor}

#### fn [clone_from](https://doc.rust-lang.org/1.86.0/core/clone/trait.Clone.html#method.clone_from){.fn}(&mut self, source: &Self) {#fn-clone_frommut-self-source-self .code-header}
:::

::: docblock
Performs copy-assignment from `source`. [Read
more](https://doc.rust-lang.org/1.86.0/core/clone/trait.Clone.html#method.clone_from)
:::
:::::::

::: {#impl-Debug-for-BearPutSpread .section .impl}
[Source](../../../src/optionstratlib/strategies/bear_put_spread.rs.html#64){.src
.rightside}[§](#impl-Debug-for-BearPutSpread){.anchor}

### impl [Debug](https://doc.rust-lang.org/1.86.0/core/fmt/trait.Debug.html "trait core::fmt::Debug"){.trait} for [BearPutSpread](struct.BearPutSpread.html "struct optionstratlib::strategies::bear_put_spread::BearPutSpread"){.struct} {#impl-debug-for-bearputspread .code-header}
:::

::::: impl-items
::: {#method.fmt .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/bear_put_spread.rs.html#64){.src
.rightside}[§](#method.fmt){.anchor}

#### fn [fmt](https://doc.rust-lang.org/1.86.0/core/fmt/trait.Debug.html#tymethod.fmt){.fn}(&self, f: &mut [Formatter](https://doc.rust-lang.org/1.86.0/core/fmt/struct.Formatter.html "struct core::fmt::Formatter"){.struct}\<\'\_\>) -\> [Result](https://doc.rust-lang.org/1.86.0/core/fmt/type.Result.html "type core::fmt::Result"){.type} {#fn-fmtself-f-mut-formatter_---result .code-header}
:::

::: docblock
Formats the value using the given formatter. [Read
more](https://doc.rust-lang.org/1.86.0/core/fmt/trait.Debug.html#tymethod.fmt)
:::
:::::

::: {#impl-DeltaNeutrality-for-BearPutSpread .section .impl}
[Source](../../../src/optionstratlib/strategies/bear_put_spread.rs.html#771){.src
.rightside}[§](#impl-DeltaNeutrality-for-BearPutSpread){.anchor}

### impl [DeltaNeutrality](../delta_neutral/trait.DeltaNeutrality.html "trait optionstratlib::strategies::delta_neutral::DeltaNeutrality"){.trait} for [BearPutSpread](struct.BearPutSpread.html "struct optionstratlib::strategies::bear_put_spread::BearPutSpread"){.struct} {#impl-deltaneutrality-for-bearputspread .code-header}
:::

::::::::::::::::::: impl-items
::: {#method.delta_neutrality .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/delta_neutral/model.rs.html#329-353){.src
.rightside}[§](#method.delta_neutrality){.anchor}

#### fn [delta_neutrality](../delta_neutral/trait.DeltaNeutrality.html#method.delta_neutrality){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[DeltaInfo](../delta_neutral/struct.DeltaInfo.html "struct optionstratlib::strategies::delta_neutral::DeltaInfo"){.struct}, [GreeksError](../../error/greeks/enum.GreeksError.html "enum optionstratlib::error::greeks::GreeksError"){.enum}\> {#fn-delta_neutralityself---resultdeltainfo-greekserror .code-header}
:::

::: docblock
Calculates the net delta of the strategy and provides detailed
information. [Read
more](../delta_neutral/trait.DeltaNeutrality.html#method.delta_neutrality)
:::

::: {#method.is_delta_neutral .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/delta_neutral/model.rs.html#364-369){.src
.rightside}[§](#method.is_delta_neutral){.anchor}

#### fn [is_delta_neutral](../delta_neutral/trait.DeltaNeutrality.html#method.is_delta_neutral){.fn}(&self) -\> [bool](https://doc.rust-lang.org/1.86.0/std/primitive.bool.html){.primitive} {#fn-is_delta_neutralself---bool .code-header}
:::

::: docblock
Checks if the strategy is delta neutral within the specified threshold.
[Read
more](../delta_neutral/trait.DeltaNeutrality.html#method.is_delta_neutral)
:::

::: {#method.get_atm_strike .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/delta_neutral/model.rs.html#393-395){.src
.rightside}[§](#method.get_atm_strike){.anchor}

#### fn [get_atm_strike](../delta_neutral/trait.DeltaNeutrality.html#method.get_atm_strike){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, [StrategyError](../../error/strategies/enum.StrategyError.html "enum optionstratlib::error::strategies::StrategyError"){.enum}\> {#fn-get_atm_strikeself---resultpositive-strategyerror .code-header}
:::

::: docblock
get_atm_strike [Read
more](../delta_neutral/trait.DeltaNeutrality.html#method.get_atm_strike)
:::

::: {#method.delta_adjustments .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/delta_neutral/model.rs.html#410-520){.src
.rightside}[§](#method.delta_adjustments){.anchor}

#### fn [delta_adjustments](../delta_neutral/trait.DeltaNeutrality.html#method.delta_adjustments){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Vec](https://doc.rust-lang.org/1.86.0/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<[DeltaAdjustment](../delta_neutral/enum.DeltaAdjustment.html "enum optionstratlib::strategies::delta_neutral::DeltaAdjustment"){.enum}\>, [GreeksError](../../error/greeks/enum.GreeksError.html "enum optionstratlib::error::greeks::GreeksError"){.enum}\> {#fn-delta_adjustmentsself---resultvecdeltaadjustment-greekserror .code-header}
:::

::: docblock
Calculates required position adjustments to maintain delta neutrality
[Read
more](../delta_neutral/trait.DeltaNeutrality.html#method.delta_adjustments)
:::

::: {#method.apply_delta_adjustments .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/delta_neutral/model.rs.html#552-600){.src
.rightside}[§](#method.apply_delta_adjustments){.anchor}

#### fn [apply_delta_adjustments](../delta_neutral/trait.DeltaNeutrality.html#method.apply_delta_adjustments){.fn}( &mut self, action: [Option](https://doc.rust-lang.org/1.86.0/core/option/enum.Option.html "enum core::option::Option"){.enum}\<[Action](../../model/types/enum.Action.html "enum optionstratlib::model::types::Action"){.enum}\>, ) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[()](https://doc.rust-lang.org/1.86.0/std/primitive.unit.html){.primitive}, [Box](https://doc.rust-lang.org/1.86.0/alloc/boxed/struct.Box.html "struct alloc::boxed::Box"){.struct}\<dyn [Error](https://doc.rust-lang.org/1.86.0/core/error/trait.Error.html "trait core::error::Error"){.trait}\>\> {#fn-apply_delta_adjustments-mut-self-action-optionaction---result-boxdyn-error .code-header}
:::

::: docblock
Apply Delta Adjustments [Read
more](../delta_neutral/trait.DeltaNeutrality.html#method.apply_delta_adjustments)
:::

::: {#method.apply_single_adjustment .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/delta_neutral/model.rs.html#632-664){.src
.rightside}[§](#method.apply_single_adjustment){.anchor}

#### fn [apply_single_adjustment](../delta_neutral/trait.DeltaNeutrality.html#method.apply_single_adjustment){.fn}( &mut self, adjustment: &[DeltaAdjustment](../delta_neutral/enum.DeltaAdjustment.html "enum optionstratlib::strategies::delta_neutral::DeltaAdjustment"){.enum}, ) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[()](https://doc.rust-lang.org/1.86.0/std/primitive.unit.html){.primitive}, [Box](https://doc.rust-lang.org/1.86.0/alloc/boxed/struct.Box.html "struct alloc::boxed::Box"){.struct}\<dyn [Error](https://doc.rust-lang.org/1.86.0/core/error/trait.Error.html "trait core::error::Error"){.trait}\>\> {#fn-apply_single_adjustment-mut-self-adjustment-deltaadjustment---result-boxdyn-error .code-header}
:::

::: docblock
Apply Single Adjustment [Read
more](../delta_neutral/trait.DeltaNeutrality.html#method.apply_single_adjustment)
:::

::: {#method.adjust_underlying_position .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/delta_neutral/model.rs.html#697-705){.src
.rightside}[§](#method.adjust_underlying_position){.anchor}

#### fn [adjust_underlying_position](../delta_neutral/trait.DeltaNeutrality.html#method.adjust_underlying_position){.fn}( &mut self, \_quantity: [Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, \_side: [Side](../../model/types/enum.Side.html "enum optionstratlib::model::types::Side"){.enum}, ) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[()](https://doc.rust-lang.org/1.86.0/std/primitive.unit.html){.primitive}, [Box](https://doc.rust-lang.org/1.86.0/alloc/boxed/struct.Box.html "struct alloc::boxed::Box"){.struct}\<dyn [Error](https://doc.rust-lang.org/1.86.0/core/error/trait.Error.html "trait core::error::Error"){.trait}\>\> {#fn-adjust_underlying_position-mut-self-_quantity-positive-_side-side---result-boxdyn-error .code-header}
:::

::: docblock
Adjust Underlying Position [Read
more](../delta_neutral/trait.DeltaNeutrality.html#method.adjust_underlying_position)
:::

::: {#method.adjust_option_position .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/delta_neutral/model.rs.html#734-754){.src
.rightside}[§](#method.adjust_option_position){.anchor}

#### fn [adjust_option_position](../delta_neutral/trait.DeltaNeutrality.html#method.adjust_option_position){.fn}( &mut self, quantity: Decimal, strike: &[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, option_type: &[OptionStyle](../../model/types/enum.OptionStyle.html "enum optionstratlib::model::types::OptionStyle"){.enum}, side: &[Side](../../model/types/enum.Side.html "enum optionstratlib::model::types::Side"){.enum}, ) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[()](https://doc.rust-lang.org/1.86.0/std/primitive.unit.html){.primitive}, [Box](https://doc.rust-lang.org/1.86.0/alloc/boxed/struct.Box.html "struct alloc::boxed::Box"){.struct}\<dyn [Error](https://doc.rust-lang.org/1.86.0/core/error/trait.Error.html "trait core::error::Error"){.trait}\>\> {#fn-adjust_option_position-mut-self-quantity-decimal-strike-positive-option_type-optionstyle-side-side---result-boxdyn-error .code-header}
:::

::: docblock
Adjust Option Position [Read
more](../delta_neutral/trait.DeltaNeutrality.html#method.adjust_option_position)
:::
:::::::::::::::::::

::: {#impl-Graph-for-BearPutSpread .section .impl}
[Source](../../../src/optionstratlib/strategies/bear_put_spread.rs.html#618-697){.src
.rightside}[§](#impl-Graph-for-BearPutSpread){.anchor}

### impl [Graph](../../visualization/utils/trait.Graph.html "trait optionstratlib::visualization::utils::Graph"){.trait} for [BearPutSpread](struct.BearPutSpread.html "struct optionstratlib::strategies::bear_put_spread::BearPutSpread"){.struct} {#impl-graph-for-bearputspread .code-header}
:::

::::::::::::::: impl-items
::: {#method.title .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/bear_put_spread.rs.html#619-626){.src
.rightside}[§](#method.title){.anchor}

#### fn [title](../../visualization/utils/trait.Graph.html#tymethod.title){.fn}(&self) -\> [String](https://doc.rust-lang.org/1.86.0/alloc/string/struct.String.html "struct alloc::string::String"){.struct} {#fn-titleself---string .code-header}
:::

::: docblock
Returns the title of the graph.
:::

::: {#method.get_x_values .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/bear_put_spread.rs.html#628-631){.src
.rightside}[§](#method.get_x_values){.anchor}

#### fn [get_x_values](../../visualization/utils/trait.Graph.html#tymethod.get_x_values){.fn}(&self) -\> [Vec](https://doc.rust-lang.org/1.86.0/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\> {#fn-get_x_valuesself---vecpositive .code-header}
:::

::: docblock
Returns a collection of positive X values for visualization. [Read
more](../../visualization/utils/trait.Graph.html#tymethod.get_x_values)
:::

::: {#method.get_vertical_lines .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/bear_put_spread.rs.html#633-645){.src
.rightside}[§](#method.get_vertical_lines){.anchor}

#### fn [get_vertical_lines](../../visualization/utils/trait.Graph.html#method.get_vertical_lines){.fn}(&self) -\> [Vec](https://doc.rust-lang.org/1.86.0/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<ChartVerticalLine\<[f64](https://doc.rust-lang.org/1.86.0/std/primitive.f64.html){.primitive}, [f64](https://doc.rust-lang.org/1.86.0/std/primitive.f64.html){.primitive}\>\> {#fn-get_vertical_linesself---vecchartverticallinef64-f64 .code-header}
:::

::: docblock
Returns a vector of vertical lines to draw on the chart. Default
implementation returns an empty vector.
:::

::: {#method.get_points .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/bear_put_spread.rs.html#647-696){.src
.rightside}[§](#method.get_points){.anchor}

#### fn [get_points](../../visualization/utils/trait.Graph.html#method.get_points){.fn}(&self) -\> [Vec](https://doc.rust-lang.org/1.86.0/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<ChartPoint\<([f64](https://doc.rust-lang.org/1.86.0/std/primitive.f64.html){.primitive}, [f64](https://doc.rust-lang.org/1.86.0/std/primitive.f64.html){.primitive})\>\> {#fn-get_pointsself---vecchartpointf64-f64 .code-header}
:::

::: docblock
Returns a vector of points to draw on the chart. Default implementation
returns an empty vector.
:::

::: {#method.graph .section .method .trait-impl}
[Source](../../../src/optionstratlib/visualization/utils.rs.html#198-240){.src
.rightside}[§](#method.graph){.anchor}

#### fn [graph](../../visualization/utils/trait.Graph.html#method.graph){.fn}( &self, backend: [GraphBackend](../../visualization/utils/enum.GraphBackend.html "enum optionstratlib::visualization::utils::GraphBackend"){.enum}\<\'\_\>, title_size: [u32](https://doc.rust-lang.org/1.86.0/std/primitive.u32.html){.primitive}, ) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[()](https://doc.rust-lang.org/1.86.0/std/primitive.unit.html){.primitive}, [Box](https://doc.rust-lang.org/1.86.0/alloc/boxed/struct.Box.html "struct alloc::boxed::Box"){.struct}\<dyn [Error](https://doc.rust-lang.org/1.86.0/core/error/trait.Error.html "trait core::error::Error"){.trait}\>\> {#fn-graph-self-backend-graphbackend_-title_size-u32---result-boxdyn-error .code-header}
:::

::: docblock
Generates a graph of profit calculations. [Read
more](../../visualization/utils/trait.Graph.html#method.graph)
:::

::: {#method.get_y_values .section .method .trait-impl}
[Source](../../../src/optionstratlib/visualization/utils.rs.html#269-279){.src
.rightside}[§](#method.get_y_values){.anchor}

#### fn [get_y_values](../../visualization/utils/trait.Graph.html#method.get_y_values){.fn}(&self) -\> [Vec](https://doc.rust-lang.org/1.86.0/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<[f64](https://doc.rust-lang.org/1.86.0/std/primitive.f64.html){.primitive}\> {#fn-get_y_valuesself---vecf64 .code-header}
:::

::: docblock
Calculates the y-axis values (profit) corresponding to the provided
x-axis data. [Read
more](../../visualization/utils/trait.Graph.html#method.get_y_values)
:::
:::::::::::::::

::: {#impl-Greeks-for-BearPutSpread .section .impl}
[Source](../../../src/optionstratlib/strategies/bear_put_spread.rs.html#765-769){.src
.rightside}[§](#impl-Greeks-for-BearPutSpread){.anchor}

### impl [Greeks](../../greeks/trait.Greeks.html "trait optionstratlib::greeks::Greeks"){.trait} for [BearPutSpread](struct.BearPutSpread.html "struct optionstratlib::strategies::bear_put_spread::BearPutSpread"){.struct} {#impl-greeks-for-bearputspread .code-header}
:::

::::::::::::::::::::: impl-items
::: {#method.get_options .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/bear_put_spread.rs.html#766-768){.src
.rightside}[§](#method.get_options){.anchor}

#### fn [get_options](../../greeks/trait.Greeks.html#tymethod.get_options){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Vec](https://doc.rust-lang.org/1.86.0/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<&[Options](../../model/option/struct.Options.html "struct optionstratlib::model::option::Options"){.struct}\>, [GreeksError](../../error/greeks/enum.GreeksError.html "enum optionstratlib::error::greeks::GreeksError"){.enum}\> {#fn-get_optionsself---resultvecoptions-greekserror .code-header}
:::

::: docblock
Returns a vector of references to the option contracts for which Greeks
will be calculated. [Read
more](../../greeks/trait.Greeks.html#tymethod.get_options)
:::

::: {#method.greeks .section .method .trait-impl}
[Source](../../../src/optionstratlib/greeks/equations.rs.html#99-116){.src
.rightside}[§](#method.greeks){.anchor}

#### fn [greeks](../../greeks/trait.Greeks.html#method.greeks){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Greek](../../greeks/struct.Greek.html "struct optionstratlib::greeks::Greek"){.struct}, [GreeksError](../../error/greeks/enum.GreeksError.html "enum optionstratlib::error::greeks::GreeksError"){.enum}\> {#fn-greeksself---resultgreek-greekserror .code-header}
:::

::: docblock
Calculates and returns all Greeks as a single `Greek` struct. [Read
more](../../greeks/trait.Greeks.html#method.greeks)
:::

::: {#method.delta .section .method .trait-impl}
[Source](../../../src/optionstratlib/greeks/equations.rs.html#126-133){.src
.rightside}[§](#method.delta){.anchor}

#### fn [delta](../../greeks/trait.Greeks.html#method.delta){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<Decimal, [GreeksError](../../error/greeks/enum.GreeksError.html "enum optionstratlib::error::greeks::GreeksError"){.enum}\> {#fn-deltaself---resultdecimal-greekserror .code-header}
:::

::: docblock
Calculates the aggregate delta value for all options. [Read
more](../../greeks/trait.Greeks.html#method.delta)
:::

::: {#method.gamma .section .method .trait-impl}
[Source](../../../src/optionstratlib/greeks/equations.rs.html#143-150){.src
.rightside}[§](#method.gamma){.anchor}

#### fn [gamma](../../greeks/trait.Greeks.html#method.gamma){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<Decimal, [GreeksError](../../error/greeks/enum.GreeksError.html "enum optionstratlib::error::greeks::GreeksError"){.enum}\> {#fn-gammaself---resultdecimal-greekserror .code-header}
:::

::: docblock
Calculates the aggregate gamma value for all options. [Read
more](../../greeks/trait.Greeks.html#method.gamma)
:::

::: {#method.theta .section .method .trait-impl}
[Source](../../../src/optionstratlib/greeks/equations.rs.html#160-167){.src
.rightside}[§](#method.theta){.anchor}

#### fn [theta](../../greeks/trait.Greeks.html#method.theta){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<Decimal, [GreeksError](../../error/greeks/enum.GreeksError.html "enum optionstratlib::error::greeks::GreeksError"){.enum}\> {#fn-thetaself---resultdecimal-greekserror .code-header}
:::

::: docblock
Calculates the aggregate theta value for all options. [Read
more](../../greeks/trait.Greeks.html#method.theta)
:::

::: {#method.vega .section .method .trait-impl}
[Source](../../../src/optionstratlib/greeks/equations.rs.html#177-184){.src
.rightside}[§](#method.vega){.anchor}

#### fn [vega](../../greeks/trait.Greeks.html#method.vega){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<Decimal, [GreeksError](../../error/greeks/enum.GreeksError.html "enum optionstratlib::error::greeks::GreeksError"){.enum}\> {#fn-vegaself---resultdecimal-greekserror .code-header}
:::

::: docblock
Calculates the aggregate vega value for all options. [Read
more](../../greeks/trait.Greeks.html#method.vega)
:::

::: {#method.rho .section .method .trait-impl}
[Source](../../../src/optionstratlib/greeks/equations.rs.html#194-201){.src
.rightside}[§](#method.rho){.anchor}

#### fn [rho](../../greeks/trait.Greeks.html#method.rho){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<Decimal, [GreeksError](../../error/greeks/enum.GreeksError.html "enum optionstratlib::error::greeks::GreeksError"){.enum}\> {#fn-rhoself---resultdecimal-greekserror .code-header}
:::

::: docblock
Calculates the aggregate rho value for all options. [Read
more](../../greeks/trait.Greeks.html#method.rho)
:::

::: {#method.rho_d .section .method .trait-impl}
[Source](../../../src/optionstratlib/greeks/equations.rs.html#211-218){.src
.rightside}[§](#method.rho_d){.anchor}

#### fn [rho_d](../../greeks/trait.Greeks.html#method.rho_d){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<Decimal, [GreeksError](../../error/greeks/enum.GreeksError.html "enum optionstratlib::error::greeks::GreeksError"){.enum}\> {#fn-rho_dself---resultdecimal-greekserror .code-header}
:::

::: docblock
Calculates the aggregate rho_d value for all options. [Read
more](../../greeks/trait.Greeks.html#method.rho_d)
:::

::: {#method.alpha .section .method .trait-impl}
[Source](../../../src/optionstratlib/greeks/equations.rs.html#228-235){.src
.rightside}[§](#method.alpha){.anchor}

#### fn [alpha](../../greeks/trait.Greeks.html#method.alpha){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<Decimal, [GreeksError](../../error/greeks/enum.GreeksError.html "enum optionstratlib::error::greeks::GreeksError"){.enum}\> {#fn-alphaself---resultdecimal-greekserror .code-header}
:::

::: docblock
Calculates the aggregate alpha value for all options. [Read
more](../../greeks/trait.Greeks.html#method.alpha)
:::
:::::::::::::::::::::

::: {#impl-Optimizable-for-BearPutSpread .section .impl}
[Source](../../../src/optionstratlib/strategies/bear_put_spread.rs.html#507-609){.src
.rightside}[§](#impl-Optimizable-for-BearPutSpread){.anchor}

### impl [Optimizable](../base/trait.Optimizable.html "trait optionstratlib::strategies::base::Optimizable"){.trait} for [BearPutSpread](struct.BearPutSpread.html "struct optionstratlib::strategies::bear_put_spread::BearPutSpread"){.struct} {#impl-optimizable-for-bearputspread .code-header}
:::

::::::::::::::::::::: impl-items
::: {#associatedtype.Strategy .section .associatedtype .trait-impl}
[Source](../../../src/optionstratlib/strategies/bear_put_spread.rs.html#508){.src
.rightside}[§](#associatedtype.Strategy){.anchor}

#### type [Strategy](../base/trait.Optimizable.html#associatedtype.Strategy){.associatedtype} = [BearPutSpread](struct.BearPutSpread.html "struct optionstratlib::strategies::bear_put_spread::BearPutSpread"){.struct} {#type-strategy-bearputspread .code-header}
:::

::: docblock
The type of strategy associated with this optimization.
:::

::: {#method.filter_combinations .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/bear_put_spread.rs.html#510-545){.src
.rightside}[§](#method.filter_combinations){.anchor}

#### fn [filter_combinations](../base/trait.Optimizable.html#method.filter_combinations){.fn}\<\'a\>( &\'a self, option_chain: &\'a [OptionChain](../../chains/chain/struct.OptionChain.html "struct optionstratlib::chains::chain::OptionChain"){.struct}, side: [FindOptimalSide](../utils/enum.FindOptimalSide.html "enum optionstratlib::strategies::utils::FindOptimalSide"){.enum}, ) -\> impl [Iterator](https://doc.rust-lang.org/1.86.0/core/iter/traits/iterator/trait.Iterator.html "trait core::iter::traits::iterator::Iterator"){.trait}\<Item = [OptionDataGroup](../../chains/utils/enum.OptionDataGroup.html "enum optionstratlib::chains::utils::OptionDataGroup"){.enum}\<\'a\>\> {#fn-filter_combinationsa-a-self-option_chain-a-optionchain-side-findoptimalside---impl-iteratoritem-optiondatagroupa .code-header}
:::

::: docblock
Filters and generates combinations of options data from the given
`OptionChain`. [Read
more](../base/trait.Optimizable.html#method.filter_combinations)
:::

::: {#method.find_optimal .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/bear_put_spread.rs.html#547-582){.src
.rightside}[§](#method.find_optimal){.anchor}

#### fn [find_optimal](../base/trait.Optimizable.html#method.find_optimal){.fn}( &mut self, option_chain: &[OptionChain](../../chains/chain/struct.OptionChain.html "struct optionstratlib::chains::chain::OptionChain"){.struct}, side: [FindOptimalSide](../utils/enum.FindOptimalSide.html "enum optionstratlib::strategies::utils::FindOptimalSide"){.enum}, criteria: [OptimizationCriteria](../utils/enum.OptimizationCriteria.html "enum optionstratlib::strategies::utils::OptimizationCriteria"){.enum}, ) {#fn-find_optimal-mut-self-option_chain-optionchain-side-findoptimalside-criteria-optimizationcriteria .code-header}
:::

::: docblock
Finds the optimal strategy based on the given criteria. The default
implementation panics. Specific strategies should override this method
to provide their own optimization logic. [Read
more](../base/trait.Optimizable.html#method.find_optimal)
:::

::: {#method.create_strategy .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/bear_put_spread.rs.html#584-608){.src
.rightside}[§](#method.create_strategy){.anchor}

#### fn [create_strategy](../base/trait.Optimizable.html#method.create_strategy){.fn}( &self, chain: &[OptionChain](../../chains/chain/struct.OptionChain.html "struct optionstratlib::chains::chain::OptionChain"){.struct}, legs: &[StrategyLegs](../../chains/enum.StrategyLegs.html "enum optionstratlib::chains::StrategyLegs"){.enum}\<\'\_\>, ) -\> Self::[Strategy](../base/trait.Optimizable.html#associatedtype.Strategy "type optionstratlib::strategies::base::Optimizable::Strategy"){.associatedtype} {#fn-create_strategy-self-chain-optionchain-legs-strategylegs_---selfstrategy .code-header}
:::

::: docblock
Creates a new strategy from the given `OptionChain` and `StrategyLegs`.
The default implementation panics. Specific strategies must override
this. [Read more](../base/trait.Optimizable.html#method.create_strategy)
:::

::: {#method.best_ratio .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/base.rs.html#760-762){.src
.rightside}[§](#method.best_ratio){.anchor}

#### fn [best_ratio](../base/trait.Optimizable.html#method.best_ratio){.fn}(&mut self, option_chain: &[OptionChain](../../chains/chain/struct.OptionChain.html "struct optionstratlib::chains::chain::OptionChain"){.struct}, side: [FindOptimalSide](../utils/enum.FindOptimalSide.html "enum optionstratlib::strategies::utils::FindOptimalSide"){.enum}) {#fn-best_ratiomut-self-option_chain-optionchain-side-findoptimalside .code-header}
:::

::: docblock
Finds the best ratio-based strategy within the given `OptionChain`.
[Read more](../base/trait.Optimizable.html#method.best_ratio)
:::

::: {#method.best_area .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/base.rs.html#769-771){.src
.rightside}[§](#method.best_area){.anchor}

#### fn [best_area](../base/trait.Optimizable.html#method.best_area){.fn}(&mut self, option_chain: &[OptionChain](../../chains/chain/struct.OptionChain.html "struct optionstratlib::chains::chain::OptionChain"){.struct}, side: [FindOptimalSide](../utils/enum.FindOptimalSide.html "enum optionstratlib::strategies::utils::FindOptimalSide"){.enum}) {#fn-best_areamut-self-option_chain-optionchain-side-findoptimalside .code-header}
:::

::: docblock
Finds the best area-based strategy within the given `OptionChain`. [Read
more](../base/trait.Optimizable.html#method.best_area)
:::

::: {#method.is_valid_short_option .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/base.rs.html#831-833){.src
.rightside}[§](#method.is_valid_short_option){.anchor}

#### fn [is_valid_short_option](../base/trait.Optimizable.html#method.is_valid_short_option){.fn}( &self, option: &[OptionData](../../chains/struct.OptionData.html "struct optionstratlib::chains::OptionData"){.struct}, side: &[FindOptimalSide](../utils/enum.FindOptimalSide.html "enum optionstratlib::strategies::utils::FindOptimalSide"){.enum}, ) -\> [bool](https://doc.rust-lang.org/1.86.0/std/primitive.bool.html){.primitive} {#fn-is_valid_short_option-self-option-optiondata-side-findoptimalside---bool .code-header}
:::

::: docblock
Checks if a short option is valid. The default implementation defers to
`is_valid_long_option`. [Read
more](../base/trait.Optimizable.html#method.is_valid_short_option)
:::

::: {#method.is_valid_long_option .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/base.rs.html#840-860){.src
.rightside}[§](#method.is_valid_long_option){.anchor}

#### fn [is_valid_long_option](../base/trait.Optimizable.html#method.is_valid_long_option){.fn}( &self, option: &[OptionData](../../chains/struct.OptionData.html "struct optionstratlib::chains::OptionData"){.struct}, side: &[FindOptimalSide](../utils/enum.FindOptimalSide.html "enum optionstratlib::strategies::utils::FindOptimalSide"){.enum}, ) -\> [bool](https://doc.rust-lang.org/1.86.0/std/primitive.bool.html){.primitive} {#fn-is_valid_long_option-self-option-optiondata-side-findoptimalside---bool .code-header}
:::

::: docblock
Checks if a long option is valid based on the given criteria. [Read
more](../base/trait.Optimizable.html#method.is_valid_long_option)
:::

::: {#method.are_valid_prices .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/base.rs.html#867-875){.src
.rightside}[§](#method.are_valid_prices){.anchor}

#### fn [are_valid_prices](../base/trait.Optimizable.html#method.are_valid_prices){.fn}(&self, legs: &[StrategyLegs](../../chains/enum.StrategyLegs.html "enum optionstratlib::chains::StrategyLegs"){.enum}\<\'\_\>) -\> [bool](https://doc.rust-lang.org/1.86.0/std/primitive.bool.html){.primitive} {#fn-are_valid_pricesself-legs-strategylegs_---bool .code-header}
:::

::: docblock
Checks if the prices in the given `StrategyLegs` are valid. Assumes the
strategy consists of one long call and one short call by default. [Read
more](../base/trait.Optimizable.html#method.are_valid_prices)
:::
:::::::::::::::::::::

::: {#impl-PnLCalculator-for-BearPutSpread .section .impl}
[Source](../../../src/optionstratlib/strategies/bear_put_spread.rs.html#773-799){.src
.rightside}[§](#impl-PnLCalculator-for-BearPutSpread){.anchor}

### impl [PnLCalculator](../../pnl/trait.PnLCalculator.html "trait optionstratlib::pnl::PnLCalculator"){.trait} for [BearPutSpread](struct.BearPutSpread.html "struct optionstratlib::strategies::bear_put_spread::BearPutSpread"){.struct} {#impl-pnlcalculator-for-bearputspread .code-header}
:::

::::::::: impl-items
::: {#method.calculate_pnl .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/bear_put_spread.rs.html#774-786){.src
.rightside}[§](#method.calculate_pnl){.anchor}

#### fn [calculate_pnl](../../pnl/trait.PnLCalculator.html#tymethod.calculate_pnl){.fn}( &self, market_price: &[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, expiration_date: [ExpirationDate](../../model/types/enum.ExpirationDate.html "enum optionstratlib::model::types::ExpirationDate"){.enum}, implied_volatility: &[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, ) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[PnL](../../pnl/utils/struct.PnL.html "struct optionstratlib::pnl::utils::PnL"){.struct}, [Box](https://doc.rust-lang.org/1.86.0/alloc/boxed/struct.Box.html "struct alloc::boxed::Box"){.struct}\<dyn [Error](https://doc.rust-lang.org/1.86.0/core/error/trait.Error.html "trait core::error::Error"){.trait}\>\> {#fn-calculate_pnl-self-market_price-positive-expiration_date-expirationdate-implied_volatility-positive---resultpnl-boxdyn-error .code-header}
:::

::: docblock
Calculates the current PnL based on market conditions. [Read
more](../../pnl/trait.PnLCalculator.html#tymethod.calculate_pnl)
:::

::: {#method.calculate_pnl_at_expiration .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/bear_put_spread.rs.html#788-798){.src
.rightside}[§](#method.calculate_pnl_at_expiration){.anchor}

#### fn [calculate_pnl_at_expiration](../../pnl/trait.PnLCalculator.html#tymethod.calculate_pnl_at_expiration){.fn}( &self, underlying_price: &[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, ) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[PnL](../../pnl/utils/struct.PnL.html "struct optionstratlib::pnl::utils::PnL"){.struct}, [Box](https://doc.rust-lang.org/1.86.0/alloc/boxed/struct.Box.html "struct alloc::boxed::Box"){.struct}\<dyn [Error](https://doc.rust-lang.org/1.86.0/core/error/trait.Error.html "trait core::error::Error"){.trait}\>\> {#fn-calculate_pnl_at_expiration-self-underlying_price-positive---resultpnl-boxdyn-error .code-header}
:::

::: docblock
Calculates the PnL at the expiration of the instrument. [Read
more](../../pnl/trait.PnLCalculator.html#tymethod.calculate_pnl_at_expiration)
:::

::: {#method.adjustments_pnl .section .method .trait-impl}
[Source](../../../src/optionstratlib/pnl/traits.rs.html#70-72){.src
.rightside}[§](#method.adjustments_pnl){.anchor}

#### fn [adjustments_pnl](../../pnl/trait.PnLCalculator.html#method.adjustments_pnl){.fn}( &self, \_adjustments: &[DeltaAdjustment](../delta_neutral/enum.DeltaAdjustment.html "enum optionstratlib::strategies::delta_neutral::DeltaAdjustment"){.enum}, ) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[PnL](../../pnl/utils/struct.PnL.html "struct optionstratlib::pnl::utils::PnL"){.struct}, [Box](https://doc.rust-lang.org/1.86.0/alloc/boxed/struct.Box.html "struct alloc::boxed::Box"){.struct}\<dyn [Error](https://doc.rust-lang.org/1.86.0/core/error/trait.Error.html "trait core::error::Error"){.trait}\>\> {#fn-adjustments_pnl-self-_adjustments-deltaadjustment---resultpnl-boxdyn-error .code-header}
:::

::: docblock
Calculates the Profit and Loss (PnL) for a series of delta adjustments
in a trading strategy. [Read
more](../../pnl/trait.PnLCalculator.html#method.adjustments_pnl)
:::
:::::::::

::: {#impl-Positionable-for-BearPutSpread .section .impl}
[Source](../../../src/optionstratlib/strategies/bear_put_spread.rs.html#322-426){.src
.rightside}[§](#impl-Positionable-for-BearPutSpread){.anchor}

### impl [Positionable](../base/trait.Positionable.html "trait optionstratlib::strategies::base::Positionable"){.trait} for [BearPutSpread](struct.BearPutSpread.html "struct optionstratlib::strategies::bear_put_spread::BearPutSpread"){.struct} {#impl-positionable-for-bearputspread .code-header}
:::

::::::::::: impl-items
::: {#method.get_position .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/bear_put_spread.rs.html#350-376){.src
.rightside}[§](#method.get_position){.anchor}

#### fn [get_position](../base/trait.Positionable.html#method.get_position){.fn}( &mut self, option_style: &[OptionStyle](../../model/types/enum.OptionStyle.html "enum optionstratlib::model::types::OptionStyle"){.enum}, side: &[Side](../../model/types/enum.Side.html "enum optionstratlib::model::types::Side"){.enum}, strike: &[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, ) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Vec](https://doc.rust-lang.org/1.86.0/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<&mut [Position](../../model/position/struct.Position.html "struct optionstratlib::model::position::Position"){.struct}\>, [PositionError](../../error/position/enum.PositionError.html "enum optionstratlib::error::position::PositionError"){.enum}\> {#fn-get_position-mut-self-option_style-optionstyle-side-side-strike-positive---resultvecmut-position-positionerror .code-header}
:::

::: docblock
Gets mutable positions matching the specified criteria from the
strategy.

##### [§](#arguments){.doc-anchor}Arguments

- `option_style` - The style of the option (Put/Call)
- `side` - The side of the position (Long/Short)
- `strike` - The strike price of the option

##### [§](#returns-1){.doc-anchor}Returns

- `Ok(Vec<&mut Position>)` - A vector containing mutable references to
  matching positions
- `Err(PositionError)` - If there was an error retrieving positions
:::

::: {#method.modify_position .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/bear_put_spread.rs.html#386-425){.src
.rightside}[§](#method.modify_position){.anchor}

#### fn [modify_position](../base/trait.Positionable.html#method.modify_position){.fn}(&mut self, position: &[Position](../../model/position/struct.Position.html "struct optionstratlib::model::position::Position"){.struct}) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[()](https://doc.rust-lang.org/1.86.0/std/primitive.unit.html){.primitive}, [PositionError](../../error/position/enum.PositionError.html "enum optionstratlib::error::position::PositionError"){.enum}\> {#fn-modify_positionmut-self-position-position---result-positionerror .code-header}
:::

::: docblock
Modifies an existing position in the strategy.

##### [§](#arguments-1){.doc-anchor}Arguments

- `position` - The new position data to update

##### [§](#returns-2){.doc-anchor}Returns

- `Ok(())` if position was successfully modified
- `Err(PositionError)` if position was not found or validation failed
:::

::: {#method.add_position .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/bear_put_spread.rs.html#323-334){.src
.rightside}[§](#method.add_position){.anchor}

#### fn [add_position](../base/trait.Positionable.html#method.add_position){.fn}(&mut self, position: &[Position](../../model/position/struct.Position.html "struct optionstratlib::model::position::Position"){.struct}) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[()](https://doc.rust-lang.org/1.86.0/std/primitive.unit.html){.primitive}, [PositionError](../../error/position/enum.PositionError.html "enum optionstratlib::error::position::PositionError"){.enum}\> {#fn-add_positionmut-self-position-position---result-positionerror .code-header}
:::

::: docblock
Adds a position to the strategy. [Read
more](../base/trait.Positionable.html#method.add_position)
:::

::: {#method.get_positions .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/bear_put_spread.rs.html#336-338){.src
.rightside}[§](#method.get_positions){.anchor}

#### fn [get_positions](../base/trait.Positionable.html#method.get_positions){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Vec](https://doc.rust-lang.org/1.86.0/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<&[Position](../../model/position/struct.Position.html "struct optionstratlib::model::position::Position"){.struct}\>, [PositionError](../../error/position/enum.PositionError.html "enum optionstratlib::error::position::PositionError"){.enum}\> {#fn-get_positionsself---resultvecposition-positionerror .code-header}
:::

::: docblock
Retrieves all positions held by the strategy. [Read
more](../base/trait.Positionable.html#method.get_positions)
:::
:::::::::::

::: {#impl-ProbabilityAnalysis-for-BearPutSpread .section .impl}
[Source](../../../src/optionstratlib/strategies/bear_put_spread.rs.html#699-763){.src
.rightside}[§](#impl-ProbabilityAnalysis-for-BearPutSpread){.anchor}

### impl [ProbabilityAnalysis](../probabilities/trait.ProbabilityAnalysis.html "trait optionstratlib::strategies::probabilities::ProbabilityAnalysis"){.trait} for [BearPutSpread](struct.BearPutSpread.html "struct optionstratlib::strategies::bear_put_spread::BearPutSpread"){.struct} {#impl-probabilityanalysis-for-bearputspread .code-header}
:::

::::::::::::::::::::: impl-items
::: {#method.get_expiration .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/bear_put_spread.rs.html#700-702){.src
.rightside}[§](#method.get_expiration){.anchor}

#### fn [get_expiration](../probabilities/trait.ProbabilityAnalysis.html#tymethod.get_expiration){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[ExpirationDate](../../model/types/enum.ExpirationDate.html "enum optionstratlib::model::types::ExpirationDate"){.enum}, [ProbabilityError](../../error/probability/enum.ProbabilityError.html "enum optionstratlib::error::probability::ProbabilityError"){.enum}\> {#fn-get_expirationself---resultexpirationdate-probabilityerror .code-header}
:::

::: docblock
Get the expiration date of the option strategy [Read
more](../probabilities/trait.ProbabilityAnalysis.html#tymethod.get_expiration)
:::

::: {#method.get_risk_free_rate .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/bear_put_spread.rs.html#704-706){.src
.rightside}[§](#method.get_risk_free_rate){.anchor}

#### fn [get_risk_free_rate](../probabilities/trait.ProbabilityAnalysis.html#tymethod.get_risk_free_rate){.fn}(&self) -\> [Option](https://doc.rust-lang.org/1.86.0/core/option/enum.Option.html "enum core::option::Option"){.enum}\<Decimal\> {#fn-get_risk_free_rateself---optiondecimal .code-header}
:::

::: docblock
Get the current risk-free interest rate [Read
more](../probabilities/trait.ProbabilityAnalysis.html#tymethod.get_risk_free_rate)
:::

::: {#method.get_profit_ranges .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/bear_put_spread.rs.html#708-734){.src
.rightside}[§](#method.get_profit_ranges){.anchor}

#### fn [get_profit_ranges](../probabilities/trait.ProbabilityAnalysis.html#tymethod.get_profit_ranges){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Vec](https://doc.rust-lang.org/1.86.0/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<[ProfitLossRange](../../model/struct.ProfitLossRange.html "struct optionstratlib::model::ProfitLossRange"){.struct}\>, [ProbabilityError](../../error/probability/enum.ProbabilityError.html "enum optionstratlib::error::probability::ProbabilityError"){.enum}\> {#fn-get_profit_rangesself---resultvecprofitlossrange-probabilityerror .code-header}
:::

::: docblock
Get the price ranges that would result in a profit [Read
more](../probabilities/trait.ProbabilityAnalysis.html#tymethod.get_profit_ranges)
:::

::: {#method.get_loss_ranges .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/bear_put_spread.rs.html#736-762){.src
.rightside}[§](#method.get_loss_ranges){.anchor}

#### fn [get_loss_ranges](../probabilities/trait.ProbabilityAnalysis.html#tymethod.get_loss_ranges){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Vec](https://doc.rust-lang.org/1.86.0/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<[ProfitLossRange](../../model/struct.ProfitLossRange.html "struct optionstratlib::model::ProfitLossRange"){.struct}\>, [ProbabilityError](../../error/probability/enum.ProbabilityError.html "enum optionstratlib::error::probability::ProbabilityError"){.enum}\> {#fn-get_loss_rangesself---resultvecprofitlossrange-probabilityerror .code-header}
:::

::: docblock
Get Profit/Loss Ranges [Read
more](../probabilities/trait.ProbabilityAnalysis.html#tymethod.get_loss_ranges)
:::

::: {#method.analyze_probabilities .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/probabilities/core.rs.html#65-102){.src
.rightside}[§](#method.analyze_probabilities){.anchor}

#### fn [analyze_probabilities](../probabilities/trait.ProbabilityAnalysis.html#method.analyze_probabilities){.fn}( &self, volatility_adj: [Option](https://doc.rust-lang.org/1.86.0/core/option/enum.Option.html "enum core::option::Option"){.enum}\<[VolatilityAdjustment](../probabilities/struct.VolatilityAdjustment.html "struct optionstratlib::strategies::probabilities::VolatilityAdjustment"){.struct}\>, trend: [Option](https://doc.rust-lang.org/1.86.0/core/option/enum.Option.html "enum core::option::Option"){.enum}\<[PriceTrend](../probabilities/struct.PriceTrend.html "struct optionstratlib::strategies::probabilities::PriceTrend"){.struct}\>, ) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[StrategyProbabilityAnalysis](../probabilities/struct.StrategyProbabilityAnalysis.html "struct optionstratlib::strategies::probabilities::StrategyProbabilityAnalysis"){.struct}, [ProbabilityError](../../error/probability/enum.ProbabilityError.html "enum optionstratlib::error::probability::ProbabilityError"){.enum}\> {#fn-analyze_probabilities-self-volatility_adj-optionvolatilityadjustment-trend-optionpricetrend---resultstrategyprobabilityanalysis-probabilityerror .code-header}
:::

::: docblock
Calculate probability analysis for a strategy [Read
more](../probabilities/trait.ProbabilityAnalysis.html#method.analyze_probabilities)
:::

::: {#method.expected_value .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/probabilities/core.rs.html#129-191){.src
.rightside}[§](#method.expected_value){.anchor}

#### fn [expected_value](../probabilities/trait.ProbabilityAnalysis.html#method.expected_value){.fn}( &self, volatility_adj: [Option](https://doc.rust-lang.org/1.86.0/core/option/enum.Option.html "enum core::option::Option"){.enum}\<[VolatilityAdjustment](../probabilities/struct.VolatilityAdjustment.html "struct optionstratlib::strategies::probabilities::VolatilityAdjustment"){.struct}\>, trend: [Option](https://doc.rust-lang.org/1.86.0/core/option/enum.Option.html "enum core::option::Option"){.enum}\<[PriceTrend](../probabilities/struct.PriceTrend.html "struct optionstratlib::strategies::probabilities::PriceTrend"){.struct}\>, ) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, [ProbabilityError](../../error/probability/enum.ProbabilityError.html "enum optionstratlib::error::probability::ProbabilityError"){.enum}\> {#fn-expected_value-self-volatility_adj-optionvolatilityadjustment-trend-optionpricetrend---resultpositive-probabilityerror .code-header}
:::

::: docblock
This function calculates the expected value of an option strategy based
on an underlying price, volatility adjustments, and price trends. [Read
more](../probabilities/trait.ProbabilityAnalysis.html#method.expected_value)
:::

::: {#method.probability_of_profit .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/probabilities/core.rs.html#206-224){.src
.rightside}[§](#method.probability_of_profit){.anchor}

#### fn [probability_of_profit](../probabilities/trait.ProbabilityAnalysis.html#method.probability_of_profit){.fn}( &self, volatility_adj: [Option](https://doc.rust-lang.org/1.86.0/core/option/enum.Option.html "enum core::option::Option"){.enum}\<[VolatilityAdjustment](../probabilities/struct.VolatilityAdjustment.html "struct optionstratlib::strategies::probabilities::VolatilityAdjustment"){.struct}\>, trend: [Option](https://doc.rust-lang.org/1.86.0/core/option/enum.Option.html "enum core::option::Option"){.enum}\<[PriceTrend](../probabilities/struct.PriceTrend.html "struct optionstratlib::strategies::probabilities::PriceTrend"){.struct}\>, ) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, [ProbabilityError](../../error/probability/enum.ProbabilityError.html "enum optionstratlib::error::probability::ProbabilityError"){.enum}\> {#fn-probability_of_profit-self-volatility_adj-optionvolatilityadjustment-trend-optionpricetrend---resultpositive-probabilityerror .code-header}
:::

::: docblock
Calculate probability of profit [Read
more](../probabilities/trait.ProbabilityAnalysis.html#method.probability_of_profit)
:::

::: {#method.probability_of_loss .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/probabilities/core.rs.html#239-257){.src
.rightside}[§](#method.probability_of_loss){.anchor}

#### fn [probability_of_loss](../probabilities/trait.ProbabilityAnalysis.html#method.probability_of_loss){.fn}( &self, volatility_adj: [Option](https://doc.rust-lang.org/1.86.0/core/option/enum.Option.html "enum core::option::Option"){.enum}\<[VolatilityAdjustment](../probabilities/struct.VolatilityAdjustment.html "struct optionstratlib::strategies::probabilities::VolatilityAdjustment"){.struct}\>, trend: [Option](https://doc.rust-lang.org/1.86.0/core/option/enum.Option.html "enum core::option::Option"){.enum}\<[PriceTrend](../probabilities/struct.PriceTrend.html "struct optionstratlib::strategies::probabilities::PriceTrend"){.struct}\>, ) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, [ProbabilityError](../../error/probability/enum.ProbabilityError.html "enum optionstratlib::error::probability::ProbabilityError"){.enum}\> {#fn-probability_of_loss-self-volatility_adj-optionvolatilityadjustment-trend-optionpricetrend---resultpositive-probabilityerror .code-header}
:::

::: docblock
Calculate probability of loss [Read
more](../probabilities/trait.ProbabilityAnalysis.html#method.probability_of_loss)
:::

::: {#method.calculate_extreme_probabilities .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/probabilities/core.rs.html#273-314){.src
.rightside}[§](#method.calculate_extreme_probabilities){.anchor}

#### fn [calculate_extreme_probabilities](../probabilities/trait.ProbabilityAnalysis.html#method.calculate_extreme_probabilities){.fn}( &self, volatility_adj: [Option](https://doc.rust-lang.org/1.86.0/core/option/enum.Option.html "enum core::option::Option"){.enum}\<[VolatilityAdjustment](../probabilities/struct.VolatilityAdjustment.html "struct optionstratlib::strategies::probabilities::VolatilityAdjustment"){.struct}\>, trend: [Option](https://doc.rust-lang.org/1.86.0/core/option/enum.Option.html "enum core::option::Option"){.enum}\<[PriceTrend](../probabilities/struct.PriceTrend.html "struct optionstratlib::strategies::probabilities::PriceTrend"){.struct}\>, ) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<([Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, [Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}), [ProbabilityError](../../error/probability/enum.ProbabilityError.html "enum optionstratlib::error::probability::ProbabilityError"){.enum}\> {#fn-calculate_extreme_probabilities-self-volatility_adj-optionvolatilityadjustment-trend-optionpricetrend---resultpositive-positive-probabilityerror .code-header}
:::

::: docblock
Calculate extreme probabilities (max profit and max loss) [Read
more](../probabilities/trait.ProbabilityAnalysis.html#method.calculate_extreme_probabilities)
:::
:::::::::::::::::::::

::: {#impl-Profit-for-BearPutSpread .section .impl}
[Source](../../../src/optionstratlib/strategies/bear_put_spread.rs.html#611-616){.src
.rightside}[§](#impl-Profit-for-BearPutSpread){.anchor}

### impl [Profit](../../pricing/trait.Profit.html "trait optionstratlib::pricing::Profit"){.trait} for [BearPutSpread](struct.BearPutSpread.html "struct optionstratlib::strategies::bear_put_spread::BearPutSpread"){.struct} {#impl-profit-for-bearputspread .code-header}
:::

::::::: impl-items
::: {#method.calculate_profit_at .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/bear_put_spread.rs.html#612-615){.src
.rightside}[§](#method.calculate_profit_at){.anchor}

#### fn [calculate_profit_at](../../pricing/trait.Profit.html#tymethod.calculate_profit_at){.fn}( &self, price: [Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, ) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<Decimal, [Box](https://doc.rust-lang.org/1.86.0/alloc/boxed/struct.Box.html "struct alloc::boxed::Box"){.struct}\<dyn [Error](https://doc.rust-lang.org/1.86.0/core/error/trait.Error.html "trait core::error::Error"){.trait}\>\> {#fn-calculate_profit_at-self-price-positive---resultdecimal-boxdyn-error .code-header}
:::

::: docblock
Calculates the profit at a specified price. [Read
more](../../pricing/trait.Profit.html#tymethod.calculate_profit_at)
:::

::: {#method.get_point_at_price .section .method .trait-impl}
[Source](../../../src/optionstratlib/pricing/payoff.rs.html#222-238){.src
.rightside}[§](#method.get_point_at_price){.anchor}

#### fn [get_point_at_price](../../pricing/trait.Profit.html#method.get_point_at_price){.fn}(&self, price: [Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}) -\> ChartPoint\<([f64](https://doc.rust-lang.org/1.86.0/std/primitive.f64.html){.primitive}, [f64](https://doc.rust-lang.org/1.86.0/std/primitive.f64.html){.primitive})\> {#fn-get_point_at_priceself-price-positive---chartpointf64-f64 .code-header}
:::

::: docblock
Creates a chart point representation of the profit at the given price.
[Read more](../../pricing/trait.Profit.html#method.get_point_at_price)
:::
:::::::

::: {#impl-Strategable-for-BearPutSpread .section .impl}
[Source](../../../src/optionstratlib/strategies/bear_put_spread.rs.html#428-436){.src
.rightside}[§](#impl-Strategable-for-BearPutSpread){.anchor}

### impl [Strategable](../base/trait.Strategable.html "trait optionstratlib::strategies::base::Strategable"){.trait} for [BearPutSpread](struct.BearPutSpread.html "struct optionstratlib::strategies::bear_put_spread::BearPutSpread"){.struct} {#impl-strategable-for-bearputspread .code-header}
:::

::::::::: impl-items
::: {#method.info .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/bear_put_spread.rs.html#429-435){.src
.rightside}[§](#method.info){.anchor}

#### fn [info](../base/trait.Strategable.html#method.info){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[StrategyBasics](../base/struct.StrategyBasics.html "struct optionstratlib::strategies::base::StrategyBasics"){.struct}, [StrategyError](../../error/strategies/enum.StrategyError.html "enum optionstratlib::error::strategies::StrategyError"){.enum}\> {#fn-infoself---resultstrategybasics-strategyerror .code-header}
:::

::: docblock
Returns basic information about the strategy, such as its name, type,
and description. [Read more](../base/trait.Strategable.html#method.info)
:::

::: {#method.type_name .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/base.rs.html#82-89){.src
.rightside}[§](#method.type_name){.anchor}

#### fn [type_name](../base/trait.Strategable.html#method.type_name){.fn}(&self) -\> [StrategyType](../base/enum.StrategyType.html "enum optionstratlib::strategies::base::StrategyType"){.enum} {#fn-type_nameself---strategytype .code-header}
:::

::: docblock
Returns the type of the strategy. [Read
more](../base/trait.Strategable.html#method.type_name)
:::

::: {#method.name .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/base.rs.html#100-107){.src
.rightside}[§](#method.name){.anchor}

#### fn [name](../base/trait.Strategable.html#method.name){.fn}(&self) -\> [String](https://doc.rust-lang.org/1.86.0/alloc/string/struct.String.html "struct alloc::string::String"){.struct} {#fn-nameself---string .code-header}
:::

::: docblock
Returns the name of the strategy. [Read
more](../base/trait.Strategable.html#method.name)
:::
:::::::::

::: {#impl-Strategies-for-BearPutSpread .section .impl}
[Source](../../../src/optionstratlib/strategies/bear_put_spread.rs.html#438-484){.src
.rightside}[§](#impl-Strategies-for-BearPutSpread){.anchor}

### impl [Strategies](../base/trait.Strategies.html "trait optionstratlib::strategies::base::Strategies"){.trait} for [BearPutSpread](struct.BearPutSpread.html "struct optionstratlib::strategies::bear_put_spread::BearPutSpread"){.struct} {#impl-strategies-for-bearputspread .code-header}
:::

::::::::::::::::::::::::::::::::::::::::::: impl-items
::: {#method.get_underlying_price .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/bear_put_spread.rs.html#439-441){.src
.rightside}[§](#method.get_underlying_price){.anchor}

#### fn [get_underlying_price](../base/trait.Strategies.html#method.get_underlying_price){.fn}(&self) -\> [Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#fn-get_underlying_priceself---positive .code-header}
:::

::: docblock
Retrieves the underlying asset price. The default implementation panics
with a message indicating that the underlying price is not applicable
for the strategy. [Read
more](../base/trait.Strategies.html#method.get_underlying_price)
:::

::: {#method.max_profit .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/bear_put_spread.rs.html#443-454){.src
.rightside}[§](#method.max_profit){.anchor}

#### fn [max_profit](../base/trait.Strategies.html#method.max_profit){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, [StrategyError](../../error/strategies/enum.StrategyError.html "enum optionstratlib::error::strategies::StrategyError"){.enum}\> {#fn-max_profitself---resultpositive-strategyerror .code-header}
:::

::: docblock
Calculates the maximum possible profit for the strategy. The default
implementation returns an error indicating that the operation is not
supported. [Read more](../base/trait.Strategies.html#method.max_profit)
:::

::: {#method.max_loss .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/bear_put_spread.rs.html#456-467){.src
.rightside}[§](#method.max_loss){.anchor}

#### fn [max_loss](../base/trait.Strategies.html#method.max_loss){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, [StrategyError](../../error/strategies/enum.StrategyError.html "enum optionstratlib::error::strategies::StrategyError"){.enum}\> {#fn-max_lossself---resultpositive-strategyerror .code-header}
:::

::: docblock
Calculates the maximum possible loss for the strategy. The default
implementation returns an error indicating that the operation is not
supported. [Read more](../base/trait.Strategies.html#method.max_loss)
:::

::: {#method.profit_area .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/bear_put_spread.rs.html#469-473){.src
.rightside}[§](#method.profit_area){.anchor}

#### fn [profit_area](../base/trait.Strategies.html#method.profit_area){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<Decimal, [StrategyError](../../error/strategies/enum.StrategyError.html "enum optionstratlib::error::strategies::StrategyError"){.enum}\> {#fn-profit_areaself---resultdecimal-strategyerror .code-header}
:::

::: docblock
Calculates the profit area for the strategy. The default implementation
returns an error indicating that the operation is not supported. [Read
more](../base/trait.Strategies.html#method.profit_area)
:::

::: {#method.profit_ratio .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/bear_put_spread.rs.html#475-483){.src
.rightside}[§](#method.profit_ratio){.anchor}

#### fn [profit_ratio](../base/trait.Strategies.html#method.profit_ratio){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<Decimal, [StrategyError](../../error/strategies/enum.StrategyError.html "enum optionstratlib::error::strategies::StrategyError"){.enum}\> {#fn-profit_ratioself---resultdecimal-strategyerror .code-header}
:::

::: docblock
Calculates the profit ratio for the strategy. The default implementation
returns an error indicating that the operation is not supported. [Read
more](../base/trait.Strategies.html#method.profit_ratio)
:::

::: {#method.set_underlying_price .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/base.rs.html#376-378){.src
.rightside}[§](#method.set_underlying_price){.anchor}

#### fn [set_underlying_price](../base/trait.Strategies.html#method.set_underlying_price){.fn}( &mut self, \_price: &[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, ) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[()](https://doc.rust-lang.org/1.86.0/std/primitive.unit.html){.primitive}, [StrategyError](../../error/strategies/enum.StrategyError.html "enum optionstratlib::error::strategies::StrategyError"){.enum}\> {#fn-set_underlying_price-mut-self-_price-positive---result-strategyerror .code-header}
:::

::: docblock
Sets the underlying price for a strategy. [Read
more](../base/trait.Strategies.html#method.set_underlying_price)
:::

::: {#method.volume .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/base.rs.html#393-395){.src
.rightside}[§](#method.volume){.anchor}

#### fn [volume](../base/trait.Strategies.html#method.volume){.fn}(&mut self) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, [StrategyError](../../error/strategies/enum.StrategyError.html "enum optionstratlib::error::strategies::StrategyError"){.enum}\> {#fn-volumemut-self---resultpositive-strategyerror .code-header}
:::

::: docblock
Returns the volume for this strategy. [Read
more](../base/trait.Strategies.html#method.volume)
:::

::: {#method.max_profit_iter .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/base.rs.html#416-418){.src
.rightside}[§](#method.max_profit_iter){.anchor}

#### fn [max_profit_iter](../base/trait.Strategies.html#method.max_profit_iter){.fn}(&mut self) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, [StrategyError](../../error/strategies/enum.StrategyError.html "enum optionstratlib::error::strategies::StrategyError"){.enum}\> {#fn-max_profit_itermut-self---resultpositive-strategyerror .code-header}
:::

::: docblock
Calculates the maximum possible profit for the strategy, potentially
using an iterative approach. Defaults to calling `max_profit`. [Read
more](../base/trait.Strategies.html#method.max_profit_iter)
:::

::: {#method.max_loss_iter .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/base.rs.html#439-441){.src
.rightside}[§](#method.max_loss_iter){.anchor}

#### fn [max_loss_iter](../base/trait.Strategies.html#method.max_loss_iter){.fn}(&mut self) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, [StrategyError](../../error/strategies/enum.StrategyError.html "enum optionstratlib::error::strategies::StrategyError"){.enum}\> {#fn-max_loss_itermut-self---resultpositive-strategyerror .code-header}
:::

::: docblock
Calculates the maximum possible loss for the strategy, potentially using
an iterative approach. Defaults to calling `max_loss`. [Read
more](../base/trait.Strategies.html#method.max_loss_iter)
:::

::: {#method.total_cost .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/base.rs.html#448-455){.src
.rightside}[§](#method.total_cost){.anchor}

#### fn [total_cost](../base/trait.Strategies.html#method.total_cost){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, [PositionError](../../error/position/enum.PositionError.html "enum optionstratlib::error::position::PositionError"){.enum}\> {#fn-total_costself---resultpositive-positionerror .code-header}
:::

::: docblock
Calculates the total cost of the strategy, which is the sum of the
absolute cost of all positions. [Read
more](../base/trait.Strategies.html#method.total_cost)
:::

::: {#method.net_cost .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/base.rs.html#463-470){.src
.rightside}[§](#method.net_cost){.anchor}

#### fn [net_cost](../base/trait.Strategies.html#method.net_cost){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<Decimal, [PositionError](../../error/position/enum.PositionError.html "enum optionstratlib::error::position::PositionError"){.enum}\> {#fn-net_costself---resultdecimal-positionerror .code-header}
:::

::: docblock
Calculates the net cost of the strategy, which is the sum of the costs
of all positions, considering premiums paid and received. [Read
more](../base/trait.Strategies.html#method.net_cost)
:::

::: {#method.net_premium_received .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/base.rs.html#478-494){.src
.rightside}[§](#method.net_premium_received){.anchor}

#### fn [net_premium_received](../base/trait.Strategies.html#method.net_premium_received){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, [StrategyError](../../error/strategies/enum.StrategyError.html "enum optionstratlib::error::strategies::StrategyError"){.enum}\> {#fn-net_premium_receivedself---resultpositive-strategyerror .code-header}
:::

::: docblock
Calculates the net premium received for the strategy. This is the total
premium received from short positions minus the total premium paid for
long positions. If the result is negative, it returns zero. [Read
more](../base/trait.Strategies.html#method.net_premium_received)
:::

::: {#method.fees .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/base.rs.html#501-518){.src
.rightside}[§](#method.fees){.anchor}

#### fn [fees](../base/trait.Strategies.html#method.fees){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, [StrategyError](../../error/strategies/enum.StrategyError.html "enum optionstratlib::error::strategies::StrategyError"){.enum}\> {#fn-feesself---resultpositive-strategyerror .code-header}
:::

::: docblock
Calculates the total fees for the strategy by summing the fees of all
positions. [Read more](../base/trait.Strategies.html#method.fees)
:::

::: {#method.range_to_show .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/base.rs.html#554-578){.src
.rightside}[§](#method.range_to_show){.anchor}

#### fn [range_to_show](../base/trait.Strategies.html#method.range_to_show){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<([Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, [Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}), [StrategyError](../../error/strategies/enum.StrategyError.html "enum optionstratlib::error::strategies::StrategyError"){.enum}\> {#fn-range_to_showself---resultpositive-positive-strategyerror .code-header}
:::

::: docblock
Determines the price range to display for the strategy's profit/loss
graph. This range is calculated based on the break-even points, the
underlying price, and the maximum and minimum strike prices. The range
is expanded by applying `STRIKE_PRICE_LOWER_BOUND_MULTIPLIER` and
`STRIKE_PRICE_UPPER_BOUND_MULTIPLIER` to the minimum and maximum prices
respectively. [Read
more](../base/trait.Strategies.html#method.range_to_show)
:::

::: {#method.best_range_to_show .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/base.rs.html#585-588){.src
.rightside}[§](#method.best_range_to_show){.anchor}

#### fn [best_range_to_show](../base/trait.Strategies.html#method.best_range_to_show){.fn}( &self, step: [Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, ) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Vec](https://doc.rust-lang.org/1.86.0/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\>, [StrategyError](../../error/strategies/enum.StrategyError.html "enum optionstratlib::error::strategies::StrategyError"){.enum}\> {#fn-best_range_to_show-self-step-positive---resultvecpositive-strategyerror .code-header}
:::

::: docblock
Generates a vector of prices within the display range, using a specified
step. [Read
more](../base/trait.Strategies.html#method.best_range_to_show)
:::

::: {#method.strikes .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/base.rs.html#595-615){.src
.rightside}[§](#method.strikes){.anchor}

#### fn [strikes](../base/trait.Strategies.html#method.strikes){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Vec](https://doc.rust-lang.org/1.86.0/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\>, [StrategyError](../../error/strategies/enum.StrategyError.html "enum optionstratlib::error::strategies::StrategyError"){.enum}\> {#fn-strikesself---resultvecpositive-strategyerror .code-header}
:::

::: docblock
Returns a sorted vector of unique strike prices for all positions in the
strategy. [Read more](../base/trait.Strategies.html#method.strikes)
:::

::: {#method.max_min_strikes .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/base.rs.html#623-651){.src
.rightside}[§](#method.max_min_strikes){.anchor}

#### fn [max_min_strikes](../base/trait.Strategies.html#method.max_min_strikes){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<([Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, [Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}), [StrategyError](../../error/strategies/enum.StrategyError.html "enum optionstratlib::error::strategies::StrategyError"){.enum}\> {#fn-max_min_strikesself---resultpositive-positive-strategyerror .code-header}
:::

::: docblock
Returns the minimum and maximum strike prices from the positions in the
strategy. Considers underlying price when applicable, ensuring the
returned range includes it. [Read
more](../base/trait.Strategies.html#method.max_min_strikes)
:::

::: {#method.range_of_profit .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/base.rs.html#659-673){.src
.rightside}[§](#method.range_of_profit){.anchor}

#### fn [range_of_profit](../base/trait.Strategies.html#method.range_of_profit){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, [StrategyError](../../error/strategies/enum.StrategyError.html "enum optionstratlib::error::strategies::StrategyError"){.enum}\> {#fn-range_of_profitself---resultpositive-strategyerror .code-header}
:::

::: docblock
Calculates the range of prices where the strategy is profitable, based
on the break-even points. [Read
more](../base/trait.Strategies.html#method.range_of_profit)
:::

::: {#method.expiration_dates .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/base.rs.html#680-682){.src
.rightside}[§](#method.expiration_dates){.anchor}

#### fn [expiration_dates](../base/trait.Strategies.html#method.expiration_dates){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Vec](https://doc.rust-lang.org/1.86.0/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<[ExpirationDate](../../model/types/enum.ExpirationDate.html "enum optionstratlib::model::types::ExpirationDate"){.enum}\>, [StrategyError](../../error/strategies/enum.StrategyError.html "enum optionstratlib::error::strategies::StrategyError"){.enum}\> {#fn-expiration_datesself---resultvecexpirationdate-strategyerror .code-header}
:::

::: docblock
Returns a vector of expiration dates for the strategy. [Read
more](../base/trait.Strategies.html#method.expiration_dates)
:::

::: {#method.set_expiration_date .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/base.rs.html#691-696){.src
.rightside}[§](#method.set_expiration_date){.anchor}

#### fn [set_expiration_date](../base/trait.Strategies.html#method.set_expiration_date){.fn}( &mut self, \_expiration_date: [ExpirationDate](../../model/types/enum.ExpirationDate.html "enum optionstratlib::model::types::ExpirationDate"){.enum}, ) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[()](https://doc.rust-lang.org/1.86.0/std/primitive.unit.html){.primitive}, [StrategyError](../../error/strategies/enum.StrategyError.html "enum optionstratlib::error::strategies::StrategyError"){.enum}\> {#fn-set_expiration_date-mut-self-_expiration_date-expirationdate---result-strategyerror .code-header}
:::

::: docblock
Sets the expiration date for the strategy. [Read
more](../base/trait.Strategies.html#method.set_expiration_date)
:::
:::::::::::::::::::::::::::::::::::::::::::

::: {#impl-StrategyConstructor-for-BearPutSpread .section .impl}
[Source](../../../src/optionstratlib/strategies/bear_put_spread.rs.html#213-303){.src
.rightside}[§](#impl-StrategyConstructor-for-BearPutSpread){.anchor}

### impl [StrategyConstructor](../trait.StrategyConstructor.html "trait optionstratlib::strategies::StrategyConstructor"){.trait} for [BearPutSpread](struct.BearPutSpread.html "struct optionstratlib::strategies::bear_put_spread::BearPutSpread"){.struct} {#impl-strategyconstructor-for-bearputspread .code-header}
:::

::::: impl-items
::: {#method.get_strategy .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/bear_put_spread.rs.html#214-302){.src
.rightside}[§](#method.get_strategy){.anchor}

#### fn [get_strategy](../trait.StrategyConstructor.html#method.get_strategy){.fn}(vec_options: &\[[Position](../../model/position/struct.Position.html "struct optionstratlib::model::position::Position"){.struct}\]) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<Self, [StrategyError](../../error/strategies/enum.StrategyError.html "enum optionstratlib::error::strategies::StrategyError"){.enum}\> {#fn-get_strategyvec_options-position---resultself-strategyerror .code-header}
:::

::: docblock
Attempts to construct a strategy from a vector of option positions.
[Read more](../trait.StrategyConstructor.html#method.get_strategy)
:::
:::::

::: {#impl-Validable-for-BearPutSpread .section .impl}
[Source](../../../src/optionstratlib/strategies/bear_put_spread.rs.html#486-505){.src
.rightside}[§](#impl-Validable-for-BearPutSpread){.anchor}

### impl [Validable](../base/trait.Validable.html "trait optionstratlib::strategies::base::Validable"){.trait} for [BearPutSpread](struct.BearPutSpread.html "struct optionstratlib::strategies::bear_put_spread::BearPutSpread"){.struct} {#impl-validable-for-bearputspread .code-header}
:::

::::: impl-items
::: {#method.validate .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/bear_put_spread.rs.html#487-504){.src
.rightside}[§](#method.validate){.anchor}

#### fn [validate](../base/trait.Validable.html#method.validate){.fn}(&self) -\> [bool](https://doc.rust-lang.org/1.86.0/std/primitive.bool.html){.primitive} {#fn-validateself---bool .code-header}
:::

::: docblock
Validates the strategy. [Read
more](../base/trait.Validable.html#method.validate)
:::
:::::
:::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::

## Auto Trait Implementations[§](#synthetic-implementations){.anchor} {#synthetic-implementations .section-header}

::::::::: {#synthetic-implementations-list}
::: {#impl-Freeze-for-BearPutSpread .section .impl}
[§](#impl-Freeze-for-BearPutSpread){.anchor}

### impl [Freeze](https://doc.rust-lang.org/1.86.0/core/marker/trait.Freeze.html "trait core::marker::Freeze"){.trait} for [BearPutSpread](struct.BearPutSpread.html "struct optionstratlib::strategies::bear_put_spread::BearPutSpread"){.struct} {#impl-freeze-for-bearputspread .code-header}
:::

::: {#impl-RefUnwindSafe-for-BearPutSpread .section .impl}
[§](#impl-RefUnwindSafe-for-BearPutSpread){.anchor}

### impl [RefUnwindSafe](https://doc.rust-lang.org/1.86.0/core/panic/unwind_safe/trait.RefUnwindSafe.html "trait core::panic::unwind_safe::RefUnwindSafe"){.trait} for [BearPutSpread](struct.BearPutSpread.html "struct optionstratlib::strategies::bear_put_spread::BearPutSpread"){.struct} {#impl-refunwindsafe-for-bearputspread .code-header}
:::

::: {#impl-Send-for-BearPutSpread .section .impl}
[§](#impl-Send-for-BearPutSpread){.anchor}

### impl [Send](https://doc.rust-lang.org/1.86.0/core/marker/trait.Send.html "trait core::marker::Send"){.trait} for [BearPutSpread](struct.BearPutSpread.html "struct optionstratlib::strategies::bear_put_spread::BearPutSpread"){.struct} {#impl-send-for-bearputspread .code-header}
:::

::: {#impl-Sync-for-BearPutSpread .section .impl}
[§](#impl-Sync-for-BearPutSpread){.anchor}

### impl [Sync](https://doc.rust-lang.org/1.86.0/core/marker/trait.Sync.html "trait core::marker::Sync"){.trait} for [BearPutSpread](struct.BearPutSpread.html "struct optionstratlib::strategies::bear_put_spread::BearPutSpread"){.struct} {#impl-sync-for-bearputspread .code-header}
:::

::: {#impl-Unpin-for-BearPutSpread .section .impl}
[§](#impl-Unpin-for-BearPutSpread){.anchor}

### impl [Unpin](https://doc.rust-lang.org/1.86.0/core/marker/trait.Unpin.html "trait core::marker::Unpin"){.trait} for [BearPutSpread](struct.BearPutSpread.html "struct optionstratlib::strategies::bear_put_spread::BearPutSpread"){.struct} {#impl-unpin-for-bearputspread .code-header}
:::

::: {#impl-UnwindSafe-for-BearPutSpread .section .impl}
[§](#impl-UnwindSafe-for-BearPutSpread){.anchor}

### impl [UnwindSafe](https://doc.rust-lang.org/1.86.0/core/panic/unwind_safe/trait.UnwindSafe.html "trait core::panic::unwind_safe::UnwindSafe"){.trait} for [BearPutSpread](struct.BearPutSpread.html "struct optionstratlib::strategies::bear_put_spread::BearPutSpread"){.struct} {#impl-unwindsafe-for-bearputspread .code-header}
:::
:::::::::

## Blanket Implementations[§](#blanket-implementations){.anchor} {#blanket-implementations .section-header}

::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::: {#blanket-implementations-list}
:::: {#impl-Any-for-T .section .impl}
[Source](https://doc.rust-lang.org/1.86.0/src/core/any.rs.html#138){.src
.rightside}[§](#impl-Any-for-T){.anchor}

### impl\<T\> [Any](https://doc.rust-lang.org/1.86.0/core/any/trait.Any.html "trait core::any::Any"){.trait} for T {#implt-any-for-t .code-header}

::: where
where T: \'static +
?[Sized](https://doc.rust-lang.org/1.86.0/core/marker/trait.Sized.html "trait core::marker::Sized"){.trait},
:::
::::

::::: impl-items
::: {#method.type_id .section .method .trait-impl}
[Source](https://doc.rust-lang.org/1.86.0/src/core/any.rs.html#139){.src
.rightside}[§](#method.type_id){.anchor}

#### fn [type_id](https://doc.rust-lang.org/1.86.0/core/any/trait.Any.html#tymethod.type_id){.fn}(&self) -\> [TypeId](https://doc.rust-lang.org/1.86.0/core/any/struct.TypeId.html "struct core::any::TypeId"){.struct} {#fn-type_idself---typeid .code-header}
:::

::: docblock
Gets the `TypeId` of `self`. [Read
more](https://doc.rust-lang.org/1.86.0/core/any/trait.Any.html#tymethod.type_id)
:::
:::::

:::: {#impl-Borrow%3CT%3E-for-T .section .impl}
[Source](https://doc.rust-lang.org/1.86.0/src/core/borrow.rs.html#209){.src
.rightside}[§](#impl-Borrow%3CT%3E-for-T){.anchor}

### impl\<T\> [Borrow](https://doc.rust-lang.org/1.86.0/core/borrow/trait.Borrow.html "trait core::borrow::Borrow"){.trait}\<T\> for T {#implt-borrowt-for-t .code-header}

::: where
where T:
?[Sized](https://doc.rust-lang.org/1.86.0/core/marker/trait.Sized.html "trait core::marker::Sized"){.trait},
:::
::::

::::: impl-items
::: {#method.borrow .section .method .trait-impl}
[Source](https://doc.rust-lang.org/1.86.0/src/core/borrow.rs.html#211){.src
.rightside}[§](#method.borrow){.anchor}

#### fn [borrow](https://doc.rust-lang.org/1.86.0/core/borrow/trait.Borrow.html#tymethod.borrow){.fn}(&self) -\> [&T](https://doc.rust-lang.org/1.86.0/std/primitive.reference.html){.primitive} {#fn-borrowself---t .code-header}
:::

::: docblock
Immutably borrows from an owned value. [Read
more](https://doc.rust-lang.org/1.86.0/core/borrow/trait.Borrow.html#tymethod.borrow)
:::
:::::

:::: {#impl-BorrowMut%3CT%3E-for-T .section .impl}
[Source](https://doc.rust-lang.org/1.86.0/src/core/borrow.rs.html#217){.src
.rightside}[§](#impl-BorrowMut%3CT%3E-for-T){.anchor}

### impl\<T\> [BorrowMut](https://doc.rust-lang.org/1.86.0/core/borrow/trait.BorrowMut.html "trait core::borrow::BorrowMut"){.trait}\<T\> for T {#implt-borrowmutt-for-t .code-header}

::: where
where T:
?[Sized](https://doc.rust-lang.org/1.86.0/core/marker/trait.Sized.html "trait core::marker::Sized"){.trait},
:::
::::

::::: impl-items
::: {#method.borrow_mut .section .method .trait-impl}
[Source](https://doc.rust-lang.org/1.86.0/src/core/borrow.rs.html#218){.src
.rightside}[§](#method.borrow_mut){.anchor}

#### fn [borrow_mut](https://doc.rust-lang.org/1.86.0/core/borrow/trait.BorrowMut.html#tymethod.borrow_mut){.fn}(&mut self) -\> [&mut T](https://doc.rust-lang.org/1.86.0/std/primitive.reference.html){.primitive} {#fn-borrow_mutmut-self---mut-t .code-header}
:::

::: docblock
Mutably borrows from an owned value. [Read
more](https://doc.rust-lang.org/1.86.0/core/borrow/trait.BorrowMut.html#tymethod.borrow_mut)
:::
:::::

:::: {#impl-CloneToUninit-for-T .section .impl}
[Source](https://doc.rust-lang.org/1.86.0/src/core/clone.rs.html#273){.src
.rightside}[§](#impl-CloneToUninit-for-T){.anchor}

### impl\<T\> [CloneToUninit](https://doc.rust-lang.org/1.86.0/core/clone/trait.CloneToUninit.html "trait core::clone::CloneToUninit"){.trait} for T {#implt-clonetouninit-for-t .code-header}

::: where
where T:
[Clone](https://doc.rust-lang.org/1.86.0/core/clone/trait.Clone.html "trait core::clone::Clone"){.trait},
:::
::::

:::::: impl-items
::: {#method.clone_to_uninit .section .method .trait-impl}
[Source](https://doc.rust-lang.org/1.86.0/src/core/clone.rs.html#275){.src
.rightside}[§](#method.clone_to_uninit){.anchor}

#### unsafe fn [clone_to_uninit](https://doc.rust-lang.org/1.86.0/core/clone/trait.CloneToUninit.html#tymethod.clone_to_uninit){.fn}(&self, dst: [\*mut](https://doc.rust-lang.org/1.86.0/std/primitive.pointer.html){.primitive} [u8](https://doc.rust-lang.org/1.86.0/std/primitive.u8.html){.primitive}) {#unsafe-fn-clone_to_uninitself-dst-mut-u8 .code-header}
:::

[]{.item-info}

::: {.stab .unstable}
🔬This is a nightly-only experimental API. (`clone_to_uninit`)
:::

::: docblock
Performs copy-assignment from `self` to `dst`. [Read
more](https://doc.rust-lang.org/1.86.0/core/clone/trait.CloneToUninit.html#tymethod.clone_to_uninit)
:::
::::::

::: {#impl-From%3CT%3E-for-T .section .impl}
[Source](https://doc.rust-lang.org/1.86.0/src/core/convert/mod.rs.html#767){.src
.rightside}[§](#impl-From%3CT%3E-for-T){.anchor}

### impl\<T\> [From](https://doc.rust-lang.org/1.86.0/core/convert/trait.From.html "trait core::convert::From"){.trait}\<T\> for T {#implt-fromt-for-t .code-header}
:::

::::: impl-items
::: {#method.from .section .method .trait-impl}
[Source](https://doc.rust-lang.org/1.86.0/src/core/convert/mod.rs.html#770){.src
.rightside}[§](#method.from){.anchor}

#### fn [from](https://doc.rust-lang.org/1.86.0/core/convert/trait.From.html#tymethod.from){.fn}(t: T) -\> T {#fn-fromt-t---t .code-header}
:::

::: docblock
Returns the argument unchanged.
:::
:::::

::: {#impl-Instrument-for-T .section .impl}
[§](#impl-Instrument-for-T){.anchor}

### impl\<T\> Instrument for T {#implt-instrument-for-t .code-header}
:::

::::::: impl-items
::: {#method.instrument .section .method .trait-impl}
[§](#method.instrument){.anchor}

#### fn [instrument]{.fn}(self, span: Span) -\> Instrumented\<Self\> {#fn-instrumentself-span-span---instrumentedself .code-header}
:::

::: docblock
Instruments this type with the provided \[`Span`\], returning an
`Instrumented` wrapper. Read more
:::

::: {#method.in_current_span .section .method .trait-impl}
[§](#method.in_current_span){.anchor}

#### fn [in_current_span]{.fn}(self) -\> Instrumented\<Self\> {#fn-in_current_spanself---instrumentedself .code-header}
:::

::: docblock
Instruments this type with the [current](super::Span::current())
[`Span`](crate::Span), returning an `Instrumented` wrapper. Read more
:::
:::::::

:::: {#impl-Into%3CU%3E-for-T .section .impl}
[Source](https://doc.rust-lang.org/1.86.0/src/core/convert/mod.rs.html#750-752){.src
.rightside}[§](#impl-Into%3CU%3E-for-T){.anchor}

### impl\<T, U\> [Into](https://doc.rust-lang.org/1.86.0/core/convert/trait.Into.html "trait core::convert::Into"){.trait}\<U\> for T {#implt-u-intou-for-t .code-header}

::: where
where U:
[From](https://doc.rust-lang.org/1.86.0/core/convert/trait.From.html "trait core::convert::From"){.trait}\<T\>,
:::
::::

::::: impl-items
::: {#method.into .section .method .trait-impl}
[Source](https://doc.rust-lang.org/1.86.0/src/core/convert/mod.rs.html#760){.src
.rightside}[§](#method.into){.anchor}

#### fn [into](https://doc.rust-lang.org/1.86.0/core/convert/trait.Into.html#tymethod.into){.fn}(self) -\> U {#fn-intoself---u .code-header}
:::

::: docblock
Calls `U::from(self)`.

That is, this conversion is whatever the implementation of
[`From`](https://doc.rust-lang.org/1.86.0/core/convert/trait.From.html "trait core::convert::From")`<T> for U`
chooses to do.
:::
:::::

::: {#impl-IntoEither-for-T .section .impl}
[Source](https://docs.rs/either/1/src/either/into_either.rs.html#64){.src
.rightside}[§](#impl-IntoEither-for-T){.anchor}

### impl\<T\> [IntoEither](https://docs.rs/either/1/either/into_either/trait.IntoEither.html "trait either::into_either::IntoEither"){.trait} for T {#implt-intoeither-for-t .code-header}
:::

:::::::: impl-items
::: {#method.into_either .section .method .trait-impl}
[Source](https://docs.rs/either/1/src/either/into_either.rs.html#29){.src
.rightside}[§](#method.into_either){.anchor}

#### fn [into_either](https://docs.rs/either/1/either/into_either/trait.IntoEither.html#method.into_either){.fn}(self, into_left: [bool](https://doc.rust-lang.org/1.86.0/std/primitive.bool.html){.primitive}) -\> [Either](https://docs.rs/either/1/either/enum.Either.html "enum either::Either"){.enum}\<Self, Self\> {#fn-into_eitherself-into_left-bool---eitherself-self .code-header}
:::

::: docblock
Converts `self` into a
[`Left`](https://docs.rs/either/1/either/enum.Either.html#variant.Left "variant either::Either::Left")
variant of
[`Either<Self, Self>`](https://docs.rs/either/1/either/enum.Either.html "enum either::Either")
if `into_left` is `true`. Converts `self` into a
[`Right`](https://docs.rs/either/1/either/enum.Either.html#variant.Right "variant either::Either::Right")
variant of
[`Either<Self, Self>`](https://docs.rs/either/1/either/enum.Either.html "enum either::Either")
otherwise. [Read
more](https://docs.rs/either/1/either/into_either/trait.IntoEither.html#method.into_either)
:::

:::: {#method.into_either_with .section .method .trait-impl}
[Source](https://docs.rs/either/1/src/either/into_either.rs.html#55-57){.src
.rightside}[§](#method.into_either_with){.anchor}

#### fn [into_either_with](https://docs.rs/either/1/either/into_either/trait.IntoEither.html#method.into_either_with){.fn}\<F\>(self, into_left: F) -\> [Either](https://docs.rs/either/1/either/enum.Either.html "enum either::Either"){.enum}\<Self, Self\> {#fn-into_either_withfself-into_left-f---eitherself-self .code-header}

::: where
where F:
[FnOnce](https://doc.rust-lang.org/1.86.0/core/ops/function/trait.FnOnce.html "trait core::ops::function::FnOnce"){.trait}(&Self)
-\>
[bool](https://doc.rust-lang.org/1.86.0/std/primitive.bool.html){.primitive},
:::
::::

::: docblock
Converts `self` into a
[`Left`](https://docs.rs/either/1/either/enum.Either.html#variant.Left "variant either::Either::Left")
variant of
[`Either<Self, Self>`](https://docs.rs/either/1/either/enum.Either.html "enum either::Either")
if `into_left(&self)` returns `true`. Converts `self` into a
[`Right`](https://docs.rs/either/1/either/enum.Either.html#variant.Right "variant either::Either::Right")
variant of
[`Either<Self, Self>`](https://docs.rs/either/1/either/enum.Either.html "enum either::Either")
otherwise. [Read
more](https://docs.rs/either/1/either/into_either/trait.IntoEither.html#method.into_either_with)
:::
::::::::

::: {#impl-Pointable-for-T .section .impl}
[§](#impl-Pointable-for-T){.anchor}

### impl\<T\> Pointable for T {#implt-pointable-for-t .code-header}
:::

::::::::::::::: impl-items
::: {#associatedconstant.ALIGN .section .associatedconstant .trait-impl}
[§](#associatedconstant.ALIGN){.anchor}

#### const [ALIGN]{.constant}: [usize](https://doc.rust-lang.org/1.86.0/std/primitive.usize.html){.primitive} {#const-align-usize .code-header}
:::

::: docblock
The alignment of pointer.
:::

::: {#associatedtype.Init .section .associatedtype .trait-impl}
[§](#associatedtype.Init){.anchor}

#### type [Init]{.associatedtype} = T {#type-init-t .code-header}
:::

::: docblock
The type for initializers.
:::

::: {#method.init .section .method .trait-impl}
[§](#method.init){.anchor}

#### unsafe fn [init]{.fn}(init: \<T as Pointable\>::Init) -\> [usize](https://doc.rust-lang.org/1.86.0/std/primitive.usize.html){.primitive} {#unsafe-fn-initinit-t-as-pointableinit---usize .code-header}
:::

::: docblock
Initializes a with the given initializer. Read more
:::

::: {#method.deref .section .method .trait-impl}
[§](#method.deref){.anchor}

#### unsafe fn [deref]{.fn}\<\'a\>(ptr: [usize](https://doc.rust-lang.org/1.86.0/std/primitive.usize.html){.primitive}) -\> [&\'a T](https://doc.rust-lang.org/1.86.0/std/primitive.reference.html){.primitive} {#unsafe-fn-derefaptr-usize---a-t .code-header}
:::

::: docblock
Dereferences the given pointer. Read more
:::

::: {#method.deref_mut .section .method .trait-impl}
[§](#method.deref_mut){.anchor}

#### unsafe fn [deref_mut]{.fn}\<\'a\>(ptr: [usize](https://doc.rust-lang.org/1.86.0/std/primitive.usize.html){.primitive}) -\> [&\'a mut T](https://doc.rust-lang.org/1.86.0/std/primitive.reference.html){.primitive} {#unsafe-fn-deref_mutaptr-usize---a-mut-t .code-header}
:::

::: docblock
Mutably dereferences the given pointer. Read more
:::

::: {#method.drop .section .method .trait-impl}
[§](#method.drop){.anchor}

#### unsafe fn [drop]{.fn}(ptr: [usize](https://doc.rust-lang.org/1.86.0/std/primitive.usize.html){.primitive}) {#unsafe-fn-dropptr-usize .code-header}
:::

::: docblock
Drops the object pointed to by the given pointer. Read more
:::
:::::::::::::::

::: {#impl-Same-for-T .section .impl}
[Source](https://docs.rs/typenum/1.18.0/src/typenum/type_operators.rs.html#34){.src
.rightside}[§](#impl-Same-for-T){.anchor}

### impl\<T\> [Same](https://docs.rs/typenum/1.18.0/typenum/type_operators/trait.Same.html "trait typenum::type_operators::Same"){.trait} for T {#implt-same-for-t .code-header}
:::

::::: impl-items
::: {#associatedtype.Output .section .associatedtype .trait-impl}
[Source](https://docs.rs/typenum/1.18.0/src/typenum/type_operators.rs.html#35){.src
.rightside}[§](#associatedtype.Output){.anchor}

#### type [Output](https://docs.rs/typenum/1.18.0/typenum/type_operators/trait.Same.html#associatedtype.Output){.associatedtype} = T {#type-output-t .code-header}
:::

::: docblock
Should always be `Self`
:::
:::::

:::: {#impl-SupersetOf%3CSS%3E-for-SP .section .impl}
[§](#impl-SupersetOf%3CSS%3E-for-SP){.anchor}

### impl\<SS, SP\> SupersetOf\<SS\> for SP {#implss-sp-supersetofss-for-sp .code-header}

::: where
where SS: SubsetOf\<SP\>,
:::
::::

::::::::::: impl-items
::: {#method.to_subset .section .method .trait-impl}
[§](#method.to_subset){.anchor}

#### fn [to_subset]{.fn}(&self) -\> [Option](https://doc.rust-lang.org/1.86.0/core/option/enum.Option.html "enum core::option::Option"){.enum}\<SS\> {#fn-to_subsetself---optionss .code-header}
:::

::: docblock
The inverse inclusion map: attempts to construct `self` from the
equivalent element of its superset. Read more
:::

::: {#method.is_in_subset .section .method .trait-impl}
[§](#method.is_in_subset){.anchor}

#### fn [is_in_subset]{.fn}(&self) -\> [bool](https://doc.rust-lang.org/1.86.0/std/primitive.bool.html){.primitive} {#fn-is_in_subsetself---bool .code-header}
:::

::: docblock
Checks if `self` is actually part of its subset `T` (and can be
converted to it).
:::

::: {#method.to_subset_unchecked .section .method .trait-impl}
[§](#method.to_subset_unchecked){.anchor}

#### fn [to_subset_unchecked]{.fn}(&self) -\> SS {#fn-to_subset_uncheckedself---ss .code-header}
:::

::: docblock
Use with care! Same as `self.to_subset` but without any property checks.
Always succeeds.
:::

::: {#method.from_subset .section .method .trait-impl}
[§](#method.from_subset){.anchor}

#### fn [from_subset]{.fn}(element: [&SS](https://doc.rust-lang.org/1.86.0/std/primitive.reference.html){.primitive}) -\> SP {#fn-from_subsetelement-ss---sp .code-header}
:::

::: docblock
The inclusion map: converts `self` to the equivalent element of its
superset.
:::
:::::::::::

:::: {#impl-ToOwned-for-T .section .impl}
[Source](https://doc.rust-lang.org/1.86.0/src/alloc/borrow.rs.html#82-84){.src
.rightside}[§](#impl-ToOwned-for-T){.anchor}

### impl\<T\> [ToOwned](https://doc.rust-lang.org/1.86.0/alloc/borrow/trait.ToOwned.html "trait alloc::borrow::ToOwned"){.trait} for T {#implt-toowned-for-t .code-header}

::: where
where T:
[Clone](https://doc.rust-lang.org/1.86.0/core/clone/trait.Clone.html "trait core::clone::Clone"){.trait},
:::
::::

::::::::: impl-items
::: {#associatedtype.Owned .section .associatedtype .trait-impl}
[Source](https://doc.rust-lang.org/1.86.0/src/alloc/borrow.rs.html#86){.src
.rightside}[§](#associatedtype.Owned){.anchor}

#### type [Owned](https://doc.rust-lang.org/1.86.0/alloc/borrow/trait.ToOwned.html#associatedtype.Owned){.associatedtype} = T {#type-owned-t .code-header}
:::

::: docblock
The resulting type after obtaining ownership.
:::

::: {#method.to_owned .section .method .trait-impl}
[Source](https://doc.rust-lang.org/1.86.0/src/alloc/borrow.rs.html#87){.src
.rightside}[§](#method.to_owned){.anchor}

#### fn [to_owned](https://doc.rust-lang.org/1.86.0/alloc/borrow/trait.ToOwned.html#tymethod.to_owned){.fn}(&self) -\> T {#fn-to_ownedself---t .code-header}
:::

::: docblock
Creates owned data from borrowed data, usually by cloning. [Read
more](https://doc.rust-lang.org/1.86.0/alloc/borrow/trait.ToOwned.html#tymethod.to_owned)
:::

::: {#method.clone_into .section .method .trait-impl}
[Source](https://doc.rust-lang.org/1.86.0/src/alloc/borrow.rs.html#91){.src
.rightside}[§](#method.clone_into){.anchor}

#### fn [clone_into](https://doc.rust-lang.org/1.86.0/alloc/borrow/trait.ToOwned.html#method.clone_into){.fn}(&self, target: [&mut T](https://doc.rust-lang.org/1.86.0/std/primitive.reference.html){.primitive}) {#fn-clone_intoself-target-mut-t .code-header}
:::

::: docblock
Uses borrowed data to replace owned data, usually by cloning. [Read
more](https://doc.rust-lang.org/1.86.0/alloc/borrow/trait.ToOwned.html#method.clone_into)
:::
:::::::::

:::: {#impl-TryFrom%3CU%3E-for-T .section .impl}
[Source](https://doc.rust-lang.org/1.86.0/src/core/convert/mod.rs.html#807-809){.src
.rightside}[§](#impl-TryFrom%3CU%3E-for-T){.anchor}

### impl\<T, U\> [TryFrom](https://doc.rust-lang.org/1.86.0/core/convert/trait.TryFrom.html "trait core::convert::TryFrom"){.trait}\<U\> for T {#implt-u-tryfromu-for-t .code-header}

::: where
where U:
[Into](https://doc.rust-lang.org/1.86.0/core/convert/trait.Into.html "trait core::convert::Into"){.trait}\<T\>,
:::
::::

::::::: impl-items
::: {#associatedtype.Error-1 .section .associatedtype .trait-impl}
[Source](https://doc.rust-lang.org/1.86.0/src/core/convert/mod.rs.html#811){.src
.rightside}[§](#associatedtype.Error-1){.anchor}

#### type [Error](https://doc.rust-lang.org/1.86.0/core/convert/trait.TryFrom.html#associatedtype.Error){.associatedtype} = [Infallible](https://doc.rust-lang.org/1.86.0/core/convert/enum.Infallible.html "enum core::convert::Infallible"){.enum} {#type-error-infallible .code-header}
:::

::: docblock
The type returned in the event of a conversion error.
:::

::: {#method.try_from .section .method .trait-impl}
[Source](https://doc.rust-lang.org/1.86.0/src/core/convert/mod.rs.html#814){.src
.rightside}[§](#method.try_from){.anchor}

#### fn [try_from](https://doc.rust-lang.org/1.86.0/core/convert/trait.TryFrom.html#tymethod.try_from){.fn}(value: U) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<T, \<T as [TryFrom](https://doc.rust-lang.org/1.86.0/core/convert/trait.TryFrom.html "trait core::convert::TryFrom"){.trait}\<U\>\>::[Error](https://doc.rust-lang.org/1.86.0/core/convert/trait.TryFrom.html#associatedtype.Error "type core::convert::TryFrom::Error"){.associatedtype}\> {#fn-try_fromvalue-u---resultt-t-as-tryfromuerror .code-header}
:::

::: docblock
Performs the conversion.
:::
:::::::

:::: {#impl-TryInto%3CU%3E-for-T .section .impl}
[Source](https://doc.rust-lang.org/1.86.0/src/core/convert/mod.rs.html#792-794){.src
.rightside}[§](#impl-TryInto%3CU%3E-for-T){.anchor}

### impl\<T, U\> [TryInto](https://doc.rust-lang.org/1.86.0/core/convert/trait.TryInto.html "trait core::convert::TryInto"){.trait}\<U\> for T {#implt-u-tryintou-for-t .code-header}

::: where
where U:
[TryFrom](https://doc.rust-lang.org/1.86.0/core/convert/trait.TryFrom.html "trait core::convert::TryFrom"){.trait}\<T\>,
:::
::::

::::::: impl-items
::: {#associatedtype.Error .section .associatedtype .trait-impl}
[Source](https://doc.rust-lang.org/1.86.0/src/core/convert/mod.rs.html#796){.src
.rightside}[§](#associatedtype.Error){.anchor}

#### type [Error](https://doc.rust-lang.org/1.86.0/core/convert/trait.TryInto.html#associatedtype.Error){.associatedtype} = \<U as [TryFrom](https://doc.rust-lang.org/1.86.0/core/convert/trait.TryFrom.html "trait core::convert::TryFrom"){.trait}\<T\>\>::[Error](https://doc.rust-lang.org/1.86.0/core/convert/trait.TryFrom.html#associatedtype.Error "type core::convert::TryFrom::Error"){.associatedtype} {#type-error-u-as-tryfromterror .code-header}
:::

::: docblock
The type returned in the event of a conversion error.
:::

::: {#method.try_into .section .method .trait-impl}
[Source](https://doc.rust-lang.org/1.86.0/src/core/convert/mod.rs.html#799){.src
.rightside}[§](#method.try_into){.anchor}

#### fn [try_into](https://doc.rust-lang.org/1.86.0/core/convert/trait.TryInto.html#tymethod.try_into){.fn}(self) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<U, \<U as [TryFrom](https://doc.rust-lang.org/1.86.0/core/convert/trait.TryFrom.html "trait core::convert::TryFrom"){.trait}\<T\>\>::[Error](https://doc.rust-lang.org/1.86.0/core/convert/trait.TryFrom.html#associatedtype.Error "type core::convert::TryFrom::Error"){.associatedtype}\> {#fn-try_intoself---resultu-u-as-tryfromterror .code-header}
:::

::: docblock
Performs the conversion.
:::
:::::::

:::: {#impl-VZip%3CV%3E-for-T .section .impl}
[§](#impl-VZip%3CV%3E-for-T){.anchor}

### impl\<V, T\> VZip\<V\> for T {#implv-t-vzipv-for-t .code-header}

::: where
where V: MultiLane\<T\>,
:::
::::

:::: impl-items
::: {#method.vzip .section .method .trait-impl}
[§](#method.vzip){.anchor}

#### fn [vzip]{.fn}(self) -\> V {#fn-vzipself---v .code-header}
:::
::::

::: {#impl-WithSubscriber-for-T .section .impl}
[§](#impl-WithSubscriber-for-T){.anchor}

### impl\<T\> WithSubscriber for T {#implt-withsubscriber-for-t .code-header}
:::

:::::::: impl-items
:::: {#method.with_subscriber .section .method .trait-impl}
[§](#method.with_subscriber){.anchor}

#### fn [with_subscriber]{.fn}\<S\>(self, subscriber: S) -\> WithDispatch\<Self\> {#fn-with_subscribersself-subscriber-s---withdispatchself .code-header}

::: where
where S:
[Into](https://doc.rust-lang.org/1.86.0/core/convert/trait.Into.html "trait core::convert::Into"){.trait}\<Dispatch\>,
:::
::::

::: docblock
Attaches the provided [`Subscriber`](super::Subscriber) to this type,
returning a \[`WithDispatch`\] wrapper. Read more
:::

::: {#method.with_current_subscriber .section .method .trait-impl}
[§](#method.with_current_subscriber){.anchor}

#### fn [with_current_subscriber]{.fn}(self) -\> WithDispatch\<Self\> {#fn-with_current_subscriberself---withdispatchself .code-header}
:::

::: docblock
Attaches the current
[default](crate::dispatcher#setting-the-default-subscriber)
[`Subscriber`](super::Subscriber) to this type, returning a
\[`WithDispatch`\] wrapper. Read more
:::
::::::::
:::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
:::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
