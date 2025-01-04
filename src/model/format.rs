/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 20/8/24
******************************************************************************/
use crate::model::option::{ExoticParams, Options};
use crate::model::position::Position;
use crate::model::types::{
    AsianAveragingType, BarrierType, BinaryType, ExpirationDate, LookbackType, OptionStyle,
    OptionType, Side,
};
use crate::strategies::base::Strategy;
use chrono::{Duration, Utc};
use rust_decimal_macros::dec;
use std::fmt;

impl fmt::Display for Options {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "{} {} {}",
            self.side, self.option_style, self.option_type
        )?;
        writeln!(
            f,
            "Underlying: {} @ ${:.2}",
            self.underlying_symbol, self.underlying_price
        )?;
        writeln!(f, "Strike: ${:.2}", self.strike_price)?;
        writeln!(f, "Expiration: {}", self.expiration_date)?;
        writeln!(
            f,
            "Implied Volatility: {:.2}%",
            self.implied_volatility * 100.0
        )?;
        writeln!(f, "Quantity: {}", self.quantity)?;
        writeln!(
            f,
            "Risk-free Rate: {:.2}%",
            self.risk_free_rate * dec!(100.0)
        )?;
        write!(f, "Dividend Yield: {:.2}%", self.dividend_yield * 100.0)?;
        if let Some(exotic) = &self.exotic_params {
            write!(f, "\nExotic Parameters: {:?}", exotic)?;
        }
        Ok(())
    }
}

impl fmt::Debug for Options {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Options")
            .field("option_type", &self.option_type)
            .field("side", &self.side)
            .field("underlying_symbol", &self.underlying_symbol)
            .field("strike_price", &self.strike_price)
            .field("expiration_date", &self.expiration_date)
            .field("implied_volatility", &self.implied_volatility)
            .field("quantity", &self.quantity)
            .field("underlying_price", &self.underlying_price)
            .field("risk_free_rate", &self.risk_free_rate)
            .field("option_style", &self.option_style)
            .field("dividend_yield", &self.dividend_yield)
            .field("exotic_params", &self.exotic_params)
            .finish()
    }
}

impl fmt::Display for ExoticParams {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut fields = vec![];

        if let Some(ref prices) = self.spot_prices {
            fields.push(format!("Spot Prices: {:?}", prices));
        }

        if let Some(min) = self.spot_min {
            fields.push(format!("Spot Min: {:.2}", min));
        }

        if let Some(max) = self.spot_max {
            fields.push(format!("Spot Max: {:.2}", max));
        }

        write!(f, "{}", fields.join(", "))
    }
}

impl fmt::Display for ExpirationDate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExpirationDate::Days(days) => {
                let duration = Duration::days((*days).to_i64());
                let expiration = Utc::now() + duration;
                write!(f, "{}", expiration.format("%Y-%m-%d %H:%M:%S UTC"))
            }
            ExpirationDate::DateTime(date_time) => {
                write!(f, "{}", date_time.format("%Y-%m-%d %H:%M:%S UTC"))
            }
        }
    }
}

impl fmt::Debug for ExpirationDate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExpirationDate::Days(days) => write!(f, "ExpirationDate::Days({:.2})", days),
            ExpirationDate::DateTime(date_time) => {
                write!(f, "ExpirationDate::DateTime({})", date_time)
            }
        }
    }
}

impl fmt::Display for Side {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Side::Long => write!(f, "Long"),
            Side::Short => write!(f, "Short"),
        }
    }
}

impl fmt::Debug for Side {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Side::Long => write!(f, "Side::Long"),
            Side::Short => write!(f, "Side::Short"),
        }
    }
}

impl fmt::Display for OptionStyle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OptionStyle::Call => write!(f, "Call"),
            OptionStyle::Put => write!(f, "Put"),
        }
    }
}

impl fmt::Debug for OptionStyle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OptionStyle::Call => write!(f, "OptionStyle::Call"),
            OptionStyle::Put => write!(f, "OptionStyle::Put"),
        }
    }
}

