:::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::: width-limiter
::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../../index.html)::[model](../index.html)::[positive](index.html)
:::

# Struct [Positive]{.struct}Copy item path

[[Source](../../../src/optionstratlib/model/positive.rs.html#40){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub struct Positive(/* private fields */);
```

Expand description

:::: docblock
A wrapper type that represents a guaranteed positive decimal value.

This type encapsulates a `Decimal` value and ensures through its API
that the contained value is always positive (greater than or equal to
zero). It provides a type-safe way to handle numeric values in financial
contexts where negative values would be invalid or meaningless.

The internal value is directly accessible only within the crate through
the `pub(crate)` visibility modifier, while external access is provided
through public methods that maintain the positive value invariant.

### [§](#example){.doc-anchor}Example

::: example-wrap
``` {.rust .rust-example-rendered}
use optionstratlib::pos;
let strike_price = pos!(100.0);
```
:::
::::

## Implementations[§](#implementations){.anchor} {#implementations .section-header}

:::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::: {#implementations-list}
::: {#impl-Positive .section .impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#152-469){.src
.rightside}[§](#impl-Positive){.anchor}

### impl [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#impl-positive .code-header}
:::

:::::::::::::::::::::::::::::::::::::::::::::::::::::::::::: impl-items
::: {#associatedconstant.ZERO .section .associatedconstant}
[Source](../../../src/optionstratlib/model/positive.rs.html#154){.src
.rightside}

#### pub const [ZERO](#associatedconstant.ZERO){.constant}: [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#pub-const-zero-positive .code-header}
:::

::: docblock
A zero value represented as a `Positive` value.
:::

::: {#associatedconstant.ONE .section .associatedconstant}
[Source](../../../src/optionstratlib/model/positive.rs.html#157){.src
.rightside}

#### pub const [ONE](#associatedconstant.ONE){.constant}: [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#pub-const-one-positive .code-header}
:::

::: docblock
A value of one represented as a `Positive` value.
:::

::: {#associatedconstant.TWO .section .associatedconstant}
[Source](../../../src/optionstratlib/model/positive.rs.html#160){.src
.rightside}

#### pub const [TWO](#associatedconstant.TWO){.constant}: [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#pub-const-two-positive .code-header}
:::

::: docblock
A value of two represented as a `Positive` value.
:::

::: {#associatedconstant.INFINITY .section .associatedconstant}
[Source](../../../src/optionstratlib/model/positive.rs.html#163){.src
.rightside}

#### pub const [INFINITY](#associatedconstant.INFINITY){.constant}: [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#pub-const-infinity-positive .code-header}
:::

::: docblock
Represents the maximum positive value possible (effectively infinity).
:::

::: {#associatedconstant.TEN .section .associatedconstant}
[Source](../../../src/optionstratlib/model/positive.rs.html#166){.src
.rightside}

#### pub const [TEN](#associatedconstant.TEN){.constant}: [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#pub-const-ten-positive .code-header}
:::

::: docblock
A value of ten represented as a `Positive` value.
:::

::: {#associatedconstant.HUNDRED .section .associatedconstant}
[Source](../../../src/optionstratlib/model/positive.rs.html#169){.src
.rightside}

#### pub const [HUNDRED](#associatedconstant.HUNDRED){.constant}: [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#pub-const-hundred-positive .code-header}
:::

::: docblock
A value of one hundred represented as a `Positive` value.
:::

::: {#associatedconstant.THOUSAND .section .associatedconstant}
[Source](../../../src/optionstratlib/model/positive.rs.html#172){.src
.rightside}

#### pub const [THOUSAND](#associatedconstant.THOUSAND){.constant}: [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#pub-const-thousand-positive .code-header}
:::

::: docblock
A value of one thousand represented as a `Positive` value.
:::

::: {#associatedconstant.PI .section .associatedconstant}
[Source](../../../src/optionstratlib/model/positive.rs.html#175){.src
.rightside}

#### pub const [PI](#associatedconstant.PI){.constant}: [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#pub-const-pi-positive .code-header}
:::

::: docblock
The mathematical constant π (pi) represented as a `Positive` value.
:::

::: {#method.new .section .method}
[Source](../../../src/optionstratlib/model/positive.rs.html#188-195){.src
.rightside}

#### pub fn [new](#method.new){.fn}(value: [f64](https://doc.rust-lang.org/1.86.0/std/primitive.f64.html){.primitive}) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<Self, [String](https://doc.rust-lang.org/1.86.0/alloc/string/struct.String.html "struct alloc::string::String"){.struct}\> {#pub-fn-newvalue-f64---resultself-string .code-header}
:::

::: docblock
Creates a new `Positive` value from a 64-bit floating-point number.

##### [§](#arguments){.doc-anchor}Arguments

- `value` - A floating-point value to convert

##### [§](#returns){.doc-anchor}Returns

- `Ok(Positive)` if the value is non-negative and valid
- `Err(String)` if the value is negative or cannot be parsed as a
  Decimal
:::

::: {#method.new_decimal .section .method}
[Source](../../../src/optionstratlib/model/positive.rs.html#207-213){.src
.rightside}

#### pub fn [new_decimal](#method.new_decimal){.fn}(value: Decimal) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<Self, [String](https://doc.rust-lang.org/1.86.0/alloc/string/struct.String.html "struct alloc::string::String"){.struct}\> {#pub-fn-new_decimalvalue-decimal---resultself-string .code-header}
:::

::: docblock
Creates a new `Positive` value directly from a `Decimal`.

##### [§](#arguments-1){.doc-anchor}Arguments

- `value` - A `Decimal` value to wrap in a `Positive`

##### [§](#returns-1){.doc-anchor}Returns

- `Ok(Positive)` if the value is non-negative
- `Err(String)` if the value is negative
:::

::: {#method.value .section .method}
[Source](../../../src/optionstratlib/model/positive.rs.html#220-222){.src
.rightside}

#### pub fn [value](#method.value){.fn}(&self) -\> Decimal {#pub-fn-valueself---decimal .code-header}
:::

::: docblock
Returns the inner `Decimal` value.

##### [§](#returns-2){.doc-anchor}Returns

The wrapped `Decimal` value.
:::

::: {#method.to_dec .section .method}
[Source](../../../src/optionstratlib/model/positive.rs.html#229-231){.src
.rightside}

#### pub fn [to_dec](#method.to_dec){.fn}(&self) -\> Decimal {#pub-fn-to_decself---decimal .code-header}
:::

::: docblock
Returns the inner `Decimal` value (alias for `value()`).

##### [§](#returns-3){.doc-anchor}Returns

The wrapped `Decimal` value.
:::

::: {#method.to_dec_ref .section .method}
[Source](../../../src/optionstratlib/model/positive.rs.html#238-240){.src
.rightside}

#### pub fn [to_dec_ref](#method.to_dec_ref){.fn}(&self) -\> &Decimal {#pub-fn-to_dec_refself---decimal .code-header}
:::

::: docblock
Returns the inner `Decimal` ref.

##### [§](#returns-4){.doc-anchor}Returns

The wrapped `Decimal` ref.
:::

::: {#method.to_dec_ref_mut .section .method}
[Source](../../../src/optionstratlib/model/positive.rs.html#258-260){.src
.rightside}

#### pub fn [to_dec_ref_mut](#method.to_dec_ref_mut){.fn}(&mut self) -\> &mut Decimal {#pub-fn-to_dec_ref_mutmut-self---mut-decimal .code-header}
:::

::: docblock
Returns a mutable reference to the inner `Decimal` value.

This method provides controlled access to the underlying `Decimal` value
within the `Positive` wrapper, allowing it to be modified while
maintaining encapsulation of the inner value. It's important to note
that direct mutation should be used with caution to ensure the positive
value invariant is maintained.

##### [§](#returns-5){.doc-anchor}Returns

- `&mut Decimal` - A mutable reference to the wrapped `Decimal` value.

##### [§](#usage){.doc-anchor}Usage

This method is typically used in contexts where the wrapped value needs
to be modified in-place while preserving the wrapper's type safety
guarantees. Care should be taken to ensure any modification preserves
the positive value constraint.
:::

::: {#method.to_f64 .section .method}
[Source](../../../src/optionstratlib/model/positive.rs.html#271-273){.src
.rightside}

#### pub fn [to_f64](#method.to_f64){.fn}(&self) -\> [f64](https://doc.rust-lang.org/1.86.0/std/primitive.f64.html){.primitive} {#pub-fn-to_f64self---f64 .code-header}
:::

::: docblock
Converts the value to a 64-bit floating-point number.

##### [§](#returns-6){.doc-anchor}Returns

The value as an `f64`.

##### [§](#panics){.doc-anchor}Panics

Panics if the conversion fails.
:::

::: {#method.to_i64 .section .method}
[Source](../../../src/optionstratlib/model/positive.rs.html#284-286){.src
.rightside}

#### pub fn [to_i64](#method.to_i64){.fn}(&self) -\> [i64](https://doc.rust-lang.org/1.86.0/std/primitive.i64.html){.primitive} {#pub-fn-to_i64self---i64 .code-header}
:::

::: docblock
Converts the value to a 64-bit signed integer.

##### [§](#returns-7){.doc-anchor}Returns

The value as an `i64`.

##### [§](#panics-1){.doc-anchor}Panics

Panics if the value cannot be represented as an `i64`.
:::

::: {#method.max .section .method}
[Source](../../../src/optionstratlib/model/positive.rs.html#297-299){.src
.rightside}

#### pub fn [max](#method.max){.fn}(self, other: [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}) -\> [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#pub-fn-maxself-other-positive---positive .code-header}
:::

::: docblock
Returns the maximum of two `Positive` values.

##### [§](#arguments-2){.doc-anchor}Arguments

- `other` - Another `Positive` value to compare with

##### [§](#returns-8){.doc-anchor}Returns

The larger of the two values.
:::

::: {#method.min .section .method}
[Source](../../../src/optionstratlib/model/positive.rs.html#310-312){.src
.rightside}

#### pub fn [min](#method.min){.fn}(self, other: [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}) -\> [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#pub-fn-minself-other-positive---positive .code-header}
:::

::: docblock
Returns the minimum of two `Positive` values.

##### [§](#arguments-3){.doc-anchor}Arguments

- `other` - Another `Positive` value to compare with

##### [§](#returns-9){.doc-anchor}Returns

The smaller of the two values.
:::

::: {#method.floor .section .method}
[Source](../../../src/optionstratlib/model/positive.rs.html#319-321){.src
.rightside}

#### pub fn [floor](#method.floor){.fn}(&self) -\> [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#pub-fn-floorself---positive .code-header}
:::

::: docblock
Rounds the value down to the nearest integer.

##### [§](#returns-10){.doc-anchor}Returns

A new `Positive` value rounded down to the nearest integer.
:::

::: {#method.powi .section .method}
[Source](../../../src/optionstratlib/model/positive.rs.html#332-334){.src
.rightside}

#### pub fn [powi](#method.powi){.fn}(&self, n: [i64](https://doc.rust-lang.org/1.86.0/std/primitive.i64.html){.primitive}) -\> [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#pub-fn-powiself-n-i64---positive .code-header}
:::

::: docblock
Raises this value to an integer power.

##### [§](#arguments-4){.doc-anchor}Arguments

- `n` - The power to raise this value to

##### [§](#returns-11){.doc-anchor}Returns

A new `Positive` value representing `self` raised to the power `n`.
:::

::: {#method.round .section .method}
[Source](../../../src/optionstratlib/model/positive.rs.html#356-358){.src
.rightside}

#### pub fn [round](#method.round){.fn}(&self) -\> [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#pub-fn-roundself---positive .code-header}
:::

::: docblock
Rounds the value to the nearest integer.

##### [§](#returns-12){.doc-anchor}Returns

A new `Positive` value rounded to the nearest integer.
:::

::: {#method.sqrt .section .method}
[Source](../../../src/optionstratlib/model/positive.rs.html#369-371){.src
.rightside}

#### pub fn [sqrt](#method.sqrt){.fn}(&self) -\> [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#pub-fn-sqrtself---positive .code-header}
:::

::: docblock
Calculates the square root of the value.

##### [§](#returns-13){.doc-anchor}Returns

A new `Positive` value representing the square root.

##### [§](#panics-2){.doc-anchor}Panics

Panics if the square root calculation fails.
:::

::: {#method.ln .section .method}
[Source](../../../src/optionstratlib/model/positive.rs.html#378-380){.src
.rightside}

#### pub fn [ln](#method.ln){.fn}(&self) -\> [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#pub-fn-lnself---positive .code-header}
:::

::: docblock
Calculates the natural logarithm of the value.

##### [§](#returns-14){.doc-anchor}Returns

A new `Positive` value representing the natural logarithm.
:::

::: {#method.round_to .section .method}
[Source](../../../src/optionstratlib/model/positive.rs.html#391-393){.src
.rightside}

#### pub fn [round_to](#method.round_to){.fn}(&self, decimal_places: [u32](https://doc.rust-lang.org/1.86.0/std/primitive.u32.html){.primitive}) -\> [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#pub-fn-round_toself-decimal_places-u32---positive .code-header}
:::

::: docblock
Rounds the value to a specified number of decimal places.

##### [§](#arguments-5){.doc-anchor}Arguments

- `decimal_places` - The number of decimal places to round to

##### [§](#returns-15){.doc-anchor}Returns

A new `Positive` value rounded to the specified decimal places.
:::

::: {#method.format_fixed_places .section .method}
[Source](../../../src/optionstratlib/model/positive.rs.html#422-428){.src
.rightside}

#### pub fn [format_fixed_places](#method.format_fixed_places){.fn}(&self, decimal_places: [u32](https://doc.rust-lang.org/1.86.0/std/primitive.u32.html){.primitive}) -\> [String](https://doc.rust-lang.org/1.86.0/alloc/string/struct.String.html "struct alloc::string::String"){.struct} {#pub-fn-format_fixed_placesself-decimal_places-u32---string .code-header}
:::

:::: docblock
Formats the value with a fixed number of decimal places, filling with
zeros if needed.

Unlike `round_to` which just rounds the value, this method ensures the
string representation always has exactly the specified number of decimal
places.

##### [§](#arguments-6){.doc-anchor}Arguments

- `decimal_places` - The exact number of decimal places to display

##### [§](#returns-16){.doc-anchor}Returns

A String representation of the value with exactly the specified number
of decimal places.

##### [§](#examples){.doc-anchor}Examples

::: example-wrap
``` {.rust .rust-example-rendered}
use optionstratlib::pos;

let value = pos!(10.5);
assert_eq!(value.format_fixed_places(2), "10.50");

let value = pos!(10.0);
assert_eq!(value.format_fixed_places(3), "10.000");

let value = pos!(10.567);
assert_eq!(value.format_fixed_places(2), "10.57"); // Rounds to 2 places
```
:::
::::

::: {#method.exp .section .method}
[Source](../../../src/optionstratlib/model/positive.rs.html#435-437){.src
.rightside}

#### pub fn [exp](#method.exp){.fn}(&self) -\> [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#pub-fn-expself---positive .code-header}
:::

::: docblock
Calculates the exponential function e\^x for this value.

##### [§](#returns-17){.doc-anchor}Returns

A new `Positive` value representing e raised to the power of `self`.
:::

::: {#method.clamp .section .method}
[Source](../../../src/optionstratlib/model/positive.rs.html#451-459){.src
.rightside}

#### pub fn [clamp](#method.clamp){.fn}(&self, min: [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, max: [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}) -\> [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#pub-fn-clampself-min-positive-max-positive---positive .code-header}
:::

::: docblock
Clamps the value between a minimum and maximum.

##### [§](#arguments-7){.doc-anchor}Arguments

- `min` - The lower bound
- `max` - The upper bound

##### [§](#returns-18){.doc-anchor}Returns

- `min` if `self` is less than `min`
- `max` if `self` is greater than `max`
- `self` if `self` is between `min` and `max` (inclusive)
:::

::: {#method.is_zero .section .method}
[Source](../../../src/optionstratlib/model/positive.rs.html#466-468){.src
.rightside}

#### pub fn [is_zero](#method.is_zero){.fn}(&self) -\> [bool](https://doc.rust-lang.org/1.86.0/std/primitive.bool.html){.primitive} {#pub-fn-is_zeroself---bool .code-header}
:::

::: docblock
Checks if the value is exactly zero.

##### [§](#returns-19){.doc-anchor}Returns

`true` if the value is zero, `false` otherwise.
:::
::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::

## Trait Implementations[§](#trait-implementations){.anchor} {#trait-implementations .section-header}

::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::: {#trait-implementations-list}
::: {#impl-AbsDiffEq-for-Positive .section .impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#971-981){.src
.rightside}[§](#impl-AbsDiffEq-for-Positive){.anchor}

### impl AbsDiffEq for [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#impl-absdiffeq-for-positive .code-header}
:::

::::::::::: impl-items
::: {#associatedtype.Epsilon .section .associatedtype .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#972){.src
.rightside}[§](#associatedtype.Epsilon){.anchor}

#### type [Epsilon]{.associatedtype} = Decimal {#type-epsilon-decimal .code-header}
:::

::: docblock
Used for specifying relative comparisons.
:::

::: {#method.default_epsilon .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#974-976){.src
.rightside}[§](#method.default_epsilon){.anchor}

#### fn [default_epsilon]{.fn}() -\> Self::Epsilon {#fn-default_epsilon---selfepsilon .code-header}
:::

::: docblock
The default tolerance to use when testing values that are close
together. Read more
:::

::: {#method.abs_diff_eq .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#978-980){.src
.rightside}[§](#method.abs_diff_eq){.anchor}

#### fn [abs_diff_eq]{.fn}(&self, other: &Self, epsilon: Self::Epsilon) -\> [bool](https://doc.rust-lang.org/1.86.0/std/primitive.bool.html){.primitive} {#fn-abs_diff_eqself-other-self-epsilon-selfepsilon---bool .code-header}
:::

::: docblock
A test for equality that uses the absolute difference to compute the
approximate equality of two numbers.
:::

::: {#method.abs_diff_ne .section .method .trait-impl}
[§](#method.abs_diff_ne){.anchor}

#### fn [abs_diff_ne]{.fn}(&self, other: [&Rhs](https://doc.rust-lang.org/1.86.0/std/primitive.reference.html){.primitive}, epsilon: Self::Epsilon) -\> [bool](https://doc.rust-lang.org/1.86.0/std/primitive.bool.html){.primitive} {#fn-abs_diff_neself-other-rhs-epsilon-selfepsilon---bool .code-header}
:::

::: docblock
The inverse of \[`AbsDiffEq::abs_diff_eq`\].
:::
:::::::::::

::: {#impl-Add%3C%26Decimal%3E-for-Positive .section .impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#855-861){.src
.rightside}[§](#impl-Add%3C%26Decimal%3E-for-Positive){.anchor}

### impl [Add](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.Add.html "trait core::ops::arith::Add"){.trait}\<&Decimal\> for [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#impl-adddecimal-for-positive .code-header}
:::

::::::: impl-items
::: {#associatedtype.Output-19 .section .associatedtype .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#856){.src
.rightside}[§](#associatedtype.Output-19){.anchor}

#### type [Output](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.Add.html#associatedtype.Output){.associatedtype} = [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#type-output-positive .code-header}
:::

::: docblock
The resulting type after applying the `+` operator.
:::

::: {#method.add-6 .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#858-860){.src
.rightside}[§](#method.add-6){.anchor}

#### fn [add](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.Add.html#tymethod.add){.fn}(self, rhs: &Decimal) -\> Self::[Output](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.Add.html#associatedtype.Output "type core::ops::arith::Add::Output"){.associatedtype} {#fn-addself-rhs-decimal---selfoutput .code-header}
:::

::: docblock
Performs the `+` operation. [Read
more](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.Add.html#tymethod.add)
:::
:::::::

::: {#impl-Add%3C%26Positive%3E-for-Decimal .section .impl}
[Source](../../../src/optionstratlib/model/decimal.rs.html#177-183){.src
.rightside}[§](#impl-Add%3C%26Positive%3E-for-Decimal){.anchor}

### impl [Add](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.Add.html "trait core::ops::arith::Add"){.trait}\<&[Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\> for Decimal {#impl-addpositive-for-decimal .code-header}
:::

::::::: impl-items
::: {#associatedtype.Output-5 .section .associatedtype .trait-impl}
[Source](../../../src/optionstratlib/model/decimal.rs.html#178){.src
.rightside}[§](#associatedtype.Output-5){.anchor}

#### type [Output](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.Add.html#associatedtype.Output){.associatedtype} = Decimal {#type-output-decimal .code-header}
:::

::: docblock
The resulting type after applying the `+` operator.
:::

::: {#method.add-1 .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/decimal.rs.html#180-182){.src
.rightside}[§](#method.add-1){.anchor}

#### fn [add](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.Add.html#tymethod.add){.fn}(self, rhs: &[Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}) -\> Decimal {#fn-addself-rhs-positive---decimal .code-header}
:::

::: docblock
Performs the `+` operation. [Read
more](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.Add.html#tymethod.add)
:::
:::::::

::: {#impl-Add%3CDecimal%3E-for-Positive .section .impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#847-853){.src
.rightside}[§](#impl-Add%3CDecimal%3E-for-Positive){.anchor}

### impl [Add](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.Add.html "trait core::ops::arith::Add"){.trait}\<Decimal\> for [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#impl-adddecimal-for-positive-1 .code-header}
:::

::::::: impl-items
::: {#associatedtype.Output-18 .section .associatedtype .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#848){.src
.rightside}[§](#associatedtype.Output-18){.anchor}

#### type [Output](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.Add.html#associatedtype.Output){.associatedtype} = [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#type-output-positive-1 .code-header}
:::

::: docblock
The resulting type after applying the `+` operator.
:::

::: {#method.add-5 .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#850-852){.src
.rightside}[§](#method.add-5){.anchor}

#### fn [add](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.Add.html#tymethod.add){.fn}(self, rhs: Decimal) -\> [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#fn-addself-rhs-decimal---positive .code-header}
:::

::: docblock
Performs the `+` operation. [Read
more](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.Add.html#tymethod.add)
:::
:::::::

::: {#impl-Add%3CPositive%3E-for-Decimal .section .impl}
[Source](../../../src/optionstratlib/model/decimal.rs.html#169-175){.src
.rightside}[§](#impl-Add%3CPositive%3E-for-Decimal){.anchor}

### impl [Add](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.Add.html "trait core::ops::arith::Add"){.trait}\<[Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\> for Decimal {#impl-addpositive-for-decimal-1 .code-header}
:::

::::::: impl-items
::: {#associatedtype.Output-4 .section .associatedtype .trait-impl}
[Source](../../../src/optionstratlib/model/decimal.rs.html#170){.src
.rightside}[§](#associatedtype.Output-4){.anchor}

#### type [Output](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.Add.html#associatedtype.Output){.associatedtype} = Decimal {#type-output-decimal-1 .code-header}
:::

::: docblock
The resulting type after applying the `+` operator.
:::

::: {#method.add .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/decimal.rs.html#172-174){.src
.rightside}[§](#method.add){.anchor}

#### fn [add](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.Add.html#tymethod.add){.fn}(self, rhs: [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}) -\> Self::[Output](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.Add.html#associatedtype.Output "type core::ops::arith::Add::Output"){.associatedtype} {#fn-addself-rhs-positive---selfoutput .code-header}
:::

::: docblock
Performs the `+` operation. [Read
more](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.Add.html#tymethod.add)
:::
:::::::

::: {#impl-Add%3CPositive%3E-for-f64 .section .impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#559-565){.src
.rightside}[§](#impl-Add%3CPositive%3E-for-f64){.anchor}

### impl [Add](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.Add.html "trait core::ops::arith::Add"){.trait}\<[Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\> for [f64](https://doc.rust-lang.org/1.86.0/std/primitive.f64.html){.primitive} {#impl-addpositive-for-f64 .code-header}
:::

::::::: impl-items
::: {#associatedtype.Output-9 .section .associatedtype .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#560){.src
.rightside}[§](#associatedtype.Output-9){.anchor}

#### type [Output](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.Add.html#associatedtype.Output){.associatedtype} = [f64](https://doc.rust-lang.org/1.86.0/std/primitive.f64.html){.primitive} {#type-output-f64 .code-header}
:::

::: docblock
The resulting type after applying the `+` operator.
:::

::: {#method.add-2 .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#562-564){.src
.rightside}[§](#method.add-2){.anchor}

#### fn [add](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.Add.html#tymethod.add){.fn}(self, rhs: [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}) -\> Self::[Output](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.Add.html#associatedtype.Output "type core::ops::arith::Add::Output"){.associatedtype} {#fn-addself-rhs-positive---selfoutput-1 .code-header}
:::

::: docblock
Performs the `+` operation. [Read
more](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.Add.html#tymethod.add)
:::
:::::::

::: {#impl-Add%3Cf64%3E-for-Positive .section .impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#653-659){.src
.rightside}[§](#impl-Add%3Cf64%3E-for-Positive){.anchor}

### impl [Add](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.Add.html "trait core::ops::arith::Add"){.trait}\<[f64](https://doc.rust-lang.org/1.86.0/std/primitive.f64.html){.primitive}\> for [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#impl-addf64-for-positive .code-header}
:::

::::::: impl-items
::: {#associatedtype.Output-14 .section .associatedtype .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#654){.src
.rightside}[§](#associatedtype.Output-14){.anchor}

#### type [Output](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.Add.html#associatedtype.Output){.associatedtype} = [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#type-output-positive-2 .code-header}
:::

::: docblock
The resulting type after applying the `+` operator.
:::

::: {#method.add-3 .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#656-658){.src
.rightside}[§](#method.add-3){.anchor}

#### fn [add](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.Add.html#tymethod.add){.fn}(self, rhs: [f64](https://doc.rust-lang.org/1.86.0/std/primitive.f64.html){.primitive}) -\> Self::[Output](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.Add.html#associatedtype.Output "type core::ops::arith::Add::Output"){.associatedtype} {#fn-addself-rhs-f64---selfoutput .code-header}
:::

::: docblock
Performs the `+` operation. [Read
more](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.Add.html#tymethod.add)
:::
:::::::

::: {#impl-Add-for-Positive .section .impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#818-824){.src
.rightside}[§](#impl-Add-for-Positive){.anchor}

### impl [Add](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.Add.html "trait core::ops::arith::Add"){.trait} for [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#impl-add-for-positive .code-header}
:::

::::::: impl-items
::: {#associatedtype.Output-15 .section .associatedtype .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#819){.src
.rightside}[§](#associatedtype.Output-15){.anchor}

#### type [Output](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.Add.html#associatedtype.Output){.associatedtype} = [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#type-output-positive-3 .code-header}
:::

::: docblock
The resulting type after applying the `+` operator.
:::

::: {#method.add-4 .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#821-823){.src
.rightside}[§](#method.add-4){.anchor}

#### fn [add](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.Add.html#tymethod.add){.fn}(self, other: [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}) -\> [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#fn-addself-other-positive---positive .code-header}
:::

::: docblock
Performs the `+` operation. [Read
more](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.Add.html#tymethod.add)
:::
:::::::

::: {#impl-AddAssign%3C%26Positive%3E-for-Decimal .section .impl}
[Source](../../../src/optionstratlib/model/decimal.rs.html#191-195){.src
.rightside}[§](#impl-AddAssign%3C%26Positive%3E-for-Decimal){.anchor}

### impl [AddAssign](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.AddAssign.html "trait core::ops::arith::AddAssign"){.trait}\<&[Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\> for Decimal {#impl-addassignpositive-for-decimal .code-header}
:::

::::: impl-items
::: {#method.add_assign-1 .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/decimal.rs.html#192-194){.src
.rightside}[§](#method.add_assign-1){.anchor}

#### fn [add_assign](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.AddAssign.html#tymethod.add_assign){.fn}(&mut self, rhs: &[Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}) {#fn-add_assignmut-self-rhs-positive .code-header}
:::

::: docblock
Performs the `+=` operation. [Read
more](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.AddAssign.html#tymethod.add_assign)
:::
:::::

::: {#impl-AddAssign%3CDecimal%3E-for-Positive .section .impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#885-889){.src
.rightside}[§](#impl-AddAssign%3CDecimal%3E-for-Positive){.anchor}

### impl [AddAssign](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.AddAssign.html "trait core::ops::arith::AddAssign"){.trait}\<Decimal\> for [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#impl-addassigndecimal-for-positive .code-header}
:::

::::: impl-items
::: {#method.add_assign-3 .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#886-888){.src
.rightside}[§](#method.add_assign-3){.anchor}

#### fn [add_assign](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.AddAssign.html#tymethod.add_assign){.fn}(&mut self, rhs: Decimal) {#fn-add_assignmut-self-rhs-decimal .code-header}
:::

::: docblock
Performs the `+=` operation. [Read
more](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.AddAssign.html#tymethod.add_assign)
:::
:::::

::: {#impl-AddAssign%3CPositive%3E-for-Decimal .section .impl}
[Source](../../../src/optionstratlib/model/decimal.rs.html#185-189){.src
.rightside}[§](#impl-AddAssign%3CPositive%3E-for-Decimal){.anchor}

### impl [AddAssign](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.AddAssign.html "trait core::ops::arith::AddAssign"){.trait}\<[Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\> for Decimal {#impl-addassignpositive-for-decimal-1 .code-header}
:::

::::: impl-items
::: {#method.add_assign .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/decimal.rs.html#186-188){.src
.rightside}[§](#method.add_assign){.anchor}

#### fn [add_assign](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.AddAssign.html#tymethod.add_assign){.fn}(&mut self, rhs: [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}) {#fn-add_assignmut-self-rhs-positive-1 .code-header}
:::

::: docblock
Performs the `+=` operation. [Read
more](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.AddAssign.html#tymethod.add_assign)
:::
:::::

::: {#impl-AddAssign-for-Positive .section .impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#879-883){.src
.rightside}[§](#impl-AddAssign-for-Positive){.anchor}

### impl [AddAssign](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.AddAssign.html "trait core::ops::arith::AddAssign"){.trait} for [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#impl-addassign-for-positive .code-header}
:::

::::: impl-items
::: {#method.add_assign-2 .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#880-882){.src
.rightside}[§](#method.add_assign-2){.anchor}

#### fn [add_assign](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.AddAssign.html#tymethod.add_assign){.fn}(&mut self, other: [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}) {#fn-add_assignmut-self-other-positive .code-header}
:::

::: docblock
Performs the `+=` operation. [Read
more](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.AddAssign.html#tymethod.add_assign)
:::
:::::

::: {#impl-AtmIvProvider-for-Positive .section .impl}
[Source](../../../src/optionstratlib/volatility/traits.rs.html#102-106){.src
.rightside}[§](#impl-AtmIvProvider-for-Positive){.anchor}

### impl [AtmIvProvider](../../volatility/trait.AtmIvProvider.html "trait optionstratlib::volatility::AtmIvProvider"){.trait} for [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#impl-atmivprovider-for-positive .code-header}
:::

::::: impl-items
::: {#method.atm_iv .section .method .trait-impl}
[Source](../../../src/optionstratlib/volatility/traits.rs.html#103-105){.src
.rightside}[§](#method.atm_iv){.anchor}

#### fn [atm_iv](../../volatility/trait.AtmIvProvider.html#tymethod.atm_iv){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<&[Option](https://doc.rust-lang.org/1.86.0/core/option/enum.Option.html "enum core::option::Option"){.enum}\<[Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\>, [Box](https://doc.rust-lang.org/1.86.0/alloc/boxed/struct.Box.html "struct alloc::boxed::Box"){.struct}\<dyn [Error](https://doc.rust-lang.org/1.86.0/core/error/trait.Error.html "trait core::error::Error"){.trait}\>\> {#fn-atm_ivself---resultoptionpositive-boxdyn-error .code-header}
:::

::: docblock
Get the at-the-money implied volatility [Read
more](../../volatility/trait.AtmIvProvider.html#tymethod.atm_iv)
:::
:::::

::: {#impl-Clone-for-Positive .section .impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#39){.src
.rightside}[§](#impl-Clone-for-Positive){.anchor}

### impl [Clone](https://doc.rust-lang.org/1.86.0/core/clone/trait.Clone.html "trait core::clone::Clone"){.trait} for [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#impl-clone-for-positive .code-header}
:::

::::::: impl-items
::: {#method.clone .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#39){.src
.rightside}[§](#method.clone){.anchor}

#### fn [clone](https://doc.rust-lang.org/1.86.0/core/clone/trait.Clone.html#tymethod.clone){.fn}(&self) -\> [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#fn-cloneself---positive .code-header}
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

::: {#impl-Debug-for-Positive .section .impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#704-714){.src
.rightside}[§](#impl-Debug-for-Positive){.anchor}

### impl [Debug](https://doc.rust-lang.org/1.86.0/core/fmt/trait.Debug.html "trait core::fmt::Debug"){.trait} for [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#impl-debug-for-positive .code-header}
:::

::::: impl-items
::: {#method.fmt-1 .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#705-713){.src
.rightside}[§](#method.fmt-1){.anchor}

#### fn [fmt](https://doc.rust-lang.org/1.86.0/core/fmt/trait.Debug.html#tymethod.fmt){.fn}(&self, f: &mut [Formatter](https://doc.rust-lang.org/1.86.0/core/fmt/struct.Formatter.html "struct core::fmt::Formatter"){.struct}\<\'\_\>) -\> [Result](https://doc.rust-lang.org/1.86.0/core/fmt/type.Result.html "type core::fmt::Result"){.type} {#fn-fmtself-f-mut-formatter_---result .code-header}
:::

::: docblock
Formats the value using the given formatter. [Read
more](https://doc.rust-lang.org/1.86.0/core/fmt/trait.Debug.html#tymethod.fmt)
:::
:::::

::: {#impl-Default-for-Positive .section .impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#965-969){.src
.rightside}[§](#impl-Default-for-Positive){.anchor}

### impl [Default](https://doc.rust-lang.org/1.86.0/core/default/trait.Default.html "trait core::default::Default"){.trait} for [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#impl-default-for-positive .code-header}
:::

::::: impl-items
::: {#method.default .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#966-968){.src
.rightside}[§](#method.default){.anchor}

#### fn [default](https://doc.rust-lang.org/1.86.0/core/default/trait.Default.html#tymethod.default){.fn}() -\> Self {#fn-default---self .code-header}
:::

::: docblock
Returns the "default value" for a type. [Read
more](https://doc.rust-lang.org/1.86.0/core/default/trait.Default.html#tymethod.default)
:::
:::::

::: {#impl-Deserialize%3C'de%3E-for-Positive .section .impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#750-816){.src
.rightside}[§](#impl-Deserialize%3C'de%3E-for-Positive){.anchor}

### impl\<\'de\> [Deserialize](https://docs.rs/serde/1.0.219/serde/de/trait.Deserialize.html "trait serde::de::Deserialize"){.trait}\<\'de\> for [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#implde-deserializede-for-positive .code-header}
:::

:::::: impl-items
:::: {#method.deserialize .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#751-815){.src
.rightside}[§](#method.deserialize){.anchor}

#### fn [deserialize](https://docs.rs/serde/1.0.219/serde/de/trait.Deserialize.html#tymethod.deserialize){.fn}\<D\>(deserializer: D) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<Self, D::[Error](https://docs.rs/serde/1.0.219/serde/de/trait.Deserializer.html#associatedtype.Error "type serde::de::Deserializer::Error"){.associatedtype}\> {#fn-deserializeddeserializer-d---resultself-derror .code-header}

::: where
where D:
[Deserializer](https://docs.rs/serde/1.0.219/serde/de/trait.Deserializer.html "trait serde::de::Deserializer"){.trait}\<\'de\>,
:::
::::

::: docblock
Deserialize this value from the given Serde deserializer. [Read
more](https://docs.rs/serde/1.0.219/serde/de/trait.Deserialize.html#tymethod.deserialize)
:::
::::::

::: {#impl-Display-for-Positive .section .impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#685-702){.src
.rightside}[§](#impl-Display-for-Positive){.anchor}

### impl [Display](https://doc.rust-lang.org/1.86.0/core/fmt/trait.Display.html "trait core::fmt::Display"){.trait} for [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#impl-display-for-positive .code-header}
:::

::::: impl-items
::: {#method.fmt .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#686-701){.src
.rightside}[§](#method.fmt){.anchor}

#### fn [fmt](https://doc.rust-lang.org/1.86.0/core/fmt/trait.Display.html#tymethod.fmt){.fn}(&self, f: &mut [Formatter](https://doc.rust-lang.org/1.86.0/core/fmt/struct.Formatter.html "struct core::fmt::Formatter"){.struct}\<\'\_\>) -\> [Result](https://doc.rust-lang.org/1.86.0/core/fmt/type.Result.html "type core::fmt::Result"){.type} {#fn-fmtself-f-mut-formatter_---result-1 .code-header}
:::

::: docblock
Formats the value using the given formatter. [Read
more](https://doc.rust-lang.org/1.86.0/core/fmt/trait.Display.html#tymethod.fmt)
:::
:::::

::: {#impl-Div%3C%26Decimal%3E-for-Positive .section .impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#905-911){.src
.rightside}[§](#impl-Div%3C%26Decimal%3E-for-Positive){.anchor}

### impl [Div](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.Div.html "trait core::ops::arith::Div"){.trait}\<&Decimal\> for [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#impl-divdecimal-for-positive .code-header}
:::

::::::: impl-items
::: {#associatedtype.Output-23 .section .associatedtype .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#906){.src
.rightside}[§](#associatedtype.Output-23){.anchor}

#### type [Output](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.Div.html#associatedtype.Output){.associatedtype} = [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#type-output-positive-4 .code-header}
:::

::: docblock
The resulting type after applying the `/` operator.
:::

::: {#method.div-6 .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#908-910){.src
.rightside}[§](#method.div-6){.anchor}

#### fn [div](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.Div.html#tymethod.div){.fn}(self, rhs: &Decimal) -\> Self::[Output](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.Div.html#associatedtype.Output "type core::ops::arith::Div::Output"){.associatedtype} {#fn-divself-rhs-decimal---selfoutput .code-header}
:::

::: docblock
Performs the `/` operation. [Read
more](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.Div.html#tymethod.div)
:::
:::::::

::: {#impl-Div%3CDecimal%3E-for-Positive .section .impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#897-903){.src
.rightside}[§](#impl-Div%3CDecimal%3E-for-Positive){.anchor}

### impl [Div](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.Div.html "trait core::ops::arith::Div"){.trait}\<Decimal\> for [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#impl-divdecimal-for-positive-1 .code-header}
:::

::::::: impl-items
::: {#associatedtype.Output-22 .section .associatedtype .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#898){.src
.rightside}[§](#associatedtype.Output-22){.anchor}

#### type [Output](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.Div.html#associatedtype.Output){.associatedtype} = [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#type-output-positive-5 .code-header}
:::

::: docblock
The resulting type after applying the `/` operator.
:::

::: {#method.div-5 .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#900-902){.src
.rightside}[§](#method.div-5){.anchor}

#### fn [div](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.Div.html#tymethod.div){.fn}(self, rhs: Decimal) -\> [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#fn-divself-rhs-decimal---positive .code-header}
:::

::: docblock
Performs the `/` operation. [Read
more](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.Div.html#tymethod.div)
:::
:::::::

::: {#impl-Div%3CPositive%3E-for-Decimal .section .impl}
[Source](../../../src/optionstratlib/model/decimal.rs.html#145-151){.src
.rightside}[§](#impl-Div%3CPositive%3E-for-Decimal){.anchor}

### impl [Div](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.Div.html "trait core::ops::arith::Div"){.trait}\<[Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\> for Decimal {#impl-divpositive-for-decimal .code-header}
:::

::::::: impl-items
::: {#associatedtype.Output-1 .section .associatedtype .trait-impl}
[Source](../../../src/optionstratlib/model/decimal.rs.html#146){.src
.rightside}[§](#associatedtype.Output-1){.anchor}

#### type [Output](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.Div.html#associatedtype.Output){.associatedtype} = Decimal {#type-output-decimal-2 .code-header}
:::

::: docblock
The resulting type after applying the `/` operator.
:::

::: {#method.div .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/decimal.rs.html#148-150){.src
.rightside}[§](#method.div){.anchor}

#### fn [div](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.Div.html#tymethod.div){.fn}(self, rhs: [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}) -\> Decimal {#fn-divself-rhs-positive---decimal .code-header}
:::

::: docblock
Performs the `/` operation. [Read
more](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.Div.html#tymethod.div)
:::
:::::::

::: {#impl-Div%3CPositive%3E-for-f64 .section .impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#543-549){.src
.rightside}[§](#impl-Div%3CPositive%3E-for-f64){.anchor}

### impl [Div](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.Div.html "trait core::ops::arith::Div"){.trait}\<[Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\> for [f64](https://doc.rust-lang.org/1.86.0/std/primitive.f64.html){.primitive} {#impl-divpositive-for-f64 .code-header}
:::

::::::: impl-items
::: {#associatedtype.Output-7 .section .associatedtype .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#544){.src
.rightside}[§](#associatedtype.Output-7){.anchor}

#### type [Output](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.Div.html#associatedtype.Output){.associatedtype} = [f64](https://doc.rust-lang.org/1.86.0/std/primitive.f64.html){.primitive} {#type-output-f64-1 .code-header}
:::

::: docblock
The resulting type after applying the `/` operator.
:::

::: {#method.div-1 .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#546-548){.src
.rightside}[§](#method.div-1){.anchor}

#### fn [div](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.Div.html#tymethod.div){.fn}(self, rhs: [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}) -\> Self::[Output](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.Div.html#associatedtype.Output "type core::ops::arith::Div::Output"){.associatedtype} {#fn-divself-rhs-positive---selfoutput .code-header}
:::

::: docblock
Performs the `/` operation. [Read
more](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.Div.html#tymethod.div)
:::
:::::::

::: {#impl-Div%3Cf64%3E-for-%26Positive .section .impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#637-643){.src
.rightside}[§](#impl-Div%3Cf64%3E-for-%26Positive){.anchor}

### impl [Div](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.Div.html "trait core::ops::arith::Div"){.trait}\<[f64](https://doc.rust-lang.org/1.86.0/std/primitive.f64.html){.primitive}\> for &[Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#impl-divf64-for-positive .code-header}
:::

::::::: impl-items
::: {#associatedtype.Output-12 .section .associatedtype .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#638){.src
.rightside}[§](#associatedtype.Output-12){.anchor}

#### type [Output](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.Div.html#associatedtype.Output){.associatedtype} = [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#type-output-positive-6 .code-header}
:::

::: docblock
The resulting type after applying the `/` operator.
:::

::: {#method.div-3 .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#640-642){.src
.rightside}[§](#method.div-3){.anchor}

#### fn [div](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.Div.html#tymethod.div){.fn}(self, rhs: [f64](https://doc.rust-lang.org/1.86.0/std/primitive.f64.html){.primitive}) -\> [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#fn-divself-rhs-f64---positive .code-header}
:::

::: docblock
Performs the `/` operation. [Read
more](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.Div.html#tymethod.div)
:::
:::::::

::: {#impl-Div%3Cf64%3E-for-Positive .section .impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#629-635){.src
.rightside}[§](#impl-Div%3Cf64%3E-for-Positive){.anchor}

### impl [Div](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.Div.html "trait core::ops::arith::Div"){.trait}\<[f64](https://doc.rust-lang.org/1.86.0/std/primitive.f64.html){.primitive}\> for [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#impl-divf64-for-positive-1 .code-header}
:::

::::::: impl-items
::: {#associatedtype.Output-11 .section .associatedtype .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#630){.src
.rightside}[§](#associatedtype.Output-11){.anchor}

#### type [Output](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.Div.html#associatedtype.Output){.associatedtype} = [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#type-output-positive-7 .code-header}
:::

::: docblock
The resulting type after applying the `/` operator.
:::

::: {#method.div-2 .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#632-634){.src
.rightside}[§](#method.div-2){.anchor}

#### fn [div](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.Div.html#tymethod.div){.fn}(self, rhs: [f64](https://doc.rust-lang.org/1.86.0/std/primitive.f64.html){.primitive}) -\> [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#fn-divself-rhs-f64---positive-1 .code-header}
:::

::: docblock
Performs the `/` operation. [Read
more](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.Div.html#tymethod.div)
:::
:::::::

::: {#impl-Div-for-Positive .section .impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#839-845){.src
.rightside}[§](#impl-Div-for-Positive){.anchor}

### impl [Div](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.Div.html "trait core::ops::arith::Div"){.trait} for [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#impl-div-for-positive .code-header}
:::

::::::: impl-items
::: {#associatedtype.Output-17 .section .associatedtype .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#840){.src
.rightside}[§](#associatedtype.Output-17){.anchor}

#### type [Output](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.Div.html#associatedtype.Output){.associatedtype} = [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#type-output-positive-8 .code-header}
:::

::: docblock
The resulting type after applying the `/` operator.
:::

::: {#method.div-4 .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#842-844){.src
.rightside}[§](#method.div-4){.anchor}

#### fn [div](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.Div.html#tymethod.div){.fn}(self, other: [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}) -\> Self::[Output](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.Div.html#associatedtype.Output "type core::ops::arith::Div::Output"){.associatedtype} {#fn-divself-other-positive---selfoutput .code-header}
:::

::: docblock
Performs the `/` operation. [Read
more](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.Div.html#tymethod.div)
:::
:::::::

::: {#impl-From%3C%26Decimal%3E-for-Positive .section .impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#597-601){.src
.rightside}[§](#impl-From%3C%26Decimal%3E-for-Positive){.anchor}

### impl [From](https://doc.rust-lang.org/1.86.0/core/convert/trait.From.html "trait core::convert::From"){.trait}\<&Decimal\> for [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#impl-fromdecimal-for-positive .code-header}
:::

::::: impl-items
::: {#method.from-9 .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#598-600){.src
.rightside}[§](#method.from-9){.anchor}

#### fn [from](https://doc.rust-lang.org/1.86.0/core/convert/trait.From.html#tymethod.from){.fn}(value: &Decimal) -\> Self {#fn-fromvalue-decimal---self .code-header}
:::

::: docblock
Converts to this type from the input type.
:::
:::::

::: {#impl-From%3C%26OptionChain%3E-for-Positive .section .impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#609-613){.src
.rightside}[§](#impl-From%3C%26OptionChain%3E-for-Positive){.anchor}

### impl [From](https://doc.rust-lang.org/1.86.0/core/convert/trait.From.html "trait core::convert::From"){.trait}\<&[OptionChain](../../chains/chain/struct.OptionChain.html "struct optionstratlib::chains::chain::OptionChain"){.struct}\> for [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#impl-fromoptionchain-for-positive .code-header}
:::

::::: impl-items
::: {#method.from-11 .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#610-612){.src
.rightside}[§](#method.from-11){.anchor}

#### fn [from](https://doc.rust-lang.org/1.86.0/core/convert/trait.From.html#tymethod.from){.fn}(value: &[OptionChain](../../chains/chain/struct.OptionChain.html "struct optionstratlib::chains::chain::OptionChain"){.struct}) -\> Self {#fn-fromvalue-optionchain---self .code-header}
:::

::: docblock
Converts to this type from the input type.
:::
:::::

::: {#impl-From%3C%26Positive%3E-for-Decimal .section .impl}
[Source](../../../src/optionstratlib/model/decimal.rs.html#112-116){.src
.rightside}[§](#impl-From%3C%26Positive%3E-for-Decimal){.anchor}

### impl [From](https://doc.rust-lang.org/1.86.0/core/convert/trait.From.html "trait core::convert::From"){.trait}\<&[Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\> for Decimal {#impl-frompositive-for-decimal .code-header}
:::

::::: impl-items
::: {#method.from-1 .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/decimal.rs.html#113-115){.src
.rightside}[§](#method.from-1){.anchor}

#### fn [from](https://doc.rust-lang.org/1.86.0/core/convert/trait.From.html#tymethod.from){.fn}(pos: &[Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}) -\> Self {#fn-frompos-positive---self .code-header}
:::

::: docblock
Converts to this type from the input type.
:::
:::::

::: {#impl-From%3C%26Positive%3E-for-Positive .section .impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#603-607){.src
.rightside}[§](#impl-From%3C%26Positive%3E-for-Positive){.anchor}

### impl [From](https://doc.rust-lang.org/1.86.0/core/convert/trait.From.html "trait core::convert::From"){.trait}\<&[Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\> for [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#impl-frompositive-for-positive .code-header}
:::

::::: impl-items
::: {#method.from-10 .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#604-606){.src
.rightside}[§](#method.from-10){.anchor}

#### fn [from](https://doc.rust-lang.org/1.86.0/core/convert/trait.From.html#tymethod.from){.fn}(value: &[Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}) -\> Self {#fn-fromvalue-positive---self .code-header}
:::

::: docblock
Converts to this type from the input type.
:::
:::::

::: {#impl-From%3C%26Positive%3E-for-f64 .section .impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#493-497){.src
.rightside}[§](#impl-From%3C%26Positive%3E-for-f64){.anchor}

### impl [From](https://doc.rust-lang.org/1.86.0/core/convert/trait.From.html "trait core::convert::From"){.trait}\<&[Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\> for [f64](https://doc.rust-lang.org/1.86.0/std/primitive.f64.html){.primitive} {#impl-frompositive-for-f64 .code-header}
:::

::::: impl-items
::: {#method.from-3 .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#494-496){.src
.rightside}[§](#method.from-3){.anchor}

#### fn [from](https://doc.rust-lang.org/1.86.0/core/convert/trait.From.html#tymethod.from){.fn}(value: &[Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}) -\> Self {#fn-fromvalue-positive---self-1 .code-header}
:::

::: docblock
Converts to this type from the input type.
:::
:::::

::: {#impl-From%3CDecimal%3E-for-Positive .section .impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#591-595){.src
.rightside}[§](#impl-From%3CDecimal%3E-for-Positive){.anchor}

### impl [From](https://doc.rust-lang.org/1.86.0/core/convert/trait.From.html "trait core::convert::From"){.trait}\<Decimal\> for [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#impl-fromdecimal-for-positive-1 .code-header}
:::

::::: impl-items
::: {#method.from-8 .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#592-594){.src
.rightside}[§](#method.from-8){.anchor}

#### fn [from](https://doc.rust-lang.org/1.86.0/core/convert/trait.From.html#tymethod.from){.fn}(value: Decimal) -\> Self {#fn-fromvalue-decimal---self-1 .code-header}
:::

::: docblock
Converts to this type from the input type.
:::
:::::

::: {#impl-From%3COptionChain%3E-for-Positive .section .impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#615-619){.src
.rightside}[§](#impl-From%3COptionChain%3E-for-Positive){.anchor}

### impl [From](https://doc.rust-lang.org/1.86.0/core/convert/trait.From.html "trait core::convert::From"){.trait}\<[OptionChain](../../chains/chain/struct.OptionChain.html "struct optionstratlib::chains::chain::OptionChain"){.struct}\> for [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#impl-fromoptionchain-for-positive-1 .code-header}
:::

::::: impl-items
::: {#method.from-12 .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#616-618){.src
.rightside}[§](#method.from-12){.anchor}

#### fn [from](https://doc.rust-lang.org/1.86.0/core/convert/trait.From.html#tymethod.from){.fn}(value: [OptionChain](../../chains/chain/struct.OptionChain.html "struct optionstratlib::chains::chain::OptionChain"){.struct}) -\> Self {#fn-fromvalue-optionchain---self-1 .code-header}
:::

::: docblock
Converts to this type from the input type.
:::
:::::

::: {#impl-From%3CPositive%3E-for-Decimal .section .impl}
[Source](../../../src/optionstratlib/model/decimal.rs.html#106-110){.src
.rightside}[§](#impl-From%3CPositive%3E-for-Decimal){.anchor}

### impl [From](https://doc.rust-lang.org/1.86.0/core/convert/trait.From.html "trait core::convert::From"){.trait}\<[Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\> for Decimal {#impl-frompositive-for-decimal-1 .code-header}
:::

::::: impl-items
::: {#method.from .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/decimal.rs.html#107-109){.src
.rightside}[§](#method.from){.anchor}

#### fn [from](https://doc.rust-lang.org/1.86.0/core/convert/trait.From.html#tymethod.from){.fn}(pos: [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}) -\> Self {#fn-frompos-positive---self-1 .code-header}
:::

::: docblock
Converts to this type from the input type.
:::
:::::

::: {#impl-From%3CPositive%3E-for-f64 .section .impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#499-503){.src
.rightside}[§](#impl-From%3CPositive%3E-for-f64){.anchor}

### impl [From](https://doc.rust-lang.org/1.86.0/core/convert/trait.From.html "trait core::convert::From"){.trait}\<[Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\> for [f64](https://doc.rust-lang.org/1.86.0/std/primitive.f64.html){.primitive} {#impl-frompositive-for-f64-1 .code-header}
:::

::::: impl-items
::: {#method.from-4 .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#500-502){.src
.rightside}[§](#method.from-4){.anchor}

#### fn [from](https://doc.rust-lang.org/1.86.0/core/convert/trait.From.html#tymethod.from){.fn}(value: [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}) -\> Self {#fn-fromvalue-positive---self-2 .code-header}
:::

::: docblock
Converts to this type from the input type.
:::
:::::

::: {#impl-From%3CPositive%3E-for-u64 .section .impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#487-491){.src
.rightside}[§](#impl-From%3CPositive%3E-for-u64){.anchor}

### impl [From](https://doc.rust-lang.org/1.86.0/core/convert/trait.From.html "trait core::convert::From"){.trait}\<[Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\> for [u64](https://doc.rust-lang.org/1.86.0/std/primitive.u64.html){.primitive} {#impl-frompositive-for-u64 .code-header}
:::

::::: impl-items
::: {#method.from-2 .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#488-490){.src
.rightside}[§](#method.from-2){.anchor}

#### fn [from](https://doc.rust-lang.org/1.86.0/core/convert/trait.From.html#tymethod.from){.fn}(pos_u64: [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}) -\> Self {#fn-frompos_u64-positive---self .code-header}
:::

::: docblock
Converts to this type from the input type.
:::
:::::

::: {#impl-From%3CPositive%3E-for-usize .section .impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#505-509){.src
.rightside}[§](#impl-From%3CPositive%3E-for-usize){.anchor}

### impl [From](https://doc.rust-lang.org/1.86.0/core/convert/trait.From.html "trait core::convert::From"){.trait}\<[Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\> for [usize](https://doc.rust-lang.org/1.86.0/std/primitive.usize.html){.primitive} {#impl-frompositive-for-usize .code-header}
:::

::::: impl-items
::: {#method.from-5 .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#506-508){.src
.rightside}[§](#method.from-5){.anchor}

#### fn [from](https://doc.rust-lang.org/1.86.0/core/convert/trait.From.html#tymethod.from){.fn}(value: [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}) -\> Self {#fn-fromvalue-positive---self-3 .code-header}
:::

::: docblock
Converts to this type from the input type.
:::
:::::

::: {#impl-From%3Cf64%3E-for-Positive .section .impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#579-583){.src
.rightside}[§](#impl-From%3Cf64%3E-for-Positive){.anchor}

### impl [From](https://doc.rust-lang.org/1.86.0/core/convert/trait.From.html "trait core::convert::From"){.trait}\<[f64](https://doc.rust-lang.org/1.86.0/std/primitive.f64.html){.primitive}\> for [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#impl-fromf64-for-positive .code-header}
:::

::::: impl-items
::: {#method.from-6 .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#580-582){.src
.rightside}[§](#method.from-6){.anchor}

#### fn [from](https://doc.rust-lang.org/1.86.0/core/convert/trait.From.html#tymethod.from){.fn}(value: [f64](https://doc.rust-lang.org/1.86.0/std/primitive.f64.html){.primitive}) -\> Self {#fn-fromvalue-f64---self .code-header}
:::

::: docblock
Converts to this type from the input type.
:::
:::::

::: {#impl-From%3Cusize%3E-for-Positive .section .impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#585-589){.src
.rightside}[§](#impl-From%3Cusize%3E-for-Positive){.anchor}

### impl [From](https://doc.rust-lang.org/1.86.0/core/convert/trait.From.html "trait core::convert::From"){.trait}\<[usize](https://doc.rust-lang.org/1.86.0/std/primitive.usize.html){.primitive}\> for [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#impl-fromusize-for-positive .code-header}
:::

::::: impl-items
::: {#method.from-7 .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#586-588){.src
.rightside}[§](#method.from-7){.anchor}

#### fn [from](https://doc.rust-lang.org/1.86.0/core/convert/trait.From.html#tymethod.from){.fn}(value: [usize](https://doc.rust-lang.org/1.86.0/std/primitive.usize.html){.primitive}) -\> Self {#fn-fromvalue-usize---self .code-header}
:::

::: docblock
Converts to this type from the input type.
:::
:::::

::: {#impl-FromStr-for-Positive .section .impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#567-577){.src
.rightside}[§](#impl-FromStr-for-Positive){.anchor}

### impl [FromStr](https://doc.rust-lang.org/1.86.0/core/str/traits/trait.FromStr.html "trait core::str::traits::FromStr"){.trait} for [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#impl-fromstr-for-positive .code-header}
:::

::::::: impl-items
::: {#associatedtype.Err .section .associatedtype .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#568){.src
.rightside}[§](#associatedtype.Err){.anchor}

#### type [Err](https://doc.rust-lang.org/1.86.0/core/str/traits/trait.FromStr.html#associatedtype.Err){.associatedtype} = [String](https://doc.rust-lang.org/1.86.0/alloc/string/struct.String.html "struct alloc::string::String"){.struct} {#type-err-string .code-header}
:::

::: docblock
The associated error which can be returned from parsing.
:::

::: {#method.from_str .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#570-576){.src
.rightside}[§](#method.from_str){.anchor}

#### fn [from_str](https://doc.rust-lang.org/1.86.0/core/str/traits/trait.FromStr.html#tymethod.from_str){.fn}(s: &[str](https://doc.rust-lang.org/1.86.0/std/primitive.str.html){.primitive}) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<Self, Self::[Err](https://doc.rust-lang.org/1.86.0/core/str/traits/trait.FromStr.html#associatedtype.Err "type core::str::traits::FromStr::Err"){.associatedtype}\> {#fn-from_strs-str---resultself-selferr .code-header}
:::

::: docblock
Parses a string `s` to return a value of this type. [Read
more](https://doc.rust-lang.org/1.86.0/core/str/traits/trait.FromStr.html#tymethod.from_str)
:::
:::::::

::: {#impl-Mul%3CDecimal%3E-for-Positive .section .impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#957-963){.src
.rightside}[§](#impl-Mul%3CDecimal%3E-for-Positive){.anchor}

### impl [Mul](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.Mul.html "trait core::ops::arith::Mul"){.trait}\<Decimal\> for [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#impl-muldecimal-for-positive .code-header}
:::

::::::: impl-items
::: {#associatedtype.Output-26 .section .associatedtype .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#958){.src
.rightside}[§](#associatedtype.Output-26){.anchor}

#### type [Output](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.Mul.html#associatedtype.Output){.associatedtype} = [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#type-output-positive-9 .code-header}
:::

::: docblock
The resulting type after applying the `*` operator.
:::

::: {#method.mul-4 .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#960-962){.src
.rightside}[§](#method.mul-4){.anchor}

#### fn [mul](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.Mul.html#tymethod.mul){.fn}(self, rhs: Decimal) -\> [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#fn-mulself-rhs-decimal---positive .code-header}
:::

::: docblock
Performs the `*` operation. [Read
more](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.Mul.html#tymethod.mul)
:::
:::::::

::: {#impl-Mul%3CPositive%3E-for-Decimal .section .impl}
[Source](../../../src/optionstratlib/model/decimal.rs.html#118-124){.src
.rightside}[§](#impl-Mul%3CPositive%3E-for-Decimal){.anchor}

### impl [Mul](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.Mul.html "trait core::ops::arith::Mul"){.trait}\<[Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\> for Decimal {#impl-mulpositive-for-decimal .code-header}
:::

::::::: impl-items
::: {#associatedtype.Output .section .associatedtype .trait-impl}
[Source](../../../src/optionstratlib/model/decimal.rs.html#119){.src
.rightside}[§](#associatedtype.Output){.anchor}

#### type [Output](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.Mul.html#associatedtype.Output){.associatedtype} = Decimal {#type-output-decimal-3 .code-header}
:::

::: docblock
The resulting type after applying the `*` operator.
:::

::: {#method.mul .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/decimal.rs.html#121-123){.src
.rightside}[§](#method.mul){.anchor}

#### fn [mul](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.Mul.html#tymethod.mul){.fn}(self, rhs: [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}) -\> Decimal {#fn-mulself-rhs-positive---decimal .code-header}
:::

::: docblock
Performs the `*` operation. [Read
more](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.Mul.html#tymethod.mul)
:::
:::::::

::: {#impl-Mul%3CPositive%3E-for-f64 .section .impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#535-541){.src
.rightside}[§](#impl-Mul%3CPositive%3E-for-f64){.anchor}

### impl [Mul](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.Mul.html "trait core::ops::arith::Mul"){.trait}\<[Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\> for [f64](https://doc.rust-lang.org/1.86.0/std/primitive.f64.html){.primitive} {#impl-mulpositive-for-f64 .code-header}
:::

::::::: impl-items
::: {#associatedtype.Output-6 .section .associatedtype .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#536){.src
.rightside}[§](#associatedtype.Output-6){.anchor}

#### type [Output](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.Mul.html#associatedtype.Output){.associatedtype} = [f64](https://doc.rust-lang.org/1.86.0/std/primitive.f64.html){.primitive} {#type-output-f64-2 .code-header}
:::

::: docblock
The resulting type after applying the `*` operator.
:::

::: {#method.mul-1 .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#538-540){.src
.rightside}[§](#method.mul-1){.anchor}

#### fn [mul](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.Mul.html#tymethod.mul){.fn}(self, rhs: [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}) -\> Self::[Output](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.Mul.html#associatedtype.Output "type core::ops::arith::Mul::Output"){.associatedtype} {#fn-mulself-rhs-positive---selfoutput .code-header}
:::

::: docblock
Performs the `*` operation. [Read
more](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.Mul.html#tymethod.mul)
:::
:::::::

::: {#impl-Mul%3Cf64%3E-for-Positive .section .impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#621-627){.src
.rightside}[§](#impl-Mul%3Cf64%3E-for-Positive){.anchor}

### impl [Mul](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.Mul.html "trait core::ops::arith::Mul"){.trait}\<[f64](https://doc.rust-lang.org/1.86.0/std/primitive.f64.html){.primitive}\> for [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#impl-mulf64-for-positive .code-header}
:::

::::::: impl-items
::: {#associatedtype.Output-10 .section .associatedtype .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#622){.src
.rightside}[§](#associatedtype.Output-10){.anchor}

#### type [Output](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.Mul.html#associatedtype.Output){.associatedtype} = [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#type-output-positive-10 .code-header}
:::

::: docblock
The resulting type after applying the `*` operator.
:::

::: {#method.mul-2 .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#624-626){.src
.rightside}[§](#method.mul-2){.anchor}

#### fn [mul](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.Mul.html#tymethod.mul){.fn}(self, rhs: [f64](https://doc.rust-lang.org/1.86.0/std/primitive.f64.html){.primitive}) -\> [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#fn-mulself-rhs-f64---positive .code-header}
:::

::: docblock
Performs the `*` operation. [Read
more](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.Mul.html#tymethod.mul)
:::
:::::::

::: {#impl-Mul-for-Positive .section .impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#949-955){.src
.rightside}[§](#impl-Mul-for-Positive){.anchor}

### impl [Mul](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.Mul.html "trait core::ops::arith::Mul"){.trait} for [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#impl-mul-for-positive .code-header}
:::

::::::: impl-items
::: {#associatedtype.Output-25 .section .associatedtype .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#950){.src
.rightside}[§](#associatedtype.Output-25){.anchor}

#### type [Output](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.Mul.html#associatedtype.Output){.associatedtype} = [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#type-output-positive-11 .code-header}
:::

::: docblock
The resulting type after applying the `*` operator.
:::

::: {#method.mul-3 .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#952-954){.src
.rightside}[§](#method.mul-3){.anchor}

#### fn [mul](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.Mul.html#tymethod.mul){.fn}(self, other: [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}) -\> [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#fn-mulself-other-positive---positive .code-header}
:::

::: docblock
Performs the `*` operation. [Read
more](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.Mul.html#tymethod.mul)
:::
:::::::

::: {#impl-MulAssign%3C%26Positive%3E-for-Decimal .section .impl}
[Source](../../../src/optionstratlib/model/decimal.rs.html#203-207){.src
.rightside}[§](#impl-MulAssign%3C%26Positive%3E-for-Decimal){.anchor}

### impl [MulAssign](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.MulAssign.html "trait core::ops::arith::MulAssign"){.trait}\<&[Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\> for Decimal {#impl-mulassignpositive-for-decimal .code-header}
:::

::::: impl-items
::: {#method.mul_assign-1 .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/decimal.rs.html#204-206){.src
.rightside}[§](#method.mul_assign-1){.anchor}

#### fn [mul_assign](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.MulAssign.html#tymethod.mul_assign){.fn}(&mut self, rhs: &[Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}) {#fn-mul_assignmut-self-rhs-positive .code-header}
:::

::: docblock
Performs the `*=` operation. [Read
more](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.MulAssign.html#tymethod.mul_assign)
:::
:::::

::: {#impl-MulAssign%3CDecimal%3E-for-Positive .section .impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#891-895){.src
.rightside}[§](#impl-MulAssign%3CDecimal%3E-for-Positive){.anchor}

### impl [MulAssign](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.MulAssign.html "trait core::ops::arith::MulAssign"){.trait}\<Decimal\> for [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#impl-mulassigndecimal-for-positive .code-header}
:::

::::: impl-items
::: {#method.mul_assign-2 .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#892-894){.src
.rightside}[§](#method.mul_assign-2){.anchor}

#### fn [mul_assign](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.MulAssign.html#tymethod.mul_assign){.fn}(&mut self, rhs: Decimal) {#fn-mul_assignmut-self-rhs-decimal .code-header}
:::

::: docblock
Performs the `*=` operation. [Read
more](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.MulAssign.html#tymethod.mul_assign)
:::
:::::

::: {#impl-MulAssign%3CPositive%3E-for-Decimal .section .impl}
[Source](../../../src/optionstratlib/model/decimal.rs.html#197-201){.src
.rightside}[§](#impl-MulAssign%3CPositive%3E-for-Decimal){.anchor}

### impl [MulAssign](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.MulAssign.html "trait core::ops::arith::MulAssign"){.trait}\<[Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\> for Decimal {#impl-mulassignpositive-for-decimal-1 .code-header}
:::

::::: impl-items
::: {#method.mul_assign .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/decimal.rs.html#198-200){.src
.rightside}[§](#method.mul_assign){.anchor}

#### fn [mul_assign](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.MulAssign.html#tymethod.mul_assign){.fn}(&mut self, rhs: [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}) {#fn-mul_assignmut-self-rhs-positive-1 .code-header}
:::

::: docblock
Performs the `*=` operation. [Read
more](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.MulAssign.html#tymethod.mul_assign)
:::
:::::

::: {#impl-Neg-for-Positive .section .impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#941-947){.src
.rightside}[§](#impl-Neg-for-Positive){.anchor}

### impl [Neg](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.Neg.html "trait core::ops::arith::Neg"){.trait} for [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#impl-neg-for-positive .code-header}
:::

::::::: impl-items
::: {#associatedtype.Output-24 .section .associatedtype .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#942){.src
.rightside}[§](#associatedtype.Output-24){.anchor}

#### type [Output](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.Neg.html#associatedtype.Output){.associatedtype} = [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#type-output-positive-12 .code-header}
:::

::: docblock
The resulting type after applying the `-` operator.
:::

::: {#method.neg .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#944-946){.src
.rightside}[§](#method.neg){.anchor}

#### fn [neg](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.Neg.html#tymethod.neg){.fn}(self) -\> Self::[Output](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.Neg.html#associatedtype.Output "type core::ops::arith::Neg::Output"){.associatedtype} {#fn-negself---selfoutput .code-header}
:::

::: docblock
Performs the unary `-` operation. [Read
more](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.Neg.html#tymethod.neg)
:::
:::::::

::: {#impl-Ord-for-Positive .section .impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#935-939){.src
.rightside}[§](#impl-Ord-for-Positive){.anchor}

### impl [Ord](https://doc.rust-lang.org/1.86.0/core/cmp/trait.Ord.html "trait core::cmp::Ord"){.trait} for [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#impl-ord-for-positive .code-header}
:::

:::::::::::::: impl-items
::: {#method.cmp .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#936-938){.src
.rightside}[§](#method.cmp){.anchor}

#### fn [cmp](https://doc.rust-lang.org/1.86.0/core/cmp/trait.Ord.html#tymethod.cmp){.fn}(&self, other: &Self) -\> [Ordering](https://doc.rust-lang.org/1.86.0/core/cmp/enum.Ordering.html "enum core::cmp::Ordering"){.enum} {#fn-cmpself-other-self---ordering .code-header}
:::

::: docblock
This method returns an
[`Ordering`](https://doc.rust-lang.org/1.86.0/core/cmp/enum.Ordering.html "enum core::cmp::Ordering")
between `self` and `other`. [Read
more](https://doc.rust-lang.org/1.86.0/core/cmp/trait.Ord.html#tymethod.cmp)
:::

:::: {#method.max-1 .section .method .trait-impl}
[[1.21.0]{.since title="Stable since Rust version 1.21.0"} ·
[Source](https://doc.rust-lang.org/1.86.0/src/core/cmp.rs.html#998-1000){.src}]{.rightside}[§](#method.max-1){.anchor}

#### fn [max](https://doc.rust-lang.org/1.86.0/core/cmp/trait.Ord.html#method.max){.fn}(self, other: Self) -\> Self {#fn-maxself-other-self---self .code-header}

::: where
where Self:
[Sized](https://doc.rust-lang.org/1.86.0/core/marker/trait.Sized.html "trait core::marker::Sized"){.trait},
:::
::::

::: docblock
Compares and returns the maximum of two values. [Read
more](https://doc.rust-lang.org/1.86.0/core/cmp/trait.Ord.html#method.max)
:::

:::: {#method.min-1 .section .method .trait-impl}
[[1.21.0]{.since title="Stable since Rust version 1.21.0"} ·
[Source](https://doc.rust-lang.org/1.86.0/src/core/cmp.rs.html#1037-1039){.src}]{.rightside}[§](#method.min-1){.anchor}

#### fn [min](https://doc.rust-lang.org/1.86.0/core/cmp/trait.Ord.html#method.min){.fn}(self, other: Self) -\> Self {#fn-minself-other-self---self .code-header}

::: where
where Self:
[Sized](https://doc.rust-lang.org/1.86.0/core/marker/trait.Sized.html "trait core::marker::Sized"){.trait},
:::
::::

::: docblock
Compares and returns the minimum of two values. [Read
more](https://doc.rust-lang.org/1.86.0/core/cmp/trait.Ord.html#method.min)
:::

:::: {#method.clamp-1 .section .method .trait-impl}
[[1.50.0]{.since title="Stable since Rust version 1.50.0"} ·
[Source](https://doc.rust-lang.org/1.86.0/src/core/cmp.rs.html#1063-1065){.src}]{.rightside}[§](#method.clamp-1){.anchor}

#### fn [clamp](https://doc.rust-lang.org/1.86.0/core/cmp/trait.Ord.html#method.clamp){.fn}(self, min: Self, max: Self) -\> Self {#fn-clampself-min-self-max-self---self .code-header}

::: where
where Self:
[Sized](https://doc.rust-lang.org/1.86.0/core/marker/trait.Sized.html "trait core::marker::Sized"){.trait},
:::
::::

::: docblock
Restrict a value to a certain interval. [Read
more](https://doc.rust-lang.org/1.86.0/core/cmp/trait.Ord.html#method.clamp)
:::
::::::::::::::

::: {#impl-PartialEq%3C%26Positive%3E-for-Positive .section .impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#481-485){.src
.rightside}[§](#impl-PartialEq%3C%26Positive%3E-for-Positive){.anchor}

### impl [PartialEq](https://doc.rust-lang.org/1.86.0/core/cmp/trait.PartialEq.html "trait core::cmp::PartialEq"){.trait}\<&[Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\> for [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#impl-partialeqpositive-for-positive .code-header}
:::

::::::: impl-items
::: {#method.eq-2 .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#482-484){.src
.rightside}[§](#method.eq-2){.anchor}

#### fn [eq](https://doc.rust-lang.org/1.86.0/core/cmp/trait.PartialEq.html#tymethod.eq){.fn}(&self, other: &&[Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}) -\> [bool](https://doc.rust-lang.org/1.86.0/std/primitive.bool.html){.primitive} {#fn-eqself-other-positive---bool .code-header}
:::

::: docblock
Tests for `self` and `other` values to be equal, and is used by `==`.
:::

::: {#method.ne-2 .section .method .trait-impl}
[[1.0.0]{.since title="Stable since Rust version 1.0.0"} ·
[Source](https://doc.rust-lang.org/1.86.0/src/core/cmp.rs.html#261){.src}]{.rightside}[§](#method.ne-2){.anchor}

#### fn [ne](https://doc.rust-lang.org/1.86.0/core/cmp/trait.PartialEq.html#method.ne){.fn}(&self, other: [&Rhs](https://doc.rust-lang.org/1.86.0/std/primitive.reference.html){.primitive}) -\> [bool](https://doc.rust-lang.org/1.86.0/std/primitive.bool.html){.primitive} {#fn-neself-other-rhs---bool .code-header}
:::

::: docblock
Tests for `!=`. The default implementation is almost always sufficient,
and should not be overridden without very good reason.
:::
:::::::

::: {#impl-PartialEq%3C%26Positive%3E-for-f64 .section .impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#511-515){.src
.rightside}[§](#impl-PartialEq%3C%26Positive%3E-for-f64){.anchor}

### impl [PartialEq](https://doc.rust-lang.org/1.86.0/core/cmp/trait.PartialEq.html "trait core::cmp::PartialEq"){.trait}\<&[Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\> for [f64](https://doc.rust-lang.org/1.86.0/std/primitive.f64.html){.primitive} {#impl-partialeqpositive-for-f64 .code-header}
:::

::::::: impl-items
::: {#method.eq-3 .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#512-514){.src
.rightside}[§](#method.eq-3){.anchor}

#### fn [eq](https://doc.rust-lang.org/1.86.0/core/cmp/trait.PartialEq.html#tymethod.eq){.fn}(&self, other: &&[Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}) -\> [bool](https://doc.rust-lang.org/1.86.0/std/primitive.bool.html){.primitive} {#fn-eqself-other-positive---bool-1 .code-header}
:::

::: docblock
Tests for `self` and `other` values to be equal, and is used by `==`.
:::

::: {#method.ne-3 .section .method .trait-impl}
[[1.0.0]{.since title="Stable since Rust version 1.0.0"} ·
[Source](https://doc.rust-lang.org/1.86.0/src/core/cmp.rs.html#261){.src}]{.rightside}[§](#method.ne-3){.anchor}

#### fn [ne](https://doc.rust-lang.org/1.86.0/core/cmp/trait.PartialEq.html#method.ne){.fn}(&self, other: [&Rhs](https://doc.rust-lang.org/1.86.0/std/primitive.reference.html){.primitive}) -\> [bool](https://doc.rust-lang.org/1.86.0/std/primitive.bool.html){.primitive} {#fn-neself-other-rhs---bool-1 .code-header}
:::

::: docblock
Tests for `!=`. The default implementation is almost always sufficient,
and should not be overridden without very good reason.
:::
:::::::

::: {#impl-PartialEq%3CDecimal%3E-for-Positive .section .impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#716-720){.src
.rightside}[§](#impl-PartialEq%3CDecimal%3E-for-Positive){.anchor}

### impl [PartialEq](https://doc.rust-lang.org/1.86.0/core/cmp/trait.PartialEq.html "trait core::cmp::PartialEq"){.trait}\<Decimal\> for [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#impl-partialeqdecimal-for-positive .code-header}
:::

::::::: impl-items
::: {#method.eq-7 .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#717-719){.src
.rightside}[§](#method.eq-7){.anchor}

#### fn [eq](https://doc.rust-lang.org/1.86.0/core/cmp/trait.PartialEq.html#tymethod.eq){.fn}(&self, other: &Decimal) -\> [bool](https://doc.rust-lang.org/1.86.0/std/primitive.bool.html){.primitive} {#fn-eqself-other-decimal---bool .code-header}
:::

::: docblock
Tests for `self` and `other` values to be equal, and is used by `==`.
:::

::: {#method.ne-7 .section .method .trait-impl}
[[1.0.0]{.since title="Stable since Rust version 1.0.0"} ·
[Source](https://doc.rust-lang.org/1.86.0/src/core/cmp.rs.html#261){.src}]{.rightside}[§](#method.ne-7){.anchor}

#### fn [ne](https://doc.rust-lang.org/1.86.0/core/cmp/trait.PartialEq.html#method.ne){.fn}(&self, other: [&Rhs](https://doc.rust-lang.org/1.86.0/std/primitive.reference.html){.primitive}) -\> [bool](https://doc.rust-lang.org/1.86.0/std/primitive.bool.html){.primitive} {#fn-neself-other-rhs---bool-2 .code-header}
:::

::: docblock
Tests for `!=`. The default implementation is almost always sufficient,
and should not be overridden without very good reason.
:::
:::::::

::: {#impl-PartialEq%3CPositive%3E-for-Decimal .section .impl}
[Source](../../../src/optionstratlib/model/decimal.rs.html#209-213){.src
.rightside}[§](#impl-PartialEq%3CPositive%3E-for-Decimal){.anchor}

### impl [PartialEq](https://doc.rust-lang.org/1.86.0/core/cmp/trait.PartialEq.html "trait core::cmp::PartialEq"){.trait}\<[Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\> for Decimal {#impl-partialeqpositive-for-decimal .code-header}
:::

::::::: impl-items
::: {#method.eq .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/decimal.rs.html#210-212){.src
.rightside}[§](#method.eq){.anchor}

#### fn [eq](https://doc.rust-lang.org/1.86.0/core/cmp/trait.PartialEq.html#tymethod.eq){.fn}(&self, other: &[Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}) -\> [bool](https://doc.rust-lang.org/1.86.0/std/primitive.bool.html){.primitive} {#fn-eqself-other-positive---bool-2 .code-header}
:::

::: docblock
Tests for `self` and `other` values to be equal, and is used by `==`.
:::

::: {#method.ne .section .method .trait-impl}
[[1.0.0]{.since title="Stable since Rust version 1.0.0"} ·
[Source](https://doc.rust-lang.org/1.86.0/src/core/cmp.rs.html#261){.src}]{.rightside}[§](#method.ne){.anchor}

#### fn [ne](https://doc.rust-lang.org/1.86.0/core/cmp/trait.PartialEq.html#method.ne){.fn}(&self, other: [&Rhs](https://doc.rust-lang.org/1.86.0/std/primitive.reference.html){.primitive}) -\> [bool](https://doc.rust-lang.org/1.86.0/std/primitive.bool.html){.primitive} {#fn-neself-other-rhs---bool-3 .code-header}
:::

::: docblock
Tests for `!=`. The default implementation is almost always sufficient,
and should not be overridden without very good reason.
:::
:::::::

::: {#impl-PartialEq%3CPositive%3E-for-f64 .section .impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#523-527){.src
.rightside}[§](#impl-PartialEq%3CPositive%3E-for-f64){.anchor}

### impl [PartialEq](https://doc.rust-lang.org/1.86.0/core/cmp/trait.PartialEq.html "trait core::cmp::PartialEq"){.trait}\<[Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\> for [f64](https://doc.rust-lang.org/1.86.0/std/primitive.f64.html){.primitive} {#impl-partialeqpositive-for-f64-1 .code-header}
:::

::::::: impl-items
::: {#method.eq-4 .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#524-526){.src
.rightside}[§](#method.eq-4){.anchor}

#### fn [eq](https://doc.rust-lang.org/1.86.0/core/cmp/trait.PartialEq.html#tymethod.eq){.fn}(&self, other: &[Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}) -\> [bool](https://doc.rust-lang.org/1.86.0/std/primitive.bool.html){.primitive} {#fn-eqself-other-positive---bool-3 .code-header}
:::

::: docblock
Tests for `self` and `other` values to be equal, and is used by `==`.
:::

::: {#method.ne-4 .section .method .trait-impl}
[[1.0.0]{.since title="Stable since Rust version 1.0.0"} ·
[Source](https://doc.rust-lang.org/1.86.0/src/core/cmp.rs.html#261){.src}]{.rightside}[§](#method.ne-4){.anchor}

#### fn [ne](https://doc.rust-lang.org/1.86.0/core/cmp/trait.PartialEq.html#method.ne){.fn}(&self, other: [&Rhs](https://doc.rust-lang.org/1.86.0/std/primitive.reference.html){.primitive}) -\> [bool](https://doc.rust-lang.org/1.86.0/std/primitive.bool.html){.primitive} {#fn-neself-other-rhs---bool-4 .code-header}
:::

::: docblock
Tests for `!=`. The default implementation is almost always sufficient,
and should not be overridden without very good reason.
:::
:::::::

::: {#impl-PartialEq%3Cf64%3E-for-%26Positive .section .impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#667-671){.src
.rightside}[§](#impl-PartialEq%3Cf64%3E-for-%26Positive){.anchor}

### impl [PartialEq](https://doc.rust-lang.org/1.86.0/core/cmp/trait.PartialEq.html "trait core::cmp::PartialEq"){.trait}\<[f64](https://doc.rust-lang.org/1.86.0/std/primitive.f64.html){.primitive}\> for &[Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#impl-partialeqf64-for-positive .code-header}
:::

::::::: impl-items
::: {#method.eq-5 .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#668-670){.src
.rightside}[§](#method.eq-5){.anchor}

#### fn [eq](https://doc.rust-lang.org/1.86.0/core/cmp/trait.PartialEq.html#tymethod.eq){.fn}(&self, other: &[f64](https://doc.rust-lang.org/1.86.0/std/primitive.f64.html){.primitive}) -\> [bool](https://doc.rust-lang.org/1.86.0/std/primitive.bool.html){.primitive} {#fn-eqself-other-f64---bool .code-header}
:::

::: docblock
Tests for `self` and `other` values to be equal, and is used by `==`.
:::

::: {#method.ne-5 .section .method .trait-impl}
[[1.0.0]{.since title="Stable since Rust version 1.0.0"} ·
[Source](https://doc.rust-lang.org/1.86.0/src/core/cmp.rs.html#261){.src}]{.rightside}[§](#method.ne-5){.anchor}

#### fn [ne](https://doc.rust-lang.org/1.86.0/core/cmp/trait.PartialEq.html#method.ne){.fn}(&self, other: [&Rhs](https://doc.rust-lang.org/1.86.0/std/primitive.reference.html){.primitive}) -\> [bool](https://doc.rust-lang.org/1.86.0/std/primitive.bool.html){.primitive} {#fn-neself-other-rhs---bool-5 .code-header}
:::

::: docblock
Tests for `!=`. The default implementation is almost always sufficient,
and should not be overridden without very good reason.
:::
:::::::

::: {#impl-PartialEq%3Cf64%3E-for-Positive .section .impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#679-683){.src
.rightside}[§](#impl-PartialEq%3Cf64%3E-for-Positive){.anchor}

### impl [PartialEq](https://doc.rust-lang.org/1.86.0/core/cmp/trait.PartialEq.html "trait core::cmp::PartialEq"){.trait}\<[f64](https://doc.rust-lang.org/1.86.0/std/primitive.f64.html){.primitive}\> for [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#impl-partialeqf64-for-positive-1 .code-header}
:::

::::::: impl-items
::: {#method.eq-6 .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#680-682){.src
.rightside}[§](#method.eq-6){.anchor}

#### fn [eq](https://doc.rust-lang.org/1.86.0/core/cmp/trait.PartialEq.html#tymethod.eq){.fn}(&self, other: &[f64](https://doc.rust-lang.org/1.86.0/std/primitive.f64.html){.primitive}) -\> [bool](https://doc.rust-lang.org/1.86.0/std/primitive.bool.html){.primitive} {#fn-eqself-other-f64---bool-1 .code-header}
:::

::: docblock
Tests for `self` and `other` values to be equal, and is used by `==`.
:::

::: {#method.ne-6 .section .method .trait-impl}
[[1.0.0]{.since title="Stable since Rust version 1.0.0"} ·
[Source](https://doc.rust-lang.org/1.86.0/src/core/cmp.rs.html#261){.src}]{.rightside}[§](#method.ne-6){.anchor}

#### fn [ne](https://doc.rust-lang.org/1.86.0/core/cmp/trait.PartialEq.html#method.ne){.fn}(&self, other: [&Rhs](https://doc.rust-lang.org/1.86.0/std/primitive.reference.html){.primitive}) -\> [bool](https://doc.rust-lang.org/1.86.0/std/primitive.bool.html){.primitive} {#fn-neself-other-rhs---bool-6 .code-header}
:::

::: docblock
Tests for `!=`. The default implementation is almost always sufficient,
and should not be overridden without very good reason.
:::
:::::::

::: {#impl-PartialEq-for-Positive .section .impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#39){.src
.rightside}[§](#impl-PartialEq-for-Positive){.anchor}

### impl [PartialEq](https://doc.rust-lang.org/1.86.0/core/cmp/trait.PartialEq.html "trait core::cmp::PartialEq"){.trait} for [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#impl-partialeq-for-positive .code-header}
:::

::::::: impl-items
::: {#method.eq-1 .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#39){.src
.rightside}[§](#method.eq-1){.anchor}

#### fn [eq](https://doc.rust-lang.org/1.86.0/core/cmp/trait.PartialEq.html#tymethod.eq){.fn}(&self, other: &[Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}) -\> [bool](https://doc.rust-lang.org/1.86.0/std/primitive.bool.html){.primitive} {#fn-eqself-other-positive---bool-4 .code-header}
:::

::: docblock
Tests for `self` and `other` values to be equal, and is used by `==`.
:::

::: {#method.ne-1 .section .method .trait-impl}
[[1.0.0]{.since title="Stable since Rust version 1.0.0"} ·
[Source](https://doc.rust-lang.org/1.86.0/src/core/cmp.rs.html#261){.src}]{.rightside}[§](#method.ne-1){.anchor}

#### fn [ne](https://doc.rust-lang.org/1.86.0/core/cmp/trait.PartialEq.html#method.ne){.fn}(&self, other: [&Rhs](https://doc.rust-lang.org/1.86.0/std/primitive.reference.html){.primitive}) -\> [bool](https://doc.rust-lang.org/1.86.0/std/primitive.bool.html){.primitive} {#fn-neself-other-rhs---bool-7 .code-header}
:::

::: docblock
Tests for `!=`. The default implementation is almost always sufficient,
and should not be overridden without very good reason.
:::
:::::::

::: {#impl-PartialOrd%3C%26Positive%3E-for-f64 .section .impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#517-521){.src
.rightside}[§](#impl-PartialOrd%3C%26Positive%3E-for-f64){.anchor}

### impl [PartialOrd](https://doc.rust-lang.org/1.86.0/core/cmp/trait.PartialOrd.html "trait core::cmp::PartialOrd"){.trait}\<&[Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\> for [f64](https://doc.rust-lang.org/1.86.0/std/primitive.f64.html){.primitive} {#impl-partialordpositive-for-f64 .code-header}
:::

::::::::::::: impl-items
::: {#method.partial_cmp .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#518-520){.src
.rightside}[§](#method.partial_cmp){.anchor}

#### fn [partial_cmp](https://doc.rust-lang.org/1.86.0/core/cmp/trait.PartialOrd.html#tymethod.partial_cmp){.fn}(&self, other: &&[Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}) -\> [Option](https://doc.rust-lang.org/1.86.0/core/option/enum.Option.html "enum core::option::Option"){.enum}\<[Ordering](https://doc.rust-lang.org/1.86.0/core/cmp/enum.Ordering.html "enum core::cmp::Ordering"){.enum}\> {#fn-partial_cmpself-other-positive---optionordering .code-header}
:::

::: docblock
This method returns an ordering between `self` and `other` values if one
exists. [Read
more](https://doc.rust-lang.org/1.86.0/core/cmp/trait.PartialOrd.html#tymethod.partial_cmp)
:::

::: {#method.lt .section .method .trait-impl}
[[1.0.0]{.since title="Stable since Rust version 1.0.0"} ·
[Source](https://doc.rust-lang.org/1.86.0/src/core/cmp.rs.html#1371){.src}]{.rightside}[§](#method.lt){.anchor}

#### fn [lt](https://doc.rust-lang.org/1.86.0/core/cmp/trait.PartialOrd.html#method.lt){.fn}(&self, other: [&Rhs](https://doc.rust-lang.org/1.86.0/std/primitive.reference.html){.primitive}) -\> [bool](https://doc.rust-lang.org/1.86.0/std/primitive.bool.html){.primitive} {#fn-ltself-other-rhs---bool .code-header}
:::

::: docblock
Tests less than (for `self` and `other`) and is used by the `<`
operator. [Read
more](https://doc.rust-lang.org/1.86.0/core/cmp/trait.PartialOrd.html#method.lt)
:::

::: {#method.le .section .method .trait-impl}
[[1.0.0]{.since title="Stable since Rust version 1.0.0"} ·
[Source](https://doc.rust-lang.org/1.86.0/src/core/cmp.rs.html#1389){.src}]{.rightside}[§](#method.le){.anchor}

#### fn [le](https://doc.rust-lang.org/1.86.0/core/cmp/trait.PartialOrd.html#method.le){.fn}(&self, other: [&Rhs](https://doc.rust-lang.org/1.86.0/std/primitive.reference.html){.primitive}) -\> [bool](https://doc.rust-lang.org/1.86.0/std/primitive.bool.html){.primitive} {#fn-leself-other-rhs---bool .code-header}
:::

::: docblock
Tests less than or equal to (for `self` and `other`) and is used by the
`<=` operator. [Read
more](https://doc.rust-lang.org/1.86.0/core/cmp/trait.PartialOrd.html#method.le)
:::

::: {#method.gt .section .method .trait-impl}
[[1.0.0]{.since title="Stable since Rust version 1.0.0"} ·
[Source](https://doc.rust-lang.org/1.86.0/src/core/cmp.rs.html#1407){.src}]{.rightside}[§](#method.gt){.anchor}

#### fn [gt](https://doc.rust-lang.org/1.86.0/core/cmp/trait.PartialOrd.html#method.gt){.fn}(&self, other: [&Rhs](https://doc.rust-lang.org/1.86.0/std/primitive.reference.html){.primitive}) -\> [bool](https://doc.rust-lang.org/1.86.0/std/primitive.bool.html){.primitive} {#fn-gtself-other-rhs---bool .code-header}
:::

::: docblock
Tests greater than (for `self` and `other`) and is used by the `>`
operator. [Read
more](https://doc.rust-lang.org/1.86.0/core/cmp/trait.PartialOrd.html#method.gt)
:::

::: {#method.ge .section .method .trait-impl}
[[1.0.0]{.since title="Stable since Rust version 1.0.0"} ·
[Source](https://doc.rust-lang.org/1.86.0/src/core/cmp.rs.html#1425){.src}]{.rightside}[§](#method.ge){.anchor}

#### fn [ge](https://doc.rust-lang.org/1.86.0/core/cmp/trait.PartialOrd.html#method.ge){.fn}(&self, other: [&Rhs](https://doc.rust-lang.org/1.86.0/std/primitive.reference.html){.primitive}) -\> [bool](https://doc.rust-lang.org/1.86.0/std/primitive.bool.html){.primitive} {#fn-geself-other-rhs---bool .code-header}
:::

::: docblock
Tests greater than or equal to (for `self` and `other`) and is used by
the `>=` operator. [Read
more](https://doc.rust-lang.org/1.86.0/core/cmp/trait.PartialOrd.html#method.ge)
:::
:::::::::::::

::: {#impl-PartialOrd%3CDecimal%3E-for-Positive .section .impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#913-917){.src
.rightside}[§](#impl-PartialOrd%3CDecimal%3E-for-Positive){.anchor}

### impl [PartialOrd](https://doc.rust-lang.org/1.86.0/core/cmp/trait.PartialOrd.html "trait core::cmp::PartialOrd"){.trait}\<Decimal\> for [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#impl-partialorddecimal-for-positive .code-header}
:::

::::::::::::: impl-items
::: {#method.partial_cmp-4 .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#914-916){.src
.rightside}[§](#method.partial_cmp-4){.anchor}

#### fn [partial_cmp](https://doc.rust-lang.org/1.86.0/core/cmp/trait.PartialOrd.html#tymethod.partial_cmp){.fn}(&self, other: &Decimal) -\> [Option](https://doc.rust-lang.org/1.86.0/core/option/enum.Option.html "enum core::option::Option"){.enum}\<[Ordering](https://doc.rust-lang.org/1.86.0/core/cmp/enum.Ordering.html "enum core::cmp::Ordering"){.enum}\> {#fn-partial_cmpself-other-decimal---optionordering .code-header}
:::

::: docblock
This method returns an ordering between `self` and `other` values if one
exists. [Read
more](https://doc.rust-lang.org/1.86.0/core/cmp/trait.PartialOrd.html#tymethod.partial_cmp)
:::

::: {#method.lt-4 .section .method .trait-impl}
[[1.0.0]{.since title="Stable since Rust version 1.0.0"} ·
[Source](https://doc.rust-lang.org/1.86.0/src/core/cmp.rs.html#1371){.src}]{.rightside}[§](#method.lt-4){.anchor}

#### fn [lt](https://doc.rust-lang.org/1.86.0/core/cmp/trait.PartialOrd.html#method.lt){.fn}(&self, other: [&Rhs](https://doc.rust-lang.org/1.86.0/std/primitive.reference.html){.primitive}) -\> [bool](https://doc.rust-lang.org/1.86.0/std/primitive.bool.html){.primitive} {#fn-ltself-other-rhs---bool-1 .code-header}
:::

::: docblock
Tests less than (for `self` and `other`) and is used by the `<`
operator. [Read
more](https://doc.rust-lang.org/1.86.0/core/cmp/trait.PartialOrd.html#method.lt)
:::

::: {#method.le-4 .section .method .trait-impl}
[[1.0.0]{.since title="Stable since Rust version 1.0.0"} ·
[Source](https://doc.rust-lang.org/1.86.0/src/core/cmp.rs.html#1389){.src}]{.rightside}[§](#method.le-4){.anchor}

#### fn [le](https://doc.rust-lang.org/1.86.0/core/cmp/trait.PartialOrd.html#method.le){.fn}(&self, other: [&Rhs](https://doc.rust-lang.org/1.86.0/std/primitive.reference.html){.primitive}) -\> [bool](https://doc.rust-lang.org/1.86.0/std/primitive.bool.html){.primitive} {#fn-leself-other-rhs---bool-1 .code-header}
:::

::: docblock
Tests less than or equal to (for `self` and `other`) and is used by the
`<=` operator. [Read
more](https://doc.rust-lang.org/1.86.0/core/cmp/trait.PartialOrd.html#method.le)
:::

::: {#method.gt-4 .section .method .trait-impl}
[[1.0.0]{.since title="Stable since Rust version 1.0.0"} ·
[Source](https://doc.rust-lang.org/1.86.0/src/core/cmp.rs.html#1407){.src}]{.rightside}[§](#method.gt-4){.anchor}

#### fn [gt](https://doc.rust-lang.org/1.86.0/core/cmp/trait.PartialOrd.html#method.gt){.fn}(&self, other: [&Rhs](https://doc.rust-lang.org/1.86.0/std/primitive.reference.html){.primitive}) -\> [bool](https://doc.rust-lang.org/1.86.0/std/primitive.bool.html){.primitive} {#fn-gtself-other-rhs---bool-1 .code-header}
:::

::: docblock
Tests greater than (for `self` and `other`) and is used by the `>`
operator. [Read
more](https://doc.rust-lang.org/1.86.0/core/cmp/trait.PartialOrd.html#method.gt)
:::

::: {#method.ge-4 .section .method .trait-impl}
[[1.0.0]{.since title="Stable since Rust version 1.0.0"} ·
[Source](https://doc.rust-lang.org/1.86.0/src/core/cmp.rs.html#1425){.src}]{.rightside}[§](#method.ge-4){.anchor}

#### fn [ge](https://doc.rust-lang.org/1.86.0/core/cmp/trait.PartialOrd.html#method.ge){.fn}(&self, other: [&Rhs](https://doc.rust-lang.org/1.86.0/std/primitive.reference.html){.primitive}) -\> [bool](https://doc.rust-lang.org/1.86.0/std/primitive.bool.html){.primitive} {#fn-geself-other-rhs---bool-1 .code-header}
:::

::: docblock
Tests greater than or equal to (for `self` and `other`) and is used by
the `>=` operator. [Read
more](https://doc.rust-lang.org/1.86.0/core/cmp/trait.PartialOrd.html#method.ge)
:::
:::::::::::::

::: {#impl-PartialOrd%3CPositive%3E-for-f64 .section .impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#529-533){.src
.rightside}[§](#impl-PartialOrd%3CPositive%3E-for-f64){.anchor}

### impl [PartialOrd](https://doc.rust-lang.org/1.86.0/core/cmp/trait.PartialOrd.html "trait core::cmp::PartialOrd"){.trait}\<[Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\> for [f64](https://doc.rust-lang.org/1.86.0/std/primitive.f64.html){.primitive} {#impl-partialordpositive-for-f64-1 .code-header}
:::

::::::::::::: impl-items
::: {#method.partial_cmp-1 .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#530-532){.src
.rightside}[§](#method.partial_cmp-1){.anchor}

#### fn [partial_cmp](https://doc.rust-lang.org/1.86.0/core/cmp/trait.PartialOrd.html#tymethod.partial_cmp){.fn}(&self, other: &[Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}) -\> [Option](https://doc.rust-lang.org/1.86.0/core/option/enum.Option.html "enum core::option::Option"){.enum}\<[Ordering](https://doc.rust-lang.org/1.86.0/core/cmp/enum.Ordering.html "enum core::cmp::Ordering"){.enum}\> {#fn-partial_cmpself-other-positive---optionordering-1 .code-header}
:::

::: docblock
This method returns an ordering between `self` and `other` values if one
exists. [Read
more](https://doc.rust-lang.org/1.86.0/core/cmp/trait.PartialOrd.html#tymethod.partial_cmp)
:::

::: {#method.lt-1 .section .method .trait-impl}
[[1.0.0]{.since title="Stable since Rust version 1.0.0"} ·
[Source](https://doc.rust-lang.org/1.86.0/src/core/cmp.rs.html#1371){.src}]{.rightside}[§](#method.lt-1){.anchor}

#### fn [lt](https://doc.rust-lang.org/1.86.0/core/cmp/trait.PartialOrd.html#method.lt){.fn}(&self, other: [&Rhs](https://doc.rust-lang.org/1.86.0/std/primitive.reference.html){.primitive}) -\> [bool](https://doc.rust-lang.org/1.86.0/std/primitive.bool.html){.primitive} {#fn-ltself-other-rhs---bool-2 .code-header}
:::

::: docblock
Tests less than (for `self` and `other`) and is used by the `<`
operator. [Read
more](https://doc.rust-lang.org/1.86.0/core/cmp/trait.PartialOrd.html#method.lt)
:::

::: {#method.le-1 .section .method .trait-impl}
[[1.0.0]{.since title="Stable since Rust version 1.0.0"} ·
[Source](https://doc.rust-lang.org/1.86.0/src/core/cmp.rs.html#1389){.src}]{.rightside}[§](#method.le-1){.anchor}

#### fn [le](https://doc.rust-lang.org/1.86.0/core/cmp/trait.PartialOrd.html#method.le){.fn}(&self, other: [&Rhs](https://doc.rust-lang.org/1.86.0/std/primitive.reference.html){.primitive}) -\> [bool](https://doc.rust-lang.org/1.86.0/std/primitive.bool.html){.primitive} {#fn-leself-other-rhs---bool-2 .code-header}
:::

::: docblock
Tests less than or equal to (for `self` and `other`) and is used by the
`<=` operator. [Read
more](https://doc.rust-lang.org/1.86.0/core/cmp/trait.PartialOrd.html#method.le)
:::

::: {#method.gt-1 .section .method .trait-impl}
[[1.0.0]{.since title="Stable since Rust version 1.0.0"} ·
[Source](https://doc.rust-lang.org/1.86.0/src/core/cmp.rs.html#1407){.src}]{.rightside}[§](#method.gt-1){.anchor}

#### fn [gt](https://doc.rust-lang.org/1.86.0/core/cmp/trait.PartialOrd.html#method.gt){.fn}(&self, other: [&Rhs](https://doc.rust-lang.org/1.86.0/std/primitive.reference.html){.primitive}) -\> [bool](https://doc.rust-lang.org/1.86.0/std/primitive.bool.html){.primitive} {#fn-gtself-other-rhs---bool-2 .code-header}
:::

::: docblock
Tests greater than (for `self` and `other`) and is used by the `>`
operator. [Read
more](https://doc.rust-lang.org/1.86.0/core/cmp/trait.PartialOrd.html#method.gt)
:::

::: {#method.ge-1 .section .method .trait-impl}
[[1.0.0]{.since title="Stable since Rust version 1.0.0"} ·
[Source](https://doc.rust-lang.org/1.86.0/src/core/cmp.rs.html#1425){.src}]{.rightside}[§](#method.ge-1){.anchor}

#### fn [ge](https://doc.rust-lang.org/1.86.0/core/cmp/trait.PartialOrd.html#method.ge){.fn}(&self, other: [&Rhs](https://doc.rust-lang.org/1.86.0/std/primitive.reference.html){.primitive}) -\> [bool](https://doc.rust-lang.org/1.86.0/std/primitive.bool.html){.primitive} {#fn-geself-other-rhs---bool-2 .code-header}
:::

::: docblock
Tests greater than or equal to (for `self` and `other`) and is used by
the `>=` operator. [Read
more](https://doc.rust-lang.org/1.86.0/core/cmp/trait.PartialOrd.html#method.ge)
:::
:::::::::::::

::: {#impl-PartialOrd%3Cf64%3E-for-%26Positive .section .impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#673-677){.src
.rightside}[§](#impl-PartialOrd%3Cf64%3E-for-%26Positive){.anchor}

### impl [PartialOrd](https://doc.rust-lang.org/1.86.0/core/cmp/trait.PartialOrd.html "trait core::cmp::PartialOrd"){.trait}\<[f64](https://doc.rust-lang.org/1.86.0/std/primitive.f64.html){.primitive}\> for &[Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#impl-partialordf64-for-positive .code-header}
:::

::::::::::::: impl-items
::: {#method.partial_cmp-3 .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#674-676){.src
.rightside}[§](#method.partial_cmp-3){.anchor}

#### fn [partial_cmp](https://doc.rust-lang.org/1.86.0/core/cmp/trait.PartialOrd.html#tymethod.partial_cmp){.fn}(&self, other: &[f64](https://doc.rust-lang.org/1.86.0/std/primitive.f64.html){.primitive}) -\> [Option](https://doc.rust-lang.org/1.86.0/core/option/enum.Option.html "enum core::option::Option"){.enum}\<[Ordering](https://doc.rust-lang.org/1.86.0/core/cmp/enum.Ordering.html "enum core::cmp::Ordering"){.enum}\> {#fn-partial_cmpself-other-f64---optionordering .code-header}
:::

::: docblock
This method returns an ordering between `self` and `other` values if one
exists. [Read
more](https://doc.rust-lang.org/1.86.0/core/cmp/trait.PartialOrd.html#tymethod.partial_cmp)
:::

::: {#method.lt-3 .section .method .trait-impl}
[[1.0.0]{.since title="Stable since Rust version 1.0.0"} ·
[Source](https://doc.rust-lang.org/1.86.0/src/core/cmp.rs.html#1371){.src}]{.rightside}[§](#method.lt-3){.anchor}

#### fn [lt](https://doc.rust-lang.org/1.86.0/core/cmp/trait.PartialOrd.html#method.lt){.fn}(&self, other: [&Rhs](https://doc.rust-lang.org/1.86.0/std/primitive.reference.html){.primitive}) -\> [bool](https://doc.rust-lang.org/1.86.0/std/primitive.bool.html){.primitive} {#fn-ltself-other-rhs---bool-3 .code-header}
:::

::: docblock
Tests less than (for `self` and `other`) and is used by the `<`
operator. [Read
more](https://doc.rust-lang.org/1.86.0/core/cmp/trait.PartialOrd.html#method.lt)
:::

::: {#method.le-3 .section .method .trait-impl}
[[1.0.0]{.since title="Stable since Rust version 1.0.0"} ·
[Source](https://doc.rust-lang.org/1.86.0/src/core/cmp.rs.html#1389){.src}]{.rightside}[§](#method.le-3){.anchor}

#### fn [le](https://doc.rust-lang.org/1.86.0/core/cmp/trait.PartialOrd.html#method.le){.fn}(&self, other: [&Rhs](https://doc.rust-lang.org/1.86.0/std/primitive.reference.html){.primitive}) -\> [bool](https://doc.rust-lang.org/1.86.0/std/primitive.bool.html){.primitive} {#fn-leself-other-rhs---bool-3 .code-header}
:::

::: docblock
Tests less than or equal to (for `self` and `other`) and is used by the
`<=` operator. [Read
more](https://doc.rust-lang.org/1.86.0/core/cmp/trait.PartialOrd.html#method.le)
:::

::: {#method.gt-3 .section .method .trait-impl}
[[1.0.0]{.since title="Stable since Rust version 1.0.0"} ·
[Source](https://doc.rust-lang.org/1.86.0/src/core/cmp.rs.html#1407){.src}]{.rightside}[§](#method.gt-3){.anchor}

#### fn [gt](https://doc.rust-lang.org/1.86.0/core/cmp/trait.PartialOrd.html#method.gt){.fn}(&self, other: [&Rhs](https://doc.rust-lang.org/1.86.0/std/primitive.reference.html){.primitive}) -\> [bool](https://doc.rust-lang.org/1.86.0/std/primitive.bool.html){.primitive} {#fn-gtself-other-rhs---bool-3 .code-header}
:::

::: docblock
Tests greater than (for `self` and `other`) and is used by the `>`
operator. [Read
more](https://doc.rust-lang.org/1.86.0/core/cmp/trait.PartialOrd.html#method.gt)
:::

::: {#method.ge-3 .section .method .trait-impl}
[[1.0.0]{.since title="Stable since Rust version 1.0.0"} ·
[Source](https://doc.rust-lang.org/1.86.0/src/core/cmp.rs.html#1425){.src}]{.rightside}[§](#method.ge-3){.anchor}

#### fn [ge](https://doc.rust-lang.org/1.86.0/core/cmp/trait.PartialOrd.html#method.ge){.fn}(&self, other: [&Rhs](https://doc.rust-lang.org/1.86.0/std/primitive.reference.html){.primitive}) -\> [bool](https://doc.rust-lang.org/1.86.0/std/primitive.bool.html){.primitive} {#fn-geself-other-rhs---bool-3 .code-header}
:::

::: docblock
Tests greater than or equal to (for `self` and `other`) and is used by
the `>=` operator. [Read
more](https://doc.rust-lang.org/1.86.0/core/cmp/trait.PartialOrd.html#method.ge)
:::
:::::::::::::

::: {#impl-PartialOrd%3Cf64%3E-for-Positive .section .impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#661-665){.src
.rightside}[§](#impl-PartialOrd%3Cf64%3E-for-Positive){.anchor}

### impl [PartialOrd](https://doc.rust-lang.org/1.86.0/core/cmp/trait.PartialOrd.html "trait core::cmp::PartialOrd"){.trait}\<[f64](https://doc.rust-lang.org/1.86.0/std/primitive.f64.html){.primitive}\> for [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#impl-partialordf64-for-positive-1 .code-header}
:::

::::::::::::: impl-items
::: {#method.partial_cmp-2 .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#662-664){.src
.rightside}[§](#method.partial_cmp-2){.anchor}

#### fn [partial_cmp](https://doc.rust-lang.org/1.86.0/core/cmp/trait.PartialOrd.html#tymethod.partial_cmp){.fn}(&self, other: &[f64](https://doc.rust-lang.org/1.86.0/std/primitive.f64.html){.primitive}) -\> [Option](https://doc.rust-lang.org/1.86.0/core/option/enum.Option.html "enum core::option::Option"){.enum}\<[Ordering](https://doc.rust-lang.org/1.86.0/core/cmp/enum.Ordering.html "enum core::cmp::Ordering"){.enum}\> {#fn-partial_cmpself-other-f64---optionordering-1 .code-header}
:::

::: docblock
This method returns an ordering between `self` and `other` values if one
exists. [Read
more](https://doc.rust-lang.org/1.86.0/core/cmp/trait.PartialOrd.html#tymethod.partial_cmp)
:::

::: {#method.lt-2 .section .method .trait-impl}
[[1.0.0]{.since title="Stable since Rust version 1.0.0"} ·
[Source](https://doc.rust-lang.org/1.86.0/src/core/cmp.rs.html#1371){.src}]{.rightside}[§](#method.lt-2){.anchor}

#### fn [lt](https://doc.rust-lang.org/1.86.0/core/cmp/trait.PartialOrd.html#method.lt){.fn}(&self, other: [&Rhs](https://doc.rust-lang.org/1.86.0/std/primitive.reference.html){.primitive}) -\> [bool](https://doc.rust-lang.org/1.86.0/std/primitive.bool.html){.primitive} {#fn-ltself-other-rhs---bool-4 .code-header}
:::

::: docblock
Tests less than (for `self` and `other`) and is used by the `<`
operator. [Read
more](https://doc.rust-lang.org/1.86.0/core/cmp/trait.PartialOrd.html#method.lt)
:::

::: {#method.le-2 .section .method .trait-impl}
[[1.0.0]{.since title="Stable since Rust version 1.0.0"} ·
[Source](https://doc.rust-lang.org/1.86.0/src/core/cmp.rs.html#1389){.src}]{.rightside}[§](#method.le-2){.anchor}

#### fn [le](https://doc.rust-lang.org/1.86.0/core/cmp/trait.PartialOrd.html#method.le){.fn}(&self, other: [&Rhs](https://doc.rust-lang.org/1.86.0/std/primitive.reference.html){.primitive}) -\> [bool](https://doc.rust-lang.org/1.86.0/std/primitive.bool.html){.primitive} {#fn-leself-other-rhs---bool-4 .code-header}
:::

::: docblock
Tests less than or equal to (for `self` and `other`) and is used by the
`<=` operator. [Read
more](https://doc.rust-lang.org/1.86.0/core/cmp/trait.PartialOrd.html#method.le)
:::

::: {#method.gt-2 .section .method .trait-impl}
[[1.0.0]{.since title="Stable since Rust version 1.0.0"} ·
[Source](https://doc.rust-lang.org/1.86.0/src/core/cmp.rs.html#1407){.src}]{.rightside}[§](#method.gt-2){.anchor}

#### fn [gt](https://doc.rust-lang.org/1.86.0/core/cmp/trait.PartialOrd.html#method.gt){.fn}(&self, other: [&Rhs](https://doc.rust-lang.org/1.86.0/std/primitive.reference.html){.primitive}) -\> [bool](https://doc.rust-lang.org/1.86.0/std/primitive.bool.html){.primitive} {#fn-gtself-other-rhs---bool-4 .code-header}
:::

::: docblock
Tests greater than (for `self` and `other`) and is used by the `>`
operator. [Read
more](https://doc.rust-lang.org/1.86.0/core/cmp/trait.PartialOrd.html#method.gt)
:::

::: {#method.ge-2 .section .method .trait-impl}
[[1.0.0]{.since title="Stable since Rust version 1.0.0"} ·
[Source](https://doc.rust-lang.org/1.86.0/src/core/cmp.rs.html#1425){.src}]{.rightside}[§](#method.ge-2){.anchor}

#### fn [ge](https://doc.rust-lang.org/1.86.0/core/cmp/trait.PartialOrd.html#method.ge){.fn}(&self, other: [&Rhs](https://doc.rust-lang.org/1.86.0/std/primitive.reference.html){.primitive}) -\> [bool](https://doc.rust-lang.org/1.86.0/std/primitive.bool.html){.primitive} {#fn-geself-other-rhs---bool-4 .code-header}
:::

::: docblock
Tests greater than or equal to (for `self` and `other`) and is used by
the `>=` operator. [Read
more](https://doc.rust-lang.org/1.86.0/core/cmp/trait.PartialOrd.html#method.ge)
:::
:::::::::::::

::: {#impl-PartialOrd-for-Positive .section .impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#919-931){.src
.rightside}[§](#impl-PartialOrd-for-Positive){.anchor}

### impl [PartialOrd](https://doc.rust-lang.org/1.86.0/core/cmp/trait.PartialOrd.html "trait core::cmp::PartialOrd"){.trait} for [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#impl-partialord-for-positive .code-header}
:::

::::::::::::: impl-items
::: {#method.partial_cmp-5 .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#920-922){.src
.rightside}[§](#method.partial_cmp-5){.anchor}

#### fn [partial_cmp](https://doc.rust-lang.org/1.86.0/core/cmp/trait.PartialOrd.html#tymethod.partial_cmp){.fn}(&self, other: &Self) -\> [Option](https://doc.rust-lang.org/1.86.0/core/option/enum.Option.html "enum core::option::Option"){.enum}\<[Ordering](https://doc.rust-lang.org/1.86.0/core/cmp/enum.Ordering.html "enum core::cmp::Ordering"){.enum}\> {#fn-partial_cmpself-other-self---optionordering .code-header}
:::

::: docblock
This method returns an ordering between `self` and `other` values if one
exists. [Read
more](https://doc.rust-lang.org/1.86.0/core/cmp/trait.PartialOrd.html#tymethod.partial_cmp)
:::

::: {#method.le-5 .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#924-926){.src
.rightside}[§](#method.le-5){.anchor}

#### fn [le](https://doc.rust-lang.org/1.86.0/core/cmp/trait.PartialOrd.html#method.le){.fn}(&self, other: &Self) -\> [bool](https://doc.rust-lang.org/1.86.0/std/primitive.bool.html){.primitive} {#fn-leself-other-self---bool .code-header}
:::

::: docblock
Tests less than or equal to (for `self` and `other`) and is used by the
`<=` operator. [Read
more](https://doc.rust-lang.org/1.86.0/core/cmp/trait.PartialOrd.html#method.le)
:::

::: {#method.ge-5 .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#928-930){.src
.rightside}[§](#method.ge-5){.anchor}

#### fn [ge](https://doc.rust-lang.org/1.86.0/core/cmp/trait.PartialOrd.html#method.ge){.fn}(&self, other: &Self) -\> [bool](https://doc.rust-lang.org/1.86.0/std/primitive.bool.html){.primitive} {#fn-geself-other-self---bool .code-header}
:::

::: docblock
Tests greater than or equal to (for `self` and `other`) and is used by
the `>=` operator. [Read
more](https://doc.rust-lang.org/1.86.0/core/cmp/trait.PartialOrd.html#method.ge)
:::

::: {#method.lt-5 .section .method .trait-impl}
[[1.0.0]{.since title="Stable since Rust version 1.0.0"} ·
[Source](https://doc.rust-lang.org/1.86.0/src/core/cmp.rs.html#1371){.src}]{.rightside}[§](#method.lt-5){.anchor}

#### fn [lt](https://doc.rust-lang.org/1.86.0/core/cmp/trait.PartialOrd.html#method.lt){.fn}(&self, other: [&Rhs](https://doc.rust-lang.org/1.86.0/std/primitive.reference.html){.primitive}) -\> [bool](https://doc.rust-lang.org/1.86.0/std/primitive.bool.html){.primitive} {#fn-ltself-other-rhs---bool-5 .code-header}
:::

::: docblock
Tests less than (for `self` and `other`) and is used by the `<`
operator. [Read
more](https://doc.rust-lang.org/1.86.0/core/cmp/trait.PartialOrd.html#method.lt)
:::

::: {#method.gt-5 .section .method .trait-impl}
[[1.0.0]{.since title="Stable since Rust version 1.0.0"} ·
[Source](https://doc.rust-lang.org/1.86.0/src/core/cmp.rs.html#1407){.src}]{.rightside}[§](#method.gt-5){.anchor}

#### fn [gt](https://doc.rust-lang.org/1.86.0/core/cmp/trait.PartialOrd.html#method.gt){.fn}(&self, other: [&Rhs](https://doc.rust-lang.org/1.86.0/std/primitive.reference.html){.primitive}) -\> [bool](https://doc.rust-lang.org/1.86.0/std/primitive.bool.html){.primitive} {#fn-gtself-other-rhs---bool-5 .code-header}
:::

::: docblock
Tests greater than (for `self` and `other`) and is used by the `>`
operator. [Read
more](https://doc.rust-lang.org/1.86.0/core/cmp/trait.PartialOrd.html#method.gt)
:::
:::::::::::::

::: {#impl-RelativeEq-for-Positive .section .impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#983-1002){.src
.rightside}[§](#impl-RelativeEq-for-Positive){.anchor}

### impl RelativeEq for [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#impl-relativeeq-for-positive .code-header}
:::

::::::::: impl-items
::: {#method.default_max_relative .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#984-986){.src
.rightside}[§](#method.default_max_relative){.anchor}

#### fn [default_max_relative]{.fn}() -\> Self::Epsilon {#fn-default_max_relative---selfepsilon .code-header}
:::

::: docblock
The default relative tolerance for testing values that are far-apart.
Read more
:::

::: {#method.relative_eq .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#988-1001){.src
.rightside}[§](#method.relative_eq){.anchor}

#### fn [relative_eq]{.fn}( &self, other: &Self, epsilon: Self::Epsilon, max_relative: Self::Epsilon, ) -\> [bool](https://doc.rust-lang.org/1.86.0/std/primitive.bool.html){.primitive} {#fn-relative_eq-self-other-self-epsilon-selfepsilon-max_relative-selfepsilon---bool .code-header}
:::

::: docblock
A test for equality that uses a relative comparison if the values are
far apart.
:::

::: {#method.relative_ne .section .method .trait-impl}
[§](#method.relative_ne){.anchor}

#### fn [relative_ne]{.fn}( &self, other: [&Rhs](https://doc.rust-lang.org/1.86.0/std/primitive.reference.html){.primitive}, epsilon: Self::Epsilon, max_relative: Self::Epsilon, ) -\> [bool](https://doc.rust-lang.org/1.86.0/std/primitive.bool.html){.primitive} {#fn-relative_ne-self-other-rhs-epsilon-selfepsilon-max_relative-selfepsilon---bool .code-header}
:::

::: docblock
The inverse of \[`RelativeEq::relative_eq`\].
:::
:::::::::

::: {#impl-Serialize-for-Positive .section .impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#722-748){.src
.rightside}[§](#impl-Serialize-for-Positive){.anchor}

### impl [Serialize](https://docs.rs/serde/1.0.219/serde/ser/trait.Serialize.html "trait serde::ser::Serialize"){.trait} for [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#impl-serialize-for-positive .code-header}
:::

:::::: impl-items
:::: {#method.serialize .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#723-747){.src
.rightside}[§](#method.serialize){.anchor}

#### fn [serialize](https://docs.rs/serde/1.0.219/serde/ser/trait.Serialize.html#tymethod.serialize){.fn}\<S\>(&self, serializer: S) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<S::[Ok](https://docs.rs/serde/1.0.219/serde/ser/trait.Serializer.html#associatedtype.Ok "type serde::ser::Serializer::Ok"){.associatedtype}, S::[Error](https://docs.rs/serde/1.0.219/serde/ser/trait.Serializer.html#associatedtype.Error "type serde::ser::Serializer::Error"){.associatedtype}\> {#fn-serializesself-serializer-s---resultsok-serror .code-header}

::: where
where S:
[Serializer](https://docs.rs/serde/1.0.219/serde/ser/trait.Serializer.html "trait serde::ser::Serializer"){.trait},
:::
::::

::: docblock
Serialize this value into the given Serde serializer. [Read
more](https://docs.rs/serde/1.0.219/serde/ser/trait.Serialize.html#tymethod.serialize)
:::
::::::

::: {#impl-Sub%3C%26Decimal%3E-for-Positive .section .impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#871-877){.src
.rightside}[§](#impl-Sub%3C%26Decimal%3E-for-Positive){.anchor}

### impl [Sub](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.Sub.html "trait core::ops::arith::Sub"){.trait}\<&Decimal\> for [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#impl-subdecimal-for-positive .code-header}
:::

::::::: impl-items
::: {#associatedtype.Output-21 .section .associatedtype .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#872){.src
.rightside}[§](#associatedtype.Output-21){.anchor}

#### type [Output](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.Sub.html#associatedtype.Output){.associatedtype} = [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#type-output-positive-13 .code-header}
:::

::: docblock
The resulting type after applying the `-` operator.
:::

::: {#method.sub-6 .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#874-876){.src
.rightside}[§](#method.sub-6){.anchor}

#### fn [sub](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.Sub.html#tymethod.sub){.fn}(self, rhs: &Decimal) -\> Self::[Output](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.Sub.html#associatedtype.Output "type core::ops::arith::Sub::Output"){.associatedtype} {#fn-subself-rhs-decimal---selfoutput .code-header}
:::

::: docblock
Performs the `-` operation. [Read
more](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.Sub.html#tymethod.sub)
:::
:::::::

::: {#impl-Sub%3C%26Positive%3E-for-Decimal .section .impl}
[Source](../../../src/optionstratlib/model/decimal.rs.html#161-167){.src
.rightside}[§](#impl-Sub%3C%26Positive%3E-for-Decimal){.anchor}

### impl [Sub](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.Sub.html "trait core::ops::arith::Sub"){.trait}\<&[Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\> for Decimal {#impl-subpositive-for-decimal .code-header}
:::

::::::: impl-items
::: {#associatedtype.Output-3 .section .associatedtype .trait-impl}
[Source](../../../src/optionstratlib/model/decimal.rs.html#162){.src
.rightside}[§](#associatedtype.Output-3){.anchor}

#### type [Output](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.Sub.html#associatedtype.Output){.associatedtype} = Decimal {#type-output-decimal-4 .code-header}
:::

::: docblock
The resulting type after applying the `-` operator.
:::

::: {#method.sub-1 .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/decimal.rs.html#164-166){.src
.rightside}[§](#method.sub-1){.anchor}

#### fn [sub](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.Sub.html#tymethod.sub){.fn}(self, rhs: &[Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}) -\> Self::[Output](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.Sub.html#associatedtype.Output "type core::ops::arith::Sub::Output"){.associatedtype} {#fn-subself-rhs-positive---selfoutput .code-header}
:::

::: docblock
Performs the `-` operation. [Read
more](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.Sub.html#tymethod.sub)
:::
:::::::

::: {#impl-Sub%3CDecimal%3E-for-Positive .section .impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#863-869){.src
.rightside}[§](#impl-Sub%3CDecimal%3E-for-Positive){.anchor}

### impl [Sub](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.Sub.html "trait core::ops::arith::Sub"){.trait}\<Decimal\> for [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#impl-subdecimal-for-positive-1 .code-header}
:::

::::::: impl-items
::: {#associatedtype.Output-20 .section .associatedtype .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#864){.src
.rightside}[§](#associatedtype.Output-20){.anchor}

#### type [Output](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.Sub.html#associatedtype.Output){.associatedtype} = [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#type-output-positive-14 .code-header}
:::

::: docblock
The resulting type after applying the `-` operator.
:::

::: {#method.sub-5 .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#866-868){.src
.rightside}[§](#method.sub-5){.anchor}

#### fn [sub](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.Sub.html#tymethod.sub){.fn}(self, rhs: Decimal) -\> [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#fn-subself-rhs-decimal---positive .code-header}
:::

::: docblock
Performs the `-` operation. [Read
more](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.Sub.html#tymethod.sub)
:::
:::::::

::: {#impl-Sub%3CPositive%3E-for-Decimal .section .impl}
[Source](../../../src/optionstratlib/model/decimal.rs.html#153-159){.src
.rightside}[§](#impl-Sub%3CPositive%3E-for-Decimal){.anchor}

### impl [Sub](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.Sub.html "trait core::ops::arith::Sub"){.trait}\<[Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\> for Decimal {#impl-subpositive-for-decimal-1 .code-header}
:::

::::::: impl-items
::: {#associatedtype.Output-2 .section .associatedtype .trait-impl}
[Source](../../../src/optionstratlib/model/decimal.rs.html#154){.src
.rightside}[§](#associatedtype.Output-2){.anchor}

#### type [Output](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.Sub.html#associatedtype.Output){.associatedtype} = Decimal {#type-output-decimal-5 .code-header}
:::

::: docblock
The resulting type after applying the `-` operator.
:::

::: {#method.sub .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/decimal.rs.html#156-158){.src
.rightside}[§](#method.sub){.anchor}

#### fn [sub](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.Sub.html#tymethod.sub){.fn}(self, rhs: [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}) -\> Self::[Output](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.Sub.html#associatedtype.Output "type core::ops::arith::Sub::Output"){.associatedtype} {#fn-subself-rhs-positive---selfoutput-1 .code-header}
:::

::: docblock
Performs the `-` operation. [Read
more](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.Sub.html#tymethod.sub)
:::
:::::::

::: {#impl-Sub%3CPositive%3E-for-f64 .section .impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#551-557){.src
.rightside}[§](#impl-Sub%3CPositive%3E-for-f64){.anchor}

### impl [Sub](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.Sub.html "trait core::ops::arith::Sub"){.trait}\<[Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\> for [f64](https://doc.rust-lang.org/1.86.0/std/primitive.f64.html){.primitive} {#impl-subpositive-for-f64 .code-header}
:::

::::::: impl-items
::: {#associatedtype.Output-8 .section .associatedtype .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#552){.src
.rightside}[§](#associatedtype.Output-8){.anchor}

#### type [Output](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.Sub.html#associatedtype.Output){.associatedtype} = [f64](https://doc.rust-lang.org/1.86.0/std/primitive.f64.html){.primitive} {#type-output-f64-3 .code-header}
:::

::: docblock
The resulting type after applying the `-` operator.
:::

::: {#method.sub-2 .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#554-556){.src
.rightside}[§](#method.sub-2){.anchor}

#### fn [sub](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.Sub.html#tymethod.sub){.fn}(self, rhs: [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}) -\> Self::[Output](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.Sub.html#associatedtype.Output "type core::ops::arith::Sub::Output"){.associatedtype} {#fn-subself-rhs-positive---selfoutput-2 .code-header}
:::

::: docblock
Performs the `-` operation. [Read
more](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.Sub.html#tymethod.sub)
:::
:::::::

::: {#impl-Sub%3Cf64%3E-for-Positive .section .impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#645-651){.src
.rightside}[§](#impl-Sub%3Cf64%3E-for-Positive){.anchor}

### impl [Sub](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.Sub.html "trait core::ops::arith::Sub"){.trait}\<[f64](https://doc.rust-lang.org/1.86.0/std/primitive.f64.html){.primitive}\> for [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#impl-subf64-for-positive .code-header}
:::

::::::: impl-items
::: {#associatedtype.Output-13 .section .associatedtype .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#646){.src
.rightside}[§](#associatedtype.Output-13){.anchor}

#### type [Output](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.Sub.html#associatedtype.Output){.associatedtype} = [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#type-output-positive-15 .code-header}
:::

::: docblock
The resulting type after applying the `-` operator.
:::

::: {#method.sub-3 .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#648-650){.src
.rightside}[§](#method.sub-3){.anchor}

#### fn [sub](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.Sub.html#tymethod.sub){.fn}(self, rhs: [f64](https://doc.rust-lang.org/1.86.0/std/primitive.f64.html){.primitive}) -\> Self::[Output](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.Sub.html#associatedtype.Output "type core::ops::arith::Sub::Output"){.associatedtype} {#fn-subself-rhs-f64---selfoutput .code-header}
:::

::: docblock
Performs the `-` operation. [Read
more](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.Sub.html#tymethod.sub)
:::
:::::::

::: {#impl-Sub-for-Positive .section .impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#826-837){.src
.rightside}[§](#impl-Sub-for-Positive){.anchor}

### impl [Sub](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.Sub.html "trait core::ops::arith::Sub"){.trait} for [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#impl-sub-for-positive .code-header}
:::

::::::: impl-items
::: {#associatedtype.Output-16 .section .associatedtype .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#827){.src
.rightside}[§](#associatedtype.Output-16){.anchor}

#### type [Output](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.Sub.html#associatedtype.Output){.associatedtype} = [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#type-output-positive-16 .code-header}
:::

::: docblock
The resulting type after applying the `-` operator.
:::

::: {#method.sub-4 .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#829-836){.src
.rightside}[§](#method.sub-4){.anchor}

#### fn [sub](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.Sub.html#tymethod.sub){.fn}(self, rhs: Self) -\> Self::[Output](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.Sub.html#associatedtype.Output "type core::ops::arith::Sub::Output"){.associatedtype} {#fn-subself-rhs-self---selfoutput .code-header}
:::

::: docblock
Performs the `-` operation. [Read
more](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.Sub.html#tymethod.sub)
:::
:::::::

::: {#impl-Sum%3C%26Positive%3E-for-Positive .section .impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#1011-1016){.src
.rightside}[§](#impl-Sum%3C%26Positive%3E-for-Positive){.anchor}

### impl\<\'a\> [Sum](https://doc.rust-lang.org/1.86.0/core/iter/traits/accum/trait.Sum.html "trait core::iter::traits::accum::Sum"){.trait}\<&\'a [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\> for [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#impla-suma-positive-for-positive .code-header}
:::

::::: impl-items
::: {#method.sum-1 .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#1012-1015){.src
.rightside}[§](#method.sum-1){.anchor}

#### fn [sum](https://doc.rust-lang.org/1.86.0/core/iter/traits/accum/trait.Sum.html#tymethod.sum){.fn}\<I: [Iterator](https://doc.rust-lang.org/1.86.0/core/iter/traits/iterator/trait.Iterator.html "trait core::iter::traits::iterator::Iterator"){.trait}\<Item = &\'a [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\>\>(iter: I) -\> Self {#fn-sumi-iteratoritem-a-positiveiter-i---self .code-header}
:::

::: docblock
Takes an iterator and generates `Self` from the elements by "summing up"
the items.
:::
:::::

::: {#impl-Sum-for-Positive .section .impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#1004-1009){.src
.rightside}[§](#impl-Sum-for-Positive){.anchor}

### impl [Sum](https://doc.rust-lang.org/1.86.0/core/iter/traits/accum/trait.Sum.html "trait core::iter::traits::accum::Sum"){.trait} for [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#impl-sum-for-positive .code-header}
:::

::::: impl-items
::: {#method.sum .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#1005-1008){.src
.rightside}[§](#method.sum){.anchor}

#### fn [sum](https://doc.rust-lang.org/1.86.0/core/iter/traits/accum/trait.Sum.html#tymethod.sum){.fn}\<I: [Iterator](https://doc.rust-lang.org/1.86.0/core/iter/traits/iterator/trait.Iterator.html "trait core::iter::traits::iterator::Iterator"){.trait}\<Item = Self\>\>(iter: I) -\> Self {#fn-sumi-iteratoritem-selfiter-i---self .code-header}
:::

::: docblock
Takes an iterator and generates `Self` from the elements by "summing up"
the items.
:::
:::::

::: {#impl-ToRound-for-Positive .section .impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#471-479){.src
.rightside}[§](#impl-ToRound-for-Positive){.anchor}

### impl [ToRound](../utils/trait.ToRound.html "trait optionstratlib::model::utils::ToRound"){.trait} for [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#impl-toround-for-positive .code-header}
:::

::::::: impl-items
::: {#method.round-1 .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#472-474){.src
.rightside}[§](#method.round-1){.anchor}

#### fn [round](../utils/trait.ToRound.html#tymethod.round){.fn}(&self) -\> Decimal {#fn-roundself---decimal .code-header}
:::

::: docblock
Rounds the number to the nearest integer. [Read
more](../utils/trait.ToRound.html#tymethod.round)
:::

::: {#method.round_to-1 .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#476-478){.src
.rightside}[§](#method.round_to-1){.anchor}

#### fn [round_to](../utils/trait.ToRound.html#tymethod.round_to){.fn}(&self, decimal_places: [u32](https://doc.rust-lang.org/1.86.0/std/primitive.u32.html){.primitive}) -\> Decimal {#fn-round_toself-decimal_places-u32---decimal .code-header}
:::

::: docblock
Rounds the number to a specified number of decimal places. [Read
more](../utils/trait.ToRound.html#tymethod.round_to)
:::
:::::::

::: {#impl-Copy-for-Positive .section .impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#39){.src
.rightside}[§](#impl-Copy-for-Positive){.anchor}

### impl [Copy](https://doc.rust-lang.org/1.86.0/core/marker/trait.Copy.html "trait core::marker::Copy"){.trait} for [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#impl-copy-for-positive .code-header}
:::

::: {#impl-Eq-for-Positive .section .impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#933){.src
.rightside}[§](#impl-Eq-for-Positive){.anchor}

### impl [Eq](https://doc.rust-lang.org/1.86.0/core/cmp/trait.Eq.html "trait core::cmp::Eq"){.trait} for [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#impl-eq-for-positive .code-header}
:::

::: {#impl-StructuralPartialEq-for-Positive .section .impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#39){.src
.rightside}[§](#impl-StructuralPartialEq-for-Positive){.anchor}

### impl [StructuralPartialEq](https://doc.rust-lang.org/1.86.0/core/marker/trait.StructuralPartialEq.html "trait core::marker::StructuralPartialEq"){.trait} for [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#impl-structuralpartialeq-for-positive .code-header}
:::
:::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::

## Auto Trait Implementations[§](#synthetic-implementations){.anchor} {#synthetic-implementations .section-header}

::::::::: {#synthetic-implementations-list}
::: {#impl-Freeze-for-Positive .section .impl}
[§](#impl-Freeze-for-Positive){.anchor}

### impl [Freeze](https://doc.rust-lang.org/1.86.0/core/marker/trait.Freeze.html "trait core::marker::Freeze"){.trait} for [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#impl-freeze-for-positive .code-header}
:::

::: {#impl-RefUnwindSafe-for-Positive .section .impl}
[§](#impl-RefUnwindSafe-for-Positive){.anchor}

### impl [RefUnwindSafe](https://doc.rust-lang.org/1.86.0/core/panic/unwind_safe/trait.RefUnwindSafe.html "trait core::panic::unwind_safe::RefUnwindSafe"){.trait} for [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#impl-refunwindsafe-for-positive .code-header}
:::

::: {#impl-Send-for-Positive .section .impl}
[§](#impl-Send-for-Positive){.anchor}

### impl [Send](https://doc.rust-lang.org/1.86.0/core/marker/trait.Send.html "trait core::marker::Send"){.trait} for [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#impl-send-for-positive .code-header}
:::

::: {#impl-Sync-for-Positive .section .impl}
[§](#impl-Sync-for-Positive){.anchor}

### impl [Sync](https://doc.rust-lang.org/1.86.0/core/marker/trait.Sync.html "trait core::marker::Sync"){.trait} for [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#impl-sync-for-positive .code-header}
:::

::: {#impl-Unpin-for-Positive .section .impl}
[§](#impl-Unpin-for-Positive){.anchor}

### impl [Unpin](https://doc.rust-lang.org/1.86.0/core/marker/trait.Unpin.html "trait core::marker::Unpin"){.trait} for [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#impl-unpin-for-positive .code-header}
:::

::: {#impl-UnwindSafe-for-Positive .section .impl}
[§](#impl-UnwindSafe-for-Positive){.anchor}

### impl [UnwindSafe](https://doc.rust-lang.org/1.86.0/core/panic/unwind_safe/trait.UnwindSafe.html "trait core::panic::unwind_safe::UnwindSafe"){.trait} for [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#impl-unwindsafe-for-positive .code-header}
:::
:::::::::

## Blanket Implementations[§](#blanket-implementations){.anchor} {#blanket-implementations .section-header}

:::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::: {#blanket-implementations-list}
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

:::: {#impl-AsRelative-for-T .section .impl}
[§](#impl-AsRelative-for-T){.anchor}

### impl\<T\> AsRelative for T {#implt-asrelative-for-t .code-header}

::: where
where T:
[Into](https://doc.rust-lang.org/1.86.0/core/convert/trait.Into.html "trait core::convert::Into"){.trait}\<[f64](https://doc.rust-lang.org/1.86.0/std/primitive.f64.html){.primitive}\>,
:::
::::

::::::::: impl-items
::: {#method.percent_width .section .method .trait-impl}
[§](#method.percent_width){.anchor}

#### fn [percent_width]{.fn}(self) -\> RelativeSize {#fn-percent_widthself---relativesize .code-header}
:::

::: docblock
Make the value a relative size of percentage of width
:::

::: {#method.percent_height .section .method .trait-impl}
[§](#method.percent_height){.anchor}

#### fn [percent_height]{.fn}(self) -\> RelativeSize {#fn-percent_heightself---relativesize .code-header}
:::

::: docblock
Make the value a relative size of percentage of height
:::

::: {#method.percent .section .method .trait-impl}
[§](#method.percent){.anchor}

#### fn [percent]{.fn}(self) -\> RelativeSize {#fn-percentself---relativesize .code-header}
:::

::: docblock
Make the value a relative size of percentage of minimal of height and
width
:::
:::::::::

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

:::: {#impl-Comparable%3CK%3E-for-Q .section .impl}
[§](#impl-Comparable%3CK%3E-for-Q){.anchor}

### impl\<Q, K\> Comparable\<K\> for Q {#implq-k-comparablek-for-q .code-header}

::: where
where Q:
[Ord](https://doc.rust-lang.org/1.86.0/core/cmp/trait.Ord.html "trait core::cmp::Ord"){.trait} +
?[Sized](https://doc.rust-lang.org/1.86.0/core/marker/trait.Sized.html "trait core::marker::Sized"){.trait},
K:
[Borrow](https://doc.rust-lang.org/1.86.0/core/borrow/trait.Borrow.html "trait core::borrow::Borrow"){.trait}\<Q\> +
?[Sized](https://doc.rust-lang.org/1.86.0/core/marker/trait.Sized.html "trait core::marker::Sized"){.trait},
:::
::::

::::: impl-items
::: {#method.compare .section .method .trait-impl}
[§](#method.compare){.anchor}

#### fn [compare]{.fn}(&self, key: [&K](https://doc.rust-lang.org/1.86.0/std/primitive.reference.html){.primitive}) -\> [Ordering](https://doc.rust-lang.org/1.86.0/core/cmp/enum.Ordering.html "enum core::cmp::Ordering"){.enum} {#fn-compareself-key-k---ordering .code-header}
:::

::: docblock
Compare self to `key` and return their ordering.
:::
:::::

:::: {#impl-Equivalent%3CK%3E-for-Q .section .impl}
[§](#impl-Equivalent%3CK%3E-for-Q){.anchor}

### impl\<Q, K\> Equivalent\<K\> for Q {#implq-k-equivalentk-for-q .code-header}

::: where
where Q:
[Eq](https://doc.rust-lang.org/1.86.0/core/cmp/trait.Eq.html "trait core::cmp::Eq"){.trait} +
?[Sized](https://doc.rust-lang.org/1.86.0/core/marker/trait.Sized.html "trait core::marker::Sized"){.trait},
K:
[Borrow](https://doc.rust-lang.org/1.86.0/core/borrow/trait.Borrow.html "trait core::borrow::Borrow"){.trait}\<Q\> +
?[Sized](https://doc.rust-lang.org/1.86.0/core/marker/trait.Sized.html "trait core::marker::Sized"){.trait},
:::
::::

::::: impl-items
::: {#method.equivalent .section .method .trait-impl}
[§](#method.equivalent){.anchor}

#### fn [equivalent]{.fn}(&self, key: [&K](https://doc.rust-lang.org/1.86.0/std/primitive.reference.html){.primitive}) -\> [bool](https://doc.rust-lang.org/1.86.0/std/primitive.bool.html){.primitive} {#fn-equivalentself-key-k---bool .code-header}
:::

::: docblock
Checks if this value is equivalent to the given key. Read more
:::
:::::

:::: {#impl-Equivalent%3CK%3E-for-Q-1 .section .impl}
[§](#impl-Equivalent%3CK%3E-for-Q-1){.anchor}

### impl\<Q, K\> Equivalent\<K\> for Q {#implq-k-equivalentk-for-q-1 .code-header}

::: where
where Q:
[Eq](https://doc.rust-lang.org/1.86.0/core/cmp/trait.Eq.html "trait core::cmp::Eq"){.trait} +
?[Sized](https://doc.rust-lang.org/1.86.0/core/marker/trait.Sized.html "trait core::marker::Sized"){.trait},
K:
[Borrow](https://doc.rust-lang.org/1.86.0/core/borrow/trait.Borrow.html "trait core::borrow::Borrow"){.trait}\<Q\> +
?[Sized](https://doc.rust-lang.org/1.86.0/core/marker/trait.Sized.html "trait core::marker::Sized"){.trait},
:::
::::

::::: impl-items
::: {#method.equivalent-1 .section .method .trait-impl}
[§](#method.equivalent-1){.anchor}

#### fn [equivalent]{.fn}(&self, key: [&K](https://doc.rust-lang.org/1.86.0/std/primitive.reference.html){.primitive}) -\> [bool](https://doc.rust-lang.org/1.86.0/std/primitive.bool.html){.primitive} {#fn-equivalentself-key-k---bool-1 .code-header}
:::

::: docblock
Compare self to `key` and return `true` if they are equal.
:::
:::::

::: {#impl-From%3CT%3E-for-T .section .impl}
[Source](https://doc.rust-lang.org/1.86.0/src/core/convert/mod.rs.html#767){.src
.rightside}[§](#impl-From%3CT%3E-for-T){.anchor}

### impl\<T\> [From](https://doc.rust-lang.org/1.86.0/core/convert/trait.From.html "trait core::convert::From"){.trait}\<T\> for T {#implt-fromt-for-t .code-header}
:::

::::: impl-items
::: {#method.from-13 .section .method .trait-impl}
[Source](https://doc.rust-lang.org/1.86.0/src/core/convert/mod.rs.html#770){.src
.rightside}[§](#method.from-13){.anchor}

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
::: {#associatedtype.Output-27 .section .associatedtype .trait-impl}
[Source](https://docs.rs/typenum/1.18.0/src/typenum/type_operators.rs.html#35){.src
.rightside}[§](#associatedtype.Output-27){.anchor}

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

:::: {#impl-ToString-for-T .section .impl}
[Source](https://doc.rust-lang.org/1.86.0/src/alloc/string.rs.html#2758){.src
.rightside}[§](#impl-ToString-for-T){.anchor}

### impl\<T\> [ToString](https://doc.rust-lang.org/1.86.0/alloc/string/trait.ToString.html "trait alloc::string::ToString"){.trait} for T {#implt-tostring-for-t .code-header}

::: where
where T:
[Display](https://doc.rust-lang.org/1.86.0/core/fmt/trait.Display.html "trait core::fmt::Display"){.trait} +
?[Sized](https://doc.rust-lang.org/1.86.0/core/marker/trait.Sized.html "trait core::marker::Sized"){.trait},
:::
::::

::::: impl-items
::: {#method.to_string .section .method .trait-impl}
[Source](https://doc.rust-lang.org/1.86.0/src/alloc/string.rs.html#2760){.src
.rightside}[§](#method.to_string){.anchor}

#### fn [to_string](https://doc.rust-lang.org/1.86.0/alloc/string/trait.ToString.html#tymethod.to_string){.fn}(&self) -\> [String](https://doc.rust-lang.org/1.86.0/alloc/string/struct.String.html "struct alloc::string::String"){.struct} {#fn-to_stringself---string .code-header}
:::

::: docblock
Converts the given value to a `String`. [Read
more](https://doc.rust-lang.org/1.86.0/alloc/string/trait.ToString.html#tymethod.to_string)
:::
:::::

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

:::: {#impl-ClosedAdd%3CRight%3E-for-T .section .impl}
[§](#impl-ClosedAdd%3CRight%3E-for-T){.anchor}

### impl\<T, Right\> ClosedAdd\<Right\> for T {#implt-right-closedaddright-for-t .code-header}

::: where
where T:
[Add](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.Add.html "trait core::ops::arith::Add"){.trait}\<Right,
Output = T\> +
[AddAssign](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.AddAssign.html "trait core::ops::arith::AddAssign"){.trait}\<Right\>,
:::
::::

:::: {#impl-ClosedAddAssign%3CRight%3E-for-T .section .impl}
[§](#impl-ClosedAddAssign%3CRight%3E-for-T){.anchor}

### impl\<T, Right\> ClosedAddAssign\<Right\> for T {#implt-right-closedaddassignright-for-t .code-header}

::: where
where T: ClosedAdd\<Right\> +
[AddAssign](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.AddAssign.html "trait core::ops::arith::AddAssign"){.trait}\<Right\>,
:::
::::

:::: {#impl-ClosedMul%3CRight%3E-for-T .section .impl}
[§](#impl-ClosedMul%3CRight%3E-for-T){.anchor}

### impl\<T, Right\> ClosedMul\<Right\> for T {#implt-right-closedmulright-for-t .code-header}

::: where
where T:
[Mul](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.Mul.html "trait core::ops::arith::Mul"){.trait}\<Right,
Output = T\> +
[MulAssign](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.MulAssign.html "trait core::ops::arith::MulAssign"){.trait}\<Right\>,
:::
::::

:::: {#impl-ClosedMulAssign%3CRight%3E-for-T .section .impl}
[§](#impl-ClosedMulAssign%3CRight%3E-for-T){.anchor}

### impl\<T, Right\> ClosedMulAssign\<Right\> for T {#implt-right-closedmulassignright-for-t .code-header}

::: where
where T: ClosedMul\<Right\> +
[MulAssign](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.MulAssign.html "trait core::ops::arith::MulAssign"){.trait}\<Right\>,
:::
::::

:::: {#impl-ClosedNeg-for-T .section .impl}
[§](#impl-ClosedNeg-for-T){.anchor}

### impl\<T\> ClosedNeg for T {#implt-closedneg-for-t .code-header}

::: where
where T:
[Neg](https://doc.rust-lang.org/1.86.0/core/ops/arith/trait.Neg.html "trait core::ops::arith::Neg"){.trait}\<Output
= T\>,
:::
::::

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
::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
:::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
