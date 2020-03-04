pub use std::collections::HashMap;

pub type Frequency = i64;
pub type Amplitude = f64;
pub type Bits = HashMap<Frequency, bool>;
pub type BitMapping = HashMap<usize, Frequency>;

#[derive(Clone, Copy, Debug)]
pub struct Reference {
    pub amplitude: Amplitude,
    pub frequency: Frequency,
}

#[derive(Debug)]
pub struct Frame {
    clk: Frequency,
    reference: Reference,
    bits: Bits,
}

#[derive(Debug)]
pub struct FrameBuilder {
    clk1: Frequency,
    clk2: Frequency,
    reference: Reference,
    map: BitMapping,
    frame_num: usize,
}

impl FrameBuilder {
    pub fn new(clk1: Frequency, clk2: Frequency, reference: Reference, map: BitMapping) -> Self {
        FrameBuilder {
            clk1,
            clk2,
            reference,
            map,
            frame_num: 0
        }
    }

    pub fn make_frame(&mut self, bits: &[bool]) -> Frame {
        assert_eq!(bits.len(), self.map.len(), "!!! :)");

        let clk = match self.frame_num % 2 {
            0 => self.clk1,
            1 => self.clk2,
            _ => panic!("!!! :)"),
        };
        let reference = self.reference;
        let bits = {
            let mut tmp: Bits = HashMap::new();
            for i in 0 .. bits.len() {
                tmp.insert(self.map[&i], bits[i]);
            }
            tmp
        };

        self.frame_num += 1;

        Frame {
            clk,
            reference,
            bits,
        }
    }
}
