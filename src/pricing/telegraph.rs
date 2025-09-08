/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 19/8/24
******************************************************************************/
//! # Telegraph Process
//!
//! A Telegraph Process (also known as a two-state process) is a stochastic process
//! that alternates between two states, typically represented as +1 and -1.
//!
//! ## Key Parameters
//!
//! - `lambda_up`: Transition rate from state -1 to +1
//! - `lambda_down`: Transition rate from state +1 to -1
//!
//! These parameters are always positive (λ_up, λ_down > 0) and typically range
//! from 0 to 10 in practice.
//!
//! ## Algorithm
//!
//! 1. The process starts in one of the two states (+1 or -1), usually chosen randomly.
//!
//! 2. At each time step dt:
//!    - If the current state is +1, there's a probability of changing to -1.
//!    - If the current state is -1, there's a probability of changing to +1.
//!
//! 3. The probability of change in an interval dt is calculated as:
//!    P(change) = 1 - e^(-λ * dt)
//!    Where λ is λ_up if the current state is -1, or λ_down if the current state is +1.
//!
//! ## Parameter Interpretation
//!
//! - Higher values indicate more frequent changes between states.
//! - Lower values indicate that the process tends to remain in a state for longer.
//!
//! Typical value ranges:
//! - Infrequent changes: 0.1 to 1
//! - Moderate changes: 1 to 5
//! - Very frequent changes: 5 to 10
//!
//! ## Relationship Between Parameters
//!
//! - If λ_up = λ_down, the process is symmetric.
//! - If λ_up > λ_down, the process tends to spend more time in the +1 state.
//! - If λ_up < λ_down, the process tends to spend more time in the -1 state.
//!
//! ## Use in Financial Modeling
//!
//! In the context of financial options, the Telegraph Process can be used to model:
//! - Changes in volatility (high/low volatility regime)
//! - Changes in market direction (bullish/bearish trend)
//! - Changes in interest rates (high/low)
//!
//! ## Parameter Estimation
//!
//! Parameters can be estimated from historical data:
//! 1. Classify historical periods into +1 and -1 states based on a threshold.
//! 2. Calculate the average duration of each state.
//! 3. Estimate λ_up as 1 / (average duration of -1 state).
//! 4. Estimate λ_down as 1 / (average duration of +1 state).
//!
//! ## Advantages
//!
//! - Allows modeling of abrupt changes in the market.
//! - Captures "regime change" behaviors that continuous models can't easily represent.
//! - Relatively simple to implement and understand.
//!
//! ## Considerations
//!
//! - The choice of λ_up and λ_down significantly affects the model's behavior.
//! - These parameters may need to be calibrated with historical or market data.
//! - In more advanced models, λ_up and λ_down could be dynamically adjusted based on changing market conditions.
//!
//! Remember that the choice of these parameters depends heavily on the specific asset
//! being modeled and the time horizon of your analysis. It's common to experiment with
//! different values and validate results against real data to find the best configuration
//! for your specific model.

use crate::Options;
use crate::error::decimal::DecimalError;
use crate::pricing::utils::simulate_returns;
use num_traits::{FromPrimitive, ToPrimitive};
use rand::random;
use rust_decimal::{Decimal, MathematicalOps};
use rust_decimal_macros::dec;
use std::error::Error;

/// Represents a Telegraph Process, a two-state continuous-time Markov chain model
/// used to simulate stochastic processes with discrete state transitions.
///
/// The Telegraph Process alternates between two states (-1 and +1), modeling regime
/// switches or market sentiment changes. State transitions are governed by Poisson
/// processes with specified rates. This model is particularly useful for simulating
/// financial markets that exhibit distinct behavioral regimes.
///
/// # Applications
///
/// - Modeling regime changes in market volatility
/// - Simulating discrete sentiment shifts in financial markets
/// - Representing asymmetric transition behaviors in stochastic systems
/// - Pricing financial derivatives under regime-switching assumptions
///
#[derive(Clone)]
pub struct TelegraphProcess {
    /// Transition rate from state -1 to +1, representing the intensity
    /// of the underlying Poisson process for upward state changes
    lambda_up: Decimal,

