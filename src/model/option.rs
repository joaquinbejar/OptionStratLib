use crate::constants::ZERO;
use crate::greeks::equations::{delta, gamma, rho, rho_d, theta, vega, Greek, Greeks};
use crate::model::types::{ExpirationDate, OptionStyle, OptionType, Side};
use crate::pnl::utils::{PnL, PnLCalculator};
use crate::pricing::binomial_model::{
    generate_binomial_tree, price_binomial, BinomialPricingParams,
};
use crate::pricing::black_scholes_model::black_scholes;
use crate::pricing::payoff::{Payoff, PayoffInfo};
use crate::pricing::telegraph::telegraph;
use crate::visualization::utils::Graph;
use chrono::{DateTime, Utc};
use plotters::prelude::*;
use std::error::Error;

#[derive(Clone, Default, Debug)]
pub struct ExoticParams {
    pub spot_prices: Option<Vec<f64>>, // Asian
    pub spot_min: Option<f64>,         // Lookback
    pub spot_max: Option<f64>,         // Lookback
}

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
    pub exotic_params: Option<ExoticParams>,
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

    pub fn calculate_price_telegraph(&self, no_steps: usize) -> f64 {
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
        self.option_type.payoff(&payoff_info) * self.quantity as f64
    }

    pub fn payoff_at_price(&self, price: f64) -> f64 {
        let payoff_info = PayoffInfo {
            spot: price,
            strike: self.strike_price,
            style: self.option_style.clone(),
            side: self.side.clone(),
            spot_prices: None,
            spot_min: None,
            spot_max: None,
        };
        self.option_type.payoff(&payoff_info) * self.quantity as f64
    }

    pub fn intrinsic_value(&self, underlying_price: f64) -> f64 {
        let payoff_info = PayoffInfo {
            spot: underlying_price,
            strike: self.strike_price,
            style: self.option_style.clone(),
            side: self.side.clone(),
            spot_prices: None,
            spot_min: None,
            spot_max: None,
        };
        self.option_type.payoff(&payoff_info) * self.quantity as f64
    }

    pub fn delta(&self) -> f64 {
        delta(self)
    }

    pub fn gamma(&self) -> f64 {
        gamma(self)
    }

    pub fn theta(&self) -> f64 {
        theta(self)
    }

    pub fn vega(&self) -> f64 {
        vega(self)
    }

    pub fn rho(&self) -> f64 {
        rho(self)
    }

    pub fn rho_d(&self) -> f64 {
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
}

impl Default for Options {
    fn default() -> Self {
        Options {
            option_type: OptionType::European,
            side: Side::Long,
            underlying_symbol: "".to_string(),
            strike_price: ZERO,
            expiration_date: ExpirationDate::Days(0.0),
            implied_volatility: ZERO,
            quantity: 0,
            underlying_price: ZERO,
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
            delta: self.delta(),
            gamma: self.gamma(),
            theta: self.theta(),
            vega: self.vega(),
            rho: self.rho(),
            rho_d: self.rho_d(),
        }
    }
}

impl PnLCalculator for Options {
    fn calculate_pnl(&self, _date_time: DateTime<Utc>, _market_price: f64) -> PnL {
        todo!()
    }

    fn calculate_pnl_at_expiration(&self, _underlying_price: Option<f64>) -> PnL {
        todo!()
    }
}

