/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 11/8/24
******************************************************************************/
use crate::constants::ZERO;
use crate::error::greeks::GreeksError;
use crate::greeks::utils::{big_n, d1, d2, n};
use crate::model::types::OptionStyle;
use crate::{Options, Positive};
use rust_decimal::{Decimal, MathematicalOps};

#[derive(Debug, PartialEq)]
pub struct Greek {
    pub delta: Decimal,
    pub gamma: Decimal,
    pub theta: Decimal,
    pub vega: Decimal,
    pub rho: Decimal,
    pub rho_d: Decimal,
    pub alpha: Decimal,
}

pub trait Greeks {
    fn get_options(&self) -> Result<Vec<&Options>, GreeksError>;

    fn greeks(&self) -> Result<Greek, GreeksError> {
        let delta = self.delta()?;
        let gamma = self.gamma()?;
        let theta = self.theta()?;
        let vega = self.vega()?;
        let rho = self.rho()?;
        let rho_d = self.rho_d()?;
        let alpha = self.alpha()?;
        Ok(Greek {
            delta,
            gamma,
            theta,
            vega,
            rho,
            rho_d,
            alpha,
        })
    }

    fn delta(&self) -> Result<Decimal, GreeksError> {
        let options = self.get_options()?;
        let mut delta_value = Decimal::ZERO;
        for option in options {
            delta_value += delta(option)?;
        }
        Ok(delta_value)
    }

    fn gamma(&self) -> Result<Decimal, GreeksError> {
        let options = self.get_options()?;
        let mut gamma_value = Decimal::ZERO;
        for option in options {
            gamma_value += gamma(option)?;
        }
        Ok(gamma_value)
    }

    fn theta(&self) -> Result<Decimal, GreeksError> {
        let options = self.get_options()?;
        let mut theta_value = Decimal::ZERO;
        for option in options {
            theta_value += theta(option)?;
        }
        Ok(theta_value)
    }

    fn vega(&self) -> Result<Decimal, GreeksError> {
        let options = self.get_options()?;
        let mut vega_value = Decimal::ZERO;
        for option in options {
            vega_value += vega(option)?;
        }
        Ok(vega_value)
    }

    fn rho(&self) -> Result<Decimal, GreeksError> {
        let options = self.get_options()?;
        let mut rho_value = Decimal::ZERO;
        for option in options {
            rho_value += rho(option)?;
        }
        Ok(rho_value)
    }

    fn rho_d(&self) -> Result<Decimal, GreeksError> {
        let options = self.get_options()?;
        let mut rho_d_value = Decimal::ZERO;
        for option in options {
            rho_d_value += rho_d(option)?;
        }
        Ok(rho_d_value)
    }

    fn alpha(&self) -> Result<Decimal, GreeksError> {
        let options = self.get_options()?;
        let mut alpha_value = Decimal::ZERO;
        for option in options {
            alpha_value += alpha(option)?;
        }
        Ok(alpha_value)
    }
}

/// Calculates the delta of an option.
///
/// The delta measures the sensitivity of an option's price to changes in the price of the
/// underlying asset. It is calculated differently for call and put options. For options
/// with zero implied volatility, the delta is determined based on whether the option is
/// in-the-money or out-of-the-money.
///
/// # Parameters
///
/// - `option: &Options`
///   A reference to an `Options` struct containing all the relevant parameters for the calculation:
///   - `underlying_price`: The current price of the underlying asset.
///   - `strike_price`: The strike price of the option.
///   - `risk_free_rate`: The annualized risk-free interest rate.
///   - `expiration_date`: The time to expiration of the option, in years.
///   - `implied_volatility`: The implied volatility of the option.
///   - `dividend_yield`: The dividend yield of the underlying asset.
///   - `quantity`: The quantity of the options.
///   - `option_style`: The style of the option (Call or Put).
///
/// # Returns
///
/// - `Ok(Decimal)`: The calculated delta value.
/// - `Err(GreeksError)`: Returns an error if any intermediate calculations fail.
///
/// # Calculation Details
///
/// - If `implied_volatility == 0`, the delta is determined based on whether the option is
///   in-the-money or out-of-the-money:
///   - Call Option:
///     - In-the-money: Delta = `sign`
///     - Out-of-the-money: Delta = 0
///   - Put Option:
///     - In-the-money: Delta = `-sign`
///     - Out-of-the-money: Delta = 0
/// - For options with non-zero implied volatility, the delta is calculated as:
///   - Call Option:
///     \[ \Delta_{\text{call}} = \text{sign} \cdot N(d1) \cdot e^{-qT} \]
///   - Put Option:
///     \[ \Delta_{\text{put}} = \text{sign} \cdot (N(d1) - 1) \cdot e^{-qT} \]
///     Where:
///     - \(N(d1)\): The cumulative distribution function (CDF) of the standard normal distribution evaluated at \(d1\).
///     - \(q\): The dividend yield.
///     - \(T\): Time to expiration.
///
/// - The delta is adjusted by multiplying it by the option quantity.
///
/// # Errors
///
/// - `GreeksError`: If the calculation of \(d1\) or the standard normal CDF (`big_n`) fails.
///
/// # Example
///
/// ```rust
/// use rust_decimal::Decimal;
/// use rust_decimal_macros::dec;
/// use tracing::{error, info};
/// use optionstratlib::constants::ZERO;
/// use optionstratlib::greeks::delta;
/// use optionstratlib::Options;
/// use optionstratlib::model::types::{ExpirationDate, OptionStyle, OptionType, Side};
/// use optionstratlib::{pos, Positive};
/// let option = Options {
///     option_type: OptionType::European,side:
///     Side::Long,underlying_price:
///     pos!(100.0),
///     strike_price: pos!(95.0),
///     risk_free_rate: dec!(0.05),
///     expiration_date: ExpirationDate::Days(pos!(30.0)),
///     implied_volatility: pos!(0.2),
///     dividend_yield: Positive::ZERO,
///     quantity: pos!(1.0),
///     option_style: OptionStyle::Call,
///     underlying_symbol: "AAPL".to_string(),
///     exotic_params: None,
/// };
///
/// match delta(&option) {
///     Ok(result) => info!("Delta: {}", result),
///     Err(e) => error!("Error calculating delta: {:?}", e),
/// }
/// ```
pub fn delta(option: &Options) -> Result<Decimal, GreeksError> {
    let dividend_yield: Positive = option.dividend_yield;

    let sign = if option.is_long() {
        Decimal::ONE
    } else {
        Decimal::NEGATIVE_ONE
    };
    if option.implied_volatility == ZERO {
        return match option.option_style {
            OptionStyle::Call => {
                if option.underlying_price >= option.strike_price {
                    Ok(sign) // Delta is 1 for Call in-the-money
                } else {
                    Ok(Decimal::ZERO) // Delta is 0 for Call out-of-the-money
                }
            }
            OptionStyle::Put => {
                if option.underlying_price <= option.strike_price {
                    Ok(sign * Decimal::NEGATIVE_ONE) // Delta is -1 for Put in-the-money
                } else {
                    Ok(Decimal::ZERO) // Delta is 0 for Put out-of-the-money
                }
            }
        };
    }

    let d1 = d1(
        option.underlying_price,
        option.strike_price,
        option.risk_free_rate,
        option.expiration_date.get_years().unwrap(),
        option.implied_volatility,
    )?;

    let expiration_date = option.expiration_date.get_years()?;
    let div_date = (-expiration_date.to_dec() * dividend_yield).exp();

    let delta = match option.option_style {
        OptionStyle::Call => sign * big_n(d1)? * div_date,
        OptionStyle::Put => sign * (big_n(d1)? - Decimal::ONE) * div_date,
    };
    let delta: Decimal = delta.clamp(Decimal::NEGATIVE_ONE, Decimal::ONE);
    let quantity: Decimal = option.quantity.into();
    Ok(delta * quantity)
}

