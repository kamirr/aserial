use std::io::Write;
use std::sync::mpsc;
use std::thread;
use rustfft::num_complex::Complex;
use rustfft::num_traits::Zero;
use cpal::traits::{DeviceTrait, EventLoopTrait, HostTrait};
use crate::Config;
use crate::plot::plot;

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
    let mut transferred = 0;
    let mut flag = true;
    let mut last_frame = std::time::Instant::now();
    let start = std::time::Instant::now();

    /* all configuration */
    let band = conf.band;
    let window_size = 8820usize;
    let cutoff_clk = conf.cutoff_clk;

    /* lock stdout, only this part of code uses it  *
     * and I need total control, including flushing */
    let stdout = std::io::stdout();
    let mut out_handle = stdout.lock();

    /* rendering buffer */
    let mut buf = vec![0u8; 800 * 600 * 4];

    /* window to display the plot */
    let mut mfb_window = minifb::Window::new(
        "FT",
        800,
        600,
        minifb::WindowOptions::default(),
    ).unwrap();

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

        /* passing cutoff_clk from below */
        if flag && output[band.clk as usize / 10].norm() > cutoff_clk {
            /* range of output indices corresponding to data-encoding frequencies */
            let from = band.base as usize / 10;
            let to = (band.base + band.scale * 255) as usize / 10;

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
            let byte = match freq_offset % band.scale >= band.scale / 2 {
                true => freq_offset / band.scale + 1,
                false => freq_offset / band.scale,
            } as u8;

            /* write out the received byte and flush */
            out_handle.write(&[byte]).unwrap();
            out_handle.flush().unwrap();

            transferred += 1;
            flag = false;

        /* passing cutoff_clk from above */
        } else if !flag && output[band.clk as usize / 10].norm() < cutoff_clk {
            flag = true;
        }

        /* make a new plot and refresh the window if more than 1/60th *
         * of a second has passed since the last refres               */
        if last_frame.elapsed() > std::time::Duration::new(0, 1000000000 / 60) {
            /* "#[SAMPLE_COUNT]; t=[TIME_IN_SECS]s" */
            let caption = format!("#{}; t={:.1}s", transferred, start.elapsed().as_secs_f32());
            /* render the plot to the preallocated buffer */
            plot(caption, &mut buf, &output);
            /* refresh the window using that buffer */
            mfb_window
                .update_with_buffer(unsafe { std::mem::transmute(&buf[..]) })
                .unwrap();

            /* mark the time of last refresh */
            last_frame = std::time::Instant::now();
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