impl Graph for Options {
    fn graph(&self, data: &[f64], file_path: &str) -> Result<(), Box<dyn Error>> {
        // Generate intrinsic value for each price in the data vector
        let intrinsic_values: Vec<f64> = data
            .iter()
            .map(|&price| self.intrinsic_value(price))
            .collect();

        let dark_red = RGBColor(220, 0, 0);

        // Set up the drawing area with a 1200x800 pixel canvas
        let root = BitMapBackend::new(file_path, (1200, 800)).into_drawing_area();
        root.fill(&WHITE)?;

        // Determine the range for the X and Y axes
        let max_price = data.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let min_price = data.iter().cloned().fold(f64::INFINITY, f64::min);

        let max_intrinsic_values = intrinsic_values
            .iter()
            .cloned()
            .fold(f64::NEG_INFINITY, f64::max);
        let min_intrinsic_values = intrinsic_values
            .iter()
            .cloned()
            .fold(f64::INFINITY, f64::min);
        let adjusted_max = (max_intrinsic_values * 1.2 - max_intrinsic_values).abs();
        let adjusted_min = (min_intrinsic_values * 1.2 - min_intrinsic_values).abs();
        let margin_value = std::cmp::max(adjusted_max as i64, adjusted_min as i64);
        let max_intrinsic_value = max_intrinsic_values + margin_value as f64;
        let min_intrinsic_value = min_intrinsic_values - margin_value as f64;

        let title: String = self.title();

        // Build the chart with specified margins and label sizes
        let mut chart = ChartBuilder::on(&root)
            .caption(title, ("sans-serif", 30))
            .margin(10)
            .top_x_label_area_size(40)
            .x_label_area_size(40)
            .y_label_area_size(60)
            .right_y_label_area_size(60)
            .build_cartesian_2d(
                min_price..max_price,
                min_intrinsic_value..max_intrinsic_value,
            )?;

        // Configure and draw the mesh grid
        chart.configure_mesh().x_labels(20).y_labels(20).draw()?;

        // Draw a horizontal line at y = 0 to indicate break-even
        chart.draw_series(LineSeries::new(
            vec![(min_price, 0.0), (max_price, 0.0)],
            &BLACK,
        ))?;

        // Draw a vertical line at the break-even price
        chart.draw_series(LineSeries::new(
            vec![
                (self.strike_price, min_intrinsic_value),
                (self.strike_price, max_intrinsic_value),
            ],
            &BLACK,
        ))?;

        let break_even_label_position = match self.side {
            Side::Long => (10, 30),
            Side::Short => (10, 600),
        };

        // Add a label at the top of the break-even line
        chart.draw_series(PointSeries::of_element(
            vec![(self.strike_price, max_intrinsic_value)],
            5,
            &BLACK,
            &|coord, _size, _style| {
                EmptyElement::at(coord)
                    + Text::new(
                        format!("Strike: {:.2}", self.strike_price),
                        break_even_label_position, // Position the text just above the top of the line
                        ("sans-serif", 15).into_font(),
                    )
            },
        ))?;

        // Draw the line series representing intrinsic values
        chart.draw_series(LineSeries::new(
            data.iter().cloned().zip(intrinsic_values.iter().cloned()),
            &dark_red,
        ))?;

        // Draw points on the graph with labels for the intrinsic values
        for (i, (&price, &value)) in data.iter().zip(intrinsic_values.iter()).enumerate() {
            if i % 10 == 0 && value != 0.0 {
                chart.draw_series(PointSeries::of_element(
                    vec![(price, value)],
                    3,
                    &dark_red,
                    &|coord, size, style| {
                        EmptyElement::at(coord)
                            + Circle::new((0, 0), size, style.filled())
                            + Text::new(
                                format!("{:.2}", value),
                                (20, 0),
                                ("sans-serif", 15).into_font(),
                            )
                    },
                ))?;
            } else {
                chart.draw_series(PointSeries::of_element(
                    vec![(price, value)],
                    0,
                    &dark_red,
                    &|coord, size, style| {
                        EmptyElement::at(coord) + Circle::new((0, 0), size, style.filled())
                    },
                ))?;
            }
        }

        // Finalize and render the chart
        root.present()?;
        Ok(())
    }

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
            ZERO,
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
        assert!(price > ZERO);
    }

    #[test]
    fn test_calculate_price_binomial_tree() {
        let option = create_sample_option();
        let (price, asset_tree, option_tree) = option.calculate_price_binomial_tree(5);
        assert!(price > ZERO);
        assert_eq!(asset_tree.len(), 6);
        assert_eq!(option_tree.len(), 6);
    }

    #[test]
    fn test_calculate_price_black_scholes() {
        let option = create_sample_option();
        let price = option.calculate_price_black_scholes();
        assert!(price > ZERO);
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

    #[test]
    fn test_calculate_time_value() {
        let option = Options::new(
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
            ZERO,
            None,
        );

        let time_value = option.time_value();
        assert!(time_value > ZERO);
        assert!(time_value < option.calculate_price_black_scholes());
    }
}

#[cfg(test)]
mod tests_time_value {
    use super::*;

