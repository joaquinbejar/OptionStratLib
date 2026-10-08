//! Benchmarks of the `optionstratlib-pricing` pricing models (#789).
//!
//! Closed-form Black-Scholes, Black-76 and Garman-Kohlhagen, the binomial
//! tree by step count, Monte-Carlo and telegraph by path and step count, the
//! Barone-Adesi-Whaley American approximation, and every exotic closed form.
//! The stochastic models draw from a seeded RNG, so a run is reproducible.
//!
//! Run with `cargo bench -p optionstratlib-pricing --bench pricing`.

use criterion::measurement::WallTime;
use criterion::{BenchmarkGroup, Criterion, Throughput, criterion_group, criterion_main};
use optionstratlib_core::model::option::ExoticParams;
use optionstratlib_core::model::types::{
    AsianAveragingType, BarrierType, BinaryType, LookbackType,
};
use optionstratlib_core::model::{
    ExpirationDate, OptionStyle, OptionType, Options, Positive, RainbowType, Side,
};
use optionstratlib_core::pos_or_panic;
use optionstratlib_core::utils::{DETERMINISTIC_RNG_DEFAULT_SEED, deterministic_rng};
use optionstratlib_pricing::pricing::{
    BinomialPricingParams, OptionPricing, RegimeVolatility, asian_black_scholes,
    barone_adesi_whaley, barrier_black_scholes, binary_black_scholes, black_76, black_scholes,
    chooser_black_scholes, cliquet_black_scholes, compound_black_scholes, exchange_black_scholes,
    garman_kohlhagen, generate_binomial_tree, lookback_black_scholes, monte_carlo_option_pricing,
    power_black_scholes, price_binomial, probability_keep_under_strike, quanto_black_scholes,
    rainbow_black_scholes, simulate_returns, spread_black_scholes, telegraph,
};
use rust_decimal_macros::dec;
use std::fmt::Debug;
use std::hint::black_box;
use std::num::NonZeroUsize;

/// Binomial step counts: a coarse tree, the common default, a converged
/// tree, and a deep one that shows the O(n^2) growth.
const BINOMIAL_STEPS: [usize; 4] = [10, 50, 200, 1_000];

/// Checks that `f` succeeds on the fixture, then measures it.
///
/// A path that fails fast would otherwise be timed as its error branch and
/// report a misleadingly cheap number.
fn bench_ok<T, E: Debug>(
    group: &mut BenchmarkGroup<'_, WallTime>,
    name: &str,
    mut f: impl FnMut() -> Result<T, E>,
) {
    if let Err(e) = f() {
        panic!("bench `{name}`: the fixture returned an error: {e:?}");
    }
    group.bench_function(name, |bench| bench.iter(|| black_box(f())));
}

fn non_zero(n: usize) -> NonZeroUsize {
    NonZeroUsize::new(n).expect("bench sizes are non-zero")
}

/// Strike 100 on a spot of 105, 90 days, 20% volatility, 5% rate, 2% yield.
fn option_of(option_type: OptionType, exotic_params: Option<ExoticParams>) -> Options {
    Options::new(
        option_type,
        Side::Long,
        "BENCH".to_string(),
        Positive::HUNDRED,
        ExpirationDate::Days(pos_or_panic!(90.0)),
        pos_or_panic!(0.2),
        Positive::ONE,
        pos_or_panic!(105.0),
        dec!(0.05),
        OptionStyle::Call,
        pos_or_panic!(0.02),
        exotic_params,
    )
}

fn european() -> Options {
    option_of(OptionType::European, None)
}

fn bench_closed_forms(c: &mut Criterion) {
    let mut group = c.benchmark_group("pricing/closed_form");
    let option = european();
    let fx = option_of(
        OptionType::European,
        Some(ExoticParams {
            foreign_rate: Some(dec!(0.03)),
            ..ExoticParams::default()
        }),
    );

    bench_ok(&mut group, "black_scholes", || {
        black_scholes(black_box(&option))
    });
    bench_ok(&mut group, "calculate_price_black_scholes", || {
        black_box(&option).calculate_price_black_scholes()
    });
    bench_ok(&mut group, "black_76", || black_76(black_box(&option)));
    bench_ok(&mut group, "garman_kohlhagen", || {
        garman_kohlhagen(black_box(&fx))
    });
    bench_ok(&mut group, "barone_adesi_whaley_put", || {
        barone_adesi_whaley(
            black_box(pos_or_panic!(95.0)),
            Positive::HUNDRED,
            pos_or_panic!(0.25),
            dec!(0.05),
            pos_or_panic!(0.02),
            pos_or_panic!(0.2),
            &OptionStyle::Put,
        )
    });
    bench_ok(&mut group, "probability_keep_under_strike", || {
        probability_keep_under_strike(black_box(option.clone()), None)
    });

    // Re-pricing a 50-strike chain: the per-chain cost callers pay.
    let strikes: Vec<Options> = (0..50)
        .map(|i| {
            let mut o = european();
            o.strike_price = pos_or_panic!(80.0 + f64::from(i));
            o
        })
        .collect();
    group.throughput(Throughput::Elements(strikes.len() as u64));
    bench_ok(&mut group, "black_scholes_chain/50", || {
        strikes
            .iter()
            .map(|o| black_scholes(black_box(o)))
            .collect::<Result<Vec<_>, _>>()
    });
    group.finish();
}

