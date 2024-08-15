use crate::model::types::{ExpirationDate, OptionStyle, OptionType, Side};
use crate::pricing::binomial_model::{
    generate_binomial_tree, price_binomial, BinomialPricingParams,
};
use crate::pricing::black_scholes_model::black_scholes;
use crate::pricing::payoff::{Payoff, PayoffInfo};

#[allow(dead_code)]
#[derive(Clone)]
pub struct Options {
    pub option_type: OptionType,
    pub side: Side,
    pub underlying_symbol: String,
    pub strike_price: f64,
    pub expiration_date: ExpirationDate,
    pub implied_volatility: f64,
    pub quantity: u32,
    pub underlying_price: f64,
    pub risk_free_rate: f64,
    pub option_style: OptionStyle,
    pub dividend_yield: f64,
    // pub spot_prices: Option<Vec<f64>>, // Asian
    // pub spot_min: Option<f64>,         // Lookback
    // pub spot_max: Option<f64>,         // Lookback
    pub exotic_params: Option<ExoticParams>,
}

#[derive(Clone, Default)]
pub struct ExoticParams {
    pub spot_prices: Option<Vec<f64>>, // Asian
    pub spot_min: Option<f64>,         // Lookback
    pub spot_max: Option<f64>,         // Lookback
}

impl Options {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        option_type: OptionType,
        side: Side,
        underlying_symbol: String,
        strike_price: f64,
        expiration_date: ExpirationDate,
        implied_volatility: f64,
        quantity: u32,
        underlying_price: f64,
        risk_free_rate: f64,
        option_style: OptionStyle,
        dividend_yield: f64,
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

    pub fn time_to_expiration(&self) -> f64 {
        self.expiration_date.get_years()
    }
    pub fn is_long(&self) -> bool {
        matches!(self.side, Side::Long)
    }

    pub fn is_short(&self) -> bool {
        matches!(self.side, Side::Short)
    }

    pub fn calculate_price_binomial(&self, no_steps: usize) -> f64 {
        let expiry = self.time_to_expiration();
        price_binomial(BinomialPricingParams {
            asset: self.underlying_price,
            volatility: self.implied_volatility,
            int_rate: self.risk_free_rate,
            strike: self.strike_price,
            expiry,
            no_steps,
            option_type: &self.option_type,
            option_style: &self.option_style,
            side: &self.side,
        })
    }

    pub fn calculate_price_binomial_tree(
        &self,
        no_steps: usize,
    ) -> (f64, Vec<Vec<f64>>, Vec<Vec<f64>>) {
        let expiry = self.time_to_expiration();
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

        let (asset_tree, option_tree) = generate_binomial_tree(&params);
        let price = match self.side {
            Side::Long => option_tree[0][0],
            Side::Short => -option_tree[0][0],
        };

        (price, asset_tree, option_tree)
    }

    pub fn calculate_price_black_scholes(&self) -> f64 {
        black_scholes(self)
    }

    pub fn payoff(&self) -> f64 {
        let payoff_info = PayoffInfo {
            spot: self.underlying_price,
            strike: self.strike_price,
            style: self.option_style.clone(),
            spot_prices: None,
            spot_min: None,
            spot_max: None,
        };
        self.option_type.payoff(&payoff_info, &self.side)
    }

    // TODO:
    // - calculate_intrinsic_value
    // - calculate_time_value
    // - is_in_the_money
    // - calculate_delta
    // - calculate_gamma
    // - calculate_theta
    // - calculate_vega
    // - calculate_rho
}

#[cfg(test)]
mod tests_options {
    use super::*;
    use chrono::{Duration, Utc};

    fn create_sample_option() -> Options {
        Options::new(
            OptionType::European,
            Side::Long,
            "AAPL".to_string(),
            100.0,
            ExpirationDate::Days(30.0),
            0.2,
            1,
            105.0,
            0.05,
            OptionStyle::Call,
            0.0,
            None,
        )
    }

    #[test]
    fn test_new_option() {
        let option = create_sample_option();
        assert_eq!(option.underlying_symbol, "AAPL");
        assert_eq!(option.strike_price, 100.0);
        assert_eq!(option.implied_volatility, 0.2);
    }

    #[test]
    fn test_time_to_expiration() {
        let option = create_sample_option();
        assert_eq!(option.time_to_expiration(), 30.0 / 365.0);

        let future_date = Utc::now() + Duration::days(60);
        let option_with_datetime = Options::new(
            OptionType::European,
            Side::Long,
            "AAPL".to_string(),
            100.0,
            ExpirationDate::DateTime(future_date),
            0.2,
            1,
            105.0,
            0.05,
            OptionStyle::Call,
            0.01,
            None,
        );
        assert!(option_with_datetime.time_to_expiration() >= 59.0 / 365.0);
        assert!(option_with_datetime.time_to_expiration() < 61.0 / 365.0);
    }

