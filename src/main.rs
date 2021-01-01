use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{BufferSize, Sample, StreamConfig};
use std::io;

fn main() {
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .expect("no output device available");
    let supported_config = device.default_output_config().expect("no output config");
    let config = StreamConfig {
        channels: supported_config.channels(),
        sample_rate: supported_config.sample_rate(),
        buffer_size: BufferSize::Default,
    };

    let mut note = Oscilator::new(440, 0.05, config.sample_rate.0);

    let stream = device
        .build_output_stream(
            &config,
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                for sample in data.iter_mut() {
                    let val = note.next();
                    *sample = Sample::from(&val);
                }
            },
            move |err| {
                eprintln!("an error occurred on the output audio stream: {}", err);
            },
        )
        .unwrap();

    stream.play().unwrap();

    io::stdin().read_line(&mut String::new()).unwrap();
}

pub struct Oscilator {
    wavelength: u32,
    pos: u32,

    loudness: f32,
}

impl Oscilator {
    pub fn new(freq: u32, loudness: f32, rate: u32) -> Oscilator {
        Oscilator {
            wavelength: rate / freq,
            pos: 0,
            loudness,
        }
    }

    pub fn next(&mut self) -> f32 {
        self.calc_next() * self.loudness
    }

    fn calc_next(&mut self) -> f32 {
        self.pos += 1;
        if self.pos > self.wavelength {
            self.pos = 0;
        }

        if self.pos < self.wavelength / 2 {
            1.0
        } else {
            -1.0
        }
    }
}
