//! The Ornstein-Uhlenbeck path generator.
//!
//! A mean-reverting path is a simulation kernel, not a volatility model: it
//! reports [`crate::error::SimulationError`] and names nothing from the
//! pricing layer. `volatility::generate_ou_process` re-exports it, so the
//! 0.21 path is unchanged.

use crate::error::SimulationError;
use crate::model::decimal::{d_add, d_mul, decimal_normal_sample, p_sqrt};
use crate::model::utils::sub_floor_zero;
use positive::Positive;
use rust_decimal::Decimal;

/// Generates a mean-reverting Ornstein-Uhlenbeck process time series
///
/// This function simulates a discrete-time Ornstein-Uhlenbeck stochastic process, which is
/// commonly used in financial mathematics to model mean-reverting processes such as interest rates,
/// volatility, or commodity prices. The process follows the stochastic differential equation:
///
/// dX_t = θ(μ - X_t)dt + σdW_t
///
/// Where:
/// - θ (theta) is the speed of reversion to the mean
/// - μ (mu) is the long-term mean level
/// - σ (sigma) is the volatility or intensity of random fluctuations
/// - W_t is a Wiener process (standard Brownian motion)
///
/// # Arguments
/// * `x0` - Initial value of the process
/// * `mu` - Long-term mean the process reverts to
/// * `theta` - Speed of mean reversion (higher values cause faster reversion)
/// * `sigma` - Volatility parameter controlling the intensity of random fluctuations
/// * `dt` - Time step size for the simulation
/// * `steps` - Number of time steps to simulate
///
/// # Returns
/// A vector containing the simulated values of the Ornstein-Uhlenbeck process at each time step
///
/// # Errors
///
/// Returns [`SimulationError::PositiveError`] when `sqrt(dt)` or the mean
/// reversion term `theta * (mu - x) * dt` leaves the representable `Positive`
/// range, and [`SimulationError::Decimal`] when the diffusion term or the
/// accumulated level leaves the representable `Decimal` range.
///
/// There is no limit to return in their place: the process level is the state
/// being simulated, so a step that cannot be represented ends the path rather
/// than continuing from a stand-in value.
///
/// # Examples
///
/// ```rust
/// use rust_decimal_macros::dec;
/// use positive::{pos_or_panic, Positive};
/// use optionstratlib::volatility::generate_ou_process;
///
/// # fn main() -> Result<(), optionstratlib::error::SimulationError> {
/// // Simulate an OU process with initial value 1.0, mean 1.5,
/// // reversion speed 0.1, volatility 0.2, time step 0.01, for 1000 steps
/// let process = generate_ou_process(
///     Positive::ONE,       // initial value
///     pos_or_panic!(1.5),       // long-term mean
///     pos_or_panic!(0.1),       // speed of reversion
///     pos_or_panic!(0.2),       // volatility
///     pos_or_panic!(0.01),      // time step
///     1000             // number of steps
/// )?;
/// # Ok(())
/// # }
/// ```
pub fn generate_ou_process(
    x0: Positive,
    mu: Positive,
    theta: Positive,
    volatility: Positive,
    dt: Positive,
    steps: usize,
) -> Result<Vec<Positive>, SimulationError> {
    let sqrt_dt = p_sqrt(&dt, "volatility::utils::generate_ou_process")?;
    let mut x = x0.to_dec();
    let mut result = Vec::with_capacity(steps);
    result.push(Positive::new_decimal(x).unwrap_or(Positive::ZERO));

    for _ in 1..steps {
        // Z√dt
        let dw = d_mul(
            decimal_normal_sample(),
            sqrt_dt.to_dec(),
            "volatility::ou::dw",
        )?;
        // θ(μ−x)dt
        let drift = theta
            .checked_mul(&sub_floor_zero(mu, &x))?
            .checked_mul(&dt)?
            .to_dec();
        // σ·Z√dt (Decimal)
        let diffusion = d_mul(dw, volatility.to_dec(), "volatility::ou::diffusion")?;
        // OU process step
        x = d_add(
            x,
            d_add(drift, diffusion, "volatility::ou::step")?,
            "volatility::ou::step",
        )?;
        x = x.max(Decimal::ZERO); // no negative values
        result.push(Positive::new_decimal(x).unwrap_or(Positive::ZERO));
    }

    Ok(result)
}

#[cfg(test)]
mod tests_generate_ou_process {
    use super::*;
    use positive::pos_or_panic;
    use rust_decimal_macros::dec;

    #[test]
    fn test_process_length() {
        let steps = 500;
        let process = generate_ou_process(
            Positive::ONE,
            pos_or_panic!(1.5),
            pos_or_panic!(0.1),
            pos_or_panic!(0.2),
            pos_or_panic!(0.01),
            steps,
        )
        .unwrap();
        assert_eq!(process.len(), steps);
    }

    #[test]
    fn test_all_values_positive() {
        let process = generate_ou_process(
            Positive::ONE,
            pos_or_panic!(1.5),
            pos_or_panic!(0.2),
            pos_or_panic!(0.3),
            pos_or_panic!(0.01),
            1000,
        )
        .unwrap();

        for value in process {
            assert!(
                value >= Positive::ZERO,
                "Found non-positive value: {value:?}"
            );
        }
    }

    #[test]
    fn test_mean_reversion_tendency() {
        let process = generate_ou_process(
            pos_or_panic!(0.1),
            Positive::ONE,
            Positive::ONE,       // high theta for fast reversion
            pos_or_panic!(0.01), // low volatility
            pos_or_panic!(0.01),
            1000,
        )
        .unwrap();

        let last = process.last().unwrap().to_dec();
        let diff = (last - dec!(1.0)).abs();
        assert!(diff < dec!(0.1), "Final value too far from mean: {last}");
    }
}