/// Computes the gamma of an option.
///
/// Gamma measures the rate of change of the option's delta with respect to changes in the underlying
/// asset's price. It is a second-order derivative of the option price and provides insight into the
/// sensitivity of delta to movements in the underlying price.
///
/// # Parameters
///
/// - `option: &Options`
///   A reference to an `Options` struct containing the following relevant parameters:
///   - `underlying_price`: The current price of the underlying asset.
///   - `strike_price`: The strike price of the option.
///   - `risk_free_rate`: The risk-free interest rate.
///   - `expiration_date`: The time to expiration in years.
///   - `implied_volatility`: The implied volatility of the option.
///   - `dividend_yield`: The dividend yield of the underlying asset.
///   - `quantity`: The quantity of the options.
///
/// # Returns
///
/// - `Ok(Decimal)`: The calculated gamma value.
/// - `Err(GreeksError)`: Returns an error if the computation of `d1` or the probability density function `n(d1)` fails.
///
/// # Calculation
///
/// Gamma is calculated using the formula:
///
/// ```math
/// \Gamma = \frac{e^{-qT} \cdot N'(d1)}{S \cdot \sigma \cdot \sqrt{T}}
/// ```
///
/// Where:
/// - \(N'(d1)\): The standard normal probability density function (PDF) evaluated at \(d1\).
/// - \(S\): The price of the underlying asset.
/// - \(\sigma\): The implied volatility of the option.
/// - \(T\): The time to expiration in years.
/// - \(q\): The dividend yield of the underlying asset.
///
/// ### Steps:
/// 1. Compute \(d1\) using the `d1` function.
/// 2. Evaluate \(N'(d1)\) using the `n` function.
/// 3. Apply the gamma formula, accounting for the effect of the dividend yield \(e^{-qT}\).
/// 4. Multiply the result by the option's quantity.
///
/// # Edge Cases
///
/// - If the implied volatility (\(\sigma\)) is zero, gamma is returned as `0`.
///
/// # Example
///
/// ```rust
/// use rust_decimal_macros::dec;
/// use tracing::{error, info};
/// use optionstratlib::greeks::gamma;
/// use optionstratlib::Options;
/// use optionstratlib::model::types::{ExpirationDate, OptionStyle, OptionType, Side};
/// use optionstratlib::pos;
/// let option = Options {
///     option_type: OptionType::European,
///     side: Side::Long,
///     underlying_price: pos!(100.0),
///     strike_price: pos!(95.0),
///     risk_free_rate: dec!(0.05),
///     expiration_date: ExpirationDate::Days(pos!(30.0)),
///     implied_volatility: pos!(0.2),
///     dividend_yield: pos!(0.01),
///     quantity: pos!(1.0),
///     option_style: OptionStyle::Call,
///     underlying_symbol: "".to_string(),
///     exotic_params: None,
/// };
///
/// match gamma(&option) {
///     Ok(result) => info!("Gamma: {}", result),
///     Err(e) => error!("Error calculating gamma: {:?}", e),
/// }
/// ```
///
/// # Notes
///
/// - This function assumes that the dividend yield \(q\) and the time to expiration \(T\) are
///   provided in consistent units.
/// - If the implied volatility or time to expiration is very small, the result may be close to 0,
///   as gamma becomes negligible in those cases.
pub fn gamma(option: &Options) -> Result<Decimal, GreeksError> {
    if option.implied_volatility == ZERO {
        return Ok(Decimal::ZERO);
    }

    let d1 = d1(
        option.underlying_price,
        option.strike_price,
        option.risk_free_rate,
        option.expiration_date.get_years().unwrap(),
        option.implied_volatility,
    )?;

    let expiration_date: Positive = option.expiration_date.get_years()?;
    let dividend_yield: Decimal = option.dividend_yield.into();
    let underlying_price: Decimal = option.underlying_price.into();
    let implied_volatility: Positive = option.implied_volatility;

    let gamma: Decimal = (expiration_date.to_dec() * -dividend_yield).exp() * n(d1)?
        / (underlying_price * implied_volatility * expiration_date.sqrt().to_dec());

    let quantity: Decimal = option.quantity.into();
    Ok(gamma * quantity)
}

/// Computes the Theta of an option.
///
/// Theta measures the sensitivity of the option's price to time decay, indicating the rate
/// at which the value of the option decreases as the expiration date approaches. This is
/// particularly important in options trading, as Theta reflects the "time decay" of the
/// option's extrinsic value.
///
/// # Parameters
///
/// - `option: &Options`
///   A reference to an `Options` struct containing the following relevant parameters:
///   - `underlying_price`: The current price of the underlying asset.
///   - `strike_price`: The strike price of the option.
///   - `risk_free_rate`: The risk-free interest rate.
///   - `expiration_date`: The time to expiration in years (provides `get_years` method).
///   - `implied_volatility`: The implied volatility of the option.
///   - `dividend_yield`: The dividend yield of the underlying asset.
///   - `option_style`: The style of the option (Call or Put).
///   - `quantity`: The quantity of the options.
///
/// # Returns
///
/// - `Ok(Decimal)`: The calculated Theta value for the option.
/// - `Err(GreeksError)`: Returns an error if any intermediate calculation fails (e.g., in `d1`, `d2`, or `n`).
///
/// # Formula
///
/// The Theta is calculated using the Black-Scholes model. The formula differs for call and put options:
///
/// **Call Options:**
///
/// ```math
/// \Theta_{\text{call}} =
/// -\frac{S \cdot \sigma \cdot e^{-qT} \cdot n(d1)}{2 \sqrt{T}}
/// - r \cdot K \cdot e^{-rT} \cdot N(d2)
/// + q \cdot S \cdot e^{-qT} \cdot N(d1)
/// ```
///
/// **Put Options:**
///
/// ```math
/// \Theta_{\text{put}} =
/// -\frac{S \cdot \sigma \cdot e^{-qT} \cdot n(d1)}{2 \sqrt{T}}
/// + r \cdot K \cdot e^{-rT} \cdot N(-d2)
/// - q \cdot S \cdot e^{-qT} \cdot N(-d1)
/// ```
///
/// Where:
/// - \( S \): Underlying price
/// - \( \sigma \): Implied volatility
/// - \( T \): Time to expiration (in years)
/// - \( r \): Risk-free rate
/// - \( q \): Dividend yield
/// - \( K \): Strike price
/// - \( N(d1) \): Cumulative distribution function (CDF) of the standard normal distribution at \( d1 \).
/// - \( n(d1) \): Probability density function (PDF) of the standard normal distribution at \( d1 \).
///
/// # Calculation Steps
/// 1. Compute \( d1 \) and \( d2 \) using the `d1` and `d2` functions.
/// 2. Calculate the common term:
///    ```math
///    \text{common\_term} = -\frac{S \cdot \sigma \cdot e^{-qT} \cdot n(d1)}{2 \sqrt{T}}
///    ```
/// 3. Apply the corresponding formula for Call or Put options, accounting for the effect of
///    dividends (\( e^{-qT} \)) and risk-free rate (\( e^{-rT} \)).
/// 4. Multiply the resulting Theta by the quantity of options.
///
/// # Example
///
/// ```rust
/// use rust_decimal_macros::dec;
/// use tracing::{error, info};
/// use optionstratlib::greeks::theta;
/// use optionstratlib::Options;
/// use optionstratlib::model::types::{ExpirationDate, OptionStyle, OptionType, Side};
/// use optionstratlib::pos;
/// let option = Options {
///     option_type: OptionType::European,
///     side: Side::Long,
///     underlying_price: pos!(100.0),
///     strike_price: pos!(95.0),
///     risk_free_rate: dec!(0.05),
///     expiration_date: ExpirationDate::Days(pos!(30.0)),
///     implied_volatility: pos!(0.2),
///     dividend_yield: pos!(0.01),
///     quantity: pos!(1.0),
///     option_style: OptionStyle::Call,
///     underlying_symbol: "".to_string(),
///     exotic_params: None,
/// };
///
/// match theta(&option) {
///     Ok(result) => info!("Theta: {}", result),
///     Err(e) => error!("Error calculating Theta: {:?}", e),
/// }
/// ```
///
/// # Notes
///
/// - A positive Theta means the option gains value as time passes (rare and usually for short positions).
/// - A negative Theta is typical for long positions, as the option loses extrinsic value over time.
/// - If the implied volatility is zero, Theta may be close to zero for far-out-of-the-money options.
pub fn theta(option: &Options) -> Result<Decimal, GreeksError> {
    let t = option.expiration_date.get_years()?;
    if t == Decimal::ZERO {
        return Ok(Decimal::ZERO);
    }

    let d1 = d1(
        option.underlying_price,
        option.strike_price,
        option.risk_free_rate,
        t,
        option.implied_volatility,
    )?;
    let d2 = d2(
        option.underlying_price,
        option.strike_price,
        option.risk_free_rate,
        t,
        option.implied_volatility,
    )?;

    let s = option.underlying_price.to_dec();
    let k = option.strike_price.to_dec();
    let r = option.risk_free_rate;
    let q = option.dividend_yield.to_dec();
    let sigma = option.implied_volatility.to_dec();

    // Common term using n
    let common_term = -(s * n(d1)? * sigma) / (Decimal::TWO * t.sqrt());

    // Pre-calculate discount factors
    let exp_minus_rt = (-r * t).exp();
    let exp_minus_qt = (-q * t).exp();

    let theta = match option.option_style {
        OptionStyle::Call => {
            common_term - r * k * exp_minus_rt * big_n(d2)? + q * s * exp_minus_qt * big_n(d1)?
        }
        OptionStyle::Put => {
            common_term + r * k * exp_minus_rt * big_n(-d2)? - q * s * exp_minus_qt * big_n(-d1)?
        }
    };

    // Adjust for quantity and convert to daily value
    Ok((theta * option.quantity.to_dec()) / Decimal::from(365))
}

