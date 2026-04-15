/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 20/8/24
******************************************************************************/
use crate::model::option::ExoticParams;
use crate::model::{Options, Position};
use crate::strategies::base::Strategy;
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
            write!(f, "\nExotic Parameters: {exotic:?}")?;
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
            fields.push(format!("Spot Prices: {prices:?}"));
        }

        if let Some(min) = self.spot_min {
            fields.push(format!("Spot Min: {min:.2}"));
        }

        if let Some(max) = self.spot_max {
            fields.push(format!("Spot Max: {max:.2}"));
        }

        if let Some(cap) = self.cliquet_local_cap {
            fields.push(format!("Cliquet Local Cap: {cap:.2}"));
        }

        if let Some(floor) = self.cliquet_local_floor {
            fields.push(format!("Cliquet Local Floor: {floor:.2}"));
        }

        if let Some(cap) = self.cliquet_global_cap {
            fields.push(format!("Cliquet Global Cap: {cap:.2}"));
        }

        if let Some(floor) = self.cliquet_global_floor {
            fields.push(format!("Cliquet Global Floor: {floor:.2}"));
        }

        if let Some(ref price) = self.rainbow_second_asset_price {
            fields.push(format!("Rainbow Second Asset Price: {price}"));
        }

        if let Some(ref vol) = self.rainbow_second_asset_volatility {
            fields.push(format!("Rainbow Second Asset Volatility: {vol}"));
        }

        if let Some(ref div) = self.rainbow_second_asset_dividend {
            fields.push(format!("Rainbow Second Asset Dividend: {div}"));
        }

        if let Some(corr) = self.rainbow_correlation {
            fields.push(format!("Rainbow Correlation: {corr:.4}"));
        }

        if let Some(ref vol) = self.spread_second_asset_volatility {
            fields.push(format!("Spread Second Asset Volatility: {vol}"));
        }

        if let Some(ref div) = self.spread_second_asset_dividend {
            fields.push(format!("Spread Second Asset Dividend: {div}"));
        }

        if let Some(corr) = self.spread_correlation {
            fields.push(format!("Spread Correlation: {corr:.4}"));
        }

        if let Some(ref vol) = self.quanto_fx_volatility {
            fields.push(format!("Quanto FX Volatility: {vol}"));
        }

        if let Some(corr) = self.quanto_fx_correlation {
            fields.push(format!("Quanto FX Correlation: {corr:.4}"));
        }

        if let Some(rate) = self.quanto_foreign_rate {
            fields.push(format!("Quanto Foreign Rate: {rate:.4}"));
        }

        if let Some(ref vol) = self.exchange_second_asset_volatility {
            fields.push(format!("Exchange Second Asset Volatility: {vol}"));
        }

        if let Some(ref div) = self.exchange_second_asset_dividend {
            fields.push(format!("Exchange Second Asset Dividend: {div}"));
        }

        if let Some(corr) = self.exchange_correlation {
            fields.push(format!("Exchange Correlation: {corr:.4}"));
        }

        write!(f, "{}", fields.join(", "))
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
            writeln!(f, "  {leg}")?;
        }
        if let Some(max_profit) = self.max_profit {
            writeln!(f, "Max Profit: ${max_profit:.2}")?;
        }
        if let Some(max_loss) = self.max_loss {
            writeln!(f, "Max Loss: ${max_loss:.2}")?;
        }
        writeln!(f, "Break-even Points:")?;
        for point in &self.break_even_points {
            writeln!(f, "  ${point:.2}")?;
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
    use crate::model::types::{BarrierType, OptionStyle, OptionType, Side};

    use chrono::{NaiveDate, TimeZone, Utc};
    use expiration_date::ExpirationDate;
    use positive::pos_or_panic;

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
            strike_price: pos_or_panic!(150.0),
            expiration_date: ExpirationDate::DateTime(Utc.from_utc_datetime(&naive_date)),
            implied_volatility: pos_or_panic!(0.25),
            quantity: pos_or_panic!(10.0),
            underlying_price: pos_or_panic!(155.0),
            risk_free_rate: dec!(0.01),
            option_style: OptionStyle::Call,
            dividend_yield: pos_or_panic!(0.02),
            exotic_params: None,
        };

        let debug_output = format!("{options:?}");
        let expected_output = "Options { \
            option_type: European, \
            side: Side::Long, \
            underlying_symbol: \"AAPL\", \
            strike_price: 150, \
            expiration_date: DateTime(2024-08-08T00:00:00Z), \
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
            strike_price: pos_or_panic!(150.0),
            expiration_date: ExpirationDate::DateTime(Utc.from_utc_datetime(&naive_date)),
            implied_volatility: pos_or_panic!(0.25),
            quantity: pos_or_panic!(10.0),
            underlying_price: pos_or_panic!(155.0),
            risk_free_rate: dec!(0.01),
            option_style: OptionStyle::Call,
            dividend_yield: pos_or_panic!(0.02),
            exotic_params: None,
        };

        let display_output = format!("{options}");
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
            cliquet_local_cap: None,
            cliquet_local_floor: None,
            cliquet_global_cap: None,
            cliquet_global_floor: None,
            rainbow_second_asset_price: None,
            rainbow_second_asset_volatility: None,
            rainbow_second_asset_dividend: None,
            rainbow_correlation: None,
            spread_second_asset_volatility: None,
            spread_second_asset_dividend: None,
            spread_correlation: None,
            quanto_fx_volatility: None,
            quanto_fx_correlation: None,
            quanto_foreign_rate: None,
            exchange_second_asset_volatility: None,
            exchange_second_asset_dividend: None,
            exchange_correlation: None,
        };
        let naive_date = NaiveDate::from_ymd_opt(2024, 8, 8)
            .expect("Invalid date")
            .and_hms_opt(0, 0, 0)
            .expect("Invalid time");

        let options = Options {
            option_type: OptionType::Barrier {
                barrier_type: BarrierType::UpAndIn,
                barrier_level: 100.0,
                rebate: None,
            },
            side: Side::Short,
            underlying_symbol: "GOOGL".to_string(),
            strike_price: pos_or_panic!(2000.0),
            expiration_date: ExpirationDate::DateTime(Utc.from_utc_datetime(&naive_date)),
            implied_volatility: pos_or_panic!(0.30),
            quantity: pos_or_panic!(5.0),
            underlying_price: pos_or_panic!(1900.0),
            risk_free_rate: dec!(0.015),
            option_style: OptionStyle::Call,
            dividend_yield: pos_or_panic!(0.01),
            exotic_params: Some(exotic_params),
        };

        let display_output = format!("{options}");
        let expected_output = "\
            Short Call Barrier Option (Type: Up-And-In Barrier, Level: 100, Rebate: None)\n\
            Underlying: GOOGL @ $1900.00\n\
            Strike: $2000.00\n\
            Expiration: 2024-08-08 00:00:00 UTC\n\
            Implied Volatility: 30.00%\n\
            Quantity: 5\n\
            Risk-free Rate: 1.50%\n\
            Dividend Yield: 1.00%\n\
            Exotic Parameters: ExoticParams { spot_prices: None, spot_min: None, spot_max: None, cliquet_local_cap: None, cliquet_local_floor: None, cliquet_global_cap: None, cliquet_global_floor: None, rainbow_second_asset_price: None, rainbow_second_asset_volatility: None, rainbow_second_asset_dividend: None, rainbow_correlation: None, spread_second_asset_volatility: None, spread_second_asset_dividend: None, spread_correlation: None, quanto_fx_volatility: None, quanto_fx_correlation: None, quanto_foreign_rate: None, exchange_second_asset_volatility: None, exchange_second_asset_dividend: None, exchange_correlation: None }";

        assert_eq!(display_output, expected_output);
    }
}

