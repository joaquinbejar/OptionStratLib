use crate::error::PricingError;
use crate::kernels::discount_factor;
use crate::pricing::utils::wiener_sqrt_dt;
use num_traits::FromPrimitive;
use optionstratlib_core::model::Options;
use optionstratlib_core::model::Positive;
use optionstratlib_core::model::decimal::{
    d_add, d_div, d_mul, d_sub, decimal_to_f64, finite_decimal,
};
use optionstratlib_core::model::types::{OptionStyle, Side};
use rand::Rng;
use rand_distr::{Distribution, Normal};
use rust_decimal::Decimal;
use std::num::NonZeroUsize;
use tracing::instrument;

/// This function performs Monte Carlo simulation to price an option.
///
/// # Arguments
///
/// * `option` - An `Options` struct containing the option's parameters, such as underlying price, strike price, risk-free rate, implied volatility, and expiration date.
/// * `steps` - An integer indicating the number of time steps in the simulation.
/// * `simulations` - An integer indicating the number of Monte Carlo simulations to run.
/// * `rng` - The generator every Wiener increment is drawn from, one standard
///   normal sample per step per path. A seeded generator such as
///   [`optionstratlib_core::utils::deterministic_rng`] makes the estimate
///   reproducible; pass `&mut rand::rng()` to draw from the thread-local RNG.
///
/// # Returns
///
/// * The estimated price of one unit of the option, signed by its side as
///   [`crate::pricing::black_scholes`] signs it: a long position returns the
///   discounted mean payoff, a short position its negation. The quantity is
///   not applied.
///
/// # Description
///
/// The function follows the below steps:
///
/// 1. Calculate the time increment `dt` based on the number of steps.
/// 2. Initialize a sum variable `payoff_sum` to accumulate the payoffs from each simulation.
/// 3. Loop through the number of simulations:
///     - For each simulation, initialize the stock price `st` to the underlying price.
///     - Loop through the number of steps:
///         - Calculate a Wiener process increment `w`.
///         - Update the stock price `st` using the Euler discretisation of geometric Brownian motion,
///           `st <- st * (1 + (r - q) dt + sigma dW)`, with `q` the option's dividend yield.
///     - Calculate the payoff of the option for this simulation:
///       `max(st - strike_price, 0)` for a call, `max(strike_price - st, 0)`
///       for a put.
///     - Add the payoff to the `payoff_sum`.
/// 4. Discount the average payoff to its present value and negate it for a
///    short position. Until #864 every option was paid as a long call.
///
/// # Errors
///
/// Returns `PricingError::ExpirationDate` when the option's
/// expiration cannot be converted to a positive year fraction, and
/// `PricingError::MethodError` when the GBM discretisation
/// encounters a non-finite value (e.g. volatility overflow) or when
/// the terminal payoff averaging produces a non-representable
/// `Decimal`.
#[instrument(skip(option, rng), fields(
    steps = steps.get(),
    simulations = simulations.get(),
    strike = %option.strike_price,
    style = ?option.option_style,
    side = ?option.side,
))]
pub fn monte_carlo_option_pricing<R: Rng + ?Sized>(
    option: &Options,
    steps: NonZeroUsize,       // Number of time steps per path
    simulations: NonZeroUsize, // Number of Monte Carlo simulations
    rng: &mut R,
) -> Result<Decimal, PricingError> {
    let steps_raw = steps.get();
    let simulations_raw = simulations.get();
    // `Positive / f64` aborted when the step count had no `Decimal` image;
    // the same conversion and division, checked (#788).
    let steps_dec = Decimal::from_f64(steps_raw as f64)
        .ok_or_else(|| PricingError::non_finite("pricing::monte_carlo::steps", steps_raw as f64))?;
    let dt = option
        .expiration_date
        .get_years()?
        .checked_div_dec(steps_dec)?;
    let mut payoff_sum = 0.0;

    let dt_dec = dt.to_dec();
    // Risk-neutral drift `(r - q) dt`. The dividend yield was missing until
    // #756, which overpriced calls on a dividend-paying underlying.
    let carry = d_sub(
        option.risk_free_rate,
        option.dividend_yield.to_dec(),
        "pricing::monte_carlo::gbm::carry",
    )?;
    let drift = d_mul(carry, dt_dec, "pricing::monte_carlo::gbm::drift")?;
    // The path runs in `f64` (#859 P3b): the inputs and the loop-invariant
    // factors are converted once, each step is the same Euler update the
    // `Decimal` path took, `st <- st * ((1 + (r - q) dt) + sigma * z * sqrt(dt))`,
    // on the same standard normal draws in the same order, and the payoff is
    // checked for finiteness before it is summed. Against the `Decimal` path
    // the price moves by about 1e-15 relative (tests_f64_path_kernel).
    let one_plus_drift = decimal_to_f64(d_add(
        Decimal::ONE,
        drift,
        "pricing::monte_carlo::gbm::growth_drift",
    )?)?;
    let sigma = decimal_to_f64(option.implied_volatility.to_dec())?;
    let spot = decimal_to_f64(option.underlying_price.to_dec())?;
    let strike = decimal_to_f64(option.strike_price.to_dec())?;
    // Loop invariant: computed inside every step until #859, where it was
    // about two thirds of the cost of a step.
    let sqrt_dt = decimal_to_f64(wiener_sqrt_dt(dt_dec)?)?;
    let normal = Normal::new(0.0, 1.0).map_err(|e| {
        PricingError::from(optionstratlib_core::error::DecimalError::arithmetic_error(
            "Normal::new(0.0, 1.0)",
            &e.to_string(),
        ))
    })?;
    for _ in 0..simulations_raw {
        let mut st = spot;
        for _ in 0..steps_raw {
            let sample = normal.sample(rng);
            let w = sample * sqrt_dt;
            let diffusion = sigma * w;
            st *= one_plus_drift + diffusion;
        }
        if !st.is_finite() {
            return Err(PricingError::non_finite(
                "pricing::monte_carlo::gbm::step",
                st,
            ));
        }
        // The payoff of the option's style; it was always the call payoff
        // until #864, so a put was priced as a call.
        let payoff = match option.option_style {
            OptionStyle::Call => st - strike,
            OptionStyle::Put => strike - st,
        }
        .max(0.0);
        if !payoff.is_finite() {
            return Err(PricingError::non_finite(
                "pricing::monte_carlo::gbm::payoff",
                payoff,
            ));
        }
        payoff_sum += payoff;
    }
    // Average value of the payoffs discounted to present value.
    // Guard every `f64` boundary against NaN / ±∞ so saturation on
    // the rate, the discount exponent, or the final average surfaces
    // a tagged `PricingError::NonFinite` instead of silently collapsing
    // to `Decimal::ZERO` through the `f2d!` cast.
    let rate_f64 = decimal_to_f64(option.risk_free_rate)
        .map_err(|_| PricingError::non_finite("pricing::monte_carlo::rate_f64::cast", f64::NAN))?;
    if !rate_f64.is_finite() {
        return Err(PricingError::non_finite(
            "pricing::monte_carlo::rate_f64",
            rate_f64,
        ));
    }
    let years = decimal_to_f64(option.expiration_date.get_years()?.to_dec())?;
    if !years.is_finite() {
        return Err(PricingError::non_finite(
            "pricing::monte_carlo::years",
            years,
        ));
    }
    let discount = (-rate_f64 * years).exp(); // scan-banned: allow -- f64 `exp`: returns inf/NaN on overflow, it does not abort; the non-finite value is rejected at the `Decimal` boundary
    if !discount.is_finite() {
        return Err(PricingError::non_finite(
            "pricing::monte_carlo::discount",
            discount,
        ));
    }
    let average_payoff = (payoff_sum / simulations_raw as f64) * discount;
    if !average_payoff.is_finite() {
        return Err(PricingError::non_finite(
            "pricing::monte_carlo::average_payoff",
            average_payoff,
        ));
    }
    let long_price = finite_decimal(average_payoff).ok_or_else(|| {
        PricingError::non_finite("pricing::monte_carlo::average_payoff::cast", average_payoff)
    })?;
    // The side is applied once, to the discounted mean, as `black_scholes`
    // applies it: a short position is the negated long price (#864).
    Ok(match option.side {
        Side::Long => long_price,
        Side::Short => -long_price,
    })
}

