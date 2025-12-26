::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::: width-limiter
:::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../../index.html)::[model](../index.html)::[positive](index.html)
:::

# Struct [Positive]{.struct} Copy item path

[[Source](../../../src/optionstratlib/model/positive.rs.html#44){.src}
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
use positive::pos_or_panic;
let strike_price = pos!(100.0);
```
:::
::::

## Implementations[§](#implementations){.anchor} {#implementations .section-header}

:::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::: {#implementations-list}
::: {#impl-Positive .section .impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#174-716){.src
.rightside}[§](#impl-Positive){.anchor}

### impl [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#impl-positive .code-header}
:::

:::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::: impl-items
::: {#associatedconstant.ZERO .section .associatedconstant}
[Source](../../../src/optionstratlib/model/positive.rs.html#176){.src
.rightside}

#### pub const [ZERO](#associatedconstant.ZERO){.constant}: [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#pub-const-zero-positive .code-header}
:::

::: docblock
A zero value represented as a `Positive` value.
:::

::: {#associatedconstant.ONE .section .associatedconstant}
[Source](../../../src/optionstratlib/model/positive.rs.html#179){.src
.rightside}

#### pub const [ONE](#associatedconstant.ONE){.constant}: [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#pub-const-one-positive .code-header}
:::

::: docblock
A value of one represented as a `Positive` value.
:::

::: {#associatedconstant.TWO .section .associatedconstant}
[Source](../../../src/optionstratlib/model/positive.rs.html#182){.src
.rightside}

#### pub const [TWO](#associatedconstant.TWO){.constant}: [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#pub-const-two-positive .code-header}
:::

::: docblock
A value of two represented as a `Positive` value.
:::

::: {#associatedconstant.INFINITY .section .associatedconstant}
[Source](../../../src/optionstratlib/model/positive.rs.html#185){.src
.rightside}

#### pub const [INFINITY](#associatedconstant.INFINITY){.constant}: [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#pub-const-infinity-positive .code-header}
:::

::: docblock
Represents the maximum positive value possible (effectively infinity).
:::

::: {#associatedconstant.TEN .section .associatedconstant}
[Source](../../../src/optionstratlib/model/positive.rs.html#188){.src
.rightside}

#### pub const [TEN](#associatedconstant.TEN){.constant}: [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#pub-const-ten-positive .code-header}
:::

::: docblock
A value of ten represented as a `Positive` value.
:::

::: {#associatedconstant.HUNDRED .section .associatedconstant}
[Source](../../../src/optionstratlib/model/positive.rs.html#191){.src
.rightside}

#### pub const [HUNDRED](#associatedconstant.HUNDRED){.constant}: [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#pub-const-hundred-positive .code-header}
:::

::: docblock
A value of one hundred represented as a `Positive` value.
:::

::: {#associatedconstant.THOUSAND .section .associatedconstant}
[Source](../../../src/optionstratlib/model/positive.rs.html#194){.src
.rightside}

#### pub const [THOUSAND](#associatedconstant.THOUSAND){.constant}: [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#pub-const-thousand-positive .code-header}
:::

::: docblock
A value of one thousand represented as a `Positive` value.
:::

::: {#associatedconstant.PI .section .associatedconstant}
[Source](../../../src/optionstratlib/model/positive.rs.html#197){.src
.rightside}

#### pub const [PI](#associatedconstant.PI){.constant}: [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#pub-const-pi-positive .code-header}
:::

::: docblock
The mathematical constant π (pi) represented as a `Positive` value.
:::

::: {#method.new .section .method}
[Source](../../../src/optionstratlib/model/positive.rs.html#210-225){.src
.rightside}

#### pub fn [new](#method.new){.fn}(value: [f64](https://doc.rust-lang.org/1.91.1/std/primitive.f64.html){.primitive}) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<Self, [DecimalError](../../error/decimal/enum.DecimalError.html "enum optionstratlib::error::decimal::DecimalError"){.enum}\> {#pub-fn-newvalue-f64---resultself-decimalerror .code-header}
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
[Source](../../../src/optionstratlib/model/positive.rs.html#237-247){.src
.rightside}

#### pub fn [new_decimal](#method.new_decimal){.fn}(value: [Decimal](../../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<Self, [DecimalError](../../error/decimal/enum.DecimalError.html "enum optionstratlib::error::decimal::DecimalError"){.enum}\> {#pub-fn-new_decimalvalue-decimal---resultself-decimalerror .code-header}
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
[Source](../../../src/optionstratlib/model/positive.rs.html#254-256){.src
.rightside}

#### pub fn [value](#method.value){.fn}(&self) -\> [Decimal](../../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct} {#pub-fn-valueself---decimal .code-header}
:::

::: docblock
Returns the inner `Decimal` value.

##### [§](#returns-2){.doc-anchor}Returns

The wrapped `Decimal` value.
:::

::: {#method.to_dec .section .method}
[Source](../../../src/optionstratlib/model/positive.rs.html#263-265){.src
.rightside}

#### pub fn [to_dec](#method.to_dec){.fn}(&self) -\> [Decimal](../../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct} {#pub-fn-to_decself---decimal .code-header}
:::

::: docblock
Returns the inner `Decimal` value (alias for `value()`).

##### [§](#returns-3){.doc-anchor}Returns

The wrapped `Decimal` value.
:::

::: {#method.to_dec_ref .section .method}
[Source](../../../src/optionstratlib/model/positive.rs.html#272-274){.src
.rightside}

#### pub fn [to_dec_ref](#method.to_dec_ref){.fn}(&self) -\> &[Decimal](../../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct} {#pub-fn-to_dec_refself---decimal .code-header}
:::

::: docblock
Returns the inner `Decimal` ref.

##### [§](#returns-4){.doc-anchor}Returns

The wrapped `Decimal` ref.
:::

::: {#method.to_dec_ref_mut .section .method}
[Source](../../../src/optionstratlib/model/positive.rs.html#292-294){.src
.rightside}

#### pub fn [to_dec_ref_mut](#method.to_dec_ref_mut){.fn}(&mut self) -\> &mut [Decimal](../../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct} {#pub-fn-to_dec_ref_mutmut-self---mut-decimal .code-header}
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
[Source](../../../src/optionstratlib/model/positive.rs.html#305-307){.src
.rightside}

#### pub fn [to_f64](#method.to_f64){.fn}(&self) -\> [f64](https://doc.rust-lang.org/1.91.1/std/primitive.f64.html){.primitive} {#pub-fn-to_f64self---f64 .code-header}
:::

::: docblock
Converts the value to a 64-bit floating-point number.

##### [§](#returns-6){.doc-anchor}Returns

The value as an `f64`.

##### [§](#panics){.doc-anchor}Panics

Panics if the conversion fails.
:::

::: {#method.to_i64 .section .method}
[Source](../../../src/optionstratlib/model/positive.rs.html#318-320){.src
.rightside}

#### pub fn [to_i64](#method.to_i64){.fn}(&self) -\> [i64](https://doc.rust-lang.org/1.91.1/std/primitive.i64.html){.primitive} {#pub-fn-to_i64self---i64 .code-header}
:::

::: docblock
Converts the value to a 64-bit signed integer.

##### [§](#returns-7){.doc-anchor}Returns

The value as an `i64`.

##### [§](#panics-1){.doc-anchor}Panics

Panics if the value cannot be represented as an `i64`.
:::

::: {#method.to_u64 .section .method}
[Source](../../../src/optionstratlib/model/positive.rs.html#334-336){.src
.rightside}

#### pub fn [to_u64](#method.to_u64){.fn}(&self) -\> [u64](https://doc.rust-lang.org/1.91.1/std/primitive.u64.html){.primitive} {#pub-fn-to_u64self---u64 .code-header}
:::

::: docblock
Converts the inner value of the struct to a `u64`.

This method assumes the inner value can be safely converted to a `u64`
and uses `unwrap()` to extract the value. If the conversion fails, this
will cause a panic at runtime.

##### [§](#returns-8){.doc-anchor}Returns

A `u64` representation of the inner value.

##### [§](#panics-2){.doc-anchor}Panics

This method will panic if the inner value cannot be converted to a
`u64`.
:::

::: {#method.to_usize .section .method}
[Source](../../../src/optionstratlib/model/positive.rs.html#347-349){.src
.rightside}

#### pub fn [to_usize](#method.to_usize){.fn}(&self) -\> [usize](https://doc.rust-lang.org/1.91.1/std/primitive.usize.html){.primitive} {#pub-fn-to_usizeself---usize .code-header}
:::

::: docblock
Converts the value to a usize signed integer.

##### [§](#returns-9){.doc-anchor}Returns

The value as an `usize`.

##### [§](#panics-3){.doc-anchor}Panics

Panics if the value cannot be represented as an `usize`.
:::

::: {#method.max .section .method}
[Source](../../../src/optionstratlib/model/positive.rs.html#360-362){.src
.rightside}

#### pub fn [max](#method.max){.fn}(self, other: [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}) -\> [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#pub-fn-maxself-other-positive---positive .code-header}
:::

::: docblock
Returns the maximum of two `Positive` values.

##### [§](#arguments-2){.doc-anchor}Arguments

- `other` - Another `Positive` value to compare with

##### [§](#returns-10){.doc-anchor}Returns

The larger of the two values.
:::

::: {#method.min .section .method}
[Source](../../../src/optionstratlib/model/positive.rs.html#373-375){.src
.rightside}

#### pub fn [min](#method.min){.fn}(self, other: [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}) -\> [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#pub-fn-minself-other-positive---positive .code-header}
:::

::: docblock
Returns the minimum of two `Positive` values.

##### [§](#arguments-3){.doc-anchor}Arguments

- `other` - Another `Positive` value to compare with

##### [§](#returns-11){.doc-anchor}Returns

The smaller of the two values.
:::

::: {#method.floor .section .method}
[Source](../../../src/optionstratlib/model/positive.rs.html#382-384){.src
.rightside}

#### pub fn [floor](#method.floor){.fn}(&self) -\> [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#pub-fn-floorself---positive .code-header}
:::

::: docblock
Rounds the value down to the nearest integer.

##### [§](#returns-12){.doc-anchor}Returns

A new `Positive` value rounded down to the nearest integer.
:::

::: {#method.powi .section .method}
[Source](../../../src/optionstratlib/model/positive.rs.html#395-397){.src
.rightside}

#### pub fn [powi](#method.powi){.fn}(&self, n: [i64](https://doc.rust-lang.org/1.91.1/std/primitive.i64.html){.primitive}) -\> [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#pub-fn-powiself-n-i64---positive .code-header}
:::

::: docblock
Raises this value to an integer power.

##### [§](#arguments-4){.doc-anchor}Arguments

- `n` - The power to raise this value to

##### [§](#returns-13){.doc-anchor}Returns

A new `Positive` value representing `self` raised to the power `n`.
:::

::: {#method.pow .section .method}
[Source](../../../src/optionstratlib/model/positive.rs.html#412-414){.src
.rightside}

#### pub fn [pow](#method.pow){.fn}(&self, n: [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}) -\> [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#pub-fn-powself-n-positive---positive .code-header}
:::

::: docblock
Computes the result of raising the current `Positive` value to the power
of the given `Positive` exponent.

##### [§](#parameters){.doc-anchor}Parameters

- `n`: A `Positive` value representing the exponent to which the current
  value will be raised.

##### [§](#returns-14){.doc-anchor}Returns

A `Positive` value representing the result of the power computation.

##### [§](#panics-4){.doc-anchor}Panics

This function does not panic as all inputs are guaranteed to be positive
by the `Positive` type.
:::

::: {#method.powu .section .method}
[Source](../../../src/optionstratlib/model/positive.rs.html#424-426){.src
.rightside}

#### pub fn [powu](#method.powu){.fn}(&self, n: [u64](https://doc.rust-lang.org/1.91.1/std/primitive.u64.html){.primitive}) -\> [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#pub-fn-powuself-n-u64---positive .code-header}
:::

::: docblock
Raises the current `Positive` value to the power of `n` using unsigned
integer exponentiation.

##### [§](#parameters-1){.doc-anchor}Parameters

- `n`: An unsigned 64-bit integer (`u64`) representing the power to
  which the value will be raised.

##### [§](#returns-15){.doc-anchor}Returns

Returns a new `Positive` instance containing the result of `self` raised
to the power of `n`.
:::

::: {#method.powd .section .method}
[Source](../../../src/optionstratlib/model/positive.rs.html#439-441){.src
.rightside}

#### pub fn [powd](#method.powd){.fn}(&self, p0: [Decimal](../../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}) -\> [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#pub-fn-powdself-p0-decimal---positive .code-header}
:::

::: docblock
Raises this value to a decimal power.

This is a crate-internal method not exposed to public API users.

##### [§](#arguments-5){.doc-anchor}Arguments

- `p0` - The power to raise this value to as a `Decimal`

##### [§](#returns-16){.doc-anchor}Returns

A new `Positive` value representing `self` raised to the power `p0`.
:::

::: {#method.round .section .method}
[Source](../../../src/optionstratlib/model/positive.rs.html#448-450){.src
.rightside}

#### pub fn [round](#method.round){.fn}(&self) -\> [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#pub-fn-roundself---positive .code-header}
:::

::: docblock
Rounds the value to the nearest integer.

##### [§](#returns-17){.doc-anchor}Returns

A new `Positive` value rounded to the nearest integer.
:::

::: {#method.round_to_nice_number .section .method}
[Source](../../../src/optionstratlib/model/positive.rs.html#490-504){.src
.rightside}

#### pub fn [round_to_nice_number](#method.round_to_nice_number){.fn}(&self) -\> [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#pub-fn-round_to_nice_numberself---positive .code-header}
:::

:::: docblock
Rounds the current value to a "nice" number, based on its magnitude.

This method computes the logarithmic magnitude of the current value and
determines a simplified number within a range of predefined "nice"
numbers, such as 1, 2, 5, or 10, scaled by a power of ten. This is often
useful for simplifying numerical values for display or plotting.

##### [§](#procedure){.doc-anchor}Procedure

1.  Compute the magnitude of the current value by taking the base-10
    logarithm and flooring it.
2.  Calculate the power of ten corresponding to the magnitude.
3.  Normalize the current value by dividing it by the calculated power
    of ten.
4.  Determine the closest "nice" number:
    - Values less than 1.5 round to 1.
    - Values less than 3 round to 2.
    - Values less than 7 round to 5.
    - All other values round to 10.
5.  Scale the "nice" number back up by the appropriate power of ten.

##### [§](#returns-18){.doc-anchor}Returns

A `Positive` value representing the rounded "nice" number, scaled
appropriately according to the magnitude of the input value.

##### [§](#panics-5){.doc-anchor}Panics

This function does not explicitly handle negative numbers or zero, as it
is assumed that `self` is a positive numeric value. Ensure `self` is
valid and non-zero prior to calling this method.

##### [§](#examples){.doc-anchor}Examples

::: example-wrap
``` {.rust .rust-example-rendered}
use positive::Positive;
let value = Positive::new(123.0).unwrap();
let rounded = value.round_to_nice_number();
assert_eq!(rounded, Positive::new(100.0).unwrap());

let value = Positive::new(6.7).unwrap();
let rounded = value.round_to_nice_number();
assert_eq!(rounded, Positive::new(5.0).unwrap());
```
:::
::::

::: {#method.sqrt .section .method}
[Source](../../../src/optionstratlib/model/positive.rs.html#515-517){.src
.rightside}

#### pub fn [sqrt](#method.sqrt){.fn}(&self) -\> [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#pub-fn-sqrtself---positive .code-header}
:::

::: docblock
Calculates the square root of the value.

##### [§](#returns-19){.doc-anchor}Returns

A new `Positive` value representing the square root.

##### [§](#panics-6){.doc-anchor}Panics

Panics if the square root calculation fails.
:::

::: {#method.ln .section .method}
[Source](../../../src/optionstratlib/model/positive.rs.html#524-526){.src
.rightside}

#### pub fn [ln](#method.ln){.fn}(&self) -\> [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#pub-fn-lnself---positive .code-header}
:::

::: docblock
Calculates the natural logarithm of the value.

##### [§](#returns-20){.doc-anchor}Returns

A new `Positive` value representing the natural logarithm.
:::

::: {#method.round_to .section .method}
[Source](../../../src/optionstratlib/model/positive.rs.html#537-539){.src
.rightside}

#### pub fn [round_to](#method.round_to){.fn}(&self, decimal_places: [u32](https://doc.rust-lang.org/1.91.1/std/primitive.u32.html){.primitive}) -\> [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#pub-fn-round_toself-decimal_places-u32---positive .code-header}
:::

::: docblock
Rounds the value to a specified number of decimal places.

##### [§](#arguments-6){.doc-anchor}Arguments

- `decimal_places` - The number of decimal places to round to

##### [§](#returns-21){.doc-anchor}Returns

A new `Positive` value rounded to the specified decimal places.
:::

::: {#method.format_fixed_places .section .method}
[Source](../../../src/optionstratlib/model/positive.rs.html#568-574){.src
.rightside}

#### pub fn [format_fixed_places](#method.format_fixed_places){.fn}(&self, decimal_places: [u32](https://doc.rust-lang.org/1.91.1/std/primitive.u32.html){.primitive}) -\> [String](https://doc.rust-lang.org/1.91.1/alloc/string/struct.String.html "struct alloc::string::String"){.struct} {#pub-fn-format_fixed_placesself-decimal_places-u32---string .code-header}
:::

:::: docblock
Formats the value with a fixed number of decimal places, filling with
zeros if needed.

Unlike `round_to` which just rounds the value, this method ensures the
string representation always has exactly the specified number of decimal
places.

##### [§](#arguments-7){.doc-anchor}Arguments

- `decimal_places` - The exact number of decimal places to display

##### [§](#returns-22){.doc-anchor}Returns

A String representation of the value with exactly the specified number
of decimal places.

##### [§](#examples-1){.doc-anchor}Examples

::: example-wrap
``` {.rust .rust-example-rendered}
use positive::pos_or_panic;

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
[Source](../../../src/optionstratlib/model/positive.rs.html#581-583){.src
.rightside}

#### pub fn [exp](#method.exp){.fn}(&self) -\> [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#pub-fn-expself---positive .code-header}
:::

::: docblock
Calculates the exponential function e\^x for this value.

##### [§](#returns-23){.doc-anchor}Returns

A new `Positive` value representing e raised to the power of `self`.
:::

::: {#method.clamp .section .method}
[Source](../../../src/optionstratlib/model/positive.rs.html#597-605){.src
.rightside}

#### pub fn [clamp](#method.clamp){.fn}(&self, min: [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, max: [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}) -\> [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#pub-fn-clampself-min-positive-max-positive---positive .code-header}
:::

::: docblock
Clamps the value between a minimum and maximum.

##### [§](#arguments-8){.doc-anchor}Arguments

- `min` - The lower bound
- `max` - The upper bound

##### [§](#returns-24){.doc-anchor}Returns

- `min` if `self` is less than `min`
- `max` if `self` is greater than `max`
- `self` if `self` is between `min` and `max` (inclusive)
:::

::: {#method.is_zero .section .method}
[Source](../../../src/optionstratlib/model/positive.rs.html#612-614){.src
.rightside}

#### pub fn [is_zero](#method.is_zero){.fn}(&self) -\> [bool](https://doc.rust-lang.org/1.91.1/std/primitive.bool.html){.primitive} {#pub-fn-is_zeroself---bool .code-header}
:::

::: docblock
Checks if the value is exactly zero.

##### [§](#returns-25){.doc-anchor}Returns

`true` if the value is zero, `false` otherwise.
:::

::: {#method.ceiling .section .method}
[Source](../../../src/optionstratlib/model/positive.rs.html#617-623){.src
.rightside}

#### pub fn [ceiling](#method.ceiling){.fn}(&self) -\> [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#pub-fn-ceilingself---positive .code-header}
:::

::: docblock
Returns the smallest integer greater than or equal to the value.
:::

::: {#method.log10 .section .method}
[Source](../../../src/optionstratlib/model/positive.rs.html#635-637){.src
.rightside}

#### pub fn [log10](#method.log10){.fn}(&self) -\> [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#pub-fn-log10self---positive .code-header}
:::

::: docblock
Computes the base-10 logarithm of the value contained in the `Positive`
instance.

##### [§](#returns-26){.doc-anchor}Returns

A new `Positive` instance containing the result of the base-10 logarithm
of the original value.

##### [§](#note){.doc-anchor}Note

It is assumed that the value contained in the `Positive` instance is
always greater than 0, as logarithms of non-positive numbers are
undefined.
:::

::: {#method.sub_or_zero .section .method}
[Source](../../../src/optionstratlib/model/positive.rs.html#646-652){.src
.rightside}

#### pub fn [sub_or_zero](#method.sub_or_zero){.fn}(&self, other: &[Decimal](../../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}) -\> [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#pub-fn-sub_or_zeroself-other-decimal---positive .code-header}
:::

::: docblock
Subtracts a decimal value from this positive value, returning zero if
the result would be negative.

##### [§](#arguments-9){.doc-anchor}Arguments

- `other` - The decimal value to subtract.

##### [§](#returns-27){.doc-anchor}Returns

- `Positive` - The result of the subtraction, or zero if the result
  would be negative.
:::

::: {#method.sub_or_none .section .method}
[Source](../../../src/optionstratlib/model/positive.rs.html#661-667){.src
.rightside}

#### pub fn [sub_or_none](#method.sub_or_none){.fn}(&self, other: &[Decimal](../../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}) -\> [Option](https://doc.rust-lang.org/1.91.1/core/option/enum.Option.html "enum core::option::Option"){.enum}\<[Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\> {#pub-fn-sub_or_noneself-other-decimal---optionpositive .code-header}
:::

::: docblock
Subtracts a decimal value from this positive value, returning None if
the result would be negative.

##### [§](#arguments-10){.doc-anchor}Arguments

- `other` - The decimal value to subtract.

##### [§](#returns-28){.doc-anchor}Returns

- `Option<Positive>` - The result of the subtraction as a
  `Some(Positive)`, or `None` if the result would be negative.
:::

::: {#method.is_multiple .section .method}
[Source](../../../src/optionstratlib/model/positive.rs.html#706-715){.src
.rightside}

#### pub fn [is_multiple](#method.is_multiple){.fn}(&self, other: [f64](https://doc.rust-lang.org/1.91.1/std/primitive.f64.html){.primitive}) -\> [bool](https://doc.rust-lang.org/1.91.1/std/primitive.bool.html){.primitive} {#pub-fn-is_multipleself-other-f64---bool .code-header}
:::

:::: docblock
Checks whether the numeric value of the current instance is a multiple
of another specified value.

This method determines if the numeric value represented by the current
instance is evenly divisible by the provided `other` value, using a
small epsilon to handle floating-point precision.

##### [§](#arguments-11){.doc-anchor}Arguments

- `other` - A 64-bit floating-point value (`f64`) representing the
  divisor against which the current instance will be tested.

##### [§](#returns-29){.doc-anchor}Returns

- `true` if the current instance is a multiple of the specified `other`
  value within a small tolerance.
- `false` if the current instance is not a multiple of the specified
  `other` value or if the current value is not finite (e.g., if it is
  `NaN`, infinity, or similar).

##### [§](#behavior){.doc-anchor}Behavior

- The method first retrieves the numeric value of the current instance
  as an `f64` using a `to_f64()` method, which is assumed to be
  implemented elsewhere in the code.
- If the resulting value is not finite (e.g., `NaN` or infinity),
  `false` is returned.
- Otherwise, the modulo operation is used to check whether the
  remainder, when divided by `other`, is approximately zero (considering
  the tolerance defined by `f64::EPSILON`).

##### [§](#notes){.doc-anchor}Notes

- `f64::EPSILON` represents the smallest difference between two distinct
  `f64` values. It is used here to account for floating-point rounding
  errors when performing equality comparisons.

##### [§](#examples-2){.doc-anchor}Examples

::: example-wrap
``` {.rust .rust-example-rendered}
use positive::pos_or_panic;
let num = pos!(10.0);
assert!(num.is_multiple(2.0));  // 10.0 is multiple of 2.0
assert!(!num.is_multiple(3.0)); // 10.0 is not a multiple of 3.0
```
:::
::::
::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::

## Trait Implementations[§](#trait-implementations){.anchor} {#trait-implementations .section-header}

:::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::: {#trait-implementations-list}
::: {#impl-AbsDiffEq-for-Positive .section .impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#1237-1247){.src
.rightside}[§](#impl-AbsDiffEq-for-Positive){.anchor}

### impl [AbsDiffEq](../../../approx/abs_diff_eq/trait.AbsDiffEq.html "trait approx::abs_diff_eq::AbsDiffEq"){.trait} for [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#impl-absdiffeq-for-positive .code-header}
:::

::::::::::: impl-items
::: {#associatedtype.Epsilon .section .associatedtype .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#1238){.src
.rightside}[§](#associatedtype.Epsilon){.anchor}

#### type [Epsilon](../../../approx/abs_diff_eq/trait.AbsDiffEq.html#associatedtype.Epsilon){.associatedtype} = [Decimal](../../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct} {#type-epsilon-decimal .code-header}
:::

::: docblock
Used for specifying relative comparisons.
:::

::: {#method.default_epsilon .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#1240-1242){.src
.rightside}[§](#method.default_epsilon){.anchor}

#### fn [default_epsilon](../../../approx/abs_diff_eq/trait.AbsDiffEq.html#tymethod.default_epsilon){.fn}() -\> Self::[Epsilon](../../../approx/abs_diff_eq/trait.AbsDiffEq.html#associatedtype.Epsilon "type approx::abs_diff_eq::AbsDiffEq::Epsilon"){.associatedtype} {#fn-default_epsilon---selfepsilon .code-header}
:::

::: docblock
The default tolerance to use when testing values that are close
together. [Read
more](../../../approx/abs_diff_eq/trait.AbsDiffEq.html#tymethod.default_epsilon)
:::

::: {#method.abs_diff_eq .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#1244-1246){.src
.rightside}[§](#method.abs_diff_eq){.anchor}

#### fn [abs_diff_eq](../../../approx/abs_diff_eq/trait.AbsDiffEq.html#tymethod.abs_diff_eq){.fn}(&self, other: &Self, epsilon: Self::[Epsilon](../../../approx/abs_diff_eq/trait.AbsDiffEq.html#associatedtype.Epsilon "type approx::abs_diff_eq::AbsDiffEq::Epsilon"){.associatedtype}) -\> [bool](https://doc.rust-lang.org/1.91.1/std/primitive.bool.html){.primitive} {#fn-abs_diff_eqself-other-self-epsilon-selfepsilon---bool .code-header}
:::

::: docblock
A test for equality that uses the absolute difference to compute the
approximate equality of two numbers.
:::

::: {#method.abs_diff_ne .section .method .trait-impl}
[Source](../../../src/approx/abs_diff_eq.rs.html#24){.src
.rightside}[§](#method.abs_diff_ne){.anchor}

#### fn [abs_diff_ne](../../../approx/abs_diff_eq/trait.AbsDiffEq.html#method.abs_diff_ne){.fn}(&self, other: [&Rhs](https://doc.rust-lang.org/1.91.1/std/primitive.reference.html){.primitive}, epsilon: Self::[Epsilon](../../../approx/abs_diff_eq/trait.AbsDiffEq.html#associatedtype.Epsilon "type approx::abs_diff_eq::AbsDiffEq::Epsilon"){.associatedtype}) -\> [bool](https://doc.rust-lang.org/1.91.1/std/primitive.bool.html){.primitive} {#fn-abs_diff_neself-other-rhs-epsilon-selfepsilon---bool .code-header}
:::

::: docblock
The inverse of
[`AbsDiffEq::abs_diff_eq`](../../../approx/abs_diff_eq/trait.AbsDiffEq.html#tymethod.abs_diff_eq "method approx::abs_diff_eq::AbsDiffEq::abs_diff_eq").
:::
:::::::::::

::: {#impl-Add%3C%26Decimal%3E-for-Positive .section .impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#1121-1127){.src
.rightside}[§](#impl-Add%3C%26Decimal%3E-for-Positive){.anchor}

### impl [Add](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.Add.html "trait core::ops::arith::Add"){.trait}\<&[Decimal](../../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}\> for [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#impl-adddecimal-for-positive .code-header}
:::

::::::: impl-items
::: {#associatedtype.Output-20 .section .associatedtype .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#1122){.src
.rightside}[§](#associatedtype.Output-20){.anchor}

#### type [Output](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.Add.html#associatedtype.Output){.associatedtype} = [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#type-output-positive .code-header}
:::

::: docblock
The resulting type after applying the `+` operator.
:::

::: {#method.add-6 .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#1124-1126){.src
.rightside}[§](#method.add-6){.anchor}

#### fn [add](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.Add.html#tymethod.add){.fn}(self, rhs: &[Decimal](../../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}) -\> Self::[Output](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.Add.html#associatedtype.Output "type core::ops::arith::Add::Output"){.associatedtype} {#fn-addself-rhs-decimal---selfoutput .code-header}
:::

::: docblock
Performs the `+` operation. [Read
more](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.Add.html#tymethod.add)
:::
:::::::

::: {#impl-Add%3C%26Positive%3E-for-Decimal .section .impl}
[Source](../../../src/optionstratlib/model/decimal.rs.html#177-183){.src
.rightside}[§](#impl-Add%3C%26Positive%3E-for-Decimal){.anchor}

### impl [Add](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.Add.html "trait core::ops::arith::Add"){.trait}\<&[Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\> for [Decimal](../../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct} {#impl-addpositive-for-decimal .code-header}
:::

::::::: impl-items
::: {#associatedtype.Output-5 .section .associatedtype .trait-impl}
[Source](../../../src/optionstratlib/model/decimal.rs.html#178){.src
.rightside}[§](#associatedtype.Output-5){.anchor}

#### type [Output](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.Add.html#associatedtype.Output){.associatedtype} = [Decimal](../../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct} {#type-output-decimal .code-header}
:::

::: docblock
The resulting type after applying the `+` operator.
:::

::: {#method.add-1 .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/decimal.rs.html#180-182){.src
.rightside}[§](#method.add-1){.anchor}

#### fn [add](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.Add.html#tymethod.add){.fn}(self, rhs: &[Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}) -\> [Decimal](../../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct} {#fn-addself-rhs-positive---decimal .code-header}
:::

::: docblock
Performs the `+` operation. [Read
more](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.Add.html#tymethod.add)
:::
:::::::

::: {#impl-Add%3CDecimal%3E-for-Positive .section .impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#1113-1119){.src
.rightside}[§](#impl-Add%3CDecimal%3E-for-Positive){.anchor}

### impl [Add](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.Add.html "trait core::ops::arith::Add"){.trait}\<[Decimal](../../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}\> for [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#impl-adddecimal-for-positive-1 .code-header}
:::

::::::: impl-items
::: {#associatedtype.Output-19 .section .associatedtype .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#1114){.src
.rightside}[§](#associatedtype.Output-19){.anchor}

#### type [Output](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.Add.html#associatedtype.Output){.associatedtype} = [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#type-output-positive-1 .code-header}
:::

::: docblock
The resulting type after applying the `+` operator.
:::

::: {#method.add-5 .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#1116-1118){.src
.rightside}[§](#method.add-5){.anchor}

#### fn [add](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.Add.html#tymethod.add){.fn}(self, rhs: [Decimal](../../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}) -\> [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#fn-addself-rhs-decimal---positive .code-header}
:::

::: docblock
Performs the `+` operation. [Read
more](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.Add.html#tymethod.add)
:::
:::::::

::: {#impl-Add%3CPositive%3E-for-Decimal .section .impl}
[Source](../../../src/optionstratlib/model/decimal.rs.html#169-175){.src
.rightside}[§](#impl-Add%3CPositive%3E-for-Decimal){.anchor}

### impl [Add](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.Add.html "trait core::ops::arith::Add"){.trait}\<[Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\> for [Decimal](../../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct} {#impl-addpositive-for-decimal-1 .code-header}
:::

::::::: impl-items
::: {#associatedtype.Output-4 .section .associatedtype .trait-impl}
[Source](../../../src/optionstratlib/model/decimal.rs.html#170){.src
.rightside}[§](#associatedtype.Output-4){.anchor}

#### type [Output](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.Add.html#associatedtype.Output){.associatedtype} = [Decimal](../../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct} {#type-output-decimal-1 .code-header}
:::

::: docblock
The resulting type after applying the `+` operator.
:::

::: {#method.add .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/decimal.rs.html#172-174){.src
.rightside}[§](#method.add){.anchor}

#### fn [add](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.Add.html#tymethod.add){.fn}(self, rhs: [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}) -\> Self::[Output](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.Add.html#associatedtype.Output "type core::ops::arith::Add::Output"){.associatedtype} {#fn-addself-rhs-positive---selfoutput .code-header}
:::

::: docblock
Performs the `+` operation. [Read
more](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.Add.html#tymethod.add)
:::
:::::::

::: {#impl-Add%3CPositive%3E-for-f64 .section .impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#806-812){.src
.rightside}[§](#impl-Add%3CPositive%3E-for-f64){.anchor}

### impl [Add](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.Add.html "trait core::ops::arith::Add"){.trait}\<[Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\> for [f64](https://doc.rust-lang.org/1.91.1/std/primitive.f64.html){.primitive} {#impl-addpositive-for-f64 .code-header}
:::

::::::: impl-items
::: {#associatedtype.Output-9 .section .associatedtype .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#807){.src
.rightside}[§](#associatedtype.Output-9){.anchor}

#### type [Output](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.Add.html#associatedtype.Output){.associatedtype} = [f64](https://doc.rust-lang.org/1.91.1/std/primitive.f64.html){.primitive} {#type-output-f64 .code-header}
:::

::: docblock
The resulting type after applying the `+` operator.
:::

::: {#method.add-2 .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#809-811){.src
.rightside}[§](#method.add-2){.anchor}

#### fn [add](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.Add.html#tymethod.add){.fn}(self, rhs: [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}) -\> Self::[Output](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.Add.html#associatedtype.Output "type core::ops::arith::Add::Output"){.associatedtype} {#fn-addself-rhs-positive---selfoutput-1 .code-header}
:::

::: docblock
Performs the `+` operation. [Read
more](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.Add.html#tymethod.add)
:::
:::::::

::: {#impl-Add%3Cf64%3E-for-Positive .section .impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#912-918){.src
.rightside}[§](#impl-Add%3Cf64%3E-for-Positive){.anchor}

### impl [Add](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.Add.html "trait core::ops::arith::Add"){.trait}\<[f64](https://doc.rust-lang.org/1.91.1/std/primitive.f64.html){.primitive}\> for [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#impl-addf64-for-positive .code-header}
:::

::::::: impl-items
::: {#associatedtype.Output-14 .section .associatedtype .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#913){.src
.rightside}[§](#associatedtype.Output-14){.anchor}

#### type [Output](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.Add.html#associatedtype.Output){.associatedtype} = [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#type-output-positive-2 .code-header}
:::

::: docblock
The resulting type after applying the `+` operator.
:::

::: {#method.add-3 .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#915-917){.src
.rightside}[§](#method.add-3){.anchor}

#### fn [add](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.Add.html#tymethod.add){.fn}(self, rhs: [f64](https://doc.rust-lang.org/1.91.1/std/primitive.f64.html){.primitive}) -\> Self::[Output](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.Add.html#associatedtype.Output "type core::ops::arith::Add::Output"){.associatedtype} {#fn-addself-rhs-f64---selfoutput .code-header}
:::

::: docblock
Performs the `+` operation. [Read
more](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.Add.html#tymethod.add)
:::
:::::::

::: {#impl-Add-for-Positive .section .impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#1076-1082){.src
.rightside}[§](#impl-Add-for-Positive){.anchor}

### impl [Add](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.Add.html "trait core::ops::arith::Add"){.trait} for [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#impl-add-for-positive .code-header}
:::

::::::: impl-items
::: {#associatedtype.Output-15 .section .associatedtype .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#1077){.src
.rightside}[§](#associatedtype.Output-15){.anchor}

#### type [Output](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.Add.html#associatedtype.Output){.associatedtype} = [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#type-output-positive-3 .code-header}
:::

::: docblock
The resulting type after applying the `+` operator.
:::

::: {#method.add-4 .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#1079-1081){.src
.rightside}[§](#method.add-4){.anchor}

#### fn [add](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.Add.html#tymethod.add){.fn}(self, other: [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}) -\> [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#fn-addself-other-positive---positive .code-header}
:::

::: docblock
Performs the `+` operation. [Read
more](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.Add.html#tymethod.add)
:::
:::::::

::: {#impl-AddAssign%3C%26Positive%3E-for-Decimal .section .impl}
[Source](../../../src/optionstratlib/model/decimal.rs.html#191-195){.src
.rightside}[§](#impl-AddAssign%3C%26Positive%3E-for-Decimal){.anchor}

### impl [AddAssign](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.AddAssign.html "trait core::ops::arith::AddAssign"){.trait}\<&[Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\> for [Decimal](../../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct} {#impl-addassignpositive-for-decimal .code-header}
:::

::::: impl-items
::: {#method.add_assign-1 .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/decimal.rs.html#192-194){.src
.rightside}[§](#method.add_assign-1){.anchor}

#### fn [add_assign](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.AddAssign.html#tymethod.add_assign){.fn}(&mut self, rhs: &[Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}) {#fn-add_assignmut-self-rhs-positive .code-header}
:::

::: docblock
Performs the `+=` operation. [Read
more](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.AddAssign.html#tymethod.add_assign)
:::
:::::

::: {#impl-AddAssign%3CDecimal%3E-for-Positive .section .impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#1151-1155){.src
.rightside}[§](#impl-AddAssign%3CDecimal%3E-for-Positive){.anchor}

### impl [AddAssign](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.AddAssign.html "trait core::ops::arith::AddAssign"){.trait}\<[Decimal](../../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}\> for [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#impl-addassigndecimal-for-positive .code-header}
:::

::::: impl-items
::: {#method.add_assign-3 .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#1152-1154){.src
.rightside}[§](#method.add_assign-3){.anchor}

#### fn [add_assign](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.AddAssign.html#tymethod.add_assign){.fn}(&mut self, rhs: [Decimal](../../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}) {#fn-add_assignmut-self-rhs-decimal .code-header}
:::

::: docblock
Performs the `+=` operation. [Read
more](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.AddAssign.html#tymethod.add_assign)
:::
:::::

::: {#impl-AddAssign%3CPositive%3E-for-Decimal .section .impl}
[Source](../../../src/optionstratlib/model/decimal.rs.html#185-189){.src
.rightside}[§](#impl-AddAssign%3CPositive%3E-for-Decimal){.anchor}

### impl [AddAssign](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.AddAssign.html "trait core::ops::arith::AddAssign"){.trait}\<[Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\> for [Decimal](../../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct} {#impl-addassignpositive-for-decimal-1 .code-header}
:::

::::: impl-items
::: {#method.add_assign .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/decimal.rs.html#186-188){.src
.rightside}[§](#method.add_assign){.anchor}

#### fn [add_assign](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.AddAssign.html#tymethod.add_assign){.fn}(&mut self, rhs: [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}) {#fn-add_assignmut-self-rhs-positive-1 .code-header}
:::

::: docblock
Performs the `+=` operation. [Read
more](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.AddAssign.html#tymethod.add_assign)
:::
:::::

::: {#impl-AddAssign-for-Positive .section .impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#1145-1149){.src
.rightside}[§](#impl-AddAssign-for-Positive){.anchor}

### impl [AddAssign](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.AddAssign.html "trait core::ops::arith::AddAssign"){.trait} for [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#impl-addassign-for-positive .code-header}
:::

::::: impl-items
::: {#method.add_assign-2 .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#1146-1148){.src
.rightside}[§](#method.add_assign-2){.anchor}

#### fn [add_assign](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.AddAssign.html#tymethod.add_assign){.fn}(&mut self, other: [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}) {#fn-add_assignmut-self-other-positive .code-header}
:::

::: docblock
Performs the `+=` operation. [Read
more](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.AddAssign.html#tymethod.add_assign)
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

#### fn [atm_iv](../../volatility/trait.AtmIvProvider.html#tymethod.atm_iv){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<&[Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}, [VolatilityError](../../error/enum.VolatilityError.html "enum optionstratlib::error::VolatilityError"){.enum}\> {#fn-atm_ivself---resultpositive-volatilityerror .code-header}
:::

::: docblock
Get the at-the-money implied volatility [Read
more](../../volatility/trait.AtmIvProvider.html#tymethod.atm_iv)
:::
:::::

::: {#impl-Clone-for-Positive .section .impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#43){.src
.rightside}[§](#impl-Clone-for-Positive){.anchor}

### impl [Clone](https://doc.rust-lang.org/1.91.1/core/clone/trait.Clone.html "trait core::clone::Clone"){.trait} for [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#impl-clone-for-positive .code-header}
:::

::::::: impl-items
::: {#method.clone .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#43){.src
.rightside}[§](#method.clone){.anchor}

#### fn [clone](https://doc.rust-lang.org/1.91.1/core/clone/trait.Clone.html#tymethod.clone){.fn}(&self) -\> [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#fn-cloneself---positive .code-header}
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

::: {#impl-ComposeSchema-for-Positive .section .impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#43){.src
.rightside}[§](#impl-ComposeSchema-for-Positive){.anchor}

### impl ComposeSchema for [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#impl-composeschema-for-positive .code-header}
:::

:::: impl-items
::: {#method.compose .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#43){.src
.rightside}[§](#method.compose){.anchor}

#### fn [compose](#tymethod.compose){.fn}(generics: [Vec](https://doc.rust-lang.org/1.91.1/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<[RefOr](../../../utoipa/openapi/enum.RefOr.html "enum utoipa::openapi::RefOr"){.enum}\<[Schema](../../../utoipa/openapi/schema/enum.Schema.html "enum utoipa::openapi::schema::Schema"){.enum}\>\>) -\> [RefOr](../../../utoipa/openapi/enum.RefOr.html "enum utoipa::openapi::RefOr"){.enum}\<[Schema](../../../utoipa/openapi/schema/enum.Schema.html "enum utoipa::openapi::schema::Schema"){.enum}\> {#fn-composegenerics-vecreforschema---reforschema .code-header}
:::
::::

::: {#impl-Debug-for-Positive .section .impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#963-973){.src
.rightside}[§](#impl-Debug-for-Positive){.anchor}

### impl [Debug](https://doc.rust-lang.org/1.91.1/core/fmt/trait.Debug.html "trait core::fmt::Debug"){.trait} for [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#impl-debug-for-positive .code-header}
:::

::::: impl-items
::: {#method.fmt-1 .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#964-972){.src
.rightside}[§](#method.fmt-1){.anchor}

#### fn [fmt](https://doc.rust-lang.org/1.91.1/core/fmt/trait.Debug.html#tymethod.fmt){.fn}(&self, f: &mut [Formatter](https://doc.rust-lang.org/1.91.1/core/fmt/struct.Formatter.html "struct core::fmt::Formatter"){.struct}\<\'\_\>) -\> [Result](https://doc.rust-lang.org/1.91.1/core/fmt/type.Result.html "type core::fmt::Result"){.type} {#fn-fmtself-f-mut-formatter_---result .code-header}
:::

::: docblock
Formats the value using the given formatter. [Read
more](https://doc.rust-lang.org/1.91.1/core/fmt/trait.Debug.html#tymethod.fmt)
:::
:::::

::: {#impl-Default-for-Positive .section .impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#1231-1235){.src
.rightside}[§](#impl-Default-for-Positive){.anchor}

### impl [Default](https://doc.rust-lang.org/1.91.1/core/default/trait.Default.html "trait core::default::Default"){.trait} for [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#impl-default-for-positive .code-header}
:::

::::: impl-items
::: {#method.default .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#1232-1234){.src
.rightside}[§](#method.default){.anchor}

#### fn [default](https://doc.rust-lang.org/1.91.1/core/default/trait.Default.html#tymethod.default){.fn}() -\> Self {#fn-default---self .code-header}
:::

::: docblock
Returns the "default value" for a type. [Read
more](https://doc.rust-lang.org/1.91.1/core/default/trait.Default.html#tymethod.default)
:::
:::::

::: {#impl-Deserialize%3C'de%3E-for-Positive .section .impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#1009-1074){.src
.rightside}[§](#impl-Deserialize%3C'de%3E-for-Positive){.anchor}

### impl\<\'de\> [Deserialize](../../../serde_core/de/trait.Deserialize.html "trait serde_core::de::Deserialize"){.trait}\<\'de\> for [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#implde-deserializede-for-positive .code-header}
:::

:::::: impl-items
:::: {#method.deserialize .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#1010-1073){.src
.rightside}[§](#method.deserialize){.anchor}

#### fn [deserialize](../../../serde_core/de/trait.Deserialize.html#tymethod.deserialize){.fn}\<D\>(deserializer: D) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<Self, D::[Error](../../../serde_core/de/trait.Deserializer.html#associatedtype.Error "type serde_core::de::Deserializer::Error"){.associatedtype}\> {#fn-deserializeddeserializer-d---resultself-derror .code-header}

::: where
where D:
[Deserializer](../../../serde_core/de/trait.Deserializer.html "trait serde_core::de::Deserializer"){.trait}\<\'de\>,
:::
::::

::: docblock
Deserialize this value from the given Serde deserializer. [Read
more](../../../serde_core/de/trait.Deserialize.html#tymethod.deserialize)
:::
::::::

::: {#impl-Display-for-Positive .section .impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#944-961){.src
.rightside}[§](#impl-Display-for-Positive){.anchor}

### impl [Display](https://doc.rust-lang.org/1.91.1/core/fmt/trait.Display.html "trait core::fmt::Display"){.trait} for [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#impl-display-for-positive .code-header}
:::

::::: impl-items
::: {#method.fmt .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#945-960){.src
.rightside}[§](#method.fmt){.anchor}

#### fn [fmt](https://doc.rust-lang.org/1.91.1/core/fmt/trait.Display.html#tymethod.fmt){.fn}(&self, f: &mut [Formatter](https://doc.rust-lang.org/1.91.1/core/fmt/struct.Formatter.html "struct core::fmt::Formatter"){.struct}\<\'\_\>) -\> [Result](https://doc.rust-lang.org/1.91.1/core/fmt/type.Result.html "type core::fmt::Result"){.type} {#fn-fmtself-f-mut-formatter_---result-1 .code-header}
:::

::: docblock
Formats the value using the given formatter. [Read
more](https://doc.rust-lang.org/1.91.1/core/fmt/trait.Display.html#tymethod.fmt)
:::
:::::

::: {#impl-Div%3C%26Decimal%3E-for-Positive .section .impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#1171-1177){.src
.rightside}[§](#impl-Div%3C%26Decimal%3E-for-Positive){.anchor}

### impl [Div](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.Div.html "trait core::ops::arith::Div"){.trait}\<&[Decimal](../../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}\> for [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#impl-divdecimal-for-positive .code-header}
:::

::::::: impl-items
::: {#associatedtype.Output-24 .section .associatedtype .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#1172){.src
.rightside}[§](#associatedtype.Output-24){.anchor}

#### type [Output](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.Div.html#associatedtype.Output){.associatedtype} = [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#type-output-positive-4 .code-header}
:::

::: docblock
The resulting type after applying the `/` operator.
:::

::: {#method.div-7 .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#1174-1176){.src
.rightside}[§](#method.div-7){.anchor}

#### fn [div](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.Div.html#tymethod.div){.fn}(self, rhs: &[Decimal](../../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}) -\> Self::[Output](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.Div.html#associatedtype.Output "type core::ops::arith::Div::Output"){.associatedtype} {#fn-divself-rhs-decimal---selfoutput .code-header}
:::

::: docblock
Performs the `/` operation. [Read
more](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.Div.html#tymethod.div)
:::
:::::::

::: {#impl-Div%3CDecimal%3E-for-Positive .section .impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#1163-1169){.src
.rightside}[§](#impl-Div%3CDecimal%3E-for-Positive){.anchor}

### impl [Div](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.Div.html "trait core::ops::arith::Div"){.trait}\<[Decimal](../../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}\> for [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#impl-divdecimal-for-positive-1 .code-header}
:::

::::::: impl-items
::: {#associatedtype.Output-23 .section .associatedtype .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#1164){.src
.rightside}[§](#associatedtype.Output-23){.anchor}

#### type [Output](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.Div.html#associatedtype.Output){.associatedtype} = [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#type-output-positive-5 .code-header}
:::

::: docblock
The resulting type after applying the `/` operator.
:::

::: {#method.div-6 .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#1166-1168){.src
.rightside}[§](#method.div-6){.anchor}

#### fn [div](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.Div.html#tymethod.div){.fn}(self, rhs: [Decimal](../../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}) -\> [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#fn-divself-rhs-decimal---positive .code-header}
:::

::: docblock
Performs the `/` operation. [Read
more](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.Div.html#tymethod.div)
:::
:::::::

::: {#impl-Div%3CPositive%3E-for-Decimal .section .impl}
[Source](../../../src/optionstratlib/model/decimal.rs.html#145-151){.src
.rightside}[§](#impl-Div%3CPositive%3E-for-Decimal){.anchor}

### impl [Div](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.Div.html "trait core::ops::arith::Div"){.trait}\<[Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\> for [Decimal](../../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct} {#impl-divpositive-for-decimal .code-header}
:::

::::::: impl-items
::: {#associatedtype.Output-1 .section .associatedtype .trait-impl}
[Source](../../../src/optionstratlib/model/decimal.rs.html#146){.src
.rightside}[§](#associatedtype.Output-1){.anchor}

#### type [Output](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.Div.html#associatedtype.Output){.associatedtype} = [Decimal](../../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct} {#type-output-decimal-2 .code-header}
:::

::: docblock
The resulting type after applying the `/` operator.
:::

::: {#method.div .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/decimal.rs.html#148-150){.src
.rightside}[§](#method.div){.anchor}

#### fn [div](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.Div.html#tymethod.div){.fn}(self, rhs: [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}) -\> [Decimal](../../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct} {#fn-divself-rhs-positive---decimal .code-header}
:::

::: docblock
Performs the `/` operation. [Read
more](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.Div.html#tymethod.div)
:::
:::::::

::: {#impl-Div%3CPositive%3E-for-f64 .section .impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#790-796){.src
.rightside}[§](#impl-Div%3CPositive%3E-for-f64){.anchor}

### impl [Div](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.Div.html "trait core::ops::arith::Div"){.trait}\<[Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\> for [f64](https://doc.rust-lang.org/1.91.1/std/primitive.f64.html){.primitive} {#impl-divpositive-for-f64 .code-header}
:::

::::::: impl-items
::: {#associatedtype.Output-7 .section .associatedtype .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#791){.src
.rightside}[§](#associatedtype.Output-7){.anchor}

#### type [Output](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.Div.html#associatedtype.Output){.associatedtype} = [f64](https://doc.rust-lang.org/1.91.1/std/primitive.f64.html){.primitive} {#type-output-f64-1 .code-header}
:::

::: docblock
The resulting type after applying the `/` operator.
:::

::: {#method.div-1 .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#793-795){.src
.rightside}[§](#method.div-1){.anchor}

#### fn [div](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.Div.html#tymethod.div){.fn}(self, rhs: [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}) -\> Self::[Output](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.Div.html#associatedtype.Output "type core::ops::arith::Div::Output"){.associatedtype} {#fn-divself-rhs-positive---selfoutput .code-header}
:::

::: docblock
Performs the `/` operation. [Read
more](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.Div.html#tymethod.div)
:::
:::::::

::: {#impl-Div%3Cf64%3E-for-%26Positive .section .impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#896-902){.src
.rightside}[§](#impl-Div%3Cf64%3E-for-%26Positive){.anchor}

### impl [Div](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.Div.html "trait core::ops::arith::Div"){.trait}\<[f64](https://doc.rust-lang.org/1.91.1/std/primitive.f64.html){.primitive}\> for &[Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#impl-divf64-for-positive .code-header}
:::

::::::: impl-items
::: {#associatedtype.Output-12 .section .associatedtype .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#897){.src
.rightside}[§](#associatedtype.Output-12){.anchor}

#### type [Output](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.Div.html#associatedtype.Output){.associatedtype} = [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#type-output-positive-6 .code-header}
:::

::: docblock
The resulting type after applying the `/` operator.
:::

::: {#method.div-3 .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#899-901){.src
.rightside}[§](#method.div-3){.anchor}

#### fn [div](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.Div.html#tymethod.div){.fn}(self, rhs: [f64](https://doc.rust-lang.org/1.91.1/std/primitive.f64.html){.primitive}) -\> [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#fn-divself-rhs-f64---positive .code-header}
:::

::: docblock
Performs the `/` operation. [Read
more](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.Div.html#tymethod.div)
:::
:::::::

::: {#impl-Div%3Cf64%3E-for-Positive .section .impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#888-894){.src
.rightside}[§](#impl-Div%3Cf64%3E-for-Positive){.anchor}

### impl [Div](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.Div.html "trait core::ops::arith::Div"){.trait}\<[f64](https://doc.rust-lang.org/1.91.1/std/primitive.f64.html){.primitive}\> for [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#impl-divf64-for-positive-1 .code-header}
:::

::::::: impl-items
::: {#associatedtype.Output-11 .section .associatedtype .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#889){.src
.rightside}[§](#associatedtype.Output-11){.anchor}

#### type [Output](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.Div.html#associatedtype.Output){.associatedtype} = [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#type-output-positive-7 .code-header}
:::

::: docblock
The resulting type after applying the `/` operator.
:::

::: {#method.div-2 .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#891-893){.src
.rightside}[§](#method.div-2){.anchor}

#### fn [div](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.Div.html#tymethod.div){.fn}(self, rhs: [f64](https://doc.rust-lang.org/1.91.1/std/primitive.f64.html){.primitive}) -\> [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#fn-divself-rhs-f64---positive-1 .code-header}
:::

::: docblock
Performs the `/` operation. [Read
more](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.Div.html#tymethod.div)
:::
:::::::

::: {#impl-Div-for-%26Positive .section .impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#1105-1111){.src
.rightside}[§](#impl-Div-for-%26Positive){.anchor}

### impl [Div](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.Div.html "trait core::ops::arith::Div"){.trait} for &[Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#impl-div-for-positive .code-header}
:::

::::::: impl-items
::: {#associatedtype.Output-18 .section .associatedtype .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#1106){.src
.rightside}[§](#associatedtype.Output-18){.anchor}

#### type [Output](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.Div.html#associatedtype.Output){.associatedtype} = [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#type-output-positive-8 .code-header}
:::

::: docblock
The resulting type after applying the `/` operator.
:::

::: {#method.div-5 .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#1108-1110){.src
.rightside}[§](#method.div-5){.anchor}

#### fn [div](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.Div.html#tymethod.div){.fn}(self, other: &[Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}) -\> Self::[Output](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.Div.html#associatedtype.Output "type core::ops::arith::Div::Output"){.associatedtype} {#fn-divself-other-positive---selfoutput .code-header}
:::

::: docblock
Performs the `/` operation. [Read
more](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.Div.html#tymethod.div)
:::
:::::::

::: {#impl-Div-for-Positive .section .impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#1097-1103){.src
.rightside}[§](#impl-Div-for-Positive){.anchor}

### impl [Div](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.Div.html "trait core::ops::arith::Div"){.trait} for [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#impl-div-for-positive-1 .code-header}
:::

::::::: impl-items
::: {#associatedtype.Output-17 .section .associatedtype .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#1098){.src
.rightside}[§](#associatedtype.Output-17){.anchor}

#### type [Output](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.Div.html#associatedtype.Output){.associatedtype} = [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#type-output-positive-9 .code-header}
:::

::: docblock
The resulting type after applying the `/` operator.
:::

::: {#method.div-4 .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#1100-1102){.src
.rightside}[§](#method.div-4){.anchor}

#### fn [div](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.Div.html#tymethod.div){.fn}(self, other: [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}) -\> Self::[Output](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.Div.html#associatedtype.Output "type core::ops::arith::Div::Output"){.associatedtype} {#fn-divself-other-positive---selfoutput-1 .code-header}
:::

::: docblock
Performs the `/` operation. [Read
more](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.Div.html#tymethod.div)
:::
:::::::

::: {#impl-From%3C%26Decimal%3E-for-Positive .section .impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#844-848){.src
.rightside}[§](#impl-From%3C%26Decimal%3E-for-Positive){.anchor}

### impl [From](https://doc.rust-lang.org/1.91.1/core/convert/trait.From.html "trait core::convert::From"){.trait}\<&[Decimal](../../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}\> for [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#impl-fromdecimal-for-positive .code-header}
:::

::::: impl-items
::: {#method.from-9 .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#845-847){.src
.rightside}[§](#method.from-9){.anchor}

#### fn [from](https://doc.rust-lang.org/1.91.1/core/convert/trait.From.html#tymethod.from){.fn}(value: &[Decimal](../../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}) -\> Self {#fn-fromvalue-decimal---self .code-header}
:::

::: docblock
Converts to this type from the input type.
:::
:::::

::: {#impl-From%3C%26OptionChain%3E-for-Positive .section .impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#856-860){.src
.rightside}[§](#impl-From%3C%26OptionChain%3E-for-Positive){.anchor}

### impl [From](https://doc.rust-lang.org/1.91.1/core/convert/trait.From.html "trait core::convert::From"){.trait}\<&[OptionChain](../../chains/chain/struct.OptionChain.html "struct optionstratlib::chains::chain::OptionChain"){.struct}\> for [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#impl-fromoptionchain-for-positive .code-header}
:::

::::: impl-items
::: {#method.from-11 .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#857-859){.src
.rightside}[§](#method.from-11){.anchor}

#### fn [from](https://doc.rust-lang.org/1.91.1/core/convert/trait.From.html#tymethod.from){.fn}(value: &[OptionChain](../../chains/chain/struct.OptionChain.html "struct optionstratlib::chains::chain::OptionChain"){.struct}) -\> Self {#fn-fromvalue-optionchain---self .code-header}
:::

::: docblock
Converts to this type from the input type.
:::
:::::

::: {#impl-From%3C%26OptionSeries%3E-for-Positive .section .impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#868-872){.src
.rightside}[§](#impl-From%3C%26OptionSeries%3E-for-Positive){.anchor}

### impl [From](https://doc.rust-lang.org/1.91.1/core/convert/trait.From.html "trait core::convert::From"){.trait}\<&[OptionSeries](../../series/struct.OptionSeries.html "struct optionstratlib::series::OptionSeries"){.struct}\> for [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#impl-fromoptionseries-for-positive .code-header}
:::

::::: impl-items
::: {#method.from-13 .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#869-871){.src
.rightside}[§](#method.from-13){.anchor}

#### fn [from](https://doc.rust-lang.org/1.91.1/core/convert/trait.From.html#tymethod.from){.fn}(value: &[OptionSeries](../../series/struct.OptionSeries.html "struct optionstratlib::series::OptionSeries"){.struct}) -\> Self {#fn-fromvalue-optionseries---self .code-header}
:::

::: docblock
Converts to this type from the input type.
:::
:::::

::: {#impl-From%3C%26Positive%3E-for-Decimal .section .impl}
[Source](../../../src/optionstratlib/model/decimal.rs.html#112-116){.src
.rightside}[§](#impl-From%3C%26Positive%3E-for-Decimal){.anchor}

### impl [From](https://doc.rust-lang.org/1.91.1/core/convert/trait.From.html "trait core::convert::From"){.trait}\<&[Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\> for [Decimal](../../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct} {#impl-frompositive-for-decimal .code-header}
:::

::::: impl-items
::: {#method.from-1 .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/decimal.rs.html#113-115){.src
.rightside}[§](#method.from-1){.anchor}

#### fn [from](https://doc.rust-lang.org/1.91.1/core/convert/trait.From.html#tymethod.from){.fn}(pos: &[Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}) -\> Self {#fn-frompos-positive---self .code-header}
:::

::: docblock
Converts to this type from the input type.
:::
:::::

::: {#impl-From%3C%26Positive%3E-for-Positive .section .impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#850-854){.src
.rightside}[§](#impl-From%3C%26Positive%3E-for-Positive){.anchor}

### impl [From](https://doc.rust-lang.org/1.91.1/core/convert/trait.From.html "trait core::convert::From"){.trait}\<&[Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\> for [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#impl-frompositive-for-positive .code-header}
:::

::::: impl-items
::: {#method.from-10 .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#851-853){.src
.rightside}[§](#method.from-10){.anchor}

#### fn [from](https://doc.rust-lang.org/1.91.1/core/convert/trait.From.html#tymethod.from){.fn}(value: &[Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}) -\> Self {#fn-fromvalue-positive---self .code-header}
:::

::: docblock
Converts to this type from the input type.
:::
:::::

::: {#impl-From%3C%26Positive%3E-for-f64 .section .impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#740-744){.src
.rightside}[§](#impl-From%3C%26Positive%3E-for-f64){.anchor}

### impl [From](https://doc.rust-lang.org/1.91.1/core/convert/trait.From.html "trait core::convert::From"){.trait}\<&[Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\> for [f64](https://doc.rust-lang.org/1.91.1/std/primitive.f64.html){.primitive} {#impl-frompositive-for-f64 .code-header}
:::

::::: impl-items
::: {#method.from-3 .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#741-743){.src
.rightside}[§](#method.from-3){.anchor}

#### fn [from](https://doc.rust-lang.org/1.91.1/core/convert/trait.From.html#tymethod.from){.fn}(value: &[Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}) -\> Self {#fn-fromvalue-positive---self-1 .code-header}
:::

::: docblock
Converts to this type from the input type.
:::
:::::

::: {#impl-From%3CDecimal%3E-for-Positive .section .impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#838-842){.src
.rightside}[§](#impl-From%3CDecimal%3E-for-Positive){.anchor}

### impl [From](https://doc.rust-lang.org/1.91.1/core/convert/trait.From.html "trait core::convert::From"){.trait}\<[Decimal](../../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}\> for [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#impl-fromdecimal-for-positive-1 .code-header}
:::

::::: impl-items
::: {#method.from-8 .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#839-841){.src
.rightside}[§](#method.from-8){.anchor}

#### fn [from](https://doc.rust-lang.org/1.91.1/core/convert/trait.From.html#tymethod.from){.fn}(value: [Decimal](../../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}) -\> Self {#fn-fromvalue-decimal---self-1 .code-header}
:::

::: docblock
Converts to this type from the input type.
:::
:::::

::: {#impl-From%3COptionChain%3E-for-Positive .section .impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#862-866){.src
.rightside}[§](#impl-From%3COptionChain%3E-for-Positive){.anchor}

### impl [From](https://doc.rust-lang.org/1.91.1/core/convert/trait.From.html "trait core::convert::From"){.trait}\<[OptionChain](../../chains/chain/struct.OptionChain.html "struct optionstratlib::chains::chain::OptionChain"){.struct}\> for [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#impl-fromoptionchain-for-positive-1 .code-header}
:::

::::: impl-items
::: {#method.from-12 .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#863-865){.src
.rightside}[§](#method.from-12){.anchor}

#### fn [from](https://doc.rust-lang.org/1.91.1/core/convert/trait.From.html#tymethod.from){.fn}(value: [OptionChain](../../chains/chain/struct.OptionChain.html "struct optionstratlib::chains::chain::OptionChain"){.struct}) -\> Self {#fn-fromvalue-optionchain---self-1 .code-header}
:::

::: docblock
Converts to this type from the input type.
:::
:::::

::: {#impl-From%3COptionSeries%3E-for-Positive .section .impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#874-878){.src
.rightside}[§](#impl-From%3COptionSeries%3E-for-Positive){.anchor}

### impl [From](https://doc.rust-lang.org/1.91.1/core/convert/trait.From.html "trait core::convert::From"){.trait}\<[OptionSeries](../../series/struct.OptionSeries.html "struct optionstratlib::series::OptionSeries"){.struct}\> for [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#impl-fromoptionseries-for-positive-1 .code-header}
:::

::::: impl-items
::: {#method.from-14 .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#875-877){.src
.rightside}[§](#method.from-14){.anchor}

#### fn [from](https://doc.rust-lang.org/1.91.1/core/convert/trait.From.html#tymethod.from){.fn}(value: [OptionSeries](../../series/struct.OptionSeries.html "struct optionstratlib::series::OptionSeries"){.struct}) -\> Self {#fn-fromvalue-optionseries---self-1 .code-header}
:::

::: docblock
Converts to this type from the input type.
:::
:::::

::: {#impl-From%3CPositive%3E-for-Decimal .section .impl}
[Source](../../../src/optionstratlib/model/decimal.rs.html#106-110){.src
.rightside}[§](#impl-From%3CPositive%3E-for-Decimal){.anchor}

### impl [From](https://doc.rust-lang.org/1.91.1/core/convert/trait.From.html "trait core::convert::From"){.trait}\<[Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\> for [Decimal](../../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct} {#impl-frompositive-for-decimal-1 .code-header}
:::

::::: impl-items
::: {#method.from .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/decimal.rs.html#107-109){.src
.rightside}[§](#method.from){.anchor}

#### fn [from](https://doc.rust-lang.org/1.91.1/core/convert/trait.From.html#tymethod.from){.fn}(pos: [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}) -\> Self {#fn-frompos-positive---self-1 .code-header}
:::

::: docblock
Converts to this type from the input type.
:::
:::::

::: {#impl-From%3CPositive%3E-for-f64 .section .impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#746-750){.src
.rightside}[§](#impl-From%3CPositive%3E-for-f64){.anchor}

### impl [From](https://doc.rust-lang.org/1.91.1/core/convert/trait.From.html "trait core::convert::From"){.trait}\<[Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\> for [f64](https://doc.rust-lang.org/1.91.1/std/primitive.f64.html){.primitive} {#impl-frompositive-for-f64-1 .code-header}
:::

::::: impl-items
::: {#method.from-4 .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#747-749){.src
.rightside}[§](#method.from-4){.anchor}

#### fn [from](https://doc.rust-lang.org/1.91.1/core/convert/trait.From.html#tymethod.from){.fn}(value: [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}) -\> Self {#fn-fromvalue-positive---self-2 .code-header}
:::

::: docblock
Converts to this type from the input type.
:::
:::::

::: {#impl-From%3CPositive%3E-for-u64 .section .impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#734-738){.src
.rightside}[§](#impl-From%3CPositive%3E-for-u64){.anchor}

### impl [From](https://doc.rust-lang.org/1.91.1/core/convert/trait.From.html "trait core::convert::From"){.trait}\<[Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\> for [u64](https://doc.rust-lang.org/1.91.1/std/primitive.u64.html){.primitive} {#impl-frompositive-for-u64 .code-header}
:::

::::: impl-items
::: {#method.from-2 .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#735-737){.src
.rightside}[§](#method.from-2){.anchor}

#### fn [from](https://doc.rust-lang.org/1.91.1/core/convert/trait.From.html#tymethod.from){.fn}(pos_u64: [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}) -\> Self {#fn-frompos_u64-positive---self .code-header}
:::

::: docblock
Converts to this type from the input type.
:::
:::::

::: {#impl-From%3CPositive%3E-for-usize .section .impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#752-756){.src
.rightside}[§](#impl-From%3CPositive%3E-for-usize){.anchor}

### impl [From](https://doc.rust-lang.org/1.91.1/core/convert/trait.From.html "trait core::convert::From"){.trait}\<[Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\> for [usize](https://doc.rust-lang.org/1.91.1/std/primitive.usize.html){.primitive} {#impl-frompositive-for-usize .code-header}
:::

::::: impl-items
::: {#method.from-5 .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#753-755){.src
.rightside}[§](#method.from-5){.anchor}

#### fn [from](https://doc.rust-lang.org/1.91.1/core/convert/trait.From.html#tymethod.from){.fn}(value: [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}) -\> Self {#fn-fromvalue-positive---self-3 .code-header}
:::

::: docblock
Converts to this type from the input type.
:::
:::::

:::: {#impl-From%3CYstep%3CT%3E%3E-for-Positive .section .impl}
[Source](../../../src/optionstratlib/simulation/steps/y.rs.html#180-187){.src
.rightside}[§](#impl-From%3CYstep%3CT%3E%3E-for-Positive){.anchor}

### impl\<T\> [From](https://doc.rust-lang.org/1.91.1/core/convert/trait.From.html "trait core::convert::From"){.trait}\<[Ystep](../../simulation/steps/struct.Ystep.html "struct optionstratlib::simulation::steps::Ystep"){.struct}\<T\>\> for [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#implt-fromystept-for-positive .code-header}

::: where
where T:
[Into](https://doc.rust-lang.org/1.91.1/core/convert/trait.Into.html "trait core::convert::Into"){.trait}\<[Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\> +
[Display](https://doc.rust-lang.org/1.91.1/core/fmt/trait.Display.html "trait core::fmt::Display"){.trait} +
[Clone](https://doc.rust-lang.org/1.91.1/core/clone/trait.Clone.html "trait core::clone::Clone"){.trait},
:::
::::

::::: impl-items
::: {#method.from-15 .section .method .trait-impl}
[Source](../../../src/optionstratlib/simulation/steps/y.rs.html#184-186){.src
.rightside}[§](#method.from-15){.anchor}

#### fn [from](https://doc.rust-lang.org/1.91.1/core/convert/trait.From.html#tymethod.from){.fn}(step: [Ystep](../../simulation/steps/struct.Ystep.html "struct optionstratlib::simulation::steps::Ystep"){.struct}\<T\>) -\> Self {#fn-fromstep-ystept---self .code-header}
:::

::: docblock
Converts to this type from the input type.
:::
:::::

::: {#impl-From%3Cf64%3E-for-Positive .section .impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#826-830){.src
.rightside}[§](#impl-From%3Cf64%3E-for-Positive){.anchor}

### impl [From](https://doc.rust-lang.org/1.91.1/core/convert/trait.From.html "trait core::convert::From"){.trait}\<[f64](https://doc.rust-lang.org/1.91.1/std/primitive.f64.html){.primitive}\> for [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#impl-fromf64-for-positive .code-header}
:::

::::: impl-items
::: {#method.from-6 .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#827-829){.src
.rightside}[§](#method.from-6){.anchor}

#### fn [from](https://doc.rust-lang.org/1.91.1/core/convert/trait.From.html#tymethod.from){.fn}(value: [f64](https://doc.rust-lang.org/1.91.1/std/primitive.f64.html){.primitive}) -\> Self {#fn-fromvalue-f64---self .code-header}
:::

::: docblock
Converts to this type from the input type.
:::
:::::

::: {#impl-From%3Cusize%3E-for-Positive .section .impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#832-836){.src
.rightside}[§](#impl-From%3Cusize%3E-for-Positive){.anchor}

### impl [From](https://doc.rust-lang.org/1.91.1/core/convert/trait.From.html "trait core::convert::From"){.trait}\<[usize](https://doc.rust-lang.org/1.91.1/std/primitive.usize.html){.primitive}\> for [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#impl-fromusize-for-positive .code-header}
:::

::::: impl-items
::: {#method.from-7 .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#833-835){.src
.rightside}[§](#method.from-7){.anchor}

#### fn [from](https://doc.rust-lang.org/1.91.1/core/convert/trait.From.html#tymethod.from){.fn}(value: [usize](https://doc.rust-lang.org/1.91.1/std/primitive.usize.html){.primitive}) -\> Self {#fn-fromvalue-usize---self .code-header}
:::

::: docblock
Converts to this type from the input type.
:::
:::::

::: {#impl-FromStr-for-Positive .section .impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#814-824){.src
.rightside}[§](#impl-FromStr-for-Positive){.anchor}

### impl [FromStr](https://doc.rust-lang.org/1.91.1/core/str/traits/trait.FromStr.html "trait core::str::traits::FromStr"){.trait} for [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#impl-fromstr-for-positive .code-header}
:::

::::::: impl-items
::: {#associatedtype.Err .section .associatedtype .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#815){.src
.rightside}[§](#associatedtype.Err){.anchor}

#### type [Err](https://doc.rust-lang.org/1.91.1/core/str/traits/trait.FromStr.html#associatedtype.Err){.associatedtype} = [String](https://doc.rust-lang.org/1.91.1/alloc/string/struct.String.html "struct alloc::string::String"){.struct} {#type-err-string .code-header}
:::

::: docblock
The associated error which can be returned from parsing.
:::

::: {#method.from_str .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#817-823){.src
.rightside}[§](#method.from_str){.anchor}

#### fn [from_str](https://doc.rust-lang.org/1.91.1/core/str/traits/trait.FromStr.html#tymethod.from_str){.fn}(s: &[str](https://doc.rust-lang.org/1.91.1/std/primitive.str.html){.primitive}) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<Self, Self::[Err](https://doc.rust-lang.org/1.91.1/core/str/traits/trait.FromStr.html#associatedtype.Err "type core::str::traits::FromStr::Err"){.associatedtype}\> {#fn-from_strs-str---resultself-selferr .code-header}
:::

::: docblock
Parses a string `s` to return a value of this type. [Read
more](https://doc.rust-lang.org/1.91.1/core/str/traits/trait.FromStr.html#tymethod.from_str)
:::
:::::::

::: {#impl-Hash-for-Positive .section .impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#43){.src
.rightside}[§](#impl-Hash-for-Positive){.anchor}

### impl [Hash](https://doc.rust-lang.org/1.91.1/core/hash/trait.Hash.html "trait core::hash::Hash"){.trait} for [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#impl-hash-for-positive .code-header}
:::

:::::::: impl-items
::: {#method.hash .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#43){.src
.rightside}[§](#method.hash){.anchor}

#### fn [hash](https://doc.rust-lang.org/1.91.1/core/hash/trait.Hash.html#tymethod.hash){.fn}\<\_\_H: [Hasher](https://doc.rust-lang.org/1.91.1/core/hash/trait.Hasher.html "trait core::hash::Hasher"){.trait}\>(&self, state: [&mut \_\_H](https://doc.rust-lang.org/1.91.1/std/primitive.reference.html){.primitive}) {#fn-hash__h-hasherself-state-mut-__h .code-header}
:::

::: docblock
Feeds this value into the given
[`Hasher`](https://doc.rust-lang.org/1.91.1/core/hash/trait.Hasher.html "trait core::hash::Hasher").
[Read
more](https://doc.rust-lang.org/1.91.1/core/hash/trait.Hash.html#tymethod.hash)
:::

:::: {#method.hash_slice .section .method .trait-impl}
[[1.3.0]{.since title="Stable since Rust version 1.3.0"} ·
[Source](https://doc.rust-lang.org/1.91.1/src/core/hash/mod.rs.html#235-237){.src}]{.rightside}[§](#method.hash_slice){.anchor}

#### fn [hash_slice](https://doc.rust-lang.org/1.91.1/core/hash/trait.Hash.html#method.hash_slice){.fn}\<H\>(data: &\[Self\], state: [&mut H](https://doc.rust-lang.org/1.91.1/std/primitive.reference.html){.primitive}) {#fn-hash_slicehdata-self-state-mut-h .code-header}

::: where
where H:
[Hasher](https://doc.rust-lang.org/1.91.1/core/hash/trait.Hasher.html "trait core::hash::Hasher"){.trait},
Self:
[Sized](https://doc.rust-lang.org/1.91.1/core/marker/trait.Sized.html "trait core::marker::Sized"){.trait},
:::
::::

::: docblock
Feeds a slice of this type into the given
[`Hasher`](https://doc.rust-lang.org/1.91.1/core/hash/trait.Hasher.html "trait core::hash::Hasher").
[Read
more](https://doc.rust-lang.org/1.91.1/core/hash/trait.Hash.html#method.hash_slice)
:::
::::::::

::: {#impl-Mul%3CDecimal%3E-for-Positive .section .impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#1223-1229){.src
.rightside}[§](#impl-Mul%3CDecimal%3E-for-Positive){.anchor}

### impl [Mul](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.Mul.html "trait core::ops::arith::Mul"){.trait}\<[Decimal](../../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}\> for [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#impl-muldecimal-for-positive .code-header}
:::

::::::: impl-items
::: {#associatedtype.Output-27 .section .associatedtype .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#1224){.src
.rightside}[§](#associatedtype.Output-27){.anchor}

#### type [Output](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.Mul.html#associatedtype.Output){.associatedtype} = [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#type-output-positive-10 .code-header}
:::

::: docblock
The resulting type after applying the `*` operator.
:::

::: {#method.mul-4 .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#1226-1228){.src
.rightside}[§](#method.mul-4){.anchor}

#### fn [mul](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.Mul.html#tymethod.mul){.fn}(self, rhs: [Decimal](../../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}) -\> [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#fn-mulself-rhs-decimal---positive .code-header}
:::

::: docblock
Performs the `*` operation. [Read
more](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.Mul.html#tymethod.mul)
:::
:::::::

::: {#impl-Mul%3CPositive%3E-for-Decimal .section .impl}
[Source](../../../src/optionstratlib/model/decimal.rs.html#118-124){.src
.rightside}[§](#impl-Mul%3CPositive%3E-for-Decimal){.anchor}

### impl [Mul](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.Mul.html "trait core::ops::arith::Mul"){.trait}\<[Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\> for [Decimal](../../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct} {#impl-mulpositive-for-decimal .code-header}
:::

::::::: impl-items
::: {#associatedtype.Output .section .associatedtype .trait-impl}
[Source](../../../src/optionstratlib/model/decimal.rs.html#119){.src
.rightside}[§](#associatedtype.Output){.anchor}

#### type [Output](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.Mul.html#associatedtype.Output){.associatedtype} = [Decimal](../../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct} {#type-output-decimal-3 .code-header}
:::

::: docblock
The resulting type after applying the `*` operator.
:::

::: {#method.mul .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/decimal.rs.html#121-123){.src
.rightside}[§](#method.mul){.anchor}

#### fn [mul](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.Mul.html#tymethod.mul){.fn}(self, rhs: [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}) -\> [Decimal](../../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct} {#fn-mulself-rhs-positive---decimal .code-header}
:::

::: docblock
Performs the `*` operation. [Read
more](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.Mul.html#tymethod.mul)
:::
:::::::

::: {#impl-Mul%3CPositive%3E-for-f64 .section .impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#782-788){.src
.rightside}[§](#impl-Mul%3CPositive%3E-for-f64){.anchor}

### impl [Mul](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.Mul.html "trait core::ops::arith::Mul"){.trait}\<[Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\> for [f64](https://doc.rust-lang.org/1.91.1/std/primitive.f64.html){.primitive} {#impl-mulpositive-for-f64 .code-header}
:::

::::::: impl-items
::: {#associatedtype.Output-6 .section .associatedtype .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#783){.src
.rightside}[§](#associatedtype.Output-6){.anchor}

#### type [Output](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.Mul.html#associatedtype.Output){.associatedtype} = [f64](https://doc.rust-lang.org/1.91.1/std/primitive.f64.html){.primitive} {#type-output-f64-2 .code-header}
:::

::: docblock
The resulting type after applying the `*` operator.
:::

::: {#method.mul-1 .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#785-787){.src
.rightside}[§](#method.mul-1){.anchor}

#### fn [mul](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.Mul.html#tymethod.mul){.fn}(self, rhs: [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}) -\> Self::[Output](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.Mul.html#associatedtype.Output "type core::ops::arith::Mul::Output"){.associatedtype} {#fn-mulself-rhs-positive---selfoutput .code-header}
:::

::: docblock
Performs the `*` operation. [Read
more](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.Mul.html#tymethod.mul)
:::
:::::::

::: {#impl-Mul%3Cf64%3E-for-Positive .section .impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#880-886){.src
.rightside}[§](#impl-Mul%3Cf64%3E-for-Positive){.anchor}

### impl [Mul](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.Mul.html "trait core::ops::arith::Mul"){.trait}\<[f64](https://doc.rust-lang.org/1.91.1/std/primitive.f64.html){.primitive}\> for [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#impl-mulf64-for-positive .code-header}
:::

::::::: impl-items
::: {#associatedtype.Output-10 .section .associatedtype .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#881){.src
.rightside}[§](#associatedtype.Output-10){.anchor}

#### type [Output](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.Mul.html#associatedtype.Output){.associatedtype} = [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#type-output-positive-11 .code-header}
:::

::: docblock
The resulting type after applying the `*` operator.
:::

::: {#method.mul-2 .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#883-885){.src
.rightside}[§](#method.mul-2){.anchor}

#### fn [mul](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.Mul.html#tymethod.mul){.fn}(self, rhs: [f64](https://doc.rust-lang.org/1.91.1/std/primitive.f64.html){.primitive}) -\> [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#fn-mulself-rhs-f64---positive .code-header}
:::

::: docblock
Performs the `*` operation. [Read
more](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.Mul.html#tymethod.mul)
:::
:::::::

::: {#impl-Mul-for-Positive .section .impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#1215-1221){.src
.rightside}[§](#impl-Mul-for-Positive){.anchor}

### impl [Mul](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.Mul.html "trait core::ops::arith::Mul"){.trait} for [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#impl-mul-for-positive .code-header}
:::

::::::: impl-items
::: {#associatedtype.Output-26 .section .associatedtype .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#1216){.src
.rightside}[§](#associatedtype.Output-26){.anchor}

#### type [Output](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.Mul.html#associatedtype.Output){.associatedtype} = [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#type-output-positive-12 .code-header}
:::

::: docblock
The resulting type after applying the `*` operator.
:::

::: {#method.mul-3 .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#1218-1220){.src
.rightside}[§](#method.mul-3){.anchor}

#### fn [mul](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.Mul.html#tymethod.mul){.fn}(self, other: [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}) -\> [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#fn-mulself-other-positive---positive .code-header}
:::

::: docblock
Performs the `*` operation. [Read
more](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.Mul.html#tymethod.mul)
:::
:::::::

::: {#impl-MulAssign%3C%26Positive%3E-for-Decimal .section .impl}
[Source](../../../src/optionstratlib/model/decimal.rs.html#203-207){.src
.rightside}[§](#impl-MulAssign%3C%26Positive%3E-for-Decimal){.anchor}

### impl [MulAssign](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.MulAssign.html "trait core::ops::arith::MulAssign"){.trait}\<&[Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\> for [Decimal](../../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct} {#impl-mulassignpositive-for-decimal .code-header}
:::

::::: impl-items
::: {#method.mul_assign-1 .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/decimal.rs.html#204-206){.src
.rightside}[§](#method.mul_assign-1){.anchor}

#### fn [mul_assign](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.MulAssign.html#tymethod.mul_assign){.fn}(&mut self, rhs: &[Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}) {#fn-mul_assignmut-self-rhs-positive .code-header}
:::

::: docblock
Performs the `*=` operation. [Read
more](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.MulAssign.html#tymethod.mul_assign)
:::
:::::

::: {#impl-MulAssign%3CDecimal%3E-for-Positive .section .impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#1157-1161){.src
.rightside}[§](#impl-MulAssign%3CDecimal%3E-for-Positive){.anchor}

### impl [MulAssign](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.MulAssign.html "trait core::ops::arith::MulAssign"){.trait}\<[Decimal](../../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}\> for [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#impl-mulassigndecimal-for-positive .code-header}
:::

::::: impl-items
::: {#method.mul_assign-2 .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#1158-1160){.src
.rightside}[§](#method.mul_assign-2){.anchor}

#### fn [mul_assign](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.MulAssign.html#tymethod.mul_assign){.fn}(&mut self, rhs: [Decimal](../../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}) {#fn-mul_assignmut-self-rhs-decimal .code-header}
:::

::: docblock
Performs the `*=` operation. [Read
more](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.MulAssign.html#tymethod.mul_assign)
:::
:::::

::: {#impl-MulAssign%3CPositive%3E-for-Decimal .section .impl}
[Source](../../../src/optionstratlib/model/decimal.rs.html#197-201){.src
.rightside}[§](#impl-MulAssign%3CPositive%3E-for-Decimal){.anchor}

### impl [MulAssign](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.MulAssign.html "trait core::ops::arith::MulAssign"){.trait}\<[Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\> for [Decimal](../../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct} {#impl-mulassignpositive-for-decimal-1 .code-header}
:::

::::: impl-items
::: {#method.mul_assign .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/decimal.rs.html#198-200){.src
.rightside}[§](#method.mul_assign){.anchor}

#### fn [mul_assign](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.MulAssign.html#tymethod.mul_assign){.fn}(&mut self, rhs: [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}) {#fn-mul_assignmut-self-rhs-positive-1 .code-header}
:::

::: docblock
Performs the `*=` operation. [Read
more](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.MulAssign.html#tymethod.mul_assign)
:::
:::::

::: {#impl-Neg-for-Positive .section .impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#1207-1213){.src
.rightside}[§](#impl-Neg-for-Positive){.anchor}

### impl [Neg](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.Neg.html "trait core::ops::arith::Neg"){.trait} for [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#impl-neg-for-positive .code-header}
:::

::::::: impl-items
::: {#associatedtype.Output-25 .section .associatedtype .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#1208){.src
.rightside}[§](#associatedtype.Output-25){.anchor}

#### type [Output](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.Neg.html#associatedtype.Output){.associatedtype} = [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#type-output-positive-13 .code-header}
:::

::: docblock
The resulting type after applying the `-` operator.
:::

::: {#method.neg .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#1210-1212){.src
.rightside}[§](#method.neg){.anchor}

#### fn [neg](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.Neg.html#tymethod.neg){.fn}(self) -\> Self::[Output](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.Neg.html#associatedtype.Output "type core::ops::arith::Neg::Output"){.associatedtype} {#fn-negself---selfoutput .code-header}
:::

::: docblock
Performs the unary `-` operation. [Read
more](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.Neg.html#tymethod.neg)
:::
:::::::

::: {#impl-Ord-for-Positive .section .impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#1201-1205){.src
.rightside}[§](#impl-Ord-for-Positive){.anchor}

### impl [Ord](https://doc.rust-lang.org/1.91.1/core/cmp/trait.Ord.html "trait core::cmp::Ord"){.trait} for [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#impl-ord-for-positive .code-header}
:::

:::::::::::::: impl-items
::: {#method.cmp .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#1202-1204){.src
.rightside}[§](#method.cmp){.anchor}

#### fn [cmp](https://doc.rust-lang.org/1.91.1/core/cmp/trait.Ord.html#tymethod.cmp){.fn}(&self, other: &Self) -\> [Ordering](https://doc.rust-lang.org/1.91.1/core/cmp/enum.Ordering.html "enum core::cmp::Ordering"){.enum} {#fn-cmpself-other-self---ordering .code-header}
:::

::: docblock
This method returns an
[`Ordering`](https://doc.rust-lang.org/1.91.1/core/cmp/enum.Ordering.html "enum core::cmp::Ordering")
between `self` and `other`. [Read
more](https://doc.rust-lang.org/1.91.1/core/cmp/trait.Ord.html#tymethod.cmp)
:::

:::: {#method.max-1 .section .method .trait-impl}
[[1.21.0]{.since title="Stable since Rust version 1.21.0"} ·
[Source](https://doc.rust-lang.org/1.91.1/src/core/cmp.rs.html#1023-1025){.src}]{.rightside}[§](#method.max-1){.anchor}

#### fn [max](https://doc.rust-lang.org/1.91.1/core/cmp/trait.Ord.html#method.max){.fn}(self, other: Self) -\> Self {#fn-maxself-other-self---self .code-header}

::: where
where Self:
[Sized](https://doc.rust-lang.org/1.91.1/core/marker/trait.Sized.html "trait core::marker::Sized"){.trait},
:::
::::

::: docblock
Compares and returns the maximum of two values. [Read
more](https://doc.rust-lang.org/1.91.1/core/cmp/trait.Ord.html#method.max)
:::

:::: {#method.min-1 .section .method .trait-impl}
[[1.21.0]{.since title="Stable since Rust version 1.21.0"} ·
[Source](https://doc.rust-lang.org/1.91.1/src/core/cmp.rs.html#1062-1064){.src}]{.rightside}[§](#method.min-1){.anchor}

#### fn [min](https://doc.rust-lang.org/1.91.1/core/cmp/trait.Ord.html#method.min){.fn}(self, other: Self) -\> Self {#fn-minself-other-self---self .code-header}

::: where
where Self:
[Sized](https://doc.rust-lang.org/1.91.1/core/marker/trait.Sized.html "trait core::marker::Sized"){.trait},
:::
::::

::: docblock
Compares and returns the minimum of two values. [Read
more](https://doc.rust-lang.org/1.91.1/core/cmp/trait.Ord.html#method.min)
:::

:::: {#method.clamp-1 .section .method .trait-impl}
[[1.50.0]{.since title="Stable since Rust version 1.50.0"} ·
[Source](https://doc.rust-lang.org/1.91.1/src/core/cmp.rs.html#1088-1090){.src}]{.rightside}[§](#method.clamp-1){.anchor}

#### fn [clamp](https://doc.rust-lang.org/1.91.1/core/cmp/trait.Ord.html#method.clamp){.fn}(self, min: Self, max: Self) -\> Self {#fn-clampself-min-self-max-self---self .code-header}

::: where
where Self:
[Sized](https://doc.rust-lang.org/1.91.1/core/marker/trait.Sized.html "trait core::marker::Sized"){.trait},
:::
::::

::: docblock
Restrict a value to a certain interval. [Read
more](https://doc.rust-lang.org/1.91.1/core/cmp/trait.Ord.html#method.clamp)
:::
::::::::::::::

::: {#impl-PartialEq%3C%26Positive%3E-for-Positive .section .impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#728-732){.src
.rightside}[§](#impl-PartialEq%3C%26Positive%3E-for-Positive){.anchor}

### impl [PartialEq](https://doc.rust-lang.org/1.91.1/core/cmp/trait.PartialEq.html "trait core::cmp::PartialEq"){.trait}\<&[Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\> for [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#impl-partialeqpositive-for-positive .code-header}
:::

::::::: impl-items
::: {#method.eq-2 .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#729-731){.src
.rightside}[§](#method.eq-2){.anchor}

#### fn [eq](https://doc.rust-lang.org/1.91.1/core/cmp/trait.PartialEq.html#tymethod.eq){.fn}(&self, other: &&[Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}) -\> [bool](https://doc.rust-lang.org/1.91.1/std/primitive.bool.html){.primitive} {#fn-eqself-other-positive---bool .code-header}
:::

::: docblock
Tests for `self` and `other` values to be equal, and is used by `==`.
:::

::: {#method.ne-2 .section .method .trait-impl}
[[1.0.0]{.since title="Stable since Rust version 1.0.0"} ·
[Source](https://doc.rust-lang.org/1.91.1/src/core/cmp.rs.html#264){.src}]{.rightside}[§](#method.ne-2){.anchor}

#### fn [ne](https://doc.rust-lang.org/1.91.1/core/cmp/trait.PartialEq.html#method.ne){.fn}(&self, other: [&Rhs](https://doc.rust-lang.org/1.91.1/std/primitive.reference.html){.primitive}) -\> [bool](https://doc.rust-lang.org/1.91.1/std/primitive.bool.html){.primitive} {#fn-neself-other-rhs---bool .code-header}
:::

::: docblock
Tests for `!=`. The default implementation is almost always sufficient,
and should not be overridden without very good reason.
:::
:::::::

::: {#impl-PartialEq%3C%26Positive%3E-for-f64 .section .impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#758-762){.src
.rightside}[§](#impl-PartialEq%3C%26Positive%3E-for-f64){.anchor}

### impl [PartialEq](https://doc.rust-lang.org/1.91.1/core/cmp/trait.PartialEq.html "trait core::cmp::PartialEq"){.trait}\<&[Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\> for [f64](https://doc.rust-lang.org/1.91.1/std/primitive.f64.html){.primitive} {#impl-partialeqpositive-for-f64 .code-header}
:::

::::::: impl-items
::: {#method.eq-3 .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#759-761){.src
.rightside}[§](#method.eq-3){.anchor}

#### fn [eq](https://doc.rust-lang.org/1.91.1/core/cmp/trait.PartialEq.html#tymethod.eq){.fn}(&self, other: &&[Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}) -\> [bool](https://doc.rust-lang.org/1.91.1/std/primitive.bool.html){.primitive} {#fn-eqself-other-positive---bool-1 .code-header}
:::

::: docblock
Tests for `self` and `other` values to be equal, and is used by `==`.
:::

::: {#method.ne-3 .section .method .trait-impl}
[[1.0.0]{.since title="Stable since Rust version 1.0.0"} ·
[Source](https://doc.rust-lang.org/1.91.1/src/core/cmp.rs.html#264){.src}]{.rightside}[§](#method.ne-3){.anchor}

#### fn [ne](https://doc.rust-lang.org/1.91.1/core/cmp/trait.PartialEq.html#method.ne){.fn}(&self, other: [&Rhs](https://doc.rust-lang.org/1.91.1/std/primitive.reference.html){.primitive}) -\> [bool](https://doc.rust-lang.org/1.91.1/std/primitive.bool.html){.primitive} {#fn-neself-other-rhs---bool-1 .code-header}
:::

::: docblock
Tests for `!=`. The default implementation is almost always sufficient,
and should not be overridden without very good reason.
:::
:::::::

::: {#impl-PartialEq%3CDecimal%3E-for-Positive .section .impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#975-979){.src
.rightside}[§](#impl-PartialEq%3CDecimal%3E-for-Positive){.anchor}

### impl [PartialEq](https://doc.rust-lang.org/1.91.1/core/cmp/trait.PartialEq.html "trait core::cmp::PartialEq"){.trait}\<[Decimal](../../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}\> for [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#impl-partialeqdecimal-for-positive .code-header}
:::

::::::: impl-items
::: {#method.eq-7 .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#976-978){.src
.rightside}[§](#method.eq-7){.anchor}

#### fn [eq](https://doc.rust-lang.org/1.91.1/core/cmp/trait.PartialEq.html#tymethod.eq){.fn}(&self, other: &[Decimal](../../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}) -\> [bool](https://doc.rust-lang.org/1.91.1/std/primitive.bool.html){.primitive} {#fn-eqself-other-decimal---bool .code-header}
:::

::: docblock
Tests for `self` and `other` values to be equal, and is used by `==`.
:::

::: {#method.ne-7 .section .method .trait-impl}
[[1.0.0]{.since title="Stable since Rust version 1.0.0"} ·
[Source](https://doc.rust-lang.org/1.91.1/src/core/cmp.rs.html#264){.src}]{.rightside}[§](#method.ne-7){.anchor}

#### fn [ne](https://doc.rust-lang.org/1.91.1/core/cmp/trait.PartialEq.html#method.ne){.fn}(&self, other: [&Rhs](https://doc.rust-lang.org/1.91.1/std/primitive.reference.html){.primitive}) -\> [bool](https://doc.rust-lang.org/1.91.1/std/primitive.bool.html){.primitive} {#fn-neself-other-rhs---bool-2 .code-header}
:::

::: docblock
Tests for `!=`. The default implementation is almost always sufficient,
and should not be overridden without very good reason.
:::
:::::::

::: {#impl-PartialEq%3CPositive%3E-for-Decimal .section .impl}
[Source](../../../src/optionstratlib/model/decimal.rs.html#209-213){.src
.rightside}[§](#impl-PartialEq%3CPositive%3E-for-Decimal){.anchor}

### impl [PartialEq](https://doc.rust-lang.org/1.91.1/core/cmp/trait.PartialEq.html "trait core::cmp::PartialEq"){.trait}\<[Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\> for [Decimal](../../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct} {#impl-partialeqpositive-for-decimal .code-header}
:::

::::::: impl-items
::: {#method.eq .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/decimal.rs.html#210-212){.src
.rightside}[§](#method.eq){.anchor}

#### fn [eq](https://doc.rust-lang.org/1.91.1/core/cmp/trait.PartialEq.html#tymethod.eq){.fn}(&self, other: &[Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}) -\> [bool](https://doc.rust-lang.org/1.91.1/std/primitive.bool.html){.primitive} {#fn-eqself-other-positive---bool-2 .code-header}
:::

::: docblock
Tests for `self` and `other` values to be equal, and is used by `==`.
:::

::: {#method.ne .section .method .trait-impl}
[[1.0.0]{.since title="Stable since Rust version 1.0.0"} ·
[Source](https://doc.rust-lang.org/1.91.1/src/core/cmp.rs.html#264){.src}]{.rightside}[§](#method.ne){.anchor}

#### fn [ne](https://doc.rust-lang.org/1.91.1/core/cmp/trait.PartialEq.html#method.ne){.fn}(&self, other: [&Rhs](https://doc.rust-lang.org/1.91.1/std/primitive.reference.html){.primitive}) -\> [bool](https://doc.rust-lang.org/1.91.1/std/primitive.bool.html){.primitive} {#fn-neself-other-rhs---bool-3 .code-header}
:::

::: docblock
Tests for `!=`. The default implementation is almost always sufficient,
and should not be overridden without very good reason.
:::
:::::::

::: {#impl-PartialEq%3CPositive%3E-for-f64 .section .impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#770-774){.src
.rightside}[§](#impl-PartialEq%3CPositive%3E-for-f64){.anchor}

### impl [PartialEq](https://doc.rust-lang.org/1.91.1/core/cmp/trait.PartialEq.html "trait core::cmp::PartialEq"){.trait}\<[Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\> for [f64](https://doc.rust-lang.org/1.91.1/std/primitive.f64.html){.primitive} {#impl-partialeqpositive-for-f64-1 .code-header}
:::

::::::: impl-items
::: {#method.eq-4 .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#771-773){.src
.rightside}[§](#method.eq-4){.anchor}

#### fn [eq](https://doc.rust-lang.org/1.91.1/core/cmp/trait.PartialEq.html#tymethod.eq){.fn}(&self, other: &[Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}) -\> [bool](https://doc.rust-lang.org/1.91.1/std/primitive.bool.html){.primitive} {#fn-eqself-other-positive---bool-3 .code-header}
:::

::: docblock
Tests for `self` and `other` values to be equal, and is used by `==`.
:::

::: {#method.ne-4 .section .method .trait-impl}
[[1.0.0]{.since title="Stable since Rust version 1.0.0"} ·
[Source](https://doc.rust-lang.org/1.91.1/src/core/cmp.rs.html#264){.src}]{.rightside}[§](#method.ne-4){.anchor}

#### fn [ne](https://doc.rust-lang.org/1.91.1/core/cmp/trait.PartialEq.html#method.ne){.fn}(&self, other: [&Rhs](https://doc.rust-lang.org/1.91.1/std/primitive.reference.html){.primitive}) -\> [bool](https://doc.rust-lang.org/1.91.1/std/primitive.bool.html){.primitive} {#fn-neself-other-rhs---bool-4 .code-header}
:::

::: docblock
Tests for `!=`. The default implementation is almost always sufficient,
and should not be overridden without very good reason.
:::
:::::::

::: {#impl-PartialEq%3Cf64%3E-for-%26Positive .section .impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#926-930){.src
.rightside}[§](#impl-PartialEq%3Cf64%3E-for-%26Positive){.anchor}

### impl [PartialEq](https://doc.rust-lang.org/1.91.1/core/cmp/trait.PartialEq.html "trait core::cmp::PartialEq"){.trait}\<[f64](https://doc.rust-lang.org/1.91.1/std/primitive.f64.html){.primitive}\> for &[Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#impl-partialeqf64-for-positive .code-header}
:::

::::::: impl-items
::: {#method.eq-5 .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#927-929){.src
.rightside}[§](#method.eq-5){.anchor}

#### fn [eq](https://doc.rust-lang.org/1.91.1/core/cmp/trait.PartialEq.html#tymethod.eq){.fn}(&self, other: &[f64](https://doc.rust-lang.org/1.91.1/std/primitive.f64.html){.primitive}) -\> [bool](https://doc.rust-lang.org/1.91.1/std/primitive.bool.html){.primitive} {#fn-eqself-other-f64---bool .code-header}
:::

::: docblock
Tests for `self` and `other` values to be equal, and is used by `==`.
:::

::: {#method.ne-5 .section .method .trait-impl}
[[1.0.0]{.since title="Stable since Rust version 1.0.0"} ·
[Source](https://doc.rust-lang.org/1.91.1/src/core/cmp.rs.html#264){.src}]{.rightside}[§](#method.ne-5){.anchor}

#### fn [ne](https://doc.rust-lang.org/1.91.1/core/cmp/trait.PartialEq.html#method.ne){.fn}(&self, other: [&Rhs](https://doc.rust-lang.org/1.91.1/std/primitive.reference.html){.primitive}) -\> [bool](https://doc.rust-lang.org/1.91.1/std/primitive.bool.html){.primitive} {#fn-neself-other-rhs---bool-5 .code-header}
:::

::: docblock
Tests for `!=`. The default implementation is almost always sufficient,
and should not be overridden without very good reason.
:::
:::::::

::: {#impl-PartialEq%3Cf64%3E-for-Positive .section .impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#938-942){.src
.rightside}[§](#impl-PartialEq%3Cf64%3E-for-Positive){.anchor}

### impl [PartialEq](https://doc.rust-lang.org/1.91.1/core/cmp/trait.PartialEq.html "trait core::cmp::PartialEq"){.trait}\<[f64](https://doc.rust-lang.org/1.91.1/std/primitive.f64.html){.primitive}\> for [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#impl-partialeqf64-for-positive-1 .code-header}
:::

::::::: impl-items
::: {#method.eq-6 .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#939-941){.src
.rightside}[§](#method.eq-6){.anchor}

#### fn [eq](https://doc.rust-lang.org/1.91.1/core/cmp/trait.PartialEq.html#tymethod.eq){.fn}(&self, other: &[f64](https://doc.rust-lang.org/1.91.1/std/primitive.f64.html){.primitive}) -\> [bool](https://doc.rust-lang.org/1.91.1/std/primitive.bool.html){.primitive} {#fn-eqself-other-f64---bool-1 .code-header}
:::

::: docblock
Tests for `self` and `other` values to be equal, and is used by `==`.
:::

::: {#method.ne-6 .section .method .trait-impl}
[[1.0.0]{.since title="Stable since Rust version 1.0.0"} ·
[Source](https://doc.rust-lang.org/1.91.1/src/core/cmp.rs.html#264){.src}]{.rightside}[§](#method.ne-6){.anchor}

#### fn [ne](https://doc.rust-lang.org/1.91.1/core/cmp/trait.PartialEq.html#method.ne){.fn}(&self, other: [&Rhs](https://doc.rust-lang.org/1.91.1/std/primitive.reference.html){.primitive}) -\> [bool](https://doc.rust-lang.org/1.91.1/std/primitive.bool.html){.primitive} {#fn-neself-other-rhs---bool-6 .code-header}
:::

::: docblock
Tests for `!=`. The default implementation is almost always sufficient,
and should not be overridden without very good reason.
:::
:::::::

::: {#impl-PartialEq-for-Positive .section .impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#43){.src
.rightside}[§](#impl-PartialEq-for-Positive){.anchor}

### impl [PartialEq](https://doc.rust-lang.org/1.91.1/core/cmp/trait.PartialEq.html "trait core::cmp::PartialEq"){.trait} for [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#impl-partialeq-for-positive .code-header}
:::

::::::: impl-items
::: {#method.eq-1 .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#43){.src
.rightside}[§](#method.eq-1){.anchor}

#### fn [eq](https://doc.rust-lang.org/1.91.1/core/cmp/trait.PartialEq.html#tymethod.eq){.fn}(&self, other: &[Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}) -\> [bool](https://doc.rust-lang.org/1.91.1/std/primitive.bool.html){.primitive} {#fn-eqself-other-positive---bool-4 .code-header}
:::

::: docblock
Tests for `self` and `other` values to be equal, and is used by `==`.
:::

::: {#method.ne-1 .section .method .trait-impl}
[[1.0.0]{.since title="Stable since Rust version 1.0.0"} ·
[Source](https://doc.rust-lang.org/1.91.1/src/core/cmp.rs.html#264){.src}]{.rightside}[§](#method.ne-1){.anchor}

#### fn [ne](https://doc.rust-lang.org/1.91.1/core/cmp/trait.PartialEq.html#method.ne){.fn}(&self, other: [&Rhs](https://doc.rust-lang.org/1.91.1/std/primitive.reference.html){.primitive}) -\> [bool](https://doc.rust-lang.org/1.91.1/std/primitive.bool.html){.primitive} {#fn-neself-other-rhs---bool-7 .code-header}
:::

::: docblock
Tests for `!=`. The default implementation is almost always sufficient,
and should not be overridden without very good reason.
:::
:::::::

::: {#impl-PartialOrd%3C%26Positive%3E-for-f64 .section .impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#764-768){.src
.rightside}[§](#impl-PartialOrd%3C%26Positive%3E-for-f64){.anchor}

### impl [PartialOrd](https://doc.rust-lang.org/1.91.1/core/cmp/trait.PartialOrd.html "trait core::cmp::PartialOrd"){.trait}\<&[Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\> for [f64](https://doc.rust-lang.org/1.91.1/std/primitive.f64.html){.primitive} {#impl-partialordpositive-for-f64 .code-header}
:::

::::::::::::: impl-items
::: {#method.partial_cmp .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#765-767){.src
.rightside}[§](#method.partial_cmp){.anchor}

#### fn [partial_cmp](https://doc.rust-lang.org/1.91.1/core/cmp/trait.PartialOrd.html#tymethod.partial_cmp){.fn}(&self, other: &&[Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}) -\> [Option](https://doc.rust-lang.org/1.91.1/core/option/enum.Option.html "enum core::option::Option"){.enum}\<[Ordering](https://doc.rust-lang.org/1.91.1/core/cmp/enum.Ordering.html "enum core::cmp::Ordering"){.enum}\> {#fn-partial_cmpself-other-positive---optionordering .code-header}
:::

::: docblock
This method returns an ordering between `self` and `other` values if one
exists. [Read
more](https://doc.rust-lang.org/1.91.1/core/cmp/trait.PartialOrd.html#tymethod.partial_cmp)
:::

::: {#method.lt .section .method .trait-impl}
[[1.0.0]{.since title="Stable since Rust version 1.0.0"} ·
[Source](https://doc.rust-lang.org/1.91.1/src/core/cmp.rs.html#1399){.src}]{.rightside}[§](#method.lt){.anchor}

#### fn [lt](https://doc.rust-lang.org/1.91.1/core/cmp/trait.PartialOrd.html#method.lt){.fn}(&self, other: [&Rhs](https://doc.rust-lang.org/1.91.1/std/primitive.reference.html){.primitive}) -\> [bool](https://doc.rust-lang.org/1.91.1/std/primitive.bool.html){.primitive} {#fn-ltself-other-rhs---bool .code-header}
:::

::: docblock
Tests less than (for `self` and `other`) and is used by the `<`
operator. [Read
more](https://doc.rust-lang.org/1.91.1/core/cmp/trait.PartialOrd.html#method.lt)
:::

::: {#method.le .section .method .trait-impl}
[[1.0.0]{.since title="Stable since Rust version 1.0.0"} ·
[Source](https://doc.rust-lang.org/1.91.1/src/core/cmp.rs.html#1417){.src}]{.rightside}[§](#method.le){.anchor}

#### fn [le](https://doc.rust-lang.org/1.91.1/core/cmp/trait.PartialOrd.html#method.le){.fn}(&self, other: [&Rhs](https://doc.rust-lang.org/1.91.1/std/primitive.reference.html){.primitive}) -\> [bool](https://doc.rust-lang.org/1.91.1/std/primitive.bool.html){.primitive} {#fn-leself-other-rhs---bool .code-header}
:::

::: docblock
Tests less than or equal to (for `self` and `other`) and is used by the
`<=` operator. [Read
more](https://doc.rust-lang.org/1.91.1/core/cmp/trait.PartialOrd.html#method.le)
:::

::: {#method.gt .section .method .trait-impl}
[[1.0.0]{.since title="Stable since Rust version 1.0.0"} ·
[Source](https://doc.rust-lang.org/1.91.1/src/core/cmp.rs.html#1435){.src}]{.rightside}[§](#method.gt){.anchor}

#### fn [gt](https://doc.rust-lang.org/1.91.1/core/cmp/trait.PartialOrd.html#method.gt){.fn}(&self, other: [&Rhs](https://doc.rust-lang.org/1.91.1/std/primitive.reference.html){.primitive}) -\> [bool](https://doc.rust-lang.org/1.91.1/std/primitive.bool.html){.primitive} {#fn-gtself-other-rhs---bool .code-header}
:::

::: docblock
Tests greater than (for `self` and `other`) and is used by the `>`
operator. [Read
more](https://doc.rust-lang.org/1.91.1/core/cmp/trait.PartialOrd.html#method.gt)
:::

::: {#method.ge .section .method .trait-impl}
[[1.0.0]{.since title="Stable since Rust version 1.0.0"} ·
[Source](https://doc.rust-lang.org/1.91.1/src/core/cmp.rs.html#1453){.src}]{.rightside}[§](#method.ge){.anchor}

#### fn [ge](https://doc.rust-lang.org/1.91.1/core/cmp/trait.PartialOrd.html#method.ge){.fn}(&self, other: [&Rhs](https://doc.rust-lang.org/1.91.1/std/primitive.reference.html){.primitive}) -\> [bool](https://doc.rust-lang.org/1.91.1/std/primitive.bool.html){.primitive} {#fn-geself-other-rhs---bool .code-header}
:::

::: docblock
Tests greater than or equal to (for `self` and `other`) and is used by
the `>=` operator. [Read
more](https://doc.rust-lang.org/1.91.1/core/cmp/trait.PartialOrd.html#method.ge)
:::
:::::::::::::

::: {#impl-PartialOrd%3CDecimal%3E-for-Positive .section .impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#1179-1183){.src
.rightside}[§](#impl-PartialOrd%3CDecimal%3E-for-Positive){.anchor}

### impl [PartialOrd](https://doc.rust-lang.org/1.91.1/core/cmp/trait.PartialOrd.html "trait core::cmp::PartialOrd"){.trait}\<[Decimal](../../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}\> for [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#impl-partialorddecimal-for-positive .code-header}
:::

::::::::::::: impl-items
::: {#method.partial_cmp-4 .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#1180-1182){.src
.rightside}[§](#method.partial_cmp-4){.anchor}

#### fn [partial_cmp](https://doc.rust-lang.org/1.91.1/core/cmp/trait.PartialOrd.html#tymethod.partial_cmp){.fn}(&self, other: &[Decimal](../../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}) -\> [Option](https://doc.rust-lang.org/1.91.1/core/option/enum.Option.html "enum core::option::Option"){.enum}\<[Ordering](https://doc.rust-lang.org/1.91.1/core/cmp/enum.Ordering.html "enum core::cmp::Ordering"){.enum}\> {#fn-partial_cmpself-other-decimal---optionordering .code-header}
:::

::: docblock
This method returns an ordering between `self` and `other` values if one
exists. [Read
more](https://doc.rust-lang.org/1.91.1/core/cmp/trait.PartialOrd.html#tymethod.partial_cmp)
:::

::: {#method.lt-4 .section .method .trait-impl}
[[1.0.0]{.since title="Stable since Rust version 1.0.0"} ·
[Source](https://doc.rust-lang.org/1.91.1/src/core/cmp.rs.html#1399){.src}]{.rightside}[§](#method.lt-4){.anchor}

#### fn [lt](https://doc.rust-lang.org/1.91.1/core/cmp/trait.PartialOrd.html#method.lt){.fn}(&self, other: [&Rhs](https://doc.rust-lang.org/1.91.1/std/primitive.reference.html){.primitive}) -\> [bool](https://doc.rust-lang.org/1.91.1/std/primitive.bool.html){.primitive} {#fn-ltself-other-rhs---bool-1 .code-header}
:::

::: docblock
Tests less than (for `self` and `other`) and is used by the `<`
operator. [Read
more](https://doc.rust-lang.org/1.91.1/core/cmp/trait.PartialOrd.html#method.lt)
:::

::: {#method.le-4 .section .method .trait-impl}
[[1.0.0]{.since title="Stable since Rust version 1.0.0"} ·
[Source](https://doc.rust-lang.org/1.91.1/src/core/cmp.rs.html#1417){.src}]{.rightside}[§](#method.le-4){.anchor}

#### fn [le](https://doc.rust-lang.org/1.91.1/core/cmp/trait.PartialOrd.html#method.le){.fn}(&self, other: [&Rhs](https://doc.rust-lang.org/1.91.1/std/primitive.reference.html){.primitive}) -\> [bool](https://doc.rust-lang.org/1.91.1/std/primitive.bool.html){.primitive} {#fn-leself-other-rhs---bool-1 .code-header}
:::

::: docblock
Tests less than or equal to (for `self` and `other`) and is used by the
`<=` operator. [Read
more](https://doc.rust-lang.org/1.91.1/core/cmp/trait.PartialOrd.html#method.le)
:::

::: {#method.gt-4 .section .method .trait-impl}
[[1.0.0]{.since title="Stable since Rust version 1.0.0"} ·
[Source](https://doc.rust-lang.org/1.91.1/src/core/cmp.rs.html#1435){.src}]{.rightside}[§](#method.gt-4){.anchor}

#### fn [gt](https://doc.rust-lang.org/1.91.1/core/cmp/trait.PartialOrd.html#method.gt){.fn}(&self, other: [&Rhs](https://doc.rust-lang.org/1.91.1/std/primitive.reference.html){.primitive}) -\> [bool](https://doc.rust-lang.org/1.91.1/std/primitive.bool.html){.primitive} {#fn-gtself-other-rhs---bool-1 .code-header}
:::

::: docblock
Tests greater than (for `self` and `other`) and is used by the `>`
operator. [Read
more](https://doc.rust-lang.org/1.91.1/core/cmp/trait.PartialOrd.html#method.gt)
:::

::: {#method.ge-4 .section .method .trait-impl}
[[1.0.0]{.since title="Stable since Rust version 1.0.0"} ·
[Source](https://doc.rust-lang.org/1.91.1/src/core/cmp.rs.html#1453){.src}]{.rightside}[§](#method.ge-4){.anchor}

#### fn [ge](https://doc.rust-lang.org/1.91.1/core/cmp/trait.PartialOrd.html#method.ge){.fn}(&self, other: [&Rhs](https://doc.rust-lang.org/1.91.1/std/primitive.reference.html){.primitive}) -\> [bool](https://doc.rust-lang.org/1.91.1/std/primitive.bool.html){.primitive} {#fn-geself-other-rhs---bool-1 .code-header}
:::

::: docblock
Tests greater than or equal to (for `self` and `other`) and is used by
the `>=` operator. [Read
more](https://doc.rust-lang.org/1.91.1/core/cmp/trait.PartialOrd.html#method.ge)
:::
:::::::::::::

::: {#impl-PartialOrd%3CPositive%3E-for-f64 .section .impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#776-780){.src
.rightside}[§](#impl-PartialOrd%3CPositive%3E-for-f64){.anchor}

### impl [PartialOrd](https://doc.rust-lang.org/1.91.1/core/cmp/trait.PartialOrd.html "trait core::cmp::PartialOrd"){.trait}\<[Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\> for [f64](https://doc.rust-lang.org/1.91.1/std/primitive.f64.html){.primitive} {#impl-partialordpositive-for-f64-1 .code-header}
:::

::::::::::::: impl-items
::: {#method.partial_cmp-1 .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#777-779){.src
.rightside}[§](#method.partial_cmp-1){.anchor}

#### fn [partial_cmp](https://doc.rust-lang.org/1.91.1/core/cmp/trait.PartialOrd.html#tymethod.partial_cmp){.fn}(&self, other: &[Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}) -\> [Option](https://doc.rust-lang.org/1.91.1/core/option/enum.Option.html "enum core::option::Option"){.enum}\<[Ordering](https://doc.rust-lang.org/1.91.1/core/cmp/enum.Ordering.html "enum core::cmp::Ordering"){.enum}\> {#fn-partial_cmpself-other-positive---optionordering-1 .code-header}
:::

::: docblock
This method returns an ordering between `self` and `other` values if one
exists. [Read
more](https://doc.rust-lang.org/1.91.1/core/cmp/trait.PartialOrd.html#tymethod.partial_cmp)
:::

::: {#method.lt-1 .section .method .trait-impl}
[[1.0.0]{.since title="Stable since Rust version 1.0.0"} ·
[Source](https://doc.rust-lang.org/1.91.1/src/core/cmp.rs.html#1399){.src}]{.rightside}[§](#method.lt-1){.anchor}

#### fn [lt](https://doc.rust-lang.org/1.91.1/core/cmp/trait.PartialOrd.html#method.lt){.fn}(&self, other: [&Rhs](https://doc.rust-lang.org/1.91.1/std/primitive.reference.html){.primitive}) -\> [bool](https://doc.rust-lang.org/1.91.1/std/primitive.bool.html){.primitive} {#fn-ltself-other-rhs---bool-2 .code-header}
:::

::: docblock
Tests less than (for `self` and `other`) and is used by the `<`
operator. [Read
more](https://doc.rust-lang.org/1.91.1/core/cmp/trait.PartialOrd.html#method.lt)
:::

::: {#method.le-1 .section .method .trait-impl}
[[1.0.0]{.since title="Stable since Rust version 1.0.0"} ·
[Source](https://doc.rust-lang.org/1.91.1/src/core/cmp.rs.html#1417){.src}]{.rightside}[§](#method.le-1){.anchor}

#### fn [le](https://doc.rust-lang.org/1.91.1/core/cmp/trait.PartialOrd.html#method.le){.fn}(&self, other: [&Rhs](https://doc.rust-lang.org/1.91.1/std/primitive.reference.html){.primitive}) -\> [bool](https://doc.rust-lang.org/1.91.1/std/primitive.bool.html){.primitive} {#fn-leself-other-rhs---bool-2 .code-header}
:::

::: docblock
Tests less than or equal to (for `self` and `other`) and is used by the
`<=` operator. [Read
more](https://doc.rust-lang.org/1.91.1/core/cmp/trait.PartialOrd.html#method.le)
:::

::: {#method.gt-1 .section .method .trait-impl}
[[1.0.0]{.since title="Stable since Rust version 1.0.0"} ·
[Source](https://doc.rust-lang.org/1.91.1/src/core/cmp.rs.html#1435){.src}]{.rightside}[§](#method.gt-1){.anchor}

#### fn [gt](https://doc.rust-lang.org/1.91.1/core/cmp/trait.PartialOrd.html#method.gt){.fn}(&self, other: [&Rhs](https://doc.rust-lang.org/1.91.1/std/primitive.reference.html){.primitive}) -\> [bool](https://doc.rust-lang.org/1.91.1/std/primitive.bool.html){.primitive} {#fn-gtself-other-rhs---bool-2 .code-header}
:::

::: docblock
Tests greater than (for `self` and `other`) and is used by the `>`
operator. [Read
more](https://doc.rust-lang.org/1.91.1/core/cmp/trait.PartialOrd.html#method.gt)
:::

::: {#method.ge-1 .section .method .trait-impl}
[[1.0.0]{.since title="Stable since Rust version 1.0.0"} ·
[Source](https://doc.rust-lang.org/1.91.1/src/core/cmp.rs.html#1453){.src}]{.rightside}[§](#method.ge-1){.anchor}

#### fn [ge](https://doc.rust-lang.org/1.91.1/core/cmp/trait.PartialOrd.html#method.ge){.fn}(&self, other: [&Rhs](https://doc.rust-lang.org/1.91.1/std/primitive.reference.html){.primitive}) -\> [bool](https://doc.rust-lang.org/1.91.1/std/primitive.bool.html){.primitive} {#fn-geself-other-rhs---bool-2 .code-header}
:::

::: docblock
Tests greater than or equal to (for `self` and `other`) and is used by
the `>=` operator. [Read
more](https://doc.rust-lang.org/1.91.1/core/cmp/trait.PartialOrd.html#method.ge)
:::
:::::::::::::

::: {#impl-PartialOrd%3Cf64%3E-for-%26Positive .section .impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#932-936){.src
.rightside}[§](#impl-PartialOrd%3Cf64%3E-for-%26Positive){.anchor}

### impl [PartialOrd](https://doc.rust-lang.org/1.91.1/core/cmp/trait.PartialOrd.html "trait core::cmp::PartialOrd"){.trait}\<[f64](https://doc.rust-lang.org/1.91.1/std/primitive.f64.html){.primitive}\> for &[Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#impl-partialordf64-for-positive .code-header}
:::

::::::::::::: impl-items
::: {#method.partial_cmp-3 .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#933-935){.src
.rightside}[§](#method.partial_cmp-3){.anchor}

#### fn [partial_cmp](https://doc.rust-lang.org/1.91.1/core/cmp/trait.PartialOrd.html#tymethod.partial_cmp){.fn}(&self, other: &[f64](https://doc.rust-lang.org/1.91.1/std/primitive.f64.html){.primitive}) -\> [Option](https://doc.rust-lang.org/1.91.1/core/option/enum.Option.html "enum core::option::Option"){.enum}\<[Ordering](https://doc.rust-lang.org/1.91.1/core/cmp/enum.Ordering.html "enum core::cmp::Ordering"){.enum}\> {#fn-partial_cmpself-other-f64---optionordering .code-header}
:::

::: docblock
This method returns an ordering between `self` and `other` values if one
exists. [Read
more](https://doc.rust-lang.org/1.91.1/core/cmp/trait.PartialOrd.html#tymethod.partial_cmp)
:::

::: {#method.lt-3 .section .method .trait-impl}
[[1.0.0]{.since title="Stable since Rust version 1.0.0"} ·
[Source](https://doc.rust-lang.org/1.91.1/src/core/cmp.rs.html#1399){.src}]{.rightside}[§](#method.lt-3){.anchor}

#### fn [lt](https://doc.rust-lang.org/1.91.1/core/cmp/trait.PartialOrd.html#method.lt){.fn}(&self, other: [&Rhs](https://doc.rust-lang.org/1.91.1/std/primitive.reference.html){.primitive}) -\> [bool](https://doc.rust-lang.org/1.91.1/std/primitive.bool.html){.primitive} {#fn-ltself-other-rhs---bool-3 .code-header}
:::

::: docblock
Tests less than (for `self` and `other`) and is used by the `<`
operator. [Read
more](https://doc.rust-lang.org/1.91.1/core/cmp/trait.PartialOrd.html#method.lt)
:::

::: {#method.le-3 .section .method .trait-impl}
[[1.0.0]{.since title="Stable since Rust version 1.0.0"} ·
[Source](https://doc.rust-lang.org/1.91.1/src/core/cmp.rs.html#1417){.src}]{.rightside}[§](#method.le-3){.anchor}

#### fn [le](https://doc.rust-lang.org/1.91.1/core/cmp/trait.PartialOrd.html#method.le){.fn}(&self, other: [&Rhs](https://doc.rust-lang.org/1.91.1/std/primitive.reference.html){.primitive}) -\> [bool](https://doc.rust-lang.org/1.91.1/std/primitive.bool.html){.primitive} {#fn-leself-other-rhs---bool-3 .code-header}
:::

::: docblock
Tests less than or equal to (for `self` and `other`) and is used by the
`<=` operator. [Read
more](https://doc.rust-lang.org/1.91.1/core/cmp/trait.PartialOrd.html#method.le)
:::

::: {#method.gt-3 .section .method .trait-impl}
[[1.0.0]{.since title="Stable since Rust version 1.0.0"} ·
[Source](https://doc.rust-lang.org/1.91.1/src/core/cmp.rs.html#1435){.src}]{.rightside}[§](#method.gt-3){.anchor}

#### fn [gt](https://doc.rust-lang.org/1.91.1/core/cmp/trait.PartialOrd.html#method.gt){.fn}(&self, other: [&Rhs](https://doc.rust-lang.org/1.91.1/std/primitive.reference.html){.primitive}) -\> [bool](https://doc.rust-lang.org/1.91.1/std/primitive.bool.html){.primitive} {#fn-gtself-other-rhs---bool-3 .code-header}
:::

::: docblock
Tests greater than (for `self` and `other`) and is used by the `>`
operator. [Read
more](https://doc.rust-lang.org/1.91.1/core/cmp/trait.PartialOrd.html#method.gt)
:::

::: {#method.ge-3 .section .method .trait-impl}
[[1.0.0]{.since title="Stable since Rust version 1.0.0"} ·
[Source](https://doc.rust-lang.org/1.91.1/src/core/cmp.rs.html#1453){.src}]{.rightside}[§](#method.ge-3){.anchor}

#### fn [ge](https://doc.rust-lang.org/1.91.1/core/cmp/trait.PartialOrd.html#method.ge){.fn}(&self, other: [&Rhs](https://doc.rust-lang.org/1.91.1/std/primitive.reference.html){.primitive}) -\> [bool](https://doc.rust-lang.org/1.91.1/std/primitive.bool.html){.primitive} {#fn-geself-other-rhs---bool-3 .code-header}
:::

::: docblock
Tests greater than or equal to (for `self` and `other`) and is used by
the `>=` operator. [Read
more](https://doc.rust-lang.org/1.91.1/core/cmp/trait.PartialOrd.html#method.ge)
:::
:::::::::::::

::: {#impl-PartialOrd%3Cf64%3E-for-Positive .section .impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#920-924){.src
.rightside}[§](#impl-PartialOrd%3Cf64%3E-for-Positive){.anchor}

### impl [PartialOrd](https://doc.rust-lang.org/1.91.1/core/cmp/trait.PartialOrd.html "trait core::cmp::PartialOrd"){.trait}\<[f64](https://doc.rust-lang.org/1.91.1/std/primitive.f64.html){.primitive}\> for [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#impl-partialordf64-for-positive-1 .code-header}
:::

::::::::::::: impl-items
::: {#method.partial_cmp-2 .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#921-923){.src
.rightside}[§](#method.partial_cmp-2){.anchor}

#### fn [partial_cmp](https://doc.rust-lang.org/1.91.1/core/cmp/trait.PartialOrd.html#tymethod.partial_cmp){.fn}(&self, other: &[f64](https://doc.rust-lang.org/1.91.1/std/primitive.f64.html){.primitive}) -\> [Option](https://doc.rust-lang.org/1.91.1/core/option/enum.Option.html "enum core::option::Option"){.enum}\<[Ordering](https://doc.rust-lang.org/1.91.1/core/cmp/enum.Ordering.html "enum core::cmp::Ordering"){.enum}\> {#fn-partial_cmpself-other-f64---optionordering-1 .code-header}
:::

::: docblock
This method returns an ordering between `self` and `other` values if one
exists. [Read
more](https://doc.rust-lang.org/1.91.1/core/cmp/trait.PartialOrd.html#tymethod.partial_cmp)
:::

::: {#method.lt-2 .section .method .trait-impl}
[[1.0.0]{.since title="Stable since Rust version 1.0.0"} ·
[Source](https://doc.rust-lang.org/1.91.1/src/core/cmp.rs.html#1399){.src}]{.rightside}[§](#method.lt-2){.anchor}

#### fn [lt](https://doc.rust-lang.org/1.91.1/core/cmp/trait.PartialOrd.html#method.lt){.fn}(&self, other: [&Rhs](https://doc.rust-lang.org/1.91.1/std/primitive.reference.html){.primitive}) -\> [bool](https://doc.rust-lang.org/1.91.1/std/primitive.bool.html){.primitive} {#fn-ltself-other-rhs---bool-4 .code-header}
:::

::: docblock
Tests less than (for `self` and `other`) and is used by the `<`
operator. [Read
more](https://doc.rust-lang.org/1.91.1/core/cmp/trait.PartialOrd.html#method.lt)
:::

::: {#method.le-2 .section .method .trait-impl}
[[1.0.0]{.since title="Stable since Rust version 1.0.0"} ·
[Source](https://doc.rust-lang.org/1.91.1/src/core/cmp.rs.html#1417){.src}]{.rightside}[§](#method.le-2){.anchor}

#### fn [le](https://doc.rust-lang.org/1.91.1/core/cmp/trait.PartialOrd.html#method.le){.fn}(&self, other: [&Rhs](https://doc.rust-lang.org/1.91.1/std/primitive.reference.html){.primitive}) -\> [bool](https://doc.rust-lang.org/1.91.1/std/primitive.bool.html){.primitive} {#fn-leself-other-rhs---bool-4 .code-header}
:::

::: docblock
Tests less than or equal to (for `self` and `other`) and is used by the
`<=` operator. [Read
more](https://doc.rust-lang.org/1.91.1/core/cmp/trait.PartialOrd.html#method.le)
:::

::: {#method.gt-2 .section .method .trait-impl}
[[1.0.0]{.since title="Stable since Rust version 1.0.0"} ·
[Source](https://doc.rust-lang.org/1.91.1/src/core/cmp.rs.html#1435){.src}]{.rightside}[§](#method.gt-2){.anchor}

#### fn [gt](https://doc.rust-lang.org/1.91.1/core/cmp/trait.PartialOrd.html#method.gt){.fn}(&self, other: [&Rhs](https://doc.rust-lang.org/1.91.1/std/primitive.reference.html){.primitive}) -\> [bool](https://doc.rust-lang.org/1.91.1/std/primitive.bool.html){.primitive} {#fn-gtself-other-rhs---bool-4 .code-header}
:::

::: docblock
Tests greater than (for `self` and `other`) and is used by the `>`
operator. [Read
more](https://doc.rust-lang.org/1.91.1/core/cmp/trait.PartialOrd.html#method.gt)
:::

::: {#method.ge-2 .section .method .trait-impl}
[[1.0.0]{.since title="Stable since Rust version 1.0.0"} ·
[Source](https://doc.rust-lang.org/1.91.1/src/core/cmp.rs.html#1453){.src}]{.rightside}[§](#method.ge-2){.anchor}

#### fn [ge](https://doc.rust-lang.org/1.91.1/core/cmp/trait.PartialOrd.html#method.ge){.fn}(&self, other: [&Rhs](https://doc.rust-lang.org/1.91.1/std/primitive.reference.html){.primitive}) -\> [bool](https://doc.rust-lang.org/1.91.1/std/primitive.bool.html){.primitive} {#fn-geself-other-rhs---bool-4 .code-header}
:::

::: docblock
Tests greater than or equal to (for `self` and `other`) and is used by
the `>=` operator. [Read
more](https://doc.rust-lang.org/1.91.1/core/cmp/trait.PartialOrd.html#method.ge)
:::
:::::::::::::

::: {#impl-PartialOrd-for-Positive .section .impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#1185-1197){.src
.rightside}[§](#impl-PartialOrd-for-Positive){.anchor}

### impl [PartialOrd](https://doc.rust-lang.org/1.91.1/core/cmp/trait.PartialOrd.html "trait core::cmp::PartialOrd"){.trait} for [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#impl-partialord-for-positive .code-header}
:::

::::::::::::: impl-items
::: {#method.partial_cmp-5 .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#1186-1188){.src
.rightside}[§](#method.partial_cmp-5){.anchor}

#### fn [partial_cmp](https://doc.rust-lang.org/1.91.1/core/cmp/trait.PartialOrd.html#tymethod.partial_cmp){.fn}(&self, other: &Self) -\> [Option](https://doc.rust-lang.org/1.91.1/core/option/enum.Option.html "enum core::option::Option"){.enum}\<[Ordering](https://doc.rust-lang.org/1.91.1/core/cmp/enum.Ordering.html "enum core::cmp::Ordering"){.enum}\> {#fn-partial_cmpself-other-self---optionordering .code-header}
:::

::: docblock
This method returns an ordering between `self` and `other` values if one
exists. [Read
more](https://doc.rust-lang.org/1.91.1/core/cmp/trait.PartialOrd.html#tymethod.partial_cmp)
:::

::: {#method.le-5 .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#1190-1192){.src
.rightside}[§](#method.le-5){.anchor}

#### fn [le](https://doc.rust-lang.org/1.91.1/core/cmp/trait.PartialOrd.html#method.le){.fn}(&self, other: &Self) -\> [bool](https://doc.rust-lang.org/1.91.1/std/primitive.bool.html){.primitive} {#fn-leself-other-self---bool .code-header}
:::

::: docblock
Tests less than or equal to (for `self` and `other`) and is used by the
`<=` operator. [Read
more](https://doc.rust-lang.org/1.91.1/core/cmp/trait.PartialOrd.html#method.le)
:::

::: {#method.ge-5 .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#1194-1196){.src
.rightside}[§](#method.ge-5){.anchor}

#### fn [ge](https://doc.rust-lang.org/1.91.1/core/cmp/trait.PartialOrd.html#method.ge){.fn}(&self, other: &Self) -\> [bool](https://doc.rust-lang.org/1.91.1/std/primitive.bool.html){.primitive} {#fn-geself-other-self---bool .code-header}
:::

::: docblock
Tests greater than or equal to (for `self` and `other`) and is used by
the `>=` operator. [Read
more](https://doc.rust-lang.org/1.91.1/core/cmp/trait.PartialOrd.html#method.ge)
:::

::: {#method.lt-5 .section .method .trait-impl}
[[1.0.0]{.since title="Stable since Rust version 1.0.0"} ·
[Source](https://doc.rust-lang.org/1.91.1/src/core/cmp.rs.html#1399){.src}]{.rightside}[§](#method.lt-5){.anchor}

#### fn [lt](https://doc.rust-lang.org/1.91.1/core/cmp/trait.PartialOrd.html#method.lt){.fn}(&self, other: [&Rhs](https://doc.rust-lang.org/1.91.1/std/primitive.reference.html){.primitive}) -\> [bool](https://doc.rust-lang.org/1.91.1/std/primitive.bool.html){.primitive} {#fn-ltself-other-rhs---bool-5 .code-header}
:::

::: docblock
Tests less than (for `self` and `other`) and is used by the `<`
operator. [Read
more](https://doc.rust-lang.org/1.91.1/core/cmp/trait.PartialOrd.html#method.lt)
:::

::: {#method.gt-5 .section .method .trait-impl}
[[1.0.0]{.since title="Stable since Rust version 1.0.0"} ·
[Source](https://doc.rust-lang.org/1.91.1/src/core/cmp.rs.html#1435){.src}]{.rightside}[§](#method.gt-5){.anchor}

#### fn [gt](https://doc.rust-lang.org/1.91.1/core/cmp/trait.PartialOrd.html#method.gt){.fn}(&self, other: [&Rhs](https://doc.rust-lang.org/1.91.1/std/primitive.reference.html){.primitive}) -\> [bool](https://doc.rust-lang.org/1.91.1/std/primitive.bool.html){.primitive} {#fn-gtself-other-rhs---bool-5 .code-header}
:::

::: docblock
Tests greater than (for `self` and `other`) and is used by the `>`
operator. [Read
more](https://doc.rust-lang.org/1.91.1/core/cmp/trait.PartialOrd.html#method.gt)
:::
:::::::::::::

::: {#impl-RelativeEq-for-Positive .section .impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#1249-1268){.src
.rightside}[§](#impl-RelativeEq-for-Positive){.anchor}

### impl [RelativeEq](../../../approx/relative_eq/trait.RelativeEq.html "trait approx::relative_eq::RelativeEq"){.trait} for [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#impl-relativeeq-for-positive .code-header}
:::

::::::::: impl-items
::: {#method.default_max_relative .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#1250-1252){.src
.rightside}[§](#method.default_max_relative){.anchor}

#### fn [default_max_relative](../../../approx/relative_eq/trait.RelativeEq.html#tymethod.default_max_relative){.fn}() -\> Self::[Epsilon](../../../approx/abs_diff_eq/trait.AbsDiffEq.html#associatedtype.Epsilon "type approx::abs_diff_eq::AbsDiffEq::Epsilon"){.associatedtype} {#fn-default_max_relative---selfepsilon .code-header}
:::

::: docblock
The default relative tolerance for testing values that are far-apart.
[Read
more](../../../approx/relative_eq/trait.RelativeEq.html#tymethod.default_max_relative)
:::

::: {#method.relative_eq .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#1254-1267){.src
.rightside}[§](#method.relative_eq){.anchor}

#### fn [relative_eq](../../../approx/relative_eq/trait.RelativeEq.html#tymethod.relative_eq){.fn}( &self, other: &Self, epsilon: Self::[Epsilon](../../../approx/abs_diff_eq/trait.AbsDiffEq.html#associatedtype.Epsilon "type approx::abs_diff_eq::AbsDiffEq::Epsilon"){.associatedtype}, max_relative: Self::[Epsilon](../../../approx/abs_diff_eq/trait.AbsDiffEq.html#associatedtype.Epsilon "type approx::abs_diff_eq::AbsDiffEq::Epsilon"){.associatedtype}, ) -\> [bool](https://doc.rust-lang.org/1.91.1/std/primitive.bool.html){.primitive} {#fn-relative_eq-self-other-self-epsilon-selfepsilon-max_relative-selfepsilon---bool .code-header}
:::

::: docblock
A test for equality that uses a relative comparison if the values are
far apart.
:::

::: {#method.relative_ne .section .method .trait-impl}
[Source](../../../src/approx/relative_eq.rs.html#22-27){.src
.rightside}[§](#method.relative_ne){.anchor}

#### fn [relative_ne](../../../approx/relative_eq/trait.RelativeEq.html#method.relative_ne){.fn}( &self, other: [&Rhs](https://doc.rust-lang.org/1.91.1/std/primitive.reference.html){.primitive}, epsilon: Self::[Epsilon](../../../approx/abs_diff_eq/trait.AbsDiffEq.html#associatedtype.Epsilon "type approx::abs_diff_eq::AbsDiffEq::Epsilon"){.associatedtype}, max_relative: Self::[Epsilon](../../../approx/abs_diff_eq/trait.AbsDiffEq.html#associatedtype.Epsilon "type approx::abs_diff_eq::AbsDiffEq::Epsilon"){.associatedtype}, ) -\> [bool](https://doc.rust-lang.org/1.91.1/std/primitive.bool.html){.primitive} {#fn-relative_ne-self-other-rhs-epsilon-selfepsilon-max_relative-selfepsilon---bool .code-header}
:::

::: docblock
The inverse of
[`RelativeEq::relative_eq`](../../../approx/relative_eq/trait.RelativeEq.html#tymethod.relative_eq "method approx::relative_eq::RelativeEq::relative_eq").
:::
:::::::::

::: {#impl-Serialize-for-Positive .section .impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#981-1007){.src
.rightside}[§](#impl-Serialize-for-Positive){.anchor}

### impl [Serialize](../../../serde_core/ser/trait.Serialize.html "trait serde_core::ser::Serialize"){.trait} for [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#impl-serialize-for-positive .code-header}
:::

:::::: impl-items
:::: {#method.serialize .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#982-1006){.src
.rightside}[§](#method.serialize){.anchor}

#### fn [serialize](../../../serde_core/ser/trait.Serialize.html#tymethod.serialize){.fn}\<S\>(&self, serializer: S) -\> [Result](https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html "enum core::result::Result"){.enum}\<S::[Ok](../../../serde_core/ser/trait.Serializer.html#associatedtype.Ok "type serde_core::ser::Serializer::Ok"){.associatedtype}, S::[Error](../../../serde_core/ser/trait.Serializer.html#associatedtype.Error "type serde_core::ser::Serializer::Error"){.associatedtype}\> {#fn-serializesself-serializer-s---resultsok-serror .code-header}

::: where
where S:
[Serializer](../../../serde_core/ser/trait.Serializer.html "trait serde_core::ser::Serializer"){.trait},
:::
::::

::: docblock
Serialize this value into the given Serde serializer. [Read
more](../../../serde_core/ser/trait.Serialize.html#tymethod.serialize)
:::
::::::

::: {#impl-Sub%3C%26Decimal%3E-for-Positive .section .impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#1137-1143){.src
.rightside}[§](#impl-Sub%3C%26Decimal%3E-for-Positive){.anchor}

### impl [Sub](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.Sub.html "trait core::ops::arith::Sub"){.trait}\<&[Decimal](../../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}\> for [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#impl-subdecimal-for-positive .code-header}
:::

::::::: impl-items
::: {#associatedtype.Output-22 .section .associatedtype .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#1138){.src
.rightside}[§](#associatedtype.Output-22){.anchor}

#### type [Output](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.Sub.html#associatedtype.Output){.associatedtype} = [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#type-output-positive-14 .code-header}
:::

::: docblock
The resulting type after applying the `-` operator.
:::

::: {#method.sub-6 .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#1140-1142){.src
.rightside}[§](#method.sub-6){.anchor}

#### fn [sub](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.Sub.html#tymethod.sub){.fn}(self, rhs: &[Decimal](../../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}) -\> Self::[Output](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.Sub.html#associatedtype.Output "type core::ops::arith::Sub::Output"){.associatedtype} {#fn-subself-rhs-decimal---selfoutput .code-header}
:::

::: docblock
Performs the `-` operation. [Read
more](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.Sub.html#tymethod.sub)
:::
:::::::

::: {#impl-Sub%3C%26Positive%3E-for-Decimal .section .impl}
[Source](../../../src/optionstratlib/model/decimal.rs.html#161-167){.src
.rightside}[§](#impl-Sub%3C%26Positive%3E-for-Decimal){.anchor}

### impl [Sub](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.Sub.html "trait core::ops::arith::Sub"){.trait}\<&[Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\> for [Decimal](../../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct} {#impl-subpositive-for-decimal .code-header}
:::

::::::: impl-items
::: {#associatedtype.Output-3 .section .associatedtype .trait-impl}
[Source](../../../src/optionstratlib/model/decimal.rs.html#162){.src
.rightside}[§](#associatedtype.Output-3){.anchor}

#### type [Output](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.Sub.html#associatedtype.Output){.associatedtype} = [Decimal](../../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct} {#type-output-decimal-4 .code-header}
:::

::: docblock
The resulting type after applying the `-` operator.
:::

::: {#method.sub-1 .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/decimal.rs.html#164-166){.src
.rightside}[§](#method.sub-1){.anchor}

#### fn [sub](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.Sub.html#tymethod.sub){.fn}(self, rhs: &[Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}) -\> Self::[Output](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.Sub.html#associatedtype.Output "type core::ops::arith::Sub::Output"){.associatedtype} {#fn-subself-rhs-positive---selfoutput .code-header}
:::

::: docblock
Performs the `-` operation. [Read
more](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.Sub.html#tymethod.sub)
:::
:::::::

::: {#impl-Sub%3CDecimal%3E-for-Positive .section .impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#1129-1135){.src
.rightside}[§](#impl-Sub%3CDecimal%3E-for-Positive){.anchor}

### impl [Sub](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.Sub.html "trait core::ops::arith::Sub"){.trait}\<[Decimal](../../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}\> for [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#impl-subdecimal-for-positive-1 .code-header}
:::

::::::: impl-items
::: {#associatedtype.Output-21 .section .associatedtype .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#1130){.src
.rightside}[§](#associatedtype.Output-21){.anchor}

#### type [Output](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.Sub.html#associatedtype.Output){.associatedtype} = [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#type-output-positive-15 .code-header}
:::

::: docblock
The resulting type after applying the `-` operator.
:::

::: {#method.sub-5 .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#1132-1134){.src
.rightside}[§](#method.sub-5){.anchor}

#### fn [sub](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.Sub.html#tymethod.sub){.fn}(self, rhs: [Decimal](../../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct}) -\> [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#fn-subself-rhs-decimal---positive .code-header}
:::

::: docblock
Performs the `-` operation. [Read
more](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.Sub.html#tymethod.sub)
:::
:::::::

::: {#impl-Sub%3CPositive%3E-for-Decimal .section .impl}
[Source](../../../src/optionstratlib/model/decimal.rs.html#153-159){.src
.rightside}[§](#impl-Sub%3CPositive%3E-for-Decimal){.anchor}

### impl [Sub](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.Sub.html "trait core::ops::arith::Sub"){.trait}\<[Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\> for [Decimal](../../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct} {#impl-subpositive-for-decimal-1 .code-header}
:::

::::::: impl-items
::: {#associatedtype.Output-2 .section .associatedtype .trait-impl}
[Source](../../../src/optionstratlib/model/decimal.rs.html#154){.src
.rightside}[§](#associatedtype.Output-2){.anchor}

#### type [Output](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.Sub.html#associatedtype.Output){.associatedtype} = [Decimal](../../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct} {#type-output-decimal-5 .code-header}
:::

::: docblock
The resulting type after applying the `-` operator.
:::

::: {#method.sub .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/decimal.rs.html#156-158){.src
.rightside}[§](#method.sub){.anchor}

#### fn [sub](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.Sub.html#tymethod.sub){.fn}(self, rhs: [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}) -\> Self::[Output](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.Sub.html#associatedtype.Output "type core::ops::arith::Sub::Output"){.associatedtype} {#fn-subself-rhs-positive---selfoutput-1 .code-header}
:::

::: docblock
Performs the `-` operation. [Read
more](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.Sub.html#tymethod.sub)
:::
:::::::

::: {#impl-Sub%3CPositive%3E-for-f64 .section .impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#798-804){.src
.rightside}[§](#impl-Sub%3CPositive%3E-for-f64){.anchor}

### impl [Sub](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.Sub.html "trait core::ops::arith::Sub"){.trait}\<[Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\> for [f64](https://doc.rust-lang.org/1.91.1/std/primitive.f64.html){.primitive} {#impl-subpositive-for-f64 .code-header}
:::

::::::: impl-items
::: {#associatedtype.Output-8 .section .associatedtype .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#799){.src
.rightside}[§](#associatedtype.Output-8){.anchor}

#### type [Output](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.Sub.html#associatedtype.Output){.associatedtype} = [f64](https://doc.rust-lang.org/1.91.1/std/primitive.f64.html){.primitive} {#type-output-f64-3 .code-header}
:::

::: docblock
The resulting type after applying the `-` operator.
:::

::: {#method.sub-2 .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#801-803){.src
.rightside}[§](#method.sub-2){.anchor}

#### fn [sub](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.Sub.html#tymethod.sub){.fn}(self, rhs: [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}) -\> Self::[Output](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.Sub.html#associatedtype.Output "type core::ops::arith::Sub::Output"){.associatedtype} {#fn-subself-rhs-positive---selfoutput-2 .code-header}
:::

::: docblock
Performs the `-` operation. [Read
more](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.Sub.html#tymethod.sub)
:::
:::::::

::: {#impl-Sub%3Cf64%3E-for-Positive .section .impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#904-910){.src
.rightside}[§](#impl-Sub%3Cf64%3E-for-Positive){.anchor}

### impl [Sub](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.Sub.html "trait core::ops::arith::Sub"){.trait}\<[f64](https://doc.rust-lang.org/1.91.1/std/primitive.f64.html){.primitive}\> for [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#impl-subf64-for-positive .code-header}
:::

::::::: impl-items
::: {#associatedtype.Output-13 .section .associatedtype .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#905){.src
.rightside}[§](#associatedtype.Output-13){.anchor}

#### type [Output](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.Sub.html#associatedtype.Output){.associatedtype} = [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#type-output-positive-16 .code-header}
:::

::: docblock
The resulting type after applying the `-` operator.
:::

::: {#method.sub-3 .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#907-909){.src
.rightside}[§](#method.sub-3){.anchor}

#### fn [sub](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.Sub.html#tymethod.sub){.fn}(self, rhs: [f64](https://doc.rust-lang.org/1.91.1/std/primitive.f64.html){.primitive}) -\> Self::[Output](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.Sub.html#associatedtype.Output "type core::ops::arith::Sub::Output"){.associatedtype} {#fn-subself-rhs-f64---selfoutput .code-header}
:::

::: docblock
Performs the `-` operation. [Read
more](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.Sub.html#tymethod.sub)
:::
:::::::

::: {#impl-Sub-for-Positive .section .impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#1084-1095){.src
.rightside}[§](#impl-Sub-for-Positive){.anchor}

### impl [Sub](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.Sub.html "trait core::ops::arith::Sub"){.trait} for [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#impl-sub-for-positive .code-header}
:::

::::::: impl-items
::: {#associatedtype.Output-16 .section .associatedtype .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#1085){.src
.rightside}[§](#associatedtype.Output-16){.anchor}

#### type [Output](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.Sub.html#associatedtype.Output){.associatedtype} = [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#type-output-positive-17 .code-header}
:::

::: docblock
The resulting type after applying the `-` operator.
:::

::: {#method.sub-4 .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#1087-1094){.src
.rightside}[§](#method.sub-4){.anchor}

#### fn [sub](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.Sub.html#tymethod.sub){.fn}(self, rhs: Self) -\> Self::[Output](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.Sub.html#associatedtype.Output "type core::ops::arith::Sub::Output"){.associatedtype} {#fn-subself-rhs-self---selfoutput .code-header}
:::

::: docblock
Performs the `-` operation. [Read
more](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.Sub.html#tymethod.sub)
:::
:::::::

::: {#impl-Sum%3C%26Positive%3E-for-Positive .section .impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#1277-1282){.src
.rightside}[§](#impl-Sum%3C%26Positive%3E-for-Positive){.anchor}

### impl\<\'a\> [Sum](https://doc.rust-lang.org/1.91.1/core/iter/traits/accum/trait.Sum.html "trait core::iter::traits::accum::Sum"){.trait}\<&\'a [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\> for [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#impla-suma-positive-for-positive .code-header}
:::

::::: impl-items
::: {#method.sum-1 .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#1278-1281){.src
.rightside}[§](#method.sum-1){.anchor}

#### fn [sum](https://doc.rust-lang.org/1.91.1/core/iter/traits/accum/trait.Sum.html#tymethod.sum){.fn}\<I: [Iterator](https://doc.rust-lang.org/1.91.1/core/iter/traits/iterator/trait.Iterator.html "trait core::iter::traits::iterator::Iterator"){.trait}\<Item = &\'a [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct}\>\>(iter: I) -\> Self {#fn-sumi-iteratoritem-a-positiveiter-i---self .code-header}
:::

::: docblock
Takes an iterator and generates `Self` from the elements by "summing up"
the items.
:::
:::::

::: {#impl-Sum-for-Positive .section .impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#1270-1275){.src
.rightside}[§](#impl-Sum-for-Positive){.anchor}

### impl [Sum](https://doc.rust-lang.org/1.91.1/core/iter/traits/accum/trait.Sum.html "trait core::iter::traits::accum::Sum"){.trait} for [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#impl-sum-for-positive .code-header}
:::

::::: impl-items
::: {#method.sum .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#1271-1274){.src
.rightside}[§](#method.sum){.anchor}

#### fn [sum](https://doc.rust-lang.org/1.91.1/core/iter/traits/accum/trait.Sum.html#tymethod.sum){.fn}\<I: [Iterator](https://doc.rust-lang.org/1.91.1/core/iter/traits/iterator/trait.Iterator.html "trait core::iter::traits::iterator::Iterator"){.trait}\<Item = Self\>\>(iter: I) -\> Self {#fn-sumi-iteratoritem-selfiter-i---self .code-header}
:::

::: docblock
Takes an iterator and generates `Self` from the elements by "summing up"
the items.
:::
:::::

::: {#impl-ToRound-for-Positive .section .impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#718-726){.src
.rightside}[§](#impl-ToRound-for-Positive){.anchor}

### impl [ToRound](../utils/trait.ToRound.html "trait optionstratlib::model::utils::ToRound"){.trait} for [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#impl-toround-for-positive .code-header}
:::

::::::: impl-items
::: {#method.round-1 .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#719-721){.src
.rightside}[§](#method.round-1){.anchor}

#### fn [round](../utils/trait.ToRound.html#tymethod.round){.fn}(&self) -\> [Decimal](../../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct} {#fn-roundself---decimal .code-header}
:::

::: docblock
Rounds the number to the nearest integer. [Read
more](../utils/trait.ToRound.html#tymethod.round)
:::

::: {#method.round_to-1 .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#723-725){.src
.rightside}[§](#method.round_to-1){.anchor}

#### fn [round_to](../utils/trait.ToRound.html#tymethod.round_to){.fn}(&self, decimal_places: [u32](https://doc.rust-lang.org/1.91.1/std/primitive.u32.html){.primitive}) -\> [Decimal](../../prelude/struct.Decimal.html "struct optionstratlib::prelude::Decimal"){.struct} {#fn-round_toself-decimal_places-u32---decimal .code-header}
:::

::: docblock
Rounds the number to a specified number of decimal places. [Read
more](../utils/trait.ToRound.html#tymethod.round_to)
:::
:::::::

::: {#impl-ToSchema-for-Positive .section .impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#43){.src
.rightside}[§](#impl-ToSchema-for-Positive){.anchor}

### impl [ToSchema](../../../utoipa/trait.ToSchema.html "trait utoipa::ToSchema"){.trait} for [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#impl-toschema-for-positive .code-header}
:::

::::::: impl-items
::: {#method.name .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#43){.src
.rightside}[§](#method.name){.anchor}

#### fn [name](../../../utoipa/trait.ToSchema.html#method.name){.fn}() -\> [Cow](https://doc.rust-lang.org/1.91.1/alloc/borrow/enum.Cow.html "enum alloc::borrow::Cow"){.enum}\<\'static, [str](https://doc.rust-lang.org/1.91.1/std/primitive.str.html){.primitive}\> {#fn-name---cowstatic-str .code-header}
:::

::: docblock
Return name of the schema. [Read
more](../../../utoipa/trait.ToSchema.html#method.name)
:::

::: {#method.schemas .section .method .trait-impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#43){.src
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

::: {#impl-Copy-for-Positive .section .impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#43){.src
.rightside}[§](#impl-Copy-for-Positive){.anchor}

### impl [Copy](https://doc.rust-lang.org/1.91.1/core/marker/trait.Copy.html "trait core::marker::Copy"){.trait} for [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#impl-copy-for-positive .code-header}
:::

::: {#impl-Eq-for-Positive .section .impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#1199){.src
.rightside}[§](#impl-Eq-for-Positive){.anchor}

### impl [Eq](https://doc.rust-lang.org/1.91.1/core/cmp/trait.Eq.html "trait core::cmp::Eq"){.trait} for [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#impl-eq-for-positive .code-header}
:::

::: {#impl-StructuralPartialEq-for-Positive .section .impl}
[Source](../../../src/optionstratlib/model/positive.rs.html#43){.src
.rightside}[§](#impl-StructuralPartialEq-for-Positive){.anchor}

### impl [StructuralPartialEq](https://doc.rust-lang.org/1.91.1/core/marker/trait.StructuralPartialEq.html "trait core::marker::StructuralPartialEq"){.trait} for [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#impl-structuralpartialeq-for-positive .code-header}
:::
::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::

## Auto Trait Implementations[§](#synthetic-implementations){.anchor} {#synthetic-implementations .section-header}

::::::::: {#synthetic-implementations-list}
::: {#impl-Freeze-for-Positive .section .impl}
[§](#impl-Freeze-for-Positive){.anchor}

### impl [Freeze](https://doc.rust-lang.org/1.91.1/core/marker/trait.Freeze.html "trait core::marker::Freeze"){.trait} for [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#impl-freeze-for-positive .code-header}
:::

::: {#impl-RefUnwindSafe-for-Positive .section .impl}
[§](#impl-RefUnwindSafe-for-Positive){.anchor}

### impl [RefUnwindSafe](https://doc.rust-lang.org/1.91.1/core/panic/unwind_safe/trait.RefUnwindSafe.html "trait core::panic::unwind_safe::RefUnwindSafe"){.trait} for [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#impl-refunwindsafe-for-positive .code-header}
:::

::: {#impl-Send-for-Positive .section .impl}
[§](#impl-Send-for-Positive){.anchor}

### impl [Send](https://doc.rust-lang.org/1.91.1/core/marker/trait.Send.html "trait core::marker::Send"){.trait} for [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#impl-send-for-positive .code-header}
:::

::: {#impl-Sync-for-Positive .section .impl}
[§](#impl-Sync-for-Positive){.anchor}

### impl [Sync](https://doc.rust-lang.org/1.91.1/core/marker/trait.Sync.html "trait core::marker::Sync"){.trait} for [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#impl-sync-for-positive .code-header}
:::

::: {#impl-Unpin-for-Positive .section .impl}
[§](#impl-Unpin-for-Positive){.anchor}

### impl [Unpin](https://doc.rust-lang.org/1.91.1/core/marker/trait.Unpin.html "trait core::marker::Unpin"){.trait} for [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#impl-unpin-for-positive .code-header}
:::

::: {#impl-UnwindSafe-for-Positive .section .impl}
[§](#impl-UnwindSafe-for-Positive){.anchor}

### impl [UnwindSafe](https://doc.rust-lang.org/1.91.1/core/panic/unwind_safe/trait.UnwindSafe.html "trait core::panic::unwind_safe::UnwindSafe"){.trait} for [Positive](struct.Positive.html "struct optionstratlib::model::positive::Positive"){.struct} {#impl-unwindsafe-for-positive .code-header}
:::
:::::::::

## Blanket Implementations[§](#blanket-implementations){.anchor} {#blanket-implementations .section-header}

:::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::: {#blanket-implementations-list}
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

:::: {#impl-Comparable%3CK%3E-for-Q .section .impl}
[Source](../../../src/equivalent/lib.rs.html#104-107){.src
.rightside}[§](#impl-Comparable%3CK%3E-for-Q){.anchor}

### impl\<Q, K\> [Comparable](../../../equivalent/trait.Comparable.html "trait equivalent::Comparable"){.trait}\<K\> for Q {#implq-k-comparablek-for-q .code-header}

::: where
where Q:
[Ord](https://doc.rust-lang.org/1.91.1/core/cmp/trait.Ord.html "trait core::cmp::Ord"){.trait} +
?[Sized](https://doc.rust-lang.org/1.91.1/core/marker/trait.Sized.html "trait core::marker::Sized"){.trait},
K:
[Borrow](https://doc.rust-lang.org/1.91.1/core/borrow/trait.Borrow.html "trait core::borrow::Borrow"){.trait}\<Q\> +
?[Sized](https://doc.rust-lang.org/1.91.1/core/marker/trait.Sized.html "trait core::marker::Sized"){.trait},
:::
::::

::::: impl-items
::: {#method.compare .section .method .trait-impl}
[Source](../../../src/equivalent/lib.rs.html#110){.src
.rightside}[§](#method.compare){.anchor}

#### fn [compare](../../../equivalent/trait.Comparable.html#tymethod.compare){.fn}(&self, key: [&K](https://doc.rust-lang.org/1.91.1/std/primitive.reference.html){.primitive}) -\> [Ordering](https://doc.rust-lang.org/1.91.1/core/cmp/enum.Ordering.html "enum core::cmp::Ordering"){.enum} {#fn-compareself-key-k---ordering .code-header}
:::

::: docblock
Compare self to `key` and return their ordering.
:::
:::::

:::: {#impl-Equivalent%3CK%3E-for-Q .section .impl}
[Source](../../../src/hashbrown/lib.rs.html#167-170){.src
.rightside}[§](#impl-Equivalent%3CK%3E-for-Q){.anchor}

### impl\<Q, K\> [Equivalent](../../../hashbrown/trait.Equivalent.html "trait hashbrown::Equivalent"){.trait}\<K\> for Q {#implq-k-equivalentk-for-q .code-header}

::: where
where Q:
[Eq](https://doc.rust-lang.org/1.91.1/core/cmp/trait.Eq.html "trait core::cmp::Eq"){.trait} +
?[Sized](https://doc.rust-lang.org/1.91.1/core/marker/trait.Sized.html "trait core::marker::Sized"){.trait},
K:
[Borrow](https://doc.rust-lang.org/1.91.1/core/borrow/trait.Borrow.html "trait core::borrow::Borrow"){.trait}\<Q\> +
?[Sized](https://doc.rust-lang.org/1.91.1/core/marker/trait.Sized.html "trait core::marker::Sized"){.trait},
:::
::::

::::: impl-items
::: {#method.equivalent .section .method .trait-impl}
[Source](../../../src/hashbrown/lib.rs.html#172){.src
.rightside}[§](#method.equivalent){.anchor}

#### fn [equivalent](../../../hashbrown/trait.Equivalent.html#tymethod.equivalent){.fn}(&self, key: [&K](https://doc.rust-lang.org/1.91.1/std/primitive.reference.html){.primitive}) -\> [bool](https://doc.rust-lang.org/1.91.1/std/primitive.bool.html){.primitive} {#fn-equivalentself-key-k---bool .code-header}
:::

::: docblock
Checks if this value is equivalent to the given key. [Read
more](../../../hashbrown/trait.Equivalent.html#tymethod.equivalent)
:::
:::::

:::: {#impl-Equivalent%3CK%3E-for-Q-1 .section .impl}
[Source](../../../src/equivalent/lib.rs.html#82-85){.src
.rightside}[§](#impl-Equivalent%3CK%3E-for-Q-1){.anchor}

### impl\<Q, K\> [Equivalent](../../../equivalent/trait.Equivalent.html "trait equivalent::Equivalent"){.trait}\<K\> for Q {#implq-k-equivalentk-for-q-1 .code-header}

::: where
where Q:
[Eq](https://doc.rust-lang.org/1.91.1/core/cmp/trait.Eq.html "trait core::cmp::Eq"){.trait} +
?[Sized](https://doc.rust-lang.org/1.91.1/core/marker/trait.Sized.html "trait core::marker::Sized"){.trait},
K:
[Borrow](https://doc.rust-lang.org/1.91.1/core/borrow/trait.Borrow.html "trait core::borrow::Borrow"){.trait}\<Q\> +
?[Sized](https://doc.rust-lang.org/1.91.1/core/marker/trait.Sized.html "trait core::marker::Sized"){.trait},
:::
::::

::::: impl-items
::: {#method.equivalent-1 .section .method .trait-impl}
[Source](../../../src/equivalent/lib.rs.html#88){.src
.rightside}[§](#method.equivalent-1){.anchor}

#### fn [equivalent](../../../equivalent/trait.Equivalent.html#tymethod.equivalent){.fn}(&self, key: [&K](https://doc.rust-lang.org/1.91.1/std/primitive.reference.html){.primitive}) -\> [bool](https://doc.rust-lang.org/1.91.1/std/primitive.bool.html){.primitive} {#fn-equivalentself-key-k---bool-1 .code-header}
:::

::: docblock
Compare self to `key` and return `true` if they are equal.
:::
:::::

::: {#impl-From%3CT%3E-for-T .section .impl}
[Source](https://doc.rust-lang.org/1.91.1/src/core/convert/mod.rs.html#785){.src
.rightside}[§](#impl-From%3CT%3E-for-T){.anchor}

### impl\<T\> [From](https://doc.rust-lang.org/1.91.1/core/convert/trait.From.html "trait core::convert::From"){.trait}\<T\> for T {#implt-fromt-for-t .code-header}
:::

::::: impl-items
::: {#method.from-16 .section .method .trait-impl}
[Source](https://doc.rust-lang.org/1.91.1/src/core/convert/mod.rs.html#788){.src
.rightside}[§](#method.from-16){.anchor}

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
::: {#associatedtype.Output-28 .section .associatedtype .trait-impl}
[Source](../../../src/typenum/type_operators.rs.html#35){.src
.rightside}[§](#associatedtype.Output-28){.anchor}

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

:::: {#impl-ClosedAdd%3CRight%3E-for-T .section .impl}
[Source](../../../src/simba/scalar/field.rs.html#32){.src
.rightside}[§](#impl-ClosedAdd%3CRight%3E-for-T){.anchor}

### impl\<T, Right\> [ClosedAdd](../../../simba/scalar/field/trait.ClosedAdd.html "trait simba::scalar::field::ClosedAdd"){.trait}\<Right\> for T {#implt-right-closedaddright-for-t .code-header}

::: where
where T:
[Add](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.Add.html "trait core::ops::arith::Add"){.trait}\<Right,
Output = T\> +
[AddAssign](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.AddAssign.html "trait core::ops::arith::AddAssign"){.trait}\<Right\>,
:::
::::

:::: {#impl-ClosedAddAssign%3CRight%3E-for-T .section .impl}
[Source](../../../src/simba/scalar/field.rs.html#42){.src
.rightside}[§](#impl-ClosedAddAssign%3CRight%3E-for-T){.anchor}

### impl\<T, Right\> [ClosedAddAssign](../../../simba/scalar/field/trait.ClosedAddAssign.html "trait simba::scalar::field::ClosedAddAssign"){.trait}\<Right\> for T {#implt-right-closedaddassignright-for-t .code-header}

::: where
where T:
[ClosedAdd](../../../simba/scalar/field/trait.ClosedAdd.html "trait simba::scalar::field::ClosedAdd"){.trait}\<Right\> +
[AddAssign](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.AddAssign.html "trait core::ops::arith::AddAssign"){.trait}\<Right\>,
:::
::::

:::: {#impl-ClosedMul%3CRight%3E-for-T .section .impl}
[Source](../../../src/simba/scalar/field.rs.html#36){.src
.rightside}[§](#impl-ClosedMul%3CRight%3E-for-T){.anchor}

### impl\<T, Right\> [ClosedMul](../../../simba/scalar/field/trait.ClosedMul.html "trait simba::scalar::field::ClosedMul"){.trait}\<Right\> for T {#implt-right-closedmulright-for-t .code-header}

::: where
where T:
[Mul](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.Mul.html "trait core::ops::arith::Mul"){.trait}\<Right,
Output = T\> +
[MulAssign](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.MulAssign.html "trait core::ops::arith::MulAssign"){.trait}\<Right\>,
:::
::::

:::: {#impl-ClosedMulAssign%3CRight%3E-for-T .section .impl}
[Source](../../../src/simba/scalar/field.rs.html#46){.src
.rightside}[§](#impl-ClosedMulAssign%3CRight%3E-for-T){.anchor}

### impl\<T, Right\> [ClosedMulAssign](../../../simba/scalar/field/trait.ClosedMulAssign.html "trait simba::scalar::field::ClosedMulAssign"){.trait}\<Right\> for T {#implt-right-closedmulassignright-for-t .code-header}

::: where
where T:
[ClosedMul](../../../simba/scalar/field/trait.ClosedMul.html "trait simba::scalar::field::ClosedMul"){.trait}\<Right\> +
[MulAssign](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.MulAssign.html "trait core::ops::arith::MulAssign"){.trait}\<Right\>,
:::
::::

:::: {#impl-ClosedNeg-for-T .section .impl}
[Source](../../../src/simba/scalar/field.rs.html#40){.src
.rightside}[§](#impl-ClosedNeg-for-T){.anchor}

### impl\<T\> [ClosedNeg](../../../simba/scalar/field/trait.ClosedNeg.html "trait simba::scalar::field::ClosedNeg"){.trait} for T {#implt-closedneg-for-t .code-header}

::: where
where T:
[Neg](https://doc.rust-lang.org/1.91.1/core/ops/arith/trait.Neg.html "trait core::ops::arith::Neg"){.trait}\<Output
= T\>,
:::
::::

:::: {#impl-DeserializeOwned-for-T .section .impl}
[Source](../../../src/serde_core/de/mod.rs.html#633){.src
.rightside}[§](#impl-DeserializeOwned-for-T){.anchor}

### impl\<T\> [DeserializeOwned](../../../serde_core/de/trait.DeserializeOwned.html "trait serde_core::de::DeserializeOwned"){.trait} for T {#implt-deserializeowned-for-t .code-header}

::: where
where T: for\<\'de\>
[Deserialize](../../../serde_core/de/trait.Deserialize.html "trait serde_core::de::Deserialize"){.trait}\<\'de\>,
:::
::::

:::: {#impl-Scalar-for-T .section .impl}
[Source](../../../src/nalgebra/base/scalar.rs.html#8){.src
.rightside}[§](#impl-Scalar-for-T){.anchor}

### impl\<T\> [Scalar](../../../nalgebra/base/scalar/trait.Scalar.html "trait nalgebra::base::scalar::Scalar"){.trait} for T {#implt-scalar-for-t .code-header}

::: where
where T: \'static +
[Clone](https://doc.rust-lang.org/1.91.1/core/clone/trait.Clone.html "trait core::clone::Clone"){.trait} +
[PartialEq](https://doc.rust-lang.org/1.91.1/core/cmp/trait.PartialEq.html "trait core::cmp::PartialEq"){.trait} +
[Debug](https://doc.rust-lang.org/1.91.1/core/fmt/trait.Debug.html "trait core::fmt::Debug"){.trait},
:::
::::
::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
:::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