/// Computes the vega of an option.
///
/// Vega measures the sensitivity of the option's price to changes in the implied volatility
/// of the underlying asset. It quantifies the expected change in the option's price for a
/// 1% change in the implied volatility. Vega is particularly important for understanding
/// how an option's value is affected by market conditions that alter volatility.
///
/// # Parameters
///
/// - `option: &Options`
///   A reference to an `Options` struct containing the necessary parameters:
///   - `underlying_price`: The current price of the underlying asset.
///   - `strike_price`: The strike price of the option.
///   - `risk_free_rate`: The annualized risk-free interest rate.
///   - `expiration_date`: The time to expiration in years (provides `get_years` method).
///   - `implied_volatility`: The implied volatility of the option.
///   - `dividend_yield`: The dividend yield of the underlying asset.
///   - `quantity`: The quantity of the options.
///   - `option_style`: The style of the option (e.g., European).
///
/// # Returns
///
/// - `Ok(Decimal)`: The computed vega value of the option.
/// - `Err(GreeksError)`: Returns an error if any intermediate calculation fails (e.g., in `d1` or `big_n`).
///
/// # Formula
///
/// Vega is computed using the Black-Scholes model formula:
///
/// ```math
/// \text{Vega} = S \cdot e^{-qT} \cdot n(d1) \cdot \sqrt{T}
/// ```
///
/// Where:
/// - \( S \): The price of the underlying asset.
/// - \( q \): The dividend yield of the underlying asset.
/// - \( T \): Time to expiration in years.
/// - \( n(d1) \): The probability density function (PDF) of the standard normal distribution at \( d1 \).
/// - \( d1 \): A parameter calculated using the Black-Scholes model.
///
/// # Calculation Steps
///
/// 1. Compute \( d1 \) using the `d1` function.
/// 2. Calculate the exponential factor \( e^{-qT} \), which accounts for the effect of dividends.
/// 3. Evaluate \( n(d1) \), the PDF of the standard normal distribution at \( d1 \).
/// 4. Multiply the underlying price, the exponential factor, \( n(d1) \), and the square root of time to expiration.
/// 5. Multiply the result by the quantity of options to adjust for position size.
///
/// # Example
///
/// ```rust
/// use rust_decimal_macros::dec;
/// use tracing::{error, info};
/// use optionstratlib::greeks::vega;
/// use optionstratlib::Options;
/// use optionstratlib::model::types::{ExpirationDate, OptionStyle, OptionType, Side};
/// use optionstratlib::pos;
///
/// let option = Options {
///     option_type: OptionType::European,
///     side: Side::Long,
///     underlying_price: pos!(100.0),
///     strike_price: pos!(95.0),
///     risk_free_rate: dec!(0.05),
///     expiration_date: ExpirationDate::Days(pos!(30.0)),
///     implied_volatility: pos!(0.2),
///     dividend_yield: pos!(0.01),
///     quantity: pos!(1.0),
///     option_style: OptionStyle::Call,
///     underlying_symbol: "".to_string(),
///     exotic_params: None,
/// };
///
/// match vega(&option) {
///     Ok(result) => info!("Vega: {}", result),
///     Err(e) => error!("Error calculating Vega: {:?}", e),
/// }
/// ```
///
/// # Notes
///
/// - Vega is usually highest for at-the-money options and decreases as the option moves deeper
///   in-the-money or out-of-the-money.
/// - For shorter time to expiration, Vega is smaller as the sensitivity to volatility diminishes.
/// - A positive Vega indicates that an increase in implied volatility will increase the option's value.
pub fn vega(option: &Options) -> Result<Decimal, GreeksError> {
    let d1 = d1(
        option.underlying_price,
        option.strike_price,
        option.risk_free_rate,
        option.expiration_date.get_years().unwrap(),
        option.implied_volatility,
    )?;

    let expiration_date: Positive = option.expiration_date.get_years()?;
    let dividend_yield: Positive = option.dividend_yield;
    let underlying_price: Decimal = option.underlying_price.to_dec();

    let vega: Decimal = underlying_price
        * (-expiration_date.to_dec() * dividend_yield).exp()
        * n(d1)?
        * expiration_date.sqrt()
        / Decimal::ONE_HUNDRED; // percentage of change in volatility

    let quantity: Decimal = option.quantity.into();
    Ok(vega * quantity)
}

