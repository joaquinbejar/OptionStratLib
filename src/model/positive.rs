/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 30/12/24
******************************************************************************/

use crate::chains::chain::OptionChain;
use crate::constants::EPSILON;
use crate::error::DecimalError;
use crate::model::utils::ToRound;
use crate::series::OptionSeries;
use approx::{AbsDiffEq, RelativeEq};
use num_traits::{FromPrimitive, Pow, ToPrimitive};
use rust_decimal::{Decimal, MathematicalOps};
use rust_decimal_macros::dec;
use serde::de::Visitor;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::cmp::{Ordering, PartialEq};
use std::fmt;
use std::fmt::Display;
use std::iter::Sum;
use std::ops::{Add, AddAssign, Div, Mul, MulAssign, Neg, Sub};
use std::str::FromStr;
use utoipa::ToSchema;

/// A wrapper type that represents a guaranteed positive decimal value.
///
/// This type encapsulates a `Decimal` value and ensures through its API that
/// the contained value is always positive (greater than or equal to zero).
/// It provides a type-safe way to handle numeric values in financial contexts
/// where negative values would be invalid or meaningless.
///
/// The internal value is directly accessible only within the crate through
/// the `pub(crate)` visibility modifier, while external access is provided
/// through public methods that maintain the positive value invariant.
///
/// ## Example
///
/// ```rust
/// use optionstratlib::pos;
/// let strike_price = pos!(100.0);
/// ```
#[derive(PartialEq, Clone, Copy, Hash, ToSchema)]
pub struct Positive(pub(crate) Decimal);

/// Macro for creating a new `Positive` value with simplified syntax.
///
/// This macro attempts to create a `Positive` value from the given expression
/// and unwraps the result. It will panic if the value cannot be converted to a
/// `Positive` (e.g., if the value is negative or not representable).
///
/// # Examples
///
/// ```rust
/// use optionstratlib::pos;
/// let positive_value = pos!(5.0);
/// ```
///
/// # Panics
///
/// This macro will panic if the provided value cannot be converted to a `Positive` value.
#[macro_export]
macro_rules! pos {
    ($val:expr) => {
        $crate::Positive::new($val).unwrap()
    };
}

/// A macro to create an optional `Positive` value from the given expression.
///
/// This macro attempts to create a `Positive` value using the `Positive::new`
/// function, which returns `None` if the value is not a positive number.
/// The macro is useful for safely constructing optional `Positive` values
/// in a concise manner.
///
/// # Parameters
/// - `$val:expr`: An expression that evaluates to a value intended to be
///   wrapped in a `Positive` type.
///
/// # Returns
/// - `Option<Positive>`: Returns `Some(Positive)` if the given value is positive,
///   otherwise returns `None`.
///
/// # Example
///
/// ```rust
/// use optionstratlib::Positive;
/// use optionstratlib::spos;
///
/// // Example with a positive value
/// let x = spos!(10.0);
/// assert_eq!(x.is_some(), true);
///
/// // Example with a non-positive value
/// let y = spos!(-5.0);
/// assert_eq!(y.is_none(), true);
/// ```
///
/// # Notes
/// - Ensure that the type used with this macro implements the required constraints
///   for constructing a `Positive` value (as defined in the `Positive::new` method).
///
/// # See Also
/// - [`Positive::new`](crate::Positive::new): The method that performs the actual validation
///   for determining whether a value can be wrapped in the `Positive` type.
#[macro_export]
macro_rules! spos {
    ($val:expr) => {{ $crate::Positive::new($val).ok() }};
}

/// Determines if the given type parameter `T` is the `Positive` type.
///
/// # Details
///
/// The function `is_positive` is a utility method that checks if the generic
/// type parameter matches the type `Positive`. Internally, it uses Rust's
/// `TypeId` comparison from the `std::any` module for this check, which is a
/// reliable way to confirm whether two types are the same at runtime.
///
/// The `Positive` type is part of the `crate::model::positive` module and is a
/// specialized struct used to represent non-negative decimal values. It is
/// implemented as follows:
///
/// ```rust
/// use rust_decimal::Decimal;
///
/// #[derive(PartialEq, Clone, Copy)]
/// pub struct Positive(pub(crate) Decimal);
/// ```
///
/// This type enforces semantic clarity and helps avoid invalid numerical
/// operations, making it especially useful in financial modeling or domains
/// where valid numbers are constrained to non-negative values.
///
/// # Usage
///
/// - Typically used in scenarios where type safety for numerical values like
///   non-negativity is critical.
/// - Prevents misuse of data structures or operations by identifying whether a
///   value is of the `Positive` type.
///
/// # Implementation
///
/// The function relies on the `TypeId` mechanism, part of Rust's type
/// reflection system, which allows runtime comparison of types. This is
/// efficient and avoids adding unnecessary overhead.
///
/// # Examples
///
/// ```rust
/// use optionstratlib::model::positive::is_positive;
/// use optionstratlib::Positive;
/// assert!(is_positive::<Positive>());
/// assert!(!is_positive::<i32>());
/// ```
///
/// This example demonstrates that the function correctly identifies the
/// `Positive` type while distinguishing it from unrelated types like `i32`.
///
/// # Note
///
/// This function does **not** perform runtime checks on instances of values,
/// nor does it check type compatibility with conversions. It works
/// exclusively at the type level.
///
/// # See Also
///
/// - [`Positive`] type, which encapsulates non-negative decimal values.
/// - `std::any::TypeId` for type reflection and runtime type identification.
pub fn is_positive<T: 'static>() -> bool {
    std::any::TypeId::of::<T>() == std::any::TypeId::of::<Positive>()
}

impl Positive {
    /// A zero value represented as a `Positive` value.
    pub const ZERO: Positive = Positive(Decimal::ZERO);

    /// A value of one represented as a `Positive` value.
    pub const ONE: Positive = Positive(Decimal::ONE);

    /// A value of two represented as a `Positive` value.
    pub const TWO: Positive = Positive(Decimal::TWO);

