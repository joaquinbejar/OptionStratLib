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

## Features

| Feature  | Default | Effect                                                              |
|----------|---------|---------------------------------------------------------------------|
| `schema` | no      | Derives `utoipa::ToSchema` on pricing types and enables `schema` in core and math |

## License

MIT
