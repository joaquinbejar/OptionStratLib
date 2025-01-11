use crate::chains::chain::OptionData;
use crate::constants::ZERO;
use crate::error::{GreeksError, OptionsError, OptionsResult};
use crate::greeks::equations::{delta, gamma, rho, rho_d, theta, vega, Greek, Greeks};
use crate::model::types::{ExpirationDate, OptionStyle, OptionType, Side};
use crate::pnl::utils::{PnL, PnLCalculator};
use crate::pricing::{
    black_scholes, generate_binomial_tree, price_binomial, telegraph, BinomialPricingParams,
    Payoff, PayoffInfo, Profit,
};
use crate::visualization::model::ChartVerticalLine;
use crate::visualization::utils::Graph;
use crate::Positive;
use num_traits::{FromPrimitive, ToPrimitive};
use plotters::prelude::{ShapeStyle, BLACK};
use rust_decimal::Decimal;
use std::error::Error;
use tracing::{error, trace};

type PriceBinomialTree = OptionsResult<(Decimal, Vec<Vec<Decimal>>, Vec<Vec<Decimal>>)>;

#[derive(Clone, Default, Debug, PartialEq)]
pub struct ExoticParams {
    pub spot_prices: Option<Vec<Positive>>, // Asian
    pub spot_min: Option<Decimal>,          // Lookback
    pub spot_max: Option<Decimal>,          // Lookback
}

#[derive(Clone, PartialEq)]
pub struct Options {
    pub option_type: OptionType,
    pub side: Side,
    pub underlying_symbol: String,
    pub strike_price: Positive,
    pub expiration_date: ExpirationDate,
    pub implied_volatility: Positive,
    pub quantity: Positive,
    pub underlying_price: Positive,
    pub risk_free_rate: Decimal,
    pub option_style: OptionStyle,
    pub dividend_yield: Positive,
    pub exotic_params: Option<ExoticParams>,
}

impl Options {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        option_type: OptionType,
        side: Side,
        underlying_symbol: String,
        strike_price: Positive,
        expiration_date: ExpirationDate,
        implied_volatility: Positive,
        quantity: Positive,
        underlying_price: Positive,
        risk_free_rate: Decimal,
        option_style: OptionStyle,
        dividend_yield: Positive,
        exotic_params: Option<ExoticParams>,
    ) -> Self {
        Options {
            option_type,
            side,
            underlying_symbol,
            strike_price,
            expiration_date,
            implied_volatility,
            quantity,
            underlying_price,
            risk_free_rate,
            option_style,
            dividend_yield,
            exotic_params,
        }
    }

    pub(crate) fn update_from_option_data(&mut self, option_data: &OptionData) {
        self.strike_price = option_data.strike_price;
        self.implied_volatility = option_data.implied_volatility.unwrap_or(Positive::ZERO);
        trace!("Updated Option: {:#?}", self);
    }

    pub fn time_to_expiration(&self) -> OptionsResult<Positive> {
        Ok(self.expiration_date.get_years()?)
    }

    pub fn is_long(&self) -> bool {
        matches!(self.side, Side::Long)
    }

    pub fn is_short(&self) -> bool {
        matches!(self.side, Side::Short)
    }

    pub fn calculate_price_binomial(&self, no_steps: usize) -> OptionsResult<Decimal> {
        if no_steps == 0 {
            return Err(OptionsError::OtherError {
                reason: "Number of steps cannot be zero".to_string(),
            });
        }
        let expiry = self.time_to_expiration()?;
        let cpb = price_binomial(BinomialPricingParams {
            asset: self.underlying_price,
            volatility: self.implied_volatility,
            int_rate: self.risk_free_rate,
            strike: self.strike_price,
            expiry,
            no_steps,
            option_type: &self.option_type,
            option_style: &self.option_style,
            side: &self.side,
        })?;
        Ok(cpb)
    }

    pub fn calculate_price_binomial_tree(&self, no_steps: usize) -> PriceBinomialTree {
        let expiry = self.time_to_expiration()?;
        let params = BinomialPricingParams {
            asset: self.underlying_price,
            volatility: self.implied_volatility,
            int_rate: self.risk_free_rate,
            strike: self.strike_price,
            expiry,
            no_steps,
            option_type: &self.option_type,
            option_style: &self.option_style,
            side: &self.side,
        };
        let (asset_tree, option_tree) = generate_binomial_tree(&params)?;
        let price = match self.side {
            Side::Long => option_tree[0][0],
            Side::Short => -option_tree[0][0],
        };
        Ok((price, asset_tree, option_tree))
    }

    pub fn calculate_price_black_scholes(&self) -> OptionsResult<Decimal> {
        Ok(black_scholes(self)?)
    }

    pub fn calculate_price_telegraph(&self, no_steps: usize) -> Result<Decimal, Box<dyn Error>> {
        telegraph(self, no_steps, None, None)
    }

    pub fn payoff(&self) -> OptionsResult<Decimal> {
        let payoff_info = PayoffInfo {
            spot: self.underlying_price,
            strike: self.strike_price,
            style: self.option_style.clone(),
            side: self.side.clone(),
            spot_prices: None,
            spot_min: None,
            spot_max: None,
        };
        let payoff = self.option_type.payoff(&payoff_info) * self.quantity.to_f64();
        Ok(Decimal::from_f64(payoff).unwrap())
    }

    pub fn payoff_at_price(&self, price: Positive) -> OptionsResult<Decimal> {
        let payoff_info = PayoffInfo {
            spot: price,
            strike: self.strike_price,
            style: self.option_style.clone(),
            side: self.side.clone(),
            spot_prices: None,
            spot_min: None,
            spot_max: None,
        };
        let price = self.option_type.payoff(&payoff_info) * self.quantity.to_f64();
        Ok(Decimal::from_f64(price).unwrap())
    }

    pub fn intrinsic_value(&self, underlying_price: Positive) -> OptionsResult<Decimal> {
        let payoff_info = PayoffInfo {
            spot: underlying_price,
            strike: self.strike_price,
            style: self.option_style.clone(),
            side: self.side.clone(),
            spot_prices: None,
            spot_min: None,
            spot_max: None,
        };
        let iv = self.option_type.payoff(&payoff_info) * self.quantity.to_f64();
        Ok(Decimal::from_f64(iv).unwrap())
    }

    pub fn delta(&self) -> Result<Decimal, GreeksError> {
        delta(self)
    }

    pub fn gamma(&self) -> Result<Decimal, GreeksError> {
        gamma(self)
    }

    pub fn theta(&self) -> Result<Decimal, GreeksError> {
        theta(self)
    }

    pub fn vega(&self) -> Result<Decimal, GreeksError> {
        vega(self)
    }

    pub fn rho(&self) -> Result<Decimal, GreeksError> {
        rho(self)
    }

    pub fn rho_d(&self) -> Result<Decimal, GreeksError> {
        rho_d(self)
    }

    pub fn is_in_the_money(&self) -> bool {
        match self.option_style {
            OptionStyle::Call => self.underlying_price >= self.strike_price,
            OptionStyle::Put => self.underlying_price <= self.strike_price,
        }
    }

    pub fn time_value(&self) -> OptionsResult<Decimal> {
        let option_price = self.calculate_price_black_scholes()?.abs();
        let intrinsic_value = self.intrinsic_value(self.underlying_price)?;

        Ok((option_price - intrinsic_value).max(Decimal::ZERO))
    }

    pub(crate) fn validate(&self) -> bool {
        if self.underlying_symbol == *"" {
            error!("Underlying symbol is empty");
            return false;
        }
        if self.implied_volatility < ZERO {
            error!("Implied volatility is less than zero");
            return false;
        }
        if self.quantity == ZERO {
            error!("Quantity is equal to zero");
            return false;
        }
        if self.risk_free_rate < Decimal::ZERO {
            error!("Risk free rate is less than zero");
            return false;
        }
        if self.strike_price == Positive::ZERO {
            error!("Strike is zero");
            return false;
        }
        if self.underlying_price == Positive::ZERO {
            error!("Underlying price is zero");
            return false;
        }
        true
    }
}

