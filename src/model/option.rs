use crate::chains::chain::OptionData;
use crate::constants::ZERO;
use crate::error::decimal::DecimalError;
use crate::error::greeks::GreeksError;
use crate::{f2du, Positive};
use crate::greeks::equations::{delta, gamma, rho, rho_d, theta, vega, Greek, Greeks};
use crate::model::types::{ExpirationDate, OptionStyle, OptionType, Side};
use crate::pnl::utils::{PnL, PnLCalculator};
use crate::pricing::binomial_model::{
    generate_binomial_tree, price_binomial, BinomialPricingParams,
};
use crate::pricing::black_scholes_model::black_scholes;
use crate::pricing::payoff::{Payoff, PayoffInfo, Profit};
use crate::pricing::telegraph;
use crate::visualization::model::ChartVerticalLine;
use crate::visualization::utils::Graph;
use chrono::{DateTime, Utc};
use plotters::prelude::{ShapeStyle, BLACK};
use rust_decimal::Decimal;
use tracing::{debug, error, trace};

#[derive(Clone, Default, Debug, PartialEq)]
pub struct ExoticParams {
    pub spot_prices: Option<Vec<f64>>, // Asian
    pub spot_min: Option<f64>,         // Lookback
    pub spot_max: Option<f64>,         // Lookback
}

#[allow(dead_code)]
#[derive(Clone, PartialEq)]
pub struct Options {
    pub option_type: OptionType,
    pub side: Side,
    pub underlying_symbol: String,
    pub strike_price: Positive,
    pub expiration_date: ExpirationDate,
    pub implied_volatility: f64,
    pub quantity: Positive,
    pub underlying_price: Positive,
    pub risk_free_rate: f64,
    pub option_style: OptionStyle,
    pub dividend_yield: f64,
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
        implied_volatility: f64,
        quantity: Positive,
        underlying_price: Positive,
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

