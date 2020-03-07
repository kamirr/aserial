pub use std::collections::HashMap;

pub type Frequency = u32;
pub type Bits = Vec<(Frequency, bool)>;
pub type BitMapping = Vec<Frequency>;

use rodio::{
    Sink, Device,
    source::{SineWave, Source, Zero}
};

use crate::multi_sine_wave::MultiSineWave;

#[derive(Debug)]
pub struct Frame {
    bits: Bits,
}

impl Frame {
    pub fn play(&self, device: &Device) -> Sink {
        let sink = Sink::new(device);
        /*let mut source: Box<dyn Source<Item = f32> + Send + Sync> = Box::new(
            Zero::new(1, 48000)
        );

        for (freq, state) in &self.bits {
            if *state {
                println!("{}", freq);
                source = Box::new(source.mix(SineWave::new(*freq)));
            }
        }*/

        let source = MultiSineWave::new(
            self.bits.iter().filter(|(_, state)| *state).map(|(freq, _)| *freq as f32).collect(),
            48000,
        );

        //println!("");

        sink.append(source);

        sink
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
        println!("{:?}:", bits);
        assert_eq!(bits.len(), self.map.len(), "!!! :)");

        Frame {
            bits: (0..bits.len())
                .map(|i| (self.map[i], bits[i]))
                .collect(),
        }
    }
}
