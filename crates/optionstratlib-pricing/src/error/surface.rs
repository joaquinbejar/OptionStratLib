/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 6/11/26
******************************************************************************/

//! Target crate (ADR-0001 D6): **pricing**. Owns `ImpliedVolatilitySurfaceError`.
//!
//! [`ImpliedVolatilitySurfaceError::NoFittedExpiry`] carries the quotes it rejected and
//! the expiries it could not fit, and `error` is a leaf of this crate's internal graph, so
//! those types live here. Their public path is `optionstratlib_pricing::volatility`.

use crate::error::{PricingError, StaticArbitrageError, SviError};
use chrono::{DateTime, TimeDelta, Utc};
use optionstratlib_core::error::DecimalError;
use optionstratlib_core::model::types::OptionStyle;
use optionstratlib_core::model::{ExpirationDateError, Positive, PositiveError};
use rust_decimal::Decimal;
use thiserror::Error;

/// One bid and ask quote of an option, as the surface reads it.
///
/// It is priced as a European option, whichever way the instrument exercises. See
/// [`ImpliedVolatilitySurface`](crate::volatility::ImpliedVolatilitySurface).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OptionQuote {
    /// When the option expires.
    pub expiry: DateTime<Utc>,
    /// The strike, in the units of the spot.
    pub strike: Positive,
    /// Call or put.
    pub style: OptionStyle,
    /// The bid, per unit of underlying.
    pub bid: Positive,
    /// The ask, per unit of underlying.
    pub ask: Positive,
    /// When the quote was observed. A quote later than the as-of time cannot change
    /// the surface.
    pub quoted_at: DateTime<Utc>,
}

/// Why a quote is not in the surface. See [`ImpliedVolatilitySurface`](crate::volatility::ImpliedVolatilitySurface) for the order of the rules.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum QuoteRejectionReason {
    /// The quote is later than the as-of time.
    LaterThanAsOf,
    /// Its expiry is not after the as-of time.
    ExpiredAtAsOf,
    /// Its strike is zero.
    ZeroStrike,
    /// Its bid is above its ask.
    Crossed,
    /// Its bid is zero.
    ZeroBid,
    /// It is older than the limit at the as-of time.
    Stale {
        /// Its age at the as-of time.
        age: TimeDelta,
        /// [`ImpliedVolatilitySurfaceConfig::max_quote_age`](crate::volatility::ImpliedVolatilitySurfaceConfig::max_quote_age).
        limit: TimeDelta,
    },
    /// Another quote of the same expiry, strike and style is later, or as late and
    /// tighter, or as tight with a lower bid, or identical.
    Superseded,
    /// Its mid is outside the no-arbitrage band of a European option.
    OutsideNoArbitrageBand {
        /// The mid of the quote.
        mid: Decimal,
        /// The bottom of the band.
        lower: Decimal,
        /// The top of the band.
        upper: Decimal,
    },
    /// Its mid prices at or below the price at the volatility floor.
    BelowVolatilityFloor,
    /// Its mid prices above the price at a volatility of 500%.
    AboveVolatilityCap,
    /// The implied volatility solve did not converge.
    NoConvergence {
        /// The last volatility evaluated.
        last_volatility: Positive,
    },
    /// The arithmetic of its band, its mid or its solve failed.
    NumericalFailure {
        /// What failed.
        reason: String,
    },
    /// Its vega is zero, or its weight has no value in a `Decimal`.
    NoWeight,
}

/// A quote that is not in the surface, and why.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuoteRejection {
    /// The quote as it was supplied.
    pub quote: OptionQuote,
    /// The first rule it broke.
    pub reason: QuoteRejectionReason,
}

/// Why an expiry has no slice.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum UnfittedReason {
    /// Too few of its quotes are clean, or have a weight, to fix five parameters.
    TooFewCleanQuotes {
        /// The usable quotes.
        clean: usize,
        /// The minimum: five.
        required: usize,
    },
    /// Its clean quotes repeat strikes, leaving too few distinct ones.
    TooFewDistinctStrikes {
        /// The distinct strikes among the usable quotes.
        distinct: usize,
        /// The minimum: five.
        required: usize,
    },
    /// The best fit is further from the quotes than the limit.
    RmseAboveLimit {
        /// The weighted RMSE of the best fit over the weighted mean observed implied
        /// volatility.
        relative_rmse: Positive,
        /// [`SviCalibrationConfig::max_relative_rmse`](crate::volatility::SviCalibrationConfig::max_relative_rmse).
        limit: Positive,
        /// The weighted RMSE of the best fit, in implied volatility.
        rmse: Positive,
    },
    /// The calibration failed some other way, as [`calibrate_svi`](crate::volatility::calibrate_svi) reported it.
    CalibrationFailed {
        /// The message of the error.
        reason: String,
    },
}