fn bench_binomial(c: &mut Criterion) {
    let mut group = c.benchmark_group("pricing/binomial");
    let european = european();
    let mut american = european.clone();
    american.option_type = OptionType::American;
    american.option_style = OptionStyle::Put;

    for steps in BINOMIAL_STEPS {
        let n = non_zero(steps);
        group.throughput(Throughput::Elements(steps as u64));
        bench_ok(&mut group, &format!("european/{steps}"), || {
            black_box(&european).calculate_price_binomial(n)
        });
        bench_ok(&mut group, &format!("american_put/{steps}"), || {
            black_box(&american).calculate_price_binomial(n)
        });
        bench_ok(&mut group, &format!("price_binomial/{steps}"), || {
            price_binomial(BinomialPricingParams {
                asset: black_box(european.underlying_price),
                volatility: european.implied_volatility,
                int_rate: european.risk_free_rate,
                strike: european.strike_price,
                expiry: pos_or_panic!(0.25),
                no_steps: n,
                option_type: &OptionType::American,
                option_style: &OptionStyle::Put,
                side: &Side::Long,
            })
        });
    }
    // The full tree is materialised for charts; only a moderate depth is
    // realistic there.
    for steps in [10, 50, 200] {
        let n = non_zero(steps);
        group.throughput(Throughput::Elements(steps as u64));
        bench_ok(
            &mut group,
            &format!("generate_binomial_tree/{steps}"),
            || {
                generate_binomial_tree(&BinomialPricingParams {
                    asset: black_box(european.underlying_price),
                    volatility: european.implied_volatility,
                    int_rate: european.risk_free_rate,
                    strike: european.strike_price,
                    expiry: pos_or_panic!(0.25),
                    no_steps: n,
                    option_type: &OptionType::European,
                    option_style: &OptionStyle::Call,
                    side: &Side::Long,
                })
            },
        );
    }
    group.finish();
}

fn bench_stochastic(c: &mut Criterion) {
    let mut group = c.benchmark_group("pricing/stochastic");
    group.sample_size(10);
    let option = european();

    for (steps, paths) in [(30, 1_000), (30, 10_000), (252, 1_000)] {
        let mut rng = deterministic_rng(DETERMINISTIC_RNG_DEFAULT_SEED);
        group.throughput(Throughput::Elements((steps * paths) as u64));
        bench_ok(
            &mut group,
            &format!("monte_carlo/{steps}_steps_x_{paths}_paths"),
            || {
                monte_carlo_option_pricing(
                    black_box(&option),
                    non_zero(steps),
                    non_zero(paths),
                    &mut rng,
                )
            },
        );
    }

    let regimes = RegimeVolatility::new(pos_or_panic!(0.3), pos_or_panic!(0.1))
        .expect("strictly positive regime volatilities");
    for (steps, paths) in [(30, 1_000), (30, 10_000), (252, 1_000)] {
        let mut rng = deterministic_rng(DETERMINISTIC_RNG_DEFAULT_SEED);
        group.throughput(Throughput::Elements((steps * paths) as u64));
        bench_ok(
            &mut group,
            &format!("telegraph/{steps}_steps_x_{paths}_paths"),
            || {
                telegraph(
                    black_box(&option),
                    non_zero(steps),
                    non_zero(paths),
                    Some(dec!(0.5)),
                    Some(dec!(0.5)),
                    regimes,
                    &mut rng,
                )
            },
        );
    }

    for len in [252, 1_008] {
        let mut rng = deterministic_rng(DETERMINISTIC_RNG_DEFAULT_SEED);
        group.throughput(Throughput::Elements(len as u64));
        bench_ok(&mut group, &format!("simulate_returns/{len}"), || {
            simulate_returns(dec!(0.05), pos_or_panic!(0.2), len, dec!(0.004), &mut rng)
        });
    }
    group.finish();
}

