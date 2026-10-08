# optionstratlib-pricing

Option pricing models, Greeks and implied volatility of
[OptionStratLib](https://github.com/joaquinbejar/OptionStratLib), on the core
domain types. The crate depends only on `optionstratlib-core`,
`optionstratlib-math` and general numeric crates: no option chain, simulation
engine, strategy, plotting, I/O or async runtime.

| Module       | Contents                                                                 |
|--------------|--------------------------------------------------------------------------|
| `pricing`    | Black-Scholes, Black-76, Garman-Kohlhagen, binomial, Monte Carlo over supplied paths, telegraph, American and exotic kernels; `OptionPricing`; `price_option_with`; `Profit`; solver defaults |
| `greeks`     | First and higher-order Greeks, `Greeks`, model-specific and numerical Greeks, Greeks of the leg types |
| `volatility` | Implied-volatility solvers, volatility models and traits                 |
| `error`      | `PricingError`, `GreeksError`, `VolatilityError`                         |

Prices, premia and Greeks cross the public boundary as `rust_decimal::Decimal`
or `Positive`; `f64` stays inside the numerical kernels.

## Place in the workspace

- **Depends on** `optionstratlib-core` and `optionstratlib-math`.
- **Must not depend on** `optionstratlib-simulation`, `-market`, `-analytics`,
  `-strategies`, `-backtest` and `-visualization`; `make check-graph` enforces
  the layering (ADR-0001 D9).
- **In the facade:** `optionstratlib::{pricing, greeks, volatility}` and the
  pricing errors in `optionstratlib::error`, under the facade feature
  `pricing`. The facade paths are the same types as the paths here; the
  [ownership
  map](https://github.com/joaquinbejar/OptionStratLib/blob/main/docs/ownership.md)
  lists every one with its feature.

Moving from 0.21? The [0.22 architecture and adoption
guide](https://github.com/joaquinbejar/OptionStratLib/blob/main/docs/migration-0.22.md)
maps each redesign to its replacement workflow; 0.22 keeps no
compatibility with 0.21.

## Minimal example

```rust
use optionstratlib_pricing::error::PricingError;
use optionstratlib_core::model::{ExpirationDate, OptionStyle, OptionType, Options, Positive, Side};
use optionstratlib_core::pos_or_panic;
use optionstratlib_pricing::pricing::black_scholes;
use rust_decimal_macros::dec;

fn main() -> Result<(), PricingError> {
    // Hull's worked example: S = 42, K = 40, r = 10%, sigma = 20%, T = 0.5.
    let call = Options::new(
        OptionType::European,
        Side::Long,
        "XYZ".to_string(),
        pos_or_panic!(40.0),
        ExpirationDate::Days(pos_or_panic!(182.5)),
        pos_or_panic!(0.2),
        Positive::ONE,
        pos_or_panic!(42.0),
        dec!(0.10),
        OptionStyle::Call,
        Positive::ZERO,
        None,
    );
    let price = black_scholes(&call)?;
    assert!((price - dec!(4.76)).abs() < dec!(0.01));
    Ok(())
}
```

## Runnable example

A runnable program that depends on this crate directly, with the smallest
dependency set and no facade, is [`osl-example-direct-pricing`](https://github.com/joaquinbejar/OptionStratLib/tree/main/examples/direct/pricing); `make tree-example-direct-pricing`
asserts its resolved graph.

## Features

| Feature  | Default | Effect                                                              |
|----------|---------|---------------------------------------------------------------------|
| `schema` | no      | Derives `utoipa::ToSchema` on pricing types and enables `schema` in core and math |

## License

MIT