#[cfg(test)]
mod tests_expiration_date {

    use chrono::{Duration, NaiveDate, TimeZone, Utc};
    use expiration_date::ExpirationDate;
    use positive::pos_or_panic;
    use tracing::info;

    #[test]
    fn test_display_days() {
        let expiration = ExpirationDate::Days(pos_or_panic!(30.5));
        let display_string = format!("{expiration}");
        assert!(display_string.contains("UTC"));
    }

    #[test]
    fn test_display_datetime() {
        let future_date = Utc::now() + Duration::days(15) + Duration::minutes(1);
        let expiration = ExpirationDate::DateTime(future_date);
        let display_string = format!("{expiration}");
        assert!(display_string.contains("UTC"));
        assert!(display_string.contains(&future_date.format("%Y-%m-%d %H:%M:%S").to_string()));
    }

    #[test]
    fn test_debug_days() {
        let expiration = ExpirationDate::Days(pos_or_panic!(45.75));
        let debug_string = format!("{expiration:?}");
        assert_eq!(debug_string, "Days(45.75)");
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
        let debug_string = format!("{expiration:?}");
        assert_eq!(debug_string, "DateTime(2023-12-31T23:59:59Z)");
    }

    #[test]
    fn test_display_past_date() {
        let past_date = Utc::now() - Duration::days(5);
        let expiration = ExpirationDate::DateTime(past_date);
        let display_string = format!("{expiration}");
        assert!(display_string.contains("UTC"));
        assert!(display_string.contains(&past_date.format("%Y-%m-%d %H:%M:%S").to_string()));
    }

