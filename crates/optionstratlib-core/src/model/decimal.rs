/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 25/12/24
******************************************************************************/
use crate::error::DecimalError;
use num_traits::FromPrimitive;
use positive::{Positive, PositiveError};
use rand::distr::Distribution;
use rand::{Rng, RngExt};
use rand_distr::StandardNormal;
use rust_decimal::{Decimal, MathematicalOps, RoundingStrategy};
use rust_decimal_macros::dec;

/// Represents the daily interest rate factor used for financial calculations,
/// approximately equivalent to 1/252 (a standard value for the number of trading days in a year).
///
/// This constant converts annual interest rates to daily rates by providing a division factor.
/// The value 0.00396825397 corresponds to 1/252, where 252 is the typical number of trading
/// days in a financial year.
///
/// # Usage
///
/// This constant is commonly used in financial calculations such as:
/// - Converting annual interest rates to daily rates
/// - Time value calculations for options pricing
/// - Discounting cash flows on a daily basis
/// - Interest accrual calculations
pub const ONE_DAY: Decimal = dec!(0.00396825397);

/// Asserts that two Decimal values are approximately equal within a given epsilon
#[macro_export]
macro_rules! assert_decimal_eq {
    ($left:expr, $right:expr, $epsilon:expr) => {
        let diff = ($left - $right).abs();
        assert!( // scan-banned: allow -- test assertion macro; it expands at the caller and no library code uses it
            diff <= $epsilon,
            "assertion failed: `(left == right)`\n  left: `{}`\n right: `{}`\n  diff: `{}`\n epsilon: `{}`",
            $left,
            $right,
            diff,
            $epsilon
        );
    };
}

/// Defines statistical operations for collections of decimal values.
///
/// This trait provides methods to calculate common statistical measures
/// for sequences or collections of `Decimal` values. It allows implementing
/// types to offer standardized statistical analysis capabilities.
///
/// ## Key Features
///
/// * Basic statistical calculations for `Decimal` collections
/// * Consistent interface for various collection types
/// * Precision-preserving operations using the `Decimal` type
///
/// ## Available Statistics
///
/// * `mean`: Calculates the arithmetic mean (average) of the values
/// * `std_dev`: Calculates the standard deviation, measuring the dispersion from the mean
///
/// ## Example
///
/// ```rust
/// use rust_decimal::Decimal;
/// use rust_decimal_macros::dec;
/// use optionstratlib_core::error::DecimalError;
/// use optionstratlib_core::model::decimal::DecimalStats;
///
/// struct DecimalSeries(Vec<Decimal>);
///
/// impl DecimalStats for DecimalSeries {
///     fn mean(&self) -> Result<Decimal, DecimalError> {
///         if self.0.is_empty() {
///             return Ok(dec!(0));
///         }
///         let sum: Decimal = self.0.iter().sum();
///         Ok(sum / Decimal::from(self.0.len()))
///     }
///
///     fn std_dev(&self) -> Result<Decimal, DecimalError> {
///         // Implementation of standard deviation calculation
///         // ...
///         Ok(dec!(0)) // Placeholder return
///     }
/// }
/// ```
pub trait DecimalStats {
    /// Calculates the arithmetic mean (average) of the collection.
    ///
    /// The mean is the sum of all values divided by the count of values.
    /// This method should handle empty collections appropriately.
    ///
    /// # Errors
    ///
    /// Implementations return [`DecimalError`] when the running sum or the
    /// division by the count leaves the representable `Decimal` range.
    fn mean(&self) -> Result<Decimal, DecimalError>;

    /// Calculates the standard deviation of the collection.
    ///
    /// The standard deviation measures the amount of variation or dispersion
    /// from the mean. A low standard deviation indicates that values tend to be
    /// close to the mean, while a high standard deviation indicates values are
    /// spread out over a wider range.
    ///
    /// # Errors
    ///
    /// Implementations return [`DecimalError`] when a centred deviation, its
    /// square, the sum of squares, or the final division leaves the
    /// representable `Decimal` range.
    fn std_dev(&self) -> Result<Decimal, DecimalError>;
}

impl DecimalStats for Vec<Decimal> {
    /// # Errors
    ///
    /// Returns [`DecimalError::Overflow`] when the sum of the values leaves
    /// the representable range — `vec![Decimal::MAX; 2]` is enough. The
    /// previous signature had nowhere to put that, and `iter().sum()` aborts
    /// with `Addition overflowed`.
    fn mean(&self) -> Result<Decimal, DecimalError> {
        if self.is_empty() {
            return Ok(Decimal::ZERO);
        }
        let sum = d_sum(self, "decimal::stats::mean::sum")?;
        // `Decimal::from(usize)` is total, and the slice is non-empty here,
        // so the divisor is neither an overflow nor a zero.
        d_div(sum, Decimal::from(self.len()), "decimal::stats::mean")
    }

    /// # Errors
    ///
    /// Returns [`DecimalError::Overflow`] when the mean, a centred deviation,
    /// its square or the sum of squares leaves the representable range, and
    /// [`DecimalError::ArithmeticError`] if the sample variance were negative.
    /// The squaring was `.powd(Decimal::TWO)`, which aborts with
    /// `Pow overflowed`.
    fn std_dev(&self) -> Result<Decimal, DecimalError> {
        // Population variance of a single value is zero
        if self.len() < 2usize {
            return Ok(Decimal::ZERO);
        }
        let mean = self.mean()?;
        let mut sq_total = Decimal::ZERO;
        for value in self {
            let centred = d_sub(*value, mean, "decimal::stats::std_dev::centred")?;
            let centred_sq = d_mul(centred, centred, "decimal::stats::std_dev::centred_sq")?;
            sq_total = d_add(sq_total, centred_sq, "decimal::stats::std_dev::sq_total")?;
        }
        // `len() >= 2` above, so `len() - 1` neither underflows nor is zero;
        // the checked form keeps that proof local.
        let degrees_of_freedom = self.len().checked_sub(1).ok_or_else(|| {
            DecimalError::arithmetic_error("decimal::stats::std_dev", "sample is empty")
        })?;
        let variance = d_div(
            sq_total,
            Decimal::from(degrees_of_freedom),
            "decimal::stats::std_dev::variance",
        )?;
        d_sqrt(variance, "decimal::stats::std_dev")
    }
}

/// `5^0` to `5^28`. `10^scale = 2^scale * 5^scale`, so a `Decimal` is
/// `mantissa / 5^scale * 2^-scale`: only the division by `5^scale` rounds,
/// and `5^28 < 2^66` keeps it in `u128`.
const POWERS_OF_FIVE: [u128; 29] = [
    1,
    5,
    25,
    125,
    625,
    3125,
    15625,
    78125,
    390625,
    1953125,
    9765625,
    48828125,
    244140625,
    1220703125,
    6103515625,
    30517578125,
    152587890625,
    762939453125,
    3814697265625,
    19073486328125,
    95367431640625,
    476837158203125,
    2384185791015625,
    11920928955078125,
    59604644775390625,
    298023223876953125,
    1490116119384765625,
    7450580596923828125,
    37252902984619140625,
];

/// Bits in an `f64` significand, the implicit leading one included.
const F64_SIGNIFICAND_BITS: u32 = 53;

/// Bias of the `f64` exponent field.
const F64_EXPONENT_BIAS: i32 = 1023;

/// Bits below the `f64` exponent field.
const F64_EXPONENT_SHIFT: u32 = 52;

#[cold]
#[inline(never)]
fn decimal_to_f64_error(value: Decimal, reason: &str) -> DecimalError {
    DecimalError::ConversionError {
        from_type: format!("Decimal: {value}"),
        to_type: "f64".to_string(),
        reason: reason.to_string(),
    }
}

