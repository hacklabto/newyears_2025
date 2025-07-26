use crate::piano::Piano;
use crate::sound_sample::SoundSampleI32;
use crate::sound_source_core::SoundSourceCore;

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

///
/// Note.  Now sort of a proof of concept.
///
pub struct Note<const PLAY_FREQUENCY: u32> {
    core: Piano<PLAY_FREQUENCY>,
}

impl<const PLAY_FREQUENCY: u32> Default for Note<PLAY_FREQUENCY> {
    fn default() -> Self {
        Self {
            core: Piano::default(),
        }
    }
}

impl<const PLAY_FREQUENCY: u32> SoundSourceCore<PLAY_FREQUENCY> for Note<PLAY_FREQUENCY> {
    type InitValuesType = SoundSourceNoteInit;

    fn get_next(self: &mut Self) -> SoundSampleI32 {
        self.core.get_next()
    }

    fn has_next(self: &Self) -> bool {
        self.core.has_next()
    }

    fn init(&mut self, init_values: &Self::InitValuesType) {
        self.core.init(init_values);
    }

    fn trigger_note_off(self: &mut Self) {
        self.core.trigger_note_off();
    }
}
