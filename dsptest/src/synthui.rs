use iced::{
    executor, keyboard, Application, Clipboard, Column, Command, Element, Settings, Subscription,
    Text,
};
use iced_native::{window, Event};
use std::{collections::HashMap, sync::mpsc};

use crate::synthmessage::SynthMessage;

pub struct SynthUI {
    should_exit: bool,
    tx: mpsc::Sender<SynthMessage>,
    key_map: HashMap<u8, bool>,
}

#[derive(Default)]
pub struct AppFlags {
    pub(crate) tx: Option<mpsc::Sender<SynthMessage>>,
}

#[derive(Debug, Clone)]
pub enum Message {
    EventOccurred(iced_native::Event),
    Exit,
}

impl Application for SynthUI {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = AppFlags;

    fn new(flags: AppFlags) -> (SynthUI, Command<Message>) {
        (
            SynthUI {
                should_exit: false,
                tx: flags.tx.expect("No TX"),
                key_map: HashMap::new(),
            },
            Command::none(),
        )
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
        let mut synth_message: Option<SynthMessage> = None;

        match message {
            Message::EventOccurred(event) => {
                if let Event::Window(window::Event::CloseRequested) = event {
                    self.should_exit = true;
                } else if let Event::Keyboard(keyboard::Event::KeyPressed {
                    key_code,
                    modifiers: _,
                }) = event
                {
                    synth_message = self.update_midi_keys(key_code, true);
                } else if let Event::Keyboard(keyboard::Event::KeyReleased {
                    key_code,
                    modifiers: _,
                }) = event
                {
                    synth_message = self.update_midi_keys(key_code, false);
                }
            }
            Message::Exit => {
                self.should_exit = true;
            }
        };

        if let Some(msg) = synth_message {
            self.tx.send(msg).expect("Unable to send message");
        }

        Command::none()
    }

    fn view(&mut self) -> Element<'_, Self::Message> {
        Column::new()
            .padding(20)
            .push(Text::new("Hello, world!"))
            .into()
    }
}

impl SynthUI {
    fn key_to_note(code: keyboard::KeyCode) -> Option<u8> {
        use keyboard::KeyCode as K;
        match code {
            K::A => Some(60),
            K::W => Some(61),
            K::S => Some(62),
            K::E => Some(63),
            K::D => Some(64),
            K::F => Some(65),
            K::T => Some(66),
            K::G => Some(67),
            K::Y => Some(68),
            K::H => Some(69),
            K::U => Some(70),
            K::J => Some(80),
            K::K => Some(81),
            K::O => Some(82),
            K::L => Some(83),
            K::P => Some(84),
            K::Semicolon => Some(85),
            K::Apostrophe => Some(86),
            _ => None,
        }
    }

    fn update_midi_keys(&mut self, code: keyboard::KeyCode, pressed: bool) -> Option<SynthMessage> {
        if let Some(note) = SynthUI::key_to_note(code) {
            if let Some(was_pressed) = self.key_map.get(&note) {
                if *was_pressed != pressed {
                    self.key_map.insert(note, pressed);

                    return Some(if pressed {
                        SynthMessage::NoteOn(note)
                    } else {
                        SynthMessage::NoteOff(note)
                    });
                }
            } else {
                self.key_map.insert(note, pressed);

                return Some(if pressed {
                    SynthMessage::NoteOn(note)
                } else {
                    SynthMessage::NoteOff(note)
                });
            }
        }
        
        None
    }
}