/// Estimates the price of a financial option using the Monte Carlo simulation method.
///
/// # Parameters
/// - `option`: A reference to an `Options` object that represents the option being evaluated.
///   This object contains necessary details such as risk-free rate, dividend yield,
///   expiration date, and the payoff calculation logic.
/// - `final_prices`: A slice of `Positive` values representing the simulated final prices of
///   the underlying asset at the option's expiration. The length of this slice
///   corresponds to the number of simulations.
///
/// # Returns
/// - `Result<Positive, PricingError>`: Returns a `Positive` value encapsulating the estimated
///   option price calculated using the Monte Carlo method. If an error occurs during intermediate
///   calculations (e.g., getting the expiration year), it returns a `PricingError`.
///
/// # How it Works
/// 1. The number of simulations is determined by the length of the `final_prices` slice. If the slice
///    is empty, the function immediately returns a price of `Positive::ZERO`.
/// 2. Calculates the discount factor `e^(-rT)` from the risk-free rate and the time to
///    expiration. The dividend yield does not enter it: it belongs to the drift `r - q` of the
///    risk-neutral law that generated `final_prices`, which is the caller's responsibility.
/// 3. For each simulated final price in the `final_prices` slice:
///    - Compute the payoff using the `option.payoff_at_price` method.
///    - Accumulate the total payoff across all simulations.
/// 4. Compute the average payoff by dividing the total payoff by the number of simulations.
///    The average payoff is then discounted using the calculated discount factor.
/// 5. Return the discounted average payoff as the estimated option price.
///
/// # Errors
/// - Propagates any error from `option.expiration_date.get_years()?` (e.g.,
///   invalid expiration date).
/// - Returns `PricingError::method_error` when `num_simulations` cannot be
///   represented as a `Decimal` (effectively unreachable for valid `usize`
///   inputs but surfaced explicitly for completeness).
/// - Propagates a per-simulation `option.payoff_at_price(...)` failure as
///   `PricingError::Options` (a payoff out of the `Decimal` range).
/// - Returns `PricingError::Decimal` when the payoff sum, the mean or the
///   discounting leaves the `Decimal` range.
///
/// This function assumes that the `Options` struct and `Positive` type
/// are implemented elsewhere in the codebase and provide necessary functionality (e.g., payoff calculation).
pub fn price_option_monte_carlo(
    option: &Options,
    final_prices: &[Positive],
) -> Result<Positive, PricingError> {
    // The number of simulations is the length of the final prices vector
    let num_simulations = final_prices.len();

    if num_simulations == 0 {
        return Ok(Positive::ZERO);
    }

    // Discount at the risk-free rate only (#651). The dividend yield shapes
    // the risk-neutral terminal law the caller supplies (drift `r - q`); it
    // never enters the discount factor `e^(-rT)`.
    let discount_factor = discount_factor(
        option.risk_free_rate,
        option.expiration_date.get_years()?.to_dec(),
        "pricing::monte_carlo::discount_exponent",
        "pricing::monte_carlo::discount_factor",
    )?;

    // Calculate payoff for each final price and sum them. A payoff that
    // cannot be evaluated is an error (#639); it used to count as a zero
    // payoff for that path, dragging the mean down silently.
    let total_payoff = final_prices
        .iter()
        .try_fold(Decimal::ZERO, |sum, final_price| {
            let payoff = option.payoff_at_price(final_price)?;
            Ok::<Decimal, PricingError>(d_add(sum, payoff, "pricing::monte_carlo::total_payoff")?)
        })?;

    // Average payoff discounted to present value. Both the mean and the
    // discounting are fused monetary flows, so they go through the checked
    // helpers; `d_div` applies banker's rounding at `DIV_DEFAULT_SCALE`.
    let n_dec = Decimal::from_usize(num_simulations).ok_or_else(|| {
        PricingError::method_error(
            "price_option_monte_carlo",
            &format!("num_simulations not representable as Decimal: {num_simulations}"),
        )
    })?;
    let mean_payoff = d_div(total_payoff, n_dec, "pricing::monte_carlo::mean")?;
    let avg_payoff = d_mul(discount_factor, mean_payoff, "pricing::monte_carlo::price")?;
    // `|x|` is a valid `Positive` for every `Decimal`, so this cannot fail;
    // it propagates rather than falling back (#639).
    Ok(Positive::new_decimal(avg_payoff.abs())?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use optionstratlib_core::constants::DAYS_IN_A_YEAR;
    use optionstratlib_core::constants::ZERO;
    use optionstratlib_core::model::ExpirationDate;
    use optionstratlib_core::model::types::{OptionStyle, OptionType, Side};
    use optionstratlib_core::utils::{DETERMINISTIC_RNG_DEFAULT_SEED, deterministic_rng};
    use optionstratlib_core::{assert_decimal_eq, f2du};
    use optionstratlib_core::{model::Positive, pos_or_panic};
    use rust_decimal::MathematicalOps;
    use rust_decimal_macros::dec;

    /// Price of `create_test_option` over 12 steps and 500 paths drawn from
    /// `deterministic_rng(DETERMINISTIC_RNG_DEFAULT_SEED)`.
    /// The average is accumulated and discounted in `f64` (the platform `exp`
    /// may differ in the last ulp), so it is compared to `1e-9`.
    const PINNED_SEEDED_PRICE: Decimal = dec!(9.88846375599113);

    fn create_test_option() -> Options {
        Options {
            option_type: OptionType::European,
            side: Side::Long,
            underlying_symbol: "TEST".to_string(),
            strike_price: Positive::HUNDRED,
            expiration_date: ExpirationDate::Days(DAYS_IN_A_YEAR), // 1 year
            implied_volatility: pos_or_panic!(0.2),
            quantity: Positive::ONE,
            contract_size: Positive::ONE,
            underlying_price: Positive::HUNDRED,
            risk_free_rate: dec!(0.05),
            option_style: OptionStyle::Call,
            dividend_yield: Positive::ZERO,
            exotic_params: None,
        }
    }

    #[test]
    fn test_monte_carlo_option_pricing_at_the_money() {
        let option = create_test_option();
        let price = monte_carlo_option_pricing(
            &option,
            optionstratlib_core::nz!(252),
            optionstratlib_core::nz!(1000),
            &mut deterministic_rng(DETERMINISTIC_RNG_DEFAULT_SEED),
        )
        .unwrap();
        // The price should be close to the Black-Scholes price for these parameters
        let expected_price = dec!(9.100); // Calculated using Black-Scholes
        assert_decimal_eq!(price, expected_price, dec!(5));
    }

    #[test]
    fn test_monte_carlo_option_pricing_zero_volatility() {
        let mut option = create_test_option();
        option.implied_volatility = Positive::ZERO;
        let price = monte_carlo_option_pricing(
            &option,
            optionstratlib_core::nz!(25),
            optionstratlib_core::nz!(100),
            &mut deterministic_rng(DETERMINISTIC_RNG_DEFAULT_SEED),
        )
        .unwrap();
        let expected_price = f64::max(
            (option.underlying_price - option.strike_price * (-option.risk_free_rate).exp()).into(),
            ZERO,
        );
        assert_decimal_eq!(price, f2du!(expected_price).unwrap(), dec!(0.1));
    }

    #[test]
    fn test_monte_carlo_option_pricing_high_volatility() {
        let mut option = create_test_option();
        option.implied_volatility = pos_or_panic!(0.5);
        let price = monte_carlo_option_pricing(
            &option,
            optionstratlib_core::nz!(252),
            optionstratlib_core::nz!(100),
            &mut deterministic_rng(DETERMINISTIC_RNG_DEFAULT_SEED),
        )
        .unwrap();
        // The price should be higher with higher volatility
        assert!(price > dec!(10.0));
    }

    #[test]
    fn test_monte_carlo_option_pricing_short_expiration() {
        let mut option = create_test_option();
        option.expiration_date = ExpirationDate::Days(pos_or_panic!(30.0)); // 30 days
        let price = monte_carlo_option_pricing(
            &option,
            optionstratlib_core::nz!(30),
            optionstratlib_core::nz!(100),
            &mut deterministic_rng(DETERMINISTIC_RNG_DEFAULT_SEED),
        )
        .unwrap();
        // The price should be lower for a shorter expiration
        assert!(price < dec!(5.0));
    }

    #[test]
    fn test_monte_carlo_option_pricing_same_seed_identical_price() {
        let option = create_test_option();
        let price = |seed: u64| {
            monte_carlo_option_pricing(
                &option,
                optionstratlib_core::nz!(50),
                optionstratlib_core::nz!(200),
                &mut deterministic_rng(seed),
            )
            .unwrap()
        };
        assert_eq!(price(7), price(7));
        assert_ne!(price(7), price(8));
    }

    #[test]
    fn test_monte_carlo_option_pricing_seeded_regression_pinned() {
        let option = create_test_option();
        let price = monte_carlo_option_pricing(
            &option,
            optionstratlib_core::nz!(12),
            optionstratlib_core::nz!(500),
            &mut deterministic_rng(DETERMINISTIC_RNG_DEFAULT_SEED),
        )
        .unwrap();
        assert_decimal_eq!(price, PINNED_SEEDED_PRICE, dec!(1e-9));
    }

    #[test]
    fn test_monte_carlo_option_pricing_seeded_converges_to_black_scholes() {
        // Black-Scholes call, S = K = 100, r = 5%, sigma = 20%, T = 1:
        // 10.4506. The ATM payoff standard deviation is about 14.7, so with
        // 4000 paths the standard error is about 0.23; the band is 4 of
        // them, wide enough to also absorb the Euler bias of 12 steps.
        let option = create_test_option();
        let price = monte_carlo_option_pricing(
            &option,
            optionstratlib_core::nz!(12),
            optionstratlib_core::nz!(4000),
            &mut deterministic_rng(DETERMINISTIC_RNG_DEFAULT_SEED),
        )
        .unwrap();
        assert_decimal_eq!(price, dec!(10.4506), dec!(0.92));
    }

    #[test]
    fn test_monte_carlo_option_pricing_dividend_yield_converges_to_black_scholes() {
        use crate::pricing::black_scholes_model::black_scholes;
        // #756: with q = 3% the drift is r - q, so the estimate converges to
        // the Black-Scholes call with the same q (8.6525). The payoff
        // standard deviation is about 13.0, so with 20 000 paths the
        // standard error is about 0.092; the band is 4.5 of them and absorbs
        // the Euler bias of 12 steps. Before the fix the estimate sat near
        // the q = 0 price 10.45, about 20 standard errors away.
        let mut option = create_test_option();
        option.dividend_yield = pos_or_panic!(0.03);
        let price = monte_carlo_option_pricing(
            &option,
            optionstratlib_core::nz!(12),
            optionstratlib_core::nz!(20_000),
            &mut deterministic_rng(DETERMINISTIC_RNG_DEFAULT_SEED),
        )
        .unwrap();
        let reference = black_scholes(&option).unwrap();
        assert_decimal_eq!(price, reference, dec!(0.41));
    }

    /// Seeded estimate over 12 steps and 20 000 paths, the setting of the
    /// dividend-yield convergence test above.
    fn seeded_price(option: &Options) -> Decimal {
        monte_carlo_option_pricing(
            option,
            optionstratlib_core::nz!(12),
            optionstratlib_core::nz!(20_000),
            &mut deterministic_rng(DETERMINISTIC_RNG_DEFAULT_SEED),
        )
        .unwrap()
    }

    #[test]
    fn test_monte_carlo_option_pricing_put_converges_to_black_scholes() {
        use crate::pricing::black_scholes_model::black_scholes;
        // #864: a put was paid `max(S_T - K, 0)` and priced as the call,
        // 10.45 here. Black-Scholes put, S = K = 100, r = 5%, sigma = 20%,
        // T = 1: 5.5735. The discounted put payoff has a standard deviation
        // of about 8.7, so with 20 000 paths the standard error is about
        // 0.062; the band is 4.5 of them and absorbs the Euler bias of 12
        // steps.
        let mut option = create_test_option();
        option.option_style = OptionStyle::Put;
        let reference = black_scholes(&option).unwrap();
        assert_decimal_eq!(reference, dec!(5.5735), dec!(0.0001));
        assert_decimal_eq!(seeded_price(&option), reference, dec!(0.28));
    }

    #[test]
    fn test_monte_carlo_option_pricing_call_converges_to_black_scholes_20k() {
        use crate::pricing::black_scholes_model::black_scholes;
        // The call at the same setting: payoff standard deviation about
        // 14.5, standard error about 0.10, band 4.5 of them.
        let option = create_test_option();
        let reference = black_scholes(&option).unwrap();
        assert_decimal_eq!(seeded_price(&option), reference, dec!(0.46));
    }

    #[test]
    fn test_monte_carlo_option_pricing_put_call_parity() {
        // C - P = S e^(-qT) - K e^(-rT). Both estimates run on the same
        // seeded paths, so their difference is exactly the discounted mean
        // of S_T - K and the only error left is that of the sample mean of
        // S_T: standard deviation about 20, standard error about 0.14 over
        // 20 000 paths; the band is 4.5 of them. With q = 3% the forward
        // term is 100 e^(-0.03) - 100 e^(-0.05) = 1.9216.
        let mut call = create_test_option();
        call.dividend_yield = pos_or_panic!(0.03);
        let mut put = call.clone();
        put.option_style = OptionStyle::Put;
        let forward = dec!(100) * (-dec!(0.03)).exp() - dec!(100) * (-dec!(0.05)).exp();
        assert_decimal_eq!(forward, dec!(1.9216), dec!(0.0001));
        let parity = seeded_price(&call) - seeded_price(&put);
        assert_decimal_eq!(parity, forward, dec!(0.61));
    }

    #[test]
    fn test_monte_carlo_option_pricing_short_is_negated_long() {
        // #864: a short position was priced as the long one. The side is
        // applied once, to the discounted mean, so on the same seeded paths
        // the short price is exactly the negated long price, as in
        // `black_scholes`.
        for style in [OptionStyle::Call, OptionStyle::Put] {
            let mut long = create_test_option();
            long.option_style = style;
            let mut short = long.clone();
            short.side = Side::Short;
            let long_price = seeded_price(&long);
            assert!(long_price > Decimal::ZERO, "{style:?}: {long_price}");
            assert_eq!(seeded_price(&short), -long_price, "{style:?}");
        }
    }
}

#[cfg(test)]
mod tests_price_option_monte_carlo {
    use super::*;
    use optionstratlib_core::model::utils::create_sample_option;
    use optionstratlib_core::model::{ExpirationDate, OptionStyle, Side};
    use optionstratlib_core::{assert_pos_relative_eq, model::Positive, pos_or_panic};
    use rust_decimal_macros::dec;

    #[test]
    fn test_empty_prices_returns_zero() {
        // Arrange
        let option = create_sample_option(
            OptionStyle::Call,
            Side::Long,
            Positive::HUNDRED,
            Positive::ONE,
            Positive::HUNDRED,
            pos_or_panic!(0.2),
        );
        let empty_prices = &[];

        // Act
        let result = price_option_monte_carlo(&option, empty_prices);

        // Assert
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Positive::ZERO);
    }

    /// A path whose quantity-scaled payoff leaves the `Decimal` range is an
    /// error; it used to count as a zero payoff and drag the mean down
    /// silently (#639).
    #[test]
    fn test_price_option_monte_carlo_unrepresentable_payoff_returns_error() {
        let mut option = create_sample_option(
            OptionStyle::Call,
            Side::Long,
            Positive::HUNDRED,
            pos_or_panic!(1000.0),
            Positive::HUNDRED,
            pos_or_panic!(0.2),
        );
        option.expiration_date = ExpirationDate::Days(pos_or_panic!(365.0));
        let prices = vec![pos_or_panic!(110.0), pos_or_panic!(1e27)];
        let result = price_option_monte_carlo(&option, &prices);
        assert!(
            matches!(result, Err(PricingError::Options(_))),
            "unrepresentable payoff: {result:?}"
        );
    }

    #[test]
    fn test_call_option_pricing() {
        // Arrange
        let mut option = create_sample_option(
            OptionStyle::Call,
            Side::Long,
            Positive::HUNDRED,
            Positive::ONE,
            Positive::HUNDRED,
            pos_or_panic!(0.2),
        );

        option.risk_free_rate = dec!(0.05);
        option.dividend_yield = pos_or_panic!(0.02);
        option.expiration_date = ExpirationDate::Days(pos_or_panic!(365.0));

        // Setup simulated prices and expected payoffs
        let prices = vec![
            pos_or_panic!(110.0),
            pos_or_panic!(90.0),
            pos_or_panic!(105.0),
        ];

        // Act
        let result = price_option_monte_carlo(&option, &prices);
        assert!(result.is_ok());
        // Mean payoff (10 + 0 + 5) / 3 = 5 discounted at r alone (#651):
        // 5 e^(-0.05) = 4.75614712. It was 5 e^(-0.03) = 4.85222766 while
        // the pricer discounted at r - q.
        assert_pos_relative_eq!(
            result.unwrap(),
            pos_or_panic!(4.75614712),
            pos_or_panic!(0.001)
        );
    }

    // #[test]
    // fn test_put_option_pricing() {
    //     // Arrange
    //     let mut mock_option = MockOptions::new();
    //
    //     // Setup expiration date (1 year from now)
    //     let expiration = Utc::now().date_naive() + chrono::Duration::days(365);
    //     mock_option.risk_free_rate = 0.05;
    //     mock_option.dividend_yield = 0.02;
    //     mock_option.expiration_date = expiration;
    //
    //     // Setup simulated prices and expected payoffs
    //     let prices = vec![
    //         Positive::from_f64(90.0),
    //         Positive::from_f64(110.0),
    //         Positive::from_f64(95.0),
    //     ];
    //
    //     // For a put option with strike 100:
    //     // Payoffs would be: 10.0, 0.0, 5.0
    //     mock_option.expect_payoff_at_price()
    //         .with(eq(Positive::from_f64(90.0)))
    //         .returning(|_| Ok(Decimal::from_str("10.0").unwrap()));
    //
    //     mock_option.expect_payoff_at_price()
    //         .with(eq(Positive::from_f64(110.0)))
    //         .returning(|_| Ok(Decimal::from_str("0.0").unwrap()));
    //
    //     mock_option.expect_payoff_at_price()
    //         .with(eq(Positive::from_f64(95.0)))
    //         .returning(|_| Ok(Decimal::from_str("5.0").unwrap()));
    //
    //     // Act
    //     let result = price_option_monte_carlo(&mock_option, &prices);
    //
    //     // Assert
    //     assert!(result.is_ok());
    //
    //     // Expected: avg payoff = (10 + 0 + 5)/3 = 5.0
    //     // Discount factor = exp(-(0.05-0.02)*1) = exp(-0.03) ≈ 0.9704
    //     // Expected price = 5.0 * 0.9704 ≈ 4.852
    //     let expected = Positive::from_f64(4.852);
    //
    //     // Using approximate comparison due to floating-point calculations
    //     let diff = (result.unwrap().0 - expected.0).abs();
    //     assert!(diff < Decimal::from_str("0.001").unwrap(),
    //             "Expected close to {}, got {}", expected.0, result.unwrap().0);
    // }
    //
    // #[test]
    // fn test_expiration_date_error_handling() {
    //     // Arrange
    //     let mut mock_option = MockOptions::new();
    //
    //     // Setup with a problematic expiration date that will cause an error
    //     let prices = vec![Positive::from_f64(100.0)];
    //
    //     mock_option.expect_payoff_at_price()
    //         .returning(|_| Ok(Decimal::from_str("10.0").unwrap()));
    //
    //     // Make the expiration date calculation fail
    //     mock_option.expiration_date = NaiveDate::from_ymd_opt(9999, 12, 31).unwrap(); // Far future date that might cause issues
    //
    //     // Create a custom implementation for get_years that returns an error
    //     impl ExpirationDate for MockOptions {
    //         fn get_years(&self) -> Result<f64, ExpirationDateError> {
    //             Err("Invalid expiration date".unwrap_or(Positive::ZERO))
    //         }
    //     }
    //
    //     // Act
    //     let result = price_option_monte_carlo(&mock_option, &prices);
    //
    //     // Assert
    //     assert!(result.is_err());
    //     assert_eq!(result.unwrap_err().to_string(), "Invalid expiration date");
    // }
    //
    // #[test]
    // fn test_payoff_calculation_error_handling() {
    //     // Arrange
    //     let mut mock_option = MockOptions::new();
    //
    //     // Setup expiration date (1 year from now)
    //     let expiration = Utc::now().date_naive() + chrono::Duration::days(365);
    //     mock_option.risk_free_rate = 0.05;
    //     mock_option.dividend_yield = 0.02;
    //     mock_option.expiration_date = expiration;
    //
    //     let prices = vec![Positive::from_f64(100.0)];
    //
    //     // Make the payoff calculation fail
    //     mock_option.expect_payoff_at_price()
    //         .returning(|_| Err("Payoff calculation failed".into()));
    //
    //     // Act & Assert
    //     // We expect a panic since the function uses expect() on the payoff calculation
    //     std::panic::catch_unwind(|| {
    //         price_option_monte_carlo(&mock_option, &prices)
    //     }).expect_err("Expected a panic but none occurred");
    // }
    //
    // #[test]
    // fn test_large_number_of_simulations() {
    //     // Arrange
    //     let mut mock_option = MockOptions::new();
    //
    //     // Setup expiration date (1 year from now)
    //     let expiration = Utc::now().date_naive() + chrono::Duration::days(365);
    //     mock_option.risk_free_rate = 0.05;
    //     mock_option.dividend_yield = 0.02;
    //     mock_option.expiration_date = expiration;
    //
    //     // Create a large number of simulations
    //     let num_simulations = 1000;
    //     let mut prices = Vec::with_capacity(num_simulations);
    //     for _ in 0..num_simulations {
    //         prices.push(Positive::from_f64(100.0));
    //     }
    //
    //     // All payoffs are 5.0 in this test
    //     mock_option.expect_payoff_at_price()
    //         .returning(|_| Ok(Decimal::from_str("5.0").unwrap()));
    //
    //     // Act
    //     let result = price_option_monte_carlo(&mock_option, &prices);
    //
    //     // Assert
    //     assert!(result.is_ok());
    //
    //     // Expected: avg payoff = 5.0
    //     // Discount factor = exp(-(0.05-0.02)*1) = exp(-0.03) ≈ 0.9704
    //     // Expected price = 5.0 * 0.9704 ≈ 4.852
    //     let expected = Positive::from_f64(4.852);
    //
    //     // Using approximate comparison due to floating-point calculations
    //     let diff = (result.unwrap().0 - expected.0).abs();
    //     assert!(diff < Decimal::from_str("0.001").unwrap(),
    //             "Expected close to {}, got {}", expected.0, result.unwrap().0);
    // }
    //
    // #[test]
    // fn test_negative_payoffs_handled_correctly() {
    //     // Arrange
    //     let mut mock_option = MockOptions::new();
    //
    //     // Setup expiration date (1 year from now)
    //     let expiration = Utc::now().date_naive() + chrono::Duration::days(365);
    //     mock_option.risk_free_rate = 0.05;
    //     mock_option.dividend_yield = 0.02;
    //     mock_option.expiration_date = expiration;
    //
    //     let prices = vec![
    //         Positive::from_f64(100.0),
    //         Positive::from_f64(110.0),
    //         Positive::from_f64(90.0),
    //     ];
    //
    //     // Some payoffs are negative (unusual but possible in some exotic options)
    //     mock_option.expect_payoff_at_price()
    //         .with(eq(Positive::from_f64(100.0)))
    //         .returning(|_| Ok(Decimal::from_str("-5.0").unwrap()));
    //
    //     mock_option.expect_payoff_at_price()
    //         .with(eq(Positive::from_f64(110.0)))
    //         .returning(|_| Ok(Decimal::from_str("10.0").unwrap()));
    //
    //     mock_option.expect_payoff_at_price()
    //         .with(eq(Positive::from_f64(90.0)))
    //         .returning(|_| Ok(Decimal::from_str("5.0").unwrap()));
    //
    //     // Act
    //     let result = price_option_monte_carlo(&mock_option, &prices);
    //
    //     // Assert
    //     assert!(result.is_ok());
    //
    //     // Expected: avg payoff = (-5 + 10 + 5)/3 ≈ 3.333
    //     // Discount factor = exp(-(0.05-0.02)*1) = exp(-0.03) ≈ 0.9704
    //     // Expected price = 3.333 * 0.9704 ≈ 3.235
    //     // Since we take the absolute value, it should be positive
    //     let expected = Positive::from_f64(3.235);
    //
    //     // Using approximate comparison due to floating-point calculations
    //     let diff = (result.unwrap().0 - expected.0).abs();
    //     assert!(diff < Decimal::from_str("0.001").unwrap(),
    //             "Expected close to {}, got {}", expected.0, result.unwrap().0);
    // }
}

