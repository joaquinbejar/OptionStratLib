/******************************************************************************
    Author: Joaquín Béjar García
    Email: jb@taunais.com 
    Date: 14/8/24
 ******************************************************************************/

use rayon::prelude::*;
use optionstratlib::model::option::Options;
use optionstratlib::model::types::{ExpirationDate, OptionStyle, OptionType, Side};
use optionstratlib::pricing::black_scholes_model::black_scholes;
use optionstratlib::greeks::utils::big_n;

struct OptionSimple {
    strike: f64,
    market_price: f64,
    implied_volatility: f64,
}

fn black_scholes_local(s: f64, k: f64, t: f64, r: f64, sigma: f64) -> f64 {
    let sqrt_t = t.sqrt();
    let d1 = ((s / k).ln() + (r + sigma * sigma / 2.0) * t) / (sigma * sqrt_t);
    let d2 = d1 - sigma * sqrt_t;

    s * big_n(d1) - k * (-r * t).exp() * big_n(d2)
}

fn volatility_mse(s: f64, options: &[OptionSimple], t: f64, r: f64) -> f64 {
    options.par_iter().map(|opt| {
        let calculated_price = black_scholes_local(s, opt.strike, t, r, opt.implied_volatility);
        (calculated_price - opt.market_price).powi(2)
    }).sum::<f64>() / options.len() as f64
}

fn optimize_parameters(s: f64, options: &[OptionSimple]) -> (f64, f64) {
    let t_range: Vec<f64> = (1..30).map(|days| days as f64 / 365.0).collect();
    let r_range: Vec<f64> = (0..100).map(|rate| rate as f64 / 1000.0).collect();

    let (best_t, best_r, _min_mse) = t_range.par_iter().flat_map(|&t| {
        r_range.par_iter().map(move |&r| {
            let mse = volatility_mse(s, options, t, r);
            (t, r, mse)
        })
    }).min_by(|a, b| a.2.partial_cmp(&b.2).unwrap()).unwrap();

    (best_t, best_r)
}


fn main() {
    let s = 2476.6;

    let broker_data = vec![
        OptionSimple { strike: 2380.0, market_price: 97.7, implied_volatility: 0.1975 },
        OptionSimple { strike: 2385.0, market_price: 92.8, implied_volatility: 0.198438 },
        OptionSimple { strike: 2390.0, market_price: 87.8, implied_volatility: 0.19375 },
        OptionSimple { strike: 2395.0, market_price: 83.0, implied_volatility: 0.200313 },
        OptionSimple { strike: 2400.0, market_price: 78.1, implied_volatility: 0.20125 },
        OptionSimple { strike: 2405.0, market_price: 73.3, implied_volatility: 0.202188 },
        OptionSimple { strike: 2410.0, market_price: 68.6, implied_volatility: 0.203125 },
        OptionSimple { strike: 2415.0, market_price: 63.9, implied_volatility: 0.204063 },
        OptionSimple { strike: 2420.0, market_price: 59.4, implied_volatility: 0.205 },
        OptionSimple { strike: 2425.0, market_price: 54.9, implied_volatility: 0.205938 },
        OptionSimple { strike: 2430.0, market_price: 50.6, implied_volatility: 0.206875 },
        OptionSimple { strike: 2435.0, market_price: 46.4, implied_volatility: 0.207813 },
        OptionSimple { strike: 2440.0, market_price: 42.4, implied_volatility: 0.20875 },
        OptionSimple { strike: 2445.0, market_price: 38.5, implied_volatility: 0.212188 },
        OptionSimple { strike: 2450.0, market_price: 34.8, implied_volatility: 0.210625 },
        OptionSimple { strike: 2455.0, market_price: 31.4, implied_volatility: 0.211563 },
        OptionSimple { strike: 2460.0, market_price: 28.2, implied_volatility: 0.21375 },
        OptionSimple { strike: 2465.0, market_price: 25.2, implied_volatility: 0.215313 },
        OptionSimple { strike: 2470.0, market_price: 22.5, implied_volatility: 0.216875 },
        OptionSimple { strike: 2475.0, market_price: 20.0, implied_volatility: 0.218438 },
        OptionSimple { strike: 2480.0, market_price: 17.6, implied_volatility: 0.22 },
        OptionSimple { strike: 2485.0, market_price: 15.4, implied_volatility: 0.22 },
        OptionSimple { strike: 2490.0, market_price: 13.4, implied_volatility: 0.22 },
        OptionSimple { strike: 2495.0, market_price: 11.6, implied_volatility: 0.22 },
        OptionSimple { strike: 2500.0, market_price: 9.9, implied_volatility: 0.22 },
        OptionSimple { strike: 2505.0, market_price: 8.0, implied_volatility: 0.22375 },
        OptionSimple { strike: 2510.0, market_price: 6.7, implied_volatility: 0.2275 },
        OptionSimple { strike: 2515.0, market_price: 5.7, implied_volatility: 0.23125 },
        OptionSimple { strike: 2520.0, market_price: 4.8, implied_volatility: 0.235 },
        OptionSimple { strike: 2525.0, market_price: 4.0, implied_volatility: 0.23875 },
        OptionSimple { strike: 2530.0, market_price: 3.4, implied_volatility: 0.2425 },
        OptionSimple { strike: 2535.0, market_price: 2.8, implied_volatility: 0.245 },
        OptionSimple { strike: 2540.0, market_price: 2.3, implied_volatility: 0.2475 },
        OptionSimple { strike: 2545.0, market_price: 1.9, implied_volatility: 0.25 },
        OptionSimple { strike: 2550.0, market_price: 1.6, implied_volatility: 0.2525 },
        OptionSimple { strike: 2555.0, market_price: 1.3, implied_volatility: 0.255 },
        OptionSimple { strike: 2560.0, market_price: 1.0, implied_volatility: 0.2575 },
        OptionSimple { strike: 2565.0, market_price: 0.8, implied_volatility: 0.26 },
        OptionSimple { strike: 2570.0, market_price: 0.6, implied_volatility: 0.2625 },
        OptionSimple { strike: 2575.0, market_price: 0.4, implied_volatility: 0.265 },
        OptionSimple { strike: 2580.0, market_price: 0.2, implied_volatility: 0.2675 },
        OptionSimple { strike: 2585.0, market_price: 0.2, implied_volatility: 0.27 },
    ];

    let (best_t, best_r) = optimize_parameters(s, &broker_data);

    println!("Best t: {:.4} años ({:.1} days)", best_t, best_t * 365.0);
    println!("Best r: {:.4}", best_r);


    for opt in &broker_data {
        let option = Options::new(
            OptionType::European,
            Side::Long,
            "GOLD".to_string(),
            opt.strike,
            ExpirationDate::Days(best_t * 365.0),
            opt.implied_volatility,
            1,
            s,
            best_r,
            OptionStyle::Call,
            0.0,
            None,
            None,
            None,
        );
        let theoretical_price = black_scholes(&option);
        let price_is;
        let option_is;
        if theoretical_price < opt.market_price {
            price_is = "Expensive";
        } else {
            price_is = "Cheap";
        }
        if opt.strike < s {
            option_is = "ATM";
        } else {
            option_is = "OTM";
        }

        println!("Strike {}: Market Price = {:.2}, Theoretical Price = {:.2}, VI = {:.2}% is {} is {}",
                 opt.strike,
                 opt.market_price,
                 theoretical_price,
                 opt.implied_volatility * 100.0,
                 price_is,
                 option_is);
    }
}