use rodio::source::Source;

pub struct BiSineWave {
    freqs: (f32, Option<f32>),
    sample_rate: u32,
    samples: usize,
    num_sample: usize,
}

impl BiSineWave {
    pub fn new(freqs: (f32, Option<f32>), sample_rate: u32, samples: usize) -> Self {
        BiSineWave {
            freqs,
            sample_rate,
            samples,
            num_sample: 0,
        }
    }
}

impl Source for BiSineWave {
    fn current_frame_len(&self) -> Option<usize> {
        Some(self.samples)
    }

    fn channels(&self) -> u16 {
        1
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn total_duration(&self) -> Option<std::time::Duration> {
        let nsecs = 1000000000 as f32 * self.samples as f32 / self.sample_rate as f32;
        Some(std::time::Duration::new(0, nsecs as u32))
    }
}

impl Iterator for BiSineWave {
    type Item = f32;

    #[inline]
    fn next(&mut self) -> Option<f32> {
        self.num_sample = self.num_sample.wrapping_add(1);
        let cnst = 2.0 * 3.14159265 * self.num_sample as f32 / self.sample_rate as f32;

        if self.num_sample < self.samples {
            let a0 = (self.freqs.0 * cnst).sin();

            Some(a0 + match self.freqs.1 {
                Some(f1) => (f1 * cnst).sin(),
                None => 0.0,
            })
        } else {
            None
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (
            self.samples,
            Some(self.samples),
        )
    }
}

impl ExactSizeIterator for BiSineWave { }
