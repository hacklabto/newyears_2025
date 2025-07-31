use crate::electric_piano::ElectricPiano;
use crate::guitar_acoustic::GuitarAcoustic;
use crate::piano::Piano;
use crate::silence::Silence;
use crate::sound_sample::SoundSampleI32;
use crate::sound_source_core::SoundSourceCore;

#[derive(Clone, PartialEq, Debug)]
pub struct SoundSourceNoteInit {
    pub key: u8,
    pub instrument: u8,
    pub velocity: u8,
}

impl SoundSourceNoteInit {
    pub fn new(key: u8, instrument: u8, velocity: u8) -> Self {
        return Self {
            key,
            instrument,
            velocity,
        };
    }
}

pub enum NoteEnum<const PLAY_FREQUENCY: u32> {
    PianoEnum {
        pcore: Piano<PLAY_FREQUENCY>,
    },
    ElectricPianoEnum {
        pcore: ElectricPiano<PLAY_FREQUENCY>,
    },
    GuitarAcousticEnum {
        pcore: GuitarAcoustic<PLAY_FREQUENCY>,
    },
    SilenceEnum {
        pcore: Silence<PLAY_FREQUENCY>,
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
            NoteEnum::ElectricPianoEnum { pcore } => pcore.get_next(),
            NoteEnum::GuitarAcousticEnum { pcore } => pcore.get_next(),
            NoteEnum::SilenceEnum { pcore } => pcore.get_next(),
            NoteEnum::Unassigned => SoundSampleI32::ZERO,
        }
    }

    fn has_next(self: &Self) -> bool {
        match &self.core {
            NoteEnum::PianoEnum { pcore } => pcore.has_next(),
            NoteEnum::ElectricPianoEnum { pcore } => pcore.has_next(),
            NoteEnum::GuitarAcousticEnum { pcore } => pcore.has_next(),
            NoteEnum::SilenceEnum { pcore } => pcore.has_next(),
            NoteEnum::Unassigned => false,
        }
    }

    fn new(init_values: Self::InitValuesType) -> Self {
        let instrument = init_values.instrument;

        let core = match instrument {
            0 => {
                panic!("TODO");
                let pcore = Piano::<PLAY_FREQUENCY>::new(init_values);
                NoteEnum::<PLAY_FREQUENCY>::PianoEnum { pcore }
            }
            6 => {
                let pcore = ElectricPiano::<PLAY_FREQUENCY>::new(init_values);
                NoteEnum::<PLAY_FREQUENCY>::ElectricPianoEnum { pcore }
            }
            16 => {
                // Dulcimer
                let pcore = Silence::<PLAY_FREQUENCY>::new(init_values);
                NoteEnum::<PLAY_FREQUENCY>::SilenceEnum { pcore }
            }
            40 => {
                // Violin
                let pcore = Silence::<PLAY_FREQUENCY>::new(init_values);
                NoteEnum::<PLAY_FREQUENCY>::SilenceEnum { pcore }
            }
            42 => {
                // Cello
                let pcore = Silence::<PLAY_FREQUENCY>::new(init_values);
                NoteEnum::<PLAY_FREQUENCY>::SilenceEnum { pcore }
            }
            48 => {
                // Timpani
                let pcore = Silence::<PLAY_FREQUENCY>::new(init_values);
                NoteEnum::<PLAY_FREQUENCY>::SilenceEnum { pcore }
            }
            _ => {
                panic!("TODO");
                let pcore = Silence::<PLAY_FREQUENCY>::new(init_values);
                NoteEnum::<PLAY_FREQUENCY>::SilenceEnum { pcore }
            }
        };

        return Self { core };
    }

    fn trigger_note_off(self: &mut Self) {
        match &mut self.core {
            NoteEnum::PianoEnum { pcore } => pcore.trigger_note_off(),
            NoteEnum::ElectricPianoEnum { pcore } => pcore.trigger_note_off(),
            NoteEnum::GuitarAcousticEnum { pcore } => pcore.trigger_note_off(),
            NoteEnum::SilenceEnum { pcore } => pcore.trigger_note_off(),
            NoteEnum::Unassigned => {}
        }
    }
}
