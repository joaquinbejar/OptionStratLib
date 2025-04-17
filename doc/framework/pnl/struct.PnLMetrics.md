:::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::: width-limiter
::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../index.html)::[pnl](index.html)
:::

# Struct [PnLMetrics]{.struct}Copy item path

[[Source](../../src/optionstratlib/pnl/metrics.rs.html#89-128){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub struct PnLMetrics {Show 19 fields
    pub total_pnl: Decimal,
    pub max_profit: Positive,
    pub max_loss: Positive,
    pub win_rate: Decimal,
    pub loss_rate: Decimal,
    pub total_steps: u32,
    pub winning_steps: u32,
    pub losing_steps: u32,
    pub avg_win: Decimal,
    pub avg_loss: Decimal,
    pub max_drawdown: Positive,
    pub sharpe_ratio: Decimal,
    pub sortino_ratio: Decimal,
    pub profit_factor: Decimal,
    pub recovery_factor: Decimal,
    pub expected_payoff: Decimal,
    pub simulation_duration: Decimal,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
}
```

Expand description

::: docblock
`PnLMetrics` struct holds various metrics related to Profit and Loss
(PnL) analysis.

This struct captures a comprehensive set of metrics including total PnL,
maximum profit and loss, win/loss rates, average win/loss amounts,
maximum drawdown, Sharpe and Sortino ratios, profit and recovery
factors, expected payoff, simulation duration, and start/end times.

It is designed to provide a detailed overview of the performance and
risk characteristics of a trading strategy or simulation.
:::

## Fields[§](#fields){.anchor} {#fields .fields .section-header}

[[§](#structfield.total_pnl){.anchor
.field}`total_pnl: Decimal`]{#structfield.total_pnl .structfield
.section-header}

::: docblock
The total profit and loss.
:::

[[§](#structfield.max_profit){.anchor
.field}`max_profit: `[`Positive`](../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}]{#structfield.max_profit
.structfield .section-header}

::: docblock
The maximum profit achieved.
:::

[[§](#structfield.max_loss){.anchor
.field}`max_loss: `[`Positive`](../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}]{#structfield.max_loss
.structfield .section-header}

::: docblock
The maximum loss incurred.
:::

[[§](#structfield.win_rate){.anchor
.field}`win_rate: Decimal`]{#structfield.win_rate .structfield
.section-header}

::: docblock
The win rate, i.e., the proportion of winning trades.
:::

[[§](#structfield.loss_rate){.anchor
.field}`loss_rate: Decimal`]{#structfield.loss_rate .structfield
.section-header}

::: docblock
The loss rate, i.e., the proportion of losing trades.
:::

[[§](#structfield.total_steps){.anchor
.field}`total_steps: `[`u32`](https://doc.rust-lang.org/1.86.0/std/primitive.u32.html){.primitive}]{#structfield.total_steps
.structfield .section-header}

::: docblock
The total number of steps in the simulation or trading period.
:::

[[§](#structfield.winning_steps){.anchor
.field}`winning_steps: `[`u32`](https://doc.rust-lang.org/1.86.0/std/primitive.u32.html){.primitive}]{#structfield.winning_steps
.structfield .section-header}

::: docblock
The number of steps that resulted in a win.
:::

[[§](#structfield.losing_steps){.anchor
.field}`losing_steps: `[`u32`](https://doc.rust-lang.org/1.86.0/std/primitive.u32.html){.primitive}]{#structfield.losing_steps
.structfield .section-header}

::: docblock
The number of steps that resulted in a loss.
:::

[[§](#structfield.avg_win){.anchor
.field}`avg_win: Decimal`]{#structfield.avg_win .structfield
.section-header}

::: docblock
The average win amount.
:::

[[§](#structfield.avg_loss){.anchor
.field}`avg_loss: Decimal`]{#structfield.avg_loss .structfield
.section-header}

::: docblock
The average loss amount.
:::

[[§](#structfield.max_drawdown){.anchor
.field}`max_drawdown: `[`Positive`](../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}]{#structfield.max_drawdown
.structfield .section-header}

::: docblock
The maximum drawdown experienced.
:::

[[§](#structfield.sharpe_ratio){.anchor
.field}`sharpe_ratio: Decimal`]{#structfield.sharpe_ratio .structfield
.section-header}

::: docblock
The Sharpe ratio, a measure of risk-adjusted return.
:::

[[§](#structfield.sortino_ratio){.anchor
.field}`sortino_ratio: Decimal`]{#structfield.sortino_ratio .structfield
.section-header}

::: docblock
The Sortino ratio, a variation of the Sharpe ratio that only considers
downside risk.
:::

[[§](#structfield.profit_factor){.anchor
.field}`profit_factor: Decimal`]{#structfield.profit_factor .structfield
.section-header}

::: docblock
The profit factor, calculated as gross profit divided by gross loss.
:::

[[§](#structfield.recovery_factor){.anchor
.field}`recovery_factor: Decimal`]{#structfield.recovery_factor
.structfield .section-header}

::: docblock
The recovery factor, a measure of how quickly losses are recovered.
:::

[[§](#structfield.expected_payoff){.anchor
.field}`expected_payoff: Decimal`]{#structfield.expected_payoff
.structfield .section-header}

::: docblock
The expected payoff per trade or step.
:::

[[§](#structfield.simulation_duration){.anchor
.field}`simulation_duration: Decimal`]{#structfield.simulation_duration
.structfield .section-header}

::: docblock
The duration of the simulation or trading period.
:::

[[§](#structfield.start_time){.anchor
.field}`start_time: `[`DateTime`](https://docs.rs/chrono/latest/chrono/datetime/struct.DateTime.html "struct chrono::datetime::DateTime"){.struct}`<`[`Utc`](https://docs.rs/chrono/latest/chrono/offset/utc/struct.Utc.html "struct chrono::offset::utc::Utc"){.struct}`>`]{#structfield.start_time
.structfield .section-header}

::: docblock
The start time of the simulation or trading period.
:::

[[§](#structfield.end_time){.anchor
.field}`end_time: `[`DateTime`](https://docs.rs/chrono/latest/chrono/datetime/struct.DateTime.html "struct chrono::datetime::DateTime"){.struct}`<`[`Utc`](https://docs.rs/chrono/latest/chrono/offset/utc/struct.Utc.html "struct chrono::offset::utc::Utc"){.struct}`>`]{#structfield.end_time
.structfield .section-header}

::: docblock
The end time of the simulation or trading period.
:::

## Trait Implementations[§](#trait-implementations){.anchor} {#trait-implementations .section-header}

:::::::::::::::::::::::::::::::::: {#trait-implementations-list}
::: {#impl-Clone-for-PnLMetrics .section .impl}
[Source](../../src/optionstratlib/pnl/metrics.rs.html#88){.src
.rightside}[§](#impl-Clone-for-PnLMetrics){.anchor}

### impl [Clone](https://doc.rust-lang.org/1.86.0/core/clone/trait.Clone.html "trait core::clone::Clone"){.trait} for [PnLMetrics](struct.PnLMetrics.html "struct optionstratlib::pnl::PnLMetrics"){.struct} {#impl-clone-for-pnlmetrics .code-header}
:::

::::::: impl-items
::: {#method.clone .section .method .trait-impl}
[Source](../../src/optionstratlib/pnl/metrics.rs.html#88){.src
.rightside}[§](#method.clone){.anchor}

#### fn [clone](https://doc.rust-lang.org/1.86.0/core/clone/trait.Clone.html#tymethod.clone){.fn}(&self) -\> [PnLMetrics](struct.PnLMetrics.html "struct optionstratlib::pnl::PnLMetrics"){.struct} {#fn-cloneself---pnlmetrics .code-header}
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

::: {#impl-Debug-for-PnLMetrics .section .impl}
[Source](../../src/optionstratlib/pnl/metrics.rs.html#88){.src
.rightside}[§](#impl-Debug-for-PnLMetrics){.anchor}

### impl [Debug](https://doc.rust-lang.org/1.86.0/core/fmt/trait.Debug.html "trait core::fmt::Debug"){.trait} for [PnLMetrics](struct.PnLMetrics.html "struct optionstratlib::pnl::PnLMetrics"){.struct} {#impl-debug-for-pnlmetrics .code-header}
:::

::::: impl-items
::: {#method.fmt .section .method .trait-impl}
[Source](../../src/optionstratlib/pnl/metrics.rs.html#88){.src
.rightside}[§](#method.fmt){.anchor}

#### fn [fmt](https://doc.rust-lang.org/1.86.0/core/fmt/trait.Debug.html#tymethod.fmt){.fn}(&self, f: &mut [Formatter](https://doc.rust-lang.org/1.86.0/core/fmt/struct.Formatter.html "struct core::fmt::Formatter"){.struct}\<\'\_\>) -\> [Result](https://doc.rust-lang.org/1.86.0/core/fmt/type.Result.html "type core::fmt::Result"){.type} {#fn-fmtself-f-mut-formatter_---result .code-header}
:::

::: docblock
Formats the value using the given formatter. [Read
more](https://doc.rust-lang.org/1.86.0/core/fmt/trait.Debug.html#tymethod.fmt)
:::
:::::

::: {#impl-Default-for-PnLMetrics .section .impl}
[Source](../../src/optionstratlib/pnl/metrics.rs.html#130-154){.src
.rightside}[§](#impl-Default-for-PnLMetrics){.anchor}

### impl [Default](https://doc.rust-lang.org/1.86.0/core/default/trait.Default.html "trait core::default::Default"){.trait} for [PnLMetrics](struct.PnLMetrics.html "struct optionstratlib::pnl::PnLMetrics"){.struct} {#impl-default-for-pnlmetrics .code-header}
:::

::::: impl-items
::: {#method.default .section .method .trait-impl}
[Source](../../src/optionstratlib/pnl/metrics.rs.html#131-153){.src
.rightside}[§](#method.default){.anchor}

#### fn [default](https://doc.rust-lang.org/1.86.0/core/default/trait.Default.html#tymethod.default){.fn}() -\> Self {#fn-default---self .code-header}
:::

::: docblock
Returns the "default value" for a type. [Read
more](https://doc.rust-lang.org/1.86.0/core/default/trait.Default.html#tymethod.default)
:::
:::::

::: {#impl-Deserialize%3C'de%3E-for-PnLMetrics .section .impl}
[Source](../../src/optionstratlib/pnl/metrics.rs.html#88){.src
.rightside}[§](#impl-Deserialize%3C'de%3E-for-PnLMetrics){.anchor}

### impl\<\'de\> [Deserialize](https://docs.rs/serde/1.0.219/serde/de/trait.Deserialize.html "trait serde::de::Deserialize"){.trait}\<\'de\> for [PnLMetrics](struct.PnLMetrics.html "struct optionstratlib::pnl::PnLMetrics"){.struct} {#implde-deserializede-for-pnlmetrics .code-header}
:::

:::::: impl-items
:::: {#method.deserialize .section .method .trait-impl}
[Source](../../src/optionstratlib/pnl/metrics.rs.html#88){.src
.rightside}[§](#method.deserialize){.anchor}

#### fn [deserialize](https://docs.rs/serde/1.0.219/serde/de/trait.Deserialize.html#tymethod.deserialize){.fn}\<\_\_D\>(\_\_deserializer: \_\_D) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<Self, \_\_D::[Error](https://docs.rs/serde/1.0.219/serde/de/trait.Deserializer.html#associatedtype.Error "type serde::de::Deserializer::Error"){.associatedtype}\> {#fn-deserialize__d__deserializer-__d---resultself-__derror .code-header}

::: where
where \_\_D:
[Deserializer](https://docs.rs/serde/1.0.219/serde/de/trait.Deserializer.html "trait serde::de::Deserializer"){.trait}\<\'de\>,
:::
::::

::: docblock
Deserialize this value from the given Serde deserializer. [Read
more](https://docs.rs/serde/1.0.219/serde/de/trait.Deserialize.html#tymethod.deserialize)
:::
::::::

::: {#impl-PartialEq-for-PnLMetrics .section .impl}
[Source](../../src/optionstratlib/pnl/metrics.rs.html#88){.src
.rightside}[§](#impl-PartialEq-for-PnLMetrics){.anchor}

### impl [PartialEq](https://doc.rust-lang.org/1.86.0/core/cmp/trait.PartialEq.html "trait core::cmp::PartialEq"){.trait} for [PnLMetrics](struct.PnLMetrics.html "struct optionstratlib::pnl::PnLMetrics"){.struct} {#impl-partialeq-for-pnlmetrics .code-header}
:::

::::::: impl-items
::: {#method.eq .section .method .trait-impl}
[Source](../../src/optionstratlib/pnl/metrics.rs.html#88){.src
.rightside}[§](#method.eq){.anchor}

#### fn [eq](https://doc.rust-lang.org/1.86.0/core/cmp/trait.PartialEq.html#tymethod.eq){.fn}(&self, other: &[PnLMetrics](struct.PnLMetrics.html "struct optionstratlib::pnl::PnLMetrics"){.struct}) -\> [bool](https://doc.rust-lang.org/1.86.0/std/primitive.bool.html){.primitive} {#fn-eqself-other-pnlmetrics---bool .code-header}
:::

::: docblock
Tests for `self` and `other` values to be equal, and is used by `==`.
:::

::: {#method.ne .section .method .trait-impl}
[[1.0.0]{.since title="Stable since Rust version 1.0.0"} ·
[Source](https://doc.rust-lang.org/1.86.0/src/core/cmp.rs.html#261){.src}]{.rightside}[§](#method.ne){.anchor}

#### fn [ne](https://doc.rust-lang.org/1.86.0/core/cmp/trait.PartialEq.html#method.ne){.fn}(&self, other: [&Rhs](https://doc.rust-lang.org/1.86.0/std/primitive.reference.html){.primitive}) -\> [bool](https://doc.rust-lang.org/1.86.0/std/primitive.bool.html){.primitive} {#fn-neself-other-rhs---bool .code-header}
:::

::: docblock
Tests for `!=`. The default implementation is almost always sufficient,
and should not be overridden without very good reason.
:::
:::::::

::: {#impl-Serialize-for-PnLMetrics .section .impl}
[Source](../../src/optionstratlib/pnl/metrics.rs.html#88){.src
.rightside}[§](#impl-Serialize-for-PnLMetrics){.anchor}

### impl [Serialize](https://docs.rs/serde/1.0.219/serde/ser/trait.Serialize.html "trait serde::ser::Serialize"){.trait} for [PnLMetrics](struct.PnLMetrics.html "struct optionstratlib::pnl::PnLMetrics"){.struct} {#impl-serialize-for-pnlmetrics .code-header}
:::

:::::: impl-items
:::: {#method.serialize .section .method .trait-impl}
[Source](../../src/optionstratlib/pnl/metrics.rs.html#88){.src
.rightside}[§](#method.serialize){.anchor}

#### fn [serialize](https://docs.rs/serde/1.0.219/serde/ser/trait.Serialize.html#tymethod.serialize){.fn}\<\_\_S\>(&self, \_\_serializer: \_\_S) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<\_\_S::[Ok](https://docs.rs/serde/1.0.219/serde/ser/trait.Serializer.html#associatedtype.Ok "type serde::ser::Serializer::Ok"){.associatedtype}, \_\_S::[Error](https://docs.rs/serde/1.0.219/serde/ser/trait.Serializer.html#associatedtype.Error "type serde::ser::Serializer::Error"){.associatedtype}\> {#fn-serialize__sself-__serializer-__s---result__sok-__serror .code-header}

::: where
where \_\_S:
[Serializer](https://docs.rs/serde/1.0.219/serde/ser/trait.Serializer.html "trait serde::ser::Serializer"){.trait},
:::
::::

::: docblock
Serialize this value into the given Serde serializer. [Read
more](https://docs.rs/serde/1.0.219/serde/ser/trait.Serialize.html#tymethod.serialize)
:::
::::::

::: {#impl-StructuralPartialEq-for-PnLMetrics .section .impl}
[Source](../../src/optionstratlib/pnl/metrics.rs.html#88){.src
.rightside}[§](#impl-StructuralPartialEq-for-PnLMetrics){.anchor}

### impl [StructuralPartialEq](https://doc.rust-lang.org/1.86.0/core/marker/trait.StructuralPartialEq.html "trait core::marker::StructuralPartialEq"){.trait} for [PnLMetrics](struct.PnLMetrics.html "struct optionstratlib::pnl::PnLMetrics"){.struct} {#impl-structuralpartialeq-for-pnlmetrics .code-header}
:::
::::::::::::::::::::::::::::::::::

## Auto Trait Implementations[§](#synthetic-implementations){.anchor} {#synthetic-implementations .section-header}

::::::::: {#synthetic-implementations-list}
::: {#impl-Freeze-for-PnLMetrics .section .impl}
[§](#impl-Freeze-for-PnLMetrics){.anchor}

### impl [Freeze](https://doc.rust-lang.org/1.86.0/core/marker/trait.Freeze.html "trait core::marker::Freeze"){.trait} for [PnLMetrics](struct.PnLMetrics.html "struct optionstratlib::pnl::PnLMetrics"){.struct} {#impl-freeze-for-pnlmetrics .code-header}
:::

::: {#impl-RefUnwindSafe-for-PnLMetrics .section .impl}
[§](#impl-RefUnwindSafe-for-PnLMetrics){.anchor}

### impl [RefUnwindSafe](https://doc.rust-lang.org/1.86.0/core/panic/unwind_safe/trait.RefUnwindSafe.html "trait core::panic::unwind_safe::RefUnwindSafe"){.trait} for [PnLMetrics](struct.PnLMetrics.html "struct optionstratlib::pnl::PnLMetrics"){.struct} {#impl-refunwindsafe-for-pnlmetrics .code-header}
:::

::: {#impl-Send-for-PnLMetrics .section .impl}
[§](#impl-Send-for-PnLMetrics){.anchor}

### impl [Send](https://doc.rust-lang.org/1.86.0/core/marker/trait.Send.html "trait core::marker::Send"){.trait} for [PnLMetrics](struct.PnLMetrics.html "struct optionstratlib::pnl::PnLMetrics"){.struct} {#impl-send-for-pnlmetrics .code-header}
:::

::: {#impl-Sync-for-PnLMetrics .section .impl}
[§](#impl-Sync-for-PnLMetrics){.anchor}

### impl [Sync](https://doc.rust-lang.org/1.86.0/core/marker/trait.Sync.html "trait core::marker::Sync"){.trait} for [PnLMetrics](struct.PnLMetrics.html "struct optionstratlib::pnl::PnLMetrics"){.struct} {#impl-sync-for-pnlmetrics .code-header}
:::

::: {#impl-Unpin-for-PnLMetrics .section .impl}
[§](#impl-Unpin-for-PnLMetrics){.anchor}

### impl [Unpin](https://doc.rust-lang.org/1.86.0/core/marker/trait.Unpin.html "trait core::marker::Unpin"){.trait} for [PnLMetrics](struct.PnLMetrics.html "struct optionstratlib::pnl::PnLMetrics"){.struct} {#impl-unpin-for-pnlmetrics .code-header}
:::

::: {#impl-UnwindSafe-for-PnLMetrics .section .impl}
[§](#impl-UnwindSafe-for-PnLMetrics){.anchor}

### impl [UnwindSafe](https://doc.rust-lang.org/1.86.0/core/panic/unwind_safe/trait.UnwindSafe.html "trait core::panic::unwind_safe::UnwindSafe"){.trait} for [PnLMetrics](struct.PnLMetrics.html "struct optionstratlib::pnl::PnLMetrics"){.struct} {#impl-unwindsafe-for-pnlmetrics .code-header}
:::
:::::::::

## Blanket Implementations[§](#blanket-implementations){.anchor} {#blanket-implementations .section-header}

::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::: {#blanket-implementations-list}
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

:::: {#impl-DeserializeOwned-for-T .section .impl}
[Source](https://docs.rs/serde/1.0.219/src/serde/de/mod.rs.html#614){.src
.rightside}[§](#impl-DeserializeOwned-for-T){.anchor}

### impl\<T\> [DeserializeOwned](https://docs.rs/serde/1.0.219/serde/de/trait.DeserializeOwned.html "trait serde::de::DeserializeOwned"){.trait} for T {#implt-deserializeowned-for-t .code-header}

::: where
where T: for\<\'de\>
[Deserialize](https://docs.rs/serde/1.0.219/serde/de/trait.Deserialize.html "trait serde::de::Deserialize"){.trait}\<\'de\>,
:::
::::

:::: {#impl-Scalar-for-T .section .impl}
[Source](https://docs.rs/nalgebra/0.25.0/src/nalgebra/base/scalar.rs.html#8){.src
.rightside}[§](#impl-Scalar-for-T){.anchor}

### impl\<T\> [Scalar](https://docs.rs/nalgebra/0.25.0/nalgebra/base/scalar/trait.Scalar.html "trait nalgebra::base::scalar::Scalar"){.trait} for T {#implt-scalar-for-t .code-header}

::: where
where T: \'static +
[Clone](https://doc.rust-lang.org/1.86.0/core/clone/trait.Clone.html "trait core::clone::Clone"){.trait} +
[PartialEq](https://doc.rust-lang.org/1.86.0/core/cmp/trait.PartialEq.html "trait core::cmp::PartialEq"){.trait} +
[Debug](https://doc.rust-lang.org/1.86.0/core/fmt/trait.Debug.html "trait core::fmt::Debug"){.trait},
:::
::::
:::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
:::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
