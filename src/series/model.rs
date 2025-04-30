use crate::chains::{OptionChain, OptionChainBuildParams};
use crate::series::params::OptionSeriesBuildParams;
use crate::{ExpirationDate, Positive};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptionSeries {
    /// The ticker symbol for the underlying asset (e.g., "AAPL", "SPY").
    pub symbol: String,

    /// The current market price of the underlying asset.
    pub underlying_price: Positive,

    /// A sorted collection of option chains, each corresponding to a different expiration date.
    pub chains: BTreeMap<ExpirationDate, OptionChain>,

    /// The risk-free interest rate used for option pricing models.
    pub risk_free_rate: Option<Decimal>,

    /// The annual dividend yield of the underlying asset.
    pub dividend_yield: Option<Positive>,
}

impl OptionSeries {
    pub fn new(symbol: String, underlying_price: Positive) -> Self {
        Self {
            symbol,
            underlying_price,
            chains: BTreeMap::new(),
            risk_free_rate: None,
            dividend_yield: None,
        }
    }

    pub fn odte(&self) -> Option<OptionChain> {
        match self.chains.first_key_value() {
            Some((expiration_date, option_chain)) => {
                if expiration_date <= &ExpirationDate::Days(Positive::ONE) {
                    Some(option_chain.clone())
                } else {
                    None
                }
            }
            None => None,
        }
    }

    pub fn get_expiration_dates(&self) -> Result<Vec<Positive>, Box<dyn Error>> {
        let keys: Result<Vec<Positive>, Box<dyn Error>> = self.chains
            .iter()
            .map(|(e, _)| e.get_days().map_err(|err| err.into()))
            .collect();

        keys
    }

    pub fn build_series(_params: &OptionSeriesBuildParams) -> Self {
        // Self {
        //     symbol: "".to_string(),
        //     underlying_price: Default::default(),
        //     chains: Default::default(),
        //     risk_free_rate: None,
        //     dividend_yield: None,
        // }
        todo!()
    }

    pub fn to_build_params(&self) -> Result<OptionSeriesBuildParams, Box<dyn Error>> {
        let chain_params = self.chains.first_key_value();
        let mut series = vec![];
        let chain_params = match chain_params { 
            Some((expiration_date, option_chain)) => {
                series.push(expiration_date.get_days()?);
                option_chain.to_build_params()?
            },
            None => {
              return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "No chains found")));  
            },
        };
        
        Ok(OptionSeriesBuildParams {
            chain_params,
            series,
        })
    }
    
}

impl Default for OptionSeries {
    fn default() -> Self {
        Self::new("".to_string(), Positive::ZERO)
    }
}

impl fmt::Display for OptionSeries {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let chains: String = self
            .chains
            .iter()
            .map(|(e, o)| format!("{}:\n{}", e, o))
            .collect();

        let risk_free_rate = match &self.risk_free_rate {
            Some(r) => format!(" risk_free_rate: {}", r),
            None => "".to_string(),
        };
        let dividend_yield = match &self.dividend_yield {
            Some(d) => format!(" dividend_yield: {}", d),
            None => "".to_string(),
        };

        write!(
            f,
            "OptionSeries {{ symbol: {}, underlying_price: {}{}{}\n{} }}",
            self.symbol, self.underlying_price, risk_free_rate, dividend_yield, chains
        )
    }
}
