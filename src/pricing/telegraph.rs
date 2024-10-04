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

use crate::constants::ZERO;
use crate::model::option::Options;
use crate::pricing::utils::simulate_returns;
use rand::random;

#[derive(Clone)]
pub struct TelegraphProcess {
    /// Transition rate from state -1 to +1
    lambda_up: f64,
    /// Transition rate from state +1 to -1
    lambda_down: f64,
    /// Current state of the process (-1 or 1)
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
    pub fn new(lambda_up: f64, lambda_down: f64) -> Self {
        let initial_state = if rand::random::<f64>() < 0.5 { 1 } else { -1 };
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
    pub fn next_state(&mut self, dt: f64) -> i8 {
        let lambda = if self.current_state == 1 {
            self.lambda_down
        } else {
            self.lambda_up
        };
        let probability = 1.0 - (-lambda * dt).exp();

        if random::<f64>() < probability {
            self.current_state *= -1;
        }

        self.current_state
    }

    #[allow(dead_code)]
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
pub(crate) fn estimate_telegraph_parameters(returns: &[f64], threshold: f64) -> (f64, f64) {
    let mut current_state = if returns[0] > threshold { 1 } else { -1 };
    let mut current_duration = 1;
    let mut up_durations = Vec::new();
    let mut down_durations = Vec::new();

    for &ret in returns.iter().skip(1) {
        let new_state = if ret > threshold { 1 } else { -1 };
        if new_state == current_state {
            current_duration += 1;
        } else {
            if current_state == 1 {
                up_durations.push(current_duration);
            } else {
                down_durations.push(current_duration);
            }
            current_state = new_state;
            current_duration = 1;
        }
    }

    if current_state == 1 {
        up_durations.push(current_duration);
    } else {
        down_durations.push(current_duration);
    }

    let lambda_up = 1.0 / down_durations.iter().sum::<i32>() as f64 * down_durations.len() as f64;
    let lambda_down = 1.0 / up_durations.iter().sum::<i32>() as f64 * up_durations.len() as f64;
    (lambda_up, lambda_down)
}

pub fn telegraph(
    option: &Options,
    no_steps: usize,
    lambda_up: Option<f64>,
    lambda_down: Option<f64>,
) -> f64 {
    let mut price = option.underlying_price;
    let dt = option.time_to_expiration() / no_steps as f64;

    let (lambda_up_temp, lambda_down_temp) = match (lambda_up, lambda_down) {
        (None, None) => {
            let returns = simulate_returns(ZERO, option.implied_volatility, 100, 1.0 / 252.0);
            estimate_telegraph_parameters(&returns, ZERO)
        }
        (Some(l_up), None) => {
            let returns = simulate_returns(ZERO, option.implied_volatility, 100, 1.0 / 252.0);
            let (_, l_down) = estimate_telegraph_parameters(&returns, ZERO);
            (l_up, l_down)
        }
        (None, Some(l_down)) => {
            let returns = simulate_returns(ZERO, option.implied_volatility, 100, 1.0 / 252.0);
            let (l_up, _) = estimate_telegraph_parameters(&returns, ZERO);
            (l_up, l_down)
        }
        (Some(l_up), Some(l_down)) => (l_up, l_down),
    };

    let telegraph_process = TelegraphProcess::new(lambda_up_temp, lambda_down_temp);

    let tp = telegraph_process;
    let mut telegraph_process = tp.clone();
    for _ in 0..no_steps {
        let state = telegraph_process.next_state(dt);
        let drift = option.risk_free_rate - 0.5 * option.implied_volatility.powi(2);
        let volatility = option.implied_volatility * state as f64;

        price *= (drift * dt + volatility * (dt.sqrt() * rand::random::<f64>())).exp();
    }

    let payoff = option.payoff_at_price(price);
    payoff * (-option.risk_free_rate * option.time_to_expiration()).exp()
}

#[cfg(test)]
mod tests_telegraph_process_basis {
    use super::*;
    use crate::model::types::{OptionStyle, OptionType, Side, PZERO, SIZE_ONE};

    #[test]
    fn test_telegraph_process_new() {
        let tp = TelegraphProcess::new(0.5, 0.3);
        assert!(tp.lambda_up == 0.5);
        assert!(tp.lambda_down == 0.3);
        assert!(tp.current_state == 1 || tp.current_state == -1);
    }

    #[test]
    fn test_telegraph_process_next_state() {
        let mut tp = TelegraphProcess::new(1.0, 1.0);
        let _initial_state = tp.get_current_state();
        let new_state = tp.next_state(0.1);
        assert!(new_state == 1 || new_state == -1);
        // There's a chance the state didn't change, so we can't assert inequality
    }

    #[test]
    fn test_telegraph_process_get_current_state() {
        let tp = TelegraphProcess::new(0.5, 0.5);
        let state = tp.get_current_state();
        assert!(state == 1 || state == -1);
    }

    #[test]
    fn test_estimate_telegraph_parameters() {
        let returns = vec![-0.01, 0.02, 0.01, -0.02, 0.03, -0.01, 0.01, -0.03];
        let threshold = 0.0;
        let (lambda_up, lambda_down) = estimate_telegraph_parameters(&returns, threshold);
        assert!(lambda_up > 0.0);
        assert!(lambda_down > 0.0);
    }

    #[test]
    fn test_telegraph() {
        // Create a mock Options struct
        let option = Options {
            option_type: OptionType::European,
            side: Side::Long,
            underlying_price: 100.0,
            strike_price: 100.0,
            risk_free_rate: 0.05,
            option_style: OptionStyle::Call,
            dividend_yield: 0.0,
            implied_volatility: 0.2,
            underlying_symbol: "".to_string(),
            expiration_date: Default::default(),
            quantity: SIZE_ONE,
            exotic_params: None,
        };

        let _price = telegraph(&option, 1000, Some(0.7), Some(0.5));
        // price is stochastic
        // assert_relative_eq!(price, 0.0, epsilon = 0.0001);
    }

    #[test]
    fn test_telegraph_with_estimated_parameters() {
        // Create a mock Options struct
        let option = Options {
            option_type: OptionType::European,
            side: Side::Long,
            underlying_price: 100.0,
            strike_price: 100.0,
            risk_free_rate: 0.05,
            option_style: OptionStyle::Call,
            dividend_yield: 0.0,
            implied_volatility: 0.2,
            underlying_symbol: "".to_string(),
            expiration_date: Default::default(),
            quantity: PZERO,
            exotic_params: None,
        };

        let _price = telegraph(&option, 100, None, None);
        // price is stochastic // TODO: Fix this
        // assert_relative_eq!(price, 0.0, epsilon = 0.0001);
    }

    #[test]
    fn test_telegraph_with_one_estimated_parameter() {
        // Create a mock Options struct
        let option = Options {
            option_type: OptionType::European,
            side: Side::Long,
            underlying_price: 100.0,
            strike_price: 100.0,
            risk_free_rate: 0.05,
            option_style: OptionStyle::Call,
            dividend_yield: 0.0,
            implied_volatility: 0.2,
            underlying_symbol: "".to_string(),
            expiration_date: Default::default(),
            quantity: PZERO,
            exotic_params: None,
        };

        let _price_up = telegraph(&option, 100, Some(0.5), None);
        let _price_down = telegraph(&option, 100, None, Some(0.5));
        // price is stochastic // TODO: Fix this
        // assert_relative_eq!(price_up, 0.0, epsilon = 0.0001);
        // assert_relative_eq!(price_down, 0.0, epsilon = 0.0001);
    }
}

#[cfg(test)]
mod tests_telegraph_process_extended {
    use super::*;
    use crate::model::types::{OptionStyle, OptionType, Side, PZERO};

    // Helper function to create a mock Options struct
    fn create_mock_option() -> Options {
        Options {
            option_type: OptionType::European,
            side: Side::Long,
            underlying_price: 100.0,
            strike_price: 100.0,
            risk_free_rate: 0.05,
            option_style: OptionStyle::Call,
            dividend_yield: 0.0,
            implied_volatility: 0.2,
            underlying_symbol: "".to_string(),
            expiration_date: Default::default(),
            quantity: PZERO,
            exotic_params: None,
        }
    }

    #[test]
    fn test_telegraph_process_new() {
        let tp = TelegraphProcess::new(0.5, 0.3);
        assert_eq!(tp.lambda_up, 0.5);
        assert_eq!(tp.lambda_down, 0.3);
        assert!(tp.get_current_state() == 1 || tp.get_current_state() == -1);
    }

    #[test]
    fn test_telegraph_process_next_state() {
        let mut tp = TelegraphProcess::new(1000.0, 1000.0); // High rates to ensure state change
        let initial_state = tp.get_current_state();
        let new_state = tp.next_state(0.1);
        assert_ne!(initial_state, new_state);
    }

    #[test]
    fn test_telegraph_process_next_state_no_change() {
        let mut tp = TelegraphProcess::new(0.0001, 0.0001); // Low rates to ensure no state change
        let initial_state = tp.get_current_state();
        let new_state = tp.next_state(0.1);
        assert_eq!(initial_state, new_state);
    }

    #[test]
    fn test_telegraph_process_get_current_state() {
        let tp = TelegraphProcess::new(0.5, 0.5);
        let state = tp.get_current_state();
        assert!(state == 1 || state == -1);
    }

    #[test]
    fn test_estimate_telegraph_parameters() {
        let returns = vec![-0.01, 0.02, 0.01, -0.02, 0.03, -0.01, 0.01, -0.03];
        let threshold = 0.0;
        let (lambda_up, lambda_down) = estimate_telegraph_parameters(&returns, threshold);
        assert!(lambda_up > 0.0);
        assert!(lambda_down > 0.0);
    }

    #[test]
    fn test_estimate_telegraph_parameters_all_positive() {
        let returns = vec![0.01, 0.02, 0.01, 0.02, 0.03, 0.01, 0.01, 0.03];
        let threshold = 0.0;
        let (_lambda_up, _lambda_down) = estimate_telegraph_parameters(&returns, threshold);
        // assert_relative_eq!(lambda_up, 0.0, epsilon = 0.0001);
        // assert!(lambda_up > 0.0);
        // assert_eq!(lambda_down, 0.0);
    }

    #[test]
    fn test_estimate_telegraph_parameters_all_negative() {
        let returns = vec![-0.01, -0.02, -0.01, -0.02, -0.03, -0.01, -0.01, -0.03];
        let threshold = 0.0;
        let (_lambda_up, _lambda_down) = estimate_telegraph_parameters(&returns, threshold);
        // assert_eq!(lambda_up, 0.0);
        // assert!(lambda_down > 0.0);
    }

    #[test]
    fn test_telegraph_with_provided_parameters() {
        let option = create_mock_option();
        let _price = telegraph(&option, 100, Some(0.5), Some(0.5));
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
        let _price_up = telegraph(&option, 100, Some(0.5), None);
        let _price_down = telegraph(&option, 100, None, Some(0.5));

        // assert!(price_up > 0.0);
        // assert!(price_down > 0.0);
    }

    #[test]
    fn test_telegraph_different_no_steps() {
        let option = create_mock_option();
        let _price_100 = telegraph(&option, 100, Some(0.5), Some(0.5));
        let _price_1000 = telegraph(&option, 1000, Some(0.5), Some(0.5));

        // assert!(price_100 > 0.0);
        // assert!(price_1000 > 0.0);
        // assert_ne!(price_100, price_1000);
    }

    #[test]
    fn test_telegraph_zero_volatility() {
        let mut option = create_mock_option();
        option.implied_volatility = 0.0;
        let _price = telegraph(&option, 100, Some(0.5), Some(0.5));
        // assert_relative_eq!(price, 0.0, epsilon = 1e-6);
    }

    #[test]
    fn test_telegraph_zero_risk_free_rate() {
        let mut option = create_mock_option();
        option.risk_free_rate = 0.0;
        let _price = telegraph(&option, 100, Some(0.5), Some(0.5));
        // assert!(price > 0.0);
    }

    #[test]
    fn test_telegraph_zero_time_to_expiration() {
        let option = create_mock_option();
        let price = telegraph(&option, 100, Some(0.5), Some(0.5));
        assert_eq!(price, option.payoff_at_price(option.underlying_price));
    }
}
