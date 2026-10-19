# Plotters for native PNG and SVG export: decision report (#920)

**Outcome: the experiment is complete, and migration is not yet approved by
its own criteria.** Five of the six migration criteria of #920 are met. The
sixth, SVG text portability, is **unmet**: the embedded font works in Blink
and WebKit, but librsvg ignores it and uses its own font. The current Plotly
SVG fails that criterion in all three engines, but that does not waive it.

**Recommendation: one more bounded experiment, SVG text as outlines (paths),
then migrate** the PNG/SVG export to the native renderer, keeping Plotly for
interactive HTML, under the conditions in
[Migration conditions](#migration-conditions). SVG portability is one of
those conditions and blocks the migration; it is not an optional follow-up.

> **Update (#923).** That experiment is done: SVG text drawn as outlines met
> the portability condition in Blink, WebKit and librsvg, with tradeoffs and
> the remaining conditions in [REPORT-outlines.md](REPORT-outlines.md). The
> rest of this report keeps its original measurements and provenance; the
> only edits are this note, the cross-references and the corrected cause of
> the loose PNG spacing below.

This is the decision report of the prototype; nothing in a published crate
changed. Measured on 2026-10-18 at `e4b3159` (Rust code unchanged since
`8b742d8`; the binaries were built there), macOS 27.0.1
arm64, `rustc 1.99.1`, Chrome and chromedriver 155.0.8059.85, release
builds, plotters 0.3.7. Open Sans is **not** installed on the machine.

## What was built

`prototypes/plotters-export` (`osl-prototype-plotters-export`, workspace
member, `publish = false`):

- `src/render.rs`: `write_png`, `write_svg`, `render_svg`. They read the same
  `GraphData` / `GraphConfig` as `Graph::to_plot`, with the same rules (legend
  labels rename series, `ColorScheme` colours series without a colour), and
  Plotly's defaults where the data leaves a choice (plotly.js colourway, 2px
  lines, 6px markers, 16px text labels, 10px markers for zero-width "label"
  markers, zero lines, `−` tick signs, Open Sans). 2D: `Series`,
  `MultiSeries`, all four `TraceMode`s, legend column on the right. 3D:
  filled cell polygons in painter's order over a fixed projection, axis
  titles and a Viridis colour bar.
- `src/svg.rs`: SVG text options: embed the font as a base64 `@font-face`
  (default), or name the family with a `sans-serif` fallback.
- `src/fixtures.rs`: the three charts, deterministic.
- `src/bin/compare.rs`, `measure.py`, `render_svg_viewers.sh`: artifacts,
  timings, leak scenarios, viewer renders.
- `benches/native_export.rs`: Criterion benchmarks of the native path.
- `tests/export.rs` plus unit tests: files, every documented error, 81
  figure sizes per chart with no panic, small numeric scales.

The three charts:

| Case | Data | What it exercises |
|---|---|---|
| `payoff` | a real `BullCallSpread`, through the library's own `Graph` impl | negative and positive P/L, zero crossing, two slope changes, zero line, current price marker, text labels, `$` in the title |
| `curves` | Black-Scholes call value over 70..130 for σ = 10..40 %, plus intrinsic value | five named series, legend, `LinesMarkers`, Unicode |
| `surface` | implied volatility over 13 strikes × 7 expiries with skew, smile and an inverted short end | a complete grid, 3D, colour scale, three axis titles |

## Artifacts

All twelve combinations (3 charts × PNG/SVG × 2 renderers) were produced, plus
the Plotters SVG without an embedded font, under `artifacts/`:
`<case>-<renderer>.<ext>`, `<case>-plotters.plain.svg`, and `artifacts.json`
with sizes. Regenerate with:

```sh
WEBDRIVER_PATH=<chromedriver 155> BROWSER_PATH=<chrome 155> \
  cargo run --release -p osl-prototype-plotters-export --features plotly-baseline \
  --bin compare -- artifacts prototypes/plotters-export/artifacts
```

### Visible differences

| | Plotly (current) | Plotters (prototype) |
|---|---|---|
| Payoff title | **Mangled**: plotly_static loads MathJax, the two `$` in the strategy title are read as LaTeX and the title renders as italic math (`5820ShortCall…`). The HTML export does not load MathJax and is unaffected. | Literal title |
| Surface axis titles | **Lost**: `to_plot` sets `layout.x_axis`/`y_axis`/`z_axis`, which a 3D scene ignores; the image shows `x`, `y`, `z` | `Strike`, `Days to expiry`, `Implied volatility`, plus a titled colour bar |
| Surface rendering | Smooth, lit WebGL surface, default red-blue scale | Flat-shaded cells with thin edges, Viridis scale; no lighting |
| Surface SVG | **A PNG embedded in an SVG**: WebGL cannot be exported as vectors | Real vector polygons |
| Tick labels | `20`, `−20` | Same format (`format_number`) |
| Text in PNG | Chrome's text shaping | ab_glyph rasterisation: readable, no kerning (the font's kerning is GPOS only, which ab_glyph does not read), and some glyph pairs (`52.66`, `SP 500`) look loosely spaced because the rasteriser drops side bearings and truncates pen positions (corrected in #923; the first version blamed the missing kerning) |
| Margins, legend | Plotly layout | Close, not identical; pixel identity was not a goal |
| `ColorScheme` on surfaces | Ignored (plotly default scale) | Ignored (Viridis) |
| `LineStyle` | Ignored by `to_plot` | Ignored too, for parity |

No plotted value, series, axis meaning or unit was lost in the Plotters
output. Plotly loses two of them today (payoff title, surface axis titles).

### Behaviour on bad data (deliberate differences)

| Input | Plotly (`make_surface`, plotly.js) | Plotters prototype |
|---|---|---|
| Surface with a missing grid point | fills it with **zero** | `IncompleteSurface` error |
| Surface with a repeated point | keeps the last value | `DuplicateSurfacePoint` error |
| Empty surface | draws a 2×2 surface of zeros | `DegenerateSurface` error |
| `x`/`y` lengths differ | plotly.js truncates to the shorter | `LengthMismatch` / `SurfaceLengthMismatch` error |
| Colour name (`"red"`) | accepted | `InvalidColor` (only `#RGB`/`#RRGGBB`); see migration conditions |
| Empty series | empty axes | empty axes |
| Constant series | flat line, widened axis | flat line, widened axis |
| Zero width or height | Plotly error | `InvalidDimensions` |
| Title, legend or colour bar leave no pixels to draw in | drawn, clipped | `NoRoom` |
| `line_width` not finite or outside 0..=100 px | drawn (Plotly clamps) | `InvalidLineWidth`; an infinite width panicked in the Plotters rasteriser before the review |
| Tiny but real range (0 to 1e-18) | own scale | own scale: 2D axes under 1e-9 are drawn rescaled and labelled with real values, tick precision follows the step (scientific below 1e-4) |
| Surface figure under 240×160 px | drawn, cramped | `TooSmall` (no layout fits the axes and colour bar; review found a 640×20 surface overflowing the colour-bar arithmetic) |
| Unwritable path | error | `Io` error (parent directories are created, as `prepare_file_path` does) |

## Measurements

`results/timing.json`, `results/timing-native-unsandboxed.json`,
`results/leaks.json`; commands under [Reproduce](#reproduce).

- **Cold** = one export in a fresh process, wall time from outside: process
  start, renderer start-up (for Plotly: chromedriver, Chrome, page load),
  export, shutdown. 10 runs per Plotly combination, 20 per Plotters one.
- **In-process** = the same export repeated inside one process (200 Plotters
  iterations, 10 Plotly), first iteration excluded from the statistics.
  `Plot::write_image` builds a new exporter per image, so Plotly pays the
  browser start-up on every image here too; a reused `StaticExporter` would
  be faster, but the library does not reuse one and that was not measured.
- Plotly runs went one at a time; no other export ran during them.

| Case | Format | Plotly cold, median (p90) | Plotters cold, median (p90) | Plotly in-process median | Plotters in-process median | Plotly bytes | Plotters bytes |
|---|---|---|---|---|---|---|---|
| payoff | PNG | 1,625 ms (3,090) | 11.1 ms (12.5) | 1,201 ms | 2.2 ms | 70,689 | 63,709 |
| payoff | SVG | 1,401 ms (1,590) | 11.0 ms (11.8) | 1,184 ms | 0.16 ms | 46,072 | 205,891 |
| curves | PNG | 1,178 ms (1,263) | 11.3 ms (12.0) | 1,093 ms | 1.7 ms | 93,806 | 74,761 |
| curves | SVG | 1,252 ms (1,307) | 11.5 ms (12.2) | 1,084 ms | 0.17 ms | 26,730 | 212,255 |
| surface | PNG | 2,013 ms (2,119) | 11.1 ms (11.8) | 1,823 ms | 2.2 ms | 137,383 | 142,453 |
| surface | SVG | 1,965 ms (2,050) | 11.1 ms (11.9) | 1,812 ms | 0.25 ms | 374,699 | 234,533 |

- Latency improves in every case and format: about 100-180× cold and
  540-7,500× in-process. No regression was observed. A cold Plotters run is
  dominated by process start-up (about 11 ms here; earlier runs on the same
  machine measured 5-14 ms medians).
- The Plotters SVG is larger because it embeds the 148 KB font (≈197 KB of
  base64). Without it (`plain.svg`) the files are 9 KB, 15 KB and 38 KB, and
  exports take 6.5-11.2 ms cold and 0.09-0.19 ms in-process. Subsetting the
  font would shrink the embedded variant.
- Under `sandbox-exec` with network access **and process creation denied**,
  all 90 native cold runs succeeded (the sandbox itself adds about 10-30 ms
  per run, `results/timing.json`); the Plotly export fails there at once
  (`Failed to spawn 'chromedriver': Operation not permitted`).
- Release binary: 2.6 MB native vs 16 MB with the Plotly baseline.

### Leaked Chrome processes

In the sequential runs above no process leaked (120 Plotly exports: 60 cold,
60 in-process). The
leaks happen when the exporter does not get to close its WebDriver session
(`results/leaks.json`, 5 rounds of each scenario). Each export
runs in a process group of its own, which chromedriver and Chrome inherit,
and `measure.py` also samples its process tree every 0.1 s; whatever of the
group or tree is still alive two seconds after the export ended is a leak,
and is terminated (`killpg` on the export's own group). Ownership comes from
the group and the recorded ancestry, never from a command-line pattern, so a
browser another program runs at the same time is never counted or signalled.
`measure.py --self-test` checks this with stand-in processes: foreign ones
survive, and leftovers are found after a late failure, an immediate failure
whose child is reparented before any sample, a timeout, and a failure to
start the second of two commands.

| Scenario | Plotly result |
|---|---|
| SIGINT to the exporting process only | processes left running in **5 of 5** rounds (chromedriver, Chrome and helpers: 7 each) |
| SIGTERM to the exporting process only | **5 of 5** (7 each) |
| SIGKILL to the exporting process only | **5 of 5** (7 each) |
| SIGINT to its whole process group (a terminal Ctrl-C) | 0 of 5: chromedriver and Chrome receive it too and exit |
| 2 exports in separate processes at once | 0 of 5 (1 of 5 in an earlier run of the same scenario) |
| 4 exports in separate processes at once | **3 of 5** (7-28 processes; the cross-process form of #724: the mutex in `graph.rs` is per process) |

So an interactive Ctrl-C cleans up, but everything that stops only the
exporting process does not: a test runner's timeout, `kill <pid>`, an IDE's
stop button, a supervisor restarting a service. Parallel test binaries or
worktrees exporting at once leak as well.

Earlier on the same machine, with chromedriver in its own process group,
`SIGKILL` to chromedriver alone left all 8 Chrome processes running,
reparented to launchd, and `killpg` on the group removed them: an interim
fix in `graph.rs` is possible, but it cannot cover the exporting process
itself being killed. The native renderer starts no process, so it has
nothing to leak.

## SVG portability

`render_svg_viewers.sh` rendered every SVG in three engines on a machine
without Open Sans (`artifacts/viewers/`, versions in `viewers.txt`). The
renders predate the review fixes; the Plotters SVGs are byte-identical
before and after them, and the Plotly SVGs differ only in their random
clip-path ids:

| SVG | Blink (Chrome 155) | WebKit (Quick Look, macOS 27) | librsvg 2.63.3 |
|---|---|---|---|
| Plotters, embedded font | Open Sans | Open Sans | **fallback sans-serif** (librsvg ignores `@font-face`) |
| Plotters, no font | fallback sans-serif | fallback sans-serif | fallback sans-serif |
| Plotly | Verdana (from its `'Open Sans', verdana, arial` stack) | Verdana | viewer font |

No label clipped in any engine at the tested sizes; the layout was measured
with Open Sans metrics, so a fallback font can shift labels by a few pixels.
By the issue's rule ("if text remains dependent on viewer fonts, mark this
criterion unmet") the criterion is **unmet**: librsvg still uses its own
font. The Plotly baseline is unmet in all three engines. Converting text to
outlines (paths) would meet it everywhere, at the cost of selectable and
accessible text (an SVG `<title>`/`aria-label` can carry the strings); it is
the next bounded experiment, and a condition of the migration. (#923 ran it:
the outlines mode is portable in all three engines; see
[REPORT-outlines.md](REPORT-outlines.md), which also replaces the Quick Look
thumbnail used above with an exact-size WebKit render.)

## Dependencies

- Native tree (`cargo tree -p osl-prototype-plotters-export -e normal`): 105
  packages, 106 with build dependencies; it contains no `plotly`,
  `plotly_static`, `tokio`, `reqwest`, `hyper`, `h2`, `fantoccini`,
  `webdriver-downloader`, `rustls-pemfile`, `font-kit` or fontconfig binding.
  With `plotly-baseline`: 255 normal, 337 with build dependencies.
- Plotters brings: `plotters`, `plotters-backend`, `plotters-bitmap`,
  `plotters-svg`, `ab_glyph`, `ab_glyph_rasterizer`, `owned_ttf_parser`,
  `ttf-parser`, `image 0.24`, `png 0.17`, `jpeg-decoder`, `color_quant`,
  `fdeflate`, `flate2`, `miniz_oxide`, `zlib-rs`, `bytemuck`, `byteorder`,
  `crc32fast`, `simd-adler32`, `adler2`, `libm`, `once_cell`, `bitflags 1`.
  Plus `base64` for the font embedding.
- **`cargo audit`: `ttf-parser 0.25.1` is unmaintained (RUSTSEC-2026-0192)**,
  reached through `ab_glyph` (and equally through Plotters' `ttf` feature).
  It parses only the font compiled into the binary, never input. On the
  owner's decision this PR adds a `.cargo/audit.toml` exception with that
  rationale, so `main`'s audit stays green with the prototype in the
  workspace. cargo-audit ignores by advisory ID across the whole lockfile,
  so the waiver itself is not scoped; its justification is, to the
  unpublished prototype, and it must be re-evaluated if the migration (or
  anything else) brings `ttf-parser` into a published crate.
- Plotters 0.3.7 (September 2024) is the latest release; `colormaps` does
  not compile without `full_palette` in it, hence that feature.
- The font asset is Open Sans Regular from `googlefonts/opensans`
  (`fonts/ttf/OpenSans-Regular.ttf`, repository head `bd7e376` when
  downloaded), SHA-256 `c53aceea…f8eb59`, SIL OFL 1.1 (`assets/OFL.txt`); the
  OFL allows bundling and embedding provided the licence travels with it.

## Criteria (#920)

| Criterion | Status | Evidence |
|---|---|---|
| All twelve reference artifacts available and valid | **met** | `artifacts/`, `tests/export.rs` (PNG signature and IHDR size, SVG structure) |
| Values, axis meaning, units, series identity; readable, unclipped labels | **met** | visible differences table; Plotly itself loses the payoff title and the surface axis titles |
| 3D surface keeps shape and interpretable height; projection, occlusion, colour scale documented | **met**, with limitations | fixed yaw 0.7 / pitch 0.5 / scale 0.8; painter's algorithm by cell centroid depth (correct for a single-valued surface over a grid, can misorder strongly folded ones); Viridis colour bar; flat shading, no lighting, no camera control |
| SVG portability in independent environments | **unmet** (met by the #923 outlines mode) | met in Blink and WebKit, unmet in librsvg; Plotly unmet in all three; #923: [REPORT-outlines.md](REPORT-outlines.md) |
| Export without Chrome, ChromeDriver or network; no `plotly_static` in the native harness | **met** | sandbox runs, dependency tree |
| Latency improves for every case and format | **met** | measurements table |

## Limitations and maintenance cost

- A second renderer to maintain next to `to_plot`: about 800 lines of code
  in `render.rs`, `svg.rs` and `error.rs`, tests and comments excluded.
  Every change to `GraphData`, `GraphConfig` or the Plotly mapping has to be
  mirrored, or one renderer drifts.
- 3D is basic: one fixed camera, flat shading, painter's ordering.
- Text quality in PNG is below Chrome's: no kerning, and uneven spacing from
  the rasteriser dropping side bearings and truncating pen positions.
- `ttf-parser` unmaintained advisory; Plotters' slow release cadence.
- Colour names are not parsed yet (since #925 they are; see condition 3).
- Viewer-dependent SVG text in librsvg until text becomes outlines; this
  blocks the migration (condition 1 below).

## Migration conditions

Only after this report is accepted, and as #920 lists:

1. **Gate: SVG text portability.** Render SVG text as outlines (or another
   approach) and show the same text in Blink, WebKit and librsvg on a
   machine without the font. Until then the migration is not approved.
   *Met by the outlines mode in #923 ([REPORT-outlines.md](REPORT-outlines.md));
   the migration decision and conditions 2 to 8 remain open.*
2. Keep `Graph::write_png` / `write_svg` signatures and error contracts,
   `Graph::to_plot`, interactive HTML and the public Plotly helpers. Keep
   `static_export` implying `plotly` at component and facade level, and drop
   the `plotly/static_export_default` edge. Decide whether the data errors
   above become `GraphError` variants (a behaviour change for surfaces that
   are zero-filled today) and document it in the release notes.
3. Parse CSS colour names (`ColorScheme::Custom` and `line_color` accept them
   with Plotly today).
   *Done in the prototype (#925): `src/color.rs` accepts the CSS named
   colours, hex with or without alpha, `rgb()`/`rgba()` and `hsl()`/`hsla()`,
   and rejects anything else with a typed error.*
4. Re-evaluate the RUSTSEC-2026-0192 exception (added here for the
   unpublished prototype) for the published crate Plotters would enter, or
   choose a text path without `ttf-parser`.
5. Native rendering tests in PR CI before retiring the browser workflow;
   keep every example `make smoke-examples SMOKE_EXPORT=1` covers.
6. Recheck feature trees and consumer fixtures; remove the two exporter
   advisory exceptions only once the lockfile and audit confirm their paths
   are gone.
7. docs.rs can then build `static_export` (no build script needs a browser);
   update the feature documentation, migration notes and an ADR.
8. Worth their own issues, not blocking: font subsetting (SVG size),
   surface lighting.

## Reproduce

```sh
# Native binary (no plotly_static linked) and baseline binary.
cargo build --release -p osl-prototype-plotters-export
cp target/release/compare /tmp/compare-native
WEBDRIVER_PATH=<chromedriver> BROWSER_PATH=<chrome> \
  cargo build --release -p osl-prototype-plotters-export --features plotly-baseline
cp target/release/compare /tmp/compare-baseline

# Timings (add --sandbox on macOS for the no-network, no-fork proof).
WEBDRIVER_PATH=<chromedriver> BROWSER_PATH=<chrome> python3 -I \
  prototypes/plotters-export/measure.py --native /tmp/compare-native \
  --baseline /tmp/compare-baseline --out /tmp/osl-920 --cold 10 --warm 200 --plotly-warm 10
# Leak scenarios.
WEBDRIVER_PATH=<chromedriver> BROWSER_PATH=<chrome> python3 -I \
  prototypes/plotters-export/measure.py --mode leaks --baseline /tmp/compare-baseline \
  --out /tmp/osl-920 --repeat 5
# SVG viewers (macOS: Chrome, rsvg-convert, qlmanage).
prototypes/plotters-export/render_svg_viewers.sh "$PWD/prototypes/plotters-export/artifacts" /tmp/osl-920-viewers
# Leak attribution self-test (no browser needed).
python3 -I prototypes/plotters-export/measure.py --self-test
# Native Criterion benchmarks.
cargo bench -p osl-prototype-plotters-export --bench native_export
```
