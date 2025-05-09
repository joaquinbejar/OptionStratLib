use plotly::{common, Layout, Plot};
use plotly::layout::Axis;
use crate::error::GraphError;
use crate::visualization::config::GraphConfig;
use crate::visualization::model::{GraphData, OutputType};
use crate::visualization::styles::PlotType;
use crate::visualization::utils::{make_scatter, make_surface, pick_color};

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
                plot.add_trace(make_scatter(&s));
            }
            GraphData::MultiSeries(list) => {
                for (idx, s) in list.into_iter().enumerate() {
                    let mut series = s;
                    if series.line_color.is_none() {
                        series.line_color = pick_color(&cfg, idx);
                    }
                    plot.add_trace(make_scatter(&series));
                }
            }
            GraphData::Surface(surf) => {
                plot.add_trace(make_surface(&surf));
            }
        }

        let mut layout = Layout::new()
            .width(cfg.width as usize)
            .height(cfg.height as usize)
            .title(common::Title::from(&cfg.title));

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
    fn write_png(
        &self,
        path: &std::path::Path,
        width: u32,
        height: u32,
    ) -> Result<(), GraphError> {
        use plotly::ImageFormat;
        self.to_plot().write_image(path, ImageFormat::PNG, width as usize, height as usize, 1.0);
        Ok(())
    }

    /// Helper to write HTML file with error handling
    fn write_html(&self, path: &std::path::Path) -> Result<(), GraphError> {
        self.to_plot().write_html(path);
        Ok(())
    }

    /// Show the plot in browser
    fn show(&self) {
        self.to_plot().show();
    }

    /// One‑stop rendering with error propagation.
    fn render(&self, output: OutputType) -> Result<(), GraphError> {
        let cfg = self.graph_config();
        match output {
            OutputType::Png(path) => self.write_png(path, cfg.width, cfg.height)?,
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