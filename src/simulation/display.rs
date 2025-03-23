use crate::simulation::types::Walktypable;
use crate::simulation::walk::{WalkParams, WalkType};
use std::fmt::{Display, Formatter};
use std::ops::AddAssign;
use crate::Positive;

impl<X, Y> Display for WalkParams<X, Y>
where
    X: Copy + Into<Positive> + AddAssign + Display,
    Y: Copy + Into<Positive> + Display + Walktypable,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "WalkParams {{ size: {}, init_step: {}, walk_type: {} }}",
            self.size, self.init_step, self.walk_type
        )
    }
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
