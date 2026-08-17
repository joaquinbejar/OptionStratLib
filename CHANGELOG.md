# Changelog

All notable changes to **OptionStratLib** are documented in this file.

The format is based on [Keep a Changelog 1.1.0](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.19.0] - 2026-08-17

Dependency refresh: every dependency moved to its latest stable minor, and the
three in-house crates jumped a breaking release each. It is **breaking** for
consumers — see *Migration*.

### Changed — breaking

- `positive` `0.5` -> `0.6`, `expiration_date` `0.2` -> `0.3`,
  `option_type` `0.1` -> `0.3`. All three appear throughout this crate's public
  API, so consumers must move in the same step.
- **JSON wire format**: `positive` 0.6 serialises `Positive` as the exact
  decimal in a *string* (`"42.5"` instead of `42.5`), so every serialised type
  carrying a `Positive` changes shape: `OptionData`, `OptionChainBuildParams`,
  `OptionSeries`, `PnL`/`PnLMetricsDocument`, `Step`/`Xstep`/`Ystep`,
  `StrategyRequest`, and the rest. Deserialisation still accepts the old
  numeric form, so stored documents keep loading; anything asserting on the
  serialised text has to be updated.
- **`utils::others::calculate_log_returns` returns `Vec<Decimal>`**, not
  `Vec<Positive>`. A log return is signed — `ln(105/110) < 0` — and the old
  signature could not represent it. `Positive::ln` itself now returns
  `Decimal` for the same reason.
- **Exotic payloads are `Positive`**, following `option_type` 0.3:
  `OptionType::Barrier { barrier_level, rebate }`,
  `Bermuda { exercise_dates }`, `Chooser { choice_date }`,
  `Cliquet { reset_dates }`, `Spread`/`Exchange { second_asset }`,
  `Quanto { exchange_rate }`, `Power { exponent }`. A negative or non-finite
  value is now unrepresentable rather than an error at pricing time, so
  `power_black_scholes` no longer has a negative-exponent rejection path and
  `barrier_black_scholes` no longer returns `PricingError::NonFinite` for its
  barrier level or rebate.
- `OptionType` and every sub-enum (`AsianAveragingType`, `BarrierType`,
  `BinaryType`, `LookbackType`, `RainbowType`) are `#[non_exhaustive]`
  upstream. Matches in this crate gained fallback arms: payoffs degrade to the
  plain intrinsic value, and the Black-Scholes kernels return
  `PricingError::UnsupportedOptionType` / `PricingError::other` rather than
  failing to compile against a future variant.

### Changed

- `rust_decimal` `1.41` -> `1.42`, `itertools` `0.14` -> `0.15`,
  `zip` `6.0` -> `8.6`, `uuid` `1.23` -> `1.24`, `utoipa` `5.4` -> `5.5`,
  `tokio` `1.52` -> `1.53`; dev-only `mockall` `0.14` -> `0.15`,
  `tempfile` `3.23` -> `3.27`, `proptest` `1.5` -> `1.11`. Every other
  dependency was already at its latest stable minor.
- `Positive::INFINITY` is deprecated upstream in favour of `Positive::MAX`
  (the value was always `Decimal::MAX`, never an infinity). The unlimited-upside
  strategies (`long_call`, `short_call`, the straddles and strangles,
  `call_butterfly`) and `ProfitRange`/`Position` now use `MAX`, so an unbounded
  max-profit renders as `79228162514264337593543950335` instead of `inf`.
- Deprecated conversions replaced: `Positive::to_i64`/`to_u64`/`to_usize` gave
  way to their `*_checked` forms, so an out-of-range day count, plot bound or
  chain-size counter saturates instead of panicking.
- `Positive::sub_or_zero` is deprecated upstream; the zero floor is now taken
  once, in `model::utils::sub_floor_zero`, on top of the checked
  `sub_or_none`. Behaviour is unchanged for the bid/ask spread, the skew scan
  and the OU drift term.
- Comparisons of the form `decimal > Positive::ZERO.into()` became
  `decimal > Positive::ZERO`: `positive` 0.6 adds
  `PartialOrd<Positive> for Decimal`, which made the inferred `.into()`
  ambiguous.

