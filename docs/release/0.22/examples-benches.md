# 0.22 examples and benches: smoke evidence

Run for #787 with `make smoke-examples SMOKE_EXPORT=1` and `make smoke-benches` on the
branch of #787 (the commit that adds this file), with a chromedriver that matches the installed Chrome
(155.0.8059.40, macOS arm64), so every PNG/SVG export ran for real. Each
example binary is run from the repository root to completion with a 300 s
timeout; a run passes with exit status 0 and no panic. `smoke-benches` compiles
every bench target (the facade's and each component crate's, with the
features a target requires) and runs each of its benchmarks for one iteration
(Criterion `--test`): 14 targets, 558 benchmark functions.
Reproduce with the two commands above (`SMOKE_EXPORT=1` needs `WEBDRIVER_PATH`;
without it a program that fails only because the export could not start is
reported as `needs-webdriver`).

## Examples

### `examples_chain` (10 pass)

| Target | Result | Time |
| --- | --- | ---: |
| `async_chain_ops` | pass | 0.3 s |
| `async_ohlcv` | pass | 0.4 s |
| `creator` | pass | 0.3 s |
| `option_chain` | pass | 0.3 s |
| `option_chain_build_synthetic` | pass | 1.8 s |
| `option_chain_curve_surface` | pass | 3.6 s |
| `option_chain_ger40` | pass | 1.6 s |
| `option_chain_raw` | pass | 1.6 s |
| `option_chain_raw_delta` | pass | 1.6 s |
| `test_yellow_highlight` | pass | 0.3 s |

### `examples_curves` (28 pass)

| Target | Result | Time |
| --- | --- | ---: |
| `charm_curve` | pass | 1.5 s |
| `charm_maturity_vector_curve` | pass | 1.6 s |
| `charm_volatility_vector_curve` | pass | 1.5 s |
| `color_curve` | pass | 1.6 s |
| `color_maturity_vector_curve` | pass | 1.7 s |
| `color_volatility_vector_curve` | pass | 1.7 s |
| `d1_curve` | pass | 1.5 s |
| `d1_d2_vector_curve` | pass | 1.6 s |
| `d2_curve` | pass | 1.9 s |
| `delta_curve` | pass | 2.1 s |
| `delta_maturity_vector_curve` | pass | 1.9 s |
| `gamma_curve` | pass | 1.9 s |
| `gamma_volatility_vector_curve` | pass | 2.0 s |
| `parametric_curve` | pass | 1.9 s |
| `parametric_vector_curve` | pass | 1.9 s |
| `rho_curve` | pass | 1.9 s |
| `rho_d_curve` | pass | 1.8 s |
| `rho_d_volatility_vector_curve` | pass | 1.9 s |
| `rho_volatility_vector_curve` | pass | 1.7 s |
| `theta_curve` | pass | 1.8 s |
| `theta_volatility_vector_curve` | pass | 1.8 s |
| `vanna_curve` | pass | 2.0 s |
| `vanna_volatility_vector_curve` | pass | 1.6 s |
| `vega_curve` | pass | 1.7 s |
| `vega_volatility_vector_curve` | pass | 2.1 s |
| `veta_curve` | pass | 2.1 s |
| `veta_volatility_vector_curve` | pass | 2.0 s |
| `volatility_smile` | pass | 1.9 s |

### `examples_exotics` (2 pass)

| Target | Result | Time |
| --- | --- | ---: |
| `barrier_pricing` | pass | 0.3 s |
| `cliquet_example` | pass | 0.2 s |

### `examples_metrics` (25 pass)