    /// Transition rate from state +1 to -1, representing the intensity
    /// of the underlying Poisson process for downward state changes
    lambda_down: Decimal,

    /// Current state of the process, which can be either -1 or +1,
    /// representing the two possible regimes of the system
    current_state: i8,
}

impl TelegraphProcess {
    /// Creates a new TelegraphProcess with the given transition rates.
    ///
    /// # Arguments
    ///
    /// * `lambda_up` - Transition rate from state -1 to +1
    /// * `lambda_down` - Transition rate from state +1 to -1
    ///
    /// # Returns
    ///
    /// A new TelegraphProcess with a randomly chosen initial state.
    pub fn new(lambda_up: Decimal, lambda_down: Decimal) -> Self {
        let initial_state = if random::<f64>() < 0.5 { 1 } else { -1 };
        TelegraphProcess {
            lambda_up,
            lambda_down,
            current_state: initial_state,
        }
    }

    /// Calculates the next state of the process.
    ///
    /// # Arguments
    ///
    /// * `dt` - Time step
    ///
    /// # Returns
    ///
    /// The new state of the process (-1 or 1)
    pub fn next_state(&mut self, dt: Decimal) -> i8 {
        let lambda = if self.current_state == 1 {
            self.lambda_down
        } else {
            self.lambda_up
        };
        let lambda_dt = -lambda * dt;
        let probability = if lambda_dt < dec!(11.7) {
            Decimal::ONE
        } else {
            Decimal::ONE - lambda_dt.exp()
        };

        if random::<f64>() < probability.to_f64().unwrap() {
            self.current_state *= -1;
        }

        self.current_state
    }

    /// Returns the current state of the process.
    ///
    /// # Returns
    ///
    /// The current state (-1 or 1)
    pub fn get_current_state(&self) -> i8 {
        self.current_state
    }
}

/// Estimates the Telegraph Process parameters from historical data.
///
/// # Arguments
///
/// * `returns` - A slice of historical returns
/// * `threshold` - The threshold used to classify states
///
/// # Description
///
/// This method updates the `lambda_up` and `lambda_down` parameters of the process
/// based on the provided historical data. It classifies each return as belonging to
/// state +1 or -1 based on the threshold, then calculates the average duration of
/// each state to estimate the transition rates.
pub(crate) fn estimate_telegraph_parameters(
    returns: &[Decimal],
    threshold: Decimal,
) -> Result<(Decimal, Decimal), DecimalError> {
    // Allow threshold to be zero - it's a valid threshold for classification
    // Returns are classified as +1 if > threshold, -1 if <= threshold
    let mut current_state = if returns[0] > threshold {
        Decimal::ONE
    } else {
        Decimal::NEGATIVE_ONE
    };
    let mut current_duration = Decimal::ONE;
    let mut up_durations = Vec::new();
    let mut down_durations = Vec::new();

    for &ret in returns.iter().skip(1) {
        let new_state = if ret > threshold {
            Decimal::ONE
        } else {
            Decimal::NEGATIVE_ONE
        };
        if new_state == current_state {
            current_duration += Decimal::ONE;
        } else {
            if current_state == Decimal::ONE {
                up_durations.push(current_duration);
            } else {
                down_durations.push(current_duration);
            }
            current_state = new_state;
            current_duration = Decimal::ONE;
        }
    }

    if current_state == Decimal::ONE {
        up_durations.push(current_duration);
    } else {
        down_durations.push(current_duration);
    }

    // Check if we have transitions in both directions
    if down_durations.is_empty() {
        return Err(DecimalError::InvalidValue {
            value: 0.0,
            reason: "No transitions from state +1 to -1 found. All returns are above threshold."
                .to_string(),
        });
    }

    if up_durations.is_empty() {
        return Err(DecimalError::InvalidValue {
            value: 0.0,
            reason: "No transitions from state -1 to +1 found. All returns are below threshold."
                .to_string(),
        });
    }

    let sum_down = down_durations.iter().sum::<Decimal>();
    let sum_up = up_durations.iter().sum::<Decimal>();

    if sum_down == Decimal::ZERO {
        return Err(DecimalError::InvalidValue {
            value: sum_down.to_f64().unwrap(),
            reason: "Sum of down durations must be non-zero".to_string(),
        });
    }

    if sum_up == Decimal::ZERO {
        return Err(DecimalError::InvalidValue {
            value: sum_up.to_f64().unwrap(),
            reason: "Sum of up durations must be non-zero".to_string(),
        });
    }

    let lambda_up = Decimal::ONE / sum_down * Decimal::from_usize(down_durations.len()).unwrap();
    let lambda_down = Decimal::ONE / sum_up * Decimal::from_usize(up_durations.len()).unwrap();
    Ok((lambda_up, lambda_down))
}

