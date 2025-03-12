use crate::chains::chain::OptionData;
use crate::constants::{IV_TOLERANCE, MAX_ITERATIONS_IV, ZERO};
use crate::error::{GreeksError, OptionsError, OptionsResult, VolatilityError};
use crate::greeks::Greeks;
use crate::model::types::{ExpirationDate, OptionStyle, OptionType, Side};
use crate::pnl::utils::{PnL, PnLCalculator};
use crate::pricing::{
    BinomialPricingParams, Payoff, PayoffInfo, Profit, black_scholes, generate_binomial_tree,
    price_binomial, telegraph,
};
use crate::visualization::model::ChartVerticalLine;
use crate::visualization::utils::Graph;
use crate::{Positive, pos};
use num_traits::{FromPrimitive, ToPrimitive};
use plotters::prelude::{BLACK, ShapeStyle};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use std::error::Error;
use tracing::{error, trace};

/// Result type for binomial tree pricing models, containing:
/// - The option price 
/// - Price tree (asset price evolution)
/// - Option value tree (option value at each node)
type PriceBinomialTree = OptionsResult<(Decimal, Vec<Vec<Decimal>>, Vec<Vec<Decimal>>)>;

/// Parameters for exotic option pricing models.
///
/// This structure holds specific data required by various exotic option types
/// such as Asian options (which depend on average prices) and Lookback options
/// (which depend on minimum/maximum prices during the option's lifetime).
///
/// Each field is optional since different exotic option types require different parameters.
#[derive(Clone, Default, PartialEq, Serialize, Deserialize, Debug)]
pub struct ExoticParams {
    /// Historical spot prices, primarily used for Asian options which
    /// depend on the average price of the underlying asset.
    pub spot_prices: Option<Vec<Positive>>, // Asian

    /// Minimum observed spot price during the option's lifetime,
    /// used for lookback option pricing.
    pub spot_min: Option<Decimal>,          // Lookback

    /// Maximum observed spot price during the option's lifetime,
    /// used for lookback option pricing.
    pub spot_max: Option<Decimal>,          // Lookback
}

/// Represents a financial option contract with its essential parameters and characteristics.
///
/// This structure contains all the necessary information to define an options contract,
/// including its type (call/put), market position (long/short), pricing parameters,
/// and contract specifications. It serves as the core data model for option pricing,
/// risk analysis, and strategy development.
///
/// The `Options` struct supports both standard option types and exotic options through
/// the optional `exotic_params` field, making it versatile for various financial modeling
/// scenarios.
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct Options {
    /// Specifies whether this is a Call or Put option, determining the fundamental
    /// right the option contract provides (buying or selling the underlying).
    pub option_type: OptionType,

    /// Indicates whether the position is Long (purchased) or Short (sold/written),
    /// which determines the profit/loss direction and risk profile.
    pub side: Side,

    /// The ticker symbol or identifier of the underlying asset (e.g., "AAPL" for Apple stock).
    pub underlying_symbol: String,

    /// The price at which the option holder can exercise their right to buy (for calls)
    /// or sell (for puts) the underlying asset.
    pub strike_price: Positive,

    /// When the option contract expires, either as days from now or as a specific date.
    pub expiration_date: ExpirationDate,

    /// The market's expectation for future volatility of the underlying asset,
    /// a key parameter for option pricing models.
    pub implied_volatility: Positive,

    /// The number of contracts in this position.
    pub quantity: Positive,

    /// The current market price of the underlying asset.
    pub underlying_price: Positive,

    /// The current risk-free interest rate used in option pricing models,
    /// typically based on treasury yields of similar duration.
    pub risk_free_rate: Decimal,

    /// The option exercise style (European or American), determining when the 
    /// option can be exercised.
    pub option_style: OptionStyle,

    /// The annualized dividend yield of the underlying asset, affecting option pricing
    /// particularly for longer-dated contracts.
    pub dividend_yield: Positive,

    /// Additional parameters required for exotic option types like Asian or Lookback options.
    /// This field is None for standard (vanilla) options.
    pub exotic_params: Option<ExoticParams>,
}
impl Options {
    
