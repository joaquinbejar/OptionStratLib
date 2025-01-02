/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 11/8/24
******************************************************************************/
use crate::constants::ZERO;
use crate::error::greeks::GreeksError;
use crate::f2du;
use crate::greeks::utils::{big_n, d1, d2, n};
use crate::model::types::OptionStyle;
use crate::Options;
use rust_decimal::{Decimal, MathematicalOps};

#[allow(dead_code)]
#[derive(Debug, PartialEq)]
pub struct Greek {
    pub delta: Decimal,
    pub gamma: Decimal,
    pub theta: Decimal,
    pub vega: Decimal,
    pub rho: Decimal,
    pub rho_d: Decimal,
}

pub trait Greeks {
    fn greeks(&self) -> Greek;
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
/// use optionstratlib::constants::ZERO;
/// use optionstratlib::greeks::equations::delta;
/// use optionstratlib::Options;
/// use optionstratlib::model::types::{ExpirationDate, OptionStyle, OptionType, Side};
/// use optionstratlib::{f2p, Positive};
/// let option = Options {
///     option_type: OptionType::European,side:
///     Side::Long,underlying_price:
///     f2p!(100.0),
///     strike_price: f2p!(95.0),
///     risk_free_rate: 0.05,
///     expiration_date: ExpirationDate::Days(30.0),
///     implied_volatility: 0.2,
///     dividend_yield: ZERO,
///     quantity: f2p!(1.0),
///     option_style: OptionStyle::Call,
///     underlying_symbol: "AAPL".to_string(),
///     exotic_params: None,
/// };
///
/// match delta(&option) {
///     Ok(result) => println!("Delta: {}", result),
///     Err(e) => eprintln!("Error calculating delta: {:?}", e),
/// }
/// ```
pub fn delta(option: &Options) -> Result<Decimal, GreeksError> {
    let dividend_yield: Decimal = f2du!(option.dividend_yield)?;

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
        option.expiration_date.get_years(),
        option.implied_volatility,
    )?;

    let expiration_date: Decimal = f2du!(option.expiration_date.get_years())?;
    let div_date = (-dividend_yield * expiration_date).exp();

