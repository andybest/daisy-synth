use iced::{
    executor, keyboard, Application, Clipboard, Column, Command, Element, Settings, Subscription,
    Text,
};
use iced_native::{window, Event};
use std::sync::mpsc;

use crate::synthmessage::SynthMessage;

pub struct SynthUI {
    should_exit: bool,
    tx: mpsc::Sender<SynthMessage>,
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

        self.tx.send(SynthMessage::Note).expect("Unable to send");
        self.tx.send(SynthMessage::Foo).expect("Unable to send");

        Command::none()
    }

    fn view(&mut self) -> Element<'_, Self::Message> {
        Column::new()
            .padding(20)
            .push(Text::new("Hello, world!"))
            .into()
    }
}