    #[test]
    fn test_display_today() {
        let today = Utc::now();
        let expiration = ExpirationDate::DateTime(today);
        let display_string = format!("{expiration}");
        info!("{}", display_string);
        assert!(display_string.contains(&today.format("%Y-%m-%d %H:%M:%S").to_string()));
    }
}

#[cfg(test)]
mod tests_position_type_display_debug {
    use super::*;
    use crate::{OptionStyle, OptionType, Side};

    use chrono::{DateTime, NaiveDate, TimeZone, Utc};
    use expiration_date::ExpirationDate;
    use positive::pos_or_panic;

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
                strike_price: pos_or_panic!(150.0),
                expiration_date: ExpirationDate::DateTime(Utc.from_utc_datetime(&naive_date)),
                implied_volatility: pos_or_panic!(0.25),
                quantity: pos_or_panic!(10.0),
                underlying_price: pos_or_panic!(155.0),
                risk_free_rate: dec!(0.01),
                option_style: OptionStyle::Call,
                dividend_yield: pos_or_panic!(0.02),
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
            premium: pos_or_panic!(5.75),
            date: naive_date,
            open_fee: pos_or_panic!(0.50),
            close_fee: pos_or_panic!(0.45),
            epic: Some("Epic123".to_string()),
            extra_fields: None,
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

        assert_eq!(format!("{position}"), expected_display);
    }

    #[test]
    fn test_position_debug() {
        let (option, naive_date) = get_option();

        let position = Position {
            option,
            premium: pos_or_panic!(5.75),
            date: naive_date,
            open_fee: pos_or_panic!(0.50),
            close_fee: pos_or_panic!(0.45),
            epic: Some("Epic123".to_string()),
            extra_fields: None,
        };

        let expected_debug = "Position { \
        option: Options { \
            option_type: European, \
            side: Side::Long, \
            underlying_symbol: \"AAPL\", \
            strike_price: 150, \
            expiration_date: DateTime(2024-08-08T00:00:00Z), \
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

        assert_eq!(format!("{position:?}"), expected_debug);
    }
}

#[cfg(test)]
mod tests_strategy_type_display_debug {
    use super::*;
    use crate::model::utils::create_sample_option_with_date;
    use crate::{OptionStyle, Side};

    use crate::strategies::base::StrategyType;
    use chrono::{NaiveDate, TimeZone, Utc};
    use positive::{Positive, pos_or_panic};
    use serde::Serialize;