impl fmt::Display for OptionType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            OptionType::European => write!(f, "European Option"),
            OptionType::American => write!(f, "American Option"),
            OptionType::Bermuda { exercise_dates } => {
                write!(f, "Bermuda Option (Exercise Dates: {:?})", exercise_dates)
            }
            OptionType::Asian { averaging_type } => {
                write!(f, "Asian Option (Averaging Type: {})", averaging_type)
            }
            OptionType::Barrier {
                barrier_type,
                barrier_level,
            } => write!(
                f,
                "Barrier Option (Type: {}, Level: {})",
                barrier_type, barrier_level
            ),
            OptionType::Binary { binary_type } => {
                write!(f, "Binary Option (Type: {})", binary_type)
            }
            OptionType::Lookback { lookback_type } => {
                write!(f, "Lookback Option (Type: {})", lookback_type)
            }
            OptionType::Compound { underlying_option } => {
                write!(f, "Compound Option (Underlying: {})", underlying_option)
            }
            OptionType::Chooser { choice_date } => {
                write!(f, "Chooser Option (Choice Date: {})", choice_date)
            }
            OptionType::Cliquet { reset_dates } => {
                write!(f, "Cliquet Option (Reset Dates: {:?})", reset_dates)
            }
            OptionType::Rainbow { num_assets } => {
                write!(f, "Rainbow Option (Number of Assets: {})", num_assets)
            }
            OptionType::Spread { second_asset } => {
                write!(f, "Spread Option (Second Asset: {})", second_asset)
            }
            OptionType::Quanto { exchange_rate } => {
                write!(f, "Quanto Option (Exchange Rate: {})", exchange_rate)
            }
            OptionType::Exchange { second_asset } => {
                write!(f, "Exchange Option (Second Asset: {})", second_asset)
            }
            OptionType::Power { exponent } => write!(f, "Power Option (Exponent: {})", exponent),
        }
    }
}

impl fmt::Display for AsianAveragingType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AsianAveragingType::Arithmetic => write!(f, "Arithmetic Averaging"),
            AsianAveragingType::Geometric => write!(f, "Geometric Averaging"),
        }
    }
}

impl fmt::Display for BarrierType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BarrierType::UpAndIn => write!(f, "Up-And-In Barrier"),
            BarrierType::UpAndOut => write!(f, "Up-And-Out Barrier"),
            BarrierType::DownAndIn => write!(f, "Down-And-In Barrier"),
            BarrierType::DownAndOut => write!(f, "Down-And-Out Barrier"),
        }
    }
}

impl fmt::Display for BinaryType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BinaryType::CashOrNothing => write!(f, "Cash-Or-Nothing Binary Option"),
            BinaryType::AssetOrNothing => write!(f, "Asset-Or-Nothing Binary Option"),
        }
    }
}

impl fmt::Display for LookbackType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LookbackType::FixedStrike => write!(f, "Fixed-Strike Lookback Option"),
            LookbackType::FloatingStrike => write!(f, "Floating-Strike Lookback Option"),
        }
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Position Details:")?;
        writeln!(f, "Option: {}", self.option)?;
        writeln!(f, "Premium per contract: ${:.2}", self.premium)?;
        writeln!(f, "Date: {}", self.date)?;
        writeln!(f, "Open Fee per contract: ${:.2}", self.open_fee)?;
        write!(f, "Close Fee per contract: ${:.2}", self.close_fee)
    }
}

impl fmt::Debug for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Position")
            .field("option", &self.option)
            .field("premium", &self.premium)
            .field("date", &self.date)
            .field("open_fee", &self.open_fee)
            .field("close_fee", &self.close_fee)
            .finish()
    }
}

impl fmt::Display for Strategy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Strategy: {}", self.name)?;
        writeln!(f, "Type: {:?}", self.kind)?;
        writeln!(f, "Description: {}", self.description)?;
        writeln!(f, "Legs:")?;
        for leg in &self.legs {
            writeln!(f, "  {}", leg)?;
        }
        if let Some(max_profit) = self.max_profit {
            writeln!(f, "Max Profit: ${:.2}", max_profit)?;
        }
        if let Some(max_loss) = self.max_loss {
            writeln!(f, "Max Loss: ${:.2}", max_loss)?;
        }
        writeln!(f, "Break-even Points:")?;
        for point in &self.break_even_points {
            writeln!(f, "  ${:.2}", point)?;
        }
        Ok(())
    }
}

