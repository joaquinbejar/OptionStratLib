use crate::error::GraphError;
use crate::utils::file::prepare_file_path;
use crate::visualization::config::GraphConfig;
use crate::visualization::model::{GraphData, OutputType};
use crate::visualization::styles::PlotType;
use crate::visualization::utils::{make_scatter, make_surface, pick_color};
use plotly::ImageFormat;
use plotly::layout::Axis;
use plotly::{Layout, Plot, common};
use tracing::debug;

pub trait Graph {
    /// Return the raw data ready for plotting.
    fn graph_data(&self) -> GraphData;

    /// Optional per‑object configuration overrides.
    fn graph_config(&self) -> GraphConfig {
        GraphConfig::default()
    }

    /// Build a `plotly::Plot` according to data + config.
    fn to_plot(&self) -> Plot {
        let cfg = self.graph_config();
        let mut plot = Plot::new();

        match self.graph_data() {
            GraphData::Series(s) => {
                let mut series = s.clone();
                if let Some(legend) = &cfg.legend {
                    if let Some(label) = legend.first() {
                        series.name = label.clone();
                    }
                }
                plot.add_trace(make_scatter(&series));
            }
            GraphData::MultiSeries(list) => {
                for (idx, s) in list.into_iter().enumerate() {
                    let mut series = s;

                    if series.line_color.is_none() {
                        series.line_color = pick_color(&cfg, idx);
                    }

                    if let Some(legend) = &cfg.legend {
                        if idx < legend.len() {
                            series.name = legend[idx].clone();
                        }
                    }

                    plot.add_trace(make_scatter(&series));
                }
            }
            GraphData::Surface(surf) => {
                let mut surface = surf.clone();
                if let Some(legend) = &cfg.legend {
                    if let Some(label) = legend.first() {
                        surface.name = label.clone();
                    }
                }
                plot.add_trace(make_surface(&surface));
            }
        }

        let mut layout = Layout::new()
            .width(cfg.width as usize)
            .height(cfg.height as usize)
            .title(common::Title::from(&cfg.title))
            .show_legend(cfg.show_legend);

        if let Some(label) = cfg.x_label {
            layout = layout.x_axis(Axis::new().title(common::Title::from(&label)));
        }
        if let Some(label) = cfg.y_label {
            layout = layout.y_axis(Axis::new().title(common::Title::from(&label)));
        }
        if let Some(label) = cfg.z_label {
            layout = layout.z_axis(Axis::new().title(common::Title::from(&label)));
        }

        plot.set_layout(layout);
        plot
    }

    /// Helper to write PNG file with error handling
    /// #[cfg(feature = "kaleido")]
    fn write_png(&self, path: &std::path::Path) -> Result<(), GraphError> {
        unsafe {
            std::env::set_var("LC_ALL", "en_US.UTF-8");
            std::env::set_var("LANG", "en_US.UTF-8");
        }

        prepare_file_path(path)?;
        debug!("Writing PNG to: {}", path.display());
        let cfg = self.graph_config();

        self.to_plot().write_image(
            path,
            ImageFormat::PNG,
            cfg.width as usize,
            cfg.height as usize,
            1.0,
        );
        Ok(())
    }

    /// Helper to write HTML file with error handling
    fn write_html(&self, path: &std::path::Path) -> Result<(), GraphError> {
        prepare_file_path(path)?;
        self.to_plot().write_html(path);
        Ok(())
    }

    fn write_svg(&self, path: &std::path::Path) -> Result<(), GraphError> {
        prepare_file_path(path)?;
        let cfg = self.graph_config();
        self.to_plot().write_image(
            path,
            ImageFormat::SVG,
            cfg.width as usize,
            cfg.height as usize,
            1.0,
        );
        Ok(())
    }

    /// Show the plot in browser
    fn show(&self) {
        self.to_plot().show();
    }

    /// One‑stop rendering with error propagation.
    fn render(&self, output: OutputType) -> Result<(), GraphError> {
        match output {
            OutputType::Png(path) => self.write_png(path)?,
            OutputType::Svg(path) => self.write_svg(path)?,
            OutputType::Html(path) => self.write_html(path)?,
            OutputType::Browser => self.show(),
        }
        Ok(())
    }

    /// Generate interactive HTML with hover info + annotations.
    fn to_interactive_html(&self, path: &std::path::Path) -> Result<(), GraphError> {
        self.write_html(path)?;
        Ok(())
    }
}

pub trait GraphType {
    fn plot_type() -> PlotType;
}