impl Default for Options {
    fn default() -> Self {
        Options {
            option_type: OptionType::European,
            side: Side::Long,
            underlying_symbol: "".to_string(),
            strike_price: Positive::ZERO,
            expiration_date: ExpirationDate::Days(Positive::ZERO),
            implied_volatility: Positive::ZERO,
            quantity: Positive::ZERO,
            underlying_price: Positive::ZERO,
            risk_free_rate: Decimal::ZERO,
            option_style: OptionStyle::Call,
            dividend_yield: Positive::ZERO,
            exotic_params: None,
        }
    }
}

impl Greeks for Options {
    fn greeks(&self) -> Greek {
        Greek {
            delta: self.delta().unwrap(),
            gamma: self.gamma().unwrap(),
            theta: self.theta().unwrap(),
            vega: self.vega().unwrap(),
            rho: self.rho().unwrap(),
            rho_d: self.rho_d().unwrap(),
        }
    }
}

impl PnLCalculator for Options {
    fn calculate_pnl(
        &self,
        market_price: &Positive,
        expiration_date: ExpirationDate,
        implied_volatility: &Positive,
    ) -> Result<PnL, Box<dyn Error>> {
        // Create a copy of the current option with updated parameters
        let mut current_option = self.clone();
        current_option.underlying_price = *market_price;
        current_option.expiration_date = expiration_date;
        current_option.implied_volatility = *implied_volatility;

        // Calculate theoretical price at current market conditions
        let current_price = current_option.calculate_price_black_scholes()?;

        // Calculate initial price (when option was created)
        let initial_price = self.calculate_price_black_scholes()?;

        // Calculate initial costs (premium paid/received)
        let (initial_costs, initial_income) = match self.side {
            Side::Long => (initial_price * self.quantity, Decimal::ZERO),
            Side::Short => (Decimal::ZERO, -initial_price * self.quantity),
        };

        // Calculate unrealized PnL adjusted for position side
        let unrealized = Some((current_price - initial_price) * self.quantity);

        Ok(PnL::new(
            None, // No realized PnL yet
            unrealized,
            initial_costs.into(),
            initial_income.into(),
            current_option.expiration_date.get_date()?,
        ))
    }

    fn calculate_pnl_at_expiration(
        &self,
        underlying_price: &Positive,
    ) -> Result<PnL, Box<dyn Error>> {
        let realized = Some(self.payoff_at_price(*underlying_price)?);
        let initial_price = self.calculate_price_black_scholes()?;

        let (initial_costs, initial_income) = match self.side {
            Side::Long => (initial_price * self.quantity, Decimal::ZERO),
            Side::Short => (Decimal::ZERO, initial_price * self.quantity),
        };

        Ok(PnL::new(
            realized, // No realized PnL yet
            None,
            initial_costs.into(),
            initial_income.into(),
            self.expiration_date.get_date()?,
        ))
    }
}

impl Profit for Options {
    fn calculate_profit_at(&self, price: Positive) -> Result<Decimal, Box<dyn Error>> {
        Ok(self.payoff_at_price(price)?)
    }
}

