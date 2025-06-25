/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 7/12/24
******************************************************************************/
use crate::chains::OptionData;
use std::fmt::{Display, Formatter, Result};

/// Represents the various configurations of option strategy legs with different complexities.
///
/// This enum provides structured representations for common option strategies that can consist
/// of two, three, four, or six legs. Each variant holds references to the corresponding `OptionData`
/// objects that make up the strategy. This allows for organized storage and manipulation of
/// multi-leg option strategies with varying complexity.
///
/// # Variants
///
/// * `TwoLegs` - Represents option strategies with two legs, such as vertical spreads,
///   straddles, or strangles.
///
/// * `ThreeLegs` - Represents option strategies with three legs, such as butterflies
///   or some ratio spreads.
///
/// * `FourLegs` - Represents option strategies with four legs, such as iron condors,
///   iron butterflies, or double calendars.
///
/// * `SixLegs` - Represents more complex option strategies with six legs, such as
///   double butterflies or advanced combinations of simpler strategies.
///
/// # Usage
///
/// This enum is typically used when implementing option strategy analysis, pricing models,
/// or visualizations where the number and configuration of legs determine the calculation
/// approach.
#[derive(Debug, Clone)]
pub enum StrategyLegs<'a> {
    /// Two-legged option strategy configuration
    ///
    /// Common examples include vertical spreads (bull/bear spreads), straddles, and strangles.
    TwoLegs {
        /// First option contract in the strategy
        first: &'a OptionData,
        /// Second option contract in the strategy
        second: &'a OptionData,
    },

    /// Three-legged option strategy configuration
    ///
    /// Common examples include butterfly spreads and some ratio spreads.
    ThreeLegs {
        /// First option contract in the strategy
        first: &'a OptionData,
        /// Second option contract in the strategy
        second: &'a OptionData,
        /// Third option contract in the strategy
        third: &'a OptionData,
    },

    /// Four-legged option strategy configuration
    ///
    /// Common examples include iron condors, iron butterflies, and condor spreads.
    FourLegs {
        /// First option contract in the strategy
        first: &'a OptionData,
        /// Second option contract in the strategy
        second: &'a OptionData,
        /// Third option contract in the strategy
        third: &'a OptionData,
        /// Fourth option contract in the strategy
        fourth: &'a OptionData,
    },

    /// Six-legged option strategy configuration
    ///
    /// Used for complex strategies like double butterflies or combinations of simpler strategies.
    SixLegs {
        /// First option contract in the strategy
        first: &'a OptionData,
        /// Second option contract in the strategy
        second: &'a OptionData,
        /// Third option contract in the strategy
        third: &'a OptionData,
        /// Fourth option contract in the strategy
        fourth: &'a OptionData,
        /// Fifth option contract in the strategy
        fifth: &'a OptionData,
        /// Sixth option contract in the strategy
        sixth: &'a OptionData,
    },
}

