/******************************************************************************
    Author: Joaquín Béjar García
    Email: jb@taunais.com 
    Date: 8/2/25
 ******************************************************************************/

use std::sync::Arc;
use crate::{OptionStyle, OptionType, Options, Positive, Side};
use crate::chains::chain::OptionData;
use crate::chains::utils::OptionDataPriceParams;
use crate::error::ChainError;

#[derive(Debug, Clone)]
pub struct FourOptions {
    pub long_call: Arc<Options>,
    pub short_call: Arc<Options>,
    pub long_put: Arc<Options>,
    pub short_put: Arc<Options>,
}


impl PartialEq for FourOptions {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.long_call, &other.long_call)
            && Arc::ptr_eq(&self.short_call, &other.short_call)
            && Arc::ptr_eq(&self.long_put, &other.long_put)
            && Arc::ptr_eq(&self.short_put, &other.short_put)
    }
}

impl OptionData {
    pub fn create_options(&mut self, price_params: &OptionDataPriceParams) -> Result<(), ChainError> {
        let symbol = if let Some(underlying_symbol) = price_params.underlying_symbol.clone() {
            underlying_symbol
        } else { 
          "NA".to_string()
        };
        let long_call = Arc::new(Options::new(
            OptionType::European,
            Side::Long,
            symbol.clone(),
            self.strike_price,
            price_params.expiration_date.clone(),
            self.implied_volatility.unwrap_or(Positive::ZERO),
            Positive::ONE,
            price_params.underlying_price,
            price_params.risk_free_rate,
            OptionStyle::Call,
            price_params.dividend_yield,
            None,
        ));

        let short_call = Arc::new(Options::new(
            OptionType::European,
            Side::Short,
            symbol.clone(),
            self.strike_price,
            price_params.expiration_date.clone(),
            self.implied_volatility.unwrap_or(Positive::ZERO),
            Positive::ONE,
            price_params.underlying_price,
            price_params.risk_free_rate,
            OptionStyle::Call,
            price_params.dividend_yield,
            None,
        ));

        let long_put = Arc::new(Options::new(
            OptionType::European,
            Side::Long,
            symbol.clone(),
            self.strike_price,
            price_params.expiration_date.clone(),
            self.implied_volatility.unwrap_or(Positive::ZERO),
            Positive::ONE,
            price_params.underlying_price,
            price_params.risk_free_rate,
            OptionStyle::Put,
            price_params.dividend_yield,
            None,
        ));

        let short_put = Arc::new(Options::new(
            OptionType::European,
            Side::Short,
            symbol.clone(),
            self.strike_price,
            price_params.expiration_date.clone(),
            self.implied_volatility.unwrap_or(Positive::ZERO),
            Positive::ONE,
            price_params.underlying_price,
            price_params.risk_free_rate,
            OptionStyle::Put,
            price_params.dividend_yield,
            None,
        ));

        self.options = Some(Box::new(FourOptions {
            long_call,
            short_call,
            long_put,
            short_put,
        }));

        Ok(())
    }
}