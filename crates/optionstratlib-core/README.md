# optionstratlib-core

The core domain model of [OptionStratLib](https://github.com/joaquinbejar/OptionStratLib):
the types every other OptionStratLib crate builds on, with no pricing model,
market data, strategy or presentation code.

| Module      | Contents                                                                 |
|-------------|--------------------------------------------------------------------------|
| `model`     | `Options`, `Position`, `Leg` and its spot, future and perpetual legs, `Trade`, payoff contracts, checked `Decimal` helpers |
| `error`     | `DecimalError`, `OptionsError`, `PositionError`, `TradeError`, `OperationErrorKind` |
| `utils`     | `TimeFrame` and date helpers, deterministic RNG, numeric helpers, `Len`  |
| `constants` | Library-wide numeric constants, market conventions and time units        |

Monetary values are `rust_decimal::Decimal` or `positive::Positive`, and
arithmetic on them is checked.

## Features

| Feature  | Default | Effect                                                              |
|----------|---------|---------------------------------------------------------------------|
| `schema` | no      | Derives `utoipa::ToSchema` on core types and enables it on `positive`, `expiration_date`, `financial_types` and `option_type` |

Most users depend on the `optionstratlib` facade, which re-exports this crate.
Depend on `optionstratlib-core` directly when you need only the domain types.

## License

MIT
