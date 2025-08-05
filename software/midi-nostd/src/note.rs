use crate::bass::Bass;
use crate::cello::Cello;
use crate::choir::Choir;
use crate::electric_piano::ElectricPiano;
use crate::french_horn::FrenchHorn;
use crate::guitar_acoustic::GuitarAcoustic;
use crate::oboe::Oboe;
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

pub enum NoteEnum<const P_FREQ: u32, const U_FREQ: u32> {
    PianoEnum {
        pcore: Piano<P_FREQ, U_FREQ>,
    },
    ElectricPianoEnum {
        pcore: ElectricPiano<P_FREQ, U_FREQ>,
    },
    GuitarAcousticEnum {
        pcore: GuitarAcoustic<P_FREQ, U_FREQ>,
    },
    SilenceEnum {
        pcore: Silence<P_FREQ, U_FREQ>,
    },
    CelloEnum {
        pcore: Cello<P_FREQ, U_FREQ>,
    },
    ViolinEnum {
        pcore: Violin<P_FREQ, U_FREQ>,
    },
    ChoirEnum {
        pcore: Choir<P_FREQ, U_FREQ>,
    },
    FrenchHornEnum {
        pcore: FrenchHorn<P_FREQ, U_FREQ>,
    },
    BassEnum {
        pcore: Bass<P_FREQ, U_FREQ>,
    },
    SaxEnum {
        pcore: Sax<P_FREQ, U_FREQ>,
    },
    OboeEnum {
        pcore: Oboe<P_FREQ, U_FREQ>,
    },
    Unassigned,
}

///
/// Note.  Now sort of a proof of concept.
///
pub struct Note<const P_FREQ: u32, const U_FREQ: u32> {
    core: NoteEnum<P_FREQ, U_FREQ>,
}

impl<const P_FREQ: u32, const U_FREQ: u32> Default for Note<P_FREQ, U_FREQ> {
    fn default() -> Self {
        Self {
            core: NoteEnum::<P_FREQ, U_FREQ>::Unassigned,
        }
    }
}