impl fmt::Debug for Strategy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Strategy")
            .field("name", &self.name)
            .field("kind", &self.kind)
            .field("description", &self.description)
            .field("legs", &self.legs)
            .field("max_profit", &self.max_profit)
            .field("max_loss", &self.max_loss)
            .field("break_even_points", &self.break_even_points)
            .finish()
    }
}

#[cfg(test)]
mod tests_options {
    use super::*;
    use crate::model::types::BarrierType;
    use crate::pos;
    use chrono::{NaiveDate, TimeZone, Utc};

    #[test]
    fn test_debug_options() {
        let naive_date = NaiveDate::from_ymd_opt(2024, 8, 8)
            .expect("Invalid date")
            .and_hms_opt(0, 0, 0)
            .expect("Invalid time");

        let options = Options {
            option_type: OptionType::European,
            side: Side::Long,
            underlying_symbol: "AAPL".to_string(),
            strike_price: pos!(150.0),
            expiration_date: ExpirationDate::DateTime(Utc.from_utc_datetime(&naive_date)),
            implied_volatility: pos!(0.25),
            quantity: pos!(10.0),
            underlying_price: pos!(155.0),
            risk_free_rate: dec!(0.01),
            option_style: OptionStyle::Call,
            dividend_yield: pos!(0.02),
            exotic_params: None,
        };

        let debug_output = format!("{:?}", options);
        let expected_output = "Options { \
            option_type: European, \
            side: Side::Long, \
            underlying_symbol: \"AAPL\", \
            strike_price: 150, \
            expiration_date: ExpirationDate::DateTime(2024-08-08 00:00:00 UTC), \
            implied_volatility: 0.25, \
            quantity: 10, \
            underlying_price: 155, \
            risk_free_rate: 0.01, \
            option_style: OptionStyle::Call, \
            dividend_yield: 0.02, \
            exotic_params: None \
        }";

        assert_eq!(debug_output, expected_output);
    }

    #[test]
    fn test_display_options() {
        let naive_date = NaiveDate::from_ymd_opt(2024, 8, 8)
            .expect("Invalid date")
            .and_hms_opt(0, 0, 0)
            .expect("Invalid time");

        let options = Options {
            option_type: OptionType::European,
            side: Side::Long,
            underlying_symbol: "AAPL".to_string(),
            strike_price: pos!(150.0),
            expiration_date: ExpirationDate::DateTime(Utc.from_utc_datetime(&naive_date)),
            implied_volatility: pos!(0.25),
            quantity: pos!(10.0),
            underlying_price: pos!(155.0),
            risk_free_rate: dec!(0.01),
            option_style: OptionStyle::Call,
            dividend_yield: pos!(0.02),
            exotic_params: None,
        };

        let display_output = format!("{}", options);
        let expected_output = "\
            Long Call European Option\n\
            Underlying: AAPL @ $155.00\n\
            Strike: $150.00\n\
            Expiration: 2024-08-08 00:00:00 UTC\n\
            Implied Volatility: 25.00%\n\
            Quantity: 10\n\
            Risk-free Rate: 1.00%\n\
            Dividend Yield: 2.00%";

        assert_eq!(display_output, expected_output);
    }

