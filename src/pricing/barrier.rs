/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 12/01/26
******************************************************************************/

use crate::Options;
use crate::error::PricingError;
use crate::greeks::big_n;
use crate::model::types::{BarrierType, OptionStyle, OptionType};
use rust_decimal::prelude::FromPrimitive;
use rust_decimal::{Decimal, MathematicalOps};
use rust_decimal_macros::dec;

/// Prices a barrier option using the Black-Scholes analytical extension.
/// Supports Down-And-In, Up-And-In, Down-And-Out, and Up-And-Out variants.
pub fn barrier_black_scholes(option: &Options) -> Result<Decimal, PricingError> {
    let (barrier_type, barrier_level, rebate) = match &option.option_type {
        OptionType::Barrier {
            barrier_type,
            barrier_level,
            rebate,
        } => (
            barrier_type,
            Decimal::from_f64(*barrier_level).unwrap(),
            Decimal::from_f64(rebate.unwrap_or(0.0)).unwrap(),
        ),
        _ => {
            return Err(PricingError::unsupported_option_type(
                "Non-Barrier",
                "Barrier BS",
            ));
        }
    };

    let s = option.underlying_price.to_dec();
    let k = option.strike_price.to_dec();
    let r = option.risk_free_rate;
    let q = option.dividend_yield.to_dec();
    let sigma = option.implied_volatility.to_dec();
    let t = option.time_to_expiration()?.to_dec();

    if t == Decimal::ZERO {
        return option
            .payoff()
            .map_err(|e| PricingError::other(&e.to_string()));
    }

    if sigma == Decimal::ZERO {
        return Err(PricingError::other(
            "Volatility cannot be zero for barrier options pricing",
        ));
    }

    let b = r - q; // Cost of carry
    let sigma2 = sigma * sigma;
    let mu = (b - sigma2 / dec!(2.0)) / sigma2;
    let lambda = (mu * mu + dec!(2.0) * r / sigma2).sqrt().unwrap();

    let sigma_sqrt_t = sigma * t.sqrt().unwrap();

    // Components used across different barrier types
    let x1 = (s / k).ln() / sigma_sqrt_t + (mu + dec!(1.0)) * sigma_sqrt_t;
    let x2 = (s / barrier_level).ln() / sigma_sqrt_t + (mu + dec!(1.0)) * sigma_sqrt_t;
    let y1 = (barrier_level * barrier_level / (s * k)).ln() / sigma_sqrt_t
        + (mu + dec!(1.0)) * sigma_sqrt_t;
    let y2 = (barrier_level / s).ln() / sigma_sqrt_t + (mu + dec!(1.0)) * sigma_sqrt_t;
    let z = (barrier_level / s).ln() / sigma_sqrt_t + lambda * sigma_sqrt_t;

    let _phi = match option.option_style {
        OptionStyle::Call => dec!(1.0),
        OptionStyle::Put => dec!(-1.0),
    };

    let _eta = match barrier_type {
        BarrierType::DownAndIn | BarrierType::DownAndOut => dec!(1.0),
        BarrierType::UpAndIn | BarrierType::UpAndOut => dec!(-1.0),
    };

    let f_a = |phi_val: Decimal, x_val: Decimal| -> Result<Decimal, PricingError> {
        let n1 = big_n(phi_val * x_val)?;
        let n2 = big_n(phi_val * (x_val - sigma_sqrt_t))?;
        Ok(phi_val * s * (-q * t).exp() * n1 - phi_val * k * (-r * t).exp() * n2)
    };

    let f_b = |phi_val: Decimal, x_val: Decimal| -> Result<Decimal, PricingError> {
        let n1 = big_n(phi_val * x_val)?;
        let n2 = big_n(phi_val * (x_val - sigma_sqrt_t))?;
        Ok(phi_val * s * (-q * t).exp() * n1 - phi_val * k * (-r * t).exp() * n2)
    };

    let f_c =
        |phi_val: Decimal, eta_val: Decimal, y_val: Decimal| -> Result<Decimal, PricingError> {
            let n1 = big_n(eta_val * y_val)?;
            let n2 = big_n(eta_val * (y_val - sigma_sqrt_t))?;
            let h_s_ratio = (barrier_level / s).powd(dec!(2.0) * (mu + dec!(1.0)));
            let h_s_ratio_mu = (barrier_level / s).powd(dec!(2.0) * mu);
            Ok(phi_val * s * (-q * t).exp() * h_s_ratio * n1
                - phi_val * k * (-r * t).exp() * h_s_ratio_mu * n2)
        };

    let f_d =
        |phi_val: Decimal, eta_val: Decimal, y_val: Decimal| -> Result<Decimal, PricingError> {
            let n1 = big_n(eta_val * y_val)?;
            let n2 = big_n(eta_val * (y_val - sigma_sqrt_t))?;
            let h_s_ratio = (barrier_level / s).powd(dec!(2.0) * (mu + dec!(1.0)));
            let h_s_ratio_mu = (barrier_level / s).powd(dec!(2.0) * mu);
            Ok(phi_val * s * (-q * t).exp() * h_s_ratio * n1
                - phi_val * k * (-r * t).exp() * h_s_ratio_mu * n2)
        };

    let f_e = |eta_val: Decimal| -> Result<Decimal, PricingError> {
        if rebate == Decimal::ZERO {
            return Ok(Decimal::ZERO);
        }
        let n1 = big_n(eta_val * (x2 - sigma_sqrt_t))?;
        let h_s_ratio_mu = (barrier_level / s).powd(dec!(2.0) * mu);
        let n2 = big_n(eta_val * (y2 - sigma_sqrt_t))?;
        Ok(rebate * (-r * t).exp() * (n1 - h_s_ratio_mu * n2))
    };

    let f_f = |eta_val: Decimal| -> Result<Decimal, PricingError> {
        if rebate == Decimal::ZERO {
            return Ok(Decimal::ZERO);
        }
        let h_s_ratio_mu_lambda = (barrier_level / s).powd(mu + lambda);
        let h_s_ratio_mu_lambda_neg = (barrier_level / s).powd(mu - lambda);
        let n1 = big_n(eta_val * z)?;
        let n2 = big_n(eta_val * (z - dec!(2.0) * lambda * sigma_sqrt_t))?;
        Ok(rebate * (h_s_ratio_mu_lambda * n1 + h_s_ratio_mu_lambda_neg * n2))
    };

    match (option.option_style, barrier_type) {
        // Down-and-out call
        (OptionStyle::Call, BarrierType::DownAndOut) => {
            if k >= barrier_level {
                Ok(f_a(dec!(1.0), x1)? - f_c(dec!(1.0), dec!(1.0), y1)? + f_e(dec!(1.0))?)
            } else {
                Ok(f_b(dec!(1.0), x2)? - f_d(dec!(1.0), dec!(1.0), y2)? + f_e(dec!(1.0))?)
            }
        }
        // Down-and-in call
        (OptionStyle::Call, BarrierType::DownAndIn) => {
            if k >= barrier_level {
                Ok(f_c(dec!(1.0), dec!(1.0), y1)? + f_f(dec!(1.0))?)
            } else {
                Ok(f_a(dec!(1.0), x1)? - f_b(dec!(1.0), x2)?
                    + f_d(dec!(1.0), dec!(1.0), y2)?
                    + f_f(dec!(1.0))?)
            }
        }
        // Up-and-out call
        (OptionStyle::Call, BarrierType::UpAndOut) => {
            if k >= barrier_level {
                Ok(f_f(dec!(-1.0))?)
            } else {
                Ok(f_a(dec!(1.0), x1)? - f_b(dec!(1.0), x2)?
                    + f_d(dec!(1.0), dec!(-1.0), y2)?
                    + f_f(dec!(-1.0))?)
            }
        }
        // Up-and-in call
        (OptionStyle::Call, BarrierType::UpAndIn) => {
            if k >= barrier_level {
                Ok(f_a(dec!(1.0), x1)? + f_f(dec!(-1.0))?)
            } else {
                Ok(f_b(dec!(1.0), x2)? - f_d(dec!(1.0), dec!(-1.0), y2)? + f_f(dec!(-1.0))?)
            }
        }
        // Down-and-out put
        (OptionStyle::Put, BarrierType::DownAndOut) => {
            if k >= barrier_level {
                Ok(f_b(dec!(-1.0), x2)? - f_d(dec!(-1.0), dec!(1.0), y2)? + f_e(dec!(1.0))?)
            } else {
                Ok(f_a(dec!(-1.0), x1)? - f_c(dec!(-1.0), dec!(1.0), y1)? + f_e(dec!(1.0))?)
            }
        }
        // Down-and-in put
        (OptionStyle::Put, BarrierType::DownAndIn) => {
            if k >= barrier_level {
                Ok(f_a(dec!(-1.0), x1)? - f_b(dec!(-1.0), x2)?
                    + f_d(dec!(-1.0), dec!(1.0), y2)?
                    + f_f(dec!(1.0))?)
            } else {
                Ok(f_c(dec!(-1.0), dec!(1.0), y1)? + f_f(dec!(1.0))?)
            }
        }
        // Up-and-out put
        (OptionStyle::Put, BarrierType::UpAndOut) => {
            if k >= barrier_level {
                Ok(f_e(dec!(-1.0))?)
            } else {
                Ok(f_a(dec!(-1.0), x1)? - f_b(dec!(-1.0), x2)?
                    + f_d(dec!(-1.0), dec!(-1.0), y2)?
                    + f_e(dec!(-1.0))?)
            }
        }
        // Up-and-in put
        (OptionStyle::Put, BarrierType::UpAndIn) => {
            if k >= barrier_level {
                Ok(f_a(dec!(-1.0), x1)? + f_f(dec!(-1.0))?)
            } else {
                Ok(f_b(dec!(-1.0), x2)? - f_d(dec!(-1.0), dec!(-1.0), y2)? + f_f(dec!(-1.0))?)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::types::{BarrierType, OptionStyle, OptionType, Side};
    use crate::{ExpirationDate, Options};
    use positive::{Positive, pos_or_panic};
    use rust_decimal_macros::dec;

    fn create_test_option(style: OptionStyle, barrier_type: BarrierType, level: f64) -> Options {
        Options {
            option_type: OptionType::Barrier {
                barrier_type,
                barrier_level: level,
                rebate: None,
            },
            side: Side::Long,
            underlying_symbol: "TEST".to_string(),
            strike_price: pos_or_panic!(100.0),
            expiration_date: ExpirationDate::Days(pos_or_panic!(182.5)), // ~0.5 year
            implied_volatility: pos_or_panic!(0.25),
            quantity: pos_or_panic!(1.0),
            underlying_price: pos_or_panic!(100.0),
            risk_free_rate: dec!(0.08),
            option_style: style,
            dividend_yield: Positive::ZERO,
            exotic_params: None,
        }
    }

    #[test]
    fn test_down_and_out_call() {
        let option = create_test_option(OptionStyle::Call, BarrierType::DownAndOut, 95.0);
        let price = barrier_black_scholes(&option).unwrap();
        // S=100, K=100, H=95, r=0.08, q=0.0, sigma=0.25, T=0.5
        // Price should be approx 5.2998
        assert!(
            price > dec!(5.2) && price < dec!(5.4),
            "Price was {}",
            price
        );
    }

    #[test]
    fn test_down_and_in_call() {
        let option = create_test_option(OptionStyle::Call, BarrierType::DownAndIn, 95.0);
        let price = barrier_black_scholes(&option).unwrap();
        // Price should be approx 3.7414
        assert!(
            price > dec!(3.7) && price < dec!(3.8),
            "Price was {}",
            price
        );
    }

    #[test]
    fn test_in_out_parity() {
        let out_option = create_test_option(OptionStyle::Call, BarrierType::DownAndOut, 95.0);
        let in_option = create_test_option(OptionStyle::Call, BarrierType::DownAndIn, 95.0);

        let out_price = barrier_black_scholes(&out_option).unwrap();
        let in_price = barrier_black_scholes(&in_option).unwrap();

        let total = out_price + in_price;

        // Should equal vanilla BS call
        let mut vanilla = out_option.clone();
        vanilla.option_type = OptionType::European;
        let vanilla_price = crate::pricing::black_scholes_model::black_scholes(&vanilla).unwrap();

        // Using a slightly larger tolerance for Decimal calculations
        assert!(
            (total - vanilla_price).abs() < dec!(0.001),
            "Total: {}, Vanilla: {}",
            total,
            vanilla_price
        );
    }

    #[test]
    fn test_up_in_out_parity_call() {
        let out_option = create_test_option(OptionStyle::Call, BarrierType::UpAndOut, 105.0);
        let in_option = create_test_option(OptionStyle::Call, BarrierType::UpAndIn, 105.0);

        let out_price = barrier_black_scholes(&out_option).unwrap();
        let in_price = barrier_black_scholes(&in_option).unwrap();
        let total = out_price + in_price;

        let mut vanilla = out_option.clone();
        vanilla.option_type = OptionType::European;
        let vanilla_price = crate::pricing::black_scholes_model::black_scholes(&vanilla).unwrap();

        assert!(
            (total - vanilla_price).abs() < dec!(0.001),
            "Total: {}, Vanilla: {}",
            total,
            vanilla_price
        );
    }

    #[test]
    fn test_down_in_out_parity_put() {
        let out_option = create_test_option(OptionStyle::Put, BarrierType::DownAndOut, 95.0);
        let in_option = create_test_option(OptionStyle::Put, BarrierType::DownAndIn, 95.0);

        let out_price = barrier_black_scholes(&out_option).unwrap();
        let in_price = barrier_black_scholes(&in_option).unwrap();
        let total = out_price + in_price;

        let mut vanilla = out_option.clone();
        vanilla.option_type = OptionType::European;
        let vanilla_price = crate::pricing::black_scholes_model::black_scholes(&vanilla).unwrap();

        assert!(
            (total - vanilla_price).abs() < dec!(0.001),
            "Total: {}, Vanilla: {}",
            total,
            vanilla_price
        );
    }

    #[test]
    fn test_up_in_out_parity_put() {
        let out_option = create_test_option(OptionStyle::Put, BarrierType::UpAndOut, 105.0);
        let in_option = create_test_option(OptionStyle::Put, BarrierType::UpAndIn, 105.0);

        let out_price = barrier_black_scholes(&out_option).unwrap();
        let in_price = barrier_black_scholes(&in_option).unwrap();
        let total = out_price + in_price;

        let mut vanilla = out_option.clone();
        vanilla.option_type = OptionType::European;
        let vanilla_price = crate::pricing::black_scholes_model::black_scholes(&vanilla).unwrap();

        assert!(
            (total - vanilla_price).abs() < dec!(0.001),
            "Total: {}, Vanilla: {}",
            total,
            vanilla_price
        );
    }

    #[test]
    fn test_barrier_greeks() {
        let option = create_test_option(OptionStyle::Call, BarrierType::DownAndOut, 95.0);

        let delta = crate::greeks::delta(&option).unwrap();
        let gamma = crate::greeks::gamma(&option).unwrap();
        let vega = crate::greeks::vega(&option).unwrap();
        let rho = crate::greeks::rho(&option).unwrap();
        // DIC delta can be high near barrier.
        assert!(
            delta > dec!(0.1) && delta < dec!(2.0),
            "Delta was {}",
            delta
        );
        // Barrier Greeks can be negative and have higher magnitudes than vanilla
        // Output values for manual verification
        println!(
            "Barrier Greeks - Delta: {}, Gamma: {}, Vega: {}, Rho: {}",
            delta, gamma, vega, rho
        );
    }
}