fn bench_exotics(c: &mut Criterion) {
    let mut group = c.benchmark_group("pricing/exotic");
    let asian = option_of(
        OptionType::Asian {
            averaging_type: AsianAveragingType::Arithmetic,
        },
        None,
    );
    let asian_geometric = option_of(
        OptionType::Asian {
            averaging_type: AsianAveragingType::Geometric,
        },
        None,
    );
    let barrier = option_of(
        OptionType::Barrier {
            barrier_type: BarrierType::UpAndOut,
            barrier_level: pos_or_panic!(130.0),
            rebate: None,
        },
        None,
    );
    let binary = option_of(
        OptionType::Binary {
            binary_type: BinaryType::CashOrNothing,
        },
        None,
    );
    let lookback = option_of(
        OptionType::Lookback {
            lookback_type: LookbackType::FloatingStrike,
        },
        Some(ExoticParams {
            spot_min: Some(dec!(95)),
            spot_max: Some(dec!(110)),
            ..ExoticParams::default()
        }),
    );
    let compound = option_of(
        OptionType::Compound {
            underlying_option: Box::new(OptionType::European),
        },
        None,
    );
    let chooser = option_of(
        OptionType::Chooser {
            choice_date: pos_or_panic!(30.0),
        },
        None,
    );
    let cliquet = option_of(
        OptionType::Cliquet {
            reset_dates: vec![pos_or_panic!(30.0), pos_or_panic!(60.0)],
        },
        Some(ExoticParams {
            cliquet_local_cap: Some(dec!(0.05)),
            cliquet_local_floor: Some(dec!(0.0)),
            ..ExoticParams::default()
        }),
    );
    let rainbow = option_of(
        OptionType::Rainbow {
            num_assets: 2,
            rainbow_type: RainbowType::BestOf,
        },
        Some(ExoticParams {
            rainbow_second_asset_price: Some(Positive::HUNDRED),
            rainbow_second_asset_volatility: Some(pos_or_panic!(0.25)),
            rainbow_second_asset_dividend: Some(Positive::ZERO),
            rainbow_correlation: Some(dec!(0.5)),
            ..ExoticParams::default()
        }),
    );
    let spread = option_of(
        OptionType::Spread {
            second_asset: Positive::HUNDRED,
        },
        Some(ExoticParams {
            spread_second_asset_volatility: Some(pos_or_panic!(0.25)),
            spread_second_asset_dividend: Some(Positive::ZERO),
            spread_correlation: Some(dec!(0.5)),
            ..ExoticParams::default()
        }),
    );
    let quanto = option_of(
        OptionType::Quanto {
            exchange_rate: pos_or_panic!(1.25),
        },
        Some(ExoticParams {
            quanto_fx_volatility: Some(pos_or_panic!(0.1)),
            quanto_fx_correlation: Some(dec!(0.3)),
            quanto_foreign_rate: Some(dec!(0.03)),
            ..ExoticParams::default()
        }),
    );
    let exchange = option_of(
        OptionType::Exchange {
            second_asset: Positive::HUNDRED,
        },
        Some(ExoticParams {
            exchange_second_asset_volatility: Some(pos_or_panic!(0.25)),
            exchange_second_asset_dividend: Some(Positive::ZERO),
            exchange_correlation: Some(dec!(0.5)),
            ..ExoticParams::default()
        }),
    );
    let mut power = option_of(
        OptionType::Power {
            exponent: pos_or_panic!(2.0),
        },
        Some(ExoticParams::default()),
    );
    power.underlying_price = pos_or_panic!(10.0);

    bench_ok(&mut group, "asian_arithmetic", || {
        asian_black_scholes(black_box(&asian))
    });
    bench_ok(&mut group, "asian_geometric", || {
        asian_black_scholes(black_box(&asian_geometric))
    });
    bench_ok(&mut group, "barrier_up_and_out", || {
        barrier_black_scholes(black_box(&barrier))
    });
    bench_ok(&mut group, "binary_cash_or_nothing", || {
        binary_black_scholes(black_box(&binary))
    });
    bench_ok(&mut group, "lookback_floating", || {
        lookback_black_scholes(black_box(&lookback))
    });
    bench_ok(&mut group, "compound", || {
        compound_black_scholes(black_box(&compound))
    });
    bench_ok(&mut group, "chooser", || {
        chooser_black_scholes(black_box(&chooser))
    });
    bench_ok(&mut group, "cliquet", || {
        cliquet_black_scholes(black_box(&cliquet))
    });
    bench_ok(&mut group, "rainbow_best_of", || {
        rainbow_black_scholes(black_box(&rainbow))
    });
    bench_ok(&mut group, "spread", || {
        spread_black_scholes(black_box(&spread))
    });
    bench_ok(&mut group, "quanto", || {
        quanto_black_scholes(black_box(&quanto))
    });
    bench_ok(&mut group, "exchange", || {
        exchange_black_scholes(black_box(&exchange))
    });
    bench_ok(&mut group, "power", || {
        power_black_scholes(black_box(&power))
    });
    group.finish();
}

criterion_group!(
    benches,
    bench_closed_forms,
    bench_binomial,
    bench_stochastic,
    bench_exotics
);
criterion_main!(benches);
