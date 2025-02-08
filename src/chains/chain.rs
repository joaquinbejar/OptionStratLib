/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 26/9/24
******************************************************************************/
use crate::chains::utils::{
    adjust_volatility, default_empty_string, generate_list_of_strikes, OptionChainBuildParams,
    OptionChainParams, OptionDataPriceParams, RandomPositionsParams,
};
use crate::chains::{
    DeltasInStrike, FourOptions, OptionsInStrike, RNDAnalysis, RNDParameters, RNDResult,
};
use crate::curves::{Curve, Point2D};
use crate::error::chains::ChainError;
use crate::geometrics::LinearInterpolation;
use crate::greeks::{delta, gamma};
use crate::model::{ExpirationDate, OptionStyle, OptionType, Options, Position, Side};
use crate::strategies::utils::FindOptimalSide;
use crate::utils::others::get_random_element;
use crate::volatility::VolatilitySmile;
use crate::{pos, Positive};
use chrono::{NaiveDate, Utc};
use num_traits::{FromPrimitive, ToPrimitive};
use rust_decimal::{Decimal, MathematicalOps};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt;
use tracing::{debug, error, info, trace};
#[cfg(not(target_arch = "wasm32"))]
use {crate::chains::utils::parse, csv::WriterBuilder, std::fs::File};

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
    pub call_middle: Option<Positive>,
    pub put_middle: Option<Positive>,
    pub(crate) implied_volatility: Option<Positive>,
    delta_call: Option<Decimal>,
    delta_put: Option<Decimal>,
    gamma: Option<Decimal>,
    volume: Option<Positive>,
    open_interest: Option<u64>,
    #[serde(skip)]
    pub options: Option<Box<FourOptions>>,
}