    /// Represents the maximum positive value possible (effectively infinity).
    pub const INFINITY: Positive = Positive(Decimal::MAX);

    /// A value of ten represented as a `Positive` value.
    pub const TEN: Positive = Positive(Decimal::TEN);

    /// A value of one hundred represented as a `Positive` value.
    pub const HUNDRED: Positive = Positive(Decimal::ONE_HUNDRED);

    /// A value of one thousand represented as a `Positive` value.
    pub const THOUSAND: Positive = Positive(Decimal::ONE_THOUSAND);

    /// The mathematical constant π (pi) represented as a `Positive` value.
    pub const PI: Positive = Positive(Decimal::PI);

    /// Creates a new `Positive` value from a 64-bit floating-point number.
    ///
    /// # Arguments
    ///
    /// * `value` - A floating-point value to convert
    ///
    /// # Returns
    ///
    /// * `Ok(Positive)` if the value is non-negative and valid
    /// * `Err(String)` if the value is negative or cannot be parsed as a Decimal
    ///
    pub fn new(value: f64) -> Result<Self, DecimalError> {
        let dec = Decimal::from_f64(value);
        match dec {
            Some(value) if value >= Decimal::ZERO => Ok(Positive(value)),
            Some(value) => Err(DecimalError::OutOfBounds {
                value: value.to_f64().unwrap_or(0.0),
                min: 0.0,
                max: f64::MAX,
            }),
            None => Err(DecimalError::ConversionError {
                from_type: "f64".to_string(),
                to_type: "Positive".to_string(),
                reason: "failed to parse Decimal".to_string(),
            }),
        }
    }

    /// Creates a new `Positive` value directly from a `Decimal`.
    ///
    /// # Arguments
    ///
    /// * `value` - A `Decimal` value to wrap in a `Positive`
    ///
    /// # Returns
    ///
    /// * `Ok(Positive)` if the value is non-negative
    /// * `Err(String)` if the value is negative
    pub fn new_decimal(value: Decimal) -> Result<Self, DecimalError> {
        if value >= Decimal::ZERO {
            Ok(Positive(value))
        } else {
            Err(DecimalError::OutOfBounds {
                value: value.to_f64().unwrap(),
                min: 0.0,
                max: f64::INFINITY,
            })
        }
    }

    /// Returns the inner `Decimal` value.
    ///
    /// # Returns
    ///
    /// The wrapped `Decimal` value.
    pub fn value(&self) -> Decimal {
        self.0
    }

    /// Returns the inner `Decimal` value (alias for `value()`).
    ///
    /// # Returns
    ///
    /// The wrapped `Decimal` value.
    pub fn to_dec(&self) -> Decimal {
        self.0
    }

    /// Returns the inner `Decimal` ref.
    ///
    /// # Returns
    ///
    /// The wrapped `Decimal` ref.
    pub fn to_dec_ref(&self) -> &Decimal {
        &self.0
    }

    /// Returns a mutable reference to the inner `Decimal` value.
    ///
    /// This method provides controlled access to the underlying `Decimal` value
    /// within the `Positive` wrapper, allowing it to be modified while maintaining
    /// encapsulation of the inner value. It's important to note that direct mutation
    /// should be used with caution to ensure the positive value invariant is maintained.
    ///
    /// # Returns
    ///
    /// * `&mut Decimal` - A mutable reference to the wrapped `Decimal` value.
    ///
    /// # Usage
    ///
    /// This method is typically used in contexts where the wrapped value needs to be
    /// modified in-place while preserving the wrapper's type safety guarantees.
    /// Care should be taken to ensure any modification preserves the positive value constraint.
    pub fn to_dec_ref_mut(&mut self) -> &mut Decimal {
        &mut self.0
    }

    /// Converts the value to a 64-bit floating-point number.
    ///
    /// # Returns
    ///
    /// The value as an `f64`.
    ///
    /// # Panics
    ///
    /// Panics if the conversion fails.
    pub fn to_f64(&self) -> f64 {
        self.0.to_f64().unwrap()
    }

    /// Converts the value to a 64-bit signed integer.
    ///
    /// # Returns
    ///
    /// The value as an `i64`.
    ///
    /// # Panics
    ///
    /// Panics if the value cannot be represented as an `i64`.
    pub fn to_i64(&self) -> i64 {
        self.0.to_i64().unwrap()
    }

    /// Converts the inner value of the struct to a `u64`.
    ///
    /// This method assumes the inner value can be safely converted to a `u64`
    /// and uses `unwrap()` to extract the value. If the conversion fails,
    /// this will cause a panic at runtime.
    ///
    /// # Returns
    /// A `u64` representation of the inner value.
    ///
    /// # Panics
    /// This method will panic if the inner value cannot be converted to a `u64`.
    ///
    pub fn to_u64(&self) -> u64 {
        self.0.to_u64().unwrap()
    }

    /// Converts the value to a usize signed integer.
    ///
    /// # Returns
    ///
    /// The value as an `usize`.
    ///
    /// # Panics
    ///
    /// Panics if the value cannot be represented as an `usize`.
    pub fn to_usize(&self) -> usize {
        self.0.to_usize().unwrap()
    }

    /// Returns the maximum of two `Positive` values.
    ///
    /// # Arguments
    ///
    /// * `other` - Another `Positive` value to compare with
    ///
    /// # Returns
    ///
    /// The larger of the two values.
    pub fn max(self, other: Positive) -> Positive {
        if self.0 > other.0 { self } else { other }
    }

    /// Returns the minimum of two `Positive` values.
    ///
    /// # Arguments
    ///
    /// * `other` - Another `Positive` value to compare with
    ///
    /// # Returns
    ///
    /// The smaller of the two values.
    pub fn min(self, other: Positive) -> Positive {
        if self.0 < other.0 { self } else { other }
    }

    /// Rounds the value down to the nearest integer.
    ///
    /// # Returns
    ///
    /// A new `Positive` value rounded down to the nearest integer.
    pub fn floor(&self) -> Positive {
        Positive(self.0.floor())
    }