    let delta = match option.option_style {
        OptionStyle::Call => sign * big_n(d1)? * div_date,
        OptionStyle::Put => sign * (big_n(d1)? - Decimal::ONE) * div_date,
    };
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
/// use optionstratlib::greeks::equations::gamma;
/// use optionstratlib::Options;
/// use optionstratlib::model::types::{ExpirationDate, OptionStyle, OptionType, Side};
/// use optionstratlib::f2p;
/// let option = Options {
///     option_type: OptionType::European,
///     side: Side::Long,
///     underlying_price: f2p!(100.0),
///     strike_price: f2p!(95.0),
///     risk_free_rate: 0.05,
///     expiration_date: ExpirationDate::Days(30.0),
///     implied_volatility: 0.2,
///     dividend_yield: 0.01,
///     quantity: f2p!(1.0),
///     option_style: OptionStyle::Call,
///     underlying_symbol: "".to_string(),
///     exotic_params: None,
/// };
///
/// match gamma(&option) {
///     Ok(result) => println!("Gamma: {}", result),
///     Err(e) => eprintln!("Error calculating gamma: {:?}", e),
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
        option.expiration_date.get_years(),
        option.implied_volatility,
    )?;

    let expiration_date: Decimal = f2du!(option.expiration_date.get_years())?;
    let dividend_yield: Decimal = f2du!(option.dividend_yield)?;
    let underlying_price: Decimal = option.underlying_price.into();
    let implied_volatility: Decimal = f2du!(option.implied_volatility)?;

    let gamma = (-dividend_yield * expiration_date).exp() * n(d1)?
        / (underlying_price * implied_volatility * expiration_date.sqrt().unwrap());

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
/// use optionstratlib::greeks::equations::theta;
/// use optionstratlib::Options;
/// use optionstratlib::model::types::{ExpirationDate, OptionStyle, OptionType, Side};
/// use optionstratlib::f2p;
/// let option = Options {
///     option_type: OptionType::European,
///     side: Side::Long,
///     underlying_price: f2p!(100.0),
///     strike_price: f2p!(95.0),
///     risk_free_rate: 0.05,
///     expiration_date: ExpirationDate::Days(30.0),
///     implied_volatility: 0.2,
///     dividend_yield: 0.01,
///     quantity: f2p!(1.0),
///     option_style: OptionStyle::Call,
///     underlying_symbol: "".to_string(),
///     exotic_params: None,
/// };
///
/// match theta(&option) {
///     Ok(result) => println!("Theta: {}", result),
///     Err(e) => eprintln!("Error calculating Theta: {:?}", e),
/// }
/// ```
///
/// # Notes
///
/// - A positive Theta means the option gains value as time passes (rare and usually for short positions).
/// - A negative Theta is typical for long positions, as the option loses extrinsic value over time.
/// - If the implied volatility is zero, Theta may be close to zero for far-out-of-the-money options.
pub fn theta(option: &Options) -> Result<Decimal, GreeksError> {
    let d1 = d1(
        option.underlying_price,
        option.strike_price,
        option.risk_free_rate,
        option.expiration_date.get_years(),
        option.implied_volatility,
    )?;
    let d2 = d2(
        option.underlying_price,
        option.strike_price,
        option.risk_free_rate,
        option.expiration_date.get_years(),
        option.implied_volatility,
    )?;

    let expiration_date: Decimal = f2du!(option.expiration_date.get_years())?;
    let dividend_yield: Decimal = f2du!(option.dividend_yield)?;
    let underlying_price: Decimal = option.underlying_price.to_dec();
    let implied_volatility: Decimal = f2du!(option.implied_volatility)?;

    let common_term: Decimal =
        -underlying_price * implied_volatility * (-dividend_yield * expiration_date).exp() * n(d1)?
            / (Decimal::TWO * expiration_date.sqrt().unwrap());

    let strike_price: Decimal = option.strike_price.to_dec();
    let risk_free_rate: Decimal = f2du!(option.risk_free_rate)?;

    let theta: Decimal = match option.option_style {
        OptionStyle::Call => {
            common_term
                - risk_free_rate
                    * strike_price
                    * (-risk_free_rate * expiration_date).exp()
                    * big_n(d2)?
                + dividend_yield
                    * underlying_price
                    * (-dividend_yield * expiration_date).exp()
                    * big_n(d1)?
        }
        OptionStyle::Put => {
            common_term
                + risk_free_rate
                    * strike_price
                    * (-risk_free_rate * expiration_date).exp()
                    * big_n(-d2)?
                - dividend_yield
                    * underlying_price
                    * (-dividend_yield * expiration_date).exp()
                    * big_n(-d1)?
        }
    };

    let quantity: Decimal = option.quantity.into();
    Ok(theta * quantity)
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
/// use optionstratlib::greeks::equations::vega;
/// use optionstratlib::Options;
/// use optionstratlib::model::types::{ExpirationDate, OptionStyle, OptionType, Side};
/// use optionstratlib::f2p;
///
/// let option = Options {
///     option_type: OptionType::European,
///     side: Side::Long,
///     underlying_price: f2p!(100.0),
///     strike_price: f2p!(95.0),
///     risk_free_rate: 0.05,
///     expiration_date: ExpirationDate::Days(30.0),
///     implied_volatility: 0.2,
///     dividend_yield: 0.01,
///     quantity: f2p!(1.0),
///     option_style: OptionStyle::Call,
///     underlying_symbol: "".to_string(),
///     exotic_params: None,
/// };
///
/// match vega(&option) {
///     Ok(result) => println!("Vega: {}", result),
///     Err(e) => eprintln!("Error calculating Vega: {:?}", e),
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
        option.expiration_date.get_years(),
        option.implied_volatility,
    )?;

    let expiration_date: Decimal = f2du!(option.expiration_date.get_years())?;
    let dividend_yield: Decimal = f2du!(option.dividend_yield)?;
    let underlying_price: Decimal = option.underlying_price.to_dec();

    let vega: Decimal = underlying_price
        * (-dividend_yield * expiration_date).exp()
        * big_n(d1)?
        * expiration_date.sqrt().unwrap();

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
/// use optionstratlib::greeks::equations::rho;
/// use optionstratlib::Options;
/// use optionstratlib::model::types::{ExpirationDate, OptionStyle, OptionType, Side};
/// use optionstratlib::f2p;
///
/// let option = Options {
///     option_type: OptionType::European,
///     side: Side::Long,
///     underlying_price: f2p!(100.0),
///     strike_price: f2p!(95.0),
///     risk_free_rate: 0.05,
///     expiration_date: ExpirationDate::Days(30.0),
///     implied_volatility: 0.2,
///     dividend_yield: 0.01,
///     quantity: f2p!(1.0),
///     option_style: OptionStyle::Call,
///     underlying_symbol: "".to_string(),
///     exotic_params: None,
/// };
///
/// match rho(&option) {
///     Ok(result) => println!("Rho: {}", result),
///     Err(e) => eprintln!("Error calculating rho: {:?}", e),
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
        option.expiration_date.get_years(),
        option.implied_volatility,
    )?;

    let risk_free_rate: Decimal = f2du!(option.risk_free_rate)?;
    let expiration_date: Decimal = f2du!(option.expiration_date.get_years())?;

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
/// use optionstratlib::greeks::equations::rho_d;
/// use optionstratlib::Options;
/// use optionstratlib::model::types::{ExpirationDate, OptionStyle, OptionType, Side};
/// use optionstratlib::f2p;
///
/// let option = Options {
///     option_type: OptionType::European,
///     side: Side::Long,
///     underlying_price: f2p!(100.0),
///     strike_price: f2p!(95.0),
///     risk_free_rate: 0.05,
///     expiration_date: ExpirationDate::Days(30.0),
///     implied_volatility: 0.2,
///     dividend_yield: 0.01,
///     quantity: f2p!(1.0),
///     option_style: OptionStyle::Call,
///     underlying_symbol: "".to_string(),
///     exotic_params: None,
/// };
///
/// match rho_d(&option) {
///     Ok(result) => println!("Dividend Rho (Rho_d): {}", result),
///     Err(e) => eprintln!("Error calculating Rho_d: {:?}", e),
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
        option.expiration_date.get_years(),
        option.implied_volatility,
    )?;

    let expiration_date: Decimal = f2du!(option.expiration_date.get_years())?;
    let dividend_yield: Decimal = f2du!(option.dividend_yield)?;
    let underlying_price: Decimal = option.underlying_price.to_dec();

    let rhod = match option.option_style {
        OptionStyle::Call => {
            -expiration_date
                * underlying_price
                * (-dividend_yield * expiration_date).exp()
                * big_n(d1)?
        }
        OptionStyle::Put => {
            expiration_date
                * underlying_price
                * (-dividend_yield * expiration_date).exp()
                * big_n(-d1)?
        }
    };

    let quantity: Decimal = option.quantity.into();
    Ok(rhod * quantity)
}

