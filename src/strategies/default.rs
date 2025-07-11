use crate::model::Position;
use crate::strategies::base::StrategyType;
use crate::strategies::long_call::LONG_CALL_DESCRIPTION;
use crate::strategies::long_put::LONG_PUT_DESCRIPTION;
use crate::strategies::poor_mans_covered_call::PMCC_DESCRIPTION;
use crate::strategies::short_call::SHORT_CALL_DESCRIPTION;
use crate::strategies::short_put::SHORT_PUT_DESCRIPTION;
use crate::strategies::{
    BearCallSpread, BearPutSpread, BullCallSpread, BullPutSpread, CallButterfly, IronButterfly,
    IronCondor, LongButterflySpread, LongCall, LongPut, LongStraddle, LongStrangle,
    PoorMansCoveredCall, ShortButterflySpread, ShortCall, ShortPut, ShortStraddle, ShortStrangle,
};

impl Default for BullCallSpread {
    fn default() -> Self {
        BullCallSpread {
            name: "Bull Call Spread".to_string(),
            kind: StrategyType::BullCallSpread,
            description: crate::strategies::bull_call_spread::BULL_CALL_SPREAD_DESCRIPTION.to_string(),
            break_even_points: Vec::new(),
            long_call: Position::default(),
            short_call: Position::default(),
        }
    }
}
impl Default for BearCallSpread {
    fn default() -> Self {
        BearCallSpread {
            name: "Bear Call Spread".to_string(),
            kind: StrategyType::BearCallSpread,
            description: crate::strategies::bear_call_spread::BEAR_CALL_SPREAD_DESCRIPTION.to_string(),
            break_even_points: Vec::new(),
            short_call: Position::default(),
            long_call: Position::default(),
        }
    }
}
impl Default for BullPutSpread {
    fn default() -> Self {
        BullPutSpread {
            name: "Bull Put Spread".to_string(),
            kind: StrategyType::BullPutSpread,
            description: crate::strategies::bull_put_spread::BULL_PUT_SPREAD_DESCRIPTION.to_string(),
            break_even_points: Vec::new(),
            long_put: Position::default(),
            short_put: Position::default(),
        }
    }
}
impl Default for BearPutSpread {
    fn default() -> Self {
        BearPutSpread {
            name: "Bear Put Spread".to_string(),
            kind: StrategyType::BearPutSpread,
            description: crate::strategies::bear_put_spread::BEAR_PUT_SPREAD_DESCRIPTION.to_string(),
            break_even_points: Vec::new(),
            long_put: Position::default(),
            short_put: Position::default(),
        }
    }
}
impl Default for LongButterflySpread {
    fn default() -> Self {
        LongButterflySpread {
            name: "Long Butterfly Spread".to_string(),
            kind: StrategyType::LongButterflySpread,
            description: crate::strategies::long_butterfly_spread::LONG_BUTTERFLY_DESCRIPTION.to_string(),
            break_even_points: Vec::new(),
            short_call: Position::default(),
            long_call_low: Position::default(),
            long_call_high: Position::default(),
        }
    }
}
impl Default for ShortButterflySpread {
    fn default() -> Self {
        ShortButterflySpread {
            name: "Short Butterfly Spread".to_string(),
            kind: StrategyType::ShortButterflySpread,
            description: crate::strategies::short_butterfly_spread::SHORT_BUTTERFLY_DESCRIPTION.to_string(),
            break_even_points: Vec::new(),
            long_call: Position::default(),
            short_call_low: Position::default(),
            short_call_high: Position::default(),
        }
    }
}
impl Default for IronCondor {
    fn default() -> Self {
        IronCondor {
            name: "Iron Condor".to_string(),
            kind: StrategyType::IronCondor,
            description: crate::strategies::iron_condor::IRON_CONDOR_DESCRIPTION.to_string(),
            break_even_points: Vec::new(),
            short_call: Position::default(),
            short_put: Position::default(),
            long_call: Position::default(),
            long_put: Position::default(),
        }
    }
}
impl Default for IronButterfly {
    fn default() -> Self {
        IronButterfly {
            name: "Iron Butterfly".to_string(),
            kind: StrategyType::IronButterfly,
            description: crate::strategies::iron_butterfly::IRON_BUTTERFLY_DESCRIPTION.to_string(),
            break_even_points: Vec::new(),
            short_call: Position::default(),
            short_put: Position::default(),
            long_call: Position::default(),
            long_put: Position::default(),
        }
    }
}
impl Default for LongStraddle {
    fn default() -> Self {
        LongStraddle {
            name: "Long Straddle".to_string(),
            kind: StrategyType::LongStraddle,
            description: crate::strategies::long_straddle::LONG_STRADDLE_DESCRIPTION.to_string(),
            break_even_points: Vec::new(),
            long_call: Position::default(),
            long_put: Position::default(),
        }
    }
}
impl Default for ShortStraddle {
    fn default() -> Self {
        ShortStraddle {
            name: "Short Straddle".to_string(),
            kind: StrategyType::ShortStraddle,
            description: crate::strategies::short_straddle::SHORT_STRADDLE_DESCRIPTION.to_string(),
            break_even_points: Vec::new(),
            short_call: Position::default(),
            short_put: Position::default(),
        }
    }
}
impl Default for LongStrangle {
    fn default() -> Self {
        LongStrangle {
            name: "Long Strangle".to_string(),
            kind: StrategyType::LongStrangle,
            description: crate::strategies::long_strangle::LONG_STRANGLE_DESCRIPTION.to_string(),
            break_even_points: Vec::new(),
            long_call: Position::default(),
            long_put: Position::default(),
        }
    }
}
impl Default for ShortStrangle {
    fn default() -> Self {
        ShortStrangle {
            name: "Short Strangle".to_string(),
            kind: StrategyType::ShortStrangle,
            description: crate::strategies::short_strangle::SHORT_STRANGLE_DESCRIPTION.to_string(),
            break_even_points: Vec::new(),
            short_call: Position::default(),
            short_put: Position::default(),
        }
    }
}
impl Default for PoorMansCoveredCall {
    fn default() -> Self {
        PoorMansCoveredCall {
            name: "Poor Man's Covered Call".to_string(),
            kind: StrategyType::PoorMansCoveredCall,
            description: PMCC_DESCRIPTION.to_string(),
            break_even_points: Vec::new(),
            long_call: Position::default(),
            short_call: Position::default(),
        }
    }
}
impl Default for CallButterfly {
    fn default() -> Self {
    CallButterfly {
            name: "Call Butterfly".to_string(),
            kind: StrategyType::CallButterfly,
            description: crate::strategies::call_butterfly::CALL_BUTTERFLY_DESCRIPTION.to_string(),
            break_even_points: Vec::new(),
            short_call_low: Position::default(),
            long_call: Position::default(),
            short_call_high: Position::default(),
        }
    }
}
impl Default for LongCall {
    fn default() -> Self {
        LongCall {
            name: "Long Call".to_string(),
            kind: StrategyType::LongCall,
            description: LONG_CALL_DESCRIPTION.to_string(),
            break_even_points: Vec::new(),
            long_call: Position::default(),
        }
    }
}
impl Default for LongPut {
    fn default() -> Self {
        LongPut {
            name: "Long Put".to_string(),
            kind: StrategyType::LongPut,
            description: LONG_PUT_DESCRIPTION.to_string(),
            break_even_points: Vec::new(),
            long_put: Position::default(),
        }
    }
}
impl Default for ShortCall {
    fn default() -> Self {
        ShortCall {
            name: "Short Call".to_string(),
            kind: StrategyType::ShortCall,
            description: SHORT_CALL_DESCRIPTION.to_string(),
            break_even_points: Vec::new(),
            short_call: Position::default(),
        }
    }
}
impl Default for ShortPut {
    fn default() -> Self {
        ShortPut {
            name: "Short Put".to_string(),
            kind: StrategyType::ShortPut,
            description: SHORT_PUT_DESCRIPTION.to_string(),
            break_even_points: Vec::new(),
            short_put: Position::default(),
        }
    }
}
