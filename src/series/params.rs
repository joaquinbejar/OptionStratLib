use std::fmt;
use serde::{Deserialize, Serialize};
use crate::chains::OptionChainBuildParams;
use crate::Positive;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptionSeriesBuildParams {
    /// Parameters for building option chains
    pub(crate) chain_params: OptionChainBuildParams,
    
    /// Number of options chain to build and its days to expiration
    pub(crate) series: Vec<Positive>,
}

impl OptionSeriesBuildParams {
    
    pub fn new(chain_params: OptionChainBuildParams, series: Vec<Positive>) -> Self {
        Self {
            chain_params,
            series
        }
    }
    
    pub fn set_underlying_price(&mut self, price: &Positive) {
        self.chain_params.set_underlying_price(price);
    }
    
    pub fn set_implied_volatility(&mut self, volatility: Option<Positive>) {
        self.chain_params.set_implied_volatility(volatility);   
    }
    
}

impl fmt::Display for OptionSeriesBuildParams {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let series = self.series.iter().map(|x| x.to_string()).collect::<Vec<String>>().join(", ");
        write!(f, "chain_params: {} , series: {}", self.chain_params, series)
    }
}