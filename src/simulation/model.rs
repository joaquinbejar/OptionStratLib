use crate::Positive;
use crate::utils::TimeFrame;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use utoipa::ToSchema;

/// Enum defining different types of random walks
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
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

    /// Telegraph process (two-state regime switching model)
    Telegraph {
        /// Time step size (fraction of year: daily=1/365, weekly=1/52, etc.)
        dt: Positive,
        /// Drift parameter (expected return)
        drift: Decimal,
        /// Base volatility parameter (annualized standard deviation)
        volatility: Positive,
        /// Transition rate from state -1 to +1 (intensity of upward regime changes)
        lambda_up: Positive,
        /// Transition rate from state +1 to -1 (intensity of downward regime changes)
        lambda_down: Positive,
        /// Optional volatility multiplier for the +1 state (default: 1.0)
        vol_multiplier_up: Option<Positive>,
        /// Optional volatility multiplier for the -1 state (default: 1.0)
        vol_multiplier_down: Option<Positive>,
    },

    /// Represents historical price data for a given timeframe.
    ///
    /// This encapsulates the historical price data, including the timeframe
    /// over which the data was collected and a vector of positive price values.
    /// It is typically used to store and process historical market data for
    /// financial analysis and simulation purposes.
    ///
    /// # Fields
    ///
    /// * `timeframe`: The `TimeFrame` over which the historical data is relevant.
    /// * `prices`: A `Vec` of `Positive` values representing the historical prices.
    Historical {
        /// The timeframe of the historical data.
        timeframe: TimeFrame,
        /// The vector of positive price values.
        prices: Vec<Positive>,

        /// A field representing an optional symbol.
        ///
        /// This field stores an optional `String` that can represent a symbol.
        /// It can hold a `Some(String)` value if a symbol is provided, or `None` if no symbol is specified.
        ///
        /// # Use Case
        ///
        /// This field can be utilized in scenarios where a symbol (e.g., stock ticker, identifier) may or may not be required.
        symbol: Option<String>,
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
                "Brownian {{ dt: {dt}, drift: {drift}, volatility: {volatility} }}"
            ),
            WalkType::GeometricBrownian {
                dt,
                drift,
                volatility,
            } => write!(
                f,
                "GeometricBrownian {{ dt: {dt}, drift: {drift}, volatility: {volatility} }}"
            ),
            WalkType::LogReturns {
                dt,
                expected_return,
                volatility,
                autocorrelation,
            } => write!(
                f,
                "LogReturns {{ dt: {dt}, expected_return: {expected_return}, volatility: {volatility}, autocorrelation: {autocorrelation:?} }}"
            ),
            WalkType::MeanReverting {
                dt,
                volatility,
                speed,
                mean,
            } => write!(
                f,
                "MeanReverting {{ dt: {dt}, volatility: {volatility}, speed: {speed}, mean: {mean} }}"
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
                "JumpDiffusion {{ dt: {dt}, drift: {drift}, volatility: {volatility}, intensity: {intensity}, jump_mean: {jump_mean}, jump_volatility: {jump_volatility} }}"
            ),
            WalkType::Garch {
                dt,
                drift,
                volatility,
                alpha,
                beta,
            } => write!(
                f,
                "Garch {{ dt: {dt}, drift: {drift}, volatility: {volatility}, alpha: {alpha}, beta: {beta} }}"
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
                "Heston {{ dt: {dt}, drift: {drift}, volatility: {volatility}, kappa: {kappa}, theta: {theta}, xi: {xi}, rho: {rho} }}"
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
                "Custom {{ dt: {dt}, drift: {drift}, volatility: {volatility}, vov: {vov}, vol_speed: {vol_speed}, vol_mean: {vol_mean} }}"
            ),
            WalkType::Telegraph {
                dt,
                drift,
                volatility,
                lambda_up,
                lambda_down,
                vol_multiplier_up,
                vol_multiplier_down,
            } => write!(
                f,
                "Telegraph {{ dt: {dt}, drift: {drift}, volatility: {volatility}, lambda_up: {lambda_up}, lambda_down: {lambda_down}, vol_multiplier_up: {vol_multiplier_up:?}, vol_multiplier_down: {vol_multiplier_down:?} }}"
            ),
            WalkType::Historical {
                timeframe,
                prices,
                symbol,
            } => write!(
                f,
                "Historical {{ timeframe: {timeframe}, prices: {prices:?}, symbol: {symbol:?} }}"
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
        };

        if let WalkType::Garch {
            dt,
            drift,
            volatility,
            alpha,
            beta,
        } = walk
        {
            assert_eq!(dt, pos!(0.01));
            assert_eq!(drift, dec!(0.05));
            assert_eq!(volatility, pos!(0.2));
            assert_eq!(alpha, pos!(0.1));
            assert_eq!(beta, pos!(0.8));
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
    fn test_telegraph_creation() {
        let walk = WalkType::Telegraph {
            dt: pos!(0.01),
            drift: dec!(0.05),
            volatility: pos!(0.2),
            lambda_up: pos!(0.5),
            lambda_down: pos!(0.3),
            vol_multiplier_up: Some(pos!(1.2)),
            vol_multiplier_down: Some(pos!(0.8)),
        };

        if let WalkType::Telegraph {
            dt,
            drift,
            volatility,
            lambda_up,
            lambda_down,
            vol_multiplier_up,
            vol_multiplier_down,
        } = walk
        {
            assert_eq!(dt, pos!(0.01));
            assert_eq!(drift, dec!(0.05));
            assert_eq!(volatility, pos!(0.2));
            assert_eq!(lambda_up, pos!(0.5));
            assert_eq!(lambda_down, pos!(0.3));
            assert_eq!(vol_multiplier_up, Some(pos!(1.2)));
            assert_eq!(vol_multiplier_down, Some(pos!(0.8)));
        } else {
            panic!("Expected Telegraph variant");
        }
    }

    #[test]
    fn test_display_brownian() {
        let walk = WalkType::Brownian {
            dt: pos!(0.01),
            drift: dec!(0.05),
            volatility: pos!(0.2),
        };

        let display = format!("{walk}");
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

        let display = format!("{walk}");
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

        let display = format!("{walk}");
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

        let display = format!("{walk}");
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

        let display = format!("{walk}");
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
        };

        let display = format!("{walk}");
        assert!(display.contains("Garch"));
        assert!(display.contains("dt: 0.01"));
        assert!(display.contains("drift: 0.05"));
        assert!(display.contains("volatility: 0.2"));
        assert!(display.contains("alpha: 0.1"));
        assert!(display.contains("beta: 0.8"));
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

        let display = format!("{walk}");
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

        let display = format!("{walk}");
        assert!(display.contains("Custom"));
        assert!(display.contains("dt: 0.01"));
        assert!(display.contains("drift: 0.05"));
        assert!(display.contains("volatility: 0.2"));
        assert!(display.contains("vov: 0.25"));
        assert!(display.contains("vol_speed: 0.15"));
        assert!(display.contains("vol_mean: 0.3"));
    }

    #[test]
    fn test_display_telegraph() {
        let walk = WalkType::Telegraph {
            dt: pos!(0.01),
            drift: dec!(0.05),
            volatility: pos!(0.2),
            lambda_up: pos!(0.5),
            lambda_down: pos!(0.3),
            vol_multiplier_up: Some(pos!(1.2)),
            vol_multiplier_down: Some(pos!(0.8)),
        };

        let display = format!("{walk}");
        assert!(display.contains("Telegraph"));
        assert!(display.contains("dt: 0.01"));
        assert!(display.contains("drift: 0.05"));
        assert!(display.contains("volatility: 0.2"));
        assert!(display.contains("lambda_up: 0.5"));
        assert!(display.contains("lambda_down: 0.3"));
        assert!(display.contains("vol_multiplier_up: Some(1.2)"));
        assert!(display.contains("vol_multiplier_down: Some(0.8)"));
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

        let display = format!("{walk}");
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

        let cloned = walk.clone();

        // Using Debug formatting to compare
        assert_eq!(format!("{walk:?}"), format!("{:?}", cloned));
    }
}

