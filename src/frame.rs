use rodio::Sink;
use crate::bi_sine_wave::BiSineWave;

#[derive(Clone, Copy, Debug)]
pub struct Band {
    pub clk: u32,
    pub base: u32,
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

    /* plays a frame on the sink `sink` for `secs` seconds corresponding to *
     * byte `byte` and with clock low or high depending on `clk`            */
    pub fn build(&mut self, clk: bool, maybe_byte: u32, sink: &Sink, secs: f32) {
        /* map byte to a frequency in range [base, base+scale*256] */
        let data_freq = (self.band.base + self.band.scale * maybe_byte as u32) as f32;
        let clk_freq = self.band.clk as f32;

        /* always play byte_freq, only play clock freq if `clk`==true */
        let freqs = (
            data_freq,
            if clk { Some(clk_freq) } else { None }
        );

        let ratio = clk_freq / data_freq;

        let source = BiSineWave::new(
            /* freqs to play */
            freqs,
            /* a0/a1 = f0/f1 because it seems to work well duh */
            ratio,
            /* sample rate */
            48000,
            /* duration in samples */
            (secs * 48000f32) as usize,
        );

        /* play it */
        sink.append(source);
    }
}
