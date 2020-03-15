use rodio::source::Source;

pub struct BiSineWave {
    /* always play freqs.0, only play freqs.1 if it's Some(_) */
    freqs: (f32, Option<f32>),
    sample_rate: u32,
    /* stop playing after this many samples */
    samples: usize,
    /* used internally to keep track of time */
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
    /* length of sound in samples */
    fn current_frame_len(&self) -> Option<usize> {
        Some(self.samples)
    }

    fn channels(&self) -> u16 {
        1
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    /* length as time with units */
    fn total_duration(&self) -> Option<std::time::Duration> {
        let nsecs = 1000000000 as f32 * self.samples as f32 / self.sample_rate as f32;
        Some(std::time::Duration::new(0, nsecs as u32))
    }
}

impl Iterator for BiSineWave {
    type Item = f32;

    #[inline]
    fn next(&mut self) -> Option<f32> {
        /* check if the sound has any more samples to play */
        if self.num_sample < self.samples {
            self.num_sample += 1;

            /* time in secs */
            let t = self.num_sample as f32 / self.sample_rate as f32;
            /* multiply by 2π so that 1Hz corresponds to one period of sine */
            let cnst = 2.0 * 3.14159265 * t;
            /* value of frequency 1 at time=t */
            let a0 = (self.freqs.0 * cnst).sin();
            /* if there's freq 2, take it's value at time=t, otherwise 0 */
            let a1 = match self.freqs.1 {
                Some(f1) => (f1 * cnst).sin(),
                None => 0.0,
            };

            Some(a0 + a1)
        } else {
            None
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (
            self.samples,       /* lower bound */
            Some(self.samples), /* upper bound (not obligatory, hence Some(_)) */
        )
    }
}

/* mark that the sound has a known, exact length */
impl ExactSizeIterator for BiSineWave { }