#[cfg(test)]
mod tests_serialize {
    use super::*;
    use crate::pos;
    use rust_decimal_macros::dec;
    use serde_json::{from_str, to_string};

    #[test]
    fn test_brownian_serialization() {
        let walk_type = WalkType::Brownian {
            dt: pos!(0.0027), // ~1/365 for daily
            drift: dec!(0.05),
            volatility: pos!(0.2),
        };

        let json = to_string(&walk_type).unwrap();

        // Check basic structure and field inclusion
        assert!(json.contains("\"Brownian\""));
        assert!(json.contains("\"dt\""));
        assert!(json.contains("\"drift\""));
        assert!(json.contains("\"volatility\""));

        // Check specific values
        assert!(json.contains("0.0027"));
        assert!(json.contains("0.05"));
        assert!(json.contains("0.2"));

        // Deserialize and verify
        let deserialized: WalkType = from_str(&json).unwrap();
        assert_eq!(walk_type, deserialized);
    }

    #[test]
    fn test_geometric_brownian_serialization() {
        let walk_type = WalkType::GeometricBrownian {
            dt: pos!(0.0192), // ~1/52 for weekly
            drift: dec!(0.07),
            volatility: pos!(0.25),
        };

        let json = to_string(&walk_type).unwrap();

        assert!(json.contains("\"GeometricBrownian\""));
        assert!(json.contains("\"dt\""));
        assert!(json.contains("\"drift\""));
        assert!(json.contains("\"volatility\""));

        assert!(json.contains("0.0192"));
        assert!(json.contains("0.07"));
        assert!(json.contains("0.25"));

        let deserialized: WalkType = from_str(&json).unwrap();
        assert_eq!(walk_type, deserialized);
    }

