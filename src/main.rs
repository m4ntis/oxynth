use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{BufferSize, Sample, StreamConfig};
use rand::prelude::*;
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

    let stream = device
        .build_output_stream(
            &config,
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                let mut rng = rand::thread_rng();

                for sample in data.iter_mut() {
                    let val = rng.gen::<f32>() * 2.0 - 1.0;
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
