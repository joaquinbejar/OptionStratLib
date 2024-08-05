use crate::model::types::{OptionStyle, OptionType, Side};
use crate::pricing::binomial_model::{
    generate_binomial_tree, price_binomial, BinomialPricingParams,
};
use chrono::{DateTime, Utc};

pub struct OptionConfig {
    pub option_type: OptionType,
    pub side: Side,
    pub underlying_symbol: String,
    pub strike_price: f64,
    pub expiration_date: DateTime<Utc>,
    pub current_price: f64,
    pub implied_volatility: f64,
    pub quantity: u32,
    pub underlying_price: f64,
    pub risk_free_rate: f64,
    pub option_style: OptionStyle,
}

#[allow(dead_code)]
pub struct Option {
    pub option_type: OptionType,
    pub side: Side,
    pub underlying_symbol: String,
    pub strike_price: f64,
    pub expiration_date: DateTime<Utc>,
    pub current_price: f64,
    pub implied_volatility: f64,
    pub quantity: u32,
    pub underlying_price: f64,
    pub risk_free_rate: f64,
    pub option_style: OptionStyle,
}

impl Option {
    pub fn new(config: OptionConfig) -> Self {
        Option {
            option_type: config.option_type,
            option_style: config.option_style,
            side: config.side,
            underlying_symbol: config.underlying_symbol,
            strike_price: config.strike_price,
            expiration_date: config.expiration_date,
            current_price: config.current_price,
            implied_volatility: config.implied_volatility,
            quantity: config.quantity,
            underlying_price: config.underlying_price,
            risk_free_rate: config.risk_free_rate,
        }
    }

    pub fn time_to_expiration(&self) -> f64 {
        let now = Utc::now();
        let duration = self.expiration_date.signed_duration_since(now);
        duration.num_days() as f64 / 365.0
    }

    pub fn is_long(&self) -> bool {
        matches!(self.side, Side::Long)
    }

    pub fn is_short(&self) -> bool {
        matches!(self.side, Side::Short)
    }

    pub fn calculate_price_binomial(&self, no_steps: usize) -> f64 {
        let expiry = self.time_to_expiration();
        price_binomial(BinomialPricingParams {
            asset: self.underlying_price,
            volatility: self.implied_volatility,
            int_rate: self.risk_free_rate,
            strike: self.strike_price,
            expiry,
            no_steps,
            option_type: &self.option_type,
            option_style: &self.option_style,
            side: &self.side,
        })
    }

    pub fn calculate_price_binomial_tree(
        &self,
        no_steps: usize,
    ) -> (f64, Vec<Vec<f64>>, Vec<Vec<f64>>) {
        let expiry = self.time_to_expiration();
        let params = BinomialPricingParams {
            asset: self.underlying_price,
            volatility: self.implied_volatility,
            int_rate: self.risk_free_rate,
            strike: self.strike_price,
            expiry,
            no_steps,
            option_type: &self.option_type,
            option_style: &self.option_style,
            side: &self.side,
        };

        let (asset_tree, option_tree) = generate_binomial_tree(&params);
        let price = match self.side {
            Side::Long => option_tree[0][0],
            Side::Short => -option_tree[0][0],
        };

        (price, asset_tree, option_tree)
    }

    // TODO:
    // - calculate_intrinsic_value
    // - calculate_time_value
    // - is_in_the_money
    // - calculate_delta
    // - calculate_gamma
    // - calculate_theta
    // - calculate_vega
    // - calculate_rho
}
