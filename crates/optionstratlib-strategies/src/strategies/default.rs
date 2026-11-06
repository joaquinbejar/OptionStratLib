use crate::strategies::base::StrategyType;
use crate::strategies::long_call::LONG_CALL_DESCRIPTION;
use crate::strategies::long_put::LONG_PUT_DESCRIPTION;
use crate::strategies::poor_mans_covered_call::PMCC_DESCRIPTION;
use crate::strategies::short_call::SHORT_CALL_DESCRIPTION;
use crate::strategies::short_put::SHORT_PUT_DESCRIPTION;
use crate::strategies::{
    BearCallSpread, BearPutSpread, BullCallLadder, BullCallSpread, BullPutSpread, IronButterfly,
    IronCondor, LongButterflySpread, LongCall, LongPut, LongStraddle, LongStrangle,
    PoorMansCoveredCall, ShortButterflySpread, ShortCall, ShortPut, ShortStraddle, ShortStrangle,
};
use chrono::{DateTime, Utc};
use optionstratlib_core::model::Position;
#[cfg(feature = "clock")]
use optionstratlib_core::utils::time::wall_clock;

impl BullCallSpread {
    /// The strategy with no legs placed yet, its legs blank and opened at `date`.
    ///
    /// What [`Default`] gives with the `clock` feature, without reading a clock.
    #[must_use]
    pub fn blank_at(date: DateTime<Utc>) -> Self {
        BullCallSpread {
            name: "Bull Call Spread".to_string(),
            kind: StrategyType::BullCallSpread,
            description: crate::strategies::bull_call_spread::BULL_CALL_SPREAD_DESCRIPTION
                .to_string(),
            break_even_points: Vec::new(),
            long_call: Position::blank_at(date),
            short_call: Position::blank_at(date),
        }
    }
}
#[cfg(feature = "clock")]
impl Default for BullCallSpread {
    fn default() -> Self {
        Self::blank_at(wall_clock())
    }
}
impl BearCallSpread {
    /// The strategy with no legs placed yet, its legs blank and opened at `date`.
    ///
    /// What [`Default`] gives with the `clock` feature, without reading a clock.
    #[must_use]
    pub fn blank_at(date: DateTime<Utc>) -> Self {
        BearCallSpread {
            name: "Bear Call Spread".to_string(),
            kind: StrategyType::BearCallSpread,
            description: crate::strategies::bear_call_spread::BEAR_CALL_SPREAD_DESCRIPTION
                .to_string(),
            break_even_points: Vec::new(),
            short_call: Position::blank_at(date),
            long_call: Position::blank_at(date),
        }
    }
}
#[cfg(feature = "clock")]
impl Default for BearCallSpread {
    fn default() -> Self {
        Self::blank_at(wall_clock())
    }
}
impl BullPutSpread {
    /// The strategy with no legs placed yet, its legs blank and opened at `date`.
    ///
    /// What [`Default`] gives with the `clock` feature, without reading a clock.
    #[must_use]
    pub fn blank_at(date: DateTime<Utc>) -> Self {
        BullPutSpread {
            name: "Bull Put Spread".to_string(),
            kind: StrategyType::BullPutSpread,
            description: crate::strategies::bull_put_spread::BULL_PUT_SPREAD_DESCRIPTION
                .to_string(),
            break_even_points: Vec::new(),
            long_put: Position::blank_at(date),
            short_put: Position::blank_at(date),
        }
    }
}
#[cfg(feature = "clock")]
impl Default for BullPutSpread {
    fn default() -> Self {
        Self::blank_at(wall_clock())
    }
}
impl BearPutSpread {
    /// The strategy with no legs placed yet, its legs blank and opened at `date`.
    ///
    /// What [`Default`] gives with the `clock` feature, without reading a clock.
    #[must_use]
    pub fn blank_at(date: DateTime<Utc>) -> Self {
        BearPutSpread {
            name: "Bear Put Spread".to_string(),
            kind: StrategyType::BearPutSpread,
            description: crate::strategies::bear_put_spread::BEAR_PUT_SPREAD_DESCRIPTION
                .to_string(),
            break_even_points: Vec::new(),
            long_put: Position::blank_at(date),
            short_put: Position::blank_at(date),
        }
    }
}
#[cfg(feature = "clock")]
impl Default for BearPutSpread {
    fn default() -> Self {
        Self::blank_at(wall_clock())
    }
}
impl LongButterflySpread {
    /// The strategy with no legs placed yet, its legs blank and opened at `date`.
    ///
    /// What [`Default`] gives with the `clock` feature, without reading a clock.
    #[must_use]
    pub fn blank_at(date: DateTime<Utc>) -> Self {
        LongButterflySpread {
            name: "Long Butterfly Spread".to_string(),
            kind: StrategyType::LongButterflySpread,
            description: crate::strategies::long_butterfly_spread::LONG_BUTTERFLY_DESCRIPTION
                .to_string(),
            break_even_points: Vec::new(),
            short_call: Position::blank_at(date),
            long_call_low: Position::blank_at(date),
            long_call_high: Position::blank_at(date),
        }
    }
}
#[cfg(feature = "clock")]
impl Default for LongButterflySpread {
    fn default() -> Self {
        Self::blank_at(wall_clock())
    }
}
impl ShortButterflySpread {
    /// The strategy with no legs placed yet, its legs blank and opened at `date`.
    ///
    /// What [`Default`] gives with the `clock` feature, without reading a clock.
    #[must_use]
    pub fn blank_at(date: DateTime<Utc>) -> Self {
        ShortButterflySpread {
            name: "Short Butterfly Spread".to_string(),
            kind: StrategyType::ShortButterflySpread,
            description: crate::strategies::short_butterfly_spread::SHORT_BUTTERFLY_DESCRIPTION
                .to_string(),
            break_even_points: Vec::new(),
            long_call: Position::blank_at(date),
            short_call_low: Position::blank_at(date),
            short_call_high: Position::blank_at(date),
        }
    }
}
#[cfg(feature = "clock")]
impl Default for ShortButterflySpread {
    fn default() -> Self {
        Self::blank_at(wall_clock())
    }
}
impl IronCondor {
    /// The strategy with no legs placed yet, its legs blank and opened at `date`.
    ///
    /// What [`Default`] gives with the `clock` feature, without reading a clock.
    #[must_use]
    pub fn blank_at(date: DateTime<Utc>) -> Self {
        IronCondor {
            name: "Iron Condor".to_string(),
            kind: StrategyType::IronCondor,
            description: crate::strategies::iron_condor::IRON_CONDOR_DESCRIPTION.to_string(),
            break_even_points: Vec::new(),
            short_call: Position::blank_at(date),
            short_put: Position::blank_at(date),
            long_call: Position::blank_at(date),
            long_put: Position::blank_at(date),
        }
    }
}
#[cfg(feature = "clock")]
impl Default for IronCondor {
    fn default() -> Self {
        Self::blank_at(wall_clock())
    }
}
impl IronButterfly {
    /// The strategy with no legs placed yet, its legs blank and opened at `date`.
    ///
    /// What [`Default`] gives with the `clock` feature, without reading a clock.
    #[must_use]
    pub fn blank_at(date: DateTime<Utc>) -> Self {
        IronButterfly {
            name: "Iron Butterfly".to_string(),
            kind: StrategyType::IronButterfly,
            description: crate::strategies::iron_butterfly::IRON_BUTTERFLY_DESCRIPTION.to_string(),
            break_even_points: Vec::new(),
            short_call: Position::blank_at(date),
            short_put: Position::blank_at(date),
            long_call: Position::blank_at(date),
            long_put: Position::blank_at(date),
        }
    }
}
#[cfg(feature = "clock")]
impl Default for IronButterfly {
    fn default() -> Self {
        Self::blank_at(wall_clock())
    }
}
impl LongStraddle {
    /// The strategy with no legs placed yet, its legs blank and opened at `date`.
    ///
    /// What [`Default`] gives with the `clock` feature, without reading a clock.
    #[must_use]
    pub fn blank_at(date: DateTime<Utc>) -> Self {
        LongStraddle {
            name: "Long Straddle".to_string(),
            kind: StrategyType::LongStraddle,
            description: crate::strategies::long_straddle::LONG_STRADDLE_DESCRIPTION.to_string(),
            break_even_points: Vec::new(),
            long_call: Position::blank_at(date),
            long_put: Position::blank_at(date),
        }
    }
}
#[cfg(feature = "clock")]
impl Default for LongStraddle {
    fn default() -> Self {
        Self::blank_at(wall_clock())
    }
}
impl ShortStraddle {
    /// The strategy with no legs placed yet, its legs blank and opened at `date`.
    ///
    /// What [`Default`] gives with the `clock` feature, without reading a clock.
    #[must_use]
    pub fn blank_at(date: DateTime<Utc>) -> Self {
        ShortStraddle {
            name: "Short Straddle".to_string(),
            kind: StrategyType::ShortStraddle,
            description: crate::strategies::short_straddle::SHORT_STRADDLE_DESCRIPTION.to_string(),
            break_even_points: Vec::new(),
            short_call: Position::blank_at(date),
            short_put: Position::blank_at(date),
        }
    }
}
#[cfg(feature = "clock")]
impl Default for ShortStraddle {
    fn default() -> Self {
        Self::blank_at(wall_clock())
    }
}
impl LongStrangle {
    /// The strategy with no legs placed yet, its legs blank and opened at `date`.
    ///
    /// What [`Default`] gives with the `clock` feature, without reading a clock.
    #[must_use]
    pub fn blank_at(date: DateTime<Utc>) -> Self {
        LongStrangle {
            name: "Long Strangle".to_string(),
            kind: StrategyType::LongStrangle,
            description: crate::strategies::long_strangle::LONG_STRANGLE_DESCRIPTION.to_string(),
            break_even_points: Vec::new(),
            long_call: Position::blank_at(date),
            long_put: Position::blank_at(date),
        }
    }
}
#[cfg(feature = "clock")]
impl Default for LongStrangle {
    fn default() -> Self {
        Self::blank_at(wall_clock())
    }
}
impl ShortStrangle {
    /// The strategy with no legs placed yet, its legs blank and opened at `date`.
    ///
    /// What [`Default`] gives with the `clock` feature, without reading a clock.
    #[must_use]
    pub fn blank_at(date: DateTime<Utc>) -> Self {
        ShortStrangle {
            name: "Short Strangle".to_string(),
            kind: StrategyType::ShortStrangle,
            description: crate::strategies::short_strangle::SHORT_STRANGLE_DESCRIPTION.to_string(),
            break_even_points: Vec::new(),
            short_call: Position::blank_at(date),
            short_put: Position::blank_at(date),
        }
    }
}
#[cfg(feature = "clock")]
impl Default for ShortStrangle {
    fn default() -> Self {
        Self::blank_at(wall_clock())
    }
}
impl PoorMansCoveredCall {
    /// The strategy with no legs placed yet, its legs blank and opened at `date`.
    ///
    /// What [`Default`] gives with the `clock` feature, without reading a clock.
    #[must_use]
    pub fn blank_at(date: DateTime<Utc>) -> Self {
        PoorMansCoveredCall {
            name: "Poor Man's Covered Call".to_string(),
            kind: StrategyType::PoorMansCoveredCall,
            description: PMCC_DESCRIPTION.to_string(),
            break_even_points: Vec::new(),
            long_call: Position::blank_at(date),
            short_call: Position::blank_at(date),
        }
    }
}
#[cfg(feature = "clock")]
impl Default for PoorMansCoveredCall {
    fn default() -> Self {
        Self::blank_at(wall_clock())
    }
}
impl BullCallLadder {
    /// The strategy with no legs placed yet, its legs blank and opened at `date`.
    ///
    /// What [`Default`] gives with the `clock` feature, without reading a clock.
    #[must_use]
    pub fn blank_at(date: DateTime<Utc>) -> Self {
        BullCallLadder {
            name: "Bull Call Ladder".to_string(),
            kind: StrategyType::BullCallLadder,
            description: crate::strategies::bull_call_ladder::BULL_CALL_LADDER_DESCRIPTION
                .to_string(),
            break_even_points: Vec::new(),
            short_call_low: Position::blank_at(date),
            long_call: Position::blank_at(date),
            short_call_high: Position::blank_at(date),
        }
    }
}
#[cfg(feature = "clock")]
impl Default for BullCallLadder {
    fn default() -> Self {
        Self::blank_at(wall_clock())
    }
}
impl LongCall {
    /// The strategy with no legs placed yet, its legs blank and opened at `date`.
    ///
    /// What [`Default`] gives with the `clock` feature, without reading a clock.
    #[must_use]
    pub fn blank_at(date: DateTime<Utc>) -> Self {
        LongCall {
            name: "Long Call".to_string(),
            kind: StrategyType::LongCall,
            description: LONG_CALL_DESCRIPTION.to_string(),
            break_even_points: Vec::new(),
            long_call: Position::blank_at(date),
        }
    }
}
#[cfg(feature = "clock")]
impl Default for LongCall {
    fn default() -> Self {
        Self::blank_at(wall_clock())
    }
}
impl LongPut {
    /// The strategy with no legs placed yet, its legs blank and opened at `date`.
    ///
    /// What [`Default`] gives with the `clock` feature, without reading a clock.
    #[must_use]
    pub fn blank_at(date: DateTime<Utc>) -> Self {
        LongPut {
            name: "Long Put".to_string(),
            kind: StrategyType::LongPut,
            description: LONG_PUT_DESCRIPTION.to_string(),
            break_even_points: Vec::new(),
            long_put: Position::blank_at(date),
        }
    }
}
#[cfg(feature = "clock")]
impl Default for LongPut {
    fn default() -> Self {
        Self::blank_at(wall_clock())
    }
}
impl ShortCall {
    /// The strategy with no legs placed yet, its legs blank and opened at `date`.
    ///
    /// What [`Default`] gives with the `clock` feature, without reading a clock.
    #[must_use]
    pub fn blank_at(date: DateTime<Utc>) -> Self {
        ShortCall {
            name: "Short Call".to_string(),
            kind: StrategyType::ShortCall,
            description: SHORT_CALL_DESCRIPTION.to_string(),
            break_even_points: Vec::new(),
            short_call: Position::blank_at(date),
        }
    }
}
#[cfg(feature = "clock")]
impl Default for ShortCall {
    fn default() -> Self {
        Self::blank_at(wall_clock())
    }
}
impl ShortPut {
    /// The strategy with no legs placed yet, its legs blank and opened at `date`.
    ///
    /// What [`Default`] gives with the `clock` feature, without reading a clock.
    #[must_use]
    pub fn blank_at(date: DateTime<Utc>) -> Self {
        ShortPut {
            name: "Short Put".to_string(),
            kind: StrategyType::ShortPut,
            description: SHORT_PUT_DESCRIPTION.to_string(),
            break_even_points: Vec::new(),
            short_put: Position::blank_at(date),
        }
    }
}
#[cfg(feature = "clock")]
impl Default for ShortPut {
    fn default() -> Self {
        Self::blank_at(wall_clock())
    }
}
