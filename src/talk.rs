use std::sync::mpsc;
use std::thread;
use rodio::Sink;

use crate::frame::*;
use crate::Config;

/* play sounds corresponding to incoming bytes */
fn play(receiver: mpsc::Receiver<Option<u8>>, conf: Config) {
    let mut fb = FrameBuilder::new(conf.band);

    let device = rodio::default_output_device().unwrap();
    let sink = Sink::new(&device);

    for maybe in receiver.iter() {
        let b = match maybe {
            Some(byte) => byte,
            None => break,
        };
        fb.build(false, b, &sink, conf.clk_low_time);
        fb.build(true, b, &sink, conf.clk_high_time);
    }

    sink.sleep_until_end();
}

/* read bytes from stding and send them to be played */
fn stdin_reader(sender: mpsc::Sender<Option<u8>>) {
    use std::io::Read;

    let stdin = std::io::stdin();
    let mut handle = stdin.lock();
    let mut buf = [0];

    loop {
        let cnt = match handle.read(&mut buf) {
            Err(_) => break,
            Ok(n) => n,
        };

        for k in 0 .. cnt {
            sender.send(Some(buf[k])).unwrap();
        }
    }

    sender.send(None).unwrap();
}

/* set up a pair of threads to read bytes and play them */
pub fn talk(conf: Config) -> std::thread::JoinHandle<()> {
    let (sender, receiver) = mpsc::channel();
    thread::spawn(move || stdin_reader(sender));
    thread::spawn(move || {
        play(receiver, conf);
        std::thread::sleep(std::time::Duration::new(1, 0));
    })
}
