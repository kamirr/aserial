pub use std::collections::HashMap;

pub type Frequency = u32;

use rodio::Sink;
use crate::bi_sine_wave::BiSineWave;

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
        let byte_freq = (self.band.base + self.band.scale * byte as u32) as f32;
        let freqs = (
            byte_freq,
            if clk { Some(self.band.clk as f32) } else { None }
        );

        let source = BiSineWave::new(
            freqs,
            48000,
            (secs * 48000f32) as usize,
        );

        sink.append(source);
    }
}