### Migration

```rust
// log returns are signed
- let r: Vec<Positive> = calculate_log_returns(&prices)?;
+ let r: Vec<Decimal>  = calculate_log_returns(&prices)?;

// exotic payloads carry `Positive`
- OptionType::Barrier { barrier_type, barrier_level: 95.0, rebate: None }
+ OptionType::Barrier { barrier_type, barrier_level: pos_or_panic!(95.0), rebate: None }

// unbounded profit
- Ok(Positive::INFINITY)
+ Ok(Positive::MAX)

// serialised prices are strings
- {"strike_price": 100.0}
+ {"strike_price": "100"}
```

### Housekeeping

- `.cargo/audit.toml`, mirroring the `positive` crate's policy file: it ignores
  RUSTSEC-2026-0235 (rkyv 0.7.46) with a reachability rationale — `rkyv` is an
  *optional* dependency of `rust_decimal` that this crate never enables, so it
  is recorded in `Cargo.lock` but never compiled (`cargo tree --all-features
  --target all -i rkyv` reports nothing) — and opts into reporting
  unmaintained/unsound/notice advisories, of which two remain outstanding
  transitively (`number_prefix` via `indicatif`, `rustls-pemfile` via
  `reqwest`). The Security Audit workflow had been failing on every run,
  including on `main`.

## [0.18.1] - 2026-08-07

### Changed