impl<const P_FREQ: u32, const U_FREQ: u32> SoundSourceCore<P_FREQ, U_FREQ>
    for Note<P_FREQ, U_FREQ>
{
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
            NoteEnum::OboeEnum { pcore } => pcore.get_next(),
            NoteEnum::Unassigned => SoundSampleI32::ZERO,
        }
    }

    fn update(self: &mut Self) {
        match &mut self.core {
            NoteEnum::PianoEnum { pcore } => pcore.update(),
            NoteEnum::ElectricPianoEnum { pcore } => pcore.update(),
            NoteEnum::GuitarAcousticEnum { pcore } => pcore.update(),
            NoteEnum::SilenceEnum { pcore } => pcore.update(),
            NoteEnum::CelloEnum { pcore } => pcore.update(),
            NoteEnum::ViolinEnum { pcore } => pcore.update(),
            NoteEnum::ChoirEnum { pcore } => pcore.update(),
            NoteEnum::FrenchHornEnum { pcore } => pcore.update(),
            NoteEnum::BassEnum { pcore } => pcore.update(),
            NoteEnum::SaxEnum { pcore } => pcore.update(),
            NoteEnum::OboeEnum { pcore } => pcore.update(),
            NoteEnum::Unassigned => {}
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
            NoteEnum::OboeEnum { pcore } => pcore.has_next(),
            NoteEnum::Unassigned => false,
        }
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
            NoteEnum::OboeEnum { pcore } => pcore.trigger_note_off(),
            NoteEnum::Unassigned => {}
        }
    }

    fn restart(self: &mut Self, vel: u8) {
        match &mut self.core {
            NoteEnum::PianoEnum { pcore } => pcore.restart(vel),
            NoteEnum::ElectricPianoEnum { pcore } => pcore.restart(vel),
            NoteEnum::GuitarAcousticEnum { pcore } => pcore.restart(vel),
            NoteEnum::SilenceEnum { pcore } => pcore.restart(vel),
            NoteEnum::CelloEnum { pcore } => pcore.restart(vel),
            NoteEnum::ViolinEnum { pcore } => pcore.restart(vel),
            NoteEnum::ChoirEnum { pcore } => pcore.restart(vel),
            NoteEnum::FrenchHornEnum { pcore } => pcore.restart(vel),
            NoteEnum::BassEnum { pcore } => pcore.restart(vel),
            NoteEnum::SaxEnum { pcore } => pcore.restart(vel),
            NoteEnum::OboeEnum { pcore } => pcore.restart(vel),
            NoteEnum::Unassigned => {}
        }
    }

    fn new(init_values: Self::InitValuesType) -> Self {
        let instrument = init_values.instrument;

        let core = match instrument {
            0 => {
                let pcore = Piano::<P_FREQ, U_FREQ>::new(init_values);
                NoteEnum::<P_FREQ, U_FREQ>::PianoEnum { pcore }
            }
            3 => {
                // Electric Grand Piano
                let pcore = Piano::<P_FREQ, U_FREQ>::new(init_values);
                NoteEnum::<P_FREQ, U_FREQ>::PianoEnum { pcore }
            }
            6 => {
                let pcore = ElectricPiano::<P_FREQ, U_FREQ>::new(init_values);
                NoteEnum::<P_FREQ, U_FREQ>::ElectricPianoEnum { pcore }
            }
            16 => {
                // Dulcimer
                let pcore = Silence::<P_FREQ, U_FREQ>::new(init_values);
                NoteEnum::<P_FREQ, U_FREQ>::SilenceEnum { pcore }
            }
            24 => {
                // "Tango Accordian".  TODO
                let pcore = GuitarAcoustic::<P_FREQ, U_FREQ>::new(init_values);
                NoteEnum::<P_FREQ, U_FREQ>::GuitarAcousticEnum { pcore }
            }
            30 => {
                // Distortion Guitar.  TODO
                let pcore = GuitarAcoustic::<P_FREQ, U_FREQ>::new(init_values);
                NoteEnum::<P_FREQ, U_FREQ>::GuitarAcousticEnum { pcore }
            }
            33 => {
                // Bass
                let pcore = Bass::<P_FREQ, U_FREQ>::new(init_values);
                NoteEnum::<P_FREQ, U_FREQ>::BassEnum { pcore }
            }
            40 => {
                // Violin
                let pcore = Violin::<P_FREQ, U_FREQ>::new(init_values);
                NoteEnum::<P_FREQ, U_FREQ>::ViolinEnum { pcore }
            }
            42 => {
                // Cello
                let pcore = Cello::<P_FREQ, U_FREQ>::new(init_values);
                NoteEnum::<P_FREQ, U_FREQ>::CelloEnum { pcore }
            }
            48 => {
                // Timpani
                let pcore = Silence::<P_FREQ, U_FREQ>::new(init_values);
                NoteEnum::<P_FREQ, U_FREQ>::SilenceEnum { pcore }
            }
            52 => {
                //Choir
                let pcore = Choir::<P_FREQ, U_FREQ>::new(init_values);
                NoteEnum::<P_FREQ, U_FREQ>::ChoirEnum { pcore }
            }
            55 => {
                // Orchestra Hit?
                let pcore = Choir::<P_FREQ, U_FREQ>::new(init_values);
                NoteEnum::<P_FREQ, U_FREQ>::ChoirEnum { pcore }
            }
            60 => {
                //French Horn
                let pcore = FrenchHorn::<P_FREQ, U_FREQ>::new(init_values);
                NoteEnum::<P_FREQ, U_FREQ>::FrenchHornEnum { pcore }
            }
            62 => {
                // Brass Section
                let pcore = FrenchHorn::<P_FREQ, U_FREQ>::new(init_values);
                NoteEnum::<P_FREQ, U_FREQ>::FrenchHornEnum { pcore }
            }
            65 => {
                //Sax
                let pcore = Sax::<P_FREQ, U_FREQ>::new(init_values);
                NoteEnum::<P_FREQ, U_FREQ>::SaxEnum { pcore }
            }
            69 => {
                // Oboe
                let pcore = Oboe::<P_FREQ, U_FREQ>::new(init_values);
                NoteEnum::<P_FREQ, U_FREQ>::OboeEnum { pcore }
            }
            _ => {
                assert_eq!(0, instrument);
                let pcore = Silence::<P_FREQ, U_FREQ>::new(init_values);
                NoteEnum::<P_FREQ, U_FREQ>::SilenceEnum { pcore }
            }
        };

        return Self { core };
    }
}