    fn create_option(side: Side, option_style: OptionStyle, underlying_price: f64) -> Options {
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
            ZERO,
            None,
        )
    }

    #[test]
    fn test_calculate_time_value_long_call() {
        let option = create_option(Side::Long, OptionStyle::Call, 105.0);
        let time_value = option.time_value();
        assert!(time_value > ZERO);
        assert!(time_value < option.calculate_price_black_scholes());
    }

    // #[test]
    // fn test_calculate_time_value_short_call() {
    //     let option = create_option(Side::Short, OptionStyle::Call, 105.0);
    //     let time_value = option.time_value();
    //     assert!(time_value > ZERO);
    //     assert!(time_value < option.calculate_price_black_scholes().abs());
    // }

    #[test]
    fn test_calculate_time_value_long_put() {
        let option = create_option(Side::Long, OptionStyle::Put, 95.0);
        let time_value = option.time_value();
        assert!(time_value > ZERO);
        assert!(time_value < option.calculate_price_black_scholes());
    }

    // #[test]
    // fn test_calculate_time_value_short_put() {
    //     let option = create_option(Side::Short, OptionStyle::Put, 95.0);
    //     let time_value = option.time_value();
    //     assert!(time_value > ZERO);
    //     assert!(time_value < option.calculate_price_black_scholes().abs());
    // }

    #[test]
    fn test_calculate_time_value_at_the_money() {
        let call = create_option(Side::Long, OptionStyle::Call, 100.0);
        let put = create_option(Side::Long, OptionStyle::Put, 100.0);

        let call_time_value = call.time_value();
        let put_time_value = put.time_value();

        assert!(call_time_value > ZERO);
        assert!(put_time_value > ZERO);
        assert_eq!(call_time_value, call.calculate_price_black_scholes());
        assert_eq!(put_time_value, put.calculate_price_black_scholes());
    }

    // #[test]
    // fn test_calculate_time_value_deep_in_the_money() {
    //     let call = create_option(Side::Long, OptionStyle::Call, 150.0);
    //     let put = create_option(Side::Long, OptionStyle::Put, 50.0);
    //
    //     let call_time_value = call.time_value();
    //     let put_time_value = put.time_value();
    //
    //     assert!(call_time_value > ZERO);
    //     // assert!(put_time_value > ZERO);
    //     assert!(call_time_value < call.calculate_price_black_scholes() - 50.0);
    //     assert!(put_time_value < put.calculate_price_black_scholes() - 50.0);
    // }
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
        assert_eq!(call_payoff_otm, ZERO); // max(95 - 100, 0) = 0
    }

    #[test]
    fn test_payoff_european_call_short() {
        let call_option = create_sample_option(OptionStyle::Call, Side::Short, 105.0);
        let call_payoff = call_option.payoff();
        assert_eq!(call_payoff, -5.0); // -max(105 - 100, 0) = -5

        let call_option_otm = create_sample_option(OptionStyle::Call, Side::Short, 95.0);
        let call_payoff_otm = call_option_otm.payoff();
        assert_eq!(call_payoff_otm, ZERO); // -max(95 - 100, 0) = 0
    }

    #[test]
    fn test_payoff_european_put_long() {
        let put_option = create_sample_option(OptionStyle::Put, Side::Long, 95.0);
        let put_payoff = put_option.payoff();
        assert_eq!(put_payoff, 5.0); // max(100 - 95, 0) = 5

        let put_option_otm = create_sample_option(OptionStyle::Put, Side::Long, 105.0);
        let put_payoff_otm = put_option_otm.payoff();
        assert_eq!(put_payoff_otm, ZERO); // max(100 - 105, 0) = 0
    }

    #[test]
    fn test_payoff_european_put_short() {
        let put_option = create_sample_option(OptionStyle::Put, Side::Short, 95.0);
        let put_payoff = put_option.payoff();
        assert_eq!(put_payoff, -5.0); // -max(100 - 95, 0) = -5

        let put_option_otm = create_sample_option(OptionStyle::Put, Side::Short, 105.0);
        let put_payoff_otm = put_option_otm.payoff();
        assert_eq!(put_payoff_otm, ZERO); // -max(100 - 105, 0) = 0
    }
}

#[cfg(test)]
mod tests_options_payoff_at_price {
    use super::*;
    use crate::model::utils::create_sample_option_simplest;

    #[test]
    fn test_payoff_european_call_long() {
        let call_option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        let call_payoff = call_option.payoff_at_price(105.0);
        assert_eq!(call_payoff, 5.0); // max(105 - 100, 0) = 5

        let call_option_otm = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        let call_payoff_otm = call_option_otm.payoff_at_price(95.0);
        assert_eq!(call_payoff_otm, ZERO); // max(95 - 100, 0) = 0
    }

    #[test]
    fn test_payoff_european_call_short() {
        let call_option = create_sample_option_simplest(OptionStyle::Call, Side::Short);
        let call_payoff = call_option.payoff_at_price(105.0);
        assert_eq!(call_payoff, -5.0); // -max(105 - 100, 0) = -5

        let call_option_otm = create_sample_option_simplest(OptionStyle::Call, Side::Short);
        let call_payoff_otm = call_option_otm.payoff_at_price(95.0);
        assert_eq!(call_payoff_otm, ZERO); // -max(95 - 100, 0) = 0
    }