/// Converts a `Decimal` to the `f64` nearest to it (ties to even).
///
/// The result is the `f64` the literal written with the same digits parses
/// to: `decimal_to_f64(dec!(2.999789999999902))` is `2.999789999999902_f64`
/// bit for bit. `Decimal::to_f64` divides the mantissa by a power of ten in
/// floating point and can land a few ULPs away from it once the value has 15
/// or more decimal places (`2.999789999999903` for that input), so this
/// function does not use it (#670). Zero, of either sign, converts to `+0.0`.
///
/// The value is `mantissa / 5^scale * 2^-scale`. The mantissa, shifted to
/// the top of a `u128`, is divided by `5^scale` in integers, which leaves a
/// quotient of at least 62 bits and an exact remainder; the quotient is
/// rounded to 53 bits from its dropped bits and the remainder, and scaled by
/// an exact power of two. It does no floating-point rounding before that one
/// step and does not allocate on success.
///
/// # Parameters
///
/// * `value` - The `Decimal` value to convert
///
/// # Errors
///
/// Returns [`DecimalError::ConversionError`] if the scale exceeds the 28
/// places a `Decimal` holds or an intermediate leaves its range. Every valid
/// `Decimal` lies within `f64` range and converts, so neither happens; they
/// are reported rather than assumed.
///
/// # Example
///
/// ```rust
/// use rust_decimal_macros::dec;
/// use optionstratlib_core::model::decimal::decimal_to_f64;
///
/// let value = decimal_to_f64(dec!(0.95));
/// assert!(matches!(value, Ok(v) if v.to_bits() == 0.95_f64.to_bits()));
///
/// // 15 decimal places: the nearest `f64`, which `Decimal::to_f64` misses.
/// let value = decimal_to_f64(dec!(2.999789999999902));
/// assert!(matches!(value, Ok(v) if v.to_bits() == 2.999_789_999_999_902_f64.to_bits()));
/// ```
pub fn decimal_to_f64(value: Decimal) -> Result<f64, DecimalError> {
    let mantissa = value.mantissa().unsigned_abs();
    if mantissa == 0 {
        return Ok(0.0);
    }
    let scale = value.scale();
    let out_of_range = || decimal_to_f64_error(value, "intermediate outside its range");
    let divisor = usize::try_from(scale)
        .ok()
        .and_then(|index| POWERS_OF_FIVE.get(index))
        .copied()
        .ok_or_else(|| decimal_to_f64_error(value, "scale exceeds the 28 places of a Decimal"))?;

    // Left-aligned, the numerator is at least `2^127`, so after the division
    // by `5^28 < 2^66` the quotient is above `2^61`: at least 62 bits.
    let shift = mantissa.leading_zeros();
    let numerator = mantissa << shift;
    let quotient = numerator.checked_div(divisor).ok_or_else(out_of_range)?;
    let remainder = numerator.checked_rem(divisor).ok_or_else(out_of_range)?;

    // Keep the top 53 bits and round on the rest: above half rounds up, a
    // tie (dropped bits exactly half and nothing in the remainder) rounds to
    // an even significand.
    let excess = u128::BITS
        .checked_sub(quotient.leading_zeros())
        .and_then(|bits| bits.checked_sub(F64_SIGNIFICAND_BITS))
        .ok_or_else(out_of_range)?;
    let half = excess
        .checked_sub(1)
        .and_then(|bits| 1_u128.checked_shl(bits))
        .ok_or_else(out_of_range)?;
    let dropped_mask = half
        .checked_shl(1)
        .and_then(|bit| bit.checked_sub(1))
        .ok_or_else(out_of_range)?;
    let dropped = quotient & dropped_mask;
    let mut significand = quotient >> excess;
    if dropped > half || (dropped == half && (remainder != 0 || significand & 1 == 1)) {
        significand = significand.checked_add(1).ok_or_else(out_of_range)?;
    }

    // `significand <= 2^53` converts exactly and the power of two is exact,
    // so the product is the rounded quotient scaled by `2^-shift * 2^-scale`.
    let exponent = i32::try_from(excess)
        .ok()
        .zip(i32::try_from(shift).ok())
        .zip(i32::try_from(scale).ok())
        .and_then(|((excess, shift), scale)| {
            F64_EXPONENT_BIAS
                .checked_add(excess)?
                .checked_sub(shift)?
                .checked_sub(scale)
        })
        .and_then(|biased| u64::try_from(biased).ok())
        .ok_or_else(out_of_range)?;
    let power_of_two = f64::from_bits(exponent << F64_EXPONENT_SHIFT);
    let magnitude = significand as f64 * power_of_two;
    Ok(if value.is_sign_negative() {
        -magnitude
    } else {
        magnitude
    })
}

/// Converts an f64 floating-point number to a Decimal.
///
/// This function attempts to convert an f64 floating-point number to a Decimal value.
/// If the conversion fails (for example, if the f64 represents NaN, infinity, or is otherwise
/// not representable as a Decimal), it returns a DecimalError with detailed information about
/// the failure.
///
/// # Parameters
///
/// * `value` - The f64 value to convert
///
/// # Returns
///
/// * `Result<Decimal, DecimalError>` - The converted Decimal value if successful, or a DecimalError
///   if the conversion fails
///
/// # Errors
///
/// Returns [`DecimalError::ConversionError`] when the `f64` operand is not
/// representable as a `Decimal`, for example `NaN`, `±Infinity`, or a value
/// whose magnitude exceeds the `Decimal` range.
///
/// # Example
///
/// ```rust
/// use rust_decimal::Decimal;
/// use tracing::info;
/// use optionstratlib_core::model::decimal::f64_to_decimal;
///
/// let float = std::f64::consts::PI;
/// match f64_to_decimal(float) {
///     Ok(decimal) => info!("Converted to Decimal: {}", decimal),
///     Err(e) => info!("Conversion error: {:?}", e)
/// }
/// ```
pub fn f64_to_decimal(value: f64) -> Result<Decimal, DecimalError> {
    // `ok_or_else`: the error formats the value and allocates three
    // `String`s, which `ok_or` paid on every successful call too (#857).
    Decimal::from_f64(value).ok_or_else(|| DecimalError::ConversionError {
        from_type: format!("f64: {value}"),
        to_type: "Decimal".to_string(),
        reason: "Failed to convert f64 to Decimal".to_string(),
    })
}

/// Attempts to convert a finite `f64` into a `Decimal`.
///
/// Returns `None` when `value` is not finite (`NaN`, `+∞`, `-∞`) or
/// when `Decimal::from_f64` rejects the conversion (the latter is
/// vanishingly rare for representable `f64`). Checked helper
/// that standardises the `is_finite()` check paired with
/// `Decimal::from_f64` at every `f64` → `Decimal` boundary inside
/// pricing, Greeks, volatility, and simulation kernels.
///
/// Callers wrap the `None` case with a domain-specific
/// `*Error::NonFinite { context, value }` via `ok_or_else`:
///
/// ```ignore
/// let v = finite_decimal(v_f64)
///     .ok_or_else(|| PricingError::non_finite("pricing::bs::call::d1", v_f64))?;
/// ```
///
/// The guard is enforced at the public boundary of every `f64`
/// numerical kernel per the rules (`rules/global_rules.md`
/// §Arithmetic).
#[must_use]
#[inline]
pub fn finite_decimal(value: f64) -> Option<Decimal> {
    if value.is_finite() {
        Decimal::from_f64(value)
    } else {
        None
    }
}

/// Draws a standard normal sample from the thread-local RNG, as a `Decimal`.
///
/// This is the unseeded path: every call reads the thread RNG, so two runs
/// never see the same stream. Reproducible callers use
/// [`decimal_normal_sample_with`] with a seeded generator such as
/// [`crate::utils::deterministic_rng`]; this function is exactly that helper
/// applied to [`rand::rng()`].
///
/// # Returns
///
/// A `Decimal` sampled from a standard normal distribution (it can be
/// negative).
///
/// # Examples
///
/// ```rust
/// use optionstratlib_core::model::decimal::decimal_normal_sample;
/// let normal = decimal_normal_sample();
/// ```
///
/// The sample is drawn from [`rand_distr::StandardNormal`], which is a unit
/// struct with no constructor and therefore nothing to reject. It replaced
/// `Normal::new(0.0, 1.0)`, whose `Err` arm had no value to return and so
/// aborted. The two are the same distribution and the same value: `Normal`
/// samples `StandardNormal` and applies `mean + std_dev * z`, which is the
/// identity at `(0.0, 1.0)`.
#[must_use]
pub fn decimal_normal_sample() -> Decimal {
    decimal_normal_sample_with(&mut rand::rng())
}

/// Draws a standard normal sample from `rng`, as a `Decimal`.
///
/// One `f64` is drawn from [`rand_distr::StandardNormal`] and converted once
/// with `Decimal::from_f64`; a sample the conversion rejects (none is
/// finite and out of range in practice) becomes zero, as in
/// [`decimal_normal_sample`]. The generator advances by exactly the draws
/// `StandardNormal` makes, so a seeded `rng` yields the same sequence of
/// samples on every run.
///
/// # Examples
///
/// ```rust
/// use optionstratlib_core::model::decimal::decimal_normal_sample_with;
/// use optionstratlib_core::utils::deterministic_rng;
///
/// let mut first = deterministic_rng(42);
/// let mut second = deterministic_rng(42);
/// assert_eq!(
///     decimal_normal_sample_with(&mut first),
///     decimal_normal_sample_with(&mut second)
/// );
/// ```
#[must_use]
#[inline]
pub fn decimal_normal_sample_with<R: Rng + ?Sized>(rng: &mut R) -> Decimal {
    let sample: f64 = StandardNormal.sample(rng);
    Decimal::from_f64(sample).unwrap_or(Decimal::ZERO)
}

/// Decimal places of a [`decimal_uniform_sample_with`] draw.
const UNIFORM_SAMPLE_SCALE: u32 = 18;

/// Number of grid points of a [`decimal_uniform_sample_with`] draw,
/// `10^UNIFORM_SAMPLE_SCALE`; below `i64::MAX`, so every point is an exact
/// `i64` mantissa.
const UNIFORM_SAMPLE_GRID: i64 = 1_000_000_000_000_000_000;

/// Draws a uniform sample on `[0, 1)` from `rng`, as a `Decimal`.
///
/// The sample is `k / 10^18`, with the integer `k` drawn uniformly from
/// `0..10^18` by [`rand::RngExt::random_range`]. Its range is therefore
/// `[0, 1 - 10^-18]`: zero is reachable, one never is. No `f64` is
/// involved, so the value is exact and identical on every platform.
///
/// For any threshold `p` in `[0, 1]` with at most 18 decimal places,
/// `P(sample < p) = p` exactly, up to the bias of `rand`'s single-sample
/// range method (below `2^-64` per grid point). A threshold with more
/// places is rounded up to the grid, an error below `10^-18`. This makes
/// `sample < p` the Bernoulli(`p`) trial of jump and regime-switch
/// decisions; `p >= 1` always fires and `p <= 0` never does.
///
/// The generator advances by the one or two `u64` draws the range method
/// makes, so a seeded `rng` yields the same sequence of samples on every
/// run.
///
/// # Examples
///
/// ```rust
/// use optionstratlib_core::model::decimal::decimal_uniform_sample_with;
/// use optionstratlib_core::utils::deterministic_rng;
/// use rust_decimal::Decimal;
///
/// let mut rng = deterministic_rng(42);
/// let u = decimal_uniform_sample_with(&mut rng);
/// assert!(u >= Decimal::ZERO && u < Decimal::ONE);
/// ```
#[must_use]
#[inline]
pub fn decimal_uniform_sample_with<R: Rng + ?Sized>(rng: &mut R) -> Decimal {
    let k = rng.random_range(0..UNIFORM_SAMPLE_GRID);
    Decimal::new(k, UNIFORM_SAMPLE_SCALE)
}

