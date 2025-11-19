:::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::: width-limiter
::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../../index.html)::[strategies](../index.html)::[bull_call_spread](index.html)
:::

# Struct [BullCallSpread]{.struct} Copy item path

[[Source](../../../src/optionstratlib/strategies/bull_call_spread.rs.html#79-97){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub struct BullCallSpread {
    pub name: String,
    pub kind: StrategyType,
    pub description: String,
    pub break_even_points: Vec<Positive>,
    pub long_call: Position,
    pub short_call: Position,
}
```

Expand description

::: docblock
Represents a Bull Call Spread options trading strategy.

A Bull Call Spread is a vertical spread strategy that involves buying a
call option with a lower strike price and selling another call option
with a higher strike price, both with the same expiration date. This
strategy is typically used when an investor expects a moderate rise in
the price of the underlying asset.

## [§](#advantages){.doc-anchor}Advantages

- Limited risk (maximum loss is the net debit paid)
- Lower cost than buying a call option outright
- Potential for profit if the underlying price rises

## [§](#disadvantages){.doc-anchor}Disadvantages

- Limited profit potential (capped by the difference between strike
  prices minus the net debit)
- Requires more capital than a single option position
- Loses value as expiration approaches if the underlying price doesn't
  rise
:::

## Fields[§](#fields){.anchor} {#fields .fields .section-header}

[[§](#structfield.name){.anchor
.field}`name: `[`String`](https://doc.rust-lang.org/1.91.1/alloc/string/struct.String.html "struct alloc::string::String"){.struct}]{#structfield.name
.structfield .section-header}

::: docblock
The name of the strategy, typically including underlying asset
information.
:::

[[§](#structfield.kind){.anchor
.field}`kind: `[`StrategyType`](../base/enum.StrategyType.html "enum optionstratlib::strategies::base::StrategyType"){.enum}]{#structfield.kind
.structfield .section-header}

::: docblock
The type of strategy, which is StrategyType::BullCallSpread for this
struct.
:::

[[§](#structfield.description){.anchor
.field}`description: `[`String`](https://doc.rust-lang.org/1.91.1/alloc/string/struct.String.html "struct alloc::string::String"){.struct}]{#structfield.description
.structfield .section-header}

::: docblock
A textual description of this specific bull call spread instance.
:::

[[§](#structfield.break_even_points){.anchor
.field}`break_even_points: `[`Vec`](https://doc.rust-lang.org/1.91.1/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}`<`[`Positive`](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}`>`]{#structfield.break_even_points
.structfield .section-header}

::: docblock
The price points at which the strategy breaks even (typically one
point).
:::

[[§](#structfield.long_call){.anchor
.field}`long_call: `[`Position`](../../model/position/struct.Position.html "struct optionstratlib::model::position::Position"){.struct}]{#structfield.long_call
.structfield .section-header}

::: docblock
The long call position (lower strike price).
:::

[[§](#structfield.short_call){.anchor
.field}`short_call: `[`Position`](../../model/position/struct.Position.html "struct optionstratlib::model::position::Position"){.struct}]{#structfield.short_call
.structfield .section-header}

::: docblock
The short call position (higher strike price).
:::

## Implementations[§](#implementations){.anchor} {#implementations .section-header}

::::::: {#implementations-list}
::: {#impl-BullCallSpread .section .impl}
[Source](../../../src/optionstratlib/strategies/bull_call_spread.rs.html#99-229){.src
.rightside}[§](#impl-BullCallSpread){.anchor}

### impl [BullCallSpread](struct.BullCallSpread.html "struct optionstratlib::strategies::bull_call_spread::BullCallSpread"){.struct} {#impl-bullcallspread .code-header}
:::

::::: impl-items
::: {#method.new .section .method}
[Source](../../../src/optionstratlib/strategies/bull_call_spread.rs.html#135-228){.src
.rightside}

#### pub fn [new](#method.new){.fn}( underlying_symbol: [String](https://doc.rust-lang.org/1.91.1/alloc/string/struct.String.html "struct alloc::string::String"){.struct}, underlying_price: [Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, long_strike: [Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, short_strike: [Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, expiration: [ExpirationDate](../../model/enum.ExpirationDate.html "enum optionstratlib::model::ExpirationDate"){.enum}, implied_volatility: [Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, risk_free_rate: [Decimal](../../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}, dividend_yield: [Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, quantity: [Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, premium_long_call: [Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, premium_short_call: [Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, open_fee_long_call: [Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, close_fee_long_call: [Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, open_fee_short_call: [Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, close_fee_short_call: [Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, ) -\> Self {#pub-fn-new-underlying_symbol-string-underlying_price-positive-long_strike-positive-short_strike-positive-expiration-expirationdate-implied_volatility-positive-risk_free_rate-decimal-dividend_yield-positive-quantity-positive-premium_long_call-positive-premium_short_call-positive-open_fee_long_call-positive-close_fee_long_call-positive-open_fee_short_call-positive-close_fee_short_call-positive---self .code-header}
:::

::: docblock
Creates a new Bull Call Spread strategy.

A Bull Call Spread is created by buying a call option with a lower
strike price and simultaneously selling a call option with a higher
strike price, both with the same expiration date. This strategy benefits
from moderate increases in the underlying asset's price.

##### [§](#arguments){.doc-anchor}Arguments

- `underlying_symbol` - The ticker symbol of the underlying asset.
- `underlying_price` - The current market price of the underlying asset.
- `long_strike` - The strike price for the long call option. If set to
  zero, defaults to the underlying price.
- `short_strike` - The strike price for the short call option. If set to
  zero, defaults to the underlying price.
- `expiration` - The expiration date for both options.
- `implied_volatility` - The implied volatility value used for option
  pricing.
- `risk_free_rate` - The risk-free interest rate used in option pricing
  calculations.
- `dividend_yield` - The dividend yield of the underlying asset.
- `quantity` - The number of contracts to create for both positions.
- `premium_long_call` - The premium paid for the long call position.
- `premium_short_call` - The premium received for the short call
  position.
- `open_fee_long_call` - The fee paid when opening the long call
  position.
- `close_fee_long_call` - The fee that will be paid when closing the
  long call position.
- `open_fee_short_call` - The fee paid when opening the short call
  position.
- `close_fee_short_call` - The fee that will be paid when closing the
  short call position.

##### [§](#returns){.doc-anchor}Returns

Returns a fully configured `BullCallSpread` strategy instance with
positions and break-even points calculated.

##### [§](#panics){.doc-anchor}Panics

This function will panic if:

- The long call position cannot be added to the strategy
- The short call position cannot be added to the strategy
- Break-even points cannot be calculated
:::
:::::
:::::::

## Trait Implementations[§](#trait-implementations){.anchor} {#trait-implementations .section-header}

:::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::: {#trait-implementations-list}
::: {#impl-BasicAble-for-BullCallSpread .section .impl}
[Source](../../../src/optionstratlib/strategies/bull_call_spread.rs.html#461-582){.src
.rightside}[§](#impl-BasicAble-for-BullCallSpread){.anchor}

### impl [BasicAble](../base/trait.BasicAble.html "trait optionstratlib::strategies::base::BasicAble"){.trait} for [BullCallSpread](struct.BullCallSpread.html "struct optionstratlib::strategies::bull_call_spread::BullCallSpread"){.struct} {#impl-basicable-for-bullcallspread .code-header}
:::

::::::::::::::::::::::::::::::::::::::::: impl-items
::: {#method.get_title .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/bull_call_spread.rs.html#462-474){.src
.rightside}[§](#method.get_title){.anchor}

#### fn [get_title](../base/trait.BasicAble.html#method.get_title){.fn}(&self) -\> [String](https://doc.rust-lang.org/1.91.1/alloc/string/struct.String.html "struct alloc::string::String"){.struct} {#fn-get_titleself---string .code-header}
:::

::: docblock
Retrieves the title associated with the current instance of the
strategy. [Read more](../base/trait.BasicAble.html#method.get_title)
:::

::: {#method.get_option_basic_type .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/bull_call_spread.rs.html#475-493){.src
.rightside}[§](#method.get_option_basic_type){.anchor}

#### fn [get_option_basic_type](../base/trait.BasicAble.html#method.get_option_basic_type){.fn}(&self) -\> [HashSet](https://doc.rust-lang.org/1.91.1/std/collections/hash/set/struct.HashSet.html "struct std::collections::hash::set::HashSet"){.struct}\<[OptionBasicType](../../model/types/struct.OptionBasicType.html "struct optionstratlib::model::types::OptionBasicType"){.struct}\<\'\_\>\> {#fn-get_option_basic_typeself---hashsetoptionbasictype_ .code-header}
:::

::: docblock
Retrieves a `HashSet` of `OptionBasicType` values associated with the
current strategy. [Read
more](../base/trait.BasicAble.html#method.get_option_basic_type)
:::

::: {#method.get_implied_volatility .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/bull_call_spread.rs.html#494-520){.src
.rightside}[§](#method.get_implied_volatility){.anchor}

#### fn [get_implied_volatility](../base/trait.BasicAble.html#method.get_implied_volatility){.fn}(&self) -\> [HashMap](https://doc.rust-lang.org/1.91.1/std/collections/hash/map/struct.HashMap.html "struct std::collections::hash::map::HashMap"){.struct}\<[OptionBasicType](../../model/types/struct.OptionBasicType.html "struct optionstratlib::model::types::OptionBasicType"){.struct}\<\'\_\>, &[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\> {#fn-get_implied_volatilityself---hashmapoptionbasictype_-positive .code-header}
:::

::: docblock
Retrieves the implied volatility for the current strategy. [Read
more](../base/trait.BasicAble.html#method.get_implied_volatility)
:::

::: {#method.get_quantity .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/bull_call_spread.rs.html#521-541){.src
.rightside}[§](#method.get_quantity){.anchor}

#### fn [get_quantity](../base/trait.BasicAble.html#method.get_quantity){.fn}(&self) -\> [HashMap](https://doc.rust-lang.org/1.91.1/std/collections/hash/map/struct.HashMap.html "struct std::collections::hash::map::HashMap"){.struct}\<[OptionBasicType](../../model/types/struct.OptionBasicType.html "struct optionstratlib::model::types::OptionBasicType"){.struct}\<\'\_\>, &[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\> {#fn-get_quantityself---hashmapoptionbasictype_-positive .code-header}
:::

::: docblock
Retrieves the quantity information associated with the strategy. [Read
more](../base/trait.BasicAble.html#method.get_quantity)
:::

::: {#method.one_option .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/bull_call_spread.rs.html#542-544){.src
.rightside}[§](#method.one_option){.anchor}

#### fn [one_option](../base/trait.BasicAble.html#method.one_option){.fn}(&self) -\> &[Options](../../model/option/struct.Options.html "struct optionstratlib::model::option::Options"){.struct} {#fn-one_optionself---options .code-header}
:::

::: docblock
This method, `one_option`, is designed to retrieve a reference to an
`Options` object. However, in this implementation, the function is not
currently functional, as it explicitly triggers an unimplemented error
when called. [Read more](../base/trait.BasicAble.html#method.one_option)
:::

::: {#method.one_option_mut .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/bull_call_spread.rs.html#545-547){.src
.rightside}[§](#method.one_option_mut){.anchor}

#### fn [one_option_mut](../base/trait.BasicAble.html#method.one_option_mut){.fn}(&mut self) -\> &mut [Options](../../model/option/struct.Options.html "struct optionstratlib::model::option::Options"){.struct} {#fn-one_option_mutmut-self---mut-options .code-header}
:::

::: docblock
Provides a mutable reference to an `Options` instance. [Read
more](../base/trait.BasicAble.html#method.one_option_mut)
:::

::: {#method.set_expiration_date .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/bull_call_spread.rs.html#548-555){.src
.rightside}[§](#method.set_expiration_date){.anchor}

#### fn [set_expiration_date](../base/trait.BasicAble.html#method.set_expiration_date){.fn}( &mut self, expiration_date: [ExpirationDate](../../model/enum.ExpirationDate.html "enum optionstratlib::model::ExpirationDate"){.enum}, ) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[()](https://doc.rust-lang.org/1.91.1/std/primitive.unit.html){.primitive}, [StrategyError](../../error/strategies/enum.StrategyError.html "enum optionstratlib::error::strategies::StrategyError"){.enum}\> {#fn-set_expiration_date-mut-self-expiration_date-expirationdate---result-strategyerror .code-header}
:::

::: docblock
Sets the expiration date for the strategy. [Read
more](../base/trait.BasicAble.html#method.set_expiration_date)
:::

::: {#method.set_underlying_price .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/bull_call_spread.rs.html#556-568){.src
.rightside}[§](#method.set_underlying_price){.anchor}

#### fn [set_underlying_price](../base/trait.BasicAble.html#method.set_underlying_price){.fn}( &mut self, price: &[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, ) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[()](https://doc.rust-lang.org/1.91.1/std/primitive.unit.html){.primitive}, [StrategyError](../../error/strategies/enum.StrategyError.html "enum optionstratlib::error::strategies::StrategyError"){.enum}\> {#fn-set_underlying_price-mut-self-price-positive---result-strategyerror .code-header}
:::

::: docblock
Sets the underlying price for this strategy. [Read
more](../base/trait.BasicAble.html#method.set_underlying_price)
:::

::: {#method.set_implied_volatility .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/bull_call_spread.rs.html#569-581){.src
.rightside}[§](#method.set_implied_volatility){.anchor}

#### fn [set_implied_volatility](../base/trait.BasicAble.html#method.set_implied_volatility){.fn}( &mut self, volatility: &[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, ) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[()](https://doc.rust-lang.org/1.91.1/std/primitive.unit.html){.primitive}, [StrategyError](../../error/strategies/enum.StrategyError.html "enum optionstratlib::error::strategies::StrategyError"){.enum}\> {#fn-set_implied_volatility-mut-self-volatility-positive---result-strategyerror .code-header}
:::

::: docblock
Updates the volatility for the strategy. [Read
more](../base/trait.BasicAble.html#method.set_implied_volatility)
:::

::: {#method.get_symbol .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/base.rs.html#412-414){.src
.rightside}[§](#method.get_symbol){.anchor}

#### fn [get_symbol](../base/trait.BasicAble.html#method.get_symbol){.fn}(&self) -\> &[str](https://doc.rust-lang.org/1.91.1/std/primitive.str.html){.primitive} {#fn-get_symbolself---str .code-header}
:::

::: docblock
Retrieves the symbol associated with the current instance by delegating
the call to the `get_symbol` method of the `one_option` object. [Read
more](../base/trait.BasicAble.html#method.get_symbol)
:::

::: {#method.get_strike .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/base.rs.html#430-432){.src
.rightside}[§](#method.get_strike){.anchor}

#### fn [get_strike](../base/trait.BasicAble.html#method.get_strike){.fn}(&self) -\> [HashMap](https://doc.rust-lang.org/1.91.1/std/collections/hash/map/struct.HashMap.html "struct std::collections::hash::map::HashMap"){.struct}\<[OptionBasicType](../../model/types/struct.OptionBasicType.html "struct optionstratlib::model::types::OptionBasicType"){.struct}\<\'\_\>, &[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\> {#fn-get_strikeself---hashmapoptionbasictype_-positive .code-header}
:::

::: docblock
Retrieves a mapping of option basic types to their associated positive
strike values. [Read
more](../base/trait.BasicAble.html#method.get_strike)
:::

::: {#method.get_strikes .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/base.rs.html#448-453){.src
.rightside}[§](#method.get_strikes){.anchor}

#### fn [get_strikes](../base/trait.BasicAble.html#method.get_strikes){.fn}(&self) -\> [Vec](https://doc.rust-lang.org/1.91.1/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<&[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\> {#fn-get_strikesself---vecpositive .code-header}
:::

::: docblock
Retrieves a vector of strike prices from the option types. [Read
more](../base/trait.BasicAble.html#method.get_strikes)
:::

::: {#method.get_side .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/base.rs.html#475-480){.src
.rightside}[§](#method.get_side){.anchor}

#### fn [get_side](../base/trait.BasicAble.html#method.get_side){.fn}(&self) -\> [HashMap](https://doc.rust-lang.org/1.91.1/std/collections/hash/map/struct.HashMap.html "struct std::collections::hash::map::HashMap"){.struct}\<[OptionBasicType](../../model/types/struct.OptionBasicType.html "struct optionstratlib::model::types::OptionBasicType"){.struct}\<\'\_\>, &[Side](../../model/types/enum.Side.html "enum optionstratlib::model::types::Side"){.enum}\> {#fn-get_sideself---hashmapoptionbasictype_-side .code-header}
:::

::: docblock
Retrieves a `HashMap` that maps each `OptionBasicType` to its
corresponding `Side`. [Read
more](../base/trait.BasicAble.html#method.get_side)
:::

::: {#method.get_type .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/base.rs.html#492-494){.src
.rightside}[§](#method.get_type){.anchor}

#### fn [get_type](../base/trait.BasicAble.html#method.get_type){.fn}(&self) -\> &[OptionType](../../model/types/enum.OptionType.html "enum optionstratlib::model::types::OptionType"){.enum} {#fn-get_typeself---optiontype .code-header}
:::

::: docblock
Retrieves the type of the option. [Read
more](../base/trait.BasicAble.html#method.get_type)
:::

::: {#method.get_style .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/base.rs.html#512-517){.src
.rightside}[§](#method.get_style){.anchor}

#### fn [get_style](../base/trait.BasicAble.html#method.get_style){.fn}(&self) -\> [HashMap](https://doc.rust-lang.org/1.91.1/std/collections/hash/map/struct.HashMap.html "struct std::collections::hash::map::HashMap"){.struct}\<[OptionBasicType](../../model/types/struct.OptionBasicType.html "struct optionstratlib::model::types::OptionBasicType"){.struct}\<\'\_\>, &[OptionStyle](../../model/types/enum.OptionStyle.html "enum optionstratlib::model::types::OptionStyle"){.enum}\> {#fn-get_styleself---hashmapoptionbasictype_-optionstyle .code-header}
:::

::: docblock
Retrieves a mapping of `OptionBasicType` to their corresponding
`OptionStyle`. [Read
more](../base/trait.BasicAble.html#method.get_style)
:::

::: {#method.get_expiration .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/base.rs.html#536-541){.src
.rightside}[§](#method.get_expiration){.anchor}

#### fn [get_expiration](../base/trait.BasicAble.html#method.get_expiration){.fn}(&self) -\> [HashMap](https://doc.rust-lang.org/1.91.1/std/collections/hash/map/struct.HashMap.html "struct std::collections::hash::map::HashMap"){.struct}\<[OptionBasicType](../../model/types/struct.OptionBasicType.html "struct optionstratlib::model::types::OptionBasicType"){.struct}\<\'\_\>, &[ExpirationDate](../../model/enum.ExpirationDate.html "enum optionstratlib::model::ExpirationDate"){.enum}\> {#fn-get_expirationself---hashmapoptionbasictype_-expirationdate .code-header}
:::

::: docblock
Retrieves a map of option basic types to their corresponding expiration
dates. [Read more](../base/trait.BasicAble.html#method.get_expiration)
:::

::: {#method.get_underlying_price .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/base.rs.html#595-597){.src
.rightside}[§](#method.get_underlying_price){.anchor}

#### fn [get_underlying_price](../base/trait.BasicAble.html#method.get_underlying_price){.fn}(&self) -\> &[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#fn-get_underlying_priceself---positive .code-header}
:::

::: docblock
Retrieves the underlying price of the financial instrument (e.g.,
option). [Read
more](../base/trait.BasicAble.html#method.get_underlying_price)
:::

::: {#method.get_risk_free_rate .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/base.rs.html#619-621){.src
.rightside}[§](#method.get_risk_free_rate){.anchor}

#### fn [get_risk_free_rate](../base/trait.BasicAble.html#method.get_risk_free_rate){.fn}(&self) -\> [HashMap](https://doc.rust-lang.org/1.91.1/std/collections/hash/map/struct.HashMap.html "struct std::collections::hash::map::HashMap"){.struct}\<[OptionBasicType](../../model/types/struct.OptionBasicType.html "struct optionstratlib::model::types::OptionBasicType"){.struct}\<\'\_\>, &[Decimal](../../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}\> {#fn-get_risk_free_rateself---hashmapoptionbasictype_-decimal .code-header}
:::

::: docblock
Retrieves the risk-free interest rate associated with a given set of
options. [Read
more](../base/trait.BasicAble.html#method.get_risk_free_rate)
:::

::: {#method.get_dividend_yield .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/base.rs.html#636-638){.src
.rightside}[§](#method.get_dividend_yield){.anchor}

#### fn [get_dividend_yield](../base/trait.BasicAble.html#method.get_dividend_yield){.fn}(&self) -\> [HashMap](https://doc.rust-lang.org/1.91.1/std/collections/hash/map/struct.HashMap.html "struct std::collections::hash::map::HashMap"){.struct}\<[OptionBasicType](../../model/types/struct.OptionBasicType.html "struct optionstratlib::model::types::OptionBasicType"){.struct}\<\'\_\>, &[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\> {#fn-get_dividend_yieldself---hashmapoptionbasictype_-positive .code-header}
:::

::: docblock
Retrieves the dividend yield of a financial option. [Read
more](../base/trait.BasicAble.html#method.get_dividend_yield)
:::
:::::::::::::::::::::::::::::::::::::::::

::: {#impl-BreakEvenable-for-BullCallSpread .section .impl}
[Source](../../../src/optionstratlib/strategies/bull_call_spread.rs.html#327-343){.src
.rightside}[§](#impl-BreakEvenable-for-BullCallSpread){.anchor}

### impl [BreakEvenable](../base/trait.BreakEvenable.html "trait optionstratlib::strategies::base::BreakEvenable"){.trait} for [BullCallSpread](struct.BullCallSpread.html "struct optionstratlib::strategies::bull_call_spread::BullCallSpread"){.struct} {#impl-breakevenable-for-bullcallspread .code-header}
:::

::::::: impl-items
::: {#method.get_break_even_points .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/bull_call_spread.rs.html#328-330){.src
.rightside}[§](#method.get_break_even_points){.anchor}

#### fn [get_break_even_points](../base/trait.BreakEvenable.html#method.get_break_even_points){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<&[Vec](https://doc.rust-lang.org/1.91.1/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\>, [StrategyError](../../error/strategies/enum.StrategyError.html "enum optionstratlib::error::strategies::StrategyError"){.enum}\> {#fn-get_break_even_pointsself---resultvecpositive-strategyerror .code-header}
:::

::: docblock
Retrieves the break-even points for the strategy. [Read
more](../base/trait.BreakEvenable.html#method.get_break_even_points)
:::

::: {#method.update_break_even_points .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/bull_call_spread.rs.html#332-342){.src
.rightside}[§](#method.update_break_even_points){.anchor}

#### fn [update_break_even_points](../base/trait.BreakEvenable.html#method.update_break_even_points){.fn}(&mut self) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[()](https://doc.rust-lang.org/1.91.1/std/primitive.unit.html){.primitive}, [StrategyError](../../error/strategies/enum.StrategyError.html "enum optionstratlib::error::strategies::StrategyError"){.enum}\> {#fn-update_break_even_pointsmut-self---result-strategyerror .code-header}
:::

::: docblock
Updates the break-even points for the strategy. [Read
more](../base/trait.BreakEvenable.html#method.update_break_even_points)
:::
:::::::

::: {#impl-Clone-for-BullCallSpread .section .impl}
[Source](../../../src/optionstratlib/strategies/bull_call_spread.rs.html#78){.src
.rightside}[§](#impl-Clone-for-BullCallSpread){.anchor}

### impl [Clone](https://doc.rust-lang.org/1.91.1/core/clone/trait.Clone.html "trait core::clone::Clone"){.trait} for [BullCallSpread](struct.BullCallSpread.html "struct optionstratlib::strategies::bull_call_spread::BullCallSpread"){.struct} {#impl-clone-for-bullcallspread .code-header}
:::

::::::: impl-items
::: {#method.clone .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/bull_call_spread.rs.html#78){.src
.rightside}[§](#method.clone){.anchor}

#### fn [clone](https://doc.rust-lang.org/1.91.1/core/clone/trait.Clone.html#tymethod.clone){.fn}(&self) -\> [BullCallSpread](struct.BullCallSpread.html "struct optionstratlib::strategies::bull_call_spread::BullCallSpread"){.struct} {#fn-cloneself---bullcallspread .code-header}
:::

::: docblock
Returns a duplicate of the value. [Read
more](https://doc.rust-lang.org/1.91.1/core/clone/trait.Clone.html#tymethod.clone)
:::

::: {#method.clone_from .section .method .trait-impl}
[[1.0.0]{.since title="Stable since Rust version 1.0.0"} ·
[Source](https://doc.rust-lang.org/1.91.1/src/core/clone.rs.html#245-247){.src}]{.rightside}[§](#method.clone_from){.anchor}

#### fn [clone_from](https://doc.rust-lang.org/1.91.1/core/clone/trait.Clone.html#method.clone_from){.fn}(&mut self, source: &Self) {#fn-clone_frommut-self-source-self .code-header}
:::

::: docblock
Performs copy-assignment from `source`. [Read
more](https://doc.rust-lang.org/1.91.1/core/clone/trait.Clone.html#method.clone_from)
:::
:::::::

::: {#impl-ComposeSchema-for-BullCallSpread .section .impl}
[Source](../../../src/optionstratlib/strategies/bull_call_spread.rs.html#78){.src
.rightside}[§](#impl-ComposeSchema-for-BullCallSpread){.anchor}

### impl ComposeSchema for [BullCallSpread](struct.BullCallSpread.html "struct optionstratlib::strategies::bull_call_spread::BullCallSpread"){.struct} {#impl-composeschema-for-bullcallspread .code-header}
:::

:::: impl-items
::: {#method.compose .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/bull_call_spread.rs.html#78){.src
.rightside}[§](#method.compose){.anchor}

#### fn [compose](#tymethod.compose){.fn}(generics: [Vec](https://doc.rust-lang.org/1.91.1/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<[RefOr](../../../utoipa/openapi/enum.RefOr.html "enum utoipa::openapi::RefOr"){.enum}\<[Schema](../../../utoipa/openapi/schema/enum.Schema.html "enum utoipa::openapi::schema::Schema"){.enum}\>\>) -\> [RefOr](../../../utoipa/openapi/enum.RefOr.html "enum utoipa::openapi::RefOr"){.enum}\<[Schema](../../../utoipa/openapi/schema/enum.Schema.html "enum utoipa::openapi::schema::Schema"){.enum}\> {#fn-composegenerics-vecreforschema---reforschema .code-header}
:::
::::

::: {#impl-Debug-for-BullCallSpread .section .impl}
[Source](../../../src/optionstratlib/strategies/bull_call_spread.rs.html#78){.src
.rightside}[§](#impl-Debug-for-BullCallSpread){.anchor}

### impl [Debug](https://doc.rust-lang.org/1.91.1/core/fmt/trait.Debug.html "trait core::fmt::Debug"){.trait} for [BullCallSpread](struct.BullCallSpread.html "struct optionstratlib::strategies::bull_call_spread::BullCallSpread"){.struct} {#impl-debug-for-bullcallspread .code-header}
:::

::::: impl-items
::: {#method.fmt .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/bull_call_spread.rs.html#78){.src
.rightside}[§](#method.fmt){.anchor}

#### fn [fmt](https://doc.rust-lang.org/1.91.1/core/fmt/trait.Debug.html#tymethod.fmt){.fn}(&self, f: &mut [Formatter](https://doc.rust-lang.org/1.91.1/core/fmt/struct.Formatter.html "struct core::fmt::Formatter"){.struct}\<\'\_\>) -\> [Result](https://doc.rust-lang.org/1.91.1/core/fmt/type.Result.html "type core::fmt::Result"){.type} {#fn-fmtself-f-mut-formatter_---result .code-header}
:::

::: docblock
Formats the value using the given formatter. [Read
more](https://doc.rust-lang.org/1.91.1/core/fmt/trait.Debug.html#tymethod.fmt)
:::
:::::

::: {#impl-Default-for-BullCallSpread .section .impl}
[Source](../../../src/optionstratlib/strategies/default.rs.html#14-26){.src
.rightside}[§](#impl-Default-for-BullCallSpread){.anchor}

### impl [Default](https://doc.rust-lang.org/1.91.1/core/default/trait.Default.html "trait core::default::Default"){.trait} for [BullCallSpread](struct.BullCallSpread.html "struct optionstratlib::strategies::bull_call_spread::BullCallSpread"){.struct} {#impl-default-for-bullcallspread .code-header}
:::

::::: impl-items
::: {#method.default .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/default.rs.html#15-25){.src
.rightside}[§](#method.default){.anchor}

#### fn [default](https://doc.rust-lang.org/1.91.1/core/default/trait.Default.html#tymethod.default){.fn}() -\> Self {#fn-default---self .code-header}
:::

::: docblock
Returns the "default value" for a type. [Read
more](https://doc.rust-lang.org/1.91.1/core/default/trait.Default.html#tymethod.default)
:::
:::::

::: {#impl-DeltaNeutrality-for-BullCallSpread .section .impl}
[Source](../../../src/optionstratlib/strategies/bull_call_spread.rs.html#833){.src
.rightside}[§](#impl-DeltaNeutrality-for-BullCallSpread){.anchor}

### impl [DeltaNeutrality](../delta_neutral/trait.DeltaNeutrality.html "trait optionstratlib::strategies::delta_neutral::DeltaNeutrality"){.trait} for [BullCallSpread](struct.BullCallSpread.html "struct optionstratlib::strategies::bull_call_spread::BullCallSpread"){.struct} {#impl-deltaneutrality-for-bullcallspread .code-header}
:::

::::::::::::::::::::: impl-items
::: {#method.delta_neutrality .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/delta_neutral/model.rs.html#346-371){.src
.rightside}[§](#method.delta_neutrality){.anchor}

#### fn [delta_neutrality](../delta_neutral/trait.DeltaNeutrality.html#method.delta_neutrality){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[DeltaInfo](../delta_neutral/struct.DeltaInfo.html "struct optionstratlib::strategies::delta_neutral::DeltaInfo"){.struct}, [GreeksError](../../error/greeks/enum.GreeksError.html "enum optionstratlib::error::greeks::GreeksError"){.enum}\> {#fn-delta_neutralityself---resultdeltainfo-greekserror .code-header}
:::

::: docblock
Calculates the net delta of the strategy and provides detailed
information. [Read
more](../delta_neutral/trait.DeltaNeutrality.html#method.delta_neutrality)
:::

::: {#method.is_delta_neutral .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/delta_neutral/model.rs.html#382-387){.src
.rightside}[§](#method.is_delta_neutral){.anchor}

#### fn [is_delta_neutral](../delta_neutral/trait.DeltaNeutrality.html#method.is_delta_neutral){.fn}(&self) -\> [bool](https://doc.rust-lang.org/1.91.1/std/primitive.bool.html){.primitive} {#fn-is_delta_neutralself---bool .code-header}
:::

::: docblock
Checks if the strategy is delta neutral within the specified threshold.
[Read
more](../delta_neutral/trait.DeltaNeutrality.html#method.is_delta_neutral)
:::

::: {#method.get_atm_strike .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/delta_neutral/model.rs.html#411-413){.src
.rightside}[§](#method.get_atm_strike){.anchor}

#### fn [get_atm_strike](../delta_neutral/trait.DeltaNeutrality.html#method.get_atm_strike){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, [StrategyError](../../error/strategies/enum.StrategyError.html "enum optionstratlib::error::strategies::StrategyError"){.enum}\> {#fn-get_atm_strikeself---resultpositive-strategyerror .code-header}
:::

::: docblock
get_atm_strike [Read
more](../delta_neutral/trait.DeltaNeutrality.html#method.get_atm_strike)
:::

::: {#method.generate_delta_adjustments .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/delta_neutral/model.rs.html#471-575){.src
.rightside}[§](#method.generate_delta_adjustments){.anchor}

#### fn [generate_delta_adjustments](../delta_neutral/trait.DeltaNeutrality.html#method.generate_delta_adjustments){.fn}( &self, net_delta: [Decimal](../../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}, option_delta_per_contract: [Decimal](../../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}, option: &[Options](../../model/option/struct.Options.html "struct optionstratlib::model::option::Options"){.struct}, ) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[DeltaAdjustment](../delta_neutral/enum.DeltaAdjustment.html "enum optionstratlib::strategies::delta_neutral::DeltaAdjustment"){.enum}, [GreeksError](../../error/greeks/enum.GreeksError.html "enum optionstratlib::error::greeks::GreeksError"){.enum}\> {#fn-generate_delta_adjustments-self-net_delta-decimal-option_delta_per_contract-decimal-option-options---resultdeltaadjustment-greekserror .code-header}
:::

::: docblock
Generates delta adjustments based on the given net delta and option
delta per contract. [Read
more](../delta_neutral/trait.DeltaNeutrality.html#method.generate_delta_adjustments)
:::

::: {#method.delta_adjustments .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/delta_neutral/model.rs.html#590-686){.src
.rightside}[§](#method.delta_adjustments){.anchor}

#### fn [delta_adjustments](../delta_neutral/trait.DeltaNeutrality.html#method.delta_adjustments){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Vec](https://doc.rust-lang.org/1.91.1/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<[DeltaAdjustment](../delta_neutral/enum.DeltaAdjustment.html "enum optionstratlib::strategies::delta_neutral::DeltaAdjustment"){.enum}\>, [GreeksError](../../error/greeks/enum.GreeksError.html "enum optionstratlib::error::greeks::GreeksError"){.enum}\> {#fn-delta_adjustmentsself---resultvecdeltaadjustment-greekserror .code-header}
:::

::: docblock
Calculates required position adjustments to maintain delta neutrality
[Read
more](../delta_neutral/trait.DeltaNeutrality.html#method.delta_adjustments)
:::

::: {#method.apply_delta_adjustments .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/delta_neutral/model.rs.html#718-766){.src
.rightside}[§](#method.apply_delta_adjustments){.anchor}

#### fn [apply_delta_adjustments](../delta_neutral/trait.DeltaNeutrality.html#method.apply_delta_adjustments){.fn}( &mut self, action: [Option](https://doc.rust-lang.org/1.91.1/core/option/enum.Option.html "enum core::option::Option"){.enum}\<[Action](../../model/types/enum.Action.html "enum optionstratlib::model::types::Action"){.enum}\>, ) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[()](https://doc.rust-lang.org/1.91.1/std/primitive.unit.html){.primitive}, [StrategyError](../../error/strategies/enum.StrategyError.html "enum optionstratlib::error::strategies::StrategyError"){.enum}\> {#fn-apply_delta_adjustments-mut-self-action-optionaction---result-strategyerror .code-header}
:::

::: docblock
Apply Delta Adjustments [Read
more](../delta_neutral/trait.DeltaNeutrality.html#method.apply_delta_adjustments)
:::

::: {#method.apply_single_adjustment .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/delta_neutral/model.rs.html#798-830){.src
.rightside}[§](#method.apply_single_adjustment){.anchor}

#### fn [apply_single_adjustment](../delta_neutral/trait.DeltaNeutrality.html#method.apply_single_adjustment){.fn}( &mut self, adjustment: &[DeltaAdjustment](../delta_neutral/enum.DeltaAdjustment.html "enum optionstratlib::strategies::delta_neutral::DeltaAdjustment"){.enum}, ) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[()](https://doc.rust-lang.org/1.91.1/std/primitive.unit.html){.primitive}, [StrategyError](../../error/strategies/enum.StrategyError.html "enum optionstratlib::error::strategies::StrategyError"){.enum}\> {#fn-apply_single_adjustment-mut-self-adjustment-deltaadjustment---result-strategyerror .code-header}
:::

::: docblock
Apply Single Adjustment [Read
more](../delta_neutral/trait.DeltaNeutrality.html#method.apply_single_adjustment)
:::

::: {#method.adjust_option_position .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/delta_neutral/model.rs.html#859-880){.src
.rightside}[§](#method.adjust_option_position){.anchor}

#### fn [adjust_option_position](../delta_neutral/trait.DeltaNeutrality.html#method.adjust_option_position){.fn}( &mut self, quantity: [Decimal](../../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}, strike: &[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, option_type: &[OptionStyle](../../model/types/enum.OptionStyle.html "enum optionstratlib::model::types::OptionStyle"){.enum}, side: &[Side](../../model/types/enum.Side.html "enum optionstratlib::model::types::Side"){.enum}, ) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[()](https://doc.rust-lang.org/1.91.1/std/primitive.unit.html){.primitive}, [StrategyError](../../error/strategies/enum.StrategyError.html "enum optionstratlib::error::strategies::StrategyError"){.enum}\> {#fn-adjust_option_position-mut-self-quantity-decimal-strike-positive-option_type-optionstyle-side-side---result-strategyerror .code-header}
:::

::: docblock
Adjust Option Position [Read
more](../delta_neutral/trait.DeltaNeutrality.html#method.adjust_option_position)
:::

::: {#method.trade_from_delta_adjustment .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/delta_neutral/model.rs.html#890-970){.src
.rightside}[§](#method.trade_from_delta_adjustment){.anchor}

#### fn [trade_from_delta_adjustment](../delta_neutral/trait.DeltaNeutrality.html#method.trade_from_delta_adjustment){.fn}( &mut self, action: [Action](../../model/types/enum.Action.html "enum optionstratlib::model::types::Action"){.enum}, ) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Vec](https://doc.rust-lang.org/1.91.1/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<[Trade](../../model/struct.Trade.html "struct optionstratlib::model::Trade"){.struct}\>, [StrategyError](../../error/strategies/enum.StrategyError.html "enum optionstratlib::error::strategies::StrategyError"){.enum}\> {#fn-trade_from_delta_adjustment-mut-self-action-action---resultvectrade-strategyerror .code-header}
:::

::: docblock
Generates a `Trade` object based on the given delta adjustment action.
[Read
more](../delta_neutral/trait.DeltaNeutrality.html#method.trade_from_delta_adjustment)
:::
:::::::::::::::::::::

::: {#impl-Deserialize%3C'de%3E-for-BullCallSpread .section .impl}
[Source](../../../src/optionstratlib/strategies/bull_call_spread.rs.html#78){.src
.rightside}[§](#impl-Deserialize%3C'de%3E-for-BullCallSpread){.anchor}

### impl\<\'de\> [Deserialize](../../../serde_core/de/trait.Deserialize.html "trait serde_core::de::Deserialize"){.trait}\<\'de\> for [BullCallSpread](struct.BullCallSpread.html "struct optionstratlib::strategies::bull_call_spread::BullCallSpread"){.struct} {#implde-deserializede-for-bullcallspread .code-header}
:::

:::::: impl-items
:::: {#method.deserialize .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/bull_call_spread.rs.html#78){.src
.rightside}[§](#method.deserialize){.anchor}

#### fn [deserialize](../../../serde_core/de/trait.Deserialize.html#tymethod.deserialize){.fn}\<\_\_D\>(\_\_deserializer: \_\_D) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<Self, \_\_D::[Error](../../../serde_core/de/trait.Deserializer.html#associatedtype.Error "type serde_core::de::Deserializer::Error"){.associatedtype}\> {#fn-deserialize__d__deserializer-__d---resultself-__derror .code-header}

::: where
where \_\_D:
[Deserializer](../../../serde_core/de/trait.Deserializer.html "trait serde_core::de::Deserializer"){.trait}\<\'de\>,
:::
::::

::: docblock
Deserialize this value from the given Serde deserializer. [Read
more](../../../serde_core/de/trait.Deserialize.html#tymethod.deserialize)
:::
::::::

::: {#impl-Display-for-BullCallSpread .section .impl}
[Source](../../../src/optionstratlib/strategies/bull_call_spread.rs.html#78){.src
.rightside}[§](#impl-Display-for-BullCallSpread){.anchor}

### impl [Display](https://doc.rust-lang.org/1.91.1/core/fmt/trait.Display.html "trait core::fmt::Display"){.trait} for [BullCallSpread](struct.BullCallSpread.html "struct optionstratlib::strategies::bull_call_spread::BullCallSpread"){.struct} {#impl-display-for-bullcallspread .code-header}
:::

::::: impl-items
::: {#method.fmt-1 .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/bull_call_spread.rs.html#78){.src
.rightside}[§](#method.fmt-1){.anchor}

#### fn [fmt](https://doc.rust-lang.org/1.91.1/core/fmt/trait.Display.html#tymethod.fmt){.fn}(&self, f: &mut [Formatter](https://doc.rust-lang.org/1.91.1/core/fmt/struct.Formatter.html "struct core::fmt::Formatter"){.struct}\<\'\_\>) -\> [Result](https://doc.rust-lang.org/1.91.1/core/fmt/type.Result.html "type core::fmt::Result"){.type} {#fn-fmtself-f-mut-formatter_---result-1 .code-header}
:::

::: docblock
Formats the value using the given formatter. [Read
more](https://doc.rust-lang.org/1.91.1/core/fmt/trait.Display.html#tymethod.fmt)
:::
:::::

::: {#impl-Graph-for-BullCallSpread .section .impl}
[Source](../../../src/optionstratlib/strategies/graph.rs.html#360-380){.src
.rightside}[§](#impl-Graph-for-BullCallSpread){.anchor}

### impl [Graph](../../visualization/trait.Graph.html "trait optionstratlib::visualization::Graph"){.trait} for [BullCallSpread](struct.BullCallSpread.html "struct optionstratlib::strategies::bull_call_spread::BullCallSpread"){.struct} {#impl-graph-for-bullcallspread .code-header}
:::

::::::: impl-items
::: {#method.graph_data .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/graph.rs.html#360-380){.src
.rightside}[§](#method.graph_data){.anchor}

#### fn [graph_data](../../visualization/trait.Graph.html#tymethod.graph_data){.fn}(&self) -\> [GraphData](../../visualization/enum.GraphData.html "enum optionstratlib::visualization::GraphData"){.enum} {#fn-graph_dataself---graphdata .code-header}
:::

::: docblock
Return the raw data ready for plotting.
:::

::: {#method.graph_config .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/graph.rs.html#360-380){.src
.rightside}[§](#method.graph_config){.anchor}

#### fn [graph_config](../../visualization/trait.Graph.html#method.graph_config){.fn}(&self) -\> [GraphConfig](../../visualization/struct.GraphConfig.html "struct optionstratlib::visualization::GraphConfig"){.struct} {#fn-graph_configself---graphconfig .code-header}
:::

::: docblock
Optional per‑object configuration overrides.
:::
:::::::

::: {#impl-Greeks-for-BullCallSpread .section .impl}
[Source](../../../src/optionstratlib/strategies/bull_call_spread.rs.html#827-831){.src
.rightside}[§](#impl-Greeks-for-BullCallSpread){.anchor}

### impl [Greeks](../../greeks/trait.Greeks.html "trait optionstratlib::greeks::Greeks"){.trait} for [BullCallSpread](struct.BullCallSpread.html "struct optionstratlib::strategies::bull_call_spread::BullCallSpread"){.struct} {#impl-greeks-for-bullcallspread .code-header}
:::

::::::::::::::::::::: impl-items
::: {#method.get_options .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/bull_call_spread.rs.html#828-830){.src
.rightside}[§](#method.get_options){.anchor}

#### fn [get_options](../../greeks/trait.Greeks.html#tymethod.get_options){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Vec](https://doc.rust-lang.org/1.91.1/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<&[Options](../../model/option/struct.Options.html "struct optionstratlib::model::option::Options"){.struct}\>, [GreeksError](../../error/greeks/enum.GreeksError.html "enum optionstratlib::error::greeks::GreeksError"){.enum}\> {#fn-get_optionsself---resultvecoptions-greekserror .code-header}
:::

::: docblock
Returns a vector of references to the option contracts for which Greeks
will be calculated. [Read
more](../../greeks/trait.Greeks.html#tymethod.get_options)
:::

::: {#method.greeks .section .method .trait-impl}
[Source](../../../src/optionstratlib/greeks/equations.rs.html#127-144){.src
.rightside}[§](#method.greeks){.anchor}

#### fn [greeks](../../greeks/trait.Greeks.html#method.greeks){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Greek](../../greeks/struct.Greek.html "struct optionstratlib::greeks::Greek"){.struct}, [GreeksError](../../error/greeks/enum.GreeksError.html "enum optionstratlib::error::greeks::GreeksError"){.enum}\> {#fn-greeksself---resultgreek-greekserror .code-header}
:::

::: docblock
Calculates and returns all Greeks as a single `Greek` struct. [Read
more](../../greeks/trait.Greeks.html#method.greeks)
:::

::: {#method.delta .section .method .trait-impl}
[Source](../../../src/optionstratlib/greeks/equations.rs.html#154-161){.src
.rightside}[§](#method.delta){.anchor}

#### fn [delta](../../greeks/trait.Greeks.html#method.delta){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Decimal](../../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}, [GreeksError](../../error/greeks/enum.GreeksError.html "enum optionstratlib::error::greeks::GreeksError"){.enum}\> {#fn-deltaself---resultdecimal-greekserror .code-header}
:::

::: docblock
Calculates the aggregate delta value for all options. [Read
more](../../greeks/trait.Greeks.html#method.delta)
:::

::: {#method.gamma .section .method .trait-impl}
[Source](../../../src/optionstratlib/greeks/equations.rs.html#171-178){.src
.rightside}[§](#method.gamma){.anchor}

#### fn [gamma](../../greeks/trait.Greeks.html#method.gamma){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Decimal](../../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}, [GreeksError](../../error/greeks/enum.GreeksError.html "enum optionstratlib::error::greeks::GreeksError"){.enum}\> {#fn-gammaself---resultdecimal-greekserror .code-header}
:::

::: docblock
Calculates the aggregate gamma value for all options. [Read
more](../../greeks/trait.Greeks.html#method.gamma)
:::

::: {#method.theta .section .method .trait-impl}
[Source](../../../src/optionstratlib/greeks/equations.rs.html#188-195){.src
.rightside}[§](#method.theta){.anchor}

#### fn [theta](../../greeks/trait.Greeks.html#method.theta){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Decimal](../../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}, [GreeksError](../../error/greeks/enum.GreeksError.html "enum optionstratlib::error::greeks::GreeksError"){.enum}\> {#fn-thetaself---resultdecimal-greekserror .code-header}
:::

::: docblock
Calculates the aggregate theta value for all options. [Read
more](../../greeks/trait.Greeks.html#method.theta)
:::

::: {#method.vega .section .method .trait-impl}
[Source](../../../src/optionstratlib/greeks/equations.rs.html#205-212){.src
.rightside}[§](#method.vega){.anchor}

#### fn [vega](../../greeks/trait.Greeks.html#method.vega){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Decimal](../../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}, [GreeksError](../../error/greeks/enum.GreeksError.html "enum optionstratlib::error::greeks::GreeksError"){.enum}\> {#fn-vegaself---resultdecimal-greekserror .code-header}
:::

::: docblock
Calculates the aggregate vega value for all options. [Read
more](../../greeks/trait.Greeks.html#method.vega)
:::

::: {#method.rho .section .method .trait-impl}
[Source](../../../src/optionstratlib/greeks/equations.rs.html#222-229){.src
.rightside}[§](#method.rho){.anchor}

#### fn [rho](../../greeks/trait.Greeks.html#method.rho){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Decimal](../../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}, [GreeksError](../../error/greeks/enum.GreeksError.html "enum optionstratlib::error::greeks::GreeksError"){.enum}\> {#fn-rhoself---resultdecimal-greekserror .code-header}
:::

::: docblock
Calculates the aggregate rho value for all options. [Read
more](../../greeks/trait.Greeks.html#method.rho)
:::

::: {#method.rho_d .section .method .trait-impl}
[Source](../../../src/optionstratlib/greeks/equations.rs.html#239-246){.src
.rightside}[§](#method.rho_d){.anchor}

#### fn [rho_d](../../greeks/trait.Greeks.html#method.rho_d){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Decimal](../../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}, [GreeksError](../../error/greeks/enum.GreeksError.html "enum optionstratlib::error::greeks::GreeksError"){.enum}\> {#fn-rho_dself---resultdecimal-greekserror .code-header}
:::

::: docblock
Calculates the aggregate rho_d value for all options. [Read
more](../../greeks/trait.Greeks.html#method.rho_d)
:::

::: {#method.alpha .section .method .trait-impl}
[Source](../../../src/optionstratlib/greeks/equations.rs.html#256-263){.src
.rightside}[§](#method.alpha){.anchor}

#### fn [alpha](../../greeks/trait.Greeks.html#method.alpha){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Decimal](../../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}, [GreeksError](../../error/greeks/enum.GreeksError.html "enum optionstratlib::error::greeks::GreeksError"){.enum}\> {#fn-alphaself---resultdecimal-greekserror .code-header}
:::

::: docblock
Calculates the aggregate alpha value for all options. [Read
more](../../greeks/trait.Greeks.html#method.alpha)
:::
:::::::::::::::::::::

::: {#impl-Optimizable-for-BullCallSpread .section .impl}
[Source](../../../src/optionstratlib/strategies/bull_call_spread.rs.html#647-751){.src
.rightside}[§](#impl-Optimizable-for-BullCallSpread){.anchor}

### impl [Optimizable](../base/trait.Optimizable.html "trait optionstratlib::strategies::base::Optimizable"){.trait} for [BullCallSpread](struct.BullCallSpread.html "struct optionstratlib::strategies::bull_call_spread::BullCallSpread"){.struct} {#impl-optimizable-for-bullcallspread .code-header}
:::

::::::::::::::::::: impl-items
::: {#associatedtype.Strategy .section .associatedtype .trait-impl}
[Source](../../../src/optionstratlib/strategies/bull_call_spread.rs.html#648){.src
.rightside}[§](#associatedtype.Strategy){.anchor}

#### type [Strategy](../base/trait.Optimizable.html#associatedtype.Strategy){.associatedtype} = [BullCallSpread](struct.BullCallSpread.html "struct optionstratlib::strategies::bull_call_spread::BullCallSpread"){.struct} {#type-strategy-bullcallspread .code-header}
:::

::: docblock
The type of strategy associated with this optimization.
:::

::: {#method.filter_combinations .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/bull_call_spread.rs.html#650-687){.src
.rightside}[§](#method.filter_combinations){.anchor}

#### fn [filter_combinations](../base/trait.Optimizable.html#method.filter_combinations){.fn}\<\'a\>( &\'a self, option_chain: &\'a [OptionChain](../../chains/chain/struct.OptionChain.html "struct optionstratlib::chains::chain::OptionChain"){.struct}, side: [FindOptimalSide](../utils/enum.FindOptimalSide.html "enum optionstratlib::strategies::utils::FindOptimalSide"){.enum}, ) -\> impl [Iterator](https://doc.rust-lang.org/1.91.1/core/iter/traits/iterator/trait.Iterator.html "trait core::iter::traits::iterator::Iterator"){.trait}\<Item = [OptionDataGroup](../../chains/utils/enum.OptionDataGroup.html "enum optionstratlib::chains::utils::OptionDataGroup"){.enum}\<\'a\>\> {#fn-filter_combinationsa-a-self-option_chain-a-optionchain-side-findoptimalside---impl-iteratoritem-optiondatagroupa .code-header}
:::

::: docblock
Filters and generates combinations of options data from the given
`OptionChain`. [Read
more](../base/trait.Optimizable.html#method.filter_combinations)
:::

::: {#method.find_optimal .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/bull_call_spread.rs.html#689-724){.src
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
[Source](../../../src/optionstratlib/strategies/bull_call_spread.rs.html#726-750){.src
.rightside}[§](#method.create_strategy){.anchor}

#### fn [create_strategy](../base/trait.Optimizable.html#method.create_strategy){.fn}( &self, chain: &[OptionChain](../../chains/chain/struct.OptionChain.html "struct optionstratlib::chains::chain::OptionChain"){.struct}, legs: &[StrategyLegs](../../chains/enum.StrategyLegs.html "enum optionstratlib::chains::StrategyLegs"){.enum}\<\'\_\>, ) -\> Self::[Strategy](../base/trait.Optimizable.html#associatedtype.Strategy "type optionstratlib::strategies::base::Optimizable::Strategy"){.associatedtype} {#fn-create_strategy-self-chain-optionchain-legs-strategylegs_---selfstrategy .code-header}
:::

::: docblock
Creates a new strategy from the given `OptionChain` and `StrategyLegs`.
The default implementation panics. Specific strategies must override
this. [Read more](../base/trait.Optimizable.html#method.create_strategy)
:::

::: {#method.get_best_ratio .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/base.rs.html#1168-1170){.src
.rightside}[§](#method.get_best_ratio){.anchor}

#### fn [get_best_ratio](../base/trait.Optimizable.html#method.get_best_ratio){.fn}(&mut self, option_chain: &[OptionChain](../../chains/chain/struct.OptionChain.html "struct optionstratlib::chains::chain::OptionChain"){.struct}, side: [FindOptimalSide](../utils/enum.FindOptimalSide.html "enum optionstratlib::strategies::utils::FindOptimalSide"){.enum}) {#fn-get_best_ratiomut-self-option_chain-optionchain-side-findoptimalside .code-header}
:::

::: docblock
Finds the best ratio-based strategy within the given `OptionChain`.
[Read more](../base/trait.Optimizable.html#method.get_best_ratio)
:::

::: {#method.get_best_area .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/base.rs.html#1177-1179){.src
.rightside}[§](#method.get_best_area){.anchor}

#### fn [get_best_area](../base/trait.Optimizable.html#method.get_best_area){.fn}(&mut self, option_chain: &[OptionChain](../../chains/chain/struct.OptionChain.html "struct optionstratlib::chains::chain::OptionChain"){.struct}, side: [FindOptimalSide](../utils/enum.FindOptimalSide.html "enum optionstratlib::strategies::utils::FindOptimalSide"){.enum}) {#fn-get_best_areamut-self-option_chain-optionchain-side-findoptimalside .code-header}
:::

::: docblock
Finds the best area-based strategy within the given `OptionChain`. [Read
more](../base/trait.Optimizable.html#method.get_best_area)
:::

::: {#method.is_valid_optimal_option .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/base.rs.html#1239-1259){.src
.rightside}[§](#method.is_valid_optimal_option){.anchor}

#### fn [is_valid_optimal_option](../base/trait.Optimizable.html#method.is_valid_optimal_option){.fn}( &self, option: &[OptionData](../../chains/struct.OptionData.html "struct optionstratlib::chains::OptionData"){.struct}, side: &[FindOptimalSide](../utils/enum.FindOptimalSide.html "enum optionstratlib::strategies::utils::FindOptimalSide"){.enum}, ) -\> [bool](https://doc.rust-lang.org/1.91.1/std/primitive.bool.html){.primitive} {#fn-is_valid_optimal_option-self-option-optiondata-side-findoptimalside---bool .code-header}
:::

::: docblock
Checks if a long option is valid based on the given criteria. [Read
more](../base/trait.Optimizable.html#method.is_valid_optimal_option)
:::

::: {#method.are_valid_legs .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/base.rs.html#1266-1274){.src
.rightside}[§](#method.are_valid_legs){.anchor}

#### fn [are_valid_legs](../base/trait.Optimizable.html#method.are_valid_legs){.fn}(&self, legs: &[StrategyLegs](../../chains/enum.StrategyLegs.html "enum optionstratlib::chains::StrategyLegs"){.enum}\<\'\_\>) -\> [bool](https://doc.rust-lang.org/1.91.1/std/primitive.bool.html){.primitive} {#fn-are_valid_legsself-legs-strategylegs_---bool .code-header}
:::

::: docblock
Checks if the prices in the given `StrategyLegs` are valid. Assumes the
strategy consists of one long call and one short call by default. [Read
more](../base/trait.Optimizable.html#method.are_valid_legs)
:::
:::::::::::::::::::

::: {#impl-PnLCalculator-for-BullCallSpread .section .impl}
[Source](../../../src/optionstratlib/strategies/bull_call_spread.rs.html#835-861){.src
.rightside}[§](#impl-PnLCalculator-for-BullCallSpread){.anchor}

### impl [PnLCalculator](../../pnl/trait.PnLCalculator.html "trait optionstratlib::pnl::PnLCalculator"){.trait} for [BullCallSpread](struct.BullCallSpread.html "struct optionstratlib::strategies::bull_call_spread::BullCallSpread"){.struct} {#impl-pnlcalculator-for-bullcallspread .code-header}
:::

::::::::::: impl-items
::: {#method.calculate_pnl .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/bull_call_spread.rs.html#836-848){.src
.rightside}[§](#method.calculate_pnl){.anchor}

#### fn [calculate_pnl](../../pnl/trait.PnLCalculator.html#tymethod.calculate_pnl){.fn}( &self, market_price: &[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, expiration_date: [ExpirationDate](../../model/enum.ExpirationDate.html "enum optionstratlib::model::ExpirationDate"){.enum}, implied_volatility: &[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, ) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[PnL](../../pnl/utils/struct.PnL.html "struct optionstratlib::pnl::utils::PnL"){.struct}, [PricingError](../../error/pricing/enum.PricingError.html "enum optionstratlib::error::pricing::PricingError"){.enum}\> {#fn-calculate_pnl-self-market_price-positive-expiration_date-expirationdate-implied_volatility-positive---resultpnl-pricingerror .code-header}
:::

::: docblock
Calculates the current PnL based on market conditions. [Read
more](../../pnl/trait.PnLCalculator.html#tymethod.calculate_pnl)
:::

::: {#method.calculate_pnl_at_expiration .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/bull_call_spread.rs.html#850-860){.src
.rightside}[§](#method.calculate_pnl_at_expiration){.anchor}

#### fn [calculate_pnl_at_expiration](../../pnl/trait.PnLCalculator.html#tymethod.calculate_pnl_at_expiration){.fn}( &self, underlying_price: &[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, ) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[PnL](../../pnl/utils/struct.PnL.html "struct optionstratlib::pnl::utils::PnL"){.struct}, [PricingError](../../error/pricing/enum.PricingError.html "enum optionstratlib::error::pricing::PricingError"){.enum}\> {#fn-calculate_pnl_at_expiration-self-underlying_price-positive---resultpnl-pricingerror .code-header}
:::

::: docblock
Calculates the PnL at the expiration of the instrument. [Read
more](../../pnl/trait.PnLCalculator.html#tymethod.calculate_pnl_at_expiration)
:::

::: {#method.adjustments_pnl .section .method .trait-impl}
[Source](../../../src/optionstratlib/pnl/traits.rs.html#70-72){.src
.rightside}[§](#method.adjustments_pnl){.anchor}

#### fn [adjustments_pnl](../../pnl/trait.PnLCalculator.html#method.adjustments_pnl){.fn}( &self, \_adjustments: &[DeltaAdjustment](../delta_neutral/enum.DeltaAdjustment.html "enum optionstratlib::strategies::delta_neutral::DeltaAdjustment"){.enum}, ) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[PnL](../../pnl/utils/struct.PnL.html "struct optionstratlib::pnl::utils::PnL"){.struct}, [PricingError](../../error/pricing/enum.PricingError.html "enum optionstratlib::error::pricing::PricingError"){.enum}\> {#fn-adjustments_pnl-self-_adjustments-deltaadjustment---resultpnl-pricingerror .code-header}
:::

::: docblock
Calculates the Profit and Loss (PnL) for a series of delta adjustments
in a trading strategy. [Read
more](../../pnl/trait.PnLCalculator.html#method.adjustments_pnl)
:::

::: {#method.diff_position_pnl .section .method .trait-impl}
[Source](../../../src/optionstratlib/pnl/traits.rs.html#95-97){.src
.rightside}[§](#method.diff_position_pnl){.anchor}

#### fn [diff_position_pnl](../../pnl/trait.PnLCalculator.html#method.diff_position_pnl){.fn}(&self, \_position: &[Position](../../model/position/struct.Position.html "struct optionstratlib::model::position::Position"){.struct}) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[PnL](../../pnl/utils/struct.PnL.html "struct optionstratlib::pnl::utils::PnL"){.struct}, [PricingError](../../error/pricing/enum.PricingError.html "enum optionstratlib::error::pricing::PricingError"){.enum}\> {#fn-diff_position_pnlself-_position-position---resultpnl-pricingerror .code-header}
:::

::: docblock
Calculates the profit and loss (PnL) for a given trading position. [Read
more](../../pnl/trait.PnLCalculator.html#method.diff_position_pnl)
:::
:::::::::::

::: {#impl-Positionable-for-BullCallSpread .section .impl}
[Source](../../../src/optionstratlib/strategies/bull_call_spread.rs.html#345-449){.src
.rightside}[§](#impl-Positionable-for-BullCallSpread){.anchor}

### impl [Positionable](../base/trait.Positionable.html "trait optionstratlib::strategies::base::Positionable"){.trait} for [BullCallSpread](struct.BullCallSpread.html "struct optionstratlib::strategies::bull_call_spread::BullCallSpread"){.struct} {#impl-positionable-for-bullcallspread .code-header}
:::

::::::::::::::::::: impl-items
::: {#method.get_position .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/bull_call_spread.rs.html#373-399){.src
.rightside}[§](#method.get_position){.anchor}

#### fn [get_position](../base/trait.Positionable.html#method.get_position){.fn}( &mut self, option_style: &[OptionStyle](../../model/types/enum.OptionStyle.html "enum optionstratlib::model::types::OptionStyle"){.enum}, side: &[Side](../../model/types/enum.Side.html "enum optionstratlib::model::types::Side"){.enum}, strike: &[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, ) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Vec](https://doc.rust-lang.org/1.91.1/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<&mut [Position](../../model/position/struct.Position.html "struct optionstratlib::model::position::Position"){.struct}\>, [PositionError](../../error/position/enum.PositionError.html "enum optionstratlib::error::position::PositionError"){.enum}\> {#fn-get_position-mut-self-option_style-optionstyle-side-side-strike-positive---resultvecmut-position-positionerror .code-header}
:::

::: docblock
Gets mutable positions matching the specified criteria from the
strategy.

##### [§](#arguments-1){.doc-anchor}Arguments

- `option_style` - The style of the option (Put/Call)
- `side` - The side of the position (Long/Short)
- `strike` - The strike price of the option

##### [§](#returns-1){.doc-anchor}Returns

- `Ok(Vec<&mut Position>)` - A vector containing mutable references to
  matching positions
- `Err(PositionError)` - If there was an error retrieving positions
:::

::: {#method.modify_position .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/bull_call_spread.rs.html#409-448){.src
.rightside}[§](#method.modify_position){.anchor}

#### fn [modify_position](../base/trait.Positionable.html#method.modify_position){.fn}(&mut self, position: &[Position](../../model/position/struct.Position.html "struct optionstratlib::model::position::Position"){.struct}) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[()](https://doc.rust-lang.org/1.91.1/std/primitive.unit.html){.primitive}, [PositionError](../../error/position/enum.PositionError.html "enum optionstratlib::error::position::PositionError"){.enum}\> {#fn-modify_positionmut-self-position-position---result-positionerror .code-header}
:::

::: docblock
Modifies an existing position in the strategy.

##### [§](#arguments-2){.doc-anchor}Arguments

- `position` - The new position data to update

##### [§](#returns-2){.doc-anchor}Returns

- `Ok(())` if position was successfully modified
- `Err(PositionError)` if position was not found or validation failed
:::

::: {#method.add_position .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/bull_call_spread.rs.html#346-357){.src
.rightside}[§](#method.add_position){.anchor}

#### fn [add_position](../base/trait.Positionable.html#method.add_position){.fn}(&mut self, position: &[Position](../../model/position/struct.Position.html "struct optionstratlib::model::position::Position"){.struct}) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[()](https://doc.rust-lang.org/1.91.1/std/primitive.unit.html){.primitive}, [PositionError](../../error/position/enum.PositionError.html "enum optionstratlib::error::position::PositionError"){.enum}\> {#fn-add_positionmut-self-position-position---result-positionerror .code-header}
:::

::: docblock
Adds a position to the strategy. [Read
more](../base/trait.Positionable.html#method.add_position)
:::

::: {#method.get_positions .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/bull_call_spread.rs.html#359-361){.src
.rightside}[§](#method.get_positions){.anchor}

#### fn [get_positions](../base/trait.Positionable.html#method.get_positions){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Vec](https://doc.rust-lang.org/1.91.1/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<&[Position](../../model/position/struct.Position.html "struct optionstratlib::model::position::Position"){.struct}\>, [PositionError](../../error/position/enum.PositionError.html "enum optionstratlib::error::position::PositionError"){.enum}\> {#fn-get_positionsself---resultvecposition-positionerror .code-header}
:::

::: docblock
Retrieves all positions held by the strategy. [Read
more](../base/trait.Positionable.html#method.get_positions)
:::

::: {#method.get_position_unique .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/base.rs.html#1368-1374){.src
.rightside}[§](#method.get_position_unique){.anchor}

#### fn [get_position_unique](../base/trait.Positionable.html#method.get_position_unique){.fn}( &mut self, \_option_style: &[OptionStyle](../../model/types/enum.OptionStyle.html "enum optionstratlib::model::types::OptionStyle"){.enum}, \_side: &[Side](../../model/types/enum.Side.html "enum optionstratlib::model::types::Side"){.enum}, ) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<&mut [Position](../../model/position/struct.Position.html "struct optionstratlib::model::position::Position"){.struct}, [PositionError](../../error/position/enum.PositionError.html "enum optionstratlib::error::position::PositionError"){.enum}\> {#fn-get_position_unique-mut-self-_option_style-optionstyle-_side-side---resultmut-position-positionerror .code-header}
:::

::: docblock
Retrieves a unique position based on the given option style and side.
[Read more](../base/trait.Positionable.html#method.get_position_unique)
:::

::: {#method.get_option_unique .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/base.rs.html#1404-1410){.src
.rightside}[§](#method.get_option_unique){.anchor}

#### fn [get_option_unique](../base/trait.Positionable.html#method.get_option_unique){.fn}( &mut self, \_option_style: &[OptionStyle](../../model/types/enum.OptionStyle.html "enum optionstratlib::model::types::OptionStyle"){.enum}, \_side: &[Side](../../model/types/enum.Side.html "enum optionstratlib::model::types::Side"){.enum}, ) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<&mut [Options](../../model/option/struct.Options.html "struct optionstratlib::model::option::Options"){.struct}, [PositionError](../../error/position/enum.PositionError.html "enum optionstratlib::error::position::PositionError"){.enum}\> {#fn-get_option_unique-mut-self-_option_style-optionstyle-_side-side---resultmut-options-positionerror .code-header}
:::

::: docblock
Retrieves a unique option based on the given style and side. [Read
more](../base/trait.Positionable.html#method.get_option_unique)
:::

::: {#method.replace_position .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/base.rs.html#1443-1445){.src
.rightside}[§](#method.replace_position){.anchor}

#### fn [replace_position](../base/trait.Positionable.html#method.replace_position){.fn}( &mut self, \_position: &[Position](../../model/position/struct.Position.html "struct optionstratlib::model::position::Position"){.struct}, ) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[()](https://doc.rust-lang.org/1.91.1/std/primitive.unit.html){.primitive}, [PositionError](../../error/position/enum.PositionError.html "enum optionstratlib::error::position::PositionError"){.enum}\> {#fn-replace_position-mut-self-_position-position---result-positionerror .code-header}
:::

::: docblock
Attempts to replace the current position with a new position. [Read
more](../base/trait.Positionable.html#method.replace_position)
:::

::: {#method.valid_premium_for_shorts .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/base.rs.html#1467-1479){.src
.rightside}[§](#method.valid_premium_for_shorts){.anchor}

#### fn [valid_premium_for_shorts](../base/trait.Positionable.html#method.valid_premium_for_shorts){.fn}(&self, min_premium: &[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}) -\> [bool](https://doc.rust-lang.org/1.91.1/std/primitive.bool.html){.primitive} {#fn-valid_premium_for_shortsself-min_premium-positive---bool .code-header}
:::

::: docblock
Checks if all short positions have a net premium received that meets or
exceeds a specified minimum. [Read
more](../base/trait.Positionable.html#method.valid_premium_for_shorts)
:::
:::::::::::::::::::

::: {#impl-ProbabilityAnalysis-for-BullCallSpread .section .impl}
[Source](../../../src/optionstratlib/strategies/bull_call_spread.rs.html#763-825){.src
.rightside}[§](#impl-ProbabilityAnalysis-for-BullCallSpread){.anchor}

### impl [ProbabilityAnalysis](../probabilities/trait.ProbabilityAnalysis.html "trait optionstratlib::strategies::probabilities::ProbabilityAnalysis"){.trait} for [BullCallSpread](struct.BullCallSpread.html "struct optionstratlib::strategies::bull_call_spread::BullCallSpread"){.struct} {#impl-probabilityanalysis-for-bullcallspread .code-header}
:::

::::::::::::::::: impl-items
::: {#method.get_profit_ranges .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/bull_call_spread.rs.html#764-793){.src
.rightside}[§](#method.get_profit_ranges){.anchor}

#### fn [get_profit_ranges](../probabilities/trait.ProbabilityAnalysis.html#tymethod.get_profit_ranges){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Vec](https://doc.rust-lang.org/1.91.1/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<[ProfitLossRange](../../model/struct.ProfitLossRange.html "struct optionstratlib::model::ProfitLossRange"){.struct}\>, [ProbabilityError](../../error/probability/enum.ProbabilityError.html "enum optionstratlib::error::probability::ProbabilityError"){.enum}\> {#fn-get_profit_rangesself---resultvecprofitlossrange-probabilityerror .code-header}
:::

::: docblock
Get the price ranges that would result in a profit [Read
more](../probabilities/trait.ProbabilityAnalysis.html#tymethod.get_profit_ranges)
:::

::: {#method.get_loss_ranges .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/bull_call_spread.rs.html#795-824){.src
.rightside}[§](#method.get_loss_ranges){.anchor}

#### fn [get_loss_ranges](../probabilities/trait.ProbabilityAnalysis.html#tymethod.get_loss_ranges){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Vec](https://doc.rust-lang.org/1.91.1/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<[ProfitLossRange](../../model/struct.ProfitLossRange.html "struct optionstratlib::model::ProfitLossRange"){.struct}\>, [ProbabilityError](../../error/probability/enum.ProbabilityError.html "enum optionstratlib::error::probability::ProbabilityError"){.enum}\> {#fn-get_loss_rangesself---resultvecprofitlossrange-probabilityerror .code-header}
:::

::: docblock
Get Profit/Loss Ranges [Read
more](../probabilities/trait.ProbabilityAnalysis.html#tymethod.get_loss_ranges)
:::

::: {#method.analyze_probabilities .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/probabilities/core.rs.html#65-102){.src
.rightside}[§](#method.analyze_probabilities){.anchor}

#### fn [analyze_probabilities](../probabilities/trait.ProbabilityAnalysis.html#method.analyze_probabilities){.fn}( &self, volatility_adj: [Option](https://doc.rust-lang.org/1.91.1/core/option/enum.Option.html "enum core::option::Option"){.enum}\<[VolatilityAdjustment](../probabilities/struct.VolatilityAdjustment.html "struct optionstratlib::strategies::probabilities::VolatilityAdjustment"){.struct}\>, trend: [Option](https://doc.rust-lang.org/1.91.1/core/option/enum.Option.html "enum core::option::Option"){.enum}\<[PriceTrend](../probabilities/struct.PriceTrend.html "struct optionstratlib::strategies::probabilities::PriceTrend"){.struct}\>, ) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[StrategyProbabilityAnalysis](../probabilities/struct.StrategyProbabilityAnalysis.html "struct optionstratlib::strategies::probabilities::StrategyProbabilityAnalysis"){.struct}, [ProbabilityError](../../error/probability/enum.ProbabilityError.html "enum optionstratlib::error::probability::ProbabilityError"){.enum}\> {#fn-analyze_probabilities-self-volatility_adj-optionvolatilityadjustment-trend-optionpricetrend---resultstrategyprobabilityanalysis-probabilityerror .code-header}
:::

::: docblock
Calculate probability analysis for a strategy [Read
more](../probabilities/trait.ProbabilityAnalysis.html#method.analyze_probabilities)
:::

::: {#method.expected_value .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/probabilities/core.rs.html#129-190){.src
.rightside}[§](#method.expected_value){.anchor}

#### fn [expected_value](../probabilities/trait.ProbabilityAnalysis.html#method.expected_value){.fn}( &self, volatility_adj: [Option](https://doc.rust-lang.org/1.91.1/core/option/enum.Option.html "enum core::option::Option"){.enum}\<[VolatilityAdjustment](../probabilities/struct.VolatilityAdjustment.html "struct optionstratlib::strategies::probabilities::VolatilityAdjustment"){.struct}\>, trend: [Option](https://doc.rust-lang.org/1.91.1/core/option/enum.Option.html "enum core::option::Option"){.enum}\<[PriceTrend](../probabilities/struct.PriceTrend.html "struct optionstratlib::strategies::probabilities::PriceTrend"){.struct}\>, ) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, [ProbabilityError](../../error/probability/enum.ProbabilityError.html "enum optionstratlib::error::probability::ProbabilityError"){.enum}\> {#fn-expected_value-self-volatility_adj-optionvolatilityadjustment-trend-optionpricetrend---resultpositive-probabilityerror .code-header}
:::

::: docblock
This function calculates the expected value of an option strategy based
on an underlying price, volatility adjustments, and price trends. [Read
more](../probabilities/trait.ProbabilityAnalysis.html#method.expected_value)
:::

::: {#method.probability_of_profit .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/probabilities/core.rs.html#205-227){.src
.rightside}[§](#method.probability_of_profit){.anchor}

#### fn [probability_of_profit](../probabilities/trait.ProbabilityAnalysis.html#method.probability_of_profit){.fn}( &self, volatility_adj: [Option](https://doc.rust-lang.org/1.91.1/core/option/enum.Option.html "enum core::option::Option"){.enum}\<[VolatilityAdjustment](../probabilities/struct.VolatilityAdjustment.html "struct optionstratlib::strategies::probabilities::VolatilityAdjustment"){.struct}\>, trend: [Option](https://doc.rust-lang.org/1.91.1/core/option/enum.Option.html "enum core::option::Option"){.enum}\<[PriceTrend](../probabilities/struct.PriceTrend.html "struct optionstratlib::strategies::probabilities::PriceTrend"){.struct}\>, ) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, [ProbabilityError](../../error/probability/enum.ProbabilityError.html "enum optionstratlib::error::probability::ProbabilityError"){.enum}\> {#fn-probability_of_profit-self-volatility_adj-optionvolatilityadjustment-trend-optionpricetrend---resultpositive-probabilityerror .code-header}
:::

::: docblock
Calculate probability of profit [Read
more](../probabilities/trait.ProbabilityAnalysis.html#method.probability_of_profit)
:::

::: {#method.probability_of_loss .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/probabilities/core.rs.html#242-264){.src
.rightside}[§](#method.probability_of_loss){.anchor}

#### fn [probability_of_loss](../probabilities/trait.ProbabilityAnalysis.html#method.probability_of_loss){.fn}( &self, volatility_adj: [Option](https://doc.rust-lang.org/1.91.1/core/option/enum.Option.html "enum core::option::Option"){.enum}\<[VolatilityAdjustment](../probabilities/struct.VolatilityAdjustment.html "struct optionstratlib::strategies::probabilities::VolatilityAdjustment"){.struct}\>, trend: [Option](https://doc.rust-lang.org/1.91.1/core/option/enum.Option.html "enum core::option::Option"){.enum}\<[PriceTrend](../probabilities/struct.PriceTrend.html "struct optionstratlib::strategies::probabilities::PriceTrend"){.struct}\>, ) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, [ProbabilityError](../../error/probability/enum.ProbabilityError.html "enum optionstratlib::error::probability::ProbabilityError"){.enum}\> {#fn-probability_of_loss-self-volatility_adj-optionvolatilityadjustment-trend-optionpricetrend---resultpositive-probabilityerror .code-header}
:::

::: docblock
Calculate probability of loss [Read
more](../probabilities/trait.ProbabilityAnalysis.html#method.probability_of_loss)
:::

::: {#method.calculate_extreme_probabilities .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/probabilities/core.rs.html#280-324){.src
.rightside}[§](#method.calculate_extreme_probabilities){.anchor}

#### fn [calculate_extreme_probabilities](../probabilities/trait.ProbabilityAnalysis.html#method.calculate_extreme_probabilities){.fn}( &self, volatility_adj: [Option](https://doc.rust-lang.org/1.91.1/core/option/enum.Option.html "enum core::option::Option"){.enum}\<[VolatilityAdjustment](../probabilities/struct.VolatilityAdjustment.html "struct optionstratlib::strategies::probabilities::VolatilityAdjustment"){.struct}\>, trend: [Option](https://doc.rust-lang.org/1.91.1/core/option/enum.Option.html "enum core::option::Option"){.enum}\<[PriceTrend](../probabilities/struct.PriceTrend.html "struct optionstratlib::strategies::probabilities::PriceTrend"){.struct}\>, ) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<([Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, [Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}), [ProbabilityError](../../error/probability/enum.ProbabilityError.html "enum optionstratlib::error::probability::ProbabilityError"){.enum}\> {#fn-calculate_extreme_probabilities-self-volatility_adj-optionvolatilityadjustment-trend-optionpricetrend---resultpositive-positive-probabilityerror .code-header}
:::

::: docblock
Calculate extreme probabilities (max profit and max loss) [Read
more](../probabilities/trait.ProbabilityAnalysis.html#method.calculate_extreme_probabilities)
:::
:::::::::::::::::

::: {#impl-Profit-for-BullCallSpread .section .impl}
[Source](../../../src/optionstratlib/strategies/bull_call_spread.rs.html#753-761){.src
.rightside}[§](#impl-Profit-for-BullCallSpread){.anchor}

### impl [Profit](../../pricing/trait.Profit.html "trait optionstratlib::pricing::Profit"){.trait} for [BullCallSpread](struct.BullCallSpread.html "struct optionstratlib::strategies::bull_call_spread::BullCallSpread"){.struct} {#impl-profit-for-bullcallspread .code-header}
:::

::::::: impl-items
::: {#method.calculate_profit_at .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/bull_call_spread.rs.html#754-760){.src
.rightside}[§](#method.calculate_profit_at){.anchor}

#### fn [calculate_profit_at](../../pricing/trait.Profit.html#tymethod.calculate_profit_at){.fn}(&self, price: &[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Decimal](../../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}, [PricingError](../../error/pricing/enum.PricingError.html "enum optionstratlib::error::pricing::PricingError"){.enum}\> {#fn-calculate_profit_atself-price-positive---resultdecimal-pricingerror .code-header}
:::

::: docblock
Calculates the profit at a specified price. [Read
more](../../pricing/trait.Profit.html#tymethod.calculate_profit_at)
:::

::: {#method.get_point_at_price .section .method .trait-impl}
[Source](../../../src/optionstratlib/pricing/payoff.rs.html#219-225){.src
.rightside}[§](#method.get_point_at_price){.anchor}

#### fn [get_point_at_price](../../pricing/trait.Profit.html#method.get_point_at_price){.fn}( &self, \_price: &[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, ) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<([Decimal](../../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}, [Decimal](../../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}), [PricingError](../../error/pricing/enum.PricingError.html "enum optionstratlib::error::pricing::PricingError"){.enum}\> {#fn-get_point_at_price-self-_price-positive---resultdecimal-decimal-pricingerror .code-header}
:::

::: docblock
Creates a chart point representation of the profit at the given price.
[Read more](../../pricing/trait.Profit.html#method.get_point_at_price)
:::
:::::::

::: {#impl-Serialize-for-BullCallSpread .section .impl}
[Source](../../../src/optionstratlib/strategies/bull_call_spread.rs.html#78){.src
.rightside}[§](#impl-Serialize-for-BullCallSpread){.anchor}

### impl [Serialize](../../../serde_core/ser/trait.Serialize.html "trait serde_core::ser::Serialize"){.trait} for [BullCallSpread](struct.BullCallSpread.html "struct optionstratlib::strategies::bull_call_spread::BullCallSpread"){.struct} {#impl-serialize-for-bullcallspread .code-header}
:::

:::::: impl-items
:::: {#method.serialize .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/bull_call_spread.rs.html#78){.src
.rightside}[§](#method.serialize){.anchor}

#### fn [serialize](../../../serde_core/ser/trait.Serialize.html#tymethod.serialize){.fn}\<\_\_S\>(&self, \_\_serializer: \_\_S) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<\_\_S::[Ok](../../../serde_core/ser/trait.Serializer.html#associatedtype.Ok "type serde_core::ser::Serializer::Ok"){.associatedtype}, \_\_S::[Error](../../../serde_core/ser/trait.Serializer.html#associatedtype.Error "type serde_core::ser::Serializer::Error"){.associatedtype}\> {#fn-serialize__sself-__serializer-__s---result__sok-__serror .code-header}

::: where
where \_\_S:
[Serializer](../../../serde_core/ser/trait.Serializer.html "trait serde_core::ser::Serializer"){.trait},
:::
::::

::: docblock
Serialize this value into the given Serde serializer. [Read
more](../../../serde_core/ser/trait.Serialize.html#tymethod.serialize)
:::
::::::

::: {#impl-Strategable-for-BullCallSpread .section .impl}
[Source](../../../src/optionstratlib/strategies/bull_call_spread.rs.html#451-459){.src
.rightside}[§](#impl-Strategable-for-BullCallSpread){.anchor}

### impl [Strategable](../base/trait.Strategable.html "trait optionstratlib::strategies::base::Strategable"){.trait} for [BullCallSpread](struct.BullCallSpread.html "struct optionstratlib::strategies::bull_call_spread::BullCallSpread"){.struct} {#impl-strategable-for-bullcallspread .code-header}
:::

::::::::: impl-items
::: {#method.info .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/bull_call_spread.rs.html#452-458){.src
.rightside}[§](#method.info){.anchor}

#### fn [info](../base/trait.Strategable.html#method.info){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[StrategyBasics](../base/struct.StrategyBasics.html "struct optionstratlib::strategies::base::StrategyBasics"){.struct}, [StrategyError](../../error/strategies/enum.StrategyError.html "enum optionstratlib::error::strategies::StrategyError"){.enum}\> {#fn-infoself---resultstrategybasics-strategyerror .code-header}
:::

::: docblock
Returns basic information about the strategy, such as its name, type,
and description. [Read more](../base/trait.Strategable.html#method.info)
:::

::: {#method.type_name .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/base.rs.html#85-92){.src
.rightside}[§](#method.type_name){.anchor}

#### fn [type_name](../base/trait.Strategable.html#method.type_name){.fn}(&self) -\> [StrategyType](../base/enum.StrategyType.html "enum optionstratlib::strategies::base::StrategyType"){.enum} {#fn-type_nameself---strategytype .code-header}
:::

::: docblock
Returns the type of the strategy. [Read
more](../base/trait.Strategable.html#method.type_name)
:::

::: {#method.name-1 .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/base.rs.html#103-110){.src
.rightside}[§](#method.name-1){.anchor}

#### fn [name](../base/trait.Strategable.html#method.name){.fn}(&self) -\> [String](https://doc.rust-lang.org/1.91.1/alloc/string/struct.String.html "struct alloc::string::String"){.struct} {#fn-nameself---string .code-header}
:::

::: docblock
Returns the name of the strategy. [Read
more](../base/trait.Strategable.html#method.name)
:::
:::::::::

::: {#impl-Strategies-for-BullCallSpread .section .impl}
[Source](../../../src/optionstratlib/strategies/bull_call_spread.rs.html#584-623){.src
.rightside}[§](#impl-Strategies-for-BullCallSpread){.anchor}

### impl [Strategies](../base/trait.Strategies.html "trait optionstratlib::strategies::base::Strategies"){.trait} for [BullCallSpread](struct.BullCallSpread.html "struct optionstratlib::strategies::bull_call_spread::BullCallSpread"){.struct} {#impl-strategies-for-bullcallspread .code-header}
:::

::::::::::::::::::::::::::::::::::::: impl-items
::: {#method.get_max_profit .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/bull_call_spread.rs.html#585-596){.src
.rightside}[§](#method.get_max_profit){.anchor}

#### fn [get_max_profit](../base/trait.Strategies.html#method.get_max_profit){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, [StrategyError](../../error/strategies/enum.StrategyError.html "enum optionstratlib::error::strategies::StrategyError"){.enum}\> {#fn-get_max_profitself---resultpositive-strategyerror .code-header}
:::

::: docblock
Calculates the maximum possible profit for the strategy. The default
implementation returns an error indicating that the operation is not
supported. [Read
more](../base/trait.Strategies.html#method.get_max_profit)
:::

::: {#method.get_max_loss .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/bull_call_spread.rs.html#597-608){.src
.rightside}[§](#method.get_max_loss){.anchor}

#### fn [get_max_loss](../base/trait.Strategies.html#method.get_max_loss){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, [StrategyError](../../error/strategies/enum.StrategyError.html "enum optionstratlib::error::strategies::StrategyError"){.enum}\> {#fn-get_max_lossself---resultpositive-strategyerror .code-header}
:::

::: docblock
Calculates the maximum possible loss for the strategy. The default
implementation returns an error indicating that the operation is not
supported. [Read
more](../base/trait.Strategies.html#method.get_max_loss)
:::

::: {#method.get_profit_area .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/bull_call_spread.rs.html#609-613){.src
.rightside}[§](#method.get_profit_area){.anchor}

#### fn [get_profit_area](../base/trait.Strategies.html#method.get_profit_area){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Decimal](../../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}, [StrategyError](../../error/strategies/enum.StrategyError.html "enum optionstratlib::error::strategies::StrategyError"){.enum}\> {#fn-get_profit_areaself---resultdecimal-strategyerror .code-header}
:::

::: docblock
Calculates the profit area for the strategy. The default implementation
returns an error indicating that the operation is not supported. [Read
more](../base/trait.Strategies.html#method.get_profit_area)
:::

::: {#method.get_profit_ratio .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/bull_call_spread.rs.html#614-622){.src
.rightside}[§](#method.get_profit_ratio){.anchor}

#### fn [get_profit_ratio](../base/trait.Strategies.html#method.get_profit_ratio){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Decimal](../../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}, [StrategyError](../../error/strategies/enum.StrategyError.html "enum optionstratlib::error::strategies::StrategyError"){.enum}\> {#fn-get_profit_ratioself---resultdecimal-strategyerror .code-header}
:::

::: docblock
Calculates the profit ratio for the strategy. The default implementation
returns an error indicating that the operation is not supported. [Read
more](../base/trait.Strategies.html#method.get_profit_ratio)
:::

::: {#method.get_volume .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/base.rs.html#778-785){.src
.rightside}[§](#method.get_volume){.anchor}

#### fn [get_volume](../base/trait.Strategies.html#method.get_volume){.fn}(&mut self) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, [StrategyError](../../error/strategies/enum.StrategyError.html "enum optionstratlib::error::strategies::StrategyError"){.enum}\> {#fn-get_volumemut-self---resultpositive-strategyerror .code-header}
:::

::: docblock
Retrieves the current volume of the strategy as sum of quantities in
their positions [Read
more](../base/trait.Strategies.html#method.get_volume)
:::

::: {#method.get_max_profit_mut .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/base.rs.html#806-808){.src
.rightside}[§](#method.get_max_profit_mut){.anchor}

#### fn [get_max_profit_mut](../base/trait.Strategies.html#method.get_max_profit_mut){.fn}(&mut self) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, [StrategyError](../../error/strategies/enum.StrategyError.html "enum optionstratlib::error::strategies::StrategyError"){.enum}\> {#fn-get_max_profit_mutmut-self---resultpositive-strategyerror .code-header}
:::

::: docblock
Calculates the maximum possible profit for the strategy, potentially
using an iterative approach. Defaults to calling `max_profit`. [Read
more](../base/trait.Strategies.html#method.get_max_profit_mut)
:::

::: {#method.get_max_loss_mut .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/base.rs.html#829-831){.src
.rightside}[§](#method.get_max_loss_mut){.anchor}

#### fn [get_max_loss_mut](../base/trait.Strategies.html#method.get_max_loss_mut){.fn}(&mut self) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, [StrategyError](../../error/strategies/enum.StrategyError.html "enum optionstratlib::error::strategies::StrategyError"){.enum}\> {#fn-get_max_loss_mutmut-self---resultpositive-strategyerror .code-header}
:::

::: docblock
Calculates the maximum possible loss for the strategy, potentially using
an iterative approach. Defaults to calling `max_loss`. [Read
more](../base/trait.Strategies.html#method.get_max_loss_mut)
:::

::: {#method.get_total_cost .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/base.rs.html#838-845){.src
.rightside}[§](#method.get_total_cost){.anchor}

#### fn [get_total_cost](../base/trait.Strategies.html#method.get_total_cost){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, [PositionError](../../error/position/enum.PositionError.html "enum optionstratlib::error::position::PositionError"){.enum}\> {#fn-get_total_costself---resultpositive-positionerror .code-header}
:::

::: docblock
Calculates the total cost of the strategy, which is the sum of the
absolute cost of all positions. [Read
more](../base/trait.Strategies.html#method.get_total_cost)
:::

::: {#method.get_net_cost .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/base.rs.html#853-860){.src
.rightside}[§](#method.get_net_cost){.anchor}

#### fn [get_net_cost](../base/trait.Strategies.html#method.get_net_cost){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Decimal](../../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}, [PositionError](../../error/position/enum.PositionError.html "enum optionstratlib::error::position::PositionError"){.enum}\> {#fn-get_net_costself---resultdecimal-positionerror .code-header}
:::

::: docblock
Calculates the net cost of the strategy, which is the sum of the costs
of all positions, considering premiums paid and received. [Read
more](../base/trait.Strategies.html#method.get_net_cost)
:::

::: {#method.get_net_premium_received .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/base.rs.html#868-884){.src
.rightside}[§](#method.get_net_premium_received){.anchor}

#### fn [get_net_premium_received](../base/trait.Strategies.html#method.get_net_premium_received){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, [StrategyError](../../error/strategies/enum.StrategyError.html "enum optionstratlib::error::strategies::StrategyError"){.enum}\> {#fn-get_net_premium_receivedself---resultpositive-strategyerror .code-header}
:::

::: docblock
Calculates the net premium received for the strategy. This is the total
premium received from short positions minus the total premium paid for
long positions. If the result is negative, it returns zero. [Read
more](../base/trait.Strategies.html#method.get_net_premium_received)
:::

::: {#method.get_fees .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/base.rs.html#891-908){.src
.rightside}[§](#method.get_fees){.anchor}

#### fn [get_fees](../base/trait.Strategies.html#method.get_fees){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, [StrategyError](../../error/strategies/enum.StrategyError.html "enum optionstratlib::error::strategies::StrategyError"){.enum}\> {#fn-get_feesself---resultpositive-strategyerror .code-header}
:::

::: docblock
Calculates the total fees for the strategy by summing the fees of all
positions. [Read more](../base/trait.Strategies.html#method.get_fees)
:::

::: {#method.get_range_to_show .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/base.rs.html#944-968){.src
.rightside}[§](#method.get_range_to_show){.anchor}

#### fn [get_range_to_show](../base/trait.Strategies.html#method.get_range_to_show){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<([Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, [Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}), [StrategyError](../../error/strategies/enum.StrategyError.html "enum optionstratlib::error::strategies::StrategyError"){.enum}\> {#fn-get_range_to_showself---resultpositive-positive-strategyerror .code-header}
:::

::: docblock
Determines the price range to display for the strategy's profit/loss
graph. This range is calculated based on the break-even points, the
underlying price, and the maximum and minimum strike prices. The range
is expanded by applying `STRIKE_PRICE_LOWER_BOUND_MULTIPLIER` and
`STRIKE_PRICE_UPPER_BOUND_MULTIPLIER` to the minimum and maximum prices
respectively. [Read
more](../base/trait.Strategies.html#method.get_range_to_show)
:::

::: {#method.get_best_range_to_show .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/base.rs.html#975-978){.src
.rightside}[§](#method.get_best_range_to_show){.anchor}

#### fn [get_best_range_to_show](../base/trait.Strategies.html#method.get_best_range_to_show){.fn}( &self, step: [Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, ) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Vec](https://doc.rust-lang.org/1.91.1/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\>, [StrategyError](../../error/strategies/enum.StrategyError.html "enum optionstratlib::error::strategies::StrategyError"){.enum}\> {#fn-get_best_range_to_show-self-step-positive---resultvecpositive-strategyerror .code-header}
:::

::: docblock
Generates a vector of prices within the display range, using a specified
step. [Read
more](../base/trait.Strategies.html#method.get_best_range_to_show)
:::

::: {#method.get_max_min_strikes .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/base.rs.html#986-1018){.src
.rightside}[§](#method.get_max_min_strikes){.anchor}

#### fn [get_max_min_strikes](../base/trait.Strategies.html#method.get_max_min_strikes){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<([Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, [Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}), [StrategyError](../../error/strategies/enum.StrategyError.html "enum optionstratlib::error::strategies::StrategyError"){.enum}\> {#fn-get_max_min_strikesself---resultpositive-positive-strategyerror .code-header}
:::

::: docblock
Returns the minimum and maximum strike prices from the positions in the
strategy. Considers underlying price when applicable, ensuring the
returned range includes it. [Read
more](../base/trait.Strategies.html#method.get_max_min_strikes)
:::

::: {#method.get_range_of_profit .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/base.rs.html#1026-1040){.src
.rightside}[§](#method.get_range_of_profit){.anchor}

#### fn [get_range_of_profit](../base/trait.Strategies.html#method.get_range_of_profit){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, [StrategyError](../../error/strategies/enum.StrategyError.html "enum optionstratlib::error::strategies::StrategyError"){.enum}\> {#fn-get_range_of_profitself---resultpositive-strategyerror .code-header}
:::

::: docblock
Calculates the range of prices where the strategy is profitable, based
on the break-even points. [Read
more](../base/trait.Strategies.html#method.get_range_of_profit)
:::

::: {#method.roll_in .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/base.rs.html#1064-1066){.src
.rightside}[§](#method.roll_in){.anchor}

#### fn [roll_in](../base/trait.Strategies.html#method.roll_in){.fn}( &mut self, \_position: &[Position](../../model/position/struct.Position.html "struct optionstratlib::model::position::Position"){.struct}, ) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[HashMap](https://doc.rust-lang.org/1.91.1/std/collections/hash/map/struct.HashMap.html "struct std::collections::hash::map::HashMap"){.struct}\<[Action](../../model/types/enum.Action.html "enum optionstratlib::model::types::Action"){.enum}, [Trade](../../model/struct.Trade.html "struct optionstratlib::model::Trade"){.struct}\>, [StrategyError](../../error/strategies/enum.StrategyError.html "enum optionstratlib::error::strategies::StrategyError"){.enum}\> {#fn-roll_in-mut-self-_position-position---resulthashmapaction-trade-strategyerror .code-header}
:::

::: docblock
Attempts to execute the roll-in functionality for the strategy. [Read
more](../base/trait.Strategies.html#method.roll_in)
:::

::: {#method.roll_out .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/base.rs.html#1102-1104){.src
.rightside}[§](#method.roll_out){.anchor}

#### fn [roll_out](../base/trait.Strategies.html#method.roll_out){.fn}( &mut self, \_position: &[Position](../../model/position/struct.Position.html "struct optionstratlib::model::position::Position"){.struct}, ) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[HashMap](https://doc.rust-lang.org/1.91.1/std/collections/hash/map/struct.HashMap.html "struct std::collections::hash::map::HashMap"){.struct}\<[Action](../../model/types/enum.Action.html "enum optionstratlib::model::types::Action"){.enum}, [Trade](../../model/struct.Trade.html "struct optionstratlib::model::Trade"){.struct}\>, [StrategyError](../../error/strategies/enum.StrategyError.html "enum optionstratlib::error::strategies::StrategyError"){.enum}\> {#fn-roll_out-mut-self-_position-position---resulthashmapaction-trade-strategyerror .code-header}
:::

::: docblock
Executes the roll-out strategy for the provided position. [Read
more](../base/trait.Strategies.html#method.roll_out)
:::
:::::::::::::::::::::::::::::::::::::

::: {#impl-StrategyConstructor-for-BullCallSpread .section .impl}
[Source](../../../src/optionstratlib/strategies/bull_call_spread.rs.html#231-325){.src
.rightside}[§](#impl-StrategyConstructor-for-BullCallSpread){.anchor}

### impl [StrategyConstructor](../trait.StrategyConstructor.html "trait optionstratlib::strategies::StrategyConstructor"){.trait} for [BullCallSpread](struct.BullCallSpread.html "struct optionstratlib::strategies::bull_call_spread::BullCallSpread"){.struct} {#impl-strategyconstructor-for-bullcallspread .code-header}
:::

::::: impl-items
::: {#method.get_strategy .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/bull_call_spread.rs.html#232-324){.src
.rightside}[§](#method.get_strategy){.anchor}

#### fn [get_strategy](../trait.StrategyConstructor.html#method.get_strategy){.fn}(vec_positions: &\[[Position](../../model/position/struct.Position.html "struct optionstratlib::model::position::Position"){.struct}\]) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<Self, [StrategyError](../../error/strategies/enum.StrategyError.html "enum optionstratlib::error::strategies::StrategyError"){.enum}\> {#fn-get_strategyvec_positions-position---resultself-strategyerror .code-header}
:::

::: docblock
Attempts to construct a strategy from a vector of option positions.
[Read more](../trait.StrategyConstructor.html#method.get_strategy)
:::
:::::

::: {#impl-ToSchema-for-BullCallSpread .section .impl}
[Source](../../../src/optionstratlib/strategies/bull_call_spread.rs.html#78){.src
.rightside}[§](#impl-ToSchema-for-BullCallSpread){.anchor}

### impl [ToSchema](../../../utoipa/trait.ToSchema.html "trait utoipa::ToSchema"){.trait} for [BullCallSpread](struct.BullCallSpread.html "struct optionstratlib::strategies::bull_call_spread::BullCallSpread"){.struct} {#impl-toschema-for-bullcallspread .code-header}
:::

::::::: impl-items
::: {#method.name .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/bull_call_spread.rs.html#78){.src
.rightside}[§](#method.name){.anchor}

#### fn [name](../../../utoipa/trait.ToSchema.html#method.name){.fn}() -\> [Cow](https://doc.rust-lang.org/1.91.1/alloc/borrow/enum.Cow.html "enum alloc::borrow::Cow"){.enum}\<\'static, [str](https://doc.rust-lang.org/1.91.1/std/primitive.str.html){.primitive}\> {#fn-name---cowstatic-str .code-header}
:::

::: docblock
Return name of the schema. [Read
more](../../../utoipa/trait.ToSchema.html#method.name)
:::

::: {#method.schemas .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/bull_call_spread.rs.html#78){.src
.rightside}[§](#method.schemas){.anchor}

#### fn [schemas](../../../utoipa/trait.ToSchema.html#method.schemas){.fn}(schemas: &mut [Vec](https://doc.rust-lang.org/1.91.1/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<([String](https://doc.rust-lang.org/1.91.1/alloc/string/struct.String.html "struct alloc::string::String"){.struct}, [RefOr](../../../utoipa/openapi/enum.RefOr.html "enum utoipa::openapi::RefOr"){.enum}\<[Schema](../../../utoipa/openapi/schema/enum.Schema.html "enum utoipa::openapi::schema::Schema"){.enum}\>)\>) {#fn-schemasschemas-mut-vecstring-reforschema .code-header}
:::

::: docblock
Implement reference
[`utoipa::openapi::schema::Schema`](../../../utoipa/openapi/schema/enum.Schema.html "enum utoipa::openapi::schema::Schema")s
for this type. [Read
more](../../../utoipa/trait.ToSchema.html#method.schemas)
:::
:::::::

::: {#impl-Validable-for-BullCallSpread .section .impl}
[Source](../../../src/optionstratlib/strategies/bull_call_spread.rs.html#625-645){.src
.rightside}[§](#impl-Validable-for-BullCallSpread){.anchor}

### impl [Validable](../base/trait.Validable.html "trait optionstratlib::strategies::base::Validable"){.trait} for [BullCallSpread](struct.BullCallSpread.html "struct optionstratlib::strategies::bull_call_spread::BullCallSpread"){.struct} {#impl-validable-for-bullcallspread .code-header}
:::

::::: impl-items
::: {#method.validate .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/bull_call_spread.rs.html#626-644){.src
.rightside}[§](#method.validate){.anchor}

#### fn [validate](../base/trait.Validable.html#method.validate){.fn}(&self) -\> [bool](https://doc.rust-lang.org/1.91.1/std/primitive.bool.html){.primitive} {#fn-validateself---bool .code-header}
:::

::: docblock
Validates the strategy. [Read
more](../base/trait.Validable.html#method.validate)
:::
:::::
::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::

## Auto Trait Implementations[§](#synthetic-implementations){.anchor} {#synthetic-implementations .section-header}

::::::::: {#synthetic-implementations-list}
::: {#impl-Freeze-for-BullCallSpread .section .impl}
[§](#impl-Freeze-for-BullCallSpread){.anchor}

### impl [Freeze](https://doc.rust-lang.org/1.91.1/core/marker/trait.Freeze.html "trait core::marker::Freeze"){.trait} for [BullCallSpread](struct.BullCallSpread.html "struct optionstratlib::strategies::bull_call_spread::BullCallSpread"){.struct} {#impl-freeze-for-bullcallspread .code-header}
:::

::: {#impl-RefUnwindSafe-for-BullCallSpread .section .impl}
[§](#impl-RefUnwindSafe-for-BullCallSpread){.anchor}

### impl [RefUnwindSafe](https://doc.rust-lang.org/1.91.1/core/panic/unwind_safe/trait.RefUnwindSafe.html "trait core::panic::unwind_safe::RefUnwindSafe"){.trait} for [BullCallSpread](struct.BullCallSpread.html "struct optionstratlib::strategies::bull_call_spread::BullCallSpread"){.struct} {#impl-refunwindsafe-for-bullcallspread .code-header}
:::

::: {#impl-Send-for-BullCallSpread .section .impl}
[§](#impl-Send-for-BullCallSpread){.anchor}

### impl [Send](https://doc.rust-lang.org/1.91.1/core/marker/trait.Send.html "trait core::marker::Send"){.trait} for [BullCallSpread](struct.BullCallSpread.html "struct optionstratlib::strategies::bull_call_spread::BullCallSpread"){.struct} {#impl-send-for-bullcallspread .code-header}
:::

::: {#impl-Sync-for-BullCallSpread .section .impl}
[§](#impl-Sync-for-BullCallSpread){.anchor}

### impl [Sync](https://doc.rust-lang.org/1.91.1/core/marker/trait.Sync.html "trait core::marker::Sync"){.trait} for [BullCallSpread](struct.BullCallSpread.html "struct optionstratlib::strategies::bull_call_spread::BullCallSpread"){.struct} {#impl-sync-for-bullcallspread .code-header}
:::

::: {#impl-Unpin-for-BullCallSpread .section .impl}
[§](#impl-Unpin-for-BullCallSpread){.anchor}

### impl [Unpin](https://doc.rust-lang.org/1.91.1/core/marker/trait.Unpin.html "trait core::marker::Unpin"){.trait} for [BullCallSpread](struct.BullCallSpread.html "struct optionstratlib::strategies::bull_call_spread::BullCallSpread"){.struct} {#impl-unpin-for-bullcallspread .code-header}
:::

::: {#impl-UnwindSafe-for-BullCallSpread .section .impl}
[§](#impl-UnwindSafe-for-BullCallSpread){.anchor}

### impl [UnwindSafe](https://doc.rust-lang.org/1.91.1/core/panic/unwind_safe/trait.UnwindSafe.html "trait core::panic::unwind_safe::UnwindSafe"){.trait} for [BullCallSpread](struct.BullCallSpread.html "struct optionstratlib::strategies::bull_call_spread::BullCallSpread"){.struct} {#impl-unwindsafe-for-bullcallspread .code-header}
:::
:::::::::

## Blanket Implementations[§](#blanket-implementations){.anchor} {#blanket-implementations .section-header}

::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::: {#blanket-implementations-list}
:::: {#impl-Any-for-T .section .impl}
[Source](https://doc.rust-lang.org/1.91.1/src/core/any.rs.html#138){.src
.rightside}[§](#impl-Any-for-T){.anchor}

### impl\<T\> [Any](https://doc.rust-lang.org/1.91.1/core/any/trait.Any.html "trait core::any::Any"){.trait} for T {#implt-any-for-t .code-header}

::: where
where T: \'static +
?[Sized](https://doc.rust-lang.org/1.91.1/core/marker/trait.Sized.html "trait core::marker::Sized"){.trait},
:::
::::

::::: impl-items
::: {#method.type_id .section .method .trait-impl}
[Source](https://doc.rust-lang.org/1.91.1/src/core/any.rs.html#139){.src
.rightside}[§](#method.type_id){.anchor}

#### fn [type_id](https://doc.rust-lang.org/1.91.1/core/any/trait.Any.html#tymethod.type_id){.fn}(&self) -\> [TypeId](https://doc.rust-lang.org/1.91.1/core/any/struct.TypeId.html "struct core::any::TypeId"){.struct} {#fn-type_idself---typeid .code-header}
:::

::: docblock
Gets the `TypeId` of `self`. [Read
more](https://doc.rust-lang.org/1.91.1/core/any/trait.Any.html#tymethod.type_id)
:::
:::::

:::: {#impl-Borrow%3CT%3E-for-T .section .impl}
[Source](https://doc.rust-lang.org/1.91.1/src/core/borrow.rs.html#212){.src
.rightside}[§](#impl-Borrow%3CT%3E-for-T){.anchor}

### impl\<T\> [Borrow](https://doc.rust-lang.org/1.91.1/core/borrow/trait.Borrow.html "trait core::borrow::Borrow"){.trait}\<T\> for T {#implt-borrowt-for-t .code-header}

::: where
where T:
?[Sized](https://doc.rust-lang.org/1.91.1/core/marker/trait.Sized.html "trait core::marker::Sized"){.trait},
:::
::::

::::: impl-items
::: {#method.borrow .section .method .trait-impl}
[Source](https://doc.rust-lang.org/1.91.1/src/core/borrow.rs.html#214){.src
.rightside}[§](#method.borrow){.anchor}

#### fn [borrow](https://doc.rust-lang.org/1.91.1/core/borrow/trait.Borrow.html#tymethod.borrow){.fn}(&self) -\> [&T](https://doc.rust-lang.org/1.91.1/std/primitive.reference.html){.primitive} {#fn-borrowself---t .code-header}
:::

::: docblock
Immutably borrows from an owned value. [Read
more](https://doc.rust-lang.org/1.91.1/core/borrow/trait.Borrow.html#tymethod.borrow)
:::
:::::

:::: {#impl-BorrowMut%3CT%3E-for-T .section .impl}
[Source](https://doc.rust-lang.org/1.91.1/src/core/borrow.rs.html#221){.src
.rightside}[§](#impl-BorrowMut%3CT%3E-for-T){.anchor}

### impl\<T\> [BorrowMut](https://doc.rust-lang.org/1.91.1/core/borrow/trait.BorrowMut.html "trait core::borrow::BorrowMut"){.trait}\<T\> for T {#implt-borrowmutt-for-t .code-header}

::: where
where T:
?[Sized](https://doc.rust-lang.org/1.91.1/core/marker/trait.Sized.html "trait core::marker::Sized"){.trait},
:::
::::

::::: impl-items
::: {#method.borrow_mut .section .method .trait-impl}
[Source](https://doc.rust-lang.org/1.91.1/src/core/borrow.rs.html#222){.src
.rightside}[§](#method.borrow_mut){.anchor}

#### fn [borrow_mut](https://doc.rust-lang.org/1.91.1/core/borrow/trait.BorrowMut.html#tymethod.borrow_mut){.fn}(&mut self) -\> [&mut T](https://doc.rust-lang.org/1.91.1/std/primitive.reference.html){.primitive} {#fn-borrow_mutmut-self---mut-t .code-header}
:::

::: docblock
Mutably borrows from an owned value. [Read
more](https://doc.rust-lang.org/1.91.1/core/borrow/trait.BorrowMut.html#tymethod.borrow_mut)
:::
:::::

:::: {#impl-CloneToUninit-for-T .section .impl}
[Source](https://doc.rust-lang.org/1.91.1/src/core/clone.rs.html#515){.src
.rightside}[§](#impl-CloneToUninit-for-T){.anchor}

### impl\<T\> [CloneToUninit](https://doc.rust-lang.org/1.91.1/core/clone/trait.CloneToUninit.html "trait core::clone::CloneToUninit"){.trait} for T {#implt-clonetouninit-for-t .code-header}

::: where
where T:
[Clone](https://doc.rust-lang.org/1.91.1/core/clone/trait.Clone.html "trait core::clone::Clone"){.trait},
:::
::::

:::::: impl-items
::: {#method.clone_to_uninit .section .method .trait-impl}
[Source](https://doc.rust-lang.org/1.91.1/src/core/clone.rs.html#517){.src
.rightside}[§](#method.clone_to_uninit){.anchor}

#### unsafe fn [clone_to_uninit](https://doc.rust-lang.org/1.91.1/core/clone/trait.CloneToUninit.html#tymethod.clone_to_uninit){.fn}(&self, dest: [\*mut](https://doc.rust-lang.org/1.91.1/std/primitive.pointer.html){.primitive} [u8](https://doc.rust-lang.org/1.91.1/std/primitive.u8.html){.primitive}) {#unsafe-fn-clone_to_uninitself-dest-mut-u8 .code-header}
:::

[]{.item-info}

::: {.stab .unstable}
🔬This is a nightly-only experimental API. (`clone_to_uninit`)
:::

::: docblock
Performs copy-assignment from `self` to `dest`. [Read
more](https://doc.rust-lang.org/1.91.1/core/clone/trait.CloneToUninit.html#tymethod.clone_to_uninit)
:::
::::::

::: {#impl-From%3CT%3E-for-T .section .impl}
[Source](https://doc.rust-lang.org/1.91.1/src/core/convert/mod.rs.html#785){.src
.rightside}[§](#impl-From%3CT%3E-for-T){.anchor}

### impl\<T\> [From](https://doc.rust-lang.org/1.91.1/core/convert/trait.From.html "trait core::convert::From"){.trait}\<T\> for T {#implt-fromt-for-t .code-header}
:::

::::: impl-items
::: {#method.from .section .method .trait-impl}
[Source](https://doc.rust-lang.org/1.91.1/src/core/convert/mod.rs.html#788){.src
.rightside}[§](#method.from){.anchor}

#### fn [from](https://doc.rust-lang.org/1.91.1/core/convert/trait.From.html#tymethod.from){.fn}(t: T) -\> T {#fn-fromt-t---t .code-header}
:::

::: docblock
Returns the argument unchanged.
:::
:::::

::: {#impl-Instrument-for-T .section .impl}
[Source](../../../src/tracing/instrument.rs.html#325){.src
.rightside}[§](#impl-Instrument-for-T){.anchor}

### impl\<T\> [Instrument](../../../tracing/instrument/trait.Instrument.html "trait tracing::instrument::Instrument"){.trait} for T {#implt-instrument-for-t .code-header}
:::

::::::: impl-items
::: {#method.instrument .section .method .trait-impl}
[Source](../../../src/tracing/instrument.rs.html#86){.src
.rightside}[§](#method.instrument){.anchor}

#### fn [instrument](../../../tracing/instrument/trait.Instrument.html#method.instrument){.fn}(self, span: [Span](../../../tracing/span/struct.Span.html "struct tracing::span::Span"){.struct}) -\> [Instrumented](../../../tracing/instrument/struct.Instrumented.html "struct tracing::instrument::Instrumented"){.struct}\<Self\> {#fn-instrumentself-span-span---instrumentedself .code-header}
:::

::: docblock
Instruments this type with the provided
[`Span`](../../../tracing/span/struct.Span.html "struct tracing::span::Span"),
returning an `Instrumented` wrapper. [Read
more](../../../tracing/instrument/trait.Instrument.html#method.instrument)
:::

::: {#method.in_current_span .section .method .trait-impl}
[Source](../../../src/tracing/instrument.rs.html#128){.src
.rightside}[§](#method.in_current_span){.anchor}

#### fn [in_current_span](../../../tracing/instrument/trait.Instrument.html#method.in_current_span){.fn}(self) -\> [Instrumented](../../../tracing/instrument/struct.Instrumented.html "struct tracing::instrument::Instrumented"){.struct}\<Self\> {#fn-in_current_spanself---instrumentedself .code-header}
:::

::: docblock
Instruments this type with the
[current](../../../tracing/span/struct.Span.html#method.current "associated function tracing::span::Span::current")
[`Span`](../../../tracing/span/struct.Span.html "struct tracing::span::Span"),
returning an `Instrumented` wrapper. [Read
more](../../../tracing/instrument/trait.Instrument.html#method.in_current_span)
:::
:::::::

:::: {#impl-Into%3CU%3E-for-T .section .impl}
[Source](https://doc.rust-lang.org/1.91.1/src/core/convert/mod.rs.html#767-769){.src
.rightside}[§](#impl-Into%3CU%3E-for-T){.anchor}

### impl\<T, U\> [Into](https://doc.rust-lang.org/1.91.1/core/convert/trait.Into.html "trait core::convert::Into"){.trait}\<U\> for T {#implt-u-intou-for-t .code-header}

::: where
where U:
[From](https://doc.rust-lang.org/1.91.1/core/convert/trait.From.html "trait core::convert::From"){.trait}\<T\>,
:::
::::

::::: impl-items
::: {#method.into .section .method .trait-impl}
[Source](https://doc.rust-lang.org/1.91.1/src/core/convert/mod.rs.html#777){.src
.rightside}[§](#method.into){.anchor}

#### fn [into](https://doc.rust-lang.org/1.91.1/core/convert/trait.Into.html#tymethod.into){.fn}(self) -\> U {#fn-intoself---u .code-header}
:::

::: docblock
Calls `U::from(self)`.

That is, this conversion is whatever the implementation of
[`From`](https://doc.rust-lang.org/1.91.1/core/convert/trait.From.html "trait core::convert::From")`<T> for U`
chooses to do.
:::
:::::

::: {#impl-IntoEither-for-T .section .impl}
[Source](../../../src/either/into_either.rs.html#64){.src
.rightside}[§](#impl-IntoEither-for-T){.anchor}

### impl\<T\> [IntoEither](../../../either/into_either/trait.IntoEither.html "trait either::into_either::IntoEither"){.trait} for T {#implt-intoeither-for-t .code-header}
:::

:::::::: impl-items
::: {#method.into_either .section .method .trait-impl}
[Source](../../../src/either/into_either.rs.html#29){.src
.rightside}[§](#method.into_either){.anchor}

#### fn [into_either](../../../either/into_either/trait.IntoEither.html#method.into_either){.fn}(self, into_left: [bool](https://doc.rust-lang.org/1.91.1/std/primitive.bool.html){.primitive}) -\> [Either](../../../either/enum.Either.html "enum either::Either"){.enum}\<Self, Self\> {#fn-into_eitherself-into_left-bool---eitherself-self .code-header}
:::

::: docblock
Converts `self` into a
[`Left`](../../../either/enum.Either.html#variant.Left "variant either::Either::Left")
variant of
[`Either<Self, Self>`](../../../either/enum.Either.html "enum either::Either")
if `into_left` is `true`. Converts `self` into a
[`Right`](../../../either/enum.Either.html#variant.Right "variant either::Either::Right")
variant of
[`Either<Self, Self>`](../../../either/enum.Either.html "enum either::Either")
otherwise. [Read
more](../../../either/into_either/trait.IntoEither.html#method.into_either)
:::

:::: {#method.into_either_with .section .method .trait-impl}
[Source](../../../src/either/into_either.rs.html#55-57){.src
.rightside}[§](#method.into_either_with){.anchor}

#### fn [into_either_with](../../../either/into_either/trait.IntoEither.html#method.into_either_with){.fn}\<F\>(self, into_left: F) -\> [Either](../../../either/enum.Either.html "enum either::Either"){.enum}\<Self, Self\> {#fn-into_either_withfself-into_left-f---eitherself-self .code-header}

::: where
where F:
[FnOnce](https://doc.rust-lang.org/1.91.1/core/ops/function/trait.FnOnce.html "trait core::ops::function::FnOnce"){.trait}(&Self)
-\>
[bool](https://doc.rust-lang.org/1.91.1/std/primitive.bool.html){.primitive},
:::
::::

::: docblock
Converts `self` into a
[`Left`](../../../either/enum.Either.html#variant.Left "variant either::Either::Left")
variant of
[`Either<Self, Self>`](../../../either/enum.Either.html "enum either::Either")
if `into_left(&self)` returns `true`. Converts `self` into a
[`Right`](../../../either/enum.Either.html#variant.Right "variant either::Either::Right")
variant of
[`Either<Self, Self>`](../../../either/enum.Either.html "enum either::Either")
otherwise. [Read
more](../../../either/into_either/trait.IntoEither.html#method.into_either_with)
:::
::::::::

:::: {#impl-PartialSchema-for-T .section .impl}
[Source](../../../src/utoipa/lib.rs.html#1375){.src
.rightside}[§](#impl-PartialSchema-for-T){.anchor}

### impl\<T\> [PartialSchema](../../../utoipa/trait.PartialSchema.html "trait utoipa::PartialSchema"){.trait} for T {#implt-partialschema-for-t .code-header}

::: where
where T: ComposeSchema +
?[Sized](https://doc.rust-lang.org/1.91.1/core/marker/trait.Sized.html "trait core::marker::Sized"){.trait},
:::
::::

::::: impl-items
::: {#method.schema .section .method .trait-impl}
[Source](../../../src/utoipa/lib.rs.html#1376){.src
.rightside}[§](#method.schema){.anchor}

#### fn [schema](../../../utoipa/trait.PartialSchema.html#tymethod.schema){.fn}() -\> [RefOr](../../../utoipa/openapi/enum.RefOr.html "enum utoipa::openapi::RefOr"){.enum}\<[Schema](../../../utoipa/openapi/schema/enum.Schema.html "enum utoipa::openapi::schema::Schema"){.enum}\> {#fn-schema---reforschema .code-header}
:::

::: docblock
Return ref or schema of implementing type that can then be used to
construct combined schemas.
:::
:::::

::: {#impl-Pointable-for-T .section .impl}
[Source](../../../src/crossbeam_epoch/atomic.rs.html#194){.src
.rightside}[§](#impl-Pointable-for-T){.anchor}

### impl\<T\> [Pointable](../../../crossbeam_epoch/atomic/trait.Pointable.html "trait crossbeam_epoch::atomic::Pointable"){.trait} for T {#implt-pointable-for-t .code-header}
:::

::::::::::::::: impl-items
::: {#associatedconstant.ALIGN .section .associatedconstant .trait-impl}
[Source](../../../src/crossbeam_epoch/atomic.rs.html#195){.src
.rightside}[§](#associatedconstant.ALIGN){.anchor}

#### const [ALIGN](../../../crossbeam_epoch/atomic/trait.Pointable.html#associatedconstant.ALIGN){.constant}: [usize](https://doc.rust-lang.org/1.91.1/std/primitive.usize.html){.primitive} {#const-align-usize .code-header}
:::

::: docblock
The alignment of pointer.
:::

::: {#associatedtype.Init .section .associatedtype .trait-impl}
[Source](../../../src/crossbeam_epoch/atomic.rs.html#197){.src
.rightside}[§](#associatedtype.Init){.anchor}

#### type [Init](../../../crossbeam_epoch/atomic/trait.Pointable.html#associatedtype.Init){.associatedtype} = T {#type-init-t .code-header}
:::

::: docblock
The type for initializers.
:::

::: {#method.init .section .method .trait-impl}
[Source](../../../src/crossbeam_epoch/atomic.rs.html#199){.src
.rightside}[§](#method.init){.anchor}

#### unsafe fn [init](../../../crossbeam_epoch/atomic/trait.Pointable.html#tymethod.init){.fn}(init: \<T as [Pointable](../../../crossbeam_epoch/atomic/trait.Pointable.html "trait crossbeam_epoch::atomic::Pointable"){.trait}\>::[Init](../../../crossbeam_epoch/atomic/trait.Pointable.html#associatedtype.Init "type crossbeam_epoch::atomic::Pointable::Init"){.associatedtype}) -\> [usize](https://doc.rust-lang.org/1.91.1/std/primitive.usize.html){.primitive} {#unsafe-fn-initinit-t-as-pointableinit---usize .code-header}
:::

::: docblock
Initializes a with the given initializer. [Read
more](../../../crossbeam_epoch/atomic/trait.Pointable.html#tymethod.init)
:::

::: {#method.deref .section .method .trait-impl}
[Source](../../../src/crossbeam_epoch/atomic.rs.html#203){.src
.rightside}[§](#method.deref){.anchor}

#### unsafe fn [deref](../../../crossbeam_epoch/atomic/trait.Pointable.html#tymethod.deref){.fn}\<\'a\>(ptr: [usize](https://doc.rust-lang.org/1.91.1/std/primitive.usize.html){.primitive}) -\> [&\'a T](https://doc.rust-lang.org/1.91.1/std/primitive.reference.html){.primitive} {#unsafe-fn-derefaptr-usize---a-t .code-header}
:::

::: docblock
Dereferences the given pointer. [Read
more](../../../crossbeam_epoch/atomic/trait.Pointable.html#tymethod.deref)
:::

::: {#method.deref_mut .section .method .trait-impl}
[Source](../../../src/crossbeam_epoch/atomic.rs.html#207){.src
.rightside}[§](#method.deref_mut){.anchor}

#### unsafe fn [deref_mut](../../../crossbeam_epoch/atomic/trait.Pointable.html#tymethod.deref_mut){.fn}\<\'a\>(ptr: [usize](https://doc.rust-lang.org/1.91.1/std/primitive.usize.html){.primitive}) -\> [&\'a mut T](https://doc.rust-lang.org/1.91.1/std/primitive.reference.html){.primitive} {#unsafe-fn-deref_mutaptr-usize---a-mut-t .code-header}
:::

::: docblock
Mutably dereferences the given pointer. [Read
more](../../../crossbeam_epoch/atomic/trait.Pointable.html#tymethod.deref_mut)
:::

::: {#method.drop .section .method .trait-impl}
[Source](../../../src/crossbeam_epoch/atomic.rs.html#211){.src
.rightside}[§](#method.drop){.anchor}

#### unsafe fn [drop](../../../crossbeam_epoch/atomic/trait.Pointable.html#tymethod.drop){.fn}(ptr: [usize](https://doc.rust-lang.org/1.91.1/std/primitive.usize.html){.primitive}) {#unsafe-fn-dropptr-usize .code-header}
:::

::: docblock
Drops the object pointed to by the given pointer. [Read
more](../../../crossbeam_epoch/atomic/trait.Pointable.html#tymethod.drop)
:::
:::::::::::::::

::: {#impl-Same-for-T .section .impl}
[Source](../../../src/typenum/type_operators.rs.html#34){.src
.rightside}[§](#impl-Same-for-T){.anchor}

### impl\<T\> [Same](../../../typenum/type_operators/trait.Same.html "trait typenum::type_operators::Same"){.trait} for T {#implt-same-for-t .code-header}
:::

::::: impl-items
::: {#associatedtype.Output .section .associatedtype .trait-impl}
[Source](../../../src/typenum/type_operators.rs.html#35){.src
.rightside}[§](#associatedtype.Output){.anchor}

#### type [Output](../../../typenum/type_operators/trait.Same.html#associatedtype.Output){.associatedtype} = T {#type-output-t .code-header}
:::

::: docblock
Should always be `Self`
:::
:::::

:::: {#impl-SupersetOf%3CSS%3E-for-SP .section .impl}
[Source](../../../src/simba/scalar/subset.rs.html#90){.src
.rightside}[§](#impl-SupersetOf%3CSS%3E-for-SP){.anchor}

### impl\<SS, SP\> [SupersetOf](../../../simba/scalar/subset/trait.SupersetOf.html "trait simba::scalar::subset::SupersetOf"){.trait}\<SS\> for SP {#implss-sp-supersetofss-for-sp .code-header}

::: where
where SS:
[SubsetOf](../../../simba/scalar/subset/trait.SubsetOf.html "trait simba::scalar::subset::SubsetOf"){.trait}\<SP\>,
:::
::::

::::::::::: impl-items
::: {#method.to_subset .section .method .trait-impl}
[Source](../../../src/simba/scalar/subset.rs.html#92){.src
.rightside}[§](#method.to_subset){.anchor}

#### fn [to_subset](../../../simba/scalar/subset/trait.SupersetOf.html#method.to_subset){.fn}(&self) -\> [Option](https://doc.rust-lang.org/1.91.1/core/option/enum.Option.html "enum core::option::Option"){.enum}\<SS\> {#fn-to_subsetself---optionss .code-header}
:::

::: docblock
The inverse inclusion map: attempts to construct `self` from the
equivalent element of its superset. [Read
more](../../../simba/scalar/subset/trait.SupersetOf.html#method.to_subset)
:::

::: {#method.is_in_subset .section .method .trait-impl}
[Source](../../../src/simba/scalar/subset.rs.html#97){.src
.rightside}[§](#method.is_in_subset){.anchor}

#### fn [is_in_subset](../../../simba/scalar/subset/trait.SupersetOf.html#tymethod.is_in_subset){.fn}(&self) -\> [bool](https://doc.rust-lang.org/1.91.1/std/primitive.bool.html){.primitive} {#fn-is_in_subsetself---bool .code-header}
:::

::: docblock
Checks if `self` is actually part of its subset `T` (and can be
converted to it).
:::

::: {#method.to_subset_unchecked .section .method .trait-impl}
[Source](../../../src/simba/scalar/subset.rs.html#102){.src
.rightside}[§](#method.to_subset_unchecked){.anchor}

#### fn [to_subset_unchecked](../../../simba/scalar/subset/trait.SupersetOf.html#tymethod.to_subset_unchecked){.fn}(&self) -\> SS {#fn-to_subset_uncheckedself---ss .code-header}
:::

::: docblock
Use with care! Same as `self.to_subset` but without any property checks.
Always succeeds.
:::

::: {#method.from_subset .section .method .trait-impl}
[Source](../../../src/simba/scalar/subset.rs.html#107){.src
.rightside}[§](#method.from_subset){.anchor}

#### fn [from_subset](../../../simba/scalar/subset/trait.SupersetOf.html#tymethod.from_subset){.fn}(element: [&SS](https://doc.rust-lang.org/1.91.1/std/primitive.reference.html){.primitive}) -\> SP {#fn-from_subsetelement-ss---sp .code-header}
:::

::: docblock
The inclusion map: converts `self` to the equivalent element of its
superset.
:::
:::::::::::

:::: {#impl-ToOwned-for-T .section .impl}
[Source](https://doc.rust-lang.org/1.91.1/src/alloc/borrow.rs.html#85-87){.src
.rightside}[§](#impl-ToOwned-for-T){.anchor}

### impl\<T\> [ToOwned](https://doc.rust-lang.org/1.91.1/alloc/borrow/trait.ToOwned.html "trait alloc::borrow::ToOwned"){.trait} for T {#implt-toowned-for-t .code-header}

::: where
where T:
[Clone](https://doc.rust-lang.org/1.91.1/core/clone/trait.Clone.html "trait core::clone::Clone"){.trait},
:::
::::

::::::::: impl-items
::: {#associatedtype.Owned .section .associatedtype .trait-impl}
[Source](https://doc.rust-lang.org/1.91.1/src/alloc/borrow.rs.html#89){.src
.rightside}[§](#associatedtype.Owned){.anchor}

#### type [Owned](https://doc.rust-lang.org/1.91.1/alloc/borrow/trait.ToOwned.html#associatedtype.Owned){.associatedtype} = T {#type-owned-t .code-header}
:::

::: docblock
The resulting type after obtaining ownership.
:::

::: {#method.to_owned .section .method .trait-impl}
[Source](https://doc.rust-lang.org/1.91.1/src/alloc/borrow.rs.html#90){.src
.rightside}[§](#method.to_owned){.anchor}

#### fn [to_owned](https://doc.rust-lang.org/1.91.1/alloc/borrow/trait.ToOwned.html#tymethod.to_owned){.fn}(&self) -\> T {#fn-to_ownedself---t .code-header}
:::

::: docblock
Creates owned data from borrowed data, usually by cloning. [Read
more](https://doc.rust-lang.org/1.91.1/alloc/borrow/trait.ToOwned.html#tymethod.to_owned)
:::

::: {#method.clone_into .section .method .trait-impl}
[Source](https://doc.rust-lang.org/1.91.1/src/alloc/borrow.rs.html#94){.src
.rightside}[§](#method.clone_into){.anchor}

#### fn [clone_into](https://doc.rust-lang.org/1.91.1/alloc/borrow/trait.ToOwned.html#method.clone_into){.fn}(&self, target: [&mut T](https://doc.rust-lang.org/1.91.1/std/primitive.reference.html){.primitive}) {#fn-clone_intoself-target-mut-t .code-header}
:::

::: docblock
Uses borrowed data to replace owned data, usually by cloning. [Read
more](https://doc.rust-lang.org/1.91.1/alloc/borrow/trait.ToOwned.html#method.clone_into)
:::
:::::::::

:::: {#impl-ToString-for-T .section .impl}
[Source](https://doc.rust-lang.org/1.91.1/src/alloc/string.rs.html#2796){.src
.rightside}[§](#impl-ToString-for-T){.anchor}

### impl\<T\> [ToString](https://doc.rust-lang.org/1.91.1/alloc/string/trait.ToString.html "trait alloc::string::ToString"){.trait} for T {#implt-tostring-for-t .code-header}

::: where
where T:
[Display](https://doc.rust-lang.org/1.91.1/core/fmt/trait.Display.html "trait core::fmt::Display"){.trait} +
?[Sized](https://doc.rust-lang.org/1.91.1/core/marker/trait.Sized.html "trait core::marker::Sized"){.trait},
:::
::::

::::: impl-items
::: {#method.to_string .section .method .trait-impl}
[Source](https://doc.rust-lang.org/1.91.1/src/alloc/string.rs.html#2798){.src
.rightside}[§](#method.to_string){.anchor}

#### fn [to_string](https://doc.rust-lang.org/1.91.1/alloc/string/trait.ToString.html#tymethod.to_string){.fn}(&self) -\> [String](https://doc.rust-lang.org/1.91.1/alloc/string/struct.String.html "struct alloc::string::String"){.struct} {#fn-to_stringself---string .code-header}
:::

::: docblock
Converts the given value to a `String`. [Read
more](https://doc.rust-lang.org/1.91.1/alloc/string/trait.ToString.html#tymethod.to_string)
:::
:::::

:::: {#impl-TryFrom%3CU%3E-for-T .section .impl}
[Source](https://doc.rust-lang.org/1.91.1/src/core/convert/mod.rs.html#827-829){.src
.rightside}[§](#impl-TryFrom%3CU%3E-for-T){.anchor}

### impl\<T, U\> [TryFrom](https://doc.rust-lang.org/1.91.1/core/convert/trait.TryFrom.html "trait core::convert::TryFrom"){.trait}\<U\> for T {#implt-u-tryfromu-for-t .code-header}

::: where
where U:
[Into](https://doc.rust-lang.org/1.91.1/core/convert/trait.Into.html "trait core::convert::Into"){.trait}\<T\>,
:::
::::

::::::: impl-items
::: {#associatedtype.Error-1 .section .associatedtype .trait-impl}
[Source](https://doc.rust-lang.org/1.91.1/src/core/convert/mod.rs.html#831){.src
.rightside}[§](#associatedtype.Error-1){.anchor}

#### type [Error](https://doc.rust-lang.org/1.91.1/core/convert/trait.TryFrom.html#associatedtype.Error){.associatedtype} = [Infallible](https://doc.rust-lang.org/1.91.1/core/convert/enum.Infallible.html "enum core::convert::Infallible"){.enum} {#type-error-infallible .code-header}
:::

::: docblock
The type returned in the event of a conversion error.
:::

::: {#method.try_from .section .method .trait-impl}
[Source](https://doc.rust-lang.org/1.91.1/src/core/convert/mod.rs.html#834){.src
.rightside}[§](#method.try_from){.anchor}

#### fn [try_from](https://doc.rust-lang.org/1.91.1/core/convert/trait.TryFrom.html#tymethod.try_from){.fn}(value: U) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<T, \<T as [TryFrom](https://doc.rust-lang.org/1.91.1/core/convert/trait.TryFrom.html "trait core::convert::TryFrom"){.trait}\<U\>\>::[Error](https://doc.rust-lang.org/1.91.1/core/convert/trait.TryFrom.html#associatedtype.Error "type core::convert::TryFrom::Error"){.associatedtype}\> {#fn-try_fromvalue-u---resultt-t-as-tryfromuerror .code-header}
:::

::: docblock
Performs the conversion.
:::
:::::::

:::: {#impl-TryInto%3CU%3E-for-T .section .impl}
[Source](https://doc.rust-lang.org/1.91.1/src/core/convert/mod.rs.html#811-813){.src
.rightside}[§](#impl-TryInto%3CU%3E-for-T){.anchor}

### impl\<T, U\> [TryInto](https://doc.rust-lang.org/1.91.1/core/convert/trait.TryInto.html "trait core::convert::TryInto"){.trait}\<U\> for T {#implt-u-tryintou-for-t .code-header}

::: where
where U:
[TryFrom](https://doc.rust-lang.org/1.91.1/core/convert/trait.TryFrom.html "trait core::convert::TryFrom"){.trait}\<T\>,
:::
::::

::::::: impl-items
::: {#associatedtype.Error .section .associatedtype .trait-impl}
[Source](https://doc.rust-lang.org/1.91.1/src/core/convert/mod.rs.html#815){.src
.rightside}[§](#associatedtype.Error){.anchor}

#### type [Error](https://doc.rust-lang.org/1.91.1/core/convert/trait.TryInto.html#associatedtype.Error){.associatedtype} = \<U as [TryFrom](https://doc.rust-lang.org/1.91.1/core/convert/trait.TryFrom.html "trait core::convert::TryFrom"){.trait}\<T\>\>::[Error](https://doc.rust-lang.org/1.91.1/core/convert/trait.TryFrom.html#associatedtype.Error "type core::convert::TryFrom::Error"){.associatedtype} {#type-error-u-as-tryfromterror .code-header}
:::

::: docblock
The type returned in the event of a conversion error.
:::

::: {#method.try_into .section .method .trait-impl}
[Source](https://doc.rust-lang.org/1.91.1/src/core/convert/mod.rs.html#818){.src
.rightside}[§](#method.try_into){.anchor}

#### fn [try_into](https://doc.rust-lang.org/1.91.1/core/convert/trait.TryInto.html#tymethod.try_into){.fn}(self) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<U, \<U as [TryFrom](https://doc.rust-lang.org/1.91.1/core/convert/trait.TryFrom.html "trait core::convert::TryFrom"){.trait}\<T\>\>::[Error](https://doc.rust-lang.org/1.91.1/core/convert/trait.TryFrom.html#associatedtype.Error "type core::convert::TryFrom::Error"){.associatedtype}\> {#fn-try_intoself---resultu-u-as-tryfromterror .code-header}
:::

::: docblock
Performs the conversion.
:::
:::::::

:::: {#impl-VZip%3CV%3E-for-T .section .impl}
[Source](../../../src/ppv_lite86/types.rs.html#221-223){.src
.rightside}[§](#impl-VZip%3CV%3E-for-T){.anchor}

### impl\<V, T\> [VZip](../../../ppv_lite86/types/trait.VZip.html "trait ppv_lite86::types::VZip"){.trait}\<V\> for T {#implv-t-vzipv-for-t .code-header}

::: where
where V:
[MultiLane](../../../ppv_lite86/types/trait.MultiLane.html "trait ppv_lite86::types::MultiLane"){.trait}\<T\>,
:::
::::

:::: impl-items
::: {#method.vzip .section .method .trait-impl}
[Source](../../../src/ppv_lite86/types.rs.html#226){.src
.rightside}[§](#method.vzip){.anchor}

#### fn [vzip](../../../ppv_lite86/types/trait.VZip.html#tymethod.vzip){.fn}(self) -\> V {#fn-vzipself---v .code-header}
:::
::::

::: {#impl-WithSubscriber-for-T .section .impl}
[Source](../../../src/tracing/instrument.rs.html#393){.src
.rightside}[§](#impl-WithSubscriber-for-T){.anchor}

### impl\<T\> [WithSubscriber](../../../tracing/instrument/trait.WithSubscriber.html "trait tracing::instrument::WithSubscriber"){.trait} for T {#implt-withsubscriber-for-t .code-header}
:::

:::::::: impl-items
:::: {#method.with_subscriber .section .method .trait-impl}
[Source](../../../src/tracing/instrument.rs.html#176-178){.src
.rightside}[§](#method.with_subscriber){.anchor}

#### fn [with_subscriber](../../../tracing/instrument/trait.WithSubscriber.html#method.with_subscriber){.fn}\<S\>(self, subscriber: S) -\> [WithDispatch](../../../tracing/instrument/struct.WithDispatch.html "struct tracing::instrument::WithDispatch"){.struct}\<Self\> {#fn-with_subscribersself-subscriber-s---withdispatchself .code-header}

::: where
where S:
[Into](https://doc.rust-lang.org/1.91.1/core/convert/trait.Into.html "trait core::convert::Into"){.trait}\<[Dispatch](../../../tracing_core/dispatcher/struct.Dispatch.html "struct tracing_core::dispatcher::Dispatch"){.struct}\>,
:::
::::

::: docblock
Attaches the provided
[`Subscriber`](../../../tracing_core/subscriber/trait.Subscriber.html "trait tracing_core::subscriber::Subscriber")
to this type, returning a
[`WithDispatch`](../../../tracing/instrument/struct.WithDispatch.html "struct tracing::instrument::WithDispatch")
wrapper. [Read
more](../../../tracing/instrument/trait.WithSubscriber.html#method.with_subscriber)
:::

::: {#method.with_current_subscriber .section .method .trait-impl}
[Source](../../../src/tracing/instrument.rs.html#228){.src
.rightside}[§](#method.with_current_subscriber){.anchor}

#### fn [with_current_subscriber](../../../tracing/instrument/trait.WithSubscriber.html#method.with_current_subscriber){.fn}(self) -\> [WithDispatch](../../../tracing/instrument/struct.WithDispatch.html "struct tracing::instrument::WithDispatch"){.struct}\<Self\> {#fn-with_current_subscriberself---withdispatchself .code-header}
:::

::: docblock
Attaches the current
[default](../../../tracing/dispatcher/index.html#setting-the-default-subscriber "mod tracing::dispatcher")
[`Subscriber`](../../../tracing_core/subscriber/trait.Subscriber.html "trait tracing_core::subscriber::Subscriber")
to this type, returning a
[`WithDispatch`](../../../tracing/instrument/struct.WithDispatch.html "struct tracing::instrument::WithDispatch")
wrapper. [Read
more](../../../tracing/instrument/trait.WithSubscriber.html#method.with_current_subscriber)
:::
::::::::

:::: {#impl-DeserializeOwned-for-T .section .impl}
[Source](../../../src/serde_core/de/mod.rs.html#633){.src
.rightside}[§](#impl-DeserializeOwned-for-T){.anchor}

### impl\<T\> [DeserializeOwned](../../../serde_core/de/trait.DeserializeOwned.html "trait serde_core::de::DeserializeOwned"){.trait} for T {#implt-deserializeowned-for-t .code-header}

::: where
where T: for\<\'de\>
[Deserialize](../../../serde_core/de/trait.Deserialize.html "trait serde_core::de::Deserialize"){.trait}\<\'de\>,
:::
::::
:::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
:::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
