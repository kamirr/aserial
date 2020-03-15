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
    pub fn build(&mut self, clk: bool, byte: u8, sink: &Sink, secs: f32) {
        /* map byte to a frequency in range [base, base+scale*255] */
        let byte_freq = (self.band.base + self.band.scale * byte as u32) as f32;

        /* always play byte_freq, only play clock freq if `clk`==true */
        let freqs = (
            byte_freq,
            if clk { Some(self.band.clk as f32) } else { None }
        );

        let source = BiSineWave::new(
            freqs,                      /* freqs to play */
            48000,                      /* sample rate */
            (secs * 48000f32) as usize, /* duration in samples */
        );

        /* play it */
        sink.append(source);
    }
}