/// Scale applied to banker's-rounding divisions in [`d_div`].
///
/// `28` matches `Decimal::MAX_SCALE` so a rounded division preserves every
/// digit of precision the backing 96-bit mantissa can represent without
/// triggering a later rescale overflow. Divisions that need a different
/// scale (for example a P&L that rounds to cents) should apply a subsequent
/// explicit `.round_dp_with_strategy(dp, RoundingStrategy::MidpointNearestEven)`.
pub const DIV_DEFAULT_SCALE: u32 = 28;

/// Checked `Decimal` addition with operand-preserving overflow reporting.
///
/// Checked helper used by every monetary-flow kernel in place of the
/// raw `+` operator. Wraps [`Decimal::checked_add`] and converts `None`
/// into a [`DecimalError::Overflow`] tagged with the static `op` string
/// passed in by the call-site.
///
/// # Errors
///
/// Returns [`DecimalError::Overflow`] when the result is outside the
/// representable `Decimal` range.
#[inline]
pub fn d_add(lhs: Decimal, rhs: Decimal, op: &'static str) -> Result<Decimal, DecimalError> {
    lhs.checked_add(rhs)
        .ok_or_else(|| DecimalError::overflow(op, lhs, rhs))
}

/// Checked sum over a slice of `Decimal` values.
///
/// Checked helper used by multi-leg strategy P&L aggregations
/// (spreads, condors, butterflies) where each leg already returns a
/// `Result<Decimal, _>` and the sum has to preserve the checked
/// semantics of the individual legs. Returns `Decimal::ZERO` on an
/// empty slice.
///
/// Delegates to [`d_sum_iter`] so the overflow-tagging semantics stay
/// in lock-step between the slice and iterator entry points.
///
/// # Errors
///
/// Returns [`DecimalError::Overflow`] on the first accumulation that
/// exceeds the representable `Decimal` range, tagged with the
/// supplied `op` string so the caller can be identified without a
/// stack trace.
#[inline]
pub fn d_sum(values: &[Decimal], op: &'static str) -> Result<Decimal, DecimalError> {
    d_sum_iter(values.iter().copied(), op)
}

