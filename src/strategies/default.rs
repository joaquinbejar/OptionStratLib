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
        todo!()
    }
}
impl Default for BearCallSpread {
    fn default() -> Self {
        todo!()
    }
}
impl Default for BullPutSpread {
    fn default() -> Self {
        todo!()
    }
}
impl Default for BearPutSpread {
    fn default() -> Self {
        todo!()
    }
}
impl Default for LongButterflySpread {
    fn default() -> Self {
        todo!()
    }
}
impl Default for ShortButterflySpread {
    fn default() -> Self {
        todo!()
    }
}
impl Default for IronCondor {
    fn default() -> Self {
        todo!()
    }
}
impl Default for IronButterfly {
    fn default() -> Self {
        todo!()
    }
}
impl Default for LongStraddle {
    fn default() -> Self {
        todo!()
    }
}
impl Default for ShortStraddle {
    fn default() -> Self {
        todo!()
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
        CallButterfly::new(
            "".to_string(),
            Default::default(),
            Default::default(),
            Default::default(),
            Default::default(),
            Default::default(),
            Default::default(),
            Default::default(),
            Default::default(),
            Default::default(),
            Default::default(),
            Default::default(),
            Default::default(),
            Default::default(),
            Default::default(),
            Default::default(),
            Default::default(),
            Default::default(),
            Default::default(),
        )
    }
}

// impl JsonDisplay for CoveredCall {}
// impl JsonDisplay for ProtectivePut {}
// impl JsonDisplay for Collar {}
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
