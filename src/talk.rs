use std::sync::mpsc;
use std::thread;
use rodio::Sink;

use crate::frame::*;
use crate::Config;

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

fn stdin_reader(sender: mpsc::Sender<Option<u8>>) {
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

        sender.send(Some(buf[0])).unwrap();
    }

    sender.send(None).unwrap();
}

pub fn talk(conf: Config) -> std::thread::JoinHandle<()> {
	let (sender, receiver) = mpsc::channel();
	thread::spawn(move || stdin_reader(sender));
	thread::spawn(move || {
		play(receiver, conf);
		std::thread::sleep(std::time::Duration::new(1, 0));
	})
}
