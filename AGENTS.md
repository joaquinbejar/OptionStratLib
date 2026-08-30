# OptionStratLib

Rust library for options trading and strategy development across multiple asset
classes. Provides pricing models, Greeks, volatility surfaces, option chains,
P&L analysis, risk metrics, backtesting, and strategy construction with a
decimal-first, type-safe API.

## Architecture

Module-oriented library under `src/`:

```
src/backtesting/     — Historical simulation of strategies
src/chains/          — Option chain modeling and parsing
src/curves/          — Yield and term-structure curves
src/error/           — Typed error enums per module
src/geometrics/      — Geometry helpers (interpolation, intersection)
src/greeks/          — Delta, Gamma, Vega, Theta, Rho
src/metrics/         — Performance and risk metrics
src/model/           — Core domain: Option, Position, Leg, Trade, types
src/pnl/             — Profit & loss evaluation
src/pricing/         — Black-Scholes, Binomial, Monte-Carlo, Telegraph, exotics
src/risk/            — Margin, VaR, risk analysis
src/series/          — Time series primitives
src/simulation/      — Price path simulation (GBM, jump-diffusion, etc.)
src/strategies/      — Spreads, condors, butterflies, straddles, strangles, …
src/surfaces/        — Volatility surfaces
src/utils/           — Shared helpers
src/visualization/   — Plotly-based charts (behind `plotly` feature)
src/volatility/      — IV solvers and volatility models
```

`prelude.rs` re-exports the common public API.

## Coding Rules

All code MUST follow `rules/global_rules.md` — read it before writing any code.

## Key Decisions

- `rust_decimal::Decimal` for all prices, strikes, premia, P&L — never `f64`
  for monetary values. `f64` is only acceptable inside numeric-analysis
  internals (root finders, pricing kernels) where the inputs/outputs at the
  public boundary are `Decimal` or the `Positive` newtype.
- `thiserror` for all errors — never `anyhow`.
- `tracing` for all logging — never `println!`, `eprintln!`, `dbg!`, `log`.
- Newtypes at public boundaries: `Positive`, `OptionStyle`, `Side`,
  `ExpirationDate`, strategy-specific `Strategy*` types.
- Checked arithmetic on `Decimal` and counter `u64` — no `saturating_*` /
  `wrapping_*` on financial math.
- Feature flags: `plotly` (charts), `static_export` (PNG/SVG export, pulls in
  `async`), `async` (tokio + reqwest + futures for I/O-backed helpers).
- Rust 2024 edition, stable toolchain.
- Tests co-located (`#[cfg(test)] mod tests`) plus integration tests in
  `tests/`. Examples under `examples/` double as executable documentation.

## Module Boundaries

- `model/` is the core domain: types, options, positions, trades. It may be
  depended on by every other module. It does not depend on `strategies/`,
  `backtesting/`, `visualization/`.
- `pricing/`, `greeks/`, `volatility/`, `simulation/` depend on `model/` and
  `utils/`. They must remain pure numeric libraries — no I/O, no async in the
  default feature set.
- `strategies/` composes `model/` + `pricing/` + `greeks/` + `pnl/` + `risk/`.
- `visualization/` depends on everything and is gated behind `plotly`.
- `backtesting/` depends on `strategies/` + `series/` + `simulation/`.
- `error/` is leaf — no cross-module deps beyond `std` and `thiserror`.

## Agent Workflow

When implementing a non-trivial change, follow this order:

1. **Model first** — types, enums, invariants in `src/model/` (no deps on other
   library modules beyond `error`, `utils`).
2. **Error variants** — extend the relevant `thiserror` enum in `src/error/`.
3. **Numerics** — pricing / greeks / volatility kernels. Pure functions, unit
   tested against analytic or reference values.
4. **Composition** — strategies, pnl, risk. Compose the numerics above.
5. **Integration** — backtesting, chains, series, simulation glue.
6. **Visualization** — only after the numeric surface is stable, behind
   `plotly`.
7. **Tests + examples** — unit in each file, integration under `tests/`,
   runnable demo under `examples/`.
8. **Docs** — `///` on every `pub` item, update `README.tpl` / `README.md` if
   the public surface changed, and update `doc/` if present.