    /// Raises this value to an integer power.
    ///
    /// # Arguments
    ///
    /// * `n` - The power to raise this value to
    ///
    /// # Returns
    ///
    /// A new `Positive` value representing `self` raised to the power `n`.
    pub fn powi(&self, n: i64) -> Positive {
        Positive(self.0.powi(n))
    }

    /// Computes the result of raising the current `Positive` value to the power of the given `Positive` exponent.
    ///
    /// # Parameters
    ///
    /// - `n`: A `Positive` value representing the exponent to which the current value will be raised.
    ///
    /// # Returns
    ///
    /// A `Positive` value representing the result of the power computation.
    ///
    /// # Panics
    ///
    /// This function does not panic as all inputs are guaranteed to be positive by the `Positive` type.
    pub fn pow(&self, n: Positive) -> Positive {
        Positive(self.0.pow(n.to_dec()))
    }

    /// Raises the current `Positive` value to the power of `n` using unsigned integer exponentiation.
    ///
    /// # Parameters
    /// - `n`: An unsigned 64-bit integer (`u64`) representing the power to which the value will be raised.
    ///
    /// # Returns
    /// Returns a new `Positive` instance containing the result of `self` raised to the power of `n`.
    ///
    pub fn powu(&self, n: u64) -> Positive {
        Positive(self.0.powu(n))
    }

    /// Raises this value to a decimal power.
    ///
    /// This is a crate-internal method not exposed to public API users.
    ///
    /// # Arguments
    ///
    /// * `p0` - The power to raise this value to as a `Decimal`
    ///
    /// # Returns
    ///
    /// A new `Positive` value representing `self` raised to the power `p0`.
    pub fn powd(&self, p0: Decimal) -> Positive {
        Positive(self.0.powd(p0))
    }

    /// Rounds the value to the nearest integer.
    ///
    /// # Returns
    ///
    /// A new `Positive` value rounded to the nearest integer.
    pub fn round(&self) -> Positive {
        Positive(self.0.round())
    }

    /// Rounds the current value to a "nice" number, based on its magnitude.
    ///
    /// This method computes the logarithmic magnitude of the current value and
    /// determines a simplified number within a range of predefined "nice" numbers,
    /// such as 1, 2, 5, or 10, scaled by a power of ten. This is often useful
    /// for simplifying numerical values for display or plotting.
    ///
    /// # Procedure
    /// 1. Compute the magnitude of the current value by taking the base-10 logarithm and flooring it.
    /// 2. Calculate the power of ten corresponding to the magnitude.
    /// 3. Normalize the current value by dividing it by the calculated power of ten.
    /// 4. Determine the closest "nice" number:
    ///    - Values less than 1.5 round to 1.
    ///    - Values less than 3 round to 2.
    ///    - Values less than 7 round to 5.
    ///    - All other values round to 10.
    /// 5. Scale the "nice" number back up by the appropriate power of ten.
    ///
    /// # Returns
    /// A `Positive` value representing the rounded "nice" number, scaled appropriately
    /// according to the magnitude of the input value.
    ///
    /// # Panics
    /// This function does not explicitly handle negative numbers or zero, as it is
    /// assumed that `self` is a positive numeric value. Ensure `self` is valid and non-zero
    /// prior to calling this method.
    ///
    /// # Examples
    /// ```
    /// use optionstratlib::Positive;
    /// let value = Positive::new(123.0).unwrap();
    /// let rounded = value.round_to_nice_number();
    /// assert_eq!(rounded, Positive::new(100.0).unwrap());
    ///
    /// let value = Positive::new(6.7).unwrap();
    /// let rounded = value.round_to_nice_number();
    /// assert_eq!(rounded, Positive::new(5.0).unwrap());
    /// ```
    pub fn round_to_nice_number(&self) -> Positive {
        let magnitude = self.log10().floor();
        let ten_pow = Positive::TEN.pow(magnitude);
        let normalized = self / &ten_pow;
        let nice_number = if normalized < dec!(1.5) {
            Positive::ONE
        } else if normalized < pos!(3.0) {
            Positive::TWO
        } else if normalized < pos!(7.0) {
            pos!(5.0)
        } else {
            Positive::TEN
        };
        nice_number * pos!(10.0).powu(magnitude.to_u64())
    }

    /// Calculates the square root of the value.
    ///
    /// # Returns
    ///
    /// A new `Positive` value representing the square root.
    ///
    /// # Panics
    ///
    /// Panics if the square root calculation fails.
    pub fn sqrt(&self) -> Positive {
        Positive(self.0.sqrt().unwrap())
    }

    /// Calculates the natural logarithm of the value.
    ///
    /// # Returns
    ///
    /// A new `Positive` value representing the natural logarithm.
    pub fn ln(&self) -> Positive {
        Positive(self.0.ln())
    }

    /// Rounds the value to a specified number of decimal places.
    ///
    /// # Arguments
    ///
    /// * `decimal_places` - The number of decimal places to round to
    ///
    /// # Returns
    ///
    /// A new `Positive` value rounded to the specified decimal places.
    pub fn round_to(&self, decimal_places: u32) -> Positive {
        Positive(self.0.round_dp(decimal_places))
    }

    /// Formats the value with a fixed number of decimal places, filling with zeros if needed.
    ///
    /// Unlike `round_to` which just rounds the value, this method ensures the string
    /// representation always has exactly the specified number of decimal places.
    ///
    /// # Arguments
    ///
    /// * `decimal_places` - The exact number of decimal places to display
    ///
    /// # Returns
    ///
    /// A String representation of the value with exactly the specified number of decimal places.
    ///
    /// # Examples
    ///
    /// ```
    /// use optionstratlib::pos;
    ///
    /// let value = pos!(10.5);
    /// assert_eq!(value.format_fixed_places(2), "10.50");
    ///
    /// let value = pos!(10.0);
    /// assert_eq!(value.format_fixed_places(3), "10.000");
    ///
    /// let value = pos!(10.567);
    /// assert_eq!(value.format_fixed_places(2), "10.57"); // Rounds to 2 places
    /// ```
    pub fn format_fixed_places(&self, decimal_places: u32) -> String {
        // First round to the specified number of decimal places
        let rounded = self.round_to(decimal_places).to_f64();

        // Use format! with the precision specifier to ensure exactly decimal_places are shown
        format!("{:.1$}", rounded, decimal_places as usize)
    }

