/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 2/10/24
******************************************************************************/
use crate::Positive;
use crate::model::position::Position;
use rust_decimal::Decimal;

/// Represents parameters for calculating margin requirements using the
/// Standard Portfolio Analysis of Risk (SPAN) methodology.
///
/// This structure contains the key parameters needed to calculate margin requirements
/// for derivatives positions using the SPAN methodology developed by the Chicago
/// Mercantile Exchange. These parameters control the risk scenarios that will be
/// evaluated when determining potential portfolio losses.
///
/// The SPAN methodology evaluates positions under various market scenarios combining price
/// movements, volatility changes, and time decay effects to determine appropriate margin
/// requirements.
#[derive(Debug, Clone)]
pub struct SPANMargin {
    /// Minimum charge applied to short option positions, typically expressed as a
    /// percentage of the underlying asset value. This serves as a risk floor for
    /// short positions regardless of other factors.
    short_option_minimum: Decimal,

    /// The range of price movements to consider when generating price scenarios,
    /// usually expressed as a percentage. Determines how far up and down the
    /// underlying price might move in the risk analysis.
    price_scan_range: Decimal,

    /// The range of volatility changes to consider when generating volatility scenarios,
    /// usually expressed as a percentage. Controls how much implied volatility might
    /// increase or decrease in the risk analysis.
    volatility_scan_range: Decimal,
}

#[allow(dead_code)]
impl SPANMargin {
    /// Creates a new SPAN margin calculator with the specified risk parameters.
    ///
    /// This constructor initializes a SPANMargin structure with the key parameters needed
    /// for calculating margin requirements using the Standard Portfolio Analysis of Risk
    /// methodology.
    ///
    /// # Parameters
    ///
    /// * `short_option_minimum` - The minimum charge applied to short option positions,
    ///   typically expressed as a percentage of the underlying asset value.
    ///
    /// * `price_scan_range` - The range of price movements to consider when generating
    ///   price scenarios, expressed as a decimal percentage (e.g., 0.05 for 5%).
    ///   Determines how far up and down the underlying price might move in risk analysis.
    ///
    /// * `volatility_scan_range` - The range of volatility changes to consider when
    ///   generating volatility scenarios, expressed as a decimal percentage.
    ///   Controls potential implied volatility fluctuations in risk analysis.
    ///
    /// # Returns
    ///
    /// A new `SPANMargin` instance configured with the provided parameters.
    ///
    /// # Example
    ///
    /// ```
    /// use rust_decimal_macros::dec;
    /// use optionstratlib::risk::SPANMargin;
    ///
    /// let margin_calculator = SPANMargin::new(
    ///     dec!(0.05),  // 5% short option minimum
    ///     dec!(0.10),  // 10% price scan range
    ///     dec!(0.15)   // 15% volatility scan range
    /// );
    /// ```
    pub fn new(
        short_option_minimum: Decimal,
        price_scan_range: Decimal,
        volatility_scan_range: Decimal,
    ) -> Self {
        SPANMargin {
            short_option_minimum,
            price_scan_range,
            volatility_scan_range,
        }
    }

    /// Calculates the margin requirement for a given position based on SPAN methodology.
    ///
    /// This method determines the margin requirement by:
    /// 1. Calculating a risk array representing potential losses across different price and
    ///    volatility scenarios
    /// 2. Determining the minimum margin requirement for short options positions
    /// 3. Taking the maximum value between the highest potential loss and the short option minimum
    ///
    /// The margin requirement helps ensure traders maintain sufficient funds to cover potential
    /// losses in adverse market conditions.
    ///
    /// # Arguments
    /// * `position` - The option position for which to calculate margin requirements
    ///
    /// # Returns
    /// * `Decimal` - The calculated margin requirement for the position
    pub fn calculate_margin(&self, position: &Position) -> Decimal {
        let risk_array = self.calculate_risk_array(position);
        let short_option_minimum = self.calculate_short_option_minimum(position);
        risk_array
            .into_iter()
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap()
            .max(short_option_minimum)
    }

    /// Calculates a risk array for a given position using SPAN (Standard Portfolio Analysis of Risk) methodology.
    ///
    /// This function generates multiple price and volatility scenarios for the underlying asset and
    /// calculates potential losses for each scenario combination. The resulting vector contains loss
    /// values for all scenarios, which can be used for risk analysis and margin calculations.
    ///
    /// # Parameters
    /// - `&self`: Reference to the SPAN margin calculator instance.
    /// - `position`: Reference to the `Position` for which to calculate risk.
    ///
    /// # Returns
    /// A vector of `Decimal` values representing potential losses under different price and
    /// volatility scenarios. Each value corresponds to the theoretical loss in a specific scenario.
    ///
    /// # Algorithm
    /// The function:
    /// 1. Generates multiple price scenarios based on the underlying asset price.
    /// 2. Generates multiple volatility scenarios based on the option's implied volatility.
    /// 3. Creates a risk matrix by calculating the potential loss for each price-volatility
    ///    scenario combination.
    ///
    /// # Example Use Case
    /// This is typically used in risk management systems to determine the appropriate
    /// margin requirements for option positions.
    fn calculate_risk_array(&self, position: &Position) -> Vec<Decimal> {
        let mut risk_array = Vec::new();
        let option = &position.option;
        let price_scenarios = self.generate_price_scenarios(option.underlying_price);
        let volatility_scenarios = self.generate_volatility_scenarios(option.implied_volatility);
        for &price in &price_scenarios {
            for &volatility in &volatility_scenarios {
                let scenario_loss = self.calculate_scenario_loss(position, price, volatility);
                risk_array.push(scenario_loss);
            }
        }
        risk_array
    }