/// Computes the rho of an options contract.
///
/// Rho measures the sensitivity of the option's price to changes in the risk-free interest rate.
/// It quantifies the expected change in the option's price for a 1% change in the risk-free rate.
/// This metric is useful for understanding how interest rate fluctuations affect the value of
/// options contracts.
///
/// # Parameters
///
/// - `option: &Options`
///   A reference to an `Options` struct containing the following fields:
///   - `underlying_price`: The current price of the underlying asset.
///   - `strike_price`: The strike price of the option.
///   - `risk_free_rate`: The annualized risk-free interest rate.
///   - `expiration_date`: The time to expiration in years (provides `get_years` method).
///   - `implied_volatility`: The implied volatility of the option.
///   - `option_style`: The style of the option (`Call` or `Put`).
///   - `quantity`: The quantity of the options.
///
/// # Returns
///
/// - `Ok(Decimal)`: The computed rho value for the options contract.
/// - `Err(GreeksError)`: Returns an error if any intermediate calculation fails (e.g., in `d2` or `big_n`).
///
/// # Formula
///
/// The rho is calculated differently for Call and Put options, as follows:
///
/// **Call Options:**
///
/// ```math
/// \rho_{\text{call}} = K \cdot T \cdot e^{-rT} \cdot N(d2)
/// ```
///
/// **Put Options:**
///
/// ```math
/// \rho_{\text{put}} = -K \cdot T \cdot e^{-rT} \cdot N(-d2)
/// ```
///
/// Where:
/// - \( K \): The strike price of the option.
/// - \( T \): The time to expiration (in years).
/// - \( r \): The risk-free interest rate.
/// - \( N(d2) \): The cumulative distribution function (CDF) of the standard normal distribution evaluated at \( d2 \).
/// - \( e^{-rT} \): The discount factor for the risk-free rate.
///
/// # Calculation Steps
///
/// 1. Compute \( d2 \) using the `d2` function.
/// 2. Calculate the discount factor \( e^{-rT} \).
/// 3. Evaluate \( N(d2) \) or \( N(-d2) \), depending on the option style.
/// 4. Multiply the strike price, time to expiration, discount factor, and \( N(d2) \) or \( N(-d2) \).
/// 5. Multiply the result by the option's quantity.
///
/// # Edge Cases
///
/// - If the discount factor (\( e^{-rT} \)) is zero, the rho is returned as zero.
/// - If \( N(d2) \) or \( N(-d2) \) is zero, the rho is returned as zero.
///
/// # Example
///
/// ```rust
/// use rust_decimal_macros::dec;
/// use tracing::{error, info};
/// use optionstratlib::greeks::rho;
/// use optionstratlib::Options;
/// use optionstratlib::model::types::{ExpirationDate, OptionStyle, OptionType, Side};
/// use optionstratlib::pos;
///
/// let option = Options {
///     option_type: OptionType::European,
///     side: Side::Long,
///     underlying_price: pos!(100.0),
///     strike_price: pos!(95.0),
///     risk_free_rate: dec!(0.05),
///     expiration_date: ExpirationDate::Days(pos!(30.0)),
///     implied_volatility: pos!(0.2),
///     dividend_yield: pos!(0.01),
///     quantity: pos!(1.0),
///     option_style: OptionStyle::Call,
///     underlying_symbol: "".to_string(),
///     exotic_params: None,
/// };
///
/// match rho(&option) {
///     Ok(result) => info!("Rho: {}", result),
///     Err(e) => error!("Error calculating rho: {:?}", e),
/// }
/// ```
///
/// # Notes
///
/// - Rho is typically higher for options with longer time to expiration, as they are more
///   sensitive to changes in the risk-free rate.
/// - Call options have positive rho values, as an increase in interest rates increases their value.
/// - Put options have negative rho values, as an increase in interest rates decreases their value.
pub fn rho(option: &Options) -> Result<Decimal, GreeksError> {
    let d2 = d2(
        option.underlying_price,
        option.strike_price,
        option.risk_free_rate,
        option.expiration_date.get_years().unwrap(),
        option.implied_volatility,
    )?;

    let risk_free_rate: Decimal = option.risk_free_rate;
    let expiration_date: Positive = option.expiration_date.get_years()?;

    let e_rt = (-risk_free_rate * expiration_date).exp();
    if e_rt == Decimal::ZERO {
        return Ok(Decimal::ZERO);
    }

    let strike_price: Decimal = option.strike_price.to_dec();

    let rho = match option.option_style {
        OptionStyle::Call => {
            let big_n_d2 = big_n(d2)?;
            if big_n_d2 == Decimal::ZERO {
                return Ok(Decimal::ZERO);
            }
            strike_price * expiration_date * e_rt * big_n_d2
        }
        OptionStyle::Put => {
            let big_n_minus_d2 = big_n(-d2)?;
            if big_n_minus_d2 == Decimal::ZERO {
                return Ok(Decimal::ZERO);
            }
            Decimal::NEGATIVE_ONE * strike_price * expiration_date * e_rt * big_n_minus_d2
        }
    };

    let quantity: Decimal = option.quantity.into();
    Ok(rho * quantity)
}

/// Computes the sensitivity of the option price to changes in the dividend yield (Rho_d).
///
/// This function calculates how the price of an option changes with respect to variations
/// in the dividend yield of the underlying asset. This metric, often referred to as "dividend rho",
/// is essential for understanding the impact of dividends on the option's value.
///
/// # Parameters
///
/// - `option: &Options`
///   A reference to an `Options` struct containing the following relevant fields:
///   - `underlying_price`: The current price of the underlying asset.
///   - `strike_price`: The strike price of the option.
///   - `risk_free_rate`: The risk-free interest rate.
///   - `expiration_date`: The time to expiration in years (provides `get_years` method).
///   - `implied_volatility`: The implied volatility of the option.
///   - `dividend_yield`: The dividend yield of the underlying asset.
///   - `quantity`: The quantity of the options.
///   - `option_style`: The style of the option (`Call` or `Put`).
///
/// # Returns
///
/// - `Ok(Decimal)`: The calculated dividend sensitivity (`Rho_d`) value for the options contract.
/// - `Err(GreeksError)`: Returns an error if any intermediate calculation fails (e.g., in `d1` or `big_n`).
///
/// # Formula
///
/// The dividend sensitivity is calculated differently for Call and Put options:
///
/// **Call Options:**
///
/// ```math
/// \rho_d^{\text{call}} = -T \cdot S \cdot e^{-qT} \cdot N(d1)
/// ```
///
/// **Put Options:**
///
/// ```math
/// \rho_d^{\text{put}} = T \cdot S \cdot e^{-qT} \cdot N(-d1)
/// ```
///
/// Where:
/// - \( T \): Time to expiration (in years).
/// - \( S \): Price of the underlying asset.
/// - \( q \): Dividend yield.
/// - \( N(d1) \): The cumulative distribution function (CDF) of the standard normal distribution evaluated at \( d1 \).
/// - \( d1 \): A parameter calculated using the Black-Scholes model.
///
/// # Calculation Steps
///
/// 1. Compute \( d1 \) using the `d1` function.
/// 2. Evaluate the exponential factor \( e^{-qT} \), which accounts for the dividend yield.
/// 3. Calculate \( N(d1) \) or \( N(-d1) \), depending on the option style.
/// 4. Use the appropriate formula for Call or Put options.
/// 5. Multiply the result by the option's quantity to adjust for position size.
///
/// # Example
///
/// ```rust
/// use rust_decimal_macros::dec;
/// use tracing::{error, info};
/// use optionstratlib::greeks::rho_d;
/// use optionstratlib::Options;
/// use optionstratlib::model::types::{ExpirationDate, OptionStyle, OptionType, Side};
/// use optionstratlib::pos;
///
/// let option = Options {
///     option_type: OptionType::European,
///     side: Side::Long,
///     underlying_price: pos!(100.0),
///     strike_price: pos!(95.0),
///     risk_free_rate: dec!(0.05),
///     expiration_date: ExpirationDate::Days(pos!(30.0)),
///     implied_volatility: pos!(0.2),
///     dividend_yield: pos!(0.01),
///     quantity: pos!(1.0),
///     option_style: OptionStyle::Call,
///     underlying_symbol: "".to_string(),
///     exotic_params: None,
/// };
///
/// match rho_d(&option) {
///     Ok(result) => info!("Dividend Rho (Rho_d): {}", result),
///     Err(e) => error!("Error calculating Rho_d: {:?}", e),
/// }
/// ```
///
/// # Notes
///
/// - **Call Options**: A higher dividend yield decreases the price of the call option,
///   leading to a negative dividend sensitivity.
/// - **Put Options**: A higher dividend yield increases the price of the put option,
///   leading to a positive dividend sensitivity.
/// - This calculation assumes that dividends are continuously compounded at the dividend yield rate.
/// - \( Rho_d \) is generally more significant for options with longer times to expiration.
pub fn rho_d(option: &Options) -> Result<Decimal, GreeksError> {
    let d1 = d1(
        option.underlying_price,
        option.strike_price,
        option.risk_free_rate,
        option.expiration_date.get_years().unwrap(),
        option.implied_volatility,
    )?;

    let expiration_date: Positive = option.expiration_date.get_years()?;
    let dividend_yield: Positive = option.dividend_yield;
    let underlying_price: Decimal = option.underlying_price.to_dec();

    let rhod = match option.option_style {
        OptionStyle::Call => {
            -expiration_date.to_dec()
                * underlying_price
                * (-expiration_date.to_dec() * dividend_yield).exp()
                * big_n(d1)?
        }
        OptionStyle::Put => {
            expiration_date.to_dec()
                * underlying_price
                * (-expiration_date.to_dec() * dividend_yield).exp()
                * big_n(-d1)?
        }
    };

    let quantity: Decimal = option.quantity.into();
    Ok(rhod * quantity)
}

pub fn alpha(option: &Options) -> Result<Decimal, GreeksError> {
    let gamma = gamma(option)?;
    let theta = theta(option)?;
    match (gamma, theta) {
        (val, _) if val == Decimal::ZERO => Ok(Decimal::ZERO),
        (_, val) if val == Decimal::ZERO => Ok(Decimal::MAX),
        _ => Ok(gamma / theta),
    }
}

