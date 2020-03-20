mod maybe_sine_wave;
mod extremum_finder;
mod frame;
mod listen;
mod plot;
mod talk;

use frame::Band;
use listen::listen;
use std::env::args;
use std::process::exit;
use talk::talk;

#[derive(Clone, Copy)]
pub struct Config {
    /* frequencies used for communication */
    pub band: Band,

    /* time for playing encoded data and silence */
    pub loud_time: f32,
    pub silent_time: f32,

    /* ratio of the amplitude of data-encoding frequency  *
     * to the baseline must be above this for the program *
     * to not discard the data as noise                   */
    pub min_noise_ratio: f32,
}

impl Config {
    /* config for transmission over cable */
    pub fn cable() -> Self {
        Config {
            band: Band {
                base: 4000,
                scale: 40,
            },
            loud_time: 0.025,
            silent_time: 0.025,
            min_noise_ratio: 100.0,
        }
    }

    /* config for loud transmission */
    pub fn loud() -> Self {
        Config {
            band: Band {
                base: 4000,
                scale: 34,
            },
            loud_time: 0.075,
            silent_time: 0.075,
            min_noise_ratio: 100.0,
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