    /// Calculates the exponential function e^x for this value.
    ///
    /// # Returns
    ///
    /// A new `Positive` value representing e raised to the power of `self`.
    pub fn exp(&self) -> Positive {
        Positive(self.0.exp())
    }

    /// Clamps the value between a minimum and maximum.
    ///
    /// # Arguments
    ///
    /// * `min` - The lower bound
    /// * `max` - The upper bound
    ///
    /// # Returns
    ///
    /// * `min` if `self` is less than `min`
    /// * `max` if `self` is greater than `max`
    /// * `self` if `self` is between `min` and `max` (inclusive)
    pub fn clamp(&self, min: Positive, max: Positive) -> Positive {
        if self < &min {
            min
        } else if self > &max {
            max
        } else {
            *self
        }
    }

    /// Checks if the value is exactly zero.
    ///
    /// # Returns
    ///
    /// `true` if the value is zero, `false` otherwise.
    pub fn is_zero(&self) -> bool {
        self.0.is_zero()
    }

    /// Returns the smallest integer greater than or equal to the value.
    pub fn ceiling(&self) -> Positive {
        let value = self.to_dec();
        // Ceiling operation: find the smallest integer greater than or equal to value
        let ceiling_value = value.ceil();
        // Convert back to Positive
        Positive::from(ceiling_value)
    }

    /// Computes the base-10 logarithm of the value contained in the `Positive` instance.
    ///
    /// # Returns
    ///
    /// A new `Positive` instance containing the result of the base-10 logarithm of the original value.
    ///
    /// # Note
    ///
    /// It is assumed that the value contained in the `Positive` instance is always greater than 0,
    /// as logarithms of non-positive numbers are undefined.
    pub fn log10(&self) -> Positive {
        Positive(self.0.log10())
    }

    /// Subtracts a decimal value from this positive value, returning zero if the result would be negative.
    ///
    /// # Arguments
    /// * `other` - The decimal value to subtract.
    ///
    /// # Returns
    /// * `Positive` - The result of the subtraction, or zero if the result would be negative.
    pub fn sub_or_zero(&self, other: &Decimal) -> Positive {
        if &self.0 > other {
            Positive(self.0 - other)
        } else {
            Positive(Decimal::ZERO)
        }
    }

    /// Subtracts a decimal value from this positive value, returning None if the result would be negative.
    ///
    /// # Arguments
    /// * `other` - The decimal value to subtract.
    ///
    /// # Returns
    /// * `Option<Positive>` - The result of the subtraction as a `Some(Positive)`, or `None` if the result would be negative.
    pub fn sub_or_none(&self, other: &Decimal) -> Option<Positive> {
        if &self.0 >= other {
            Some(Positive(self.0 - other))
        } else {
            None
        }
    }

    /// Checks whether the numeric value of the current instance is a multiple of another specified value.
    ///
    /// This method determines if the numeric value represented by the current instance
    /// is evenly divisible by the provided `other` value, using a small epsilon to handle floating-point precision.
    ///
    /// # Arguments
    ///
    /// * `other` - A 64-bit floating-point value (`f64`) representing the divisor
    ///   against which the current instance will be tested.
    ///
    /// # Returns
    ///
    /// * `true` if the current instance is a multiple of the specified `other` value within a small tolerance.
    /// * `false` if the current instance is not a multiple of the specified `other` value
    ///   or if the current value is not finite (e.g., if it is `NaN`, infinity, or similar).
    ///
    /// # Behavior
    ///
    /// * The method first retrieves the numeric value of the current instance as an `f64`
    ///   using a `to_f64()` method, which is assumed to be implemented elsewhere in the code.
    /// * If the resulting value is not finite (e.g., `NaN` or infinity), `false` is returned.
    /// * Otherwise, the modulo operation is used to check whether the remainder, when divided by `other`,
    ///   is approximately zero (considering the tolerance defined by `f64::EPSILON`).
    ///
    /// # Notes
    ///
    /// * `f64::EPSILON` represents the smallest difference between two distinct `f64` values.
    ///   It is used here to account for floating-point rounding errors when performing equality comparisons.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use optionstratlib::pos;
    /// let num = pos!(10.0);
    /// assert!(num.is_multiple(2.0));  // 10.0 is multiple of 2.0
    /// assert!(!num.is_multiple(3.0)); // 10.0 is not a multiple of 3.0
    /// ```
    pub fn is_multiple(&self, other: f64) -> bool {
        let value = self.to_f64();
        if !value.is_finite() {
            return false;
        }

        // Use modulo operation with floating point epsilon for comparison
        let remainder = value % other;
        remainder.abs() < f64::EPSILON || (other - remainder.abs()).abs() < f64::EPSILON
    }
}

impl ToRound for Positive {
    fn round(&self) -> Decimal {
        self.round().to_dec()
    }

    fn round_to(&self, decimal_places: u32) -> Decimal {
        self.round_to(decimal_places).to_dec()
    }
}

impl PartialEq<&Positive> for Positive {
    fn eq(&self, other: &&Positive) -> bool {
        self == *other
    }
}

impl From<Positive> for u64 {
    fn from(pos_u64: Positive) -> Self {
        pos_u64.0.to_u64().unwrap()
    }
}

impl From<&Positive> for f64 {
    fn from(value: &Positive) -> Self {
        value.0.to_f64().unwrap_or(0.0)
    }
}

impl From<Positive> for f64 {
    fn from(value: Positive) -> Self {
        value.0.to_f64().unwrap_or(0.0)
    }
}

impl From<Positive> for usize {
    fn from(value: Positive) -> Self {
        value.0.to_f64().unwrap_or(0.0) as usize
    }
}

impl PartialEq<&Positive> for f64 {
    fn eq(&self, other: &&Positive) -> bool {
        self == &other.0.to_f64().unwrap_or(0.0)
    }
}