#[cfg(test)]
mod tests_delta_equations {
    use super::*;
    use crate::constants::ZERO;
    use crate::f2p;
    use crate::model::types::{ExpirationDate, OptionStyle, Side};
    use crate::model::utils::create_sample_option;
    use crate::utils::logger::setup_logger;
    use approx::assert_relative_eq;
    use num_traits::ToPrimitive;
    use tracing::info;

    #[test]
    fn test_delta_no_volatility_itm() {
        setup_logger();
        let option = create_sample_option(
            OptionStyle::Call,
            Side::Long,
            f2p!(150.0),
            f2p!(1.0),
            f2p!(150.0),
            ZERO,
        );
        let delta_value = delta(&option).unwrap();
        info!("Zero Volatility: {}", delta_value);
        assert_relative_eq!(delta_value.to_f64().unwrap(), 1.0, epsilon = 1e-8);
    }

    #[test]
    fn test_delta_no_volatility_otm() {
        setup_logger();
        let option = create_sample_option(
            OptionStyle::Call,
            Side::Long,
            f2p!(110.0),
            f2p!(1.0),
            f2p!(150.0),
            ZERO,
        );
        let delta_value = delta(&option).unwrap().to_f64().unwrap();
        info!("Zero Volatility: {}", delta_value);
        assert_relative_eq!(delta_value, ZERO, epsilon = 1e-8);
    }

    #[test]
    fn test_delta_no_volatility_itm_put() {
        setup_logger();
        let option = create_sample_option(
            OptionStyle::Put,
            Side::Long,
            f2p!(150.0),
            f2p!(1.0),
            f2p!(150.0),
            ZERO,
        );
        let delta_value = delta(&option).unwrap().to_f64().unwrap();
        info!("Zero Volatility: {}", delta_value);
        assert_relative_eq!(delta_value, -1.0, epsilon = 1e-8);
    }

