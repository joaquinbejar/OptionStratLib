# optionstratlib-visualization

Charts for [OptionStratLib](https://github.com/joaquinbejar/OptionStratLib):
backend-neutral chart models, the `Graph` contract, its implementations for
the library's types (options, positions, curves, surfaces, random walks,
simulators and every concrete strategy), and an optional Plotly backend with
static PNG/SVG export, plus the terminal tables of option chains and
simulation statistics. It is the leaf of the workspace: it depends on
`optionstratlib-core`, `optionstratlib-math`, `optionstratlib-pricing`,
`optionstratlib-simulation`, `optionstratlib-market`,
`optionstratlib-strategies` and `optionstratlib-backtest`, and no other
OptionStratLib crate depends on it.

| Module          | Contents                                                                 |
|-----------------|--------------------------------------------------------------------------|
| `visualization` | `Graph`, `GraphData`, `GraphConfig`, series, surfaces, styles, `PlotBuilder`, `Plottable`, the graph implementations; Plotly trace builders behind `plotly` |
| `visualization::terminal` | `ChainReport` and `SimulationReport`: option chains and simulation statistics as bordered terminal tables, rendered to a `String` or printed to stdout on request |
| `error`         | `GraphError`                                                             |

## Place in the workspace

- **Depends on** `optionstratlib-core`, `-math`, `-pricing`, `-simulation`,
  `-market`, `-strategies` and `-backtest`.
- **Must not be a dependency** of any other OptionStratLib crate: it is the
  leaf of the layering, and only the `optionstratlib` facade depends on it;
  `make check-graph` enforces the layering (ADR-0001 D9).
- **In the facade:** `optionstratlib::visualization`, `GraphError` and the
  aggregate `Error` in `optionstratlib::error` and the root
  `impl_graph_for_payoff_strategy!`, under the facade feature `visualization`;
  the facade's `plotly` and `static_export` forward to the features below. The
  facade paths are the same types as the paths here; the [ownership
  map](https://github.com/joaquinbejar/OptionStratLib/blob/main/docs/ownership.md)
  lists every one with its feature.

Moving from 0.21? The [0.22 architecture and adoption
guide](https://github.com/joaquinbejar/OptionStratLib/blob/main/docs/migration-0.22.md)
maps each redesign to its replacement workflow; 0.22 keeps no
compatibility with 0.21.

## Example

A runnable program that depends on this crate directly, with the smallest
dependency set and no facade, is [`osl-example-direct-visualization`](https://github.com/joaquinbejar/OptionStratLib/tree/main/examples/direct/visualization); `make tree-example-direct-visualization`
asserts its resolved graph.

## Features

| Feature         | Default | Effect                                                                 |
|-----------------|---------|------------------------------------------------------------------------|
| `plotly`        | no      | Plotly rendering: `to_plot`, `write_html`, `show`, `render`, trace builders. Resolves no image-export or async package |
| `static_export` | no      | PNG and SVG export (`write_png`, `write_svg`) through `plotly_static`; implies `plotly` |

Without either feature the crate builds the chart data (`graph_data`,
`graph_config`) and the terminal tables, and resolves no Plotly package.
It is the only OptionStratLib crate that depends on a terminal-table
package (`prettytable-rs`).

## License

MIT
