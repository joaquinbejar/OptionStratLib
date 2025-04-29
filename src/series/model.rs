use crate::chains::OptionChain;
use crate::{ExpirationDate, Positive};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

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