    #[test]
    fn test_delta_no_volatility_otm_put() {
        setup_logger();
        let option = create_sample_option(
            OptionStyle::Put,
            Side::Long,
            f2p!(160.0),
            f2p!(1.0),
            f2p!(150.0),
            ZERO,
        );
        let delta_value = delta(&option).unwrap().to_f64().unwrap();
        info!("Zero Volatility: {}", delta_value);
        assert_relative_eq!(delta_value, ZERO, epsilon = 1e-8);
    }

    #[test]
    fn test_delta_no_volatility_itm_short() {
        setup_logger();
        let option = create_sample_option(
            OptionStyle::Call,
            Side::Short,
            f2p!(150.0),
            f2p!(1.0),
            f2p!(150.0),
            ZERO,
        );
        let delta_value = delta(&option).unwrap().to_f64().unwrap();
        info!("Zero Volatility: {}", delta_value);
        assert_relative_eq!(delta_value, -1.0, epsilon = 1e-8);
    }

    #[test]
    fn test_delta_no_volatility_otm_short() {
        setup_logger();
        let option = create_sample_option(
            OptionStyle::Call,
            Side::Short,
            f2p!(110.0),
            f2p!(1.0),
            f2p!(150.0),
            ZERO,
        );
        let delta_value = delta(&option).unwrap().to_f64().unwrap();
        info!("Zero Volatility: {}", delta_value);
        assert_relative_eq!(delta_value, ZERO, epsilon = 1e-8);
    }

    #[test]
    fn test_delta_no_volatility_itm_put_short() {
        setup_logger();
        let option = create_sample_option(
            OptionStyle::Put,
            Side::Short,
            f2p!(150.0),
            f2p!(1.0),
            f2p!(150.0),
            ZERO,
        );
        let delta_value = delta(&option).unwrap().to_f64().unwrap();
        info!("Zero Volatility: {}", delta_value);
        assert_relative_eq!(delta_value, 1.0, epsilon = 1e-8);
    }

    #[test]
    fn test_delta_no_volatility_otm_put_short() {
        setup_logger();
        let option = create_sample_option(
            OptionStyle::Put,
            Side::Short,
            f2p!(160.0),
            f2p!(1.0),
            f2p!(150.0),
            ZERO,
        );
        let delta_value = delta(&option).unwrap().to_f64().unwrap();
        info!("Zero Volatility: {}", delta_value);
        assert_relative_eq!(delta_value, ZERO, epsilon = 1e-8);
    }

    #[test]
    fn test_delta_deep_in_the_money_call() {
        setup_logger();
        let option = create_sample_option(
            OptionStyle::Call,
            Side::Long,
            f2p!(150.0),
            f2p!(1.0),
            f2p!(100.0),
            0.20,
        );
        let delta_value = delta(&option).unwrap().to_f64().unwrap();
        info!("Deep ITM Call Delta: {}", delta_value);
        assert_relative_eq!(delta_value, 0.9991784198733309, epsilon = 1e-8);
    }

    #[test]
    fn test_delta_deep_out_of_the_money_call() {
        let option = create_sample_option(
            OptionStyle::Call,
            Side::Long,
            f2p!(50.0),
            f2p!(1.0),
            f2p!(100.0),
            0.20,
        );
        let delta_value = delta(&option).unwrap().to_f64().unwrap();
        info!("Deep OTM Call Delta: {}", delta_value);
        assert_relative_eq!(delta_value, 2.0418256951423236e-33, epsilon = 1e-4);
    }

    #[test]
    fn test_delta_at_the_money_put() {
        let option = create_sample_option(
            OptionStyle::Put,
            Side::Long,
            f2p!(100.0),
            f2p!(1.0),
            f2p!(100.0),
            0.20,
        );
        let delta_value = delta(&option).unwrap().to_f64().unwrap();
        info!("ATM Put Delta: {}", delta_value);
        assert_relative_eq!(delta_value, -0.4596584975686261, epsilon = 1e-8);
    }