impl OptionData {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        strike_price: Positive,
        call_bid: Option<Positive>,
        call_ask: Option<Positive>,
        put_bid: Option<Positive>,
        put_ask: Option<Positive>,
        implied_volatility: Option<Positive>,
        delta_call: Option<Decimal>,
        delta_put: Option<Decimal>,
        gamma: Option<Decimal>,
        volume: Option<Positive>,
        open_interest: Option<u64>,
    ) -> Self {
        OptionData {
            strike_price,
            call_bid,
            call_ask,
            put_bid,
            put_ask,
            call_middle: None,
            put_middle: None,
            implied_volatility,
            delta_call,
            delta_put,
            gamma,
            volume,
            open_interest,
            options: None,
        }
    }

    pub fn validate(&self) -> bool {
        if self.strike_price == Positive::ZERO {
            error!("Error: Strike price cannot be zero");
            return false;
        }
        if self.implied_volatility.is_none() {
            error!("Error: Implied volatility cannot be None");
            return false;
        }
        if !self.valid_call() {
            error!("Error: Invalid call");
            return false;
        }
        if !self.valid_put() {
            error!("Error: Invalid put");
            return false;
        }
        true
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
            Some(iv) => iv,
            None => match self.implied_volatility {
                Some(iv) => iv / Positive::HUNDRED,
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
            implied_volatility,
            pos!(1.0),
            price_params.underlying_price,
            price_params.risk_free_rate,
            option_style,
            price_params.dividend_yield,
            None,
        ))
    }

    fn get_option_for_iv(
        &self,
        price_params: &OptionDataPriceParams,
        side: Side,
        option_style: OptionStyle,
        initial_iv: Positive,
    ) -> Result<Options, ChainError> {
        Ok(Options::new(
            OptionType::European,
            side,
            "OptionData".to_string(),
            self.strike_price,
            price_params.expiration_date.clone(),
            initial_iv,
            pos!(1.0),
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
        if self.options.is_none() {
            self.create_options(price_params)?;
        }

        let options = self.options.as_ref().unwrap();

        let call_ask = options.long_call.calculate_price_black_scholes()?;
        self.call_ask = Some(Positive(call_ask.abs()));

        let call_bid = options.short_call.calculate_price_black_scholes()?;
        self.call_bid = Some(Positive(call_bid.abs()));

        let put_ask = options.long_put.calculate_price_black_scholes()?;
        self.put_ask = Some(Positive(put_ask.abs()));

        let put_bid = options.short_put.calculate_price_black_scholes()?;
        self.put_bid = Some(Positive(put_bid.abs()));

        self.set_mid_prices();
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

        self.set_mid_prices();
    }

    pub fn calculate_delta(&mut self, price_params: &OptionDataPriceParams) {
        if self.implied_volatility.is_none() {
            trace!("Implied volatility not found, calculating it");
            if let Err(e) = self.calculate_implied_volatility(price_params) {
                error!("Failed to calculate implied volatility: {}", e);
                return;
            }
        }

        // Now proceed with delta calculation
        let option: Options = match self.get_option(price_params, Side::Long, OptionStyle::Call) {
            Ok(option) => option,
            Err(e) => {
                error!("Failed to get option for delta calculation: {}", e);
                return;
            }
        };

        match delta(&option) {
            Ok(d) => self.delta_call = Some(d),
            Err(e) => {
                error!("Delta calculation failed: {}", e);
                self.delta_call = None;
            }
        }

        let option: Options = match self.get_option(price_params, Side::Long, OptionStyle::Put) {
            Ok(option) => option,
            Err(e) => {
                error!("Failed to get option for delta calculation: {}", e);
                return;
            }
        };

        match delta(&option) {
            Ok(d) => self.delta_put = Some(d),
            Err(e) => {
                error!("Delta calculation failed: {}", e);
                self.delta_put = None;
            }
        }
    }

    pub fn calculate_gamma(&mut self, price_params: &OptionDataPriceParams) {
        if self.implied_volatility.is_none() {
            trace!("Implied volatility not found, calculating it");
            if let Err(e) = self.calculate_implied_volatility(price_params) {
                error!("Failed to calculate implied volatility: {}", e);
                return;
            }
        }

        // Now proceed with delta calculation
        let option: Options = match self.get_option(price_params, Side::Long, OptionStyle::Call) {
            Ok(option) => option,
            Err(e) => {
                error!("Failed to get option for delta calculation: {}", e);
                return;
            }
        };

        match gamma(&option) {
            Ok(d) => self.gamma = Some(d),
            Err(e) => {
                error!("Gamma calculation failed: {}", e);
                self.gamma = None;
            }
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

    pub fn set_mid_prices(&mut self) {
        self.call_middle = match (self.call_bid, self.call_ask) {
            (Some(bid), Some(ask)) => Some((bid + ask) / pos!(2.0)),
            _ => None,
        };
        self.put_middle = match (self.put_bid, self.put_ask) {
            (Some(bid), Some(ask)) => Some((bid + ask) / pos!(2.0)),
            _ => None,
        };
    }

    pub fn get_mid_prices(&self) -> (Option<Positive>, Option<Positive>) {
        (self.call_middle, self.put_middle)
    }

    pub fn calculate_implied_volatility(
        &mut self,
        price_params: &OptionDataPriceParams,
    ) -> Result<(), ChainError> {
        trace!(
            "call_middle {:?} put_middle {:?}",
            self.call_middle,
            self.put_middle
        );
        if self.call_middle.is_none() || self.put_middle.is_none() {
            info!("Calculation middel prices for IV calculation:");
            self.calculate_prices(price_params)?;
        }

        // Try to calculate IV for calls if we have mid price
        if let Some(call_price) = self.call_middle {
            // Initial IV guess based on moneyness
            let initial_iv = if price_params.underlying_price > self.strike_price {
                pos!(0.5) // ITM
            } else {
                pos!(0.3) // OTM
            };

            let option =
                self.get_option_for_iv(price_params, Side::Long, OptionStyle::Call, initial_iv)?;

            match option.calculate_implied_volatility(call_price.to_dec()) {
                Ok(iv) => {
                    debug!("Successfully calculated call IV: {}", iv);
                    self.implied_volatility = Some(iv * Positive::HUNDRED);
                    return Ok(());
                }
                Err(e) => {
                    debug!("Failed to calculate call IV: {}", e);
                }
            }
        }

        // If call IV calculation failed or wasn't possible, try puts
        if let Some(put_price) = self.put_middle {
            // Initial IV guess based on moneyness
            let initial_iv = if price_params.underlying_price < self.strike_price {
                pos!(5.0) // ITM
            } else {
                pos!(3.0) // OTM
            };

            let option =
                self.get_option_for_iv(price_params, Side::Long, OptionStyle::Put, initial_iv)?;

            match option.calculate_implied_volatility(put_price.to_dec()) {
                Ok(iv) => {
                    debug!("Successfully calculated put IV: {}", iv);
                    self.implied_volatility = Some(iv * Positive::HUNDRED);
                    return Ok(());
                }
                Err(e) => {
                    debug!("Failed to calculate put IV: {}", e);
                }
            }
        }

        Err(ChainError::invalid_volatility(
            None,
            "Could not calculate implied volatility from either calls or puts",
        ))
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
            call_middle: None,
            put_middle: None,
            implied_volatility: None,
            delta_call: None,
            delta_put: None,
            gamma: None,
            volume: None,
            open_interest: None,
            options: None,
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
            "{:<10} {:<10} {:<10} {:<10} {:<10} {:<10} {:<10} {:.3}{:<8} {:.3}{:<4} {:.3}{:<5} {:.2}{:<8} {:<10} {:<10}",
            self.strike_price.to_string(),
            default_empty_string(self.call_bid),
            default_empty_string(self.call_ask),
            default_empty_string(self.call_middle),
            default_empty_string(self.put_bid),
            default_empty_string(self.put_ask),
            default_empty_string(self.put_middle),
            self.implied_volatility.unwrap_or(Positive::ZERO),
            " ".to_string(),
            self.delta_call.unwrap_or(Decimal::ZERO),
            " ".to_string(),
            self.delta_put.unwrap_or(Decimal::ZERO),
            " ".to_string(),
            self.gamma.unwrap_or(Decimal::ZERO) * Decimal::ONE_HUNDRED,
            " ".to_string(),
            default_empty_string(self.volume),
            default_empty_string(self.open_interest),
        )?;
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OptionChain {
    pub symbol: String,
    pub underlying_price: Positive,
    expiration_date: String,
    pub(crate) options: BTreeSet<OptionData>,
    pub(crate) risk_free_rate: Option<Decimal>,
    pub(crate) dividend_yield: Option<Positive>,
}

impl OptionChain {
    pub fn new(
        symbol: &str,
        underlying_price: Positive,
        expiration_date: String,
        risk_free_rate: Option<Decimal>,
        dividend_yield: Option<Positive>,
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
            params
                .price_params
                .expiration_date
                .get_date_string()
                .unwrap(),
            None,
            None,
        );

        let strikes = generate_list_of_strikes(
            params.price_params.underlying_price,
            params.chain_size,
            params.strike_interval,
        );

        for strike in strikes {
            let atm_distance = strike.to_dec() - params.price_params.underlying_price;
            let adjusted_volatility = adjust_volatility(
                params.price_params.implied_volatility,
                params.skew_factor,
                atm_distance.to_f64().unwrap(),
            );
            let mut option_data = OptionData::new(
                strike,
                None,
                None,
                None,
                None,
                adjusted_volatility,
                None,
                None,
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
                params.price_params.underlying_symbol.clone(),
            );
            if option_data.calculate_prices(&price_params).is_ok() {
                option_data.apply_spread(params.spread, params.decimal_places);
                option_data.calculate_delta(&price_params);
                option_data.calculate_gamma(&price_params);
            }

            option_chain.options.insert(option_data);
        }
        debug!("Option chain: {}", option_chain);
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
        delta_call: Option<Decimal>,
        delta_put: Option<Decimal>,
        gamma: Option<Decimal>,
        volume: Option<Positive>,
        open_interest: Option<u64>,
    ) {
        let mut option_data = OptionData {
            strike_price,
            call_bid,
            call_ask,
            put_bid,
            put_ask,
            call_middle: None,
            put_middle: None,
            implied_volatility,
            delta_call,
            delta_put,
            gamma,
            volume,
            open_interest,
            options: None,
        };
        option_data.set_mid_prices();

        let expiration_date = match ExpirationDate::from_string(&self.expiration_date) {
            Ok(date) => date,
            Err(e) => {
                panic!("Failed to parse expiration date: {}", e);
            }
        };

        let params = OptionDataPriceParams::new(
            self.underlying_price,
            expiration_date,
            implied_volatility,
            self.risk_free_rate.unwrap_or(Decimal::ZERO),
            self.dividend_yield.unwrap_or(Positive::ZERO),
            Some(self.symbol.clone()),
        );

        if let Err(e) = option_data.create_options(&params) {
            error!(
                "Failed to create options for strike {}: {}",
                strike_price, e
            );
            println!(
                "Failed to create options for strike {}: {}",
                strike_price, e
            );
        }
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

    pub fn set_from_title(&mut self, file: &str) -> Result<(), Box<dyn Error>> {
        let file_name = file.split('/').next_back().unwrap();
        let file_name = file_name
            .rsplit_once('.')
            .map_or(file_name, |(name, _ext)| name);
        let parts: Vec<&str> = file_name.split('-').collect();
        if parts.len() != 5 {
            return Err("Invalid file name format: expected exactly 5 parts (symbol, day, month, year, price)".to_string().into());
        }

        self.symbol = parts[0].to_string();
        self.expiration_date = format!("{}-{}-{}", parts[1], parts[2], parts[3]);
        let underlying_price_str = parts[4].replace(",", ".");

        match underlying_price_str.parse::<f64>() {
            Ok(price) => {
                self.underlying_price = pos!(price);
                Ok(())
            }
            Err(_) => panic!("Invalid underlying price format in file name"),
        }
    }

    pub fn update_mid_prices(&mut self) {
        let modified_options: BTreeSet<OptionData> = self
            .options
            .iter()
            .map(|option| {
                let mut option = option.clone();
                option.set_mid_prices();
                option
            })
            .collect();

        self.options = modified_options;
    }

    pub fn update_greeks(&mut self) {
        let modified_options: BTreeSet<OptionData> = self
            .options
            .iter()
            .map(|option| {
                let mut option = option.clone(); // Create a clone we can modify
                let params = self.get_params(option.strike_price).unwrap();
                option.calculate_delta(&params);
                option.calculate_gamma(&params);
                option
            })
            .collect();

        self.options = modified_options;
    }

    pub fn update_implied_volatilities(&mut self) {
        let modified_options: BTreeSet<OptionData> = self
            .options
            .iter()
            .map(|option| {
                let mut option = option.clone();
                let params = self.get_params(option.strike_price).unwrap();
                if let Err(e) = option.calculate_implied_volatility(&params) {
                    debug!(
                        "Failed to calculate IV for strike {}: {}",
                        option.strike_price, e
                    );
                }
                option
            })
            .collect();

        self.options = modified_options;
    }

    #[cfg(not(target_arch = "wasm32"))]
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
            "Delta",
            "Gamma",
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
                default_empty_string(option.delta_call),
                default_empty_string(option.delta_put),
                default_empty_string(option.gamma),
                default_empty_string(option.volume),
                default_empty_string(option.open_interest),
            ])?;
        }

        wtr.flush()?;
        Ok(())
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn save_to_json(&self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let full_path = format!("{}/{}.json", file_path, self.get_title());
        let file = File::create(full_path)?;
        serde_json::to_writer_pretty(file, &self)?;
        Ok(())
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn load_from_csv(file_path: &str) -> Result<Self, Box<dyn Error>> {
        let mut rdr = csv::Reader::from_path(file_path)?;
        let mut options = BTreeSet::new();
        for result in rdr.records() {
            let record = result?;
            debug!("To CSV: {:?}", record);

            let mut option_data = OptionData {
                strike_price: record[0].parse()?,
                call_bid: parse(&record[1]),
                call_ask: parse(&record[2]),
                put_bid: parse(&record[3]),
                put_ask: parse(&record[4]),
                call_middle: None,
                put_middle: None,
                implied_volatility: parse(&record[5]),
                delta_call: parse(&record[6]),
                delta_put: parse(&record[7]),
                gamma: parse(&record[8]),
                volume: parse(&record[9]),
                open_interest: parse(&record[10]),
                options: None,
            };
            option_data.set_mid_prices();
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
        match option_chain.set_from_title(file_path) {
            Ok(_) => {
                // TODO: find other way to set symbol, underlying_price and expiration_date
            }
            Err(e) => {
                debug!("Failed to set title from file name: {}", e);
            }
        }
        Ok(option_chain)
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn load_from_json(file_path: &str) -> Result<Self, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let mut option_chain: OptionChain = serde_json::from_reader(file)?;
        option_chain.update_mid_prices();
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
                            option.implied_volatility.unwrap_or(Positive::ZERO),
                            params.option_qty,
                            self.underlying_price,
                            params.risk_free_rate,
                            OptionStyle::Put,
                            params.dividend_yield,
                            None,
                        ),
                        option.put_ask.unwrap_or(Positive::ZERO),
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
                            option.implied_volatility.unwrap_or(Positive::ZERO),
                            params.option_qty,
                            self.underlying_price,
                            params.risk_free_rate,
                            OptionStyle::Put,
                            params.dividend_yield,
                            None,
                        ),
                        option.put_bid.unwrap_or(Positive::ZERO),
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
                            option.implied_volatility.unwrap_or(Positive::ZERO),
                            params.option_qty,
                            self.underlying_price,
                            params.risk_free_rate,
                            OptionStyle::Call,
                            params.dividend_yield,
                            None,
                        ),
                        option.call_ask.unwrap_or(Positive::ZERO),
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
                            option.implied_volatility.unwrap_or(Positive::ZERO),
                            params.option_qty,
                            self.underlying_price,
                            params.risk_free_rate,
                            OptionStyle::Call,
                            params.dividend_yield,
                            None,
                        ),
                        option.call_bid.unwrap_or(Positive::ZERO),
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
        self.options
            .iter()
            .filter(|option| option.implied_volatility.is_some())
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
    /// use optionstratlib::{pos, Positive};
    /// let mut option_chain = OptionChain::new("TEST", pos!(100.0), "2024-01-01".to_string(), None, None);
    /// for (option1, option2) in option_chain.get_double_iter() {
    ///     info!("{:?}, {:?}", option1, option2);
    /// }
    /// ```
    pub fn get_double_iter(&self) -> impl Iterator<Item = (&OptionData, &OptionData)> {
        self.options
            .iter()
            .filter(|option| option.implied_volatility.is_some())
            .filter(|option| option.implied_volatility.unwrap() > Positive::ZERO)
            .enumerate()
            .flat_map(|(i, item1)| {
                self.options
                    .iter()
                    .filter(|option| option.implied_volatility.is_some())
                    .filter(|option| option.implied_volatility.unwrap() > Positive::ZERO)
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
    /// use optionstratlib::pos;
    /// let mut option_chain = OptionChain::new("TEST", pos!(100.0), "2024-01-01".to_string(), None, None);
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
    /// use optionstratlib::pos;
    /// let mut option_chain = OptionChain::new("TEST", pos!(100.0), "2024-01-01".to_string(), None, None);
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
    /// use optionstratlib::pos;
    /// let mut option_chain = OptionChain::new("TEST", pos!(100.0), "2024-01-01".to_string(), None, None);
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
    /// use optionstratlib::pos;
    /// let mut option_chain = OptionChain::new("TEST", pos!(100.0), "2024-01-01".to_string(), None, None);
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
    /// use optionstratlib::pos;
    /// let mut option_chain = OptionChain::new("TEST", pos!(100.0), "2024-01-01".to_string(), None, None);
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

    /// Retrieves the call option price for a specific strike price
    ///
    /// This helper method finds and returns the ask price of a call option
    /// at the specified strike price from the option chain.
    ///
    /// # Arguments
    /// * `strike` - The strike price to look up
    ///
    /// # Returns
    /// * `Some(Decimal)` - The call option ask price if found
    /// * `None` - If no option exists at the specified strike or if the price is not available
    ///
    /// # Notes
    /// * Uses the ask price as it represents the cost to buy the option
    /// * Converts the price to Decimal for consistency in calculations
    pub fn get_call_price(&self, strike: Positive) -> Option<Decimal> {
        self.options
            .iter()
            .find(|opt| opt.strike_price == strike)
            .and_then(|opt| opt.call_ask)
            .map(|price| price.to_dec())
    }

    /// Retrieves the implied volatility for the at-the-money (ATM) option
    ///
    /// Finds the option with strike price equal to the current underlying price
    /// and returns its implied volatility.
    ///
    /// # Returns
    /// * `Ok(Decimal)` - The ATM implied volatility if found
    /// * `Err(String)` - Error message if ATM implied volatility is not available
    ///
    /// # Examples
    /// ```
    /// use tracing::info;
    /// use optionstratlib::chains::chain::OptionChain;
    /// use optionstratlib::pos;
    /// let chain = OptionChain::new("TEST", pos!(100.0), "2024-12-31".to_string(), None, None);
    /// match chain.get_atm_implied_volatility() {
    ///     Ok(vol) => info!("ATM volatility: {}", vol),
    ///     Err(e) => info!("Error: {}", e),
    /// }
    /// ```
    ///
    /// # Notes
    /// * ATM strike is defined as the strike equal to the current underlying price
    /// * Important for volatility skew calculations and option pricing
    /// * Returns implied volatility as a decimal for precise calculations
    pub fn get_atm_implied_volatility(&self) -> Result<Decimal, String> {
        let atm_strike = self.underlying_price;

        self.options
            .iter()
            .find(|opt| opt.strike_price == atm_strike)
            .and_then(|opt| opt.implied_volatility)
            .map(|iv| iv.value())
            .ok_or_else(|| "No ATM implied volatility available".to_string())
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
            self.risk_free_rate.unwrap_or(Decimal::ZERO),
            self.dividend_yield.unwrap_or(Positive::ZERO),
            Some(self.symbol.clone()),
        ))
    }
}

impl RNDAnalysis for OptionChain {
    /// Implementation of RND calculation for option chains
    ///
    /// # Numerical Method
    /// 1. Calculates second derivative of option prices
    /// 2. Applies Breeden-Litzenberger formula
    /// 3. Normalizes resulting densities
    ///
    /// # Error Conditions
    /// * Empty option chain
    /// * Zero derivative tolerance
    /// * Failed density calculations
    fn calculate_rnd(&self, params: &RNDParameters) -> Result<RNDResult, Box<dyn Error>> {
        let mut densities = BTreeMap::new();
        let mut h = params.derivative_tolerance.to_dec();

        // Step 1: Validate parameters
        if h == Positive::ZERO {
            return Err(Box::from(
                "Derivative tolerance must be greater than zero".to_string(),
            ));
        }

        // Step 2: Get all available strikes
        let strikes: Vec<Positive> = self.options.iter().map(|opt| opt.strike_price).collect();
        if strikes.is_empty() {
            return Err(Box::from(
                "No strikes available for RND calculation".to_string(),
            ));
        }

        // Calculate minimum strike interval
        let min_interval = strikes
            .windows(2)
            .map(|w| w[1] - w[0])
            .min()
            .ok_or("Cannot determine strike interval")?;

        if h < min_interval.to_dec() {
            h = min_interval.to_dec();
        }

        // Step 3: Calculate time to expiry
        let expiry_date = NaiveDate::parse_from_str(&self.expiration_date, "%Y-%m-%d")?
            .and_hms_opt(23, 59, 59)
            .ok_or("Invalid expiry date time")?;

        let now = Utc::now().naive_utc();
        let time_to_expiry =
            Decimal::from_f64((expiry_date - now).num_days() as f64 / 365.0).unwrap();

        // Step 4: Calculate discount factor
        let discount = (-params.risk_free_rate * time_to_expiry).exp();

        // Debug information
        #[cfg(test)]
        {
            debug!("Time to expiry: {} years", time_to_expiry);
            debug!("Discount factor: {}", discount);
            debug!("Step size h: {}", h);
        }

        // Step 5: Calculate RND for each strike
        for opt in self.options.iter() {
            let k = opt.strike_price;

            // Debug prices
            #[cfg(test)]
            {
                debug!("Processing strike {}", k);
                debug!("Call price at k: {:?}", self.get_call_price(k));
                debug!("Call price at k+h: {:?}", self.get_call_price(k + h));
                debug!("Call price at k-h: {:?}", self.get_call_price(k - h));
            }

            if let (Some(call_price), Some(call_up), Some(call_down)) = (
                self.get_call_price(k),
                self.get_call_price(k + h),
                self.get_call_price(k - h),
            ) {
                // Calculate second derivative
                let second_derivative = (call_up + call_down - Decimal::TWO * call_price) / (h * h);

                #[cfg(test)]
                {
                    debug!("Second derivative: {}", second_derivative);
                }

                // Calculate density using Breeden-Litzenberger formula
                let density = second_derivative * discount;

                #[cfg(test)]
                {
                    debug!("Density: {}", density);
                }

                // Store valid density
                if !density.is_sign_negative() && !density.is_zero() {
                    densities.insert(k, density);
                }
            }
        }

        // Step 6: Validate and normalize densities
        if densities.is_empty() {
            return Err(Box::from("Failed to calculate valid densities".to_string()));
        }

        let total: Decimal = densities.values().sum();
        if !total.is_zero() {
            for density in densities.values_mut() {
                *density /= total;
            }
        }

        #[cfg(test)]
        {
            debug!("Total number of densities: {}", densities.len());
            debug!("Sum of densities: {}", total);
        }

        Ok(RNDResult::new(densities))
    }

    /// Implementation of volatility skew calculation
    ///
    /// Extracts and analyzes the relationship between strike prices
    /// and implied volatilities.
    ///
    /// # Error Conditions
    /// * Missing ATM volatility
    /// * Insufficient valid data points
    fn calculate_skew(&self) -> Result<Vec<(Positive, Decimal)>, Box<dyn Error>> {
        let mut skew = Vec::new();
        let atm_strike = self.underlying_price;
        let atm_vol = self.get_atm_implied_volatility()?;

        for opt in self.options.iter() {
            if let Some(iv) = opt.implied_volatility {
                let relative_strike = opt.strike_price / atm_strike;
                let vol_diff = iv.to_dec() - atm_vol;
                skew.push((relative_strike, vol_diff));
            }
        }

        if skew.is_empty() {
            return Err(Box::from("No valid data for skew calculation".to_string()));
        }

        Ok(skew)
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
            ---------------------------------------------------------------------"
        )?;
        writeln!(
            f,
            "{:<10} {:<10} {:<10} {:<10} {:<10} {:<10} {:<10} {:<13} {:<10} {:<10} {:<10} {:<10} {:<10}",
            "Strike",
            "Call Bid",
            "Call Ask",
            "Call Mid",
            "Put Bid",
            "Put Ask",
            "Put Mid",
            "Implied Vol.",
            "Delta",
            "Delta",
            "Gamma",
            "Volume",
            "Open Interest"
        )?;
        writeln!(
            f,
            "----------------------------------------------------------------------------------\
            ------------------------------------------------------------------"
        )?;

        for option in &self.options {
            writeln!(f, "{}", option,)?;
        }
        Ok(())
    }
}