#[cfg(test)]
pub mod tests_delta_equations {
    use super::*;
    use crate::constants::{DAYS_IN_A_YEAR, ZERO};
    use crate::model::types::{ExpirationDate, OptionStyle, Side};
    use crate::model::utils::create_sample_option;
    use crate::strategies::DELTA_THRESHOLD;
    use crate::utils::logger::setup_logger;
    use crate::{assert_decimal_eq, pos};
    use approx::assert_relative_eq;
    use num_traits::ToPrimitive;
    use rust_decimal_macros::dec;
    use tracing::info;

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_delta_no_volatility_itm() {
        setup_logger();
        let option = create_sample_option(
            OptionStyle::Call,
            Side::Long,
            pos!(150.0),
            pos!(1.0),
            pos!(150.0),
            Positive::ZERO,
        );
        let delta_value = delta(&option).unwrap();
        info!("Zero Volatility: {}", delta_value);
        assert_relative_eq!(delta_value.to_f64().unwrap(), 1.0, epsilon = 1e-8);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_delta_no_volatility_otm() {
        setup_logger();
        let option = create_sample_option(
            OptionStyle::Call,
            Side::Long,
            pos!(110.0),
            pos!(1.0),
            pos!(150.0),
            Positive::ZERO,
        );
        let delta_value = delta(&option).unwrap().to_f64().unwrap();
        info!("Zero Volatility: {}", delta_value);
        assert_relative_eq!(delta_value, ZERO, epsilon = 1e-8);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_delta_no_volatility_itm_put() {
        setup_logger();
        let option = create_sample_option(
            OptionStyle::Put,
            Side::Long,
            pos!(150.0),
            pos!(1.0),
            pos!(150.0),
            Positive::ZERO,
        );
        let delta_value = delta(&option).unwrap().to_f64().unwrap();
        info!("Zero Volatility: {}", delta_value);
        assert_relative_eq!(delta_value, -1.0, epsilon = 1e-8);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_delta_no_volatility_otm_put() {
        setup_logger();
        let option = create_sample_option(
            OptionStyle::Put,
            Side::Long,
            pos!(160.0),
            pos!(1.0),
            pos!(150.0),
            Positive::ZERO,
        );
        let delta_value = delta(&option).unwrap().to_f64().unwrap();
        info!("Zero Volatility: {}", delta_value);
        assert_relative_eq!(delta_value, ZERO, epsilon = 1e-8);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_delta_no_volatility_itm_short() {
        setup_logger();
        let option = create_sample_option(
            OptionStyle::Call,
            Side::Short,
            pos!(150.0),
            pos!(1.0),
            pos!(150.0),
            Positive::ZERO,
        );
        let delta_value = delta(&option).unwrap().to_f64().unwrap();
        info!("Zero Volatility: {}", delta_value);
        assert_relative_eq!(delta_value, -1.0, epsilon = 1e-8);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_delta_no_volatility_otm_short() {
        setup_logger();
        let option = create_sample_option(
            OptionStyle::Call,
            Side::Short,
            pos!(110.0),
            pos!(1.0),
            pos!(150.0),
            Positive::ZERO,
        );
        let delta_value = delta(&option).unwrap().to_f64().unwrap();
        info!("Zero Volatility: {}", delta_value);
        assert_relative_eq!(delta_value, ZERO, epsilon = 1e-8);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_delta_no_volatility_itm_put_short() {
        setup_logger();
        let option = create_sample_option(
            OptionStyle::Put,
            Side::Short,
            pos!(150.0),
            pos!(1.0),
            pos!(150.0),
            Positive::ZERO,
        );
        let delta_value = delta(&option).unwrap().to_f64().unwrap();
        info!("Zero Volatility: {}", delta_value);
        assert_relative_eq!(delta_value, 1.0, epsilon = 1e-8);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_delta_no_volatility_otm_put_short() {
        setup_logger();
        let option = create_sample_option(
            OptionStyle::Put,
            Side::Short,
            pos!(160.0),
            pos!(1.0),
            pos!(150.0),
            Positive::ZERO,
        );
        let delta_value = delta(&option).unwrap().to_f64().unwrap();
        info!("Zero Volatility: {}", delta_value);
        assert_relative_eq!(delta_value, ZERO, epsilon = 1e-8);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_delta_deep_in_the_money_call() {
        setup_logger();
        let option = create_sample_option(
            OptionStyle::Call,
            Side::Long,
            pos!(150.0),
            pos!(1.0),
            pos!(100.0),
            pos!(0.20),
        );
        let delta_value = delta(&option).unwrap().to_f64().unwrap();
        info!("Deep ITM Call Delta: {}", delta_value);
        assert_relative_eq!(delta_value, 0.9991784198733309, epsilon = 1e-8);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_delta_deep_out_of_the_money_call() {
        let option = create_sample_option(
            OptionStyle::Call,
            Side::Long,
            pos!(50.0),
            pos!(1.0),
            pos!(100.0),
            pos!(0.20),
        );
        let delta_value = delta(&option).unwrap().to_f64().unwrap();
        info!("Deep OTM Call Delta: {}", delta_value);
        assert_relative_eq!(delta_value, 2.0418256951423236e-33, epsilon = 1e-4);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_delta_at_the_money_put() {
        let option = create_sample_option(
            OptionStyle::Put,
            Side::Long,
            pos!(100.0),
            pos!(1.0),
            pos!(100.0),
            pos!(0.20),
        );
        let delta_value = delta(&option).unwrap();
        info!("ATM Put Delta: {}", delta_value);
        assert_decimal_eq!(delta_value, dec!(-0.459658497), DELTA_THRESHOLD);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_delta_short_term_high_volatility() {
        let mut option = create_sample_option(
            OptionStyle::Call,
            Side::Long,
            pos!(100.0),
            pos!(1.0),
            pos!(100.0),
            pos!(0.50),
        );
        option.expiration_date = ExpirationDate::Days(pos!(7.0));
        let delta_value = delta(&option).unwrap().to_f64().unwrap();
        info!("Short-term High Vol Call Delta: {}", delta_value);
        assert_relative_eq!(delta_value, 0.519229469584234, epsilon = 1e-4);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_delta_long_term_low_volatility() {
        let mut option = create_sample_option(
            OptionStyle::Put,
            Side::Long,
            pos!(100.0),
            pos!(1.0),
            pos!(100.0),
            pos!(0.10),
        );
        option.expiration_date = ExpirationDate::Days(DAYS_IN_A_YEAR);
        let delta_value = delta(&option).unwrap();
        info!("Long-term Low Vol Put Delta: {}", delta_value);
        assert_decimal_eq!(delta_value, dec!(-0.2882625996), DELTA_THRESHOLD);
    }
}

#[cfg(test)]
pub mod tests_gamma_equations {
    use super::*;
    use crate::constants::DAYS_IN_A_YEAR;
    use crate::model::types::{ExpirationDate, OptionStyle, Side};
    use crate::model::utils::create_sample_option;
    use crate::pos;
    use crate::utils::logger::setup_logger;
    use approx::assert_relative_eq;
    use num_traits::ToPrimitive;
    use tracing::info;

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_gamma_deep_in_the_money_call() {
        setup_logger();
        let option = create_sample_option(
            OptionStyle::Call,
            Side::Long,
            pos!(150.0),
            pos!(1.0),
            pos!(120.0),
            pos!(0.2),
        );
        let gamma_value = gamma(&option).unwrap().to_f64().unwrap();
        info!("Deep ITM Call Gamma: {}", gamma_value);
        assert_relative_eq!(gamma_value, 0.000016049457791525, epsilon = 1e-8);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_gamma_deep_out_of_the_money_call() {
        let option = create_sample_option(
            OptionStyle::Call,
            Side::Long,
            pos!(50.0),
            pos!(1.0),
            pos!(100.0),
            pos!(0.20),
        );
        let gamma_value = gamma(&option).unwrap().to_f64().unwrap();
        info!("Deep OTM Call Gamma: {}", gamma_value);
        assert_relative_eq!(gamma_value, 0.0, epsilon = 1e-34);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_gamma_at_the_money_put() {
        let option = create_sample_option(
            OptionStyle::Put,
            Side::Long,
            pos!(100.0),
            pos!(1.0),
            pos!(100.0),
            pos!(0.20),
        );
        let gamma_value = gamma(&option).unwrap().to_f64().unwrap();
        info!("ATM Put Gamma: {}", gamma_value);
        assert_relative_eq!(gamma_value, 0.06917076441486919, epsilon = 1e-8);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_gamma_short_term_high_volatility() {
        let mut option = create_sample_option(
            OptionStyle::Call,
            Side::Long,
            pos!(100.0),
            pos!(1.0),
            pos!(100.0),
            pos!(0.50),
        );
        option.expiration_date = ExpirationDate::Days(pos!(7.0));
        let gamma_value = gamma(&option).unwrap().to_f64().unwrap();
        info!("Short-term High Vol Call Gamma: {}", gamma_value);
        assert_relative_eq!(gamma_value, 0.05753657912620555, epsilon = 1e-8);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_gamma_long_term_low_volatility() {
        let mut option = create_sample_option(
            OptionStyle::Put,
            Side::Long,
            pos!(100.0),
            pos!(1.0),
            pos!(100.0),
            pos!(0.10),
        );
        option.expiration_date = ExpirationDate::Days(DAYS_IN_A_YEAR);
        let gamma_value = gamma(&option).unwrap().to_f64().unwrap();
        info!("Long-term Low Vol Put Gamma: {}", gamma_value);
        assert_relative_eq!(gamma_value, 0.033953150664723986, epsilon = 1e-8);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_gamma_zero_volatility() {
        let option = create_sample_option(
            OptionStyle::Call,
            Side::Long,
            pos!(100.0),
            pos!(1.0),
            pos!(100.0),
            Positive::ZERO,
        );
        let gamma_value = gamma(&option).unwrap().to_f64().unwrap();
        info!("Zero Volatility Call Gamma: {}", gamma_value);
        assert_relative_eq!(gamma_value, 0.0, epsilon = 1e-8);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_gamma_extreme_high_volatility() {
        let option = create_sample_option(
            OptionStyle::Put,
            Side::Short,
            pos!(100.0),
            pos!(1.0),
            pos!(100.0),
            pos!(5.0),
        );
        let gamma_value = gamma(&option).unwrap().to_f64().unwrap();
        info!("Extreme High Volatility Put Gamma: {}", gamma_value);
        assert_relative_eq!(gamma_value, 0.002146478293943308, epsilon = 1e-8);
    }
}

#[cfg(test)]
mod tests_gamma_equations_values {
    use super::*;
    use crate::model::types::{ExpirationDate, OptionStyle, Side};
    use crate::utils::logger::setup_logger;
    use crate::{pos, OptionType};
    use approx::assert_relative_eq;
    use num_traits::ToPrimitive;
    use tracing::info;

    #[test]
    fn test_50_vol_10() {
        setup_logger();
        let option = Options::new(
            OptionType::European,
            Side::Long,
            "XYZ".parse().unwrap(),
            pos!(50.0),
            ExpirationDate::Days(pos!(365.0)),
            pos!(0.10),
            pos!(1.0),
            pos!(50.0),
            Decimal::ZERO,
            OptionStyle::Call,
            Positive::ZERO,
            None,
        );
        let gamma_value = gamma(&option).unwrap().to_f64().unwrap();
        info!("Gamma: {}", gamma_value);
        assert_relative_eq!(gamma_value, 0.0796887828189609, epsilon = 1e-8);
    }

    #[test]
    fn test_50_vol_5() {
        setup_logger();
        let option = Options::new(
            OptionType::European,
            Side::Long,
            "XYZ".parse().unwrap(),
            pos!(50.0),
            ExpirationDate::Days(pos!(365.0)),
            pos!(0.05),
            pos!(1.0),
            pos!(50.0),
            Decimal::ZERO,
            OptionStyle::Call,
            Positive::ZERO,
            None,
        );
        let gamma_value = gamma(&option).unwrap().to_f64().unwrap();
        info!("Gamma: {}", gamma_value);
        assert_relative_eq!(gamma_value, 0.15952705216736393, epsilon = 1e-8);
    }

    #[test]
    fn test_50_vol_20() {
        setup_logger();
        let option = Options::new(
            OptionType::European,
            Side::Long,
            "XYZ".parse().unwrap(),
            pos!(50.0),
            ExpirationDate::Days(pos!(365.0)),
            pos!(0.2),
            pos!(1.0),
            pos!(50.0),
            Decimal::ZERO,
            OptionStyle::Call,
            Positive::ZERO,
            None,
        );
        let gamma_value = gamma(&option).unwrap().to_f64().unwrap();
        info!("Gamma: {}", gamma_value);
        assert_relative_eq!(gamma_value, 0.03969525474873078, epsilon = 1e-8);
    }
}

#[cfg(test)]
pub mod tests_vega_equation {
    use super::*;
    use crate::constants::DAYS_IN_A_YEAR;
    use crate::model::types::{ExpirationDate, OptionType, Side};
    use crate::{pos, Positive};
    use num_traits::ToPrimitive;
    use rust_decimal_macros::dec;

    fn create_test_option(
        underlying_price: Positive,
        strike_price: Positive,
        implied_volatility: Positive,
        dividend_yield: Positive,
        expiration_in_days: Positive,
    ) -> Options {
        Options::new(
            OptionType::European,
            Side::Long,
            "TEST".to_string(),
            strike_price,
            ExpirationDate::Days(expiration_in_days),
            implied_volatility,
            pos!(1.0), // Quantity
            underlying_price,
            dec!(0.05), // Risk-free rate
            OptionStyle::Call,
            dividend_yield,
            None, // No exotic params for this test
        )
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_vega_atm() {
        let option = create_test_option(
            pos!(100.0),
            pos!(100.0),
            pos!(0.2),
            Positive::ZERO,
            DAYS_IN_A_YEAR,
        );
        let vega = vega(&option).unwrap().to_f64().unwrap();
        let expected_vega = 0.3752403469;
        assert!(
            (vega - expected_vega).abs() < 1e-5,
            "Vega ATM test failed: expected {}, got {}",
            expected_vega,
            vega
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_vega_otm() {
        let option = create_test_option(
            pos!(90.0),
            pos!(100.0),
            pos!(0.2),
            Positive::ZERO,
            DAYS_IN_A_YEAR,
        );
        let vega = vega(&option).unwrap().to_f64().unwrap();
        let expected_vega = 0.35347991;
        assert!(
            (vega - expected_vega).abs() < 1e-5,
            "Vega OTM test failed: expected {}, got {}",
            expected_vega,
            vega
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_vega_short_expiration() {
        let option = create_test_option(
            pos!(100.0),
            pos!(100.0),
            pos!(0.2),
            Positive::ZERO,
            Positive::ONE,
        );
        let vega = vega(&option).unwrap().to_f64().unwrap();
        let expected_vega = 0.020878089;
        assert!(
            (vega - expected_vega).abs() < 1e-5,
            "Vega short expiration test failed: expected {}, got {}",
            expected_vega,
            vega
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_vega_with_dividends() {
        let option = create_test_option(
            pos!(100.0),
            pos!(100.0),
            pos!(0.2),
            pos!(0.03),
            Positive::ONE,
        );
        let vega = vega(&option).unwrap().to_f64().unwrap();
        let expected_vega = 0.0208763735;
        assert!(
            (vega - expected_vega).abs() < 1e-5,
            "Vega with dividends test failed: expected {}, got {}",
            expected_vega,
            vega
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_vega_itm() {
        let option = create_test_option(
            pos!(110.0),
            pos!(100.0),
            pos!(0.2),
            Positive::ZERO,
            Positive::ONE,
        );
        let vega = vega(&option).unwrap().to_f64().unwrap();
        let expected_vega = 0.0;
        assert!(
            (vega - expected_vega).abs() < 1e-5,
            "Vega ITM test failed: expected {}, got {}",
            expected_vega,
            vega
        );
    }
}

#[cfg(test)]
pub mod tests_rho_equations {
    use super::*;
    use crate::constants::DAYS_IN_A_YEAR;
    use crate::model::types::{ExpirationDate, OptionStyle, OptionType, Side};
    use crate::pos;
    use approx::assert_relative_eq;
    use num_traits::ToPrimitive;
    use rust_decimal_macros::dec;

    fn create_test_option(style: OptionStyle) -> Options {
        Options {
            option_type: OptionType::European,
            side: Side::Long,
            underlying_symbol: "TEST".to_string(),
            strike_price: pos!(100.0),
            expiration_date: ExpirationDate::Days(DAYS_IN_A_YEAR),
            implied_volatility: pos!(0.2),
            quantity: pos!(1.0),
            underlying_price: pos!(100.0),
            risk_free_rate: dec!(0.05),
            option_style: style,
            dividend_yield: Positive::ZERO,
            exotic_params: None,
        }
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_rho_call_option() {
        let option = create_test_option(OptionStyle::Call);
        let result = rho(&option).unwrap().to_f64().unwrap();
        assert_relative_eq!(result, 53.232481545376345, epsilon = 1e-8);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_rho_put_option() {
        let option = create_test_option(OptionStyle::Put);
        let result = rho(&option).unwrap().to_f64().unwrap();
        assert_relative_eq!(result, -41.89046090469506, epsilon = 1e-8);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_rho_zero_time_to_expiry() {
        let mut option = create_test_option(OptionStyle::Call);
        option.expiration_date = ExpirationDate::Days(Positive::ZERO);
        let result = rho(&option).is_err();
        assert!(result);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_rho_zero_risk_free_rate() {
        let mut option = create_test_option(OptionStyle::Call);
        option.risk_free_rate = dec!(0.0);
        let result = rho(&option).unwrap().to_f64().unwrap();
        assert_relative_eq!(result, 46.0172162722971, epsilon = 1e-8);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_rho_deep_out_of_money_call() {
        let mut option = create_test_option(OptionStyle::Call);
        option.strike_price = pos!(1000.0);
        let result = rho(&option).unwrap().to_f64().unwrap();
        assert_relative_eq!(result, 0.0, epsilon = 1e-8);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_rho_deep_out_of_money_put() {
        let mut option = create_test_option(OptionStyle::Put);
        option.strike_price = pos!(1.0);
        let result = rho(&option).unwrap().to_f64().unwrap();
        assert_relative_eq!(result, 0.0, epsilon = 1e-8);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_rho_high_volatility() {
        let mut option = create_test_option(OptionStyle::Call);
        option.implied_volatility = Positive::ONE;
        let result = rho(&option).unwrap().to_f64().unwrap();
        assert_relative_eq!(result, 31.043868837728198, epsilon = 0.0001);
    }
}

#[cfg(test)]
pub mod tests_theta_long_equations {
    use super::*;
    use crate::constants::DAYS_IN_A_YEAR;
    use crate::model::types::{ExpirationDate, Side};
    use crate::model::utils::create_sample_option;
    use crate::pos;
    use approx::assert_relative_eq;
    use num_traits::ToPrimitive;

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_theta_call_option() {
        // Create a sample call option
        let option = create_sample_option(
            OptionStyle::Call,
            Side::Long,
            pos!(150.0), // underlying price
            pos!(1.0),   // quantity
            pos!(155.0), // strike price
            pos!(0.20),  // implied volatility
        );

        // Expected theta value for a call option (precomputed or from known source)
        let expected_theta = -0.0561725050;

        // Compute the theta value using the function
        let calculated_theta = theta(&option).unwrap().to_f64().unwrap();

        // Assert the calculated theta is close to the expected value
        assert_relative_eq!(calculated_theta, expected_theta, epsilon = 1e-8);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_theta_put_option() {
        // Create a sample put option
        let option = create_sample_option(
            OptionStyle::Put,
            Side::Long,
            pos!(150.0), // underlying price
            pos!(1.0),   // quantity
            pos!(145.0), // strike price
            pos!(0.25),  // implied volatility
        );

        // Expected theta value for a put option (precomputed or from known source)
        let expected_theta = -0.055928204732;

        // Compute the theta value using the function
        let calculated_theta = theta(&option).unwrap().to_f64().unwrap();

        // Assert the calculated theta is close to the expected value
        assert_relative_eq!(calculated_theta, expected_theta, epsilon = 1e-8);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_theta_call_option_near_expiry() {
        // Create a sample call option near expiry
        let mut option = create_sample_option(
            OptionStyle::Call,
            Side::Long,
            pos!(150.0), // underlying price
            pos!(1.0),   // quantity
            pos!(150.0), // strike price
            pos!(0.15),  // implied volatility
        );
        option.expiration_date = ExpirationDate::Days(pos!(1.0)); // Option close to expiry

        // Expected theta value for a near-expiry call option (precomputed)
        let expected_theta = -0.24315788969;

        // Compute the theta value using the function
        let calculated_theta = theta(&option).unwrap().to_f64().unwrap();

        // Assert the calculated theta is close to the expected value
        assert_relative_eq!(calculated_theta, expected_theta, epsilon = 1e-8);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_theta_put_option_far_from_expiry() {
        // Create a sample put option far from expiry
        let mut option = create_sample_option(
            OptionStyle::Put,
            Side::Long,
            pos!(140.0), // underlying price
            pos!(1.0),   // quantity
            pos!(130.0), // strike price
            pos!(0.30),  // implied volatility
        );
        option.expiration_date = ExpirationDate::Days(DAYS_IN_A_YEAR); // Option far from expiry

        // Expected theta value for a far-expiry put option (precomputed)
        let expected_theta = -0.0139607780805;

        // Compute the theta value using the function
        let calculated_theta = theta(&option).unwrap().to_f64().unwrap();

        // Assert the calculated theta is close to the expected value
        assert_relative_eq!(calculated_theta, expected_theta, epsilon = 1e-8);
    }
}

#[cfg(test)]
pub mod tests_theta_short_equations {
    use super::*;
    use crate::constants::DAYS_IN_A_YEAR;
    use crate::model::types::{ExpirationDate, Side};
    use crate::model::utils::create_sample_option;
    use crate::pos;
    use approx::assert_relative_eq;
    use num_traits::ToPrimitive;

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_theta_short_call_option() {
        // Create a sample short call option
        let option = create_sample_option(
            OptionStyle::Call,
            Side::Short,
            pos!(150.0), // underlying price
            pos!(1.0),   // quantity
            pos!(155.0), // strike price
            pos!(0.20),  // implied volatility
        );

        // Expected theta value for a short call option (precomputed or from known source)
        let expected_theta = -0.05617250509;

        // Compute the theta value using the function
        let calculated_theta = theta(&option).unwrap().to_f64().unwrap();

        // Assert the calculated theta is close to the expected value
        assert_relative_eq!(calculated_theta, expected_theta, epsilon = 1e-8);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_theta_short_put_option() {
        // Create a sample short put option
        let option = create_sample_option(
            OptionStyle::Put,
            Side::Short,
            pos!(150.0), // underlying price
            pos!(1.0),   // quantity
            pos!(145.0), // strike price
            pos!(0.25),  // implied volatility
        );

        // Expected theta value for a short put option (precomputed or from known source)
        let expected_theta = -0.05592820473;

        // Compute the theta value using the function
        let calculated_theta = theta(&option).unwrap().to_f64().unwrap();

        // Assert the calculated theta is close to the expected value
        assert_relative_eq!(calculated_theta, expected_theta, epsilon = 1e-8);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_theta_short_call_option_near_expiry() {
        // Create a sample short call option near expiry
        let mut option = create_sample_option(
            OptionStyle::Call,
            Side::Short,
            pos!(150.0), // underlying price
            pos!(1.0),   // quantity
            pos!(150.0), // strike price
            pos!(0.15),  // implied volatility
        );
        option.expiration_date = ExpirationDate::Days(pos!(1.0)); // Option close to expiry

        // Expected theta value for a short near-expiry call option (precomputed)
        let expected_theta = -0.2431578896;

        // Compute the theta value using the function
        let calculated_theta = theta(&option).unwrap().to_f64().unwrap();

        // Assert the calculated theta is close to the expected value
        assert_relative_eq!(calculated_theta, expected_theta, epsilon = 1e-8);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_theta_short_put_option_far_from_expiry() {
        // Create a sample short put option far from expiry
        let mut option = create_sample_option(
            OptionStyle::Put,
            Side::Short,
            pos!(140.0), // underlying price
            pos!(1.0),   // quantity
            pos!(130.0), // strike price
            pos!(0.30),  // implied volatility
        );
        option.expiration_date = ExpirationDate::Days(DAYS_IN_A_YEAR); // Option far from expiry

        // Expected theta value for a far-expiry short put option (precomputed)
        let expected_theta = -0.01396077;

        // Compute the theta value using the function
        let calculated_theta = theta(&option).unwrap().to_f64().unwrap();

        // Assert the calculated theta is close to the expected value
        assert_relative_eq!(calculated_theta, expected_theta, epsilon = 1e-8);
    }
}

#[cfg(test)]
mod tests_greeks_trait {
    use super::*;
    use crate::model::types::{ExpirationDate, OptionStyle, OptionType, Side};
    use crate::{assert_decimal_eq, pos};
    use rust_decimal_macros::dec;

    // A simple struct for testing the Greeks trait
    struct TestOptionCollection {
        options: Vec<Options>,
    }

    impl Greeks for TestOptionCollection {
        fn get_options(&self) -> Result<Vec<&Options>, GreeksError> {
            Ok(self.options.iter().collect())
        }
    }

    // Helper function to create a test option
    fn create_test_option(side: Side, style: OptionStyle, quantity: Positive) -> Options {
        Options::new(
            OptionType::European,
            side,
            "TEST".to_string(),
            pos!(100.0), // strike_price
            ExpirationDate::Days(pos!(30.0)),
            pos!(0.2), // implied_volatility
            quantity,
            pos!(100.0), // underlying_price
            dec!(0.05),  // risk_free_rate
            style,
            pos!(0.01), // dividend_yield
            None,       // exotic_params
        )
    }

    #[test]
    fn test_greeks_single_option() {
        let option = create_test_option(Side::Long, OptionStyle::Call, pos!(1.0));
        let collection = TestOptionCollection {
            options: vec![option],
        };

        let greeks = collection.greeks().unwrap();

        // Test each greek value
        assert_decimal_eq!(greeks.delta, dec!(0.539519922), dec!(0.000001));
        assert_decimal_eq!(greeks.gamma, dec!(0.069170764), dec!(0.000001));
        assert_decimal_eq!(greeks.theta, dec!(-0.04351001), dec!(0.000001));
        assert_decimal_eq!(greeks.vega, dec!(0.1137053), dec!(0.000001));
        assert_decimal_eq!(greeks.rho, dec!(4.233121), dec!(0.000001));
        assert_decimal_eq!(greeks.rho_d, dec!(-4.434410), dec!(0.000001));
    }

    #[test]
    fn test_greeks_multiple_options() {
        let option1 = create_test_option(Side::Long, OptionStyle::Call, pos!(1.0));
        let option2 = create_test_option(Side::Short, OptionStyle::Put, pos!(1.0));
        let collection = TestOptionCollection {
            options: vec![option1, option2],
        };

        let greeks = collection.greeks().unwrap();

        // Test aggregated greek values
        assert!(
            greeks.delta.abs() > dec!(0.0),
            "Delta should be non-zero for multiple options"
        );
        assert!(
            greeks.gamma.abs() > dec!(0.0),
            "Gamma should be non-zero for multiple options"
        );
        assert!(
            greeks.theta.abs() > dec!(0.0),
            "Theta should be non-zero for multiple options"
        );
        assert!(
            greeks.vega.abs() > dec!(0.0),
            "Vega should be non-zero for multiple options"
        );
        assert!(
            greeks.rho.abs() > dec!(0.0),
            "Rho should be non-zero for multiple options"
        );
        assert!(
            greeks.rho_d.abs() > dec!(0.0),
            "Rho_d should be non-zero for multiple options"
        );
    }

    #[test]
    fn test_greeks_zero_quantity() {
        let option = create_test_option(Side::Long, OptionStyle::Call, pos!(0.0));
        let collection = TestOptionCollection {
            options: vec![option],
        };

        let greeks = collection.greeks().unwrap();

        // All greeks should be zero for zero quantity
        assert_eq!(greeks.delta, dec!(0.0));
        assert_eq!(greeks.gamma, dec!(0.0));
        assert_eq!(greeks.theta, dec!(0.0));
        assert_eq!(greeks.vega, dec!(0.0));
        assert_eq!(greeks.rho, dec!(0.0));
        assert_eq!(greeks.rho_d, dec!(0.0));
    }

    #[test]
    fn test_greeks_opposing_positions() {
        let option1 = Options::new(
            OptionType::European,
            Side::Long,
            "TEST".to_string(),
            pos!(50.0), // strike_price
            ExpirationDate::Days(pos!(365.0)),
            pos!(0.2), // implied_volatility
            Positive::ONE,
            pos!(50.0), // underlying_price
            dec!(0.05), // risk_free_rate
            OptionStyle::Call,
            pos!(0.01), // dividend_yield
            None,       // exotic_params
        );
        let option2 = Options::new(
            OptionType::European,
            Side::Short,
            "TEST".to_string(),
            pos!(50.0), // strike_price
            ExpirationDate::Days(pos!(365.0)),
            pos!(0.2), // implied_volatility
            Positive::ONE,
            pos!(50.0), // underlying_price
            dec!(0.05), // risk_free_rate
            OptionStyle::Call,
            pos!(0.01), // dividend_yield
            None,       // exotic_params
        );
        let collection = TestOptionCollection {
            options: vec![option1, option2],
        };

        let greeks = collection.greeks().unwrap();

        // Opposing positions should mostly cancel out
        assert_decimal_eq!(greeks.delta, Decimal::ZERO, dec!(0.000001));
        assert_decimal_eq!(greeks.gamma, dec!(0.0743013), dec!(0.000001));
        assert_decimal_eq!(greeks.vega, dec!(0.37150664), dec!(0.000001));
        assert_decimal_eq!(greeks.rho, dec!(53.232481), dec!(0.000001));
    }

    #[test]
    fn test_individual_greek_methods() {
        let option1 = create_test_option(Side::Long, OptionStyle::Call, pos!(1.0));
        let option2 = create_test_option(Side::Short, OptionStyle::Put, pos!(1.0));
        let collection = TestOptionCollection {
            options: vec![option1, option2],
        };

        // Test each individual greek method
        let delta = collection.delta().unwrap();
        let gamma = collection.gamma().unwrap();
        let theta = collection.theta().unwrap();
        let vega = collection.vega().unwrap();
        let rho = collection.rho().unwrap();
        let rho_d = collection.rho_d().unwrap();

        // Verify each value is non-zero (actual values depend on input parameters)
        assert!(delta.abs() > dec!(0.0), "Delta calculation failed");
        assert!(gamma.abs() > dec!(0.0), "Gamma calculation failed");
        assert!(theta.abs() > dec!(0.0), "Theta calculation failed");
        assert!(vega.abs() > dec!(0.0), "Vega calculation failed");
        assert!(rho.abs() > dec!(0.0), "Rho calculation failed");
        assert!(rho_d.abs() > dec!(0.0), "Rho_d calculation failed");
    }

    #[test]
    fn test_empty_option_collection() {
        let collection = TestOptionCollection { options: vec![] };

        // All greeks should be zero for empty collection
        let greeks = collection.greeks().unwrap();
        assert_eq!(greeks.delta, dec!(0.0));
        assert_eq!(greeks.gamma, dec!(0.0));
        assert_eq!(greeks.theta, dec!(0.0));
        assert_eq!(greeks.vega, dec!(0.0));
        assert_eq!(greeks.rho, dec!(0.0));
        assert_eq!(greeks.rho_d, dec!(0.0));
    }

    #[test]
    fn test_greeks_with_different_expirations() {
        let mut option1 = create_test_option(Side::Long, OptionStyle::Call, pos!(1.0));
        let mut option2 = create_test_option(Side::Long, OptionStyle::Call, pos!(1.0));

        // Set different expiration dates
        option1.expiration_date = ExpirationDate::Days(pos!(30.0));
        option2.expiration_date = ExpirationDate::Days(pos!(60.0));

        let collection = TestOptionCollection {
            options: vec![option1, option2],
        };

        let greeks = collection.greeks().unwrap();

        // Verify values are calculated correctly for different expirations
        assert!(greeks.delta.abs() > dec!(0.0));
        assert!(greeks.gamma.abs() > dec!(0.0));
        assert!(greeks.theta.abs() > dec!(0.0));
        assert!(greeks.vega.abs() > dec!(0.0));
        assert!(greeks.rho.abs() > dec!(0.0));
        assert!(greeks.rho_d.abs() > dec!(0.0));
    }
}