    /// Creates a new options contract with the specified parameters.
    ///
    /// This constructor creates an instance of `Options` with all the required parameters
    /// for defining and pricing an option contract. It supports both standard (vanilla) 
    /// options and exotic options through the optional `exotic_params` parameter.
    ///
    /// # Parameters
    ///
    /// * `option_type` - Specifies whether this is a Call or Put option, determining the fundamental
    ///   right the option contract provides.
    /// * `side` - Indicates whether the position is Long (purchased) or Short (sold/written),
    ///   which determines the profit/loss direction.
    /// * `underlying_symbol` - The ticker symbol or identifier of the underlying asset (e.g., "AAPL").
    /// * `strike_price` - The price at which the option can be exercised, represented as a `Positive` value.
    /// * `expiration_date` - When the option contract expires, either as days from now or as a specific date.
    /// * `implied_volatility` - The market's expectation for future volatility of the underlying asset,
    ///   a key parameter for option pricing.
    /// * `quantity` - The number of contracts in this position, represented as a `Positive` value.
    /// * `underlying_price` - The current market price of the underlying asset.
    /// * `risk_free_rate` - The current risk-free interest rate used in option pricing models.
    /// * `option_style` - The option exercise style (European or American), determining when the 
    ///   option can be exercised.
    /// * `dividend_yield` - The annualized dividend yield of the underlying asset, affecting option pricing.
    /// * `exotic_params` - Additional parameters required for exotic option types. Set to `None` for
    ///   standard (vanilla) options.
    ///
    /// # Returns
    ///
    /// A fully configured `Options` instance with all the specified parameters.
    ///
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        option_type: OptionType,
        side: Side,
        underlying_symbol: String,
        strike_price: Positive,
        expiration_date: ExpirationDate,
        implied_volatility: Positive,
        quantity: Positive,
        underlying_price: Positive,
        risk_free_rate: Decimal,
        option_style: OptionStyle,
        dividend_yield: Positive,
        exotic_params: Option<ExoticParams>,
    ) -> Self {
        Options {
            option_type,
            side,
            underlying_symbol,
            strike_price,
            expiration_date,
            implied_volatility,
            quantity,
            underlying_price,
            risk_free_rate,
            option_style,
            dividend_yield,
            exotic_params,
        }
    }

    /// Updates option parameters using data from an OptionData structure.
    ///
    /// This method updates the option's strike price and implied volatility based on the
    /// values provided in the option_data parameter. If the implied volatility is not 
    /// available in the option data, it defaults to zero.
    ///
    /// # Arguments
    ///
    /// * `option_data` - A reference to an OptionData structure containing updated option parameters.
    ///
    pub(crate) fn update_from_option_data(&mut self, option_data: &OptionData) {
        self.strike_price = option_data.strike_price;
        self.implied_volatility = option_data.implied_volatility.unwrap_or(Positive::ZERO);
        trace!("Updated Option: {:#?}", self);
    }

    /// Calculates the time to expiration of the option in years.
    ///
    /// This function computes the time remaining until the option's expiration date, 
    /// expressed as a positive decimal value representing years. This is a key parameter
    /// used in option pricing models.
    ///
    /// # Returns
    ///
    /// * `OptionsResult<Positive>` - A result containing the time to expiration in years
    ///   as a Positive value, or an error if the calculation failed.
    ///
    pub fn time_to_expiration(&self) -> OptionsResult<Positive> {
        Ok(self.expiration_date.get_years()?)
    }

    /// Determines if the option position is long (purchased).
    ///
    /// A long position indicates that the option has been bought, meaning the holder
    /// has the right to exercise the option according to its terms.
    ///
    /// # Returns
    ///
    /// * `bool` - Returns true if the option is held as a long position, false otherwise.
    ///
    pub fn is_long(&self) -> bool {
        matches!(self.side, Side::Long)
    }

    /// Determines if the option position is short (written/sold).
    ///
    /// A short position indicates that the option has been sold or written, meaning
    /// the holder has the obligation to fulfill the contract terms if the option is exercised.
    ///
    /// # Returns
    ///
    /// * `bool` - Returns true if the option is held as a short position, false otherwise.
    ///
    pub fn is_short(&self) -> bool {
        matches!(self.side, Side::Short)
    }

    /// Calculates the price of an option using the binomial tree model.
    ///
    /// This method implements the binomial option pricing model which constructs a 
    /// discrete-time lattice (tree) of possible future underlying asset prices to
    /// determine the option's value. The approach is particularly valuable for pricing
    /// American options and other early-exercise scenarios.
    ///
    /// The calculation divides the time to expiration into a specified number of steps,
    /// creating a binomial tree that represents possible price paths of the underlying asset.
    /// The option's value is then calculated by working backward from expiration to the
    /// present value.
    ///
    /// # Parameters
    ///
    /// * `no_steps` - The number of steps to use in the binomial tree calculation.
    ///   Higher values increase accuracy but also computational cost.
    ///
    /// # Returns
    ///
    /// * `OptionsResult<Decimal>` - A result containing the calculated option price as a
    ///   Decimal value, or an OptionsError if the calculation failed.
    ///
    /// # Errors
    ///
    /// Returns an `OptionsError::OtherError` if:
    /// * The number of steps is zero
    /// * The time to expiration calculation fails
    /// * The binomial price calculation fails
    pub fn calculate_price_binomial(&self, no_steps: usize) -> OptionsResult<Decimal> {
        if no_steps == 0 {
            return Err(OptionsError::OtherError {
                reason: "Number of steps cannot be zero".to_string(),
            });
        }
        let expiry = self.time_to_expiration()?;
        let cpb = price_binomial(BinomialPricingParams {
            asset: self.underlying_price,
            volatility: self.implied_volatility,
            int_rate: self.risk_free_rate,
            strike: self.strike_price,
            expiry,
            no_steps,
            option_type: &self.option_type,
            option_style: &self.option_style,
            side: &self.side,
        })?;
        Ok(cpb)
    }

    /// Calculates option price using the binomial tree model.
    ///
    /// This method implements a binomial tree (lattice) approach to option pricing, which
    /// discretizes the underlying asset's price movement over time. The model builds a tree
    /// of possible future asset prices and works backwards to determine the current option value.
    ///
    /// # Parameters
    ///
    /// * `no_steps` - The number of discrete time steps to use in the model. Higher values
    ///   increase precision but also computational cost.
    ///
    /// # Returns
    ///
    /// * `PriceBinomialTree` - A result containing:
    ///   - The calculated option price
    ///   - The asset price tree (underlying price evolution)
    ///   - The option value tree (option price at each node)
    ///
    /// This method is particularly valuable for pricing American options and other early-exercise
    /// scenarios that cannot be accurately priced using closed-form solutions.
    pub fn calculate_price_binomial_tree(&self, no_steps: usize) -> PriceBinomialTree {
        let expiry = self.time_to_expiration()?;
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
        let (asset_tree, option_tree) = generate_binomial_tree(&params)?;
        let price = match self.side {
            Side::Long => option_tree[0][0],
            Side::Short => -option_tree[0][0],
        };
        Ok((price, asset_tree, option_tree))
    }

    /// Calculates option price using the Black-Scholes model.
    ///
    /// This method implements the Black-Scholes option pricing formula, which provides
    /// a closed-form solution for European-style options. The model assumes lognormal
    /// distribution of underlying asset prices and constant volatility.
    ///
    /// # Returns
    ///
    /// * `OptionsResult<Decimal>` - A result containing the calculated option price
    ///   as a Decimal value, or an error if the calculation failed.
    ///
    /// This method is computationally efficient but limited to European options without
    /// early exercise capabilities.
    pub fn calculate_price_black_scholes(&self) -> OptionsResult<Decimal> {
        Ok(black_scholes(self)?)
    }

    /// Calculates option price using the Telegraph equation approach.
    ///
    /// This method implements a finite-difference method based on the Telegraph equation
    /// to price options. This approach can handle a variety of option styles and types,
    /// including path-dependent options.
    ///
    /// # Parameters
    ///
    /// * `no_steps` - The number of discrete time steps to use in the model. Higher values
    ///   increase precision but also computational cost.
    ///
    /// # Returns
    ///
    /// * `Result<Decimal, Box<dyn Error>>` - A result containing the calculated option price
    ///   as a Decimal value, or a boxed error if the calculation failed.
    pub fn calculate_price_telegraph(&self, no_steps: usize) -> Result<Decimal, Box<dyn Error>> {
        telegraph(self, no_steps, None, None)
    }

    /// Calculates the intrinsic value (payoff) of the option at the current underlying price.
    ///
    /// The payoff represents what the option would be worth if exercised immediately,
    /// based on the current market conditions. For out-of-the-money options, the payoff
    /// will be zero.
    ///
    /// # Returns
    ///
    /// * `OptionsResult<Decimal>` - A result containing the calculated payoff as a 
    ///   Decimal value, adjusted for the quantity of contracts held, or an error if 
    ///   the calculation failed.
    ///
    /// This method is useful for determining the exercise value of an option and for
    /// analyzing whether an option has intrinsic value.
    pub fn payoff(&self) -> OptionsResult<Decimal> {
        let payoff_info = PayoffInfo {
            spot: self.underlying_price,
            strike: self.strike_price,
            style: self.option_style,
            side: self.side,
            spot_prices: None,
            spot_min: None,
            spot_max: None,
        };
        let payoff = self.option_type.payoff(&payoff_info) * self.quantity.to_f64();
        Ok(Decimal::from_f64(payoff).unwrap())
    }

    /// Calculates the financial payoff value of the option at a specific underlying price.
    ///
    /// This method determines the option's payoff based on its type, strike price, style,
    /// and side (long/short) at the given underlying price. The result represents the
    /// total profit or loss for the option position at that price, adjusted by the position quantity.
    ///
    /// # Parameters
    ///
    /// * `price` - A `Positive` value representing the hypothetical price of the underlying asset.
    ///
    /// # Returns
    ///
    /// * `OptionsResult<Decimal>` - The calculated payoff value as a `Decimal`, wrapped in a `Result` type.
    ///   Returns an `Err` if the payoff calculation encounters an error.
    ///
    pub fn payoff_at_price(&self, price: Positive) -> OptionsResult<Decimal> {
        let payoff_info = PayoffInfo {
            spot: price,
            strike: self.strike_price,
            style: self.option_style,
            side: self.side,
            spot_prices: None,
            spot_min: None,
            spot_max: None,
        };
        let price = self.option_type.payoff(&payoff_info) * self.quantity.to_f64();
        Ok(Decimal::from_f64(price).unwrap())
    }

    /// Calculates the intrinsic value of the option.
    ///
    /// The intrinsic value is the difference between the underlying asset's price and the option's strike price.
    /// For call options, the intrinsic value is the maximum of zero and the difference between the underlying price and the strike price.
    /// For put options, the intrinsic value is the maximum of zero and the difference between the strike price and the underlying price.
    ///
    /// # Arguments
    ///
    /// * `underlying_price` - The current price of the underlying asset.
    ///
    /// # Returns
    ///
    /// * `OptionsResult<Decimal>` - The intrinsic value of the option, or an error if the calculation fails.
    pub fn intrinsic_value(&self, underlying_price: Positive) -> OptionsResult<Decimal> {
        let payoff_info = PayoffInfo {
            spot: underlying_price,
            strike: self.strike_price,
            style: self.option_style,
            side: self.side,
            spot_prices: None,
            spot_min: None,
            spot_max: None,
        };
        let iv = self.option_type.payoff(&payoff_info) * self.quantity.to_f64();
        Ok(Decimal::from_f64(iv).unwrap())
    }

    /// Determines whether an option is "in-the-money" based on its current price relative to strike price.
    ///
    /// An option is considered in-the-money when:
    /// - For Call options: the underlying asset price is greater than or equal to the strike price
    /// - For Put options: the underlying asset price is less than or equal to the strike price
    ///
    /// This status is important for evaluating the option's current value and potential profitability.
    ///
    /// # Returns
    /// `true` if the option is in-the-money, `false` otherwise
    pub fn is_in_the_money(&self) -> bool {
        match self.option_style {
            OptionStyle::Call => self.underlying_price >= self.strike_price,
            OptionStyle::Put => self.underlying_price <= self.strike_price,
        }
    }

    /// Calculates the time value component of an option's price.
    ///
    /// Time value represents the portion of an option's premium that exceeds its intrinsic value.
    /// It reflects the market's expectation that the option may become more valuable before expiration
    /// due to potential favorable movements in the underlying asset price.
    ///
    /// The calculation uses the Black-Scholes model to determine the total option price,
    /// then subtracts the intrinsic value to find the time value component.
    ///
    /// # Returns
    /// - `Ok(Decimal)` containing the time value (never negative, minimum value is zero)
    /// - `Err` if the price calculation encounters an error
    pub fn time_value(&self) -> OptionsResult<Decimal> {
        let option_price = self.calculate_price_black_scholes()?.abs();
        let intrinsic_value = self.intrinsic_value(self.underlying_price)?;
        Ok((option_price - intrinsic_value).max(Decimal::ZERO))
    }

    /// Validates that the option parameters are in a valid state for calculations.
    ///
    /// This function performs comprehensive validation of the option's critical parameters
    /// to ensure they meet basic requirements for meaningful financial calculations.
    /// It logs detailed error messages when validation fails.
    ///
    /// Validation checks include:
    /// - Underlying symbol is not empty
    /// - Implied volatility is non-negative
    /// - Quantity is non-zero
    /// - Risk-free rate is non-negative
    /// - Strike price is positive and non-zero
    /// - Underlying price is positive and non-zero
    ///
    /// # Returns
    /// `true` if all parameters are valid, `false` if any validation fails
    pub(crate) fn validate(&self) -> bool {
        if self.underlying_symbol == *"" {
            error!("Underlying symbol is empty");
            return false;
        }
        if self.implied_volatility < ZERO {
            error!("Implied volatility is less than zero");
            return false;
        }
        if self.quantity == ZERO {
            error!("Quantity is equal to zero");
            return false;
        }
        if self.risk_free_rate < Decimal::ZERO {
            error!("Risk free rate is less than zero");
            return false;
        }
        if self.strike_price == Positive::ZERO {
            error!("Strike is zero");
            return false;
        }
        if self.underlying_price == Positive::ZERO {
            error!("Underlying price is zero");
            return false;
        }
        true
    }

    /// **calculate_implied_volatility**:
    ///
    /// This function estimates the implied volatility of an option based on its market price
    /// using binary search. Implied volatility is a key metric in options trading that reflects
    /// the market's view of the expected volatility of the underlying asset.
    ///
    /// ### Parameters:
    ///
    /// - `market_price`: The market price of the option as a `Decimal`. This represents the cost
    ///   at which the option is traded in the market.
    ///
    /// ### Returns:
    ///
    /// - `Ok(Positive)`: A `Positive` value representing the calculated implied volatility as a percentage.
    /// - `Err(ImpliedVolatilityError)`: An error indicating the reason calculation failed, such as:
    ///     - No convergence within the maximum number of iterations.
    ///     - Invalid option parameters.
    ///
    /// ### Implementation Details:
    ///
    /// - **Binary Search**: The function uses a binary search approach to iteratively find
    ///   the implied volatility (`volatility`) that narrows the difference between the calculated
    ///   option price (via Black-Scholes) and the target `market_price`.
    ///
    /// - **Short Options Adjustment**: For short options, the market price is inverted (negated),
    ///   and this adjustment ensures proper calculation of implied volatility.
    ///
    /// - **Bounds and Iteration**: The method starts with a maximum bound (`5.0`, representing 500%
    ///   volatility) and a lower bound (`0.0`). It adjusts these bounds based on whether the computed
    ///   price is above or below the target and repeats until convergence or the maximum number of
    ///   iterations is reached (`MAX_ITERATIONS_IV`).
    ///
    /// - **Convergence Tolerance**: The function stops iterating when the computed price is within `IV_TOLERANCE`
    ///   of the target market price or when the difference between the high and low bounds is smaller
    ///   than a threshold (`0.0001`).
    ///
    /// ### Error Cases:
    /// - **No Convergence**: If the binary search exhausts the allowed number of iterations (`MAX_ITERATIONS_IV`)
    ///   without sufficiently narrowing down the implied volatility, the function returns an `ImpliedVolatilityError::NoConvergence`.
    /// - **Invalid Parameters**: Bounds violations or invalid market inputs can potentially cause other errors
    ///   during calculations.
    ///
    /// ### Example Usage:
    /// ```rust
    /// use rust_decimal_macros::dec;
    /// use rust_decimal::Decimal;
    /// use tracing::{error, info};
    /// use optionstratlib::{pos, ExpirationDate, OptionStyle, OptionType, Options, Side};
    ///
    /// let options = Options::new(
    ///             OptionType::European,
    ///             Side::Short,
    ///             "TEST".to_string(),
    ///             pos!(6050.0), // strike
    ///             ExpirationDate::Days(pos!(60.0)),
    ///             pos!(0.1),     // initial iv
    ///             pos!(1.0),     // qty
    ///             pos!(6032.18), // underlying
    ///             dec!(0.0),     // rate
    ///             OptionStyle::Call,
    ///             pos!(0.0), // div
    ///             None,
    ///         ); // Configure your option parameters
    /// let market_price = dec!(133.5);  
    ///
    /// match options.calculate_implied_volatility(market_price) {
    ///     Ok(volatility) => info!("Implied Volatility: {}", volatility.to_dec()),
    ///     Err(e) => error!("Failed to calculate implied volatility: {:?}", e),
    /// }
    /// ```
    pub fn calculate_implied_volatility(
        &self,
        market_price: Decimal,
    ) -> Result<Positive, VolatilityError> {
        let is_short = self.is_short();
        let target_price = if is_short {
            -market_price
        } else {
            market_price
        };

        // Initialize high and low bounds for volatility
        let mut high = pos!(5.0); // 500% max volatility
        let mut low = pos!(0.0);

        // Binary search through volatilities until we find one that gives us our target price
        // or until we reach maximum iterations
        for _ in 0..MAX_ITERATIONS_IV {
            // Calculate midpoint volatility
            let mid_vol = (high.to_dec() + low.to_dec()) / Decimal::TWO;
            let volatility = Positive(mid_vol);

            // Calculate option price at this volatility
            let mut option_copy = self.clone();
            option_copy.implied_volatility = volatility;
            let price = option_copy.calculate_price_black_scholes()?;

            // Adjust price for short positions
            let actual_price = if is_short { -price } else { price };

            // Check if we're close enough to the target price
            if (actual_price - target_price).abs() < IV_TOLERANCE {
                return Ok(volatility);
            }

            // Update bounds based on whether this price was too high or too low
            if actual_price > target_price {
                high = volatility;
            } else {
                low = volatility;
            }

            // Check if our range is too small (meaning we've converged)
            if (high - low).to_dec() < dec!(0.0001) {
                return Ok(volatility);
            }
        }

        // If we haven't found a solution after max iterations
        Err(VolatilityError::NoConvergence {
            iterations: MAX_ITERATIONS_IV,
            last_volatility: (high + low) / Positive::TWO,
        })
    }
}