| Target | Result | Time |
| --- | --- | ---: |
| `all_composite_metrics` | pass | 9.0 s |
| `all_liquidity_metrics` | pass | 5.7 s |
| `all_price_metrics` | pass | 4.7 s |
| `all_risk_metrics` | pass | 6.7 s |
| `all_stress_metrics` | pass | 9.8 s |
| `all_temporal_metrics` | pass | 9.9 s |
| `bid_ask_spread` | pass | 1.5 s |
| `charm_metrics` | pass | 3.8 s |
| `color_metrics` | pass | 3.6 s |
| `delta_gamma_profile` | pass | 3.5 s |
| `dollar_gamma_curve` | pass | 2.9 s |
| `implied_volatility_curve` | pass | 1.7 s |
| `implied_volatility_surface` | pass | 2.4 s |
| `open_interest` | pass | 1.5 s |
| `price_shock_impact` | pass | 3.4 s |
| `put_call_ratio_premium_weighted` | pass | 1.6 s |
| `risk_reversal_curve` | pass | 1.6 s |
| `smile_dynamics` | pass | 3.4 s |
| `strike_concentration_premium_weighted` | pass | 1.6 s |
| `theta_metrics` | pass | 3.4 s |
| `time_decay_profile` | pass | 3.6 s |
| `vanna_volga_surface` | pass | 2.4 s |
| `volatility_sensitivity` | pass | 3.6 s |
| `volatility_skew` | pass | 1.7 s |
| `volume_profile` | pass | 3.7 s |

### `examples_pricing` (4 pass)

| Target | Result | Time |
| --- | --- | ---: |
| `black_76` | pass | 0.3 s |
| `black_76_greeks` | pass | 0.3 s |
| `garman_kohlhagen` | pass | 0.3 s |
| `garman_kohlhagen_greeks` | pass | 0.3 s |

### `examples_simulation` (13 pass)

| Target | Result | Time |
| --- | --- | ---: |
| `exit_policy_example` | pass | 0.2 s |
| `historical_build_chain` | pass | 4.3 s |
| `long_call_strategy_simulation` | pass | 2.0 s |
| `position_simulator` | pass | 2.2 s |
| `random_walk` | pass | 2.5 s |
| `random_walk_build_chain` | pass | 25.5 s |
| `random_walk_build_series` | pass | 2.0 s |
| `random_walk_chain` | pass | 1.8 s |
| `short_put_simulation` | pass | 7.9 s |
| `short_put_strategy_simulation` | pass | 1.8 s |
| `simulator` | pass | 6.0 s |
| `strategy_simulator` | pass | 2.0 s |
| `unified_pricing` | pass | 3.3 s |

### `examples_strategies` (20 pass)

| Target | Result | Time |
| --- | --- | ---: |
| `strategy_bear_call_spread` | pass | 1.7 s |
| `strategy_bear_put_spread` | pass | 1.9 s |
| `strategy_bull_call_ladder` | pass | 1.5 s |
| `strategy_bull_call_spread` | pass | 1.7 s |
| `strategy_bull_put_spread` | pass | 1.5 s |
| `strategy_custom_complex` | pass | 1.7 s |
| `strategy_custom_dax` | pass | 13.5 s |
| `strategy_custom_short_strangle` | pass | 6.2 s |
| `strategy_custom_simple` | pass | 2.3 s |
| `strategy_graph` | pass | 1.8 s |
| `strategy_iron_butterfly` | pass | 1.6 s |
| `strategy_iron_condor` | pass | 1.9 s |
| `strategy_long_butterfly_spread` | pass | 1.8 s |
| `strategy_long_straddle` | pass | 2.1 s |
| `strategy_long_strangle` | pass | 1.6 s |
| `strategy_poor_mans_covered_call` | pass | 1.6 s |
| `strategy_short_butterfly_spread` | pass | 1.6 s |
| `strategy_short_straddle` | pass | 1.6 s |
| `strategy_short_strangle` | pass | 1.6 s |
| `strategy_short_strangle_delta_simple` | pass | 1.6 s |

### `examples_strategies_best` (28 pass)

