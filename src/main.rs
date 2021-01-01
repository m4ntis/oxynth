use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{BufferSize, Sample, StreamConfig};
use piston_window::*;
use std::sync::mpsc::{channel, sync_channel, Receiver, Sender, SyncSender};
use std::thread;

pub struct Oscillator {
    pub loudness: f32,

    sample_rate: u32,

    wavelength: u32,
    pos: u32,

    samples: SyncSender<f32>,
    freqs: Receiver<u32>,
    quit: Receiver<bool>,
}

impl Oscillator {
    pub fn new(
        loudness: f32,
        sample_rate: u32,
    ) -> (Oscillator, Receiver<f32>, Sender<u32>, SyncSender<bool>) {
        let (send_samples, recv_samples) = sync_channel::<f32>(10);
        let (send_freqs, recv_freqs) = channel::<u32>();
        let (quit_s, quit_r) = sync_channel::<bool>(0);

        (
            Oscillator {
                loudness,

                sample_rate,

                wavelength: 0,
                pos: 0,

                samples: send_samples,
                freqs: recv_freqs,
                quit: quit_r,
            },
            recv_samples,
            send_freqs,
            quit_s,
        )
    }

    pub fn run(&mut self) {
        loop {
            if let Ok(_) = self.quit.try_recv() {
                return;
            }

            if let Ok(freq) = self.freqs.try_recv() {
                if freq == 0 {
                    self.wavelength = 0
                } else {
                    self.wavelength = self.sample_rate / freq * 2
                }
            }

            let sample = self.next();
            self.samples.send(sample).unwrap();
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

        // Return 0 for a deactivated ooscillator
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

fn match_keys(key: Key) -> u32 {
    match key {
        Key::A => 349,           // F4
        Key::W => 370,           // F#4
        Key::S => 392,           // G4
        Key::E => 415,           // G#4
        Key::D => 440,           // A4
        Key::R => 466,           // A#4
        Key::F => 494,           // B4
        Key::G => 523,           // C5
        Key::Y => 554,           // C#5
        Key::H => 587,           // D5
        Key::U => 622,           // D#5
        Key::J => 659,           // E5
        Key::K => 698,           // F5
        Key::O => 740,           // F#5
        Key::L => 784,           // G5
        Key::P => 831,           // G#5
        Key::Semicolon => 880,   // A5
        Key::LeftBracket => 932, // A#5
        Key::Unknown => 988,     // B5
        Key::Backslash => 1046,  // C6
        _ => 0,
    }
}

fn main() {
    // Setup and configure audio device for default device
    let host = cpal::default_host();
    let (device, config) = setup_default_device(host);

    let (mut osc, samples, freqs, quit) = Oscillator::new(0.05, config.sample_rate.0);
    thread::spawn(move || osc.run());

    let stream = build_stream(device, &config, samples);
    stream.play().unwrap();

    let mut window: PistonWindow = WindowSettings::new("rynth", (640, 320))
        .exit_on_esc(true)
        .build()
        .unwrap();

    let mut last = Key::A;

    while let Some(e) = window.next() {
        match e {
            Event::Input(inp, _) => match inp {
                Input::Button(args) => {
                    if let Button::Keyboard(key) = args.button {
                        match args.state {
                            ButtonState::Press => {
                                let freq = match_keys(key);
                                if freq != 0 {
                                    freqs.send(freq).unwrap();
                                    last = key;
                                }
                            }
                            ButtonState::Release => {
                                if key == last {
                                    freqs.send(0).unwrap();
                                }
                            }
                        }
                    }
                }
                Input::Close(_) => {
                    quit.send(true).unwrap();
                }
                _ => {}
            },
            _ => {}
        }
    }
}

// Sets up a host's default device and build defaul output config for a stream
fn setup_default_device<T: HostTrait>(host: T) -> (T::Device, StreamConfig) {
    let device = host
        .default_output_device()
        .expect("no output device available");

    let supported_config = device.default_output_config().expect("no output config");
    let config = StreamConfig {
        channels: supported_config.channels(),
        sample_rate: supported_config.sample_rate(),
        buffer_size: BufferSize::Default,
    };

    (device, config)
}

// Builds an output stream to a device, configured by config and reading from
// samples
fn build_stream<T: DeviceTrait>(
    device: T,
    config: &StreamConfig,
    samples: Receiver<f32>,
) -> T::Stream {
    device
        .build_output_stream(
            &config,
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                for sample in data.iter_mut() {
                    match samples.recv() {
                        Ok(sample_val) => {
                            *sample = Sample::from(&sample_val);
                        }
                        Err(_) => {
                            // Default to sending 0 on err
                            *sample = Sample::from(&0.0);
                        }
                    }
                }
            },
            move |err| {
                eprintln!("an error occurred on the output audio stream: {}", err);
            },
        )
        .unwrap()
}
