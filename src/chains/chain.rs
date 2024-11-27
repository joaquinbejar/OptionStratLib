/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 26/9/24
******************************************************************************/
use crate::chains::utils::{
    adjust_volatility, default_empty_string, generate_list_of_strikes, parse,
    OptionChainBuildParams, OptionDataPriceParams, RandomPositionsParams,
};
use crate::greeks::equations::delta;
use crate::model::option::Options;
use crate::model::position::Position;
use crate::model::types::{OptionStyle, OptionType, PositiveF64, Side, PZERO};
use crate::pricing::black_scholes_model::black_scholes;
use crate::strategies::utils::FindOptimalSide;
use crate::utils::others::get_random_element;
use crate::{pos, spos};
use chrono::Utc;
use csv::WriterBuilder;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::BTreeSet;
use std::error::Error;
use std::fmt;
use std::fmt::Display;
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
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub(crate) struct OptionData {
    pub(crate) strike_price: PositiveF64,
    pub(crate) call_bid: Option<PositiveF64>,
    pub(crate) call_ask: Option<PositiveF64>,
    pub(crate) put_bid: Option<PositiveF64>,
    pub(crate) put_ask: Option<PositiveF64>,
    pub(crate) implied_volatility: Option<PositiveF64>,
    delta: Option<f64>,
    volume: Option<PositiveF64>,
    open_interest: Option<u64>,
}

#[allow(dead_code)]
impl OptionData {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        strike_price: PositiveF64,
        call_bid: Option<PositiveF64>,
        call_ask: Option<PositiveF64>,
        put_bid: Option<PositiveF64>,
        put_ask: Option<PositiveF64>,
        implied_volatility: Option<PositiveF64>,
        delta: Option<f64>,
        volume: Option<PositiveF64>,
        open_interest: Option<u64>,
    ) -> Self {
        OptionData {
            strike_price,
            call_bid,
            call_ask,
            put_bid,
            put_ask,
            implied_volatility,
            delta,
            volume,
            open_interest,
        }
    }

    pub(crate) fn validate(&self) -> bool {
        self.strike_price > PZERO
            && self.implied_volatility.is_some()
            && (self.valid_call() || self.valid_put())
    }

    pub(crate) fn valid_call(&self) -> bool {
        self.strike_price > PZERO
            && self.implied_volatility.is_some()
            && self.call_bid.is_some()
            && self.call_ask.is_some()
    }

    pub(crate) fn valid_put(&self) -> bool {
        self.strike_price > PZERO
            && self.implied_volatility.is_some()
            && self.put_bid.is_some()
            && self.put_ask.is_some()
    }

    pub fn get_call_buy_price(&self) -> Option<PositiveF64> {
        self.call_ask
    }

    pub fn get_call_sell_price(&self) -> Option<PositiveF64> {
        self.call_bid
    }

    pub fn get_put_buy_price(&self) -> Option<PositiveF64> {
        self.put_ask
    }

    pub fn get_put_sell_price(&self) -> Option<PositiveF64> {
        self.put_bid
    }

    fn get_option(&self, price_params: &OptionDataPriceParams) -> Result<Options, String> {
        let implied_volatility = match price_params.implied_volatility {
            Some(iv) => iv.value(),
            None => match self.implied_volatility {
                Some(iv) => iv.value(),
                None => {
                    return Err("Implied volatility not found".to_string());
                }
            },
        };

        Ok(Options::new(
            OptionType::European,
            Side::Long,
            "OptionData".to_string(),
            self.strike_price,
            price_params.expiration_date.clone(),
            implied_volatility,
            pos!(1.0),
            price_params.underlying_price,
            price_params.risk_free_rate,
            OptionStyle::Call,
            price_params.dividend_yield,
            None,
        ))
    }

    pub fn calculate_prices(&mut self, price_params: &OptionDataPriceParams) -> Result<(), String> {
        let mut option: Options = self.get_option(price_params)?;

        self.call_ask = spos!(black_scholes(&option).abs());
        option.side = Side::Short;
        self.call_bid = spos!(black_scholes(&option).abs());
        option.option_style = OptionStyle::Put;
        self.put_bid = spos!(black_scholes(&option).abs());
        option.side = Side::Long;
        self.put_ask = spos!(black_scholes(&option).abs());
        Ok(())
    }

    pub fn apply_spread(&mut self, spread: PositiveF64, decimal_places: i32) {
        fn round_to_decimal(
            number: PositiveF64,
            decimal_places: i32,
            shift: f64,
        ) -> Option<PositiveF64> {
            let multiplier = 10_f64.powi(decimal_places);
            spos!(((number.value() + shift) * multiplier).round() / multiplier)
        }

        let half_spread = spread / 2.0;

        if let Some(call_ask) = self.call_ask {
            if call_ask < half_spread {
                self.call_ask = None;
            } else {
                self.call_ask = round_to_decimal(call_ask, decimal_places, half_spread.value());
            }
        }
        if let Some(call_bid) = self.call_bid {
            if call_bid < half_spread {
                self.call_bid = None;
            } else {
                self.call_bid = round_to_decimal(call_bid, decimal_places, -half_spread.value());
            }
        }
        if let Some(put_ask) = self.put_ask {
            if put_ask < half_spread {
                self.put_ask = None;
            } else {
                self.put_ask = round_to_decimal(put_ask, decimal_places, half_spread.value());
            }
        }
        if let Some(put_bid) = self.put_bid {
            if put_bid < half_spread {
                self.put_bid = None;
            } else {
                self.put_bid = round_to_decimal(put_bid, decimal_places, -half_spread.value());
            }
        }
    }

    pub fn calculate_delta(&mut self, price_params: &OptionDataPriceParams) {
        let option: Options = match self.get_option(price_params) {
            Ok(option) => option,
            Err(_) => {
                return;
            }
        };
        self.delta = Some(delta(&option));
    }
}

