use rodio::source::Source;

pub struct MultiSineWave {
    freqs: Vec<f32>,
    sample_rate: u32,
    num_sample: usize,
}

impl MultiSineWave {
    pub fn new(freqs: Vec<f32>, sample_rate: u32) -> Self {
        MultiSineWave {
            freqs,
            sample_rate,
            num_sample: 0,
        }
    }
}

impl Source for MultiSineWave {
    fn current_frame_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> u16 {
        1
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn total_duration(&self) -> Option<std::time::Duration> {
        None
    }
}

impl Iterator for MultiSineWave {
    type Item = f32;

    #[inline]
    fn next(&mut self) -> Option<f32> {
        self.num_sample = self.num_sample.wrapping_add(1);
        let cnst = 2.0 * 3.14159265 * self.num_sample as f32 / self.sample_rate as f32;

        Some(
            self.freqs
                .iter()
                .map(|freq| cnst * freq)
                .map(|arg| arg.sin())
                .fold(0f32, |acc, amplitude| acc + amplitude)
        )
    }
}