impl Default for Options {
    fn default() -> Self {
        Options {
            option_type: OptionType::European,
            side: Side::Long,
            underlying_symbol: "".to_string(),
            strike_price: Positive::ZERO,
            expiration_date: ExpirationDate::Days(Positive::ZERO),
            implied_volatility: Positive::ZERO,
            quantity: Positive::ZERO,
            underlying_price: Positive::ZERO,
            risk_free_rate: Decimal::ZERO,
            option_style: OptionStyle::Call,
            dividend_yield: Positive::ZERO,
            exotic_params: None,
        }
    }
}

impl Greeks for Options {
    fn get_options(&self) -> Result<Vec<&Options>, GreeksError> {
        Ok(vec![self])
    }
}

impl PnLCalculator for Options {
    fn calculate_pnl(
        &self,
        market_price: &Positive,
        expiration_date: ExpirationDate,
        implied_volatility: &Positive,
    ) -> Result<PnL, Box<dyn Error>> {
        // Create a copy of the current option with updated parameters
        let mut current_option = self.clone();
        current_option.underlying_price = *market_price;
        current_option.expiration_date = expiration_date;
        current_option.implied_volatility = *implied_volatility;

        // Calculate theoretical price at current market conditions
        let current_price = current_option.calculate_price_black_scholes()?;

        // Calculate initial price (when option was created)
        let initial_price = self.calculate_price_black_scholes()?;

        // Calculate initial costs (premium paid/received)
        let (initial_costs, initial_income) = match self.side {
            Side::Long => (initial_price * self.quantity, Decimal::ZERO),
            Side::Short => (Decimal::ZERO, -initial_price * self.quantity),
        };

        // Calculate unrealized PnL adjusted for position side
        let unrealized = Some((current_price - initial_price) * self.quantity);

        Ok(PnL::new(
            None, // No realized PnL yet
            unrealized,
            initial_costs.into(),
            initial_income.into(),
            current_option.expiration_date.get_date()?,
        ))
    }

