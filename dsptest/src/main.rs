use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use iced::{
    executor, keyboard, Application, Clipboard, Column, Command, Element,
    Settings, Subscription, Text,
};
use iced_native::{window, Event};

fn main() -> iced::Result {

    SynthUI::run(Settings {
        exit_on_close_request: false,
        ..Settings::default()
    })
}

#[derive(Debug, Clone)]
pub enum Message {
    EventOccurred(iced_native::Event),
    Exit,
}

pub struct SynthUI {
    should_exit: bool,
    stream: cpal::Stream
}

impl Application for SynthUI {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = ();

    fn new(_flags: ()) -> (SynthUI, Command<Message>) {
        let host = cpal::default_host();
        let device = host.default_output_device()
            .expect("Failed to open output device");
    
        let config = cpal::StreamConfig {
            channels: 2,
            sample_rate: cpal::SampleRate(48000),
            buffer_size: cpal::BufferSize::Default
        };

        let stream = device.build_output_stream(&config,
            move | data: &mut [f32], _: &cpal::OutputCallbackInfo | {
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

        (SynthUI {
            should_exit: false,
            stream
        }, Command::none())
    }

    fn title(&self) -> String {
        String::from("Synth")
    }

    fn subscription(&self) -> Subscription<Message> {
        iced_native::subscription::events().map(Message::EventOccurred)
    }

    fn should_exit(&self) -> bool {
        self.should_exit
    }

    fn update(&mut self, message: Message, _clipboard: &mut Clipboard) -> Command<Message> {
        match message {
            Message::EventOccurred(event) => {
                if let Event::Window(window::Event::CloseRequested) = event {
                    self.should_exit = true;
                } else if let Event::Keyboard(keyboard::Event::KeyPressed {
                    key_code,
                    modifiers: _,
                }) = event
                {
                    println!("Key pressed: {}", key_code as u32);
                } else if let Event::Keyboard(keyboard::Event::KeyReleased {
                    key_code,
                    modifiers: _,
                }) = event
                {
                    println!("Key released: {}", key_code as u32);
                }
            }
            Message::Exit => {
                self.should_exit = true;
            }
        };

        Command::none()
    }

    fn view(&mut self) -> Element<'_, Self::Message> {
        Column::new()
            .padding(20)
            .push(Text::new("Hello, world!"))
            .into()
    }
}