    #[test]
    fn test_log_returns_serialization() {
        let walk_type = WalkType::LogReturns {
            dt: pos!(0.0833), // ~1/12 for monthly
            expected_return: dec!(0.06),
            volatility: pos!(0.18),
            autocorrelation: Some(dec!(0.1)),
        };

        let json = to_string(&walk_type).unwrap();

        assert!(json.contains("\"LogReturns\""));
        assert!(json.contains("\"dt\""));
        assert!(json.contains("\"expected_return\""));
        assert!(json.contains("\"volatility\""));
        assert!(json.contains("\"autocorrelation\""));

        assert!(json.contains("0.0833"));
        assert!(json.contains("0.06"));
        assert!(json.contains("0.18"));
        assert!(json.contains("0.1"));

        let deserialized: WalkType = from_str(&json).unwrap();
        assert_eq!(walk_type, deserialized);
    }

    #[test]
    fn test_log_returns_with_none_autocorrelation() {
        let walk_type = WalkType::LogReturns {
            dt: pos!(0.0833),
            expected_return: dec!(0.06),
            volatility: pos!(0.18),
            autocorrelation: None,
        };

        let json = to_string(&walk_type).unwrap();

        // Check that autocorrelation is null
        assert!(json.contains("\"autocorrelation\":null"));

        let deserialized: WalkType = from_str(&json).unwrap();
        assert_eq!(walk_type, deserialized);
    }

    #[test]
    fn test_mean_reverting_serialization() {
        let walk_type = WalkType::MeanReverting {
            dt: pos!(0.0027),
            volatility: pos!(0.3),
            speed: pos!(0.5),
            mean: pos!(100.0),
        };

        let json = to_string(&walk_type).unwrap();

        assert!(json.contains("\"MeanReverting\""));
        assert!(json.contains("\"dt\""));
        assert!(json.contains("\"volatility\""));
        assert!(json.contains("\"speed\""));
        assert!(json.contains("\"mean\""));

        assert!(json.contains("0.0027"));
        assert!(json.contains("0.3"));
        assert!(json.contains("0.5"));
        assert!(json.contains("100"));

        let deserialized: WalkType = from_str(&json).unwrap();
        assert_eq!(walk_type, deserialized);
    }

    #[test]
    fn test_jump_diffusion_serialization() {
        let walk_type = WalkType::JumpDiffusion {
            dt: pos!(0.0027),
            drift: dec!(0.04),
            volatility: pos!(0.15),
            intensity: pos!(3.0),   // 3 jumps per year expected
            jump_mean: dec!(-0.05), // Negative mean for downward jumps
            jump_volatility: pos!(0.1),
        };

        let json = to_string(&walk_type).unwrap();

        assert!(json.contains("\"JumpDiffusion\""));
        assert!(json.contains("\"dt\""));
        assert!(json.contains("\"drift\""));
        assert!(json.contains("\"volatility\""));
        assert!(json.contains("\"intensity\""));
        assert!(json.contains("\"jump_mean\""));
        assert!(json.contains("\"jump_volatility\""));

        assert!(json.contains("0.0027"));
        assert!(json.contains("0.04"));
        assert!(json.contains("0.15"));
        assert!(json.contains("3"));
        assert!(json.contains("-0.05"));
        assert!(json.contains("0.1"));

        let deserialized: WalkType = from_str(&json).unwrap();
        assert_eq!(walk_type, deserialized);
    }

    #[test]
    fn test_garch_serialization() {
        let walk_type = WalkType::Garch {
            dt: pos!(0.0027),
            drift: dec!(0.03),
            volatility: pos!(0.2),
            alpha: pos!(0.1),
            beta: pos!(0.8),
        };

        let json = to_string(&walk_type).unwrap();

        assert!(json.contains("\"Garch\""));
        assert!(json.contains("\"dt\""));
        assert!(json.contains("\"drift\""));
        assert!(json.contains("\"volatility\""));
        assert!(json.contains("\"alpha\""));
        assert!(json.contains("\"beta\""));

        let deserialized: WalkType = from_str(&json).unwrap();
        assert_eq!(walk_type, deserialized);
    }

