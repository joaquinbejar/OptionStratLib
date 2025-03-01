/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 2/10/24
******************************************************************************/
use crate::model::position::Position;
use crate::{Positive, pos};
use num_traits::ToPrimitive;
use rust_decimal::Decimal;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SPANMargin {
    scanning_range: f64,
    short_option_minimum: f64,
    price_scan_range: f64,
    volatility_scan_range: f64,
}

#[allow(dead_code)]
impl SPANMargin {
    pub fn new(
        scanning_range: f64,
        short_option_minimum: f64,
        price_scan_range: f64,
        volatility_scan_range: f64,
    ) -> Self {
        SPANMargin {
            scanning_range,
            short_option_minimum,
            price_scan_range,
            volatility_scan_range,
        }
    }

    pub fn calculate_margin(&self, position: &Position) -> f64 {
        let risk_array = self.calculate_risk_array(position);
        let short_option_minimum = self.calculate_short_option_minimum(position);

        risk_array
            .into_iter()
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap()
            .max(short_option_minimum)
    }

    fn calculate_risk_array(&self, position: &Position) -> Vec<f64> {
        let mut risk_array = Vec::new();
        let option = &position.option;

        let price_scenarios = self.generate_price_scenarios(option.underlying_price.into());
        let volatility_scenarios = self.generate_volatility_scenarios(option.implied_volatility);

        for &price in &price_scenarios {
            for &volatility in &volatility_scenarios {
                let scenario_loss = self.calculate_scenario_loss(position, price, volatility);
                risk_array.push(scenario_loss);
            }
        }

        risk_array
    }

    fn generate_price_scenarios(&self, underlying_price: f64) -> Vec<f64> {
        vec![
            underlying_price * (1.0 - self.price_scan_range),
            underlying_price,
            underlying_price * (1.0 + self.price_scan_range),
        ]
    }

    fn generate_volatility_scenarios(&self, implied_volatility: Positive) -> Vec<Positive> {
        vec![
            implied_volatility * (1.0 - self.volatility_scan_range),
            implied_volatility,
            implied_volatility * (1.0 + self.volatility_scan_range),
        ]
    }

    fn calculate_scenario_loss(
        &self,
        position: &Position,
        scenario_price: f64,
        scenario_volatility: Positive,
    ) -> f64 {
        let option = &position.option;
        let current_price = option.calculate_price_black_scholes().unwrap();

        let mut scenario_option = option.clone();
        scenario_option.underlying_price = pos!(scenario_price);
        scenario_option.implied_volatility = scenario_volatility;
        let scenario_price = scenario_option.calculate_price_black_scholes().unwrap();

        ((scenario_price - current_price)
            * option.quantity
            * if option.is_short() {
                Decimal::NEGATIVE_ONE
            } else {
                Decimal::ONE
            })
        .to_f64()
        .unwrap()
    }

    fn calculate_short_option_minimum(&self, position: &Position) -> f64 {
        let option = &position.option;
        if option.is_short() {
            self.short_option_minimum * option.underlying_price * option.quantity
        } else {
            0.0
        }
    }
}

#[cfg(test)]
mod tests_span {
    use super::*;
    use crate::model::types::{OptionStyle, Side};
    use crate::model::utils::create_sample_option;
    use crate::pos;
    use crate::utils::logger::setup_logger;
    use chrono::Utc;
    use tracing::info;

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_span_margin() {
        setup_logger();
        let option = create_sample_option(
            OptionStyle::Call,
            Side::Short,
            pos!(155.0),
            pos!(1.0),
            pos!(150.0),
            pos!(0.2),
        );

        let position = Position {
            option,
            premium: pos!(5.0),
            date: Utc::now(),
            open_fee: pos!(0.5),
            close_fee: pos!(0.5),
        };

        let span = SPANMargin::new(
            0.15, // scanning_range (15%)
            0.1,  // short_option_minimum (10%)
            0.05, // price_scan_range (5%)
            0.1,  // volatility_scan_range (10%)
        );

        let margin = span.calculate_margin(&position);
        assert!(margin > 0.0, "Margin should be positive");
        info!("Calculated margin: {}", margin);
    }
}