    #[test]
    fn test_display_options_with_exotic_params() {
        let exotic_params = ExoticParams {
            spot_prices: None,
            spot_min: None,
            spot_max: None,
        };
        let naive_date = NaiveDate::from_ymd_opt(2024, 8, 8)
            .expect("Invalid date")
            .and_hms_opt(0, 0, 0)
            .expect("Invalid time");

        let options = Options {
            option_type: OptionType::Barrier {
                barrier_type: BarrierType::UpAndIn,
                barrier_level: 100.0,
            },
            side: Side::Short,
            underlying_symbol: "GOOGL".to_string(),
            strike_price: pos!(2000.0),
            expiration_date: ExpirationDate::DateTime(Utc.from_utc_datetime(&naive_date)),
            implied_volatility: pos!(0.30),
            quantity: pos!(5.0),
            underlying_price: pos!(1900.0),
            risk_free_rate: dec!(0.015),
            option_style: OptionStyle::Call,
            dividend_yield: pos!(0.01),
            exotic_params: Some(exotic_params),
        };

        let display_output = format!("{}", options);
        let expected_output = "\
            Short Call Barrier Option (Type: Up-And-In Barrier, Level: 100)\n\
            Underlying: GOOGL @ $1900.00\n\
            Strike: $2000.00\n\
            Expiration: 2024-08-08 00:00:00 UTC\n\
            Implied Volatility: 30.00%\n\
            Quantity: 5\n\
            Risk-free Rate: 1.50%\n\
            Dividend Yield: 1.00%\n\
            Exotic Parameters: ExoticParams { spot_prices: None, spot_min: None, spot_max: None }";

        assert_eq!(display_output, expected_output);
    }
}

#[cfg(test)]
mod tests_expiration_date {
    use super::*;
    use crate::pos;
    use chrono::{Duration, NaiveDate, TimeZone};
    use tracing::info;

    #[test]
    fn test_display_days() {
        let expiration = ExpirationDate::Days(pos!(30.5));
        let display_string = format!("{}", expiration);
        assert!(display_string.contains("UTC"));
    }

    #[test]
    fn test_display_datetime() {
        let future_date = Utc::now() + Duration::days(15) + Duration::minutes(1);
        let expiration = ExpirationDate::DateTime(future_date);
        let display_string = format!("{}", expiration);
        assert!(display_string.contains("UTC"));
        assert!(display_string.contains(&future_date.format("%Y-%m-%d %H:%M:%S").to_string()));
    }

    #[test]
    fn test_debug_days() {
        let expiration = ExpirationDate::Days(pos!(45.75));
        let debug_string = format!("{:?}", expiration);
        assert_eq!(debug_string, "ExpirationDate::Days(45.75)");
    }

    #[test]
    fn test_debug_datetime() {
        let date = Utc.from_utc_datetime(
            &NaiveDate::from_ymd_opt(2023, 12, 31)
                .expect("Invalid date")
                .and_hms_opt(23, 59, 59)
                .expect("Invalid Time"),
        );

        let expiration = ExpirationDate::DateTime(date);
        let debug_string = format!("{:?}", expiration);
        assert_eq!(
            debug_string,
            "ExpirationDate::DateTime(2023-12-31 23:59:59 UTC)"
        );
    }

    #[test]
    fn test_display_past_date() {
        let past_date = Utc::now() - Duration::days(5);
        let expiration = ExpirationDate::DateTime(past_date);
        let display_string = format!("{}", expiration);
        assert!(display_string.contains("UTC"));
        assert!(display_string.contains(&past_date.format("%Y-%m-%d %H:%M:%S").to_string()));
    }

    #[test]
    fn test_display_today() {
        let today = Utc::now();
        let expiration = ExpirationDate::DateTime(today);
        let display_string = format!("{}", expiration);
        info!("{}", display_string);
        assert!(display_string.contains(&today.format("%Y-%m-%d %H:%M:%S").to_string()));
    }
}

#[cfg(test)]
mod tests_side_option_style_display_debug {
    use super::*;

    #[test]
    fn test_side_display() {
        assert_eq!(format!("{}", Side::Long), "Long");
        assert_eq!(format!("{}", Side::Short), "Short");
    }

    #[test]
    fn test_side_debug() {
        assert_eq!(format!("{:?}", Side::Long), "Side::Long");
        assert_eq!(format!("{:?}", Side::Short), "Side::Short");
    }

    #[test]
    fn test_option_style_display() {
        assert_eq!(format!("{}", OptionStyle::Call), "Call");
        assert_eq!(format!("{}", OptionStyle::Put), "Put");
    }

    #[test]
    fn test_option_style_debug() {
        assert_eq!(format!("{:?}", OptionStyle::Call), "OptionStyle::Call");
        assert_eq!(format!("{:?}", OptionStyle::Put), "OptionStyle::Put");
    }
}

#[cfg(test)]
mod tests_option_type_display_debug {
    use super::*;