    fn calculate_pnl_at_expiration(
        &self,
        underlying_price: &Positive,
    ) -> Result<PnL, Box<dyn Error>> {
        let realized = Some(self.payoff_at_price(*underlying_price)?);
        let initial_price = self.calculate_price_black_scholes()?;

        let (initial_costs, initial_income) = match self.side {
            Side::Long => (initial_price * self.quantity, Decimal::ZERO),
            Side::Short => (Decimal::ZERO, initial_price * self.quantity),
        };

        Ok(PnL::new(
            realized, // No realized PnL yet
            None,
            initial_costs.into(),
            initial_income.into(),
            self.expiration_date.get_date()?,
        ))
    }
}

impl Profit for Options {
    fn calculate_profit_at(&self, price: Positive) -> Result<Decimal, Box<dyn Error>> {
        Ok(self.payoff_at_price(price)?)
    }
}

impl Graph for Options {
    fn title(&self) -> String {
        format!(
            "Underlying: {} @ ${:.0} {} {} {}",
            self.underlying_symbol,
            self.strike_price,
            self.side,
            self.option_style,
            self.option_type
        )
    }

    fn get_values(&self, data: &[Positive]) -> Vec<f64> {
        data.iter()
            .map(|&price| self.intrinsic_value(price).unwrap().to_f64().unwrap())
            .collect()
    }

    fn get_vertical_lines(&self) -> Vec<ChartVerticalLine<f64, f64>> {
        let vertical_lines = vec![ChartVerticalLine {
            x_coordinate: self.strike_price.to_f64(),
            y_range: (-50000.0, 50000.0),
            label: "Strike".to_string(),
            label_offset: (5.0, 5.0),
            line_color: BLACK,
            label_color: BLACK,
            line_style: ShapeStyle::from(&BLACK).stroke_width(1),
            font_size: 18,
        }];

        vertical_lines
    }
}