    #[test]
    fn test_delta_short_term_high_volatility() {
        let mut option = create_sample_option(
            OptionStyle::Call,
            Side::Long,
            f2p!(100.0),
            f2p!(1.0),
            f2p!(100.0),
            0.50,
        );
        option.expiration_date = ExpirationDate::Days(7.0);
        let delta_value = delta(&option).unwrap().to_f64().unwrap();
        info!("Short-term High Vol Call Delta: {}", delta_value);
        assert_relative_eq!(delta_value, 0.519229469584234, epsilon = 1e-4);
    }

    #[test]
    fn test_delta_long_term_low_volatility() {
        let mut option = create_sample_option(
            OptionStyle::Put,
            Side::Long,
            f2p!(100.0),
            f2p!(1.0),
            f2p!(100.0),
            0.10,
        );
        option.expiration_date = ExpirationDate::Days(365.0);
        let delta_value = delta(&option).unwrap().to_f64().unwrap();
        info!("Long-term Low Vol Put Delta: {}", delta_value);
        assert_relative_eq!(delta_value, -0.2882625994992622, epsilon = 1e-8);
    }
}

#[cfg(test)]
mod tests_gamma_equations {
    use super::*;
    use crate::f2p;
    use crate::model::types::{ExpirationDate, OptionStyle, Side};
    use crate::model::utils::create_sample_option;
    use crate::utils::logger::setup_logger;
    use approx::assert_relative_eq;
    use num_traits::ToPrimitive;
    use tracing::info;

    #[test]
    fn test_gamma_deep_in_the_money_call() {
        setup_logger();
        let option = create_sample_option(
            OptionStyle::Call,
            Side::Long,
            f2p!(150.0),
            f2p!(1.0),
            f2p!(120.0),
            0.2,
        );
        let gamma_value = gamma(&option).unwrap().to_f64().unwrap();
        info!("Deep ITM Call Gamma: {}", gamma_value);
        assert_relative_eq!(gamma_value, 0.000016049457791525, epsilon = 1e-8);
    }

    #[test]
    fn test_gamma_deep_out_of_the_money_call() {
        let option = create_sample_option(
            OptionStyle::Call,
            Side::Long,
            f2p!(50.0),
            f2p!(1.0),
            f2p!(100.0),
            0.20,
        );
        let gamma_value = gamma(&option).unwrap().to_f64().unwrap();
        info!("Deep OTM Call Gamma: {}", gamma_value);
        assert_relative_eq!(gamma_value, 0.0, epsilon = 1e-34);
    }

    #[test]
    fn test_gamma_at_the_money_put() {
        let option = create_sample_option(
            OptionStyle::Put,
            Side::Long,
            f2p!(100.0),
            f2p!(1.0),
            f2p!(100.0),
            0.20,
        );
        let gamma_value = gamma(&option).unwrap().to_f64().unwrap();
        info!("ATM Put Gamma: {}", gamma_value);
        assert_relative_eq!(gamma_value, 0.06917076441486919, epsilon = 1e-8);
    }

    #[test]
    fn test_gamma_short_term_high_volatility() {
        let mut option = create_sample_option(
            OptionStyle::Call,
            Side::Long,
            f2p!(100.0),
            f2p!(1.0),
            f2p!(100.0),
            0.50,
        );
        option.expiration_date = ExpirationDate::Days(7.0);
        let gamma_value = gamma(&option).unwrap().to_f64().unwrap();
        info!("Short-term High Vol Call Gamma: {}", gamma_value);
        assert_relative_eq!(gamma_value, 0.05753657912620555, epsilon = 1e-8);
    }

    #[test]
    fn test_gamma_long_term_low_volatility() {
        let mut option = create_sample_option(
            OptionStyle::Put,
            Side::Long,
            f2p!(100.0),
            f2p!(1.0),
            f2p!(100.0),
            0.10,
        );
        option.expiration_date = ExpirationDate::Days(365.0);
        let gamma_value = gamma(&option).unwrap().to_f64().unwrap();
        info!("Long-term Low Vol Put Gamma: {}", gamma_value);
        assert_relative_eq!(gamma_value, 0.033953150664723986, epsilon = 1e-8);
    }

    #[test]
    fn test_gamma_zero_volatility() {
        let option = create_sample_option(
            OptionStyle::Call,
            Side::Long,
            f2p!(100.0),
            f2p!(1.0),
            f2p!(100.0),
            0.0,
        );
        let gamma_value = gamma(&option).unwrap().to_f64().unwrap();
        info!("Zero Volatility Call Gamma: {}", gamma_value);
        assert_relative_eq!(gamma_value, 0.0, epsilon = 1e-8);
    }

