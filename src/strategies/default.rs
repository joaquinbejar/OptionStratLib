use crate::model::Position;
use crate::strategies::base::StrategyType;
use crate::strategies::{
    BearCallSpread, BearPutSpread, BullCallSpread, BullPutSpread, CallButterfly, CustomStrategy,
    IronButterfly, IronCondor, LongButterflySpread, LongStraddle, LongStrangle,
    PoorMansCoveredCall, ShortButterflySpread, ShortStraddle, ShortStrangle,
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
        todo!()
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
impl Default for CustomStrategy {
    fn default() -> Self {
        todo!()
    }
}

// impl JsonDisplay for CoveredCall {}
// impl JsonDisplay for ProtectivePut {}
// impl JsonDisplay for Collar {}
// impl JsonDisplay for LongCall {}
// impl JsonDisplay for LongPut {}
// impl JsonDisplay for ShortCall {}
// impl JsonDisplay for ShortPut {}
