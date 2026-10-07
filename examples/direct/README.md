# Direct-component examples

Runnable programs that depend on the OptionStratLib component crates
directly, without the `optionstratlib` facade. Each is a copyable
`Cargo.toml` plus a short `src/main.rs`, and each shows the smallest
dependency set of one capability. Layout and naming follow ADR-0004 section
8: `examples/direct/<scenario>/`, package `osl-example-direct-<scenario>`,
a workspace member with `publish = false`.

| Scenario | Package | Components it depends on | Facade equivalent | Same graph proved by fixture |
| --- | --- | --- | --- | --- |
| [`math`](math) | `osl-example-direct-math` | math | `features = ["math"]` | none: `make tree-example-direct-math` asserts its graph |
| [`pricing`](pricing) | `osl-example-direct-pricing` | core, pricing | `features = ["pricing"]` | `pricing-only` |
| [`market`](market) | `osl-example-direct-market` | core, market | `features = ["market"]` | `market-minimal` |
| [`analytics`](analytics) | `osl-example-direct-analytics` | core, market, analytics | `features = ["analytics"]` | `analytics-only` |
| [`strategies`](strategies) | `osl-example-direct-strategies` | core, strategies | `features = ["strategies"]` | `facade-strategies` |
| [`simulation`](simulation) | `osl-example-direct-simulation` | core, simulation | `features = ["simulation"]` | `simulation-only` |
| [`backtest`](backtest) | `osl-example-direct-backtest` | core, simulation, strategies, backtest | `features = ["backtest"]` | `full-backtest` |
| [`visualization`](visualization) | `osl-example-direct-visualization` | core, strategies, visualization (`plotly`) | `features = ["plotly"]` | `facade-plotly` |

Every published component has a program: `math` is the curve and
interpolation one, and `optionstratlib-core` (the domain model every other
crate builds on) is in the graph of all eight and imported directly by seven.

## The trade-off

A direct dependency names exactly the layers you use, and the resolved graph
is the one the table's fixture asserts: no option chain in the pricing
program, no simulation engine in the market one, no Plotly outside
`visualization`. The facade is one dependency and one `prelude`, and its
default enables every capability (opt out with `default-features = false` and
a capability feature). Pick the facade when you use most of the library or
want the convenience paths; pick components when a service needs one layer
and a short build. Both name the same types: `optionstratlib::model::Options`
*is* `optionstratlib_core::model::Options`.

## What is checked

- `make test-direct-component-examples` lints with Clippy (`-D warnings`),
  tests and runs every example; the same tests run in
  `cargo test --workspace`.
- `make tree-example-direct-<scenario>` prints the resolved graph of one
  example and asserts its `expect.toml`, and `make check-fixtures` asserts
  all of them together with the consumer fixtures. The graphs list
  `tracing-subscriber` on purpose: each example installs it as its own
  application logger, and no component crate resolves it.
- Every manifest is self-contained: explicit versions for every
  third-party dependency and nothing inherited from the workspace, so it can
  be copied into another project. The OptionStratLib crates are declared by
  path plus version (ADR-0001 D1); in your project keep the version and drop
  the path.
- `make check-direct-examples-packaged` copies each example out of the tree
  and builds, tests and runs the copy against the packaged component crates
  (the `.crate` files `make check-components` produces), through
  `[patch.crates-io]`, which is what a consumer gets once they are published.

Run one:

```sh
cargo run -p osl-example-direct-simulation
```