    #[test]
    fn test_gamma_extreme_high_volatility() {
        let option = create_sample_option(
            OptionStyle::Put,
            Side::Short,
            f2p!(100.0),
            f2p!(1.0),
            f2p!(100.0),
            5.0,
        );
        let gamma_value = gamma(&option).unwrap().to_f64().unwrap();
        info!("Extreme High Volatility Put Gamma: {}", gamma_value);
        assert_relative_eq!(gamma_value, 0.002146478293943308, epsilon = 1e-8);
    }
}

#[cfg(test)]
mod tests_vega_equation {
    use super::*;
    use crate::model::types::{ExpirationDate, OptionType, Side};
    use crate::{f2p, Positive};
    use num_traits::ToPrimitive;

    fn create_test_option(
        underlying_price: Positive,
        strike_price: Positive,
        implied_volatility: f64,
        dividend_yield: f64,
        expiration_in_days: f64,
    ) -> Options {
        Options::new(
            OptionType::European,
            Side::Long,
            "TEST".to_string(),
            strike_price,
            ExpirationDate::Days(expiration_in_days),
            implied_volatility,
            f2p!(1.0), // Quantity
            underlying_price,
            0.05, // Risk-free rate
            OptionStyle::Call,
            dividend_yield,
            None, // No exotic params for this test
        )
    }

    #[test]
    fn test_vega_atm() {
        let option = create_test_option(f2p!(100.0), f2p!(100.0), 0.2, 0.0, 365.0);
        let vega = vega(&option).unwrap().to_f64().unwrap();
        let expected_vega = 63.68306511756191;
        assert!(
            (vega - expected_vega).abs() < 1e-5,
            "Vega ATM test failed: expected {}, got {}",
            expected_vega,
            vega
        );
    }

    #[test]
    fn test_vega_otm() {
        let option = create_test_option(f2p!(90.0), f2p!(100.0), 0.2, 0.0, 365.0);
        let vega = vega(&option).unwrap().to_f64().unwrap();
        let expected_vega = 38.68485587005888;
        assert!(
            (vega - expected_vega).abs() < 1e-5,
            "Vega OTM test failed: expected {}, got {}",
            expected_vega,
            vega
        );
    }

    #[test]
    fn test_vega_short_expiration() {
        let option = create_test_option(f2p!(100.0), f2p!(100.0), 0.2, 0.0, 1.0);
        let vega = vega(&option).unwrap().to_f64().unwrap();
        let expected_vega = 2.6553722124554757;
        assert!(
            (vega - expected_vega).abs() < 1e-5,
            "Vega short expiration test failed: expected {}, got {}",
            expected_vega,
            vega
        );
    }

    #[test]
    fn test_vega_with_dividends() {
        let option = create_test_option(f2p!(100.0), f2p!(100.0), 0.2, 0.03, 1.0);
        let vega = vega(&option).unwrap().to_f64().unwrap();
        let expected_vega = 2.6551539716535117;
        assert!(
            (vega - expected_vega).abs() < 1e-5,
            "Vega with dividends test failed: expected {}, got {}",
            expected_vega,
            vega
        );
    }

    #[test]
    fn test_vega_itm() {
        let option = create_test_option(f2p!(110.0), f2p!(100.0), 0.2, 0.0, 1.0);
        let vega = vega(&option).unwrap().to_f64().unwrap();
        let expected_vega = 5.757663148492351;
        assert!(
            (vega - expected_vega).abs() < 1e-5,
            "Vega ITM test failed: expected {}, got {}",
            expected_vega,
            vega
        );
    }
}

#[cfg(test)]
mod tests_rho_equations {
    use super::*;
    use crate::f2p;
    use crate::model::types::{ExpirationDate, OptionStyle, OptionType, Side};
    use approx::assert_relative_eq;
    use num_traits::ToPrimitive;

    fn create_test_option(style: OptionStyle) -> Options {
        Options {
            option_type: OptionType::European,
            side: Side::Long,
            underlying_symbol: "TEST".to_string(),
            strike_price: f2p!(100.0),
            expiration_date: ExpirationDate::Days(365.0),
            implied_volatility: 0.2,
            quantity: f2p!(1.0),
            underlying_price: f2p!(100.0),
            risk_free_rate: 0.05,
            option_style: style,
            dividend_yield: 0.0,
            exotic_params: None,
        }
    }

