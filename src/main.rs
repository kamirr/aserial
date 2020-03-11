mod bi_sine_wave;
mod frame;
mod listen;
mod talk;

use std::process::exit;
use std::env::args;
use frame::Band;
use listen::listen;
use talk::talk;

#[derive(Clone, Copy)]
pub struct Config {
    pub band: Band,
    pub cutoff_clk: f32,
    pub clk_high_time: f32,
    pub clk_low_time: f32,
}

#[allow(dead_code)]
fn cable_conf() -> Config {
    Config {
        band: Band { clk: 15000, base: 4000, scale: 40 },
        cutoff_clk: 1.9,
        clk_low_time: 0.025,
        clk_high_time: 0.025,
    }
}

#[allow(dead_code)]
fn loud_conf() -> Config {
    Config {
        band: Band { clk: 1000, base: 4000, scale: 30 },
        cutoff_clk: 0.2,
        clk_low_time: 0.045,
        clk_high_time: 0.045,
    }
}

fn usage() -> ! {
	eprintln!("usage: aserial listen|talk");
	exit(1)
}

fn main() {
    let args: Vec<String> = args().collect();
    let conf = loud_conf();

    if args.len() != 2 {
        usage();
    }

    match args[1].as_ref() {
        "listen" => listen(conf),
        "talk" => talk(conf),
        _ => usage(),
    }.join().unwrap();
}
