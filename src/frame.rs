use crate::maybe_sine_wave::*;
use crate::Config;
use rodio::Sink;

#[derive(Debug)]
pub struct FrameBuilder {
    conf: Config,
}

impl FrameBuilder {
    pub fn new(conf: Config) -> Self {
        FrameBuilder { conf }
    }

    /* plays a frame on the sink `sink` for `secs` seconds corresponding to *
     * byte `byte` and with clock low or high depending on `clk`            */
    pub fn build(&mut self, maybe_byte: Option<u32>, sink: &Sink, secs: f32) {
        /* map byte to a frequency in range [base, base+scale*256] */
        let data_freq = maybe_byte.map(|b| (self.conf.base + self.conf.scale * b) as f32);
        let source = MaybeSineWave::new(
            /* freq to play */
            data_freq,
            /* sample rate */
            48000,
            /* duration in samples */
            (secs * 48000f32) as usize,
        );

        /* play it */
        sink.append(source);
    }
}