    #[test]
    fn test_payoff_european_put_long() {
        let put_option = create_sample_option_simplest(OptionStyle::Put, Side::Long);
        let put_payoff = put_option.payoff_at_price(95.0);
        assert_eq!(put_payoff, 5.0); // max(100 - 95, 0) = 5

        let put_option_otm = create_sample_option_simplest(OptionStyle::Put, Side::Long);
        let put_payoff_otm = put_option_otm.payoff_at_price(105.0);
        assert_eq!(put_payoff_otm, ZERO); // max(100 - 105, 0) = 0
    }

    #[test]
    fn test_payoff_european_put_short() {
        let put_option = create_sample_option_simplest(OptionStyle::Put, Side::Short);
        let put_payoff = put_option.payoff_at_price(95.0);
        assert_eq!(put_payoff, -5.0); // -max(100 - 95, 0) = -5

        let put_option_otm = create_sample_option_simplest(OptionStyle::Put, Side::Short);
        let put_payoff_otm = put_option_otm.payoff_at_price(105.0);
        assert_eq!(put_payoff_otm, ZERO); // -max(100 - 105, 0) = 0
    }
}

#[cfg(test)]
mod tests_options_payoffs_with_quantity {
    use super::*;
    use crate::model::utils::create_sample_option;

    #[test]
    fn test_payoff_call_long() {
        let option = create_sample_option(OptionStyle::Call, Side::Long, 105.0, 10);
        assert_eq!(option.payoff(), 50.0);

        let option_otm = create_sample_option(OptionStyle::Call, Side::Long, 95.0, 4);
        assert_eq!(option_otm.payoff(), ZERO);
    }

    #[test]
    fn test_payoff_call_short() {
        let option = create_sample_option(OptionStyle::Call, Side::Short, 105.0, 3);
        assert_eq!(option.payoff(), -15.0);

        let option_otm = create_sample_option(OptionStyle::Call, Side::Short, 95.0, 7);
        assert_eq!(option_otm.payoff(), ZERO);
    }

    #[test]
    fn test_payoff_put_long() {
        let option = create_sample_option(OptionStyle::Put, Side::Long, 95.0, 2);
        assert_eq!(option.payoff(), 10.0);

        let option_otm = create_sample_option(OptionStyle::Put, Side::Long, 105.0, 7);
        assert_eq!(option_otm.payoff(), ZERO);
    }

    #[test]
    fn test_payoff_put_short() {
        let option = create_sample_option(OptionStyle::Put, Side::Short, 95.0, 3);
        assert_eq!(option.payoff(), -15.0);

        let option_otm = create_sample_option(OptionStyle::Put, Side::Short, 105.0, 3);
        assert_eq!(option_otm.payoff(), ZERO);
    }

    #[test]
    fn test_payoff_with_quantity() {
        let option = create_sample_option(OptionStyle::Call, Side::Long, 110.0, 3);
        assert_eq!(option.payoff(), 30.0); // (110 - 100) * 3
    }

    #[test]
    fn test_intrinsic_value_call_long() {
        let option = create_sample_option(OptionStyle::Call, Side::Long, 100.0, 11);
        assert_eq!(option.intrinsic_value(105.0), 55.0);
        assert_eq!(option.intrinsic_value(95.0), ZERO);
    }

    #[test]
    fn test_intrinsic_value_call_short() {
        let option = create_sample_option(OptionStyle::Call, Side::Short, 100.0, 13);
        assert_eq!(option.intrinsic_value(105.0), -65.0);
        assert_eq!(option.intrinsic_value(95.0), ZERO);
    }

    #[test]
    fn test_intrinsic_value_put_long() {
        let option = create_sample_option(OptionStyle::Put, Side::Long, 100.0, 17);
        assert_eq!(option.intrinsic_value(95.0), 85.0);
        assert_eq!(option.intrinsic_value(105.0), ZERO);
    }

    #[test]
    fn test_intrinsic_value_put_short() {
        let option = create_sample_option(OptionStyle::Put, Side::Short, 100.0, 19);
        assert_eq!(option.intrinsic_value(95.0), -95.0);
        assert_eq!(option.intrinsic_value(105.0), ZERO);
    }

    #[test]
    fn test_intrinsic_value_with_quantity() {
        let option = create_sample_option(OptionStyle::Call, Side::Long, 100.0, 23);
        assert_eq!(option.intrinsic_value(110.0), 230.0); // (110 - 100) * 23
    }
}