#[cfg(test)]
mod tests_options {
    use super::*;
    use crate::model::utils::create_sample_option_simplest;
    use crate::pos;
    use approx::assert_relative_eq;
    use chrono::{Duration, Utc};
    use rust_decimal_macros::dec;

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_new_option() {
        let option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        assert_eq!(option.underlying_symbol, "AAPL");
        assert_eq!(option.strike_price, 100.0);
        assert_eq!(option.implied_volatility, 0.2);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_time_to_expiration() {
        let option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        assert_relative_eq!(
            option.time_to_expiration().unwrap().to_f64(),
            30.0 / 365.0,
            epsilon = 0.0001
        );

        let future_date = Utc::now() + Duration::days(60);
        let option_with_datetime = Options::new(
            OptionType::European,
            Side::Long,
            "AAPL".to_string(),
            pos!(100.0),
            ExpirationDate::DateTime(future_date),
            pos!(0.2),
            Positive::ONE,
            pos!(105.0),
            dec!(0.05),
            OptionStyle::Call,
            pos!(0.01),
            None,
        );
        assert!(option_with_datetime.time_to_expiration().unwrap() >= 59.0 / 365.0);
        assert!(option_with_datetime.time_to_expiration().unwrap() < 61.0 / 365.0);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_is_long_and_short() {
        let long_option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        assert!(long_option.is_long());
        assert!(!long_option.is_short());

        let short_option = Options::new(
            OptionType::European,
            Side::Short,
            "AAPL".to_string(),
            pos!(100.0),
            ExpirationDate::Days(pos!(30.0)),
            pos!(0.2),
            Positive::ONE,
            pos!(105.0),
            dec!(0.05),
            OptionStyle::Call,
            pos!(0.01),
            None,
        );
        assert!(!short_option.is_long());
        assert!(short_option.is_short());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_calculate_price_binomial() {
        let option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        let price = option.calculate_price_binomial(100).unwrap();
        assert!(price > Decimal::ZERO);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_calculate_price_binomial_tree() {
        let option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        let (price, asset_tree, option_tree) = option.calculate_price_binomial_tree(5).unwrap();
        assert!(price > Decimal::ZERO);
        assert_eq!(asset_tree.len(), 6);
        assert_eq!(option_tree.len(), 6);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_calculate_price_binomial_tree_short() {
        let option = create_sample_option_simplest(OptionStyle::Call, Side::Short);
        let (price, asset_tree, option_tree) = option.calculate_price_binomial_tree(5).unwrap();
        assert!(price > Decimal::ZERO);
        assert_eq!(asset_tree.len(), 6);
        assert_eq!(option_tree.len(), 6);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_calculate_price_black_scholes() {
        let option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        let price = option.calculate_price_black_scholes().unwrap();
        assert!(price > Decimal::ZERO);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_payoff_european_call_long() {
        let call_option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        let call_payoff = call_option.payoff().unwrap();
        assert_eq!(call_payoff, Decimal::ZERO); // max(100 - 100, 0) = 0

        let put_option = Options::new(
            OptionType::European,
            Side::Long,
            "AAPL".to_string(),
            pos!(100.0),
            ExpirationDate::Days(pos!(30.0)),
            pos!(0.2),
            Positive::ONE,
            pos!(95.0),
            dec!(0.05),
            OptionStyle::Put,
            pos!(0.01),
            None,
        );
        let put_payoff = put_option.payoff().unwrap();
        assert_eq!(put_payoff.to_f64().unwrap(), 5.0); // max(100 - 95, 0) = 5
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_calculate_time_value() {
        let option = Options::new(
            OptionType::European,
            Side::Long,
            "AAPL".to_string(),
            pos!(100.0),
            ExpirationDate::Days(pos!(30.0)),
            pos!(0.2),
            Positive::ONE,
            pos!(105.0),
            dec!(0.05),
            OptionStyle::Call,
            Positive::ZERO,
            None,
        );

        let time_value = option.time_value().unwrap();
        assert!(time_value > Decimal::ZERO);
        assert!(time_value < option.calculate_price_black_scholes().unwrap());
    }
}

#[cfg(test)]
mod tests_valid_option {
    use super::*;
    use crate::pos;
    use rust_decimal_macros::dec;

    fn create_valid_option() -> Options {
        Options {
            option_type: OptionType::European,
            side: Side::Long,
            underlying_symbol: "AAPL".to_string(),
            strike_price: pos!(100.0),
            expiration_date: ExpirationDate::Days(pos!(30.0)),
            implied_volatility: pos!(0.2),
            quantity: Positive::ONE,
            underlying_price: pos!(105.0),
            risk_free_rate: dec!(0.05),
            option_style: OptionStyle::Call,
            dividend_yield: pos!(0.01),
            exotic_params: None,
        }
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_valid_option() {
        let option = create_valid_option();
        assert!(option.validate());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_empty_underlying_symbol() {
        let mut option = create_valid_option();
        option.underlying_symbol = "".to_string();
        assert!(!option.validate());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_zero_strike_price() {
        let mut option = create_valid_option();
        option.strike_price = Positive::ZERO;
        assert!(!option.validate());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_zero_quantity() {
        let mut option = create_valid_option();
        option.quantity = Positive::ZERO;
        assert!(!option.validate());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_zero_underlying_price() {
        let mut option = create_valid_option();
        option.underlying_price = Positive::ZERO;
        assert!(!option.validate());
    }
}

#[cfg(test)]
mod tests_time_value {
    use super::*;
    use crate::model::utils::create_sample_option_simplest_strike;
    use crate::utils::logger::setup_logger;
    use crate::{assert_decimal_eq, pos};
    use rust_decimal_macros::dec;
    use tracing::debug;

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_calculate_time_value_long_call() {
        setup_logger();
        let option =
            create_sample_option_simplest_strike(Side::Long, OptionStyle::Call, pos!(105.0));
        let time_value = option.time_value().unwrap();
        assert!(time_value > Decimal::ZERO);
        assert!(time_value <= option.calculate_price_black_scholes().unwrap());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_calculate_time_value_short_call() {
        let option =
            create_sample_option_simplest_strike(Side::Short, OptionStyle::Call, pos!(105.0));
        let time_value = option.time_value().unwrap();
        assert!(time_value > Decimal::ZERO);
        assert!(time_value <= option.calculate_price_black_scholes().unwrap().abs());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_calculate_time_value_long_put() {
        setup_logger();
        let option = create_sample_option_simplest_strike(Side::Long, OptionStyle::Put, pos!(95.0));
        let time_value = option.time_value().unwrap();
        assert!(time_value > Decimal::ZERO);
        assert!(time_value <= option.calculate_price_black_scholes().unwrap());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_calculate_time_value_short_put() {
        let option =
            create_sample_option_simplest_strike(Side::Short, OptionStyle::Put, pos!(95.0));
        let time_value = option.time_value().unwrap();
        assert!(time_value > Decimal::ZERO);
        assert!(time_value <= option.calculate_price_black_scholes().unwrap().abs());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_calculate_time_value_at_the_money() {
        let call = create_sample_option_simplest_strike(Side::Long, OptionStyle::Call, pos!(100.0));
        let put = create_sample_option_simplest_strike(Side::Long, OptionStyle::Put, pos!(100.0));

        let call_time_value = call.time_value().unwrap();
        let put_time_value = put.time_value().unwrap();

        assert!(call_time_value > Decimal::ZERO);
        assert!(put_time_value > Decimal::ZERO);
        assert_eq!(
            call_time_value,
            call.calculate_price_black_scholes().unwrap()
        );
        assert_eq!(put_time_value, put.calculate_price_black_scholes().unwrap());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_calculate_time_value_deep_in_the_money() {
        setup_logger();
        let call = create_sample_option_simplest_strike(Side::Long, OptionStyle::Call, pos!(150.0));
        let put = create_sample_option_simplest_strike(Side::Long, OptionStyle::Put, pos!(50.0));

        let call_time_value = call.time_value().unwrap();
        let put_time_value = put.time_value().unwrap();

        let call_price = call.calculate_price_black_scholes().unwrap();
        let put_price = put.calculate_price_black_scholes().unwrap();

        assert_decimal_eq!(call_time_value, call_price, dec!(0.01));
        assert_decimal_eq!(put_time_value, put_price, dec!(0.01));
        debug!("Call time value: {}", call_time_value);
        debug!("Call BS price: {}", call_price);
        debug!("Put time value: {}", put_time_value);
        debug!("Put BS price: {}", put_price);
        assert!(call_time_value <= call_price);
        assert!(put_time_value <= put_price);
    }
}

#[cfg(test)]
mod tests_options_payoffs {
    use super::*;
    use crate::model::utils::create_sample_option_simplest_strike;
    use crate::pos;
    use crate::utils::logger::setup_logger;
    use rust_decimal_macros::dec;

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_payoff_european_call_long() {
        setup_logger();
        let call_option =
            create_sample_option_simplest_strike(Side::Long, OptionStyle::Call, pos!(95.0));
        let call_payoff = call_option.payoff().unwrap();
        assert_eq!(call_payoff, dec!(5.0)); // max(100 - 95, 0) = 5

        let call_option_otm =
            create_sample_option_simplest_strike(Side::Long, OptionStyle::Call, pos!(105.0));
        let call_payoff_otm = call_option_otm.payoff().unwrap();
        assert_eq!(call_payoff_otm, Decimal::ZERO); // max(100 - 105, 0) = 0
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_payoff_european_call_short() {
        setup_logger();
        let call_option =
            create_sample_option_simplest_strike(Side::Short, OptionStyle::Call, pos!(95.0));
        let call_payoff = call_option.payoff().unwrap();
        assert_eq!(call_payoff, dec!(-5.0)); // -max(100 - 95, 0) = -5

        let call_option_otm =
            create_sample_option_simplest_strike(Side::Short, OptionStyle::Call, pos!(105.0));
        let call_payoff_otm = call_option_otm.payoff().unwrap();
        assert_eq!(call_payoff_otm, Decimal::ZERO); // -max(95 - 100, 0) = 0
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_payoff_european_put_long() {
        let put_option =
            create_sample_option_simplest_strike(Side::Long, OptionStyle::Put, pos!(105.0));
        let put_payoff = put_option.payoff().unwrap();
        assert_eq!(put_payoff, dec!(5.0)); // max(105 - 100, 0) = 5

        let put_option_otm =
            create_sample_option_simplest_strike(Side::Long, OptionStyle::Put, pos!(95.0));
        let put_payoff_otm = put_option_otm.payoff().unwrap();
        assert_eq!(put_payoff_otm, Decimal::ZERO); // max(95 - 100, 0) = 0
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_payoff_european_put_short() {
        let put_option =
            create_sample_option_simplest_strike(Side::Short, OptionStyle::Put, pos!(105.0));
        let put_payoff = put_option.payoff().unwrap();
        assert_eq!(put_payoff, dec!(-5.0)); // -max(105 - 100, 0) = -5

        let put_option_otm =
            create_sample_option_simplest_strike(Side::Short, OptionStyle::Put, pos!(95.0));
        let put_payoff_otm = put_option_otm.payoff().unwrap();
        assert_eq!(put_payoff_otm, Decimal::ZERO); // -max(95 - 100, 0) = 0
    }
}

#[cfg(test)]
mod tests_options_payoff_at_price {
    use super::*;
    use crate::model::utils::create_sample_option_simplest;
    use crate::pos;
    use rust_decimal_macros::dec;

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_payoff_european_call_long() {
        let call_option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        let call_payoff = call_option.payoff_at_price(pos!(105.0)).unwrap();
        assert_eq!(call_payoff, dec!(5.0)); // max(105 - 100, 0) = 5

        let call_option_otm = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        let call_payoff_otm = call_option_otm.payoff_at_price(pos!(95.0)).unwrap();
        assert_eq!(call_payoff_otm, Decimal::ZERO); // max(95 - 100, 0) = 0
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_payoff_european_call_short() {
        let call_option = create_sample_option_simplest(OptionStyle::Call, Side::Short);
        let call_payoff = call_option.payoff_at_price(pos!(105.0)).unwrap();
        assert_eq!(call_payoff, dec!(-5.0)); // -max(105 - 100, 0) = -5

        let call_option_otm = create_sample_option_simplest(OptionStyle::Call, Side::Short);
        let call_payoff_otm = call_option_otm.payoff_at_price(pos!(95.0)).unwrap();
        assert_eq!(call_payoff_otm, Decimal::ZERO); // -max(95 - 100, 0) = 0
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_payoff_european_put_long() {
        let put_option = create_sample_option_simplest(OptionStyle::Put, Side::Long);
        let put_payoff = put_option.payoff_at_price(pos!(95.0)).unwrap();
        assert_eq!(put_payoff, dec!(5.0)); // max(100 - 95, 0) = 5

        let put_option_otm = create_sample_option_simplest(OptionStyle::Put, Side::Long);
        let put_payoff_otm = put_option_otm.payoff_at_price(pos!(105.0)).unwrap();
        assert_eq!(put_payoff_otm, Decimal::ZERO); // max(100 - 105, 0) = 0
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_payoff_european_put_short() {
        let put_option = create_sample_option_simplest(OptionStyle::Put, Side::Short);
        let put_payoff = put_option.payoff_at_price(pos!(95.0)).unwrap();
        assert_eq!(put_payoff, dec!(-5.0)); // -max(100 - 95, 0) = -5

        let put_option_otm = create_sample_option_simplest(OptionStyle::Put, Side::Short);
        let put_payoff_otm = put_option_otm.payoff_at_price(pos!(105.0)).unwrap();
        assert_eq!(put_payoff_otm, Decimal::ZERO); // -max(100 - 105, 0) = 0
    }
}

#[cfg(test)]
mod tests_options_payoffs_with_quantity {
    use super::*;
    use crate::model::utils::create_sample_option;
    use crate::pos;
    use rust_decimal_macros::dec;

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_payoff_call_long() {
        let option = create_sample_option(
            OptionStyle::Call,
            Side::Long,
            pos!(105.0),
            pos!(10.0),
            pos!(100.0),
            pos!(0.02),
        );
        assert_eq!(option.payoff().unwrap().to_f64().unwrap(), 50.0);

        let option_otm = create_sample_option(
            OptionStyle::Call,
            Side::Long,
            pos!(95.0),
            pos!(4.0),
            pos!(100.0),
            pos!(0.02),
        );
        assert_eq!(option_otm.payoff().unwrap(), Decimal::ZERO);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_payoff_call_short() {
        let option = create_sample_option(
            OptionStyle::Call,
            Side::Short,
            pos!(105.0),
            pos!(3.0),
            pos!(100.0),
            pos!(0.02),
        );
        assert_eq!(option.payoff().unwrap().to_f64().unwrap(), -15.0);

        let option_otm = create_sample_option(
            OptionStyle::Call,
            Side::Short,
            pos!(95.0),
            pos!(7.0),
            pos!(100.0),
            pos!(0.02),
        );
        assert_eq!(option_otm.payoff().unwrap(), Decimal::ZERO);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_payoff_put_long() {
        let option = create_sample_option(
            OptionStyle::Put,
            Side::Long,
            pos!(95.0),
            pos!(2.0),
            pos!(100.0),
            pos!(0.02),
        );
        assert_eq!(option.payoff().unwrap().to_f64().unwrap(), 10.0);

        let option_otm = create_sample_option(
            OptionStyle::Put,
            Side::Long,
            pos!(105.0),
            pos!(7.0),
            pos!(100.0),
            pos!(0.02),
        );
        assert_eq!(option_otm.payoff().unwrap(), Decimal::ZERO);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_payoff_put_short() {
        let option = create_sample_option(
            OptionStyle::Put,
            Side::Short,
            pos!(95.0),
            pos!(3.0),
            pos!(100.0),
            pos!(0.02),
        );
        assert_eq!(option.payoff().unwrap().to_f64().unwrap(), -15.0);

        let option_otm = create_sample_option(
            OptionStyle::Put,
            Side::Short,
            pos!(105.0),
            pos!(3.0),
            pos!(100.0),
            pos!(0.02),
        );
        assert_eq!(option_otm.payoff().unwrap(), Decimal::ZERO);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_payoff_with_quantity() {
        let option = create_sample_option(
            OptionStyle::Call,
            Side::Long,
            pos!(110.0),
            pos!(3.0),
            pos!(100.0),
            pos!(0.02),
        );
        assert_eq!(option.payoff().unwrap().to_f64().unwrap(), 30.0); // (110 - 100) * 3
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_intrinsic_value_call_long() {
        let option = create_sample_option(
            OptionStyle::Call,
            Side::Long,
            pos!(100.0),
            pos!(11.0),
            pos!(100.0),
            pos!(0.02),
        );
        assert_eq!(option.intrinsic_value(pos!(105.0)).unwrap(), dec!(55.0));
        assert_eq!(option.intrinsic_value(pos!(95.0)).unwrap(), Decimal::ZERO);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_intrinsic_value_call_short() {
        let option = create_sample_option(
            OptionStyle::Call,
            Side::Short,
            pos!(100.0),
            pos!(13.0),
            pos!(100.0),
            pos!(0.02),
        );
        assert_eq!(option.intrinsic_value(pos!(105.0)).unwrap(), dec!(-65.0));
        assert_eq!(option.intrinsic_value(pos!(95.0)).unwrap(), Decimal::ZERO);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_intrinsic_value_put_long() {
        let option = create_sample_option(
            OptionStyle::Put,
            Side::Long,
            pos!(100.0),
            pos!(17.0),
            pos!(100.0),
            pos!(0.02),
        );
        assert_eq!(option.intrinsic_value(pos!(95.0)).unwrap(), dec!(85.0));
        assert_eq!(option.intrinsic_value(pos!(105.0)).unwrap(), Decimal::ZERO);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_intrinsic_value_put_short() {
        let option = create_sample_option(
            OptionStyle::Put,
            Side::Short,
            pos!(100.0),
            pos!(19.0),
            pos!(100.0),
            pos!(0.02),
        );
        assert_eq!(option.intrinsic_value(pos!(95.0)).unwrap(), dec!(-95.0));
        assert_eq!(option.intrinsic_value(pos!(105.0)).unwrap(), Decimal::ZERO);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_intrinsic_value_with_quantity() {
        let option = create_sample_option(
            OptionStyle::Call,
            Side::Long,
            pos!(100.0),
            pos!(23.0),
            pos!(100.0),
            pos!(0.02),
        );
        assert_eq!(option.intrinsic_value(pos!(110.0)).unwrap(), dec!(230.0)); // (110 - 100) * 23
    }
}

#[cfg(test)]
mod tests_in_the_money {
    use super::*;
    use crate::model::utils::create_sample_option;
    use crate::pos;

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_call_in_the_money() {
        let mut option = create_sample_option(
            OptionStyle::Call,
            Side::Long,
            pos!(110.0),
            pos!(1.0),
            pos!(110.0),
            pos!(0.02),
        );
        option.strike_price = pos!(100.0);
        assert!(option.is_in_the_money());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_call_at_the_money() {
        let mut option = create_sample_option(
            OptionStyle::Call,
            Side::Long,
            pos!(100.0),
            pos!(1.0),
            pos!(110.0),
            pos!(0.02),
        );
        option.strike_price = pos!(100.0);
        assert!(option.is_in_the_money());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_call_out_of_the_money() {
        let mut option = create_sample_option(
            OptionStyle::Call,
            Side::Long,
            pos!(90.0),
            pos!(1.0),
            pos!(110.0),
            pos!(0.02),
        );
        option.strike_price = pos!(100.0);
        assert!(!option.is_in_the_money());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_put_in_the_money() {
        let mut option = create_sample_option(
            OptionStyle::Put,
            Side::Long,
            pos!(90.0),
            pos!(1.0),
            pos!(110.0),
            pos!(0.02),
        );
        option.strike_price = pos!(100.0);
        assert!(option.is_in_the_money());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_put_at_the_money() {
        let mut option = create_sample_option(
            OptionStyle::Put,
            Side::Long,
            pos!(100.0),
            pos!(1.0),
            pos!(110.0),
            pos!(0.02),
        );
        option.strike_price = pos!(100.0);
        assert!(option.is_in_the_money());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_put_out_of_the_money() {
        let mut option = create_sample_option(
            OptionStyle::Put,
            Side::Long,
            pos!(110.0),
            pos!(1.0),
            pos!(110.0),
            pos!(0.02),
        );
        option.strike_price = pos!(100.0);
        assert!(!option.is_in_the_money());
    }
}

#[cfg(test)]
mod tests_greeks {
    use super::*;
    use crate::model::utils::create_sample_option_simplest;
    use crate::{assert_decimal_eq, pos};
    use rust_decimal_macros::dec;

    const EPSILON: Decimal = dec!(1e-6);

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_delta() {
        let delta = create_sample_option_simplest(OptionStyle::Call, Side::Long)
            .delta()
            .unwrap();
        assert_decimal_eq!(delta, dec!(0.539519922), EPSILON);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_delta_size() {
        let mut option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        option.quantity = pos!(2.0);
        assert_decimal_eq!(option.delta().unwrap(), dec!(1.0790398), EPSILON);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_gamma() {
        let option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        assert_decimal_eq!(option.gamma().unwrap(), dec!(0.0691707), EPSILON);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_gamma_size() {
        let mut option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        option.quantity = pos!(2.0);
        assert_decimal_eq!(option.gamma().unwrap(), dec!(0.1383415), EPSILON);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_theta() {
        let option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        assert_decimal_eq!(option.theta().unwrap(), dec!(-0.043510019), EPSILON);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_theta_size() {
        let mut option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        option.quantity = pos!(2.0);
        assert_decimal_eq!(option.theta().unwrap(), dec!(-0.0870200), EPSILON);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_vega() {
        let option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        assert_decimal_eq!(option.vega().unwrap(), dec!(0.113705366), EPSILON);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_vega_size() {
        let mut option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        option.quantity = pos!(2.0);
        assert_decimal_eq!(option.vega().unwrap(), dec!(0.2274107), EPSILON);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_rho() {
        let option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        assert_decimal_eq!(option.rho().unwrap(), dec!(0.0423312145), EPSILON);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_rho_size() {
        let mut option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        option.quantity = pos!(2.0);
        assert_decimal_eq!(option.rho().unwrap(), dec!(0.08466242), EPSILON);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_rho_d() {
        let option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        assert_decimal_eq!(option.rho_d().unwrap(), dec!(-0.04434410320), EPSILON);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_rho_d_size() {
        let mut option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        option.quantity = pos!(2.0);
        assert_decimal_eq!(option.rho_d().unwrap(), dec!(-0.0886882064063), EPSILON);
    }
}

#[cfg(test)]
mod tests_greek_trait {
    use super::*;
    use crate::assert_decimal_eq;
    use crate::model::utils::create_sample_option_simplest;
    use rust_decimal_macros::dec;

    const EPSILON: Decimal = dec!(1e-6);

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_greeks_implementation() {
        let option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        let greeks = option.greeks().unwrap();

        assert_decimal_eq!(greeks.delta, option.delta().unwrap(), EPSILON);
        assert_decimal_eq!(greeks.gamma, option.gamma().unwrap(), EPSILON);
        assert_decimal_eq!(greeks.theta, option.theta().unwrap(), EPSILON);
        assert_decimal_eq!(greeks.vega, option.vega().unwrap(), EPSILON);
        assert_decimal_eq!(greeks.rho, option.rho().unwrap(), EPSILON);
        assert_decimal_eq!(greeks.rho_d, option.rho_d().unwrap(), EPSILON);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_greeks_consistency() {
        let option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        let greeks = option.greeks().unwrap();

        assert!(
            greeks.delta >= Decimal::NEGATIVE_ONE && greeks.delta <= Decimal::ONE,
            "Delta should be between -1 and 1"
        );
        assert!(
            greeks.gamma >= Decimal::ZERO,
            "Gamma should be non-negative"
        );
        assert!(greeks.vega >= Decimal::ZERO, "Vega should be non-negative");
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_greeks_for_different_options() {
        let call_option = Options::new(
            OptionType::European,
            Side::Long,
            "TEST".to_string(),
            pos!(5790.0), // strike
            ExpirationDate::Days(pos!(18.0)),
            pos!(0.1),     // initial iv
            pos!(1.0),     // qty
            pos!(5781.88), // underlying
            dec!(0.05),    // rate
            OptionStyle::Call,
            pos!(0.0), // div
            None,
        );
        let mut put_option = call_option.clone();
        put_option.option_style = OptionStyle::Put;

        let call_greeks = call_option.greeks().unwrap();
        let put_greeks = put_option.greeks().unwrap();

        assert_decimal_eq!(
            call_greeks.delta + put_greeks.delta.abs(),
            Decimal::ONE,
            EPSILON
        );
        assert_decimal_eq!(call_greeks.gamma, put_greeks.gamma, EPSILON);
        assert_decimal_eq!(call_greeks.vega, put_greeks.vega, EPSILON);
    }
}

#[cfg(test)]
mod tests_graph {
    use super::*;
    use crate::model::utils::create_sample_option_simplest;
    use crate::pos;
    use crate::visualization::utils::Graph;
    use approx::assert_relative_eq;

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_title() {
        let option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        let expected_title = "Underlying: AAPL @ $100 Long Call European Option".to_string();
        assert_eq!(option.title(), expected_title);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_get_values() {
        let option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        let prices = vec![pos!(90.0), pos!(100.0), pos!(110.0)];
        let values = option.get_values(&prices);

        assert_eq!(values.len(), 3);
        assert_relative_eq!(values[0], 0.0, epsilon = 1e-6);
        assert_relative_eq!(values[1], 0.0, epsilon = 1e-6);
        assert_relative_eq!(values[2], 10.0, epsilon = 1e-6);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_get_vertical_lines() {
        let option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        let vertical_lines = option.get_vertical_lines();

        assert_eq!(vertical_lines.len(), 1);
        assert_eq!(vertical_lines[0].label, "Strike");
        assert_relative_eq!(vertical_lines[0].x_coordinate, 100.0, epsilon = 1e-6);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_title_put_option() {
        let option = create_sample_option_simplest(OptionStyle::Put, Side::Long);
        let expected_title = "Underlying: AAPL @ $100 Long Put European Option".to_string();
        assert_eq!(option.title(), expected_title);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_get_values_put_option() {
        let option = create_sample_option_simplest(OptionStyle::Put, Side::Long);
        let prices = vec![pos!(90.0), pos!(100.0), pos!(110.0)];
        let values = option.get_values(&prices);

        assert_eq!(values.len(), 3);
        assert_relative_eq!(values[0], 10.0, epsilon = 1e-6);
        assert_relative_eq!(values[1], 0.0, epsilon = 1e-6);
        assert_relative_eq!(values[2], 0.0, epsilon = 1e-6);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_get_values_short_option() {
        let option = create_sample_option_simplest(OptionStyle::Call, Side::Short);
        let prices = vec![pos!(90.0), pos!(100.0), pos!(110.0)];
        let values = option.get_values(&prices);

        assert_eq!(values.len(), 3);
        assert_relative_eq!(values[0], 0.0, epsilon = 1e-6);
        assert_relative_eq!(values[1], 0.0, epsilon = 1e-6);
        assert_relative_eq!(values[2], -10.0, epsilon = 1e-6);
    }
}

#[cfg(test)]
mod tests_calculate_price_binomial {
    use super::*;
    use crate::model::utils::{
        create_sample_option, create_sample_option_simplest, create_sample_option_with_date,
    };
    use crate::pos;
    use chrono::Utc;
    use rust_decimal_macros::dec;
    use std::str::FromStr;

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_european_call_option_basic() {
        // Test a basic European call option with standard parameters
        let option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        let result = option.calculate_price_binomial(100);
        assert!(result.is_ok());
        let price = result.unwrap();
        // Price should be positive for a long call at-the-money
        assert!(price > Decimal::ZERO);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_american_put_option() {
        // Test American put option which should have early exercise value
        let option = Options::new(
            OptionType::American,
            Side::Long,
            "TEST".to_string(),
            pos!(100.0),
            ExpirationDate::Days(pos!(30.0)),
            pos!(0.2),  // volatility
            pos!(1.0),  // quantity
            pos!(95.0), // underlying price (slightly ITM for put)
            dec!(0.05), // risk-free rate
            OptionStyle::Put,
            Positive::ZERO, // dividend yield
            None,
        );

        let result = option.calculate_price_binomial(100);
        assert!(result.is_ok());
        let price = result.unwrap();
        // Price should be positive and reflect early exercise premium
        assert!(price > Decimal::ZERO);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_zero_volatility() {
        let mut option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        option.implied_volatility = Positive::ZERO;
        let result = option.calculate_price_binomial(100);
        assert!(result.is_ok());
        // With zero volatility, price should equal discounted intrinsic value
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_zero_time_to_expiry() {
        // Test option at expiration
        let now = Utc::now().naive_utc();
        let option = create_sample_option_with_date(
            OptionStyle::Call,
            Side::Long,
            pos!(100.0),
            pos!(1.0),
            pos!(95.0),
            pos!(0.2),
            now,
        );

        let result = option.calculate_price_binomial(100);
        assert!(result.is_ok());
        let price = result.unwrap();
        // At expiry, price should equal intrinsic value
        assert_eq!(price, Decimal::from(5));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_invalid_steps() {
        let option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        let result = option.calculate_price_binomial(0);
        assert!(result.is_err());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_deep_itm_call() {
        let option = create_sample_option(
            OptionStyle::Call,
            Side::Long,
            pos!(150.0), // Underlying price much higher than strike
            pos!(1.0),
            pos!(100.0),
            pos!(0.2),
        );

        let result = option.calculate_price_binomial(100);
        assert!(result.is_ok());
        let price = result.unwrap();
        // Price should be close to intrinsic value for deep ITM
        assert!(price > Decimal::from(45)); // At least intrinsic - some time value
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_deep_otm_put() {
        let option = create_sample_option(
            OptionStyle::Put,
            Side::Long,
            pos!(150.0), // Underlying price much higher than strike
            pos!(1.0),
            pos!(100.0),
            pos!(0.2),
        );

        let result = option.calculate_price_binomial(100);
        assert!(result.is_ok());
        let price = result.unwrap();
        // Price should be very small for deep OTM
        assert!(price < Decimal::from(1));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_convergence() {
        let option = create_sample_option_simplest(OptionStyle::Call, Side::Long);

        // Test that increasing steps leads to convergence
        let price_100 = option.calculate_price_binomial(100).unwrap();
        let price_1000 = option.calculate_price_binomial(1000).unwrap();

        // Prices should be close to each other
        let diff = (price_1000 - price_100).abs();
        assert!(diff < Decimal::from_str("0.1").unwrap());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_short_position() {
        let long_call_option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        let mut short_call_option = long_call_option.clone();
        short_call_option.side = Side::Short;
        let mut short_put_option = short_call_option.clone();
        short_put_option.option_style = OptionStyle::Put;
        let mut long_put_option = short_put_option.clone();
        long_put_option.side = Side::Long;

        let long_call_price = long_call_option.calculate_price_binomial(100).unwrap();
        let short_call_price = short_call_option.calculate_price_binomial(100).unwrap();
        let long_put_price = long_put_option.calculate_price_binomial(100).unwrap();
        let short_put_price = short_put_option.calculate_price_binomial(100).unwrap();

        // Short position should be negative of long position
        assert_eq!(long_call_price, -short_call_price);
        assert_eq!(long_put_price, -short_put_price);
    }
}

#[cfg(test)]
mod tests_options_black_scholes {
    use super::*;
    use crate::{assert_decimal_eq, pos};
    use rust_decimal_macros::dec;

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_new_option_call() {
        let option = Options::new(
            OptionType::European,
            Side::Long,
            "SP500".to_string(),
            pos!(5790.0),
            ExpirationDate::Days(pos!(18.0)),
            pos!(0.1117),
            pos!(1.0),
            pos!(5781.88),
            dec!(0.05),
            OptionStyle::Call,
            pos!(0.0),
            None,
        );
        assert_decimal_eq!(
            option.calculate_price_black_scholes().unwrap(),
            pos!(60.306_765_882_668_3),
            dec!(1e-8)
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_new_option_call_bis() {
        let option = Options::new(
            OptionType::European,
            Side::Long,
            "SP500".to_string(),
            pos!(6050.0),
            ExpirationDate::Days(pos!(61.2)),
            pos!(0.12594),
            pos!(1.0),
            pos!(6032.18),
            dec!(0.0),
            OptionStyle::Call,
            pos!(0.0),
            None,
        );
        assert_decimal_eq!(
            option.calculate_price_black_scholes().unwrap(),
            pos!(115.56),
            dec!(1e-2)
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_new_option_put() {
        let option = Options::new(
            OptionType::European,
            Side::Long,
            "SP500".to_string(),
            pos!(6050.0),
            ExpirationDate::Days(pos!(61.2)),
            pos!(0.1258),
            pos!(1.0),
            pos!(6032.18),
            dec!(0.0),
            OptionStyle::Put,
            pos!(0.0),
            None,
        );
        assert_decimal_eq!(
            option.calculate_price_black_scholes().unwrap(),
            pos!(133.25),
            dec!(1e-2)
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_new_option_call_short() {
        let option = Options::new(
            OptionType::European,
            Side::Short,
            "SP500".to_string(),
            pos!(6050.0),
            ExpirationDate::Days(pos!(60.0)),
            pos!(0.12594),
            pos!(1.0),
            pos!(6032.18),
            dec!(0.0),
            OptionStyle::Call,
            pos!(0.0),
            None,
        );
        assert_decimal_eq!(
            option.calculate_price_black_scholes().unwrap(),
            dec!(-114.34),
            dec!(1e-2)
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_new_option_put_short() {
        let option = Options::new(
            OptionType::European,
            Side::Short,
            "SP500".to_string(),
            pos!(6050.0),
            ExpirationDate::Days(pos!(60.0)),
            pos!(0.12594),
            pos!(1.0),
            pos!(6032.18),
            dec!(0.0),
            OptionStyle::Put,
            pos!(0.0),
            None,
        );
        assert_decimal_eq!(
            option.calculate_price_black_scholes().unwrap(),
            dec!(-132.16),
            dec!(1e-2)
        );
    }
}

#[cfg(test)]
mod tests_calculate_implied_volatility {
    use super::*;
    use crate::error::VolatilityError;
    use crate::{assert_pos_relative_eq, pos};
    use rust_decimal_macros::dec;

    #[test]
    fn test_implied_volatility_call() {
        let option = Options::new(
            OptionType::European,
            Side::Long,
            "TEST".to_string(),
            pos!(5790.0), // strike
            ExpirationDate::Days(pos!(18.0)),
            pos!(0.1),     // initial iv
            pos!(1.0),     // qty
            pos!(5781.88), // underlying
            dec!(0.05),    // rate
            OptionStyle::Call,
            pos!(0.0), // div
            None,
        );

        let market_price = dec!(60.30);
        let iv = option.calculate_implied_volatility(market_price).unwrap();

        assert_pos_relative_eq!(iv, pos!(0.111618041), Positive(IV_TOLERANCE));
    }

    #[test]
    fn test_implied_volatility_put() {
        let option = Options::new(
            OptionType::European,
            Side::Long,
            "TEST".to_string(),
            pos!(6050.0), // strike
            ExpirationDate::Days(pos!(60.0)),
            pos!(0.1),     // initial iv
            pos!(1.0),     // qty
            pos!(6032.18), // underlying
            dec!(0.0),     // rate
            OptionStyle::Put,
            pos!(0.0), // div
            None,
        );

        let market_price = dec!(132.16);
        let iv = option.calculate_implied_volatility(market_price).unwrap();
        assert_pos_relative_eq!(iv, pos!(0.125961), Positive(IV_TOLERANCE));
    }

    #[test]
    fn test_implied_volatility_call_short() {
        let option = Options::new(
            OptionType::European,
            Side::Short,
            "TEST".to_string(),
            pos!(6050.0), // strike
            ExpirationDate::Days(pos!(60.0)),
            pos!(0.1),     // initial iv
            pos!(1.0),     // qty
            pos!(6032.18), // underlying
            dec!(0.0),     // rate
            OptionStyle::Call,
            pos!(0.0), // div
            None,
        );

        let market_price = dec!(-114.16);
        let iv = option.calculate_implied_volatility(market_price).unwrap();

        assert_pos_relative_eq!(iv, pos!(0.1258087), Positive(IV_TOLERANCE));
    }

    #[test]
    fn test_implied_volatility_put_short() {
        let option = Options::new(
            OptionType::European,
            Side::Short,
            "TEST".to_string(),
            pos!(6050.0), // strike
            ExpirationDate::Days(pos!(60.0)),
            pos!(0.1),     // initial iv
            pos!(1.0),     // qty
            pos!(6032.18), // underlying
            dec!(0.0),     // rate
            OptionStyle::Put,
            pos!(0.0), // div
            None,
        );

        let market_price = dec!(-132.27);
        let iv = option.calculate_implied_volatility(market_price).unwrap();
        assert_pos_relative_eq!(iv, pos!(0.12611389), Positive(IV_TOLERANCE));
    }

    #[test]
    fn test_invalid_market_price() {
        let option = Options::default();
        let result = option.calculate_implied_volatility(Decimal::ZERO);
        assert!(matches!(result, Err(VolatilityError::OptionError { .. })));
    }

    #[test]
    fn test_expired_option() {
        let option = Options::new(
            OptionType::European,
            Side::Long,
            "TEST".to_string(),
            pos!(100.0),
            ExpirationDate::Days(Positive::ZERO),
            pos!(0.2),
            pos!(1.0),
            pos!(100.0),
            dec!(0.05),
            OptionStyle::Call,
            pos!(0.0),
            None,
        );

        let result = option.calculate_implied_volatility(dec!(2.5));
        assert!(matches!(result, Err(VolatilityError::OptionError { .. })));
    }

    #[test]
    fn test_convergence_edge_cases() {
        let option = Options::new(
            OptionType::European,
            Side::Long,
            "TEST".to_string(),
            pos!(5790.0), // strike
            ExpirationDate::Days(pos!(18.0)),
            pos!(0.1),     // initial iv
            pos!(1.0),     // qty
            pos!(5781.88), // underlying
            dec!(0.05),    // rate
            OptionStyle::Call,
            pos!(0.0), // div
            None,
        );

        // Test with small initial vol
        let iv = option.calculate_implied_volatility(dec!(60.30)).unwrap();
        assert_pos_relative_eq!(iv, pos!(0.111328125), pos!(0.01));

        // Test with large initial vol
        let iv = option.calculate_implied_volatility(dec!(60.30)).unwrap();
        assert_pos_relative_eq!(iv, pos!(0.111328125), pos!(0.01));
    }
}

#[cfg(test)]
mod tests_serialize_deserialize {
    use super::*;
    use crate::model::utils::create_sample_option_simplest_strike;

    #[test]
    fn test_serialize_deserialize_options() {
        let options =
            create_sample_option_simplest_strike(Side::Long, OptionStyle::Call, pos!(95.0));
        let serialized = serde_json::to_string(&options).expect("Failed to serialize");
        let deserialized: Options =
            serde_json::from_str(&serialized).expect("Failed to deserialize");
        assert_eq!(options, deserialized);
    }
}
