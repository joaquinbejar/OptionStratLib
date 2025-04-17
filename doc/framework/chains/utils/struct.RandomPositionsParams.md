::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::: width-limiter
:::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../../index.html)::[chains](../index.html)::[utils](index.html)
:::

# Struct [RandomPositionsParams]{.struct}Copy item path

[[Source](../../../src/optionstratlib/chains/utils.rs.html#513-538){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub struct RandomPositionsParams {
    pub qty_puts_long: Option<usize>,
    pub qty_puts_short: Option<usize>,
    pub qty_calls_long: Option<usize>,
    pub qty_calls_short: Option<usize>,
    pub expiration_date: ExpirationDate,
    pub option_qty: Positive,
    pub risk_free_rate: Decimal,
    pub dividend_yield: Positive,
    pub open_put_fee: Positive,
    pub open_call_fee: Positive,
    pub close_put_fee: Positive,
    pub close_call_fee: Positive,
}
```

Expand description

::: docblock
Parameters for generating random positions in an option chain
:::

## Fields[§](#fields){.anchor} {#fields .fields .section-header}

[[§](#structfield.qty_puts_long){.anchor
.field}`qty_puts_long: `[`Option`](https://doc.rust-lang.org/1.86.0/core/option/enum.Option.html "enum core::option::Option"){.enum}`<`[`usize`](https://doc.rust-lang.org/1.86.0/std/primitive.usize.html){.primitive}`>`]{#structfield.qty_puts_long
.structfield .section-header}

::: docblock
Number of long put positions to generate
:::

[[§](#structfield.qty_puts_short){.anchor
.field}`qty_puts_short: `[`Option`](https://doc.rust-lang.org/1.86.0/core/option/enum.Option.html "enum core::option::Option"){.enum}`<`[`usize`](https://doc.rust-lang.org/1.86.0/std/primitive.usize.html){.primitive}`>`]{#structfield.qty_puts_short
.structfield .section-header}

::: docblock
Number of short put positions to generate
:::

[[§](#structfield.qty_calls_long){.anchor
.field}`qty_calls_long: `[`Option`](https://doc.rust-lang.org/1.86.0/core/option/enum.Option.html "enum core::option::Option"){.enum}`<`[`usize`](https://doc.rust-lang.org/1.86.0/std/primitive.usize.html){.primitive}`>`]{#structfield.qty_calls_long
.structfield .section-header}

::: docblock
Number of long call positions to generate
:::

[[§](#structfield.qty_calls_short){.anchor
.field}`qty_calls_short: `[`Option`](https://doc.rust-lang.org/1.86.0/core/option/enum.Option.html "enum core::option::Option"){.enum}`<`[`usize`](https://doc.rust-lang.org/1.86.0/std/primitive.usize.html){.primitive}`>`]{#structfield.qty_calls_short
.structfield .section-header}

::: docblock
Number of short call positions to generate
:::

[[§](#structfield.expiration_date){.anchor
.field}`expiration_date: `[`ExpirationDate`](../../model/types/enum.ExpirationDate.html "enum optionstratlib::model::types::ExpirationDate"){.enum}]{#structfield.expiration_date
.structfield .section-header}

::: docblock
Expiration date for the options
:::

[[§](#structfield.option_qty){.anchor
.field}`option_qty: `[`Positive`](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}]{#structfield.option_qty
.structfield .section-header}

::: docblock
Quantity for each option position
:::

[[§](#structfield.risk_free_rate){.anchor
.field}`risk_free_rate: Decimal`]{#structfield.risk_free_rate
.structfield .section-header}

::: docblock
Risk free interest rate
:::

[[§](#structfield.dividend_yield){.anchor
.field}`dividend_yield: `[`Positive`](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}]{#structfield.dividend_yield
.structfield .section-header}

::: docblock
Dividend yield of the underlying
:::

[[§](#structfield.open_put_fee){.anchor
.field}`open_put_fee: `[`Positive`](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}]{#structfield.open_put_fee
.structfield .section-header}

::: docblock
Fee for opening put positions
:::

[[§](#structfield.open_call_fee){.anchor
.field}`open_call_fee: `[`Positive`](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}]{#structfield.open_call_fee
.structfield .section-header}

::: docblock
Fee for opening call positions
:::

[[§](#structfield.close_put_fee){.anchor
.field}`close_put_fee: `[`Positive`](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}]{#structfield.close_put_fee
.structfield .section-header}

::: docblock
Fee for closing put positions
:::

[[§](#structfield.close_call_fee){.anchor
.field}`close_call_fee: `[`Positive`](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}]{#structfield.close_call_fee
.structfield .section-header}

::: docblock
Fee for closing call positions
:::

## Implementations[§](#implementations){.anchor} {#implementations .section-header}

::::::::: {#implementations-list}
::: {#impl-RandomPositionsParams .section .impl}
[Source](../../../src/optionstratlib/chains/utils.rs.html#540-617){.src
.rightside}[§](#impl-RandomPositionsParams){.anchor}

### impl [RandomPositionsParams](struct.RandomPositionsParams.html "struct optionstratlib::chains::utils::RandomPositionsParams"){.struct} {#impl-randompositionsparams .code-header}
:::

::::::: impl-items
::: {#method.new .section .method}
[Source](../../../src/optionstratlib/chains/utils.rs.html#572-600){.src
.rightside}

#### pub fn [new](#method.new){.fn}( qty_puts_long: [Option](https://doc.rust-lang.org/1.86.0/core/option/enum.Option.html "enum core::option::Option"){.enum}\<[usize](https://doc.rust-lang.org/1.86.0/std/primitive.usize.html){.primitive}\>, qty_puts_short: [Option](https://doc.rust-lang.org/1.86.0/core/option/enum.Option.html "enum core::option::Option"){.enum}\<[usize](https://doc.rust-lang.org/1.86.0/std/primitive.usize.html){.primitive}\>, qty_calls_long: [Option](https://doc.rust-lang.org/1.86.0/core/option/enum.Option.html "enum core::option::Option"){.enum}\<[usize](https://doc.rust-lang.org/1.86.0/std/primitive.usize.html){.primitive}\>, qty_calls_short: [Option](https://doc.rust-lang.org/1.86.0/core/option/enum.Option.html "enum core::option::Option"){.enum}\<[usize](https://doc.rust-lang.org/1.86.0/std/primitive.usize.html){.primitive}\>, expiration_date: [ExpirationDate](../../model/types/enum.ExpirationDate.html "enum optionstratlib::model::types::ExpirationDate"){.enum}, option_qty: [Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, risk_free_rate: Decimal, dividend_yield: [Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, open_put_fee: [Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, open_call_fee: [Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, close_put_fee: [Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, close_call_fee: [Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, ) -\> Self {#pub-fn-new-qty_puts_long-optionusize-qty_puts_short-optionusize-qty_calls_long-optionusize-qty_calls_short-optionusize-expiration_date-expirationdate-option_qty-positive-risk_free_rate-decimal-dividend_yield-positive-open_put_fee-positive-open_call_fee-positive-close_put_fee-positive-close_call_fee-positive---self .code-header}
:::

::: docblock
Creates a new instance of `RandomPositionsParams` with the specified
parameters.

This constructor initializes a configuration object that defines
parameters for generating random option positions in an option chain. It
allows specifying the quantity of different option types (puts/calls,
long/short), expiration settings, and various fee structures.

##### [§](#parameters){.doc-anchor}Parameters

- `qty_puts_long` - Optional number of long put positions to generate
- `qty_puts_short` - Optional number of short put positions to generate
- `qty_calls_long` - Optional number of long call positions to generate
- `qty_calls_short` - Optional number of short call positions to
  generate
- `expiration_date` - The expiration date for the options (can be
  specified as days from now or absolute date)
- `option_qty` - The quantity of contracts for each option position
- `risk_free_rate` - The risk-free interest rate used for option pricing
  calculations
- `dividend_yield` - The dividend yield of the underlying asset
- `open_put_fee` - The fee charged when opening put positions
- `open_call_fee` - The fee charged when opening call positions
- `close_put_fee` - The fee charged when closing put positions
- `close_call_fee` - The fee charged when closing call positions

##### [§](#returns){.doc-anchor}Returns

A new `RandomPositionsParams` instance with the specified configuration.

##### [§](#note){.doc-anchor}Note

This function has many parameters, but this is justified by the complex
nature of option position generation which requires detailed
configuration.
:::

::: {#method.total_positions .section .method}
[Source](../../../src/optionstratlib/chains/utils.rs.html#611-616){.src
.rightside}

#### pub fn [total_positions](#method.total_positions){.fn}(&self) -\> [usize](https://doc.rust-lang.org/1.86.0/std/primitive.usize.html){.primitive} {#pub-fn-total_positionsself---usize .code-header}
:::

::: docblock
Returns the total number of positions to generate.

This method calculates the sum of all option position types (puts
long/short and calls long/short) that need to be generated based on the
current configuration. If any position type is not specified (None), it
is treated as zero.

##### [§](#returns-1){.doc-anchor}Returns

The total number of option positions to be generated.
:::
:::::::
:::::::::

## Trait Implementations[§](#trait-implementations){.anchor} {#trait-implementations .section-header}

::::::::::::: {#trait-implementations-list}
::: {#impl-Clone-for-RandomPositionsParams .section .impl}
[Source](../../../src/optionstratlib/chains/utils.rs.html#512){.src
.rightside}[§](#impl-Clone-for-RandomPositionsParams){.anchor}

### impl [Clone](https://doc.rust-lang.org/1.86.0/core/clone/trait.Clone.html "trait core::clone::Clone"){.trait} for [RandomPositionsParams](struct.RandomPositionsParams.html "struct optionstratlib::chains::utils::RandomPositionsParams"){.struct} {#impl-clone-for-randompositionsparams .code-header}
:::

::::::: impl-items
::: {#method.clone .section .method .trait-impl}
[Source](../../../src/optionstratlib/chains/utils.rs.html#512){.src
.rightside}[§](#method.clone){.anchor}

#### fn [clone](https://doc.rust-lang.org/1.86.0/core/clone/trait.Clone.html#tymethod.clone){.fn}(&self) -\> [RandomPositionsParams](struct.RandomPositionsParams.html "struct optionstratlib::chains::utils::RandomPositionsParams"){.struct} {#fn-cloneself---randompositionsparams .code-header}
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

::: {#impl-Debug-for-RandomPositionsParams .section .impl}
[Source](../../../src/optionstratlib/chains/utils.rs.html#512){.src
.rightside}[§](#impl-Debug-for-RandomPositionsParams){.anchor}

### impl [Debug](https://doc.rust-lang.org/1.86.0/core/fmt/trait.Debug.html "trait core::fmt::Debug"){.trait} for [RandomPositionsParams](struct.RandomPositionsParams.html "struct optionstratlib::chains::utils::RandomPositionsParams"){.struct} {#impl-debug-for-randompositionsparams .code-header}
:::

::::: impl-items
::: {#method.fmt .section .method .trait-impl}
[Source](../../../src/optionstratlib/chains/utils.rs.html#512){.src
.rightside}[§](#method.fmt){.anchor}

#### fn [fmt](https://doc.rust-lang.org/1.86.0/core/fmt/trait.Debug.html#tymethod.fmt){.fn}(&self, f: &mut [Formatter](https://doc.rust-lang.org/1.86.0/core/fmt/struct.Formatter.html "struct core::fmt::Formatter"){.struct}\<\'\_\>) -\> [Result](https://doc.rust-lang.org/1.86.0/core/fmt/type.Result.html "type core::fmt::Result"){.type} {#fn-fmtself-f-mut-formatter_---result .code-header}
:::

::: docblock
Formats the value using the given formatter. [Read
more](https://doc.rust-lang.org/1.86.0/core/fmt/trait.Debug.html#tymethod.fmt)
:::
:::::
:::::::::::::

## Auto Trait Implementations[§](#synthetic-implementations){.anchor} {#synthetic-implementations .section-header}

::::::::: {#synthetic-implementations-list}
::: {#impl-Freeze-for-RandomPositionsParams .section .impl}
[§](#impl-Freeze-for-RandomPositionsParams){.anchor}

### impl [Freeze](https://doc.rust-lang.org/1.86.0/core/marker/trait.Freeze.html "trait core::marker::Freeze"){.trait} for [RandomPositionsParams](struct.RandomPositionsParams.html "struct optionstratlib::chains::utils::RandomPositionsParams"){.struct} {#impl-freeze-for-randompositionsparams .code-header}
:::

::: {#impl-RefUnwindSafe-for-RandomPositionsParams .section .impl}
[§](#impl-RefUnwindSafe-for-RandomPositionsParams){.anchor}

### impl [RefUnwindSafe](https://doc.rust-lang.org/1.86.0/core/panic/unwind_safe/trait.RefUnwindSafe.html "trait core::panic::unwind_safe::RefUnwindSafe"){.trait} for [RandomPositionsParams](struct.RandomPositionsParams.html "struct optionstratlib::chains::utils::RandomPositionsParams"){.struct} {#impl-refunwindsafe-for-randompositionsparams .code-header}
:::

::: {#impl-Send-for-RandomPositionsParams .section .impl}
[§](#impl-Send-for-RandomPositionsParams){.anchor}

### impl [Send](https://doc.rust-lang.org/1.86.0/core/marker/trait.Send.html "trait core::marker::Send"){.trait} for [RandomPositionsParams](struct.RandomPositionsParams.html "struct optionstratlib::chains::utils::RandomPositionsParams"){.struct} {#impl-send-for-randompositionsparams .code-header}
:::

::: {#impl-Sync-for-RandomPositionsParams .section .impl}
[§](#impl-Sync-for-RandomPositionsParams){.anchor}

### impl [Sync](https://doc.rust-lang.org/1.86.0/core/marker/trait.Sync.html "trait core::marker::Sync"){.trait} for [RandomPositionsParams](struct.RandomPositionsParams.html "struct optionstratlib::chains::utils::RandomPositionsParams"){.struct} {#impl-sync-for-randompositionsparams .code-header}
:::

::: {#impl-Unpin-for-RandomPositionsParams .section .impl}
[§](#impl-Unpin-for-RandomPositionsParams){.anchor}

### impl [Unpin](https://doc.rust-lang.org/1.86.0/core/marker/trait.Unpin.html "trait core::marker::Unpin"){.trait} for [RandomPositionsParams](struct.RandomPositionsParams.html "struct optionstratlib::chains::utils::RandomPositionsParams"){.struct} {#impl-unpin-for-randompositionsparams .code-header}
:::

::: {#impl-UnwindSafe-for-RandomPositionsParams .section .impl}
[§](#impl-UnwindSafe-for-RandomPositionsParams){.anchor}

### impl [UnwindSafe](https://doc.rust-lang.org/1.86.0/core/panic/unwind_safe/trait.UnwindSafe.html "trait core::panic::unwind_safe::UnwindSafe"){.trait} for [RandomPositionsParams](struct.RandomPositionsParams.html "struct optionstratlib::chains::utils::RandomPositionsParams"){.struct} {#impl-unwindsafe-for-randompositionsparams .code-header}
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
::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
:::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
