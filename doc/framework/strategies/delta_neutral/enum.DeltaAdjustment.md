:::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::: width-limiter
::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../../index.html)::[strategies](../index.html)::[delta_neutral](index.html)
:::

# Enum [DeltaAdjustment]{.enum} Copy item path

[[Source](../../../src/optionstratlib/strategies/delta_neutral/model.rs.html#100-168){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub enum DeltaAdjustment {
    BuyOptions {
        quantity: Positive,
        strike: Positive,
        option_style: OptionStyle,
        side: Side,
    },
    SellOptions {
        quantity: Positive,
        strike: Positive,
        option_style: OptionStyle,
        side: Side,
    },
    BuyUnderlying(Positive),
    SellUnderlying(Positive),
    NoAdjustmentNeeded,
    SameSize(DeltaAdjustmentSameSize),
}
```

Expand description

::: docblock
The `DeltaAdjustment` enum is used to define how a trading strategy can
be modified to achieve or maintain a delta-neutral state. Delta
neutrality refers to a situation where the combined delta of all
positions is close to zero, minimizing directional market risk.

Variants:

- `BuyOptions`: Represents buying option contracts with specific
  parameters.
- `SellOptions`: Represents selling option contracts with specific
  parameters.
- `BuyUnderlying`: Represents buying units of the underlying asset.
- `SellUnderlying`: Represents selling units of the underlying asset.
- `NoAdjustmentNeeded`: Indicates that the strategy is already delta
  neutral within a specified threshold.
- `SameSize`: Represents combining two `DeltaAdjustment` actions of to
  keep the same size for balancing or maintaining neutrality.

Each variant provides detailed parameters needed for the associated
adjustment, such as the quantity of options or underlying asset and
other relevant details.
:::

## Variants[§](#variants){.anchor} {#variants .variants .section-header}

::::::::::::::::::::::::::::::::: variants
::: {#variant.BuyOptions .section .variant}
[§](#variant.BuyOptions){.anchor}

### BuyOptions {#buyoptions .code-header}
:::

::: docblock
Represents buying a number of option contracts to adjust delta.

Fields:

- `quantity`: The number of contracts to buy. This value is represented
  using the `Positive` type, ensuring it is non-negative.
- `strike`: The strike price of the options being purchased.
- `option_type`: Defines the type of option being purchased, either
  `Call` or `Put`, indicated by the `OptionStyle` enum.
:::

::::::::::: {#variant.BuyOptions.fields .sub-variant}
#### Fields

:::: sub-variant-field
[[§](#variant.BuyOptions.field.quantity){.anchor
.field}`quantity: `[`Positive`](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}]{#variant.BuyOptions.field.quantity
.section-header}

::: docblock
Number of contracts to buy.
:::
::::

:::: sub-variant-field
[[§](#variant.BuyOptions.field.strike){.anchor
.field}`strike: `[`Positive`](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}]{#variant.BuyOptions.field.strike
.section-header}

::: docblock
Strike price of the options.
:::
::::

:::: sub-variant-field
[[§](#variant.BuyOptions.field.option_style){.anchor
.field}`option_style: `[`OptionStyle`](../../model/types/enum.OptionStyle.html "enum optionstratlib::model::types::OptionStyle"){.enum}]{#variant.BuyOptions.field.option_style
.section-header}

::: docblock
Type of the option (Call or Put).
:::
::::

:::: sub-variant-field
[[§](#variant.BuyOptions.field.side){.anchor
.field}`side: `[`Side`](../../model/types/enum.Side.html "enum optionstratlib::model::types::Side"){.enum}]{#variant.BuyOptions.field.side
.section-header}

::: docblock
Side of the option (Long or Short).
:::
::::
:::::::::::

::: {#variant.SellOptions .section .variant}
[§](#variant.SellOptions){.anchor}

### SellOptions {#selloptions .code-header}
:::

::: docblock
Represents selling a number of option contracts to adjust delta.

Fields:

- `quantity`: The number of contracts to sell. This value is represented
  using the `Positive` type, ensuring it is non-negative.
- `strike`: The strike price of the options being sold.
- `option_type`: Defines the type of option being sold, either `Call` or
  `Put`, indicated by the `OptionStyle` enum.
:::

::::::::::: {#variant.SellOptions.fields .sub-variant}
#### Fields

:::: sub-variant-field
[[§](#variant.SellOptions.field.quantity){.anchor
.field}`quantity: `[`Positive`](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}]{#variant.SellOptions.field.quantity
.section-header}

::: docblock
Number of contracts to sell.
:::
::::

:::: sub-variant-field
[[§](#variant.SellOptions.field.strike){.anchor
.field}`strike: `[`Positive`](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}]{#variant.SellOptions.field.strike
.section-header}

::: docblock
Strike price of the options.
:::
::::

:::: sub-variant-field
[[§](#variant.SellOptions.field.option_style){.anchor
.field}`option_style: `[`OptionStyle`](../../model/types/enum.OptionStyle.html "enum optionstratlib::model::types::OptionStyle"){.enum}]{#variant.SellOptions.field.option_style
.section-header}

::: docblock
Type of the option (Call or Put).
:::
::::

:::: sub-variant-field
[[§](#variant.SellOptions.field.side){.anchor
.field}`side: `[`Side`](../../model/types/enum.Side.html "enum optionstratlib::model::types::Side"){.enum}]{#variant.SellOptions.field.side
.section-header}

::: docblock
side of the option (Long or Short).
:::
::::
:::::::::::

::: {#variant.BuyUnderlying .section .variant}
[§](#variant.BuyUnderlying){.anchor}

### BuyUnderlying([Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}) {#buyunderlyingpositive .code-header}
:::

::: docblock
Represents buying a quantity of the underlying asset to adjust delta.

This variant is used when buying shares, units, or equivalent of the
underlying asset is necessary to adjust the delta of a strategy.

- The `Positive` argument specifies the quantity being purchased.
:::

::: {#variant.SellUnderlying .section .variant}
[§](#variant.SellUnderlying){.anchor}

### SellUnderlying([Positive](../../model/positive/struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}) {#sellunderlyingpositive .code-header}
:::

::: docblock
Represents selling a quantity of the underlying asset to adjust delta.

This variant is used when selling shares, units, or equivalent of the
underlying asset is necessary to adjust the delta of a strategy.

- The `Positive` argument specifies the quantity being sold.
:::

::: {#variant.NoAdjustmentNeeded .section .variant}
[§](#variant.NoAdjustmentNeeded){.anchor}

### NoAdjustmentNeeded {#noadjustmentneeded .code-header}
:::

::: docblock
Indicates that no adjustment is needed to achieve delta neutrality.

This is used when the strategy's current delta falls within the
acceptable neutrality threshold, and no action is required.
:::

::: {#variant.SameSize .section .variant}
[§](#variant.SameSize){.anchor}

### SameSize(DeltaAdjustmentSameSize) {#samesizedeltaadjustmentsamesize .code-header}
:::

::: docblock
Combines two adjustments of the same size to maintain or balance delta
neutrality.

Fields:

- `DeltaAdjustment, DeltaAdjustment`: The two adjustments to keep the
  same size being combined.
:::
:::::::::::::::::::::::::::::::::

## Trait Implementations[§](#trait-implementations){.anchor} {#trait-implementations .section-header}

::::::::::::::::::::::::::::::::::::: {#trait-implementations-list}
::: {#impl-ComposeSchema-for-DeltaAdjustment .section .impl}
[Source](../../../src/optionstratlib/strategies/delta_neutral/model.rs.html#99){.src
.rightside}[§](#impl-ComposeSchema-for-DeltaAdjustment){.anchor}

### impl ComposeSchema for [DeltaAdjustment](enum.DeltaAdjustment.html "enum optionstratlib::strategies::delta_neutral::DeltaAdjustment"){.enum} {#impl-composeschema-for-deltaadjustment .code-header}
:::

:::: impl-items
::: {#method.compose .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/delta_neutral/model.rs.html#99){.src
.rightside}[§](#method.compose){.anchor}

#### fn [compose](#tymethod.compose){.fn}(generics: [Vec](https://doc.rust-lang.org/1.91.1/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<[RefOr](../../../utoipa/openapi/enum.RefOr.html "enum utoipa::openapi::RefOr"){.enum}\<[Schema](../../../utoipa/openapi/schema/enum.Schema.html "enum utoipa::openapi::schema::Schema"){.enum}\>\>) -\> [RefOr](../../../utoipa/openapi/enum.RefOr.html "enum utoipa::openapi::RefOr"){.enum}\<[Schema](../../../utoipa/openapi/schema/enum.Schema.html "enum utoipa::openapi::schema::Schema"){.enum}\> {#fn-composegenerics-vecreforschema---reforschema .code-header}
:::
::::

::: {#impl-Debug-for-DeltaAdjustment .section .impl}
[Source](../../../src/optionstratlib/strategies/delta_neutral/model.rs.html#99){.src
.rightside}[§](#impl-Debug-for-DeltaAdjustment){.anchor}

### impl [Debug](https://doc.rust-lang.org/1.91.1/core/fmt/trait.Debug.html "trait core::fmt::Debug"){.trait} for [DeltaAdjustment](enum.DeltaAdjustment.html "enum optionstratlib::strategies::delta_neutral::DeltaAdjustment"){.enum} {#impl-debug-for-deltaadjustment .code-header}
:::

::::: impl-items
::: {#method.fmt .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/delta_neutral/model.rs.html#99){.src
.rightside}[§](#method.fmt){.anchor}

#### fn [fmt](https://doc.rust-lang.org/1.91.1/core/fmt/trait.Debug.html#tymethod.fmt){.fn}(&self, f: &mut [Formatter](https://doc.rust-lang.org/1.91.1/core/fmt/struct.Formatter.html "struct core::fmt::Formatter"){.struct}\<\'\_\>) -\> [Result](https://doc.rust-lang.org/1.91.1/core/fmt/type.Result.html "type core::fmt::Result"){.type} {#fn-fmtself-f-mut-formatter_---result .code-header}
:::

::: docblock
Formats the value using the given formatter. [Read
more](https://doc.rust-lang.org/1.91.1/core/fmt/trait.Debug.html#tymethod.fmt)
:::
:::::

::: {#impl-Deserialize%3C'de%3E-for-DeltaAdjustment .section .impl}
[Source](../../../src/optionstratlib/strategies/delta_neutral/model.rs.html#99){.src
.rightside}[§](#impl-Deserialize%3C'de%3E-for-DeltaAdjustment){.anchor}

### impl\<\'de\> [Deserialize](../../../serde_core/de/trait.Deserialize.html "trait serde_core::de::Deserialize"){.trait}\<\'de\> for [DeltaAdjustment](enum.DeltaAdjustment.html "enum optionstratlib::strategies::delta_neutral::DeltaAdjustment"){.enum} {#implde-deserializede-for-deltaadjustment .code-header}
:::

:::::: impl-items
:::: {#method.deserialize .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/delta_neutral/model.rs.html#99){.src
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

::: {#impl-Display-for-DeltaAdjustment .section .impl}
[Source](../../../src/optionstratlib/strategies/delta_neutral/model.rs.html#176-219){.src
.rightside}[§](#impl-Display-for-DeltaAdjustment){.anchor}

### impl [Display](https://doc.rust-lang.org/1.91.1/core/fmt/trait.Display.html "trait core::fmt::Display"){.trait} for [DeltaAdjustment](enum.DeltaAdjustment.html "enum optionstratlib::strategies::delta_neutral::DeltaAdjustment"){.enum} {#impl-display-for-deltaadjustment .code-header}
:::

::::: impl-items
::: {#method.fmt-1 .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/delta_neutral/model.rs.html#177-218){.src
.rightside}[§](#method.fmt-1){.anchor}

#### fn [fmt](https://doc.rust-lang.org/1.91.1/core/fmt/trait.Display.html#tymethod.fmt){.fn}(&self, f: &mut [Formatter](https://doc.rust-lang.org/1.91.1/core/fmt/struct.Formatter.html "struct core::fmt::Formatter"){.struct}\<\'\_\>) -\> [Result](https://doc.rust-lang.org/1.91.1/core/fmt/type.Result.html "type core::fmt::Result"){.type} {#fn-fmtself-f-mut-formatter_---result-1 .code-header}
:::

::: docblock
Formats the value using the given formatter. [Read
more](https://doc.rust-lang.org/1.91.1/core/fmt/trait.Display.html#tymethod.fmt)
:::
:::::

::: {#impl-PartialEq-for-DeltaAdjustment .section .impl}
[Source](../../../src/optionstratlib/strategies/delta_neutral/model.rs.html#99){.src
.rightside}[§](#impl-PartialEq-for-DeltaAdjustment){.anchor}

### impl [PartialEq](https://doc.rust-lang.org/1.91.1/core/cmp/trait.PartialEq.html "trait core::cmp::PartialEq"){.trait} for [DeltaAdjustment](enum.DeltaAdjustment.html "enum optionstratlib::strategies::delta_neutral::DeltaAdjustment"){.enum} {#impl-partialeq-for-deltaadjustment .code-header}
:::

::::::: impl-items
::: {#method.eq .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/delta_neutral/model.rs.html#99){.src
.rightside}[§](#method.eq){.anchor}

#### fn [eq](https://doc.rust-lang.org/1.91.1/core/cmp/trait.PartialEq.html#tymethod.eq){.fn}(&self, other: &[DeltaAdjustment](enum.DeltaAdjustment.html "enum optionstratlib::strategies::delta_neutral::DeltaAdjustment"){.enum}) -\> [bool](https://doc.rust-lang.org/1.91.1/std/primitive.bool.html){.primitive} {#fn-eqself-other-deltaadjustment---bool .code-header}
:::

::: docblock
Tests for `self` and `other` values to be equal, and is used by `==`.
:::

::: {#method.ne .section .method .trait-impl}
[[1.0.0]{.since title="Stable since Rust version 1.0.0"} ·
[Source](https://doc.rust-lang.org/1.91.1/src/core/cmp.rs.html#264){.src}]{.rightside}[§](#method.ne){.anchor}

#### fn [ne](https://doc.rust-lang.org/1.91.1/core/cmp/trait.PartialEq.html#method.ne){.fn}(&self, other: [&Rhs](https://doc.rust-lang.org/1.91.1/std/primitive.reference.html){.primitive}) -\> [bool](https://doc.rust-lang.org/1.91.1/std/primitive.bool.html){.primitive} {#fn-neself-other-rhs---bool .code-header}
:::

::: docblock
Tests for `!=`. The default implementation is almost always sufficient,
and should not be overridden without very good reason.
:::
:::::::

::: {#impl-Serialize-for-DeltaAdjustment .section .impl}
[Source](../../../src/optionstratlib/strategies/delta_neutral/model.rs.html#99){.src
.rightside}[§](#impl-Serialize-for-DeltaAdjustment){.anchor}

### impl [Serialize](../../../serde_core/ser/trait.Serialize.html "trait serde_core::ser::Serialize"){.trait} for [DeltaAdjustment](enum.DeltaAdjustment.html "enum optionstratlib::strategies::delta_neutral::DeltaAdjustment"){.enum} {#impl-serialize-for-deltaadjustment .code-header}
:::

:::::: impl-items
:::: {#method.serialize .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/delta_neutral/model.rs.html#99){.src
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

::: {#impl-ToSchema-for-DeltaAdjustment .section .impl}
[Source](../../../src/optionstratlib/strategies/delta_neutral/model.rs.html#99){.src
.rightside}[§](#impl-ToSchema-for-DeltaAdjustment){.anchor}

### impl [ToSchema](../../../utoipa/trait.ToSchema.html "trait utoipa::ToSchema"){.trait} for [DeltaAdjustment](enum.DeltaAdjustment.html "enum optionstratlib::strategies::delta_neutral::DeltaAdjustment"){.enum} {#impl-toschema-for-deltaadjustment .code-header}
:::

::::::: impl-items
::: {#method.name .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/delta_neutral/model.rs.html#99){.src
.rightside}[§](#method.name){.anchor}

#### fn [name](../../../utoipa/trait.ToSchema.html#method.name){.fn}() -\> [Cow](https://doc.rust-lang.org/1.91.1/alloc/borrow/enum.Cow.html "enum alloc::borrow::Cow"){.enum}\<\'static, [str](https://doc.rust-lang.org/1.91.1/std/primitive.str.html){.primitive}\> {#fn-name---cowstatic-str .code-header}
:::

::: docblock
Return name of the schema. [Read
more](../../../utoipa/trait.ToSchema.html#method.name)
:::

::: {#method.schemas .section .method .trait-impl}
[Source](../../../src/optionstratlib/strategies/delta_neutral/model.rs.html#99){.src
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

::: {#impl-StructuralPartialEq-for-DeltaAdjustment .section .impl}
[Source](../../../src/optionstratlib/strategies/delta_neutral/model.rs.html#99){.src
.rightside}[§](#impl-StructuralPartialEq-for-DeltaAdjustment){.anchor}

### impl [StructuralPartialEq](https://doc.rust-lang.org/1.91.1/core/marker/trait.StructuralPartialEq.html "trait core::marker::StructuralPartialEq"){.trait} for [DeltaAdjustment](enum.DeltaAdjustment.html "enum optionstratlib::strategies::delta_neutral::DeltaAdjustment"){.enum} {#impl-structuralpartialeq-for-deltaadjustment .code-header}
:::
:::::::::::::::::::::::::::::::::::::

## Auto Trait Implementations[§](#synthetic-implementations){.anchor} {#synthetic-implementations .section-header}

::::::::: {#synthetic-implementations-list}
::: {#impl-Freeze-for-DeltaAdjustment .section .impl}
[§](#impl-Freeze-for-DeltaAdjustment){.anchor}

### impl [Freeze](https://doc.rust-lang.org/1.91.1/core/marker/trait.Freeze.html "trait core::marker::Freeze"){.trait} for [DeltaAdjustment](enum.DeltaAdjustment.html "enum optionstratlib::strategies::delta_neutral::DeltaAdjustment"){.enum} {#impl-freeze-for-deltaadjustment .code-header}
:::

::: {#impl-RefUnwindSafe-for-DeltaAdjustment .section .impl}
[§](#impl-RefUnwindSafe-for-DeltaAdjustment){.anchor}

### impl [RefUnwindSafe](https://doc.rust-lang.org/1.91.1/core/panic/unwind_safe/trait.RefUnwindSafe.html "trait core::panic::unwind_safe::RefUnwindSafe"){.trait} for [DeltaAdjustment](enum.DeltaAdjustment.html "enum optionstratlib::strategies::delta_neutral::DeltaAdjustment"){.enum} {#impl-refunwindsafe-for-deltaadjustment .code-header}
:::

::: {#impl-Send-for-DeltaAdjustment .section .impl}
[§](#impl-Send-for-DeltaAdjustment){.anchor}

### impl [Send](https://doc.rust-lang.org/1.91.1/core/marker/trait.Send.html "trait core::marker::Send"){.trait} for [DeltaAdjustment](enum.DeltaAdjustment.html "enum optionstratlib::strategies::delta_neutral::DeltaAdjustment"){.enum} {#impl-send-for-deltaadjustment .code-header}
:::

::: {#impl-Sync-for-DeltaAdjustment .section .impl}
[§](#impl-Sync-for-DeltaAdjustment){.anchor}

### impl [Sync](https://doc.rust-lang.org/1.91.1/core/marker/trait.Sync.html "trait core::marker::Sync"){.trait} for [DeltaAdjustment](enum.DeltaAdjustment.html "enum optionstratlib::strategies::delta_neutral::DeltaAdjustment"){.enum} {#impl-sync-for-deltaadjustment .code-header}
:::

::: {#impl-Unpin-for-DeltaAdjustment .section .impl}
[§](#impl-Unpin-for-DeltaAdjustment){.anchor}

### impl [Unpin](https://doc.rust-lang.org/1.91.1/core/marker/trait.Unpin.html "trait core::marker::Unpin"){.trait} for [DeltaAdjustment](enum.DeltaAdjustment.html "enum optionstratlib::strategies::delta_neutral::DeltaAdjustment"){.enum} {#impl-unpin-for-deltaadjustment .code-header}
:::

::: {#impl-UnwindSafe-for-DeltaAdjustment .section .impl}
[§](#impl-UnwindSafe-for-DeltaAdjustment){.anchor}

### impl [UnwindSafe](https://doc.rust-lang.org/1.91.1/core/panic/unwind_safe/trait.UnwindSafe.html "trait core::panic::unwind_safe::UnwindSafe"){.trait} for [DeltaAdjustment](enum.DeltaAdjustment.html "enum optionstratlib::strategies::delta_neutral::DeltaAdjustment"){.enum} {#impl-unwindsafe-for-deltaadjustment .code-header}
:::
:::::::::

## Blanket Implementations[§](#blanket-implementations){.anchor} {#blanket-implementations .section-header}

:::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::: {#blanket-implementations-list}
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
::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
:::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
