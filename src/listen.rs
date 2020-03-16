use std::io::Write;
use std::sync::mpsc;
use std::thread;
use rustfft::num_complex::Complex;
use rustfft::num_traits::Zero;
use cpal::traits::{DeviceTrait, EventLoopTrait, HostTrait};
use crate::extremum_finder::*;
use crate::Config;
use crate::plot::Plot;

/* Hann window to get rid of unwanted hight frequencies */
fn window_fn(f: f32, n: usize, n_total: usize) -> f32 {
    f * (3.1415 * n as f32 / n_total as f32).sin().powi(2)
}

/* converts each a+bi to r+0i where r=(a*a + b*b)/window_size */
fn normalize(output: &mut [Complex<f32>], window_size: usize) {
    for n in 0 .. output.len() {
        let value = output[n].norm_sqr();
        let normalized = value / window_size as f32;
        output[n] = Complex::new(normalized, 0.0);
    }
}

/* passes samples through a window window_fn */
fn get_windowed_samples(window: &Vec<f32>) -> Vec<Complex<f32>> {
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
    let mut max_cnt = 0;
    let mut min_cnt = 0;
    let mut transferred = 0;
    let mut flag = true; /* is clk below cutoff_clk? */
    let mut maxs = vec![0.0; 3];
    let mut mins = vec![0.0; 3];
    let mut ef = ExtremumFinder::new();
    let start = std::time::Instant::now();

    /* all configuration */
    let band = conf.band;
    let window_size = 8820usize;

    /* lock stdout, only this part of code uses it  *
     * and I need total control, including flushing */
    let stdout = std::io::stdout();
    let mut out_handle = stdout.lock();

    let mut plt = Plot::new();

    /* used to compute FFT efficiently */
    let mut planner = rustfft::FFTplanner::new(false /* false=FFT, true=FFT^-1 */);
    let mut output: Vec<Complex<f32>> = vec![Complex::zero(); window_size];

    /* take each window from the microphone */
    for window in receiver.iter() {
        i += 1;

        let mut input = get_windowed_samples(&window);

        /* compute FFT */
        planner
            .plan_fft(window_size)
            .process(&mut input, &mut output);

        /* 'normalize' FFT output */
        normalize(&mut output, window_size);

        let clk_idx = band.clk as usize / 10;
        let clk_value = output[clk_idx].norm();

        let baseline = output[0 .. band.base as usize / 10 - 10]
            .iter()
            .map(|c| c.norm())
            .fold(0.0, |a, b| a + b)
            / (band.base / 10 - 5) as f32;

        let valid = clk_value / baseline > conf.noise_tolerance;

        if let Some(ex) = ef.push(clk_value) {
            match ex {
                Extremum::Maximum(v) => {
                    let idx = max_cnt % maxs.len();
                    maxs[idx] = v;
                    max_cnt += 1;
                },
                Extremum::Minimum(v) => {
                    let idx = min_cnt % mins.len();
                    mins[idx] = v;
                    min_cnt += 1;
                },
            }
        }

        let lowest_max = maxs.iter().cloned().fold(0./0., f32::min);
        let highest_min = mins.iter().cloned().fold(0./0., f32::max);
        let floating_cutoff = (lowest_max - highest_min) * 0.9 + highest_min;

        /* passing cutoff_clk from below */
        if valid && flag && clk_value > floating_cutoff {
            /* range of output indices corresponding to data-encoding frequencies */
            let from = band.base as usize / 10;
            let to = (band.base + band.scale * 256) as usize / 10;

            /* find the loudest frequency in that range */
            let freq_offset = output[from .. to]
                .iter()
                .enumerate()
                /* the line below maps floats to integers while preserving the order *
                 * because float aren't ordered (cuz NaNs) and we need a maximum     */
                .max_by_key(|(_, f)| (f.norm_sqr() * 10000.0) as u32)
                .unwrap().0 * 10; /* mutliply by 10 because the unit is 0.1Hz and I want 1Hz */
            /* map it to integers */
            let freq_offset = freq_offset as u32;

            /* round to nearest multiple of band.scale */
            let maybe_byte = match freq_offset % band.scale >= band.scale / 2 {
                true => freq_offset / band.scale + 1,
                false => freq_offset / band.scale,
            };

            if maybe_byte != 0 {
                let byte = (maybe_byte - 1) as u8;

                /* write out the received byte and flush */
                out_handle.write(&[byte]).unwrap();
                out_handle.flush().unwrap();

                transferred += 1;
            }

            flag = false;

        /* passing cutoff_clk from above */
        } else if !flag && clk_value < floating_cutoff {
            flag = true;
        }

        if plt.needs_refreshing() {
            /* "#[SAMPLE_COUNT]; t=[TIME_IN_SECS]s" */
            let time = start.elapsed().as_secs_f32();
            let caption = format!("#{}; t={:.1}s", transferred, time);
            /* refresh the plot */
            plt.refresh(caption, &output, highest_min, lowest_max);
        }
    }

    println!("\n=================\nprocessed {} windows", i);
}

fn process_sample(sample: f32, window: &mut Vec<f32>, window_size: usize, step: usize, sender: &mpsc::Sender<Vec<f32>>) {
    window.push(sample);

    /* if the window is of necessary size... */
    if window.len() == window_size {
        /* copy all data that should be retained for the next window */
        let mut other_window: Vec<f32> = window[step .. window.len()].into();
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
    let device = host.default_input_device().expect("no input device available");
    let format = device.default_input_format().unwrap();
    let stream_id = event_loop.build_input_stream(&device, &format).unwrap();
    event_loop.play_stream(stream_id).expect("failed to play_stream");

    /* collects samples from microphone */
    let mut window = Vec::<f32>::new();

    /* window step size */
    let step = 200usize;
    let window_size = 8820usize;

    /* run a function for each new event */
    event_loop.run(move |_, stream_result| {
        /* if the event is an input buffer... */
        if let cpal::StreamData::Input{ buffer } = stream_result.unwrap() {
            /* check the buffer type... */
            match buffer {
                /* ATM only support buffers of f32 samples */
                cpal::UnknownTypeInputBuffer::F32(buffer) => {
                    /* process each sample in the buffer */
                    for sample in &*buffer {
                        process_sample(*sample, &mut window, window_size, step, &sender);
                    }
                },
                /* ignore all buffers that don't have F32 samples */
                _ => { }
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