    #[test]
    fn test_is_long_and_short() {
        let long_option = create_sample_option();
        assert!(long_option.is_long());
        assert!(!long_option.is_short());

        let short_option = Options::new(
            OptionType::European,
            Side::Short,
            "AAPL".to_string(),
            100.0,
            ExpirationDate::Days(30.0),
            0.2,
            1,
            105.0,
            0.05,
            OptionStyle::Call,
            0.01,
            None,
        );
        assert!(!short_option.is_long());
        assert!(short_option.is_short());
    }

    #[test]
    fn test_calculate_price_binomial() {
        let option = create_sample_option();
        let price = option.calculate_price_binomial(100);
        assert!(price > 0.0);
    }

    #[test]
    fn test_calculate_price_binomial_tree() {
        let option = create_sample_option();
        let (price, asset_tree, option_tree) = option.calculate_price_binomial_tree(5);
        assert!(price > 0.0);
        assert_eq!(asset_tree.len(), 6);
        assert_eq!(option_tree.len(), 6);
    }

    #[test]
    fn test_calculate_price_black_scholes() {
        let option = create_sample_option();
        let price = option.calculate_price_black_scholes();
        assert!(price > 0.0);
    }

    #[test]
    fn test_payoff_european_call_long() {
        let call_option = create_sample_option();
        let call_payoff = call_option.payoff();
        assert_eq!(call_payoff, 5.0); // max(105 - 100, 0) = 5

        let put_option = Options::new(
            OptionType::European,
            Side::Long,
            "AAPL".to_string(),
            100.0,
            ExpirationDate::Days(30.0),
            0.2,
            1,
            95.0,
            0.05,
            OptionStyle::Put,
            0.01,
            None,
        );
        let put_payoff = put_option.payoff();
        assert_eq!(put_payoff, 5.0); // max(100 - 95, 0) = 5
    }
}

#[cfg(test)]
mod tests_options_payoffs {
    use super::*;

    fn create_sample_option(
        option_style: OptionStyle,
        side: Side,
        underlying_price: f64,
    ) -> Options {
        Options::new(
            OptionType::European,
            side,
            "AAPL".to_string(),
            100.0,
            ExpirationDate::Days(30.0),
            0.2,
            1,
            underlying_price,
            0.05,
            option_style,
            0.01,
            None,
        )
    }

    #[test]
    fn test_payoff_european_call_long() {
        let call_option = create_sample_option(OptionStyle::Call, Side::Long, 105.0);
        let call_payoff = call_option.payoff();
        assert_eq!(call_payoff, 5.0); // max(105 - 100, 0) = 5

        let call_option_otm = create_sample_option(OptionStyle::Call, Side::Long, 95.0);
        let call_payoff_otm = call_option_otm.payoff();
        assert_eq!(call_payoff_otm, 0.0); // max(95 - 100, 0) = 0
    }

    #[test]
    fn test_payoff_european_call_short() {
        let call_option = create_sample_option(OptionStyle::Call, Side::Short, 105.0);
        let call_payoff = call_option.payoff();
        assert_eq!(call_payoff, -5.0); // -max(105 - 100, 0) = -5

        let call_option_otm = create_sample_option(OptionStyle::Call, Side::Short, 95.0);
        let call_payoff_otm = call_option_otm.payoff();
        assert_eq!(call_payoff_otm, 0.0); // -max(95 - 100, 0) = 0
    }

    #[test]
    fn test_payoff_european_put_long() {
        let put_option = create_sample_option(OptionStyle::Put, Side::Long, 95.0);
        let put_payoff = put_option.payoff();
        assert_eq!(put_payoff, 5.0); // max(100 - 95, 0) = 5

        let put_option_otm = create_sample_option(OptionStyle::Put, Side::Long, 105.0);
        let put_payoff_otm = put_option_otm.payoff();
        assert_eq!(put_payoff_otm, 0.0); // max(100 - 105, 0) = 0
    }

    #[test]
    fn test_payoff_european_put_short() {
        let put_option = create_sample_option(OptionStyle::Put, Side::Short, 95.0);
        let put_payoff = put_option.payoff();
        assert_eq!(put_payoff, -5.0); // -max(100 - 95, 0) = -5

        let put_option_otm = create_sample_option(OptionStyle::Put, Side::Short, 105.0);
        let put_payoff_otm = put_option_otm.payoff();
        assert_eq!(put_payoff_otm, 0.0); // -max(100 - 105, 0) = 0
    }
}