    #[test]
    fn test_debug_european_option() {
        let option = OptionType::European;
        let debug_output = format!("{:?}", option);
        assert_eq!(debug_output, "European");
    }

    #[test]
    fn test_display_european_option() {
        let option = OptionType::European;
        let display_output = format!("{}", option);
        assert_eq!(display_output, "European Option");
    }

    #[test]
    fn test_debug_american_option() {
        let option = OptionType::American;
        let debug_output = format!("{:?}", option);
        assert_eq!(debug_output, "American");
    }

    #[test]
    fn test_display_american_option() {
        let option = OptionType::American;
        let display_output = format!("{}", option);
        assert_eq!(display_output, "American Option");
    }

    #[test]
    fn test_debug_bermuda_option() {
        let option = OptionType::Bermuda {
            exercise_dates: vec![1.0, 2.0, 3.0],
        };
        let debug_output = format!("{:?}", option);
        assert_eq!(debug_output, "Bermuda { exercise_dates: [1.0, 2.0, 3.0] }");
    }

    #[test]
    fn test_display_bermuda_option() {
        let option = OptionType::Bermuda {
            exercise_dates: vec![1.0, 2.0, 3.0],
        };
        let display_output = format!("{}", option);
        assert_eq!(
            display_output,
            "Bermuda Option (Exercise Dates: [1.0, 2.0, 3.0])"
        );
    }

    #[test]
    fn test_debug_asian_option() {
        let option = OptionType::Asian {
            averaging_type: AsianAveragingType::Arithmetic,
        };
        let debug_output = format!("{:?}", option);
        assert_eq!(debug_output, "Asian { averaging_type: Arithmetic }");
    }

    #[test]
    fn test_display_asian_option() {
        let option = OptionType::Asian {
            averaging_type: AsianAveragingType::Arithmetic,
        };
        let display_output = format!("{}", option);
        assert_eq!(
            display_output,
            "Asian Option (Averaging Type: Arithmetic Averaging)"
        );
    }

    #[test]
    fn test_debug_barrier_option() {
        let option = OptionType::Barrier {
            barrier_type: BarrierType::UpAndIn,
            barrier_level: 100.0,
        };
        let debug_output = format!("{:?}", option);
        assert_eq!(
            debug_output,
            "Barrier { barrier_type: UpAndIn, barrier_level: 100.0 }"
        );
    }

    #[test]
    fn test_display_barrier_option() {
        let option = OptionType::Barrier {
            barrier_type: BarrierType::UpAndIn,
            barrier_level: 100.0,
        };
        let display_output = format!("{}", option);
        assert_eq!(
            display_output,
            "Barrier Option (Type: Up-And-In Barrier, Level: 100)"
        );
    }

    #[test]
    fn test_debug_binary_option() {
        let option = OptionType::Binary {
            binary_type: BinaryType::CashOrNothing,
        };
        let debug_output = format!("{:?}", option);
        assert_eq!(debug_output, "Binary { binary_type: CashOrNothing }");
    }

    #[test]
    fn test_display_binary_option() {
        let option = OptionType::Binary {
            binary_type: BinaryType::CashOrNothing,
        };
        let display_output = format!("{}", option);
        assert_eq!(
            display_output,
            "Binary Option (Type: Cash-Or-Nothing Binary Option)"
        );
    }

    #[test]
    fn test_debug_lookback_option() {
        let option = OptionType::Lookback {
            lookback_type: LookbackType::FixedStrike,
        };
        let debug_output = format!("{:?}", option);
        assert_eq!(debug_output, "Lookback { lookback_type: FixedStrike }");
    }

    #[test]
    fn test_display_lookback_option() {
        let option = OptionType::Lookback {
            lookback_type: LookbackType::FixedStrike,
        };
        let display_output = format!("{}", option);
        assert_eq!(
            display_output,
            "Lookback Option (Type: Fixed-Strike Lookback Option)"
        );
    }

    #[test]
    fn test_debug_compound_option() {
        let option = OptionType::Compound {
            underlying_option: Box::new(OptionType::European),
        };
        let debug_output = format!("{:?}", option);
        assert_eq!(debug_output, "Compound { underlying_option: European }");
    }

