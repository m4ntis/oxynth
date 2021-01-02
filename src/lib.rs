use std::sync::mpsc::{channel, sync_channel, Receiver, Sender, SyncSender};

pub struct Oscillator {
    loudness: f32,
    transpose: i32,

    sample_rate: u32,

    wavelength: u32,
    pos: u32,

    samples: SyncSender<f32>,

    freqs: Receiver<u32>,
    transpose_r: Receiver<i32>,
    quit: Receiver<bool>,
}

impl Oscillator {
    pub fn new(
        loudness: f32,
        sample_rate: u32,
    ) -> (
        Oscillator,
        Receiver<f32>,
        Sender<u32>,
        Sender<i32>,
        SyncSender<bool>,
    ) {
        let (samples_s, samples_r) = sync_channel::<f32>(10);
        let (freqs_s, freqs_r) = channel::<u32>();
        let (transpose_s, transpose_r) = channel::<i32>();
        let (quit_s, quit_r) = sync_channel::<bool>(0);

        (
            Oscillator {
                loudness,
                transpose: 0,

                sample_rate,

                wavelength: 0,
                pos: 0,

                samples: samples_s,

                freqs: freqs_r,
                transpose_r,
                quit: quit_r,
            },
            samples_r,
            freqs_s,
            transpose_s,
            quit_s,
        )
    }

    pub fn run(&mut self) {
        loop {
            if let Ok(_) = self.quit.try_recv() {
                return;
            }

            if let Ok(transpose) = self.transpose_r.try_recv() {
                self.transpose += transpose;
                // TODO: Update on the spot or from next note?
            }

            if let Ok(freq) = self.freqs.try_recv() {
                self.set_wavelength(freq);
            }

            let sample = self.next();
            self.samples.send(sample).unwrap();
        }
    }

    fn set_wavelength(&mut self, freq: u32) {
        if freq == 0 {
            self.wavelength = 0;
        } else {
            if self.transpose == 0 {
                self.wavelength = self.sample_rate / freq * 2;
                return;
            }
            if self.transpose < 0 {
                self.wavelength = (self.sample_rate / freq * 2) * (2 * -self.transpose) as u32;
            } else {
                self.wavelength = (self.sample_rate / freq * 2) / (2 * self.transpose) as u32;
            }
        }
    }

    // Calculates next value determined by pos, factoring loudness
    fn next(&mut self) -> f32 {
        self.next_abs() * self.loudness
    }

    // Calculates next value determined by pos, disreguarding loudness
    fn next_abs(&mut self) -> f32 {
        // Advance position
        self.pos += 1;
        if self.pos > self.wavelength {
            self.pos = 0;
        }

        // Return 0 for a deactivated oscillator
        if self.wavelength == 0 {
            return 0.0;
        }

        // Calc sample value determined by position in wave
        if self.pos < self.wavelength / 2 {
            1.0
        } else {
            -1.0
        }
    }
}