    #[test]
    fn test_rho_call_option() {
        let option = create_test_option(OptionStyle::Call);
        let result = rho(&option).unwrap().to_f64().unwrap();
        assert_relative_eq!(result, 53.232481545376345, epsilon = 1e-8);
    }

    #[test]
    fn test_rho_put_option() {
        let option = create_test_option(OptionStyle::Put);
        let result = rho(&option).unwrap().to_f64().unwrap();
        assert_relative_eq!(result, -41.89046090469506, epsilon = 1e-8);
    }

    #[test]
    fn test_rho_zero_time_to_expiry() {
        let mut option = create_test_option(OptionStyle::Call);
        option.expiration_date = ExpirationDate::Days(0.0);
        let result = rho(&option).is_err();
        assert!(result);
    }

    #[test]
    fn test_rho_zero_risk_free_rate() {
        let mut option = create_test_option(OptionStyle::Call);
        option.risk_free_rate = 0.0;
        let result = rho(&option).unwrap().to_f64().unwrap();
        assert_relative_eq!(result, 46.0172162722971, epsilon = 1e-8);
    }

    #[test]
    fn test_rho_deep_out_of_money_call() {
        let mut option = create_test_option(OptionStyle::Call);
        option.strike_price = f2p!(1000.0);
        let result = rho(&option).unwrap().to_f64().unwrap();
        assert_relative_eq!(result, 0.0, epsilon = 1e-8);
    }

    #[test]
    fn test_rho_deep_out_of_money_put() {
        let mut option = create_test_option(OptionStyle::Put);
        option.strike_price = f2p!(1.0);
        let result = rho(&option).unwrap().to_f64().unwrap();
        assert_relative_eq!(result, 0.0, epsilon = 1e-8);
    }

    #[test]
    fn test_rho_high_volatility() {
        let mut option = create_test_option(OptionStyle::Call);
        option.implied_volatility = 1.0;
        let result = rho(&option).unwrap().to_f64().unwrap();
        assert_relative_eq!(result, 31.043868837728198, epsilon = 0.0001);
    }
}

#[cfg(test)]
mod tests_theta_long_equations {
    use super::*;
    use crate::f2p;
    use crate::model::types::{ExpirationDate, Side};
    use crate::model::utils::create_sample_option;
    use approx::assert_relative_eq;
    use num_traits::ToPrimitive;

    #[test]
    fn test_theta_call_option() {
        // Create a sample call option
        let option = create_sample_option(
            OptionStyle::Call,
            Side::Long,
            f2p!(150.0), // underlying price
            f2p!(1.0),   // quantity
            f2p!(155.0), // strike price
            0.20,        // implied volatility
        );

        // Expected theta value for a call option (precomputed or from known source)
        let expected_theta = -20.487619692230428;

        // Compute the theta value using the function
        let calculated_theta = theta(&option).unwrap().to_f64().unwrap();

        // Assert the calculated theta is close to the expected value
        assert_relative_eq!(calculated_theta, expected_theta, epsilon = 1e-8);
    }

    #[test]
    fn test_theta_put_option() {
        // Create a sample put option
        let option = create_sample_option(
            OptionStyle::Put,
            Side::Long,
            f2p!(150.0), // underlying price
            f2p!(1.0),   // quantity
            f2p!(145.0), // strike price
            0.25,        // implied volatility
        );

        // Expected theta value for a put option (precomputed or from known source)
        let expected_theta = -20.395533137333533;

        // Compute the theta value using the function
        let calculated_theta = theta(&option).unwrap().to_f64().unwrap();

        // Assert the calculated theta is close to the expected value
        assert_relative_eq!(calculated_theta, expected_theta, epsilon = 1e-8);
    }

    #[test]
    fn test_theta_call_option_near_expiry() {
        // Create a sample call option near expiry
        let mut option = create_sample_option(
            OptionStyle::Call,
            Side::Long,
            f2p!(150.0), // underlying price
            f2p!(1.0),   // quantity
            f2p!(150.0), // strike price
            0.15,        // implied volatility
        );
        option.expiration_date = ExpirationDate::Days(1.0); // Option close to expiry

        // Expected theta value for a near-expiry call option (precomputed)
        let expected_theta = -88.75028112939226;

        // Compute the theta value using the function
        let calculated_theta = theta(&option).unwrap().to_f64().unwrap();

        // Assert the calculated theta is close to the expected value
        assert_relative_eq!(calculated_theta, expected_theta, epsilon = 1e-8);
    }

