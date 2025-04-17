::::::::::::::::::::::::::::::::::: width-limiter
:::::::::::::::::::::::::::::::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../index.html)::[pnl](index.html)
:::

# Trait [PnLCalculator]{.trait}Copy item path

[[Source](../../src/optionstratlib/pnl/traits.rs.html#15-73){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub trait PnLCalculator {
    // Required methods
    fn calculate_pnl(
        &self,
        _underlying_price: &Positive,
        _expiration_date: ExpirationDate,
        _implied_volatility: &Positive,
    ) -> Result<PnL, Box<dyn Error>>;
    fn calculate_pnl_at_expiration(
        &self,
        _underlying_price: &Positive,
    ) -> Result<PnL, Box<dyn Error>>;

    // Provided method
    fn adjustments_pnl(
        &self,
        _adjustments: &DeltaAdjustment,
    ) -> Result<PnL, Box<dyn Error>> { ... }
}
```

Expand description

::: docblock
Defines the interface for profit and loss (PnL) calculation on financial
instruments.

This trait provides methods to calculate the profit and loss of
financial instruments (particularly options) under different scenarios:
at current market conditions and at expiration. Implementations of this
trait can provide specific PnL calculation logic for different types of
financial instruments or strategies.
:::

## Required Methods[§](#required-methods){.anchor} {#required-methods .section-header}

::::::: methods
::: {#tymethod.calculate_pnl .section .method}
[Source](../../src/optionstratlib/pnl/traits.rs.html#29-34){.src
.rightside}

#### fn [calculate_pnl](#tymethod.calculate_pnl){.fn}( &self, \_underlying_price: &[Positive](../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, \_expiration_date: [ExpirationDate](../model/types/enum.ExpirationDate.html "enum optionstratlib::model::types::ExpirationDate"){.enum}, \_implied_volatility: &[Positive](../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, ) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[PnL](utils/struct.PnL.html "struct optionstratlib::pnl::utils::PnL"){.struct}, [Box](https://doc.rust-lang.org/1.86.0/alloc/boxed/struct.Box.html "struct alloc::boxed::Box"){.struct}\<dyn [Error](https://doc.rust-lang.org/1.86.0/core/error/trait.Error.html "trait core::error::Error"){.trait}\>\> {#fn-calculate_pnl-self-_underlying_price-positive-_expiration_date-expirationdate-_implied_volatility-positive---resultpnl-boxdyn-error .code-header}
:::

::: docblock
Calculates the current PnL based on market conditions.

This method computes the profit and loss of a financial instrument given
the current underlying price, time to expiration, and implied
volatility. It returns a complete PnL structure with realized and
unrealized values.

##### [§](#parameters){.doc-anchor}Parameters

- `_underlying_price` - The current market price of the underlying asset
- `_expiration_date` - The expiration date of the instrument
- `_implied_volatility` - The current implied volatility

##### [§](#returns){.doc-anchor}Returns

- `Result<PnL, Box<dyn Error>>` - The calculated PnL or an error
:::

::: {#tymethod.calculate_pnl_at_expiration .section .method}
[Source](../../src/optionstratlib/pnl/traits.rs.html#47-50){.src
.rightside}

#### fn [calculate_pnl_at_expiration](#tymethod.calculate_pnl_at_expiration){.fn}( &self, \_underlying_price: &[Positive](../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, ) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[PnL](utils/struct.PnL.html "struct optionstratlib::pnl::utils::PnL"){.struct}, [Box](https://doc.rust-lang.org/1.86.0/alloc/boxed/struct.Box.html "struct alloc::boxed::Box"){.struct}\<dyn [Error](https://doc.rust-lang.org/1.86.0/core/error/trait.Error.html "trait core::error::Error"){.trait}\>\> {#fn-calculate_pnl_at_expiration-self-_underlying_price-positive---resultpnl-boxdyn-error .code-header}
:::

::: docblock
Calculates the PnL at the expiration of the instrument.

This method computes the final profit and loss at the expiration date,
which is typically simpler than the pre-expiration calculation since
time value and volatility no longer factor into the price.

##### [§](#parameters-1){.doc-anchor}Parameters

- `_underlying_price` - The price of the underlying asset at expiration

##### [§](#returns-1){.doc-anchor}Returns

- `Result<PnL, Box<dyn Error>>` - The calculated PnL at expiration or an
  error
:::
:::::::

## Provided Methods[§](#provided-methods){.anchor} {#provided-methods .section-header}

::::: methods
::: {#method.adjustments_pnl .section .method}
[Source](../../src/optionstratlib/pnl/traits.rs.html#70-72){.src
.rightside}

#### fn [adjustments_pnl](#method.adjustments_pnl){.fn}( &self, \_adjustments: &[DeltaAdjustment](../strategies/delta_neutral/enum.DeltaAdjustment.html "enum optionstratlib::strategies::delta_neutral::DeltaAdjustment"){.enum}, ) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[PnL](utils/struct.PnL.html "struct optionstratlib::pnl::utils::PnL"){.struct}, [Box](https://doc.rust-lang.org/1.86.0/alloc/boxed/struct.Box.html "struct alloc::boxed::Box"){.struct}\<dyn [Error](https://doc.rust-lang.org/1.86.0/core/error/trait.Error.html "trait core::error::Error"){.trait}\>\> {#fn-adjustments_pnl-self-_adjustments-deltaadjustment---resultpnl-boxdyn-error .code-header}
:::

::: docblock
Calculates the Profit and Loss (PnL) for a series of delta adjustments
in a trading strategy.

##### [§](#arguments){.doc-anchor}Arguments

- `_adjustments` - A vector of `DeltaAdjustment` instances representing
  the adjustments made to maintain delta neutrality in a trading
  strategy.

##### [§](#returns-2){.doc-anchor}Returns

- `Result<PnL, Box<dyn Error>>` - If successful, returns a `PnL` object
  containing information about realized and unrealized profits/losses,
  costs, and income. Otherwise, returns an error.

##### [§](#panics){.doc-anchor}Panics

This function always panics with the message "adjustments_pnl is not
implemented for this Strategy." It serves as a placeholder or trait
method that must be implemented by specific strategy implementations.
:::
:::::

## Implementors[§](#implementors){.anchor} {#implementors .section-header}

:::::::::::::::::::::: {#implementors-list}
::: {#impl-PnLCalculator-for-Options .section .impl}
[Source](../../src/optionstratlib/model/option.rs.html#665-722){.src
.rightside}[§](#impl-PnLCalculator-for-Options){.anchor}

### impl [PnLCalculator](trait.PnLCalculator.html "trait optionstratlib::pnl::PnLCalculator"){.trait} for [Options](../model/option/struct.Options.html "struct optionstratlib::model::option::Options"){.struct} {#impl-pnlcalculator-for-options .code-header}
:::

:::: {#impl-PnLCalculator-for-Position .section .impl}
[Source](../../src/optionstratlib/model/position.rs.html#686-765){.src
.rightside}[§](#impl-PnLCalculator-for-Position){.anchor}

### impl [PnLCalculator](trait.PnLCalculator.html "trait optionstratlib::pnl::PnLCalculator"){.trait} for [Position](../model/position/struct.Position.html "struct optionstratlib::model::position::Position"){.struct} {#impl-pnlcalculator-for-position .code-header}

::: docblock
#### [§](#position-profit-and-loss-pnl-calculator){.doc-anchor}Position Profit and Loss (PnL) Calculator

This trait implementation provides methods to calculate the profit and
loss (PnL) for option positions under different market scenarios.
:::
::::

::: docblock
The implementation offers two main calculations:

1.  Current PnL based on updated market conditions
2.  PnL at expiration based on a projected underlying price

These calculations are essential for risk management, position
monitoring, and strategy planning in options trading.
:::

::: {#impl-PnLCalculator-for-BearCallSpread .section .impl}
[Source](../../src/optionstratlib/strategies/bear_call_spread.rs.html#783-809){.src
.rightside}[§](#impl-PnLCalculator-for-BearCallSpread){.anchor}

### impl [PnLCalculator](trait.PnLCalculator.html "trait optionstratlib::pnl::PnLCalculator"){.trait} for [BearCallSpread](../strategies/bear_call_spread/struct.BearCallSpread.html "struct optionstratlib::strategies::bear_call_spread::BearCallSpread"){.struct} {#impl-pnlcalculator-for-bearcallspread .code-header}
:::

::: {#impl-PnLCalculator-for-BearPutSpread .section .impl}
[Source](../../src/optionstratlib/strategies/bear_put_spread.rs.html#773-799){.src
.rightside}[§](#impl-PnLCalculator-for-BearPutSpread){.anchor}

### impl [PnLCalculator](trait.PnLCalculator.html "trait optionstratlib::pnl::PnLCalculator"){.trait} for [BearPutSpread](../strategies/bear_put_spread/struct.BearPutSpread.html "struct optionstratlib::strategies::bear_put_spread::BearPutSpread"){.struct} {#impl-pnlcalculator-for-bearputspread .code-header}
:::

::: {#impl-PnLCalculator-for-BullCallSpread .section .impl}
[Source](../../src/optionstratlib/strategies/bull_call_spread.rs.html#786-812){.src
.rightside}[§](#impl-PnLCalculator-for-BullCallSpread){.anchor}

### impl [PnLCalculator](trait.PnLCalculator.html "trait optionstratlib::pnl::PnLCalculator"){.trait} for [BullCallSpread](../strategies/bull_call_spread/struct.BullCallSpread.html "struct optionstratlib::strategies::bull_call_spread::BullCallSpread"){.struct} {#impl-pnlcalculator-for-bullcallspread .code-header}
:::

::: {#impl-PnLCalculator-for-BullPutSpread .section .impl}
[Source](../../src/optionstratlib/strategies/bull_put_spread.rs.html#887-913){.src
.rightside}[§](#impl-PnLCalculator-for-BullPutSpread){.anchor}

### impl [PnLCalculator](trait.PnLCalculator.html "trait optionstratlib::pnl::PnLCalculator"){.trait} for [BullPutSpread](../strategies/bull_put_spread/struct.BullPutSpread.html "struct optionstratlib::strategies::bull_put_spread::BullPutSpread"){.struct} {#impl-pnlcalculator-for-bullputspread .code-header}
:::

::: {#impl-PnLCalculator-for-LongButterflySpread .section .impl}
[Source](../../src/optionstratlib/strategies/butterfly_spread.rs.html#978-1014){.src
.rightside}[§](#impl-PnLCalculator-for-LongButterflySpread){.anchor}

### impl [PnLCalculator](trait.PnLCalculator.html "trait optionstratlib::pnl::PnLCalculator"){.trait} for [LongButterflySpread](../strategies/butterfly_spread/struct.LongButterflySpread.html "struct optionstratlib::strategies::butterfly_spread::LongButterflySpread"){.struct} {#impl-pnlcalculator-for-longbutterflyspread .code-header}
:::

::: {#impl-PnLCalculator-for-ShortButterflySpread .section .impl}
[Source](../../src/optionstratlib/strategies/butterfly_spread.rs.html#1945-1981){.src
.rightside}[§](#impl-PnLCalculator-for-ShortButterflySpread){.anchor}

### impl [PnLCalculator](trait.PnLCalculator.html "trait optionstratlib::pnl::PnLCalculator"){.trait} for [ShortButterflySpread](../strategies/butterfly_spread/struct.ShortButterflySpread.html "struct optionstratlib::strategies::butterfly_spread::ShortButterflySpread"){.struct} {#impl-pnlcalculator-for-shortbutterflyspread .code-header}
:::

::: {#impl-PnLCalculator-for-CallButterfly .section .impl}
[Source](../../src/optionstratlib/strategies/call_butterfly.rs.html#954-990){.src
.rightside}[§](#impl-PnLCalculator-for-CallButterfly){.anchor}

### impl [PnLCalculator](trait.PnLCalculator.html "trait optionstratlib::pnl::PnLCalculator"){.trait} for [CallButterfly](../strategies/call_butterfly/struct.CallButterfly.html "struct optionstratlib::strategies::call_butterfly::CallButterfly"){.struct} {#impl-pnlcalculator-for-callbutterfly .code-header}
:::

::: {#impl-PnLCalculator-for-CustomStrategy .section .impl}
[Source](../../src/optionstratlib/strategies/custom.rs.html#757-789){.src
.rightside}[§](#impl-PnLCalculator-for-CustomStrategy){.anchor}

### impl [PnLCalculator](trait.PnLCalculator.html "trait optionstratlib::pnl::PnLCalculator"){.trait} for [CustomStrategy](../strategies/custom/struct.CustomStrategy.html "struct optionstratlib::strategies::custom::CustomStrategy"){.struct} {#impl-pnlcalculator-for-customstrategy .code-header}
:::

::: {#impl-PnLCalculator-for-IronButterfly .section .impl}
[Source](../../src/optionstratlib/strategies/iron_butterfly.rs.html#1020-1058){.src
.rightside}[§](#impl-PnLCalculator-for-IronButterfly){.anchor}

### impl [PnLCalculator](trait.PnLCalculator.html "trait optionstratlib::pnl::PnLCalculator"){.trait} for [IronButterfly](../strategies/iron_butterfly/struct.IronButterfly.html "struct optionstratlib::strategies::iron_butterfly::IronButterfly"){.struct} {#impl-pnlcalculator-for-ironbutterfly .code-header}
:::

::: {#impl-PnLCalculator-for-IronCondor .section .impl}
[Source](../../src/optionstratlib/strategies/iron_condor.rs.html#1055-1093){.src
.rightside}[§](#impl-PnLCalculator-for-IronCondor){.anchor}

### impl [PnLCalculator](trait.PnLCalculator.html "trait optionstratlib::pnl::PnLCalculator"){.trait} for [IronCondor](../strategies/iron_condor/struct.IronCondor.html "struct optionstratlib::strategies::iron_condor::IronCondor"){.struct} {#impl-pnlcalculator-for-ironcondor .code-header}
:::

::: {#impl-PnLCalculator-for-PoorMansCoveredCall .section .impl}
[Source](../../src/optionstratlib/strategies/poor_mans_covered_call.rs.html#811-837){.src
.rightside}[§](#impl-PnLCalculator-for-PoorMansCoveredCall){.anchor}

### impl [PnLCalculator](trait.PnLCalculator.html "trait optionstratlib::pnl::PnLCalculator"){.trait} for [PoorMansCoveredCall](../strategies/poor_mans_covered_call/struct.PoorMansCoveredCall.html "struct optionstratlib::strategies::poor_mans_covered_call::PoorMansCoveredCall"){.struct} {#impl-pnlcalculator-for-poormanscoveredcall .code-header}
:::

::: {#impl-PnLCalculator-for-LongStraddle .section .impl}
[Source](../../src/optionstratlib/strategies/straddle.rs.html#1618-1644){.src
.rightside}[§](#impl-PnLCalculator-for-LongStraddle){.anchor}

### impl [PnLCalculator](trait.PnLCalculator.html "trait optionstratlib::pnl::PnLCalculator"){.trait} for [LongStraddle](../strategies/straddle/struct.LongStraddle.html "struct optionstratlib::strategies::straddle::LongStraddle"){.struct} {#impl-pnlcalculator-for-longstraddle .code-header}
:::

::: {#impl-PnLCalculator-for-ShortStraddle .section .impl}
[Source](../../src/optionstratlib/strategies/straddle.rs.html#838-864){.src
.rightside}[§](#impl-PnLCalculator-for-ShortStraddle){.anchor}

### impl [PnLCalculator](trait.PnLCalculator.html "trait optionstratlib::pnl::PnLCalculator"){.trait} for [ShortStraddle](../strategies/straddle/struct.ShortStraddle.html "struct optionstratlib::strategies::straddle::ShortStraddle"){.struct} {#impl-pnlcalculator-for-shortstraddle .code-header}
:::

::: {#impl-PnLCalculator-for-LongStrangle .section .impl}
[Source](../../src/optionstratlib/strategies/strangle.rs.html#1922-2029){.src
.rightside}[§](#impl-PnLCalculator-for-LongStrangle){.anchor}

### impl [PnLCalculator](trait.PnLCalculator.html "trait optionstratlib::pnl::PnLCalculator"){.trait} for [LongStrangle](../strategies/strangle/struct.LongStrangle.html "struct optionstratlib::strategies::strangle::LongStrangle"){.struct} {#impl-pnlcalculator-for-longstrangle .code-header}
:::

::: {#impl-PnLCalculator-for-ShortStrangle .section .impl}
[Source](../../src/optionstratlib/strategies/strangle.rs.html#934-1055){.src
.rightside}[§](#impl-PnLCalculator-for-ShortStrangle){.anchor}

### impl [PnLCalculator](trait.PnLCalculator.html "trait optionstratlib::pnl::PnLCalculator"){.trait} for [ShortStrangle](../strategies/strangle/struct.ShortStrangle.html "struct optionstratlib::strategies::strangle::ShortStrangle"){.struct} {#impl-pnlcalculator-for-shortstrangle .code-header}
:::
::::::::::::::::::::::
::::::::::::::::::::::::::::::::::
:::::::::::::::::::::::::::::::::::
