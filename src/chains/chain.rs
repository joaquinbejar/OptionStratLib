/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 26/9/24
******************************************************************************/
use crate::chains::utils::{
    adjust_volatility, default_empty_string, generate_list_of_strikes, parse,
    OptionChainBuildParams, OptionChainParams, OptionDataPriceParams, RandomPositionsParams,
};
use crate::chains::{DeltasInStrike, OptionsInStrike};
use crate::error::chains::ChainError;
use crate::greeks::equations::delta;
use crate::model::{ExpirationDate, OptionStyle, OptionType, Side,Position,Options};
use crate::pricing::black_scholes_model::black_scholes;
use crate::strategies::utils::FindOptimalSide;
use crate::utils::others::get_random_element;
use crate::{f2p, sf2p, Positive};
use chrono::Utc;
use csv::WriterBuilder;
use num_traits::ToPrimitive;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::BTreeSet;
use std::error::Error;
use std::fmt;
use std::fs::File;
use rust_decimal::Decimal;
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
pub struct OptionData {
    pub(crate) strike_price: Positive,
    pub(crate) call_bid: Option<Positive>,
    pub(crate) call_ask: Option<Positive>,
    pub(crate) put_bid: Option<Positive>,
    pub(crate) put_ask: Option<Positive>,
    pub(crate) implied_volatility: Option<Positive>,
    delta: Option<f64>,
    volume: Option<Positive>,
    open_interest: Option<u64>,
}

impl OptionData {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        strike_price: Positive,
        call_bid: Option<Positive>,
        call_ask: Option<Positive>,
        put_bid: Option<Positive>,
        put_ask: Option<Positive>,
        implied_volatility: Option<Positive>,
        delta: Option<f64>,
        volume: Option<Positive>,
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
        self.strike_price > Positive::ZERO
            && self.implied_volatility.is_some()
            && (self.valid_call() || self.valid_put())
    }

    pub(crate) fn valid_call(&self) -> bool {
        self.strike_price > Positive::ZERO
            && self.implied_volatility.is_some()
            && self.call_bid.is_some()
            && self.call_ask.is_some()
    }

    pub(crate) fn valid_put(&self) -> bool {
        self.strike_price > Positive::ZERO
            && self.implied_volatility.is_some()
            && self.put_bid.is_some()
            && self.put_ask.is_some()
    }

    pub fn get_call_buy_price(&self) -> Option<Positive> {
        self.call_ask
    }

    pub fn get_call_sell_price(&self) -> Option<Positive> {
        self.call_bid
    }

    pub fn get_put_buy_price(&self) -> Option<Positive> {
        self.put_ask
    }

    pub fn get_put_sell_price(&self) -> Option<Positive> {
        self.put_bid
    }

    fn get_option(
        &self,
        price_params: &OptionDataPriceParams,
        side: Side,
        option_style: OptionStyle,
    ) -> Result<Options, ChainError> {
        let implied_volatility = match price_params.implied_volatility {
            Some(iv) => iv.value(),
            None => match self.implied_volatility {
                Some(iv) => iv.value(),
                None => {
                    return Err(ChainError::invalid_volatility(
                        None,
                        "Implied volatility not found",
                    ));
                }
            },
        };

        Ok(Options::new(
            OptionType::European,
            side,
            "OptionData".to_string(),
            self.strike_price,
            price_params.expiration_date.clone(),
            implied_volatility.to_f64().unwrap(),
            f2p!(1.0),
            price_params.underlying_price,
            price_params.risk_free_rate,
            option_style,
            price_params.dividend_yield,
            None,
        ))
    }

    fn get_options_in_strike(
        &self,
        price_params: &OptionDataPriceParams,
        side: Side,
        option_style: OptionStyle,
    ) -> Result<OptionsInStrike, ChainError> {
        let mut option: Options = self.get_option(price_params, side, option_style)?;
        option.option_style = OptionStyle::Call;
        option.side = Side::Long;
        let long_call = option.clone();
        option.side = Side::Short;
        let short_call = option.clone();
        option.option_style = OptionStyle::Put;
        let short_put = option.clone();
        option.side = Side::Long;
        let long_put = option.clone();
        Ok(OptionsInStrike {
            long_call,
            short_call,
            long_put,
            short_put,
        })
    }

    pub fn calculate_prices(
        &mut self,
        price_params: &OptionDataPriceParams,
    ) -> Result<(), ChainError> {
        let mut option: Options = self.get_option(price_params, Side::Long, OptionStyle::Call)?;

        self.call_ask = Some(black_scholes(&option).abs().into());
        option.side = Side::Short;
        self.call_bid = sf2p!(black_scholes(&option).abs());
        option.option_style = OptionStyle::Put;
        self.put_bid = sf2p!(black_scholes(&option).abs());
        option.side = Side::Long;
        self.put_ask = sf2p!(black_scholes(&option).abs());
        Ok(())
    }

    pub fn apply_spread(&mut self, spread: Positive, decimal_places: i32) {
        fn round_to_decimal(
            number: Positive,
            decimal_places: i32,
            shift: Decimal,
        ) -> Option<Positive> {
            let multiplier = Positive::TEN.powi(decimal_places as i64);
            Some(((number + shift) * multiplier).round() / multiplier)
        }

        let half_spread: Decimal = (spread / Positive::TWO).into();

        if let Some(call_ask) = self.call_ask {
            if call_ask < half_spread {
                self.call_ask = None;
            } else {
                self.call_ask = round_to_decimal(call_ask, decimal_places, half_spread);
            }
        }
        if let Some(call_bid) = self.call_bid {
            if call_bid < half_spread {
                self.call_bid = None;
            } else {
                self.call_bid = round_to_decimal(call_bid, decimal_places, -half_spread);
            }
        }
        if let Some(put_ask) = self.put_ask {
            if put_ask < half_spread {
                self.put_ask = None;
            } else {
                self.put_ask = round_to_decimal(put_ask, decimal_places, half_spread);
            }
        }
        if let Some(put_bid) = self.put_bid {
            if put_bid < half_spread {
                self.put_bid = None;
            } else {
                self.put_bid = round_to_decimal(put_bid, decimal_places, -half_spread);
            }
        }
    }

    pub fn calculate_delta(&mut self, price_params: &OptionDataPriceParams) {
        let option: Options = match self.get_option(price_params, Side::Long, OptionStyle::Call) {
            Ok(option) => option,
            Err(_) => {
                return;
            }
        };

        match delta(&option) {
            Ok(d) => self.delta = d.to_f64(),
            Err(_) => self.delta = None,
        }
    }

    pub fn get_deltas(
        &self,
        price_params: &OptionDataPriceParams,
    ) -> Result<DeltasInStrike, ChainError> {
        let options_in_strike =
            self.get_options_in_strike(price_params, Side::Long, OptionStyle::Call)?;
        Ok(options_in_strike.deltas()?)
    }

    pub fn is_valid_optimal_side(
        &self,
        underlying_price: Positive,
        side: &FindOptimalSide,
    ) -> bool {
        match side {
            FindOptimalSide::Upper => self.strike_price >= underlying_price,
            FindOptimalSide::Lower => self.strike_price <= underlying_price,
            FindOptimalSide::All => true,
            FindOptimalSide::Range(start, end) => {
                self.strike_price >= *start && self.strike_price <= *end
            }
        }
    }
}

impl Default for OptionData {
    fn default() -> Self {
        OptionData {
            strike_price: Positive::ZERO,
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

impl PartialOrd for OptionData {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.strike_price.cmp(&other.strike_price))
    }
}

impl Eq for OptionData {}

impl Ord for OptionData {
    fn cmp(&self, other: &Self) -> Ordering {
        self.strike_price.cmp(&other.strike_price)
    }
}

impl fmt::Display for OptionData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:<10} {:<10} {:<10} {:<10} {:<10} {:.3}{:<8} {:.3}{:<8} {:<10} {:<10}",
            self.strike_price.to_string(),
            default_empty_string(self.call_bid),
            default_empty_string(self.call_ask),
            default_empty_string(self.put_bid),
            default_empty_string(self.put_ask),
            self.implied_volatility.unwrap_or(f2p!(0.0)),
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
    pub underlying_price: Positive,
    expiration_date: String,
    pub(crate) options: BTreeSet<OptionData>,
    pub(crate) risk_free_rate: Option<f64>,
    pub(crate) dividend_yield: Option<f64>,
}

impl OptionChain {
    pub fn new(
        symbol: &str,
        underlying_price: Positive,
        expiration_date: String,
        risk_free_rate: Option<f64>,
        dividend_yield: Option<f64>,
    ) -> Self {
        OptionChain {
            symbol: symbol.to_string(),
            underlying_price,
            expiration_date,
            options: BTreeSet::new(),
            risk_free_rate,
            dividend_yield,
        }
    }

