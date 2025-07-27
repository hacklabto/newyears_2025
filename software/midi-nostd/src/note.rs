use crate::guitar_acoustic::GuitarAcoustic;
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

pub enum NoteEnum<const PLAY_FREQUENCY: u32> {
    PianoEnum {
        pcore: Piano<PLAY_FREQUENCY>,
    },
    GuitarAcousticEnum {
        pcore: GuitarAcoustic<PLAY_FREQUENCY>,
    },
    Unassigned,
}

///
/// Note.  Now sort of a proof of concept.
///
pub struct Note<const PLAY_FREQUENCY: u32> {
    core: NoteEnum<PLAY_FREQUENCY>,
}

impl<const PLAY_FREQUENCY: u32> Default for Note<PLAY_FREQUENCY> {
    fn default() -> Self {
        Self {
            core: NoteEnum::<PLAY_FREQUENCY>::Unassigned,
        }
    }
}

impl<const PLAY_FREQUENCY: u32> SoundSourceCore<PLAY_FREQUENCY> for Note<PLAY_FREQUENCY> {
    type InitValuesType = SoundSourceNoteInit;

    fn get_next(self: &mut Self) -> SoundSampleI32 {
        match &mut self.core {
            NoteEnum::PianoEnum { pcore } => pcore.get_next(),
            NoteEnum::GuitarAcousticEnum { pcore } => pcore.get_next(),
            NoteEnum::Unassigned => SoundSampleI32::ZERO,
        }
    }

    fn has_next(self: &Self) -> bool {
        match &self.core {
            NoteEnum::PianoEnum { pcore } => pcore.has_next(),
            NoteEnum::GuitarAcousticEnum { pcore } => pcore.has_next(),
            NoteEnum::Unassigned => false,
        }
    }

    fn init(&mut self, init_values: &Self::InitValuesType) {
        let mut pcore = GuitarAcoustic::<PLAY_FREQUENCY>::default();
        GuitarAcoustic::<PLAY_FREQUENCY>::init(&mut pcore, init_values);
        let test = NoteEnum::<PLAY_FREQUENCY>::GuitarAcousticEnum { pcore };
        self.core = test;
    }

    fn trigger_note_off(self: &mut Self) {
        match &mut self.core {
            NoteEnum::PianoEnum { pcore } => pcore.trigger_note_off(),
            NoteEnum::GuitarAcousticEnum { pcore } => pcore.trigger_note_off(),
            NoteEnum::Unassigned => {}
        }
    }
}
