use rustfft::num_complex::Complex;
use plotters::drawing::bitmap_pixel::BGRXPixel;
use plotters::prelude::*;

/* plot the FFT */
pub fn plot(caption: String, buf: &mut [u8], output: &Vec<Complex<f32>>) {
    /* window size */
    let size: (u32, u32) = (800, 600);

    /* like, eh, copied from an example from plotters repo */
    let root = BitMapBackend::<BGRXPixel>::with_buffer_and_format(&mut buf[..], size)
        .unwrap()
        .into_drawing_area();

    /* clears the plot */
    root
        .fill(&WHITE)
        .unwrap();

    /* builds the chart */
    let mut chart = ChartBuilder::on(&root)
        .caption(&caption, ("sans-serif", 50).into_font())
        .margin(5)
        .x_label_area_size(30)
        .y_label_area_size(30)
        .build_ranged(30f32..1600f32, 0f32..2f32)
        .unwrap();
    chart.configure_mesh().draw().unwrap();

    /* take the norm of each number in FFTs output and plot it (in RED) */
    chart
        .draw_series(LineSeries::new(
            (30..1600).map(|n| {
                (n as f32, output[n].norm() as f32)
            }),
            &RED,
        ))
        .unwrap();

    /* copied as well, I guess this part does the rendering and stuff */
    chart
        .configure_series_labels()
        .background_style(&WHITE.mix(0.8))
        .border_style(&BLACK)
        .draw()
        .unwrap();
}