    /// Generates a vector of price scenarios for risk analysis based on the underlying asset price.
    ///
    /// This function creates three price scenarios for risk assessment:
    /// - Downside scenario: The underlying price decreased by the price scan range percentage
    /// - Base scenario: The current underlying price (unchanged)
    /// - Upside scenario: The underlying price increased by the price scan range percentage
    ///
    /// The scenarios are used in SPAN margin calculations to evaluate potential portfolio
    /// performance across different market conditions.
    ///
    /// # Arguments
    ///
    /// * `underlying_price` - The current price of the underlying asset as a `Positive` value
    ///
    /// # Returns
    ///
    /// A vector of three `Positive` values representing the price scenarios
    fn generate_price_scenarios(&self, underlying_price: Positive) -> Vec<Positive> {
        vec![
            underlying_price * (Decimal::ONE - self.price_scan_range),
            underlying_price,
            underlying_price * (Decimal::ONE + self.price_scan_range),
        ]
    }

    /// Generates a vector of implied volatility scenarios for risk analysis.
    ///
    /// This function creates three volatility scenarios for risk assessment:
    /// - Low volatility scenario: Current volatility decreased by the volatility scan range percentage
    /// - Base scenario: The current implied volatility (unchanged)
    /// - High volatility scenario: Current volatility increased by the volatility scan range percentage
    ///
    /// These scenarios are essential for evaluating how changes in market volatility might
    /// affect option prices and portfolio risk in the SPAN methodology.
    ///
    /// # Arguments
    ///
    /// * `implied_volatility` - The current implied volatility as a `Positive` value
    ///
    /// # Returns
    ///
    /// A vector of three `Positive` values representing the volatility scenarios
    fn generate_volatility_scenarios(&self, implied_volatility: Positive) -> Vec<Positive> {
        vec![
            implied_volatility * (Decimal::ONE - self.volatility_scan_range),
            implied_volatility,
            implied_volatility * (Decimal::ONE + self.volatility_scan_range),
        ]
    }

    /// Calculates the potential profit or loss for a position in a given price and volatility scenario.
    ///
    /// This function computes how the value of an option position would change under different
    /// market conditions by comparing the current option price with the theoretical price in the scenario.
    ///
    /// # Arguments
    /// * `position` - The option position to evaluate, containing the option details and position information
    /// * `scenario_price` - The hypothetical price of the underlying asset in the scenario
    /// * `scenario_volatility` - The hypothetical implied volatility level in the scenario
    ///
    /// # Returns
    /// A `Decimal` representing the profit (positive) or loss (negative) based on the scenario.
    /// For long positions, a higher scenario price results in positive returns.
    /// For short positions, the sign is flipped (losses when scenario price increases).
    ///
    fn calculate_scenario_loss(
        &self,
        position: &Position,
        scenario_price: Positive,
        scenario_volatility: Positive,
    ) -> Decimal {
        let option = &position.option;
        let current_price = option.calculate_price_black_scholes().unwrap();
        let mut scenario_option = option.clone();
        scenario_option.underlying_price = scenario_price;
        scenario_option.implied_volatility = scenario_volatility;
        let scenario_price = scenario_option.calculate_price_black_scholes().unwrap();
        (scenario_price - current_price)
            * option.quantity
            * if option.is_short() {
                Decimal::NEGATIVE_ONE
            } else {
                Decimal::ONE
            }
    }

    /// Calculates the minimum margin requirement for short option positions.
    ///
    /// This method implements part of the SPAN (Standard Portfolio Analysis of Risk) margin
    /// methodology by calculating the minimum margin requirement specifically for short option
    /// positions. Short options carry inherent risk that requires a baseline margin regardless
    /// of other factors.
    ///
    /// # Arguments
    /// * `position` - A reference to a `Position` containing the option details.
    ///
    /// # Returns
    /// * `Decimal` - The calculated minimum margin requirement for short option positions.
    ///   Returns zero for long positions as this minimum applies only to short positions.
    ///
    /// # Behavior
    /// For short options, the minimum margin is calculated as:
    /// `short_option_minimum * underlying_price * quantity`
    ///
    /// For long options, the function returns zero as the short option minimum doesn't apply.
    fn calculate_short_option_minimum(&self, position: &Position) -> Decimal {
        let option = &position.option;
        if option.is_short() {
            self.short_option_minimum * option.underlying_price * option.quantity
        } else {
            Decimal::ZERO
        }
    }
}

#[cfg(test)]
mod tests_span {
    use super::*;
    use crate::model::types::{OptionStyle, Side};
    use crate::model::utils::create_sample_option;
    use crate::pos;

    use chrono::Utc;
    use rust_decimal_macros::dec;
    use tracing::info;

    #[test]
    fn test_span_margin() {
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
            epic: Some("Epic123".to_string()),
            extra_fields: None,
        };

        let span = SPANMargin::new(
            dec!(0.1),  // short_option_minimum (10%)
            dec!(0.05), // price_scan_range (5%)
            dec!(0.1),  // volatility_scan_range (10%)
        );

        let margin = span.calculate_margin(&position);
        assert!(margin > Decimal::ZERO, "Margin should be positive");
        info!("Calculated margin: {}", margin);
    }
}
