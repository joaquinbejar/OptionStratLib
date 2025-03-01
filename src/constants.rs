/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 11/8/24
******************************************************************************/
use crate::Positive;
use plotters::style::RGBColor;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

pub const PI: Decimal = dec!(3.1415926535897932384626433832);
pub const ZERO: f64 = 0.0;
#[allow(dead_code)]
pub(crate) const SECONDS_IN_A_DAY: i64 = 86400;
pub(crate) const DAYS_IN_A_YEAR: Positive = Positive(dec!(365.0));
pub(crate) const TOLERANCE: Decimal = dec!(1e-8);
pub const EPSILON: Decimal = dec!(1e-16);
pub(crate) const MIN_VOLATILITY: Positive = Positive(dec!(1e-16));
pub(crate) const MAX_VOLATILITY: Positive = Positive::HUNDRED;
pub(crate) const DARK_GREEN: RGBColor = RGBColor(0, 150, 0);
pub(crate) const DARK_RED: RGBColor = RGBColor(220, 0, 0);
pub(crate) const DARK_BLUE: RGBColor = RGBColor(0, 0, 150);
pub(crate) const STRIKE_PRICE_LOWER_BOUND_MULTIPLIER: f64 = 0.98;
pub(crate) const STRIKE_PRICE_UPPER_BOUND_MULTIPLIER: f64 = 1.02;
pub(crate) const TRADING_DAYS: Positive = Positive(dec!(252.0));
pub(crate) const TRADING_HOURS: Positive = Positive(dec!(6.5));
pub(crate) const SECONDS_PER_HOUR: Positive = Positive(dec!(3600.0));
pub(crate) const MINUTES_PER_HOUR: Positive = Positive(dec!(60.0));
pub(crate) const MILLISECONDS_PER_SECOND: Positive = Positive(dec!(1000.0));
pub(crate) const MICROSECONDS_PER_SECOND: Positive = Positive(dec!(1_000_000.0));
pub(crate) const WEEKS_PER_YEAR: Positive = Positive(dec!(52.0));
pub(crate) const MONTHS_PER_YEAR: Positive = Positive(dec!(12.0));
pub(crate) const QUARTERS_PER_YEAR: Positive = Positive(dec!(4.0));
pub(crate) const MAX_ITERATIONS_IV: u32 = 1000;
pub(crate) const IV_TOLERANCE: Decimal = dec!(1e-5);
