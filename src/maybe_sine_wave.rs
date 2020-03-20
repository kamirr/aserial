use rodio::source::Source;

pub struct MaybeSineWave {
    /* only play a freq if it's Some(_) */
    freq: Option<f32>,
    sample_rate: u32,
    /* stop playing after this many samples */
    samples: usize,
    /* used internally to keep track of time */
    num_sample: usize,
}

impl MaybeSineWave {
    pub fn new(
        freq: Option<f32>,
        sample_rate: u32,
        samples: usize,
    ) -> Self {
        MaybeSineWave {
            freq,
            sample_rate,
            samples,
            num_sample: 0,
        }
    }
}

impl Source for MaybeSineWave {
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
        let nsecs = 1_000_000_000 as f32 * self.samples as f32 / self.sample_rate as f32;
        Some(std::time::Duration::new(0, nsecs as u32))
    }
}

impl Iterator for MaybeSineWave {
    type Item = f32;

    #[inline]
    fn next(&mut self) -> Option<f32> {
        /* check if the sound has any more samples to play */
        if self.num_sample < self.samples {
            self.num_sample += 1;

            /* time in secs */
            let t = self.num_sample as f32 / self.sample_rate as f32;
            /* multiply by 2π so that 1Hz corresponds to one period of sine */
            let cnst = 2.0 * std::f32::consts::PI * t;
            
            Some(match self.freq {
                Some(f) => (f * cnst).sin(),
                None => 0.0,
            })
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
impl ExactSizeIterator for MaybeSineWave {}