    pub(crate) fn update_from_option_data(&mut self, option_data: &OptionData) {
        self.strike_price = option_data.strike_price;
        self.implied_volatility = option_data.implied_volatility.unwrap_or(Positive::ZERO).to_f64();
        trace!("Updated Option: {:#?}", self);
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

    pub fn calculate_price_binomial(&self, no_steps: usize) -> Decimal {
        let expiry = self.time_to_expiration();
        price_binomial(BinomialPricingParams {
            asset: self.underlying_price,
            volatility: f2du!(self.implied_volatility).unwrap(),
            int_rate: f2du!(self.risk_free_rate).unwrap(),
            strike: self.strike_price,
            expiry: f2du!(expiry).unwrap(),
            no_steps,
            option_type: &self.option_type,
            option_style: &self.option_style,
            side: &self.side,
        })
        .unwrap()
    }

    pub fn calculate_price_binomial_tree(
        &self,
        no_steps: usize,
    ) -> (Decimal, Vec<Vec<Decimal>>, Vec<Vec<Decimal>>) {
        let expiry = self.time_to_expiration();
        let params = BinomialPricingParams {
            asset: self.underlying_price,
            volatility: f2du!(self.implied_volatility).unwrap(),
            int_rate: f2du!(self.risk_free_rate).unwrap(),
            strike: self.strike_price,
            expiry: f2du!(expiry).unwrap(),
            no_steps,
            option_type: &self.option_type,
            option_style: &self.option_style,
            side: &self.side,
        };
        let (asset_tree, option_tree) = generate_binomial_tree(&params).unwrap();
        let price = match self.side {
            Side::Long => option_tree[0][0],
            Side::Short => -option_tree[0][0],
        };
        (price, asset_tree, option_tree)
    }

    pub fn calculate_price_black_scholes(&self) -> f64 {
        black_scholes(self)
    }

    pub fn calculate_price_telegraph(&self, no_steps: usize) -> Result<Decimal, DecimalError> {
        telegraph(self, no_steps, None, None)
    }

    pub fn payoff(&self) -> f64 {
        let payoff_info = PayoffInfo {
            spot: self.underlying_price,
            strike: self.strike_price,
            style: self.option_style.clone(),
            side: self.side.clone(),
            spot_prices: None,
            spot_min: None,
            spot_max: None,
        };
        self.option_type.payoff(&payoff_info) * self.quantity.to_f64()
    }

    pub fn payoff_at_price(&self, price: Positive) -> f64 {
        let payoff_info = PayoffInfo {
            spot: price,
            strike: self.strike_price,
            style: self.option_style.clone(),
            side: self.side.clone(),
            spot_prices: None,
            spot_min: None,
            spot_max: None,
        };
        self.option_type.payoff(&payoff_info) * self.quantity.to_f64()
    }

    pub fn intrinsic_value(&self, underlying_price: Positive) -> f64 {
        let payoff_info = PayoffInfo {
            spot: underlying_price,
            strike: self.strike_price,
            style: self.option_style.clone(),
            side: self.side.clone(),
            spot_prices: None,
            spot_min: None,
            spot_max: None,
        };
        self.option_type.payoff(&payoff_info) * self.quantity.to_f64()
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

    pub fn time_value(&self) -> f64 {
        let option_price = self.calculate_price_black_scholes().abs();
        let intrinsic_value = self.intrinsic_value(self.underlying_price);

        (option_price - intrinsic_value).max(ZERO)
    }

    pub(crate) fn validate(&self) -> bool {
        if self.underlying_symbol == *"" {
            error!("Underlying symbol is empty");
            return false;
        }
        if self.strike_price <= Positive::ZERO {
            debug!("Strike price is less than or equal to zero");
            return false;
        }
        if self.implied_volatility < ZERO {
            error!("Implied volatility is less than zero");
            return false;
        }
        if self.quantity == 0.0 {
            error!("Quantity is equal to zero");
            return false;
        }
        if self.underlying_price <= Positive::ZERO {
            error!("Underlying price is less than or equal to zero");
            return false;
        }
        if self.risk_free_rate < ZERO {
            error!("Risk free rate is less than zero");
            return false;
        }
        if self.dividend_yield < ZERO {
            error!("Dividend yield is less than zero");
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
            expiration_date: ExpirationDate::Days(0.0),
            implied_volatility: ZERO,
            quantity: Positive::ZERO,
            underlying_price: Positive::ZERO,
            risk_free_rate: ZERO,
            option_style: OptionStyle::Call,
            dividend_yield: ZERO,
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
    fn calculate_pnl(&self, _date_time: DateTime<Utc>, _market_price: Positive) -> PnL {
        todo!()
    }

    fn calculate_pnl_at_expiration(&self, _underlying_price: Option<Positive>) -> PnL {
        todo!()
    }
}

impl Profit for Options {
    fn calculate_profit_at(&self, price: Positive) -> f64 {
        self.payoff_at_price(price)
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
            .map(|&price| self.intrinsic_value(price))
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
    use crate::f2p;
    use chrono::{Duration, Utc};

    #[test]
    fn test_new_option() {
        let option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        assert_eq!(option.underlying_symbol, "AAPL");
        assert_eq!(option.strike_price, 100.0);
        assert_eq!(option.implied_volatility, 0.2);
    }

    #[test]
    fn test_time_to_expiration() {
        let option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        assert_eq!(option.time_to_expiration(), 30.0 / 365.0);

        let future_date = Utc::now() + Duration::days(60);
        let option_with_datetime = Options::new(
            OptionType::European,
            Side::Long,
            "AAPL".to_string(),
            f2p!(100.0),
            ExpirationDate::DateTime(future_date),
            0.2,
            Positive::ONE,
            f2p!(105.0),
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
        let long_option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        assert!(long_option.is_long());
        assert!(!long_option.is_short());

        let short_option = Options::new(
            OptionType::European,
            Side::Short,
            "AAPL".to_string(),
            f2p!(100.0),
            ExpirationDate::Days(30.0),
            0.2,
            Positive::ONE,
            f2p!(105.0),
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
        let option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        let price = option.calculate_price_binomial(100);
        assert!(price > Decimal::ZERO);
    }

    #[test]
    fn test_calculate_price_binomial_tree() {
        let option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        let (price, asset_tree, option_tree) = option.calculate_price_binomial_tree(5);
        assert!(price > Decimal::ZERO);
        assert_eq!(asset_tree.len(), 6);
        assert_eq!(option_tree.len(), 6);
    }

    #[test]
    fn test_calculate_price_binomial_tree_short() {
        let option = create_sample_option_simplest(OptionStyle::Call, Side::Short);
        let (price, asset_tree, option_tree) = option.calculate_price_binomial_tree(5);
        assert!(price > Decimal::ZERO);
        assert_eq!(asset_tree.len(), 6);
        assert_eq!(option_tree.len(), 6);
    }

    #[test]
    fn test_calculate_price_black_scholes() {
        let option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        let price = option.calculate_price_black_scholes();
        assert!(price > ZERO);
    }

    #[test]
    fn test_payoff_european_call_long() {
        let call_option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        let call_payoff = call_option.payoff();
        assert_eq!(call_payoff, 0.0); // max(100 - 100, 0) = 0

        let put_option = Options::new(
            OptionType::European,
            Side::Long,
            "AAPL".to_string(),
            f2p!(100.0),
            ExpirationDate::Days(30.0),
            0.2,
            Positive::ONE,
            f2p!(95.0),
            0.05,
            OptionStyle::Put,
            0.01,
            None,
        );
        let put_payoff = put_option.payoff();
        assert_eq!(put_payoff, 5.0); // max(100 - 95, 0) = 5
    }

    #[test]
    fn test_calculate_time_value() {
        let option = Options::new(
            OptionType::European,
            Side::Long,
            "AAPL".to_string(),
            f2p!(100.0),
            ExpirationDate::Days(30.0),
            0.2,
            Positive::ONE,
            f2p!(105.0),
            0.05,
            OptionStyle::Call,
            ZERO,
            None,
        );

        let time_value = option.time_value();
        assert!(time_value > ZERO);
        assert!(time_value < option.calculate_price_black_scholes());
    }
}

#[cfg(test)]
mod tests_valid_option {
    use super::*;
    use crate::f2p;

    fn create_valid_option() -> Options {
        Options {
            option_type: OptionType::European,
            side: Side::Long,
            underlying_symbol: "AAPL".to_string(),
            strike_price: f2p!(100.0),
            expiration_date: ExpirationDate::Days(30.0),
            implied_volatility: 0.2,
            quantity: Positive::ONE,
            underlying_price: f2p!(105.0),
            risk_free_rate: 0.05,
            option_style: OptionStyle::Call,
            dividend_yield: 0.01,
            exotic_params: None,
        }
    }

    #[test]
    fn test_valid_option() {
        let option = create_valid_option();
        assert!(option.validate());
    }

    #[test]
    fn test_empty_underlying_symbol() {
        let mut option = create_valid_option();
        option.underlying_symbol = "".to_string();
        assert!(!option.validate());
    }

    #[test]
    fn test_zero_strike_price() {
        let mut option = create_valid_option();
        option.strike_price = Positive::ZERO;
        assert!(!option.validate());
    }

    #[test]
    fn test_negative_strike_price() {
        let mut option = create_valid_option();

        // Isolate the potential panic-inducing operation outside the closure
        let result = std::panic::catch_unwind(|| {
            // We are only testing the invalid value creation here, not the assignment
            f2p!(-10.0);
        });

        assert!(
            result.is_err(),
            "Positive value must be positive, got -10"
        );

        // Proceed with assignment after the panic check
        if result.is_ok() {
            option.strike_price = f2p!(-10.0); // This line won't run due to expected panic
        }
    }

    #[test]
    fn test_negative_implied_volatility() {
        let mut option = create_valid_option();
        option.implied_volatility = -0.1;
        assert!(!option.validate());
    }

    #[test]
    fn test_zero_quantity() {
        let mut option = create_valid_option();
        option.quantity = Positive::ZERO;
        assert!(!option.validate());
    }

    #[test]
    fn test_zero_underlying_price() {
        let mut option = create_valid_option();
        option.underlying_price = Positive::ZERO;
        assert!(!option.validate());
    }

    #[test]
    fn test_negative_underlying_price() {
        let mut option = create_valid_option();

        // Isolate the potential panic-inducing operation outside the closure
        let result = std::panic::catch_unwind(|| {
            // We are only testing the invalid value creation here, not the assignment
            f2p!(-10.0);
        });

        assert!(
            result.is_err(),
            "Positive value must be positive, got -10"
        );

        // Proceed with assignment after the panic check
        if result.is_ok() {
            option.underlying_price = f2p!(-10.0); // This line won't run due to expected panic
        }
    }

    #[test]
    fn test_negative_risk_free_rate() {
        let mut option = create_valid_option();
        option.risk_free_rate = -0.01;
        assert!(!option.validate());
    }

    #[test]
    fn test_negative_dividend_yield() {
        let mut option = create_valid_option();
        option.dividend_yield = -0.01;
        assert!(!option.validate());
    }
}

#[cfg(test)]
mod tests_time_value {
    use super::*;
    use crate::model::utils::create_sample_option_simplest_strike;
    use crate::f2p;
    use crate::utils::logger::setup_logger;
    use approx::assert_relative_eq;
    use tracing::debug;

    #[test]
    fn test_calculate_time_value_long_call() {
        setup_logger();
        let option =
            create_sample_option_simplest_strike(Side::Long, OptionStyle::Call, f2p!(105.0));
        let time_value = option.time_value();
        assert!(time_value > ZERO);
        assert!(time_value <= option.calculate_price_black_scholes());
    }

    #[test]
    fn test_calculate_time_value_short_call() {
        let option =
            create_sample_option_simplest_strike(Side::Short, OptionStyle::Call, f2p!(105.0));
        let time_value = option.time_value();
        assert!(time_value > ZERO);
        assert!(time_value <= option.calculate_price_black_scholes().abs());
    }

    #[test]
    fn test_calculate_time_value_long_put() {
        setup_logger();
        let option = create_sample_option_simplest_strike(Side::Long, OptionStyle::Put, f2p!(95.0));
        let time_value = option.time_value();
        assert!(time_value > ZERO);
        assert!(time_value <= option.calculate_price_black_scholes());
    }

    #[test]
    fn test_calculate_time_value_short_put() {
        let option =
            create_sample_option_simplest_strike(Side::Short, OptionStyle::Put, f2p!(95.0));
        let time_value = option.time_value();
        assert!(time_value > ZERO);
        assert!(time_value <= option.calculate_price_black_scholes().abs());
    }

    #[test]
    fn test_calculate_time_value_at_the_money() {
        let call = create_sample_option_simplest_strike(Side::Long, OptionStyle::Call, f2p!(100.0));
        let put = create_sample_option_simplest_strike(Side::Long, OptionStyle::Put, f2p!(100.0));

        let call_time_value = call.time_value();
        let put_time_value = put.time_value();

        assert!(call_time_value > ZERO);
        assert!(put_time_value > ZERO);
        assert_eq!(call_time_value, call.calculate_price_black_scholes());
        assert_eq!(put_time_value, put.calculate_price_black_scholes());
    }

    #[test]
    fn test_calculate_time_value_deep_in_the_money() {
        setup_logger();
        let call = create_sample_option_simplest_strike(Side::Long, OptionStyle::Call, f2p!(150.0));
        let put = create_sample_option_simplest_strike(Side::Long, OptionStyle::Put, f2p!(50.0));

        let call_time_value = call.time_value();
        let put_time_value = put.time_value();

        assert_relative_eq!(
            call_time_value,
            call.calculate_price_black_scholes(),
            epsilon = 0.01
        );
        assert_relative_eq!(
            put_time_value,
            put.calculate_price_black_scholes(),
            epsilon = 0.01
        );
        debug!("Call time value: {}", call_time_value);
        debug!("Call BS price: {}", call.calculate_price_black_scholes());
        debug!("Put time value: {}", put_time_value);
        debug!("Put BS price: {}", put.calculate_price_black_scholes());
        assert!(call_time_value <= call.calculate_price_black_scholes());
        assert!(put_time_value <= put.calculate_price_black_scholes());
    }
}

#[cfg(test)]
mod tests_options_payoffs {
    use super::*;
    use crate::model::utils::create_sample_option_simplest_strike;
    use crate::f2p;
    use crate::utils::logger::setup_logger;

    #[test]
    fn test_payoff_european_call_long() {
        setup_logger();
        let call_option =
            create_sample_option_simplest_strike(Side::Long, OptionStyle::Call, f2p!(95.0));
        let call_payoff = call_option.payoff();
        assert_eq!(call_payoff, 5.0); // max(100 - 95, 0) = 5

        let call_option_otm =
            create_sample_option_simplest_strike(Side::Long, OptionStyle::Call, f2p!(105.0));
        let call_payoff_otm = call_option_otm.payoff();
        assert_eq!(call_payoff_otm, ZERO); // max(100 - 105, 0) = 0
    }

    #[test]
    fn test_payoff_european_call_short() {
        setup_logger();
        let call_option =
            create_sample_option_simplest_strike(Side::Short, OptionStyle::Call, f2p!(95.0));
        let call_payoff = call_option.payoff();
        assert_eq!(call_payoff, -5.0); // -max(100 - 95, 0) = -5

        let call_option_otm =
            create_sample_option_simplest_strike(Side::Short, OptionStyle::Call, f2p!(105.0));
        let call_payoff_otm = call_option_otm.payoff();
        assert_eq!(call_payoff_otm, ZERO); // -max(95 - 100, 0) = 0
    }

    #[test]
    fn test_payoff_european_put_long() {
        let put_option =
            create_sample_option_simplest_strike(Side::Long, OptionStyle::Put, f2p!(105.0));
        let put_payoff = put_option.payoff();
        assert_eq!(put_payoff, 5.0); // max(105 - 100, 0) = 5

        let put_option_otm =
            create_sample_option_simplest_strike(Side::Long, OptionStyle::Put, f2p!(95.0));
        let put_payoff_otm = put_option_otm.payoff();
        assert_eq!(put_payoff_otm, ZERO); // max(95 - 100, 0) = 0
    }

    #[test]
    fn test_payoff_european_put_short() {
        let put_option =
            create_sample_option_simplest_strike(Side::Short, OptionStyle::Put, f2p!(105.0));
        let put_payoff = put_option.payoff();
        assert_eq!(put_payoff, -5.0); // -max(105 - 100, 0) = -5

        let put_option_otm =
            create_sample_option_simplest_strike(Side::Short, OptionStyle::Put, f2p!(95.0));
        let put_payoff_otm = put_option_otm.payoff();
        assert_eq!(put_payoff_otm, ZERO); // -max(95 - 100, 0) = 0
    }
}

#[cfg(test)]
mod tests_options_payoff_at_price {
    use super::*;
    use crate::model::utils::create_sample_option_simplest;
    use crate::f2p;

    #[test]
    fn test_payoff_european_call_long() {
        let call_option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        let call_payoff = call_option.payoff_at_price(f2p!(105.0));
        assert_eq!(call_payoff, 5.0); // max(105 - 100, 0) = 5

        let call_option_otm = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        let call_payoff_otm = call_option_otm.payoff_at_price(f2p!(95.0));
        assert_eq!(call_payoff_otm, ZERO); // max(95 - 100, 0) = 0
    }

    #[test]
    fn test_payoff_european_call_short() {
        let call_option = create_sample_option_simplest(OptionStyle::Call, Side::Short);
        let call_payoff = call_option.payoff_at_price(f2p!(105.0));
        assert_eq!(call_payoff, -5.0); // -max(105 - 100, 0) = -5

        let call_option_otm = create_sample_option_simplest(OptionStyle::Call, Side::Short);
        let call_payoff_otm = call_option_otm.payoff_at_price(f2p!(95.0));
        assert_eq!(call_payoff_otm, ZERO); // -max(95 - 100, 0) = 0
    }

    #[test]
    fn test_payoff_european_put_long() {
        let put_option = create_sample_option_simplest(OptionStyle::Put, Side::Long);
        let put_payoff = put_option.payoff_at_price(f2p!(95.0));
        assert_eq!(put_payoff, 5.0); // max(100 - 95, 0) = 5

        let put_option_otm = create_sample_option_simplest(OptionStyle::Put, Side::Long);
        let put_payoff_otm = put_option_otm.payoff_at_price(f2p!(105.0));
        assert_eq!(put_payoff_otm, ZERO); // max(100 - 105, 0) = 0
    }

    #[test]
    fn test_payoff_european_put_short() {
        let put_option = create_sample_option_simplest(OptionStyle::Put, Side::Short);
        let put_payoff = put_option.payoff_at_price(f2p!(95.0));
        assert_eq!(put_payoff, -5.0); // -max(100 - 95, 0) = -5

        let put_option_otm = create_sample_option_simplest(OptionStyle::Put, Side::Short);
        let put_payoff_otm = put_option_otm.payoff_at_price(f2p!(105.0));
        assert_eq!(put_payoff_otm, ZERO); // -max(100 - 105, 0) = 0
    }
}

#[cfg(test)]
mod tests_options_payoffs_with_quantity {
    use super::*;
    use crate::model::utils::create_sample_option;
    use crate::f2p;

    #[test]
    fn test_payoff_call_long() {
        let option = create_sample_option(
            OptionStyle::Call,
            Side::Long,
            f2p!(105.0),
            f2p!(10.0),
            f2p!(100.0),
            0.02,
        );
        assert_eq!(option.payoff(), 50.0);

        let option_otm = create_sample_option(
            OptionStyle::Call,
            Side::Long,
            f2p!(95.0),
            f2p!(4.0),
            f2p!(100.0),
            0.02,
        );
        assert_eq!(option_otm.payoff(), ZERO);
    }

    #[test]
    fn test_payoff_call_short() {
        let option = create_sample_option(
            OptionStyle::Call,
            Side::Short,
            f2p!(105.0),
            f2p!(3.0),
            f2p!(100.0),
            0.02,
        );
        assert_eq!(option.payoff(), -15.0);

        let option_otm = create_sample_option(
            OptionStyle::Call,
            Side::Short,
            f2p!(95.0),
            f2p!(7.0),
            f2p!(100.0),
            0.02,
        );
        assert_eq!(option_otm.payoff(), ZERO);
    }

    #[test]
    fn test_payoff_put_long() {
        let option = create_sample_option(
            OptionStyle::Put,
            Side::Long,
            f2p!(95.0),
            f2p!(2.0),
            f2p!(100.0),
            0.02,
        );
        assert_eq!(option.payoff(), 10.0);

        let option_otm = create_sample_option(
            OptionStyle::Put,
            Side::Long,
            f2p!(105.0),
            f2p!(7.0),
            f2p!(100.0),
            0.02,
        );
        assert_eq!(option_otm.payoff(), ZERO);
    }

    #[test]
    fn test_payoff_put_short() {
        let option = create_sample_option(
            OptionStyle::Put,
            Side::Short,
            f2p!(95.0),
            f2p!(3.0),
            f2p!(100.0),
            0.02,
        );
        assert_eq!(option.payoff(), -15.0);

        let option_otm = create_sample_option(
            OptionStyle::Put,
            Side::Short,
            f2p!(105.0),
            f2p!(3.0),
            f2p!(100.0),
            0.02,
        );
        assert_eq!(option_otm.payoff(), ZERO);
    }

    #[test]
    fn test_payoff_with_quantity() {
        let option = create_sample_option(
            OptionStyle::Call,
            Side::Long,
            f2p!(110.0),
            f2p!(3.0),
            f2p!(100.0),
            0.02,
        );
        assert_eq!(option.payoff(), 30.0); // (110 - 100) * 3
    }

    #[test]
    fn test_intrinsic_value_call_long() {
        let option = create_sample_option(
            OptionStyle::Call,
            Side::Long,
            f2p!(100.0),
            f2p!(11.0),
            f2p!(100.0),
            0.02,
        );
        assert_eq!(option.intrinsic_value(f2p!(105.0)), 55.0);
        assert_eq!(option.intrinsic_value(f2p!(95.0)), ZERO);
    }

    #[test]
    fn test_intrinsic_value_call_short() {
        let option = create_sample_option(
            OptionStyle::Call,
            Side::Short,
            f2p!(100.0),
            f2p!(13.0),
            f2p!(100.0),
            0.02,
        );
        assert_eq!(option.intrinsic_value(f2p!(105.0)), -65.0);
        assert_eq!(option.intrinsic_value(f2p!(95.0)), ZERO);
    }

    #[test]
    fn test_intrinsic_value_put_long() {
        let option = create_sample_option(
            OptionStyle::Put,
            Side::Long,
            f2p!(100.0),
            f2p!(17.0),
            f2p!(100.0),
            0.02,
        );
        assert_eq!(option.intrinsic_value(f2p!(95.0)), 85.0);
        assert_eq!(option.intrinsic_value(f2p!(105.0)), ZERO);
    }

    #[test]
    fn test_intrinsic_value_put_short() {
        let option = create_sample_option(
            OptionStyle::Put,
            Side::Short,
            f2p!(100.0),
            f2p!(19.0),
            f2p!(100.0),
            0.02,
        );
        assert_eq!(option.intrinsic_value(f2p!(95.0)), -95.0);
        assert_eq!(option.intrinsic_value(f2p!(105.0)), ZERO);
    }

    #[test]
    fn test_intrinsic_value_with_quantity() {
        let option = create_sample_option(
            OptionStyle::Call,
            Side::Long,
            f2p!(100.0),
            f2p!(23.0),
            f2p!(100.0),
            0.02,
        );
        assert_eq!(option.intrinsic_value(f2p!(110.0)), 230.0); // (110 - 100) * 23
    }
}

#[cfg(test)]
mod tests_in_the_money {
    use super::*;
    use crate::model::utils::create_sample_option;
    use crate::f2p;

    #[test]
    fn test_call_in_the_money() {
        let mut option = create_sample_option(
            OptionStyle::Call,
            Side::Long,
            f2p!(110.0),
            f2p!(1.0),
            f2p!(110.0),
            0.02,
        );
        option.strike_price = f2p!(100.0);
        assert!(option.is_in_the_money());
    }

    #[test]
    fn test_call_at_the_money() {
        let mut option = create_sample_option(
            OptionStyle::Call,
            Side::Long,
            f2p!(100.0),
            f2p!(1.0),
            f2p!(110.0),
            0.02,
        );
        option.strike_price = f2p!(100.0);
        assert!(option.is_in_the_money());
    }

    #[test]
    fn test_call_out_of_the_money() {
        let mut option = create_sample_option(
            OptionStyle::Call,
            Side::Long,
            f2p!(90.0),
            f2p!(1.0),
            f2p!(110.0),
            0.02,
        );
        option.strike_price = f2p!(100.0);
        assert!(!option.is_in_the_money());
    }

    #[test]
    fn test_put_in_the_money() {
        let mut option = create_sample_option(
            OptionStyle::Put,
            Side::Long,
            f2p!(90.0),
            f2p!(1.0),
            f2p!(110.0),
            0.02,
        );
        option.strike_price = f2p!(100.0);
        assert!(option.is_in_the_money());
    }

    #[test]
    fn test_put_at_the_money() {
        let mut option = create_sample_option(
            OptionStyle::Put,
            Side::Long,
            f2p!(100.0),
            f2p!(1.0),
            f2p!(110.0),
            0.02,
        );
        option.strike_price = f2p!(100.0);
        assert!(option.is_in_the_money());
    }

    #[test]
    fn test_put_out_of_the_money() {
        let mut option = create_sample_option(
            OptionStyle::Put,
            Side::Long,
            f2p!(110.0),
            f2p!(1.0),
            f2p!(110.0),
            0.02,
        );
        option.strike_price = f2p!(100.0);
        assert!(!option.is_in_the_money());
    }
}

#[cfg(test)]
mod tests_greeks {
    use super::*;
    use crate::model::utils::create_sample_option_simplest;
    use crate::{assert_decimal_eq, f2p};
    use rust_decimal_macros::dec;

    const EPSILON: Decimal = dec!(1e-6);

    #[test]
    fn test_delta() {
        let delta = create_sample_option_simplest(OptionStyle::Call, Side::Long)
            .delta()
            .unwrap();
        assert_decimal_eq!(delta, dec!(0.539519922), EPSILON);
    }

    #[test]
    fn test_delta_size() {
        let mut option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        option.quantity = f2p!(2.0);
        assert_decimal_eq!(option.delta().unwrap(), dec!(1.0790398), EPSILON);
    }

    #[test]
    fn test_gamma() {
        let option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        assert_decimal_eq!(option.gamma().unwrap(), dec!(0.0691707), EPSILON);
    }

    #[test]
    fn test_gamma_size() {
        let mut option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        option.quantity = f2p!(2.0);
        assert_decimal_eq!(option.gamma().unwrap(), dec!(0.1383415), EPSILON);
    }

    #[test]
    fn test_theta() {
        let option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        assert_decimal_eq!(option.theta().unwrap(), dec!(-15.8697818), EPSILON);
    }

    #[test]
    fn test_theta_size() {
        let mut option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        option.quantity = f2p!(2.0);
        assert_decimal_eq!(option.theta().unwrap(), dec!(-31.739563), EPSILON);
    }

    #[test]
    fn test_vega() {
        let option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        assert_decimal_eq!(option.vega().unwrap(), dec!(15.4675554), EPSILON);
    }

    #[test]
    fn test_vega_size() {
        let mut option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        option.quantity = f2p!(2.0);
        assert_decimal_eq!(option.vega().unwrap(), dec!(30.9351108), EPSILON);
    }

    #[test]
    fn test_rho() {
        let option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        assert_decimal_eq!(option.rho().unwrap(), dec!(4.23312145), EPSILON);
    }

    #[test]
    fn test_rho_size() {
        let mut option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        option.quantity = f2p!(2.0);
        assert_decimal_eq!(option.rho().unwrap(), dec!(8.46624291), EPSILON);
    }

    #[test]
    fn test_rho_d() {
        let option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        assert_decimal_eq!(option.rho_d().unwrap(), dec!(-4.43441032), EPSILON);
    }

    #[test]
    fn test_rho_d_size() {
        let mut option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        option.quantity = f2p!(2.0);
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
    use crate::f2p;
    use crate::visualization::utils::Graph;
    use approx::assert_relative_eq;

    #[test]
    fn test_title() {
        let option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        let expected_title = "Underlying: AAPL @ $100 Long Call European Option".to_string();
        assert_eq!(option.title(), expected_title);
    }

    #[test]
    fn test_get_values() {
        let option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        let prices = vec![f2p!(90.0), f2p!(100.0), f2p!(110.0)];
        let values = option.get_values(&prices);

        assert_eq!(values.len(), 3);
        assert_relative_eq!(values[0], 0.0, epsilon = 1e-6);
        assert_relative_eq!(values[1], 0.0, epsilon = 1e-6);
        assert_relative_eq!(values[2], 10.0, epsilon = 1e-6);
    }

    #[test]
    fn test_get_vertical_lines() {
        let option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        let vertical_lines = option.get_vertical_lines();

        assert_eq!(vertical_lines.len(), 1);
        assert_eq!(vertical_lines[0].label, "Strike");
        assert_relative_eq!(vertical_lines[0].x_coordinate, 100.0, epsilon = 1e-6);
    }

    #[test]
    fn test_title_put_option() {
        let option = create_sample_option_simplest(OptionStyle::Put, Side::Long);
        let expected_title = "Underlying: AAPL @ $100 Long Put European Option".to_string();
        assert_eq!(option.title(), expected_title);
    }

    #[test]
    fn test_get_values_put_option() {
        let option = create_sample_option_simplest(OptionStyle::Put, Side::Long);
        let prices = vec![f2p!(90.0), f2p!(100.0), f2p!(110.0)];
        let values = option.get_values(&prices);

        assert_eq!(values.len(), 3);
        assert_relative_eq!(values[0], 10.0, epsilon = 1e-6);
        assert_relative_eq!(values[1], 0.0, epsilon = 1e-6);
        assert_relative_eq!(values[2], 0.0, epsilon = 1e-6);
    }

    #[test]
    fn test_get_values_short_option() {
        let option = create_sample_option_simplest(OptionStyle::Call, Side::Short);
        let prices = vec![f2p!(90.0), f2p!(100.0), f2p!(110.0)];
        let values = option.get_values(&prices);

        assert_eq!(values.len(), 3);
        assert_relative_eq!(values[0], 0.0, epsilon = 1e-6);
        assert_relative_eq!(values[1], 0.0, epsilon = 1e-6);
        assert_relative_eq!(values[2], -10.0, epsilon = 1e-6);
    }
}