impl Default for OptionData {
    fn default() -> Self {
        OptionData {
            strike_price: PZERO,
            call_bid: None,
            call_ask: None,
            put_bid: None,
            put_ask: None,
            implied_volatility: None,
            delta: None,
            volume: None,
            open_interest: None,
        }
    }
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

impl Display for OptionData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:<10} {:<10} {:<10} {:<10} {:<10} {:.3}{:<8} {:.3}{:<8} {:<10} {:<10}",
            self.strike_price.to_string(),
            default_empty_string(self.call_bid),
            default_empty_string(self.call_ask),
            default_empty_string(self.put_bid),
            default_empty_string(self.put_ask),
            self.implied_volatility.unwrap_or(pos!(0.0)),
            " ".to_string(),
            self.delta.unwrap_or(0.0),
            " ".to_string(),
            default_empty_string(self.volume),
            default_empty_string(self.open_interest),
        )?;
        Ok(())
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

    pub fn build_chain(params: &OptionChainBuildParams) -> Self {
        let mut option_chain = OptionChain::new(
            &params.symbol,
            params.price_params.underlying_price,
            params.price_params.expiration_date.get_date_string(),
        );

        let strikes = generate_list_of_strikes(
            params.price_params.underlying_price,
            params.chain_size,
            params.strike_interval,
        );

        for strike in strikes {
            let atm_distance = strike.value() - params.price_params.underlying_price.value();
            let adjusted_volatility = adjust_volatility(
                params.price_params.implied_volatility,
                params.skew_factor,
                atm_distance,
            );
            let mut option_data = OptionData::new(
                strike,
                None,
                None,
                None,
                None,
                adjusted_volatility,
                None,
                params.volume,
                None,
            );

            let price_params = OptionDataPriceParams::new(
                params.price_params.underlying_price,
                params.price_params.expiration_date.clone(),
                adjusted_volatility,
                params.price_params.risk_free_rate,
                params.price_params.dividend_yield,
            );
            if option_data.calculate_prices(&price_params).is_ok() {
                option_data.apply_spread(params.spread, params.decimal_places);
                option_data.calculate_delta(&price_params);
            }

            option_chain.options.insert(option_data);
        }

        option_chain
    }

    pub(crate) fn filter_option_data(&self, side: FindOptimalSide) -> Vec<&OptionData> {
        self.options
            .iter()
            .filter(|option| match side {
                FindOptimalSide::Upper => option.strike_price > self.underlying_price,
                FindOptimalSide::Lower => option.strike_price < self.underlying_price,
                FindOptimalSide::All => true,
                FindOptimalSide::Range(start, end) => {
                    option.strike_price >= start && option.strike_price <= end
                }
            })
            .collect()
    }

    #[allow(clippy::too_many_arguments)]
    pub fn add_option(
        &mut self,
        strike_price: PositiveF64,
        call_bid: Option<PositiveF64>,
        call_ask: Option<PositiveF64>,
        put_bid: Option<PositiveF64>,
        put_ask: Option<PositiveF64>,
        implied_volatility: Option<PositiveF64>,
        delta: Option<f64>,
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
                default_empty_string(option.call_bid),
                default_empty_string(option.call_ask),
                default_empty_string(option.put_bid),
                default_empty_string(option.put_ask),
                default_empty_string(option.implied_volatility),
                default_empty_string(option.delta),
                default_empty_string(option.volume),
                default_empty_string(option.open_interest),
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

            let option_data = OptionData {
                strike_price: record[0].parse()?,
                call_bid: parse(&record[1]),
                call_ask: parse(&record[2]),
                put_bid: parse(&record[3]),
                put_ask: parse(&record[4]),
                implied_volatility: parse(&record[5]),
                delta: parse(&record[6]),
                volume: parse(&record[7]),
                open_interest: parse(&record[8]),
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

    /// Creates random positions based on specified quantities of puts and calls
    ///
    /// # Arguments
    ///
    /// * `qty_puts_long` - Number of long put positions to create
    /// * `qty_puts_short` - Number of short put positions to create  
    /// * `qty_calls_long` - Number of long call positions to create
    /// * `qty_calls_short` - Number of short call positions to create
    ///
    /// # Returns
    ///
    /// * `Result<Vec<Position>, String>` - Vector of created positions or error message
    pub fn get_random_positions(
        &self,
        params: RandomPositionsParams,
    ) -> Result<Vec<Position>, String> {
        if params.total_positions() == 0 {
            return Err("The sum of the quantities must be greater than 0".to_string());
        }

        let mut positions = Vec::with_capacity(params.total_positions());

        // Add long put positions
        if let Some(qty) = params.qty_puts_long {
            for _ in 0..qty {
                if let Some(option) = get_random_element(&self.options) {
                    let position = Position::new(
                        Options::new(
                            OptionType::European,
                            Side::Long,
                            self.symbol.clone(),
                            option.strike_price,
                            params.expiration_date.clone(),
                            option.implied_volatility.unwrap_or(pos!(0.0)).value(),
                            params.option_qty,
                            self.underlying_price,
                            params.risk_free_rate,
                            OptionStyle::Put,
                            params.dividend_yield,
                            None,
                        ),
                        option.put_ask.unwrap_or(pos!(0.0)).value(),
                        Utc::now(),
                        params.open_put_fee,
                        params.close_put_fee,
                    );
                    positions.push(position);
                }
            }
        }

        // Add short put positions
        if let Some(qty) = params.qty_puts_short {
            for _ in 0..qty {
                if let Some(option) = get_random_element(&self.options) {
                    let position = Position::new(
                        Options::new(
                            OptionType::European,
                            Side::Short,
                            self.symbol.clone(),
                            option.strike_price,
                            params.expiration_date.clone(),
                            option.implied_volatility.unwrap_or(pos!(0.0)).value(),
                            params.option_qty,
                            self.underlying_price,
                            params.risk_free_rate,
                            OptionStyle::Put,
                            params.dividend_yield,
                            None,
                        ),
                        option.put_bid.unwrap_or(pos!(0.0)).value(),
                        Utc::now(),
                        params.open_put_fee,
                        params.close_put_fee,
                    );
                    positions.push(position);
                }
            }
        }

        // Add long call positions
        if let Some(qty) = params.qty_calls_long {
            for _ in 0..qty {
                if let Some(option) = get_random_element(&self.options) {
                    let position = Position::new(
                        Options::new(
                            OptionType::European,
                            Side::Long,
                            self.symbol.clone(),
                            option.strike_price,
                            params.expiration_date.clone(),
                            option.implied_volatility.unwrap_or(pos!(0.0)).value(),
                            params.option_qty,
                            self.underlying_price,
                            params.risk_free_rate,
                            OptionStyle::Call,
                            params.dividend_yield,
                            None,
                        ),
                        option.call_ask.unwrap_or(pos!(0.0)).value(),
                        Utc::now(),
                        params.open_call_fee,
                        params.close_call_fee,
                    );
                    positions.push(position);
                }
            }
        }

        // Add short call positions
        if let Some(qty) = params.qty_calls_short {
            for _ in 0..qty {
                if let Some(option) = get_random_element(&self.options) {
                    let position = Position::new(
                        Options::new(
                            OptionType::European,
                            Side::Short,
                            self.symbol.clone(),
                            option.strike_price,
                            params.expiration_date.clone(),
                            option.implied_volatility.unwrap_or(pos!(0.0)).value(),
                            params.option_qty,
                            self.underlying_price,
                            params.risk_free_rate,
                            OptionStyle::Call,
                            params.dividend_yield,
                            None,
                        ),
                        option.call_bid.unwrap_or(pos!(0.0)).value(),
                        Utc::now(),
                        params.open_call_fee,
                        params.close_call_fee,
                    );
                    positions.push(position);
                }
            }
        }

        Ok(positions)
    }
}

impl Display for OptionChain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Symbol: {}", self.symbol)?;
        writeln!(f, "Underlying Price: {:.1}", self.underlying_price)?;
        writeln!(f, "Expiration Date: {}", self.expiration_date)?;
        writeln!(
            f,
            "-------------------------------------------------------------------------------\
            ---------------------------"
        )?;
        writeln!(
            f,
            "{:<10} {:<10} {:<10} {:<10} {:<10} {:<13} {:<10} {:<10} {:<10}",
            "Strike",
            "Call Bid",
            "Call Ask",
            "Put Bid",
            "Put Ask",
            "Implied Vol.",
            "Delta",
            "Volume",
            "Open Interest"
        )?;
        writeln!(
            f,
            "----------------------------------------------------------------------------------\
            ------------------------"
        )?;

        for option in &self.options {
            writeln!(f, "{}", option,)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests_chain_base {
    use super::*;
    use crate::model::types::ExpirationDate;
    use crate::spos;
    use crate::utils::logger::setup_logger;
    use std::fs;
    use tracing::info;

    #[test]
    fn test_new_option_chain() {
        let chain = OptionChain::new("SP500", pos!(5781.88), "18-oct-2024".to_string());
        assert_eq!(chain.symbol, "SP500");
        assert_eq!(chain.underlying_price, 5781.88);
        assert_eq!(chain.expiration_date, "18-oct-2024");
        assert!(chain.options.is_empty());
    }

    #[test]
    fn test_new_option_chain_build_chain() {
        setup_logger();
        let params = OptionChainBuildParams::new(
            "SP500".to_string(),
            None,
            10,
            pos!(1.0),
            0.0,
            pos!(0.02),
            2,
            OptionDataPriceParams::new(
                pos!(100.0),
                ExpirationDate::Days(30.0),
                spos!(0.17),
                0.0,
                0.05,
            ),
        );

        let chain = OptionChain::build_chain(&params);

        assert_eq!(chain.symbol, "SP500");
        info!("{}", chain);
        assert_eq!(chain.options.len(), 21);
        assert_eq!(chain.underlying_price, pos!(100.0));
        let first = chain.options.iter().next().unwrap();
        assert_eq!(first.call_ask.unwrap(), 10.04);
        assert_eq!(first.call_bid.unwrap(), 10.02);
        assert_eq!(first.put_ask, spos!(0.04));
        assert_eq!(first.put_bid, spos!(0.02));
        let last = chain.options.iter().next_back().unwrap();
        assert_eq!(last.call_ask, spos!(0.06));
        assert_eq!(last.call_bid, spos!(0.04));
        assert_eq!(last.put_ask, spos!(10.06));
        assert_eq!(last.put_bid, spos!(10.04));
    }

    #[test]
    fn test_new_option_chain_build_chain_long() {
        setup_logger();
        let params = OptionChainBuildParams::new(
            "SP500".to_string(),
            None,
            25,
            pos!(25.0),
            0.000002,
            pos!(0.02),
            2,
            OptionDataPriceParams::new(
                pos!(5878.10),
                ExpirationDate::Days(60.0),
                spos!(0.03),
                0.0,
                0.05,
            ),
        );
        let chain = OptionChain::build_chain(&params);

        assert_eq!(chain.symbol, "SP500");
        info!("{}", chain);
        assert_eq!(chain.options.len(), 51);
        assert_eq!(chain.underlying_price, pos!(5878.10));
        let first = chain.options.iter().next().unwrap();
        assert_eq!(first.call_ask.unwrap(), 628.11);
        assert_eq!(first.call_bid.unwrap(), 628.09);
        assert_eq!(first.put_ask, None);
        assert_eq!(first.put_bid, None);
        let last = chain.options.iter().next_back().unwrap();
        assert_eq!(last.call_ask, None);
        assert_eq!(last.call_bid, None);
        assert_eq!(last.put_ask, spos!(621.91));
        assert_eq!(last.put_bid, spos!(621.89));
    }

    #[test]
    fn test_add_option() {
        let mut chain = OptionChain::new("SP500", pos!(5781.88), "18-oct-2024".to_string());
        chain.add_option(
            pos!(5520.0),
            spos!(274.26),
            spos!(276.06),
            spos!(13.22),
            spos!(14.90),
            spos!(16.31),
            Some(0.5),
            spos!(100.0),
            Some(100),
        );
        assert_eq!(chain.options.len(), 1);
        // first option in the chain
        let option = chain.options.iter().next().unwrap();
        assert_eq!(option.strike_price, 5520.0);
        assert!(option.call_bid.is_some());
        assert_eq!(option.call_bid.unwrap(), 274.26);
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
            spos!(274.26),
            spos!(276.06),
            spos!(13.22),
            spos!(14.90),
            spos!(16.31),
            Some(0.5),
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
            spos!(274.26),
            spos!(276.06),
            spos!(13.22),
            spos!(14.90),
            spos!(16.31),
            Some(0.5),
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
            spos!(274.26),
            spos!(276.06),
            spos!(13.22),
            spos!(14.90),
            spos!(16.31),
            Some(0.5),
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
            spos!(274.26),
            spos!(276.06),
            spos!(13.22),
            spos!(14.90),
            spos!(16.31),
            Some(0.5),
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

#[cfg(test)]
mod tests_option_data {
    use super::*;
    use crate::constants::ZERO;
    use crate::model::types::ExpirationDate;
    use crate::pos;
    use crate::spos;
    use crate::utils::logger::setup_logger;
    use tracing::info;

    fn create_valid_option_data() -> OptionData {
        OptionData::new(
            pos!(100.0),   // strike_price
            spos!(9.5),    // call_bid
            spos!(10.0),   // call_ask
            spos!(8.5),    // put_bid
            spos!(9.0),    // put_ask
            spos!(0.2),    // implied_volatility
            Some(-0.3),    // delta
            spos!(1000.0), // volume
            Some(500),     // open_interest
        )
    }

    #[test]
    fn test_new_option_data() {
        let option_data = create_valid_option_data();
        assert_eq!(option_data.strike_price, pos!(100.0));
        assert_eq!(option_data.call_bid, spos!(9.5));
        assert_eq!(option_data.call_ask, spos!(10.0));
        assert_eq!(option_data.put_bid, spos!(8.5));
        assert_eq!(option_data.put_ask, spos!(9.0));
        assert_eq!(option_data.implied_volatility, spos!(0.2));
        assert_eq!(option_data.delta, Some(-0.3));
        assert_eq!(option_data.volume, spos!(1000.0));
        assert_eq!(option_data.open_interest, Some(500));
    }

    #[test]
    fn test_validate_valid_option() {
        let option_data = create_valid_option_data();
        assert!(option_data.validate());
    }

    #[test]
    fn test_validate_zero_strike() {
        let mut option_data = create_valid_option_data();
        option_data.strike_price = PZERO;
        assert!(!option_data.validate());
    }

    #[test]
    fn test_validate_no_implied_volatility() {
        let mut option_data = create_valid_option_data();
        option_data.implied_volatility = None;
        assert!(!option_data.validate());
    }

    #[test]
    fn test_validate_missing_both_sides() {
        let option_data = OptionData::new(
            pos!(100.0),
            None,
            None,
            None,
            None,
            spos!(0.2),
            None,
            None,
            None,
        );
        assert!(!option_data.validate());
    }

    #[test]
    fn test_valid_call() {
        let option_data = create_valid_option_data();
        assert!(option_data.valid_call());
    }

    #[test]
    fn test_valid_call_missing_bid() {
        let mut option_data = create_valid_option_data();
        option_data.call_bid = None;
        assert!(!option_data.valid_call());
    }

    #[test]
    fn test_valid_call_missing_ask() {
        let mut option_data = create_valid_option_data();
        option_data.call_ask = None;
        assert!(!option_data.valid_call());
    }

    #[test]
    fn test_valid_put() {
        let option_data = create_valid_option_data();
        assert!(option_data.valid_put());
    }

    #[test]
    fn test_valid_put_missing_bid() {
        let mut option_data = create_valid_option_data();
        option_data.put_bid = None;
        assert!(!option_data.valid_put());
    }

    #[test]
    fn test_valid_put_missing_ask() {
        let mut option_data = create_valid_option_data();
        option_data.put_ask = None;
        assert!(!option_data.valid_put());
    }

    #[test]
    fn test_calculate_prices_success() {
        let mut option_data = OptionData::new(
            pos!(100.0),
            None,
            None,
            None,
            None,
            spos!(0.2),
            None,
            None,
            None,
        );
        let price_params = OptionDataPriceParams::new(
            pos!(100.0),
            ExpirationDate::Days(30.0),
            spos!(0.2),
            ZERO,
            ZERO,
        );

        let result = option_data.calculate_prices(&price_params);

        assert!(result.is_ok());
        assert!(option_data.call_ask.is_some());
        assert!(option_data.call_bid.is_some());
        assert!(option_data.put_ask.is_some());
        assert!(option_data.put_bid.is_some());
    }

    #[test]
    fn test_calculate_prices_missing_volatility() {
        setup_logger();
        let mut option_data =
            OptionData::new(pos!(100.0), None, None, None, None, None, None, None, None);

        let price_params =
            OptionDataPriceParams::new(pos!(100.0), ExpirationDate::Days(30.0), None, ZERO, ZERO);
        let _ = option_data.calculate_prices(&price_params);

        info!("{}", option_data);
        assert_eq!(option_data.call_ask, None);
        assert_eq!(option_data.call_bid, None);
        assert_eq!(option_data.put_ask, None);
        assert_eq!(option_data.put_bid, None);
        assert_eq!(option_data.implied_volatility, None);
        assert_eq!(option_data.delta, None);
        assert_eq!(option_data.strike_price, pos!(100.0));
    }

    #[test]
    fn test_calculate_prices_override_volatility() {
        setup_logger();
        let mut option_data = OptionData::new(
            pos!(100.0),
            None,
            None,
            None,
            None,
            spos!(0.2),
            None,
            None,
            None,
        );

        let price_params = OptionDataPriceParams::new(
            pos!(110.0),
            ExpirationDate::Days(30.0),
            spos!(0.12),
            0.05,
            0.01,
        );
        let result = option_data.calculate_prices(&price_params);

        assert!(result.is_ok());
        info!("{}", option_data);
        assert_eq!(option_data.call_ask, spos!(10.412135042233558));
        assert_eq!(option_data.call_bid, spos!(10.412135042233558));
        assert_eq!(option_data.put_ask, spos!(0.002019418653974231));
        assert_eq!(option_data.put_bid, spos!(0.002019418653974231));
        option_data.apply_spread(pos!(0.02), 2);
        info!("{}", option_data);
        assert_eq!(option_data.call_ask, spos!(10.42));
        assert_eq!(option_data.call_bid, spos!(10.4));
        assert_eq!(option_data.put_ask, None);
        assert_eq!(option_data.put_bid, None);
    }

    #[test]
    fn test_calculate_prices_with_all_parameters() {
        let mut option_data = OptionData::new(
            pos!(100.0),
            None,
            None,
            None,
            None,
            spos!(0.2),
            None,
            None,
            None,
        );
        let price_params = OptionDataPriceParams::new(
            pos!(100.0),
            ExpirationDate::Days(30.0),
            spos!(0.2),
            0.05,
            0.01,
        );

        let result = option_data.calculate_prices(&price_params);

        assert!(result.is_ok());
        assert!(option_data.call_ask.is_some());
        assert!(option_data.call_bid.is_some());
        assert!(option_data.put_ask.is_some());
        assert!(option_data.put_bid.is_some());
    }
}

#[cfg(test)]
mod tests_get_random_positions {
    use super::*;
    use crate::model::types::{ExpirationDate, PositiveF64};
    use crate::pos;
    use crate::utils::logger::setup_logger;

    fn create_test_chain() -> OptionChain {
        // Create a sample option chain
        let mut chain = OptionChain::new("TEST", pos!(100.0), "2024-01-01".to_string());

        // Add some test options with different strikes
        chain.add_option(
            pos!(95.0),   // strike_price
            spos!(4.0),   // call_bid
            spos!(4.2),   // call_ask
            spos!(3.0),   // put_bid
            spos!(3.2),   // put_ask
            spos!(0.2),   // implied_volatility
            Some(0.5),    // delta
            spos!(100.0), // volume
            Some(50),     // open_interest
        );

        chain.add_option(
            pos!(100.0),
            spos!(3.0),
            spos!(3.2),
            spos!(3.0),
            spos!(3.2),
            spos!(0.2),
            Some(0.5),
            spos!(100.0),
            Some(50),
        );

        chain.add_option(
            pos!(105.0),
            spos!(2.0),
            spos!(2.2),
            spos!(4.0),
            spos!(4.2),
            spos!(0.2),
            Some(0.5),
            spos!(100.0),
            Some(50),
        );

        chain
    }

    #[test]
    fn test_zero_quantity() {
        setup_logger();
        let chain = create_test_chain();
        let params = RandomPositionsParams::new(
            None,
            None,
            None,
            None,
            ExpirationDate::Days(30.0),
            pos!(1.0),
            0.05,
            0.02,
            1.0,
            1.0,
            1.0,
            1.0,
        );
        let result = chain.get_random_positions(params);

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "The sum of the quantities must be greater than 0".to_string()
        );
    }

    #[test]
    fn test_long_puts_only() {
        setup_logger();
        let chain = create_test_chain();
        let params = RandomPositionsParams::new(
            Some(2),
            None,
            None,
            None,
            ExpirationDate::Days(30.0),
            pos!(1.0),
            0.05,
            0.02,
            1.0,
            1.0,
            1.0,
            1.0,
        );
        let result = chain.get_random_positions(params);

        assert!(result.is_ok());
        let positions = result.unwrap();
        assert_eq!(positions.len(), 2);

        for position in positions {
            assert_eq!(position.option.option_style, OptionStyle::Put);
            assert_eq!(position.option.side, Side::Long);
            // Premium should be ask price for long positions
            assert!(position.premium > 0.0);
        }
    }

    #[test]
    fn test_short_puts_only() {
        setup_logger();
        let chain = create_test_chain();
        let params = RandomPositionsParams::new(
            None,
            Some(2),
            None,
            None,
            ExpirationDate::Days(30.0),
            pos!(1.0),
            0.05,
            0.02,
            1.0,
            1.0,
            1.0,
            1.0,
        );
        let result = chain.get_random_positions(params);

        assert!(result.is_ok());
        let positions = result.unwrap();
        assert_eq!(positions.len(), 2);

        for position in positions {
            assert_eq!(position.option.option_style, OptionStyle::Put);
            assert_eq!(position.option.side, Side::Short);
            // Premium should be bid price for short positions
            assert!(position.premium > 0.0);
        }
    }

    #[test]
    fn test_long_calls_only() {
        setup_logger();
        let chain = create_test_chain();
        let params = RandomPositionsParams::new(
            None,
            None,
            Some(2),
            None,
            ExpirationDate::Days(30.0),
            pos!(1.0),
            0.05,
            0.02,
            1.0,
            1.0,
            1.0,
            1.0,
        );
        let result = chain.get_random_positions(params);

        assert!(result.is_ok());
        let positions = result.unwrap();
        assert_eq!(positions.len(), 2);

        for position in positions {
            assert_eq!(position.option.option_style, OptionStyle::Call);
            assert_eq!(position.option.side, Side::Long);
            // Premium should be ask price for long positions
            assert!(position.premium > 0.0);
        }
    }

    #[test]
    fn test_short_calls_only() {
        setup_logger();
        let chain = create_test_chain();
        let params = RandomPositionsParams::new(
            None,
            None,
            None,
            Some(2),
            ExpirationDate::Days(30.0),
            pos!(1.0),
            0.05,
            0.02,
            1.0,
            1.0,
            1.0,
            1.0,
        );
        let result = chain.get_random_positions(params);

        assert!(result.is_ok());
        let positions = result.unwrap();
        assert_eq!(positions.len(), 2);

        for position in positions {
            assert_eq!(position.option.option_style, OptionStyle::Call);
            assert_eq!(position.option.side, Side::Short);
            // Premium should be bid price for short positions
            assert!(position.premium > 0.0);
        }
    }

    #[test]
    fn test_mixed_positions() {
        setup_logger();
        let chain = create_test_chain();
        let params = RandomPositionsParams::new(
            Some(1),
            Some(1),
            Some(1),
            Some(1),
            ExpirationDate::Days(30.0),
            pos!(1.0),
            0.05,
            0.02,
            1.0,
            1.0,
            1.0,
            1.0,
        );
        let result = chain.get_random_positions(params);

        assert!(result.is_ok());
        let positions = result.unwrap();
        assert_eq!(positions.len(), 4);

        let mut long_puts = 0;
        let mut short_puts = 0;
        let mut long_calls = 0;
        let mut short_calls = 0;

        for position in positions {
            match (position.option.option_style, position.option.side) {
                (OptionStyle::Put, Side::Long) => long_puts += 1,
                (OptionStyle::Put, Side::Short) => short_puts += 1,
                (OptionStyle::Call, Side::Long) => long_calls += 1,
                (OptionStyle::Call, Side::Short) => short_calls += 1,
            }
            // All premiums should be positive
            assert!(position.premium > 0.0);
        }

        assert_eq!(long_puts, 1);
        assert_eq!(short_puts, 1);
        assert_eq!(long_calls, 1);
        assert_eq!(short_calls, 1);
    }

    #[test]
    fn test_empty_chain() {
        setup_logger();
        let chain = OptionChain::new("TEST", pos!(100.0), "2024-01-01".to_string());
        let params = RandomPositionsParams::new(
            Some(1),
            None,
            None,
            None,
            ExpirationDate::Days(30.0),
            pos!(1.0),
            0.05,
            0.02,
            1.0,
            1.0,
            1.0,
            1.0,
        );
        let result = chain.get_random_positions(params);

        assert!(result.is_ok());
        let positions = result.unwrap();
        assert!(positions.is_empty());
    }
}

#[cfg(test)]
mod tests_option_data_get_prices {
    use super::*;
    use crate::pos;
    use crate::spos;

    fn create_test_option_data() -> OptionData {
        OptionData::new(
            pos!(100.0),
            spos!(9.5),
            spos!(10.0),
            spos!(8.5),
            spos!(9.0),
            spos!(0.2),
            Some(-0.3),
            spos!(1000.0),
            Some(500),
        )
    }

    #[test]
    fn test_get_call_buy_price() {
        let data = create_test_option_data();
        assert_eq!(data.get_call_buy_price(), spos!(10.0));
    }

    #[test]
    fn test_get_call_sell_price() {
        let data = create_test_option_data();
        assert_eq!(data.get_call_sell_price(), spos!(9.5));
    }

    #[test]
    fn test_get_put_buy_price() {
        let data = create_test_option_data();
        assert_eq!(data.get_put_buy_price(), spos!(9.0));
    }

    #[test]
    fn test_get_put_sell_price() {
        let data = create_test_option_data();
        assert_eq!(data.get_put_sell_price(), spos!(8.5));
    }

    #[test]
    fn test_get_prices_with_none_values() {
        let data = OptionData::new(
            pos!(100.0),
            None,
            None,
            None,
            None,
            spos!(0.2),
            None,
            None,
            None,
        );
        assert_eq!(data.get_call_buy_price(), None);
        assert_eq!(data.get_call_sell_price(), None);
        assert_eq!(data.get_put_buy_price(), None);
        assert_eq!(data.get_put_sell_price(), None);
    }
}

#[cfg(test)]
mod tests_option_data_display {
    use super::*;
    use crate::pos;
    use crate::spos;

    #[test]
    fn test_display_full_data() {
        let data = OptionData::new(
            pos!(100.0),
            spos!(9.5),
            spos!(10.0),
            spos!(8.5),
            spos!(9.0),
            spos!(0.2),
            Some(-0.3),
            spos!(1000.0),
            Some(500),
        );
        let display_string = format!("{}", data);
        assert!(display_string.contains("100"));
        assert!(display_string.contains("9.5"));
        assert!(display_string.contains("10"));
        assert!(display_string.contains("8.5"));
        assert!(display_string.contains("9"));
        assert!(display_string.contains("0.200"));
        assert!(display_string.contains("-0.300"));
        assert!(display_string.contains("1000"));
        assert!(display_string.contains("500"));
    }

    #[test]
    fn test_display_empty_data() {
        let data = OptionData::default();
        let display_string = format!("{}", data);

        assert!(display_string.contains("0.0"));
        assert!(display_string.contains("")); // Para campos None
    }
}

#[cfg(test)]
mod tests_filter_option_data {
    use super::*;
    use crate::pos;

    fn create_test_chain() -> OptionChain {
        let mut chain = OptionChain::new("TEST", pos!(100.0), "2024-01-01".to_string());

        // Añadir opciones con diferentes strikes
        for strike in [90.0, 95.0, 100.0, 105.0, 110.0].iter() {
            chain.add_option(
                pos!(*strike),
                None,
                None,
                None,
                None,
                spos!(0.2),
                None,
                None,
                None,
            );
        }
        chain
    }

    #[test]
    fn test_filter_upper() {
        let chain = create_test_chain();
        let filtered = chain.filter_option_data(FindOptimalSide::Upper);
        assert_eq!(filtered.len(), 2);
        assert!(filtered
            .iter()
            .all(|opt| opt.strike_price > chain.underlying_price));
    }

    #[test]
    fn test_filter_lower() {
        let chain = create_test_chain();
        let filtered = chain.filter_option_data(FindOptimalSide::Lower);
        assert_eq!(filtered.len(), 2);
        assert!(filtered
            .iter()
            .all(|opt| opt.strike_price < chain.underlying_price));
    }

    #[test]
    fn test_filter_all() {
        let chain = create_test_chain();
        let filtered = chain.filter_option_data(FindOptimalSide::All);
        assert_eq!(filtered.len(), 5);
    }

    #[test]
    fn test_filter_range() {
        let chain = create_test_chain();
        let filtered = chain.filter_option_data(FindOptimalSide::Range(pos!(95.0), pos!(105.0)));
        assert_eq!(filtered.len(), 3);
        assert!(filtered
            .iter()
            .all(|opt| opt.strike_price >= pos!(95.0) && opt.strike_price <= pos!(105.0)));
    }
}

#[cfg(test)]
mod tests_strike_price_range_vec {
    use super::*;
    use crate::pos;

    #[test]
    fn test_empty_chain() {
        let chain = OptionChain::new("TEST", pos!(100.0), "2024-01-01".to_string());
        assert_eq!(chain.strike_price_range_vec(5.0), None);
    }

    #[test]
    fn test_single_option() {
        let mut chain = OptionChain::new("TEST", pos!(100.0), "2024-01-01".to_string());
        chain.add_option(
            pos!(100.0),
            None,
            None,
            None,
            None,
            spos!(0.2),
            None,
            None,
            None,
        );
        let range = chain.strike_price_range_vec(5.0).unwrap();
        assert_eq!(range.len(), 1);
        assert_eq!(range[0], 100.0);
    }

    #[test]
    fn test_multiple_options() {
        let mut chain = OptionChain::new("TEST", pos!(100.0), "2024-01-01".to_string());
        for strike in [90.0, 95.0, 100.0].iter() {
            chain.add_option(
                pos!(*strike),
                None,
                None,
                None,
                None,
                spos!(0.2),
                None,
                None,
                None,
            );
        }
        let range = chain.strike_price_range_vec(5.0).unwrap();
        assert_eq!(range, vec![90.0, 95.0, 100.0]);
    }

    #[test]
    fn test_step_size() {
        let mut chain = OptionChain::new("TEST", pos!(100.0), "2024-01-01".to_string());
        for strike in [90.0, 100.0].iter() {
            chain.add_option(
                pos!(*strike),
                None,
                None,
                None,
                None,
                spos!(0.2),
                None,
                None,
                None,
            );
        }
        let range = chain.strike_price_range_vec(2.0).unwrap();
        assert_eq!(range.len(), 6); // [90, 92, 94, 96, 98, 100]
        assert_eq!(range[1] - range[0], 2.0);
    }
}
