use crate::extremum_finder::*;
use crate::plot::Plot;
use crate::Config;
use cpal::traits::{DeviceTrait, EventLoopTrait, HostTrait};
use rustfft::num_complex::Complex;
use rustfft::num_traits::Zero;
use std::io::Write;
use std::sync::mpsc;
use std::thread;

/* Hann window to get rid of unwanted hight frequencies */
fn window_fn(f: f32, n: usize, n_total: usize) -> f32 {
    f * (std::f32::consts::PI * n as f32 / n_total as f32)
        .sin()
        .powi(2)
}

/* converts each a+bi to r+0i where r=(a*a + b*b)/window_size */
fn normalize(fft: &[Complex<f32>], out: &mut [f32], window_size: usize) {
    assert_eq!(fft.len(), out.len(), "yee? :)");
    for k in 0..out.len() {
        out[k] = fft[k].norm_sqr() / k as f32 * out.len() as f32 / window_size as f32;
    }
}

/* passes samples through a window window_fn */
fn get_windowed_samples(window: &[f32]) -> Vec<Complex<f32>> {
    window
        .iter()
        .enumerate()
        .map(|(i, f)| Complex::<f32>::new(window_fn(*f, i, window.len()), 0.0))
        .collect()
}

/* take windows and decode them into bytes */
fn audio_processing(receiver: mpsc::Receiver<Vec<f32>>, conf: Config) {
    /* bookkeeping */
    let mut i = 0;
    let mut transferred = 0;
    let mut min_vbr = 99999f32;
    let start = std::time::Instant::now();

    /* all configuration */
    let window_size = 8820usize;

    /* lock stdout, only this part of code uses it  *
     * and I need total control, including flushing */
    let stdout = std::io::stdout();
    let mut out_handle = stdout.lock();

    let mut plt = Plot::new();

    /* used to compute FFT efficiently */
    let mut planner = rustfft::FFTplanner::new(false /* false=FFT, true=FFT^-1 */);
    let mut fft_output: Vec<Complex<f32>> = vec![Complex::zero(); window_size];
    let mut output: Vec<f32> = vec![0.0; window_size];

    let mut finders = [ExtremumFinder::new(); 256*2];
    let mut was_below_baseline = [false; 256*2];

    /* take each window from the microphone */
    for window in receiver.iter() {
        i += 1;

        let mut input = get_windowed_samples(&window);

        /* compute FFT */
        planner
            .plan_fft(window_size)
            .process(&mut input, &mut fft_output);

        /* 'normalize' FFT output */
        normalize(&fft_output, &mut output, window_size);

        for k in 0 .. 256usize*2 {
            let freq_idx = (conf.base + conf.scale * k as u32) as usize / 10;
            let from = conf.base as usize / 10;
            let to = (conf.base + conf.scale * 255) as usize / 10;
            let val = output[freq_idx];

            let baseline = output[from..to]
                .iter()
                .fold(0.0, |a, b| a + b)
                / (to - from) as f32;

            let vbr = val;
            let above = vbr > conf.vbr_hi;
            let below = vbr < conf.vbr_lo;
            if below {
                if !was_below_baseline[k] {
                    eprintln!("{:02x} got below", k);
                }
                was_below_baseline[k] = true;
            }

            if let Some(ex) = finders[k].push(val) {
                if let Extremum::Maximum(_) = ex {
                    if above && was_below_baseline[k] {
                        let recv = (k % 256) as u8;

                        min_vbr = min_vbr.min(vbr);
                        was_below_baseline[k] = false;
                        eprintln!("recv({:02x}) vbr: {}", recv, vbr);
                        out_handle.write_all(&[recv]).unwrap();
                        out_handle.flush().unwrap();
                        transferred += 1;
                    }
                }
            }
        }

        if plt.needs_refreshing() {
            /* "#[SAMPLE_COUNT]; t=[TIME_IN_SECS]s" */
            let time = start.elapsed().as_secs_f32();
            let caption = format!("#{}; t={:.1}s", transferred, time);
            /* refresh the plot */
            plt.refresh(caption, &output);
        }
    }

    println!("\n=================\nprocessed {} windows", i);
}

fn process_sample(
    sample: f32,
    window: &mut Vec<f32>,
    window_size: usize,
    step: usize,
    sender: &mpsc::Sender<Vec<f32>>,
) {
    window.push(sample);

    /* if the window is of necessary size... */
    if window.len() == window_size {
        /* copy all data that should be retained for the next window */
        let mut other_window: Vec<f32> = window[step..window.len()].into();
        /* swap it with the current window */
        std::mem::swap(&mut other_window, window);

        /* send the window to the other thread */
        sender.send(other_window).unwrap();
    }
}

/* read samples from default input device */
fn audio_input(sender: mpsc::Sender<Vec<f32>>) {
    /* start the event loop with the stream of samples from the microphone */
    let host = cpal::default_host();
    let event_loop = host.event_loop();
    let device = host
        .default_input_device()
        .expect("no input device available");
    let format = device.default_input_format().unwrap();
    let stream_id = event_loop.build_input_stream(&device, &format).unwrap();
    event_loop
        .play_stream(stream_id)
        .expect("failed to play_stream");

    /* collects samples from microphone */
    let mut window = Vec::<f32>::new();

    /* window step size */
    let step = 200usize;
    let window_size = 8820usize;

    /* run a function for each new event */
    event_loop.run(move |_, stream_result| {
        /* if the event is an input buffer... */
        if let cpal::StreamData::Input { buffer } = stream_result.unwrap() {
            /* check the buffer type... */
            match buffer {
                /* ATM only support buffers of f32 samples */
                cpal::UnknownTypeInputBuffer::F32(buffer) => {
                    /* process each sample in the buffer */
                    for sample in &*buffer {
                        process_sample(*sample, &mut window, window_size, step, &sender);
                    }
                }
                /* ignore all buffers that don't have F32 samples */
                _ => panic!("audio input samples aren't f32"),
            }
        }
    });
}

pub fn listen(conf: Config) {
    /* create a channel */
    let (sender, receiver) = mpsc::channel();

    /* launch a thread listening on a microphone */
    thread::spawn(move || audio_input(sender));

    /* process incoming audio */
    audio_processing(receiver, conf)
}