impl From<SviError> for UnfittedReason {
    fn from(error: SviError) -> Self {
        match error {
            SviError::TooFewObservations { got, required } => Self::TooFewCleanQuotes {
                clean: got,
                required,
            },
            SviError::InsufficientDistinctStrikes { distinct, required } => {
                Self::TooFewDistinctStrikes { distinct, required }
            }
            SviError::RmseAboveLimit {
                relative_rmse,
                limit,
                rmse,
            } => Self::RmseAboveLimit {
                relative_rmse,
                limit,
                rmse,
            },
            other => Self::CalibrationFailed {
                reason: other.to_string(),
            },
        }
    }
}

/// An expiry with a quote at or before the as-of time and no slice.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnfittedExpiry {
    /// The expiry.
    pub expiry: DateTime<Utc>,
    /// Its time to expiry in years, or `None` if it is not after the as-of time.
    pub time_to_expiry: Option<Positive>,
    /// How many of its quotes were clean, and so are in no slice and in no rejection.
    pub clean_quotes: usize,
    /// Why it has no slice.
    pub reason: UnfittedReason,
}

/// Reasons why a calibrated implied volatility surface cannot be built, read,
/// shocked or checked.
///
/// A quote the cleaning rejects and an expiry that cannot be fitted are not errors:
/// they go in the surface, in
/// [`ImpliedVolatilitySurface::rejections`](crate::volatility::ImpliedVolatilitySurface::rejections)
/// and
/// [`ImpliedVolatilitySurface::unfitted`](crate::volatility::ImpliedVolatilitySurface::unfitted).
/// Only a surface with no fitted expiry at all is an error, and it carries both.
///
/// The aggregate `optionstratlib::error::Error` has no variant for it yet: adding
/// one to that exhaustive enum is breaking, and it comes with the 0.24.0 bump.
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum ImpliedVolatilitySurfaceError {
    /// The spot is zero, so nothing has a forward.
    #[error("the spot of a volatility surface must be positive")]
    ZeroSpot,

    /// The age above which a quote is stale is negative, so every quote would be.
    #[error("the largest quote age must not be negative, got {limit}")]
    NegativeQuoteAgeLimit {
        /// The rejected limit.
        limit: TimeDelta,
    },

    /// The points per expiry of the grid of the arbitrage report are outside the range
    /// allowed: too few to see a slice at all, or too many to allocate and check.
    #[error("the arbitrage grid has {points} points per expiry, outside [{minimum}, {maximum}]")]
    ArbitrageGridPointsOutOfRange {
        /// The points per expiry asked for.
        points: usize,
        /// The fewest allowed: as many as a raw SVI slice has parameters.
        minimum: usize,
        /// The most allowed.
        maximum: usize,
    },

    /// No expiry could be fitted, so there is no surface. Both lists say why.
    #[error(
        "no expiry could be fitted: {} unfitted expiries and {} rejected quotes",
        .unfitted.len(),
        .rejections.len()
    )]
    NoFittedExpiry {
        /// Each expiry with a quote at or before the as-of time, and why it has no fit.
        unfitted: Vec<UnfittedExpiry>,
        /// Each rejected quote and why, in the order of the input.
        rejections: Vec<QuoteRejection>,
    },

    /// The expiry asked for is not after the as-of time of the surface.
    #[error("expiry {expiry} is not after the as-of time {as_of}")]
    ExpiryNotAfterAsOf {
        /// The expiry asked for.
        expiry: DateTime<Utc>,
        /// The as-of time of the surface.
        as_of: DateTime<Utc>,
    },

    /// The time to expiry asked for is zero, so total variance has no implied
    /// volatility.
    #[error("the time to expiry asked of a volatility surface must be positive")]
    ZeroTimeToExpiry,

    /// The instant asked about is before the as-of time, so the surface has no age
    /// at it: it was built from data that instant had not seen.
    #[error("instant {instant} is before the as-of time {as_of}")]
    InstantBeforeAsOf {
        /// The instant asked about.
        instant: DateTime<Utc>,
        /// The as-of time of the surface.
        as_of: DateTime<Utc>,
    },

    /// The log moneyness asked for is further beyond the quotes of an expiry the
    /// answer reads than the policy allows.
    #[error(
        "log moneyness {log_moneyness} at expiry {time_to_expiry} is {extension} beyond quotes spanning [{lowest}, {highest}], and the policy allows {limit}"
    )]
    WingOutOfPolicy {
        /// The log moneyness asked for.
        log_moneyness: Decimal,
        /// The time to expiry asked for, in years.
        time_to_expiry: Positive,
        /// The lowest log moneyness of the quotes of the slice furthest from it.
        lowest: Decimal,
        /// The highest log moneyness of the quotes of the slice furthest from it.
        highest: Decimal,
        /// How far beyond those quotes it is, in log moneyness.
        extension: Positive,
        /// [`ExtrapolationPolicy::max_wing_extension`](crate::volatility::ExtrapolationPolicy::max_wing_extension).
        limit: Positive,
    },

    /// The time to expiry asked for is further before the first fitted expiry than
    /// the policy allows.
    #[error(
        "time to expiry {time_to_expiry} is before the first fitted expiry {first} by more than the policy allows, {limit}"
    )]
    ShortExpiryOutOfPolicy {
        /// The time to expiry asked for, in years.
        time_to_expiry: Positive,
        /// The first fitted expiry, in years.
        first: Positive,
        /// [`ExtrapolationPolicy::max_short_extension`](crate::volatility::ExtrapolationPolicy::max_short_extension).
        limit: Positive,
    },

    /// The time to expiry asked for is further after the last fitted expiry than
    /// the policy allows.
    #[error(
        "time to expiry {time_to_expiry} is after the last fitted expiry {last} by more than the policy allows, {limit}"
    )]
    LongExpiryOutOfPolicy {
        /// The time to expiry asked for, in years.
        time_to_expiry: Positive,
        /// The last fitted expiry, in years.
        last: Positive,
        /// [`ExtrapolationPolicy::max_long_extension`](crate::volatility::ExtrapolationPolicy::max_long_extension).
        limit: Positive,
    },

    /// A grid to export has no point, so the export would say nothing.
    #[error("the export grid of a volatility surface must have a point on each axis")]
    EmptyExportGrid,

    /// A level or term shock takes the at-the-money volatility of an expiry to zero
    /// or below.
    #[error(
        "a shift of {shift} takes the at-the-money volatility {at_the_money} of expiry {time_to_expiry} to zero or below"
    )]
    ShockedVolatilityNotPositive {
        /// The expiry, in years.
        time_to_expiry: Positive,
        /// Its at-the-money implied volatility before the shock.
        at_the_money: Positive,
        /// The shift the shock applies there, in implied volatility.
        shift: Decimal,
    },

    /// A shock gives an expiry parameters that are not a valid raw SVI slice.
    #[error("a shock gives expiry {time_to_expiry} invalid svi parameters: {source}")]
    InvalidShockedParameters {
        /// The expiry, in years.
        time_to_expiry: Positive,
        /// What the parameters break.
        source: SviError,
    },

    /// A slice could not be built or evaluated.
    #[error(transparent)]
    Svi(#[from] SviError),

    /// An option could not be priced at its fitted implied volatility.
    #[error(transparent)]
    Pricing(#[from] PricingError),

    /// A static arbitrage check could not run to its end, so there is no report: a
    /// check that was not completed must not read as one that passed.
    #[error(transparent)]
    StaticArbitrage(#[from] StaticArbitrageError),

    /// A checked `Decimal` operation failed.
    #[error(transparent)]
    Decimal(#[from] DecimalError),

    /// A value could not be represented as a `Positive`.
    #[error(transparent)]
    Positive(#[from] PositiveError),

    /// A time to expiry could not be represented.
    #[error(transparent)]
    ExpirationDate(#[from] ExpirationDateError),
}

#[cfg(test)]
mod tests_implied_volatility_surface_error {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_surface_error_no_fitted_expiry_names_both_counts() {
        let message = ImpliedVolatilitySurfaceError::NoFittedExpiry {
            unfitted: Vec::new(),
            rejections: Vec::new(),
        }
        .to_string();
        assert!(message.contains("0 unfitted"), "{message}");
        assert!(message.contains("0 rejected"), "{message}");
    }

    #[test]
    fn test_surface_error_wing_names_the_extension_and_the_limit() {
        let message = ImpliedVolatilitySurfaceError::WingOutOfPolicy {
            log_moneyness: dec!(0.7),
            time_to_expiry: Positive::ONE,
            lowest: dec!(-0.5),
            highest: dec!(0.5),
            extension: Positive::new_decimal(dec!(0.2)).unwrap(),
            limit: Positive::ZERO,
        }
        .to_string();
        assert!(message.contains("0.7"), "{message}");
        assert!(message.contains("0.2"), "{message}");
        assert!(message.contains("[-0.5, 0.5]"), "{message}");
    }

    #[test]
    fn test_surface_error_invalid_shock_names_the_expiry_and_the_cause() {
        let message = ImpliedVolatilitySurfaceError::InvalidShockedParameters {
            time_to_expiry: Positive::TWO,
            source: SviError::ZeroForward,
        }
        .to_string();
        assert!(message.contains("expiry 2"), "{message}");
        assert!(message.contains("forward"), "{message}");
    }

    #[test]
    fn test_surface_error_lower_errors_convert_with_from() {
        let decimal: ImpliedVolatilitySurfaceError =
            DecimalError::arithmetic_error("surface", "boom").into();
        assert!(matches!(decimal, ImpliedVolatilitySurfaceError::Decimal(_)));
        let svi: ImpliedVolatilitySurfaceError = SviError::ZeroForward.into();
        assert!(matches!(svi, ImpliedVolatilitySurfaceError::Svi(_)));
        let check: ImpliedVolatilitySurfaceError = StaticArbitrageError::EmptyGrid.into();
        assert!(matches!(
            check,
            ImpliedVolatilitySurfaceError::StaticArbitrage(_)
        ));
    }
}
