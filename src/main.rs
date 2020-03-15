mod bi_sine_wave;
mod frame;
mod listen;
mod plot;
mod talk;

use std::process::exit;
use std::env::args;
use frame::Band;
use listen::listen;
use talk::talk;

#[derive(Clone, Copy)]
pub struct Config {
    /* frequencies used for communication */
    pub band: Band,

    /* clock intensity at which data is collected, should be adjusted *
     * algorithmically but I couldn't be bothered to implement it,    *
     * so it isn't lmao                                               */
    pub cutoff_clk: f32,

    /* time for playing encoded data with and w/o the clock */
    pub clk_high_time: f32,
    pub clk_low_time: f32,
}

impl Config {
    /* config for transmission over cable */
    pub fn cable() -> Self {
        Config {
            band: Band { clk: 15000, base: 4000, scale: 40 },
            cutoff_clk: 1.9,
            clk_low_time: 0.025,
            clk_high_time: 0.025,
        }
    }

    /* config for loud transmission */
    pub fn loud() -> Config {
        Config {
            band: Band { clk: 1000, base: 4000, scale: 30 },
            cutoff_clk: 2.0,
            clk_low_time: 0.05,
            clk_high_time: 0.05,
        }
    }
}

/* print usage and exit */
/* the ! return type means that the function never returns */
fn usage() -> ! {
    eprintln!("usage: aserial loud|cable listen|talk");
    exit(1 /* 1, because stuff went wrong */)
}

fn main() {
    /* args[0] = program path (don't care) *
     * args[1] = listen / talk             *
     * args[2] = loud / cable              */
    let args: Vec<String> = args().collect();

    /* make sure that there are exactly 3 arguments */
    if args.len() != 3 {
        usage();
    }

    /* choose the config */
    let conf = match args[1].as_ref() {
        "loud" => Config::loud(),
        "cable" => Config::cable(),
        _ => usage(),
    };

    /* start receiving / transmitting bytes */
    match args[2].as_ref() {
        "listen" => listen(conf),
        "talk" => talk(conf),
        _ => usage(),
    }
}
