pub use std::collections::HashMap;

pub type Frequency = u32;
pub type Bits = Vec<(Frequency, bool)>;
pub type BitMapping = Vec<Frequency>;

use rodio::Sink;
use crate::multi_sine_wave::MultiSineWave;

#[derive(Debug)]
pub struct Frame {
    bits: Bits,
}

impl Frame {
    pub fn play(&self, sink: &Sink, secs: f32) {
        let source = MultiSineWave::new(
            self.bits.iter().filter(|(_, state)| *state).map(|(freq, _)| *freq as f32).collect(),
            48000,
            Some((secs * 48000f32) as usize),
        );

        sink.append(source);
    }
}

#[derive(Debug)]
pub struct FrameBuilder {
    map: BitMapping,
}

impl FrameBuilder {
    pub fn new(map: BitMapping) -> Self {
        FrameBuilder { map, }
    }

    pub fn make_frame(&mut self, bits: &[bool]) -> Frame {
        assert_eq!(bits.len(), self.map.len(), "!!! :)");

        Frame {
            bits: (0..bits.len())
                .map(|i| (self.map[i], bits[i]))
                .collect(),
        }
    }
}
