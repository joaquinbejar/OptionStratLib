/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 26/9/24
******************************************************************************/
use crate::model::types::{PositiveF64, PZERO};
use crate::pos;
use csv::WriterBuilder;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::BTreeSet;
use std::error::Error;
use std::fmt;
use std::fs::File;
use tracing::debug;

/// Struct representing a row in an option chain.
///
/// # Fields
///
/// * `strike_price` - The strike price of the option, represented as a positive floating-point number.
/// * `call_bid` - The bid price for the call option, represented as a positive floating-point number.
/// * `call_ask` - The ask price for the call option, represented as a positive floating-point number.
/// * `put_bid` - The bid price for the put option, represented as a positive floating-point number.
/// * `put_ask` - The ask price for the put option, represented as a positive floating-point number.
/// * `implied_volatility` - The implied volatility of the option, represented as a positive floating-point number.
/// * `delta` - The delta of the option, represented as a floating-point number. This measures the sensitivity of the option's price to changes in the price of the underlying asset.
/// * `volume` - The volume of the option traded, represented as an optional positive floating-point number. It might be `None` if the data is not available.
/// * `open_interest` - The open interest of the option, represented as an optional unsigned integer. This represents the total number of outstanding option contracts that have not yet been settled or closed.
///
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub(crate) struct OptionData {
    pub(crate) strike_price: PositiveF64,
    pub(crate) call_bid: PositiveF64,
    pub(crate) call_ask: PositiveF64,
    put_bid: PositiveF64,
    put_ask: PositiveF64,
    pub(crate) implied_volatility: PositiveF64,
    delta: f64,
    volume: Option<PositiveF64>,
    open_interest: Option<u64>,
}

#[allow(clippy::non_canonical_partial_ord_impl)]
impl PartialOrd for OptionData {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.strike_price.partial_cmp(&other.strike_price)
    }
}

impl Eq for OptionData {}