/// The `f64` path kernel (#859 P3b) against the `Decimal` path it replaced,
/// on the same seeded draws, over spots, volatilities, styles and sides.
#[cfg(test)]
mod tests_f64_path_kernel {
    use super::*;
    use crate::pricing::utils::wiener_increment;
    use optionstratlib_core::model::ExpirationDate;
    use optionstratlib_core::model::types::OptionType;
    use optionstratlib_core::pos_or_panic;
    use optionstratlib_core::utils::deterministic_rng;
    use rust_decimal_macros::dec;

    /// The previous price: the path in `Decimal`, the rest unchanged.
    fn decimal_path_price<R: Rng + ?Sized>(
        option: &Options,
        steps: usize,
        simulations: usize,
        rng: &mut R,
    ) -> Decimal {
        let steps_dec = Decimal::from(steps);
        let dt = option.expiration_date.get_years().unwrap().to_dec() / steps_dec;
        let carry = option.risk_free_rate - option.dividend_yield.to_dec();
        let growth_drift = Decimal::ONE + carry * dt;
        let sigma = option.implied_volatility.to_dec();
        let strike = option.strike_price.to_dec();
        let sqrt_dt = wiener_sqrt_dt(dt).unwrap();
        let mut payoff_sum = 0.0;
        for _ in 0..simulations {
            let mut st = option.underlying_price.to_dec();
            for _ in 0..steps {
                let w = wiener_increment(sqrt_dt, rng).unwrap();
                st *= growth_drift + sigma * w;
            }
            let payoff = match option.option_style {
                OptionStyle::Call => st - strike,
                OptionStyle::Put => strike - st,
            }
            .max(Decimal::ZERO);
            payoff_sum += decimal_to_f64(payoff).unwrap();
        }
        let rate = decimal_to_f64(option.risk_free_rate).unwrap();
        let years = decimal_to_f64(option.expiration_date.get_years().unwrap().to_dec()).unwrap();
        let price =
            finite_decimal(payoff_sum / simulations as f64 * (-rate * years).exp()).unwrap();
        match option.side {
            Side::Long => price,
            Side::Short => -price,
        }
    }

