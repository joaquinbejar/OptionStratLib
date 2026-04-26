# Changelog

All notable changes to **OptionStratLib** are documented in this file.

The format is based on [Keep a Changelog 1.1.0](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