/// Prices an option using the Telegraph process simulation method.
///
/// This function simulates the underlying asset price movement using a Telegraph process,
/// which oscillates between two states, affecting the volatility of the price path.
/// It provides a more sophisticated model than standard geometric Brownian motion by
/// capturing regime-switching behavior in market volatility.
///
/// # Arguments
///
/// * `option` - Reference to the Options structure containing all option parameters
/// * `no_steps` - Number of time steps for the simulation
/// * `lambda_up` - Optional transition rate from down state (-1) to up state (+1)
/// * `lambda_down` - Optional transition rate from up state (+1) to down state (-1)
///
/// # Returns
///
/// * `Result<Decimal, Box<dyn Error>>` - The simulated option price or an error
///
/// # Details
///
/// The function handles parameter estimation automatically if transition rates are not provided.
/// When missing, it simulates returns based on the option's implied volatility to estimate
/// appropriate telegraph parameters.
pub fn telegraph(
    option: &Options,
    no_steps: usize,
    lambda_up: Option<Decimal>,
    lambda_down: Option<Decimal>,
) -> Result<Decimal, Box<dyn Error>> {
    let mut price = option.underlying_price;
    let dt = option.time_to_expiration()?.to_dec() / Decimal::from_f64(no_steps as f64).unwrap();

    let one_over_252 = Decimal::from_f64(1.0 / 252.0).unwrap();

    let (lambda_up_temp, lambda_down_temp) = match (lambda_up, lambda_down) {
        (None, None) => {
            let returns =
                simulate_returns(Decimal::ZERO, option.implied_volatility, 100, one_over_252)?;
            estimate_telegraph_parameters(&returns, Decimal::ZERO)?
        }
        (Some(l_up), None) => {
            let returns =
                simulate_returns(Decimal::ZERO, option.implied_volatility, 100, one_over_252)?;
            let (_, l_down) = estimate_telegraph_parameters(&returns, Decimal::ZERO)?;
            (l_up, l_down)
        }
        (None, Some(l_down)) => {
            let returns =
                simulate_returns(Decimal::ZERO, option.implied_volatility, 100, one_over_252)?;
            let (l_up, _) = estimate_telegraph_parameters(&returns, Decimal::ZERO)?;
            (l_up, l_down)
        }
        (Some(l_up), Some(l_down)) => (l_up, l_down),
    };
    let telegraph_process = TelegraphProcess::new(lambda_up_temp, lambda_down_temp);

    let tp = telegraph_process;
    let mut telegraph_process = tp.clone();
    for _ in 0..no_steps {
        let state = telegraph_process.next_state(dt);
        let drift: Decimal = option.risk_free_rate - dec!(0.5) * option.implied_volatility.powi(2);
        let volatility: Decimal =
            option.implied_volatility.to_dec() * Decimal::from_f64(state as f64).unwrap();

        let rh = Decimal::from_f64(dt.sqrt().unwrap().to_f64().unwrap() * random::<f64>()).unwrap();
        let lhs = drift * dt + volatility;

        let update = (lhs * rh).exp();
        price *= update;
    }

    let payoff = option.payoff_at_price(&price)?;
    let result = payoff * (-option.risk_free_rate * option.time_to_expiration()?).exp();
    Ok(result)
}