| Target | Result | Time |
| --- | --- | ---: |
| `strategy_bear_call_spread_best_area` | pass | 1.6 s |
| `strategy_bear_call_spread_best_ratio` | pass | 1.6 s |
| `strategy_bear_put_spread_best_area` | pass | 1.6 s |
| `strategy_bear_put_spread_best_ratio` | pass | 1.7 s |
| `strategy_bull_call_ladder_best_area` | pass | 1.7 s |
| `strategy_bull_call_ladder_best_ratio` | pass | 1.6 s |
| `strategy_bull_call_spread_best_area` | pass | 1.6 s |
| `strategy_bull_call_spread_best_ratio` | pass | 1.7 s |
| `strategy_bull_put_spread_best_area` | pass | 1.7 s |
| `strategy_bull_put_spread_best_ratio` | pass | 1.6 s |
| `strategy_iron_butterfly_best_area` | pass | 1.8 s |
| `strategy_iron_butterfly_best_ratio` | pass | 2.1 s |
| `strategy_iron_condor_best_area` | pass | 5.0 s |
| `strategy_iron_condor_best_ratio` | pass | 5.2 s |
| `strategy_long_butterfly_spread_best_area` | pass | 2.6 s |
| `strategy_long_butterfly_spread_best_ratio` | pass | 2.3 s |
| `strategy_long_straddle_best_area` | pass | 1.8 s |
| `strategy_long_straddle_best_ratio` | pass | 1.8 s |
| `strategy_long_strangle_best_area` | pass | 1.7 s |
| `strategy_long_strangle_best_ratio` | pass | 1.6 s |
| `strategy_poor_mans_covered_call_best_area` | pass | 2.3 s |
| `strategy_poor_mans_covered_call_best_ratio` | pass | 1.7 s |
| `strategy_short_butterfly_spread_best_area` | pass | 2.0 s |
| `strategy_short_butterfly_spread_best_ratio` | pass | 1.9 s |
| `strategy_short_straddle_best_area` | pass | 1.9 s |
| `strategy_short_straddle_best_ratio` | pass | 1.6 s |
| `strategy_short_strangle_best_area` | pass | 1.7 s |
| `strategy_short_strangle_best_ratio` | pass | 1.6 s |

### `examples_strategies_delta` (28 pass)

| Target | Result | Time |
| --- | --- | ---: |
| `strategy_bear_call_spread_delta` | pass | 0.3 s |
| `strategy_bear_call_spread_extended_delta` | pass | 0.3 s |
| `strategy_bear_put_spread_delta` | pass | 0.2 s |
| `strategy_bear_put_spread_extended_delta` | pass | 0.3 s |
| `strategy_bull_call_ladder_delta` | pass | 0.2 s |
| `strategy_bull_call_ladder_extended_delta` | pass | 0.2 s |
| `strategy_bull_call_spread_delta` | pass | 0.2 s |
| `strategy_bull_call_spread_extended_delta` | pass | 0.3 s |
| `strategy_bull_put_spread_delta` | pass | 0.4 s |
| `strategy_bull_put_spread_extended_delta` | pass | 0.3 s |
| `strategy_iron_butterfly_delta` | pass | 0.3 s |
| `strategy_iron_butterfly_extended_delta` | pass | 0.3 s |
| `strategy_iron_condor_delta` | pass | 0.2 s |
| `strategy_iron_condor_extended_delta` | pass | 0.3 s |
| `strategy_long_butterfly_spread_delta` | pass | 0.3 s |
| `strategy_long_butterfly_spread_extended_delta` | pass | 0.3 s |
| `strategy_long_straddle_delta` | pass | 0.3 s |
| `strategy_long_straddle_extended_delta` | pass | 0.5 s |
| `strategy_long_strangle_delta` | pass | 0.2 s |
| `strategy_long_strangle_extended_delta` | pass | 0.2 s |
| `strategy_pmcc_delta` | pass | 0.4 s |
| `strategy_pmcc_extended_delta` | pass | 0.3 s |
| `strategy_short_butterfly_spread_delta` | pass | 0.3 s |
| `strategy_short_butterfly_spread_extended_delta` | pass | 0.3 s |
| `strategy_short_straddle_delta` | pass | 0.3 s |
| `strategy_short_straddle_extended_delta` | pass | 0.3 s |
| `strategy_short_strangle_delta` | pass | 0.2 s |
| `strategy_short_strangle_extended_delta` | pass | 0.3 s |

### `examples_surfaces` (13 pass)

