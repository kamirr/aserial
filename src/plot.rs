use plotters::drawing::bitmap_pixel::BGRXPixel;
use plotters::prelude::*;
use std::slice::from_raw_parts_mut;

pub struct Plot {
    buf: Vec<u32>,
    size: (u32, u32),
    win: minifb::Window,
    last_frame: std::time::Instant,
}

impl Plot {
    /* create a new plotter */
    pub fn new() -> Self {
        let size = (800u32, 600u32);
        let buf = vec![0u32; (size.0 * size.1) as usize];
        let win = minifb::Window::new(
            "FT",
            size.0 as usize,
            size.1 as usize,
            minifb::WindowOptions::default(),
        )
        .unwrap();
        let last_frame = std::time::Instant::now();

        Plot {
            buf,
            size,
            win,
            last_frame,
        }
    }

    pub fn needs_refreshing(&self) -> bool {
        let tick = std::time::Duration::new(0, 1_000_000_000 / 60);
        self.last_frame.elapsed() > tick
    }

    /* plot the FFT */
    pub fn refresh(&mut self, caption: String, output: &[f32]) {
        self.draw(caption, output);
        self.render();

        /* mark the time of last refresh */
        self.last_frame = std::time::Instant::now();
    }

    /* draw the plot in a buffer */
    fn draw(&mut self, caption: String, output: &[f32]) {
        /* plotters needs a u8 buffer to write to, hence *
         * this cast from &mut[u32] to &mut[u8]          */
        let buf_u8: &mut [u8] = unsafe {
            from_raw_parts_mut(
                &mut self.buf[..] as *mut [u32] as *mut u8,
                (self.size.0 * self.size.1 * 4) as usize,
            )
        };

        /* like, eh, copied from an example from plotters repo */
        let root = BitMapBackend::<BGRXPixel>::with_buffer_and_format(buf_u8, self.size)
            .unwrap()
            .into_drawing_area();

        /* clears the plot */
        root.fill(&WHITE).unwrap();

        /* builds the chart */
        let mut chart = ChartBuilder::on(&root)
            .caption(&caption, ("sans-serif", 50).into_font())
            .margin(5)
            .x_label_area_size(30)
            .y_label_area_size(30)
            .build_ranged(30f32..1600f32, 0f32..0.1)
            .unwrap();
        chart.configure_mesh().draw().unwrap();

        /* take the norm of each number in FFTs output and plot it (in RED) */
        chart
            .draw_series(LineSeries::new(
                (30..1600).map(|n| (n as f32, output[n])),
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

    /* render the buffer to the screen */
    fn render(&mut self) {
        /* fill the window using the buffer buffer */
        self.win.update_with_buffer(&self.buf).unwrap();
    }
}