#[cfg(test)]
mod tests_telegraph_process_basis {
    use super::*;
    use crate::model::types::{OptionStyle, OptionType, Side};
    use crate::{Positive, pos};
    use rust_decimal_macros::dec;

    #[test]
    fn test_telegraph_process_new() {
        let tp = TelegraphProcess::new(dec!(0.5), dec!(0.3));
        assert_eq!(tp.lambda_up, dec!(0.5));
        assert_eq!(tp.lambda_down, dec!(0.3));
        assert!(tp.current_state == 1 || tp.current_state == -1);
    }

    #[test]
    fn test_telegraph_process_next_state() {
        let mut tp = TelegraphProcess::new(Decimal::ONE, Decimal::ONE);
        let _initial_state = tp.get_current_state();
        let new_state = tp.next_state(dec!(0.1));
        assert!(new_state == 1 || new_state == -1);
        // There's a chance the state didn't change, so we can't assert inequality
    }

    #[test]
    fn test_telegraph_process_get_current_state() {
        let tp = TelegraphProcess::new(dec!(0.5), dec!(0.5));
        let state = tp.get_current_state();
        assert!(state == 1 || state == -1);
    }

    #[test]
    fn test_estimate_telegraph_parameters() {
        let returns = vec![
            dec!(-0.01),
            dec!(0.02),
            dec!(0.01),
            dec!(-0.02),
            dec!(0.03),
            dec!(-0.01),
            dec!(0.01),
            dec!(-0.03),
        ];
        let threshold = dec!(0.01);
        let (lambda_up, lambda_down) = estimate_telegraph_parameters(&returns, threshold).unwrap();
        assert!(lambda_up > Decimal::ZERO);
        assert!(lambda_down > Decimal::ZERO);
    }

    #[test]
    fn test_telegraph() {
        // Create a mock Options struct
        let option = Options {
            option_type: OptionType::European,
            side: Side::Long,
            underlying_price: pos!(100.0),
            strike_price: pos!(100.0),
            risk_free_rate: dec!(0.05),
            option_style: OptionStyle::Call,
            dividend_yield: Positive::ZERO,
            implied_volatility: pos!(0.2),
            underlying_symbol: "".to_string(),
            expiration_date: Default::default(),
            quantity: Positive::ONE,
            exotic_params: None,
        };

        let _price = telegraph(&option, 1000, Some(dec!(0.7)), Some(dec!(0.5)));
        // price is stochastic
        // assert_relative_eq!(price, 0.0, epsilon = 0.0001);
    }
}

#[cfg(test)]
mod tests_telegraph_process_extended {
    use super::*;
    use crate::Positive;
    use crate::model::types::{OptionStyle, OptionType, Side};
    use crate::pos;
    use rust_decimal_macros::dec;

    // Helper function to create a mock Options struct
    fn create_mock_option() -> Options {
        Options {
            option_type: OptionType::European,
            side: Side::Long,
            underlying_price: pos!(100.0),
            strike_price: pos!(100.0),
            risk_free_rate: dec!(0.05),
            option_style: OptionStyle::Call,
            dividend_yield: Positive::ZERO,
            implied_volatility: pos!(0.2),
            underlying_symbol: "".to_string(),
            expiration_date: Default::default(),
            quantity: Positive::ZERO,
            exotic_params: None,
        }
    }

    #[test]
    fn test_telegraph_process_new() {
        let tp = TelegraphProcess::new(dec!(0.5), dec!(0.3));
        assert_eq!(tp.lambda_up, dec!(0.5));
        assert_eq!(tp.lambda_down, dec!(0.3));
        assert!(tp.get_current_state() == 1 || tp.get_current_state() == -1);
    }

    #[test]
    fn test_telegraph_process_next_state() {
        let mut tp = TelegraphProcess::new(dec!(1000.0), dec!(1000.0)); // High rates to ensure state change
        let initial_state = tp.get_current_state();
        let new_state = tp.next_state(dec!(0.1));
        assert_ne!(initial_state, new_state);
    }