    #[test]
    fn test_display_compound_option() {
        let option = OptionType::Compound {
            underlying_option: Box::new(OptionType::European),
        };
        let display_output = format!("{}", option);
        assert_eq!(
            display_output,
            "Compound Option (Underlying: European Option)"
        );
    }

    #[test]
    fn test_debug_chooser_option() {
        let option = OptionType::Chooser { choice_date: 2.0 };
        let debug_output = format!("{:?}", option);
        assert_eq!(debug_output, "Chooser { choice_date: 2.0 }");
    }

    #[test]
    fn test_display_chooser_option() {
        let option = OptionType::Chooser { choice_date: 2.0 };
        let display_output = format!("{}", option);
        assert_eq!(display_output, "Chooser Option (Choice Date: 2)");
    }

    #[test]
    fn test_debug_cliquet_option() {
        let option = OptionType::Cliquet {
            reset_dates: vec![1.0, 2.0],
        };
        let debug_output = format!("{:?}", option);
        assert_eq!(debug_output, "Cliquet { reset_dates: [1.0, 2.0] }");
    }

    #[test]
    fn test_display_cliquet_option() {
        let option = OptionType::Cliquet {
            reset_dates: vec![1.0, 2.0],
        };
        let display_output = format!("{}", option);
        assert_eq!(display_output, "Cliquet Option (Reset Dates: [1.0, 2.0])");
    }

    #[test]
    fn test_debug_rainbow_option() {
        let option = OptionType::Rainbow { num_assets: 3 };
        let debug_output = format!("{:?}", option);
        assert_eq!(debug_output, "Rainbow { num_assets: 3 }");
    }

    #[test]
    fn test_display_rainbow_option() {
        let option = OptionType::Rainbow { num_assets: 3 };
        let display_output = format!("{}", option);
        assert_eq!(display_output, "Rainbow Option (Number of Assets: 3)");
    }

    #[test]
    fn test_debug_spread_option() {
        let option = OptionType::Spread { second_asset: 50.0 };
        let debug_output = format!("{:?}", option);
        assert_eq!(debug_output, "Spread { second_asset: 50.0 }");
    }

    #[test]
    fn test_display_spread_option() {
        let option = OptionType::Spread { second_asset: 50.0 };
        let display_output = format!("{}", option);
        assert_eq!(display_output, "Spread Option (Second Asset: 50)");
    }

    #[test]
    fn test_debug_quanto_option() {
        let option = OptionType::Quanto { exchange_rate: 1.2 };
        let debug_output = format!("{:?}", option);
        assert_eq!(debug_output, "Quanto { exchange_rate: 1.2 }");
    }

    #[test]
    fn test_display_quanto_option() {
        let option = OptionType::Quanto { exchange_rate: 1.2 };
        let display_output = format!("{}", option);
        assert_eq!(display_output, "Quanto Option (Exchange Rate: 1.2)");
    }

    #[test]
    fn test_debug_exchange_option() {
        let option = OptionType::Exchange { second_asset: 75.0 };
        let debug_output = format!("{:?}", option);
        assert_eq!(debug_output, "Exchange { second_asset: 75.0 }");
    }

    #[test]
    fn test_display_exchange_option() {
        let option = OptionType::Exchange { second_asset: 75.0 };
        let display_output = format!("{}", option);
        assert_eq!(display_output, "Exchange Option (Second Asset: 75)");
    }

    #[test]
    fn test_debug_power_option() {
        let option = OptionType::Power { exponent: 2.5 };
        let debug_output = format!("{:?}", option);
        assert_eq!(debug_output, "Power { exponent: 2.5 }");
    }

    #[test]
    fn test_display_power_option() {
        let option = OptionType::Power { exponent: 2.5 };
        let display_output = format!("{}", option);
        assert_eq!(display_output, "Power Option (Exponent: 2.5)");
    }
}

#[cfg(test)]
mod tests_position_type_display_debug {
    use super::*;
    use crate::pos;
    use chrono::{DateTime, NaiveDate, TimeZone};

