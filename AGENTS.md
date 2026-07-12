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
