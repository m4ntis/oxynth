use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{BufferSize, Sample, StreamConfig};
use oxynth::{Oscillator, OscillatorCmd};
use piston_window::*;
use std::sync::mpsc::Receiver;
use std::thread;

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

    let (mut osc, samples, cmds) = Oscillator::new(0.05, config.sample_rate.0);
    thread::spawn(move || osc.start());

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
                                if key == Key::NumPadPlus {
                                    cmds.send(OscillatorCmd::Transpose(1)).unwrap();
                                }
                                if key == Key::NumPadMinus {
                                    cmds.send(OscillatorCmd::Transpose(-1)).unwrap();
                                }

                                let freq = match_keys(key);
                                if freq != 0 {
                                    cmds.send(OscillatorCmd::Activate(freq)).unwrap();
                                    last = key;
                                }
                            }
                            ButtonState::Release => {
                                if key == last {
                                    cmds.send(OscillatorCmd::Deactivate).unwrap();
                                }
                            }
                        }
                    }
                }
                Input::Close(_) => {
                    cmds.send(OscillatorCmd::Stop).unwrap();
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
