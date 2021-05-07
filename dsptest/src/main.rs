use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use iced::{Application, Settings};
use synthmessage::SynthMessage;
use synthui::SynthUI;

use std::sync::mpsc;

use crate::synthui::AppFlags;

mod synthui;
pub mod synthmessage;

fn main() -> iced::Result {
    let host = cpal::default_host();
        let device = host.default_output_device()
            .expect("Failed to open output device");
    
        let config = cpal::StreamConfig {
            channels: 2,
            sample_rate: cpal::SampleRate(48000),
            buffer_size: cpal::BufferSize::Default
        };

        // Set up a channel to communicate with the audio thread
        let (tx, rx) = mpsc::channel::<SynthMessage>();

        let stream = device.build_output_stream(&config,
            move | data: &mut [f32], _: &cpal::OutputCallbackInfo | {
                for msg in rx.try_iter() {
                    println!("{:?}", msg);
                }

                for i in 0..data.len() {
                    if i > 0 {
                        data[i] = (rand::random::<f32>() + data[i-1]) / 2.0;
                    } else {
                        data[i] = (rand::random::<f32>() + data[data.len() - 1]) / 2.0;
                    }
                }
            },
            move |err| {
                eprintln!("an error occurred on stream: {}", err);
            } 
        ).unwrap();

        stream.play().expect("Could not play stream");

    SynthUI::run(Settings {
        exit_on_close_request: false,
        flags: AppFlags {  tx: Some(tx) },
        ..Settings::default()
    })
}
