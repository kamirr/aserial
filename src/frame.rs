pub use std::collections::HashMap;

pub type Frequency = u32;

use rodio::Sink;
use crate::multi_sine_wave::MultiSineWave;

#[derive(Clone, Copy, Debug)]
pub struct Band {
    pub clk: Frequency,
    pub base: Frequency,
    pub scale: u32,
}

#[derive(Debug)]
pub struct FrameBuilder {
    band: Band
}

impl FrameBuilder {
    pub fn new(band: Band) -> Self {
        FrameBuilder { band }
    }

    pub fn build(&mut self, clk: bool, byte: u8, sink: &Sink, secs: f32) {
        let mut freqs = vec![(self.band.base + self.band.scale * byte as u32) as f32];
        if clk {
            freqs.push(self.band.clk as f32);
        }

        let source = MultiSineWave::new(
            freqs,
            48000,
            Some((secs * 48000f32) as usize),
        );

        sink.append(source);
    }
}
