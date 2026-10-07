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

`Positive`, `ExpirationDate`, `Side`, `OptionStyle`, `OptionType` and the
other foundational types come from the standalone `positive`,
`expiration_date`, `financial_types` and `option_type` crates. Core
re-exports them unchanged (the crate docs list every path), so a value from
any of those paths is the original type and needs no conversion.

## Place in the workspace

- **Depends on** no other OptionStratLib crate; the foundational `positive`,
  `expiration_date`, `financial_types` and `option_type`, which it re-exports.
- **Must not depend on** every other OptionStratLib crate (it is the bottom
  layer); `make check-graph` enforces the layering (ADR-0001 D9).
- **In the facade:** always present: `optionstratlib::{model, utils,
  constants}`, the core errors in `optionstratlib::error`, the root types
  `Options`, `ExpirationDate`, `OptionStyle`, `OptionType`, `RainbowType`,
  `Side` and the root macros. The facade paths are the same types as the paths
  here; the [ownership
  map](https://github.com/joaquinbejar/OptionStratLib/blob/main/docs/ownership.md)
  lists every one with its feature.

<!-- #553: link the 0.21 to 0.22 migration guide here -->

## Minimal example

```rust
use optionstratlib_core::error::OptionsError;
use optionstratlib_core::model::{ExpirationDate, OptionStyle, OptionType, Options, Positive, Side};
use optionstratlib_core::pos_or_panic;
use rust_decimal_macros::dec;

fn main() -> Result<(), OptionsError> {
    // A call struck at 100 with the underlying at 105; the payoff at
    // expiry needs no pricing model.
    let option = Options::new(
        OptionType::European,
        Side::Long,
        "XYZ".to_string(),
        pos_or_panic!(100.0),
        ExpirationDate::Days(pos_or_panic!(30.0)),
        pos_or_panic!(0.2),
        Positive::ONE,
        pos_or_panic!(105.0),
        dec!(0.05),
        OptionStyle::Call,
        Positive::ZERO,
        None,
    );
    assert_eq!(option.payoff()?, dec!(5));
    Ok(())
}
```

## Runnable example

A runnable program that depends on this crate directly, with the smallest
dependency set and no facade, is [`osl-example-direct-pricing`](https://github.com/joaquinbejar/OptionStratLib/tree/main/examples/direct/pricing); `make tree-example-direct-pricing`
asserts its resolved graph. Core is in every example's graph; this is the smallest program that uses it.

## Features

| Feature  | Default | Effect                                                              |
|----------|---------|---------------------------------------------------------------------|
| `schema` | no      | Derives `utoipa::ToSchema` on core types and enables it on `positive`, `expiration_date`, `financial_types` and `option_type` |

Most users depend on the `optionstratlib` facade, which re-exports this crate.
Depend on `optionstratlib-core` directly when you need only the domain types.

## License

MIT