    pub fn build_chain(params: &OptionChainBuildParams) -> Self {
        let mut option_chain = OptionChain::new(
            &params.symbol,
            params.price_params.underlying_price,
            params.price_params.expiration_date.get_date_string(),
            None,
            None,
        );

        let strikes = generate_list_of_strikes(
            params.price_params.underlying_price,
            params.chain_size,
            params.strike_interval,
        );

        for strike in strikes {
            let atm_distance = strike - params.price_params.underlying_price;
            let adjusted_volatility = adjust_volatility(
                params.price_params.implied_volatility,
                params.skew_factor,
                atm_distance.into(),
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

    #[allow(dead_code)]
    pub(crate) fn filter_options_in_strike(
        &self,
        price_params: &OptionDataPriceParams,
        side: FindOptimalSide,
    ) -> Result<Vec<OptionsInStrike>, ChainError> {
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
            .map(|option| option.get_options_in_strike(price_params, Side::Long, OptionStyle::Call))
            .collect()
    }

    #[allow(clippy::too_many_arguments)]
    pub fn add_option(
        &mut self,
        strike_price: Positive,
        call_bid: Option<Positive>,
        call_ask: Option<Positive>,
        put_bid: Option<Positive>,
        put_ask: Option<Positive>,
        implied_volatility: Option<Positive>,
        delta: Option<f64>,
        volume: Option<Positive>,
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
            Ok(price) => self.underlying_price = f2p!(price),
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
            underlying_price: Positive::ZERO,
            expiration_date: "unknown".to_string(),
            options,
            risk_free_rate: None,
            dividend_yield: None,
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
                range.push(current_price.to_f64());
                current_price += f2p!(step);
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
    /// * `Result<Vec<Position>, ChainError>` - Vector of created positions or error message
    pub fn get_random_positions(
        &self,
        params: RandomPositionsParams,
    ) -> Result<Vec<Position>, ChainError> {
        if params.total_positions() == 0 {
            return Err(ChainError::invalid_parameters(
                "total_positions",
                "The sum of the quantities must be greater than 0",
            ));
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
                            option.implied_volatility.unwrap_or(f2p!(0.0)).to_f64(),
                            params.option_qty,
                            self.underlying_price,
                            params.risk_free_rate,
                            OptionStyle::Put,
                            params.dividend_yield,
                            None,
                        ),
                        option.put_ask.unwrap_or(f2p!(0.0)).to_f64(),
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
                            option.implied_volatility.unwrap_or(f2p!(0.0)).to_f64(),
                            params.option_qty,
                            self.underlying_price,
                            params.risk_free_rate,
                            OptionStyle::Put,
                            params.dividend_yield,
                            None,
                        ),
                        option.put_bid.unwrap_or(f2p!(0.0)).to_f64(),
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
                            option.implied_volatility.unwrap_or(f2p!(0.0)).to_f64(),
                            params.option_qty,
                            self.underlying_price,
                            params.risk_free_rate,
                            OptionStyle::Call,
                            params.dividend_yield,
                            None,
                        ),
                        option.call_ask.unwrap_or(f2p!(0.0)).to_f64(),
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
                            option.implied_volatility.unwrap_or(f2p!(0.0)).to_f64(),
                            params.option_qty,
                            self.underlying_price,
                            params.risk_free_rate,
                            OptionStyle::Call,
                            params.dividend_yield,
                            None,
                        ),
                        option.call_bid.unwrap_or(f2p!(0.0)).to_f64(),
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

    /// Returns an iterator over the `options` field in the `OptionChain` structure.
    ///
    /// This method provides a mechanism to traverse through the set of options
    /// (`OptionData`) associated with an `OptionChain`.
    ///
    /// # Returns
    ///
    /// An iterator that yields references to the `OptionData` elements in the `options` field.
    /// Since the `options` field is stored as a `BTreeSet`, the elements are ordered
    /// in ascending order based on the sorting rules of `BTreeSet` (typically defined by `Ord` implementation).
    ///
    pub fn get_single_iter(&self) -> impl Iterator<Item = &OptionData> {
        self.options.iter()
    }

    /// Returns an iterator that generates pairs of distinct option combinations from the `OptionChain`.
    ///
    /// This function iterates over all unique combinations of two options from the `options` collection
    /// without repetition. In mathematical terms, it generates combinations where order does not matter
    /// and an option cannot combine with itself.
    ///
    /// # Returns
    ///
    /// An iterator producing tuples of references to two distinct `OptionData` instances.
    ///
    /// # Example
    ///
    /// ```rust
    /// use tracing::info;
    /// use optionstratlib::chains::chain::OptionChain;
    /// use optionstratlib::{f2p, Positive};
    /// let mut option_chain = OptionChain::new("TEST", f2p!(100.0), "2024-01-01".to_string(), None, None);
    /// for (option1, option2) in option_chain.get_double_iter() {
    ///     info!("{:?}, {:?}", option1, option2);
    /// }
    /// ```
    pub fn get_double_iter(&self) -> impl Iterator<Item = (&OptionData, &OptionData)> {
        self.options.iter().enumerate().flat_map(|(i, item1)| {
            self.options
                .iter()
                .skip(i + 1)
                .map(move |item2| (item1, item2))
        })
    }

    /// Returns an iterator that generates inclusive pairs of option combinations from the `OptionChain`.
    ///
    /// This function iterates over all combinations of two options from the `options` collection,
    /// including pairing an option with itself.
    ///
    /// # Returns
    ///
    /// An iterator producing tuples with two references to `OptionData`, potentially including
    /// self-pairs (e.g., `(option, option)`).
    ///
    /// # Example
    ///
    /// ```rust
    /// use tracing::info;
    /// use optionstratlib::chains::chain::OptionChain;
    /// use optionstratlib::Positive;
    /// use optionstratlib::f2p;
    /// let mut option_chain = OptionChain::new("TEST", f2p!(100.0), "2024-01-01".to_string(), None, None);
    /// for (option1, option2) in option_chain.get_double_inclusive_iter() {
    ///     info!("{:?}, {:?}", option1, option2);
    /// }
    /// ```
    pub fn get_double_inclusive_iter(&self) -> impl Iterator<Item = (&OptionData, &OptionData)> {
        self.options
            .iter()
            .enumerate()
            .flat_map(|(i, item1)| self.options.iter().skip(i).map(move |item2| (item1, item2)))
    }

    /// Returns an iterator that generates unique triplets of distinct option combinations from the `OptionChain`.
    ///
    /// This function iterates over all unique combinations of three options from the `options` collection
    /// without repetition.
    ///
    /// # Returns
    ///
    /// An iterator producing tuples containing references to three distinct `OptionData` instances.
    ///
    /// # Example
    ///
    /// ```rust
    /// use tracing::info;
    /// use optionstratlib::chains::chain::OptionChain;
    /// use optionstratlib::Positive;
    /// use optionstratlib::f2p;
    /// let mut option_chain = OptionChain::new("TEST", f2p!(100.0), "2024-01-01".to_string(), None, None);
    /// for (option1, option2, option3) in option_chain.get_triple_iter() {
    ///     info!("{:?}, {:?}, {:?}", option1, option2, option3);
    /// }
    /// ```
    pub fn get_triple_iter(&self) -> impl Iterator<Item = (&OptionData, &OptionData, &OptionData)> {
        self.options.iter().enumerate().flat_map(move |(i, item1)| {
            self.options
                .iter()
                .skip(i + 1)
                .enumerate()
                .flat_map(move |(j, item2)| {
                    self.options
                        .iter()
                        .skip(i + j + 2)
                        .map(move |item3| (item1, item2, item3))
                })
        })
    }

    /// Returns an iterator that generates inclusive triplets of option combinations from the `OptionChain`.
    ///
    /// This function iterates over all combinations of three options from the `options` collection,
    /// including those where the same option may be included more than once.
    ///
    /// # Returns
    ///
    /// An iterator producing tuples with three references to `OptionData`, potentially including
    /// repeated elements (e.g., `(option1, option2, option1)`).
    ///
    /// # Example
    ///
    /// ```rust
    /// use tracing::info;
    /// use optionstratlib::chains::chain::OptionChain;
    /// use optionstratlib::Positive;
    /// use optionstratlib::f2p;
    /// let mut option_chain = OptionChain::new("TEST", f2p!(100.0), "2024-01-01".to_string(), None, None);
    /// for (option1, option2, option3) in option_chain.get_triple_inclusive_iter() {
    ///     info!("{:?}, {:?}, {:?}", option1, option2, option3);
    /// }
    /// ```
    pub fn get_triple_inclusive_iter(
        &self,
    ) -> impl Iterator<Item = (&OptionData, &OptionData, &OptionData)> {
        self.options.iter().enumerate().flat_map(move |(i, item1)| {
            self.options
                .iter()
                .skip(i)
                .enumerate()
                .flat_map(move |(j, item2)| {
                    self.options
                        .iter()
                        .skip(i + j)
                        .map(move |item3| (item1, item2, item3))
                })
        })
    }

    /// Returns an iterator that generates unique quadruples of distinct option combinations from the `OptionChain`.
    ///
    /// This function iterates over all unique combinations of four options from the `options` collection
    /// without repetition.
    ///
    /// # Returns
    ///
    /// An iterator producing tuples containing references to four distinct `OptionData` instances.
    ///
    /// # Example
    ///
    /// ```rust
    /// use tracing::info;
    /// use optionstratlib::chains::chain::OptionChain;
    /// use optionstratlib::Positive;
    /// use optionstratlib::f2p;
    /// let mut option_chain = OptionChain::new("TEST", f2p!(100.0), "2024-01-01".to_string(), None, None);
    /// for (option1, option2, option3, option4) in option_chain.get_quad_iter() {
    ///     info!("{:?}, {:?}, {:?}, {:?}", option1, option2, option3, option4);
    /// }
    /// ```
    pub fn get_quad_iter(
        &self,
    ) -> impl Iterator<Item = (&OptionData, &OptionData, &OptionData, &OptionData)> {
        self.options.iter().enumerate().flat_map(move |(i, item1)| {
            self.options
                .iter()
                .skip(i + 1)
                .enumerate()
                .flat_map(move |(j, item2)| {
                    self.options
                        .iter()
                        .skip(i + j + 2)
                        .enumerate()
                        .flat_map(move |(k, item3)| {
                            self.options
                                .iter()
                                .skip(i + j + k + 3)
                                .map(move |item4| (item1, item2, item3, item4))
                        })
                })
        })
    }

    /// Returns an iterator that generates inclusive quadruples of option combinations from the `OptionChain`.
    ///
    /// This function iterates over all combinations of four options from the `options` collection,
    /// including those where the same option may be included more than once.
    ///
    /// # Returns
    ///
    /// An iterator producing tuples with four references to `OptionData`, potentially including
    /// repeated elements (e.g., `(option1, option2, option1, option4)`).
    ///
    /// # Example
    ///
    /// ```rust
    /// use tracing::info;
    /// use optionstratlib::chains::chain::OptionChain;
    /// use optionstratlib::Positive;
    /// use optionstratlib::f2p;
    /// let mut option_chain = OptionChain::new("TEST", f2p!(100.0), "2024-01-01".to_string(), None, None);
    /// for (option1, option2, option3, option4) in option_chain.get_quad_inclusive_iter() {
    ///     info!("{:?}, {:?}, {:?}, {:?}", option1, option2, option3, option4);
    /// }
    /// ```
    pub fn get_quad_inclusive_iter(
        &self,
    ) -> impl Iterator<Item = (&OptionData, &OptionData, &OptionData, &OptionData)> {
        self.options.iter().enumerate().flat_map(move |(i, item1)| {
            self.options
                .iter()
                .skip(i)
                .enumerate()
                .flat_map(move |(j, item2)| {
                    self.options
                        .iter()
                        .skip(i + j)
                        .enumerate()
                        .flat_map(move |(k, item3)| {
                            self.options
                                .iter()
                                .skip(i + j + k)
                                .map(move |item4| (item1, item2, item3, item4))
                        })
                })
        })
    }
}

impl OptionChainParams for OptionChain {
    fn get_params(&self, strike_price: Positive) -> Result<OptionDataPriceParams, ChainError> {
        let option = self
            .options
            .iter()
            .find(|option| option.strike_price == strike_price);
        if option.is_none() {
            let reason = format!("Option with strike price {} not found", strike_price);
            return Err(ChainError::invalid_strike(strike_price.to_f64(), &reason));
        }
        Ok(OptionDataPriceParams::new(
            self.underlying_price,
            ExpirationDate::from_string(&self.expiration_date)?,
            option.unwrap().implied_volatility,
            self.risk_free_rate.unwrap_or(0.0),
            self.dividend_yield.unwrap_or(0.0),
        ))
    }
}

impl fmt::Display for OptionChain {
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
    use crate::sf2p;
    use crate::utils::logger::setup_logger;
    use std::fs;
    use tracing::info;

    #[test]
    fn test_new_option_chain() {
        let chain = OptionChain::new(
            "SP500",
            f2p!(5781.88),
            "18-oct-2024".to_string(),
            None,
            None,
        );
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
            f2p!(1.0),
            0.0,
            f2p!(0.02),
            2,
            OptionDataPriceParams::new(
                f2p!(100.0),
                ExpirationDate::Days(30.0),
                sf2p!(0.17),
                0.0,
                0.05,
            ),
        );

        let chain = OptionChain::build_chain(&params);

        assert_eq!(chain.symbol, "SP500");
        info!("{}", chain);
        assert_eq!(chain.options.len(), 21);
        assert_eq!(chain.underlying_price, f2p!(100.0));
        let first = chain.options.iter().next().unwrap();
        assert_eq!(first.call_ask.unwrap(), 10.04);
        assert_eq!(first.call_bid.unwrap(), 10.02);
        assert_eq!(first.put_ask, sf2p!(0.04));
        assert_eq!(first.put_bid, sf2p!(0.02));
        let last = chain.options.iter().next_back().unwrap();
        assert_eq!(last.call_ask, sf2p!(0.06));
        assert_eq!(last.call_bid, sf2p!(0.04));
        assert_eq!(last.put_ask, sf2p!(10.06));
        assert_eq!(last.put_bid, sf2p!(10.04));
    }

    #[test]
    fn test_new_option_chain_build_chain_long() {
        setup_logger();
        let params = OptionChainBuildParams::new(
            "SP500".to_string(),
            None,
            25,
            f2p!(25.0),
            0.000002,
            f2p!(0.02),
            2,
            OptionDataPriceParams::new(
                f2p!(5878.10),
                ExpirationDate::Days(60.0),
                sf2p!(0.03),
                0.0,
                0.05,
            ),
        );
        let chain = OptionChain::build_chain(&params);

        assert_eq!(chain.symbol, "SP500");
        info!("{}", chain);
        assert_eq!(chain.options.len(), 51);
        assert_eq!(chain.underlying_price, f2p!(5878.10));
        let first = chain.options.iter().next().unwrap();
        assert_eq!(first.call_ask.unwrap(), 628.11);
        assert_eq!(first.call_bid.unwrap(), 628.09);
        assert_eq!(first.put_ask, None);
        assert_eq!(first.put_bid, None);
        let last = chain.options.iter().next_back().unwrap();
        assert_eq!(last.call_ask, None);
        assert_eq!(last.call_bid, None);
        assert_eq!(last.put_ask, sf2p!(621.91));
        assert_eq!(last.put_bid, sf2p!(621.89));
    }

    #[test]
    fn test_add_option() {
        let mut chain = OptionChain::new(
            "SP500",
            f2p!(5781.88),
            "18-oct-2024".to_string(),
            None,
            None,
        );
        chain.add_option(
            f2p!(5520.0),
            sf2p!(274.26),
            sf2p!(276.06),
            sf2p!(13.22),
            sf2p!(14.90),
            sf2p!(16.31),
            Some(0.5),
            sf2p!(100.0),
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
        let chain = OptionChain::new(
            "SP500",
            f2p!(5781.88),
            "18-oct-2024".to_string(),
            None,
            None,
        );
        assert_eq!(chain.get_title(), "SP500-18-oct-2024-5781.88");
    }

    #[test]
    fn test_get_title_ii() {
        let chain = OptionChain::new(
            "SP500",
            f2p!(5781.88),
            "18 oct 2024".to_string(),
            None,
            None,
        );
        assert_eq!(chain.get_title(), "SP500-18-oct-2024-5781.88");
    }

    #[test]
    fn test_set_from_title_i() {
        let mut chain = OptionChain::new("", Positive::ZERO, "".to_string(), None, None);
        chain.set_from_title("SP500-18-oct-2024-5781.88.csv");
        assert_eq!(chain.symbol, "SP500");
        assert_eq!(chain.expiration_date, "18-oct-2024");
        assert_eq!(chain.underlying_price, 5781.88);
    }

    #[test]
    fn test_set_from_title_ii() {
        let mut chain = OptionChain::new("", Positive::ZERO, "".to_string(), None, None);
        chain.set_from_title("path/SP500-18-oct-2024-5781.88.csv");
        assert_eq!(chain.symbol, "SP500");
        assert_eq!(chain.expiration_date, "18-oct-2024");
        assert_eq!(chain.underlying_price, 5781.88);
    }

    #[test]
    fn test_set_from_title_iii() {
        let mut chain = OptionChain::new("", Positive::ZERO, "".to_string(), None, None);
        chain.set_from_title("path/SP500-18-oct-2024-5781.csv");
        assert_eq!(chain.symbol, "SP500");
        assert_eq!(chain.expiration_date, "18-oct-2024");
        assert_eq!(chain.underlying_price, 5781.0);
    }

    #[test]
    fn test_set_from_title_iv() {
        let mut chain = OptionChain::new("", Positive::ZERO, "".to_string(), None, None);
        chain.set_from_title("path/SP500-18-oct-2024-5781.88.json");
        assert_eq!(chain.symbol, "SP500");
        assert_eq!(chain.expiration_date, "18-oct-2024");
        assert_eq!(chain.underlying_price, 5781.88);
    }

    #[test]
    fn test_set_from_title_v() {
        let mut chain = OptionChain::new("", Positive::ZERO, "".to_string(), None, None);
        chain.set_from_title("path/SP500-18-oct-2024-5781.json");
        assert_eq!(chain.symbol, "SP500");
        assert_eq!(chain.expiration_date, "18-oct-2024");
        assert_eq!(chain.underlying_price, 5781.0);
    }

    #[test]
    fn test_save_to_csv() {
        let mut chain = OptionChain::new(
            "SP500",
            f2p!(5781.88),
            "18-oct-2024".to_string(),
            None,
            None,
        );
        chain.add_option(
            f2p!(5520.0),
            sf2p!(274.26),
            sf2p!(276.06),
            sf2p!(13.22),
            sf2p!(14.90),
            sf2p!(16.31),
            Some(0.5),
            sf2p!(100.0),
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
        let mut chain = OptionChain::new(
            "SP500",
            f2p!(5781.88),
            "18-oct-2024".to_string(),
            None,
            None,
        );
        chain.add_option(
            f2p!(5520.0),
            sf2p!(274.26),
            sf2p!(276.06),
            sf2p!(13.22),
            sf2p!(14.90),
            sf2p!(16.31),
            Some(0.5),
            sf2p!(100.0),
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
        let mut chain = OptionChain::new(
            "SP500",
            f2p!(5781.89),
            "18-oct-2024".to_string(),
            None,
            None,
        );
        chain.add_option(
            f2p!(5520.0),
            sf2p!(274.26),
            sf2p!(276.06),
            sf2p!(13.22),
            sf2p!(14.90),
            sf2p!(16.31),
            Some(0.5),
            sf2p!(100.0),
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
        let mut chain =
            OptionChain::new("SP500", f2p!(5781.9), "18-oct-2024".to_string(), None, None);
        chain.add_option(
            f2p!(5520.0),
            sf2p!(274.26),
            sf2p!(276.06),
            sf2p!(13.22),
            sf2p!(14.90),
            sf2p!(16.31),
            Some(0.5),
            sf2p!(100.0),
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
    use crate::model::types::ExpirationDate;
    use crate::f2p;
    use crate::sf2p;
    use crate::utils::logger::setup_logger;
    use tracing::info;
    use crate::constants::ZERO;

    fn create_valid_option_data() -> OptionData {
        OptionData::new(
            f2p!(100.0),   // strike_price
            sf2p!(9.5),    // call_bid
            sf2p!(10.0),   // call_ask
            sf2p!(8.5),    // put_bid
            sf2p!(9.0),    // put_ask
            sf2p!(0.2),    // implied_volatility
            Some(-0.3),    // delta
            sf2p!(1000.0), // volume
            Some(500),     // open_interest
        )
    }

    #[test]
    fn test_new_option_data() {
        let option_data = create_valid_option_data();
        assert_eq!(option_data.strike_price, f2p!(100.0));
        assert_eq!(option_data.call_bid, sf2p!(9.5));
        assert_eq!(option_data.call_ask, sf2p!(10.0));
        assert_eq!(option_data.put_bid, sf2p!(8.5));
        assert_eq!(option_data.put_ask, sf2p!(9.0));
        assert_eq!(option_data.implied_volatility, sf2p!(0.2));
        assert_eq!(option_data.delta, Some(-0.3));
        assert_eq!(option_data.volume, sf2p!(1000.0));
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
        option_data.strike_price = Positive::ZERO;
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
            f2p!(100.0),
            None,
            None,
            None,
            None,
            sf2p!(0.2),
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
            f2p!(100.0),
            None,
            None,
            None,
            None,
            sf2p!(0.2),
            None,
            None,
            None,
        );
        let price_params = OptionDataPriceParams::new(
            f2p!(100.0),
            ExpirationDate::Days(30.0),
            sf2p!(0.2),
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
            OptionData::new(f2p!(100.0), None, None, None, None, None, None, None, None);

        let price_params =
            OptionDataPriceParams::new(f2p!(100.0), ExpirationDate::Days(30.0), None, ZERO, ZERO);
        let _ = option_data.calculate_prices(&price_params);

        info!("{}", option_data);
        assert_eq!(option_data.call_ask, None);
        assert_eq!(option_data.call_bid, None);
        assert_eq!(option_data.put_ask, None);
        assert_eq!(option_data.put_bid, None);
        assert_eq!(option_data.implied_volatility, None);
        assert_eq!(option_data.delta, None);
        assert_eq!(option_data.strike_price, f2p!(100.0));
    }

    #[test]
    fn test_calculate_prices_override_volatility() {
        setup_logger();
        let mut option_data = OptionData::new(
            f2p!(100.0),
            None,
            None,
            None,
            None,
            sf2p!(0.2),
            None,
            None,
            None,
        );

        let price_params = OptionDataPriceParams::new(
            f2p!(110.0),
            ExpirationDate::Days(30.0),
            sf2p!(0.12),
            0.05,
            0.01,
        );
        let result = option_data.calculate_prices(&price_params);

        assert!(result.is_ok());
        info!("{}", option_data);
        assert_eq!(option_data.call_ask, sf2p!(10.412135042233587));
        assert_eq!(option_data.call_bid, sf2p!(10.412135042233587));
        assert_eq!(option_data.put_ask, sf2p!(0.002019418653973759));
        assert_eq!(option_data.put_bid, sf2p!(0.002019418653973759));
        option_data.apply_spread(f2p!(0.02), 2);
        info!("{}", option_data);
        assert_eq!(option_data.call_ask, sf2p!(10.42));
        assert_eq!(option_data.call_bid, sf2p!(10.4));
        assert_eq!(option_data.put_ask, None);
        assert_eq!(option_data.put_bid, None);
    }

    #[test]
    fn test_calculate_prices_with_all_parameters() {
        let mut option_data = OptionData::new(
            f2p!(100.0),
            None,
            None,
            None,
            None,
            sf2p!(0.2),
            None,
            None,
            None,
        );
        let price_params = OptionDataPriceParams::new(
            f2p!(100.0),
            ExpirationDate::Days(30.0),
            sf2p!(0.2),
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
    use crate::error::chains::ChainBuildErrorKind;
    use crate::model::types::ExpirationDate;
    use crate::f2p;
    use crate::utils::logger::setup_logger;

    fn create_test_chain() -> OptionChain {
        // Create a sample option chain
        let mut chain = OptionChain::new("TEST", f2p!(100.0), "2024-01-01".to_string(), None, None);

        // Add some test options with different strikes
        chain.add_option(
            f2p!(95.0),   // strike_price
            sf2p!(4.0),   // call_bid
            sf2p!(4.2),   // call_ask
            sf2p!(3.0),   // put_bid
            sf2p!(3.2),   // put_ask
            sf2p!(0.2),   // implied_volatility
            Some(0.5),    // delta
            sf2p!(100.0), // volume
            Some(50),     // open_interest
        );

        chain.add_option(
            f2p!(100.0),
            sf2p!(3.0),
            sf2p!(3.2),
            sf2p!(3.0),
            sf2p!(3.2),
            sf2p!(0.2),
            Some(0.5),
            sf2p!(100.0),
            Some(50),
        );

        chain.add_option(
            f2p!(105.0),
            sf2p!(2.0),
            sf2p!(2.2),
            sf2p!(4.0),
            sf2p!(4.2),
            sf2p!(0.2),
            Some(0.5),
            sf2p!(100.0),
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
            f2p!(1.0),
            0.05,
            0.02,
            1.0,
            1.0,
            1.0,
            1.0,
        );
        let result = chain.get_random_positions(params);
        assert!(result.is_err());
        let error = result.unwrap_err();
        match error {
            ChainError::ChainBuildError(ChainBuildErrorKind::InvalidParameters {
                parameter,
                reason,
            }) => {
                assert_eq!(parameter, "total_positions");
                assert_eq!(reason, "The sum of the quantities must be greater than 0");
            }
            _ => panic!("Incorrect error type"),
        }
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
            f2p!(1.0),
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
            f2p!(1.0),
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
            f2p!(1.0),
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
            f2p!(1.0),
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
            f2p!(1.0),
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
        let chain = OptionChain::new("TEST", f2p!(100.0), "2024-01-01".to_string(), None, None);
        let params = RandomPositionsParams::new(
            Some(1),
            None,
            None,
            None,
            ExpirationDate::Days(30.0),
            f2p!(1.0),
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
    use crate::f2p;
    use crate::sf2p;

    fn create_test_option_data() -> OptionData {
        OptionData::new(
            f2p!(100.0),
            sf2p!(9.5),
            sf2p!(10.0),
            sf2p!(8.5),
            sf2p!(9.0),
            sf2p!(0.2),
            Some(-0.3),
            sf2p!(1000.0),
            Some(500),
        )
    }

    #[test]
    fn test_get_call_buy_price() {
        let data = create_test_option_data();
        assert_eq!(data.get_call_buy_price(), sf2p!(10.0));
    }

    #[test]
    fn test_get_call_sell_price() {
        let data = create_test_option_data();
        assert_eq!(data.get_call_sell_price(), sf2p!(9.5));
    }

    #[test]
    fn test_get_put_buy_price() {
        let data = create_test_option_data();
        assert_eq!(data.get_put_buy_price(), sf2p!(9.0));
    }

    #[test]
    fn test_get_put_sell_price() {
        let data = create_test_option_data();
        assert_eq!(data.get_put_sell_price(), sf2p!(8.5));
    }

    #[test]
    fn test_get_prices_with_none_values() {
        let data = OptionData::new(
            f2p!(100.0),
            None,
            None,
            None,
            None,
            sf2p!(0.2),
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
    use crate::f2p;
    use crate::sf2p;

    #[test]
    fn test_display_full_data() {
        let data = OptionData::new(
            f2p!(100.0),
            sf2p!(9.5),
            sf2p!(10.0),
            sf2p!(8.5),
            sf2p!(9.0),
            sf2p!(0.2),
            Some(-0.3),
            sf2p!(1000.0),
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
    use crate::f2p;

    fn create_test_chain() -> OptionChain {
        let mut chain = OptionChain::new("TEST", f2p!(100.0), "2024-01-01".to_string(), None, None);

        for strike in [90.0, 95.0, 100.0, 105.0, 110.0].iter() {
            chain.add_option(
                f2p!(*strike),
                None,
                None,
                None,
                None,
                sf2p!(0.2),
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
        let filtered = chain.filter_option_data(FindOptimalSide::Range(f2p!(95.0), f2p!(105.0)));
        assert_eq!(filtered.len(), 3);
        assert!(filtered
            .iter()
            .all(|opt| opt.strike_price >= f2p!(95.0) && opt.strike_price <= f2p!(105.0)));
    }
}

#[cfg(test)]
mod tests_strike_price_range_vec {
    use super::*;
    use crate::f2p;

    #[test]
    fn test_empty_chain() {
        let chain = OptionChain::new("TEST", f2p!(100.0), "2024-01-01".to_string(), None, None);
        assert_eq!(chain.strike_price_range_vec(5.0), None);
    }

    #[test]
    fn test_single_option() {
        let mut chain = OptionChain::new("TEST", f2p!(100.0), "2024-01-01".to_string(), None, None);
        chain.add_option(
            f2p!(100.0),
            None,
            None,
            None,
            None,
            sf2p!(0.2),
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
        let mut chain = OptionChain::new("TEST", f2p!(100.0), "2024-01-01".to_string(), None, None);
        for strike in [90.0, 95.0, 100.0].iter() {
            chain.add_option(
                f2p!(*strike),
                None,
                None,
                None,
                None,
                sf2p!(0.2),
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
        let mut chain = OptionChain::new("TEST", f2p!(100.0), "2024-01-01".to_string(), None, None);
        for strike in [90.0, 100.0].iter() {
            chain.add_option(
                f2p!(*strike),
                None,
                None,
                None,
                None,
                sf2p!(0.2),
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

#[cfg(test)]
mod tests_option_data_get_option {
    use super::*;
    use crate::error::chains::OptionDataErrorKind;
    use crate::model::types::ExpirationDate;

    fn create_test_option_data() -> OptionData {
        OptionData::new(
            f2p!(100.0),   // strike_price
            sf2p!(9.5),    // call_bid
            sf2p!(10.0),   // call_ask
            sf2p!(8.5),    // put_bid
            sf2p!(9.0),    // put_ask
            sf2p!(0.2),    // implied_volatility
            Some(-0.3),    // delta
            sf2p!(1000.0), // volume
            Some(500),     // open_interest
        )
    }

    #[test]
    fn test_get_option_success() {
        let option_data = create_test_option_data();
        let price_params = OptionDataPriceParams::new(
            f2p!(100.0),
            ExpirationDate::Days(30.0),
            sf2p!(0.25),
            0.05,
            0.02,
        );

        let result = option_data.get_option(&price_params, Side::Long, OptionStyle::Call);
        assert!(result.is_ok());

        let option = result.unwrap();
        assert_eq!(option.strike_price, f2p!(100.0));
        assert_eq!(option.implied_volatility, 0.25); // Uses provided IV
        assert_eq!(option.underlying_price, f2p!(100.0));
        assert_eq!(option.risk_free_rate, 0.05);
        assert_eq!(option.dividend_yield, 0.02);
        assert_eq!(option.side, Side::Long);
        assert_eq!(option.option_style, OptionStyle::Call);
    }

    #[test]
    fn test_get_option_using_data_iv() {
        let option_data = create_test_option_data();
        let price_params = OptionDataPriceParams::new(
            f2p!(100.0),
            ExpirationDate::Days(30.0),
            None, // No IV provided in params
            0.05,
            0.02,
        );

        let result = option_data.get_option(&price_params, Side::Long, OptionStyle::Call);
        assert!(result.is_ok());

        let option = result.unwrap();
        assert_eq!(option.implied_volatility, 0.2); // Uses IV from option_data
    }

    #[test]
    fn test_get_option_missing_iv() {
        let mut option_data = create_test_option_data();
        option_data.implied_volatility = None;

        let price_params =
            OptionDataPriceParams::new(f2p!(100.0), ExpirationDate::Days(30.0), None, 0.05, 0.02);

        let result = option_data.get_option(&price_params, Side::Long, OptionStyle::Call);
        assert!(result.is_err());

        let error = result.unwrap_err();
        match error {
            ChainError::OptionDataError(OptionDataErrorKind::InvalidVolatility {
                volatility,
                reason,
            }) => {
                assert_eq!(volatility, None);
                assert_eq!(reason, "Implied volatility not found");
            }
            _ => panic!("Incorrect error type"),
        }
    }
}

#[cfg(test)]
mod tests_option_data_get_options_in_strike {
    use super::*;
    use crate::assert_decimal_eq;
    use crate::error::chains::OptionDataErrorKind;
    use crate::model::types::ExpirationDate;
    use rust_decimal_macros::dec;

    fn create_test_option_data() -> OptionData {
        OptionData::new(
            f2p!(100.0),   // strike_price
            sf2p!(9.5),    // call_bid
            sf2p!(10.0),   // call_ask
            sf2p!(8.5),    // put_bid
            sf2p!(9.0),    // put_ask
            sf2p!(0.2),    // implied_volatility
            Some(-0.3),    // delta
            sf2p!(1000.0), // volume
            Some(500),     // open_interest
        )
    }

    #[test]
    fn test_get_options_in_strike_success() {
        let option_data = create_test_option_data();
        let price_params = OptionDataPriceParams::new(
            f2p!(100.0),
            ExpirationDate::Days(30.0),
            sf2p!(0.25),
            0.05,
            0.02,
        );

        let result =
            option_data.get_options_in_strike(&price_params, Side::Long, OptionStyle::Call);
        assert!(result.is_ok());

        let options = result.unwrap();

        // Check long call
        assert_eq!(options.long_call.strike_price, f2p!(100.0));
        assert_eq!(options.long_call.option_style, OptionStyle::Call);
        assert_eq!(options.long_call.side, Side::Long);

        // Check short call
        assert_eq!(options.short_call.strike_price, f2p!(100.0));
        assert_eq!(options.short_call.option_style, OptionStyle::Call);
        assert_eq!(options.short_call.side, Side::Short);

        // Check long put
        assert_eq!(options.long_put.strike_price, f2p!(100.0));
        assert_eq!(options.long_put.option_style, OptionStyle::Put);
        assert_eq!(options.long_put.side, Side::Long);

        // Check short put
        assert_eq!(options.short_put.strike_price, f2p!(100.0));
        assert_eq!(options.short_put.option_style, OptionStyle::Put);
        assert_eq!(options.short_put.side, Side::Short);
    }

    #[test]
    fn test_get_options_in_strike_using_data_iv() {
        let option_data = create_test_option_data();
        let price_params =
            OptionDataPriceParams::new(f2p!(100.0), ExpirationDate::Days(30.0), None, 0.05, 0.02);

        let result =
            option_data.get_options_in_strike(&price_params, Side::Long, OptionStyle::Call);
        assert!(result.is_ok());

        let options = result.unwrap();
        assert_eq!(options.long_call.implied_volatility, 0.2);
        assert_eq!(options.short_call.implied_volatility, 0.2);
        assert_eq!(options.long_put.implied_volatility, 0.2);
        assert_eq!(options.short_put.implied_volatility, 0.2);
    }

    #[test]
    fn test_get_options_in_strike_missing_iv() {
        let mut option_data = create_test_option_data();
        option_data.implied_volatility = None;

        let price_params =
            OptionDataPriceParams::new(f2p!(100.0), ExpirationDate::Days(30.0), None, 0.05, 0.02);

        let result =
            option_data.get_options_in_strike(&price_params, Side::Long, OptionStyle::Call);
        assert!(result.is_err());
        let error = result.unwrap_err();
        match error {
            ChainError::OptionDataError(OptionDataErrorKind::InvalidVolatility {
                volatility,
                reason,
            }) => {
                assert_eq!(volatility, None);
                assert_eq!(reason, "Implied volatility not found");
            }
            _ => panic!("Incorrect error type"),
        }
    }

    #[test]
    fn test_get_options_in_strike_all_properties() {
        let option_data = create_test_option_data();
        let price_params = OptionDataPriceParams::new(
            f2p!(110.0),
            ExpirationDate::Days(45.0),
            sf2p!(0.3),
            0.06,
            0.03,
        );

        let result =
            option_data.get_options_in_strike(&price_params, Side::Long, OptionStyle::Call);
        assert!(result.is_ok());

        let options = result.unwrap();

        // Verify common properties across all options
        let check_common_properties = |option: &Options| {
            assert_eq!(option.strike_price, f2p!(100.0));
            assert_eq!(option.underlying_price, f2p!(110.0));
            assert_eq!(option.implied_volatility, 0.3);
            assert_eq!(option.risk_free_rate, 0.06);
            assert_eq!(option.dividend_yield, 0.03);
            assert_eq!(option.option_type, OptionType::European);
            assert_eq!(option.quantity, f2p!(1.0));
        };

        check_common_properties(&options.long_call);
        check_common_properties(&options.short_call);
        check_common_properties(&options.long_put);
        check_common_properties(&options.short_put);
    }

    #[test]
    fn test_get_options_in_strike_deltas() {
        let option_data = create_test_option_data();
        let price_params = OptionDataPriceParams::new(
            f2p!(110.0),
            ExpirationDate::Days(45.0),
            sf2p!(0.3),
            0.06,
            0.03,
        );

        let result =
            option_data.get_options_in_strike(&price_params, Side::Long, OptionStyle::Call);
        assert!(result.is_ok());

        let options = result.unwrap();

        let epsilon = dec!(1e-8);

        assert_decimal_eq!(
            options.long_call.delta().unwrap(),
            dec!(0.844825189),
            epsilon
        );
        assert_decimal_eq!(
            options.short_call.delta().unwrap(),
            dec!(-0.844825189),
            epsilon
        );
        assert_decimal_eq!(
            options.long_put.delta().unwrap(),
            dec!(-0.151483012),
            epsilon
        );
        assert_decimal_eq!(
            options.short_put.delta().unwrap(),
            dec!(0.151483012),
            epsilon
        );
    }
}

#[cfg(test)]
mod tests_filter_options_in_strike {
    use super::*;
    use crate::model::types::ExpirationDate;
    use crate::{f2p, sf2p};
    use rust_decimal::Decimal;

    fn create_test_chain() -> OptionChain {
        let mut chain = OptionChain::new("TEST", f2p!(100.0), "2024-01-01".to_string(), None, None);

        for strike in [90.0, 95.0, 100.0, 105.0, 110.0].iter() {
            chain.add_option(
                f2p!(*strike),
                sf2p!(1.0),    // call_bid
                sf2p!(1.2),    // call_ask
                sf2p!(1.0),    // put_bid
                sf2p!(1.2),    // put_ask
                sf2p!(0.2),    // implied_volatility
                Some(-0.3),    // delta
                sf2p!(1000.0), // volume
                Some(500),     // open_interest
            );
        }
        chain
    }

    #[test]
    fn test_filter_upper_strikes() {
        let chain = create_test_chain();
        let price_params = OptionDataPriceParams::new(
            f2p!(100.0),
            ExpirationDate::Days(30.0),
            sf2p!(0.2),
            0.05,
            0.02,
        );

        let result = chain.filter_options_in_strike(&price_params, FindOptimalSide::Upper);
        assert!(result.is_ok());

        let filtered_options = result.unwrap();
        assert_eq!(filtered_options.len(), 2);

        for opt in filtered_options {
            assert!(opt.long_call.strike_price > chain.underlying_price);
            assert_eq!(opt.long_call.option_type, OptionType::European);
            assert_eq!(opt.long_call.side, Side::Long);
            assert_eq!(opt.short_call.side, Side::Short);
            assert_eq!(opt.long_put.side, Side::Long);
            assert_eq!(opt.short_put.side, Side::Short);
        }
    }

    #[test]
    fn test_filter_lower_strikes() {
        let chain = create_test_chain();
        let price_params = OptionDataPriceParams::new(
            f2p!(100.0),
            ExpirationDate::Days(30.0),
            sf2p!(0.2),
            0.05,
            0.02,
        );

        let result = chain.filter_options_in_strike(&price_params, FindOptimalSide::Lower);
        assert!(result.is_ok());

        let filtered_options = result.unwrap();
        assert_eq!(filtered_options.len(), 2);

        for opt in filtered_options {
            assert!(opt.long_call.strike_price < chain.underlying_price);
        }
    }

    #[test]
    fn test_filter_all_strikes() {
        let chain = create_test_chain();
        let price_params = OptionDataPriceParams::new(
            f2p!(100.0),
            ExpirationDate::Days(30.0),
            sf2p!(0.2),
            0.05,
            0.02,
        );

        let result = chain.filter_options_in_strike(&price_params, FindOptimalSide::All);
        assert!(result.is_ok());

        let filtered_options = result.unwrap();
        assert_eq!(filtered_options.len(), 5);
    }

    #[test]
    fn test_filter_range_strikes() {
        let chain = create_test_chain();
        let price_params = OptionDataPriceParams::new(
            f2p!(100.0),
            ExpirationDate::Days(30.0),
            sf2p!(0.2),
            0.05,
            0.02,
        );

        let result = chain.filter_options_in_strike(
            &price_params,
            FindOptimalSide::Range(f2p!(95.0), f2p!(105.0)),
        );
        assert!(result.is_ok());

        let filtered_options = result.unwrap();
        assert_eq!(filtered_options.len(), 3);

        for opt in filtered_options {
            assert!(opt.long_call.strike_price >= f2p!(95.0));
            assert!(opt.long_call.strike_price <= f2p!(105.0));
        }
    }

    #[test]
    fn test_filter_empty_chain() {
        let chain = OptionChain::new("TEST", f2p!(100.0), "2024-01-01".to_string(), None, None);
        let price_params = OptionDataPriceParams::new(
            f2p!(100.0),
            ExpirationDate::Days(30.0),
            sf2p!(0.2),
            0.05,
            0.02,
        );

        let result = chain.filter_options_in_strike(&price_params, FindOptimalSide::All);
        assert!(result.is_ok());

        let filtered_options = result.unwrap();
        assert!(filtered_options.is_empty());
    }

    #[test]
    fn test_filter_invalid_range() {
        let chain = create_test_chain();
        let price_params = OptionDataPriceParams::new(
            f2p!(100.0),
            ExpirationDate::Days(30.0),
            sf2p!(0.2),
            0.05,
            0.02,
        );

        let result = chain.filter_options_in_strike(
            &price_params,
            FindOptimalSide::Range(f2p!(200.0), f2p!(300.0)),
        );
        assert!(result.is_ok());

        let filtered_options = result.unwrap();
        assert!(filtered_options.is_empty());
    }

    #[test]
    fn test_filter_all_strikes_deltas() {
        let chain = create_test_chain();
        let price_params = OptionDataPriceParams::new(
            f2p!(100.0),
            ExpirationDate::Days(30.0),
            sf2p!(0.2),
            0.05,
            0.02,
        );

        let result = chain.filter_options_in_strike(&price_params, FindOptimalSide::All);
        assert!(result.is_ok());

        let filtered_options = result.unwrap();
        assert_eq!(filtered_options.len(), 5);

        for opt in filtered_options {
            assert_eq!(opt.long_call.option_type, OptionType::European);
            assert_eq!(opt.long_call.side, Side::Long);
            assert_eq!(opt.short_call.side, Side::Short);
            assert_eq!(opt.long_put.side, Side::Long);
            assert_eq!(opt.short_put.side, Side::Short);

            let deltas = opt.deltas().unwrap();
            assert!(deltas.long_call > Decimal::ZERO);
            assert!(deltas.short_call < Decimal::ZERO);
            assert!(deltas.long_put < Decimal::ZERO);
            assert!(deltas.short_put > Decimal::ZERO);
        }
    }
}

#[cfg(test)]
mod tests_chain_iterators {
    use super::*;
    use crate::sf2p;

    fn create_test_chain() -> OptionChain {
        let mut chain = OptionChain::new("TEST", f2p!(100.0), "2024-01-01".to_string(), None, None);

        // Add three options with different strikes
        chain.add_option(
            f2p!(90.0),   // strike_price
            sf2p!(5.0),   // call_bid
            sf2p!(5.5),   // call_ask
            sf2p!(1.0),   // put_bid
            sf2p!(1.5),   // put_ask
            sf2p!(0.2),   // implied_volatility
            Some(0.6),    // delta
            sf2p!(100.0), // volume
            Some(50),     // open_interest
        );

        chain.add_option(
            f2p!(100.0),
            sf2p!(3.0),
            sf2p!(3.5),
            sf2p!(3.0),
            sf2p!(3.5),
            sf2p!(0.25),
            Some(0.5),
            sf2p!(150.0),
            Some(75),
        );

        chain.add_option(
            f2p!(110.0),
            sf2p!(1.0),
            sf2p!(1.5),
            sf2p!(5.0),
            sf2p!(5.5),
            sf2p!(0.3),
            Some(0.4),
            sf2p!(80.0),
            Some(40),
        );

        chain
    }

    #[test]
    fn test_get_double_iter_empty() {
        let chain = OptionChain::new("TEST", f2p!(100.0), "2024-01-01".to_string(), None, None);
        let pairs: Vec<_> = chain.get_double_iter().collect();
        assert!(pairs.is_empty());
    }

    #[test]
    fn test_get_double_iter_single() {
        let mut chain = OptionChain::new("TEST", f2p!(100.0), "2024-01-01".to_string(), None, None);
        chain.add_option(
            f2p!(100.0),
            sf2p!(3.0),
            sf2p!(3.5),
            sf2p!(3.0),
            sf2p!(3.5),
            sf2p!(0.25),
            Some(0.5),
            sf2p!(150.0),
            Some(75),
        );

        let pairs: Vec<_> = chain.get_double_iter().collect();
        assert!(pairs.is_empty()); // No pairs with single element
    }

    #[test]
    fn test_get_double_iter_multiple() {
        let chain = create_test_chain();
        let pairs: Vec<_> = chain.get_double_iter().collect();

        // Should have 3 pairs: (90,100), (90,110), (100,110)
        assert_eq!(pairs.len(), 3);

        // Check strikes of pairs
        assert_eq!(pairs[0].0.strike_price, f2p!(90.0));
        assert_eq!(pairs[0].1.strike_price, f2p!(100.0));

        assert_eq!(pairs[1].0.strike_price, f2p!(90.0));
        assert_eq!(pairs[1].1.strike_price, f2p!(110.0));

        assert_eq!(pairs[2].0.strike_price, f2p!(100.0));
        assert_eq!(pairs[2].1.strike_price, f2p!(110.0));
    }

    #[test]
    fn test_get_double_inclusive_iter_empty() {
        let chain = OptionChain::new("TEST", f2p!(100.0), "2024-01-01".to_string(), None, None);
        let pairs: Vec<_> = chain.get_double_inclusive_iter().collect();
        assert!(pairs.is_empty());
    }

    #[test]
    fn test_get_double_inclusive_iter_single() {
        let mut chain = OptionChain::new("TEST", f2p!(100.0), "2024-01-01".to_string(), None, None);
        chain.add_option(
            f2p!(100.0),
            sf2p!(3.0),
            sf2p!(3.5),
            sf2p!(3.0),
            sf2p!(3.5),
            sf2p!(0.25),
            Some(0.5),
            sf2p!(150.0),
            Some(75),
        );

        let pairs: Vec<_> = chain.get_double_inclusive_iter().collect();
        assert_eq!(pairs.len(), 1); // Should have one pair (self-pair)
        assert_eq!(pairs[0].0.strike_price, pairs[0].1.strike_price);
    }

    #[test]
    fn test_get_double_inclusive_iter_multiple() {
        let chain = create_test_chain();
        let pairs: Vec<_> = chain.get_double_inclusive_iter().collect();

        // Should have 6 pairs: (90,90), (90,100), (90,110), (100,100), (100,110), (110,110)
        assert_eq!(pairs.len(), 6);

        // Check strikes of pairs
        assert_eq!(pairs[0].0.strike_price, f2p!(90.0));
        assert_eq!(pairs[0].1.strike_price, f2p!(90.0));

        assert_eq!(pairs[1].0.strike_price, f2p!(90.0));
        assert_eq!(pairs[1].1.strike_price, f2p!(100.0));

        assert_eq!(pairs[2].0.strike_price, f2p!(90.0));
        assert_eq!(pairs[2].1.strike_price, f2p!(110.0));

        assert_eq!(pairs[3].0.strike_price, f2p!(100.0));
        assert_eq!(pairs[3].1.strike_price, f2p!(100.0));

        assert_eq!(pairs[4].0.strike_price, f2p!(100.0));
        assert_eq!(pairs[4].1.strike_price, f2p!(110.0));

        assert_eq!(pairs[5].0.strike_price, f2p!(110.0));
        assert_eq!(pairs[5].1.strike_price, f2p!(110.0));
    }
}

#[cfg(test)]
mod tests_chain_iterators_bis {
    use super::*;
    use crate::sf2p;

    fn create_test_chain() -> OptionChain {
        let mut chain = OptionChain::new("TEST", f2p!(100.0), "2024-01-01".to_string(), None, None);

        // Add four options with different strikes
        chain.add_option(
            f2p!(90.0),   // strike_price
            sf2p!(5.0),   // call_bid
            sf2p!(5.5),   // call_ask
            sf2p!(1.0),   // put_bid
            sf2p!(1.5),   // put_ask
            sf2p!(0.2),   // implied_volatility
            Some(0.6),    // delta
            sf2p!(100.0), // volume
            Some(50),     // open_interest
        );

        chain.add_option(
            f2p!(100.0),
            sf2p!(3.0),
            sf2p!(3.5),
            sf2p!(3.0),
            sf2p!(3.5),
            sf2p!(0.25),
            Some(0.5),
            sf2p!(150.0),
            Some(75),
        );

        chain.add_option(
            f2p!(110.0),
            sf2p!(1.0),
            sf2p!(1.5),
            sf2p!(5.0),
            sf2p!(5.5),
            sf2p!(0.3),
            Some(0.4),
            sf2p!(80.0),
            Some(40),
        );

        chain.add_option(
            f2p!(120.0),
            sf2p!(0.5),
            sf2p!(1.0),
            sf2p!(7.0),
            sf2p!(7.5),
            sf2p!(0.35),
            Some(0.3),
            sf2p!(60.0),
            Some(30),
        );

        chain
    }

    // Tests for Triple Iterator
    #[test]
    fn test_get_triple_iter_empty() {
        let chain = OptionChain::new("TEST", f2p!(100.0), "2024-01-01".to_string(), None, None);
        let triples: Vec<_> = chain.get_triple_iter().collect();
        assert!(triples.is_empty());
    }

    #[test]
    fn test_get_triple_iter_two_elements() {
        let mut chain = OptionChain::new("TEST", f2p!(100.0), "2024-01-01".to_string(), None, None);
        // Add two options
        chain.add_option(f2p!(90.0), None, None, None, None, None, None, None, None);
        chain.add_option(f2p!(100.0), None, None, None, None, None, None, None, None);

        let triples: Vec<_> = chain.get_triple_iter().collect();
        assert!(triples.is_empty()); // Not enough elements for a triple
    }

    #[test]
    fn test_get_triple_iter_multiple() {
        let chain = create_test_chain();
        let triples: Vec<_> = chain.get_triple_iter().collect();

        // Should have 4 triples: (90,100,110), (90,100,120), (90,110,120), (100,110,120)
        assert_eq!(triples.len(), 4);

        // Check first triple
        assert_eq!(triples[0].0.strike_price, f2p!(90.0));
        assert_eq!(triples[0].1.strike_price, f2p!(100.0));
        assert_eq!(triples[0].2.strike_price, f2p!(110.0));

        // Check last triple
        assert_eq!(triples[3].0.strike_price, f2p!(100.0));
        assert_eq!(triples[3].1.strike_price, f2p!(110.0));
        assert_eq!(triples[3].2.strike_price, f2p!(120.0));
    }

    // Tests for Triple Inclusive Iterator
    #[test]
    fn test_get_triple_inclusive_iter_empty() {
        let chain = OptionChain::new("TEST", f2p!(100.0), "2024-01-01".to_string(), None, None);
        let triples: Vec<_> = chain.get_triple_inclusive_iter().collect();
        assert!(triples.is_empty());
    }

    #[test]
    fn test_get_triple_inclusive_iter_single() {
        let mut chain = OptionChain::new("TEST", f2p!(100.0), "2024-01-01".to_string(), None, None);
        chain.add_option(f2p!(100.0), None, None, None, None, None, None, None, None);

        let triples: Vec<_> = chain.get_triple_inclusive_iter().collect();
        assert_eq!(triples.len(), 1);
        assert_eq!(triples[0].0.strike_price, triples[0].1.strike_price);
        assert_eq!(triples[0].1.strike_price, triples[0].2.strike_price);
    }

    #[test]
    fn test_get_triple_inclusive_iter_multiple() {
        let chain = create_test_chain();
        let triples: Vec<_> = chain.get_triple_inclusive_iter().collect();

        // Count should be (n+2)(n+1)n/6 where n is the number of elements
        assert_eq!(triples.len(), 20); // For 4 elements: 4*5*6/6 = 20

        // Check first few triples (including self-references)
        assert_eq!(triples[0].0.strike_price, f2p!(90.0));
        assert_eq!(triples[0].1.strike_price, f2p!(90.0));
        assert_eq!(triples[0].2.strike_price, f2p!(90.0));
    }

    // Tests for Quad Iterator
    #[test]
    fn test_get_quad_iter_empty() {
        let chain = OptionChain::new("TEST", f2p!(100.0), "2024-01-01".to_string(), None, None);
        let quads: Vec<_> = chain.get_quad_iter().collect();
        assert!(quads.is_empty());
    }

    #[test]
    fn test_get_quad_iter_three_elements() {
        let mut chain = OptionChain::new("TEST", f2p!(100.0), "2024-01-01".to_string(), None, None);
        // Add three options
        chain.add_option(f2p!(90.0), None, None, None, None, None, None, None, None);
        chain.add_option(f2p!(100.0), None, None, None, None, None, None, None, None);
        chain.add_option(f2p!(110.0), None, None, None, None, None, None, None, None);

        let quads: Vec<_> = chain.get_quad_iter().collect();
        assert!(quads.is_empty()); // Not enough elements for a quad
    }

    #[test]
    fn test_get_quad_iter_multiple() {
        let chain = create_test_chain();
        let quads: Vec<_> = chain.get_quad_iter().collect();

        // Should have 1 quad: (90,100,110,120)
        assert_eq!(quads.len(), 1);

        // Check the quad
        assert_eq!(quads[0].0.strike_price, f2p!(90.0));
        assert_eq!(quads[0].1.strike_price, f2p!(100.0));
        assert_eq!(quads[0].2.strike_price, f2p!(110.0));
        assert_eq!(quads[0].3.strike_price, f2p!(120.0));
    }

    // Tests for Quad Inclusive Iterator
    #[test]
    fn test_get_quad_inclusive_iter_empty() {
        let chain = OptionChain::new("TEST", f2p!(100.0), "2024-01-01".to_string(), None, None);
        let quads: Vec<_> = chain.get_quad_inclusive_iter().collect();
        assert!(quads.is_empty());
    }

    #[test]
    fn test_get_quad_inclusive_iter_single() {
        let mut chain = OptionChain::new("TEST", f2p!(100.0), "2024-01-01".to_string(), None, None);
        chain.add_option(f2p!(100.0), None, None, None, None, None, None, None, None);

        let quads: Vec<_> = chain.get_quad_inclusive_iter().collect();
        assert_eq!(quads.len(), 1);
        assert_eq!(quads[0].0.strike_price, quads[0].1.strike_price);
        assert_eq!(quads[0].1.strike_price, quads[0].2.strike_price);
        assert_eq!(quads[0].2.strike_price, quads[0].3.strike_price);
    }

    #[test]
    fn test_get_quad_inclusive_iter_multiple() {
        let chain = create_test_chain();
        let quads: Vec<_> = chain.get_quad_inclusive_iter().collect();

        // Count should be (n+3)(n+2)(n+1)n/24 where n is the number of elements
        assert_eq!(quads.len(), 35); // For 4 elements: 7*6*5*4/24 = 35

        // Check first quad (self-reference)
        assert_eq!(quads[0].0.strike_price, f2p!(90.0));
        assert_eq!(quads[0].1.strike_price, f2p!(90.0));
        assert_eq!(quads[0].2.strike_price, f2p!(90.0));
        assert_eq!(quads[0].3.strike_price, f2p!(90.0));

        // Check last quad
        assert_eq!(quads[34].0.strike_price, f2p!(120.0));
        assert_eq!(quads[34].1.strike_price, f2p!(120.0));
        assert_eq!(quads[34].2.strike_price, f2p!(120.0));
        assert_eq!(quads[34].3.strike_price, f2p!(120.0));
    }
}

#[cfg(test)]
mod tests_is_valid_optimal_side {
    use super::*;

    #[test]
    fn test_upper_side_valid() {
        let option_data = OptionData::new(
            f2p!(110.0), // strike price higher than underlying
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        );
        let underlying_price = f2p!(100.0);

        assert!(option_data.is_valid_optimal_side(underlying_price, &FindOptimalSide::Upper));
    }

    #[test]
    fn test_upper_side_invalid() {
        let option_data = OptionData::new(
            f2p!(90.0), // strike price lower than underlying
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        );
        let underlying_price = f2p!(100.0);

        assert!(!option_data.is_valid_optimal_side(underlying_price, &FindOptimalSide::Upper));
    }

    #[test]
    fn test_lower_side_valid() {
        let option_data = OptionData::new(
            f2p!(90.0), // strike price lower than underlying
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        );
        let underlying_price = f2p!(100.0);

        assert!(option_data.is_valid_optimal_side(underlying_price, &FindOptimalSide::Lower));
    }

    #[test]
    fn test_lower_side_invalid() {
        let option_data = OptionData::new(
            f2p!(110.0), // strike price higher than underlying
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        );
        let underlying_price = f2p!(100.0);

        assert!(!option_data.is_valid_optimal_side(underlying_price, &FindOptimalSide::Lower));
    }

    #[test]
    fn test_all_side() {
        let option_data =
            OptionData::new(f2p!(100.0), None, None, None, None, None, None, None, None);
        let underlying_price = f2p!(100.0);

        assert!(option_data.is_valid_optimal_side(underlying_price, &FindOptimalSide::All));
    }

    #[test]
    fn test_range_side_valid() {
        let option_data = OptionData::new(
            f2p!(100.0), // strike price within range
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        );
        let range_start = f2p!(90.0);
        let range_end = f2p!(110.0);

        assert!(option_data
            .is_valid_optimal_side(f2p!(100.0), &FindOptimalSide::Range(range_start, range_end)));
    }

    #[test]
    fn test_range_side_invalid_below() {
        let option_data = OptionData::new(
            f2p!(80.0), // strike price below range
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        );
        let range_start = f2p!(90.0);
        let range_end = f2p!(110.0);

        assert!(!option_data
            .is_valid_optimal_side(f2p!(100.0), &FindOptimalSide::Range(range_start, range_end)));
    }

    #[test]
    fn test_range_side_invalid_above() {
        let option_data = OptionData::new(
            f2p!(120.0), // strike price above range
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        );
        let range_start = f2p!(90.0);
        let range_end = f2p!(110.0);

        assert!(!option_data
            .is_valid_optimal_side(f2p!(100.0), &FindOptimalSide::Range(range_start, range_end)));
    }

    #[test]
    fn test_range_side_at_boundaries() {
        let option_data_lower = OptionData::new(
            f2p!(90.0), // strike price at lower boundary
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        );
        let option_data_upper = OptionData::new(
            f2p!(110.0), // strike price at upper boundary
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        );
        let range_start = f2p!(90.0);
        let range_end = f2p!(110.0);

        assert!(option_data_lower
            .is_valid_optimal_side(f2p!(100.0), &FindOptimalSide::Range(range_start, range_end)));
        assert!(option_data_upper
            .is_valid_optimal_side(f2p!(100.0), &FindOptimalSide::Range(range_start, range_end)));
    }
}