/// Checked sum over any `IntoIterator` of `Decimal` values.
///
/// Zero-allocation counterpart to [`d_sum`]. Use at aggregation sites
/// that already have a natural iterator (e.g. `self.positions.iter()
/// .map(..)`) to avoid the intermediate `Vec<Decimal>` that `d_sum`
/// forces. Returns `Decimal::ZERO` on an empty iterator.
///
/// # Errors
///
/// Returns [`DecimalError::Overflow`] on the first accumulation that
/// exceeds the representable `Decimal` range, tagged with the
/// supplied `op` string so the caller can be identified without a
/// stack trace.
#[inline]
pub fn d_sum_iter<I>(iter: I, op: &'static str) -> Result<Decimal, DecimalError>
where
    I: IntoIterator<Item = Decimal>,
{
    let mut acc = Decimal::ZERO;
    for v in iter {
        acc = acc
            .checked_add(v)
            .ok_or_else(|| DecimalError::overflow(op, acc, v))?;
    }
    Ok(acc)
}

/// Checked `Decimal` subtraction with operand-preserving overflow reporting.
///
/// Checked helper used by every monetary-flow kernel in place of the
/// raw `-` operator. Wraps [`Decimal::checked_sub`] and converts `None`
/// into a [`DecimalError::Overflow`] tagged with the static `op` string
/// passed in by the call-site.
///
/// # Errors
///
/// Returns [`DecimalError::Overflow`] when the result is outside the
/// representable `Decimal` range.
#[inline]
pub fn d_sub(lhs: Decimal, rhs: Decimal, op: &'static str) -> Result<Decimal, DecimalError> {
    lhs.checked_sub(rhs)
        .ok_or_else(|| DecimalError::overflow(op, lhs, rhs))
}

/// Checked `Decimal` multiplication with operand-preserving overflow reporting.
///
/// Checked helper used by every monetary-flow kernel in place of the
/// raw `*` operator. Wraps [`Decimal::checked_mul`] and converts `None`
/// into a [`DecimalError::Overflow`] tagged with the static `op` string
/// passed in by the call-site.
///
/// # Errors
///
/// Returns [`DecimalError::Overflow`] when the result is outside the
/// representable `Decimal` range.
#[inline]
pub fn d_mul(lhs: Decimal, rhs: Decimal, op: &'static str) -> Result<Decimal, DecimalError> {
    lhs.checked_mul(rhs)
        .ok_or_else(|| DecimalError::overflow(op, lhs, rhs))
}

/// Checked product over any `IntoIterator` of `Decimal` values.
///
/// Multiplicative counterpart to [`d_sum_iter`], with the same
/// overflow-tagging semantics and the same zero-allocation shape. Returns
/// [`Decimal::ONE`] on an empty iterator, so an empty product is the
/// multiplicative identity just as an empty sum is [`Decimal::ZERO`].
///
/// # Ordering
///
/// The fold runs strictly left to right, and that order is part of the
/// contract rather than an implementation detail. `Decimal` multiplication
/// rounds as soon as a product needs more decimal places than `Decimal`'s
/// scale limit of 28, which makes it non-associative: `(a * b) *
/// c` and `a * (b * c)` can differ in their last digit. A parallel reducer
/// picks its bracketing from the chunking rayon happens to choose on the day,
/// so the same binary on the same input can return different last digits from
/// one run to the next. Callers that have to reconcile, cache or
/// regression-test a result at full precision need the fixed order this
/// gives them.
///
/// # Errors
///
/// Returns [`DecimalError::Overflow`] on the first accumulation that
/// exceeds the representable `Decimal` range, tagged with the supplied `op`
/// string so the call-site can be identified without a stack trace.
#[inline]
pub fn d_product_iter<I>(iter: I, op: &'static str) -> Result<Decimal, DecimalError>
where
    I: IntoIterator<Item = Decimal>,
{
    let mut acc = Decimal::ONE;
    for v in iter {
        acc = acc
            .checked_mul(v)
            .ok_or_else(|| DecimalError::overflow(op, acc, v))?;
    }
    Ok(acc)
}

/// Checked `Decimal` division with banker's rounding at scale 28.
///
/// Checked helper used by every monetary-flow kernel in place of the
/// raw `/` operator. Performs [`Decimal::checked_div`] then re-rounds the
/// quotient with [`RoundingStrategy::MidpointNearestEven`] to the default
/// [`DIV_DEFAULT_SCALE`]. This policy is applied uniformly across the
/// crate so long-chain divisions do not silently accumulate bias.
///
/// # Errors
///
/// - Returns [`DecimalError::Overflow`] when the quotient is outside the
///   representable `Decimal` range.
/// - Returns [`DecimalError::ArithmeticError`] when `rhs` is zero.
#[inline]
pub fn d_div(lhs: Decimal, rhs: Decimal, op: &'static str) -> Result<Decimal, DecimalError> {
    if rhs.is_zero() {
        return Err(DecimalError::arithmetic_error(op, "division by zero"));
    }
    let raw = lhs
        .checked_div(rhs)
        .ok_or_else(|| DecimalError::overflow(op, lhs, rhs))?;
    Ok(raw.round_dp_with_strategy(DIV_DEFAULT_SCALE, RoundingStrategy::MidpointNearestEven))
}

/// Checked `e^x` with underflow flushed to zero.
///
/// Checked helper used by every kernel in place of
/// [`MathematicalOps::exp`], which panics with `Exp overflowed` /
/// `Exp underflowed` instead of reporting the failure.
///
/// A negative argument whose exponential is smaller than the smallest
/// representable `Decimal` (`1e-28`) returns `Decimal::ZERO`: that is the
/// value the discount factor takes at the limit, and it is the only
/// representable answer. A positive argument that overflows is a real
/// failure and is reported as such — the caller asked for a number the type
/// cannot hold.
///
/// # Errors
///
/// Returns [`DecimalError::Overflow`] when `x` is positive and `e^x` is
/// outside the representable `Decimal` range.
#[inline]
pub fn d_exp(x: Decimal, op: &'static str) -> Result<Decimal, DecimalError> {
    if let Some(value) = x.checked_exp() {
        return Ok(value);
    }
    if x.is_sign_negative() {
        // e^x < 1e-28 for x <= -65: zero is the representable limit.
        return Ok(Decimal::ZERO);
    }
    Err(DecimalError::overflow(op, x, Decimal::ZERO))
}

/// Checked natural logarithm.
///
/// Checked helper used in place of [`MathematicalOps::ln`], which
/// panics on zero and on negative inputs. Note that a ratio such as `S / K`
/// can round down to exactly zero for extreme operands, so the zero case is
/// reachable from ordinary-looking code.
///
/// # Errors
///
/// Returns [`DecimalError::ArithmeticError`] when `x` is zero or negative,
/// or when the series evaluation fails to converge to a representable
/// value.
#[inline]
pub fn d_ln(x: Decimal, op: &'static str) -> Result<Decimal, DecimalError> {
    if x <= Decimal::ZERO {
        return Err(DecimalError::arithmetic_error(
            op,
            "logarithm of a non-positive value",
        ));
    }
    x.checked_ln()
        .ok_or_else(|| DecimalError::arithmetic_error(op, "logarithm is not representable"))
}

/// Natural logarithm evaluated in `f64`, for the numeric kernels (#857).
///
/// [`d_ln`] runs `rust_decimal`'s series to 28 places and costs about 8 µs;
/// a kernel whose other inputs are already `f64`-accurate gains nothing from
/// those places. This takes the logarithm in `f64` and converts the result
/// back once:
///
/// - for `|x - 1| < 1/2` it evaluates `ln_1p(x - 1)`, with `x - 1` exact in
///   `Decimal`, so the result keeps its relative accuracy as `x` approaches
///   one (where `ln(x)` would lose it to cancellation) and `ln(1)` is exactly
///   zero;
/// - elsewhere it evaluates `ln(x)`, whose magnitude is at least `ln(3/2)`.
///
/// In both cases the result is within a few `f64` ulps, about `1e-15`
/// relative, of the exact logarithm. The public [`d_ln`] stays `Decimal`.
///
/// # Errors
///
/// Returns [`DecimalError::ArithmeticError`] when `x` is zero or negative, or
/// when the `f64` result is not finite, and the conversion errors of
/// [`decimal_to_f64`] and [`f64_to_decimal`].
#[inline]
pub fn d_ln_f64(x: Decimal, op: &'static str) -> Result<Decimal, DecimalError> {
    if x <= Decimal::ZERO {
        return Err(DecimalError::arithmetic_error(
            op,
            "logarithm of a non-positive value",
        ));
    }
    let shifted = x
        .checked_sub(Decimal::ONE)
        .ok_or_else(|| DecimalError::arithmetic_error(op, "x - 1 is not representable"))?;
    let value = if shifted.abs() < LN_1P_BAND {
        decimal_to_f64(shifted)?.ln_1p()
    } else {
        decimal_to_f64(x)?.ln() // scan-banned: allow -- f64 `ln`: x > 0 here, and `f64::ln` returns -inf/NaN rather than aborting; a non-finite result is rejected below
    };
    if !value.is_finite() {
        return Err(DecimalError::arithmetic_error(
            op,
            "logarithm is not finite in f64",
        ));
    }
    f64_to_decimal(value)
}

/// `|x - 1|` below which [`d_ln_f64`] evaluates `ln_1p(x - 1)`.
const LN_1P_BAND: Decimal = Decimal::from_parts(5, 0, 0, false, 1);

/// Checked `base^exponent`.
///
/// Checked helper used in place of [`MathematicalOps::powd`], which
/// panics with `Pow overflowed` both when the result is too large and when
/// it underflows below the representable scale.
///
/// # Errors
///
/// Returns [`DecimalError::Overflow`] when the power is outside the
/// representable `Decimal` range.
#[inline]
pub fn d_powd(base: Decimal, exponent: Decimal, op: &'static str) -> Result<Decimal, DecimalError> {
    base.checked_powd(exponent)
        .ok_or_else(|| DecimalError::overflow(op, base, exponent))
}

/// Upper bound on the Newton steps of [`d_sqrt`]; the upstream iteration
/// converges in a few dozen steps for every representable input and only
/// exceeds this bound when it oscillates between two adjacent values.
const SQRT_MAX_ITERATIONS: u32 = 1000;

/// Checked square root.
///
/// Checked replacement for [`MathematicalOps::sqrt`]. The upstream
/// implementation (`rust_decimal` 1.43, `maths.rs`) runs the same Newton
/// iteration but aborts the process with `geo mean circuit breaker` when the
/// iteration oscillates between two values that differ in the 28th decimal
/// instead of converging, which happens for inputs a few units above a
/// perfect square such as `4.0000000000000000000000000003` (#588). This
/// version keeps the upstream initial guess and update step, so every input
/// on which upstream converges yields the bit-identical result, and resolves
/// the period-2 oscillation the moment it appears by returning the candidate
/// whose square is closest to `x` (a bounded iteration count is the backstop
/// for any other non-convergence).
///
/// # Errors
///
/// Returns [`DecimalError::ArithmeticError`] when `x` is negative or when an
/// intermediate quotient or sum leaves the representable `Decimal` range.
pub fn d_sqrt(x: Decimal, op: &'static str) -> Result<Decimal, DecimalError> {
    sqrt_with_iterations(x, op).map(|(value, _)| value)
}

/// The Newton iteration behind [`d_sqrt`], reporting how many steps it took.
///
/// The count is what proves the period-2 cycle is resolved as soon as it
/// appears rather than at the backstop; a test that measured elapsed time
/// instead failed under coverage instrumentation, which slows every call
/// (#604).
fn sqrt_with_iterations(x: Decimal, op: &'static str) -> Result<(Decimal, u32), DecimalError> {
    if x.is_sign_negative() {
        return Err(DecimalError::arithmetic_error(
            op,
            "square root of a negative value",
        ));
    }
    if x.is_zero() {
        return Ok((Decimal::ZERO, 0));
    }
    let overflow = || DecimalError::arithmetic_error(op, "square root iteration overflowed");
    // Same seed as upstream: half the input, or the input itself when the
    // half is not representable.
    let mut result = x.checked_div(Decimal::TWO).ok_or_else(overflow)?;
    if result.is_zero() {
        result = x;
    }
    let mut last = result.checked_add(Decimal::ONE).ok_or_else(overflow)?;
    let mut before_last = last;
    let mut iterations = 0u32;
    // Squared distance from `x`; a candidate whose square is not
    // representable is treated as infinitely far so the other one wins.
    let error_of = |candidate: Decimal| -> Decimal {
        candidate
            .checked_mul(candidate)
            .and_then(|square| square.checked_sub(x))
            .map(|d| d.abs())
            .unwrap_or(Decimal::MAX)
    };
    while last != result {
        iterations = iterations.checked_add(1).ok_or_else(overflow)?;
        // A period-2 cycle (`result` back to the value two steps ago) is how
        // the upstream iteration fails to converge; resolve it as soon as it
        // appears. The iteration bound is the backstop for anything else.
        if result == before_last || iterations > SQRT_MAX_ITERATIONS {
            return if error_of(last) <= error_of(result) {
                Ok((last, iterations))
            } else {
                Ok((result, iterations))
            };
        }
        before_last = last;
        last = result;
        let quotient = x.checked_div(result).ok_or_else(overflow)?;
        result = result
            .checked_add(quotient)
            .ok_or_else(overflow)?
            .checked_div(Decimal::TWO)
            .ok_or_else(overflow)?;
    }
    Ok((result, iterations))
}

/// Checked square root of a [`Positive`], routed through [`d_sqrt`] so that
/// no production path reaches the panicking upstream `sqrt` that
/// `Positive::checked_sqrt` still wraps (#588).
///
/// # Errors
///
/// Returns [`PositiveError::ArithmeticError`] when the iteration overflows
/// or the root cannot be represented as a `Positive`; the error type matches
/// `Positive::checked_sqrt` so call sites keep their conversions.
#[inline]
pub fn p_sqrt(x: &Positive, op: &'static str) -> Result<Positive, PositiveError> {
    let root =
        d_sqrt(x.to_dec(), op).map_err(|e| PositiveError::arithmetic_error(op, &e.to_string()))?;
    Positive::new_decimal(root)
}

/// Converts a Decimal value to f64 without error checking.
///
/// This macro converts a Decimal type to an f64 floating-point value.
/// It's an "unchecked" version that doesn't handle potential conversion errors.
///
/// # Parameters
/// * `$val` - A Decimal value to be converted to f64
///
/// # Example
/// ```rust
/// use rust_decimal_macros::dec;
/// use optionstratlib_core::d2fu;
/// let decimal_value = dec!(10.5);
/// let float_value = d2fu!(decimal_value);
/// ```
#[macro_export]
macro_rules! d2fu {
    ($val:expr) => {
        $crate::model::decimal::decimal_to_f64($val)
    };
}

/// Converts a Decimal value to f64 with error propagation.
///
/// This macro converts a Decimal type to an f64 floating-point value.
/// It propagates any errors that might occur during conversion using the `?` operator.
///
/// # Parameters
/// * `$val` - A Decimal value to be converted to f64
///
#[macro_export]
macro_rules! d2f {
    ($val:expr) => {
        $crate::model::decimal::decimal_to_f64($val)?
    };
}

/// Builds a `NonZeroUsize` from a literal or constant expression.
///
/// Ergonomic shorthand for the `NonZeroUsize::new(N).expect(..)` pattern
/// at call sites that know the value is non-zero by construction (tests,
/// examples, benchmarks). Use this whenever passing literal step or
/// simulation counts to one of the public pricing kernels migrated
/// in #337.
///
/// # Panics
///
/// Panics with a descriptive message if `$val` evaluates to zero. For
/// runtime values coming from JSON, CLI, or external APIs prefer
/// `NonZeroUsize::new(x).ok_or_else(..)` at the boundary instead.
///
/// # Examples
///
/// ```rust
/// use optionstratlib_core::nz;
/// use std::num::NonZeroUsize;
///
/// let steps = nz!(100);
/// assert_eq!(steps.get(), 100);
/// ```
#[macro_export]
macro_rules! nz {
    ($val:expr) => {{
        // The panic expands at the caller, never inside the library: no
        // production code uses `nz!`, only tests, examples and doc examples,
        // all of which pass a non-zero literal. A runtime count must go
        // through `NonZeroUsize::new(x).ok_or_else(..)` at the boundary, as
        // the doc comment above says.
        ::std::num::NonZeroUsize::new($val)
            .unwrap_or_else(|| panic!("nz!({}) must be non-zero", stringify!($val))) // scan-banned: allow -- expands at the caller only; no library code uses `nz!`
    }};
}

/// Converts an f64 value to Decimal without error checking.
///
/// This macro converts an f64 floating-point value to a Decimal type.
/// It's an "unchecked" version that doesn't handle potential conversion errors.
///
/// # Parameters
/// * `$val` - An f64 value to be converted to Decimal
///
/// # Example
/// ```rust
/// use optionstratlib_core::f2du;
/// let float_value = 10.5;
/// let decimal_value = f2du!(float_value);
/// ```
#[macro_export]
macro_rules! f2du {
    ($val:expr) => {
        $crate::model::decimal::f64_to_decimal($val)
    };
}

/// Converts an f64 value to Decimal with error propagation.
///
/// This macro converts an f64 floating-point value to a Decimal type.
/// It propagates any errors that might occur during conversion using the `?` operator.
///
/// # Parameters
/// * `$val` - An f64 value to be converted to Decimal
///
#[macro_export]
macro_rules! f2d {
    ($val:expr) => {
        $crate::model::decimal::f64_to_decimal($val)?
    };
}

/// Conversion helpers' sanity tests (public module to support doctest wiring).
#[cfg(test)]
pub mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_f64_to_decimal_valid() {
        let value = 42.42;
        let result = f64_to_decimal(value);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Decimal::from_str("42.42").unwrap());
    }

    #[test]
    fn test_f64_to_decimal_zero() {
        let value = 0.0;
        let result = f64_to_decimal(value);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Decimal::from_str("0").unwrap());
    }

    #[test]
    fn test_decimal_to_f64_valid() {
        let decimal = Decimal::from_str("42.42").unwrap();
        let result = decimal_to_f64(decimal);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42.42);
    }

    #[test]
    fn test_decimal_to_f64_zero() {
        let decimal = Decimal::from_str("0").unwrap();
        let result = decimal_to_f64(decimal);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0.0);
    }

    /// Reproducer from #588: upstream `Decimal::sqrt` aborts with
    /// `geo mean circuit breaker` on this input because the Newton
    /// iteration oscillates at the 28th decimal instead of converging.
    #[test]
    fn test_d_sqrt_resolves_upstream_circuit_breaker() {
        let x = dec!(4.0000000000000000000000000003);
        let root = d_sqrt(x, "test").unwrap();
        let residual = (root * root - x).abs();
        assert!(
            residual <= dec!(0.0000000000000000000000000001),
            "sqrt({x}) = {root}, residual {residual}"
        );
    }

    /// Every input on which upstream converges must yield the bit-identical
    /// result: same seed, same update step.
    #[test]
    fn test_d_sqrt_matches_upstream_on_converging_inputs() {
        let inputs = [
            Decimal::ZERO,
            Decimal::ONE,
            Decimal::TWO,
            dec!(4),
            dec!(0.25),
            dec!(0.0000000000000000000000000001),
            dec!(123456.789),
            Decimal::MAX,
        ];
        for x in inputs {
            let expected = x.sqrt().unwrap_or_default();
            assert_eq!(d_sqrt(x, "test").unwrap(), expected, "sqrt({x})");
        }
    }

    /// The bit-identical claim, checked byte for byte on a seeded sweep:
    /// random mantissa/scale pairs plus squares of short roots nudged by a
    /// few units in the 28th decimal (the region where upstream can
    /// oscillate). Inputs on which upstream itself aborts are skipped
    /// through `catch_unwind`; a converging upstream result must be
    /// reproduced exactly, `serialize()` bytes included, so a scale
    /// difference cannot hide behind numerical equality.
    #[test]
    fn test_d_sqrt_matches_upstream_on_a_seeded_sweep() {
        use rand::{RngExt, SeedableRng};
        let mut rng = rand::rngs::StdRng::seed_from_u64(0x5eed_5eed_0588);
        let mut inputs: Vec<Decimal> = Vec::with_capacity(6_000);
        for _ in 0..4_000 {
            let mantissa: i64 = rng.random_range(0..=i64::MAX);
            let scale: u32 = rng.random_range(0..=28);
            inputs.push(Decimal::new(mantissa, scale));
        }
        let ulp = dec!(0.0000000000000000000000000001);
        for root in [1u32, 2, 3, 7, 12, 100, 1_000, 65_536] {
            let square = Decimal::from(root) * Decimal::from(root);
            for units in -6i32..=6 {
                let nudge = ulp * Decimal::from(units);
                if let Some(x) = square.checked_add(nudge) {
                    inputs.push(x);
                }
            }
        }
        let mut compared = 0usize;
        let mut skipped = 0usize;
        for x in inputs {
            let upstream = std::panic::catch_unwind(|| x.sqrt());
            match upstream {
                Ok(Some(expected)) => {
                    let actual = d_sqrt(x, "sweep").unwrap();
                    assert_eq!(
                        actual.serialize(),
                        expected.serialize(),
                        "sqrt({x}): upstream {expected}, d_sqrt {actual}"
                    );
                    compared += 1;
                }
                Ok(None) => unreachable!("non-negative inputs only"),
                Err(_) => {
                    // Upstream aborted: the whole point of d_sqrt.
                    assert!(d_sqrt(x, "sweep").is_ok(), "d_sqrt({x}) must be total");
                    skipped += 1;
                }
            }
        }
        assert!(compared > 4_000, "compared {compared}, skipped {skipped}");
    }

    /// The reproducer resolves at the third iteration, not at the bound:
    /// a period-2 cycle is detected as soon as it appears.
    #[test]
    fn test_d_sqrt_resolves_the_cycle_in_a_few_iterations() {
        // The property is a count, not a duration: the period-2 cycle is
        // resolved as soon as it appears instead of running to the
        // 1000-iteration backstop. A wall-clock assertion here failed under
        // coverage instrumentation, which slows every call (#604).
        let oscillating = dec!(4.0000000000000000000000000003);
        let (_, iterations) = sqrt_with_iterations(oscillating, "cycle").unwrap();
        assert!(
            iterations < 10,
            "cycle detection is not immediate: {iterations} iterations"
        );
        assert!(iterations < SQRT_MAX_ITERATIONS);
        // Control: an input upstream converges on takes a comparable number
        // of steps, so the bound above is not vacuous.
        let (_, converging) =
            sqrt_with_iterations(dec!(4.0000000000000000000000000004), "control").unwrap();
        assert!(converging < 60, "control took {converging} iterations");
    }

    #[test]
    fn test_d_sqrt_rejects_negative_input() {
        assert!(d_sqrt(dec!(-1), "test").is_err());
        assert!(d_sqrt(dec!(-0.0000000000000000000000000001), "test").is_err());
    }

    #[test]
    fn test_p_sqrt_matches_d_sqrt() {
        let x = Positive::new_decimal(dec!(4.0000000000000000000000000003)).unwrap();
        let expected = d_sqrt(x.to_dec(), "test").unwrap();
        assert_eq!(p_sqrt(&x, "test").unwrap().to_dec(), expected);
    }
}

