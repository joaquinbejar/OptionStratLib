# Changelog

All notable changes to **OptionStratLib** are documented in this file.

The format is based on [Keep a Changelog 1.1.0](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Fixed

- **`prepare_file_path` reported failure for a file that was already gone.**
  It tested `Path::exists` and then removed, so anything deleting the file in
  between made `remove_file` fail with `NotFound` — for a postcondition that
  already held. Every `write_html` / `write_png` caller inherited it. Two
  tests sharing a working directory were enough to lose the race, and it
  aborted a whole `cargo tarpaulin` run on `main` with `Failed to remove
  existing file: multiple_curves_test.html`, taking a coverage report down
  over a file nobody was reading. The removal is now attempted
  unconditionally and `NotFound` is accepted, which also makes the function
  idempotent for concurrent writers downstream.
- The four `plotters` tests that came in `_bis` pairs wrote to the same two
  paths as their originals, so they raced each other by construction. Each
  pair now writes its own file.
- **The arithmetic Asian is accurate at short maturities, where it used to
  return confident wrong prices** (#462). The general branch of the
  Turnbull-Wakeman second moment recovered `σ²_asian·T` from the difference of
  two terms that grow as `2S²/(a·c·T²)` while the answer shrinks with `T`; at
  fourteen minutes to expiry the terms are around `2.8e15` and the signal is
  `3.7e-8`. It overpriced by a factor of **612** at eighty-six seconds and
  collapsed to approximately zero below `1e-5` days. The moment is now twice
  the first divided difference of `φ(w) = (e^w − 1)/w`, which makes all three
  removable singularities ordinary points and never subtracts two terms larger
  than the result. Relative error across the nine maturities of the issue's
  table is now at most `6.9e-12`, against `1e-9` asserted.

  Prices are **bit-for-bit unchanged at 7, 30, 90, 182.5 and 365 days**, call
  and put. One ordinary maturity moves: **at 1 day the price changes by a
  relative `2.5e-10`, and it moves toward the reference** — measured against
  50-digit `mpmath` quadrature, the old value sat `2.5e-10` away from it and
  the new one sits `8.5e-14` away. A one-day option moving by `2.5e-10`
  relative is orders of magnitude below a tick, so no quoted price changes;
  the direction and size are stated here so the claim can be checked rather
  than taken.

### Changed

- **`Curve::bilinear_interpolate` now answers across the whole interior
  domain, and its exact-match branch fails on a repeated abscissa** (#451).
  The method read a four-sample window starting at the bracket index, which
  `find_bracket_points` only guarantees to `i + 1`, so every `x` in the last
  two segments left the window past the end and returned
  `InterpolationError::Bilinear`. The cell is now built from the segment
  bracketing `x` and the segment two positions on, the far one clamped to the
  curve's last segment where it would run off the end. On the final segment
  the two edges coincide and the answer is the linear interpolant there; on
  the second-to-last the far edge is the immediately following segment.

  The clamp reaches only those last two segments, so every abscissa the
  method already answered keeps its answer digit for digit, and no existing
  test changed. Values were validated against a reference implementation of
  the rule in exact rational arithmetic, cross-checked against `mpmath` at 60
  digits, over three curves including a non-uniform one.

  The near edge is deliberately not clamped, unlike `cubic_interpolate` at its
  own boundary: the window start is the denominator of the fraction along the
  cell, so moving it back off the segment holding `x` pushes that fraction
  past `1`, turns two of the four weights negative and stops the answer being
  a convex combination of the cell's corners. On `(0,0), (1,1), (2,4), (3,9)`
  that variant returns `9.5` at `x = 2.5`, above every ordinate on the curve.

  The exact-match branch now goes through `Curve::exact_point_at` like the
  three other interpolators, so several ordinates at the queried abscissa
  return `InterpolationError::DegenerateInterval` instead of the lowest of
  them. This is a behaviour change on a curve that breaks the
  one-point-per-abscissa rule, and it closes the exception the #466 entry
  below left open. No signature changed and the public API is unchanged.

- **The crate moves to 0.21.0, which is the breaking bump for a `0.x`
  version.** `cargo-semver-checks` compares against the published 0.20.0 and
  rejected the rename below under `inherent_method_missing`, since a version
  unchanged from its baseline is read as a minor bump and a minor bump may not
  remove a public method. Every earlier change in this cycle was a return type
  moving to `Result`, which no lint covers, so this is the first one the check
  could see.

- **`Surface::get_curve` is renamed to `Surface::project_onto` and returns
  `Vec<Point2D>` instead of `Curve`, which is a breaking change** (#466).
  One change with one reason: the method is a projection, and both its name
  and its type said it was a curve. Projecting a surface onto one axis is
  multi-valued by construction: every row of a grid contributes a different
  height above the same projected abscissa, and two rows that agree on the
  two surviving coordinates project onto the very same point. A `Curve` is a
  function of its abscissa and stores its points in a `BTreeSet`, so it
  silently dropped the collisions. The vector keeps every point:
  `surface.project_onto(axis).len() == surface.points.len()`, always.
  Contents are sorted ascending by the projected `(x, y)` pair, which is the
  order the `BTreeSet` gave; only its deduplication is gone.

  The method has no production callers and its `Axis` parameter type is not
  re-exported, so it is uncallable from outside the crate. To migrate inside
  it, aggregate the ordinates sharing an abscissa to one value, then build the
  `Curve`; `Curve::from_vector(surface.project_onto(axis))` reproduces the old
  behaviour, losses included. The note on `Surface::get_curve` under the #450
  entry below describes the method by its former name.

- **Interpolating a curve at a repeated abscissa now fails instead of
  returning the lowest ordinate stacked there** (#466). `Curve` is documented
  as a function of its abscissa, but nothing enforces it (`points` is a `pub`
  field, so no constructor could). `linear_interpolate`, `cubic_interpolate`
  and `spline_interpolate` return `InterpolationError::DegenerateInterval`
  when several points share the requested `x`, rather than picking one of
  them: the value there is not defined and no choice among them is less
  arbitrary than another. A curve with one point per abscissa is unaffected,
  and `AxisOperations::get_values` still reads every ordinate at an abscissa.
  The exact-match branch of `Curve::bilinear_interpolate` was left out of that
  pass and is covered by the #451 entry above, which brings it into line.

  The one-point-per-abscissa rule, and what each consumer does when it is
  broken, are now stated on `Curve::new`, `Curve::from_vector`,
  `Curve::merge`, `get_point`, `contains_point`, `get_values`,
  `merge_axis_interpolate`, the four interpolation traits and
  `find_bracket_points`, with the matching one-height-per-xy-coordinate rule
  on `Surface::new`.
- **`model::utils::mean_and_std` is now fallible, which is a breaking change**
  (#470). It returns `Result<(Positive, Positive), PositiveError>` instead of
  `(Positive, Positive)`. It summed with the raw `Positive` operator, which
  panics on overflow, and divided by `vec.len() as f64`, which divides by zero
  on an empty sample; every `get_profit_ranges` implementation averages its leg
  volatilities through it. The returned figures are unchanged for every sample
  the panicking form accepted — the squared deviations are still taken in
  `f64` — but a deviation that overflows is now reported instead of collapsing
  to zero through `unwrap_or(Positive::ZERO)`. To migrate, add `?` where the
  value feeds a fallible function.

  `PositionError` gains a `PositiveError` variant in the same change, so
  `Position::total_cost` and `Position::fees` can carry the cause of a
  `Positive` overflow instead of flattening it into a message. Both already
  returned `Result`, so their signatures are unchanged. Matching exhaustively
  on `PositionError` needs a new arm.

  `model::resolve_expiration_date` and `model::reject_unrepresentable_expiration`
  are new. `ExpirationDate::get_date()` aborts with
  `` `DateTime + TimeDelta` overflowed `` for a day count no calendar can
  represent, and every strategy reaches it through
  `calculate_pnl_at_expiration`; these resolve or reject it instead. The guard
  #441 added inside `OptionChain::build_chain` is replaced by a call to the
  shared one. The instant returned is identical for every input the panicking
  form accepted.

- **Two `ProtectivePut` methods are now fallible, which is a breaking change**
  (#460). `total_fees` returns `Result<Positive, PositiveError>` instead of
  `Positive`, and `protection_level` returns
  `Result<Decimal, StrategyError>` instead of `Decimal`. Both aborted the
  process on inputs a caller can supply: the fee sum used the raw `Positive +
  Positive`, which panics for a fee at `Positive::MAX`, and `protection_level`
  divided by the spot cost basis with the raw operator, which a zero
  underlying price turns into a panic. Adding `try_` twins instead would have
  left the panicking methods in place, which is the opposite of what this
  sweep is for.

  To migrate, add `?` where the value feeds a fallible function, or
  `.expect("…")` at a boundary that cannot propagate. `ProtectivePut::new` now
  rejects a zero underlying price as well, so a position built through the
  constructor never reaches the error arm of `protection_level`; a
  deserialized one can.

- **A zero-volatility American or Bermuda option is no longer priced as a
  European, so its price goes up** (#449). `price_binomial` short-circuited
  every contract with `volatility == 0` to the discounted forward payoff,
  `e^{-rT}·payoff(S·e^{rT})`, which is the European value and silently drops
  the early-exercise premium. The same branch is taken when the lattice
  collapses (`u == d`, `σ√dt` below the representable scale), so both paths
  change. An American is now worth the better of exercising immediately and
  holding to expiry; a Bermuda the best of its schedule and expiry, with a
  schedule that is empty, or entirely beyond expiry, still pricing as a
  European. A European is unchanged.

  A zero-volatility American put at `S = 90`, `K = 100`, `r = 5%`, `T = 1`:

  | | before | after |
  |---|---|---|
  | price | `5.122942450071404` | `10` |

  `10` is `K − S`, the value of exercising on the spot. The old number was
  `e^{-0.05}(100 − 90·e^{0.05})`, roughly half. The matching American call at
  `S = 110`, `K = 100` is unchanged at `14.877057549928595`, since a call on
  a non-dividend-paying forward is never exercised early while `r > 0`; flip
  the rate to `r = −5%` and it moves from `4.872890362397602` to `10`.
- **`Point2D` and `Point3D` compare, order and hash on all their coordinates**
  (#450). Both types broke the `Ord` contract, which requires `a == b` exactly
  when `a.cmp(&b)` is `Ordering::Equal`: `Point2D` compared equal on `x` alone
  while ordering on `(x, y)`, and `Point3D` compared equal on `(x, y)` while
  ordering on `(x, y, z)`. Two points could be equal *and* strictly ordered.
  `PartialEq` now reads every coordinate on both types, and `Point3D` gains a
  `Hash` impl (it had none). Ordering is unchanged.
- **A surface no longer loses half its grid when its axes are merged.**
  `Surface` indexes itself by `Point2D` through `AxisOperations<Point3D,
  Point2D>`, and `merge_indexes` deduplicates those indices in a `HashSet`.
  With `Point2D` hashing on `x` alone, every column of the grid collapsed onto
  a single cell: merging a 2x2 surface with itself produced **two** indices
  instead of four, and `merge_axis_interpolate` then worked from the truncated
  axis. It now keeps all of them. Anything asserting the narrow result will
  see more indices, and more interpolated points, than before.
- **A `BTreeSet` of points no longer depends on how it was built.**
  `BTreeSet::from_iter` (and therefore `collect`) sorts and then deduplicates
  adjacent elements with `PartialEq`, while `insert` deduplicates with `Ord`.
  While those two disagreed, the same points produced different sets by
  different routes: four `Point3D` stacked on one xy-coordinate collected to a
  set of **one** and inserted to a set of **four**. Collecting now keeps every
  distinct point, so `Surface::get_curve` returns the whole projection — an
  `n`-by-`m` grid yields up to `n * m` points where it used to yield `n`,
  having silently dropped all but the greatest ordinate per abscissa. Curves
  collected from an option chain are unaffected, their abscissae being unique
  strikes.
- `Surface::bilinear_interpolate` now reports `"Invalid quadrilateral"` for
  points stacked on one xy-coordinate. That check existed but was unreachable:
  the collapse above reduced such a set to a single point first, so the
  `"Need at least four points"` guard answered instead.
- Membership probes against `Point2D` / `Point3D` are now exact. Code doing
  `points.contains(&probe)`, `points.iter().any(|p| p == &probe)` or
  `HashSet`/`BTreeSet` lookups used to match any point sharing the probe's
  leading coordinates; it now matches only the point itself. Nothing in the
  crate relied on the loose behaviour except `merge_indexes` above, but a
  downstream caller might.
- **`OptionData::apply_spread` widens a thin quote instead of withdrawing it**
  (#439). A contract whose mid sat below one full spread used to lose `bid`,
  `ask` **and** `middle`. Both sides are now quoted around the mid — `mid ±
  half_spread` — and floored at one tick (`10^-decimal_places`); a supplied mid
  is kept, held inside the widened quote rather than cleared. Only a quote with
  no mid and no two-sided book is still erased.
- **Chains keep their cheap strikes, so row counts change.**
  `OptionChain::build_chain` stops generating strikes once both wings come back
  unpriced, so the erasure above also truncated the chain: a build with
  `chain_size = n` returned `n + 1` rows and now returns the full `2n + 1`.
  Downstream assertions on row counts, and anything iterating a chain, will see
  the wings that used to disappear as they decayed.
- `apply_spread` no longer changes the previous quote for a mid at or above the
  tick. Below the tick it does deviate deliberately: a bid that used to round
  to `0.00` is now floored at one tick, since a zero bid is not a market.
- A `decimal_places` beyond `Decimal`'s maximum scale used to panic through
  `Positive::round_to`; `apply_spread` now leaves the quotes untouched and logs.

### Fixed

- **Every arithmetic Asian option with `r = q` was mispriced** (#454). The
  Turnbull-Wakeman second moment short-circuited its removable `b = 0`
  singularity to `M2 = S² e^{σ² T}`, which is `E[S_T²]`: the second moment of
  the *terminal* price, not of its average. The `b → 0` limit of the double
  integral the function actually implements is
  `M2 = (2 S² / (σ² T²)) [ (e^{σ² T} − 1) / σ² − T ]`. The stand-in handed the
  moment matching `σ_adj = σ` where the average carries `σ_adj ≈ σ / √3`, so
  the price came out far too high and the branch was discontinuous: a one-year
  ATM call on `S = K = 100`, `σ = 20%`, `r = q = 4%` returned
  **7.6532330880**, against `4.4308752837` and `4.4308753262` for a carry a
  billionth either side. It now returns **4.4308753052**, which agrees with
  quadrature on the defining integral to `3e-10` and with both neighbours to
  `2.2e-8`. A forward-priced or fully-carried underlying is an ordinary
  contract, so every pinned `r = q` Asian value moves.
- **A zero-volatility Asian option priced off the terminal forward instead of
  the average** (#447). Both kernels short-circuited `σ = 0` to
  `(S e^{bT} − K)⁺ e^{−rT}`, but a deterministic path still has to be
  averaged: the geometric mean of `S e^{b t}` over the window is `S e^{bT/2}`
  and the arithmetic mean is `S (e^{bT} − 1) / (bT)`, which are also the
  `σ → 0` limits of the Kemna-Vorst and Turnbull-Wakeman formulas the branches
  stand in for. A one-year ATM call on `S = K = 100`, `r = 5%`, `q = 0`
  returned **4.8770575499** from both kernels and now returns
  **2.4080487528** (geometric) and **2.4182085485** (arithmetic). The old
  value was only correct at `b = 0`, where every average collapses to `S`.
- **The simple chooser was priced with the wrong formula (#448).**
  `chooser_black_scholes` discounted its two `y` legs at the choice date `t`
  instead of at expiry `T`, and built `y1` from `b*t` where Rubinstein (1991)
  has `b*T + σ²*t/2`, so every simple chooser came out too expensive. On
  Haug's worked example (S = K = 50, t = 0.25, T = 0.5, r = b = 0.08,
  σ = 0.25; reference **6.1071**) the function returned **6.524120** and now
  returns **6.107077**: 6.8% high before, within 1e-14 of the closed form now.
  The module doc already stated the corrected formula; the code did not
  implement it.
- With no diffusion left to run to the choice date (`σ√t` collapsing to zero)
  the surviving branch is now picked on the sign of `ln(S/K) + b*T`, the
  forward moneyness at expiry, rather than on `ln(S/K)`. A chooser whose spot
  sits below the strike but whose forward sits above it prices as the call,
  not as the put. A zero numerator, which that limit used to report as an
  undefined `0 / 0`, is exactly forward parity — where call and put are worth
  the same — and now returns that common value instead of an error (#448).
- **Reachable panic in the lower break-even of eight strategies.**
  `strike - credit` was computed with the raw `Positive - Decimal` operator,
  which panics when the credit (or, on a long structure, the debit) exceeds the
  strike — reachable from the optimizers as soon as a chain keeps its cheap
  wings. `ShortStraddle`, `ShortStrangle`, `IronCondor`, `IronButterfly`,
  `LongStraddle`, `LongStrangle`, `BearPutSpread` and `ShortButterflySpread` now
  share `lower_break_even`, which floors at zero. A lower break-even of `0.00`
  means the strategy has none (#439).

## [0.20.0] - 2026-08-28

Two things land together. The option chain now carries the full twelve-greek set
per strike, and every greek in the crate returns the sensitivity of the
**position** rather than of one long contract. Both are **breaking**: the first
for anyone constructing `OptionData` with an exhaustive struct literal, the
second for anyone who was compensating for the old unsigned behaviour. See
*Migration*.

### Added

- `OptionData::greeks_call` and `OptionData::greeks_put`, each an
  `Option<GreeksSnapshot>` carrying all twelve greeks for that option style, so
  consumers no longer convert through `TryFrom<&OptionData> for Options` and
  recompute on every read. Both are computed for **one long contract**: a
  consumer holding a short negates all twelve, since every value is a derivative
  of the long position value.
- `OptionData::calculate_greeks`, which populates both snapshots and refreshes
  the `delta_call`, `delta_put` and `gamma` mirror fields together.
- `OptionChain::update_greek_snapshots`, the full-set counterpart to
  `update_greeks`.
- `OptionChainBuildParams::with_greek_snapshots`, opting a chain build into the
  snapshots. Off by default.
- `impl From<Greek> for GreeksSnapshot`.
- `alpha` is now re-exported from `crate::greeks`, alongside the other eleven.

### Fixed

- **Breaking behaviour change.** Every Black-Scholes/Merton greek in
  `greeks::equations` now respects `Side`. Only `delta` did before, so any
  strategy holding a short leg reported gamma, theta, vega, rho and the
  higher-order greeks with the sign of the equivalent long position, next to a
  delta that was signed correctly. A short-premium position looked as though it
  was losing to time decay when it was collecting it. `Options::quantity` is
  `Positive` and can never carry direction, so `Side` is the only carrier of it
  (#428).
- **Breaking behaviour change.** The Black-76 and Garman-Kohlhagen greek
  families now use the same position-sign convention. Previously only their
  deltas respected `Side`; a short futures or FX option therefore reported
  gamma, vega, theta and rho with the sign of the equivalent long. All greeks in
  both families are now signed by `Side` exactly once and scale linearly with
  `quantity` (#436).
- `PortfolioGreeks::from_positions` re-applied `quantity * sign` to greeks that
  already carried both, which squared the position size and cancelled the sign.
  `DeltaAdjustment` applied the same second sign to delta.
- `rho_d` and `vanna` returned an error at expiry where their ten siblings
  returned zero, so `Greeks::greeks()` lost the entire set for any expired
  option. Both now return zero.
- The non-European fallbacks in `delta` and `gamma` returned a per-contract long
  value, dropping both side and quantity, because the numerical engine prices
  through an absolute value.

### Changed

- `Greeks::greeks()` computes the shared Black-Scholes intermediates once per
  option rather than once per greek, which makes it about 6.5x faster and cuts
  the cost of a chain build with greek snapshots by about 4.7x. Every value is
  unchanged (#431).
- `GreeksSnapshot` no longer sets `#[serde(deny_unknown_fields)]`. It is a wire
  type now, and adding a thirteenth greek must not break deserialization for
  consumers built against an older version.
- `OptionData::set_volatility` and `set_extra_params` drop the stored snapshots
  rather than leave them stale against changed pricing inputs, and
  `OptionChain::update_expiration_date` does the same.

### Migration

`OptionData` gained two public fields. Any exhaustive struct literal needs
`..Default::default()` or the two new fields; `OptionData::new` is unchanged and
needs no edit. Serialized chains are unaffected in both directions: the new
fields are skipped when absent, and a payload written before this release
deserializes with both set to `None`.

On the sign convention, any consumer that was compensating by negating the
unsigned greeks itself must stop, in all three families. A long and a short of
the same contract now net to exactly zero. `alpha` is the one exception and is
unchanged, being the ratio `gamma / theta`, which a short negates in both terms.
`OptionData::greeks_call` and `greeks_put` are also unaffected, being built
through `get_option(Side::Long, style)` and so per-long-contract by
construction.

## [0.19.1] - 2026-08-28

Correctness and housekeeping follow-up to the cost-of-carry fix in 0.19.0. No
API changes.

### Fixed

- `vomma` and `veta` applied the position quantity twice, because `vega`
  already carries it. Both are first derivatives of vega and are linear in
  position size, so a 10-lot position reported 10x the true value and the error
  propagated into `PortfolioGreeks`.
- `d1`'s documentation still described its third argument as the risk-free rate
  after it was renamed to `carry_rate`, stating two different meanings for the
  same argument in one doc block. `d1` and `d2` are public and in the prelude,
  so a caller following it disagreed with `option.delta()` by about 10%.
- Four stale `charm` expectations meant two tests over the identical portfolio
  asserted different numbers, both within 2% of their tolerance.
- `test_dividend_high_q_carry_regression` pinned eleven greeks at `1e-28`, far
  tighter than the f64 normal CDF behind them supports. Relaxed to `1e-12`.

### Changed

- The `format_check` CI job ran `make fmt`, which formats rather than checks and
  exited 0 whatever the input, so no pull request was ever format-checked. It
  now runs `make fmt-check`.

## [0.19.0] - 2026-08-17

Dependency refresh: every dependency moved to its latest stable minor, and the
three in-house crates jumped a breaking release each. It is **breaking** for
consumers — see *Migration*.

### Added

- **`simulation::expanding_window_vols` is public** (also re-exported from the
  prelude): the per-step, look-ahead-free volatility estimator that
  `walk_steps` / `walk_steps_par` use for `WalkType::Historical`, whose
  volatility is not a model parameter and has to be estimated from the series.
  A consumer that generates a historical path itself — with
  `WalkTypeAble::generate_with_vol`, pricing its own chains — can now use the
  same estimate instead of falling back to a constant volatility or keeping a
  second copy of the mathematics. The contract is documented and tested:
  one estimate per price, element `i` a function of `prices[..=i]` only
  (extending the series leaves earlier estimates untouched), the first two
  indices backfilled with the first computable estimate, `Ok(None)` below three
  prices, and the last estimate equal to whole-series `constant_volatility`.
  (#423, downstream OptionChain-Simulator#63)

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

### Fixed

- **The Security Audit workflow is green again** (#422). It had been red on
  `main` since 2026-08-05 on five advisories. The dependency refresh above
  resolves three of them outright — `crossbeam-epoch` (RUSTSEC-2026-0204),
  `quinn-proto` (RUSTSEC-2026-0185) and `rustls-webpki` (RUSTSEC-2026-0104) now
  resolve to patched releases. The remaining two are unreachable and carry
  documented waivers in `.cargo/audit.toml`, each with rationale, owner and a
  2027-02-15 review date:
  - RUSTSEC-2026-0235 (`rkyv` 0.7.46) — lockfile-only, an optional dependency
    of `rust_decimal` that this crate never enables.
  - RUSTSEC-2025-0119 (`number_prefix` 0.4.0, unmaintained) — likewise
    lockfile-only, via `indicatif`.
  - RUSTSEC-2025-0134 (`rustls-pemfile` 1.0.4, unmaintained) — a *build*
    dependency of `plotly_static` (through `webdriver-downloader` and
    `reqwest` 0.11), reachable only with `static_export`; it downloads a
    webdriver on the build machine and is never linked into the library.
  With those waived, the workflow now runs with `denyWarnings: true`, so a new
  unmaintained or unsound dependency fails the gate instead of passing as a
  warning.

### Housekeeping

- `.cargo/audit.toml` added, mirroring the `positive` crate's policy file: one
  documented waiver per advisory (rationale, owner, review date) and
  `informational_warnings` on, so unmaintained/unsound/notice advisories are
  reported rather than dropped. See *Fixed* above for the entries.
- The version strings in the crate-level docs (and therefore in the generated
  `README.md`) say `0.19.0`; they had been left at `0.18.0` through the 0.18.1
  release.
- `cargo test` no longer spawns a browser. Five visualization tests needed a
  WebDriver whose major version matches the installed browser (PNG/SVG export
  through `plotly_static`), one really did hand a chart to the default browser
  despite a comment claiming otherwise (`OutputType::Browser` calls
  `Plot::show()`), and four doc examples wrote a PNG when executed. The tests
  are now `#[ignore]`d with a reason and the doc examples are `no_run`, so they
  still compile and are still runnable on demand: `make test-visual`, a new
  target that runs exactly the ignored set. The `make test` recipe also passes
  `--features static_export,plotly`; the comma was missing, so `plotly` was
  being parsed as a test-name filter rather than a feature.

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