    fn option(spot: Decimal, sigma: Decimal, style: OptionStyle, side: Side) -> Options {
        Options {
            option_type: OptionType::European,
            side,
            underlying_symbol: "TEST".to_string(),
            strike_price: Positive::HUNDRED,
            expiration_date: ExpirationDate::Days(pos_or_panic!(90.0)),
            implied_volatility: Positive::new_decimal(sigma).unwrap(),
            quantity: Positive::ONE,
            contract_size: Positive::ONE,
            underlying_price: Positive::new_decimal(spot).unwrap(),
            risk_free_rate: dec!(0.05),
            option_style: style,
            dividend_yield: pos_or_panic!(0.02),
            exotic_params: None,
        }
    }

    #[test]
    fn test_f64_path_kernel_within_the_owner_bound() {
        let steps = NonZeroUsize::new(30).unwrap();
        let simulations = NonZeroUsize::new(400).unwrap();
        let mut worst = Decimal::ZERO;
        for spot in [dec!(70), dec!(95), dec!(100), dec!(105), dec!(140)] {
            for sigma in [dec!(0.05), dec!(0.2), dec!(0.8)] {
                for style in [OptionStyle::Call, OptionStyle::Put] {
                    for side in [Side::Long, Side::Short] {
                        let option = option(spot, sigma, style, side);
                        let fast = monte_carlo_option_pricing(
                            &option,
                            steps,
                            simulations,
                            &mut deterministic_rng(859),
                        )
                        .unwrap();
                        let reference = decimal_path_price(
                            &option,
                            steps.get(),
                            simulations.get(),
                            &mut deterministic_rng(859),
                        );
                        // Relative to the price, read as absolute below one
                        // cent so a worthless deep out-of-the-money option is
                        // not held to a relative bound.
                        let scale = reference.abs().max(dec!(0.01));
                        let error = (fast - reference).abs() / scale;
                        assert!(
                            error <= dec!(0.000000000001),
                            "S={spot} sigma={sigma} {style:?} {side:?}: {fast} vs {reference}"
                        );
                        worst = worst.max(error);
                    }
                }
            }
        }
        // Measured 2026-10-09: 3.6e-13 at worst (the 30 products of a path
        // each round once in `f64`); held at 5e-13 so a regression shows.
        assert!(worst < dec!(0.0000000000005), "worst {worst}");
    }
}