#[cfg(test)]
mod tests_decimal_stats {
    use super::*;

    #[test]
    fn test_decimal_stats_empty_and_singleton_return_zero() {
        let empty: Vec<Decimal> = vec![];
        assert_eq!(empty.mean().unwrap(), Decimal::ZERO);
        assert_eq!(empty.std_dev().unwrap(), Decimal::ZERO);
        let one = vec![dec!(3)];
        assert_eq!(one.mean().unwrap(), dec!(3));
        assert_eq!(one.std_dev().unwrap(), Decimal::ZERO);
    }

    #[test]
    fn test_decimal_stats_ordinary_sample_is_unchanged() {
        // The textbook series: mean 5, sample variance 32 / 7.
        let values = vec![
            dec!(2),
            dec!(4),
            dec!(4),
            dec!(4),
            dec!(5),
            dec!(5),
            dec!(7),
            dec!(9),
        ];
        assert_eq!(values.mean().unwrap(), dec!(5));
        let expected = d_div(dec!(32), dec!(7), "test")
            .and_then(|v| d_sqrt(v, "test"))
            .unwrap();
        assert_eq!(values.std_dev().unwrap(), expected);
    }

    #[test]
    fn test_decimal_stats_saturated_sample_reports_overflow_instead_of_aborting() {
        // `iter().sum()` aborted here with `Addition overflowed`, and the
        // squaring in `std_dev` with `Pow overflowed`. Both are a
        // `DecimalError` the caller can read now.
        let values = vec![Decimal::MAX, Decimal::MAX];
        assert!(values.mean().is_err());
        assert!(values.std_dev().is_err());
    }

    #[test]
    fn test_decimal_stats_std_dev_reports_a_deviation_that_overflows() {
        // The mean is finite but a centred deviation is not.
        let values = vec![Decimal::MAX, Decimal::MIN, Decimal::MAX, Decimal::MIN];
        assert!(values.std_dev().is_err());
    }
}

#[cfg(test)]
mod tests_random_generation {
    use super::*;
    use approx::assert_relative_eq;
    use num_traits::ToPrimitive;
    use rand::distr::Distribution;
    use std::collections::HashMap;

