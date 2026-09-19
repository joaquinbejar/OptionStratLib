/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 19/9/25
******************************************************************************/

//! [`BasicAble`] implementations for the core model types.
//!
//! A single `Options` contract, and the `Position` that wraps one, satisfy
//! the strategy-level accessor contract by reporting themselves as a
//! one-leg strategy. The implementations live next to the trait (local
//! trait, core-owned type) so the core model never references the
//! strategies layer.

use crate::error::StrategyError;
use crate::model::types::{OptionBasicType, OptionStyle, OptionType, Side};
use crate::model::{ExpirationDate, Options, Position};
use crate::strategies::base::BasicAble;
use positive::Positive;
use rust_decimal::Decimal;
use std::collections::{HashMap, HashSet};

impl BasicAble for Options {
    fn get_title(&self) -> String {
        format!(
            "Underlying: {} @ ${:.0} {} {} {}",
            self.underlying_symbol,
            self.strike_price,
            self.side,
            self.option_style,
            self.option_type
        )
    }
    fn get_option_basic_type(&self) -> HashSet<OptionBasicType<'_>> {
        let mut hash_set = HashSet::new();
        hash_set.insert(OptionBasicType {
            option_style: &self.option_style,
            side: &self.side,
            strike_price: &self.strike_price,
            expiration_date: &self.expiration_date,
        });
        hash_set
    }
    fn get_symbol(&self) -> &str {
        self.underlying_symbol.as_str()
    }
    fn get_strike(&self) -> HashMap<OptionBasicType<'_>, &Positive> {
        let option_basic_type = match self.get_option_basic_type().iter().next().copied() {
            Some(option_basic_type) => option_basic_type,
            None => return HashMap::new(),
        };
        HashMap::from([(option_basic_type, &self.strike_price)])
    }
    fn get_side(&self) -> HashMap<OptionBasicType<'_>, &Side> {
        let option_basic_type = match self.get_option_basic_type().iter().next().copied() {
            Some(option_basic_type) => option_basic_type,
            None => return HashMap::new(),
        };
        HashMap::from([(option_basic_type, &self.side)])
    }
    fn get_type(&self) -> &OptionType {
        &self.option_type
    }
    fn get_style(&self) -> HashMap<OptionBasicType<'_>, &OptionStyle> {
        let option_basic_type = match self.get_option_basic_type().iter().next().copied() {
            Some(option_basic_type) => option_basic_type,
            None => return HashMap::new(),
        };
        HashMap::from([(option_basic_type, &self.option_style)])
    }
    fn get_expiration(&self) -> HashMap<OptionBasicType<'_>, &ExpirationDate> {
        let option_basic_type = match self.get_option_basic_type().iter().next().copied() {
            Some(option_basic_type) => option_basic_type,
            None => return HashMap::new(),
        };
        HashMap::from([(option_basic_type, &self.expiration_date)])
    }
    fn get_implied_volatility(&self) -> HashMap<OptionBasicType<'_>, &Positive> {
        let option_basic_type = match self.get_option_basic_type().iter().next().copied() {
            Some(option_basic_type) => option_basic_type,
            None => return HashMap::new(),
        };
        HashMap::from([(option_basic_type, &self.implied_volatility)])
    }
    fn get_quantity(&self) -> HashMap<OptionBasicType<'_>, &Positive> {
        let option_basic_type = match self.get_option_basic_type().iter().next().copied() {
            Some(option_basic_type) => option_basic_type,
            None => return HashMap::new(),
        };
        HashMap::from([(option_basic_type, &self.quantity)])
    }
    fn get_underlying_price(&self) -> &Positive {
        &self.underlying_price
    }
    fn get_risk_free_rate(&self) -> HashMap<OptionBasicType<'_>, &Decimal> {
        let option_basic_type = match self.get_option_basic_type().iter().next().copied() {
            Some(option_basic_type) => option_basic_type,
            None => return HashMap::new(),
        };
        HashMap::from([(option_basic_type, &self.risk_free_rate)])
    }
    fn get_dividend_yield(&self) -> HashMap<OptionBasicType<'_>, &Positive> {
        let option_basic_type = match self.get_option_basic_type().iter().next().copied() {
            Some(option_basic_type) => option_basic_type,
            None => return HashMap::new(),
        };
        HashMap::from([(option_basic_type, &self.dividend_yield)])
    }
    fn one_option(&self) -> &Options {
        self
    }
    fn one_option_mut(&mut self) -> &mut Options {
        self
    }
    fn set_implied_volatility(&mut self, volatility: &Positive) -> Result<(), StrategyError> {
        self.implied_volatility = *volatility;
        Ok(())
    }
    fn set_underlying_price(&mut self, price: &Positive) -> Result<(), StrategyError> {
        self.underlying_price = *price;
        Ok(())
    }
    fn set_expiration_date(
        &mut self,
        expiration_date: ExpirationDate,
    ) -> Result<(), StrategyError> {
        self.expiration_date = expiration_date;
        Ok(())
    }
}

