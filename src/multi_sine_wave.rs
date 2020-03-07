use rodio::source::Source;

pub struct MultiSineWave {
    freqs: Vec<f32>,
    sample_rate: u32,
    samples: Option<usize>,
    num_sample: usize,
}

impl MultiSineWave {
    pub fn new(freqs: Vec<f32>, sample_rate: u32, samples: Option<usize>) -> Self {
        MultiSineWave {
            freqs,
            sample_rate,
            samples,
            num_sample: 0,
        }
    }
}

impl Source for MultiSineWave {
    fn current_frame_len(&self) -> Option<usize> {
        self.samples
    }

    fn channels(&self) -> u16 {
        1
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn total_duration(&self) -> Option<std::time::Duration> {
        self.samples.map(|num| {
            let nsecs = 1000000000 as f32 * num as f32 / self.sample_rate as f32;
            std::time::Duration::new(0, nsecs as u32)
        })
    }
}

impl Iterator for MultiSineWave {
    type Item = f32;

    #[inline]
    fn next(&mut self) -> Option<f32> {
        self.num_sample = self.num_sample.wrapping_add(1);
        let cnst = 2.0 * 3.14159265 * self.num_sample as f32 / self.sample_rate as f32;

        let limit = self.samples.unwrap_or(self.num_sample + 1);
        if self.num_sample < limit {
            Some(
                self.freqs
                    .iter()
                    .map(|freq| cnst * freq)
                    .map(|arg| arg.sin())
                    .fold(0f32, |acc, amplitude| acc + amplitude)
            )
        } else {
            None
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (
            match self.samples {
                Some(num) => num,
                None => 0,
            },
            self.samples,
        )
    }
}

impl ExactSizeIterator for MultiSineWave { }
