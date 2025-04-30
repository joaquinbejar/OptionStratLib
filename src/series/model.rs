use crate::chains::{OptionChain, OptionChainBuildParams};
use crate::series::params::OptionSeriesBuildParams;
use crate::{ExpirationDate, Positive};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
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

    pub fn build_series(params: &OptionSeriesBuildParams) -> Self {
        todo!()
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
