use std::sync::mpsc;
use std::thread;
use rustfft::num_complex::Complex;
use rustfft::num_traits::Zero;
use crate::Config;

/* Hann window to get rid of unwanted hight frequencies */
fn window_fn(f: f32, n: usize, n_total: usize) -> f32 {
    f * (3.1415 * n as f32 / n_total as f32).sin().powi(2)
}

fn plot(caption: String, buf: &mut [u8], output: &Vec<Complex<f32>>) {
    use plotters::drawing::bitmap_pixel::BGRXPixel;
    use plotters::prelude::*;

    let size: (u32, u32) = (800, 600);

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
        .build_ranged(30f32..1600f32, 0f32..2f32)
        .unwrap();

    chart.configure_mesh().draw().unwrap();

    chart
        .draw_series(LineSeries::new(
            (30..1600).map(|n| {
                (n as f32, output[n].norm() as f32)
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

/* converts a+bi to r+0i where r=sqrt(a*a + b*b) */
fn normalize(output: &mut [Complex<f32>], window_size: usize) {
    for n in 0 .. output.len() {
        let value = output[n].norm_sqr();
        let normalized = value / window_size as f32;
        output[n] = Complex::new(normalized, 0.0);
    }
}

fn get_windowed_samples(window: &Vec<f32>) -> Vec<Complex<f32>> {
    window
        .iter()
        .enumerate()
        .map(|(i, f)| Complex::<f32>::new(window_fn(*f, i, window.len()), 0.0))
        .collect()
}

fn read(receiver: mpsc::Receiver<Vec<f32>>, conf: Config) {
    use std::io::Write;

    let mut i = 0;
    let mut transferred = 0;

    let band = conf.band;
    let window_size = 8820usize;
    let cutoff_clk = conf.cutoff_clk;
    let start = std::time::Instant::now();

    let mut buf = vec![0u8; 800 * 600 * 4];
    let mut mfb_window = minifb::Window::new(
        "FT",
        800,
        600,
        minifb::WindowOptions::default(),
    ).unwrap();

    let mut planner = rustfft::FFTplanner::new(false);
    let mut output: Vec<Complex<f32>> = vec![Complex::zero(); window_size];
    let mut last_frame = std::time::Instant::now();
    let mut flag = true;

    for window in receiver.iter() {
        i += 1;

        let mut input = get_windowed_samples(&window);

        planner
            .plan_fft(window_size)
            .process(&mut input, &mut output);

        normalize(&mut output, window_size);

        if flag && output[band.clk as usize / 10].norm() > cutoff_clk {
            let from = band.base as usize / 10;
            let to = (band.base + band.scale * 255) as usize / 10;

            let freq_offset = output[from .. to]
                .iter()
                .enumerate()
                .max_by_key(|(_, f)| (f.norm_sqr() * 10000.0) as u32)
                .unwrap().0 * 10;
            let freq_offset = freq_offset as u32;

            /* round to nearest multiple of 20 */
            let byte = match freq_offset % band.scale >= band.scale / 2 {
                true => freq_offset / band.scale + 1,
                false => freq_offset / band.scale,
            } as u8;

            let stdout = std::io::stdout();
            let mut out_handle = stdout.lock();
            out_handle.write(&[byte]).unwrap();
            out_handle.flush().unwrap();

            transferred += 1;
            flag = false;
        } else if !flag && output[band.clk as usize / 10].norm() < cutoff_clk {
            flag = true;
        }

        /* make a new plot and refresh the window if more than 1/60s of a second has passed *
         * since the last refresh                                                           */
        if last_frame.elapsed() > std::time::Duration::new(0, 1000000000 / 60) {
            let caption = format!("#{}; t={:.1}s", transferred, start.elapsed().as_secs_f32());
            plot(caption, &mut buf, &output);
            mfb_window
                .update_with_buffer(unsafe { std::mem::transmute(&buf[..]) })
                .unwrap();

            last_frame = std::time::Instant::now();
        }
    }

    println!("\n=================\nprocessed {} windows", i);
}

/* read samples from default input device */
fn audio_input(sender: mpsc::Sender<Vec<f32>>) {
    use cpal::traits::{DeviceTrait, EventLoopTrait, HostTrait};
    use std::sync::{Mutex, Arc};

    let host = cpal::default_host();
    let event_loop = host.event_loop();
    let device = host.default_input_device().expect("no output device available");
    let format = device.default_input_format().unwrap();
    let stream_id = event_loop.build_input_stream(&device, &format).unwrap();
    event_loop.play_stream(stream_id).expect("failed to play_stream");

    let safe_sender = Arc::new(Mutex::new(sender));
    let safe_window = Arc::new(Mutex::new(Vec::<f32>::new()));

    let step = 200usize;
    let window_size = 8820usize;

    event_loop.run(|_, stream_result| {
        if let cpal::StreamData::Input{ buffer } = stream_result.unwrap() {
            match buffer {
                /* ATM only support buffers of f32 samples */
                cpal::UnknownTypeInputBuffer::F32(buffer) => {
                    for sample in &*buffer {
                        let mut window = safe_window.lock().unwrap();

                        window.push(*sample);

                        if window.len() == window_size {
                            let mut other_window: Vec<f32> = window[step .. window.len()].into();
                            std::mem::swap(&mut other_window, &mut *window);

                            let sender = safe_sender.lock().unwrap();
                            sender.send(other_window).unwrap();
                        }
                    }
                },
                _ => { }
            }
        }
    });
}

pub fn listen(conf: Config) -> std::thread::JoinHandle<()> {
    let (sender, receiver) = mpsc::channel();
    thread::spawn(move || audio_input(sender));
    thread::spawn(move || read(receiver, conf))
}