impl Graph for Options {
    fn title(&self) -> String {
        format!(
            "Underlying: {} @ ${:.0} {} {} {}",
            self.underlying_symbol,
            self.strike_price,
            self.side,
            self.option_style,
            self.option_type
        )
    }

    fn get_values(&self, data: &[Positive]) -> Vec<f64> {
        data.iter()
            .map(|&price| self.intrinsic_value(price).unwrap().to_f64().unwrap())
            .collect()
    }

    fn get_vertical_lines(&self) -> Vec<ChartVerticalLine<f64, f64>> {
        let vertical_lines = vec![ChartVerticalLine {
            x_coordinate: self.strike_price.to_f64(),
            y_range: (-50000.0, 50000.0),
            label: "Strike".to_string(),
            label_offset: (5.0, 5.0),
            line_color: BLACK,
            label_color: BLACK,
            line_style: ShapeStyle::from(&BLACK).stroke_width(1),
            font_size: 18,
        }];

        vertical_lines
    }
}

#[cfg(test)]
mod tests_options {
    use super::*;
    use crate::model::utils::create_sample_option_simplest;
    use crate::pos;
    use approx::assert_relative_eq;
    use chrono::{Duration, Utc};
    use rust_decimal_macros::dec;



    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_new_option() {
        let option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        assert_eq!(option.underlying_symbol, "AAPL");
        assert_eq!(option.strike_price, 100.0);
        assert_eq!(option.implied_volatility, 0.2);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_time_to_expiration() {
        let option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        assert_relative_eq!(
            option.time_to_expiration().unwrap().to_f64(),
            30.0 / 365.0,
            epsilon = 0.0001
        );

        let future_date = Utc::now() + Duration::days(60);
        let option_with_datetime = Options::new(
            OptionType::European,
            Side::Long,
            "AAPL".to_string(),
            pos!(100.0),
            ExpirationDate::DateTime(future_date),
            pos!(0.2),
            Positive::ONE,
            pos!(105.0),
            dec!(0.05),
            OptionStyle::Call,
            pos!(0.01),
            None,
        );
        assert!(option_with_datetime.time_to_expiration().unwrap() >= 59.0 / 365.0);
        assert!(option_with_datetime.time_to_expiration().unwrap() < 61.0 / 365.0);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_is_long_and_short() {
        let long_option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        assert!(long_option.is_long());
        assert!(!long_option.is_short());

        let short_option = Options::new(
            OptionType::European,
            Side::Short,
            "AAPL".to_string(),
            pos!(100.0),
            ExpirationDate::Days(pos!(30.0)),
            pos!(0.2),
            Positive::ONE,
            pos!(105.0),
            dec!(0.05),
            OptionStyle::Call,
            pos!(0.01),
            None,
        );
        assert!(!short_option.is_long());
        assert!(short_option.is_short());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_calculate_price_binomial() {
        let option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        let price = option.calculate_price_binomial(100).unwrap();
        assert!(price > Decimal::ZERO);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_calculate_price_binomial_tree() {
        let option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        let (price, asset_tree, option_tree) = option.calculate_price_binomial_tree(5).unwrap();
        assert!(price > Decimal::ZERO);
        assert_eq!(asset_tree.len(), 6);
        assert_eq!(option_tree.len(), 6);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_calculate_price_binomial_tree_short() {
        let option = create_sample_option_simplest(OptionStyle::Call, Side::Short);
        let (price, asset_tree, option_tree) = option.calculate_price_binomial_tree(5).unwrap();
        assert!(price > Decimal::ZERO);
        assert_eq!(asset_tree.len(), 6);
        assert_eq!(option_tree.len(), 6);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_calculate_price_black_scholes() {
        let option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        let price = option.calculate_price_black_scholes().unwrap();
        assert!(price > Decimal::ZERO);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_payoff_european_call_long() {
        let call_option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        let call_payoff = call_option.payoff().unwrap();
        assert_eq!(call_payoff, Decimal::ZERO); // max(100 - 100, 0) = 0

        let put_option = Options::new(
            OptionType::European,
            Side::Long,
            "AAPL".to_string(),
            pos!(100.0),
            ExpirationDate::Days(pos!(30.0)),
            pos!(0.2),
            Positive::ONE,
            pos!(95.0),
            dec!(0.05),
            OptionStyle::Put,
            pos!(0.01),
            None,
        );
        let put_payoff = put_option.payoff().unwrap();
        assert_eq!(put_payoff.to_f64().unwrap(), 5.0); // max(100 - 95, 0) = 5
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_calculate_time_value() {
        let option = Options::new(
            OptionType::European,
            Side::Long,
            "AAPL".to_string(),
            pos!(100.0),
            ExpirationDate::Days(pos!(30.0)),
            pos!(0.2),
            Positive::ONE,
            pos!(105.0),
            dec!(0.05),
            OptionStyle::Call,
            Positive::ZERO,
            None,
        );

        let time_value = option.time_value().unwrap();
        assert!(time_value > Decimal::ZERO);
        assert!(time_value < option.calculate_price_black_scholes().unwrap());
    }
}

#[cfg(test)]
mod tests_valid_option {
    use super::*;
    use crate::pos;
    use rust_decimal_macros::dec;



    fn create_valid_option() -> Options {
        Options {
            option_type: OptionType::European,
            side: Side::Long,
            underlying_symbol: "AAPL".to_string(),
            strike_price: pos!(100.0),
            expiration_date: ExpirationDate::Days(pos!(30.0)),
            implied_volatility: pos!(0.2),
            quantity: Positive::ONE,
            underlying_price: pos!(105.0),
            risk_free_rate: dec!(0.05),
            option_style: OptionStyle::Call,
            dividend_yield: pos!(0.01),
            exotic_params: None,
        }
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_valid_option() {
        let option = create_valid_option();
        assert!(option.validate());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_empty_underlying_symbol() {
        let mut option = create_valid_option();
        option.underlying_symbol = "".to_string();
        assert!(!option.validate());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_zero_strike_price() {
        let mut option = create_valid_option();
        option.strike_price = Positive::ZERO;
        assert!(!option.validate());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_zero_quantity() {
        let mut option = create_valid_option();
        option.quantity = Positive::ZERO;
        assert!(!option.validate());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_zero_underlying_price() {
        let mut option = create_valid_option();
        option.underlying_price = Positive::ZERO;
        assert!(!option.validate());
    }
}

#[cfg(test)]
mod tests_time_value {
    use super::*;
    use crate::model::utils::create_sample_option_simplest_strike;
    use crate::utils::logger::setup_logger;
    use crate::{assert_decimal_eq, pos};
    use rust_decimal_macros::dec;
    use tracing::debug;



    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_calculate_time_value_long_call() {
        setup_logger();
        let option =
            create_sample_option_simplest_strike(Side::Long, OptionStyle::Call, pos!(105.0));
        let time_value = option.time_value().unwrap();
        assert!(time_value > Decimal::ZERO);
        assert!(time_value <= option.calculate_price_black_scholes().unwrap());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_calculate_time_value_short_call() {
        let option =
            create_sample_option_simplest_strike(Side::Short, OptionStyle::Call, pos!(105.0));
        let time_value = option.time_value().unwrap();
        assert!(time_value > Decimal::ZERO);
        assert!(time_value <= option.calculate_price_black_scholes().unwrap().abs());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_calculate_time_value_long_put() {
        setup_logger();
        let option = create_sample_option_simplest_strike(Side::Long, OptionStyle::Put, pos!(95.0));
        let time_value = option.time_value().unwrap();
        assert!(time_value > Decimal::ZERO);
        assert!(time_value <= option.calculate_price_black_scholes().unwrap());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_calculate_time_value_short_put() {
        let option =
            create_sample_option_simplest_strike(Side::Short, OptionStyle::Put, pos!(95.0));
        let time_value = option.time_value().unwrap();
        assert!(time_value > Decimal::ZERO);
        assert!(time_value <= option.calculate_price_black_scholes().unwrap().abs());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_calculate_time_value_at_the_money() {
        let call = create_sample_option_simplest_strike(Side::Long, OptionStyle::Call, pos!(100.0));
        let put = create_sample_option_simplest_strike(Side::Long, OptionStyle::Put, pos!(100.0));

        let call_time_value = call.time_value().unwrap();
        let put_time_value = put.time_value().unwrap();

        assert!(call_time_value > Decimal::ZERO);
        assert!(put_time_value > Decimal::ZERO);
        assert_eq!(
            call_time_value,
            call.calculate_price_black_scholes().unwrap()
        );
        assert_eq!(put_time_value, put.calculate_price_black_scholes().unwrap());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_calculate_time_value_deep_in_the_money() {
        setup_logger();
        let call = create_sample_option_simplest_strike(Side::Long, OptionStyle::Call, pos!(150.0));
        let put = create_sample_option_simplest_strike(Side::Long, OptionStyle::Put, pos!(50.0));

        let call_time_value = call.time_value().unwrap();
        let put_time_value = put.time_value().unwrap();

        let call_price = call.calculate_price_black_scholes().unwrap();
        let put_price = put.calculate_price_black_scholes().unwrap();

        assert_decimal_eq!(call_time_value, call_price, dec!(0.01));
        assert_decimal_eq!(put_time_value, put_price, dec!(0.01));
        debug!("Call time value: {}", call_time_value);
        debug!("Call BS price: {}", call_price);
        debug!("Put time value: {}", put_time_value);
        debug!("Put BS price: {}", put_price);
        assert!(call_time_value <= call_price);
        assert!(put_time_value <= put_price);
    }
}

#[cfg(test)]
mod tests_options_payoffs {
    use super::*;
    use crate::model::utils::create_sample_option_simplest_strike;
    use crate::pos;
    use crate::utils::logger::setup_logger;
    use rust_decimal_macros::dec;



    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_payoff_european_call_long() {
        setup_logger();
        let call_option =
            create_sample_option_simplest_strike(Side::Long, OptionStyle::Call, pos!(95.0));
        let call_payoff = call_option.payoff().unwrap();
        assert_eq!(call_payoff, dec!(5.0)); // max(100 - 95, 0) = 5

        let call_option_otm =
            create_sample_option_simplest_strike(Side::Long, OptionStyle::Call, pos!(105.0));
        let call_payoff_otm = call_option_otm.payoff().unwrap();
        assert_eq!(call_payoff_otm, Decimal::ZERO); // max(100 - 105, 0) = 0
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_payoff_european_call_short() {
        setup_logger();
        let call_option =
            create_sample_option_simplest_strike(Side::Short, OptionStyle::Call, pos!(95.0));
        let call_payoff = call_option.payoff().unwrap();
        assert_eq!(call_payoff, dec!(-5.0)); // -max(100 - 95, 0) = -5

        let call_option_otm =
            create_sample_option_simplest_strike(Side::Short, OptionStyle::Call, pos!(105.0));
        let call_payoff_otm = call_option_otm.payoff().unwrap();
        assert_eq!(call_payoff_otm, Decimal::ZERO); // -max(95 - 100, 0) = 0
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_payoff_european_put_long() {
        let put_option =
            create_sample_option_simplest_strike(Side::Long, OptionStyle::Put, pos!(105.0));
        let put_payoff = put_option.payoff().unwrap();
        assert_eq!(put_payoff, dec!(5.0)); // max(105 - 100, 0) = 5

        let put_option_otm =
            create_sample_option_simplest_strike(Side::Long, OptionStyle::Put, pos!(95.0));
        let put_payoff_otm = put_option_otm.payoff().unwrap();
        assert_eq!(put_payoff_otm, Decimal::ZERO); // max(95 - 100, 0) = 0
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_payoff_european_put_short() {
        let put_option =
            create_sample_option_simplest_strike(Side::Short, OptionStyle::Put, pos!(105.0));
        let put_payoff = put_option.payoff().unwrap();
        assert_eq!(put_payoff, dec!(-5.0)); // -max(105 - 100, 0) = -5

        let put_option_otm =
            create_sample_option_simplest_strike(Side::Short, OptionStyle::Put, pos!(95.0));
        let put_payoff_otm = put_option_otm.payoff().unwrap();
        assert_eq!(put_payoff_otm, Decimal::ZERO); // -max(95 - 100, 0) = 0
    }
}

#[cfg(test)]
mod tests_options_payoff_at_price {
    use super::*;
    use crate::model::utils::create_sample_option_simplest;
    use crate::pos;
    use rust_decimal_macros::dec;



    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_payoff_european_call_long() {
        let call_option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        let call_payoff = call_option.payoff_at_price(pos!(105.0)).unwrap();
        assert_eq!(call_payoff, dec!(5.0)); // max(105 - 100, 0) = 5

        let call_option_otm = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        let call_payoff_otm = call_option_otm.payoff_at_price(pos!(95.0)).unwrap();
        assert_eq!(call_payoff_otm, Decimal::ZERO); // max(95 - 100, 0) = 0
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_payoff_european_call_short() {
        let call_option = create_sample_option_simplest(OptionStyle::Call, Side::Short);
        let call_payoff = call_option.payoff_at_price(pos!(105.0)).unwrap();
        assert_eq!(call_payoff, dec!(-5.0)); // -max(105 - 100, 0) = -5

        let call_option_otm = create_sample_option_simplest(OptionStyle::Call, Side::Short);
        let call_payoff_otm = call_option_otm.payoff_at_price(pos!(95.0)).unwrap();
        assert_eq!(call_payoff_otm, Decimal::ZERO); // -max(95 - 100, 0) = 0
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_payoff_european_put_long() {
        let put_option = create_sample_option_simplest(OptionStyle::Put, Side::Long);
        let put_payoff = put_option.payoff_at_price(pos!(95.0)).unwrap();
        assert_eq!(put_payoff, dec!(5.0)); // max(100 - 95, 0) = 5

        let put_option_otm = create_sample_option_simplest(OptionStyle::Put, Side::Long);
        let put_payoff_otm = put_option_otm.payoff_at_price(pos!(105.0)).unwrap();
        assert_eq!(put_payoff_otm, Decimal::ZERO); // max(100 - 105, 0) = 0
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_payoff_european_put_short() {
        let put_option = create_sample_option_simplest(OptionStyle::Put, Side::Short);
        let put_payoff = put_option.payoff_at_price(pos!(95.0)).unwrap();
        assert_eq!(put_payoff, dec!(-5.0)); // -max(100 - 95, 0) = -5

        let put_option_otm = create_sample_option_simplest(OptionStyle::Put, Side::Short);
        let put_payoff_otm = put_option_otm.payoff_at_price(pos!(105.0)).unwrap();
        assert_eq!(put_payoff_otm, Decimal::ZERO); // -max(100 - 105, 0) = 0
    }
}

#[cfg(test)]
mod tests_options_payoffs_with_quantity {
    use super::*;
    use crate::model::utils::create_sample_option;
    use crate::pos;
    use rust_decimal_macros::dec;



    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_payoff_call_long() {
        let option = create_sample_option(
            OptionStyle::Call,
            Side::Long,
            pos!(105.0),
            pos!(10.0),
            pos!(100.0),
            pos!(0.02),
        );
        assert_eq!(option.payoff().unwrap().to_f64().unwrap(), 50.0);

        let option_otm = create_sample_option(
            OptionStyle::Call,
            Side::Long,
            pos!(95.0),
            pos!(4.0),
            pos!(100.0),
            pos!(0.02),
        );
        assert_eq!(option_otm.payoff().unwrap(), Decimal::ZERO);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_payoff_call_short() {
        let option = create_sample_option(
            OptionStyle::Call,
            Side::Short,
            pos!(105.0),
            pos!(3.0),
            pos!(100.0),
            pos!(0.02),
        );
        assert_eq!(option.payoff().unwrap().to_f64().unwrap(), -15.0);

        let option_otm = create_sample_option(
            OptionStyle::Call,
            Side::Short,
            pos!(95.0),
            pos!(7.0),
            pos!(100.0),
            pos!(0.02),
        );
        assert_eq!(option_otm.payoff().unwrap(), Decimal::ZERO);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_payoff_put_long() {
        let option = create_sample_option(
            OptionStyle::Put,
            Side::Long,
            pos!(95.0),
            pos!(2.0),
            pos!(100.0),
            pos!(0.02),
        );
        assert_eq!(option.payoff().unwrap().to_f64().unwrap(), 10.0);

        let option_otm = create_sample_option(
            OptionStyle::Put,
            Side::Long,
            pos!(105.0),
            pos!(7.0),
            pos!(100.0),
            pos!(0.02),
        );
        assert_eq!(option_otm.payoff().unwrap(), Decimal::ZERO);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_payoff_put_short() {
        let option = create_sample_option(
            OptionStyle::Put,
            Side::Short,
            pos!(95.0),
            pos!(3.0),
            pos!(100.0),
            pos!(0.02),
        );
        assert_eq!(option.payoff().unwrap().to_f64().unwrap(), -15.0);

        let option_otm = create_sample_option(
            OptionStyle::Put,
            Side::Short,
            pos!(105.0),
            pos!(3.0),
            pos!(100.0),
            pos!(0.02),
        );
        assert_eq!(option_otm.payoff().unwrap(), Decimal::ZERO);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_payoff_with_quantity() {
        let option = create_sample_option(
            OptionStyle::Call,
            Side::Long,
            pos!(110.0),
            pos!(3.0),
            pos!(100.0),
            pos!(0.02),
        );
        assert_eq!(option.payoff().unwrap().to_f64().unwrap(), 30.0); // (110 - 100) * 3
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_intrinsic_value_call_long() {
        let option = create_sample_option(
            OptionStyle::Call,
            Side::Long,
            pos!(100.0),
            pos!(11.0),
            pos!(100.0),
            pos!(0.02),
        );
        assert_eq!(option.intrinsic_value(pos!(105.0)).unwrap(), dec!(55.0));
        assert_eq!(option.intrinsic_value(pos!(95.0)).unwrap(), Decimal::ZERO);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_intrinsic_value_call_short() {
        let option = create_sample_option(
            OptionStyle::Call,
            Side::Short,
            pos!(100.0),
            pos!(13.0),
            pos!(100.0),
            pos!(0.02),
        );
        assert_eq!(option.intrinsic_value(pos!(105.0)).unwrap(), dec!(-65.0));
        assert_eq!(option.intrinsic_value(pos!(95.0)).unwrap(), Decimal::ZERO);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_intrinsic_value_put_long() {
        let option = create_sample_option(
            OptionStyle::Put,
            Side::Long,
            pos!(100.0),
            pos!(17.0),
            pos!(100.0),
            pos!(0.02),
        );
        assert_eq!(option.intrinsic_value(pos!(95.0)).unwrap(), dec!(85.0));
        assert_eq!(option.intrinsic_value(pos!(105.0)).unwrap(), Decimal::ZERO);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_intrinsic_value_put_short() {
        let option = create_sample_option(
            OptionStyle::Put,
            Side::Short,
            pos!(100.0),
            pos!(19.0),
            pos!(100.0),
            pos!(0.02),
        );
        assert_eq!(option.intrinsic_value(pos!(95.0)).unwrap(), dec!(-95.0));
        assert_eq!(option.intrinsic_value(pos!(105.0)).unwrap(), Decimal::ZERO);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_intrinsic_value_with_quantity() {
        let option = create_sample_option(
            OptionStyle::Call,
            Side::Long,
            pos!(100.0),
            pos!(23.0),
            pos!(100.0),
            pos!(0.02),
        );
        assert_eq!(option.intrinsic_value(pos!(110.0)).unwrap(), dec!(230.0)); // (110 - 100) * 23
    }
}

#[cfg(test)]
mod tests_in_the_money {
    use super::*;
    use crate::model::utils::create_sample_option;
    use crate::pos;



    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_call_in_the_money() {
        let mut option = create_sample_option(
            OptionStyle::Call,
            Side::Long,
            pos!(110.0),
            pos!(1.0),
            pos!(110.0),
            pos!(0.02),
        );
        option.strike_price = pos!(100.0);
        assert!(option.is_in_the_money());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_call_at_the_money() {
        let mut option = create_sample_option(
            OptionStyle::Call,
            Side::Long,
            pos!(100.0),
            pos!(1.0),
            pos!(110.0),
            pos!(0.02),
        );
        option.strike_price = pos!(100.0);
        assert!(option.is_in_the_money());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_call_out_of_the_money() {
        let mut option = create_sample_option(
            OptionStyle::Call,
            Side::Long,
            pos!(90.0),
            pos!(1.0),
            pos!(110.0),
            pos!(0.02),
        );
        option.strike_price = pos!(100.0);
        assert!(!option.is_in_the_money());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_put_in_the_money() {
        let mut option = create_sample_option(
            OptionStyle::Put,
            Side::Long,
            pos!(90.0),
            pos!(1.0),
            pos!(110.0),
            pos!(0.02),
        );
        option.strike_price = pos!(100.0);
        assert!(option.is_in_the_money());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_put_at_the_money() {
        let mut option = create_sample_option(
            OptionStyle::Put,
            Side::Long,
            pos!(100.0),
            pos!(1.0),
            pos!(110.0),
            pos!(0.02),
        );
        option.strike_price = pos!(100.0);
        assert!(option.is_in_the_money());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_put_out_of_the_money() {
        let mut option = create_sample_option(
            OptionStyle::Put,
            Side::Long,
            pos!(110.0),
            pos!(1.0),
            pos!(110.0),
            pos!(0.02),
        );
        option.strike_price = pos!(100.0);
        assert!(!option.is_in_the_money());
    }
}

#[cfg(test)]
mod tests_greeks {
    use super::*;
    use crate::model::utils::create_sample_option_simplest;
    use crate::{assert_decimal_eq, pos};
    use rust_decimal_macros::dec;

    const EPSILON: Decimal = dec!(1e-6);



    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_delta() {
        let delta = create_sample_option_simplest(OptionStyle::Call, Side::Long)
            .delta()
            .unwrap();
        assert_decimal_eq!(delta, dec!(0.539519922), EPSILON);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_delta_size() {
        let mut option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        option.quantity = pos!(2.0);
        assert_decimal_eq!(option.delta().unwrap(), dec!(1.0790398), EPSILON);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_gamma() {
        let option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        assert_decimal_eq!(option.gamma().unwrap(), dec!(0.0691707), EPSILON);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_gamma_size() {
        let mut option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        option.quantity = pos!(2.0);
        assert_decimal_eq!(option.gamma().unwrap(), dec!(0.1383415), EPSILON);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_theta() {
        let option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        assert_decimal_eq!(option.theta().unwrap(), dec!(-15.8697818), EPSILON);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_theta_size() {
        let mut option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        option.quantity = pos!(2.0);
        assert_decimal_eq!(option.theta().unwrap(), dec!(-31.739563), EPSILON);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_vega() {
        let option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        assert_decimal_eq!(option.vega().unwrap(), dec!(15.4675554), EPSILON);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_vega_size() {
        let mut option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        option.quantity = pos!(2.0);
        assert_decimal_eq!(option.vega().unwrap(), dec!(30.9351108), EPSILON);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_rho() {
        let option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        assert_decimal_eq!(option.rho().unwrap(), dec!(4.23312145), EPSILON);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_rho_size() {
        let mut option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        option.quantity = pos!(2.0);
        assert_decimal_eq!(option.rho().unwrap(), dec!(8.46624291), EPSILON);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_rho_d() {
        let option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        assert_decimal_eq!(option.rho_d().unwrap(), dec!(-4.43441032), EPSILON);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_rho_d_size() {
        let mut option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        option.quantity = pos!(2.0);
        assert_decimal_eq!(option.rho_d().unwrap(), dec!(-8.86882064), EPSILON);
    }
}

#[cfg(test)]
mod tests_greek_trait {
    use super::*;
    use crate::assert_decimal_eq;
    use crate::model::utils::create_sample_option_simplest;
    use rust_decimal_macros::dec;

    const EPSILON: Decimal = dec!(1e-6);



    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_greeks_implementation() {
        let option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        let greeks = option.greeks();

        assert_decimal_eq!(greeks.delta, option.delta().unwrap(), EPSILON);
        assert_decimal_eq!(greeks.gamma, option.gamma().unwrap(), EPSILON);
        assert_decimal_eq!(greeks.theta, option.theta().unwrap(), EPSILON);
        assert_decimal_eq!(greeks.vega, option.vega().unwrap(), EPSILON);
        assert_decimal_eq!(greeks.rho, option.rho().unwrap(), EPSILON);
        assert_decimal_eq!(greeks.rho_d, option.rho_d().unwrap(), EPSILON);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_greeks_consistency() {
        let option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        let greeks = option.greeks();

        assert!(
            greeks.delta >= Decimal::NEGATIVE_ONE && greeks.delta <= Decimal::ONE,
            "Delta should be between -1 and 1"
        );
        assert!(
            greeks.gamma >= Decimal::ZERO,
            "Gamma should be non-negative"
        );
        assert!(greeks.vega >= Decimal::ZERO, "Vega should be non-negative");
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_greeks_for_different_options() {
        let call_option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        let mut put_option = call_option.clone();
        put_option.option_style = OptionStyle::Put;

        let call_greeks = call_option.greeks();
        let put_greeks = put_option.greeks();

        // assert_decimal_eq!(call_greeks.delta + put_greeks.delta, Decimal::ZERO, EPSILON); // TODO: Fix this
        assert_decimal_eq!(call_greeks.gamma, put_greeks.gamma, EPSILON);
        assert_decimal_eq!(call_greeks.vega, put_greeks.vega, EPSILON);
        // assert_decimal_eq!(call_greeks.rho, put_greeks.rho, EPSILON); // TODO: Fix this
    }
}

#[cfg(test)]
mod tests_graph {
    use super::*;
    use crate::model::utils::create_sample_option_simplest;
    use crate::pos;
    use crate::visualization::utils::Graph;
    use approx::assert_relative_eq;



    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_title() {
        let option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        let expected_title = "Underlying: AAPL @ $100 Long Call European Option".to_string();
        assert_eq!(option.title(), expected_title);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_get_values() {
        let option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        let prices = vec![pos!(90.0), pos!(100.0), pos!(110.0)];
        let values = option.get_values(&prices);

        assert_eq!(values.len(), 3);
        assert_relative_eq!(values[0], 0.0, epsilon = 1e-6);
        assert_relative_eq!(values[1], 0.0, epsilon = 1e-6);
        assert_relative_eq!(values[2], 10.0, epsilon = 1e-6);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_get_vertical_lines() {
        let option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        let vertical_lines = option.get_vertical_lines();

        assert_eq!(vertical_lines.len(), 1);
        assert_eq!(vertical_lines[0].label, "Strike");
        assert_relative_eq!(vertical_lines[0].x_coordinate, 100.0, epsilon = 1e-6);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_title_put_option() {
        let option = create_sample_option_simplest(OptionStyle::Put, Side::Long);
        let expected_title = "Underlying: AAPL @ $100 Long Put European Option".to_string();
        assert_eq!(option.title(), expected_title);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_get_values_put_option() {
        let option = create_sample_option_simplest(OptionStyle::Put, Side::Long);
        let prices = vec![pos!(90.0), pos!(100.0), pos!(110.0)];
        let values = option.get_values(&prices);

        assert_eq!(values.len(), 3);
        assert_relative_eq!(values[0], 10.0, epsilon = 1e-6);
        assert_relative_eq!(values[1], 0.0, epsilon = 1e-6);
        assert_relative_eq!(values[2], 0.0, epsilon = 1e-6);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_get_values_short_option() {
        let option = create_sample_option_simplest(OptionStyle::Call, Side::Short);
        let prices = vec![pos!(90.0), pos!(100.0), pos!(110.0)];
        let values = option.get_values(&prices);

        assert_eq!(values.len(), 3);
        assert_relative_eq!(values[0], 0.0, epsilon = 1e-6);
        assert_relative_eq!(values[1], 0.0, epsilon = 1e-6);
        assert_relative_eq!(values[2], -10.0, epsilon = 1e-6);
    }
}

#[cfg(test)]
mod tests_calculate_price_binomial {
    use super::*;
    use crate::model::utils::{
        create_sample_option, create_sample_option_simplest, create_sample_option_with_date,
    };
    use crate::pos;
    use chrono::Utc;
    use rust_decimal_macros::dec;
    use std::str::FromStr;

    #[test]
    fn test_european_call_option_basic() {
        // Test a basic European call option with standard parameters
        let option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        let result = option.calculate_price_binomial(100);
        assert!(result.is_ok());
        let price = result.unwrap();
        // Price should be positive for a long call at-the-money
        assert!(price > Decimal::ZERO);
    }

    #[test]
    fn test_american_put_option() {
        // Test American put option which should have early exercise value
        let option = Options::new(
            OptionType::American,
            Side::Long,
            "TEST".to_string(),
            pos!(100.0),
            ExpirationDate::Days(pos!(30.0)),
            pos!(0.2),  // volatility
            pos!(1.0),  // quantity
            pos!(95.0), // underlying price (slightly ITM for put)
            dec!(0.05), // risk-free rate
            OptionStyle::Put,
            Positive::ZERO, // dividend yield
            None,
        );

        let result = option.calculate_price_binomial(100);
        assert!(result.is_ok());
        let price = result.unwrap();
        // Price should be positive and reflect early exercise premium
        assert!(price > Decimal::ZERO);
    }

    #[test]
    fn test_zero_volatility() {
        let mut option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        option.implied_volatility = Positive::ZERO;
        let result = option.calculate_price_binomial(100);
        assert!(result.is_ok());
        // With zero volatility, price should equal discounted intrinsic value
    }

    #[test]
    fn test_zero_time_to_expiry() {
        // Test option at expiration
        let now = Utc::now().naive_utc();
        let option = create_sample_option_with_date(
            OptionStyle::Call,
            Side::Long,
            pos!(100.0),
            pos!(1.0),
            pos!(95.0),
            pos!(0.2),
            now,
        );

        let result = option.calculate_price_binomial(100);
        assert!(result.is_ok());
        let price = result.unwrap();
        // At expiry, price should equal intrinsic value
        assert_eq!(price, Decimal::from(5)); // 100 - 95 = 5
    }

    #[test]
    fn test_invalid_steps() {
        let option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        let result = option.calculate_price_binomial(0);
        assert!(result.is_err());
    }

    #[test]
    fn test_deep_itm_call() {
        let option = create_sample_option(
            OptionStyle::Call,
            Side::Long,
            pos!(150.0), // Underlying price much higher than strike
            pos!(1.0),
            pos!(100.0),
            pos!(0.2),
        );

        let result = option.calculate_price_binomial(100);
        assert!(result.is_ok());
        let price = result.unwrap();
        // Price should be close to intrinsic value for deep ITM
        assert!(price > Decimal::from(45)); // At least intrinsic - some time value
    }

    #[test]
    fn test_deep_otm_put() {
        let option = create_sample_option(
            OptionStyle::Put,
            Side::Long,
            pos!(150.0), // Underlying price much higher than strike
            pos!(1.0),
            pos!(100.0),
            pos!(0.2),
        );

        let result = option.calculate_price_binomial(100);
        assert!(result.is_ok());
        let price = result.unwrap();
        // Price should be very small for deep OTM
        assert!(price < Decimal::from(1));
    }

    #[test]
    fn test_convergence() {
        let option = create_sample_option_simplest(OptionStyle::Call, Side::Long);

        // Test that increasing steps leads to convergence
        let price_100 = option.calculate_price_binomial(100).unwrap();
        let price_1000 = option.calculate_price_binomial(1000).unwrap();

        // Prices should be close to each other
        let diff = (price_1000 - price_100).abs();
        assert!(diff < Decimal::from_str("0.1").unwrap());
    }

    #[test]
    fn test_short_position() {
        let long_call_option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        let mut short_call_option = long_call_option.clone();
        short_call_option.side = Side::Short;
        let mut short_put_option = short_call_option.clone();
        short_put_option.option_style = OptionStyle::Put;
        let mut long_put_option = short_put_option.clone();
        long_put_option.side = Side::Long;

        let long_call_price = long_call_option.calculate_price_binomial(100).unwrap();
        let short_call_price = short_call_option.calculate_price_binomial(100).unwrap();
        let long_put_price = long_put_option.calculate_price_binomial(100).unwrap();
        let short_put_price = short_put_option.calculate_price_binomial(100).unwrap();

        // Short position should be negative of long position
        assert_eq!(long_call_price, -short_call_price);
        assert_eq!(long_put_price, -short_put_price);
    }
}