    fn get_option() -> (Options, DateTime<Utc>) {
        let naive_date = NaiveDate::from_ymd_opt(2024, 8, 8)
            .expect("Invalid date")
            .and_hms_opt(0, 0, 0)
            .expect("Invalid time");

        (
            Options {
                option_type: OptionType::European,
                side: Side::Long,
                underlying_symbol: "AAPL".to_string(),
                strike_price: pos!(150.0),
                expiration_date: ExpirationDate::DateTime(Utc.from_utc_datetime(&naive_date)),
                implied_volatility: pos!(0.25),
                quantity: pos!(10.0),
                underlying_price: pos!(155.0),
                risk_free_rate: dec!(0.01),
                option_style: OptionStyle::Call,
                dividend_yield: pos!(0.02),
                exotic_params: None,
            },
            Utc.from_utc_datetime(&naive_date),
        )
    }

    #[test]
    fn test_position_display() {
        let (option, naive_date) = get_option();
        let position = Position {
            option,
            premium: 5.75,
            date: naive_date,
            open_fee: 0.50,
            close_fee: 0.45,
        };

        let expected_display = "Position Details:\n\
                Option: Long Call European Option\n\
                Underlying: AAPL @ $155.00\n\
                Strike: $150.00\n\
                Expiration: 2024-08-08 00:00:00 UTC\n\
                Implied Volatility: 25.00%\n\
                Quantity: 10\n\
                Risk-free Rate: 1.00%\n\
                Dividend Yield: 2.00%\n\
                Premium per contract: $5.75\n\
                Date: 2024-08-08 00:00:00 UTC\n\
                Open Fee per contract: $0.50\n\
                Close Fee per contract: $0.45";

        assert_eq!(format!("{}", position), expected_display);
    }

    #[test]
    fn test_position_debug() {
        let (option, naive_date) = get_option();

        let position = Position {
            option,
            premium: 5.75,
            date: naive_date,
            open_fee: 0.50,
            close_fee: 0.45,
        };

        let expected_debug = "Position { \
        option: Options { \
            option_type: European, \
            side: Side::Long, \
            underlying_symbol: \"AAPL\", \
            strike_price: 150, \
            expiration_date: ExpirationDate::DateTime(2024-08-08 00:00:00 UTC), \
            implied_volatility: 0.25, \
            quantity: 10, \
            underlying_price: 155, \
            risk_free_rate: 0.01, \
            option_style: OptionStyle::Call, \
            dividend_yield: 0.02, \
            exotic_params: None \
        }, \
        premium: 5.75, \
        date: 2024-08-08T00:00:00Z, \
        open_fee: 0.5, \
        close_fee: 0.45 \
    }";

        assert_eq!(format!("{:?}", position), expected_debug);
    }
}

#[cfg(test)]
mod tests_strategy_type_display_debug {
    use super::*;
    use crate::model::utils::create_sample_option_with_date;
    use crate::pos;
    use crate::strategies::base::StrategyType;
    use chrono::{NaiveDate, TimeZone};