Steps 3 and 5 can be parallelized across agents when the model layer is
stable.

## What the gates do not check

Six facts about this repository's checks, each with the command that shows
it. They are gaps, not advice.

1. **`make doc` built no documentation.** It ran
   `cargo clippy -- -W missing-docs`; it is `cargo doc --all-features
   --no-deps` as of this commit. `make create-doc` has no `--all-features`,
   so a doc link inside `plotly` / `static_export` / `async` code was
   resolved by nothing. Add `/// See [`ThisItemDoesNotExist`].` to
   `pub trait Graph` in `src/visualization/plotly.rs`, then:

   ```
   make create-doc                    # exit 0
   cargo doc --all-features --no-deps # exit 101, unresolved link
   ```

   Two `private_intra_doc_links` warnings stand on a clean tree and are not
   worth re-investigating: `RNDStatistics::new` (`src/chains/rnd.rs`) and
   `lower_break_even` (`src/strategies/base.rs`).

2. **`make public-api-check` is not a documentation gate.** It builds
   rustdoc with `--all-features`, but that build runs under
   `--cap-lints warn`, so the lints `src/lib.rs` denies (`missing_docs`,
   `rustdoc::broken_intra_doc_links`) are capped to warnings. The cap is not
   `cargo public-api`'s choice: it comes from the `rustdoc-json` builder it
   calls, which defaults to it and passes it through
   (`rustdoc-json-0.9.10/src/builder.rs:316` sets
   `cap_lints: Some(String::from("warn"))`, `:159` forwards it). An upgrade
   changing that default changes this behaviour without anything here
   changing. With the same broken link in place:

   ```
   cargo +nightly-2026-08-28 public-api -sss --all-features > /dev/null
   # exit 0, full snapshot produced, and the link reported as
   # "warning: unresolved link to ..." — not as an error, so it reads
   # like ordinary noise rather than something hiding under a green check
   ```

3. **`cargo test --all-features` never exercises the default feature set.**
   `src/visualization/default.rs` is `#[cfg(not(feature = "plotly"))]`, so
   the `Graph` a default-features consumer gets is never compiled. Append
   `compile_error!("x");` to that file:

   ```
   cargo check --all-features   # exit 0
   cargo check                  # exit 101
   ```

   `make test` is what covers all three: default, `plotly`, and
   `static_export,plotly`, run separately.

4. **`cargo-semver-checks` compares against the published baseline, not the
   snapshot.** `.github/workflows/semver.yml` passes no `baseline-rev`, so
   the baseline is the crates.io release. A rename, or a variant added to an
   exhaustive public enum, therefore needs a minor bump for a 0.x crate.
   `make public-api-check` only diffs against
   `public-api/optionstratlib.txt` and never asks for the bump.

   ```
   grep -A4 cargo-semver-checks-action .github/workflows/semver.yml
   curl -sL -H 'User-Agent: optionstratlib' \
     https://crates.io/api/v1/crates/optionstratlib/versions   # baseline
   ```

5. **`gh` reports refusals on stderr only.** A `gh pr merge` blocked by a
   base-branch policy writes nothing to stdout, so a caller capturing only
   stdout sees a silent no-op:

   ```
   gh pr view 999999 --repo joaquinbejar/OptionStratLib 2>/dev/null
   # no output; the message went to stderr
   ```

6. **`make scan-banned` is a grep, so confirm it sees your construct before
   trusting a clean run.** It is line-oriented `awk`, and its comment filter
   skipped every line beginning with `*`, which is what the continuation
   lines of a multi-line expression are indented to — five `.exp()` calls in
   `src/pricing/compound.rs` were invisible to it until this commit. It also
   cannot tell receiver types apart, so `f64::exp` and `Decimal::exp` look
   alike and the `f64` sites carry `// scan-banned: allow` markers saying so.
   Before relying on it for a new construct, plant one and check it is
   caught:

   ```
   printf '\npub fn probe(x: Decimal) -> Decimal {\n    x\n        * dec!(2)\n        .exp()\n}\n' >> src/pricing/utils.rs
   make scan-banned    # must fail (exit 2) and print the offending line
   git checkout -- src/pricing/utils.rs
   ```
