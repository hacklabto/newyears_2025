use crate::bass::Bass;
use crate::cello::Cello;
use crate::choir::Choir;
use crate::electric_piano::ElectricPiano;
use crate::french_horn::FrenchHorn;
use crate::guitar_acoustic::GuitarAcoustic;
use crate::piano::Piano;
use crate::sax::Sax;
use crate::silence::Silence;
use crate::sound_sample::SoundSampleI32;
use crate::sound_source_core::SoundSourceCore;
use crate::violin::Violin;

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
    CelloEnum {
        pcore: Cello<PLAY_FREQUENCY>,
    },
    ViolinEnum {
        pcore: Violin<PLAY_FREQUENCY>,
    },
    ChoirEnum {
        pcore: Choir<PLAY_FREQUENCY>,
    },
    FrenchHornEnum {
        pcore: FrenchHorn<PLAY_FREQUENCY>,
    },
    BassEnum {
        pcore: Bass<PLAY_FREQUENCY>,
    },
    SaxEnum {
        pcore: Sax<PLAY_FREQUENCY>,
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
            NoteEnum::CelloEnum { pcore } => pcore.get_next(),
            NoteEnum::ViolinEnum { pcore } => pcore.get_next(),
            NoteEnum::ChoirEnum { pcore } => pcore.get_next(),
            NoteEnum::FrenchHornEnum { pcore } => pcore.get_next(),
            NoteEnum::BassEnum { pcore } => pcore.get_next(),
            NoteEnum::SaxEnum { pcore } => pcore.get_next(),
            NoteEnum::Unassigned => SoundSampleI32::ZERO,
        }
    }

    fn has_next(self: &Self) -> bool {
        match &self.core {
            NoteEnum::PianoEnum { pcore } => pcore.has_next(),
            NoteEnum::ElectricPianoEnum { pcore } => pcore.has_next(),
            NoteEnum::GuitarAcousticEnum { pcore } => pcore.has_next(),
            NoteEnum::SilenceEnum { pcore } => pcore.has_next(),
            NoteEnum::CelloEnum { pcore } => pcore.has_next(),
            NoteEnum::ViolinEnum { pcore } => pcore.has_next(),
            NoteEnum::ChoirEnum { pcore } => pcore.has_next(),
            NoteEnum::FrenchHornEnum { pcore } => pcore.has_next(),
            NoteEnum::BassEnum { pcore } => pcore.has_next(),
            NoteEnum::SaxEnum { pcore } => pcore.has_next(),
            NoteEnum::Unassigned => false,
        }
    }

    fn new(init_values: Self::InitValuesType) -> Self {
        let instrument = init_values.instrument;

        let core = match instrument {
            0 => {
                let pcore = Piano::<PLAY_FREQUENCY>::new(init_values);
                NoteEnum::<PLAY_FREQUENCY>::PianoEnum { pcore }
                //let pcore = Cello::<PLAY_FREQUENCY>::new(init_values);
                //NoteEnum::<PLAY_FREQUENCY>::CelloEnum { pcore }
                //let pcore = ElectricPiano::<PLAY_FREQUENCY>::new(init_values);
                //NoteEnum::<PLAY_FREQUENCY>::ElectricPianoEnum { pcore }
                //let pcore = Violin::<PLAY_FREQUENCY>::new(init_values);
                //NoteEnum::<PLAY_FREQUENCY>::ViolinEnum { pcore }
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
            30 => {
                // Distortion Guitar.  TODO
                let pcore = GuitarAcoustic::<PLAY_FREQUENCY>::new(init_values);
                NoteEnum::<PLAY_FREQUENCY>::GuitarAcousticEnum { pcore }
            }
            33 => {
                // Bass
                let pcore = Bass::<PLAY_FREQUENCY>::new(init_values);
                NoteEnum::<PLAY_FREQUENCY>::BassEnum { pcore }
            }
            40 => {
                // Violin
                let pcore = Violin::<PLAY_FREQUENCY>::new(init_values);
                NoteEnum::<PLAY_FREQUENCY>::ViolinEnum { pcore }
            }
            42 => {
                // Cello
                let pcore = Cello::<PLAY_FREQUENCY>::new(init_values);
                NoteEnum::<PLAY_FREQUENCY>::CelloEnum { pcore }
            }
            48 => {
                // Timpani
                let pcore = Silence::<PLAY_FREQUENCY>::new(init_values);
                NoteEnum::<PLAY_FREQUENCY>::SilenceEnum { pcore }
            }
            52 => {
                //Choir
                let pcore = Choir::<PLAY_FREQUENCY>::new(init_values);
                NoteEnum::<PLAY_FREQUENCY>::ChoirEnum { pcore }
            }
            60 => {
                //French Horn
                let pcore = FrenchHorn::<PLAY_FREQUENCY>::new(init_values);
                NoteEnum::<PLAY_FREQUENCY>::FrenchHornEnum { pcore }
            }
            65 => {
                //Sax
                let pcore = Sax::<PLAY_FREQUENCY>::new(init_values);
                NoteEnum::<PLAY_FREQUENCY>::SaxEnum { pcore }
            }
            _ => {
                //assert_eq!(0, instrument);
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
            NoteEnum::CelloEnum { pcore } => pcore.trigger_note_off(),
            NoteEnum::ViolinEnum { pcore } => pcore.trigger_note_off(),
            NoteEnum::ChoirEnum { pcore } => pcore.trigger_note_off(),
            NoteEnum::FrenchHornEnum { pcore } => pcore.trigger_note_off(),
            NoteEnum::BassEnum { pcore } => pcore.trigger_note_off(),
            NoteEnum::SaxEnum { pcore } => pcore.trigger_note_off(),
            NoteEnum::Unassigned => {}
        }
    }
}
