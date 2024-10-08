use anyhow::Error;
use plotters::prelude::*;
use std::ops::Range;
use std::path::Path;
use std::path::PathBuf;

pub struct Line {
    data_points: Vec<(f64, f64)>,
    style: ShapeStyle,
    label: String,
    dashed: bool,
}

pub struct Plotter {
    output_path: PathBuf,
    size: (u32, u32),
    range_y: Range<f64>,
    range_x: Range<f64>,
    lines: Vec<Line>,
}

impl Line {
    pub fn new(
        data_points: impl IntoIterator<Item = (f64, f64)>,
        style: impl Into<ShapeStyle>,
        label: impl Into<String>,
        dashed: bool,
    ) -> Self {
        Self {
            data_points: data_points.into_iter().collect(),
            style: style.into(),
            label: label.into(),
            dashed,
        }
    }
}

impl Plotter {
    pub fn new<P: AsRef<Path>>(
        output_path: P,
        size: (u32, u32),
        viewport: (Range<f64>, Range<f64>),
        lines: impl IntoIterator<Item = Line>,
    ) -> Self {
        Self {
            output_path: output_path.as_ref().to_path_buf(),
            size,
            range_y: viewport.1,
            range_x: viewport.0,
            lines: lines.into_iter().collect(),
        }
    }

    pub fn draw(self) -> Result<(), Error> {
        let root = SVGBackend::new(&self.output_path, self.size)
            .into_drawing_area();
        root.fill(&WHITE)?;

        let label_size = {
            let (x, y) = self.size;
            let min = x.min(y);
            min / 30
        };
        let mut chart = ChartBuilder::on(&root)
            .margin(label_size)
            .x_label_area_size(label_size)
            .y_label_area_size(label_size)
            .build_cartesian_2d(self.range_x, self.range_y)?;

        chart.configure_mesh().draw()?;

        for line in self.lines {
            if line.dashed {
                chart.draw_series(DashedLineSeries::new(line.data_points, 5, 5, line.style))
            } else {
                chart.draw_series(LineSeries::new(line.data_points, line.style))
            }?
            .label(line.label)
            .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], line.style));
        }

        chart
            .configure_series_labels()
            .background_style(&WHITE.mix(0.8))
            .border_style(&BLACK)
            .draw()?;

        root.present()?;

        Ok(())
    }
}
