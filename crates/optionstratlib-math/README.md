# optionstratlib-math

Generic numerical containers and algorithms of
[OptionStratLib](https://github.com/joaquinbejar/OptionStratLib): 2D curves,
3D surfaces, and the construction, interpolation, arithmetic and metric
machinery they share. Nothing here knows about option chains, Greeks, pricing
or plotting; the crate depends only on `optionstratlib-core` and general
numeric crates.

| Module       | Contents                                                                  |
|--------------|---------------------------------------------------------------------------|
| `curves`     | `Curve`, `Point2D`, `Curvable`, `StatisticalCurve`                         |
| `surfaces`   | `Surface`, `Point3D`, `Surfacable`                                         |
| `geometrics` | Construction, linear / bilinear / cubic / spline interpolation, arithmetic, metric extraction |
| `error`      | `CurveError`, `SurfaceError`, `InterpolationError`, `MetricsError`         |

Coordinates are `rust_decimal::Decimal`, and arithmetic on them is checked.

## Place in the workspace

- **Depends on** `optionstratlib-core`.
- **Must not depend on** `optionstratlib-pricing`, `-simulation`, `-market`,
  `-analytics`, `-strategies`, `-backtest` and `-visualization`; `make
  check-graph` enforces the layering (ADR-0001 D9).
- **In the facade:** `optionstratlib::{curves, surfaces, geometrics}` and the
  math errors in `optionstratlib::error`, under the facade feature `math`. The
  facade paths are the same types as the paths here; the [ownership
  map](https://github.com/joaquinbejar/OptionStratLib/blob/main/docs/ownership.md)
  lists every one with its feature.

<!-- #553: link the 0.21 to 0.22 migration guide here -->

## Minimal example

```rust
use optionstratlib_math::curves::{Curve, Point2D};
use std::collections::BTreeSet;

fn main() {
    let points: BTreeSet<Point2D> = [(0, 0), (1, 2), (2, 4)]
        .into_iter()
        .map(|(x, y)| Point2D::new(x, y))
        .collect();
    let curve = Curve::new(points);
    assert_eq!(curve.points.len(), 3);
}
```

## Runnable example

A runnable program that depends on this crate directly, with the smallest
dependency set and no facade, is [`osl-example-direct-math`](https://github.com/joaquinbejar/OptionStratLib/tree/main/examples/direct/math); `make tree-example-direct-math`
asserts its resolved graph.

## Features

| Feature  | Default | Effect                                                              |
|----------|---------|---------------------------------------------------------------------|
| `schema` | no      | Derives `utoipa::ToSchema` on math types and enables `optionstratlib-core/schema` |

Option-specific projections (`BasicCurves`, `BasicSurfaces`) live in the
analytics layer and plotting in visualization; most users get all of it
through the `optionstratlib` facade.

## License

MIT
