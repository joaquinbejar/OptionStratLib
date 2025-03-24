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

#[cfg(test)]
mod tests_walk_type {
    use super::*;
    use crate::pos;
    use rust_decimal_macros::dec;

    #[test]
    fn test_brownian_creation() {
        let walk = WalkType::Brownian {
            dt: pos!(0.01),
            drift: dec!(0.05),
            volatility: pos!(0.2),
        };

        if let WalkType::Brownian {
            dt,
            drift,
            volatility,
        } = walk
        {
            assert_eq!(dt, pos!(0.01));
            assert_eq!(drift, dec!(0.05));
            assert_eq!(volatility, pos!(0.2));
        } else {
            panic!("Expected Brownian variant");
        }
    }

    #[test]
    fn test_geometric_brownian_creation() {
        let walk = WalkType::GeometricBrownian {
            dt: pos!(0.01),
            drift: dec!(0.05),
            volatility: pos!(0.2),
        };

        if let WalkType::GeometricBrownian {
            dt,
            drift,
            volatility,
        } = walk
        {
            assert_eq!(dt, pos!(0.01));
            assert_eq!(drift, dec!(0.05));
            assert_eq!(volatility, pos!(0.2));
        } else {
            panic!("Expected GeometricBrownian variant");
        }
    }

    #[test]
    fn test_log_returns_creation() {
        let walk = WalkType::LogReturns {
            dt: pos!(0.01),
            expected_return: dec!(0.05),
            volatility: pos!(0.2),
            autocorrelation: Some(dec!(0.1)),
        };

        if let WalkType::LogReturns {
            dt,
            expected_return,
            volatility,
            autocorrelation,
        } = walk
        {
            assert_eq!(dt, pos!(0.01));
            assert_eq!(expected_return, dec!(0.05));
            assert_eq!(volatility, pos!(0.2));
            assert_eq!(autocorrelation, Some(dec!(0.1)));
        } else {
            panic!("Expected LogReturns variant");
        }
    }

    #[test]
    fn test_mean_reverting_creation() {
        let walk = WalkType::MeanReverting {
            dt: pos!(0.01),
            volatility: pos!(0.2),
            speed: pos!(0.1),
            mean: pos!(100.0),
        };

        if let WalkType::MeanReverting {
            dt,
            volatility,
            speed,
            mean,
        } = walk
        {
            assert_eq!(dt, pos!(0.01));
            assert_eq!(volatility, pos!(0.2));
            assert_eq!(speed, pos!(0.1));
            assert_eq!(mean, pos!(100.0));
        } else {
            panic!("Expected MeanReverting variant");
        }
    }

    #[test]
    fn test_jump_diffusion_creation() {
        let walk = WalkType::JumpDiffusion {
            dt: pos!(0.01),
            drift: dec!(0.05),
            volatility: pos!(0.2),
            intensity: pos!(1.0),
            jump_mean: dec!(-0.05),
            jump_volatility: pos!(0.3),
        };

        if let WalkType::JumpDiffusion {
            dt,
            drift,
            volatility,
            intensity,
            jump_mean,
            jump_volatility,
        } = walk
        {
            assert_eq!(dt, pos!(0.01));
            assert_eq!(drift, dec!(0.05));
            assert_eq!(volatility, pos!(0.2));
            assert_eq!(intensity, pos!(1.0));
            assert_eq!(jump_mean, dec!(-0.05));
            assert_eq!(jump_volatility, pos!(0.3));
        } else {
            panic!("Expected JumpDiffusion variant");
        }
    }

    #[test]
    fn test_garch_creation() {
        let walk = WalkType::Garch {
            dt: pos!(0.01),
            drift: dec!(0.05),
            volatility: pos!(0.2),
            alpha: pos!(0.1),
            beta: pos!(0.8),
            omega: pos!(0.02),
        };

        if let WalkType::Garch {
            dt,
            drift,
            volatility,
            alpha,
            beta,
            omega,
        } = walk
        {
            assert_eq!(dt, pos!(0.01));
            assert_eq!(drift, dec!(0.05));
            assert_eq!(volatility, pos!(0.2));
            assert_eq!(alpha, pos!(0.1));
            assert_eq!(beta, pos!(0.8));
            assert_eq!(omega, pos!(0.02));
        } else {
            panic!("Expected Garch variant");
        }
    }

