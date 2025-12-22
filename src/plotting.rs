//! Plotting data.

use std::{error::Error, ops::Range};

use plotters::prelude::*;

pub struct Plot<'a> {
    root: DrawingArea<BitMapBackend<'a>, plotters::coord::Shift>,
    chart: ChartContext<
        'a,
        BitMapBackend<'a>,
        Cartesian2d<plotters::coord::types::RangedCoordf32, plotters::coord::types::RangedCoordf32>,
    >,
    x_range: Range<f32>,
    y_range: Range<f32>,
    x_step: f32,
}

impl<'a> Plot<'a> {
    pub fn new(
        file_name: &'a str,
        caption: impl AsRef<str>,
        x_range: Range<f32>,
        y_range: Range<f32>,
        x_step: f32,
    ) -> Result<Plot<'a>, Box<dyn Error>> {
        let root = BitMapBackend::new(file_name, (640, 480)).into_drawing_area();
        root.fill(&WHITE)?;
        let mut chart = ChartBuilder::on(&root)
            .caption(caption, ("sans-serif", 50).into_font())
            .margin(5)
            .x_label_area_size(30)
            .y_label_area_size(30)
            .build_cartesian_2d(x_range.clone(), y_range.clone())?;

        chart.configure_mesh().draw()?;

        Ok(Self {
            root,
            chart,
            x_range,
            y_range,
            x_step,
        })
    }
    pub fn plot(
        &mut self,
        mut func: impl FnMut(f32) -> f32,
        style: impl Into<ShapeStyle> + Clone + 'a,
        caption: impl AsRef<str>,
    ) -> Result<(), Box<dyn Error>> {
        self.chart
            .draw_series(LineSeries::new(
                self.x_range
                    .clone()
                    .step(self.x_step)
                    .values()
                    .map(|x| (x, func(x))),
                style.clone(),
            ))?
            .label(caption.as_ref())
            .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], style.clone()));
        Ok(())
    }
    pub fn plot_points(
        &mut self,
        mut func: impl FnMut(f32) -> f32,
        style: impl Into<ShapeStyle> + Clone + 'a,
        caption: impl AsRef<str>,
    ) -> Result<(), Box<dyn Error>> {
        self.chart
            .draw_series(PointSeries::<(f32, f32), _, Circle<_, _>, i32>::new(
                self.x_range
                    .clone()
                    .step(self.x_step)
                    .values()
                    .map(|x| (x, func(x))),
                5i32,
                style.clone(),
            ))?
            .label(caption.as_ref())
            .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], style.clone()));
        Ok(())
    }
}
