/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 16/2/25
******************************************************************************/
use crate::error::StrategyError;
use crate::strategies::build::model::OptionWithCosts;
use crate::strategies::Strategies;

pub trait StrategyConstructor: Strategies {
    fn get_strategy(_vec_options: &Vec<OptionWithCosts>) -> Result<Self, StrategyError>
    where
        Self: Sized,
    {
        Err(StrategyError::NotImplemented)
    }
}