    #[test]
    fn test_heston_creation() {
        let walk = WalkType::Heston {
            dt: pos!(0.01),
            drift: dec!(0.05),
            volatility: pos!(0.2),
            kappa: pos!(2.0),
            theta: pos!(0.04),
            xi: pos!(0.3),
            rho: dec!(-0.7),
        };

        if let WalkType::Heston {
            dt,
            drift,
            volatility,
            kappa,
            theta,
            xi,
            rho,
        } = walk
        {
            assert_eq!(dt, pos!(0.01));
            assert_eq!(drift, dec!(0.05));
            assert_eq!(volatility, pos!(0.2));
            assert_eq!(kappa, pos!(2.0));
            assert_eq!(theta, pos!(0.04));
            assert_eq!(xi, pos!(0.3));
            assert_eq!(rho, dec!(-0.7));
        } else {
            panic!("Expected Heston variant");
        }
    }

    #[test]
    fn test_custom_creation() {
        let walk = WalkType::Custom {
            dt: pos!(0.01),
            drift: dec!(0.05),
            volatility: pos!(0.2),
            vov: pos!(0.25),
            vol_speed: pos!(0.15),
            vol_mean: pos!(0.3),
        };

        if let WalkType::Custom {
            dt,
            drift,
            volatility,
            vov,
            vol_speed,
            vol_mean,
        } = walk
        {
            assert_eq!(dt, pos!(0.01));
            assert_eq!(drift, dec!(0.05));
            assert_eq!(volatility, pos!(0.2));
            assert_eq!(vov, pos!(0.25));
            assert_eq!(vol_speed, pos!(0.15));
            assert_eq!(vol_mean, pos!(0.3));
        } else {
            panic!("Expected Custom variant");
        }
    }

    #[test]
    fn test_display_brownian() {
        let walk = WalkType::Brownian {
            dt: pos!(0.01),
            drift: dec!(0.05),
            volatility: pos!(0.2),
        };

        let display = format!("{}", walk);
        assert!(display.contains("Brownian"));
        assert!(display.contains("dt: 0.01"));
        assert!(display.contains("drift: 0.05"));
        assert!(display.contains("volatility: 0.2"));
    }

    #[test]
    fn test_display_geometric_brownian() {
        let walk = WalkType::GeometricBrownian {
            dt: pos!(0.01),
            drift: dec!(0.05),
            volatility: pos!(0.2),
        };

        let display = format!("{}", walk);
        assert!(display.contains("GeometricBrownian"));
        assert!(display.contains("dt: 0.01"));
        assert!(display.contains("drift: 0.05"));
        assert!(display.contains("volatility: 0.2"));
    }

    #[test]
    fn test_display_log_returns() {
        let walk = WalkType::LogReturns {
            dt: pos!(0.01),
            expected_return: dec!(0.05),
            volatility: pos!(0.2),
            autocorrelation: Some(dec!(0.1)),
        };

        let display = format!("{}", walk);
        assert!(display.contains("LogReturns"));
        assert!(display.contains("dt: 0.01"));
        assert!(display.contains("expected_return: 0.05"));
        assert!(display.contains("volatility: 0.2"));
        assert!(display.contains("autocorrelation: Some(0.1)"));
    }

    #[test]
    fn test_display_mean_reverting() {
        let walk = WalkType::MeanReverting {
            dt: pos!(0.01),
            volatility: pos!(0.2),
            speed: pos!(0.1),
            mean: pos!(100.0),
        };

        let display = format!("{}", walk);
        assert!(display.contains("MeanReverting"));
        assert!(display.contains("dt: 0.01"));
        assert!(display.contains("volatility: 0.2"));
        assert!(display.contains("speed: 0.1"));
        assert!(display.contains("mean: 100"));
    }

