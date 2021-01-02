use std::sync::mpsc::{sync_channel, Receiver, SyncSender};

/// A command to control an Oscillator.
pub enum OscillatorCmd {
    /// Activate / change Oscillator frequency.
    Activate(u32),
    /// Deactivate Oscillator.
    Deactivate,
    /// Transpose Oscillator Up or Down by octaves.
    Transpose(i32),
    /// Stop cleanly stops the Oscillator. A stopped Oscillator needs to be re-started in order to
    /// execute new commands.
    Stop,
}

/// An Oscillator used for a single voice.
pub struct Oscillator {
    loudness: f32,
    transpose: i32,

    sample_rate: u32,

    wavelength: u32,
    pos: u32,

    samples: SyncSender<f32>,
    cmds: Receiver<OscillatorCmd>,
}

impl Oscillator {
    /// Creates a new Oscillator.
    ///
    /// new returns the Oscillator, as well as a samples receiver and an
    /// OscillatorCmd sender.
    pub fn new(
        loudness: f32,
        sample_rate: u32,
    ) -> (Oscillator, Receiver<f32>, SyncSender<OscillatorCmd>) {
        let (samples_s, samples_r) = sync_channel::<f32>(10);
        let (cmd_s, cmd_r) = sync_channel::<OscillatorCmd>(5);

        (
            Oscillator {
                loudness,
                transpose: 0,

                sample_rate,

                wavelength: 0,
                pos: 0,

                samples: samples_s,
                cmds: cmd_r,
            },
            samples_r,
            cmd_s,
        )
    }

    /// Starts the Oscillator, should be called on it's own thread.
    ///
    /// The Oscillator will start publishing samples, and process incoming
    /// commands.
    pub fn start(&mut self) {
        loop {
            if let Ok(cmd) = self.cmds.try_recv() {
                match cmd {
                    OscillatorCmd::Activate(freq) => {
                        self.set_wavelength(freq);
                    }
                    OscillatorCmd::Deactivate => {
                        self.set_wavelength(0);
                    }
                    OscillatorCmd::Transpose(transpose) => {
                        // TODO: Update on the spot or from next note?
                        self.transpose += transpose;
                    }
                    OscillatorCmd::Stop => {
                        // TODO: Should consume rest of cmds?
                        return;
                    }
                }
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
