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

pub(crate) const ZERO: f64 = 0.0;
pub(crate) const INFINITY_POSITIVE: f64 = f64::INFINITY;
pub(crate) const INFINITY_NEGATIVE: f64 = f64::NEG_INFINITY;

pub(crate) const DARK_GREEN: RGBColor = RGBColor(0, 150, 0);
pub(crate) const DARK_RED: RGBColor = RGBColor(220, 0, 0);
