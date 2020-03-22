mod maybe_sine_wave;
mod extremum_finder;
mod frame;
mod listen;
mod plot;
mod talk;

use serde::{Deserialize, Serialize};
use listen::listen;
use std::env::args;
use std::process::exit;
use talk::talk;

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct Config {
    /* frequencies used for communication */
    pub base: u32,
    pub scale: u32,

    /* time for playing encoded data and silence */
    pub loud_time: f32,
    pub silent_time: f32,

    pub vbr_lo: f32,
    pub vbr_hi: f32,
}

impl Config {
    pub fn new() -> Self {
        Config {
            base: 4000,
            scale: 40,
            loud_time: 0.021,
            silent_time: 0.021,
            vbr_hi: 100.0,
            vbr_lo: 10.0,
        }
    }

    pub fn create() {
        let cnf = Config::new();
        let serialized = toml::to_string(&cnf).unwrap();
        std::fs::write("conf.toml", serialized.as_bytes()).unwrap();
    }

    pub fn load() -> Self {
        let serialized = std::fs::read_to_string("conf.toml").unwrap();
        toml::from_str(&serialized).expect("malformed conf.toml")
    }
}

/* print usage and exit */
/* the ! return type means that the function never returns */
fn usage() -> ! {
    eprintln!("usage: aserial makeconf|listen|talk");
    exit(1 /* 1, because stuff went wrong */)
}

fn main() {
    /* args[0] = program path (don't care) *
     * args[1] = listen / talk             *
     * args[2] = loud / cable              */
    let args: Vec<String> = args().collect();

    /* make sure that there are exactly 2 arguments */
    if args.len() != 2 {
        usage();
    }

    /* start receiving / transmitting bytes */
    match args[1].as_ref() {
        "makeconf" => Config::create(),
        "listen" => listen(Config::load()),
        "talk" => talk(Config::load()),
        _ => usage(),
    }
}
