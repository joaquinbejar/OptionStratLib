//! Criterion benchmarks of the native exporter (#920).
//!
//! Each case rendered to PNG (file) and SVG (string, with and without the
//! embedded font), fixtures built once outside the measured loop. The
//! Plotly baseline is not benchmarked here: one Plotly export takes seconds
//! and needs Chrome, so it is timed by the `compare` binary and
//! `measure.py` instead, never by a normal benchmark run.
//!
//! Run with `cargo bench -p osl-prototype-plotters-export --bench native_export`.

use criterion::{Criterion, criterion_group, criterion_main};
use osl_prototype_plotters_export::fixtures::Case;
use osl_prototype_plotters_export::{SvgText, render_svg, write_png};
use std::hint::black_box;

fn native_export(c: &mut Criterion) {
    let dir = std::env::temp_dir().join(format!("osl-plotters-bench-{}", std::process::id()));
    let mut group = c.benchmark_group("native_export");
    for case in Case::ALL {
        let Ok(fixture) = case.build() else {
            tracing::error!(case = case.slug(), "fixture failed to build; skipped");
            continue;
        };
        let png = dir.join(format!("{}.png", case.slug()));
        group.bench_function(format!("{}/png", case.slug()), |b| {
            b.iter(|| write_png(black_box(&fixture.data), black_box(&fixture.config), &png))
        });
        group.bench_function(format!("{}/svg", case.slug()), |b| {
            b.iter(|| {
                render_svg(
                    black_box(&fixture.data),
                    black_box(&fixture.config),
                    SvgText::EmbeddedFont,
                )
            })
        });
        group.bench_function(format!("{}/svg-plain", case.slug()), |b| {
            b.iter(|| {
                render_svg(
                    black_box(&fixture.data),
                    black_box(&fixture.config),
                    SvgText::ViewerFont,
                )
            })
        });
    }
    group.finish();
    if let Err(error) = std::fs::remove_dir_all(&dir) {
        tracing::warn!(%error, dir = %dir.display(), "bench output left behind");
    }
}

criterion_group!(benches, native_export);
criterion_main!(benches);
