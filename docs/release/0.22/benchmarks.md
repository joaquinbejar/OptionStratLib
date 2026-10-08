# 0.22 benchmarks: evidence

Criterion run of every bench target in the workspace (#789): the facade's
`benches/mod.rs` and the thirteen component targets added by #794, 558
benchmarks in all. This file records the machine, the toolchain, the exact
command, a per-crate summary, the bottlenecks the numbers show, and the full
results. The bottlenecks are filed as follow-up issues grouped by crate; no
optimisation was made in #789.

## Run

- Commit measured: `ed16ffc1` (`origin/main` at `f2cb1580` plus the
  facade merge-bench change of this branch, which makes
  `curve_merge_multiply` and `surface_merge_multiply` assert that the merge
  succeeds before timing it).
- Date: 2026-10-08, 20:18 to 21:31 local time on the bench host.
- Every benchmark ran once in `--test` mode first; all 558 succeeded. Each
  Criterion bench in the component crates also runs its fixture once and
  panics on an error, so no number below is the time of an error branch.

## Machine

| Item | Value |
| --- | --- |
| Host | `flumix`, dedicated bench machine on the local network |
| CPU | 12th Gen Intel Core i7-12650H: 10 cores (6 performance cores with 2 threads each at up to 4.6 to 4.7 GHz, 4 efficiency cores at up to 3.5 GHz), 16 threads, 1 socket, 1 NUMA node, AVX2 |
| Caches | L1d 416 KiB (10 instances), L2 9.5 MiB (7 instances), L3 24 MiB |
| Memory | 31 GiB (32 639 160 kB) |
| OS | Ubuntu 24.04.4 LTS |
| Kernel | `Linux 6.8.0-139-generic #139-Ubuntu SMP PREEMPT_DYNAMIC x86_64` |
| CPU governor | `performance`, turbo enabled |
| Background load | Idle Docker stacks (Portainer, one web service, a BuildKit builder) left running; load average about 1 before the run |

The CPU is hybrid. Benchmarks were not pinned to the performance cores, so a
single-threaded benchmark may be scheduled on an efficiency core for part of
a sample; the confidence intervals below show this was rare (see Noise).

## Toolchain

| Tool | Version |
| --- | --- |
| rustc | `rustc 1.99.0 (b940084d7 2026-09-28)` |
| cargo | `cargo 1.99.0 (5f94df478 2026-08-27)` |
| criterion | `0.8.2` (`default-features = false`, `html_reports`) |
| profile | `bench` (release, workspace defaults) |

## Command

`cargo bench --workspace --all-features` does not build on a machine
without Chrome: `static_export` pulls `plotly_static`, whose build script
needs a browser, and the `examples_*` members enable it. No bench renders an
image, so the run selects the ten library packages and every feature the
bench targets need except `static_export`:

```sh
cargo bench \
  -p optionstratlib -p optionstratlib-core -p optionstratlib-math \
  -p optionstratlib-pricing -p optionstratlib-simulation -p optionstratlib-market \
  -p optionstratlib-analytics -p optionstratlib-strategies -p optionstratlib-backtest \
  -p optionstratlib-visualization \
  --features "optionstratlib/plotly optionstratlib/io optionstratlib/synthetic optionstratlib-market/io optionstratlib-market/synthetic" \
  --bench '*' -- --warm-up-time 2 --measurement-time 4 --noplot
```

All 14 bench targets built and ran with it (`benches`, `core`, `math`,
`pricing`, `greeks`, `volatility`, `simulation`, `chains`, `chains_io`,
`synthetic`, `analytics`, `strategies`, `backtest`, `visualization`). Default
sample count 100; groups whose iterations are slow set 10 or 20 in the bench
source. Criterion lengthened the measurement window on its own where 4 s
could not hold the requested samples (25 warnings of the "Unable to complete
N samples" kind); those results are complete.

## Noise

534 of the 558 confidence intervals are narrower than 5% of the estimate.
The 24 wider ones are mostly benches that run on the rayon pool
(`surface_merge_multiply/side_8/2` 21%, the synthetic generators 8 to 14%,
`curve_merge_multiply/size_128/3` 14%) or last under a nanosecond
(`to_dec`, 12 to 14%). A change smaller than its interval is within noise.

## Summary per crate

Representative estimates; the full tables follow.

| Crate | Path | Estimate | Note |
| --- | --- | ---: | --- |
| core | `d_add` / `d_mul` / `d_div` | 11.7 ns / 5.3 ns / 41.6 ns | checked `Decimal` arithmetic is cheap |
| core | `d_exp` / `d_sqrt` / `d_ln` / `d_powd` | 764 ns / 797 ns / 8.00 µs / 10.6 µs | transcendental `Decimal` functions dominate every kernel that calls them |
| core | `decimal_to_f64` / `f64_to_decimal` | 13.3 ns / 148 ns | the way back from `f64` costs 11x the way out |
| core | `calculate_log_returns`, 1 008 prices | 8.19 ms | 8.1 µs per return: one `ln` each |
| core | `Position::pnl_at_expiration`, 201-point sweep | 28.1 µs | |
| math | linear interpolation, 32 / 2 048 points | 1.16 µs / 57.3 µs | O(n) per read |
| math | spline interpolation, 32 / 2 048 points | 17.4 µs / 1.23 ms | rebuilds the spline per read |
| math | 100 spline reads on 128 points | 6.97 ms | |
| math | `Curve::intersect_with`, 512 x 512 | 9.37 ms | O(n * m) |
| math | `Surface::merge_with` (add), 16 x 16 | 62.4 ms | 2 601 grid reads |
| pricing | Black-Scholes, strike 100 on spot 105 | 20.6 µs | `d1` alone 7.74 µs |
| pricing | Black-Scholes, at the money (facade fixture) | 5.40 µs | `ln(1)` is free, see note below |
| pricing | Greek snapshot (12 Greeks) / single delta | 17.2 µs / 9.32 µs | the shared kernels pay off: the 12 singles sum to about 153 µs |
| pricing | binomial European, 200 / 1 000 steps | 5.35 ms / 109 ms | |
| pricing | binomial American put, 200 / 1 000 steps | 36.7 ms / 1.21 s | 7 to 11x the European tree |
| pricing | Monte-Carlo, 30 steps x 10 000 paths | 420 ms | 0.71 M steps/s |
| pricing | telegraph, 30 steps x 10 000 paths | 7.11 ms | 42 M steps/s, `f64` kernel |
| pricing | `implied_volatility` (grid), at the money / deep OTM | 7.85 ms / 31.5 ms | |
| pricing | `calculate_implied_volatility`, 50-strike smile | 15.1 ms | 302 µs per strike |
| pricing | `historical_volatility`, window 21, 1 008 returns | 3.63 ms | O(n * w) |
| pricing | exotics | 1.0 µs (power) to 321 µs (rainbow) | |
| simulation | Brownian / GBM / Heston / Historical walk, 1 008 steps | 345 µs / 1.86 ms / 6.32 ms / 12.0 ms | |
| simulation | `Simulator::new`, 1 000 paths x 30 steps | 55.4 ms | serial over paths |
| market | `build_chain`, 21 / 201 strikes | 1.67 ms / 16.7 ms | about 80 µs per strike |
| market | `update_greeks`, 101 strikes | 3.48 ms | |
| market | `vega_exposure` / `gamma_exposure`, 101 strikes | 2.72 ms / 944 ns | vega recomputed, gamma stored |
| market | JSON in memory / `save_to_json` / `load_from_json`, 101 strikes | 83 µs and 95 µs / 4.80 ms / 14.8 ms | unbuffered file I/O |
| market | `read_ohlcv_from_zip`, 74 061 rows | 25.7 ms | |
| market | `generator_optionchain`, 30 steps, 21 strikes | 11.5 ms | |
| analytics | position `calculate_pnl` / SPAN margin | 40.7 µs / 313 µs | Black-Scholes bound |
| analytics | `calculate_rnd`, 21 strikes, 50 and 200 points | 21.5 µs and 21.5 µs | identical: the point count is not used |
| analytics | `volatility_sensitivity_surface`, 20 x 20 | 7.93 ms | |
| strategies | construction, 1 to 4 legs | 0.75 to 2.25 µs | |
| strategies | iron condor profit at a price / Greeks | 444 ns / 64.2 µs | |
| strategies | bull call spread optimiser, SP500 45 strikes | 2.36 ms | |
| strategies | long butterfly optimiser, SP500 45 strikes | 99.0 ms | |
| strategies | iron condor optimiser, 21 / 41 / SP500 45 strikes | 41.7 ms / 682 ms / 935 ms | O(n^4) |
| backtest | long call to expiration, 100 / 1 000 paths x 30 steps | 60.9 ms / 614 ms | one Black-Scholes per step, serial |
| backtest | short put with profit-or-loss exit, 1 000 paths | 21.5 ms | |
| visualization | chart data: curve of 2 048 points / iron condor payoff / 100 walks x 252 | 17.6 µs / 602 µs / 28.5 µs | no bottleneck |

Note on the facade's own benches: `Pricing Methods/black_scholes` and
`Greeks Calculations/*` use an at-the-money fixture, where `S / K = 1` makes
the logarithm in `d1` trivial. They report 5.4 µs for a price and 3.1 µs for
a delta, against 20.6 µs and 9.3 µs on the component benches' strike 100
on spot 105. The off-the-money figures are the ones a chain pays.

The facade's `curve_merge_multiply` and `surface_merge_multiply` now time
successful merges only (this branch). Before #795 some combinations failed
fast and were timed as their error branch, so these rows are not comparable
with earlier measurements of the same ids.

## Bottlenecks

Each item names the measurement, the cause found in the source, and the
suggested fix. They are drafted as issues grouped by crate. "Bit-identical"
marks a fix that cannot change any result; the others need an accuracy
comparison and quant review before they land.

### optionstratlib-core

- **Transcendental `Decimal` functions.** `d_ln` costs 8.00 µs, `d_powd`
  10.6 µs, `d_sqrt` 797 ns and `d_exp` 764 ns, against 5 to 42 ns for the
  arithmetic. Every Black-Scholes `d1` pays one `ln`, `calculate_log_returns`
  pays one per price (8.19 ms for 1 008), and the GBM walks one `exp` per
  step. Fix: `f64` kernels for `ln`, `exp`, `sqrt` and `pow` inside the
  numeric internals, with finite checks and one controlled conversion back to
  `Decimal`, keeping the `Decimal` functions at the public boundary.
- **`f64_to_decimal` allocates on success.** 148 ns against 13 ns for
  `decimal_to_f64`: it builds its error with `ok_or(...)`, which formats the
  value and allocates two `String`s on every call. Fix: `ok_or_else`.
  Bit-identical; low impact (seven call sites).

### optionstratlib-math

- **Interpolation is O(n) and allocates on every read.**
  `find_bracket_points` collects `get_points()` (a fresh `BTreeSet<&Point>`)
  into a `Vec` and scans it linearly: linear interpolation costs 1.16 µs at
  32 points and 57.3 µs at 2 048. Surface reads scale with the point count
  too (about 300 µs at 32 x 32, every method), and `Surface::merge_with`
  reads both surfaces at each of its 2 601 grid points (62.4 ms at 16 x 16,
  on the rayon pool). Fix: bracket with `BTreeSet::range` (O(log n), no
  allocation) and a neighbour search on the surface grid that does not scan
  every point. Bit-identical.
- **Spline interpolation rebuilds the spline per read.** 17.4 µs at 32
  points, 1.23 ms at 2 048; 100 reads on 128 points take 6.97 ms, 15x the
  linear sweep. Fix: solve the second derivatives once per curve (a prepared
  interpolator or a batch read API). Bit-identical.
- **`Curve::intersect_with` is O(n * m) and rebuilds the other curve's point
  set per outer point.** 9.37 ms at 512 x 512. Fix: a two-pointer walk over
  the sorted abscissas, O(n + m). Bit-identical.

### optionstratlib-pricing

- **Black-Scholes computes `d1` twice.** `calculate_d_values_with_yield`
  calls `d1()` and then `d2()`, and `d2()` calls `d1()` again, so every
  price pays two logarithms: `d1` is 7.74 µs of the 20.6 µs price. Fix:
  `d2 = d1 - sigma * sqrt(T)` from the `d1` already computed, which is what
  `d2()` returns. Bit-identical; about a third off every price, and off
  everything priced through it (chain construction, P&L, SPAN, backtests).
- **The American binomial tree computes two `d_powd` per node.**
  `lattice_spot` evaluates `u^i` and `d^(step - i)` for every node of every
  step: the American put costs 36.7 ms at 200 steps and 1.21 s at 1 000,
  7 to 11x the European tree. Fix: a power table built once per tree (O(n)
  powers instead of O(n^2)). The European tree itself is 109 ms at 1 000
  steps (217 ns per node); an `f64` lattice is the larger follow-up.
- **Monte-Carlo pays a square root and a conversion per step.**
  `wiener_increment` recomputes `d_sqrt(dt)` on every step (loop invariant)
  and converts each `f64` normal sample to `Decimal`: 1.4 µs per path-step,
  420 ms for 30 x 10 000, against 24 ns per step for the `f64` telegraph
  kernel. Fix: hoist `sqrt(dt)` (bit-identical), then an `f64` path kernel.
- **Implied-volatility solvers.** `implied_volatility` scans a grid of
  `100 * max_iterations` candidates: 7.85 ms at the money and 31.5 ms deep
  out of the money with `max_iterations = 100`; `calculate_iv` (the same
  grid with 10) 0.77 to 3.09 ms; `calculate_implied_volatility` 80 to 294 µs
  and 15.1 ms for a 50-strike smile. Fix: Newton on vega with a bracketing
  fallback on the shared Black-Scholes kernel, used by all three.
- **`historical_volatility` recomputes each window.** 3.63 ms for 1 008
  returns with a 21-return window, 32x `constant_volatility` over the same
  series. Fix: rolling sums of `r` and `r^2`, O(n). `ewma_volatility` takes
  a `Decimal` square root per element (1.90 ms for 1 008).

### optionstratlib-simulation

- **Stochastic walks pay `Decimal` transcendentals per step.** GBM and
  log-returns cost 1.8 µs per step against 0.34 µs for the Brownian walk
  (one `d_exp` per step); Heston 6.3 µs, GARCH 3.4 µs, Historical 12 µs
  (log returns and expanding-window volatilities, 12.2 ms for 1 008 prices).
  Fix: the per-step state in `f64` with a checked conversion of each emitted
  price.
- **`Simulator::new` is serial over paths.** 5.54 ms for 100 paths and
  55.4 ms for 1 000 (30 steps), linear, one core busy of 16. Seeded walks
  already derive one seed per path, so the paths can be generated on the
  rayon pool and collected in index order without changing any path.

### optionstratlib-market

- **Chain JSON I/O is unbuffered.** `save_to_json` hands a bare `File` to
  `serde_json::to_writer_pretty` and `load_from_json` one to
  `serde_json::from_reader`: 4.80 ms to save 101 strikes against 83 µs to
  serialize them, 14.8 ms to load against 95 µs to parse (3.48 ms of the load
  is the intended `update_greeks`). Fix: `BufWriter` / `BufReader`.
  Bit-identical.
- **Seven exposures recompute their Greek.** `vega_exposure`,
  `theta_exposure`, `vanna_exposure`, `vomma_exposure`, `veta_exposure`,
  `charm_exposure` and `color_exposure` build two `Options` per strike and
  re-run the Greek: 2.72 ms for vega at 101 strikes, while `gamma_exposure`
  and `delta_exposure` read the stored values (944 ns and 5.8 µs). Fix: read
  the per-style Greek snapshots when the chain carries them, compute only
  when it does not.
- **Chain construction is serial.** `build_chain` costs about 80 µs per
  strike (16.7 ms for 201) and `update_greeks` 34 µs per strike, both one
  strike after another and both dominated by Black-Scholes. Fix: the pricing
  items above first; then price strikes on the rayon pool above a measured
  size threshold.

### optionstratlib-strategies

- **The optimisers are serial and build each candidate twice.**
  `filter_combinations` constructs, validates and bounds every combination
  to filter it, then `find_optimal` constructs it again to score it. The
  four-leg search grows as n^4: iron condor 2.49 ms at 11 strikes, 41.7 ms
  at 21, 682 ms at 41 and 935 ms on the 45-strike SP500 chain; the long
  butterfly 99.0 ms on SP500. Fix: build each candidate once, score
  candidates on the rayon pool with a deterministic tie-break, and reject on
  side and quotes before materialising a strategy.

### optionstratlib-backtest

- **Single-leg backtests reprice every step serially.** The path evaluator
  clones the option and runs Black-Scholes at every step: a long call held
  to expiration costs 60.9 ms for 100 paths and 614 ms for 1 000 (20 µs per
  step, one price). Fix: evaluate paths on the rayon pool in index order,
  reuse one scratch option per path, and take the Black-Scholes fix above.

### No bottleneck found

- **optionstratlib-analytics**: P&L (40.7 µs), SPAN (313 µs), the stress
  surfaces and the projections are all bound by Black-Scholes and follow the
  pricing fixes; nothing analytics-specific stands out.
- **optionstratlib-visualization**: chart data generation is 0.2 to 602 µs;
  the largest, a payoff chart, is a profit sweep of the strategy.
- **Measured and not worth an issue**: strike lookups in a chain are linear
  but cost 50 ns to 1.4 µs at 101 strikes; `DecimalStats::std_dev` costs
  137 ns per element; the leg iterators enumerate 250 000 quadruples in
  9.9 ms at 51 strikes, a small share of the optimiser.

### Correctness findings

Found while reading the hot paths; not performance, reported separately.

- `monte_carlo_option_pricing` ignores `option_style` and `side`: the
  payoff is always `max(S_T - K, 0)`, so a put is priced as a call.
- `RNDParameters::interpolation_points` is never read by `calculate_rnd`
  (50 and 200 points measure identically).
- `Curve::intersect_with` reports sample points that coincide within 1e-6
  in both coordinates, not crossings between samples.

## Results

Lower and upper are the bounds of Criterion's 95% confidence interval of
the mean; throughput is shown where the bench declares one.

### optionstratlib-core: `core`

| Benchmark | Lower | Estimate | Upper | Throughput |
|---|---:|---:|---:|---:|
| `core/decimal/d_add` | 11.435 ns | 11.713 ns | 12.059 ns |  |
| `core/decimal/d_mul` | 5.2966 ns | 5.3117 ns | 5.3362 ns |  |
| `core/decimal/d_div` | 41.490 ns | 41.628 ns | 41.756 ns |  |
| `core/decimal/d_exp` | 762.95 ns | 764.43 ns | 767.28 ns |  |
| `core/decimal/d_ln` | 8.0000 µs | 8.0010 µs | 8.0021 µs |  |
| `core/decimal/d_sqrt` | 796.12 ns | 796.71 ns | 797.29 ns |  |
| `core/decimal/d_powd` | 10.631 µs | 10.632 µs | 10.635 µs |  |
| `core/decimal/decimal_to_f64` | 13.288 ns | 13.292 ns | 13.298 ns |  |
| `core/decimal/f64_to_decimal` | 147.84 ns | 147.89 ns | 147.95 ns |  |
| `core/decimal/d_sum/21` | 73.816 ns | 73.932 ns | 74.041 ns | 284.04 Melem/s |
| `core/decimal/mean/21` | 81.798 ns | 81.944 ns | 82.096 ns | 256.27 Melem/s |
| `core/decimal/std_dev/21` | 3.5673 µs | 3.5742 µs | 3.5823 µs | 5.8754 Melem/s |
| `core/decimal/d_sum/252` | 3.0581 µs | 3.0610 µs | 3.0645 µs | 82.327 Melem/s |
| `core/decimal/mean/252` | 3.0414 µs | 3.0427 µs | 3.0440 µs | 82.821 Melem/s |
| `core/decimal/std_dev/252` | 31.628 µs | 31.690 µs | 31.765 µs | 7.9521 Melem/s |
| `core/decimal/d_sum/1008` | 22.728 µs | 22.749 µs | 22.772 µs | 44.309 Melem/s |
| `core/decimal/mean/1008` | 22.695 µs | 22.754 µs | 22.842 µs | 44.300 Melem/s |
| `core/decimal/std_dev/1008` | 135.62 µs | 138.48 µs | 141.99 µs | 7.2790 Melem/s |
| `core/positive/new_from_f64` | 35.192 ns | 35.275 ns | 35.384 ns |  |
| `core/positive/add` | 10.576 ns | 10.623 ns | 10.674 ns |  |
| `core/positive/mul` | 10.703 ns | 10.722 ns | 10.745 ns |  |
| `core/positive/checked_div` | 25.033 ns | 25.060 ns | 25.089 ns |  |
| `core/positive/to_f64` | 13.963 ns | 13.969 ns | 13.974 ns |  |
| `core/positive/to_dec` | 280.89 ps | 301.16 ps | 324.35 ps |  |
| `core/positive/calculate_log_returns/21` | 160.13 µs | 160.17 µs | 160.22 µs | 131.11 Kelem/s |
| `core/positive/calculate_log_returns/252` | 2.0388 ms | 2.0421 ms | 2.0471 ms | 123.40 Kelem/s |
| `core/positive/calculate_log_returns/1008` | 8.1921 ms | 8.1928 ms | 8.1936 ms | 123.04 Kelem/s |
| `core/payoff/option_type_payoff` | 73.112 ns | 73.256 ns | 73.457 ns |  |
| `core/payoff/options_payoff` | 48.552 ns | 48.606 ns | 48.659 ns |  |
| `core/payoff/options_payoff_at_price` | 89.878 ns | 90.089 ns | 90.403 ns |  |
| `core/payoff/options_intrinsic_value` | 89.209 ns | 89.264 ns | 89.323 ns |  |
| `core/payoff/position_pnl_at_expiration` | 154.15 ns | 154.54 ns | 155.07 ns |  |
| `core/payoff/position_unrealized_pnl` | 36.979 ns | 37.028 ns | 37.079 ns |  |
| `core/payoff/position_break_even` | 95.005 ns | 95.139 ns | 95.259 ns |  |
| `core/payoff/position_total_cost` | 48.612 ns | 49.943 ns | 51.511 ns |  |
| `core/payoff/position_pnl_at_expiration_sweep/201` | 28.033 µs | 28.056 µs | 28.077 µs | 7.1644 Melem/s |
| `core/construction/options_new` | 271.43 ns | 271.58 ns | 271.74 ns |  |
| `core/construction/position_new` | 410.62 ns | 411.02 ns | 411.53 ns |  |
| `core/construction/options_clone` | 19.544 ns | 19.599 ns | 19.651 ns |  |
| `core/construction/options_time_to_expiration` | 97.240 ns | 97.306 ns | 97.376 ns |  |
| `core/construction/expiration_days_get_years` | 97.358 ns | 97.433 ns | 97.521 ns |  |
| `core/construction/options_to_json` | 340.76 ns | 341.38 ns | 341.96 ns |  |
| `core/construction/options_from_json` | 447.24 ns | 448.87 ns | 451.61 ns |  |

### optionstratlib-math: `math`

| Benchmark | Lower | Estimate | Upper | Throughput |
|---|---:|---:|---:|---:|
| `math/construction/curve_new/32` | 397.98 ns | 399.88 ns | 402.04 ns | 80.024 Melem/s |
| `math/construction/curve_from_vector/32` | 716.50 ns | 719.43 ns | 722.33 ns | 44.479 Melem/s |
| `math/construction/curve_parametric/32` | 36.096 µs | 36.676 µs | 37.315 µs | 872.50 Kelem/s |
| `math/construction/curve_new/128` | 1.6181 µs | 1.6202 µs | 1.6224 µs | 79.003 Melem/s |
| `math/construction/curve_from_vector/128` | 2.7770 µs | 2.7818 µs | 2.7870 µs | 46.013 Melem/s |
| `math/construction/curve_parametric/128` | 89.096 µs | 89.674 µs | 90.240 µs | 1.4274 Melem/s |
| `math/construction/curve_new/512` | 6.8612 µs | 6.8660 µs | 6.8709 µs | 74.571 Melem/s |
| `math/construction/curve_from_vector/512` | 11.530 µs | 11.772 µs | 12.084 µs | 43.494 Melem/s |
| `math/construction/curve_parametric/512` | 162.08 µs | 164.10 µs | 166.22 µs | 3.1201 Melem/s |
| `math/construction/curve_new/2048` | 28.210 µs | 28.233 µs | 28.255 µs | 72.539 Melem/s |
| `math/construction/curve_from_vector/2048` | 45.975 µs | 46.021 µs | 46.067 µs | 44.502 Melem/s |
| `math/construction/curve_parametric/2048` | 584.48 µs | 588.49 µs | 592.55 µs | 3.4801 Melem/s |
| `math/construction/surface_from_vector/8x8` | 2.2223 µs | 2.2251 µs | 2.2278 µs | 28.763 Melem/s |
| `math/construction/surface_parametric/8x8` | 74.016 µs | 75.133 µs | 76.268 µs | 851.83 Kelem/s |
| `math/construction/surface_from_vector/16x16` | 9.4834 µs | 9.4938 µs | 9.5047 µs | 26.965 Melem/s |
| `math/construction/surface_parametric/16x16` | 135.20 µs | 146.50 µs | 160.10 µs | 1.7474 Melem/s |
| `math/construction/surface_from_vector/32x32` | 40.054 µs | 40.109 µs | 40.166 µs | 25.531 Melem/s |
| `math/construction/surface_parametric/32x32` | 436.31 µs | 441.36 µs | 446.94 µs | 2.3201 Melem/s |
| `math/curve_interpolation/linear/32` | 1.1544 µs | 1.1552 µs | 1.1560 µs |  |
| `math/curve_interpolation/bilinear/32` | 1.3538 µs | 1.3568 µs | 1.3600 µs |  |
| `math/curve_interpolation/cubic/32` | 1.5305 µs | 1.5317 µs | 1.5330 µs |  |
| `math/curve_interpolation/spline/32` | 17.325 µs | 17.371 µs | 17.434 µs |  |
| `math/curve_interpolation/linear/128` | 3.7351 µs | 3.7374 µs | 3.7396 µs |  |
| `math/curve_interpolation/bilinear/128` | 4.0054 µs | 4.0080 µs | 4.0105 µs |  |
| `math/curve_interpolation/cubic/128` | 4.3220 µs | 4.3258 µs | 4.3298 µs |  |
| `math/curve_interpolation/spline/128` | 69.165 µs | 69.228 µs | 69.304 µs |  |
| `math/curve_interpolation/linear/512` | 14.628 µs | 14.638 µs | 14.649 µs |  |
| `math/curve_interpolation/bilinear/512` | 15.175 µs | 15.188 µs | 15.200 µs |  |
| `math/curve_interpolation/cubic/512` | 15.536 µs | 15.551 µs | 15.565 µs |  |
| `math/curve_interpolation/spline/512` | 293.08 µs | 293.24 µs | 293.41 µs |  |
| `math/curve_interpolation/linear/2048` | 57.256 µs | 57.313 µs | 57.380 µs |  |
| `math/curve_interpolation/bilinear/2048` | 59.234 µs | 59.271 µs | 59.307 µs |  |
| `math/curve_interpolation/cubic/2048` | 59.988 µs | 60.140 µs | 60.372 µs |  |
| `math/curve_interpolation/spline/2048` | 1.2285 ms | 1.2309 ms | 1.2350 ms |  |
| `math/curve_interpolation/linear_sweep_100/128` | 402.12 µs | 402.44 µs | 402.77 µs | 248.48 Kelem/s |
| `math/curve_interpolation/bilinear_sweep_100/128` | 432.68 µs | 433.26 µs | 433.84 µs | 230.81 Kelem/s |
| `math/curve_interpolation/cubic_sweep_100/128` | 458.72 µs | 459.52 µs | 460.32 µs | 217.62 Kelem/s |
| `math/curve_interpolation/spline_sweep_100/128` | 6.9537 ms | 6.9659 ms | 6.9835 ms | 14.356 Kelem/s |
| `math/surface_interpolation/linear/8x8` | 16.546 µs | 16.552 µs | 16.558 µs |  |
| `math/surface_interpolation/bilinear/8x8` | 15.882 µs | 15.916 µs | 15.977 µs |  |
| `math/surface_interpolation/cubic/8x8` | 24.468 µs | 24.478 µs | 24.490 µs |  |
| `math/surface_interpolation/spline/8x8` | 9.6005 µs | 9.6026 µs | 9.6048 µs |  |
| `math/surface_interpolation/linear/16x16` | 67.145 µs | 67.174 µs | 67.205 µs |  |
| `math/surface_interpolation/bilinear/16x16` | 63.975 µs | 64.035 µs | 64.115 µs |  |
| `math/surface_interpolation/cubic/16x16` | 74.103 µs | 74.158 µs | 74.230 µs |  |
| `math/surface_interpolation/spline/16x16` | 41.628 µs | 41.635 µs | 41.640 µs |  |
| `math/surface_interpolation/linear/32x32` | 308.92 µs | 309.52 µs | 310.37 µs |  |
| `math/surface_interpolation/bilinear/32x32` | 296.17 µs | 296.32 µs | 296.51 µs |  |
| `math/surface_interpolation/cubic/32x32` | 303.42 µs | 313.13 µs | 325.01 µs |  |
| `math/surface_interpolation/spline/32x32` | 202.88 µs | 203.17 µs | 203.45 µs |  |
| `math/metrics/curve_metrics/128` | 301.05 µs | 301.64 µs | 302.62 µs | 424.35 Kelem/s |
| `math/metrics/curve_metrics/512` | 1.2907 ms | 1.2910 ms | 1.2914 ms | 396.59 Kelem/s |
| `math/metrics/surface_metrics/8x8` | 111.28 µs | 111.31 µs | 111.34 µs | 574.99 Kelem/s |
| `math/metrics/surface_metrics/16x16` | 498.59 µs | 498.73 µs | 498.86 µs | 513.30 Kelem/s |
| `math/transformations/curve_translate/512` | 27.545 µs | 27.596 µs | 27.642 µs |  |
| `math/transformations/curve_scale/512` | 26.593 µs | 26.617 µs | 26.639 µs |  |
| `math/transformations/curve_extrema/512` | 4.6437 µs | 4.7124 µs | 4.8040 µs |  |
| `math/transformations/curve_measure_under/512` | 79.441 µs | 79.602 µs | 79.754 µs |  |
| `math/transformations/curve_derivative_at/512` | 11.064 µs | 11.091 µs | 11.116 µs |  |
| `math/transformations/curve_intersect_with/512` | 9.3554 ms | 9.3658 ms | 9.3769 ms |  |
| `math/transformations/curve_merge_with_add/512` | 648.08 µs | 649.69 µs | 651.57 µs |  |
| `math/transformations/surface_translate/16x16` | 20.584 µs | 20.609 µs | 20.635 µs |  |
| `math/transformations/surface_extrema/16x16` | 2.1806 µs | 2.1853 µs | 2.1902 µs |  |
| `math/transformations/surface_merge_with_add/16x16` | 62.237 ms | 62.364 ms | 62.514 ms |  |

### optionstratlib-pricing: `pricing`

| Benchmark | Lower | Estimate | Upper | Throughput |
|---|---:|---:|---:|---:|
| `pricing/closed_form/black_scholes` | 20.603 µs | 20.610 µs | 20.619 µs |  |
| `pricing/closed_form/calculate_price_black_scholes` | 20.604 µs | 20.606 µs | 20.609 µs |  |
| `pricing/closed_form/black_76` | 19.527 µs | 19.529 µs | 19.532 µs |  |
| `pricing/closed_form/garman_kohlhagen` | 20.670 µs | 20.675 µs | 20.681 µs |  |
| `pricing/closed_form/barone_adesi_whaley_put` | 231.57 µs | 231.64 µs | 231.72 µs |  |
| `pricing/closed_form/probability_keep_under_strike` | 9.6266 µs | 9.6278 µs | 9.6290 µs |  |
| `pricing/closed_form/black_scholes_chain/50` | 969.43 µs | 971.21 µs | 974.67 µs | 51.482 Kelem/s |
| `pricing/binomial/european/10` | 26.188 µs | 26.863 µs | 27.700 µs | 372.26 Kelem/s |
| `pricing/binomial/american_put/10` | 51.360 µs | 51.370 µs | 51.382 µs | 194.67 Kelem/s |
| `pricing/binomial/price_binomial/10` | 51.529 µs | 51.662 µs | 51.887 µs | 193.56 Kelem/s |
| `pricing/binomial/european/50` | 384.43 µs | 384.62 µs | 384.84 µs | 130.00 Kelem/s |
| `pricing/binomial/american_put/50` | 1.6800 ms | 1.6828 ms | 1.6866 ms | 29.712 Kelem/s |
| `pricing/binomial/price_binomial/50` | 1.6931 ms | 1.6934 ms | 1.6937 ms | 29.527 Kelem/s |
| `pricing/binomial/european/200` | 5.3370 ms | 5.3477 ms | 5.3621 ms | 37.399 Kelem/s |
| `pricing/binomial/american_put/200` | 36.714 ms | 36.720 ms | 36.726 ms | 5.4466 Kelem/s |
| `pricing/binomial/price_binomial/200` | 36.281 ms | 36.541 ms | 36.888 ms | 5.4734 Kelem/s |
| `pricing/binomial/european/1000` | 108.60 ms | 108.68 ms | 108.83 ms | 9.2012 Kelem/s |
| `pricing/binomial/american_put/1000` | 1.2116 s | 1.2138 s | 1.2169 s |  |
| `pricing/binomial/price_binomial/1000` | 1.2467 s | 1.2490 s | 1.2522 s |  |
| `pricing/binomial/generate_binomial_tree/10` | 49.460 µs | 49.478 µs | 49.501 µs | 202.11 Kelem/s |
| `pricing/binomial/generate_binomial_tree/50` | 1.6283 ms | 1.6285 ms | 1.6287 ms | 30.704 Kelem/s |
| `pricing/binomial/generate_binomial_tree/200` | 34.891 ms | 35.321 ms | 35.873 ms | 5.6624 Kelem/s |
| `pricing/stochastic/monte_carlo/30_steps_x_1000_paths` | 41.884 ms | 41.899 ms | 41.910 ms | 716.01 Kelem/s |
| `pricing/stochastic/monte_carlo/30_steps_x_10000_paths` | 418.98 ms | 419.94 ms | 421.04 ms | 714.40 Kelem/s |
| `pricing/stochastic/monte_carlo/252_steps_x_1000_paths` | 355.83 ms | 355.98 ms | 356.17 ms | 707.90 Kelem/s |
| `pricing/stochastic/telegraph/30_steps_x_1000_paths` | 716.58 µs | 718.10 µs | 721.59 µs | 41.777 Melem/s |
| `pricing/stochastic/telegraph/30_steps_x_10000_paths` | 7.1097 ms | 7.1137 ms | 7.1196 ms | 42.172 Melem/s |
| `pricing/stochastic/telegraph/252_steps_x_1000_paths` | 2.2598 ms | 2.2639 ms | 2.2722 ms | 111.31 Melem/s |
| `pricing/stochastic/simulate_returns/252` | 1.4625 ms | 1.4630 ms | 1.4637 ms | 172.25 Kelem/s |
| `pricing/stochastic/simulate_returns/1008` | 5.8493 ms | 5.9681 ms | 6.2301 ms | 168.90 Kelem/s |
| `pricing/exotic/asian_arithmetic` | 22.568 µs | 22.572 µs | 22.577 µs |  |
| `pricing/exotic/asian_geometric` | 21.596 µs | 21.664 µs | 21.801 µs |  |
| `pricing/exotic/barrier_up_and_out` | 69.036 µs | 69.048 µs | 69.062 µs |  |
| `pricing/exotic/binary_cash_or_nothing` | 10.678 µs | 10.680 µs | 10.682 µs |  |
| `pricing/exotic/lookback_floating` | 5.0693 µs | 5.0709 µs | 5.0727 µs |  |
| `pricing/exotic/compound` | 31.463 µs | 31.470 µs | 31.479 µs |  |
| `pricing/exotic/chooser` | 29.620 µs | 29.633 µs | 29.647 µs |  |
| `pricing/exotic/cliquet` | 44.504 µs | 44.512 µs | 44.520 µs |  |
| `pricing/exotic/rainbow_best_of` | 320.57 µs | 320.62 µs | 320.68 µs |  |
| `pricing/exotic/spread` | 16.722 µs | 16.738 µs | 16.756 µs |  |
| `pricing/exotic/quanto` | 11.590 µs | 11.600 µs | 11.613 µs |  |
| `pricing/exotic/exchange` | 11.288 µs | 11.290 µs | 11.293 µs |  |
| `pricing/exotic/power` | 1.0132 µs | 1.0153 µs | 1.0181 µs |  |

### optionstratlib-pricing: `greeks`

| Benchmark | Lower | Estimate | Upper | Throughput |
|---|---:|---:|---:|---:|
| `greeks/analytic/delta` | 9.3116 µs | 9.3234 µs | 9.3400 µs |  |
| `greeks/analytic/gamma` | 11.847 µs | 11.849 µs | 11.851 µs |  |
| `greeks/analytic/theta` | 13.590 µs | 13.592 µs | 13.594 µs |  |
| `greeks/analytic/vega` | 11.924 µs | 11.929 µs | 11.934 µs |  |
| `greeks/analytic/rho` | 9.6061 µs | 9.6077 µs | 9.6096 µs |  |
| `greeks/analytic/rho_d` | 9.3789 µs | 9.3804 µs | 9.3820 µs |  |
| `greeks/analytic/alpha` | 25.558 µs | 25.583 µs | 25.611 µs |  |
| `greeks/analytic/vanna` | 11.916 µs | 11.919 µs | 11.924 µs |  |
| `greeks/analytic/vomma` | 12.535 µs | 12.917 µs | 13.396 µs |  |
| `greeks/analytic/veta` | 12.345 µs | 12.347 µs | 12.349 µs |  |
| `greeks/analytic/charm` | 12.484 µs | 12.497 µs | 12.515 µs |  |
| `greeks/analytic/color` | 12.334 µs | 12.342 µs | 12.354 µs |  |
| `greeks/analytic/greeks_snapshot` | 17.153 µs | 17.174 µs | 17.206 µs |  |
| `greeks/analytic/position_greeks_snapshot` | 17.142 µs | 17.144 µs | 17.146 µs |  |
| `greeks/variants/numerical_delta` | 36.261 µs | 36.346 µs | 36.463 µs |  |
| `greeks/variants/numerical_gamma` | 54.400 µs | 54.414 µs | 54.430 µs |  |
| `greeks/variants/numerical_vega` | 36.162 µs | 36.169 µs | 36.176 µs |  |
| `greeks/variants/numerical_theta` | 36.031 µs | 36.047 µs | 36.075 µs |  |
| `greeks/variants/numerical_rho` | 36.060 µs | 36.067 µs | 36.074 µs |  |
| `greeks/variants/delta_b76` | 16.772 µs | 16.774 µs | 16.776 µs |  |
| `greeks/variants/gamma_b76` | 20.327 µs | 20.335 µs | 20.346 µs |  |
| `greeks/variants/vega_b76` | 20.385 µs | 20.412 µs | 20.453 µs |  |
| `greeks/variants/delta_gk` | 16.692 µs | 16.694 µs | 16.696 µs |  |
| `greeks/variants/gamma_gk` | 20.116 µs | 20.171 µs | 20.248 µs |  |
| `greeks/variants/vega_gk` | 20.178 µs | 20.193 µs | 20.209 µs |  |
| `greeks/kernels/d1` | 7.5073 µs | 7.7355 µs | 7.9913 µs |  |
| `greeks/kernels/n_pdf` | 1.2304 µs | 1.2307 µs | 1.2311 µs |  |
| `greeks/kernels/big_n_cdf` | 198.26 ns | 198.78 ns | 199.54 ns |  |
| `greeks/chain/delta/10` | 92.372 µs | 92.465 µs | 92.573 µs | 108.15 Kelem/s |
| `greeks/chain/greeks_snapshot/10` | 166.58 µs | 166.60 µs | 166.61 µs | 60.025 Kelem/s |
| `greeks/chain/delta/50` | 497.38 µs | 497.85 µs | 498.39 µs | 100.43 Kelem/s |
| `greeks/chain/greeks_snapshot/50` | 933.06 µs | 935.60 µs | 939.14 µs | 53.442 Kelem/s |
| `greeks/chain/delta/200` | 2.0608 ms | 2.0633 ms | 2.0659 ms | 96.933 Kelem/s |
| `greeks/chain/greeks_snapshot/200` | 3.8830 ms | 3.9706 ms | 4.0796 ms | 50.370 Kelem/s |

### optionstratlib-pricing: `volatility`

| Benchmark | Lower | Estimate | Upper | Throughput |
|---|---:|---:|---:|---:|
| `volatility/implied/calculate_implied_volatility/atm` | 79.964 µs | 80.000 µs | 80.038 µs |  |
| `volatility/implied/implied_volatility/atm` | 7.8243 ms | 7.8502 ms | 7.8803 ms |  |
| `volatility/implied/calculate_iv/atm` | 726.14 µs | 766.42 µs | 812.14 µs |  |
| `volatility/implied/calculate_implied_volatility/otm` | 293.48 µs | 293.57 µs | 293.68 µs |  |
| `volatility/implied/implied_volatility/otm` | 27.624 ms | 27.683 ms | 27.750 ms |  |
| `volatility/implied/calculate_iv/otm` | 2.6987 ms | 2.7018 ms | 2.7052 ms |  |
| `volatility/implied/calculate_implied_volatility/deep_otm` | 249.57 µs | 249.61 µs | 249.64 µs |  |
| `volatility/implied/implied_volatility/deep_otm` | 31.426 ms | 31.470 ms | 31.518 ms |  |
| `volatility/implied/calculate_iv/deep_otm` | 3.0838 ms | 3.0944 ms | 3.1068 ms |  |
| `volatility/implied/calculate_implied_volatility_chain/50` | 15.108 ms | 15.110 ms | 15.112 ms | 3.3091 Kelem/s |
| `volatility/estimators/constant/63` | 8.3210 µs | 8.3441 µs | 8.3857 µs | 7.5502 Melem/s |
| `volatility/estimators/historical_window_21/63` | 158.26 µs | 158.33 µs | 158.39 µs | 397.91 Kelem/s |
| `volatility/estimators/ewma/63` | 91.965 µs | 92.173 µs | 92.545 µs | 683.50 Kelem/s |
| `volatility/estimators/garch/63` | 94.787 µs | 94.842 µs | 94.907 µs | 664.26 Kelem/s |
| `volatility/estimators/heston_simulation/63` | 165.55 µs | 165.88 µs | 166.34 µs | 379.79 Kelem/s |
| `volatility/estimators/constant/252` | 29.366 µs | 29.381 µs | 29.395 µs | 8.5770 Melem/s |
| `volatility/estimators/historical_window_21/252` | 852.79 µs | 854.60 µs | 857.59 µs | 294.87 Kelem/s |
| `volatility/estimators/ewma/252` | 465.84 µs | 466.13 µs | 466.43 µs | 540.63 Kelem/s |
| `volatility/estimators/garch/252` | 466.52 µs | 467.34 µs | 468.69 µs | 539.22 Kelem/s |
| `volatility/estimators/heston_simulation/252` | 664.07 µs | 664.53 µs | 664.99 µs | 379.22 Kelem/s |
| `volatility/estimators/constant/1008` | 113.39 µs | 113.47 µs | 113.55 µs | 8.8833 Melem/s |
| `volatility/estimators/historical_window_21/1008` | 3.6326 ms | 3.6338 ms | 3.6350 ms | 277.39 Kelem/s |
| `volatility/estimators/ewma/1008` | 1.9027 ms | 1.9034 ms | 1.9041 ms | 529.57 Kelem/s |
| `volatility/estimators/garch/1008` | 1.6183 ms | 1.6193 ms | 1.6203 ms | 622.50 Kelem/s |
| `volatility/estimators/heston_simulation/1008` | 2.6563 ms | 2.6595 ms | 2.6626 ms | 379.02 Kelem/s |
| `volatility/utilities/uncertain_volatility_bounds` | 34.743 µs | 34.801 µs | 34.873 µs |  |
| `volatility/utilities/adjust_volatility_day_to_year` | 1.6748 µs | 1.6757 µs | 1.6765 µs |  |

### optionstratlib-simulation: `simulation`

| Benchmark | Lower | Estimate | Upper | Throughput |
|---|---:|---:|---:|---:|
| `simulation/walk/brownian/252` | 60.118 µs | 60.251 µs | 60.402 µs | 4.1825 Melem/s |
| `simulation/walk/geometric_brownian/252` | 440.19 µs | 440.48 µs | 440.77 µs | 572.10 Kelem/s |
| `simulation/walk/log_returns/252` | 442.33 µs | 442.57 µs | 442.81 µs | 569.41 Kelem/s |
| `simulation/walk/mean_reverting/252` | 87.681 µs | 87.785 µs | 87.893 µs | 2.8707 Melem/s |
| `simulation/walk/jump_diffusion/252` | 88.335 µs | 88.432 µs | 88.548 µs | 2.8496 Melem/s |
| `simulation/walk/garch/252` | 824.17 µs | 824.58 µs | 824.98 µs | 305.61 Kelem/s |
| `simulation/walk/heston/252` | 1.6160 ms | 1.6313 ms | 1.6507 ms | 154.48 Kelem/s |
| `simulation/walk/custom/252` | 213.83 µs | 214.02 µs | 214.21 µs | 1.1774 Melem/s |
| `simulation/walk/telegraph/252` | 707.65 µs | 708.36 µs | 709.32 µs | 355.75 Kelem/s |
| `simulation/walk/historical/252` | 2.9879 ms | 2.9884 ms | 2.9889 ms | 84.325 Kelem/s |
| `simulation/walk/brownian/1008` | 344.92 µs | 345.30 µs | 345.70 µs | 2.9192 Melem/s |
| `simulation/walk/geometric_brownian/1008` | 1.8557 ms | 1.8579 ms | 1.8605 ms | 542.55 Kelem/s |
| `simulation/walk/log_returns/1008` | 1.8794 ms | 1.8802 ms | 1.8810 ms | 536.12 Kelem/s |
| `simulation/walk/mean_reverting/1008` | 428.03 µs | 431.36 µs | 435.79 µs | 2.3368 Melem/s |
| `simulation/walk/jump_diffusion/1008` | 467.71 µs | 468.03 µs | 468.39 µs | 2.1537 Melem/s |
| `simulation/walk/garch/1008` | 3.4164 ms | 3.4219 ms | 3.4302 ms | 294.57 Kelem/s |
| `simulation/walk/heston/1008` | 6.2917 ms | 6.3150 ms | 6.3389 ms | 159.62 Kelem/s |
| `simulation/walk/custom/1008` | 914.49 µs | 915.98 µs | 918.16 µs | 1.1005 Melem/s |
| `simulation/walk/telegraph/1008` | 2.9290 ms | 2.9301 ms | 2.9311 ms | 344.02 Kelem/s |
| `simulation/walk/historical/1008` | 11.995 ms | 12.021 ms | 12.055 ms | 83.852 Kelem/s |
| `simulation/simulator/new/100_paths_x_30_steps` | 5.5311 ms | 5.5384 ms | 5.5490 ms | 541.67 Kelem/s |
| `simulation/simulator/new/1000_paths_x_30_steps` | 55.335 ms | 55.373 ms | 55.464 ms | 541.78 Kelem/s |
| `simulation/simulator/new/100_paths_x_252_steps` | 46.926 ms | 46.928 ms | 46.930 ms | 536.99 Kelem/s |
| `simulation/simulator/get_last_positive_values/1000_paths` | 2.0544 µs | 2.0559 µs | 2.0571 µs | 486.41 Melem/s |
| `simulation/simulator/get_mc_option_price/1000_paths` | 158.73 µs | 158.87 µs | 159.05 µs | 6.2943 Melem/s |
| `simulation/process/generate_ou_process/252` | 105.99 µs | 106.10 µs | 106.25 µs | 2.3751 Melem/s |
| `simulation/process/expanding_window_vols/252` | 2.9808 ms | 2.9819 ms | 2.9834 ms | 84.510 Kelem/s |
| `simulation/process/generate_ou_process/1008` | 414.85 µs | 415.37 µs | 415.97 µs | 2.4268 Melem/s |
| `simulation/process/expanding_window_vols/1008` | 12.059 ms | 12.247 ms | 12.477 ms | 82.308 Kelem/s |

### optionstratlib-market: `chains`

| Benchmark | Lower | Estimate | Upper | Throughput |
|---|---:|---:|---:|---:|
| `market/chain_build/build_chain/21` | 1.6677 ms | 1.6683 ms | 1.6690 ms | 12.588 Kelem/s |
| `market/chain_build/build_chain_with_greeks/21` | 2.4847 ms | 2.4851 ms | 2.4854 ms | 8.4505 Kelem/s |
| `market/chain_build/to_build_params/21` | 2.9104 µs | 2.9142 µs | 2.9172 µs | 7.2061 Melem/s |
| `market/chain_build/build_chain/51` | 4.1890 ms | 4.1935 ms | 4.2024 ms | 12.162 Kelem/s |
| `market/chain_build/build_chain_with_greeks/51` | 6.1590 ms | 6.1595 ms | 6.1603 ms | 8.2798 Kelem/s |
| `market/chain_build/to_build_params/51` | 5.9265 µs | 5.9312 µs | 5.9350 µs | 8.5985 Melem/s |
| `market/chain_build/build_chain/101` | 8.3565 ms | 8.3572 ms | 8.3582 ms | 12.085 Kelem/s |
| `market/chain_build/build_chain_with_greeks/101` | 12.260 ms | 12.264 ms | 12.273 ms | 8.2355 Kelem/s |
| `market/chain_build/to_build_params/101` | 10.946 µs | 10.961 µs | 10.973 µs | 9.2143 Melem/s |
| `market/chain_build/build_chain/201` | 16.687 ms | 16.695 ms | 16.716 ms | 12.039 Kelem/s |
| `market/chain_build/build_chain_with_greeks/201` | 24.469 ms | 24.473 ms | 24.479 ms | 8.2132 Kelem/s |
| `market/chain_build/to_build_params/201` | 21.543 µs | 21.583 µs | 21.613 µs | 9.3127 Melem/s |
| `market/chain_lookup/atm_option_data/21` | 50.032 ns | 50.075 ns | 50.118 ns |  |
| `market/chain_lookup/get_strikes/21` | 31.824 ns | 31.869 ns | 31.915 ns |  |
| `market/chain_lookup/get_optiondata_with_strike/21` | 289.33 ns | 290.21 ns | 291.16 ns |  |
| `market/chain_lookup/get_call_price/21` | 51.248 ns | 51.347 ns | 51.436 ns |  |
| `market/chain_lookup/filter_option_data_upper/21` | 182.45 ns | 182.70 ns | 182.97 ns |  |
| `market/chain_lookup/get_atm_implied_volatility/21` | 51.295 ns | 51.342 ns | 51.392 ns |  |
| `market/chain_lookup/get_position_with_delta/21` | 507.63 ns | 508.55 ns | 509.45 ns |  |
| `market/chain_lookup/atm_option_data/101` | 256.81 ns | 257.15 ns | 257.48 ns |  |
| `market/chain_lookup/get_strikes/101` | 132.60 ns | 132.88 ns | 133.18 ns |  |
| `market/chain_lookup/get_optiondata_with_strike/101` | 1.3840 µs | 1.3883 µs | 1.3930 µs |  |
| `market/chain_lookup/get_call_price/101` | 259.21 ns | 259.52 ns | 259.87 ns |  |
| `market/chain_lookup/filter_option_data_upper/101` | 780.38 ns | 781.55 ns | 782.83 ns |  |
| `market/chain_lookup/get_atm_implied_volatility/101` | 256.94 ns | 257.22 ns | 257.49 ns |  |
| `market/chain_lookup/get_position_with_delta/101` | 2.0293 µs | 2.0330 µs | 2.0366 µs |  |
| `market/chain_iterators/double_iter_count/21` | 3.9433 µs | 3.9534 µs | 3.9659 µs |  |
| `market/chain_iterators/triple_iter_count/21` | 43.736 µs | 43.825 µs | 43.919 µs |  |
| `market/chain_iterators/quad_iter_count/21` | 302.47 µs | 303.24 µs | 304.08 µs |  |
| `market/chain_iterators/double_iter_count/51` | 21.598 µs | 21.650 µs | 21.701 µs |  |
| `market/chain_iterators/triple_iter_count/51` | 578.32 µs | 580.31 µs | 582.23 µs |  |
| `market/chain_iterators/quad_iter_count/51` | 9.8466 ms | 9.8650 ms | 9.8833 ms |  |
| `market/chain_refresh/update_greeks/21` | 692.68 µs | 703.86 µs | 721.64 µs | 29.835 Kelem/s |
| `market/chain_refresh/update_mid_prices/21` | 5.0443 µs | 5.0565 µs | 5.0682 µs | 4.1531 Melem/s |
| `market/chain_refresh/clone/21` | 1.5438 µs | 1.5466 µs | 1.5494 µs | 13.578 Melem/s |
| `market/chain_refresh/gamma_exposure/21` | 186.81 ns | 187.88 ns | 188.87 ns | 111.77 Melem/s |
| `market/chain_refresh/delta_exposure/21` | 588.80 ns | 590.64 ns | 592.55 ns | 35.554 Melem/s |
| `market/chain_refresh/vega_exposure/21` | 540.51 µs | 540.57 µs | 540.64 µs | 38.848 Kelem/s |
| `market/chain_refresh/update_greeks/101` | 3.4751 ms | 3.4829 ms | 3.4945 ms | 28.999 Kelem/s |
| `market/chain_refresh/update_mid_prices/101` | 22.570 µs | 22.613 µs | 22.655 µs | 4.4664 Melem/s |
| `market/chain_refresh/clone/101` | 7.8301 µs | 7.8395 µs | 7.8495 µs | 12.883 Melem/s |
| `market/chain_refresh/gamma_exposure/101` | 930.65 ns | 943.72 ns | 959.64 ns | 107.02 Melem/s |
| `market/chain_refresh/delta_exposure/101` | 5.8187 µs | 5.8379 µs | 5.8660 µs | 17.301 Melem/s |
| `market/chain_refresh/vega_exposure/101` | 2.7149 ms | 2.7151 ms | 2.7154 ms | 37.199 Kelem/s |
| `market/chain_serde/to_json/21` | 17.662 µs | 17.669 µs | 17.678 µs | 542.59 MiB/s |
| `market/chain_serde/from_json/21` | 18.962 µs | 18.995 µs | 19.028 µs | 504.74 MiB/s |
| `market/chain_serde/to_json/101` | 83.014 µs | 83.033 µs | 83.052 µs | 550.34 MiB/s |
| `market/chain_serde/from_json/101` | 94.451 µs | 94.618 µs | 94.784 µs | 482.96 MiB/s |
| `market/series_build/build_series/3` | 7.2936 ms | 7.3049 ms | 7.3256 ms |  |
| `market/series_build/build_series/12` | 28.773 ms | 29.307 ms | 30.251 ms |  |

### optionstratlib-market: `chains_io` (`io`)

| Benchmark | Lower | Estimate | Upper | Throughput |
|---|---:|---:|---:|---:|
| `market/chain_io/save_to_csv/21` | 30.658 µs | 30.791 µs | 30.928 µs | 682.02 Kelem/s |
| `market/chain_io/load_from_csv/21` | 22.702 µs | 22.710 µs | 22.718 µs | 924.70 Kelem/s |
| `market/chain_io/save_to_json/21` | 1.0158 ms | 1.0240 ms | 1.0334 ms | 20.508 Kelem/s |
| `market/chain_io/load_from_json/21` | 2.9545 ms | 2.9551 ms | 2.9557 ms | 7.1063 Kelem/s |
| `market/chain_io/save_to_csv/101` | 104.50 µs | 104.66 µs | 104.83 µs | 965.06 Kelem/s |
| `market/chain_io/load_from_csv/101` | 71.596 µs | 71.724 µs | 71.843 µs | 1.4082 Melem/s |
| `market/chain_io/save_to_json/101` | 4.7877 ms | 4.7980 ms | 4.8079 ms | 21.051 Kelem/s |
| `market/chain_io/load_from_json/101` | 14.483 ms | 14.756 ms | 15.096 ms | 6.8448 Kelem/s |
| `market/chain_io/load_from_json/sp500_fixture` | 3.5900 ms | 3.5907 ms | 3.5914 ms | 28.129 Kelem/s |
| `market/ohlcv_zip/read_ohlcv_from_zip/all_74061` | 25.640 ms | 25.733 ms | 25.890 ms |  |
| `market/ohlcv_zip/read_ohlcv_from_zip/one_day` | 22.191 ms | 22.253 ms | 22.344 ms |  |

### optionstratlib-market: `synthetic` (`synthetic`)

| Benchmark | Lower | Estimate | Upper | Throughput |
|---|---:|---:|---:|---:|
| `market/synthetic/generator_optionchain/10_steps_half_width_10` | 5.0213 ms | 5.2182 ms | 5.4486 ms | 1.9164 Kelem/s |
| `market/synthetic/generator_optionchain/30_steps_half_width_10` | 11.511 ms | 11.541 ms | 11.558 ms | 2.5994 Kelem/s |
| `market/synthetic/generator_optionchain/30_steps_half_width_25` | 21.303 ms | 21.397 ms | 21.575 ms | 1.4020 Kelem/s |
| `market/synthetic/generator_optionseries/10_steps_3_expirations` | 16.236 ms | 16.454 ms | 16.659 ms |  |
| `market/synthetic/generator_optionseries/30_steps_3_expirations` | 31.307 ms | 33.100 ms | 35.917 ms |  |

### optionstratlib-analytics: `analytics`

| Benchmark | Lower | Estimate | Upper | Throughput |
|---|---:|---:|---:|---:|
| `analytics/probability/single_point` | 9.5539 µs | 9.5914 µs | 9.6451 µs |  |
| `analytics/probability/single_point_with_trend` | 9.5571 µs | 9.5584 µs | 9.5598 µs |  |
| `analytics/probability/price_range` | 17.128 µs | 17.130 µs | 17.132 µs |  |
| `analytics/pnl_risk/position_calculate_pnl` | 40.631 µs | 40.695 µs | 40.805 µs |  |
| `analytics/pnl_risk/position_calculate_pnl_at_expiration` | 158.29 ns | 158.41 ns | 158.54 ns |  |
| `analytics/pnl_risk/options_calculate_pnl` | 40.665 µs | 40.755 µs | 40.904 µs |  |
| `analytics/pnl_risk/span_margin` | 312.43 µs | 312.56 µs | 312.80 µs |  |
| `analytics/pnl_risk/position_calculate_pnl_sweep/101` | 3.9925 ms | 4.0034 ms | 4.0219 ms | 25.229 Kelem/s |
| `analytics/rnd/calculate_rnd/21_strikes_50_points` | 21.451 µs | 21.453 µs | 21.457 µs |  |
| `analytics/rnd/calculate_rnd/21_strikes_200_points` | 21.448 µs | 21.456 µs | 21.470 µs |  |
| `analytics/rnd/calculate_skew/21` | 970.93 ns | 978.69 ns | 984.45 ns |  |
| `analytics/rnd/calculate_rnd/51_strikes_50_points` | 51.299 µs | 51.310 µs | 51.319 µs |  |
| `analytics/rnd/calculate_rnd/51_strikes_200_points` | 51.321 µs | 51.336 µs | 51.360 µs |  |
| `analytics/rnd/calculate_skew/51` | 2.4462 µs | 2.4688 µs | 2.4853 µs |  |
| `analytics/projections/delta_curve/21` | 208.79 µs | 208.84 µs | 208.89 µs |  |
| `analytics/projections/gamma_curve/21` | 270.34 µs | 270.39 µs | 270.46 µs |  |
| `analytics/projections/vanna_surface/21x5` | 1.3332 ms | 1.3356 ms | 1.3407 ms |  |
| `analytics/projections/theta_time_surface/21x5` | 1.5766 ms | 1.6299 ms | 1.7159 ms |  |
| `analytics/metrics/iv_curve/21` | 588.40 ns | 589.14 ns | 589.91 ns |  |
| `analytics/metrics/volatility_skew/21` | 1.8982 µs | 1.8985 µs | 1.8989 µs |  |
| `analytics/metrics/premium_weighted_pcr/21` | 1.6728 µs | 1.6769 µs | 1.6834 µs |  |
| `analytics/metrics/risk_reversal_curve/21` | 1.1338 µs | 1.1348 µs | 1.1358 µs |  |
| `analytics/metrics/dollar_gamma_curve/21` | 270.42 µs | 270.51 µs | 270.59 µs |  |
| `analytics/metrics/delta_gamma_curve/21` | 479.07 µs | 479.29 µs | 479.60 µs |  |
| `analytics/metrics/theta_curve/21` | 307.31 µs | 307.39 µs | 307.50 µs |  |
| `analytics/metrics/price_shock_curve/21` | 478.87 µs | 479.19 µs | 479.43 µs |  |
| `analytics/metrics/iv_surface/21x5` | 99.825 µs | 99.979 µs | 100.17 µs |  |
| `analytics/metrics/vanna_volga_surface/10x10` | 22.472 µs | 22.539 µs | 22.627 µs | 4.4368 Melem/s |
| `analytics/metrics/volatility_sensitivity_surface/10x10` | 2.1470 ms | 2.1475 ms | 2.1480 ms | 46.566 Kelem/s |
| `analytics/metrics/time_decay_surface/10x5` | 988.53 µs | 988.98 µs | 989.81 µs | 101.11 Kelem/s |
| `analytics/metrics/theta_surface/10x5` | 712.89 µs | 713.13 µs | 713.43 µs | 140.23 Kelem/s |
| `analytics/metrics/vanna_volga_surface/20x20` | 94.849 µs | 95.188 µs | 95.671 µs | 4.2022 Melem/s |
| `analytics/metrics/volatility_sensitivity_surface/20x20` | 7.9281 ms | 7.9307 ms | 7.9335 ms | 50.437 Kelem/s |
| `analytics/metrics/time_decay_surface/20x5` | 1.9104 ms | 1.9132 ms | 1.9195 ms | 209.07 Kelem/s |
| `analytics/metrics/theta_surface/20x5` | 1.3992 ms | 1.4449 ms | 1.5158 ms | 276.83 Kelem/s |

### optionstratlib-strategies: `strategies`

| Benchmark | Lower | Estimate | Upper | Throughput |
|---|---:|---:|---:|---:|
| `strategies/construction/long_call/1_leg` | 748.65 ns | 749.26 ns | 749.94 ns |  |
| `strategies/construction/bull_call_spread/2_legs` | 1.3134 µs | 1.3157 µs | 1.3185 µs |  |
| `strategies/construction/short_strangle/2_legs` | 1.3998 µs | 1.4014 µs | 1.4036 µs |  |
| `strategies/construction/long_butterfly/3_legs` | 2.2479 µs | 2.2491 µs | 2.2504 µs |  |
| `strategies/construction/iron_condor/4_legs` | 1.7109 µs | 1.7184 µs | 1.7285 µs |  |
| `strategies/construction/iron_butterfly/4_legs` | 1.6799 µs | 1.6849 µs | 1.6896 µs |  |
| `strategies/evaluation/long_call/get_break_even_points` | 1.4259 ns | 1.4451 ns | 1.4652 ns |  |
| `strategies/evaluation/long_call/update_break_even_points` | 181.81 ns | 182.48 ns | 183.14 ns |  |
| `strategies/evaluation/long_call/calculate_profit_at` | 118.48 ns | 119.22 ns | 120.24 ns |  |
| `strategies/evaluation/long_call/get_max_profit` | 4.5233 ns | 4.5610 ns | 4.6059 ns |  |
| `strategies/evaluation/long_call/get_max_loss` | 68.447 ns | 68.518 ns | 68.600 ns |  |
| `strategies/evaluation/long_call/get_total_cost` | 73.187 ns | 73.359 ns | 73.580 ns |  |
| `strategies/evaluation/long_call/get_profit_area` | 42.626 ns | 42.646 ns | 42.668 ns |  |
| `strategies/evaluation/long_call/greeks` | 13.712 µs | 13.730 µs | 13.763 µs |  |
| `strategies/evaluation/long_call/calculate_profit_sweep_201` | 27.547 µs | 27.568 µs | 27.591 µs |  |
| `strategies/evaluation/bull_call_spread/get_break_even_points` | 1.3681 ns | 1.3919 ns | 1.4167 ns |  |
| `strategies/evaluation/bull_call_spread/update_break_even_points` | 274.39 ns | 276.73 ns | 278.82 ns |  |
| `strategies/evaluation/bull_call_spread/calculate_profit_at` | 271.78 ns | 271.89 ns | 272.02 ns |  |
| `strategies/evaluation/bull_call_spread/get_max_profit` | 278.96 ns | 280.24 ns | 281.64 ns |  |
| `strategies/evaluation/bull_call_spread/get_max_loss` | 238.56 ns | 239.87 ns | 241.31 ns |  |
| `strategies/evaluation/bull_call_spread/get_total_cost` | 105.88 ns | 106.02 ns | 106.18 ns |  |
| `strategies/evaluation/bull_call_spread/get_profit_area` | 421.60 ns | 422.58 ns | 423.63 ns |  |
| `strategies/evaluation/bull_call_spread/get_profit_ratio` | 615.63 ns | 615.86 ns | 616.06 ns |  |
| `strategies/evaluation/bull_call_spread/greeks` | 29.957 µs | 29.961 µs | 29.965 µs |  |
| `strategies/evaluation/bull_call_spread/calculate_profit_sweep_201` | 54.114 µs | 54.136 µs | 54.156 µs |  |
| `strategies/evaluation/short_strangle/get_break_even_points` | 1.3689 ns | 1.3964 ns | 1.4265 ns |  |
| `strategies/evaluation/short_strangle/update_break_even_points` | 371.77 ns | 373.00 ns | 374.03 ns |  |
| `strategies/evaluation/short_strangle/calculate_profit_at` | 221.51 ns | 221.67 ns | 221.83 ns |  |
| `strategies/evaluation/short_strangle/get_max_profit` | 292.98 ns | 293.98 ns | 295.24 ns |  |
| `strategies/evaluation/short_strangle/get_max_loss` | 4.5233 ns | 4.5608 ns | 4.6054 ns |  |
| `strategies/evaluation/short_strangle/get_total_cost` | 82.421 ns | 82.644 ns | 82.956 ns |  |
| `strategies/evaluation/short_strangle/get_profit_area` | 572.12 ns | 572.49 ns | 572.83 ns |  |
| `strategies/evaluation/short_strangle/get_profit_ratio` | 425.22 ns | 425.76 ns | 426.45 ns |  |
| `strategies/evaluation/short_strangle/greeks` | 32.187 µs | 32.192 µs | 32.197 µs |  |
| `strategies/evaluation/short_strangle/calculate_profit_sweep_201` | 50.411 µs | 50.448 µs | 50.493 µs |  |
| `strategies/evaluation/long_butterfly/get_break_even_points` | 1.3514 ns | 1.3779 ns | 1.4056 ns |  |
| `strategies/evaluation/long_butterfly/update_break_even_points` | 1.0101 µs | 1.0124 µs | 1.0146 µs |  |
| `strategies/evaluation/long_butterfly/calculate_profit_at` | 423.69 ns | 424.37 ns | 425.14 ns |  |
| `strategies/evaluation/long_butterfly/get_max_profit` | 390.27 ns | 390.56 ns | 390.87 ns |  |
| `strategies/evaluation/long_butterfly/get_max_loss` | 780.00 ns | 780.97 ns | 782.19 ns |  |
| `strategies/evaluation/long_butterfly/get_total_cost` | 170.24 ns | 170.49 ns | 170.79 ns |  |
| `strategies/evaluation/long_butterfly/get_profit_area` | 523.08 ns | 524.56 ns | 527.03 ns |  |
| `strategies/evaluation/long_butterfly/get_profit_ratio` | 1.2816 µs | 1.2828 µs | 1.2840 µs |  |
| `strategies/evaluation/long_butterfly/greeks` | 46.990 µs | 47.091 µs | 47.287 µs |  |
| `strategies/evaluation/long_butterfly/calculate_profit_sweep_201` | 81.343 µs | 81.410 µs | 81.475 µs |  |
| `strategies/evaluation/iron_condor/get_break_even_points` | 1.3565 ns | 1.3819 ns | 1.4083 ns |  |
| `strategies/evaluation/iron_condor/update_break_even_points` | 560.34 ns | 563.95 ns | 567.64 ns |  |
| `strategies/evaluation/iron_condor/calculate_profit_at` | 443.70 ns | 444.18 ns | 444.64 ns |  |
| `strategies/evaluation/iron_condor/get_max_profit` | 1.3292 µs | 1.3304 µs | 1.3318 µs |  |
| `strategies/evaluation/iron_condor/get_max_loss` | 983.96 ns | 984.58 ns | 985.25 ns |  |
| `strategies/evaluation/iron_condor/get_total_cost` | 205.90 ns | 206.31 ns | 206.84 ns |  |
| `strategies/evaluation/iron_condor/get_profit_area` | 1.5367 µs | 1.5739 µs | 1.6205 µs |  |
| `strategies/evaluation/iron_condor/get_profit_ratio` | 2.4259 µs | 2.4302 µs | 2.4372 µs |  |
| `strategies/evaluation/iron_condor/greeks` | 64.179 µs | 64.193 µs | 64.207 µs |  |
| `strategies/evaluation/iron_condor/calculate_profit_sweep_201` | 101.07 µs | 101.39 µs | 101.87 µs |  |
| `strategies/evaluation/iron_butterfly/get_break_even_points` | 1.3604 ns | 1.3852 ns | 1.4116 ns |  |
| `strategies/evaluation/iron_butterfly/update_break_even_points` | 561.28 ns | 564.03 ns | 566.48 ns |  |
| `strategies/evaluation/iron_butterfly/calculate_profit_at` | 484.94 ns | 485.63 ns | 486.40 ns |  |
| `strategies/evaluation/iron_butterfly/get_max_profit` | 1.3331 µs | 1.3342 µs | 1.3354 µs |  |
| `strategies/evaluation/iron_butterfly/get_max_loss` | 983.19 ns | 985.31 ns | 988.30 ns |  |
| `strategies/evaluation/iron_butterfly/get_total_cost` | 208.44 ns | 215.35 ns | 223.52 ns |  |
| `strategies/evaluation/iron_butterfly/get_profit_area` | 1.5249 µs | 1.5280 µs | 1.5331 µs |  |
| `strategies/evaluation/iron_butterfly/get_profit_ratio` | 2.4226 µs | 2.4236 µs | 2.4246 µs |  |
| `strategies/evaluation/iron_butterfly/greeks` | 63.729 µs | 63.740 µs | 63.749 µs |  |
| `strategies/evaluation/iron_butterfly/calculate_profit_sweep_201` | 103.67 µs | 103.74 µs | 103.79 µs |  |
| `strategies/analysis/iron_condor/probability_of_profit` | 35.680 µs | 35.684 µs | 35.687 µs |  |
| `strategies/analysis/iron_condor/expected_value` | 114.03 µs | 114.19 µs | 114.42 µs |  |
| `strategies/analysis/iron_condor/analyze_probabilities` | 152.86 µs | 153.03 µs | 153.44 µs |  |
| `strategies/analysis/short_strangle/delta_neutrality` | 56.350 µs | 56.367 µs | 56.388 µs |  |
| `strategies/analysis/short_strangle/delta_adjustments` | 56.908 µs | 56.951 µs | 56.995 µs |  |
| `strategies/optimiser/bull_call_spread_best_ratio/sp500_45` | 2.3575 ms | 2.3579 ms | 2.3585 ms |  |
| `strategies/optimiser/bull_call_spread_best_area/sp500_45` | 2.1889 ms | 2.1925 ms | 2.1963 ms |  |
| `strategies/optimiser/short_strangle_best_area/sp500_45` | 3.2489 ms | 3.2517 ms | 3.2588 ms |  |
| `strategies/optimiser/long_butterfly_best_ratio/sp500_45` | 99.006 ms | 99.017 ms | 99.024 ms |  |
| `strategies/optimiser/iron_condor_best_ratio/sp500_45` | 934.47 ms | 934.76 ms | 935.00 ms |  |
| `strategies/optimiser/bull_call_spread_best_ratio/synthetic_11` | 148.82 µs | 149.42 µs | 150.19 µs | 73.620 Kelem/s |
| `strategies/optimiser/iron_condor_best_ratio/synthetic_11` | 2.3813 ms | 2.4936 ms | 2.6605 ms | 4.4113 Kelem/s |
| `strategies/optimiser/bull_call_spread_best_ratio/synthetic_21` | 563.50 µs | 564.45 µs | 564.90 µs | 37.204 Kelem/s |
| `strategies/optimiser/iron_condor_best_ratio/synthetic_21` | 41.643 ms | 41.661 ms | 41.671 ms |  |
| `strategies/optimiser/bull_call_spread_best_ratio/synthetic_41` | 2.2444 ms | 2.2464 ms | 2.2493 ms | 18.252 Kelem/s |
| `strategies/optimiser/iron_condor_best_ratio/synthetic_41` | 681.39 ms | 681.80 ms | 682.19 ms |  |

### optionstratlib-backtest: `backtest`

| Benchmark | Lower | Estimate | Upper | Throughput |
|---|---:|---:|---:|---:|
| `backtest/run/long_call_expiration/100_paths` | 60.868 ms | 60.902 ms | 61.002 ms | 49.260 Kelem/s |
| `backtest/run/short_put_profit_or_loss/100_paths` | 2.1529 ms | 2.1531 ms | 2.1533 ms | 1.3933 Melem/s |
| `backtest/run/long_call_expiration/1000_paths` | 613.46 ms | 613.52 ms | 613.57 ms | 48.898 Kelem/s |
| `backtest/run/short_put_profit_or_loss/1000_paths` | 21.499 ms | 21.548 ms | 21.633 ms | 1.3922 Melem/s |
| `backtest/statistics/from_results/100` | 13.461 µs | 13.486 µs | 13.513 µs | 7.4152 Melem/s |
| `backtest/statistics/from_results/1000` | 166.37 µs | 166.77 µs | 167.26 µs | 5.9961 Melem/s |

### optionstratlib-visualization: `visualization`

| Benchmark | Lower | Estimate | Upper | Throughput |
|---|---:|---:|---:|---:|
| `visualization/geometry/curve_graph_data/128` | 1.0894 µs | 1.0911 µs | 1.0928 µs | 117.31 Melem/s |
| `visualization/geometry/curve_graph_data/2048` | 17.577 µs | 17.599 µs | 17.619 µs | 116.37 Melem/s |
| `visualization/geometry/curves_graph_data/5x128` | 6.2825 µs | 6.2939 µs | 6.3055 µs | 101.69 Melem/s |
| `visualization/geometry/surface_graph_data/16x16` | 1.9167 µs | 1.9206 µs | 1.9244 µs | 133.29 Melem/s |
| `visualization/geometry/surface_graph_data/64x64` | 29.858 µs | 30.005 µs | 30.219 µs | 136.51 Melem/s |
| `visualization/payoff/options_graph_data` | 13.373 µs | 13.376 µs | 13.379 µs |  |
| `visualization/payoff/position_graph_data` | 13.370 µs | 13.387 µs | 13.423 µs |  |
| `visualization/payoff/long_call_graph_data` | 3.8690 µs | 3.8704 µs | 3.8722 µs |  |
| `visualization/payoff/bull_call_spread_graph_data` | 216.67 µs | 216.72 µs | 216.76 µs |  |
| `visualization/payoff/iron_condor_graph_data` | 601.41 µs | 601.76 µs | 602.16 µs |  |
| `visualization/simulation/simulator_graph_data/10_paths_x_252` | 2.7554 µs | 2.7604 µs | 2.7662 µs | 912.93 Melem/s |
| `visualization/simulation/simulator_graph_data/100_paths_x_252` | 28.458 µs | 28.501 µs | 28.552 µs | 884.19 Melem/s |
| `visualization/simulation/random_walk_graph_data/1008` | 215.28 ns | 215.68 ns | 216.15 ns | 4.6736 Gelem/s |
| `visualization/terminal/chain_render_table/21` | 86.163 µs | 86.286 µs | 86.411 µs |  |
| `visualization/terminal/simulation_render_summary/100` | 22.523 µs | 22.566 µs | 22.610 µs |  |
| `visualization/terminal/simulation_render_individual_results/100` | 227.57 µs | 228.02 µs | 228.46 µs |  |

### optionstratlib (facade): `benches` (`benches/mod.rs`)

| Benchmark | Lower | Estimate | Upper | Throughput |
|---|---:|---:|---:|---:|
| `Chain Generators/build_chain 11 strikes` | 833.13 µs | 834.62 µs | 836.61 µs |  |
| `Chain Generators/build_chain 11 strikes + greeks` | 1.2243 ms | 1.2245 ms | 1.2247 ms |  |
| `Chain Generators/build_chain 21 strikes` | 1.6771 ms | 1.6774 ms | 1.6777 ms |  |
| `Chain Generators/build_chain 21 strikes + greeks` | 2.4749 ms | 2.4751 ms | 2.4753 ms |  |
| `Chain Generators/build_chain 31 strikes` | 2.5279 ms | 2.5282 ms | 2.5284 ms |  |
| `Chain Generators/build_chain 31 strikes + greeks` | 3.7170 ms | 3.7189 ms | 3.7217 ms |  |
| `Chain Generators/generator_optionchain 10 steps` | 3.8264 ms | 3.8650 ms | 3.9046 ms |  |
| `Chain Generators/generator_optionchain 25 steps` | 7.8303 ms | 8.2792 ms | 8.8146 ms |  |
| `Chain Generators/generator_positive 1000 steps` | 1.8602 ms | 1.8622 ms | 1.8637 ms |  |
| `OptionData Operations/create minimal option data` | 277.57 ns | 278.53 ns | 279.68 ns |  |
| `OptionData Operations/create full option data` | 435.45 ns | 436.10 ns | 436.82 ns |  |
| `OptionData Operations/validate option data` | 6.9902 ns | 7.0177 ns | 7.0479 ns |  |
| `OptionData Operations/calculate standard prices` | 11.098 µs | 11.102 µs | 11.105 µs |  |
| `OptionData Operations/complete option processing` | 11.140 µs | 11.172 µs | 11.234 µs |  |
| `OptionData Operations/process 50 option data items` | 558.44 µs | 558.59 µs | 558.74 µs |  |
| `Creation Operations/new from f64` | 34.707 ns | 35.553 ns | 36.588 ns |  |
| `Creation Operations/new from decimal` | 6.2136 ns | 6.2324 ns | 6.2547 ns |  |
| `Creation Operations/pos! macro` | 30.581 ns | 30.646 ns | 30.727 ns |  |
| `Arithmetic Operations/addition` | 10.668 ns | 10.723 ns | 10.774 ns |  |
| `Arithmetic Operations/subtraction` | 12.502 ns | 12.546 ns | 12.595 ns |  |
| `Arithmetic Operations/multiplication` | 10.743 ns | 10.772 ns | 10.800 ns |  |
| `Arithmetic Operations/division` | 21.198 ns | 21.225 ns | 21.256 ns |  |
| `Arithmetic Operations/decimal operations` | 57.334 ns | 57.398 ns | 57.459 ns |  |
| `Conversion Operations/to_f64` | 14.973 ns | 15.466 ns | 16.061 ns |  |
| `Conversion Operations/to_dec` | 343.19 ps | 366.17 ps | 386.38 ps |  |
| `Conversion Operations/to_i64` | 8.2692 ns | 8.2758 ns | 8.2835 ns |  |
| `Mathematical Operations/powi` | 23.087 ns | 23.130 ns | 23.169 ns |  |
| `Mathematical Operations/sqrt` | 515.54 ns | 516.08 ns | 516.60 ns |  |
| `Mathematical Operations/ln` | 2.1005 µs | 2.1018 µs | 2.1036 µs |  |
| `Mathematical Operations/exp` | 1.8318 µs | 1.8322 µs | 1.8326 µs |  |
| `Mathematical Operations/round` | 12.648 ns | 12.656 ns | 12.665 ns |  |
| `Mathematical Operations/floor` | 8.7843 ns | 8.7946 ns | 8.8058 ns |  |
| `Mathematical Operations/round_to` | 4.8650 ns | 4.8709 ns | 4.8762 ns |  |
| `Comparison Operations/max` | 4.1342 ns | 4.1373 ns | 4.1404 ns |  |
| `Comparison Operations/min` | 3.9133 ns | 3.9180 ns | 3.9231 ns |  |
| `Comparison Operations/partial_eq` | 2.9585 ns | 2.9633 ns | 2.9679 ns |  |
| `Comparison Operations/partial_ord` | 2.9556 ns | 2.9602 ns | 2.9648 ns |  |
| `Pricing Methods/black_scholes` | 5.4024 µs | 5.4047 µs | 5.4069 µs |  |
| `Pricing Methods/binomial_50_steps` | 373.31 µs | 373.95 µs | 374.85 µs |  |
| `Pricing Methods/telegraph_50_steps` | 8.8877 ms | 8.8915 ms | 8.8952 ms |  |
| `Greeks Calculations/delta` | 3.0698 µs | 3.1352 µs | 3.2188 µs |  |
| `Greeks Calculations/gamma` | 4.3780 µs | 4.3796 µs | 4.3812 µs |  |
| `Greeks Calculations/theta` | 6.1309 µs | 6.1330 µs | 6.1351 µs |  |
| `Greeks Calculations/vega` | 4.4540 µs | 4.4553 µs | 4.4568 µs |  |
| `Greeks Calculations/rho` | 3.3380 µs | 3.3396 µs | 3.3421 µs |  |
| `Greeks Calculations/vanna` | 4.4615 µs | 4.4634 µs | 4.4655 µs |  |
| `Greeks Calculations/vomma` | 4.6899 µs | 4.6906 µs | 4.6914 µs |  |
| `Greeks Calculations/veta` | 4.8305 µs | 4.8394 µs | 4.8529 µs |  |
| `Greeks Calculations/charm` | 4.9811 µs | 4.9826 µs | 4.9841 µs |  |
| `Greeks Calculations/color` | 4.8450 µs | 4.8594 µs | 4.8795 µs |  |
| `Greeks Calculations/all_greeks` | 9.6193 µs | 9.6209 µs | 9.6224 µs |  |
| `Valuations/payoff` | 46.064 ns | 46.098 ns | 46.139 ns |  |
| `Valuations/intrinsic_value` | 117.55 ns | 117.63 ns | 117.73 ns |  |
| `Valuations/time_value` | 5.4669 µs | 5.4687 µs | 5.4704 µs |  |
| `Valuations/pnl_calculation` | 25.642 µs | 25.646 µs | 25.651 µs |  |
| `Binary Tree Operations/binomial_tree_10_steps` | 49.439 µs | 49.465 µs | 49.506 µs |  |
| `Binary Tree Operations/binomial_tree_50_steps` | 1.6254 ms | 1.6265 ms | 1.6275 ms |  |
| `Binary Tree Operations/binomial_tree_100_steps` | 7.5989 ms | 7.6033 ms | 7.6111 ms |  |
| `Binary Tree Operations/binomial_tree_200_steps` | 35.246 ms | 35.628 ms | 36.121 ms |  |
| `Maturity Impact on Pricing/black_scholes_1_days` | 5.7518 µs | 5.7541 µs | 5.7564 µs |  |
| `Maturity Impact on Pricing/black_scholes_7_days` | 5.3730 µs | 5.3751 µs | 5.3773 µs |  |
| `Maturity Impact on Pricing/black_scholes_30_days` | 5.4002 µs | 5.4050 µs | 5.4122 µs |  |
| `Maturity Impact on Pricing/black_scholes_90_days` | 5.3337 µs | 5.3354 µs | 5.3372 µs |  |
| `Maturity Impact on Pricing/black_scholes_365_days` | 4.6760 µs | 4.6835 µs | 4.6954 µs |  |
| `Position Costs and Fees/total_cost` | 50.221 ns | 50.304 ns | 50.394 ns |  |
| `Position Costs and Fees/premium_received` | 5.8030 ns | 5.8305 ns | 5.8624 ns |  |
| `Position Costs and Fees/net_premium_received` | 5.8003 ns | 5.8227 ns | 5.8472 ns |  |
| `Position Costs and Fees/fees` | 28.089 ns | 28.159 ns | 28.234 ns |  |
| `Position Costs and Fees/net_cost` | 55.914 ns | 56.023 ns | 56.127 ns |  |
| `Position Profit Calculations/break_even` | 97.555 ns | 97.747 ns | 97.964 ns |  |
| `Position Profit Calculations/pnl_at_expiration` | 155.30 ns | 155.45 ns | 155.62 ns |  |
| `Position Profit Calculations/unrealized_pnl` | 37.913 ns | 38.038 ns | 38.189 ns |  |
| `Position Profit Calculations/calculate_pnl` | 25.582 µs | 25.587 µs | 25.591 µs |  |
| `Time Calculations/days_held` | 53.171 ns | 53.367 ns | 53.649 ns |  |
| `Time Calculations/days_to_expiration` | 5.8024 ns | 5.8256 ns | 5.8512 ns |  |
| `Validation Operations/validate` | 16.718 ns | 16.774 ns | 16.840 ns |  |
| `Validation Operations/is_long` | 216.04 ps | 216.19 ps | 216.35 ps |  |
| `Validation Operations/is_short` | 213.66 ps | 214.27 ps | 215.34 ps |  |
| `Strategies/long_call_max_profit` | 4.8631 ns | 4.9023 ns | 4.9404 ns |  |
| `Strategies/long_call_max_loss` | 67.891 ns | 67.965 ns | 68.048 ns |  |
| `Strategies/bull_call_spread_max_profit` | 282.21 ns | 282.33 ns | 282.46 ns |  |
| `Strategies/bull_call_spread_max_loss` | 240.63 ns | 240.77 ns | 240.92 ns |  |
| `Strategies/iron_condor_max_profit` | 891.86 ns | 893.65 ns | 896.65 ns |  |
| `Strategies/iron_condor_max_loss` | 988.15 ns | 988.64 ns | 989.15 ns |  |
| `Strategies/iron_butterfly_max_profit` | 892.69 ns | 893.59 ns | 894.77 ns |  |
| `Strategies/iron_butterfly_max_loss` | 985.75 ns | 986.25 ns | 986.77 ns |  |
| `curve_merge_multiply/size_32/2` | 125.52 µs | 126.78 µs | 128.25 µs | 15.776 Kelem/s |
| `curve_merge_multiply/size_32/3` | 163.92 µs | 164.67 µs | 165.44 µs | 18.218 Kelem/s |
| `curve_merge_multiply/size_32/5` | 229.45 µs | 230.66 µs | 232.21 µs | 21.677 Kelem/s |
| `curve_merge_multiply/size_32/10` | 381.29 µs | 382.33 µs | 383.38 µs | 26.155 Kelem/s |
| `curve_merge_multiply/size_128/2` | 252.34 µs | 253.30 µs | 254.44 µs | 7.8959 Kelem/s |
| `curve_merge_multiply/size_128/3` | 336.42 µs | 359.28 µs | 387.84 µs | 8.3499 Kelem/s |
| `curve_merge_multiply/size_128/5` | 474.47 µs | 475.51 µs | 476.56 µs | 10.515 Kelem/s |
| `curve_merge_multiply/size_128/10` | 853.45 µs | 855.78 µs | 858.63 µs | 11.685 Kelem/s |
| `curve_merge_multiply/size_512/2` | 640.84 µs | 642.00 µs | 643.12 µs | 3.1153 Kelem/s |
| `curve_merge_multiply/size_512/3` | 899.36 µs | 905.09 µs | 912.86 µs | 3.3146 Kelem/s |
| `curve_merge_multiply/size_512/5` | 1.4170 ms | 1.4196 ms | 1.4222 ms | 3.5221 Kelem/s |
| `curve_merge_multiply/size_512/10` | 2.7305 ms | 2.7405 ms | 2.7520 ms | 3.6489 Kelem/s |
| `surface_merge_multiply/side_8/2` | 22.016 ms | 24.216 ms | 27.081 ms |  |
| `surface_merge_multiply/side_8/3` | 31.893 ms | 32.007 ms | 32.240 ms |  |
| `surface_merge_multiply/side_8/5` | 51.023 ms | 51.076 ms | 51.137 ms |  |
| `surface_merge_multiply/side_8/10` | 97.425 ms | 97.527 ms | 97.813 ms |  |
| `surface_merge_multiply/side_16/2` | 62.620 ms | 62.851 ms | 63.114 ms |  |
| `surface_merge_multiply/side_16/3` | 92.587 ms | 92.660 ms | 92.724 ms |  |
| `surface_merge_multiply/side_16/5` | 151.82 ms | 152.55 ms | 153.65 ms |  |
| `surface_merge_multiply/side_16/10` | 296.73 ms | 300.50 ms | 306.27 ms |  |
| `decimal_product_fold/parallel_reduce/2` | 12.559 µs | 12.657 µs | 12.762 µs |  |
| `decimal_product_fold/sequential_fold/2` | 68.758 ns | 68.777 ns | 68.796 ns |  |
| `decimal_product_fold/parallel_reduce/3` | 14.490 µs | 14.666 µs | 14.859 µs |  |
| `decimal_product_fold/sequential_fold/3` | 155.33 ns | 155.35 ns | 155.37 ns |  |
| `decimal_product_fold/parallel_reduce/5` | 18.610 µs | 18.709 µs | 18.800 µs |  |
| `decimal_product_fold/sequential_fold/5` | 331.39 ns | 331.44 ns | 331.49 ns |  |
| `decimal_product_fold/parallel_reduce/10` | 19.703 µs | 19.816 µs | 19.926 µs |  |
| `decimal_product_fold/sequential_fold/10` | 772.52 ns | 772.68 ns | 772.86 ns |  |

## Criterion output

The run's standard output with Criterion's per-benchmark progress lines
(`Benchmarking ...: Warming up`, `Collecting`, `Analyzing`) removed; every
result, throughput and outlier line is kept as printed.

<details>
<summary>Full output (2 794 lines)</summary>

```text
    Finished `bench` profile [optimized] target(s) in 0.08s
     Running benches/mod.rs (target/release/deps/benches-334b2944a0713840)
Chain Generators/build_chain 11 strikes
                        time:   [833.13 µs 834.62 µs 836.61 µs]
Found 3 outliers among 20 measurements (15.00%)
  3 (15.00%) high severe
Chain Generators/build_chain 11 strikes + greeks
                        time:   [1.2243 ms 1.2245 ms 1.2247 ms]
Chain Generators/build_chain 21 strikes
                        time:   [1.6771 ms 1.6774 ms 1.6777 ms]
Chain Generators/build_chain 21 strikes + greeks
                        time:   [2.4749 ms 2.4751 ms 2.4753 ms]
Found 2 outliers among 20 measurements (10.00%)
  1 (5.00%) low mild
  1 (5.00%) high mild
Chain Generators/build_chain 31 strikes
                        time:   [2.5279 ms 2.5282 ms 2.5284 ms]
Found 1 outliers among 20 measurements (5.00%)
  1 (5.00%) high mild
Chain Generators/build_chain 31 strikes + greeks
                        time:   [3.7170 ms 3.7189 ms 3.7217 ms]
Found 2 outliers among 20 measurements (10.00%)
  2 (10.00%) high severe
Chain Generators/generator_optionchain 10 steps
                        time:   [3.8264 ms 3.8650 ms 3.9046 ms]
Chain Generators/generator_optionchain 25 steps
                        time:   [7.8303 ms 8.2792 ms 8.8146 ms]
Found 4 outliers among 20 measurements (20.00%)
  1 (5.00%) low mild
  3 (15.00%) high severe
Chain Generators/generator_positive 1000 steps
                        time:   [1.8602 ms 1.8622 ms 1.8637 ms]
Found 3 outliers among 20 measurements (15.00%)
  3 (15.00%) high mild

OptionData Operations/create minimal option data
                        time:   [277.57 ns 278.53 ns 279.68 ns]
Found 18 outliers among 100 measurements (18.00%)
  18 (18.00%) high severe
OptionData Operations/create full option data
                        time:   [435.45 ns 436.10 ns 436.82 ns]
Found 16 outliers among 100 measurements (16.00%)
  15 (15.00%) high mild
  1 (1.00%) high severe
OptionData Operations/validate option data
                        time:   [6.9902 ns 7.0177 ns 7.0479 ns]
Found 21 outliers among 100 measurements (21.00%)
  3 (3.00%) low mild
  5 (5.00%) high mild
  13 (13.00%) high severe
OptionData Operations/calculate standard prices
                        time:   [11.098 µs 11.102 µs 11.105 µs]
Found 3 outliers among 100 measurements (3.00%)
  1 (1.00%) low mild
  2 (2.00%) high mild
OptionData Operations/complete option processing
                        time:   [11.140 µs 11.172 µs 11.234 µs]
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) high mild
  1 (1.00%) high severe
OptionData Operations/process 50 option data items
                        time:   [558.44 µs 558.59 µs 558.74 µs]
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) high mild
  1 (1.00%) high severe

Creation Operations/new from f64
                        time:   [34.707 ns 35.553 ns 36.588 ns]
Found 10 outliers among 100 measurements (10.00%)
  2 (2.00%) high mild
  8 (8.00%) high severe
Creation Operations/new from decimal
                        time:   [6.2136 ns 6.2324 ns 6.2547 ns]
Found 20 outliers among 100 measurements (20.00%)
  1 (1.00%) high mild
  19 (19.00%) high severe
Creation Operations/pos! macro
                        time:   [30.581 ns 30.646 ns 30.727 ns]
Found 3 outliers among 100 measurements (3.00%)
  1 (1.00%) high mild
  2 (2.00%) high severe

Arithmetic Operations/addition
                        time:   [10.668 ns 10.723 ns 10.774 ns]
Arithmetic Operations/subtraction
                        time:   [12.502 ns 12.546 ns 12.595 ns]
Arithmetic Operations/multiplication
                        time:   [10.743 ns 10.772 ns 10.800 ns]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
Arithmetic Operations/division
                        time:   [21.198 ns 21.225 ns 21.256 ns]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
Arithmetic Operations/decimal operations
                        time:   [57.334 ns 57.398 ns 57.459 ns]
Found 9 outliers among 100 measurements (9.00%)
  5 (5.00%) low mild
  3 (3.00%) high mild
  1 (1.00%) high severe

Conversion Operations/to_f64
                        time:   [14.973 ns 15.466 ns 16.061 ns]
Found 13 outliers among 100 measurements (13.00%)
  2 (2.00%) low mild
  2 (2.00%) high mild
  9 (9.00%) high severe
Conversion Operations/to_dec
                        time:   [343.19 ps 366.17 ps 386.38 ps]
Conversion Operations/to_i64
                        time:   [8.2692 ns 8.2758 ns 8.2835 ns]
Found 5 outliers among 100 measurements (5.00%)
  3 (3.00%) high mild
  2 (2.00%) high severe

Mathematical Operations/powi
                        time:   [23.087 ns 23.130 ns 23.169 ns]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
Mathematical Operations/sqrt
                        time:   [515.54 ns 516.08 ns 516.60 ns]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
Mathematical Operations/ln
                        time:   [2.1005 µs 2.1018 µs 2.1036 µs]
Found 3 outliers among 100 measurements (3.00%)
  3 (3.00%) high severe
Mathematical Operations/exp
                        time:   [1.8318 µs 1.8322 µs 1.8326 µs]
Found 4 outliers among 100 measurements (4.00%)
  1 (1.00%) high mild
  3 (3.00%) high severe
Mathematical Operations/round
                        time:   [12.648 ns 12.656 ns 12.665 ns]
Found 9 outliers among 100 measurements (9.00%)
  1 (1.00%) high mild
  8 (8.00%) high severe
Mathematical Operations/floor
                        time:   [8.7843 ns 8.7946 ns 8.8058 ns]
Mathematical Operations/round_to
                        time:   [4.8650 ns 4.8709 ns 4.8762 ns]
Found 5 outliers among 100 measurements (5.00%)
  2 (2.00%) low mild
  1 (1.00%) high mild
  2 (2.00%) high severe

Comparison Operations/max
                        time:   [4.1342 ns 4.1373 ns 4.1404 ns]
Found 8 outliers among 100 measurements (8.00%)
  2 (2.00%) low mild
  4 (4.00%) high mild
  2 (2.00%) high severe
Comparison Operations/min
                        time:   [3.9133 ns 3.9180 ns 3.9231 ns]
Found 14 outliers among 100 measurements (14.00%)
  2 (2.00%) low severe
  4 (4.00%) low mild
  2 (2.00%) high mild
  6 (6.00%) high severe
Comparison Operations/partial_eq
                        time:   [2.9585 ns 2.9633 ns 2.9679 ns]
Found 8 outliers among 100 measurements (8.00%)
  1 (1.00%) low severe
  2 (2.00%) low mild
  4 (4.00%) high mild
  1 (1.00%) high severe
Comparison Operations/partial_ord
                        time:   [2.9556 ns 2.9602 ns 2.9648 ns]
Found 5 outliers among 100 measurements (5.00%)
  1 (1.00%) low mild
  1 (1.00%) high mild
  3 (3.00%) high severe

Pricing Methods/black_scholes
                        time:   [5.4024 µs 5.4047 µs 5.4069 µs]
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) low severe
  1 (1.00%) low mild
Pricing Methods/binomial_50_steps
                        time:   [373.31 µs 373.95 µs 374.85 µs]
Found 5 outliers among 100 measurements (5.00%)
  1 (1.00%) high mild
  4 (4.00%) high severe
Pricing Methods/telegraph_50_steps
                        time:   [8.8877 ms 8.8915 ms 8.8952 ms]

Greeks Calculations/delta
                        time:   [3.0698 µs 3.1352 µs 3.2188 µs]
Found 10 outliers among 100 measurements (10.00%)
  1 (1.00%) high mild
  9 (9.00%) high severe
Greeks Calculations/gamma
                        time:   [4.3780 µs 4.3796 µs 4.3812 µs]
Found 4 outliers among 100 measurements (4.00%)
  3 (3.00%) high mild
  1 (1.00%) high severe
Greeks Calculations/theta
                        time:   [6.1309 µs 6.1330 µs 6.1351 µs]
Greeks Calculations/vega
                        time:   [4.4540 µs 4.4553 µs 4.4568 µs]
Found 6 outliers among 100 measurements (6.00%)
  1 (1.00%) low mild
  4 (4.00%) high mild
  1 (1.00%) high severe
Greeks Calculations/rho time:   [3.3380 µs 3.3396 µs 3.3421 µs]
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) low mild
  1 (1.00%) high severe
Greeks Calculations/vanna
                        time:   [4.4615 µs 4.4634 µs 4.4655 µs]
Found 11 outliers among 100 measurements (11.00%)
  1 (1.00%) high mild
  10 (10.00%) high severe
Greeks Calculations/vomma
                        time:   [4.6899 µs 4.6906 µs 4.6914 µs]
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) low mild
  1 (1.00%) high mild
Greeks Calculations/veta
                        time:   [4.8305 µs 4.8394 µs 4.8529 µs]
Found 3 outliers among 100 measurements (3.00%)
  3 (3.00%) high severe
Greeks Calculations/charm
                        time:   [4.9811 µs 4.9826 µs 4.9841 µs]
Found 3 outliers among 100 measurements (3.00%)
  3 (3.00%) high mild
Greeks Calculations/color
                        time:   [4.8450 µs 4.8594 µs 4.8795 µs]
Found 17 outliers among 100 measurements (17.00%)
  17 (17.00%) high severe
Greeks Calculations/all_greeks
                        time:   [9.6193 µs 9.6209 µs 9.6224 µs]
Found 5 outliers among 100 measurements (5.00%)
  1 (1.00%) low severe
  3 (3.00%) high mild
  1 (1.00%) high severe

Valuations/payoff       time:   [46.064 ns 46.098 ns 46.139 ns]
Found 20 outliers among 100 measurements (20.00%)
  1 (1.00%) low mild
  19 (19.00%) high severe
Valuations/intrinsic_value
                        time:   [117.55 ns 117.63 ns 117.73 ns]
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high severe
Valuations/time_value   time:   [5.4669 µs 5.4687 µs 5.4704 µs]
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) low mild
  1 (1.00%) high mild
Valuations/pnl_calculation
                        time:   [25.642 µs 25.646 µs 25.651 µs]
Found 6 outliers among 100 measurements (6.00%)
  4 (4.00%) low mild
  2 (2.00%) high mild

Binary Tree Operations/binomial_tree_10_steps
                        time:   [49.439 µs 49.465 µs 49.506 µs]
Found 5 outliers among 50 measurements (10.00%)
  1 (2.00%) high mild
  4 (8.00%) high severe
Binary Tree Operations/binomial_tree_50_steps
                        time:   [1.6254 ms 1.6265 ms 1.6275 ms]
Binary Tree Operations/binomial_tree_100_steps
                        time:   [7.5989 ms 7.6033 ms 7.6111 ms]
Found 3 outliers among 50 measurements (6.00%)
  1 (2.00%) high mild
  2 (4.00%) high severe
Binary Tree Operations/binomial_tree_200_steps
                        time:   [35.246 ms 35.628 ms 36.121 ms]
Found 5 outliers among 50 measurements (10.00%)
  2 (4.00%) high mild
  3 (6.00%) high severe

Maturity Impact on Pricing/black_scholes_1_days
                        time:   [5.7518 µs 5.7541 µs 5.7564 µs]
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) high mild
  1 (1.00%) high severe
Maturity Impact on Pricing/black_scholes_7_days
                        time:   [5.3730 µs 5.3751 µs 5.3773 µs]
Found 4 outliers among 100 measurements (4.00%)
  1 (1.00%) low mild
  3 (3.00%) high mild
Maturity Impact on Pricing/black_scholes_30_days
                        time:   [5.4002 µs 5.4050 µs 5.4122 µs]
Found 8 outliers among 100 measurements (8.00%)
  2 (2.00%) low mild
  6 (6.00%) high severe
Maturity Impact on Pricing/black_scholes_90_days
                        time:   [5.3337 µs 5.3354 µs 5.3372 µs]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) low mild
Maturity Impact on Pricing/black_scholes_365_days
                        time:   [4.6760 µs 4.6835 µs 4.6954 µs]
Found 4 outliers among 100 measurements (4.00%)
  1 (1.00%) low severe
  1 (1.00%) high mild
  2 (2.00%) high severe

Position Costs and Fees/total_cost
                        time:   [50.221 ns 50.304 ns 50.394 ns]
Position Costs and Fees/premium_received
                        time:   [5.8030 ns 5.8305 ns 5.8624 ns]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
Position Costs and Fees/net_premium_received
                        time:   [5.8003 ns 5.8227 ns 5.8472 ns]
Position Costs and Fees/fees
                        time:   [28.089 ns 28.159 ns 28.234 ns]
Position Costs and Fees/net_cost
                        time:   [55.914 ns 56.023 ns 56.127 ns]

Position Profit Calculations/break_even
                        time:   [97.555 ns 97.747 ns 97.964 ns]
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high severe
Position Profit Calculations/pnl_at_expiration
                        time:   [155.30 ns 155.45 ns 155.62 ns]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
Position Profit Calculations/unrealized_pnl
                        time:   [37.913 ns 38.038 ns 38.189 ns]
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) high mild
  1 (1.00%) high severe
Position Profit Calculations/calculate_pnl
                        time:   [25.582 µs 25.587 µs 25.591 µs]
Found 3 outliers among 100 measurements (3.00%)
  1 (1.00%) low mild
  2 (2.00%) high mild

Time Calculations/days_held
                        time:   [53.171 ns 53.367 ns 53.649 ns]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high severe
Time Calculations/days_to_expiration
                        time:   [5.8024 ns 5.8256 ns 5.8512 ns]

Validation Operations/validate
                        time:   [16.718 ns 16.774 ns 16.840 ns]
Found 21 outliers among 100 measurements (21.00%)
  1 (1.00%) high mild
  20 (20.00%) high severe
Validation Operations/is_long
                        time:   [216.04 ps 216.19 ps 216.35 ps]
Found 9 outliers among 100 measurements (9.00%)
  5 (5.00%) high mild
  4 (4.00%) high severe
Validation Operations/is_short
                        time:   [213.66 ps 214.27 ps 215.34 ps]
Found 10 outliers among 100 measurements (10.00%)
  4 (4.00%) high mild
  6 (6.00%) high severe

Strategies/long_call_max_profit
                        time:   [4.8631 ns 4.9023 ns 4.9404 ns]
Strategies/long_call_max_loss
                        time:   [67.891 ns 67.965 ns 68.048 ns]
Found 9 outliers among 100 measurements (9.00%)
  2 (2.00%) low mild
  7 (7.00%) high mild
Strategies/bull_call_spread_max_profit
                        time:   [282.21 ns 282.33 ns 282.46 ns]
Found 4 outliers among 100 measurements (4.00%)
  4 (4.00%) high mild
Strategies/bull_call_spread_max_loss
                        time:   [240.63 ns 240.77 ns 240.92 ns]
Found 3 outliers among 100 measurements (3.00%)
  3 (3.00%) high mild
Strategies/iron_condor_max_profit
                        time:   [891.86 ns 893.65 ns 896.65 ns]
Found 4 outliers among 100 measurements (4.00%)
  2 (2.00%) high mild
  2 (2.00%) high severe
Strategies/iron_condor_max_loss
                        time:   [988.15 ns 988.64 ns 989.15 ns]
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high mild
Strategies/iron_butterfly_max_profit
                        time:   [892.69 ns 893.59 ns 894.77 ns]
Found 4 outliers among 100 measurements (4.00%)
  4 (4.00%) high severe
Strategies/iron_butterfly_max_loss
                        time:   [985.75 ns 986.25 ns 986.77 ns]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild

curve_merge_multiply/size_32/2
                        time:   [125.52 µs 126.78 µs 128.25 µs]
                        thrpt:  [15.594 Kelem/s 15.776 Kelem/s 15.933 Kelem/s]
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) high mild
  1 (1.00%) high severe
curve_merge_multiply/size_32/3
                        time:   [163.92 µs 164.67 µs 165.44 µs]
                        thrpt:  [18.133 Kelem/s 18.218 Kelem/s 18.302 Kelem/s]
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high mild
curve_merge_multiply/size_32/5
                        time:   [229.45 µs 230.66 µs 232.21 µs]
                        thrpt:  [21.532 Kelem/s 21.677 Kelem/s 21.791 Kelem/s]
Found 9 outliers among 100 measurements (9.00%)
  1 (1.00%) low severe
  3 (3.00%) low mild
  3 (3.00%) high mild
  2 (2.00%) high severe
curve_merge_multiply/size_32/10
                        time:   [381.29 µs 382.33 µs 383.38 µs]
                        thrpt:  [26.084 Kelem/s 26.155 Kelem/s 26.227 Kelem/s]
Found 3 outliers among 100 measurements (3.00%)
  1 (1.00%) low mild
  2 (2.00%) high mild
curve_merge_multiply/size_128/2
                        time:   [252.34 µs 253.30 µs 254.44 µs]
                        thrpt:  [7.8604 Kelem/s 7.8959 Kelem/s 7.9260 Kelem/s]
Found 11 outliers among 100 measurements (11.00%)
  2 (2.00%) low mild
  6 (6.00%) high mild
  3 (3.00%) high severe
curve_merge_multiply/size_128/3
                        time:   [336.42 µs 359.28 µs 387.84 µs]
                        thrpt:  [7.7352 Kelem/s 8.3499 Kelem/s 8.9174 Kelem/s]
Found 9 outliers among 100 measurements (9.00%)
  1 (1.00%) high mild
  8 (8.00%) high severe
curve_merge_multiply/size_128/5
                        time:   [474.47 µs 475.51 µs 476.56 µs]
                        thrpt:  [10.492 Kelem/s 10.515 Kelem/s 10.538 Kelem/s]

Warning: Unable to complete 100 samples in 4.0s. You may wish to increase target time to 4.3s, enable flat sampling, or reduce sample count to 60.
curve_merge_multiply/size_128/10
                        time:   [853.45 µs 855.78 µs 858.63 µs]
                        thrpt:  [11.646 Kelem/s 11.685 Kelem/s 11.717 Kelem/s]
Found 3 outliers among 100 measurements (3.00%)
  2 (2.00%) high mild
  1 (1.00%) high severe
curve_merge_multiply/size_512/2
                        time:   [640.84 µs 642.00 µs 643.12 µs]
                        thrpt:  [3.1098 Kelem/s 3.1153 Kelem/s 3.1209 Kelem/s]
Found 4 outliers among 100 measurements (4.00%)
  1 (1.00%) low mild
  1 (1.00%) high mild
  2 (2.00%) high severe

Warning: Unable to complete 100 samples in 4.0s. You may wish to increase target time to 4.5s, enable flat sampling, or reduce sample count to 60.
curve_merge_multiply/size_512/3
                        time:   [899.36 µs 905.09 µs 912.86 µs]
                        thrpt:  [3.2864 Kelem/s 3.3146 Kelem/s 3.3357 Kelem/s]
Found 6 outliers among 100 measurements (6.00%)
  1 (1.00%) low mild
  3 (3.00%) high mild
  2 (2.00%) high severe

Warning: Unable to complete 100 samples in 4.0s. You may wish to increase target time to 7.1s, enable flat sampling, or reduce sample count to 50.
curve_merge_multiply/size_512/5
                        time:   [1.4170 ms 1.4196 ms 1.4222 ms]
                        thrpt:  [3.5156 Kelem/s 3.5221 Kelem/s 3.5286 Kelem/s]
Found 6 outliers among 100 measurements (6.00%)
  1 (1.00%) low severe
  3 (3.00%) low mild
  1 (1.00%) high mild
  1 (1.00%) high severe
curve_merge_multiply/size_512/10
                        time:   [2.7305 ms 2.7405 ms 2.7520 ms]
                        thrpt:  [3.6338 Kelem/s 3.6489 Kelem/s 3.6624 Kelem/s]
Found 6 outliers among 100 measurements (6.00%)
  1 (1.00%) high mild
  5 (5.00%) high severe

surface_merge_multiply/side_8/2
                        time:   [22.016 ms 24.216 ms 27.081 ms]
                        thrpt:  [73.852  elem/s 82.591  elem/s 90.843  elem/s]
Found 2 outliers among 10 measurements (20.00%)
  2 (20.00%) high severe
surface_merge_multiply/side_8/3
                        time:   [31.893 ms 32.007 ms 32.240 ms]
                        thrpt:  [93.054  elem/s 93.728  elem/s 94.063  elem/s]
Found 1 outliers among 10 measurements (10.00%)
  1 (10.00%) high mild
surface_merge_multiply/side_8/5
                        time:   [51.023 ms 51.076 ms 51.137 ms]
                        thrpt:  [97.776  elem/s 97.892  elem/s 97.995  elem/s]

Warning: Unable to complete 10 samples in 4.0s. You may wish to increase target time to 5.4s or enable flat sampling.
surface_merge_multiply/side_8/10
                        time:   [97.425 ms 97.527 ms 97.813 ms]
                        thrpt:  [102.24  elem/s 102.54  elem/s 102.64  elem/s]
Found 2 outliers among 10 measurements (20.00%)
  2 (20.00%) high severe
surface_merge_multiply/side_16/2
                        time:   [62.620 ms 62.851 ms 63.114 ms]
                        thrpt:  [31.689  elem/s 31.821  elem/s 31.939  elem/s]
Found 2 outliers among 10 measurements (20.00%)
  1 (10.00%) high mild
  1 (10.00%) high severe

Warning: Unable to complete 10 samples in 4.0s. You may wish to increase target time to 5.1s or enable flat sampling.
surface_merge_multiply/side_16/3
                        time:   [92.587 ms 92.660 ms 92.724 ms]
                        thrpt:  [32.354  elem/s 32.377  elem/s 32.402  elem/s]
surface_merge_multiply/side_16/5
                        time:   [151.82 ms 152.55 ms 153.65 ms]
                        thrpt:  [32.542  elem/s 32.776  elem/s 32.934  elem/s]
Found 2 outliers among 10 measurements (20.00%)
  1 (10.00%) high mild
  1 (10.00%) high severe
surface_merge_multiply/side_16/10
                        time:   [296.73 ms 300.50 ms 306.27 ms]
                        thrpt:  [32.651  elem/s 33.277  elem/s 33.701  elem/s]
Found 2 outliers among 10 measurements (20.00%)
  2 (20.00%) high severe

decimal_product_fold/parallel_reduce/2
                        time:   [12.559 µs 12.657 µs 12.762 µs]
Found 7 outliers among 100 measurements (7.00%)
  3 (3.00%) low mild
  4 (4.00%) high mild
decimal_product_fold/sequential_fold/2
                        time:   [68.758 ns 68.777 ns 68.796 ns]
Found 8 outliers among 100 measurements (8.00%)
  1 (1.00%) low mild
  4 (4.00%) high mild
  3 (3.00%) high severe
decimal_product_fold/parallel_reduce/3
                        time:   [14.490 µs 14.666 µs 14.859 µs]
Found 3 outliers among 100 measurements (3.00%)
  3 (3.00%) high mild
decimal_product_fold/sequential_fold/3
                        time:   [155.33 ns 155.35 ns 155.37 ns]
Found 10 outliers among 100 measurements (10.00%)
  5 (5.00%) high mild
  5 (5.00%) high severe
decimal_product_fold/parallel_reduce/5
                        time:   [18.610 µs 18.709 µs 18.800 µs]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) low mild
decimal_product_fold/sequential_fold/5
                        time:   [331.39 ns 331.44 ns 331.49 ns]
Found 10 outliers among 100 measurements (10.00%)
  6 (6.00%) high mild
  4 (4.00%) high severe
decimal_product_fold/parallel_reduce/10
                        time:   [19.703 µs 19.816 µs 19.926 µs]
Found 3 outliers among 100 measurements (3.00%)
  1 (1.00%) low severe
  2 (2.00%) high mild
decimal_product_fold/sequential_fold/10
                        time:   [772.52 ns 772.68 ns 772.86 ns]
Found 7 outliers among 100 measurements (7.00%)
  5 (5.00%) high mild
  2 (2.00%) high severe

     Running benches/analytics.rs (target/release/deps/analytics-87b3662b4808deea)
analytics/probability/single_point
                        time:   [9.5539 µs 9.5914 µs 9.6451 µs]
Found 12 outliers among 100 measurements (12.00%)
  4 (4.00%) high mild
  8 (8.00%) high severe
analytics/probability/single_point_with_trend
                        time:   [9.5571 µs 9.5584 µs 9.5598 µs]
Found 5 outliers among 100 measurements (5.00%)
  1 (1.00%) low mild
  3 (3.00%) high mild
  1 (1.00%) high severe
analytics/probability/price_range
                        time:   [17.128 µs 17.130 µs 17.132 µs]
Found 7 outliers among 100 measurements (7.00%)
  4 (4.00%) low mild
  3 (3.00%) high mild

analytics/pnl_risk/position_calculate_pnl
                        time:   [40.631 µs 40.695 µs 40.805 µs]
Found 6 outliers among 100 measurements (6.00%)
  4 (4.00%) high mild
  2 (2.00%) high severe
analytics/pnl_risk/position_calculate_pnl_at_expiration
                        time:   [158.29 ns 158.41 ns 158.54 ns]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
analytics/pnl_risk/options_calculate_pnl
                        time:   [40.665 µs 40.755 µs 40.904 µs]
Found 8 outliers among 100 measurements (8.00%)
  1 (1.00%) low mild
  5 (5.00%) high mild
  2 (2.00%) high severe
analytics/pnl_risk/span_margin
                        time:   [312.43 µs 312.56 µs 312.80 µs]
Found 6 outliers among 100 measurements (6.00%)
  1 (1.00%) low severe
  1 (1.00%) low mild
  2 (2.00%) high mild
  2 (2.00%) high severe
analytics/pnl_risk/position_calculate_pnl_sweep/101
                        time:   [3.9925 ms 4.0034 ms 4.0219 ms]
                        thrpt:  [25.112 Kelem/s 25.229 Kelem/s 25.298 Kelem/s]
Found 8 outliers among 100 measurements (8.00%)
  2 (2.00%) high mild
  6 (6.00%) high severe

analytics/rnd/calculate_rnd/21_strikes_50_points
                        time:   [21.451 µs 21.453 µs 21.457 µs]
Found 2 outliers among 20 measurements (10.00%)
  1 (5.00%) high mild
  1 (5.00%) high severe
analytics/rnd/calculate_rnd/21_strikes_200_points
                        time:   [21.448 µs 21.456 µs 21.470 µs]
Found 1 outliers among 20 measurements (5.00%)
  1 (5.00%) high severe
analytics/rnd/calculate_skew/21
                        time:   [970.93 ns 978.69 ns 984.45 ns]
Found 3 outliers among 20 measurements (15.00%)
  3 (15.00%) high severe
analytics/rnd/calculate_rnd/51_strikes_50_points
                        time:   [51.299 µs 51.310 µs 51.319 µs]
Found 1 outliers among 20 measurements (5.00%)
  1 (5.00%) low mild
analytics/rnd/calculate_rnd/51_strikes_200_points
                        time:   [51.321 µs 51.336 µs 51.360 µs]
Found 2 outliers among 20 measurements (10.00%)
  1 (5.00%) low mild
  1 (5.00%) high severe
analytics/rnd/calculate_skew/51
                        time:   [2.4462 µs 2.4688 µs 2.4853 µs]
Found 3 outliers among 20 measurements (15.00%)
  3 (15.00%) high severe

analytics/projections/delta_curve/21
                        time:   [208.79 µs 208.84 µs 208.89 µs]
Found 1 outliers among 20 measurements (5.00%)
  1 (5.00%) high severe
analytics/projections/gamma_curve/21
                        time:   [270.34 µs 270.39 µs 270.46 µs]
Found 2 outliers among 20 measurements (10.00%)
  1 (5.00%) high mild
  1 (5.00%) high severe
analytics/projections/vanna_surface/21x5
                        time:   [1.3332 ms 1.3356 ms 1.3407 ms]
Found 7 outliers among 20 measurements (35.00%)
  4 (20.00%) low mild
  1 (5.00%) high mild
  2 (10.00%) high severe
analytics/projections/theta_time_surface/21x5
                        time:   [1.5766 ms 1.6299 ms 1.7159 ms]
Found 4 outliers among 20 measurements (20.00%)
  1 (5.00%) low mild
  3 (15.00%) high severe

analytics/metrics/iv_curve/21
                        time:   [588.40 ns 589.14 ns 589.91 ns]
Found 1 outliers among 20 measurements (5.00%)
  1 (5.00%) low mild
analytics/metrics/volatility_skew/21
                        time:   [1.8982 µs 1.8985 µs 1.8989 µs]
Found 1 outliers among 20 measurements (5.00%)
  1 (5.00%) low severe
analytics/metrics/premium_weighted_pcr/21
                        time:   [1.6728 µs 1.6769 µs 1.6834 µs]
Found 1 outliers among 20 measurements (5.00%)
  1 (5.00%) high severe
analytics/metrics/risk_reversal_curve/21
                        time:   [1.1338 µs 1.1348 µs 1.1358 µs]
analytics/metrics/dollar_gamma_curve/21
                        time:   [270.42 µs 270.51 µs 270.59 µs]
analytics/metrics/delta_gamma_curve/21
                        time:   [479.07 µs 479.29 µs 479.60 µs]
Found 2 outliers among 20 measurements (10.00%)
  2 (10.00%) high severe
analytics/metrics/theta_curve/21
                        time:   [307.31 µs 307.39 µs 307.50 µs]
Found 1 outliers among 20 measurements (5.00%)
  1 (5.00%) high severe
analytics/metrics/price_shock_curve/21
                        time:   [478.87 µs 479.19 µs 479.43 µs]
Found 5 outliers among 20 measurements (25.00%)
  1 (5.00%) low mild
  4 (20.00%) high severe
analytics/metrics/iv_surface/21x5
                        time:   [99.825 µs 99.979 µs 100.17 µs]
Found 4 outliers among 20 measurements (20.00%)
  3 (15.00%) low severe
  1 (5.00%) high severe
analytics/metrics/vanna_volga_surface/10x10
                        time:   [22.472 µs 22.539 µs 22.627 µs]
                        thrpt:  [4.4196 Melem/s 4.4368 Melem/s 4.4501 Melem/s]
Found 3 outliers among 20 measurements (15.00%)
  3 (15.00%) low severe
analytics/metrics/volatility_sensitivity_surface/10x10
                        time:   [2.1470 ms 2.1475 ms 2.1480 ms]
                        thrpt:  [46.555 Kelem/s 46.566 Kelem/s 46.577 Kelem/s]
analytics/metrics/time_decay_surface/10x5
                        time:   [988.53 µs 988.98 µs 989.81 µs]
                        thrpt:  [101.03 Kelem/s 101.11 Kelem/s 101.16 Kelem/s]
Found 4 outliers among 20 measurements (20.00%)
  1 (5.00%) high mild
  3 (15.00%) high severe
analytics/metrics/theta_surface/10x5
                        time:   [712.89 µs 713.13 µs 713.43 µs]
                        thrpt:  [140.17 Kelem/s 140.23 Kelem/s 140.27 Kelem/s]
analytics/metrics/vanna_volga_surface/20x20
                        time:   [94.849 µs 95.188 µs 95.671 µs]
                        thrpt:  [4.1810 Melem/s 4.2022 Melem/s 4.2172 Melem/s]
Found 6 outliers among 20 measurements (30.00%)
  4 (20.00%) low severe
  1 (5.00%) high mild
  1 (5.00%) high severe
analytics/metrics/volatility_sensitivity_surface/20x20
                        time:   [7.9281 ms 7.9307 ms 7.9335 ms]
                        thrpt:  [50.419 Kelem/s 50.437 Kelem/s 50.453 Kelem/s]
Found 1 outliers among 20 measurements (5.00%)
  1 (5.00%) high severe
analytics/metrics/time_decay_surface/20x5
                        time:   [1.9104 ms 1.9132 ms 1.9195 ms]
                        thrpt:  [208.39 Kelem/s 209.07 Kelem/s 209.38 Kelem/s]
Found 1 outliers among 20 measurements (5.00%)
  1 (5.00%) high severe
analytics/metrics/theta_surface/20x5
                        time:   [1.3992 ms 1.4449 ms 1.5158 ms]
                        thrpt:  [263.88 Kelem/s 276.83 Kelem/s 285.89 Kelem/s]
Found 3 outliers among 20 measurements (15.00%)
  3 (15.00%) high severe

     Running benches/backtest.rs (target/release/deps/backtest-e801d440f1689e8a)
backtest/run/long_call_expiration/100_paths
                        time:   [60.868 ms 60.902 ms 61.002 ms]
                        thrpt:  [49.178 Kelem/s 49.260 Kelem/s 49.287 Kelem/s]
Found 1 outliers among 10 measurements (10.00%)
  1 (10.00%) high severe
backtest/run/short_put_profit_or_loss/100_paths
                        time:   [2.1529 ms 2.1531 ms 2.1533 ms]
                        thrpt:  [1.3932 Melem/s 1.3933 Melem/s 1.3935 Melem/s]
Found 1 outliers among 10 measurements (10.00%)
  1 (10.00%) high mild

Warning: Unable to complete 10 samples in 4.0s. You may wish to increase target time to 6.1s.
backtest/run/long_call_expiration/1000_paths
                        time:   [613.46 ms 613.52 ms 613.57 ms]
                        thrpt:  [48.894 Kelem/s 48.898 Kelem/s 48.903 Kelem/s]
Found 1 outliers among 10 measurements (10.00%)
  1 (10.00%) low mild
backtest/run/short_put_profit_or_loss/1000_paths
                        time:   [21.499 ms 21.548 ms 21.633 ms]
                        thrpt:  [1.3868 Melem/s 1.3922 Melem/s 1.3954 Melem/s]
Found 1 outliers among 10 measurements (10.00%)
  1 (10.00%) high severe

backtest/statistics/from_results/100
                        time:   [13.461 µs 13.486 µs 13.513 µs]
                        thrpt:  [7.4003 Melem/s 7.4152 Melem/s 7.4291 Melem/s]
backtest/statistics/from_results/1000
                        time:   [166.37 µs 166.77 µs 167.26 µs]
                        thrpt:  [5.9786 Melem/s 5.9961 Melem/s 6.0108 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild

     Running benches/core.rs (target/release/deps/core-eaa04caa4c1598ae)
core/decimal/d_add      time:   [11.435 ns 11.713 ns 12.059 ns]
Found 7 outliers among 100 measurements (7.00%)
  2 (2.00%) high mild
  5 (5.00%) high severe
core/decimal/d_mul      time:   [5.2966 ns 5.3117 ns 5.3362 ns]
Found 6 outliers among 100 measurements (6.00%)
  4 (4.00%) high mild
  2 (2.00%) high severe
core/decimal/d_div      time:   [41.490 ns 41.628 ns 41.756 ns]
core/decimal/d_exp      time:   [762.95 ns 764.43 ns 767.28 ns]
Found 3 outliers among 100 measurements (3.00%)
  1 (1.00%) high mild
  2 (2.00%) high severe
core/decimal/d_ln       time:   [8.0000 µs 8.0010 µs 8.0021 µs]
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high mild
core/decimal/d_sqrt     time:   [796.12 ns 796.71 ns 797.29 ns]
Found 4 outliers among 100 measurements (4.00%)
  2 (2.00%) high mild
  2 (2.00%) high severe
core/decimal/d_powd     time:   [10.631 µs 10.632 µs 10.635 µs]
Found 7 outliers among 100 measurements (7.00%)
  4 (4.00%) high mild
  3 (3.00%) high severe
core/decimal/decimal_to_f64
                        time:   [13.288 ns 13.292 ns 13.298 ns]
Found 13 outliers among 100 measurements (13.00%)
  2 (2.00%) low severe
  1 (1.00%) low mild
  7 (7.00%) high mild
  3 (3.00%) high severe
core/decimal/f64_to_decimal
                        time:   [147.84 ns 147.89 ns 147.95 ns]
Found 6 outliers among 100 measurements (6.00%)
  2 (2.00%) high mild
  4 (4.00%) high severe
core/decimal/d_sum/21   time:   [73.816 ns 73.932 ns 74.041 ns]
                        thrpt:  [283.63 Melem/s 284.04 Melem/s 284.49 Melem/s]
Found 4 outliers among 100 measurements (4.00%)
  4 (4.00%) high mild
core/decimal/mean/21    time:   [81.798 ns 81.944 ns 82.096 ns]
                        thrpt:  [255.80 Melem/s 256.27 Melem/s 256.73 Melem/s]
Found 6 outliers among 100 measurements (6.00%)
  1 (1.00%) low mild
  3 (3.00%) high mild
  2 (2.00%) high severe
core/decimal/std_dev/21 time:   [3.5673 µs 3.5742 µs 3.5823 µs]
                        thrpt:  [5.8621 Melem/s 5.8754 Melem/s 5.8867 Melem/s]
core/decimal/d_sum/252  time:   [3.0581 µs 3.0610 µs 3.0645 µs]
                        thrpt:  [82.233 Melem/s 82.327 Melem/s 82.404 Melem/s]
Found 12 outliers among 100 measurements (12.00%)
  1 (1.00%) low mild
  4 (4.00%) high mild
  7 (7.00%) high severe
core/decimal/mean/252   time:   [3.0414 µs 3.0427 µs 3.0440 µs]
                        thrpt:  [82.785 Melem/s 82.821 Melem/s 82.856 Melem/s]
Found 9 outliers among 100 measurements (9.00%)
  2 (2.00%) low mild
  6 (6.00%) high mild
  1 (1.00%) high severe
core/decimal/std_dev/252
                        time:   [31.628 µs 31.690 µs 31.765 µs]
                        thrpt:  [7.9333 Melem/s 7.9521 Melem/s 7.9675 Melem/s]
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) high mild
  1 (1.00%) high severe
core/decimal/d_sum/1008 time:   [22.728 µs 22.749 µs 22.772 µs]
                        thrpt:  [44.266 Melem/s 44.309 Melem/s 44.350 Melem/s]
Found 3 outliers among 100 measurements (3.00%)
  3 (3.00%) high severe
core/decimal/mean/1008  time:   [22.695 µs 22.754 µs 22.842 µs]
                        thrpt:  [44.129 Melem/s 44.300 Melem/s 44.415 Melem/s]
Found 9 outliers among 100 measurements (9.00%)
  1 (1.00%) low mild
  4 (4.00%) high mild
  4 (4.00%) high severe
core/decimal/std_dev/1008
                        time:   [135.62 µs 138.48 µs 141.99 µs]
                        thrpt:  [7.0991 Melem/s 7.2790 Melem/s 7.4324 Melem/s]
Found 8 outliers among 100 measurements (8.00%)
  8 (8.00%) high severe

core/positive/new_from_f64
                        time:   [35.192 ns 35.275 ns 35.384 ns]
Found 5 outliers among 100 measurements (5.00%)
  3 (3.00%) high mild
  2 (2.00%) high severe
core/positive/add       time:   [10.576 ns 10.623 ns 10.674 ns]
core/positive/mul       time:   [10.703 ns 10.722 ns 10.745 ns]
Found 20 outliers among 100 measurements (20.00%)
  20 (20.00%) high severe
core/positive/checked_div
                        time:   [25.033 ns 25.060 ns 25.089 ns]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high severe
core/positive/to_f64    time:   [13.963 ns 13.969 ns 13.974 ns]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
core/positive/to_dec    time:   [280.89 ps 301.16 ps 324.35 ps]
core/positive/calculate_log_returns/21
                        time:   [160.13 µs 160.17 µs 160.22 µs]
                        thrpt:  [131.07 Kelem/s 131.11 Kelem/s 131.14 Kelem/s]
Found 3 outliers among 100 measurements (3.00%)
  1 (1.00%) high mild
  2 (2.00%) high severe
core/positive/calculate_log_returns/252
                        time:   [2.0388 ms 2.0421 ms 2.0471 ms]
                        thrpt:  [123.10 Kelem/s 123.40 Kelem/s 123.60 Kelem/s]
Found 8 outliers among 100 measurements (8.00%)
  4 (4.00%) high mild
  4 (4.00%) high severe
core/positive/calculate_log_returns/1008
                        time:   [8.1921 ms 8.1928 ms 8.1936 ms]
                        thrpt:  [123.02 Kelem/s 123.04 Kelem/s 123.05 Kelem/s]
Found 3 outliers among 100 measurements (3.00%)
  2 (2.00%) high mild
  1 (1.00%) high severe

core/payoff/option_type_payoff
                        time:   [73.112 ns 73.256 ns 73.457 ns]
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high severe
core/payoff/options_payoff
                        time:   [48.552 ns 48.606 ns 48.659 ns]
core/payoff/options_payoff_at_price
                        time:   [89.878 ns 90.089 ns 90.403 ns]
Found 3 outliers among 100 measurements (3.00%)
  1 (1.00%) high mild
  2 (2.00%) high severe
core/payoff/options_intrinsic_value
                        time:   [89.209 ns 89.264 ns 89.323 ns]
core/payoff/position_pnl_at_expiration
                        time:   [154.15 ns 154.54 ns 155.07 ns]
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high severe
core/payoff/position_unrealized_pnl
                        time:   [36.979 ns 37.028 ns 37.079 ns]
Found 8 outliers among 100 measurements (8.00%)
  7 (7.00%) high mild
  1 (1.00%) high severe
core/payoff/position_break_even
                        time:   [95.005 ns 95.139 ns 95.259 ns]
core/payoff/position_total_cost
                        time:   [48.612 ns 49.943 ns 51.511 ns]
Found 7 outliers among 100 measurements (7.00%)
  7 (7.00%) high severe
core/payoff/position_pnl_at_expiration_sweep/201
                        time:   [28.033 µs 28.056 µs 28.077 µs]
                        thrpt:  [7.1590 Melem/s 7.1644 Melem/s 7.1702 Melem/s]

core/construction/options_new
                        time:   [271.43 ns 271.58 ns 271.74 ns]
Found 14 outliers among 100 measurements (14.00%)
  2 (2.00%) low mild
  12 (12.00%) high mild
core/construction/position_new
                        time:   [410.62 ns 411.02 ns 411.53 ns]
core/construction/options_clone
                        time:   [19.544 ns 19.599 ns 19.651 ns]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
core/construction/options_time_to_expiration
                        time:   [97.240 ns 97.306 ns 97.376 ns]
Found 4 outliers among 100 measurements (4.00%)
  4 (4.00%) high mild
core/construction/expiration_days_get_years
                        time:   [97.358 ns 97.433 ns 97.521 ns]
Found 20 outliers among 100 measurements (20.00%)
  2 (2.00%) high mild
  18 (18.00%) high severe
core/construction/options_to_json
                        time:   [340.76 ns 341.38 ns 341.96 ns]
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) high mild
  1 (1.00%) high severe
core/construction/options_from_json
                        time:   [447.24 ns 448.87 ns 451.61 ns]
Found 5 outliers among 100 measurements (5.00%)
  2 (2.00%) low severe
  1 (1.00%) low mild
  1 (1.00%) high mild
  1 (1.00%) high severe

     Running benches/chains.rs (target/release/deps/chains-c6b8f15d64adcabc)
market/chain_build/build_chain/21
                        time:   [1.6677 ms 1.6683 ms 1.6690 ms]
                        thrpt:  [12.582 Kelem/s 12.588 Kelem/s 12.592 Kelem/s]
Found 3 outliers among 20 measurements (15.00%)
  2 (10.00%) high mild
  1 (5.00%) high severe
market/chain_build/build_chain_with_greeks/21
                        time:   [2.4847 ms 2.4851 ms 2.4854 ms]
                        thrpt:  [8.4493 Kelem/s 8.4505 Kelem/s 8.4516 Kelem/s]
Found 1 outliers among 20 measurements (5.00%)
  1 (5.00%) high mild
market/chain_build/to_build_params/21
                        time:   [2.9104 µs 2.9142 µs 2.9172 µs]
                        thrpt:  [7.1988 Melem/s 7.2061 Melem/s 7.2155 Melem/s]
Found 1 outliers among 20 measurements (5.00%)
  1 (5.00%) high mild
market/chain_build/build_chain/51
                        time:   [4.1890 ms 4.1935 ms 4.2024 ms]
                        thrpt:  [12.136 Kelem/s 12.162 Kelem/s 12.175 Kelem/s]
Found 3 outliers among 20 measurements (15.00%)
  1 (5.00%) high mild
  2 (10.00%) high severe
market/chain_build/build_chain_with_greeks/51
                        time:   [6.1590 ms 6.1595 ms 6.1603 ms]
                        thrpt:  [8.2788 Kelem/s 8.2798 Kelem/s 8.2806 Kelem/s]
Found 1 outliers among 20 measurements (5.00%)
  1 (5.00%) high mild
market/chain_build/to_build_params/51
                        time:   [5.9265 µs 5.9312 µs 5.9350 µs]
                        thrpt:  [8.5931 Melem/s 8.5985 Melem/s 8.6055 Melem/s]
Found 3 outliers among 20 measurements (15.00%)
  2 (10.00%) high mild
  1 (5.00%) high severe
market/chain_build/build_chain/101
                        time:   [8.3565 ms 8.3572 ms 8.3582 ms]
                        thrpt:  [12.084 Kelem/s 12.085 Kelem/s 12.086 Kelem/s]
market/chain_build/build_chain_with_greeks/101
                        time:   [12.260 ms 12.264 ms 12.273 ms]
                        thrpt:  [8.2292 Kelem/s 8.2355 Kelem/s 8.2382 Kelem/s]
Found 4 outliers among 20 measurements (20.00%)
  2 (10.00%) high mild
  2 (10.00%) high severe
market/chain_build/to_build_params/101
                        time:   [10.946 µs 10.961 µs 10.973 µs]
                        thrpt:  [9.2045 Melem/s 9.2143 Melem/s 9.2267 Melem/s]
Found 4 outliers among 20 measurements (20.00%)
  4 (20.00%) high mild
market/chain_build/build_chain/201
                        time:   [16.687 ms 16.695 ms 16.716 ms]
                        thrpt:  [12.024 Kelem/s 12.039 Kelem/s 12.045 Kelem/s]
Found 2 outliers among 20 measurements (10.00%)
  2 (10.00%) high severe

Warning: Unable to complete 20 samples in 4.0s. You may wish to increase target time to 5.1s, enable flat sampling, or reduce sample count to 10.
market/chain_build/build_chain_with_greeks/201
                        time:   [24.469 ms 24.473 ms 24.479 ms]
                        thrpt:  [8.2112 Kelem/s 8.2132 Kelem/s 8.2146 Kelem/s]
Found 3 outliers among 20 measurements (15.00%)
  1 (5.00%) low mild
  2 (10.00%) high severe
market/chain_build/to_build_params/201
                        time:   [21.543 µs 21.583 µs 21.613 µs]
                        thrpt:  [9.3000 Melem/s 9.3127 Melem/s 9.3301 Melem/s]
Found 4 outliers among 20 measurements (20.00%)
  4 (20.00%) high severe

market/chain_lookup/atm_option_data/21
                        time:   [50.032 ns 50.075 ns 50.118 ns]
Found 6 outliers among 100 measurements (6.00%)
  1 (1.00%) low mild
  3 (3.00%) high mild
  2 (2.00%) high severe
market/chain_lookup/get_strikes/21
                        time:   [31.824 ns 31.869 ns 31.915 ns]
Found 5 outliers among 100 measurements (5.00%)
  2 (2.00%) low mild
  1 (1.00%) high mild
  2 (2.00%) high severe
market/chain_lookup/get_optiondata_with_strike/21
                        time:   [289.33 ns 290.21 ns 291.16 ns]
market/chain_lookup/get_call_price/21
                        time:   [51.248 ns 51.347 ns 51.436 ns]
Found 7 outliers among 100 measurements (7.00%)
  3 (3.00%) low mild
  2 (2.00%) high mild
  2 (2.00%) high severe
market/chain_lookup/filter_option_data_upper/21
                        time:   [182.45 ns 182.70 ns 182.97 ns]
Found 5 outliers among 100 measurements (5.00%)
  2 (2.00%) low mild
  3 (3.00%) high mild
market/chain_lookup/get_atm_implied_volatility/21
                        time:   [51.295 ns 51.342 ns 51.392 ns]
Found 12 outliers among 100 measurements (12.00%)
  2 (2.00%) low severe
  4 (4.00%) high mild
  6 (6.00%) high severe
market/chain_lookup/get_position_with_delta/21
                        time:   [507.63 ns 508.55 ns 509.45 ns]
market/chain_lookup/atm_option_data/101
                        time:   [256.81 ns 257.15 ns 257.48 ns]
Found 7 outliers among 100 measurements (7.00%)
  3 (3.00%) low mild
  1 (1.00%) high mild
  3 (3.00%) high severe
market/chain_lookup/get_strikes/101
                        time:   [132.60 ns 132.88 ns 133.18 ns]
Found 11 outliers among 100 measurements (11.00%)
  3 (3.00%) low mild
  8 (8.00%) high mild
market/chain_lookup/get_optiondata_with_strike/101
                        time:   [1.3840 µs 1.3883 µs 1.3930 µs]
market/chain_lookup/get_call_price/101
                        time:   [259.21 ns 259.52 ns 259.87 ns]
Found 6 outliers among 100 measurements (6.00%)
  5 (5.00%) high mild
  1 (1.00%) high severe
market/chain_lookup/filter_option_data_upper/101
                        time:   [780.38 ns 781.55 ns 782.83 ns]
Found 3 outliers among 100 measurements (3.00%)
  3 (3.00%) high mild
market/chain_lookup/get_atm_implied_volatility/101
                        time:   [256.94 ns 257.22 ns 257.49 ns]
Found 4 outliers among 100 measurements (4.00%)
  1 (1.00%) low mild
  1 (1.00%) high mild
  2 (2.00%) high severe
market/chain_lookup/get_position_with_delta/101
                        time:   [2.0293 µs 2.0330 µs 2.0366 µs]
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) low mild
  1 (1.00%) high mild

market/chain_iterators/double_iter_count/21
                        time:   [3.9433 µs 3.9534 µs 3.9659 µs]
Found 7 outliers among 100 measurements (7.00%)
  2 (2.00%) low mild
  4 (4.00%) high mild
  1 (1.00%) high severe
market/chain_iterators/triple_iter_count/21
                        time:   [43.736 µs 43.825 µs 43.919 µs]
Found 4 outliers among 100 measurements (4.00%)
  2 (2.00%) low mild
  2 (2.00%) high mild
market/chain_iterators/quad_iter_count/21
                        time:   [302.47 µs 303.24 µs 304.08 µs]
Found 4 outliers among 100 measurements (4.00%)
  2 (2.00%) low mild
  2 (2.00%) high mild
market/chain_iterators/double_iter_count/51
                        time:   [21.598 µs 21.650 µs 21.701 µs]
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) low mild
  1 (1.00%) high mild
market/chain_iterators/triple_iter_count/51
                        time:   [578.32 µs 580.31 µs 582.23 µs]
Found 3 outliers among 100 measurements (3.00%)
  2 (2.00%) low mild
  1 (1.00%) high severe
market/chain_iterators/quad_iter_count/51
                        time:   [9.8466 ms 9.8650 ms 9.8833 ms]

market/chain_refresh/update_greeks/21
                        time:   [692.68 µs 703.86 µs 721.64 µs]
                        thrpt:  [29.101 Kelem/s 29.835 Kelem/s 30.317 Kelem/s]
Found 8 outliers among 100 measurements (8.00%)
  2 (2.00%) high mild
  6 (6.00%) high severe
market/chain_refresh/update_mid_prices/21
                        time:   [5.0443 µs 5.0565 µs 5.0682 µs]
                        thrpt:  [4.1435 Melem/s 4.1531 Melem/s 4.1631 Melem/s]
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) low mild
market/chain_refresh/clone/21
                        time:   [1.5438 µs 1.5466 µs 1.5494 µs]
                        thrpt:  [13.553 Melem/s 13.578 Melem/s 13.603 Melem/s]
Found 4 outliers among 100 measurements (4.00%)
  1 (1.00%) low mild
  1 (1.00%) high mild
  2 (2.00%) high severe
market/chain_refresh/gamma_exposure/21
                        time:   [186.81 ns 187.88 ns 188.87 ns]
                        thrpt:  [111.19 Melem/s 111.77 Melem/s 112.42 Melem/s]
market/chain_refresh/delta_exposure/21
                        time:   [588.80 ns 590.64 ns 592.55 ns]
                        thrpt:  [35.440 Melem/s 35.554 Melem/s 35.666 Melem/s]
Found 3 outliers among 100 measurements (3.00%)
  3 (3.00%) high mild
market/chain_refresh/vega_exposure/21
                        time:   [540.51 µs 540.57 µs 540.64 µs]
                        thrpt:  [38.843 Kelem/s 38.848 Kelem/s 38.852 Kelem/s]
Found 3 outliers among 100 measurements (3.00%)
  3 (3.00%) high mild
market/chain_refresh/update_greeks/101
                        time:   [3.4751 ms 3.4829 ms 3.4945 ms]
                        thrpt:  [28.903 Kelem/s 28.999 Kelem/s 29.064 Kelem/s]
Found 5 outliers among 100 measurements (5.00%)
  1 (1.00%) high mild
  4 (4.00%) high severe
market/chain_refresh/update_mid_prices/101
                        time:   [22.570 µs 22.613 µs 22.655 µs]
                        thrpt:  [4.4583 Melem/s 4.4664 Melem/s 4.4750 Melem/s]
market/chain_refresh/clone/101
                        time:   [7.8301 µs 7.8395 µs 7.8495 µs]
                        thrpt:  [12.867 Melem/s 12.883 Melem/s 12.899 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high severe
market/chain_refresh/gamma_exposure/101
                        time:   [930.65 ns 943.72 ns 959.64 ns]
                        thrpt:  [105.25 Melem/s 107.02 Melem/s 108.53 Melem/s]
Found 10 outliers among 100 measurements (10.00%)
  2 (2.00%) high mild
  8 (8.00%) high severe
market/chain_refresh/delta_exposure/101
                        time:   [5.8187 µs 5.8379 µs 5.8660 µs]
                        thrpt:  [17.218 Melem/s 17.301 Melem/s 17.358 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high severe
market/chain_refresh/vega_exposure/101
                        time:   [2.7149 ms 2.7151 ms 2.7154 ms]
                        thrpt:  [37.195 Kelem/s 37.199 Kelem/s 37.202 Kelem/s]
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high mild

market/chain_serde/to_json/21
                        time:   [17.662 µs 17.669 µs 17.678 µs]
                        thrpt:  [542.34 MiB/s 542.59 MiB/s 542.83 MiB/s]
Found 3 outliers among 100 measurements (3.00%)
  1 (1.00%) high mild
  2 (2.00%) high severe
market/chain_serde/from_json/21
                        time:   [18.962 µs 18.995 µs 19.028 µs]
                        thrpt:  [503.86 MiB/s 504.74 MiB/s 505.61 MiB/s]
Found 3 outliers among 100 measurements (3.00%)
  3 (3.00%) high mild
market/chain_serde/to_json/101
                        time:   [83.014 µs 83.033 µs 83.052 µs]
                        thrpt:  [550.21 MiB/s 550.34 MiB/s 550.47 MiB/s]
Found 5 outliers among 100 measurements (5.00%)
  2 (2.00%) low mild
  1 (1.00%) high mild
  2 (2.00%) high severe
market/chain_serde/from_json/101
                        time:   [94.451 µs 94.618 µs 94.784 µs]
                        thrpt:  [482.11 MiB/s 482.96 MiB/s 483.81 MiB/s]
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) low mild
  1 (1.00%) high mild

market/series_build/build_series/3
                        time:   [7.2936 ms 7.3049 ms 7.3256 ms]
                        thrpt:  [409.52  elem/s 410.68  elem/s 411.32  elem/s]
Found 2 outliers among 20 measurements (10.00%)
  2 (10.00%) high severe

Warning: Unable to complete 20 samples in 4.0s. You may wish to increase target time to 6.1s, enable flat sampling, or reduce sample count to 10.
market/series_build/build_series/12
                        time:   [28.773 ms 29.307 ms 30.251 ms]
                        thrpt:  [396.68  elem/s 409.46  elem/s 417.06  elem/s]
Found 3 outliers among 20 measurements (15.00%)
  1 (5.00%) low mild
  2 (10.00%) high severe

     Running benches/chains_io.rs (target/release/deps/chains_io-46e99960c1e8dc98)
market/chain_io/save_to_csv/21
                        time:   [30.658 µs 30.791 µs 30.928 µs]
                        thrpt:  [678.99 Kelem/s 682.02 Kelem/s 684.97 Kelem/s]
Found 15 outliers among 100 measurements (15.00%)
  4 (4.00%) high mild
  11 (11.00%) high severe
market/chain_io/load_from_csv/21
                        time:   [22.702 µs 22.710 µs 22.718 µs]
                        thrpt:  [924.40 Kelem/s 924.70 Kelem/s 925.01 Kelem/s]
Found 6 outliers among 100 measurements (6.00%)
  5 (5.00%) high mild
  1 (1.00%) high severe

Warning: Unable to complete 100 samples in 4.0s. You may wish to increase target time to 5.2s, enable flat sampling, or reduce sample count to 60.
market/chain_io/save_to_json/21
                        time:   [1.0158 ms 1.0240 ms 1.0334 ms]
                        thrpt:  [20.321 Kelem/s 20.508 Kelem/s 20.672 Kelem/s]
market/chain_io/load_from_json/21
                        time:   [2.9545 ms 2.9551 ms 2.9557 ms]
                        thrpt:  [7.1049 Kelem/s 7.1063 Kelem/s 7.1077 Kelem/s]
Found 5 outliers among 100 measurements (5.00%)
  5 (5.00%) high mild
market/chain_io/save_to_csv/101
                        time:   [104.50 µs 104.66 µs 104.83 µs]
                        thrpt:  [963.47 Kelem/s 965.06 Kelem/s 966.52 Kelem/s]
Found 15 outliers among 100 measurements (15.00%)
  1 (1.00%) low severe
  2 (2.00%) low mild
  8 (8.00%) high mild
  4 (4.00%) high severe
market/chain_io/load_from_csv/101
                        time:   [71.596 µs 71.724 µs 71.843 µs]
                        thrpt:  [1.4058 Melem/s 1.4082 Melem/s 1.4107 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
market/chain_io/save_to_json/101
                        time:   [4.7877 ms 4.7980 ms 4.8079 ms]
                        thrpt:  [21.007 Kelem/s 21.051 Kelem/s 21.096 Kelem/s]
market/chain_io/load_from_json/101
                        time:   [14.483 ms 14.756 ms 15.096 ms]
                        thrpt:  [6.6903 Kelem/s 6.8448 Kelem/s 6.9739 Kelem/s]
Found 11 outliers among 100 measurements (11.00%)
  11 (11.00%) high severe
market/chain_io/load_from_json/sp500_fixture
                        time:   [3.5900 ms 3.5907 ms 3.5914 ms]
                        thrpt:  [28.123 Kelem/s 28.129 Kelem/s 28.134 Kelem/s]
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) high mild
  1 (1.00%) high severe

market/ohlcv_zip/read_ohlcv_from_zip/all_74061
                        time:   [25.640 ms 25.733 ms 25.890 ms]
Found 1 outliers among 10 measurements (10.00%)
  1 (10.00%) high severe
market/ohlcv_zip/read_ohlcv_from_zip/one_day
                        time:   [22.191 ms 22.253 ms 22.344 ms]

     Running benches/synthetic.rs (target/release/deps/synthetic-3e621b7073e1986c)
market/synthetic/generator_optionchain/10_steps_half_width_10
                        time:   [5.0213 ms 5.2182 ms 5.4486 ms]
                        thrpt:  [1.8353 Kelem/s 1.9164 Kelem/s 1.9915 Kelem/s]
Found 2 outliers among 10 measurements (20.00%)
  1 (10.00%) high mild
  1 (10.00%) high severe
market/synthetic/generator_optionchain/30_steps_half_width_10
                        time:   [11.511 ms 11.541 ms 11.558 ms]
                        thrpt:  [2.5957 Kelem/s 2.5994 Kelem/s 2.6061 Kelem/s]
market/synthetic/generator_optionchain/30_steps_half_width_25
                        time:   [21.303 ms 21.397 ms 21.575 ms]
                        thrpt:  [1.3905 Kelem/s 1.4020 Kelem/s 1.4083 Kelem/s]
Found 2 outliers among 10 measurements (20.00%)
  2 (20.00%) high mild
market/synthetic/generator_optionseries/10_steps_3_expirations
                        time:   [16.236 ms 16.454 ms 16.659 ms]
                        thrpt:  [600.29  elem/s 607.77  elem/s 615.93  elem/s]
market/synthetic/generator_optionseries/30_steps_3_expirations
                        time:   [31.307 ms 33.100 ms 35.917 ms]
                        thrpt:  [835.25  elem/s 906.34  elem/s 958.24  elem/s]
Found 2 outliers among 10 measurements (20.00%)
  1 (10.00%) high mild
  1 (10.00%) high severe

     Running benches/math.rs (target/release/deps/math-81ea8baffb7b7d2d)
math/construction/curve_new/32
                        time:   [397.98 ns 399.88 ns 402.04 ns]
                        thrpt:  [79.594 Melem/s 80.024 Melem/s 80.407 Melem/s]
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) high mild
  1 (1.00%) high severe
math/construction/curve_from_vector/32
                        time:   [716.50 ns 719.43 ns 722.33 ns]
                        thrpt:  [44.301 Melem/s 44.479 Melem/s 44.661 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
math/construction/curve_parametric/32
                        time:   [36.096 µs 36.676 µs 37.315 µs]
                        thrpt:  [857.58 Kelem/s 872.50 Kelem/s 886.53 Kelem/s]
Found 4 outliers among 100 measurements (4.00%)
  4 (4.00%) low mild
math/construction/curve_new/128
                        time:   [1.6181 µs 1.6202 µs 1.6224 µs]
                        thrpt:  [78.895 Melem/s 79.003 Melem/s 79.105 Melem/s]
Found 9 outliers among 100 measurements (9.00%)
  2 (2.00%) low mild
  6 (6.00%) high mild
  1 (1.00%) high severe
math/construction/curve_from_vector/128
                        time:   [2.7770 µs 2.7818 µs 2.7870 µs]
                        thrpt:  [45.928 Melem/s 46.013 Melem/s 46.094 Melem/s]
Found 4 outliers among 100 measurements (4.00%)
  4 (4.00%) high mild
math/construction/curve_parametric/128
                        time:   [89.096 µs 89.674 µs 90.240 µs]
                        thrpt:  [1.4184 Melem/s 1.4274 Melem/s 1.4367 Melem/s]
Found 5 outliers among 100 measurements (5.00%)
  5 (5.00%) low mild
math/construction/curve_new/512
                        time:   [6.8612 µs 6.8660 µs 6.8709 µs]
                        thrpt:  [74.518 Melem/s 74.571 Melem/s 74.622 Melem/s]
Found 5 outliers among 100 measurements (5.00%)
  4 (4.00%) high mild
  1 (1.00%) high severe
math/construction/curve_from_vector/512
                        time:   [11.530 µs 11.772 µs 12.084 µs]
                        thrpt:  [42.369 Melem/s 43.494 Melem/s 44.406 Melem/s]
Found 8 outliers among 100 measurements (8.00%)
  8 (8.00%) high severe
math/construction/curve_parametric/512
                        time:   [162.08 µs 164.10 µs 166.22 µs]
                        thrpt:  [3.0803 Melem/s 3.1201 Melem/s 3.1589 Melem/s]
Found 6 outliers among 100 measurements (6.00%)
  6 (6.00%) high mild
math/construction/curve_new/2048
                        time:   [28.210 µs 28.233 µs 28.255 µs]
                        thrpt:  [72.482 Melem/s 72.539 Melem/s 72.598 Melem/s]
Found 3 outliers among 100 measurements (3.00%)
  1 (1.00%) low mild
  2 (2.00%) high mild
math/construction/curve_from_vector/2048
                        time:   [45.975 µs 46.021 µs 46.067 µs]
                        thrpt:  [44.457 Melem/s 44.502 Melem/s 44.546 Melem/s]
Found 3 outliers among 100 measurements (3.00%)
  1 (1.00%) low severe
  1 (1.00%) low mild
  1 (1.00%) high mild
math/construction/curve_parametric/2048
                        time:   [584.48 µs 588.49 µs 592.55 µs]
                        thrpt:  [3.4562 Melem/s 3.4801 Melem/s 3.5040 Melem/s]
Found 7 outliers among 100 measurements (7.00%)
  4 (4.00%) high mild
  3 (3.00%) high severe
math/construction/surface_from_vector/8x8
                        time:   [2.2223 µs 2.2251 µs 2.2278 µs]
                        thrpt:  [28.727 Melem/s 28.763 Melem/s 28.798 Melem/s]
Found 3 outliers among 100 measurements (3.00%)
  1 (1.00%) low mild
  1 (1.00%) high mild
  1 (1.00%) high severe
math/construction/surface_parametric/8x8
                        time:   [74.016 µs 75.133 µs 76.268 µs]
                        thrpt:  [839.15 Kelem/s 851.83 Kelem/s 864.68 Kelem/s]
Found 6 outliers among 100 measurements (6.00%)
  5 (5.00%) high mild
  1 (1.00%) high severe
math/construction/surface_from_vector/16x16
                        time:   [9.4834 µs 9.4938 µs 9.5047 µs]
                        thrpt:  [26.934 Melem/s 26.965 Melem/s 26.994 Melem/s]
Found 6 outliers among 100 measurements (6.00%)
  1 (1.00%) low mild
  4 (4.00%) high mild
  1 (1.00%) high severe
math/construction/surface_parametric/16x16
                        time:   [135.20 µs 146.50 µs 160.10 µs]
                        thrpt:  [1.5990 Melem/s 1.7474 Melem/s 1.8934 Melem/s]
Found 7 outliers among 100 measurements (7.00%)
  2 (2.00%) high mild
  5 (5.00%) high severe
math/construction/surface_from_vector/32x32
                        time:   [40.054 µs 40.109 µs 40.166 µs]
                        thrpt:  [25.494 Melem/s 25.531 Melem/s 25.566 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) low mild
math/construction/surface_parametric/32x32
                        time:   [436.31 µs 441.36 µs 446.94 µs]
                        thrpt:  [2.2911 Melem/s 2.3201 Melem/s 2.3469 Melem/s]
Found 4 outliers among 100 measurements (4.00%)
  2 (2.00%) high mild
  2 (2.00%) high severe

math/curve_interpolation/linear/32
                        time:   [1.1544 µs 1.1552 µs 1.1560 µs]
Found 9 outliers among 100 measurements (9.00%)
  3 (3.00%) low mild
  3 (3.00%) high mild
  3 (3.00%) high severe
math/curve_interpolation/bilinear/32
                        time:   [1.3538 µs 1.3568 µs 1.3600 µs]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
math/curve_interpolation/cubic/32
                        time:   [1.5305 µs 1.5317 µs 1.5330 µs]
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high mild
math/curve_interpolation/spline/32
                        time:   [17.325 µs 17.371 µs 17.434 µs]
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high severe
math/curve_interpolation/linear/128
                        time:   [3.7351 µs 3.7374 µs 3.7396 µs]
Found 5 outliers among 100 measurements (5.00%)
  2 (2.00%) low severe
  2 (2.00%) high mild
  1 (1.00%) high severe
math/curve_interpolation/bilinear/128
                        time:   [4.0054 µs 4.0080 µs 4.0105 µs]
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) low severe
  1 (1.00%) low mild
math/curve_interpolation/cubic/128
                        time:   [4.3220 µs 4.3258 µs 4.3298 µs]
Found 15 outliers among 100 measurements (15.00%)
  3 (3.00%) low severe
  12 (12.00%) high severe
math/curve_interpolation/spline/128
                        time:   [69.165 µs 69.228 µs 69.304 µs]
Found 3 outliers among 100 measurements (3.00%)
  2 (2.00%) high mild
  1 (1.00%) high severe
math/curve_interpolation/linear/512
                        time:   [14.628 µs 14.638 µs 14.649 µs]
Found 7 outliers among 100 measurements (7.00%)
  5 (5.00%) high mild
  2 (2.00%) high severe
math/curve_interpolation/bilinear/512
                        time:   [15.175 µs 15.188 µs 15.200 µs]
Found 5 outliers among 100 measurements (5.00%)
  2 (2.00%) low severe
  3 (3.00%) high mild
math/curve_interpolation/cubic/512
                        time:   [15.536 µs 15.551 µs 15.565 µs]
Found 4 outliers among 100 measurements (4.00%)
  2 (2.00%) low mild
  1 (1.00%) high mild
  1 (1.00%) high severe
math/curve_interpolation/spline/512
                        time:   [293.08 µs 293.24 µs 293.41 µs]
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high mild
math/curve_interpolation/linear/2048
                        time:   [57.256 µs 57.313 µs 57.380 µs]
Found 4 outliers among 100 measurements (4.00%)
  2 (2.00%) low severe
  1 (1.00%) high mild
  1 (1.00%) high severe
math/curve_interpolation/bilinear/2048
                        time:   [59.234 µs 59.271 µs 59.307 µs]
Found 10 outliers among 100 measurements (10.00%)
  5 (5.00%) low mild
  4 (4.00%) high mild
  1 (1.00%) high severe
math/curve_interpolation/cubic/2048
                        time:   [59.988 µs 60.140 µs 60.372 µs]
Found 23 outliers among 100 measurements (23.00%)
  1 (1.00%) low severe
  22 (22.00%) high severe

Warning: Unable to complete 100 samples in 4.0s. You may wish to increase target time to 6.2s, enable flat sampling, or reduce sample count to 50.
math/curve_interpolation/spline/2048
                        time:   [1.2285 ms 1.2309 ms 1.2350 ms]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high severe
math/curve_interpolation/linear_sweep_100/128
                        time:   [402.12 µs 402.44 µs 402.77 µs]
                        thrpt:  [248.28 Kelem/s 248.48 Kelem/s 248.68 Kelem/s]
Found 4 outliers among 100 measurements (4.00%)
  2 (2.00%) low mild
  2 (2.00%) high mild
math/curve_interpolation/bilinear_sweep_100/128
                        time:   [432.68 µs 433.26 µs 433.84 µs]
                        thrpt:  [230.50 Kelem/s 230.81 Kelem/s 231.12 Kelem/s]
Found 6 outliers among 100 measurements (6.00%)
  2 (2.00%) low mild
  4 (4.00%) high mild
math/curve_interpolation/cubic_sweep_100/128
                        time:   [458.72 µs 459.52 µs 460.32 µs]
                        thrpt:  [217.24 Kelem/s 217.62 Kelem/s 218.00 Kelem/s]
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) low mild
math/curve_interpolation/spline_sweep_100/128
                        time:   [6.9537 ms 6.9659 ms 6.9835 ms]
                        thrpt:  [14.319 Kelem/s 14.356 Kelem/s 14.381 Kelem/s]
Found 4 outliers among 100 measurements (4.00%)
  1 (1.00%) high mild
  3 (3.00%) high severe

math/surface_interpolation/linear/8x8
                        time:   [16.546 µs 16.552 µs 16.558 µs]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
math/surface_interpolation/bilinear/8x8
                        time:   [15.882 µs 15.916 µs 15.977 µs]
Found 18 outliers among 100 measurements (18.00%)
  1 (1.00%) low mild
  13 (13.00%) high mild
  4 (4.00%) high severe
math/surface_interpolation/cubic/8x8
                        time:   [24.468 µs 24.478 µs 24.490 µs]
Found 3 outliers among 100 measurements (3.00%)
  3 (3.00%) high severe
math/surface_interpolation/spline/8x8
                        time:   [9.6005 µs 9.6026 µs 9.6048 µs]
Found 4 outliers among 100 measurements (4.00%)
  1 (1.00%) low mild
  3 (3.00%) high mild
math/surface_interpolation/linear/16x16
                        time:   [67.145 µs 67.174 µs 67.205 µs]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high severe
math/surface_interpolation/bilinear/16x16
                        time:   [63.975 µs 64.035 µs 64.115 µs]
Found 14 outliers among 100 measurements (14.00%)
  6 (6.00%) high mild
  8 (8.00%) high severe
math/surface_interpolation/cubic/16x16
                        time:   [74.103 µs 74.158 µs 74.230 µs]
Found 7 outliers among 100 measurements (7.00%)
  2 (2.00%) high mild
  5 (5.00%) high severe
math/surface_interpolation/spline/16x16
                        time:   [41.628 µs 41.635 µs 41.640 µs]
Found 7 outliers among 100 measurements (7.00%)
  3 (3.00%) low mild
  4 (4.00%) high mild
math/surface_interpolation/linear/32x32
                        time:   [308.92 µs 309.52 µs 310.37 µs]
Found 3 outliers among 100 measurements (3.00%)
  3 (3.00%) high severe
math/surface_interpolation/bilinear/32x32
                        time:   [296.17 µs 296.32 µs 296.51 µs]
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) high mild
  1 (1.00%) high severe
math/surface_interpolation/cubic/32x32
                        time:   [303.42 µs 313.13 µs 325.01 µs]
Found 10 outliers among 100 measurements (10.00%)
  1 (1.00%) low mild
  1 (1.00%) high mild
  8 (8.00%) high severe
math/surface_interpolation/spline/32x32
                        time:   [202.88 µs 203.17 µs 203.45 µs]
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high severe

math/metrics/curve_metrics/128
                        time:   [301.05 µs 301.64 µs 302.62 µs]
                        thrpt:  [422.98 Kelem/s 424.35 Kelem/s 425.17 Kelem/s]
Found 10 outliers among 100 measurements (10.00%)
  2 (2.00%) low severe
  5 (5.00%) low mild
  1 (1.00%) high mild
  2 (2.00%) high severe

Warning: Unable to complete 100 samples in 4.0s. You may wish to increase target time to 6.5s, enable flat sampling, or reduce sample count to 50.
math/metrics/curve_metrics/512
                        time:   [1.2907 ms 1.2910 ms 1.2914 ms]
                        thrpt:  [396.48 Kelem/s 396.59 Kelem/s 396.68 Kelem/s]
Found 5 outliers among 100 measurements (5.00%)
  1 (1.00%) low mild
  3 (3.00%) high mild
  1 (1.00%) high severe
math/metrics/surface_metrics/8x8
                        time:   [111.28 µs 111.31 µs 111.34 µs]
                        thrpt:  [574.83 Kelem/s 574.99 Kelem/s 575.13 Kelem/s]
Found 3 outliers among 100 measurements (3.00%)
  1 (1.00%) low mild
  1 (1.00%) high mild
  1 (1.00%) high severe
math/metrics/surface_metrics/16x16
                        time:   [498.59 µs 498.73 µs 498.86 µs]
                        thrpt:  [513.17 Kelem/s 513.30 Kelem/s 513.45 Kelem/s]
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high mild

math/transformations/curve_translate/512
                        time:   [27.545 µs 27.596 µs 27.642 µs]
Found 3 outliers among 100 measurements (3.00%)
  3 (3.00%) low mild
math/transformations/curve_scale/512
                        time:   [26.593 µs 26.617 µs 26.639 µs]
Found 4 outliers among 100 measurements (4.00%)
  1 (1.00%) low mild
  2 (2.00%) high mild
  1 (1.00%) high severe
math/transformations/curve_extrema/512
                        time:   [4.6437 µs 4.7124 µs 4.8040 µs]
Found 12 outliers among 100 measurements (12.00%)
  12 (12.00%) high severe
math/transformations/curve_measure_under/512
                        time:   [79.441 µs 79.602 µs 79.754 µs]
math/transformations/curve_derivative_at/512
                        time:   [11.064 µs 11.091 µs 11.116 µs]
Found 3 outliers among 100 measurements (3.00%)
  3 (3.00%) high mild
math/transformations/curve_intersect_with/512
                        time:   [9.3554 ms 9.3658 ms 9.3769 ms]
Found 20 outliers among 100 measurements (20.00%)
  18 (18.00%) high mild
  2 (2.00%) high severe
math/transformations/curve_merge_with_add/512
                        time:   [648.08 µs 649.69 µs 651.57 µs]
Found 7 outliers among 100 measurements (7.00%)
  4 (4.00%) low mild
  3 (3.00%) high severe
math/transformations/surface_translate/16x16
                        time:   [20.584 µs 20.609 µs 20.635 µs]
Found 6 outliers among 100 measurements (6.00%)
  2 (2.00%) low mild
  2 (2.00%) high mild
  2 (2.00%) high severe
math/transformations/surface_extrema/16x16
                        time:   [2.1806 µs 2.1853 µs 2.1902 µs]
Found 3 outliers among 100 measurements (3.00%)
  2 (2.00%) high mild
  1 (1.00%) high severe

Warning: Unable to complete 100 samples in 4.0s. You may wish to increase target time to 6.2s, or reduce sample count to 60.
math/transformations/surface_merge_with_add/16x16
                        time:   [62.237 ms 62.364 ms 62.514 ms]
Found 7 outliers among 100 measurements (7.00%)
  1 (1.00%) high mild
  6 (6.00%) high severe

     Running benches/greeks.rs (target/release/deps/greeks-6f81c39aaa83963e)
greeks/analytic/delta   time:   [9.3116 µs 9.3234 µs 9.3400 µs]
Found 21 outliers among 100 measurements (21.00%)
  1 (1.00%) high mild
  20 (20.00%) high severe
greeks/analytic/gamma   time:   [11.847 µs 11.849 µs 11.851 µs]
greeks/analytic/theta   time:   [13.590 µs 13.592 µs 13.594 µs]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) low mild
greeks/analytic/vega    time:   [11.924 µs 11.929 µs 11.934 µs]
Found 4 outliers among 100 measurements (4.00%)
  2 (2.00%) high mild
  2 (2.00%) high severe
greeks/analytic/rho     time:   [9.6061 µs 9.6077 µs 9.6096 µs]
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) high mild
  1 (1.00%) high severe
greeks/analytic/rho_d   time:   [9.3789 µs 9.3804 µs 9.3820 µs]
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) low mild
greeks/analytic/alpha   time:   [25.558 µs 25.583 µs 25.611 µs]
Found 5 outliers among 100 measurements (5.00%)
  5 (5.00%) high severe
greeks/analytic/vanna   time:   [11.916 µs 11.919 µs 11.924 µs]
Found 5 outliers among 100 measurements (5.00%)
  3 (3.00%) high mild
  2 (2.00%) high severe
greeks/analytic/vomma   time:   [12.535 µs 12.917 µs 13.396 µs]
Found 22 outliers among 100 measurements (22.00%)
  4 (4.00%) low mild
  1 (1.00%) high mild
  17 (17.00%) high severe
greeks/analytic/veta    time:   [12.345 µs 12.347 µs 12.349 µs]
Found 4 outliers among 100 measurements (4.00%)
  1 (1.00%) low mild
  3 (3.00%) high mild
greeks/analytic/charm   time:   [12.484 µs 12.497 µs 12.515 µs]
Found 13 outliers among 100 measurements (13.00%)
  5 (5.00%) high mild
  8 (8.00%) high severe
greeks/analytic/color   time:   [12.334 µs 12.342 µs 12.354 µs]
Found 3 outliers among 100 measurements (3.00%)
  1 (1.00%) high mild
  2 (2.00%) high severe
greeks/analytic/greeks_snapshot
                        time:   [17.153 µs 17.174 µs 17.206 µs]
Found 5 outliers among 100 measurements (5.00%)
  1 (1.00%) high mild
  4 (4.00%) high severe
greeks/analytic/position_greeks_snapshot
                        time:   [17.142 µs 17.144 µs 17.146 µs]
Found 3 outliers among 100 measurements (3.00%)
  1 (1.00%) low mild
  1 (1.00%) high mild
  1 (1.00%) high severe

greeks/variants/numerical_delta
                        time:   [36.261 µs 36.346 µs 36.463 µs]
Found 4 outliers among 100 measurements (4.00%)
  1 (1.00%) high mild
  3 (3.00%) high severe
greeks/variants/numerical_gamma
                        time:   [54.400 µs 54.414 µs 54.430 µs]
Found 4 outliers among 100 measurements (4.00%)
  1 (1.00%) low mild
  2 (2.00%) high mild
  1 (1.00%) high severe
greeks/variants/numerical_vega
                        time:   [36.162 µs 36.169 µs 36.176 µs]
Found 4 outliers among 100 measurements (4.00%)
  1 (1.00%) low severe
  1 (1.00%) low mild
  2 (2.00%) high mild
greeks/variants/numerical_theta
                        time:   [36.031 µs 36.047 µs 36.075 µs]
Found 4 outliers among 100 measurements (4.00%)
  2 (2.00%) high mild
  2 (2.00%) high severe
greeks/variants/numerical_rho
                        time:   [36.060 µs 36.067 µs 36.074 µs]
greeks/variants/delta_b76
                        time:   [16.772 µs 16.774 µs 16.776 µs]
Found 6 outliers among 100 measurements (6.00%)
  2 (2.00%) high mild
  4 (4.00%) high severe
greeks/variants/gamma_b76
                        time:   [20.327 µs 20.335 µs 20.346 µs]
Found 5 outliers among 100 measurements (5.00%)
  3 (3.00%) high mild
  2 (2.00%) high severe
greeks/variants/vega_b76
                        time:   [20.385 µs 20.412 µs 20.453 µs]
Found 6 outliers among 100 measurements (6.00%)
  1 (1.00%) high mild
  5 (5.00%) high severe
greeks/variants/delta_gk
                        time:   [16.692 µs 16.694 µs 16.696 µs]
Found 7 outliers among 100 measurements (7.00%)
  1 (1.00%) low mild
  5 (5.00%) high mild
  1 (1.00%) high severe
greeks/variants/gamma_gk
                        time:   [20.116 µs 20.171 µs 20.248 µs]
Found 6 outliers among 100 measurements (6.00%)
  6 (6.00%) high severe
greeks/variants/vega_gk time:   [20.178 µs 20.193 µs 20.209 µs]
Found 5 outliers among 100 measurements (5.00%)
  2 (2.00%) high mild
  3 (3.00%) high severe

greeks/kernels/d1       time:   [7.5073 µs 7.7355 µs 7.9913 µs]
Found 11 outliers among 100 measurements (11.00%)
  1 (1.00%) low mild
  1 (1.00%) high mild
  9 (9.00%) high severe
greeks/kernels/n_pdf    time:   [1.2304 µs 1.2307 µs 1.2311 µs]
Found 5 outliers among 100 measurements (5.00%)
  1 (1.00%) low mild
  2 (2.00%) high mild
  2 (2.00%) high severe
greeks/kernels/big_n_cdf
                        time:   [198.26 ns 198.78 ns 199.54 ns]
Found 4 outliers among 100 measurements (4.00%)
  2 (2.00%) high mild
  2 (2.00%) high severe

greeks/chain/delta/10   time:   [92.372 µs 92.465 µs 92.573 µs]
                        thrpt:  [108.02 Kelem/s 108.15 Kelem/s 108.26 Kelem/s]
Found 16 outliers among 100 measurements (16.00%)
  16 (16.00%) high mild
greeks/chain/greeks_snapshot/10
                        time:   [166.58 µs 166.60 µs 166.61 µs]
                        thrpt:  [60.019 Kelem/s 60.025 Kelem/s 60.032 Kelem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high severe
greeks/chain/delta/50   time:   [497.38 µs 497.85 µs 498.39 µs]
                        thrpt:  [100.32 Kelem/s 100.43 Kelem/s 100.53 Kelem/s]
Found 16 outliers among 100 measurements (16.00%)
  16 (16.00%) high mild

Warning: Unable to complete 100 samples in 4.0s. You may wish to increase target time to 4.7s, enable flat sampling, or reduce sample count to 60.
greeks/chain/greeks_snapshot/50
                        time:   [933.06 µs 935.60 µs 939.14 µs]
                        thrpt:  [53.240 Kelem/s 53.442 Kelem/s 53.587 Kelem/s]
Found 12 outliers among 100 measurements (12.00%)
  4 (4.00%) low mild
  5 (5.00%) high mild
  3 (3.00%) high severe
greeks/chain/delta/200  time:   [2.0608 ms 2.0633 ms 2.0659 ms]
                        thrpt:  [96.808 Kelem/s 96.933 Kelem/s 97.051 Kelem/s]
Found 14 outliers among 100 measurements (14.00%)
  14 (14.00%) high mild
greeks/chain/greeks_snapshot/200
                        time:   [3.8830 ms 3.9706 ms 4.0796 ms]
                        thrpt:  [49.025 Kelem/s 50.370 Kelem/s 51.506 Kelem/s]
Found 16 outliers among 100 measurements (16.00%)
  1 (1.00%) high mild
  15 (15.00%) high severe

     Running benches/pricing.rs (target/release/deps/pricing-11dd1f5bd7ab8e63)
pricing/closed_form/black_scholes
                        time:   [20.603 µs 20.610 µs 20.619 µs]
Found 4 outliers among 100 measurements (4.00%)
  1 (1.00%) high mild
  3 (3.00%) high severe
pricing/closed_form/calculate_price_black_scholes
                        time:   [20.604 µs 20.606 µs 20.609 µs]
Found 3 outliers among 100 measurements (3.00%)
  1 (1.00%) low mild
  1 (1.00%) high mild
  1 (1.00%) high severe
pricing/closed_form/black_76
                        time:   [19.527 µs 19.529 µs 19.532 µs]
Found 3 outliers among 100 measurements (3.00%)
  2 (2.00%) low mild
  1 (1.00%) high severe
pricing/closed_form/garman_kohlhagen
                        time:   [20.670 µs 20.675 µs 20.681 µs]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high severe
pricing/closed_form/barone_adesi_whaley_put
                        time:   [231.57 µs 231.64 µs 231.72 µs]
pricing/closed_form/probability_keep_under_strike
                        time:   [9.6266 µs 9.6278 µs 9.6290 µs]
Found 3 outliers among 100 measurements (3.00%)
  1 (1.00%) low mild
  1 (1.00%) high mild
  1 (1.00%) high severe

Warning: Unable to complete 100 samples in 4.0s. You may wish to increase target time to 4.9s, enable flat sampling, or reduce sample count to 60.
pricing/closed_form/black_scholes_chain/50
                        time:   [969.43 µs 971.21 µs 974.67 µs]
                        thrpt:  [51.300 Kelem/s 51.482 Kelem/s 51.577 Kelem/s]
Found 3 outliers among 100 measurements (3.00%)
  1 (1.00%) high mild
  2 (2.00%) high severe

pricing/binomial/european/10
                        time:   [26.188 µs 26.863 µs 27.700 µs]
                        thrpt:  [361.01 Kelem/s 372.26 Kelem/s 381.86 Kelem/s]
Found 4 outliers among 100 measurements (4.00%)
  1 (1.00%) low mild
  3 (3.00%) high severe
pricing/binomial/american_put/10
                        time:   [51.360 µs 51.370 µs 51.382 µs]
                        thrpt:  [194.62 Kelem/s 194.67 Kelem/s 194.70 Kelem/s]
Found 5 outliers among 100 measurements (5.00%)
  1 (1.00%) low severe
  3 (3.00%) high mild
  1 (1.00%) high severe
pricing/binomial/price_binomial/10
                        time:   [51.529 µs 51.662 µs 51.887 µs]
                        thrpt:  [192.72 Kelem/s 193.56 Kelem/s 194.07 Kelem/s]
Found 7 outliers among 100 measurements (7.00%)
  1 (1.00%) low mild
  2 (2.00%) high mild
  4 (4.00%) high severe
pricing/binomial/european/50
                        time:   [384.43 µs 384.62 µs 384.84 µs]
                        thrpt:  [129.92 Kelem/s 130.00 Kelem/s 130.06 Kelem/s]
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high severe
pricing/binomial/american_put/50
                        time:   [1.6800 ms 1.6828 ms 1.6866 ms]
                        thrpt:  [29.646 Kelem/s 29.712 Kelem/s 29.762 Kelem/s]
Found 7 outliers among 100 measurements (7.00%)
  4 (4.00%) high mild
  3 (3.00%) high severe
pricing/binomial/price_binomial/50
                        time:   [1.6931 ms 1.6934 ms 1.6937 ms]
                        thrpt:  [29.522 Kelem/s 29.527 Kelem/s 29.531 Kelem/s]
Found 3 outliers among 100 measurements (3.00%)
  2 (2.00%) high mild
  1 (1.00%) high severe
pricing/binomial/european/200
                        time:   [5.3370 ms 5.3477 ms 5.3621 ms]
                        thrpt:  [37.299 Kelem/s 37.399 Kelem/s 37.474 Kelem/s]
Found 6 outliers among 100 measurements (6.00%)
  2 (2.00%) high mild
  4 (4.00%) high severe
pricing/binomial/american_put/200
                        time:   [36.714 ms 36.720 ms 36.726 ms]
                        thrpt:  [5.4457 Kelem/s 5.4466 Kelem/s 5.4475 Kelem/s]
Found 11 outliers among 100 measurements (11.00%)
  2 (2.00%) low mild
  8 (8.00%) high mild
  1 (1.00%) high severe
pricing/binomial/price_binomial/200
                        time:   [36.281 ms 36.541 ms 36.888 ms]
                        thrpt:  [5.4219 Kelem/s 5.4734 Kelem/s 5.5125 Kelem/s]
Found 11 outliers among 100 measurements (11.00%)
  3 (3.00%) high mild
  8 (8.00%) high severe

Warning: Unable to complete 100 samples in 4.0s. You may wish to increase target time to 10.9s, or reduce sample count to 30.
pricing/binomial/european/1000
                        time:   [108.60 ms 108.68 ms 108.83 ms]
                        thrpt:  [9.1889 Kelem/s 9.2012 Kelem/s 9.2081 Kelem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high severe

Warning: Unable to complete 100 samples in 4.0s. You may wish to increase target time to 121.1s, or reduce sample count to 10.
pricing/binomial/american_put/1000
                        time:   [1.2116 s 1.2138 s 1.2169 s]
                        thrpt:  [821.78  elem/s 823.85  elem/s 825.37  elem/s]
Found 16 outliers among 100 measurements (16.00%)
  1 (1.00%) low mild
  4 (4.00%) high mild
  11 (11.00%) high severe

Warning: Unable to complete 100 samples in 4.0s. You may wish to increase target time to 124.9s, or reduce sample count to 10.
pricing/binomial/price_binomial/1000
                        time:   [1.2467 s 1.2490 s 1.2522 s]
                        thrpt:  [798.58  elem/s 800.63  elem/s 802.10  elem/s]
Found 13 outliers among 100 measurements (13.00%)
  1 (1.00%) high mild
  12 (12.00%) high severe
pricing/binomial/generate_binomial_tree/10
                        time:   [49.460 µs 49.478 µs 49.501 µs]
                        thrpt:  [202.02 Kelem/s 202.11 Kelem/s 202.18 Kelem/s]
Found 3 outliers among 100 measurements (3.00%)
  1 (1.00%) low mild
  1 (1.00%) high mild
  1 (1.00%) high severe
pricing/binomial/generate_binomial_tree/50
                        time:   [1.6283 ms 1.6285 ms 1.6287 ms]
                        thrpt:  [30.700 Kelem/s 30.704 Kelem/s 30.707 Kelem/s]
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high mild
pricing/binomial/generate_binomial_tree/200
                        time:   [34.891 ms 35.321 ms 35.873 ms]
                        thrpt:  [5.5753 Kelem/s 5.6624 Kelem/s 5.7322 Kelem/s]
Found 11 outliers among 100 measurements (11.00%)
  3 (3.00%) high mild
  8 (8.00%) high severe

pricing/stochastic/monte_carlo/30_steps_x_1000_paths
                        time:   [41.884 ms 41.899 ms 41.910 ms]
                        thrpt:  [715.82 Kelem/s 716.01 Kelem/s 716.26 Kelem/s]

Warning: Unable to complete 10 samples in 4.0s. You may wish to increase target time to 4.2s.
pricing/stochastic/monte_carlo/30_steps_x_10000_paths
                        time:   [418.98 ms 419.94 ms 421.04 ms]
                        thrpt:  [712.52 Kelem/s 714.40 Kelem/s 716.03 Kelem/s]
Found 1 outliers among 10 measurements (10.00%)
  1 (10.00%) high mild
pricing/stochastic/monte_carlo/252_steps_x_1000_paths
                        time:   [355.83 ms 355.98 ms 356.17 ms]
                        thrpt:  [707.53 Kelem/s 707.90 Kelem/s 708.21 Kelem/s]
Found 2 outliers among 10 measurements (20.00%)
  1 (10.00%) high mild
  1 (10.00%) high severe
pricing/stochastic/telegraph/30_steps_x_1000_paths
                        time:   [716.58 µs 718.10 µs 721.59 µs]
                        thrpt:  [41.575 Melem/s 41.777 Melem/s 41.865 Melem/s]
Found 1 outliers among 10 measurements (10.00%)
  1 (10.00%) high severe
pricing/stochastic/telegraph/30_steps_x_10000_paths
                        time:   [7.1097 ms 7.1137 ms 7.1196 ms]
                        thrpt:  [42.137 Melem/s 42.172 Melem/s 42.196 Melem/s]
Found 1 outliers among 10 measurements (10.00%)
  1 (10.00%) low mild
pricing/stochastic/telegraph/252_steps_x_1000_paths
                        time:   [2.2598 ms 2.2639 ms 2.2722 ms]
                        thrpt:  [110.91 Melem/s 111.31 Melem/s 111.51 Melem/s]
Found 1 outliers among 10 measurements (10.00%)
  1 (10.00%) high severe
pricing/stochastic/simulate_returns/252
                        time:   [1.4625 ms 1.4630 ms 1.4637 ms]
                        thrpt:  [172.17 Kelem/s 172.25 Kelem/s 172.31 Kelem/s]
pricing/stochastic/simulate_returns/1008
                        time:   [5.8493 ms 5.9681 ms 6.2301 ms]
                        thrpt:  [161.80 Kelem/s 168.90 Kelem/s 172.33 Kelem/s]
Found 2 outliers among 10 measurements (20.00%)
  2 (20.00%) high severe

pricing/exotic/asian_arithmetic
                        time:   [22.568 µs 22.572 µs 22.577 µs]
Found 5 outliers among 100 measurements (5.00%)
  5 (5.00%) high mild
pricing/exotic/asian_geometric
                        time:   [21.596 µs 21.664 µs 21.801 µs]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high severe
pricing/exotic/barrier_up_and_out
                        time:   [69.036 µs 69.048 µs 69.062 µs]
Found 6 outliers among 100 measurements (6.00%)
  5 (5.00%) high mild
  1 (1.00%) high severe
pricing/exotic/binary_cash_or_nothing
                        time:   [10.678 µs 10.680 µs 10.682 µs]
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high mild
pricing/exotic/lookback_floating
                        time:   [5.0693 µs 5.0709 µs 5.0727 µs]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
pricing/exotic/compound time:   [31.463 µs 31.470 µs 31.479 µs]
Found 6 outliers among 100 measurements (6.00%)
  1 (1.00%) low mild
  3 (3.00%) high mild
  2 (2.00%) high severe
pricing/exotic/chooser  time:   [29.620 µs 29.633 µs 29.647 µs]
Found 7 outliers among 100 measurements (7.00%)
  4 (4.00%) high mild
  3 (3.00%) high severe
pricing/exotic/cliquet  time:   [44.504 µs 44.512 µs 44.520 µs]
Found 3 outliers among 100 measurements (3.00%)
  3 (3.00%) high mild
pricing/exotic/rainbow_best_of
                        time:   [320.57 µs 320.62 µs 320.68 µs]
Found 3 outliers among 100 measurements (3.00%)
  2 (2.00%) high mild
  1 (1.00%) high severe
pricing/exotic/spread   time:   [16.722 µs 16.738 µs 16.756 µs]
Found 9 outliers among 100 measurements (9.00%)
  2 (2.00%) low mild
  3 (3.00%) high mild
  4 (4.00%) high severe
pricing/exotic/quanto   time:   [11.590 µs 11.600 µs 11.613 µs]
Found 10 outliers among 100 measurements (10.00%)
  1 (1.00%) low severe
  2 (2.00%) high mild
  7 (7.00%) high severe
pricing/exotic/exchange time:   [11.288 µs 11.290 µs 11.293 µs]
Found 4 outliers among 100 measurements (4.00%)
  2 (2.00%) high mild
  2 (2.00%) high severe
pricing/exotic/power    time:   [1.0132 µs 1.0153 µs 1.0181 µs]
Found 6 outliers among 100 measurements (6.00%)
  3 (3.00%) high mild
  3 (3.00%) high severe

     Running benches/volatility.rs (target/release/deps/volatility-88e69787032c4c70)
volatility/implied/calculate_implied_volatility/atm
                        time:   [79.964 µs 80.000 µs 80.038 µs]
Found 4 outliers among 100 measurements (4.00%)
  2 (2.00%) low mild
  1 (1.00%) high mild
  1 (1.00%) high severe
volatility/implied/implied_volatility/atm
                        time:   [7.8243 ms 7.8502 ms 7.8803 ms]
Found 9 outliers among 100 measurements (9.00%)
  5 (5.00%) high mild
  4 (4.00%) high severe
volatility/implied/calculate_iv/atm
                        time:   [726.14 µs 766.42 µs 812.14 µs]
Found 7 outliers among 100 measurements (7.00%)
  1 (1.00%) high mild
  6 (6.00%) high severe
volatility/implied/calculate_implied_volatility/otm
                        time:   [293.48 µs 293.57 µs 293.68 µs]
Found 7 outliers among 100 measurements (7.00%)
  1 (1.00%) low mild
  1 (1.00%) high mild
  5 (5.00%) high severe
volatility/implied/implied_volatility/otm
                        time:   [27.624 ms 27.683 ms 27.750 ms]
Found 20 outliers among 100 measurements (20.00%)
  3 (3.00%) low severe
  1 (1.00%) low mild
  3 (3.00%) high mild
  13 (13.00%) high severe
volatility/implied/calculate_iv/otm
                        time:   [2.6987 ms 2.7018 ms 2.7052 ms]
Found 13 outliers among 100 measurements (13.00%)
  1 (1.00%) low severe
  2 (2.00%) low mild
  3 (3.00%) high mild
  7 (7.00%) high severe
volatility/implied/calculate_implied_volatility/deep_otm
                        time:   [249.57 µs 249.61 µs 249.64 µs]
Found 11 outliers among 100 measurements (11.00%)
  2 (2.00%) low severe
  2 (2.00%) low mild
  5 (5.00%) high mild
  2 (2.00%) high severe
volatility/implied/implied_volatility/deep_otm
                        time:   [31.426 ms 31.470 ms 31.518 ms]
Found 19 outliers among 100 measurements (19.00%)
  1 (1.00%) low severe
  3 (3.00%) low mild
  6 (6.00%) high mild
  9 (9.00%) high severe
volatility/implied/calculate_iv/deep_otm
                        time:   [3.0838 ms 3.0944 ms 3.1068 ms]
Found 27 outliers among 100 measurements (27.00%)
  2 (2.00%) low severe
  11 (11.00%) low mild
  6 (6.00%) high mild
  8 (8.00%) high severe
volatility/implied/calculate_implied_volatility_chain/50
                        time:   [15.108 ms 15.110 ms 15.112 ms]
                        thrpt:  [3.3086 Kelem/s 3.3091 Kelem/s 3.3095 Kelem/s]
Found 3 outliers among 100 measurements (3.00%)
  2 (2.00%) high mild
  1 (1.00%) high severe

volatility/estimators/constant/63
                        time:   [8.3210 µs 8.3441 µs 8.3857 µs]
                        thrpt:  [7.5128 Melem/s 7.5502 Melem/s 7.5712 Melem/s]
Found 15 outliers among 100 measurements (15.00%)
  2 (2.00%) high mild
  13 (13.00%) high severe
volatility/estimators/historical_window_21/63
                        time:   [158.26 µs 158.33 µs 158.39 µs]
                        thrpt:  [397.76 Kelem/s 397.91 Kelem/s 398.07 Kelem/s]
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) low mild
  1 (1.00%) high mild
volatility/estimators/ewma/63
                        time:   [91.965 µs 92.173 µs 92.545 µs]
                        thrpt:  [680.75 Kelem/s 683.50 Kelem/s 685.04 Kelem/s]
Found 6 outliers among 100 measurements (6.00%)
  4 (4.00%) high mild
  2 (2.00%) high severe
volatility/estimators/garch/63
                        time:   [94.787 µs 94.842 µs 94.907 µs]
                        thrpt:  [663.81 Kelem/s 664.26 Kelem/s 664.65 Kelem/s]
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) low mild
  1 (1.00%) high severe
volatility/estimators/heston_simulation/63
                        time:   [165.55 µs 165.88 µs 166.34 µs]
                        thrpt:  [378.74 Kelem/s 379.79 Kelem/s 380.56 Kelem/s]
Found 10 outliers among 100 measurements (10.00%)
  1 (1.00%) low severe
  3 (3.00%) low mild
  2 (2.00%) high mild
  4 (4.00%) high severe
volatility/estimators/constant/252
                        time:   [29.366 µs 29.381 µs 29.395 µs]
                        thrpt:  [8.5728 Melem/s 8.5770 Melem/s 8.5815 Melem/s]

Warning: Unable to complete 100 samples in 4.0s. You may wish to increase target time to 4.3s, enable flat sampling, or reduce sample count to 60.
volatility/estimators/historical_window_21/252
                        time:   [852.79 µs 854.60 µs 857.59 µs]
                        thrpt:  [293.85 Kelem/s 294.87 Kelem/s 295.50 Kelem/s]
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high severe
volatility/estimators/ewma/252
                        time:   [465.84 µs 466.13 µs 466.43 µs]
                        thrpt:  [540.27 Kelem/s 540.63 Kelem/s 540.96 Kelem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
volatility/estimators/garch/252
                        time:   [466.52 µs 467.34 µs 468.69 µs]
                        thrpt:  [537.66 Kelem/s 539.22 Kelem/s 540.17 Kelem/s]
Found 5 outliers among 100 measurements (5.00%)
  2 (2.00%) low mild
  1 (1.00%) high mild
  2 (2.00%) high severe
volatility/estimators/heston_simulation/252
                        time:   [664.07 µs 664.53 µs 664.99 µs]
                        thrpt:  [378.95 Kelem/s 379.22 Kelem/s 379.48 Kelem/s]
Found 8 outliers among 100 measurements (8.00%)
  3 (3.00%) low mild
  4 (4.00%) high mild
  1 (1.00%) high severe
volatility/estimators/constant/1008
                        time:   [113.39 µs 113.47 µs 113.55 µs]
                        thrpt:  [8.8771 Melem/s 8.8833 Melem/s 8.8899 Melem/s]
volatility/estimators/historical_window_21/1008
                        time:   [3.6326 ms 3.6338 ms 3.6350 ms]
                        thrpt:  [277.30 Kelem/s 277.39 Kelem/s 277.49 Kelem/s]
volatility/estimators/ewma/1008
                        time:   [1.9027 ms 1.9034 ms 1.9041 ms]
                        thrpt:  [529.38 Kelem/s 529.57 Kelem/s 529.76 Kelem/s]
volatility/estimators/garch/1008
                        time:   [1.6183 ms 1.6193 ms 1.6203 ms]
                        thrpt:  [622.12 Kelem/s 622.50 Kelem/s 622.88 Kelem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
volatility/estimators/heston_simulation/1008
                        time:   [2.6563 ms 2.6595 ms 2.6626 ms]
                        thrpt:  [378.58 Kelem/s 379.02 Kelem/s 379.47 Kelem/s]
Found 4 outliers among 100 measurements (4.00%)
  3 (3.00%) low mild
  1 (1.00%) high mild

volatility/utilities/uncertain_volatility_bounds
                        time:   [34.743 µs 34.801 µs 34.873 µs]
Found 5 outliers among 100 measurements (5.00%)
  1 (1.00%) high mild
  4 (4.00%) high severe
volatility/utilities/adjust_volatility_day_to_year
                        time:   [1.6748 µs 1.6757 µs 1.6765 µs]
Found 4 outliers among 100 measurements (4.00%)
  1 (1.00%) low mild
  1 (1.00%) high mild
  2 (2.00%) high severe

     Running benches/simulation.rs (target/release/deps/simulation-d5455cda21994b9f)
simulation/walk/brownian/252
                        time:   [60.118 µs 60.251 µs 60.402 µs]
                        thrpt:  [4.1721 Melem/s 4.1825 Melem/s 4.1918 Melem/s]
Found 6 outliers among 100 measurements (6.00%)
  4 (4.00%) high mild
  2 (2.00%) high severe
simulation/walk/geometric_brownian/252
                        time:   [440.19 µs 440.48 µs 440.77 µs]
                        thrpt:  [571.72 Kelem/s 572.10 Kelem/s 572.48 Kelem/s]
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high mild
simulation/walk/log_returns/252
                        time:   [442.33 µs 442.57 µs 442.81 µs]
                        thrpt:  [569.09 Kelem/s 569.41 Kelem/s 569.71 Kelem/s]
simulation/walk/mean_reverting/252
                        time:   [87.681 µs 87.785 µs 87.893 µs]
                        thrpt:  [2.8671 Melem/s 2.8707 Melem/s 2.8741 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high severe
simulation/walk/jump_diffusion/252
                        time:   [88.335 µs 88.432 µs 88.548 µs]
                        thrpt:  [2.8459 Melem/s 2.8496 Melem/s 2.8528 Melem/s]
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high severe

Warning: Unable to complete 100 samples in 4.0s. You may wish to increase target time to 4.2s, enable flat sampling, or reduce sample count to 60.
simulation/walk/garch/252
                        time:   [824.17 µs 824.58 µs 824.98 µs]
                        thrpt:  [305.46 Kelem/s 305.61 Kelem/s 305.76 Kelem/s]
Found 3 outliers among 100 measurements (3.00%)
  3 (3.00%) high severe

Warning: Unable to complete 100 samples in 4.0s. You may wish to increase target time to 8.0s, enable flat sampling, or reduce sample count to 40.
simulation/walk/heston/252
                        time:   [1.6160 ms 1.6313 ms 1.6507 ms]
                        thrpt:  [152.66 Kelem/s 154.48 Kelem/s 155.94 Kelem/s]
Found 6 outliers among 100 measurements (6.00%)
  1 (1.00%) high mild
  5 (5.00%) high severe
simulation/walk/custom/252
                        time:   [213.83 µs 214.02 µs 214.21 µs]
                        thrpt:  [1.1764 Melem/s 1.1774 Melem/s 1.1785 Melem/s]
Found 4 outliers among 100 measurements (4.00%)
  4 (4.00%) high mild
simulation/walk/telegraph/252
                        time:   [707.65 µs 708.36 µs 709.32 µs]
                        thrpt:  [355.27 Kelem/s 355.75 Kelem/s 356.11 Kelem/s]
Found 3 outliers among 100 measurements (3.00%)
  1 (1.00%) high mild
  2 (2.00%) high severe
simulation/walk/historical/252
                        time:   [2.9879 ms 2.9884 ms 2.9889 ms]
                        thrpt:  [84.311 Kelem/s 84.325 Kelem/s 84.339 Kelem/s]
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) low mild
  1 (1.00%) high mild
simulation/walk/brownian/1008
                        time:   [344.92 µs 345.30 µs 345.70 µs]
                        thrpt:  [2.9158 Melem/s 2.9192 Melem/s 2.9224 Melem/s]
Found 12 outliers among 100 measurements (12.00%)
  12 (12.00%) high mild
simulation/walk/geometric_brownian/1008
                        time:   [1.8557 ms 1.8579 ms 1.8605 ms]
                        thrpt:  [541.80 Kelem/s 542.55 Kelem/s 543.18 Kelem/s]
Found 7 outliers among 100 measurements (7.00%)
  1 (1.00%) high mild
  6 (6.00%) high severe
simulation/walk/log_returns/1008
                        time:   [1.8794 ms 1.8802 ms 1.8810 ms]
                        thrpt:  [535.88 Kelem/s 536.12 Kelem/s 536.35 Kelem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
simulation/walk/mean_reverting/1008
                        time:   [428.03 µs 431.36 µs 435.79 µs]
                        thrpt:  [2.3131 Melem/s 2.3368 Melem/s 2.3550 Melem/s]
Found 20 outliers among 100 measurements (20.00%)
  2 (2.00%) high mild
  18 (18.00%) high severe
simulation/walk/jump_diffusion/1008
                        time:   [467.71 µs 468.03 µs 468.39 µs]
                        thrpt:  [2.1520 Melem/s 2.1537 Melem/s 2.1552 Melem/s]
Found 7 outliers among 100 measurements (7.00%)
  7 (7.00%) high mild
simulation/walk/garch/1008
                        time:   [3.4164 ms 3.4219 ms 3.4302 ms]
                        thrpt:  [293.86 Kelem/s 294.57 Kelem/s 295.05 Kelem/s]
Found 4 outliers among 100 measurements (4.00%)
  1 (1.00%) high mild
  3 (3.00%) high severe
simulation/walk/heston/1008
                        time:   [6.2917 ms 6.3150 ms 6.3389 ms]
                        thrpt:  [159.02 Kelem/s 159.62 Kelem/s 160.21 Kelem/s]

Warning: Unable to complete 100 samples in 4.0s. You may wish to increase target time to 4.6s, enable flat sampling, or reduce sample count to 60.
simulation/walk/custom/1008
                        time:   [914.49 µs 915.98 µs 918.16 µs]
                        thrpt:  [1.0978 Melem/s 1.1005 Melem/s 1.1022 Melem/s]
Found 6 outliers among 100 measurements (6.00%)
  4 (4.00%) high mild
  2 (2.00%) high severe
simulation/walk/telegraph/1008
                        time:   [2.9290 ms 2.9301 ms 2.9311 ms]
                        thrpt:  [343.89 Kelem/s 344.02 Kelem/s 344.14 Kelem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high severe
simulation/walk/historical/1008
                        time:   [11.995 ms 12.021 ms 12.055 ms]
                        thrpt:  [83.618 Kelem/s 83.852 Kelem/s 84.034 Kelem/s]
Found 9 outliers among 100 measurements (9.00%)
  3 (3.00%) high mild
  6 (6.00%) high severe

simulation/simulator/new/100_paths_x_30_steps
                        time:   [5.5311 ms 5.5384 ms 5.5490 ms]
                        thrpt:  [540.64 Kelem/s 541.67 Kelem/s 542.39 Kelem/s]
Found 2 outliers among 10 measurements (20.00%)
  2 (20.00%) high severe
simulation/simulator/new/1000_paths_x_30_steps
                        time:   [55.335 ms 55.373 ms 55.464 ms]
                        thrpt:  [540.89 Kelem/s 541.78 Kelem/s 542.15 Kelem/s]
Found 1 outliers among 10 measurements (10.00%)
  1 (10.00%) high mild
simulation/simulator/new/100_paths_x_252_steps
                        time:   [46.926 ms 46.928 ms 46.930 ms]
                        thrpt:  [536.97 Kelem/s 536.99 Kelem/s 537.01 Kelem/s]
Found 1 outliers among 10 measurements (10.00%)
  1 (10.00%) high severe
simulation/simulator/get_last_positive_values/1000_paths
                        time:   [2.0544 µs 2.0559 µs 2.0571 µs]
                        thrpt:  [486.13 Melem/s 486.41 Melem/s 486.77 Melem/s]
simulation/simulator/get_mc_option_price/1000_paths
                        time:   [158.73 µs 158.87 µs 159.05 µs]
                        thrpt:  [6.2874 Melem/s 6.2943 Melem/s 6.2998 Melem/s]

simulation/process/generate_ou_process/252
                        time:   [105.99 µs 106.10 µs 106.25 µs]
                        thrpt:  [2.3719 Melem/s 2.3751 Melem/s 2.3777 Melem/s]
Found 4 outliers among 100 measurements (4.00%)
  1 (1.00%) high mild
  3 (3.00%) high severe
simulation/process/expanding_window_vols/252
                        time:   [2.9808 ms 2.9819 ms 2.9834 ms]
                        thrpt:  [84.467 Kelem/s 84.510 Kelem/s 84.541 Kelem/s]
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high severe
simulation/process/generate_ou_process/1008
                        time:   [414.85 µs 415.37 µs 415.97 µs]
                        thrpt:  [2.4232 Melem/s 2.4268 Melem/s 2.4298 Melem/s]
Found 9 outliers among 100 measurements (9.00%)
  2 (2.00%) low severe
  2 (2.00%) low mild
  2 (2.00%) high mild
  3 (3.00%) high severe
simulation/process/expanding_window_vols/1008
                        time:   [12.059 ms 12.247 ms 12.477 ms]
                        thrpt:  [80.787 Kelem/s 82.308 Kelem/s 83.586 Kelem/s]
Found 10 outliers among 100 measurements (10.00%)
  10 (10.00%) high severe

     Running benches/strategies.rs (target/release/deps/strategies-e0c669c61aabdce9)
strategies/construction/long_call/1_leg
                        time:   [748.65 ns 749.26 ns 749.94 ns]
strategies/construction/bull_call_spread/2_legs
                        time:   [1.3134 µs 1.3157 µs 1.3185 µs]
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) high mild
  1 (1.00%) high severe
strategies/construction/short_strangle/2_legs
                        time:   [1.3998 µs 1.4014 µs 1.4036 µs]
Found 7 outliers among 100 measurements (7.00%)
  4 (4.00%) high mild
  3 (3.00%) high severe
strategies/construction/long_butterfly/3_legs
                        time:   [2.2479 µs 2.2491 µs 2.2504 µs]
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) high mild
  1 (1.00%) high severe
strategies/construction/iron_condor/4_legs
                        time:   [1.7109 µs 1.7184 µs 1.7285 µs]
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) high mild
  1 (1.00%) high severe
strategies/construction/iron_butterfly/4_legs
                        time:   [1.6799 µs 1.6849 µs 1.6896 µs]
Found 3 outliers among 100 measurements (3.00%)
  2 (2.00%) low severe
  1 (1.00%) low mild

strategies/evaluation/long_call/get_break_even_points
                        time:   [1.4259 ns 1.4451 ns 1.4652 ns]
strategies/evaluation/long_call/update_break_even_points
                        time:   [181.81 ns 182.48 ns 183.14 ns]
Found 6 outliers among 100 measurements (6.00%)
  4 (4.00%) low severe
  1 (1.00%) low mild
  1 (1.00%) high severe
strategies/evaluation/long_call/calculate_profit_at
                        time:   [118.48 ns 119.22 ns 120.24 ns]
Found 22 outliers among 100 measurements (22.00%)
  22 (22.00%) high severe
strategies/evaluation/long_call/get_max_profit
                        time:   [4.5233 ns 4.5610 ns 4.6059 ns]
Found 20 outliers among 100 measurements (20.00%)
  20 (20.00%) high severe
strategies/evaluation/long_call/get_max_loss
                        time:   [68.447 ns 68.518 ns 68.600 ns]
Found 9 outliers among 100 measurements (9.00%)
  2 (2.00%) low mild
  6 (6.00%) high mild
  1 (1.00%) high severe
strategies/evaluation/long_call/get_total_cost
                        time:   [73.187 ns 73.359 ns 73.580 ns]
Found 21 outliers among 100 measurements (21.00%)
  1 (1.00%) low severe
  13 (13.00%) high mild
  7 (7.00%) high severe
strategies/evaluation/long_call/get_profit_area
                        time:   [42.626 ns 42.646 ns 42.668 ns]
Found 20 outliers among 100 measurements (20.00%)
  16 (16.00%) high mild
  4 (4.00%) high severe
strategies/evaluation/long_call/greeks
                        time:   [13.712 µs 13.730 µs 13.763 µs]
Found 5 outliers among 100 measurements (5.00%)
  2 (2.00%) high mild
  3 (3.00%) high severe
strategies/evaluation/long_call/calculate_profit_sweep_201
                        time:   [27.547 µs 27.568 µs 27.591 µs]
strategies/evaluation/bull_call_spread/get_break_even_points
                        time:   [1.3681 ns 1.3919 ns 1.4167 ns]
strategies/evaluation/bull_call_spread/update_break_even_points
                        time:   [274.39 ns 276.73 ns 278.82 ns]
Found 8 outliers among 100 measurements (8.00%)
  3 (3.00%) low severe
  3 (3.00%) low mild
  1 (1.00%) high mild
  1 (1.00%) high severe
strategies/evaluation/bull_call_spread/calculate_profit_at
                        time:   [271.78 ns 271.89 ns 272.02 ns]
Found 16 outliers among 100 measurements (16.00%)
  1 (1.00%) high mild
  15 (15.00%) high severe
strategies/evaluation/bull_call_spread/get_max_profit
                        time:   [278.96 ns 280.24 ns 281.64 ns]
Found 16 outliers among 100 measurements (16.00%)
  16 (16.00%) high severe
strategies/evaluation/bull_call_spread/get_max_loss
                        time:   [238.56 ns 239.87 ns 241.31 ns]
Found 17 outliers among 100 measurements (17.00%)
  1 (1.00%) high mild
  16 (16.00%) high severe
strategies/evaluation/bull_call_spread/get_total_cost
                        time:   [105.88 ns 106.02 ns 106.18 ns]
Found 12 outliers among 100 measurements (12.00%)
  12 (12.00%) high mild
strategies/evaluation/bull_call_spread/get_profit_area
                        time:   [421.60 ns 422.58 ns 423.63 ns]
Found 16 outliers among 100 measurements (16.00%)
  16 (16.00%) high severe
strategies/evaluation/bull_call_spread/get_profit_ratio
                        time:   [615.63 ns 615.86 ns 616.06 ns]
strategies/evaluation/bull_call_spread/greeks
                        time:   [29.957 µs 29.961 µs 29.965 µs]
Found 3 outliers among 100 measurements (3.00%)
  1 (1.00%) low mild
  2 (2.00%) high mild
strategies/evaluation/bull_call_spread/calculate_profit_sweep_201
                        time:   [54.114 µs 54.136 µs 54.156 µs]
strategies/evaluation/short_strangle/get_break_even_points
                        time:   [1.3689 ns 1.3964 ns 1.4265 ns]
Found 9 outliers among 100 measurements (9.00%)
  4 (4.00%) high mild
  5 (5.00%) high severe
strategies/evaluation/short_strangle/update_break_even_points
                        time:   [371.77 ns 373.00 ns 374.03 ns]
Found 9 outliers among 100 measurements (9.00%)
  1 (1.00%) low severe
  5 (5.00%) low mild
  2 (2.00%) high mild
  1 (1.00%) high severe
strategies/evaluation/short_strangle/calculate_profit_at
                        time:   [221.51 ns 221.67 ns 221.83 ns]
Found 16 outliers among 100 measurements (16.00%)
  16 (16.00%) high mild
strategies/evaluation/short_strangle/get_max_profit
                        time:   [292.98 ns 293.98 ns 295.24 ns]
Found 19 outliers among 100 measurements (19.00%)
  1 (1.00%) high mild
  18 (18.00%) high severe
strategies/evaluation/short_strangle/get_max_loss
                        time:   [4.5233 ns 4.5608 ns 4.6054 ns]
Found 22 outliers among 100 measurements (22.00%)
  1 (1.00%) high mild
  21 (21.00%) high severe
strategies/evaluation/short_strangle/get_total_cost
                        time:   [82.421 ns 82.644 ns 82.956 ns]
Found 3 outliers among 100 measurements (3.00%)
  2 (2.00%) high mild
  1 (1.00%) high severe
strategies/evaluation/short_strangle/get_profit_area
                        time:   [572.12 ns 572.49 ns 572.83 ns]
strategies/evaluation/short_strangle/get_profit_ratio
                        time:   [425.22 ns 425.76 ns 426.45 ns]
Found 16 outliers among 100 measurements (16.00%)
  16 (16.00%) high severe
strategies/evaluation/short_strangle/greeks
                        time:   [32.187 µs 32.192 µs 32.197 µs]
Found 6 outliers among 100 measurements (6.00%)
  1 (1.00%) low mild
  2 (2.00%) high mild
  3 (3.00%) high severe
strategies/evaluation/short_strangle/calculate_profit_sweep_201
                        time:   [50.411 µs 50.448 µs 50.493 µs]
Found 15 outliers among 100 measurements (15.00%)
  6 (6.00%) low mild
  7 (7.00%) high mild
  2 (2.00%) high severe
strategies/evaluation/long_butterfly/get_break_even_points
                        time:   [1.3514 ns 1.3779 ns 1.4056 ns]
strategies/evaluation/long_butterfly/update_break_even_points
                        time:   [1.0101 µs 1.0124 µs 1.0146 µs]
Found 7 outliers among 100 measurements (7.00%)
  7 (7.00%) low mild
strategies/evaluation/long_butterfly/calculate_profit_at
                        time:   [423.69 ns 424.37 ns 425.14 ns]
Found 4 outliers among 100 measurements (4.00%)
  1 (1.00%) high mild
  3 (3.00%) high severe
strategies/evaluation/long_butterfly/get_max_profit
                        time:   [390.27 ns 390.56 ns 390.87 ns]
Found 15 outliers among 100 measurements (15.00%)
  14 (14.00%) high mild
  1 (1.00%) high severe
strategies/evaluation/long_butterfly/get_max_loss
                        time:   [780.00 ns 780.97 ns 782.19 ns]
Found 3 outliers among 100 measurements (3.00%)
  3 (3.00%) high severe
strategies/evaluation/long_butterfly/get_total_cost
                        time:   [170.24 ns 170.49 ns 170.79 ns]
Found 8 outliers among 100 measurements (8.00%)
  1 (1.00%) low mild
  7 (7.00%) high mild
strategies/evaluation/long_butterfly/get_profit_area
                        time:   [523.08 ns 524.56 ns 527.03 ns]
Found 19 outliers among 100 measurements (19.00%)
  17 (17.00%) high mild
  2 (2.00%) high severe
strategies/evaluation/long_butterfly/get_profit_ratio
                        time:   [1.2816 µs 1.2828 µs 1.2840 µs]
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high mild
strategies/evaluation/long_butterfly/greeks
                        time:   [46.990 µs 47.091 µs 47.287 µs]
Found 6 outliers among 100 measurements (6.00%)
  1 (1.00%) high mild
  5 (5.00%) high severe
strategies/evaluation/long_butterfly/calculate_profit_sweep_201
                        time:   [81.343 µs 81.410 µs 81.475 µs]
strategies/evaluation/iron_condor/get_break_even_points
                        time:   [1.3565 ns 1.3819 ns 1.4083 ns]
strategies/evaluation/iron_condor/update_break_even_points
                        time:   [560.34 ns 563.95 ns 567.64 ns]
Found 9 outliers among 100 measurements (9.00%)
  4 (4.00%) low severe
  2 (2.00%) low mild
  3 (3.00%) high mild
strategies/evaluation/iron_condor/calculate_profit_at
                        time:   [443.70 ns 444.18 ns 444.64 ns]
Found 3 outliers among 100 measurements (3.00%)
  3 (3.00%) high mild
strategies/evaluation/iron_condor/get_max_profit
                        time:   [1.3292 µs 1.3304 µs 1.3318 µs]
Found 16 outliers among 100 measurements (16.00%)
  10 (10.00%) high mild
  6 (6.00%) high severe
strategies/evaluation/iron_condor/get_max_loss
                        time:   [983.96 ns 984.58 ns 985.25 ns]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
strategies/evaluation/iron_condor/get_total_cost
                        time:   [205.90 ns 206.31 ns 206.84 ns]
Found 8 outliers among 100 measurements (8.00%)
  1 (1.00%) low mild
  5 (5.00%) high mild
  2 (2.00%) high severe
strategies/evaluation/iron_condor/get_profit_area
                        time:   [1.5367 µs 1.5739 µs 1.6205 µs]
Found 8 outliers among 100 measurements (8.00%)
  8 (8.00%) high severe
strategies/evaluation/iron_condor/get_profit_ratio
                        time:   [2.4259 µs 2.4302 µs 2.4372 µs]
Found 4 outliers among 100 measurements (4.00%)
  1 (1.00%) high mild
  3 (3.00%) high severe
strategies/evaluation/iron_condor/greeks
                        time:   [64.179 µs 64.193 µs 64.207 µs]
Found 3 outliers among 100 measurements (3.00%)
  1 (1.00%) high mild
  2 (2.00%) high severe
strategies/evaluation/iron_condor/calculate_profit_sweep_201
                        time:   [101.07 µs 101.39 µs 101.87 µs]
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high severe
strategies/evaluation/iron_butterfly/get_break_even_points
                        time:   [1.3604 ns 1.3852 ns 1.4116 ns]
strategies/evaluation/iron_butterfly/update_break_even_points
                        time:   [561.28 ns 564.03 ns 566.48 ns]
Found 7 outliers among 100 measurements (7.00%)
  6 (6.00%) low severe
  1 (1.00%) low mild
strategies/evaluation/iron_butterfly/calculate_profit_at
                        time:   [484.94 ns 485.63 ns 486.40 ns]
Found 3 outliers among 100 measurements (3.00%)
  3 (3.00%) high severe
strategies/evaluation/iron_butterfly/get_max_profit
                        time:   [1.3331 µs 1.3342 µs 1.3354 µs]
Found 17 outliers among 100 measurements (17.00%)
  16 (16.00%) high mild
  1 (1.00%) high severe
strategies/evaluation/iron_butterfly/get_max_loss
                        time:   [983.19 ns 985.31 ns 988.30 ns]
Found 5 outliers among 100 measurements (5.00%)
  2 (2.00%) high mild
  3 (3.00%) high severe
strategies/evaluation/iron_butterfly/get_total_cost
                        time:   [208.44 ns 215.35 ns 223.52 ns]
Found 6 outliers among 100 measurements (6.00%)
  6 (6.00%) high severe
strategies/evaluation/iron_butterfly/get_profit_area
                        time:   [1.5249 µs 1.5280 µs 1.5331 µs]
Found 4 outliers among 100 measurements (4.00%)
  1 (1.00%) high mild
  3 (3.00%) high severe
strategies/evaluation/iron_butterfly/get_profit_ratio
                        time:   [2.4226 µs 2.4236 µs 2.4246 µs]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
strategies/evaluation/iron_butterfly/greeks
                        time:   [63.729 µs 63.740 µs 63.749 µs]
strategies/evaluation/iron_butterfly/calculate_profit_sweep_201
                        time:   [103.67 µs 103.74 µs 103.79 µs]

strategies/analysis/iron_condor/probability_of_profit
                        time:   [35.680 µs 35.684 µs 35.687 µs]
Found 1 outliers among 20 measurements (5.00%)
  1 (5.00%) low mild
strategies/analysis/iron_condor/expected_value
                        time:   [114.03 µs 114.19 µs 114.42 µs]
Found 4 outliers among 20 measurements (20.00%)
  2 (10.00%) high mild
  2 (10.00%) high severe
strategies/analysis/iron_condor/analyze_probabilities
                        time:   [152.86 µs 153.03 µs 153.44 µs]
Found 1 outliers among 20 measurements (5.00%)
  1 (5.00%) high severe
strategies/analysis/short_strangle/delta_neutrality
                        time:   [56.350 µs 56.367 µs 56.388 µs]
strategies/analysis/short_strangle/delta_adjustments
                        time:   [56.908 µs 56.951 µs 56.995 µs]
Found 4 outliers among 20 measurements (20.00%)
  2 (10.00%) high mild
  2 (10.00%) high severe

strategies/optimiser/bull_call_spread_best_ratio/sp500_45
                        time:   [2.3575 ms 2.3579 ms 2.3585 ms]
Found 1 outliers among 10 measurements (10.00%)
  1 (10.00%) high severe
strategies/optimiser/bull_call_spread_best_area/sp500_45
                        time:   [2.1889 ms 2.1925 ms 2.1963 ms]
Found 2 outliers among 10 measurements (20.00%)
  2 (20.00%) high mild
strategies/optimiser/short_strangle_best_area/sp500_45
                        time:   [3.2489 ms 3.2517 ms 3.2588 ms]
Found 2 outliers among 10 measurements (20.00%)
  1 (10.00%) low severe
  1 (10.00%) high severe

Warning: Unable to complete 10 samples in 4.0s. You may wish to increase target time to 5.4s or enable flat sampling.
strategies/optimiser/long_butterfly_best_ratio/sp500_45
                        time:   [99.006 ms 99.017 ms 99.024 ms]
Found 2 outliers among 10 measurements (20.00%)
  1 (10.00%) high mild
  1 (10.00%) high severe

Warning: Unable to complete 10 samples in 4.0s. You may wish to increase target time to 9.4s.
strategies/optimiser/iron_condor_best_ratio/sp500_45
                        time:   [934.47 ms 934.76 ms 935.00 ms]
Found 1 outliers among 10 measurements (10.00%)
  1 (10.00%) low mild
strategies/optimiser/bull_call_spread_best_ratio/synthetic_11
                        time:   [148.82 µs 149.42 µs 150.19 µs]
                        thrpt:  [73.239 Kelem/s 73.620 Kelem/s 73.914 Kelem/s]
Found 3 outliers among 10 measurements (30.00%)
  1 (10.00%) low mild
  2 (20.00%) high severe
strategies/optimiser/iron_condor_best_ratio/synthetic_11
                        time:   [2.3813 ms 2.4936 ms 2.6605 ms]
                        thrpt:  [4.1345 Kelem/s 4.4113 Kelem/s 4.6193 Kelem/s]
Found 2 outliers among 10 measurements (20.00%)
  1 (10.00%) low mild
  1 (10.00%) high severe
strategies/optimiser/bull_call_spread_best_ratio/synthetic_21
                        time:   [563.50 µs 564.45 µs 564.90 µs]
                        thrpt:  [37.175 Kelem/s 37.204 Kelem/s 37.267 Kelem/s]
strategies/optimiser/iron_condor_best_ratio/synthetic_21
                        time:   [41.643 ms 41.661 ms 41.671 ms]
                        thrpt:  [503.95  elem/s 504.07  elem/s 504.28  elem/s]
Found 1 outliers among 10 measurements (10.00%)
  1 (10.00%) low mild
strategies/optimiser/bull_call_spread_best_ratio/synthetic_41
                        time:   [2.2444 ms 2.2464 ms 2.2493 ms]
                        thrpt:  [18.228 Kelem/s 18.252 Kelem/s 18.268 Kelem/s]

Warning: Unable to complete 10 samples in 4.0s. You may wish to increase target time to 6.9s.
strategies/optimiser/iron_condor_best_ratio/synthetic_41
                        time:   [681.39 ms 681.80 ms 682.19 ms]
                        thrpt:  [60.100  elem/s 60.135  elem/s 60.171  elem/s]
Found 2 outliers among 10 measurements (20.00%)
  1 (10.00%) low severe
  1 (10.00%) high severe

     Running benches/visualization.rs (target/release/deps/visualization-4e8d2c77fb4aefca)
visualization/geometry/curve_graph_data/128
                        time:   [1.0894 µs 1.0911 µs 1.0928 µs]
                        thrpt:  [117.13 Melem/s 117.31 Melem/s 117.49 Melem/s]
Found 3 outliers among 100 measurements (3.00%)
  1 (1.00%) low severe
  2 (2.00%) low mild
visualization/geometry/curve_graph_data/2048
                        time:   [17.577 µs 17.599 µs 17.619 µs]
                        thrpt:  [116.24 Melem/s 116.37 Melem/s 116.52 Melem/s]
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) low mild
  1 (1.00%) high mild
visualization/geometry/curves_graph_data/5x128
                        time:   [6.2825 µs 6.2939 µs 6.3055 µs]
                        thrpt:  [101.50 Melem/s 101.69 Melem/s 101.87 Melem/s]
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) low mild
  1 (1.00%) high mild
visualization/geometry/surface_graph_data/16x16
                        time:   [1.9167 µs 1.9206 µs 1.9244 µs]
                        thrpt:  [133.03 Melem/s 133.29 Melem/s 133.56 Melem/s]
Found 6 outliers among 100 measurements (6.00%)
  3 (3.00%) low mild
  3 (3.00%) high mild
visualization/geometry/surface_graph_data/64x64
                        time:   [29.858 µs 30.005 µs 30.219 µs]
                        thrpt:  [135.54 Melem/s 136.51 Melem/s 137.18 Melem/s]
Found 4 outliers among 100 measurements (4.00%)
  2 (2.00%) high mild
  2 (2.00%) high severe

visualization/payoff/options_graph_data
                        time:   [13.373 µs 13.376 µs 13.379 µs]
visualization/payoff/position_graph_data
                        time:   [13.370 µs 13.387 µs 13.423 µs]
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high severe
visualization/payoff/long_call_graph_data
                        time:   [3.8690 µs 3.8704 µs 3.8722 µs]
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) high mild
  1 (1.00%) high severe
visualization/payoff/bull_call_spread_graph_data
                        time:   [216.67 µs 216.72 µs 216.76 µs]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
visualization/payoff/iron_condor_graph_data
                        time:   [601.41 µs 601.76 µs 602.16 µs]
Found 3 outliers among 100 measurements (3.00%)
  3 (3.00%) high severe

visualization/simulation/simulator_graph_data/10_paths_x_252
                        time:   [2.7554 µs 2.7604 µs 2.7662 µs]
                        thrpt:  [910.99 Melem/s 912.93 Melem/s 914.56 Melem/s]
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) high mild
  1 (1.00%) high severe
visualization/simulation/simulator_graph_data/100_paths_x_252
                        time:   [28.458 µs 28.501 µs 28.552 µs]
                        thrpt:  [882.61 Melem/s 884.19 Melem/s 885.50 Melem/s]
Found 19 outliers among 100 measurements (19.00%)
  2 (2.00%) high mild
  17 (17.00%) high severe
visualization/simulation/random_walk_graph_data/1008
                        time:   [215.28 ns 215.68 ns 216.15 ns]
                        thrpt:  [4.6633 Gelem/s 4.6736 Gelem/s 4.6822 Gelem/s]
Found 18 outliers among 100 measurements (18.00%)
  1 (1.00%) high mild
  17 (17.00%) high severe

visualization/terminal/chain_render_table/21
                        time:   [86.163 µs 86.286 µs 86.411 µs]
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high mild
visualization/terminal/simulation_render_summary/100
                        time:   [22.523 µs 22.566 µs 22.610 µs]
Found 10 outliers among 100 measurements (10.00%)
  2 (2.00%) low mild
  8 (8.00%) high mild
visualization/terminal/simulation_render_individual_results/100
                        time:   [227.57 µs 228.02 µs 228.46 µs]
Found 4 outliers among 100 measurements (4.00%)
  4 (4.00%) high mild
```

</details>