    #[test]
    fn test_strategy_display() {
        #[derive(Serialize)]
        struct ExtraFields {
            custom_field: String,
        }
        let extra_fields = ExtraFields {
            custom_field: "Custom Value".to_string(),
        };
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
                        Positive::HUNDRED,
                        Positive::ONE,
                        Positive::HUNDRED,
                        pos_or_panic!(0.02),
                        naive_date,
                    ),
                    pos_or_panic!(5.75),
                    Utc.from_utc_datetime(&naive_date),
                    pos_or_panic!(0.50),
                    pos_or_panic!(0.45),
                    Some("Epic123".to_string()),
                    None,
                ),
                Position::new(
                    create_sample_option_with_date(
                        OptionStyle::Call,
                        Side::Short,
                        Positive::HUNDRED,
                        Positive::ONE,
                        Positive::HUNDRED,
                        pos_or_panic!(0.02),
                        naive_date,
                    ),
                    pos_or_panic!(5.75),
                    Utc.from_utc_datetime(&naive_date),
                    pos_or_panic!(0.50),
                    pos_or_panic!(0.45),
                    Some("Epic123".to_string()),
                    Some(serde_json::to_value(&extra_fields).unwrap()),
                ),
            ],
            max_profit: Some(10.0),
            max_loss: Some(5.0),
            break_even_points: vec![pos_or_panic!(102.0), pos_or_panic!(108.0)],
        };

        let expected_output = "Strategy: Bull Call Spread\nType: BullCallSpread\nDescription: A bullish options strategy\nLegs:\n  Position Details:\nOption: Long Call European Option\nUnderlying: AAPL @ $100.00\nStrike: $100.00\nExpiration: 2024-08-08 00:00:00 UTC\nImplied Volatility: 2.00%\nQuantity: 1\nRisk-free Rate: 5.00%\nDividend Yield: 1.00%\nPremium per contract: $5.75\nDate: 2024-08-08 00:00:00 UTC\nOpen Fee per contract: $0.50\nClose Fee per contract: $0.45\n  Position Details:\nOption: Short Call European Option\nUnderlying: AAPL @ $100.00\nStrike: $100.00\nExpiration: 2024-08-08 00:00:00 UTC\nImplied Volatility: 2.00%\nQuantity: 1\nRisk-free Rate: 5.00%\nDividend Yield: 1.00%\nPremium per contract: $5.75\nDate: 2024-08-08 00:00:00 UTC\nOpen Fee per contract: $0.50\nClose Fee per contract: $0.45\nMax Profit: $10.00\nMax Loss: $5.00\nBreak-even Points:\n  $102.00\n  $108.00\n";

        assert_eq!(format!("{strategy}"), expected_output);
    }

    #[test]
    fn test_strategy_debug() {
        #[derive(Serialize)]
        struct ExtraFields {
            custom_field: String,
        }
        let extra_fields = ExtraFields {
            custom_field: "Custom Value".to_string(),
        };
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
                        Positive::HUNDRED,
                        Positive::ONE,
                        pos_or_panic!(110.0),
                        pos_or_panic!(0.02),
                        naive_date,
                    ),
                    pos_or_panic!(5.75),
                    Utc.from_utc_datetime(&naive_date),
                    pos_or_panic!(0.50),
                    pos_or_panic!(0.45),
                    Some("Epic123".to_string()),
                    None,
                ),
                Position::new(
                    create_sample_option_with_date(
                        OptionStyle::Call,
                        Side::Short,
                        Positive::HUNDRED,
                        Positive::ONE,
                        pos_or_panic!(110.0),
                        pos_or_panic!(0.02),
                        naive_date,
                    ),
                    pos_or_panic!(5.75),
                    Utc.from_utc_datetime(&naive_date),
                    pos_or_panic!(0.50),
                    pos_or_panic!(0.45),
                    Some("Epic123".to_string()),
                    Some(serde_json::to_value(&extra_fields).unwrap()),
                ),
            ],
            max_profit: Some(8.0),
            max_loss: Some(2.0),
            break_even_points: vec![pos_or_panic!(82.0), pos_or_panic!(88.0)],
        };

        let expected_output = "Strategy { name: \"Bear Put Spread\", kind: BearPutSpread, description: \"A bearish options strategy\", legs: [Position { option: Options { option_type: European, side: Side::Long, underlying_symbol: \"AAPL\", strike_price: 110, expiration_date: DateTime(2024-08-08T00:00:00Z), implied_volatility: 0.02, quantity: 1, underlying_price: 100, risk_free_rate: 0.05, option_style: OptionStyle::Call, dividend_yield: 0.01, exotic_params: None }, premium: 5.75, date: 2024-08-08T00:00:00Z, open_fee: 0.5, close_fee: 0.45 }, Position { option: Options { option_type: European, side: Side::Short, underlying_symbol: \"AAPL\", strike_price: 110, expiration_date: DateTime(2024-08-08T00:00:00Z), implied_volatility: 0.02, quantity: 1, underlying_price: 100, risk_free_rate: 0.05, option_style: OptionStyle::Call, dividend_yield: 0.01, exotic_params: None }, premium: 5.75, date: 2024-08-08T00:00:00Z, open_fee: 0.5, close_fee: 0.45 }], max_profit: Some(8.0), max_loss: Some(2.0), break_even_points: [82, 88] }";

        assert_eq!(format!("{strategy:?}"), expected_output);
    }
}