    #[test]
    fn test_heston_serialization() {
        let walk_type = WalkType::Heston {
            dt: pos!(0.0027),
            drift: dec!(0.05),
            volatility: pos!(0.2),
            kappa: pos!(1.5),
            theta: pos!(0.04),
            xi: pos!(0.3),
            rho: dec!(-0.7),
        };

        let json = to_string(&walk_type).unwrap();

        assert!(json.contains("\"Heston\""));
        assert!(json.contains("\"dt\""));
        assert!(json.contains("\"drift\""));
        assert!(json.contains("\"volatility\""));
        assert!(json.contains("\"kappa\""));
        assert!(json.contains("\"theta\""));
        assert!(json.contains("\"xi\""));
        assert!(json.contains("\"rho\""));

        let deserialized: WalkType = from_str(&json).unwrap();
        assert_eq!(walk_type, deserialized);
    }

    #[test]
    fn test_custom_serialization() {
        let walk_type = WalkType::Custom {
            dt: pos!(0.0027),
            drift: dec!(0.05),
            volatility: pos!(0.2),
            vov: pos!(0.1),
            vol_speed: pos!(0.5),
            vol_mean: pos!(0.2),
        };

        let json = to_string(&walk_type).unwrap();

        assert!(json.contains("\"Custom\""));
        assert!(json.contains("\"dt\""));
        assert!(json.contains("\"drift\""));
        assert!(json.contains("\"volatility\""));
        assert!(json.contains("\"vov\""));
        assert!(json.contains("\"vol_speed\""));
        assert!(json.contains("\"vol_mean\""));

        let deserialized: WalkType = from_str(&json).unwrap();
        assert_eq!(walk_type, deserialized);
    }

    #[test]
    fn test_telegraph_serialization() {
        let walk_type = WalkType::Telegraph {
            dt: pos!(0.0027),
            drift: dec!(0.05),
            volatility: pos!(0.2),
            lambda_up: pos!(0.5),
            lambda_down: pos!(0.3),
            vol_multiplier_up: Some(pos!(1.2)),
            vol_multiplier_down: Some(pos!(0.8)),
        };

        let json = to_string(&walk_type).unwrap();

        assert!(json.contains("\"Telegraph\""));
        assert!(json.contains("\"dt\""));
        assert!(json.contains("\"drift\""));
        assert!(json.contains("\"volatility\""));
        assert!(json.contains("\"lambda_up\""));
        assert!(json.contains("\"lambda_down\""));
        assert!(json.contains("\"vol_multiplier_up\""));
        assert!(json.contains("\"vol_multiplier_down\""));

        let deserialized: WalkType = from_str(&json).unwrap();
        assert_eq!(walk_type, deserialized);
    }

    #[test]
    fn test_historical_serialization() {
        let walk_type = WalkType::Historical {
            timeframe: TimeFrame::Day,
            prices: vec![
                pos!(100.0),
                pos!(101.5),
                pos!(99.8),
                pos!(102.3),
                pos!(103.1),
            ],
            symbol: None,
        };

        let json = to_string(&walk_type).unwrap();

        assert!(json.contains("\"Historical\""));
        assert!(json.contains("\"timeframe\""));
        assert!(json.contains("\"prices\""));
        assert!(json.contains("Day"));
        assert!(json.contains("100"));
        assert!(json.contains("101.5"));
        assert!(json.contains("99.8"));
        assert!(json.contains("102.3"));
        assert!(json.contains("103.1"));

        let deserialized: WalkType = from_str(&json).unwrap();
        assert_eq!(walk_type, deserialized);
    }

    #[test]
    fn test_deserialize_from_json_string() {
        let json = r#"{
            "GeometricBrownian": {
                "dt": 0.0027,
                "drift": 0.06,
                "volatility": 0.22
            }
        }"#;

        let walk_type: WalkType = from_str(json).unwrap();

        match walk_type {
            WalkType::GeometricBrownian {
                dt,
                drift,
                volatility,
            } => {
                assert_eq!(dt, pos!(0.0027));
                assert_eq!(drift, dec!(0.06));
                assert_eq!(volatility, pos!(0.22));
            }
            _ => panic!("Wrong variant deserialized"),
        }
    }

    #[test]
    fn test_negative_decimal_values() {
        let walk_type = WalkType::JumpDiffusion {
            dt: pos!(0.0027),
            drift: dec!(-0.03), // Negative drift
            volatility: pos!(0.15),
            intensity: pos!(2.0),
            jump_mean: dec!(-0.1), // Negative jump mean
            jump_volatility: pos!(0.05),
        };

        let json = to_string(&walk_type).unwrap();

        assert!(json.contains("-0.03"));
        assert!(json.contains("-0.1"));

        let deserialized: WalkType = from_str(&json).unwrap();
        assert_eq!(walk_type, deserialized);
    }
}
