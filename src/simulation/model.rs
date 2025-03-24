use crate::Positive;
use rust_decimal::Decimal;
use std::fmt::{Display, Formatter};

/// Enum defining different types of random walks
#[derive(Debug, Clone, Copy)]
pub enum WalkType {
    /// Standard Brownian motion (normal increments)
    Brownian {
        /// Time step size (fraction of year: daily=1/365, weekly=1/52, etc.)
        dt: Positive,
        /// Drift parameter (expected return or growth rate)
        drift: Decimal,
        /// Volatility parameter (annualized standard deviation)
        volatility: Positive,
    },

    /// Geometric Brownian motion (log-normal increments)
    GeometricBrownian {
        /// Time step size (fraction of year: daily=1/365, weekly=1/52, etc.)
        dt: Positive,
        /// Drift parameter (expected return or growth rate)
        drift: Decimal,
        /// Volatility parameter (annualized standard deviation)
        volatility: Positive,
    },

    /// Log-Returns model (simulates directly log-returns instead of prices)
    LogReturns {
        /// Time step size (fraction of year: daily=1/365, weekly=1/52, etc.)
        dt: Positive,
        /// Expected return (mean of log returns)
        expected_return: Decimal,
        /// Volatility parameter (annualized standard deviation of log returns)
        volatility: Positive,
        /// Optional autocorrelation parameter (-1 to 1)
        autocorrelation: Option<Decimal>,
    },

    /// Mean-reverting process (Ornstein-Uhlenbeck)
    MeanReverting {
        /// Time step size (fraction of year: daily=1/365, weekly=1/52, etc.)
        dt: Positive,
        /// Volatility parameter (annualized standard deviation)
        volatility: Positive,
        /// Mean reversion speed (rate at which process reverts to mean)
        speed: Positive,
        /// Long-term mean (equilibrium level)
        mean: Positive,
    },

    /// Jump diffusion process (normal increments with occasional jumps)
    JumpDiffusion {
        /// Time step size (fraction of year: daily=1/365, weekly=1/52, etc.)
        dt: Positive,
        /// Drift parameter (expected return of continuous part)
        drift: Decimal,
        /// Volatility parameter (annualized standard deviation of continuous part)
        volatility: Positive,
        /// Jump intensity (annual frequency of jumps)
        intensity: Positive,
        /// Jump size mean (average jump magnitude)
        jump_mean: Decimal,
        /// Jump size volatility (standard deviation of jump size)
        jump_volatility: Positive,
    },

    /// GARCH process (time-varying volatility)
    Garch {
        /// Time step size (fraction of year: daily=1/365, weekly=1/52, etc.)
        dt: Positive,
        /// Drift parameter (expected return)
        drift: Decimal,
        /// Initial volatility parameter (starting volatility level)
        volatility: Positive,
        /// GARCH alpha parameter (impact of past observations)
        alpha: Positive,
        /// GARCH beta parameter (persistence of volatility)
        beta: Positive,
        /// Long-term variance (unconditional variance)
        omega: Positive,
    },

    /// Heston model (stochastic volatility)
    Heston {
        /// Time step size (fraction of year: daily=1/365, weekly=1/52, etc.)
        dt: Positive,
        /// Drift parameter (expected return)
        drift: Decimal,
        /// Initial volatility parameter (starting volatility level)
        volatility: Positive,
        /// Mean reversion speed of volatility
        kappa: Positive,
        /// Long-term variance (equilibrium level of variance)
        theta: Positive,
        /// Volatility of volatility (standard deviation of variance process)
        xi: Positive,
        /// Correlation between price and volatility processes
        rho: Decimal,
    },

    /// Custom process defined by a function
    Custom {
        /// Time step size (fraction of year: daily=1/365, weekly=1/52, etc.)
        dt: Positive,
        /// Drift parameter (expected change)
        drift: Decimal,
        /// Volatility parameter (may be interpreted differently based on custom implementation)
        volatility: Positive,
        /// Volatility of Volatility parameter (annualized standard deviation)
        vov: Positive,
        /// Mean reversion speed (rate at which process reverts to mean)
        vol_speed: Positive,
        /// Long-term mean (equilibrium level)
        vol_mean: Positive,
    },
}

impl Display for WalkType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            WalkType::Brownian {
                dt,
                drift,
                volatility,
            } => write!(
                f,
                "Brownian {{ dt: {}, drift: {}, volatility: {} }}",
                dt, drift, volatility
            ),
            WalkType::GeometricBrownian {
                dt,
                drift,
                volatility,
            } => write!(
                f,
                "GeometricBrownian {{ dt: {}, drift: {}, volatility: {} }}",
                dt, drift, volatility
            ),
            WalkType::LogReturns {
                dt,
                expected_return,
                volatility,
                autocorrelation,
            } => write!(
                f,
                "LogReturns {{ dt: {}, expected_return: {}, volatility: {}, autocorrelation: {:?} }}",
                dt, expected_return, volatility, autocorrelation
            ),
            WalkType::MeanReverting {
                dt,
                volatility,
                speed,
                mean,
            } => write!(
                f,
                "MeanReverting {{ dt: {}, volatility: {}, speed: {}, mean: {} }}",
                dt, volatility, speed, mean
            ),
            WalkType::JumpDiffusion {
                dt,
                drift,
                volatility,
                intensity,
                jump_mean,
                jump_volatility,
            } => write!(
                f,
                "JumpDiffusion {{ dt: {}, drift: {}, volatility: {}, intensity: {}, jump_mean: {}, jump_volatility: {} }}",
                dt, drift, volatility, intensity, jump_mean, jump_volatility
            ),
            WalkType::Garch {
                dt,
                drift,
                volatility,
                alpha,
                beta,
                omega,
            } => write!(
                f,
                "Garch {{ dt: {}, drift: {}, volatility: {}, alpha: {}, beta: {}, omega: {} }}",
                dt, drift, volatility, alpha, beta, omega
            ),
            WalkType::Heston {
                dt,
                drift,
                volatility,
                kappa,
                theta,
                xi,
                rho,
            } => write!(
                f,
                "Heston {{ dt: {}, drift: {}, volatility: {}, kappa: {}, theta: {}, xi: {}, rho: {} }}",
                dt, drift, volatility, kappa, theta, xi, rho
            ),
            WalkType::Custom {
                dt,
                drift,
                volatility,
                vov,
                vol_speed,
                vol_mean,
            } => write!(
                f,
                "Custom {{ dt: {}, drift: {}, volatility: {}, vov: {}, vol_speed: {}, vol_mean: {} }}",
                dt, drift, volatility, vov, vol_speed, vol_mean
            ),
        }
    }
}