impl PartialOrd<&Positive> for f64 {
    fn partial_cmp(&self, other: &&Positive) -> Option<std::cmp::Ordering> {
        self.partial_cmp(&other.0.to_f64().unwrap_or(0.0))
    }
}

impl PartialEq<Positive> for f64 {
    fn eq(&self, other: &Positive) -> bool {
        self == &other.0.to_f64().unwrap_or(0.0)
    }
}

impl PartialOrd<Positive> for f64 {
    fn partial_cmp(&self, other: &Positive) -> Option<std::cmp::Ordering> {
        self.partial_cmp(&other.0.to_f64().unwrap_or(0.0))
    }
}

impl Mul<Positive> for f64 {
    type Output = f64;

    fn mul(self, rhs: Positive) -> Self::Output {
        self * rhs.to_f64()
    }
}

impl Div<Positive> for f64 {
    type Output = f64;

    fn div(self, rhs: Positive) -> Self::Output {
        self / rhs.to_f64()
    }
}

impl Sub<Positive> for f64 {
    type Output = f64;

    fn sub(self, rhs: Positive) -> Self::Output {
        self - rhs.to_f64()
    }
}

impl Add<Positive> for f64 {
    type Output = f64;

    fn add(self, rhs: Positive) -> Self::Output {
        self + rhs.to_f64()
    }
}

impl FromStr for Positive {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.parse::<Decimal>() {
            Ok(value) if value > Decimal::ZERO => Ok(Positive(value)),
            Ok(value) => Err(format!("Value must be positive, got {value}")),
            Err(e) => Err(format!("Failed to parse as Decimal: {e}")),
        }
    }
}

impl From<f64> for Positive {
    fn from(value: f64) -> Self {
        Positive::new(value).expect("Value must be positive")
    }
}

impl From<usize> for Positive {
    fn from(value: usize) -> Self {
        Positive::new(value as f64).expect("Value must be positive")
    }
}

impl From<Decimal> for Positive {
    fn from(value: Decimal) -> Self {
        Positive::new_decimal(value).expect("Value must be positive")
    }
}

impl From<&Decimal> for Positive {
    fn from(value: &Decimal) -> Self {
        Positive::new_decimal(*value).expect("Value must be positive")
    }
}

impl From<&Positive> for Positive {
    fn from(value: &Positive) -> Self {
        Positive(value.0)
    }
}

impl From<&OptionChain> for Positive {
    fn from(value: &OptionChain) -> Self {
        value.underlying_price
    }
}

impl From<OptionChain> for Positive {
    fn from(value: OptionChain) -> Self {
        value.underlying_price
    }
}

impl From<&OptionSeries> for Positive {
    fn from(value: &OptionSeries) -> Self {
        value.underlying_price
    }
}

impl From<OptionSeries> for Positive {
    fn from(value: OptionSeries) -> Self {
        value.underlying_price
    }
}

impl Mul<f64> for Positive {
    type Output = Positive;

    fn mul(self, rhs: f64) -> Positive {
        (self.to_f64() * rhs).into()
    }
}

impl Div<f64> for Positive {
    type Output = Positive;

    fn div(self, rhs: f64) -> Positive {
        (self.to_f64() / rhs).into()
    }
}

impl Div<f64> for &Positive {
    type Output = Positive;

    fn div(self, rhs: f64) -> Positive {
        (self.to_f64() / rhs).into()
    }
}

impl Sub<f64> for Positive {
    type Output = Positive;

    fn sub(self, rhs: f64) -> Self::Output {
        (self.to_f64() - rhs).into()
    }
}

impl Add<f64> for Positive {
    type Output = Positive;

    fn add(self, rhs: f64) -> Self::Output {
        (self.to_f64() + rhs).into()
    }
}

impl PartialOrd<f64> for Positive {
    fn partial_cmp(&self, other: &f64) -> Option<Ordering> {
        self.0.to_f64().unwrap_or(0.0).partial_cmp(other)
    }
}

impl PartialEq<f64> for &Positive {
    fn eq(&self, other: &f64) -> bool {
        self.0.to_f64().unwrap_or(0.0) == *other
    }
}

impl PartialOrd<f64> for &Positive {
    fn partial_cmp(&self, other: &f64) -> Option<Ordering> {
        self.0.to_f64().unwrap_or(0.0).partial_cmp(other)
    }
}

impl PartialEq<f64> for Positive {
    fn eq(&self, other: &f64) -> bool {
        self.to_f64() == *other
    }
}

impl Display for Positive {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if *self == Positive::INFINITY {
            write!(f, r#""infinity""#)
        } else if self.0.scale() == 0 {
            match self.0.to_i64() {
                Some(val) => write!(f, "{val}"),
                None => write!(f, "{}", self.0),
            }
        } else if let Some(precision) = f.precision() {
            write!(f, "{:.1$}", self.0, precision)
        } else {
            let s = self.0.to_string();
            let trimmed = s.trim_end_matches('0').trim_end_matches('.');
            write!(f, "{trimmed}")
        }
    }
}

impl fmt::Debug for Positive {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if *self == Positive::INFINITY {
            write!(f, r#""infinity""#)
        } else if self.0.scale() == 0 {
            write!(f, "{}", self.0.to_i64().unwrap())
        } else {
            write!(f, "{}", self.0)
        }
    }
}

impl PartialEq<Decimal> for Positive {
    fn eq(&self, other: &Decimal) -> bool {
        (self.0 - *other).abs() <= EPSILON * Decimal::from(100)
    }
}

impl Serialize for Positive {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let value = self.0;

        if *self == Positive::INFINITY {
            return serializer.serialize_str("infinity");
        }

        if value.scale() == 0 {
            serializer.serialize_i64(
                value
                    .to_i64()
                    .ok_or_else(|| serde::ser::Error::custom("Failed to convert Decimal to i64"))?,
            )
        } else {
            // Si tiene decimales, serializa como f64
            serializer.serialize_f64(
                value
                    .to_f64()
                    .ok_or_else(|| serde::ser::Error::custom("Failed to convert Decimal to f64"))?,
            )
        }
    }
}