    #[test]
    fn test_display_jump_diffusion() {
        let walk = WalkType::JumpDiffusion {
            dt: pos!(0.01),
            drift: dec!(0.05),
            volatility: pos!(0.2),
            intensity: pos!(1.0),
            jump_mean: dec!(-0.05),
            jump_volatility: pos!(0.3),
        };

        let display = format!("{}", walk);
        assert!(display.contains("JumpDiffusion"));
        assert!(display.contains("dt: 0.01"));
        assert!(display.contains("drift: 0.05"));
        assert!(display.contains("volatility: 0.2"));
        assert!(display.contains("intensity: 1"));
        assert!(display.contains("jump_mean: -0.05"));
        assert!(display.contains("jump_volatility: 0.3"));
    }

    #[test]
    fn test_display_garch() {
        let walk = WalkType::Garch {
            dt: pos!(0.01),
            drift: dec!(0.05),
            volatility: pos!(0.2),
            alpha: pos!(0.1),
            beta: pos!(0.8),
            omega: pos!(0.02),
        };

        let display = format!("{}", walk);
        assert!(display.contains("Garch"));
        assert!(display.contains("dt: 0.01"));
        assert!(display.contains("drift: 0.05"));
        assert!(display.contains("volatility: 0.2"));
        assert!(display.contains("alpha: 0.1"));
        assert!(display.contains("beta: 0.8"));
        assert!(display.contains("omega: 0.02"));
    }

    #[test]
    fn test_display_heston() {
        let walk = WalkType::Heston {
            dt: pos!(0.01),
            drift: dec!(0.05),
            volatility: pos!(0.2),
            kappa: pos!(2.0),
            theta: pos!(0.04),
            xi: pos!(0.3),
            rho: dec!(-0.7),
        };

        let display = format!("{}", walk);
        assert!(display.contains("Heston"));
        assert!(display.contains("dt: 0.01"));
        assert!(display.contains("drift: 0.05"));
        assert!(display.contains("volatility: 0.2"));
        assert!(display.contains("kappa: 2"));
        assert!(display.contains("theta: 0.04"));
        assert!(display.contains("xi: 0.3"));
        assert!(display.contains("rho: -0.7"));
    }

    #[test]
    fn test_display_custom() {
        let walk = WalkType::Custom {
            dt: pos!(0.01),
            drift: dec!(0.05),
            volatility: pos!(0.2),
            vov: pos!(0.25),
            vol_speed: pos!(0.15),
            vol_mean: pos!(0.3),
        };

        let display = format!("{}", walk);
        assert!(display.contains("Custom"));
        assert!(display.contains("dt: 0.01"));
        assert!(display.contains("drift: 0.05"));
        assert!(display.contains("volatility: 0.2"));
        assert!(display.contains("vov: 0.25"));
        assert!(display.contains("vol_speed: 0.15"));
        assert!(display.contains("vol_mean: 0.3"));
    }

    #[test]
    fn test_log_returns_without_autocorrelation() {
        let walk = WalkType::LogReturns {
            dt: pos!(0.01),
            expected_return: dec!(0.05),
            volatility: pos!(0.2),
            autocorrelation: None,
        };

        if let WalkType::LogReturns {
            dt,
            expected_return,
            volatility,
            autocorrelation,
        } = walk
        {
            assert_eq!(dt, pos!(0.01));
            assert_eq!(expected_return, dec!(0.05));
            assert_eq!(volatility, pos!(0.2));
            assert_eq!(autocorrelation, None);
        } else {
            panic!("Expected LogReturns variant");
        }

        let display = format!("{}", walk);
        assert!(display.contains("LogReturns"));
        assert!(display.contains("autocorrelation: None"));
    }

    #[test]
    fn test_clone() {
        let walk = WalkType::Brownian {
            dt: pos!(0.01),
            drift: dec!(0.05),
            volatility: pos!(0.2),
        };

        let cloned = walk;

        if let WalkType::Brownian {
            dt,
            drift,
            volatility,
        } = cloned
        {
            assert_eq!(dt, pos!(0.01));
            assert_eq!(drift, dec!(0.05));
            assert_eq!(volatility, pos!(0.2));
        } else {
            panic!("Expected Brownian variant");
        }
    }

    #[test]
    fn test_clone_equality() {
        let walk = WalkType::Brownian {
            dt: pos!(0.01),
            drift: dec!(0.05),
            volatility: pos!(0.2),
        };

        let cloned = walk;

        // Using Debug formatting to compare
        assert_eq!(format!("{:?}", walk), format!("{:?}", cloned));
    }
}
