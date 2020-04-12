use rodio::Sink;
use std::io::Read;
use std::sync::mpsc;
use std::thread;

use crate::frame::*;
use crate::Config;

/* play sounds corresponding to incoming bytes */
fn play(receiver: mpsc::Receiver<Vec<u8>>, conf: Config) {
    let mut fb = FrameBuilder::new(conf);
    let mut transferred = 0;
    let start = std::time::Instant::now();

    /* build a sink from the default output device */
    let device = rodio::default_output_device().unwrap();
    let sink = Sink::new(&device);

    /* take buffers from output */
    for bytes in receiver.iter() {
        /* for each byte play it twice: once w/o the clock, and once with it */
        for b in bytes {
            let shift = match transferred % 2 {
                0 => 0,
                1 => 256,
                _ => panic!(),
            };
            transferred += 1;

            fb.build(Some(b as u32 + shift), &sink, conf.loud_time);
            fb.build(None, &sink, conf.silent_time);
        }
    }

    /* sleep until all the sounds have finished playing */
    sink.sleep_until_end();

    let rate = transferred as f32 / start.elapsed().as_secs_f32();
    println!("sent {}B; rate: {:.2}Bps", transferred, rate);
}

/* read bytes from stding and send them to be played */
fn stdin_reader(sender: mpsc::Sender<Vec<u8>>) {
    /* lock the stdin for total control */
    let stdin = std::io::stdin();
    let mut handle = stdin.lock();

    /* preallocate a buffer of 256 bytes to avoid sending *
     * too many small messages, which is slooooooooooooow */
    let mut buf = [0; 256];

    loop {
        /* try filling the buffer with bytes from stdin. Note that pressing Enter    *
         * flushes the buffer prematurely so it isn't always full. Also, the input   *
         * size might not be divisible by 256. Thus, store the actual number of read *
         * bytes in `cnt`. In case of any error, break the loop                      */
        let cnt = match handle.read(&mut buf) {
            Err(_) => break,
            Ok(0) => break,
            Ok(n) => n, /* n is the number of read bytes */
        };

        /* convert the received bytes into a vector -- it needs to    *
         * be allocated on the heap to be sent safely between threads */
        let bytes = buf[0..cnt].into();
        sender.send(bytes).unwrap();
    }
}

pub fn talk(conf: Config) {
    /* create a channel */
    let (sender, receiver) = mpsc::channel();

    /* launch a thread for playing sounds */
    let loud_thread = thread::spawn(move || {
        play(receiver, conf);
        std::thread::sleep(std::time::Duration::new(1, 0));
    });

    /* read stdin */
    stdin_reader(sender);

    /* wait for the playing thread */
    loud_thread.join().unwrap();
}