    #[test]
    fn test_theta_put_option_far_from_expiry() {
        // Create a sample put option far from expiry
        let mut option = create_sample_option(
            OptionStyle::Put,
            Side::Long,
            f2p!(140.0), // underlying price
            f2p!(1.0),   // quantity
            f2p!(130.0), // strike price
            0.30,        // implied volatility
        );
        option.expiration_date = ExpirationDate::Days(365.0); // Option far from expiry

        // Expected theta value for a far-expiry put option (precomputed)
        let expected_theta = -5.024569007193639;

        // Compute the theta value using the function
        let calculated_theta = theta(&option).unwrap().to_f64().unwrap();

        // Assert the calculated theta is close to the expected value
        assert_relative_eq!(calculated_theta, expected_theta, epsilon = 1e-8);
    }
}

#[cfg(test)]
mod tests_theta_short_equations {
    use super::*;
    use crate::f2p;
    use crate::model::types::{ExpirationDate, Side};
    use crate::model::utils::create_sample_option;
    use approx::assert_relative_eq;
    use num_traits::ToPrimitive;

    #[test]
    fn test_theta_short_call_option() {
        // Create a sample short call option
        let option = create_sample_option(
            OptionStyle::Call,
            Side::Short,
            f2p!(150.0), // underlying price
            f2p!(1.0),   // quantity
            f2p!(155.0), // strike price
            0.20,        // implied volatility
        );

        // Expected theta value for a short call option (precomputed or from known source)
        let expected_theta = -20.487619692230428;

        // Compute the theta value using the function
        let calculated_theta = theta(&option).unwrap().to_f64().unwrap();

        // Assert the calculated theta is close to the expected value
        assert_relative_eq!(calculated_theta, expected_theta, epsilon = 1e-8);
    }

    #[test]
    fn test_theta_short_put_option() {
        // Create a sample short put option
        let option = create_sample_option(
            OptionStyle::Put,
            Side::Short,
            f2p!(150.0), // underlying price
            f2p!(1.0),   // quantity
            f2p!(145.0), // strike price
            0.25,        // implied volatility
        );

        // Expected theta value for a short put option (precomputed or from known source)
        let expected_theta = -20.395533137333533;

        // Compute the theta value using the function
        let calculated_theta = theta(&option).unwrap().to_f64().unwrap();

        // Assert the calculated theta is close to the expected value
        assert_relative_eq!(calculated_theta, expected_theta, epsilon = 1e-8);
    }

    #[test]
    fn test_theta_short_call_option_near_expiry() {
        // Create a sample short call option near expiry
        let mut option = create_sample_option(
            OptionStyle::Call,
            Side::Short,
            f2p!(150.0), // underlying price
            f2p!(1.0),   // quantity
            f2p!(150.0), // strike price
            0.15,        // implied volatility
        );
        option.expiration_date = ExpirationDate::Days(1.0); // Option close to expiry

        // Expected theta value for a short near-expiry call option (precomputed)
        let expected_theta = -88.75028112939226;

        // Compute the theta value using the function
        let calculated_theta = theta(&option).unwrap().to_f64().unwrap();

        // Assert the calculated theta is close to the expected value
        assert_relative_eq!(calculated_theta, expected_theta, epsilon = 1e-8);
    }

    #[test]
    fn test_theta_short_put_option_far_from_expiry() {
        // Create a sample short put option far from expiry
        let mut option = create_sample_option(
            OptionStyle::Put,
            Side::Short,
            f2p!(140.0), // underlying price
            f2p!(1.0),   // quantity
            f2p!(130.0), // strike price
            0.30,        // implied volatility
        );
        option.expiration_date = ExpirationDate::Days(365.0); // Option far from expiry

        // Expected theta value for a far-expiry short put option (precomputed)
        let expected_theta = -5.024569007193639;

        // Compute the theta value using the function
        let calculated_theta = theta(&option).unwrap().to_f64().unwrap();

        // Assert the calculated theta is close to the expected value
        assert_relative_eq!(calculated_theta, expected_theta, epsilon = 1e-8);
    }
}