impl BasicAble for Position {
    fn get_title(&self) -> String {
        self.option.get_title()
    }
    fn get_option_basic_type(&self) -> HashSet<OptionBasicType<'_>> {
        self.option.get_option_basic_type()
    }
    fn get_symbol(&self) -> &str {
        self.option.get_symbol()
    }
    fn get_strike(&self) -> HashMap<OptionBasicType<'_>, &Positive> {
        self.option.get_strike()
    }
    fn get_strikes(&self) -> Vec<&Positive> {
        self.option.get_strikes()
    }
    fn get_side(&self) -> HashMap<OptionBasicType<'_>, &Side> {
        self.option.get_side()
    }
    fn get_type(&self) -> &OptionType {
        self.option.get_type()
    }
    fn get_style(&self) -> HashMap<OptionBasicType<'_>, &OptionStyle> {
        self.option.get_style()
    }
    fn get_expiration(&self) -> HashMap<OptionBasicType<'_>, &ExpirationDate> {
        self.option.get_expiration()
    }
    fn get_implied_volatility(&self) -> HashMap<OptionBasicType<'_>, &Positive> {
        self.option.get_implied_volatility()
    }
    fn get_quantity(&self) -> HashMap<OptionBasicType<'_>, &Positive> {
        self.option.get_quantity()
    }
    fn get_underlying_price(&self) -> &Positive {
        self.option.get_underlying_price()
    }
    fn get_risk_free_rate(&self) -> HashMap<OptionBasicType<'_>, &Decimal> {
        self.option.get_risk_free_rate()
    }
    fn get_dividend_yield(&self) -> HashMap<OptionBasicType<'_>, &Positive> {
        self.option.get_dividend_yield()
    }
    fn one_option(&self) -> &Options {
        &self.option
    }
    fn one_option_mut(&mut self) -> &mut Options {
        &mut self.option
    }

    fn set_expiration_date(
        &mut self,
        expiration_date: ExpirationDate,
    ) -> Result<(), StrategyError> {
        self.option.set_expiration_date(expiration_date)
    }
    fn set_underlying_price(&mut self, _price: &Positive) -> Result<(), StrategyError> {
        self.option.set_underlying_price(_price)
    }
    /// Updates the volatility for the strategy.
    ///
    /// # Parameters
    /// - `_volatility`: A reference to a `Positive` value representing the new volatility to set.
    ///
    /// # Returns
    /// - `Ok(())`: If the update operation succeeds (currently unimplemented).
    /// - `Err(StrategyError)`: If there is an error during the update process (place-holder as functionality is not implemented).
    ///
    /// # Notes
    /// This method is currently unimplemented, and calling it will result in the `unimplemented!` macro being triggered, which causes a panic.
    /// This function is a stub and should be implemented to handle setting the volatility specific to the strategy.
    ///
    fn set_implied_volatility(&mut self, _volatility: &Positive) -> Result<(), StrategyError> {
        self.option.set_implied_volatility(_volatility)
    }
}