    /// Seed of the sampled streams, so every run draws the same values.
    const SEED: u64 = 685;

    #[test]
    fn test_normal_sample_returns() {
        // Run the function multiple times to ensure it always returns a positive value
        let mut rng = crate::utils::deterministic_rng(SEED);
        for _ in 0..1000 {
            let sample = decimal_normal_sample_with(&mut rng);
            assert!(sample <= Decimal::TEN);
            assert!(sample >= -Decimal::TEN);
        }
    }

    #[test]
    fn test_normal_sample_distribution() {
        // Generate a large number of samples to check distribution characteristics
        const NUM_SAMPLES: usize = 10000;
        let mut samples = Vec::with_capacity(NUM_SAMPLES);
        let mut rng = crate::utils::deterministic_rng(SEED);

        for _ in 0..NUM_SAMPLES {
            samples.push(decimal_normal_sample_with(&mut rng).to_f64().unwrap());
        }

        // Calculate mean and standard deviation
        let sum: f64 = samples.iter().sum();
        let mean = sum / NUM_SAMPLES as f64;

        let variance_sum: f64 = samples.iter().map(|&x| (x - mean).powi(2)).sum();
        let std_dev = (variance_sum / NUM_SAMPLES as f64).sqrt();

        // Check if the distribution approximately matches a standard normal
        // Note: These tests use wide tolerances since we're working with random samples
        assert_relative_eq!(mean, 0.0, epsilon = 0.04);
        assert_relative_eq!(std_dev, 1.0, epsilon = 0.03);
    }

    #[test]
    fn test_normal_distribution_transformation() {
        let mut t_rng = crate::utils::deterministic_rng(SEED);
        // Deliberately a distribution with a negative mean.
        let normal = rand_distr::Normal::new(-1.0, 0.5).unwrap();

        // Count occurrences of values after transformation
        let mut value_counts: HashMap<i32, usize> = HashMap::new();
        const SAMPLES: usize = 5000;

        for _ in 0..SAMPLES {
            let raw_sample = normal.sample(&mut t_rng);
            let positive_sample = raw_sample.to_f64().unwrap();

            // Bucket values to the nearest integer for counting
            let bucket = (positive_sample.round() as i32).max(0);
            *value_counts.entry(bucket).or_insert(0) += 1;
        }

        // Verify that zero values appear frequently (due to negative values being transformed)
        assert!(value_counts.get(&0).unwrap_or(&0) > &(SAMPLES / 10));

        // Verify that we have a range of positive values
        let max_bucket = value_counts.keys().max().unwrap_or(&0);
        assert!(*max_bucket > 0);
    }

    #[test]
    fn test_normal_sample_consistency() {
        // This test ensures that multiple calls in sequence produce different values
        // Unseeded on purpose: the test is about the thread-RNG entry point.
        let sample1 = decimal_normal_sample();
        let sample2 = decimal_normal_sample();
        let sample3 = decimal_normal_sample();

        // It's statistically extremely unlikely to get the same value three times in a row
        // This verifies that the RNG is properly producing different values
        assert!(sample1 != sample2 || sample2 != sample3);
    }

    #[test]
    fn test_decimal_normal_sample_with_same_seed_is_identical() {
        let mut first = crate::utils::deterministic_rng(42);
        let mut second = crate::utils::deterministic_rng(42);
        for _ in 0..1000 {
            assert_eq!(
                decimal_normal_sample_with(&mut first),
                decimal_normal_sample_with(&mut second)
            );
        }
    }

    #[test]
    fn test_decimal_normal_sample_with_is_the_converted_standard_normal_draw() {
        // The helper adds nothing to the stream but the `Decimal` conversion.
        let mut helper = crate::utils::deterministic_rng(7);
        let mut raw = crate::utils::deterministic_rng(7);
        for _ in 0..1000 {
            let draw: f64 = StandardNormal.sample(&mut raw);
            assert_eq!(
                decimal_normal_sample_with(&mut helper),
                Decimal::from_f64(draw).unwrap_or(Decimal::ZERO)
            );
        }
    }

    #[test]
    fn test_decimal_normal_sample_with_seed_pins_first_draws() {
        let mut rng = crate::utils::deterministic_rng(42);
        let draws: Vec<Decimal> = (0..3)
            .map(|_| decimal_normal_sample_with(&mut rng))
            .collect();
        // Recorded from this implementation (`StdRng`, rand 0.10).
        let expected = vec![
            dec!(0.0694279183619634),
            dec!(0.1329381219941254),
            dec!(0.2625763573739537),
        ];
        assert_eq!(draws, expected);
    }

    #[test]
    fn test_decimal_uniform_sample_with_range_is_half_open_unit_interval() {
        let mut rng = crate::utils::deterministic_rng(SEED);
        for _ in 0..100_000 {
            let u = decimal_uniform_sample_with(&mut rng);
            assert!(u >= Decimal::ZERO, "{u} below zero");
            assert!(u < Decimal::ONE, "{u} not below one");
            assert_eq!(u.scale(), UNIFORM_SAMPLE_SCALE, "{u} off the grid");
        }
    }

    #[test]
    fn test_decimal_uniform_sample_with_same_seed_is_identical() {
        let mut first = crate::utils::deterministic_rng(42);
        let mut second = crate::utils::deterministic_rng(42);
        for _ in 0..1000 {
            assert_eq!(
                decimal_uniform_sample_with(&mut first),
                decimal_uniform_sample_with(&mut second)
            );
        }
    }

    #[test]
    fn test_decimal_uniform_sample_with_seed_pins_first_draws() {
        let mut rng = crate::utils::deterministic_rng(42);
        let draws: Vec<Decimal> = (0..3)
            .map(|_| decimal_uniform_sample_with(&mut rng))
            .collect();
        // Recorded from this implementation (`StdRng`, rand 0.10).
        let expected = vec![
            dec!(0.526557409002773877),
            dec!(0.542725209903143899),
            dec!(0.636465099143894996),
        ];
        assert_eq!(draws, expected);
    }

    #[test]
    fn test_decimal_uniform_sample_with_moments_match_uniform() {
        // U(0,1) has mean 1/2 and variance 1/12. Over N draws the sample
        // mean has standard error sqrt(1/12 / N) = 0.000913 at N = 100_000;
        // the bound is five standard errors. The empirical frequency of
        // `u < p` is Binomial(N, p) / N, standard error sqrt(p (1-p) / N),
        // and each is checked within five standard errors as well.
        const N: usize = 100_000;
        let mut rng = crate::utils::deterministic_rng(SEED);
        let thresholds = [dec!(0.004), dec!(0.1), dec!(0.5), dec!(0.9)];
        let mut below = [0_u64; 4];
        let mut sum = 0.0_f64;
        for _ in 0..N {
            let u = decimal_uniform_sample_with(&mut rng);
            sum += u.to_f64().unwrap();
            for (count, p) in below.iter_mut().zip(thresholds) {
                if u < p {
                    *count += 1;
                }
            }
        }
        let n = N as f64;
        let mean_se = (1.0 / 12.0 / n).sqrt();
        assert!((sum / n - 0.5).abs() < 5.0 * mean_se, "mean {}", sum / n);
        for (count, p) in below.iter().zip(thresholds) {
            let p = p.to_f64().unwrap();
            let freq = *count as f64 / n;
            let se = (p * (1.0 - p) / n).sqrt();
            assert!((freq - p).abs() < 5.0 * se, "P(u < {p}) = {freq}");
        }
    }

    /// Generator that returns the same 64-bit word on every draw.
    struct ConstantWord(u64);

    impl rand::TryRng for ConstantWord {
        type Error = std::convert::Infallible;

        fn try_next_u32(&mut self) -> Result<u32, Self::Error> {
            Ok((self.0 >> 32) as u32)
        }

        fn try_next_u64(&mut self) -> Result<u64, Self::Error> {
            Ok(self.0)
        }

        fn try_fill_bytes(&mut self, dst: &mut [u8]) -> Result<(), Self::Error> {
            dst.fill(0);
            Ok(())
        }
    }

