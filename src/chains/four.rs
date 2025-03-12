/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 8/2/25
******************************************************************************/

use crate::chains::chain::OptionData;
use crate::chains::utils::OptionDataPriceParams;
use crate::error::ChainError;
use crate::{OptionStyle, OptionType, Options, Positive, Side};
use std::sync::Arc;

/// Represents a combination of four option positions that form a complete option strategy.
///
/// This struct encapsulates a set of four option contracts that together can create various
/// option strategies such as iron condors, iron butterflies, straddles, strangles, or custom
/// four-legged option combinations.
///
/// Each component is stored as an `Arc<Options>` to allow efficient sharing of option contract
/// data across different parts of the application without unnecessary cloning.
///
/// # Fields
///
/// * `long_call` - A call option that is purchased (long position), giving the right to buy
///   the underlying asset at the strike price.
///
/// * `short_call` - A call option that is sold (short position), creating an obligation to sell
///   the underlying asset at the strike price if the buyer exercises.
///
/// * `long_put` - A put option that is purchased (long position), giving the right to sell
///   the underlying asset at the strike price.
///
/// * `short_put` - A put option that is sold (short position), creating an obligation to buy
///   the underlying asset at the strike price if the buyer exercises.
///
/// # Usage
///
/// This structure is typically used in option strategy analysis, risk management,
/// and portfolio modeling where multiple option positions are evaluated together
/// to assess combined payoff profiles and risk characteristics.
#[derive(Debug, Clone)]
pub struct FourOptions {
    /// A purchased call option contract, giving the right to buy the underlying asset
    pub long_call: Arc<Options>,

    /// A sold call option contract, creating the obligation to sell the underlying if exercised
    pub short_call: Arc<Options>,

    /// A purchased put option contract, giving the right to sell the underlying asset
    pub long_put: Arc<Options>,

    /// A sold put option contract, creating the obligation to buy the underlying if exercised
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
    /// Creates a complete set of four standard option contracts based on specified pricing parameters.
    ///
    /// This method constructs four option contracts (long call, short call, long put, short put)
    /// with identical strike prices and expiration dates, all based on the same underlying asset.
    /// The resulting options are stored within the `OptionData` instance for further analysis
    /// or trading strategy evaluation.
    ///
    /// # Parameters
    ///
    /// * `price_params` - A reference to `OptionDataPriceParams` containing essential pricing inputs
    ///   including underlying price, expiration date, risk-free rate, dividend yield, and optionally
    ///   the underlying symbol and implied volatility.
    ///
    /// # Returns
    ///
    /// * `Result<(), ChainError>` - Returns `Ok(())` if option creation succeeds, or a `ChainError`
    ///   if any issues occur during creation.
    ///
    pub fn create_options(
        &mut self,
        price_params: &OptionDataPriceParams,
    ) -> Result<(), ChainError> {
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
            price_params.expiration_date,
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
            price_params.expiration_date,
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
            price_params.expiration_date,
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
            price_params.expiration_date,
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
