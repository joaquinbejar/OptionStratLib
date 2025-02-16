/******************************************************************************
    Author: Joaquín Béjar García
    Email: jb@taunais.com 
    Date: 16/2/25
 ******************************************************************************/
use serde::{Deserialize, Serialize};
use crate::{Options, Positive};
use crate::error::StrategyError;
use crate::strategies::base::StrategyType;
use crate::strategies::{BearCallSpread, BearPutSpread, BullCallSpread, BullPutSpread, CallButterfly, CustomStrategy, IronButterfly, IronCondor, LongButterflySpread, LongStraddle, LongStrangle, PoorMansCoveredCall, ShortButterflySpread, ShortStraddle, ShortStrangle, Strategies};


#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct OptionWithCosts {
    pub open_fee: Positive,
    pub close_fee: Positive,
    pub premium: Positive,
    pub option: Options,
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct StrategyRequest {
    pub strategy_type: StrategyType,
    pub options: Vec<OptionWithCosts>,
}

pub trait StrategyConstructor: Strategies {
    fn get_strategy( _vec_options: &Vec<OptionWithCosts>) -> Result<Self, StrategyError> {
        Err(StrategyError::NotImplemented)
    }
}

impl StrategyRequest {
    pub fn new(strategy_type: StrategyType, options: Vec<OptionWithCosts>) -> Self {
        Self {
            strategy_type,
            options,
        }
    }
    
    pub fn get_strategy(&self) -> Result<Box<dyn Strategies>, StrategyError> {
        match self.strategy_type {
            StrategyType::BullCallSpread => Ok(Box::new(BullCallSpread::get_strategy(&self.options))),
            StrategyType::BearCallSpread => Ok(Box::new(BearCallSpread::get_strategy(&self.options))),
            StrategyType::BullPutSpread => Ok(Box::new(BullPutSpread::get_strategy(&self.options))),
            StrategyType::BearPutSpread => Ok(Box::new(BearPutSpread::get_strategy(&self.options))),
            StrategyType::LongButterflySpread => Ok(Box::new(LongButterflySpread::get_strategy(&self.options))),
            StrategyType::ShortButterflySpread => Ok(Box::new(ShortButterflySpread::get_strategy(&self.options))),
            StrategyType::IronCondor => Ok(Box::new(IronCondor::get_strategy(&self.options))),
            StrategyType::IronButterfly => Ok(Box::new(IronButterfly::get_strategy(&self.options))),
            StrategyType::LongStraddle => Ok(Box::new(LongStraddle::get_strategy(&self.options))),
            StrategyType::ShortStraddle => Ok(Box::new(ShortStraddle::get_strategy(&self.options))),
            StrategyType::LongStrangle => Ok(Box::new(LongStrangle::get_strategy(&self.options))),
            StrategyType::ShortStrangle => Ok(Box::new(ShortStrangle::get_strategy(&self.options))),
            StrategyType::CoveredCall => Err(StrategyError::NotImplemented),
            StrategyType::ProtectivePut => Err(StrategyError::NotImplemented),
            StrategyType::Collar => Err(StrategyError::NotImplemented),
            StrategyType::LongCall => Err(StrategyError::NotImplemented),
            StrategyType::LongPut => Err(StrategyError::NotImplemented),
            StrategyType::ShortCall =>Err(StrategyError::NotImplemented),
            StrategyType::ShortPut => Err(StrategyError::NotImplemented),
            StrategyType::PoorMansCoveredCall => Ok(Box::new(PoorMansCoveredCall::get_strategy(&self.options))),
            StrategyType::CallButterfly => Ok(Box::new(CallButterfly::get_strategy(&self.options))),
            StrategyType::Custom => Ok(Box::new(CustomStrategy::get_strategy(&self.options))),
        }
    }
}