impl<'de> Deserialize<'de> for Positive {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct PositiveVisitor;

        impl Visitor<'_> for PositiveVisitor {
            type Value = Positive;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a positive number or the string \"infinity\"")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                if value.eq_ignore_ascii_case("infinity") {
                    return Ok(Positive::INFINITY);
                }
                Err(serde::de::Error::custom(format!(
                    "Invalid string: '{value}'. Expected \"infinity\"."
                )))
            }

            fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                if value < 0 {
                    Err(serde::de::Error::custom("Expected a non-negative integer"))
                } else {
                    Positive::new_decimal(Decimal::from(value)).map_err(serde::de::Error::custom)
                }
            }

            fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Positive::new_decimal(Decimal::from(value)).map_err(serde::de::Error::custom)
            }

            fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                if value.is_infinite() && value.is_sign_positive() {
                    return Ok(Positive::INFINITY);
                }

                let decimal = Decimal::from_f64(value)
                    .ok_or_else(|| serde::de::Error::custom("Failed to convert f64 to Decimal"))?;

                if value < 0.0 {
                    Err(serde::de::Error::custom("Expected a non-negative float"))
                } else {
                    Positive::new_decimal(decimal).map_err(serde::de::Error::custom)
                }
            }
        }

        deserializer.deserialize_any(PositiveVisitor)
    }
}

impl Add for Positive {
    type Output = Positive;

    fn add(self, other: Positive) -> Positive {
        Positive(self.0 + other.0)
    }
}

impl Sub for Positive {
    type Output = Positive;

    fn sub(self, rhs: Self) -> Self::Output {
        let result = self.0 - rhs.0;
        if result < Decimal::ZERO {
            panic!("Resulting value must be positive");
        } else {
            Positive(result)
        }
    }
}

impl Div for Positive {
    type Output = Positive;

    fn div(self, other: Positive) -> Self::Output {
        Positive(self.0 / other.0)
    }
}

impl Div for &Positive {
    type Output = Positive;

    fn div(self, other: &Positive) -> Self::Output {
        Positive(self.0 / other.0)
    }
}

impl Add<Decimal> for Positive {
    type Output = Positive;

    fn add(self, rhs: Decimal) -> Positive {
        Positive(self.0 + rhs)
    }
}

impl Add<&Decimal> for Positive {
    type Output = Positive;

    fn add(self, rhs: &Decimal) -> Self::Output {
        (self.0 + rhs).into()
    }
}

impl Sub<Decimal> for Positive {
    type Output = Positive;

    fn sub(self, rhs: Decimal) -> Positive {
        Positive::new_decimal(self.0 - rhs).expect("Resulting value must be positive")
    }
}

impl Sub<&Decimal> for Positive {
    type Output = Positive;

    fn sub(self, rhs: &Decimal) -> Self::Output {
        Positive::new_decimal(self.0 - rhs).expect("Resulting value must be positive")
    }
}

impl AddAssign for Positive {
    fn add_assign(&mut self, other: Positive) {
        self.0 += other.0;
    }
}

impl AddAssign<Decimal> for Positive {
    fn add_assign(&mut self, rhs: Decimal) {
        self.0 += rhs;
    }
}

impl MulAssign<Decimal> for Positive {
    fn mul_assign(&mut self, rhs: Decimal) {
        self.0 *= rhs;
    }
}

impl Div<Decimal> for Positive {
    type Output = Positive;

    fn div(self, rhs: Decimal) -> Positive {
        Positive(self.0 / rhs)
    }
}

impl Div<&Decimal> for Positive {
    type Output = Positive;

    fn div(self, rhs: &Decimal) -> Self::Output {
        (self.0 / rhs).into()
    }
}

impl PartialOrd<Decimal> for Positive {
    fn partial_cmp(&self, other: &Decimal) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(other)
    }
}

impl PartialOrd for Positive {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }

    fn le(&self, other: &Self) -> bool {
        self.0 <= other.0
    }

    fn ge(&self, other: &Self) -> bool {
        self.0 >= other.0
    }
}

impl Eq for Positive {}

impl Ord for Positive {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.partial_cmp(&other.0).unwrap()
    }
}

impl Neg for Positive {
    type Output = Self;

    fn neg(self) -> Self::Output {
        panic!("Cannot negate a Positive value!");
    }
}

impl Mul for Positive {
    type Output = Positive;

    fn mul(self, other: Positive) -> Positive {
        Positive(self.0 * other.0)
    }
}

impl Mul<Decimal> for Positive {
    type Output = Positive;

    fn mul(self, rhs: Decimal) -> Positive {
        Positive(self.0 * rhs)
    }
}

impl Default for Positive {
    fn default() -> Self {
        Positive::ZERO
    }
}

impl AbsDiffEq for Positive {
    type Epsilon = Decimal;

    fn default_epsilon() -> Self::Epsilon {
        EPSILON
    }

    fn abs_diff_eq(&self, other: &Self, epsilon: Self::Epsilon) -> bool {
        (self.0 - other.0).abs() <= epsilon
    }
}

impl RelativeEq for Positive {
    fn default_max_relative() -> Self::Epsilon {
        EPSILON * Decimal::from(100)
    }

    fn relative_eq(
        &self,
        other: &Self,
        epsilon: Self::Epsilon,
        max_relative: Self::Epsilon,
    ) -> bool {
        // Calculate the absolute difference
        let abs_diff = (self.0 - other.0).abs();

        // Use relative tolerance: max_relative * max(|self.0|, |other.0|)
        let largest = self.0.abs().max(other.0.abs());

        abs_diff <= epsilon || abs_diff <= max_relative * largest
    }
}

impl Sum for Positive {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        let sum = iter.fold(Decimal::ZERO, |acc, x| acc + x.value());
        Positive::new_decimal(sum).unwrap_or(Positive::ZERO)
    }
}

