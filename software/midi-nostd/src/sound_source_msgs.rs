#[derive(Clone, PartialEq, Debug)]
pub struct SoundSourceNoteInit {
    pub key: u8,
    pub instrument: u8,
}

impl SoundSourceNoteInit {
    pub fn new(key: u8, instrument: u8) -> Self {
        return Self { key, instrument };
    }
}
