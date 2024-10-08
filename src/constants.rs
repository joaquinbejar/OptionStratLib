/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 11/8/24
******************************************************************************/
use plotters::style::RGBColor;

#[allow(dead_code)]
pub(crate) const SECONDS_IN_A_DAY: i64 = 86400;

#[allow(dead_code)]
pub(crate) const DAYS_IN_A_YEAR: i64 = 365;

pub(crate) const TOLERANCE: f64 = 1e-8;

pub(crate) const MIN_VOLATILITY: f64 = 1e-16;
pub(crate) const MAX_VOLATILITY: f64 = 100.0; // 10000%

pub const ZERO: f64 = 0.0;

pub(crate) const DARK_GREEN: RGBColor = RGBColor(0, 150, 0);
pub(crate) const DARK_RED: RGBColor = RGBColor(220, 0, 0);

pub(crate) const DARK_BLUE: RGBColor = RGBColor(0, 0, 150);

pub(crate) const STRIKE_PRICE_LOWER_BOUND_MULTIPLIER: f64 = 0.98;
pub(crate) const STRIKE_PRICE_UPPER_BOUND_MULTIPLIER: f64 = 1.02;