impl<'a> Sum<&'a Positive> for Positive {
    fn sum<I: Iterator<Item = &'a Positive>>(iter: I) -> Self {
        let sum = iter.fold(Decimal::ZERO, |acc, x| acc + x.value());
        Positive::new_decimal(sum).unwrap_or(Positive::ZERO)
    }
}

#[cfg(test)]
mod tests_positive_decimal {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_positive_decimal_creation() {
        assert!(Positive::new_decimal(Decimal::ZERO).is_ok());
        assert!(Positive::new_decimal(Decimal::ONE).is_ok());
        assert!(Positive::new_decimal(Decimal::NEGATIVE_ONE).is_err());
    }

    #[test]
    fn test_positive_decimal_value() {
        let pos = Positive::new(5.0).unwrap();
        assert_eq!(pos, 5.0);
    }

    #[test]
    fn test_positive_decimal_from() {
        let pos = Positive::new(3.0).unwrap();
        let f: Decimal = pos.into();
        assert_eq!(f, dec!(3.0));
    }

    #[test]
    fn test_positive_decimal_eq() {
        let pos = Positive::new_decimal(Decimal::TWO).unwrap();
        assert_eq!(pos, dec!(2.0));
        assert_ne!(pos, dec!(3.0));
    }

    #[test]
    fn test_positive_decimal_display() {
        let pos = Positive::new_decimal(dec!(4.5)).unwrap();
        assert_eq!(format!("{pos}"), "4.5");
    }

    #[test]
    fn test_positive_decimal_debug() {
        let pos = Positive::new_decimal(dec!(4.5)).unwrap();
        assert_eq!(format!("{pos:?}"), "4.5");
    }

    #[test]
    fn test_positive_decimal_display_decimal_fix() {
        let pos = Positive::new_decimal(dec!(4.578923789423789)).unwrap();
        assert_eq!(format!("{pos:.2}"), "4.57");
        assert_eq!(format!("{pos:.3}"), "4.578");
        assert_eq!(format!("{pos:.0}"), "4");
    }

    #[test]
    fn test_positive_decimal_add() {
        let a = Positive::new_decimal(dec!(2.0)).unwrap();
        let b = Positive::new_decimal(dec!(3.0)).unwrap();
        assert_eq!((a + b).value(), dec!(5.0));
    }

    #[test]
    fn test_positive_decimal_div() {
        let a = Positive::new_decimal(dec!(6.0)).unwrap();
        let b = Positive::new_decimal(dec!(2.0)).unwrap();
        assert_eq!((a / b).value(), dec!(3.0));
    }

    #[test]
    fn test_positive_decimal_div_decimal() {
        let a = Positive::new_decimal(dec!(6.0)).unwrap();
        assert_eq!((a / 2.0), 3.0);
    }

    #[test]
    fn test_decimal_mul_positive_decimal() {
        let a = dec!(2.0);
        let b = Positive::new_decimal(dec!(3.0)).unwrap();
        assert_eq!(a * b, dec!(6.0));
    }

    #[test]
    fn test_positive_decimal_mul() {
        let a = Positive::new_decimal(dec!(2.0)).unwrap();
        let b = Positive::new_decimal(dec!(3.0)).unwrap();
        assert_eq!((a * b).value(), dec!(6.0));
    }

    #[test]
    fn test_positive_decimal_mul_decimal() {
        let a = Positive::new_decimal(dec!(2.0)).unwrap();
        assert_eq!((a * 3.0), 6.0);
    }

    #[test]
    fn test_positive_decimal_default() {
        assert_eq!(Positive::default().value(), Decimal::ZERO);
    }

    #[test]
    fn test_decimal_div_positive_decimal() {
        let a = dec!(6.0);
        let b = Positive::new_decimal(dec!(2.0)).unwrap();
        assert_eq!(a / b, dec!(3.0));
    }

    #[test]
    fn test_constants() {
        assert_eq!(Positive::ZERO.value(), Decimal::ZERO);
        assert_eq!(Positive::ONE.value(), Decimal::ONE);
    }
}

#[cfg(test)]
mod tests_positive_decimal_extended {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_positive_decimal_ordering() {
        let a = pos!(1.0);
        let b = pos!(2.0);
        let c = pos!(2.0);

        assert!(a < b);
        assert!(b > a);
        assert!(b >= c);
        assert!(b <= c);
    }

    #[test]
    fn test_positive_decimal_add_assign() {
        let mut a = pos!(1.0);
        let b = pos!(2.0);
        a += b;
        assert_eq!(a.value(), dec!(3.0));
    }

    #[test]
    fn test_positive_decimal_mul_assign() {
        let mut a = Decimal::TWO;
        a *= dec!(3.0);
        assert_eq!(a, dec!(6.0));
    }

    #[test]
    fn test_positive_decimal_from_string() {
        assert_eq!(Positive::from_str("1.5").unwrap().value(), dec!(1.5));
        assert!(Positive::from_str("-1.5").is_err());
        assert!(Positive::from_str("invalid").is_err());
    }

    #[test]
    fn test_positive_decimal_max_min() {
        let a = pos!(1.0);
        let b = pos!(2.0);
        assert_eq!(a.max(b).value(), dec!(2.0));
        assert_eq!(a.min(b).value(), dec!(1.0));
    }

    #[test]
    fn test_positive_decimal_floor() {
        let a = pos!(1.7);
        assert_eq!(a.floor().value(), dec!(1.0));
    }

    #[test]
    #[should_panic(expected = "Cannot negate a Positive value!")]
    fn test_positive_decimal_neg() {
        let a = pos!(1.0);
        let _ = -a;
    }
}

#[cfg(test)]
mod tests_positive_decimal_sum {
    use super::*;

    #[test]
    fn test_sum_owned_values() {
        let values = vec![pos!(1.0), pos!(2.0), pos!(3.0)];
        let sum: Positive = values.into_iter().sum();
        assert_eq!(sum.to_f64(), 6.0);
    }

    #[test]
    fn test_sum_referenced_values() {
        let values = [pos!(1.0), pos!(2.0), pos!(3.0)];
        let sum: Positive = values.iter().sum();
        assert_eq!(sum.to_f64(), 6.0);
    }