impl VolatilitySmile for OptionChain {
    /// Computes the volatility smile for the option chain.
    ///
    /// This function calculates the volatility smile by interpolating the implied volatilities
    /// for all strike prices in the option chain.  It uses the available implied volatilities
    /// from the `options` field and performs linear interpolation to estimate missing values.
    ///
    /// # Returns
    ///
    /// A `Curve` object representing the volatility smile. The x-coordinates of the curve
    /// correspond to the strike prices, and the y-coordinates represent the corresponding
    /// implied volatilities.
    fn smile(&self) -> Curve {
        // Build a BTreeSet with the known points (options with implied volatility)
        let mut bt_points: BTreeSet<Point2D> = self
            .options
            .iter()
            .filter_map(|option| {
                // Only include options with a valid implied volatility
                option
                    .implied_volatility
                    .map(|vol| Point2D::new(option.strike_price.to_dec(), vol.to_dec()))
            })
            .collect();

        // Create an initial Curve object using the known points
        let curve = Curve::new(bt_points.clone());

        // Interpolate missing points (options without implied volatility)
        for option in self
            .options
            .iter()
            .filter(|o| o.implied_volatility.is_none())
        {
            // Use linear interpolation to estimate the missing implied volatility
            if let Ok(interpolated_point) = curve.linear_interpolate(option.strike_price.to_dec()) {
                bt_points.insert(interpolated_point);
            }
        }

        // Return the final Curve with all points, including interpolated ones
        Curve::new(bt_points)
    }
}

