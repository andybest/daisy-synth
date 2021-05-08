#[derive(Debug)]
pub enum SynthMessage {
    NoteOn(u8),
    NoteOff(u8)
}