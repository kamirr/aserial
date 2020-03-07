mod multi_sine_wave;
mod frame;

#[allow(dead_code)]
fn play() {
    use frame::*;

    let mut fb = FrameBuilder::new(vec![400, 700, 800]);

    let device = rodio::default_output_device().unwrap();
    let time_per_sample = 500000000;

    let mut play_impl = |b1, b2, b3| {
        let s = fb.make_frame(&[b1, b2, b3]).play(&device);
        std::thread::sleep(std::time::Duration::new(0, time_per_sample));
        s.stop();
        std::thread::sleep(std::time::Duration::new(0, time_per_sample));
    };

    for b in &[false, true] {
        for c in &[false, true] {
                play_impl(true, *b, *c);
        }
    }
}

fn window_fn(f: f32, n: usize, n_total: usize) -> f32 {
    f * (3.1415 * n as f32 / n_total as f32).sin().powi(2)
}

#[allow(dead_code)]
fn read() {
    use plotters::drawing::bitmap_pixel::BGRXPixel;
    use plotters::prelude::*;
    use rustfft::num_complex::Complex;
    use rustfft::num_traits::Zero;

    let file = std::fs::File::open("rec_new.mp3").unwrap();
    let source = rodio::Decoder::new(std::io::BufReader::new(file)).unwrap();
    let samples: Vec<f32> = source.map(|i| i as f32 / 32768.0).collect();

    let mut i = 0;

    let step = 500usize;
    let channels = 2usize;
    let window_size = 8820usize;
    let rate = 44100usize;

    let mut buf = vec![0u8; 800 * 600 * 4];
    let mut mfb_window = minifb::Window::new(
        "FT",
        800,
        600,
        minifb::WindowOptions::default(),
    ).unwrap();

    let mut then = std::time::Instant::now();
    let mut planner = rustfft::FFTplanner::new(false);
    let mut output: Vec<Complex<f32>> = vec![Complex::zero(); window_size];

    for window in samples.windows(window_size).step_by(step) {
        i += 1;

        let mut input: Vec<Complex<f32>> = window
            .iter()
            .enumerate()
            .map(|(i, f)| Complex::<f32>::new(window_fn(*f, i, window_size), 0.0))
            .collect();

        planner
            .plan_fft(window_size)
            .process(&mut input, &mut output);

        {
            let size: (u32, u32) = (800, 600);
            let caption = format!("t={:.2}s", i as f32 / (rate as f32 * channels as f32) * step as f32);

            let root = BitMapBackend::<BGRXPixel>::with_buffer_and_format(&mut buf[..], size)
                .unwrap()
                .into_drawing_area();

            root
                .fill(&WHITE)
                .unwrap();

            let mut chart = ChartBuilder::on(&root)
                .caption(&caption, ("sans-serif", 50).into_font())
                .margin(5)
                .x_label_area_size(30)
                .y_label_area_size(30)
                .build_ranged(30f32..90f32, 0f32..20f32)
                .unwrap();

            chart.configure_mesh().draw().unwrap();

            chart
                .draw_series(LineSeries::new(
                    (30..90).map(|n| {
                        let value = output[window_size as usize / 2 + n as usize].norm_sqr();
                        let normalized = value / window_size as f32;

                        (n as f32, normalized)
                    }),
                    &RED,
                ))
                .unwrap()
                .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &RED));

            chart
                .configure_series_labels()
                .background_style(&WHITE.mix(0.8))
                .border_style(&BLACK)
                .draw().unwrap();
        }

        mfb_window
            .update_with_buffer(unsafe { std::mem::transmute(&buf[..]) })
            .unwrap();

        let frame_dur = std::time::Duration::new(0, 1000000000 / 88);
        let tick = std::time::Duration::new(0, 1666667);
        while then.elapsed() < frame_dur {
            std::thread::sleep(tick);
        }
        then = std::time::Instant::now();
    }

    println!("processed {} windows", i);
}

fn main() {
    //play();
    read();
}