#[cfg(test)]
mod tests_chain_base {
    use super::*;
    use crate::model::types::ExpirationDate;
    use crate::utils::logger::setup_logger;
    use crate::{pos, spos};
    use rust_decimal_macros::dec;
    #[cfg(not(target_arch = "wasm32"))]
    use std::fs;
    use tracing::info;

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_new_option_chain() {
        let chain = OptionChain::new(
            "SP500",
            pos!(5781.88),
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
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
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
                ExpirationDate::Days(pos!(30.0)),
                spos!(0.17),
                Decimal::ZERO,
                pos!(0.05),
                Some("SP500".to_string()),
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
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
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
                ExpirationDate::Days(pos!(60.0)),
                spos!(0.03),
                Decimal::ZERO,
                pos!(0.05),
                Some("SP500".to_string()),
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
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_add_option() {
        let mut chain = OptionChain::new(
            "SP500",
            pos!(5781.88),
            "18-oct-2024".to_string(),
            None,
            None,
        );
        chain.add_option(
            pos!(5520.0),
            spos!(274.26),
            spos!(276.06),
            spos!(13.22),
            spos!(14.90),
            spos!(16.31),
            Some(dec!(0.5)),
            Some(dec!(0.5)),
            Some(dec!(0.5)),
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
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_get_title_i() {
        let chain = OptionChain::new(
            "SP500",
            pos!(5781.88),
            "18-oct-2024".to_string(),
            None,
            None,
        );
        assert_eq!(chain.get_title(), "SP500-18-oct-2024-5781.88");
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_get_title_ii() {
        let chain = OptionChain::new(
            "SP500",
            pos!(5781.88),
            "18 oct 2024".to_string(),
            None,
            None,
        );
        assert_eq!(chain.get_title(), "SP500-18-oct-2024-5781.88");
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_set_from_title_i() {
        let mut chain = OptionChain::new("", Positive::ZERO, "".to_string(), None, None);
        let _ = chain.set_from_title("SP500-18-oct-2024-5781.88.csv");
        assert_eq!(chain.symbol, "SP500");
        assert_eq!(chain.expiration_date, "18-oct-2024");
        assert_eq!(chain.underlying_price, 5781.88);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_set_from_title_ii() {
        let mut chain = OptionChain::new("", Positive::ZERO, "".to_string(), None, None);
        let _ = chain.set_from_title("path/SP500-18-oct-2024-5781.88.csv");
        assert_eq!(chain.symbol, "SP500");
        assert_eq!(chain.expiration_date, "18-oct-2024");
        assert_eq!(chain.underlying_price, 5781.88);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_set_from_title_iii() {
        let mut chain = OptionChain::new("", Positive::ZERO, "".to_string(), None, None);
        let _ = chain.set_from_title("path/SP500-18-oct-2024-5781.csv");
        assert_eq!(chain.symbol, "SP500");
        assert_eq!(chain.expiration_date, "18-oct-2024");
        assert_eq!(chain.underlying_price, 5781.0);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_set_from_title_iv() {
        let mut chain = OptionChain::new("", Positive::ZERO, "".to_string(), None, None);
        let _ = chain.set_from_title("path/SP500-18-oct-2024-5781.88.json");
        assert_eq!(chain.symbol, "SP500");
        assert_eq!(chain.expiration_date, "18-oct-2024");
        assert_eq!(chain.underlying_price, 5781.88);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_set_from_title_v() {
        let mut chain = OptionChain::new("", Positive::ZERO, "".to_string(), None, None);
        let _ = chain.set_from_title("path/SP500-18-oct-2024-5781.json");
        assert_eq!(chain.symbol, "SP500");
        assert_eq!(chain.expiration_date, "18-oct-2024");
        assert_eq!(chain.underlying_price, 5781.0);
    }

    #[test]
    #[cfg(not(target_arch = "wasm32"))]
    fn test_save_to_csv() {
        let mut chain = OptionChain::new(
            "SP500",
            pos!(5781.88),
            "18-oct-2024".to_string(),
            None,
            None,
        );
        chain.add_option(
            pos!(5520.0),
            spos!(274.26),
            spos!(276.06),
            spos!(13.22),
            spos!(14.90),
            spos!(16.31),
            Some(dec!(0.5)),
            Some(dec!(0.5)),
            Some(dec!(0.5)),
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
    #[cfg(not(target_arch = "wasm32"))]
    fn test_save_to_json() {
        let mut chain = OptionChain::new(
            "SP500",
            pos!(5781.88),
            "18-oct-2024".to_string(),
            None,
            None,
        );
        chain.add_option(
            pos!(5520.0),
            spos!(274.26),
            spos!(276.06),
            spos!(13.22),
            spos!(14.90),
            spos!(16.31),
            Some(dec!(0.5)),
            Some(dec!(0.5)),
            Some(dec!(0.5)),
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
    #[cfg(not(target_arch = "wasm32"))]
    fn test_load_from_csv() {
        setup_logger();
        let mut chain = OptionChain::new(
            "SP500",
            pos!(5781.89),
            "18-oct-2024".to_string(),
            None,
            None,
        );
        chain.add_option(
            pos!(5520.0),
            spos!(274.26),
            spos!(276.06),
            spos!(13.22),
            spos!(14.90),
            spos!(16.31),
            Some(dec!(0.5)),
            Some(dec!(0.5)),
            Some(dec!(0.5)),
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
    #[cfg(not(target_arch = "wasm32"))]
    fn test_load_from_json() {
        let mut chain =
            OptionChain::new("SP500", pos!(5781.9), "18-oct-2024".to_string(), None, None);
        chain.add_option(
            pos!(5520.0),
            spos!(274.26),
            spos!(276.06),
            spos!(13.22),
            spos!(14.90),
            spos!(16.31),
            Some(dec!(0.5)),
            Some(dec!(0.5)),
            Some(dec!(0.5)),
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
    use crate::model::types::ExpirationDate;
    use crate::spos;
    use crate::utils::logger::setup_logger;
    use crate::{assert_pos_relative_eq, pos};
    use num_traits::ToPrimitive;
    use rust_decimal_macros::dec;
    use tracing::info;

    fn create_valid_option_data() -> OptionData {
        OptionData::new(
            pos!(100.0),      // strike_price
            spos!(9.5),       // call_bid
            spos!(10.0),      // call_ask
            spos!(8.5),       // put_bid
            spos!(9.0),       // put_ask
            spos!(0.2),       // implied_volatility
            Some(dec!(-0.3)), // delta
            Some(dec!(0.7)),
            Some(dec!(0.5)),
            spos!(1000.0), // volume
            Some(500),     // open_interest
        )
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_new_option_data() {
        let option_data = create_valid_option_data();
        assert_eq!(option_data.strike_price, pos!(100.0));
        assert_eq!(option_data.call_bid, spos!(9.5));
        assert_eq!(option_data.call_ask, spos!(10.0));
        assert_eq!(option_data.put_bid, spos!(8.5));
        assert_eq!(option_data.put_ask, spos!(9.0));
        assert_eq!(option_data.implied_volatility, spos!(0.2));
        assert_eq!(option_data.delta_call.unwrap().to_f64(), Some(-0.3));
        assert_eq!(option_data.volume, spos!(1000.0));
        assert_eq!(option_data.open_interest, Some(500));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_validate_valid_option() {
        setup_logger();
        let option_data = create_valid_option_data();
        assert!(option_data.validate());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_validate_zero_strike() {
        let mut option_data = create_valid_option_data();
        option_data.strike_price = Positive::ZERO;
        assert!(!option_data.validate());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_validate_no_implied_volatility() {
        let mut option_data = create_valid_option_data();
        option_data.implied_volatility = None;
        assert!(!option_data.validate());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
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
            None,
            None,
        );
        assert!(!option_data.validate());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_valid_call() {
        let option_data = create_valid_option_data();
        assert!(option_data.valid_call());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_valid_call_missing_bid() {
        let mut option_data = create_valid_option_data();
        option_data.call_bid = None;
        assert!(!option_data.valid_call());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_valid_call_missing_ask() {
        let mut option_data = create_valid_option_data();
        option_data.call_ask = None;
        assert!(!option_data.valid_call());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_valid_put() {
        let option_data = create_valid_option_data();
        assert!(option_data.valid_put());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_valid_put_missing_bid() {
        let mut option_data = create_valid_option_data();
        option_data.put_bid = None;
        assert!(!option_data.valid_put());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_valid_put_missing_ask() {
        let mut option_data = create_valid_option_data();
        option_data.put_ask = None;
        assert!(!option_data.valid_put());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
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
            None,
            None,
        );
        let price_params = OptionDataPriceParams::new(
            pos!(100.0),
            ExpirationDate::Days(pos!(30.0)),
            spos!(0.2),
            Decimal::ZERO,
            Positive::ZERO,
            None,
        );

        let result = option_data.calculate_prices(&price_params);

        assert!(result.is_ok());
        assert!(option_data.call_ask.is_some());
        assert!(option_data.call_bid.is_some());
        assert!(option_data.put_ask.is_some());
        assert!(option_data.put_bid.is_some());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_calculate_prices_missing_volatility() {
        setup_logger();
        let mut option_data = OptionData::new(
            pos!(100.0),
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
        );

        let price_params = OptionDataPriceParams::new(
            pos!(100.0),
            ExpirationDate::Days(pos!(30.0)),
            None,
            Decimal::ZERO,
            Positive::ZERO,
            None,
        );
        let _ = option_data.calculate_prices(&price_params);

        info!("{}", option_data);
        assert_eq!(option_data.call_ask, None);
        assert_eq!(option_data.call_bid, None);
        assert_eq!(option_data.put_ask, None);
        assert_eq!(option_data.put_bid, None);
        assert_eq!(option_data.implied_volatility, None);
        assert_eq!(option_data.delta_call, None);
        assert_eq!(option_data.strike_price, pos!(100.0));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
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
            None,
            None,
        );

        let price_params = OptionDataPriceParams::new(
            pos!(110.0),
            ExpirationDate::Days(pos!(30.0)),
            spos!(0.12),
            dec!(0.05),
            pos!(0.01),
            None,
        );
        let result = option_data.calculate_prices(&price_params);

        assert!(result.is_ok());
        info!("{}", option_data);
        assert_pos_relative_eq!(option_data.call_ask.unwrap(), pos!(10.51108), pos!(0.0001));
        assert_pos_relative_eq!(option_data.call_bid.unwrap(), pos!(10.51108), pos!(0.0001));
        assert_pos_relative_eq!(option_data.put_ask.unwrap(), pos!(0.100966), pos!(0.0001));
        assert_pos_relative_eq!(option_data.put_bid.unwrap(), pos!(0.100966), pos!(0.0001));
        option_data.apply_spread(pos!(0.02), 2);
        info!("{}", option_data);
        assert_eq!(option_data.call_ask, spos!(10.52));
        assert_eq!(option_data.call_bid, spos!(10.50));
        assert_eq!(option_data.put_ask, spos!(0.11));
        assert_eq!(option_data.put_bid, spos!(0.09));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
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
            None,
            None,
        );
        let price_params = OptionDataPriceParams::new(
            pos!(100.0),
            ExpirationDate::Days(pos!(30.0)),
            spos!(0.2),
            dec!(0.05),
            pos!(0.01),
            None,
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
    use crate::utils::logger::setup_logger;
    use crate::{pos, spos};
    use rust_decimal_macros::dec;

    fn create_test_chain() -> OptionChain {
        // Create a sample option chain
        let mut chain = OptionChain::new("TEST", pos!(100.0), "2024-01-01".to_string(), None, None);

        // Add some test options with different strikes
        chain.add_option(
            pos!(95.0),      // strike_price
            spos!(4.0),      // call_bid
            spos!(4.2),      // call_ask
            spos!(3.0),      // put_bid
            spos!(3.2),      // put_ask
            spos!(0.2),      // implied_volatility
            Some(dec!(0.5)), // delta
            Some(dec!(0.5)),
            Some(dec!(0.5)),
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
            Some(dec!(0.5)),
            Some(dec!(0.5)),
            Some(dec!(0.5)),
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
            Some(dec!(0.5)),
            Some(dec!(0.5)),
            Some(dec!(0.5)),
            spos!(100.0),
            Some(50),
        );

        chain
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_zero_quantity() {
        setup_logger();
        let chain = create_test_chain();
        let params = RandomPositionsParams::new(
            None,
            None,
            None,
            None,
            ExpirationDate::Days(pos!(30.0)),
            pos!(1.0),
            dec!(0.05),
            pos!(0.02),
            Positive::ONE,
            Positive::ONE,
            Positive::ONE,
            Positive::ONE,
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
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_long_puts_only() {
        setup_logger();
        let chain = create_test_chain();
        let params = RandomPositionsParams::new(
            Some(2),
            None,
            None,
            None,
            ExpirationDate::Days(pos!(30.0)),
            pos!(1.0),
            dec!(0.05),
            pos!(0.02),
            Positive::ONE,
            Positive::ONE,
            Positive::ONE,
            Positive::ONE,
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
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_short_puts_only() {
        setup_logger();
        let chain = create_test_chain();
        let params = RandomPositionsParams::new(
            None,
            Some(2),
            None,
            None,
            ExpirationDate::Days(pos!(30.0)),
            pos!(1.0),
            dec!(0.05),
            pos!(0.02),
            Positive::ONE,
            Positive::ONE,
            Positive::ONE,
            Positive::ONE,
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
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_long_calls_only() {
        setup_logger();
        let chain = create_test_chain();
        let params = RandomPositionsParams::new(
            None,
            None,
            Some(2),
            None,
            ExpirationDate::Days(pos!(30.0)),
            pos!(1.0),
            dec!(0.05),
            pos!(0.02),
            Positive::ONE,
            Positive::ONE,
            Positive::ONE,
            Positive::ONE,
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
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_short_calls_only() {
        setup_logger();
        let chain = create_test_chain();
        let params = RandomPositionsParams::new(
            None,
            None,
            None,
            Some(2),
            ExpirationDate::Days(pos!(30.0)),
            pos!(1.0),
            dec!(0.05),
            pos!(0.02),
            Positive::ONE,
            Positive::ONE,
            Positive::ONE,
            Positive::ONE,
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
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_mixed_positions() {
        setup_logger();
        let chain = create_test_chain();
        let params = RandomPositionsParams::new(
            Some(1),
            Some(1),
            Some(1),
            Some(1),
            ExpirationDate::Days(pos!(30.0)),
            pos!(1.0),
            dec!(0.05),
            pos!(0.02),
            Positive::ONE,
            Positive::ONE,
            Positive::ONE,
            Positive::ONE,
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
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_empty_chain() {
        setup_logger();
        let chain = OptionChain::new("TEST", pos!(100.0), "2024-01-01".to_string(), None, None);
        let params = RandomPositionsParams::new(
            Some(1),
            None,
            None,
            None,
            ExpirationDate::Days(pos!(30.0)),
            pos!(1.0),
            dec!(0.05),
            pos!(0.02),
            Positive::ONE,
            Positive::ONE,
            Positive::ONE,
            Positive::ONE,
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
    use rust_decimal_macros::dec;

    fn create_test_option_data() -> OptionData {
        OptionData::new(
            pos!(100.0),
            spos!(9.5),
            spos!(10.0),
            spos!(8.5),
            spos!(9.0),
            spos!(0.2),
            Some(dec!(-0.3)),
            Some(dec!(0.7)),
            Some(dec!(0.5)),
            spos!(1000.0),
            Some(500),
        )
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_get_call_buy_price() {
        let data = create_test_option_data();
        assert_eq!(data.get_call_buy_price(), spos!(10.0));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_get_call_sell_price() {
        let data = create_test_option_data();
        assert_eq!(data.get_call_sell_price(), spos!(9.5));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_get_put_buy_price() {
        let data = create_test_option_data();
        assert_eq!(data.get_put_buy_price(), spos!(9.0));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_get_put_sell_price() {
        let data = create_test_option_data();
        assert_eq!(data.get_put_sell_price(), spos!(8.5));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
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
    use rust_decimal_macros::dec;

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_display_full_data() {
        let data = OptionData::new(
            pos!(100.0),
            spos!(9.5),
            spos!(10.0),
            spos!(8.5),
            spos!(9.0),
            spos!(0.2),
            Some(dec!(-0.3)),
            Some(dec!(0.7)),
            Some(dec!(0.5)),
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
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
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
    use crate::{pos, spos};

    fn create_test_chain() -> OptionChain {
        let mut chain = OptionChain::new("TEST", pos!(100.0), "2024-01-01".to_string(), None, None);

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
                None,
                None,
            );
        }
        chain
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_filter_upper() {
        let chain = create_test_chain();
        let filtered = chain.filter_option_data(FindOptimalSide::Upper);
        assert_eq!(filtered.len(), 2);
        assert!(filtered
            .iter()
            .all(|opt| opt.strike_price > chain.underlying_price));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_filter_lower() {
        let chain = create_test_chain();
        let filtered = chain.filter_option_data(FindOptimalSide::Lower);
        assert_eq!(filtered.len(), 2);
        assert!(filtered
            .iter()
            .all(|opt| opt.strike_price < chain.underlying_price));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_filter_all() {
        let chain = create_test_chain();
        let filtered = chain.filter_option_data(FindOptimalSide::All);
        assert_eq!(filtered.len(), 5);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
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
    use crate::{pos, spos};

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_empty_chain() {
        let chain = OptionChain::new("TEST", pos!(100.0), "2024-01-01".to_string(), None, None);
        assert_eq!(chain.strike_price_range_vec(5.0), None);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_single_option() {
        let mut chain = OptionChain::new("TEST", pos!(100.0), "2024-01-01".to_string(), None, None);
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
            None,
            None,
        );
        let range = chain.strike_price_range_vec(5.0).unwrap();
        assert_eq!(range.len(), 1);
        assert_eq!(range[0], 100.0);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_multiple_options() {
        let mut chain = OptionChain::new("TEST", pos!(100.0), "2024-01-01".to_string(), None, None);
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
                None,
                None,
            );
        }
        let range = chain.strike_price_range_vec(5.0).unwrap();
        assert_eq!(range, vec![90.0, 95.0, 100.0]);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_step_size() {
        let mut chain = OptionChain::new("TEST", pos!(100.0), "2024-01-01".to_string(), None, None);
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
    use crate::{pos, spos};
    use num_traits::ToPrimitive;
    use rust_decimal_macros::dec;

    fn create_test_option_data() -> OptionData {
        OptionData::new(
            pos!(100.0),      // strike_price
            spos!(9.5),       // call_bid
            spos!(10.0),      // call_ask
            spos!(8.5),       // put_bid
            spos!(9.0),       // put_ask
            spos!(0.2),       // implied_volatility
            Some(dec!(-0.3)), // delta
            Some(dec!(0.7)),  // delta
            Some(dec!(0.3)),  // gamma
            spos!(1000.0),    // volume
            Some(500),        // open_interest
        )
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_get_option_success() {
        let option_data = create_test_option_data();
        let price_params = OptionDataPriceParams::new(
            pos!(100.0),
            ExpirationDate::Days(pos!(30.0)),
            spos!(0.25),
            dec!(0.05),
            pos!(0.02),
            None,
        );

        let result = option_data.get_option(&price_params, Side::Long, OptionStyle::Call);
        assert!(result.is_ok());

        let option = result.unwrap();
        assert_eq!(option.strike_price, pos!(100.0));
        assert_eq!(option.implied_volatility, 0.25); // Uses provided IV
        assert_eq!(option.underlying_price, pos!(100.0));
        assert_eq!(option.risk_free_rate.to_f64().unwrap(), 0.05);
        assert_eq!(option.dividend_yield.to_f64(), 0.02);
        assert_eq!(option.side, Side::Long);
        assert_eq!(option.option_style, OptionStyle::Call);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_get_option_using_data_iv() {
        let option_data = create_test_option_data();
        let price_params = OptionDataPriceParams::new(
            pos!(100.0),
            ExpirationDate::Days(pos!(30.0)),
            None, // No IV provided in params
            dec!(0.05),
            pos!(0.02),
            None,
        );

        let result = option_data.get_option(&price_params, Side::Long, OptionStyle::Call);
        assert!(result.is_ok());

        let option = result.unwrap();
        assert_eq!(option.implied_volatility, 0.002); // Uses IV from option_data
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_get_option_missing_iv() {
        let mut option_data = create_test_option_data();
        option_data.implied_volatility = None;

        let price_params = OptionDataPriceParams::new(
            pos!(100.0),
            ExpirationDate::Days(pos!(30.0)),
            None,
            dec!(0.05),
            pos!(0.02),
            None,
        );

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
    use crate::error::chains::OptionDataErrorKind;
    use crate::greeks::Greeks;
    use crate::model::types::ExpirationDate;
    use crate::{assert_decimal_eq, pos, spos};
    use num_traits::ToPrimitive;
    use rust_decimal_macros::dec;

    fn create_test_option_data() -> OptionData {
        OptionData::new(
            pos!(100.0),      // strike_price
            spos!(9.5),       // call_bid
            spos!(10.0),      // call_ask
            spos!(8.5),       // put_bid
            spos!(9.0),       // put_ask
            spos!(0.2),       // implied_volatility
            Some(dec!(-0.3)), // delta
            Some(dec!(-0.3)),
            Some(dec!(0.3)),
            spos!(1000.0), // volume
            Some(500),     // open_interest
        )
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_get_options_in_strike_success() {
        let option_data = create_test_option_data();
        let price_params = OptionDataPriceParams::new(
            pos!(100.0),
            ExpirationDate::Days(pos!(30.0)),
            spos!(0.25),
            dec!(0.05),
            pos!(0.02),
            None,
        );

        let result =
            option_data.get_options_in_strike(&price_params, Side::Long, OptionStyle::Call);
        assert!(result.is_ok());

        let options = result.unwrap();

        // Check long call
        assert_eq!(options.long_call.strike_price, pos!(100.0));
        assert_eq!(options.long_call.option_style, OptionStyle::Call);
        assert_eq!(options.long_call.side, Side::Long);

        // Check short call
        assert_eq!(options.short_call.strike_price, pos!(100.0));
        assert_eq!(options.short_call.option_style, OptionStyle::Call);
        assert_eq!(options.short_call.side, Side::Short);

        // Check long put
        assert_eq!(options.long_put.strike_price, pos!(100.0));
        assert_eq!(options.long_put.option_style, OptionStyle::Put);
        assert_eq!(options.long_put.side, Side::Long);

        // Check short put
        assert_eq!(options.short_put.strike_price, pos!(100.0));
        assert_eq!(options.short_put.option_style, OptionStyle::Put);
        assert_eq!(options.short_put.side, Side::Short);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_get_options_in_strike_using_data_iv() {
        let option_data = create_test_option_data();
        let price_params = OptionDataPriceParams::new(
            pos!(100.0),
            ExpirationDate::Days(pos!(30.0)),
            None,
            dec!(0.05),
            pos!(0.02),
            None,
        );

        let result =
            option_data.get_options_in_strike(&price_params, Side::Long, OptionStyle::Call);
        assert!(result.is_ok());

        let options = result.unwrap();
        assert_eq!(options.long_call.implied_volatility, 0.002);
        assert_eq!(options.short_call.implied_volatility, 0.002);
        assert_eq!(options.long_put.implied_volatility, 0.002);
        assert_eq!(options.short_put.implied_volatility, 0.002);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_get_options_in_strike_missing_iv() {
        let mut option_data = create_test_option_data();
        option_data.implied_volatility = None;

        let price_params = OptionDataPriceParams::new(
            pos!(100.0),
            ExpirationDate::Days(pos!(30.0)),
            None,
            dec!(0.05),
            pos!(0.02),
            None,
        );

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
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_get_options_in_strike_all_properties() {
        let option_data = create_test_option_data();
        let price_params = OptionDataPriceParams::new(
            pos!(110.0),
            ExpirationDate::Days(pos!(45.0)),
            spos!(0.3),
            dec!(0.06),
            pos!(0.03),
            None,
        );

        let result =
            option_data.get_options_in_strike(&price_params, Side::Long, OptionStyle::Call);
        assert!(result.is_ok());

        let options = result.unwrap();

        // Verify common properties across all options
        let check_common_properties = |option: &Options| {
            assert_eq!(option.strike_price, pos!(100.0));
            assert_eq!(option.underlying_price, pos!(110.0));
            assert_eq!(option.implied_volatility, 0.3);
            assert_eq!(option.risk_free_rate.to_f64().unwrap(), 0.06);
            assert_eq!(option.dividend_yield.to_f64(), 0.03);
            assert_eq!(option.option_type, OptionType::European);
            assert_eq!(option.quantity, pos!(1.0));
        };

        check_common_properties(&options.long_call);
        check_common_properties(&options.short_call);
        check_common_properties(&options.long_put);
        check_common_properties(&options.short_put);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_get_options_in_strike_deltas() {
        let option_data = create_test_option_data();
        let price_params = OptionDataPriceParams::new(
            pos!(110.0),
            ExpirationDate::Days(pos!(45.0)),
            spos!(0.3),
            dec!(0.06),
            pos!(0.03),
            None,
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
    use crate::{pos, spos};
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;

    fn create_test_chain() -> OptionChain {
        let mut chain = OptionChain::new("TEST", pos!(100.0), "2024-01-01".to_string(), None, None);

        for strike in [90.0, 95.0, 100.0, 105.0, 110.0].iter() {
            chain.add_option(
                pos!(*strike),
                spos!(1.0),       // call_bid
                spos!(1.2),       // call_ask
                spos!(1.0),       // put_bid
                spos!(1.2),       // put_ask
                spos!(0.2),       // implied_volatility
                Some(dec!(-0.3)), // delta
                Some(dec!(-0.3)),
                Some(dec!(0.3)),
                spos!(1000.0), // volume
                Some(500),     // open_interest
            );
        }
        chain
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_filter_upper_strikes() {
        let chain = create_test_chain();
        let price_params = OptionDataPriceParams::new(
            pos!(100.0),
            ExpirationDate::Days(pos!(30.0)),
            spos!(0.2),
            dec!(0.05),
            pos!(0.02),
            None,
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
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_filter_lower_strikes() {
        let chain = create_test_chain();
        let price_params = OptionDataPriceParams::new(
            pos!(100.0),
            ExpirationDate::Days(pos!(30.0)),
            spos!(0.2),
            dec!(0.05),
            pos!(0.02),
            None,
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
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_filter_all_strikes() {
        let chain = create_test_chain();
        let price_params = OptionDataPriceParams::new(
            pos!(100.0),
            ExpirationDate::Days(pos!(30.0)),
            spos!(0.2),
            dec!(0.05),
            pos!(0.02),
            None,
        );

        let result = chain.filter_options_in_strike(&price_params, FindOptimalSide::All);
        assert!(result.is_ok());

        let filtered_options = result.unwrap();
        assert_eq!(filtered_options.len(), 5);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_filter_range_strikes() {
        let chain = create_test_chain();
        let price_params = OptionDataPriceParams::new(
            pos!(100.0),
            ExpirationDate::Days(pos!(30.0)),
            spos!(0.2),
            dec!(0.05),
            pos!(0.02),
            None,
        );

        let result = chain.filter_options_in_strike(
            &price_params,
            FindOptimalSide::Range(pos!(95.0), pos!(105.0)),
        );
        assert!(result.is_ok());

        let filtered_options = result.unwrap();
        assert_eq!(filtered_options.len(), 3);

        for opt in filtered_options {
            assert!(opt.long_call.strike_price >= pos!(95.0));
            assert!(opt.long_call.strike_price <= pos!(105.0));
        }
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_filter_empty_chain() {
        let chain = OptionChain::new("TEST", pos!(100.0), "2024-01-01".to_string(), None, None);
        let price_params = OptionDataPriceParams::new(
            pos!(100.0),
            ExpirationDate::Days(pos!(30.0)),
            spos!(0.2),
            dec!(0.05),
            pos!(0.02),
            None,
        );

        let result = chain.filter_options_in_strike(&price_params, FindOptimalSide::All);
        assert!(result.is_ok());

        let filtered_options = result.unwrap();
        assert!(filtered_options.is_empty());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_filter_invalid_range() {
        let chain = create_test_chain();
        let price_params = OptionDataPriceParams::new(
            pos!(100.0),
            ExpirationDate::Days(pos!(30.0)),
            spos!(0.2),
            dec!(0.05),
            pos!(0.02),
            None,
        );

        let result = chain.filter_options_in_strike(
            &price_params,
            FindOptimalSide::Range(pos!(200.0), pos!(300.0)),
        );
        assert!(result.is_ok());

        let filtered_options = result.unwrap();
        assert!(filtered_options.is_empty());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_filter_all_strikes_deltas() {
        let chain = create_test_chain();
        let price_params = OptionDataPriceParams::new(
            pos!(100.0),
            ExpirationDate::Days(pos!(30.0)),
            spos!(0.2),
            dec!(0.05),
            pos!(0.02),
            None,
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
    use crate::spos;
    use rust_decimal_macros::dec;

    fn create_test_chain() -> OptionChain {
        let mut chain = OptionChain::new("TEST", pos!(100.0), "2024-01-01".to_string(), None, None);

        // Add three options with different strikes
        chain.add_option(
            pos!(90.0),      // strike_price
            spos!(5.0),      // call_bid
            spos!(5.5),      // call_ask
            spos!(1.0),      // put_bid
            spos!(1.5),      // put_ask
            spos!(0.2),      // implied_volatility
            Some(dec!(0.6)), // delta
            Some(dec!(0.5)),
            Some(dec!(0.5)),
            spos!(100.0), // volume
            Some(50),     // open_interest
        );

        chain.add_option(
            pos!(100.0),
            spos!(3.0),
            spos!(3.5),
            spos!(3.0),
            spos!(3.5),
            spos!(0.25),
            Some(dec!(0.5)),
            Some(dec!(0.5)),
            Some(dec!(0.5)),
            spos!(150.0),
            Some(75),
        );

        chain.add_option(
            pos!(110.0),
            spos!(1.0),
            spos!(1.5),
            spos!(5.0),
            spos!(5.5),
            spos!(0.3),
            Some(dec!(0.4)),
            Some(dec!(0.5)),
            Some(dec!(0.5)),
            spos!(80.0),
            Some(40),
        );

        chain
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_get_double_iter_empty() {
        let chain = OptionChain::new("TEST", pos!(100.0), "2024-01-01".to_string(), None, None);
        let pairs: Vec<_> = chain.get_double_iter().collect();
        assert!(pairs.is_empty());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_get_double_iter_single() {
        let mut chain = OptionChain::new("TEST", pos!(100.0), "2024-01-01".to_string(), None, None);
        chain.add_option(
            pos!(100.0),
            spos!(3.0),
            spos!(3.5),
            spos!(3.0),
            spos!(3.5),
            spos!(0.25),
            Some(dec!(0.5)),
            Some(dec!(0.5)),
            Some(dec!(0.5)),
            spos!(150.0),
            Some(75),
        );

        let pairs: Vec<_> = chain.get_double_iter().collect();
        assert!(pairs.is_empty()); // No pairs with single element
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_get_double_iter_multiple() {
        let chain = create_test_chain();
        let pairs: Vec<_> = chain.get_double_iter().collect();

        // Should have 3 pairs: (90,100), (90,110), (100,110)
        assert_eq!(pairs.len(), 3);

        // Check strikes of pairs
        assert_eq!(pairs[0].0.strike_price, pos!(90.0));
        assert_eq!(pairs[0].1.strike_price, pos!(100.0));

        assert_eq!(pairs[1].0.strike_price, pos!(90.0));
        assert_eq!(pairs[1].1.strike_price, pos!(110.0));

        assert_eq!(pairs[2].0.strike_price, pos!(100.0));
        assert_eq!(pairs[2].1.strike_price, pos!(110.0));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_get_double_inclusive_iter_empty() {
        let chain = OptionChain::new("TEST", pos!(100.0), "2024-01-01".to_string(), None, None);
        let pairs: Vec<_> = chain.get_double_inclusive_iter().collect();
        assert!(pairs.is_empty());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_get_double_inclusive_iter_single() {
        let mut chain = OptionChain::new("TEST", pos!(100.0), "2024-01-01".to_string(), None, None);
        chain.add_option(
            pos!(100.0),
            spos!(3.0),
            spos!(3.5),
            spos!(3.0),
            spos!(3.5),
            spos!(0.25),
            Some(dec!(0.5)),
            Some(dec!(0.5)),
            Some(dec!(0.5)),
            spos!(150.0),
            Some(75),
        );

        let pairs: Vec<_> = chain.get_double_inclusive_iter().collect();
        assert_eq!(pairs.len(), 1); // Should have one pair (self-pair)
        assert_eq!(pairs[0].0.strike_price, pairs[0].1.strike_price);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_get_double_inclusive_iter_multiple() {
        let chain = create_test_chain();
        let pairs: Vec<_> = chain.get_double_inclusive_iter().collect();

        // Should have 6 pairs: (90,90), (90,100), (90,110), (100,100), (100,110), (110,110)
        assert_eq!(pairs.len(), 6);

        // Check strikes of pairs
        assert_eq!(pairs[0].0.strike_price, pos!(90.0));
        assert_eq!(pairs[0].1.strike_price, pos!(90.0));

        assert_eq!(pairs[1].0.strike_price, pos!(90.0));
        assert_eq!(pairs[1].1.strike_price, pos!(100.0));

        assert_eq!(pairs[2].0.strike_price, pos!(90.0));
        assert_eq!(pairs[2].1.strike_price, pos!(110.0));

        assert_eq!(pairs[3].0.strike_price, pos!(100.0));
        assert_eq!(pairs[3].1.strike_price, pos!(100.0));

        assert_eq!(pairs[4].0.strike_price, pos!(100.0));
        assert_eq!(pairs[4].1.strike_price, pos!(110.0));

        assert_eq!(pairs[5].0.strike_price, pos!(110.0));
        assert_eq!(pairs[5].1.strike_price, pos!(110.0));
    }
}

#[cfg(test)]
mod tests_chain_iterators_bis {
    use super::*;
    use crate::spos;
    use rust_decimal_macros::dec;

    fn create_test_chain() -> OptionChain {
        let mut chain = OptionChain::new("TEST", pos!(100.0), "2024-01-01".to_string(), None, None);

        // Add four options with different strikes
        chain.add_option(
            pos!(90.0),      // strike_price
            spos!(5.0),      // call_bid
            spos!(5.5),      // call_ask
            spos!(1.0),      // put_bid
            spos!(1.5),      // put_ask
            spos!(0.2),      // implied_volatility
            Some(dec!(0.6)), // delta
            Some(dec!(0.5)),
            Some(dec!(0.5)),
            spos!(100.0), // volume
            Some(50),     // open_interest
        );

        chain.add_option(
            pos!(100.0),
            spos!(3.0),
            spos!(3.5),
            spos!(3.0),
            spos!(3.5),
            spos!(0.25),
            Some(dec!(0.5)),
            Some(dec!(0.5)),
            Some(dec!(0.5)),
            spos!(150.0),
            Some(75),
        );

        chain.add_option(
            pos!(110.0),
            spos!(1.0),
            spos!(1.5),
            spos!(5.0),
            spos!(5.5),
            spos!(0.3),
            Some(dec!(0.4)),
            Some(dec!(0.5)),
            Some(dec!(0.5)),
            spos!(80.0),
            Some(40),
        );

        chain.add_option(
            pos!(120.0),
            spos!(0.5),
            spos!(1.0),
            spos!(7.0),
            spos!(7.5),
            spos!(0.35),
            Some(dec!(0.3)),
            Some(dec!(0.5)),
            Some(dec!(0.5)),
            spos!(60.0),
            Some(30),
        );

        chain
    }

    // Tests for Triple Iterator
    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_get_triple_iter_empty() {
        let chain = OptionChain::new("TEST", pos!(100.0), "2024-01-01".to_string(), None, None);
        let triples: Vec<_> = chain.get_triple_iter().collect();
        assert!(triples.is_empty());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_get_triple_iter_two_elements() {
        let mut chain = OptionChain::new("TEST", pos!(100.0), "2024-01-01".to_string(), None, None);
        // Add two options
        chain.add_option(
            pos!(90.0),
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
        );
        chain.add_option(
            pos!(100.0),
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
        );

        let triples: Vec<_> = chain.get_triple_iter().collect();
        assert!(triples.is_empty()); // Not enough elements for a triple
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_get_triple_iter_multiple() {
        let chain = create_test_chain();
        let triples: Vec<_> = chain.get_triple_iter().collect();

        // Should have 4 triples: (90,100,110), (90,100,120), (90,110,120), (100,110,120)
        assert_eq!(triples.len(), 4);

        // Check first triple
        assert_eq!(triples[0].0.strike_price, pos!(90.0));
        assert_eq!(triples[0].1.strike_price, pos!(100.0));
        assert_eq!(triples[0].2.strike_price, pos!(110.0));

        // Check last triple
        assert_eq!(triples[3].0.strike_price, pos!(100.0));
        assert_eq!(triples[3].1.strike_price, pos!(110.0));
        assert_eq!(triples[3].2.strike_price, pos!(120.0));
    }

    // Tests for Triple Inclusive Iterator
    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_get_triple_inclusive_iter_empty() {
        let chain = OptionChain::new("TEST", pos!(100.0), "2024-01-01".to_string(), None, None);
        let triples: Vec<_> = chain.get_triple_inclusive_iter().collect();
        assert!(triples.is_empty());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_get_triple_inclusive_iter_single() {
        let mut chain = OptionChain::new("TEST", pos!(100.0), "2024-01-01".to_string(), None, None);
        chain.add_option(
            pos!(100.0),
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
        );

        let triples: Vec<_> = chain.get_triple_inclusive_iter().collect();
        assert_eq!(triples.len(), 1);
        assert_eq!(triples[0].0.strike_price, triples[0].1.strike_price);
        assert_eq!(triples[0].1.strike_price, triples[0].2.strike_price);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_get_triple_inclusive_iter_multiple() {
        let chain = create_test_chain();
        let triples: Vec<_> = chain.get_triple_inclusive_iter().collect();

        // Count should be (n+2)(n+1)n/6 where n is the number of elements
        assert_eq!(triples.len(), 20); // For 4 elements: 4*5*6/6 = 20

        // Check first few triples (including self-references)
        assert_eq!(triples[0].0.strike_price, pos!(90.0));
        assert_eq!(triples[0].1.strike_price, pos!(90.0));
        assert_eq!(triples[0].2.strike_price, pos!(90.0));
    }

    // Tests for Quad Iterator
    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_get_quad_iter_empty() {
        let chain = OptionChain::new("TEST", pos!(100.0), "2024-01-01".to_string(), None, None);
        let quads: Vec<_> = chain.get_quad_iter().collect();
        assert!(quads.is_empty());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_get_quad_iter_three_elements() {
        let mut chain = OptionChain::new("TEST", pos!(100.0), "2024-01-01".to_string(), None, None);
        // Add three options
        chain.add_option(
            pos!(90.0),
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
        );
        chain.add_option(
            pos!(100.0),
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
        );
        chain.add_option(
            pos!(110.0),
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
        );

        let quads: Vec<_> = chain.get_quad_iter().collect();
        assert!(quads.is_empty()); // Not enough elements for a quad
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_get_quad_iter_multiple() {
        let chain = create_test_chain();
        let quads: Vec<_> = chain.get_quad_iter().collect();

        // Should have 1 quad: (90,100,110,120)
        assert_eq!(quads.len(), 1);

        // Check the quad
        assert_eq!(quads[0].0.strike_price, pos!(90.0));
        assert_eq!(quads[0].1.strike_price, pos!(100.0));
        assert_eq!(quads[0].2.strike_price, pos!(110.0));
        assert_eq!(quads[0].3.strike_price, pos!(120.0));
    }

    // Tests for Quad Inclusive Iterator
    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_get_quad_inclusive_iter_empty() {
        let chain = OptionChain::new("TEST", pos!(100.0), "2024-01-01".to_string(), None, None);
        let quads: Vec<_> = chain.get_quad_inclusive_iter().collect();
        assert!(quads.is_empty());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_get_quad_inclusive_iter_single() {
        let mut chain = OptionChain::new("TEST", pos!(100.0), "2024-01-01".to_string(), None, None);
        chain.add_option(
            pos!(100.0),
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
        );

        let quads: Vec<_> = chain.get_quad_inclusive_iter().collect();
        assert_eq!(quads.len(), 1);
        assert_eq!(quads[0].0.strike_price, quads[0].1.strike_price);
        assert_eq!(quads[0].1.strike_price, quads[0].2.strike_price);
        assert_eq!(quads[0].2.strike_price, quads[0].3.strike_price);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_get_quad_inclusive_iter_multiple() {
        let chain = create_test_chain();
        let quads: Vec<_> = chain.get_quad_inclusive_iter().collect();

        // Count should be (n+3)(n+2)(n+1)n/24 where n is the number of elements
        assert_eq!(quads.len(), 35); // For 4 elements: 7*6*5*4/24 = 35

        // Check first quad (self-reference)
        assert_eq!(quads[0].0.strike_price, pos!(90.0));
        assert_eq!(quads[0].1.strike_price, pos!(90.0));
        assert_eq!(quads[0].2.strike_price, pos!(90.0));
        assert_eq!(quads[0].3.strike_price, pos!(90.0));

        // Check last quad
        assert_eq!(quads[34].0.strike_price, pos!(120.0));
        assert_eq!(quads[34].1.strike_price, pos!(120.0));
        assert_eq!(quads[34].2.strike_price, pos!(120.0));
        assert_eq!(quads[34].3.strike_price, pos!(120.0));
    }
}

#[cfg(test)]
mod tests_is_valid_optimal_side {
    use super::*;

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_upper_side_valid() {
        let option_data = OptionData::new(
            pos!(110.0), // strike price higher than underlying
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
        );
        let underlying_price = pos!(100.0);

        assert!(option_data.is_valid_optimal_side(underlying_price, &FindOptimalSide::Upper));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_upper_side_invalid() {
        let option_data = OptionData::new(
            pos!(90.0), // strike price lower than underlying
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
        );
        let underlying_price = pos!(100.0);

        assert!(!option_data.is_valid_optimal_side(underlying_price, &FindOptimalSide::Upper));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_lower_side_valid() {
        let option_data = OptionData::new(
            pos!(90.0), // strike price lower than underlying
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
        );
        let underlying_price = pos!(100.0);

        assert!(option_data.is_valid_optimal_side(underlying_price, &FindOptimalSide::Lower));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_lower_side_invalid() {
        let option_data = OptionData::new(
            pos!(110.0), // strike price higher than underlying
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
        );
        let underlying_price = pos!(100.0);

        assert!(!option_data.is_valid_optimal_side(underlying_price, &FindOptimalSide::Lower));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_all_side() {
        let option_data = OptionData::new(
            pos!(100.0),
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
        );
        let underlying_price = pos!(100.0);

        assert!(option_data.is_valid_optimal_side(underlying_price, &FindOptimalSide::All));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_range_side_valid() {
        let option_data = OptionData::new(
            pos!(100.0), // strike price within range
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
        );
        let range_start = pos!(90.0);
        let range_end = pos!(110.0);

        assert!(option_data
            .is_valid_optimal_side(pos!(100.0), &FindOptimalSide::Range(range_start, range_end)));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_range_side_invalid_below() {
        let option_data = OptionData::new(
            pos!(80.0), // strike price below range
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
        );
        let range_start = pos!(90.0);
        let range_end = pos!(110.0);

        assert!(!option_data
            .is_valid_optimal_side(pos!(100.0), &FindOptimalSide::Range(range_start, range_end)));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_range_side_invalid_above() {
        let option_data = OptionData::new(
            pos!(120.0), // strike price above range
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
        );
        let range_start = pos!(90.0);
        let range_end = pos!(110.0);

        assert!(!option_data
            .is_valid_optimal_side(pos!(100.0), &FindOptimalSide::Range(range_start, range_end)));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_range_side_at_boundaries() {
        let option_data_lower = OptionData::new(
            pos!(90.0), // strike price at lower boundary
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
        );
        let option_data_upper = OptionData::new(
            pos!(110.0), // strike price at upper boundary
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
        );
        let range_start = pos!(90.0);
        let range_end = pos!(110.0);

        assert!(option_data_lower
            .is_valid_optimal_side(pos!(100.0), &FindOptimalSide::Range(range_start, range_end)));
        assert!(option_data_upper
            .is_valid_optimal_side(pos!(100.0), &FindOptimalSide::Range(range_start, range_end)));
    }
}

#[cfg(test)]
mod rnd_analysis_tests {
    use super::*;
    use crate::{pos, spos};
    use rust_decimal_macros::dec;

    // Helper function to create a standard option chain for testing
    fn create_standard_chain() -> OptionChain {
        let mut chain = OptionChain::new("TEST", pos!(100.0), "2025-02-01".to_string(), None, None);

        // Add a range of options with known prices and volatilities
        let strikes = [90.0, 95.0, 100.0, 105.0, 110.0];
        let call_asks = [10.04, 5.37, 1.95, 0.43, 0.06];
        let implied_vols = [0.17, 0.17, 0.17, 0.17, 0.17];

        for ((&strike, &call_ask), &impl_vol) in strikes
            .iter()
            .zip(call_asks.iter())
            .zip(implied_vols.iter())
        {
            chain.add_option(
                pos!(strike),
                spos!(call_ask - 0.02), // bid slightly lower than ask
                spos!(call_ask),
                None,
                None,
                spos!(impl_vol),
                None,
                None,
                None,
                spos!(100.0),
                Some(50),
            );
        }

        chain
    }

    mod calculate_rnd_tests {
        use super::*;

        #[test]
        #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
        fn test_basic_rnd_calculation() {
            let chain = create_standard_chain();
            let params = RNDParameters {
                risk_free_rate: dec!(0.05),
                interpolation_points: 100,
                derivative_tolerance: pos!(1.0),
            };

            let result = chain.calculate_rnd(&params);
            assert!(result.is_ok());

            let rnd = result.unwrap();

            // Verify densities exist
            assert!(!rnd.densities.is_empty());

            // Verify total probability is approximately 1
            let total: Decimal = rnd.densities.values().sum();
            assert!((total - dec!(1.0)).abs() < dec!(0.0001));

            // Verify all densities are non-negative
            assert!(rnd.densities.values().all(|&d| !d.is_sign_negative()));
        }

        #[test]
        #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
        fn test_tolerance_adjustment() {
            let chain = create_standard_chain();
            let params = RNDParameters {
                risk_free_rate: dec!(0.05),
                interpolation_points: 100,
                derivative_tolerance: pos!(0.1), // Smaller than strike interval
            };

            let result = chain.calculate_rnd(&params);
            assert!(result.is_ok());
        }

        #[test]
        #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
        fn test_default() {
            let chain = OptionChain::new("TEST", pos!(100.0), "2025-02-01".to_string(), None, None);
            let params = RNDParameters::default();

            let result = chain.calculate_rnd(&params);
            assert!(result.is_err());
            assert_eq!(
                result.unwrap_err().to_string(),
                "Derivative tolerance must be greater than zero"
            );
        }

        #[test]
        #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
        fn test_zero_tolerance() {
            let chain = create_standard_chain();
            let params = RNDParameters {
                derivative_tolerance: Positive::ZERO,
                ..Default::default()
            };

            let result = chain.calculate_rnd(&params);
            assert!(result.is_err());
            assert_eq!(
                result.unwrap_err().to_string(),
                "Derivative tolerance must be greater than zero"
            );
        }

        #[test]
        #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
        fn test_expired_option() {
            let mut chain = create_standard_chain();
            chain.expiration_date = "2023-01-01".to_string(); // Past date

            let params = RNDParameters {
                risk_free_rate: dec!(0.05),
                interpolation_points: 100,
                derivative_tolerance: pos!(1.0),
            };

            let result = chain.calculate_rnd(&params);
            assert!(result.is_ok()); // Should still work with past date
        }
    }

    mod calculate_skew_tests {
        use super::*;

        #[test]
        #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
        fn test_basic_skew_calculation() {
            let chain = create_standard_chain();
            let result = chain.calculate_skew();

            assert!(result.is_ok());
            let skew = result.unwrap();

            // Verify we have skew data
            assert!(!skew.is_empty());

            // Verify relative strikes are ordered
            for window in skew.windows(2) {
                assert!(window[0].0 < window[1].0);
            }
        }

        #[test]
        #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
        fn test_flat_volatility_surface() {
            let chain = create_standard_chain(); // All vols are 0.17
            let result = chain.calculate_skew().unwrap();

            // All vol differences should be close to zero
            for (_, vol_diff) in result {
                assert!(vol_diff.abs() < dec!(0.0001));
            }
        }

        #[test]
        #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
        fn test_empty_chain_skew() {
            let chain = OptionChain::new("TEST", pos!(100.0), "2025-02-01".to_string(), None, None);

            let result = chain.calculate_skew();
            assert!(result.is_err());
            assert_eq!(
                result.unwrap_err().to_string(),
                "No ATM implied volatility available"
            );
        }

        #[test]
        #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
        fn test_missing_implied_volatility() {
            let mut chain = create_standard_chain();

            // Add an option without implied volatility
            chain.add_option(
                pos!(115.0),
                spos!(0.1),
                spos!(0.2),
                None,
                None,
                None, // No implied volatility
                None,
                None,
                None,
                spos!(100.0),
                Some(50),
            );

            let result = chain.calculate_skew();
            assert!(result.is_ok()); // Should work with partial data
        }

        #[test]
        #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
        fn test_relative_strike_calculation() {
            let chain = create_standard_chain();
            let result = chain.calculate_skew().unwrap();

            // For ATM strike (100.0), relative strike should be 1.0
            let atm_strike = result
                .iter()
                .find(|(rel_strike, _)| (*rel_strike - dec!(1.0)) < pos!(0.0001));
            assert!(atm_strike.is_some());
        }
    }

    mod calculate_rnd_tests_bis {
        use super::*;

        #[test]
        #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
        fn test_invalid_date_format() {
            let mut chain = create_standard_chain();
            chain.expiration_date = "invalid_date".to_string();

            let params = RNDParameters {
                risk_free_rate: dec!(0.05),
                interpolation_points: 100,
                derivative_tolerance: pos!(1.0),
            };

            let result = chain.calculate_rnd(&params);
            assert!(result.is_err());
        }

        #[test]
        #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
        fn test_negative_risk_free_rate() {
            let chain = create_standard_chain();
            let params = RNDParameters {
                risk_free_rate: dec!(-0.05),
                interpolation_points: 100,
                derivative_tolerance: pos!(1.0),
            };

            let result = chain.calculate_rnd(&params);
            assert!(result.is_ok());
        }

        #[test]
        #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
        fn test_verify_rnd_properties() {
            let chain = create_standard_chain();
            let params = RNDParameters {
                risk_free_rate: dec!(0.05),
                interpolation_points: 100,
                derivative_tolerance: pos!(5.0),
            };

            let result = chain.calculate_rnd(&params).unwrap();
            let densities = &result.densities;

            // Verify mode is near the money
            let mode = densities
                .iter()
                .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
                .unwrap();
            assert_eq!(*mode.0, pos!(100.0));

            // Verify densities decrease away from the money
            let atm_density = densities.get(&pos!(100.0)).unwrap();
            for (strike, density) in densities.iter() {
                if strike < &pos!(90.0) || strike > &pos!(110.0) {
                    assert!(density < atm_density);
                }
            }
        }

        #[test]
        #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
        fn test_strike_interval_detection() {
            let mut chain = create_standard_chain();

            // Add option with different strike interval
            chain.add_option(
                pos!(102.5),
                spos!(1.0),
                spos!(1.1),
                None,
                None,
                spos!(0.17),
                None,
                None,
                None,
                spos!(100.0),
                Some(50),
            );

            let params = RNDParameters {
                risk_free_rate: dec!(0.05),
                interpolation_points: 100,
                derivative_tolerance: pos!(0.1),
            };

            let result = chain.calculate_rnd(&params);
            assert!(result.is_ok());
        }
    }

    mod calculate_skew_tests_bis {
        use super::*;

        #[test]
        #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
        fn test_skew_with_smile() {
            let mut chain =
                OptionChain::new("TEST", pos!(100.0), "2025-02-01".to_string(), None, None);

            // Create new options with a volatility smile
            let strikes = [90.0, 95.0, 100.0, 105.0, 110.0];
            let call_asks = [10.04, 5.37, 1.95, 0.43, 0.06];
            let smile_vols = [0.20, 0.18, 0.17, 0.18, 0.20];

            for ((&strike, &call_ask), &vol) in
                strikes.iter().zip(call_asks.iter()).zip(smile_vols.iter())
            {
                chain.add_option(
                    pos!(strike),
                    spos!(call_ask - 0.02),
                    spos!(call_ask),
                    None,
                    None,
                    spos!(vol),
                    None,
                    None,
                    None,
                    spos!(100.0),
                    Some(50),
                );
            }

            let result = chain.calculate_skew().unwrap();

            // First half of skew should be decreasing
            for window in result.windows(2).take(result.len() / 2) {
                assert!(window[0].1 > window[1].1);
            }

            // Second half of skew should be increasing
            for window in result.windows(2).skip(result.len() / 2) {
                assert!(window[0].1 < window[1].1);
            }
        }

        #[test]
        #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
        fn test_skew_monotonic() {
            let mut chain =
                OptionChain::new("TEST", pos!(100.0), "2025-02-01".to_string(), None, None);

            // Create new options with monotonic skew
            let strikes = [90.0, 95.0, 100.0, 105.0, 110.0];
            let call_asks = [10.04, 5.37, 1.95, 0.43, 0.06];
            let skew_vols = [0.22, 0.20, 0.17, 0.15, 0.14];

            for ((&strike, &call_ask), &vol) in
                strikes.iter().zip(call_asks.iter()).zip(skew_vols.iter())
            {
                chain.add_option(
                    pos!(strike),
                    spos!(call_ask - 0.02),
                    spos!(call_ask),
                    None,
                    None,
                    spos!(vol),
                    None,
                    None,
                    None,
                    spos!(100.0),
                    Some(50),
                );
            }

            let result = chain.calculate_skew().unwrap();

            // Verify monotonic decrease
            for window in result.windows(2) {
                assert!(window[0].1 > window[1].1);
            }
        }

        #[test]
        #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
        fn test_strike_range_coverage() {
            let chain = create_standard_chain();
            let result = chain.calculate_skew().unwrap();

            // Get min and max relative strikes
            let min_rel_strike = result.iter().map(|(k, _)| k).min().unwrap();
            let max_rel_strike = result.iter().map(|(k, _)| k).max().unwrap();

            // Verify range coverage
            assert!(*min_rel_strike < pos!(1.0)); // Have strikes below ATM
            assert!(*max_rel_strike > pos!(1.0)); // Have strikes above ATM
        }
    }
}

#[cfg(test)]
mod tests_option_data_implied_volatility {
    use super::*;
    use crate::utils::setup_logger_with_level;
    use crate::{assert_pos_relative_eq, spos};
    use rust_decimal_macros::dec;

    #[test]
    fn test_calculate_iv_from_call() {
        setup_logger_with_level("debug");
        let mut option_data = OptionData::new(
            pos!(21395.0),        // strike
            spos!(280.0),         // call_bid
            spos!(285.0),         // call_ask
            Some(Positive::ZERO), // put_bid
            spos!(4.0),           // put_ask
            None,                 // initial IV
            None,                 // delta
            None,
            None,
            None, // volume
            None, // open_interest
        );

        option_data.set_mid_prices();

        let params = OptionDataPriceParams::new(
            pos!(21637.0),                   // underlying price
            ExpirationDate::Days(pos!(7.0)), // expiration
            None,                            // IV (will be calculated)
            dec!(0.05),                      // risk-free rate
            pos!(0.0),                       // dividend yield
            None,
        );

        let result = option_data.calculate_implied_volatility(&params);

        assert!(result.is_ok(), "Failed to calculate IV: {:?}", result);
        assert!(option_data.implied_volatility.is_some());

        let iv = option_data.implied_volatility.unwrap() / pos!(100.0);
        assert!(iv > pos!(0.0) && iv < pos!(2.0));
    }

    #[test]
    fn test_calculate_iv_from_put() {
        setup_logger_with_level("debug");
        let mut option_data = OptionData::new(
            pos!(21700.0), // strike
            spos!(30.2),   // call_bid
            spos!(35.1),   // call_ask
            spos!(93.2),   // put_bid
            spos!(98.0),   // put_ask
            None,          // initial IV
            None,          // delta
            None,          // volume
            None,          // open_interest
            None,
            None,
        );

        option_data.set_mid_prices();

        let params = OptionDataPriceParams::new(
            pos!(21637.0),                   // underlying price
            ExpirationDate::Days(pos!(1.0)), // expiration
            None,                            // IV (will be calculated)
            dec!(0.0),                       // risk-free rate
            pos!(0.0),                       // dividend yield
            None,
        );

        let result = option_data.calculate_implied_volatility(&params);

        assert!(result.is_ok(), "Failed to calculate IV: {:?}", result);
        assert!(option_data.implied_volatility.is_some());
        assert_pos_relative_eq!(
            option_data.implied_volatility.unwrap() / pos!(100.0),
            pos!(0.13008),
            pos!(0.0001)
        );

        let iv = option_data.implied_volatility.unwrap() / pos!(100.0);
        assert!(iv > pos!(0.0) && iv < pos!(2.0));
    }

    #[test]
    fn test_calculate_iv_no_prices() {
        let mut option_data = OptionData::new(
            pos!(100.0),
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
        );

        let params = OptionDataPriceParams::new(
            pos!(100.0),
            ExpirationDate::Days(pos!(30.0)),
            None,
            dec!(0.05),
            pos!(0.0),
            None,
        );

        let result = option_data.calculate_implied_volatility(&params);
        assert!(result.is_err());
    }

    #[test]
    fn test_calculate_iv_option_data_to_option() {
        setup_logger_with_level("debug");
        let mut option_data = OptionData::new(
            pos!(21700.0), // strike
            spos!(30.2),   // call_bid
            spos!(35.1),   // call_ask
            spos!(93.2),   // put_bid
            spos!(98.0),   // put_ask
            None,          // initial IV
            None,          // delta
            None,          // volume
            None,          // open_interest
            None,
            None,
        );

        option_data.set_mid_prices();

        let params = OptionDataPriceParams::new(
            pos!(21637.0),                   // underlying price
            ExpirationDate::Days(pos!(1.0)), // expiration
            None,                            // IV (will be calculated)
            dec!(0.0),                       // risk-free rate
            pos!(0.0),                       // dividend yield
            None,
        );

        let result = option_data.calculate_implied_volatility(&params);

        assert!(result.is_ok(), "Failed to calculate IV: {:?}", result);
        assert!(option_data.implied_volatility.is_some());
        let volatility = pos!(0.13008);
        assert_pos_relative_eq!(
            option_data.implied_volatility.unwrap() / pos!(100.0),
            volatility,
            pos!(0.0001)
        );

        let option = option_data
            .get_option(&params, Side::Long, OptionStyle::Call)
            .unwrap();
        assert_eq!(
            option.implied_volatility,
            option_data.implied_volatility.unwrap() / pos!(100.0)
        );

        let option = option_data
            .get_option(&params, Side::Short, OptionStyle::Put)
            .unwrap();
        assert_eq!(
            option.implied_volatility,
            option_data.implied_volatility.unwrap() / pos!(100.0)
        );

        let option = option_data
            .get_option(&params, Side::Long, OptionStyle::Put)
            .unwrap();
        assert_eq!(
            option.implied_volatility,
            option_data.implied_volatility.unwrap() / pos!(100.0)
        );
    }
}

#[cfg(test)]
mod tests_chain_implied_volatility {
    use super::*;
    use crate::utils::time::{get_today_or_tomorrow_formatted, get_tomorrow_formatted};
    use crate::{assert_pos_relative_eq, pos, spos};
    use rust_decimal_macros::dec;

    #[test]
    fn test_update_implied_volatilities() {
        let mut chain = OptionChain::new(
            "TEST",
            pos!(21637.0),
            get_tomorrow_formatted(),
            Some(dec!(0.0)),
            Some(pos!(0.0)),
        );

        chain.add_option(
            pos!(21395.0),        // strike
            spos!(250.0),         // call_bid
            spos!(254.0),         // call_ask
            Some(Positive::ZERO), // put_bid
            spos!(4.0),           // put_ask
            None,                 // implied_volatility (empezamos sin IV)
            None,                 // delta
            None,                 // volume
            None,                 // open_interest
            None,
            None,
        );

        chain.add_option(
            pos!(21700.0), // ATM strike
            spos!(30.2),   // call_bid
            spos!(35.1),   // call_ask
            spos!(93.2),   // put_bid
            spos!(98.0),   // put_ask
            None,          // implied_volatility
            None,          // delta
            None,          // volume
            None,          // open_interest
            None,
            None,
        );

        chain.update_mid_prices();
        chain.update_greeks();

        for option in chain.options.iter() {
            assert!(
                option.implied_volatility.is_some(),
                "IV should be calculated for strike {}",
                option.strike_price
            );

            let iv = option.implied_volatility.unwrap() / pos!(100.0);
            assert!(
                iv > pos!(0.0) && iv < pos!(2.0),
                "IV should be reasonable for strike {}: {}",
                option.strike_price,
                iv
            );

            debug!("Strike: {}, IV: {}", option.strike_price, iv);
        }
    }

    #[test]
    fn test_update_implied_volatilities_today() {
        let mut chain = OptionChain::new(
            "TEST",
            pos!(21637.0),
            get_today_or_tomorrow_formatted(),
            Some(dec!(0.0)),
            Some(pos!(0.0)),
        );

        chain.add_option(
            pos!(21395.0),        // strike
            spos!(250.0),         // call_bid
            spos!(254.0),         // call_ask
            Some(Positive::ZERO), // put_bid
            spos!(4.0),           // put_ask
            None,                 // implied_volatility (empezamos sin IV)
            None,                 // delta
            None,                 // volume
            None,                 // open_interest
            None,
            None,
        );

        chain.add_option(
            pos!(21700.0), // ATM strike
            spos!(30.2),   // call_bid
            spos!(35.1),   // call_ask
            spos!(93.2),   // put_bid
            spos!(98.0),   // put_ask
            None,          // implied_volatility
            None,          // delta
            None,          // volume
            None,          // open_interest
            None,
            None,
        );

        chain.update_mid_prices();
        chain.update_greeks();

        for option in chain.options.iter() {
            assert!(
                option.implied_volatility.is_some(),
                "IV should be calculated for strike {}",
                option.strike_price
            );

            let iv = option.implied_volatility.unwrap() / pos!(100.0);
            assert!(
                iv > pos!(0.0) && iv < pos!(5.0),
                "IV should be reasonable for strike {}: {}",
                option.strike_price,
                iv
            );

            debug!("Strike: {}, IV: {}", option.strike_price, iv);
        }
    }

    #[test]
    fn test_update_implied_volatilities_missing_prices() {
        let mut chain = OptionChain::new(
            "TEST",
            pos!(100.0),
            get_tomorrow_formatted(),
            Some(dec!(0.05)),
            Some(pos!(0.0)),
        );

        chain.add_option(
            pos!(100.0),
            None, // call_bid
            None, // call_ask
            None, // put_bid
            None, // put_ask
            None, // implied_volatility
            None, // delta
            None, // volume
            None, // open_interest
            None,
            None,
        );

        chain.update_mid_prices();
        chain.update_implied_volatilities();

        for option in chain.options.iter() {
            assert!(
                option.implied_volatility.is_none(),
                "IV should be None for options without prices"
            );
        }
    }

    #[test]
    fn test_update_implied_volatilities_maintain_existing() {
        let mut chain = OptionChain::new(
            "TEST",
            pos!(100.0),
            get_tomorrow_formatted(),
            Some(dec!(0.05)),
            Some(pos!(0.0)),
        );

        chain.add_option(
            pos!(100.0),
            None, // call_bid
            None, // call_ask
            None, // put_bid
            None, // put_ask
            spos!(0.2),
            None, // delta
            None, // volume
            None, // open_interest
            None,
            None,
        );

        let original_iv = chain.options.iter().next().unwrap().implied_volatility;
        chain.update_mid_prices();
        chain.update_implied_volatilities();

        for option in chain.options.iter() {
            assert_pos_relative_eq!(
                option.implied_volatility.unwrap() / pos!(100.0),
                original_iv.unwrap(),
                pos!(0.001)
            );
        }
    }
}

#[cfg(test)]
mod tests_option_data_delta {
    use super::*;
    use crate::model::types::ExpirationDate;
    use crate::utils::setup_logger_with_level;
    use crate::{pos, spos};
    use rust_decimal_macros::dec;

    // Helper function to create a standard test OptionDataPriceParams
    fn create_standard_price_params() -> OptionDataPriceParams {
        OptionDataPriceParams::new(
            pos!(100.0),                      // underlying_price
            ExpirationDate::Days(pos!(30.0)), // expiration_date
            spos!(0.2),                       // implied_volatility
            dec!(0.05),                       // risk_free_rate
            pos!(0.02),                       // dividend_yield
            None,
        )
    }

    // Helper function to create standard OptionData
    fn create_standard_option_data() -> OptionData {
        OptionData::new(
            pos!(100.0), // strike_price
            spos!(5.0),  // call_bid
            spos!(5.5),  // call_ask
            spos!(4.5),  // put_bid
            spos!(5.0),  // put_ask
            spos!(0.2),  // implied_volatility
            None,        // delta
            None,
            None,
            spos!(1000.0), // volume
            Some(500),     // open_interest
        )
    }

    #[test]
    fn test_calculate_delta_standard_call() {
        let price_params = create_standard_price_params();
        let mut option_data = create_standard_option_data();

        option_data.calculate_delta(&price_params);

        assert!(option_data.delta_call.is_some());
        let delta = option_data.delta_call.unwrap();

        // Typical at-the-money call delta should be around 0.5
        assert!(delta > dec!(0.4) && delta < dec!(0.6));
    }

    #[test]
    fn test_calculate_delta_near_the_money() {
        let mut price_params = create_standard_price_params();
        price_params.underlying_price = pos!(105.0); // Slightly ITM

        let mut option_data = create_standard_option_data();
        option_data.calculate_delta(&price_params);

        assert!(option_data.delta_call.is_some());
        let delta = option_data.delta_call.unwrap();

        // Near-the-money call delta should be slightly higher than 0.5
        assert!(delta > dec!(0.7) && delta < dec!(0.9));
    }

    #[test]
    fn test_calculate_delta_deep_itm() {
        let mut price_params = create_standard_price_params();
        price_params.underlying_price = pos!(150.0); // Deep ITM

        let mut option_data = create_standard_option_data();
        option_data.calculate_delta(&price_params);

        assert!(option_data.delta_call.is_some());
        let delta = option_data.delta_call.unwrap();

        // Deep ITM call delta should be close to 1
        assert!(delta > dec!(0.9) && delta <= dec!(1.0));
    }

    #[test]
    fn test_calculate_delta_deep_otm() {
        let mut price_params = create_standard_price_params();
        price_params.underlying_price = pos!(50.0); // Deep OTM

        let mut option_data = create_standard_option_data();
        option_data.calculate_delta(&price_params);

        assert!(option_data.delta_call.is_some());
        let delta = option_data.delta_call.unwrap();

        // Deep OTM call delta should be close to 0
        assert!(delta >= Decimal::ZERO && delta < dec!(0.1));
    }

    #[test]
    fn test_calculate_delta_no_volatility() {
        setup_logger_with_level("debug");
        let mut price_params = create_standard_price_params();
        price_params.implied_volatility = None;

        let mut option_data = create_standard_option_data();
        option_data.implied_volatility = None;

        option_data.calculate_delta(&price_params);

        // If no volatility is provided, delta calculation should fail
        assert!(option_data.delta_call.is_none());
    }

    #[test]
    fn test_calculate_delta_multiple_calls() {
        let price_params = create_standard_price_params();
        let mut option_data = create_standard_option_data();

        // Call delta multiple times to ensure consistent behavior
        for _ in 0..3 {
            option_data.calculate_delta(&price_params);
            assert!(option_data.delta_call.is_some());
        }
    }

    #[test]
    fn test_calculate_delta_different_expiration() {
        let mut price_params = create_standard_price_params();
        price_params.expiration_date = ExpirationDate::Days(pos!(60.0)); // Longer expiration

        let mut option_data = create_standard_option_data();
        option_data.calculate_delta(&price_params);

        assert!(option_data.delta_call.is_some());
        let delta = option_data.delta_call.unwrap();

        // Delta should still be reasonable with longer expiration
        assert!(delta > Decimal::ZERO && delta <= dec!(1.0));
    }
}