    #[test]
    fn test_strategy_display() {
        let naive_date = NaiveDate::from_ymd_opt(2024, 8, 8)
            .expect("Invalid date")
            .and_hms_opt(0, 0, 0)
            .expect("Invalid time");
        let strategy = Strategy {
            name: "Bull Call Spread".to_string(),
            kind: StrategyType::BullCallSpread,
            description: "A bullish options strategy".to_string(),
            legs: vec![
                Position::new(
                    create_sample_option_with_date(
                        OptionStyle::Call,
                        Side::Long,
                        pos!(100.0),
                        pos!(1.0),
                        pos!(100.0),
                        pos!(0.02),
                        naive_date,
                    ),
                    5.0,
                    Utc.from_utc_datetime(&naive_date),
                    0.5,
                    0.45,
                ),
                Position::new(
                    create_sample_option_with_date(
                        OptionStyle::Call,
                        Side::Short,
                        pos!(100.0),
                        pos!(1.0),
                        pos!(100.0),
                        pos!(0.02),
                        naive_date,
                    ),
                    5.0,
                    Utc.from_utc_datetime(&naive_date),
                    0.5,
                    0.45,
                ),
            ],
            max_profit: Some(10.0),
            max_loss: Some(5.0),
            break_even_points: vec![pos!(102.0), pos!(108.0)],
        };

        let expected_output = "Strategy: Bull Call Spread\nType: BullCallSpread\nDescription: A bullish options strategy\nLegs:\n  Position Details:\nOption: Long Call European Option\nUnderlying: AAPL @ $100.00\nStrike: $100.00\nExpiration: 2024-08-08 00:00:00 UTC\nImplied Volatility: 2.00%\nQuantity: 1\nRisk-free Rate: 5.00%\nDividend Yield: 1.00%\nPremium per contract: $5.00\nDate: 2024-08-08 00:00:00 UTC\nOpen Fee per contract: $0.50\nClose Fee per contract: $0.45\n  Position Details:\nOption: Short Call European Option\nUnderlying: AAPL @ $100.00\nStrike: $100.00\nExpiration: 2024-08-08 00:00:00 UTC\nImplied Volatility: 2.00%\nQuantity: 1\nRisk-free Rate: 5.00%\nDividend Yield: 1.00%\nPremium per contract: $5.00\nDate: 2024-08-08 00:00:00 UTC\nOpen Fee per contract: $0.50\nClose Fee per contract: $0.45\nMax Profit: $10.00\nMax Loss: $5.00\nBreak-even Points:\n  $102.00\n  $108.00\n";

        assert_eq!(format!("{}", strategy), expected_output);
    }

    #[test]
    fn test_strategy_debug() {
        let naive_date = NaiveDate::from_ymd_opt(2024, 8, 8)
            .expect("Invalid date")
            .and_hms_opt(0, 0, 0)
            .expect("Invalid time");

        let strategy = Strategy {
            name: "Bear Put Spread".to_string(),
            kind: StrategyType::BearPutSpread,
            description: "A bearish options strategy".to_string(),
            legs: vec![
                Position::new(
                    create_sample_option_with_date(
                        OptionStyle::Call,
                        Side::Long,
                        pos!(100.0),
                        pos!(1.0),
                        pos!(110.0),
                        pos!(0.02),
                        naive_date,
                    ),
                    5.0,
                    Utc.from_utc_datetime(&naive_date),
                    0.5,
                    0.45,
                ),
                Position::new(
                    create_sample_option_with_date(
                        OptionStyle::Call,
                        Side::Short,
                        pos!(100.0),
                        pos!(1.0),
                        pos!(110.0),
                        pos!(0.02),
                        naive_date,
                    ),
                    5.0,
                    Utc.from_utc_datetime(&naive_date),
                    0.5,
                    0.45,
                ),
            ],
            max_profit: Some(8.0),
            max_loss: Some(2.0),
            break_even_points: vec![pos!(82.0), pos!(88.0)],
        };

        let expected_output = "Strategy { name: \"Bear Put Spread\", kind: BearPutSpread, description: \"A bearish options strategy\", legs: [Position { option: Options { option_type: European, side: Side::Long, underlying_symbol: \"AAPL\", strike_price: 110, expiration_date: ExpirationDate::DateTime(2024-08-08 00:00:00 UTC), implied_volatility: 0.02, quantity: 1, underlying_price: 100, risk_free_rate: 0.05, option_style: OptionStyle::Call, dividend_yield: 0.01, exotic_params: None }, premium: 5.0, date: 2024-08-08T00:00:00Z, open_fee: 0.5, close_fee: 0.45 }, Position { option: Options { option_type: European, side: Side::Short, underlying_symbol: \"AAPL\", strike_price: 110, expiration_date: ExpirationDate::DateTime(2024-08-08 00:00:00 UTC), implied_volatility: 0.02, quantity: 1, underlying_price: 100, risk_free_rate: 0.05, option_style: OptionStyle::Call, dividend_yield: 0.01, exotic_params: None }, premium: 5.0, date: 2024-08-08T00:00:00Z, open_fee: 0.5, close_fee: 0.45 }], max_profit: Some(8.0), max_loss: Some(2.0), break_even_points: [82, 88] }";

        assert_eq!(format!("{:?}", strategy), expected_output);
    }
}