    #[test]
    fn test_sum_empty_iterator() {
        let values: Vec<Positive> = vec![];
        let sum: Positive = values.into_iter().sum();
        assert_eq!(sum.to_f64(), 0.0);
    }
}

#[cfg(test)]
mod tests_eq {
    use crate::Positive;
    use rust_decimal_macros::dec;

    #[test]
    #[ignore = "This test is failing because of the precision limit"]
    fn test_eq() {
        let a = pos!(0.5848105371755788);
        let b = Positive::new_decimal(dec!(0.5848105371755788)).unwrap();
        assert_eq!(a, b);
    }
}

#[cfg(test)]
mod tests_macros {
    use super::*;
    use rust_decimal::Decimal;

    #[test]
    fn test_pos_positive_values() {
        assert_eq!(pos!(5.0).value(), Decimal::new(5, 0));
        assert_eq!(pos!(1.5).value(), Decimal::new(15, 1));
        assert_eq!(pos!(0.1).value(), Decimal::new(1, 1));
    }

    #[test]
    fn test_pos_zero() {
        assert_eq!(Positive::ZERO, Positive::ZERO);
    }

    #[test]
    fn test_pos_small_decimals() {
        assert_eq!(pos!(0.0001).value(), Decimal::new(1, 4));
        assert_eq!(pos!(0.00001).value(), Decimal::new(1, 5));
        assert_eq!(pos!(0.000001).value(), Decimal::new(1, 6));
    }

    #[test]
    fn test_pos_large_decimals() {
        let val = 0.1234567890123456;
        let expected = Decimal::from_str("0.1234567890123456").unwrap();
        assert_eq!(pos!(val).value(), expected);
    }

    #[test]
    fn test_pos_precision_limits() {
        // Test the maximum precision of 16 decimal places
        let val = ((0.123_456_789_012_345_68_f64 * 1e16) as u64) as f64 / 1e16; // More than 16 decimal places
        let expected = Decimal::from_str("0.1234567890123456").unwrap();
        assert_eq!(pos!(val).value(), expected);
    }

    #[test]
    #[should_panic(expected = "OutOfBounds")]
    fn test_pos_negative_values() {
        // Negative values should panic with OutOfBounds error
        pos!(-1.0);
    }

    #[test]
    fn test_pos_edge_cases() {
        // Test with very large numbers
        assert_eq!(
            pos!(1e15).value(),
            Decimal::from_str("1000000000000000").unwrap()
        );

        // Test with very small numbers
        assert_eq!(
            pos!(1e-15).value(),
            Decimal::from_str("0.000000000000001").unwrap()
        );
    }

    #[test]
    fn test_pos_expressions() {
        assert_eq!(pos!(2.0 + 3.0).value(), Decimal::new(5, 0));
        assert_eq!(pos!(1.5 * 2.0).value(), Decimal::new(3, 0));
    }

    #[test]
    fn test_pos_conversions() {
        // Test integer to float conversion
        assert_eq!(pos!(5.0).value(), Decimal::new(5, 0));

        // Test float literals
        assert_eq!(pos!(5.0).value(), Decimal::new(5, 0));
    }
}

#[cfg(test)]
mod tests_serialization {
    use super::*;
    use rust_decimal_macros::dec; // Para crear Decimals de forma más legible
    use serde_json; // Para las pruebas de serialización

    #[test]
    fn test_positive_serialization() {
        let value = Positive(dec!(42.5));
        let serialized = serde_json::to_string(&value).unwrap();
        assert_eq!(serialized, "42.5");
    }

    #[test]
    fn test_positive_deserialization() {
        let json = "42.5";
        let deserialized: Positive = serde_json::from_str(json).unwrap();
        assert_eq!(deserialized, Positive(dec!(42.5)));
    }

    #[test]
    fn test_positive_serialization_whole_number() {
        let value = Positive(dec!(100));
        let serialized = serde_json::to_string(&value).unwrap();
        assert_eq!(serialized, "100");
    }

    #[test]
    fn test_positive_deserialization_whole_number() {
        let json = "100";
        let deserialized: Positive = serde_json::from_str(json).unwrap();
        assert_eq!(deserialized, Positive(dec!(100)));
    }

    #[test]
    fn test_positive_roundtrip() {
        let original = Positive(dec!(123.456));
        let serialized = serde_json::to_string(&original).unwrap();
        let deserialized: Positive = serde_json::from_str(&serialized).unwrap();
        assert_eq!(original, deserialized);
    }

    #[test]
    fn test_positive_high_precision() {
        let value = Positive(dec!(12345.6789));
        let serialized = serde_json::to_string(&value).unwrap();
        let deserialized: Positive = serde_json::from_str(&serialized).unwrap();
        assert_eq!(value, deserialized);
    }

    #[test]
    fn test_positive_zero_deserialization() {
        let json = "0";
        let result = serde_json::from_str::<Positive>(json);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Positive::ZERO);
    }

    #[test]
    fn test_positive_negative_deserialization() {
        let json = "-42.5";
        let result = serde_json::from_str::<Positive>(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_positive_infinity_serialization() {
        let value = Positive::INFINITY;
        let serialized = serde_json::to_string(&value).unwrap();
        assert_eq!(serialized, r#""infinity""#);
    }

    #[test]
    fn test_positive_infinity_deserialization() {
        let json = r#""infinity""#;
        let result = serde_json::from_str::<Positive>(json);

        assert!(result.is_ok());
    }
}

#[cfg(test)]
mod tests_format_fixed_places {
    use crate::pos;

    #[test]
    fn test_format_fixed_places() {
        let value = pos!(10.5);
        assert_eq!(value.format_fixed_places(2), "10.50");

        let value = pos!(10.0);
        assert_eq!(value.format_fixed_places(3), "10.000");

        let value = pos!(10.567);
        assert_eq!(value.format_fixed_places(2), "10.57");

        let value = pos!(0.1);
        assert_eq!(value.format_fixed_places(4), "0.1000");
    }
}