    #[test]
    fn test_telegraph_process_get_current_state() {
        let tp = TelegraphProcess::new(dec!(0.5), dec!(0.5));
        let state = tp.get_current_state();
        assert!(state == 1 || state == -1);
    }

    #[test]
    fn test_estimate_telegraph_parameters() {
        let returns = vec![
            dec!(-0.01),
            dec!(0.02),
            dec!(0.01),
            dec!(-0.02),
            dec!(0.03),
            dec!(-0.01),
            dec!(0.01),
            dec!(-0.03),
        ];
        let threshold = Decimal::ZERO;
        let result = estimate_telegraph_parameters(&returns, threshold);
        assert!(result.is_ok());
        let (lambda_up, lambda_down) = result.unwrap();
        assert!(lambda_up > Decimal::ZERO);
        assert!(lambda_down > Decimal::ZERO);
    }

    #[test]
    fn test_estimate_telegraph_parameters_all_positive() {
        let returns = vec![
            dec!(0.01),
            dec!(0.02),
            dec!(0.01),
            dec!(0.02),
            dec!(0.03),
            dec!(0.01),
            dec!(0.01),
            dec!(0.03),
        ];
        let threshold = Decimal::ZERO;
        // All returns are positive, so all will be in state +1, no state transitions
        // This should result in an error due to empty down_durations
        assert!(estimate_telegraph_parameters(&returns, threshold).is_err());
    }

    #[test]
    fn test_estimate_telegraph_parameters_all_negative() {
        let returns = vec![
            dec!(-0.01),
            dec!(-0.02),
            dec!(-0.01),
            dec!(-0.02),
            dec!(-0.03),
            dec!(-0.01),
            dec!(-0.01),
            dec!(-0.03),
        ];
        let threshold = dec!(0.01);
        assert!(estimate_telegraph_parameters(&returns, threshold).is_err());
    }

    #[test]
    fn test_telegraph_with_provided_parameters() {
        let option = create_mock_option();
        let _price = telegraph(&option, 100, Some(dec!(0.5)), Some(dec!(0.5)));
        // assert!(price > 0.0);
    }

    #[test]
    fn test_telegraph_with_estimated_parameters() {
        let option = create_mock_option();
        let _price = telegraph(&option, 100, None, None);
        // assert!(price > 0.0);
    }

    #[test]
    fn test_telegraph_with_one_estimated_parameter() {
        let option = create_mock_option();
        let _price_up = telegraph(&option, 100, Some(dec!(0.5)), None);
        let _price_down = telegraph(&option, 100, None, Some(dec!(0.5)));

        // assert!(price_up > 0.0);
        // assert!(price_down > 0.0);
    }

    #[test]
    fn test_telegraph_different_no_steps() {
        let option = create_mock_option();
        let _price_100 = telegraph(&option, 100, Some(dec!(0.5)), Some(dec!(0.5)));
        let _price_1000 = telegraph(&option, 1000, Some(dec!(0.5)), Some(dec!(0.5)));

        // assert!(price_100 > 0.0);
        // assert!(price_1000 > 0.0);
        // assert_ne!(price_100, price_1000);
    }

    #[test]
    fn test_telegraph_zero_volatility() {
        let mut option = create_mock_option();
        option.implied_volatility = Positive::ZERO;
        let _price = telegraph(&option, 100, Some(dec!(0.5)), Some(dec!(0.5)));
        // assert_relative_eq!(price, 0.0, epsilon = 1e-6);
    }

    #[test]
    fn test_telegraph_zero_risk_free_rate() {
        let mut option = create_mock_option();
        option.risk_free_rate = Decimal::ZERO;
        let _price = telegraph(&option, 100, Some(dec!(0.5)), Some(dec!(0.5)));
        // assert!(price > 0.0);
    }

    #[test]
    fn test_telegraph_zero_time_to_expiration() {
        let option = create_mock_option();
        let price = telegraph(&option, 100, Some(dec!(0.5)), Some(dec!(0.5))).unwrap();
        assert_eq!(
            price,
            option.payoff_at_price(&option.underlying_price).unwrap()
        );
    }
}
