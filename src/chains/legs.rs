/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 7/12/24
******************************************************************************/
use crate::chains::chain::OptionData;
use std::fmt::{Display, Formatter, Result};

#[derive(Debug, Clone)]
pub enum StrategyLegs<'a> {
    TwoLegs {
        first: &'a OptionData,
        second: &'a OptionData,
    },
    ThreeLegs {
        first: &'a OptionData,
        second: &'a OptionData,
        third: &'a OptionData,
    },
    FourLegs {
        first: &'a OptionData,
        second: &'a OptionData,
        third: &'a OptionData,
        fourth: &'a OptionData,
    },
    SixLegs {
        first: &'a OptionData,
        second: &'a OptionData,
        third: &'a OptionData,
        fourth: &'a OptionData,
        fifth: &'a OptionData,
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
                    "Two Legs Strategy:\n1st Leg: {}\n2nd Leg: {}\n3rd Leg: {}",
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
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;
    use super::*;
    use crate::Positive;
    use crate::{f2p, pos};

    // Helper function to create a test OptionData with a specific strike price
    fn create_test_option(strike: Decimal) -> OptionData {
        OptionData::new(pos!(strike), None, None, None, None, None, None, None, None)
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
                assert_eq!(first.strike_price, f2p!(100.0));
                assert_eq!(second.strike_price, f2p!(110.0));
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
                assert_eq!(first.strike_price, f2p!(100.0));
                assert_eq!(second.strike_price, f2p!(110.0));
                assert_eq!(third.strike_price, f2p!(120.0));
                assert_eq!(fourth.strike_price, f2p!(130.0));
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
                assert_eq!(first.strike_price, f2p!(100.0));
                assert_eq!(second.strike_price, f2p!(110.0));
                assert_eq!(third.strike_price, f2p!(120.0));
                assert_eq!(fourth.strike_price, f2p!(130.0));
                assert_eq!(fifth.strike_price, f2p!(140.0));
                assert_eq!(sixth.strike_price, f2p!(150.0));
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
                assert_eq!(first.strike_price, f2p!(100.0));
                assert_eq!(second.strike_price, f2p!(110.0));
            }
            _ => panic!("Expected TwoLegs variant after cloning"),
        }
    }
}