    #[test]
    fn test_decimal_uniform_sample_with_extreme_words_hit_grid_ends() {
        // The all-zero word maps to the lowest grid point, the all-ones word
        // to the highest: the range is exactly `[0, 1 - 10^-18]`.
        assert_eq!(
            decimal_uniform_sample_with(&mut ConstantWord(0)),
            Decimal::ZERO
        );
        assert_eq!(
            decimal_uniform_sample_with(&mut ConstantWord(u64::MAX)),
            dec!(0.999999999999999999)
        );
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod checked_helpers_tests {
    use super::*;

    #[test]
    fn d_add_happy_path() {
        let result = d_add(dec!(1.25), dec!(2.50), "test::add");
        assert_eq!(result.unwrap(), dec!(3.75));
    }

    #[test]
    fn d_add_overflow_on_max_plus_max() {
        let err = d_add(Decimal::MAX, Decimal::MAX, "test::add").unwrap_err();
        match err {
            DecimalError::Overflow { operation, .. } => assert_eq!(operation, "test::add"),
            other => panic!("expected Overflow, got {other:?}"),
        }
    }

    /// A checked add that reports `Ok` has not necessarily moved the value.
    ///
    /// `Decimal::MAX` has scale 0 and a mantissa filling all 96 bits. Adding a
    /// fractional addend needs a 29th significant digit, which does not exist,
    /// so `rust_decimal` rescales the addend down to scale 0 instead of failing:
    /// anything below half a unit rounds to zero, the addition succeeds, and the
    /// result is `Decimal::MAX` again. The overflow path is never reached. The
    /// silent window is `|x| < 0.5`, not `|x| < 1` — at `0.5` the addend rounds
    /// away from zero to `1` and the sum does overflow, which is why the two
    /// halves of this test sit three hundredths apart.
    ///
    /// This has already broken two call sites in this crate:
    ///
    /// - `chains::optiondata::unlock` lifted a locked quote by one tick and read
    ///   `Ok` as "lifted", so it re-emitted the very quote it existed to remove.
    /// - `greeks::equations::alpha_from` returns `Decimal::MAX` as its
    ///   "theta vanished" sentinel. Summing one such leg with an ordinary one
    ///   returned `Ok` with the running total standing still and the ordinary
    ///   leg's alpha silently dropped, so that aggregation needs an explicit
    ///   guard on the operand; no arithmetic check can catch it.
    ///
    /// The rule this implies: when you nudge a value and need to know it moved,
    /// compare the result against the input. `checked_*` returning `Ok` is not
    /// that check — it only says the result was representable.
    #[test]
    fn checked_add_near_max_reports_ok_without_moving_the_value() {
        // Below half a unit: rounds to zero, succeeds, and stands still.
        assert_eq!(
            Decimal::MAX.checked_add(dec!(0.01)),
            Some(Decimal::MAX),
            "a fractional addend rescales away instead of overflowing"
        );
        assert_eq!(
            d_add(Decimal::MAX, dec!(0.47), "test::add").unwrap(),
            Decimal::MAX,
            "the crate helper inherits the same behaviour"
        );

        // At half a unit and above: rounds away from zero and does overflow, so
        // the boundary is visible rather than implied.
        assert_eq!(Decimal::MAX.checked_add(dec!(0.5)), None);
        assert!(d_add(Decimal::MAX, dec!(0.5), "test::add").is_err());
    }

    #[test]
    fn d_sub_happy_path() {
        let result = d_sub(dec!(10), dec!(3.5), "test::sub");
        assert_eq!(result.unwrap(), dec!(6.5));
    }

    #[test]
    fn d_sub_overflow_on_min_minus_max() {
        let err = d_sub(Decimal::MIN, Decimal::MAX, "test::sub").unwrap_err();
        assert!(
            matches!(err, DecimalError::Overflow { operation, .. } if operation == "test::sub")
        );
    }

    #[test]
    fn d_mul_happy_path() {
        let result = d_mul(dec!(2.5), dec!(4), "test::mul");
        assert_eq!(result.unwrap(), dec!(10.0));
    }

    #[test]
    fn d_mul_overflow_on_max_times_two() {
        let err = d_mul(Decimal::MAX, dec!(2), "test::mul").unwrap_err();
        assert!(
            matches!(err, DecimalError::Overflow { operation, .. } if operation == "test::mul")
        );
    }

    #[test]
    fn d_div_happy_path_exact() {
        let result = d_div(dec!(10), dec!(4), "test::div");
        assert_eq!(result.unwrap(), dec!(2.5));
    }

    #[test]
    fn d_div_applies_banker_rounding() {
        // Divide by three is recurring and must round half-to-even at the default scale.
        let result = d_div(dec!(1), dec!(3), "test::div").unwrap();
        // `1 / 3` at scale 28 under banker's rounding produces the canonical
        // `0.3333333333333333333333333333` (scale 28). Any other trailing digit
        // would signal the rounding strategy drifted.
        assert_eq!(result, dec!(0.3333333333333333333333333333));
    }

    #[test]
    fn d_div_zero_denominator_returns_arithmetic_error() {
        let err = d_div(dec!(1), Decimal::ZERO, "test::div").unwrap_err();
        assert!(matches!(err, DecimalError::ArithmeticError { .. }));
    }

    #[test]
    fn d_div_tag_is_preserved_on_overflow() {
        // `Decimal::MIN / 0.5` overflows because the quotient is 2 * MIN.
        let err = d_div(Decimal::MIN, dec!(0.5), "test::div").unwrap_err();
        match err {
            DecimalError::Overflow { operation, .. } => assert_eq!(operation, "test::div"),
            other => panic!("expected Overflow, got {other:?}"),
        }
    }

    #[test]
    fn d_sum_empty_returns_zero() {
        assert_eq!(d_sum(&[], "test::sum").unwrap(), Decimal::ZERO);
    }

    #[test]
    fn d_sum_happy_path() {
        let result = d_sum(&[dec!(1.5), dec!(2.25), dec!(-0.75), dec!(10)], "test::sum");
        assert_eq!(result.unwrap(), dec!(13));
    }

    #[test]
    fn d_sum_overflow_returns_tagged_error() {
        let err = d_sum(&[Decimal::MAX, Decimal::MAX], "test::sum").unwrap_err();
        assert!(
            matches!(err, DecimalError::Overflow { operation, .. } if operation == "test::sum")
        );
    }

    #[test]
    fn d_sum_iter_empty_returns_zero() {
        let empty: std::iter::Empty<Decimal> = std::iter::empty();
        assert_eq!(d_sum_iter(empty, "test::sum_iter").unwrap(), Decimal::ZERO);
    }

    #[test]
    fn d_sum_iter_happy_path_matches_d_sum() {
        let values = [dec!(1.5), dec!(2.25), dec!(-0.75), dec!(10)];
        let via_iter = d_sum_iter(values.iter().copied(), "test::sum_iter").unwrap();
        let via_slice = d_sum(&values, "test::sum_iter").unwrap();
        assert_eq!(via_iter, dec!(13));
        assert_eq!(via_iter, via_slice);
    }

    #[test]
    fn d_sum_iter_accepts_lazy_map() {
        // Exercise the no-allocation pathway: a `map` adapter over a range
        // should aggregate without any intermediate `Vec`.
        let sum = d_sum_iter((1i64..=4).map(Decimal::from), "test::sum_iter_lazy").unwrap();
        assert_eq!(sum, dec!(10));
    }

    #[test]
    fn d_sum_iter_overflow_returns_tagged_error() {
        let err = d_sum_iter([Decimal::MAX, Decimal::MAX], "test::sum_iter").unwrap_err();
        assert!(
            matches!(err, DecimalError::Overflow { operation, .. } if operation == "test::sum_iter")
        );
    }
}

#[cfg(test)]
mod tests_decimal_to_f64_rounding {
    use super::*;
    use num_traits::ToPrimitive;
    use rust_decimal_macros::dec;

    fn converted(value: Decimal) -> f64 {
        match decimal_to_f64(value) {
            Ok(v) => v,
            Err(e) => panic!("every Decimal converts: {value}: {e}"),
        }
    }

    /// The digits of `mantissa * 10^-scale`, written without `Decimal`'s
    /// `Display`, so the expectation does not share code with the helper.
    fn digits(negative: bool, mantissa: u128, scale: u32) -> String {
        let raw = mantissa.to_string();
        let width = match usize::try_from(scale) {
            Ok(w) => w,
            Err(e) => panic!("scale {scale} fits usize: {e}"),
        };
        let padded = format!("{raw:0>w$}", w = width + 1);
        let (integer, fraction) = padded.split_at(padded.len() - width);
        let sign = if negative { "-" } else { "" };
        if fraction.is_empty() {
            format!("{sign}{integer}")
        } else {
            format!("{sign}{integer}.{fraction}")
        }
    }

    /// A fixed-seed 64-bit linear congruential generator (Knuth's MMIX
    /// constants), so the sweep needs no RNG dependency and draws the same
    /// inputs on every run. The step is taken in `u128` with an explicit
    /// modulus; `draw` returns the high 32 bits, the well-mixed ones.
    struct Lcg(u64);

    impl Lcg {
        fn draw(&mut self) -> u64 {
            let step = (u128::from(self.0) * 6_364_136_223_846_793_005 + 1_442_695_040_888_963_407)
                % (1_u128 << 64);
            self.0 = match u64::try_from(step) {
                Ok(state) => state,
                Err(e) => panic!("a value below 2^64 fits u64: {e}"),
            };
            self.0 >> 32
        }
    }

    #[test]
    fn test_decimal_to_f64_seeded_15_to_28_places_matches_parse() {
        let mut rng = Lcg(0x0656_f64d_ec1a);
        // Draws on which `Decimal::to_f64` misses the nearest `f64`: the sweep
        // must reach them for the comparison to mean anything.
        let mut plain_misses = 0_u32;
        for _ in 0..20_000 {
            // A 96-bit mantissa (the full `Decimal` range), a scale of 15 to
            // 28 and a sign, all from the same draw stream.
            let mantissa = (u128::from(rng.draw()) << 64)
                | (u128::from(rng.draw()) << 32)
                | u128::from(rng.draw());
            let scale = match u32::try_from(rng.draw() % 14) {
                Ok(offset) => 15 + offset,
                Err(e) => panic!("a value below 14 fits u32: {e}"),
            };
            let negative = rng.draw() % 2 == 1;
            let signed = match i128::try_from(mantissa) {
                Ok(m) if negative => -m,
                Ok(m) => m,
                Err(e) => panic!("96-bit mantissa fits i128: {e}"),
            };
            let value = match Decimal::try_from_i128_with_scale(signed, scale) {
                Ok(v) => v,
                Err(e) => panic!("96-bit mantissa with scale {scale} is a Decimal: {e}"),
            };
            let text = digits(negative && mantissa != 0, mantissa, scale);
            let expected: f64 = match text.parse() {
                Ok(v) => v,
                Err(e) => panic!("{text} parses: {e}"),
            };
            assert_eq!(
                converted(value).to_bits(),
                expected.to_bits(),
                "{value} ({text})"
            );
            if value.to_f64().map(f64::to_bits) != Some(expected.to_bits()) {
                plain_misses += 1;
            }
        }
        assert!(plain_misses > 0, "the sweep never reached a to_f64 miss");
    }

    #[test]
    fn test_decimal_to_f64_edges_match_parse() {
        let edges = [
            Decimal::ZERO,
            Decimal::ONE,
            Decimal::NEGATIVE_ONE,
            Decimal::MAX,
            Decimal::MIN,
            Decimal::new(1, 28),
            Decimal::new(-1, 28),
            Decimal::from_i128_with_scale(79_228_162_514_264_337_593_543_950_335, 28),
            Decimal::from_i128_with_scale(-79_228_162_514_264_337_593_543_950_335, 15),
            dec!(0.95),
            dec!(-0.37),
            dec!(0.10),
        ];
        for value in edges {
            let expected: f64 = match value.to_string().parse() {
                Ok(v) => v,
                Err(e) => panic!("{value} parses: {e}"),
            };
            assert_eq!(converted(value).to_bits(), expected.to_bits(), "{value}");
        }
    }

    /// `Decimal::to_f64`, which `decimal_to_f64` went through before #670,
    /// lands away from the nearest `f64` on these; the conversion returns the
    /// `f64` literal written with the same digits.
    #[test]
    fn test_decimal_to_f64_where_to_f64_differs_matches_literal() {
        let cases = [
            (dec!(2.999789999999902), 2.999_789_999_999_902_f64),
            (
                dec!(0.1234567890123456789012345678),
                // The nearest `f64` to the 28 digits, written in its
                // shortest form (the full literal trips `excessive_precision`).
                0.123_456_789_012_345_68_f64,
            ),
        ];
        for (value, literal) in cases {
            assert_eq!(converted(value).to_bits(), literal.to_bits(), "{value}");
            let plain = match value.to_f64() {
                Some(v) => v,
                None => panic!("{value} has an f64 form"),
            };
            assert_ne!(plain.to_bits(), literal.to_bits(), "{value}");
        }
    }

    /// The case #670 names: `2.999789999999902` has 15 decimal places, and
    /// `Decimal::to_f64` returns `2.999789999999903`, three ULPs above the
    /// nearest `f64`. The merged conversion returns the nearest one.
    #[test]
    fn test_decimal_to_f64_fifteen_places_returns_nearest_not_to_f64() {
        let value = dec!(2.999789999999902);
        let nearest = 2.999_789_999_999_902_f64;
        let former = 2.999_789_999_999_903_f64;
        assert_eq!(converted(value).to_bits(), nearest.to_bits());
        assert_eq!(value.to_f64().map(f64::to_bits), Some(former.to_bits()));
        assert_eq!(former.to_bits() - nearest.to_bits(), 3);
    }

    /// `2^53`, the largest significand an `f64` holds.
    const TWO_POW_53: u128 = 1 << 53;

    /// Draws a `Decimal` with a mantissa below `bound` and a scale of 0 to
    /// 28, and the digits it is written with.
    fn draw_decimal(rng: &mut Lcg, bound: u128) -> (Decimal, String) {
        let mantissa = ((u128::from(rng.draw()) << 64)
            | (u128::from(rng.draw()) << 32)
            | u128::from(rng.draw()))
            % bound;
        let scale = match u32::try_from(rng.draw() % 29) {
            Ok(s) => s,
            Err(e) => panic!("a value below 29 fits u32: {e}"),
        };
        let negative = rng.draw() % 2 == 1;
        let signed = match i128::try_from(mantissa) {
            Ok(m) if negative => -m,
            Ok(m) => m,
            Err(e) => panic!("96-bit mantissa fits i128: {e}"),
        };
        let value = match Decimal::try_from_i128_with_scale(signed, scale) {
            Ok(v) => v,
            Err(e) => panic!("mantissa with scale {scale} is a Decimal: {e}"),
        };
        (value, digits(negative && mantissa != 0, mantissa, scale))
    }

    fn assert_matches_parse(value: Decimal, text: &str) {
        let expected: f64 = match text.parse() {
            Ok(v) => v,
            Err(e) => panic!("{text} parses: {e}"),
        };
        assert_eq!(
            converted(value).to_bits(),
            expected.to_bits(),
            "{value} ({text})"
        );
    }

    /// Mantissas up to `2^53` (exact in `f64`) and over the full 96 bits,
    /// each at every scale from 0 to 28, against the parse of the written
    /// digits.
    #[test]
    fn test_decimal_to_f64_seeded_every_scale_matches_parse() {
        let mut rng = Lcg(0x0670_f64d_ec1a);
        for bound in [TWO_POW_53 + 1, 1_u128 << 96] {
            for _ in 0..20_000 {
                let (value, text) = draw_decimal(&mut rng, bound);
                assert_matches_parse(value, &text);
            }
        }
    }

    /// Exact ties between two `f64`s round to the even significand: `2^53 +
    /// 1` and `2^53 + 3` sit halfway between neighbours, as integers and
    /// scaled by `5^s / 10^s = 2^-s`, which leaves the division by `5^s` with
    /// no remainder.
    #[test]
    fn test_decimal_to_f64_exact_ties_round_to_even() {
        for offset in [1_u128, 3] {
            for scale in [0_u32, 1, 5, 18] {
                let mantissa = (TWO_POW_53 + offset) * 5_u128.pow(scale);
                let signed = match i128::try_from(mantissa) {
                    Ok(m) => m,
                    Err(e) => panic!("{mantissa} fits i128: {e}"),
                };
                for signed in [signed, -signed] {
                    let value = Decimal::from_i128_with_scale(signed, scale);
                    assert_matches_parse(value, &value.to_string());
                }
            }
        }
        // `2^53 + 1` rounds down to `2^53`, `2^53 + 3` up to `2^53 + 4`.
        let down = Decimal::from_i128_with_scale(9_007_199_254_740_993, 0);
        let up = Decimal::from_i128_with_scale(9_007_199_254_740_995, 0);
        assert_eq!(
            converted(down).to_bits(),
            9_007_199_254_740_992_f64.to_bits()
        );
        assert_eq!(converted(up).to_bits(), 9_007_199_254_740_996_f64.to_bits());
    }

    /// Around `2^53`, the largest exact significand, at the smallest and
    /// largest scales.
    #[test]
    fn test_decimal_to_f64_around_two_pow_53_matches_parse() {
        let limit = match i128::try_from(TWO_POW_53) {
            Ok(l) => l,
            Err(e) => panic!("2^53 fits i128: {e}"),
        };
        for mantissa in [limit - 1, limit, limit + 1, limit + 2, limit + 3] {
            for scale in [0, 1, 21, 22, 23, 28] {
                for signed in [mantissa, -mantissa] {
                    let value = Decimal::from_i128_with_scale(signed, scale);
                    assert_matches_parse(value, &value.to_string());
                }
            }
        }
    }

    /// Zero of either sign and at any scale converts to `+0.0`.
    #[test]
    fn test_decimal_to_f64_zero_any_sign_or_scale_is_positive_zero() {
        let mut negative_zero = Decimal::new(0, 25);
        negative_zero.set_sign_negative(true);
        for value in [Decimal::ZERO, Decimal::new(0, 25), negative_zero] {
            assert_eq!(converted(value).to_bits(), 0.0_f64.to_bits(), "{value:?}");
        }
    }
}

/// [`d_ln_f64`] against the 28-place [`d_ln`] (#857).
#[cfg(test)]
mod tests_d_ln_f64 {
    use super::*;
    use rust_decimal_macros::dec;

    /// The owner's bound (2026-10-09): 1e-12 relative.
    const TOLERANCE: Decimal = dec!(0.000000000001);

    fn relative_error(x: Decimal) -> Decimal {
        let fast = d_ln_f64(x, "test").expect("fast ln");
        let exact = d_ln(x, "test").expect("series ln");
        if exact.is_zero() {
            return fast.abs();
        }
        (fast - exact).abs() / exact.abs()
    }

    #[test]
    fn test_d_ln_f64_one_is_exactly_zero() {
        assert_eq!(d_ln_f64(Decimal::ONE, "test").ok(), Some(Decimal::ZERO));
    }

    #[test]
    fn test_d_ln_f64_rejects_non_positive() {
        assert!(d_ln_f64(Decimal::ZERO, "test").is_err());
        assert!(d_ln_f64(dec!(-1), "test").is_err());
    }

    #[test]
    fn test_d_ln_f64_within_the_owner_bound() {
        let inputs = [
            dec!(0.0000000000000000000000000001),
            dec!(0.000001),
            dec!(0.3),
            dec!(0.5),
            dec!(0.5000000001),
            dec!(0.99),
            dec!(0.9999999),
            dec!(0.9999999999999),
            dec!(1.0000000000001),
            dec!(1.0000001),
            dec!(1.01),
            dec!(1.4999999999),
            dec!(1.5),
            dec!(2.718281828459045),
            dec!(57.81),
            dec!(1000000),
            dec!(79228162514264337593543950335),
        ];
        let mut worst = Decimal::ZERO;
        for x in inputs {
            let error = relative_error(x);
            assert!(error <= TOLERANCE, "ln({x}): relative error {error}");
            worst = worst.max(error);
        }
        // Measured 2026-10-09: below 1e-15; held at 1e-14 so a regression
        // shows.
        assert!(worst < dec!(0.00000000000001), "worst {worst}");
    }
}
