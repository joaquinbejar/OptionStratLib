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

## Features

| Feature  | Default | Effect                                                              |
|----------|---------|---------------------------------------------------------------------|
| `schema` | no      | Derives `utoipa::ToSchema` on math types and enables `optionstratlib-core/schema` |

Option-specific projections (`BasicCurves`, `BasicSurfaces`) live in the
analytics layer and plotting in visualization; most users get all of it
through the `optionstratlib` facade.

## License

MIT