impl Ord for OptionData {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap_or_else(|| {
            if self.strike_price.value().is_nan() {
                if other.strike_price.value().is_nan() {
                    Ordering::Equal
                } else {
                    Ordering::Greater
                }
            } else {
                Ordering::Less
            }
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OptionChain {
    pub(crate) symbol: String,
    pub underlying_price: PositiveF64,
    expiration_date: String,
    pub(crate) options: BTreeSet<OptionData>,
}

impl OptionChain {
    pub fn new(symbol: &str, underlying_price: PositiveF64, expiration_date: String) -> Self {
        OptionChain {
            symbol: symbol.to_string(),
            underlying_price,
            expiration_date,
            options: BTreeSet::new(),
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn add_option(
        &mut self,
        strike_price: PositiveF64,
        call_bid: PositiveF64,
        call_ask: PositiveF64,
        put_bid: PositiveF64,
        put_ask: PositiveF64,
        implied_volatility: PositiveF64,
        delta: f64,
        volume: Option<PositiveF64>,
        open_interest: Option<u64>,
    ) {
        let option_data = OptionData {
            strike_price,
            call_bid,
            call_ask,
            put_bid,
            put_ask,
            implied_volatility,
            delta,
            volume,
            open_interest,
        };
        self.options.insert(option_data);
    }

    pub fn get_title(&self) -> String {
        let symbol_cleaned = self.symbol.replace(" ", "-");
        let expiration_date_cleaned = self.expiration_date.replace(" ", "-");
        format!(
            "{}-{}-{}",
            symbol_cleaned, expiration_date_cleaned, self.underlying_price
        )
    }

    pub fn set_from_title(&mut self, file: &str) {
        let file_name = file.split('/').last().unwrap();
        let file_name = file_name
            .rsplit_once('.')
            .map_or(file_name, |(name, _ext)| name);
        let parts: Vec<&str> = file_name.split('-').collect();
        if parts.len() != 5 {
            panic!("Invalid file name format: expected exactly 5 parts (symbol, day, month, year, price)");
        }

        self.symbol = parts[0].to_string();
        self.expiration_date = format!("{}-{}-{}", parts[1], parts[2], parts[3]);
        let underlying_price_str = parts[4].replace(",", ".");

        match underlying_price_str.parse::<f64>() {
            Ok(price) => self.underlying_price = pos!(price),
            Err(_) => panic!("Invalid underlying price format in file name"),
        }
    }

    pub fn save_to_csv(&self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let full_path = format!("{}/{}.csv", file_path, self.get_title());
        let mut wtr = WriterBuilder::new().from_path(full_path)?;
        wtr.write_record([
            "Strike Price",
            "Call Bid",
            "Call Ask",
            "Put Bid",
            "Put Ask",
            "Implied Volatility",
            "Delta",
            "Volume",
            "Open Interest",
        ])?;
        for option in &self.options {
            wtr.write_record(&[
                option.strike_price.to_string(),
                option.call_bid.to_string(),
                option.call_ask.to_string(),
                option.put_bid.to_string(),
                option.put_ask.to_string(),
                option.implied_volatility.to_string(),
                option.delta.to_string(),
                option
                    .volume
                    .map_or_else(|| "".to_string(), |v| v.to_string()),
                option
                    .open_interest
                    .map_or_else(|| "".to_string(), |oi| oi.to_string()),
            ])?;
        }

        wtr.flush()?;
        Ok(())
    }

    pub fn save_to_json(&self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let full_path = format!("{}/{}.json", file_path, self.get_title());
        let file = File::create(full_path)?;
        serde_json::to_writer_pretty(file, &self)?;
        Ok(())
    }

    pub fn load_from_csv(file_path: &str) -> Result<Self, Box<dyn Error>> {
        let mut rdr = csv::Reader::from_path(file_path)?;
        let mut options = BTreeSet::new();
        for result in rdr.records() {
            let record = result?;
            debug!("To CSV: {:?}", record);
            let delta = record[6].parse().unwrap_or(0.0);
            let option_data = OptionData {
                strike_price: record[0].parse()?,
                call_bid: record[1].parse()?,
                call_ask: record[2].parse()?,
                put_bid: record[3].parse()?,
                put_ask: record[4].parse()?,
                implied_volatility: record[5].parse()?,
                delta,
                volume: Some(record[7].parse()?),
                open_interest: Some(record[8].parse()?),
            };
            options.insert(option_data);
        }

        let mut option_chain = OptionChain {
            symbol: "unknown".to_string(),
            underlying_price: PZERO,
            expiration_date: "unknown".to_string(),
            options,
        };
        option_chain.set_from_title(file_path);
        Ok(option_chain)
    }

    pub fn load_from_json(file_path: &str) -> Result<Self, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let mut option_chain: OptionChain = serde_json::from_reader(file)?;
        option_chain.set_from_title(file_path);
        Ok(option_chain)
    }

    pub fn strike_price_range_vec(&self, step: f64) -> Option<Vec<f64>> {
        let first = self.options.iter().next();
        let last = self.options.iter().next_back();

        if let (Some(first), Some(last)) = (first, last) {
            let mut range = Vec::new();
            let mut current_price = first.strike_price;

            while current_price <= last.strike_price {
                range.push(current_price.value());
                current_price += pos!(step);
            }

            Some(range)
        } else {
            None
        }
    }
}

impl fmt::Display for OptionChain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Symbol: {}", self.symbol)?; // Cambiado de {:.1} a {}
        writeln!(f, "Underlying Price: {:.1}", self.underlying_price)?;
        writeln!(f, "Expiration Date: {}", self.expiration_date)?;
        writeln!(
            f,
            "------------------------------------------------------------------"
        )?;
        writeln!(
            f,
            "{:<10} {:<10} {:<10} {:<10} {:<10} {:<15}",
            "Strike", "Call Bid", "Call Ask", "Put Bid", "Put Ask", "Implied Vol"
        )?;
        writeln!(
            f,
            "------------------------------------------------------------------"
        )?;

        for option in &self.options {
            writeln!(
                f,
                "{:<10.1} {:<10.1} {:<10.1} {:<10.1} {:<10.1} {:<15.3}",
                option.strike_price,
                option.call_bid,
                option.call_ask,
                option.put_bid,
                option.put_ask,
                option.implied_volatility
            )?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests_chain_base {
    use super::*;
    use crate::spos;
    use crate::utils::logger::setup_logger;
    use std::fs;

    #[test]
    fn test_new_option_chain() {
        let chain = OptionChain::new("SP500", pos!(5781.88), "18-oct-2024".to_string());
        assert_eq!(chain.symbol, "SP500");
        assert_eq!(chain.underlying_price, 5781.88);
        assert_eq!(chain.expiration_date, "18-oct-2024");
        assert!(chain.options.is_empty());
    }

    #[test]
    fn test_add_option() {
        let mut chain = OptionChain::new("SP500", pos!(5781.88), "18-oct-2024".to_string());
        chain.add_option(
            pos!(5520.0),
            pos!(274.26),
            pos!(276.06),
            pos!(13.22),
            pos!(14.90),
            pos!(16.31),
            0.5,
            spos!(100.0),
            Some(100),
        );
        assert_eq!(chain.options.len(), 1);
        // first option in the chain
        let option = chain.options.iter().next().unwrap();
        assert_eq!(option.strike_price, 5520.0);
        assert_eq!(option.call_bid, 274.26);
    }

    #[test]
    fn test_get_title_i() {
        let chain = OptionChain::new("SP500", pos!(5781.88), "18-oct-2024".to_string());
        assert_eq!(chain.get_title(), "SP500-18-oct-2024-5781.88");
    }

    #[test]
    fn test_get_title_ii() {
        let chain = OptionChain::new("SP500", pos!(5781.88), "18 oct 2024".to_string());
        assert_eq!(chain.get_title(), "SP500-18-oct-2024-5781.88");
    }

    #[test]
    fn test_set_from_title_i() {
        let mut chain = OptionChain::new("", PZERO, "".to_string());
        chain.set_from_title("SP500-18-oct-2024-5781.88.csv");
        assert_eq!(chain.symbol, "SP500");
        assert_eq!(chain.expiration_date, "18-oct-2024");
        assert_eq!(chain.underlying_price, 5781.88);
    }

    #[test]
    fn test_set_from_title_ii() {
        let mut chain = OptionChain::new("", PZERO, "".to_string());
        chain.set_from_title("path/SP500-18-oct-2024-5781.88.csv");
        assert_eq!(chain.symbol, "SP500");
        assert_eq!(chain.expiration_date, "18-oct-2024");
        assert_eq!(chain.underlying_price, 5781.88);
    }

    #[test]
    fn test_set_from_title_iii() {
        let mut chain = OptionChain::new("", PZERO, "".to_string());
        chain.set_from_title("path/SP500-18-oct-2024-5781.csv");
        assert_eq!(chain.symbol, "SP500");
        assert_eq!(chain.expiration_date, "18-oct-2024");
        assert_eq!(chain.underlying_price, 5781.0);
    }

    #[test]
    fn test_set_from_title_iv() {
        let mut chain = OptionChain::new("", PZERO, "".to_string());
        chain.set_from_title("path/SP500-18-oct-2024-5781.88.json");
        assert_eq!(chain.symbol, "SP500");
        assert_eq!(chain.expiration_date, "18-oct-2024");
        assert_eq!(chain.underlying_price, 5781.88);
    }

    #[test]
    fn test_set_from_title_v() {
        let mut chain = OptionChain::new("", PZERO, "".to_string());
        chain.set_from_title("path/SP500-18-oct-2024-5781.json");
        assert_eq!(chain.symbol, "SP500");
        assert_eq!(chain.expiration_date, "18-oct-2024");
        assert_eq!(chain.underlying_price, 5781.0);
    }

    #[test]
    fn test_save_to_csv() {
        let mut chain = OptionChain::new("SP500", pos!(5781.88), "18-oct-2024".to_string());
        chain.add_option(
            pos!(5520.0),
            pos!(274.26),
            pos!(276.06),
            pos!(13.22),
            pos!(14.90),
            pos!(16.31),
            0.5,
            spos!(100.0),
            Some(100),
        );
        let result = chain.save_to_csv(".");
        assert!(result.is_ok());
        let file_name = "./SP500-18-oct-2024-5781.88.csv".to_string();
        let remove_result = fs::remove_file(file_name);
        assert!(remove_result.is_ok());
    }

    #[test]
    fn test_save_to_json() {
        let mut chain = OptionChain::new("SP500", pos!(5781.88), "18-oct-2024".to_string());
        chain.add_option(
            pos!(5520.0),
            pos!(274.26),
            pos!(276.06),
            pos!(13.22),
            pos!(14.90),
            pos!(16.31),
            0.5,
            spos!(100.0),
            Some(100),
        );
        let result = chain.save_to_json(".");
        assert!(result.is_ok());

        let file_name = "./SP500-18-oct-2024-5781.88.json".to_string();
        let remove_result = fs::remove_file(file_name);
        assert!(remove_result.is_ok());
    }

    #[test]
    fn test_load_from_csv() {
        setup_logger();
        let mut chain = OptionChain::new("SP500", pos!(5781.89), "18-oct-2024".to_string());
        chain.add_option(
            pos!(5520.0),
            pos!(274.26),
            pos!(276.06),
            pos!(13.22),
            pos!(14.90),
            pos!(16.31),
            0.5,
            spos!(100.0),
            Some(100),
        );
        let result = chain.save_to_csv(".");
        assert!(result.is_ok());

        let result = OptionChain::load_from_csv("./SP500-18-oct-2024-5781.89.csv");
        assert!(result.is_ok());
        let chain = result.unwrap();
        assert_eq!(chain.symbol, "SP500");
        assert_eq!(chain.expiration_date, "18-oct-2024");
        assert_eq!(chain.underlying_price, 5781.89);

        let file_name = "./SP500-18-oct-2024-5781.89.csv".to_string();
        let remove_result = fs::remove_file(file_name);
        assert!(remove_result.is_ok());
    }

    #[test]
    fn test_load_from_json() {
        let mut chain = OptionChain::new("SP500", pos!(5781.9), "18-oct-2024".to_string());
        chain.add_option(
            pos!(5520.0),
            pos!(274.26),
            pos!(276.06),
            pos!(13.22),
            pos!(14.90),
            pos!(16.31),
            0.5,
            spos!(100.0),
            Some(100),
        );
        let result = chain.save_to_json(".");
        assert!(result.is_ok());

        let result = OptionChain::load_from_json("./SP500-18-oct-2024-5781.9.json");
        assert!(result.is_ok());
        let chain = result.unwrap();
        assert_eq!(chain.symbol, "SP500");
        assert_eq!(chain.expiration_date, "18-oct-2024");
        assert_eq!(chain.underlying_price, 5781.9);

        let file_name = "./SP500-18-oct-2024-5781.9.json".to_string();
        let remove_result = fs::remove_file(file_name);
        assert!(remove_result.is_ok());
    }
}