impl Display for StrategyLegs<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            StrategyLegs::TwoLegs { first, second } => {
                write!(
                    f,
                    "Two Legs Strategy:\n1st Leg: {}\n2nd Leg: {}",
                    first, second
                )
            }
            StrategyLegs::ThreeLegs {
                first,
                second,
                third,
            } => {
                write!(
                    f,
                    "Three Legs Strategy:\n1st Leg: {}\n2nd Leg: {}\n3rd Leg: {}",
                    first, second, third
                )
            }
            StrategyLegs::FourLegs {
                first,
                second,
                third,
                fourth,
            } => {
                write!(
                    f,
                    "Four Legs Strategy:\n1st Leg: {}\n2nd Leg: {}\n3rd Leg: {}\n4th Leg: {}",
                    first, second, third, fourth
                )
            }
            StrategyLegs::SixLegs {
                first,
                second,
                third,
                fourth,
                fifth,
                sixth,
            } => {
                write!(
                    f,
                    "Six Legs Strategy:\n1st Leg: {}\n2nd Leg: {}\n3rd Leg: {}\n4th Leg: {}\n5th Leg: {}\n6th Leg: {}",
                    first, second, third, fourth, fifth, sixth
                )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pos;
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;

    // Helper function to create a test OptionData with a specific strike price
    fn create_test_option(strike: Decimal) -> OptionData {
        OptionData::new(
            strike.into(),
            None,
            None,
            None,
            None,
            pos!(0.2),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
    }

    #[test]
    fn test_two_legs_creation() {
        let option1 = create_test_option(dec!(100.0));
        let option2 = create_test_option(dec!(110.0));

        let strategy = StrategyLegs::TwoLegs {
            first: &option1,
            second: &option2,
        };

        match strategy {
            StrategyLegs::TwoLegs { first, second } => {
                assert_eq!(first.strike_price, pos!(100.0));
                assert_eq!(second.strike_price, pos!(110.0));
            }
            _ => panic!("Expected TwoLegs variant"),
        }
    }

    #[test]
    fn test_four_legs_creation() {
        let option1 = create_test_option(dec!(100.0));
        let option2 = create_test_option(dec!(110.0));
        let option3 = create_test_option(dec!(120.0));
        let option4 = create_test_option(dec!(130.0));

        let strategy = StrategyLegs::FourLegs {
            first: &option1,
            second: &option2,
            third: &option3,
            fourth: &option4,
        };

        match strategy {
            StrategyLegs::FourLegs {
                first,
                second,
                third,
                fourth,
            } => {
                assert_eq!(first.strike_price, pos!(100.0));
                assert_eq!(second.strike_price, pos!(110.0));
                assert_eq!(third.strike_price, pos!(120.0));
                assert_eq!(fourth.strike_price, pos!(130.0));
            }
            _ => panic!("Expected FourLegs variant"),
        }
    }

    #[test]
    fn test_six_legs_creation() {
        let option1 = create_test_option(dec!(100.0));
        let option2 = create_test_option(dec!(110.0));
        let option3 = create_test_option(dec!(120.0));
        let option4 = create_test_option(dec!(130.0));
        let option5 = create_test_option(dec!(140.0));
        let option6 = create_test_option(dec!(150.0));

        let strategy = StrategyLegs::SixLegs {
            first: &option1,
            second: &option2,
            third: &option3,
            fourth: &option4,
            fifth: &option5,
            sixth: &option6,
        };

        match strategy {
            StrategyLegs::SixLegs {
                first,
                second,
                third,
                fourth,
                fifth,
                sixth,
            } => {
                assert_eq!(first.strike_price, pos!(100.0));
                assert_eq!(second.strike_price, pos!(110.0));
                assert_eq!(third.strike_price, pos!(120.0));
                assert_eq!(fourth.strike_price, pos!(130.0));
                assert_eq!(fifth.strike_price, pos!(140.0));
                assert_eq!(sixth.strike_price, pos!(150.0));
            }
            _ => panic!("Expected SixLegs variant"),
        }
    }

    #[test]
    fn test_display_two_legs() {
        let option1 = create_test_option(dec!(100.0));
        let option2 = create_test_option(dec!(110.0));

        let strategy = StrategyLegs::TwoLegs {
            first: &option1,
            second: &option2,
        };

        let display_string = format!("{}", strategy);
        assert!(display_string.contains("Two Legs Strategy"));
        assert!(display_string.contains("1st Leg"));
        assert!(display_string.contains("2nd Leg"));
    }

    #[test]
    fn test_display_four_legs() {
        let option1 = create_test_option(dec!(100.0));
        let option2 = create_test_option(dec!(110.0));
        let option3 = create_test_option(dec!(120.0));
        let option4 = create_test_option(dec!(130.0));

        let strategy = StrategyLegs::FourLegs {
            first: &option1,
            second: &option2,
            third: &option3,
            fourth: &option4,
        };

        let display_string = format!("{}", strategy);
        assert!(display_string.contains("Four Legs Strategy"));
        assert!(display_string.contains("1st Leg"));
        assert!(display_string.contains("2nd Leg"));
        assert!(display_string.contains("3rd Leg"));
        assert!(display_string.contains("4th Leg"));
    }

    #[test]
    fn test_display_six_legs() {
        let option1 = create_test_option(dec!(100.0));
        let option2 = create_test_option(dec!(110.0));
        let option3 = create_test_option(dec!(120.0));
        let option4 = create_test_option(dec!(130.0));
        let option5 = create_test_option(dec!(140.0));
        let option6 = create_test_option(dec!(150.0));

        let strategy = StrategyLegs::SixLegs {
            first: &option1,
            second: &option2,
            third: &option3,
            fourth: &option4,
            fifth: &option5,
            sixth: &option6,
        };

        let display_string = format!("{}", strategy);
        assert!(display_string.contains("Six Legs Strategy"));
        assert!(display_string.contains("1st Leg"));
        assert!(display_string.contains("2nd Leg"));
        assert!(display_string.contains("3rd Leg"));
        assert!(display_string.contains("4th Leg"));
        assert!(display_string.contains("5th Leg"));
        assert!(display_string.contains("6th Leg"));
    }

    #[test]
    fn test_clone() {
        let option1 = create_test_option(dec!(100.0));
        let option2 = create_test_option(dec!(110.0));

        let strategy = StrategyLegs::TwoLegs {
            first: &option1,
            second: &option2,
        };

        let cloned_strategy = strategy.clone();

        match cloned_strategy {
            StrategyLegs::TwoLegs { first, second } => {
                assert_eq!(first.strike_price, pos!(100.0));
                assert_eq!(second.strike_price, pos!(110.0));
            }
            _ => panic!("Expected TwoLegs variant after cloning"),
        }
    }

    #[test]
    fn test_display_three_legs() {
        let option1 = create_test_option(dec!(100.0));
        let option2 = create_test_option(dec!(110.0));
        let option3 = create_test_option(dec!(120.0));

        let strategy = StrategyLegs::ThreeLegs {
            first: &option1,
            second: &option2,
            third: &option3,
        };

        let display_string = format!("{}", strategy);
        assert!(display_string.contains("Three Legs Strategy")); // Note: This is actually a bug in the Display implementation
        assert!(display_string.contains("1st Leg"));
        assert!(display_string.contains("2nd Leg"));
        assert!(display_string.contains("3rd Leg"));

        // Verify strikes are displayed in correct order
        let lines: Vec<&str> = display_string.lines().collect();
        assert!(lines[1].contains(&option1.strike_price.to_string()));
        assert!(lines[2].contains(&option2.strike_price.to_string()));
        assert!(lines[3].contains(&option3.strike_price.to_string()));
    }
}
