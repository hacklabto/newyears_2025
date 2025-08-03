use crate::adsr::CoreAdsr;
use crate::double_oscillator::DoubleOscillator;
use crate::filter::Filter;
use crate::midi_notes::midi_note_to_freq;
use crate::note::SoundSourceNoteInit;
use crate::oscillator::CoreOscillator;
use crate::oscillator::OscillatorType;
use crate::sound_sample::SoundSampleI32;
use crate::sound_source_core::SoundSourceCore;

type ElectricPianoOscillatorPair<const P_FREQ: u32, const U_FREQ: u32> = DoubleOscillator<
    P_FREQ,
    U_FREQ,
    CoreOscillator<P_FREQ, U_FREQ, 50, 100, { OscillatorType::SawTooth as usize }>,
    CoreOscillator<P_FREQ, U_FREQ, 5, 100, { OscillatorType::PulseWidth as usize }>,
    true,
>;

type ElectricPianoOscillatorAdsr<const P_FREQ: u32, const U_FREQ: u32> =
    CoreAdsr<P_FREQ, U_FREQ, 0, 5140, 50, 660, ElectricPianoOscillatorPair<P_FREQ, U_FREQ>>;

type ElectricPianoFiltered<const P_FREQ: u32, const U_FREQ: u32> =
    Filter<P_FREQ, U_FREQ, ElectricPianoOscillatorAdsr<P_FREQ, U_FREQ>>;

pub struct ElectricPiano<const P_FREQ: u32, const U_FREQ: u32> {
    core: ElectricPianoFiltered<P_FREQ, U_FREQ>,
}

impl<const P_FREQ: u32, const U_FREQ: u32> SoundSourceCore<P_FREQ, U_FREQ>
    for ElectricPiano<P_FREQ, U_FREQ>
{
    type InitValuesType = SoundSourceNoteInit;

    fn get_next(self: &mut Self) -> SoundSampleI32 {
        self.core.get_next()
    }

    fn update(self: &mut Self) {
        self.core.update()
    }

    fn has_next(self: &Self) -> bool {
        self.core.has_next()
    }

    fn new(init_values: Self::InitValuesType) -> Self {
        let frequency_1 = midi_note_to_freq(init_values.key);
        let frequency_2 = midi_note_to_freq(init_values.key + 24 + 9);
        let cutoff_frequency =
            200 + ((init_values.key as u32) * 4) + ((init_values.velocity as u32) / 2);

        let adsr_init = (init_values.velocity as i32) << 8;
        let core = ElectricPianoFiltered::<P_FREQ, U_FREQ>::new((
            ((frequency_1, frequency_2), adsr_init),
            cutoff_frequency,
        ));
        return Self { core };
    }

    fn trigger_note_off(self: &mut Self) {
        self.core.trigger_note_off();
    }

    fn restart(self: &mut Self, vel: u8) {
        self.core.restart(vel);
    }
}
