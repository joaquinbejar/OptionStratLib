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