- `statrs` bumped `0.18` -> `0.19` (pulling `nalgebra 0.35` / `simba 0.10.2`), which
  drops the unmaintained `paste` crate (RUSTSEC-2024-0436) from the dependency graph
  entirely — `cargo tree -i paste --all-features` now prints nothing. The `statrs`
  surface this crate uses (`distribution::{Normal, ContinuousCDF}`) is unchanged
  between the two lines, so no code changes. Requested by the Layer V fleet, where
  this chain was the only path bringing `paste` into every downstream tree (#420,
  Layer-V/common-rs#266). (#420)

## [0.18.0] - 2026-07-12

Overhaul of the option-chain walk generators (issues #404-#411, PRs
#412-#419): one shared, error-generic walk driver; unified generator
contracts; per-step stochastic volatility propagated into rebuilt
chains; smile-preserving rebuilds; rayon-parallel per-step builds
(25-step chain walk: 12.46 ms -> 0.88 ms, ~14x); and deterministic
multi-step behavioral test coverage.

### Performance

**Parallel per-step chain builds** (#411): new
`simulation::walk_steps_par` — a parallel variant of `walk_steps` with an
identical contract that fans the per-step y-value construction out to the
rayon thread pool (the walk itself, per-step volatilities and x-step
sequence stay serial and deterministic). `generator_optionchain` and
`generator_optionseries` now use it; output is identical to the serial
driver for the same inputs. Criterion (Apple Silicon):
`generator_optionchain` 10 steps 2.59 ms → 0.47 ms (−82%), 25 steps
7.01 ms → 0.88 ms (−87%).

### Testing

**Behavioral test coverage for the walk generators** (#410): a
deterministic ramp walker (`simulation::walk_test_support`, test-only)
replaces RNG-driven size-1 smoke tests. New multi-step tests pin: exact
price propagation, y-index increments, per-step time-to-expiry decay,
rebuilt-chain expiration tracking, ATM IV tracking of the walk
volatility, Historical walks replaying the provided prices with the
expanding-window estimate, truncation exactly at expiration, series
aging (and the walk stopping once every series expiration has passed),
empty-walker outputs and `size = 0` across all three generators.

### Fixed

**IV smile preserved across chain rebuilds** (#409):

- `OptionChain::to_build_params` now fits `skew_slope` / `smile_curve`
  from the chain's own per-strike IVs by least squares (the exact inverse
  of the parametric model `build_chain` uses), instead of resetting them
  to the `SKEW_SLOPE` / `SKEW_SMILE_CURVE` constants. Round-tripping a
  real market chain previously flattened a ~7 vol-point smile to under
  0.1 vol-points; the smile shape now survives rebuilds (and therefore
  survives the whole simulated walk in `generator_optionchain`). The
  constants remain as fallback when the fit is underdetermined.
- `adjust_volatility` now caps adjusted per-strike IVs at 200% instead of
  100%, so legitimate high-vol wings survive; the cap logs at `debug!`
  when it engages.

### Added

**Per-step volatility paths** (#408):

- `simulation::WalkPath` — walker output carrying `prices` plus optional
  per-step ANNUALIZED `vols`.
- `WalkTypeAble::generate_with_vol()` plus `garch_with_vol` /
  `heston_with_vol` / `custom_with_vol` / `telegraph_with_vol` provided
  methods. The stochastic-volatility models already simulated a vol path
  internally and discarded it; it is now returned. The built-in dynamics
  live in public kernels (`garch_walk`, `heston_walk`, `custom_walk`,
  `telegraph_walk`) shared by both method families; the price-path
  methods remain standalone override points (overriding one method never
  changes the other's default). Implementors overriding a price-path
  method must also override the `*_with_vol` sibling (wrapping their own
  dynamics or composing the public kernel) for the walk generators —
  which consume `generate_with_vol` — to see their dynamics.

### Changed

**Chains/series rebuilt with per-step volatility** (#408):

- Under `Garch` / `Heston` / `Custom` / `Telegraph` walks,
  `generator_optionchain` and `generator_optionseries` now stamp each
  rebuilt chain with the simulated volatility prevailing at that step
  instead of freezing the walk's initial volatility for the whole walk.
- `Historical` walks now use an expanding-window volatility estimate that
  only uses prices up to each step (the previous full-sample estimate had
  look-ahead bias). The estimate at the final step matches the old
  full-sample value.
- Per-step IVs are capped at 100% before stamping a chain (`build_chain`
  rejects IV > 1; simulated vol paths can spike above it).

**Single generic walk driver** (#407):

- `simulation::walk_steps` — the shared dispatch/advance/build loop behind
  all step generators; custom generators can now be written as a closure
  over it instead of forking a 100-line function.
- `WalkType::volatility()` — accessor for the variant's volatility
  parameter (`None` for `Historical`).
- `WalkTypeAble::generate()` — provided method dispatching to the walk
  method matching `params.walk_type`; adding a new `WalkType` variant now
  requires touching only the enum and the trait, not every generator.

### Changed

**`generator_positive` relocated** (#407): it never depended on option
chains, so it moved from `chains::` to `simulation::`. The old path
`chains::generator_positive` remains as a deprecated re-export; the
prelude now re-exports the new location (no deprecation warnings for
prelude users).

**Unified walk-generator contracts** (#406) — the three walk generators
(`chains::generator_optionchain`, `chains::generator_positive`,
`series::generator_optionseries`) now share one documented contract.
Behavior changes observable from the public API:

- `WalkType::Historical` with fewer prices than `WalkParams::size` now
  returns `ChainError::Simulation(InsufficientHistoricalData)` from ALL
  three generators. Previously `generator_optionchain` and
  `generator_optionseries` silently returned a 1-step walk that was
  indistinguishable from a legitimate size-1 walk.
- `generator_positive` no longer panics when a custom walker returns an
  empty vector; it returns the initial step only, like the other two.
- Walks longer than `WalkParams::size` are now truncated at runtime by
  all three generators (previously chains generators only checked this
  with a `debug_assert!`, a no-op in release builds).
- A step-advance failure other than reaching expiration is now
  propagated as an error instead of silently truncating the walk.
- `generator_optionseries` now ages the series along the walk: each
  step's series expirations are reduced by the elapsed walk time and
  expired entries are dropped (previously rebuilt series kept their
  original expirations for the whole walk). The walk ends early once
  every expiration has passed.
- The undocumented `0.20` volatility fallback in
  `generator_optionseries` was removed (dead code under the unified
  contract).

## [0.17.2] - 2026-04-26

Release adding two new closed-form pricing models:
- **Black-76** (Black 1976) for European options on futures and forwards.
- **Garman–Kohlhagen** (1983) for European FX options.

`0.17.0` and `0.17.1` were preparatory iterations of this work
(`0.17.0` was never published; `0.17.1` shipped to crates.io with a
partial subset). `0.17.2` is the first version that ships both models
together. `PricingEngine` is `#[non_exhaustive]` (semver-major from the
0.16.x line) and the two new variants are appended at the tail of the
enum so existing discriminants are preserved.

### Added

**Black-76 model** (Black 1976):
- `pricing::black_76`: closed-form `black_76(option) -> Result<Decimal, PricingError>`
  for European options on futures / forwards. Reuses the existing `d1`
  / `d2` / `big_n` helpers; `Decimal` end-to-end via `d_mul` / `d_sub`;
  `tracing::instrument` on the entry point. Only `OptionType::European`
  is supported — American, Bermuda and exotics return
  `PricingError::UnsupportedOptionType`.
- `pricing::Black76` trait with default `calculate_price_black_76`
  (mirrors `BlackScholes`).
- `pricing::PricingEngine::ClosedFormBlack76` variant + dispatch from
  `price_option`.
- `greeks::utils::calculate_d_values_black_76` `pub(crate)` helper.
- `examples/examples_pricing/src/bin/black_76.rs`: runnable demo
  (Hull canonical example, ITM commodity-futures call, unified-API
  dispatch, short-side sign convention).

**Garman–Kohlhagen model** (Garman & Kohlhagen 1983):
- `pricing::garman_kohlhagen`: closed-form
  `garman_kohlhagen(option) -> Result<Decimal, PricingError>` for
  European options on FX spot rates. Structurally identical to BSM
  with `q = r_f`; the implementation delegates to `black_scholes`
  after type validation, guaranteeing bit-exact equivalence (verified
  to `1e-9` in the tests).
- `pricing::GarmanKohlhagen` trait with default
  `calculate_price_garman_kohlhagen` (mirrors the `BlackScholes`
  trait pattern).
- `pricing::PricingEngine::ClosedFormGK` variant + dispatch from
  `price_option`.
- `examples/examples_pricing/src/bin/garman_kohlhagen.rs`: runnable
  demo (Hull canonical USD/GBP, ITM EUR/USD with FX parity check,
  unified-API dispatch, symmetric-rate degenerate case).

**Infrastructure updates**:
- `examples/examples_pricing/`: new workspace member with binaries for
  both models.
- `lib.rs` mermaid: `Forward-Priced` subgraph routing
  `black_76 -> {Future, Forward}`; new `FX / Currency` subgraph routing
  `garman_kohlhagen -> FX Spot`.

### Changed

- `pricing::PricingEngine` is now `#[non_exhaustive]` so future engine
  variants do not require a new major bump.
- `pricing::mod.rs` Core Models / Model Selection Guidelines /
  Performance Considerations now include both Black-76 and
  Garman–Kohlhagen with explicit field mapping documentation.
- `financial_types` bumped to `0.2.2` (adds `UnderlyingAssetType::Future`
  and `UnderlyingAssetType::Forward`).
- `PricingError` and `GreeksError` pass-through in closed-form dispatch
  (BS, Black-76, GK) for full error-variant fidelity.

## [0.16.5] - 2026-04-20

Documentation-only release. Refresh the crate-level rustdoc and
mermaid diagrams so they describe the 0.16.x quality discipline
(checked arithmetic, `NonFinite` guards, `NonZeroUsize` step counts,
`deny(indexing_slicing)` / `deny(missing_docs)`, structured tracing,
deterministic RNG, pricing-identity regression tests) and the
post-migration example layout.

### Changed

- `src/lib.rs`: new "Quality & Discipline (0.16.x)" section with the
  full list of crate-wide invariants; new **Arithmetic-Error Cascade**
  mermaid diagram (`d_add` / `d_sum_iter` / `finite_decimal` →
  `DecimalError::Overflow` / `PricingError::NonFinite` / …); new
  **Observability** diagram showing the five instrumented public hot
  paths.
- Testing section updated to the current count (3760 unit + 205
  doctest) and mentions the seeded-RNG helper and the pricing-identity
  regression tests.
- Examples section lists every sub-crate under `examples/` and the
  correct `--manifest-path=` invocation (with a note about the
  demo-friendly hourly grid on simulation-heavy examples).
- `README.tpl` passthrough regenerates `README.md` with the updated
  module docs.

[Unreleased]: https://github.com/joaquinbejar/OptionStratLib/compare/v0.16.5...HEAD
[0.16.5]: https://github.com/joaquinbejar/OptionStratLib/releases/tag/v0.16.5

## [0.16.4] - 2026-04-20

### Changed

- Bump workspace dependencies: `rust_decimal` 1.40 → 1.41,
  `rayon` 1.11 → 1.12, `uuid` 1.19 → 1.23, `tokio` 1.43 → 1.52.

### Fixed

- Repair three doctests broken by the `NonZeroUsize` migration
  in 0.16.0: `pricing` module-level examples for `telegraph` and
  `monte_carlo_option_pricing` now wrap literal step / simulation
  counts with `nz!(..)`; the `utils::deterministic_rng` doctest
  uses `rand::RngExt` for `random::<u64>()`.

[0.16.4]: https://github.com/joaquinbejar/OptionStratLib/releases/tag/v0.16.4

## [0.16.3] - 2026-04-20

Hot-fix targeting the runnable-example audit.

### Fixed

- Simulation-heavy demo binaries
  (`long_call_strategy_simulation`, `short_put_strategy_simulation`,
  `position_simulator`, `strategy_simulator`, `random_walk_chain`)
  now use an hourly grid over the week instead of a minute-level
  grid (10 080 steps × 100 simulations, 43 200 for the chain
  walker). The code paths are exercised identically; the demos
  just run in a few seconds in debug mode rather than the minutes
  the example runner timed out on. (#385, #386)
- `examples_volatility::test` brute-force scan cut from
  1 000 000 to 10 000 iterations — the example is a demo, not a
  local benchmark. (#386)

[0.16.3]: https://github.com/joaquinbejar/OptionStratLib/releases/tag/v0.16.3

## [0.16.2] - 2026-04-19

Hot-fix for two panic / I/O bugs caught while running every example
binary under `examples/`.

### Fixed

- Strategy P&L / break-even arithmetic crossed the `Positive`
  boundary without a guard and panicked mid-optimizer-scan
  (`Positive invariant broken in add_decimal / sub`) in:
  - `CallButterfly::update_break_even_points`,
  - `CallButterfly::get_profit_area`,
  - `LongButterflySpread::update_break_even_points`,
  - `BullPutSpread::get_max_loss`.
  All four sites now lower to `Decimal`, then rewrap via
  `Positive::new_decimal(..)` — invalid candidates are dropped
  cleanly or surfaced as typed `StrategyError` instead of
  panicking. Unblocks `strategy_call_butterfly_best_{area,ratio}`,
  `strategy_long_butterfly_spread_best_{area,ratio}`,
  `strategy_call_butterfly_delta`, and
  `strategy_bull_put_spread_extended_delta` examples. (#387)
- `examples_chain::async_chain_ops` was passing a filename where a
  directory was expected and failing with `ENOENT`; it now writes
  under `std::env::temp_dir()/optionstratlib-async-chain-ops` and
  creates the directory up front. (#388)
- `examples_chain::creator` pointed at a Germany-40 JSON file that
  was never committed; now reads the one that ships in
  `examples/Chains/`. (#388)

[0.16.2]: https://github.com/joaquinbejar/OptionStratLib/releases/tag/v0.16.2

## [0.16.1] - 2026-04-19

Hot-fix for CI flakiness introduced by sub-day `ExpirationDate`
arithmetic in test fixtures, plus a doc-link warning.

### Fixed

- Chain test fixtures (`create_test_option_chain`) now use
  `get_x_days_formatted(30)` instead of `get_tomorrow_formatted()`.
  `Actual365Fixed::day_count` in `expiration_date 0.2.0` truncates
  to integer days, so tomorrow's fixed 18:30 UTC expiry evaluated
  after that time collapsed to `t = 0` and broke every
  Black-Scholes-driven axis on the chain curve/surface tests
  (`test_curve_multiple_axes`, `test_curve_price_short_put`,
  `test_surface_different_greeks`, `test_vanna_surface`). 30 days
  puts every test well above the integer-truncation boundary.
- `constants.rs`: `MAX_NEWTON_ITER` no longer links to the private
  `MAX_ITERATIONS_IV` — the doc just names the crate-private
  counterpart in prose, so `cargo doc` emits zero warnings again.

[0.16.1]: https://github.com/joaquinbejar/OptionStratLib/releases/tag/v0.16.1

## [0.16.0] - 2026-04-19

Breaking release. Focus: panic-free core, arithmetic discipline,
typed errors everywhere, and a crate-wide discipline pass over
attributes, docs, and test hygiene.

### Added

- Checked `Decimal` helpers `d_add` / `d_sub` / `d_mul` / `d_div`
  plus `d_sum` and the iterator-based `d_sum_iter` in
  `src/model/decimal.rs`. Every monetary-path kernel now routes
  through them instead of raw `+ - * /`, surfacing `DecimalError::Overflow`
  with an operation tag. (#335, #336, #337, #338, #372)
- Domain-specific `NonFinite { context, value }` variants on
  `PricingError`, `GreeksError`, `VolatilityError`, and
  `SimulationError` plus the crate-private `finite_decimal(f64)`
  guard used at every `f64 → Decimal` boundary. (#336, #337, #338)
- Public `tracing::instrument` on hot paths: `pricing::black_scholes`,
  `pricing::monte_carlo_option_pricing`, `pricing::price_binomial`,
  `volatility::utils::implied_volatility`, and
  `strategies::base::Optimizable::{get_best_ratio, get_best_area}`. (#342)
- `utils::deterministic_rng(seed)` plus
  `DETERMINISTIC_RNG_DEFAULT_SEED` — canonical entry point for
  reproducible Monte-Carlo / simulation tests. (#344)
- Deterministic regression tests under
  `tests/unit/pricing/identities_test.rs` covering put-call parity,
  CRR binomial convergence to Black-Scholes, and Greek
  sanity identities (`Γ_c == Γ_p`, `V_c == V_p`,
  `Δ_c − Δ_p ≈ e^{-qT}`). (#345)
- `CHANGELOG.md` following Keep a Changelog 1.1.0. (#346)

### Changed

- Breaking: step / simulation counts on `price_binomial`,
  `monte_carlo_option_pricing`, and related kernels are now
  `NonZeroUsize` so zero is structurally invalid at the type
  level. (#337)
- Breaking: many public surfaces now return
  `Result<T, concrete_error>` instead of panicking; `unsafe`
  blocks have been removed from the core in favour of typed
  guards. (#333, #334, #335, #338)
- Canonical `#[derive]` ordering
  (`Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash,
  Default, …, Serialize, Deserialize, ToSchema`), `#[repr(u8)]`
  on small stable enums, `#[serde(deny_unknown_fields)]` on
  input DTOs, and `#[serde(rename_all = "snake_case")]` on
  public-facing enums unless an existing wire contract
  forbids it (e.g. `BasicAxisTypes` keeps Pascal case). (#340)
- `#[inline]` applied on hot-path helpers and public entry
  points, `#[inline(never)]` on multi-arg builders, and
  `#[cold] #[inline(never)]` on every error constructor across
  `src/error/*`. (#339)
- `CustomStrategy::calculate_profit_at` no longer allocates a
  `Vec<Decimal>` per invocation; aggregates via `try_fold` + `d_add`. (#372)

### Fixed

- Doc-coverage floor: crate-level
  `#![deny(missing_docs, rustdoc::broken_intra_doc_links)]`
  with every previously-bare `pub` item now documented, and
  broken intra-doc links (e.g. `DecimalError::Overflow` →
  `crate::error::DecimalError::Overflow`) repaired. (#343)
- Unchecked `[]` indexing in production code migrated to
  `.get(..).ok_or_else(..)` on the highest-risk paths
  (`OptionChain` file-name / CSV readers, binomial-root lookup
  in `Option::binomial_price`) and
  `#![deny(clippy::indexing_slicing)]` enforced crate-wide
  with scoped, documented escapes on the remaining modules
  as follow-up work. (#341)

### Internal

- `#[must_use]` applied across the pure / builder public
  surface to catch discarded results at compile time.

[0.16.0]: https://github.com/joaquinbejar/OptionStratLib/releases/tag/v0.16.0