| Target | Result | Time |
| --- | --- | ---: |
| `charm_surface` | pass | 3.1 s |
| `color_surface` | pass | 3.1 s |
| `d1_surface` | pass | 5.4 s |
| `d2_surface` | pass | 2.9 s |
| `delta_surface` | pass | 3.0 s |
| `gamma_surface` | pass | 3.0 s |
| `rho_d_surface` | pass | 2.9 s |
| `rho_surface` | pass | 3.0 s |
| `theta_surface` | pass | 3.5 s |
| `vanna_surface` | pass | 3.1 s |
| `vega_surface` | pass | 3.0 s |
| `veta_surface` | pass | 3.1 s |
| `vomma_surface` | pass | 3.0 s |

### `examples_visualization` (3 pass)

| Target | Result | Time |
| --- | --- | ---: |
| `option_graph` | pass | 1.5 s |
| `position_graph` | pass | 1.5 s |
| `simple_visualization` | pass | 0.3 s |

### `examples_volatility` (3 pass)

| Target | Result | Time |
| --- | --- | ---: |
| `test` | pass | 6.0 s |
| `volatility` | pass | 0.2 s |
| `volatility_utils` | pass | 0.4 s |

### `osl-example-direct-analytics` (1 pass)

| Target | Result | Time |
| --- | --- | ---: |
| `osl-example-direct-analytics` | pass | 0.3 s |

### `osl-example-direct-backtest` (1 pass)

| Target | Result | Time |
| --- | --- | ---: |
| `osl-example-direct-backtest` | pass | 0.7 s |

### `osl-example-direct-market` (1 pass)

| Target | Result | Time |
| --- | --- | ---: |
| `osl-example-direct-market` | pass | 0.3 s |

### `osl-example-direct-math` (1 pass)

| Target | Result | Time |
| --- | --- | ---: |
| `osl-example-direct-math` | pass | 0.2 s |

### `osl-example-direct-pricing` (1 pass)

| Target | Result | Time |
| --- | --- | ---: |
| `osl-example-direct-pricing` | pass | 0.2 s |

### `osl-example-direct-simulation` (1 pass)

| Target | Result | Time |
| --- | --- | ---: |
| `osl-example-direct-simulation` | pass | 0.3 s |

### `osl-example-direct-strategies` (1 pass)

| Target | Result | Time |
| --- | --- | ---: |
| `osl-example-direct-strategies` | pass | 0.2 s |

### `osl-example-direct-visualization` (1 pass)

| Target | Result | Time |
| --- | --- | ---: |
| `osl-example-direct-visualization` | pass | 0.3 s |

## Benches

### `optionstratlib` (1 pass)

| Target | Result | Time |
| --- | --- | ---: |
| `benches` | pass | 0.7 s |

### `optionstratlib-analytics` (1 pass)

| Target | Result | Time |
| --- | --- | ---: |
| `analytics` | pass | 0.4 s |

### `optionstratlib-backtest` (1 pass)

| Target | Result | Time |
| --- | --- | ---: |
| `backtest` | pass | 2.3 s |

### `optionstratlib-core` (1 pass)

| Target | Result | Time |
| --- | --- | ---: |
| `core` | pass | 0.5 s |

### `optionstratlib-market` (3 pass)

| Target | Result | Time |
| --- | --- | ---: |
| `chains` | pass | 0.7 s |
| `chains_io` | pass | 0.7 s |
| `synthetic` | pass | 0.4 s |

### `optionstratlib-math` (1 pass)

| Target | Result | Time |
| --- | --- | ---: |
| `math` | pass | 0.5 s |

### `optionstratlib-pricing` (3 pass)

| Target | Result | Time |
| --- | --- | ---: |
| `greeks` | pass | 0.3 s |
| `pricing` | pass | 4.1 s |
| `volatility` | pass | 0.4 s |

### `optionstratlib-simulation` (1 pass)

| Target | Result | Time |
| --- | --- | ---: |
| `simulation` | pass | 0.6 s |

### `optionstratlib-strategies` (1 pass)

| Target | Result | Time |
| --- | --- | ---: |
| `strategies` | pass | 1.4 s |

### `optionstratlib-visualization` (1 pass)

| Target | Result | Time |
| --- | --- | ---: |
| `visualization` | pass | 0.4 s |
