/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 23/10/24
******************************************************************************/

/// Represents different timeframes for volatility calculations
#[derive(Debug, Clone, Copy)]
pub enum TimeFrame {
    Microsecond, // 1-microsecond data
    Millisecond, // 1-millisecond data
    Second,      // 1-second data
    Minute,      // 1-minute data
    Hour,        // 1-hour data
    Day,         // Daily data
    Week,        // Weekly data
    Month,       // Monthly data
    Quarter,     // Quarterly data
    Year,        // Yearly data
    Custom(f64), // Custom periods per year
}

const TRADING_DAYS: f64 = 252.0;
impl TimeFrame {
    /// Returns the number of periods in a year for this timeframe
    pub(crate) fn periods_per_year(&self) -> f64 {
        match self {
            TimeFrame::Microsecond => TRADING_DAYS * 6.5 * 60.0 * 60.0 * 1_000_000.0, // Microseconds in trading year
            TimeFrame::Millisecond => TRADING_DAYS * 6.5 * 60.0 * 60.0 * 1_000.0, // Milliseconds in trading year
            TimeFrame::Second => TRADING_DAYS * 6.5 * 60.0 * 60.0, // Seconds in trading year
            TimeFrame::Minute => TRADING_DAYS * 6.5 * 60.0,        // Minutes in trading year
            TimeFrame::Hour => TRADING_DAYS * 6.5,                 // Hours in trading year
            TimeFrame::Day => TRADING_DAYS,                        // Trading days in a year
            TimeFrame::Week => 52.0,                               // Weeks in a year
            TimeFrame::Month => 12.0,                              // Months in a year
            TimeFrame::Quarter => 4.0,                             // Quarters in a year
            TimeFrame::Year => 1.0,                                // Base unit
            TimeFrame::Custom(periods) => *periods,                // Custom periods per year
        }
    }
}
