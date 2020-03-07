mod multi_sine_wave;
mod frame;

use std::sync::mpsc;
use rustfft::num_complex::Complex;
use rustfft::num_traits::Zero;

#[allow(dead_code)]
fn play(receiver: mpsc::Receiver<Vec<bool>>, freqs: Vec<u32>) {
    use rodio::Sink;
    use frame::*;

    let mut fb = FrameBuilder::new(freqs);

    let device = rodio::default_output_device().unwrap();
    let sink = Sink::new(&device);
    let time_per_sample = 0.07;
    let silence_ratio = 1.0;

    for v in receiver.iter() {
        //println!("{:?}", &v);
        fb.make_frame(&v).play(&sink, time_per_sample);
        fb.make_frame(&[false, false, false, false, false]).play(&sink, time_per_sample * silence_ratio);
    }

    sink.sleep_until_end();
}

#[allow(dead_code)]
fn stdin_reader(sender: mpsc::Sender<Vec<bool>>) {
    use std::io::Read;

    let stdin = std::io::stdin();
    let mut handle = stdin.lock();
    let mut buf = [0];

    loop {
        match handle.read(&mut buf) {
            Err(_) => break,
            Ok(0) => break,
            _ => { },
        }

        let b = buf[0];
        let (a1, b1) = ((b >> 0) & 1 == 1, (b >> 1) & 1 == 1);
        let (a2, b2) = ((b >> 2) & 1 == 1, (b >> 3) & 1 == 1);
        let (a3, b3) = ((b >> 4) & 1 == 1, (b >> 5) & 1 == 1);
        let (a4, b4) = ((b >> 6) & 1 == 1, (b >> 7) & 1 == 1);

        sender.send(vec![true, a1, b1, a2, b2]).unwrap();
        sender.send(vec![true, a3, b3, a4, b4]).unwrap();
    }
}

fn window_fn(f: f32, n: usize, n_total: usize) -> f32 {
    f * (3.1415 * n as f32 / n_total as f32).sin().powi(2)
}

fn plot(t: f32, buf: &mut [u8], output: &Vec<Complex<f32>>) {
    use plotters::drawing::bitmap_pixel::BGRXPixel;
    use plotters::prelude::*;

    let size: (u32, u32) = (800, 600);
    let caption = format!("t={:.1}s", t);

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
        .build_ranged(30f32..300f32, 0f32..20f32)
        .unwrap();

    chart.configure_mesh().draw().unwrap();

    chart
        .draw_series(LineSeries::new(
            (30..300).map(|n| {
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

fn normalize(output: &mut [Complex<f32>], window_size: usize) {
    for n in 0 .. output.len() {
        let value = output[n].norm_sqr();
        let normalized = value / window_size as f32;
        output[n] = Complex::new(normalized, 0.0);
    }
}

fn read(receiver: mpsc::Receiver<Vec<f32>>, freqs: Vec<u32>) {
    use std::io::Write;

    let mut i = 0;

    let window_size = 8820usize;
    let cutoff_clk = 3f32;
    let cutoff_bits = 3f32;
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
    let mut flag = true;
    let mut bits: Vec<i32> = vec![];

    let (mut av1, mut av2, mut av3, mut av4) = (0.0, 0.0, 0.0, 0.0);
    let mut averaging_over = 0;

    for window in receiver.iter() {
        i += 1;

        let mut input: Vec<Complex<f32>> = window
            .iter()
            .enumerate()
            .map(|(i, f)| Complex::<f32>::new(window_fn(*f, i, window_size), 0.0))
            .collect();

        planner
            .plan_fft(window_size)
            .process(&mut input, &mut output);

        normalize(&mut output, window_size);

        if flag && output[freqs[0] as usize / 10].norm() > cutoff_clk {
            //println!("got {} {}", bits[bits.len() - 1], bits[bits.len() - 2]);

            flag = false;
            averaging_over = 0;
            av1 = 0.0;
            av2 = 0.0;
            av3 = 0.0;
            av4 = 0.0;
        } else if !flag && output[freqs[0] as usize / 10].norm() < cutoff_clk {
            bits.push(if av1 / averaging_over as f32 > cutoff_bits { 1 } else { 0 });
            bits.push(if av2 / averaging_over as f32 > cutoff_bits { 1 } else { 0 });
            bits.push(if av3 / averaging_over as f32 > cutoff_bits { 1 } else { 0 });
            bits.push(if av4 / averaging_over as f32 > cutoff_bits { 1 } else { 0 });

            if bits.len() >= 8 {
                let rem = bits.split_off(8);
                let eight = bits;
                bits = rem;

                let byte = {
                    let mut tmp = 0u8;
                    for b in &eight {
                        tmp <<= 1;
                        tmp |= *b as u8;
                    }

                    tmp.reverse_bits()
                };

                let stdout = std::io::stdout();
                let mut out_handle = stdout.lock();
                out_handle.write(&[byte]).unwrap();
                out_handle.flush().unwrap();
            }

            flag = true;
        }

        if !flag {
            averaging_over += 1;
            av1 += output[freqs[1] as usize / 10].norm();
            av2 += output[freqs[2] as usize / 10].norm();
            av3 += output[freqs[3] as usize / 10].norm();
            av4 += output[freqs[4] as usize / 10].norm();
        }

        plot(start.elapsed().as_secs_f32(), &mut buf, &output);

        mfb_window
            .update_with_buffer(unsafe { std::mem::transmute(&buf[..]) })
            .unwrap();
    }

    println!("\n=================\nprocessed {} windows", i);
}

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

    let step = 300usize;
    let window_size = 8820usize;

    event_loop.run(|_, stream_result| {
        if let cpal::StreamData::Input{ buffer } = stream_result.unwrap() {
            match buffer {
                cpal::UnknownTypeInputBuffer::F32(buffer) => {
                    for sample in &*buffer {
                        let mut window = safe_window.lock().unwrap();
                        let sender = safe_sender.lock().unwrap();

                        window.push(*sample);

                        if window.len() == window_size {
                            let mut other_window: Vec<f32> = window[step .. window.len()].into();
                            std::mem::swap(&mut other_window, &mut *window);
                            sender.send(other_window).unwrap();
                        }
                    }
                },
                _ => { }
            }
        }
    });
}

fn main() {
    use std::thread;

    let freqs = &[1200, 1500, 1700, 1900, 2100];

    let (sender, receiver) = mpsc::channel();
    let freqs1 = freqs.to_vec();
    let player = thread::spawn(move || play(receiver, freqs1));
    thread::spawn(move || stdin_reader(sender));

    let (sender, receiver) = mpsc::channel();
    let freqs2 = freqs.to_vec();
    let reader = thread::spawn(move || read(receiver, freqs2));
    thread::spawn(move || audio_input(sender));

    player.join().unwrap();
    reader.join().unwrap();
}
