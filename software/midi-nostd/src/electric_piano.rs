use crate::instrument_template_basic::FrequencyCalculator;
use crate::instrument_template_basic::InstrumentTemplateBasic;
use crate::note::SoundSourceNoteInit;
use crate::oscillator::OscillatorType;

pub struct ElectricPianoLowPassCalculator {}

impl FrequencyCalculator for ElectricPianoLowPassCalculator {
    fn get_cutoff_frequency(init_values: &SoundSourceNoteInit) -> u32 {
        // experimentally derived
        200 + ((init_values.key as u32) * 5) + ((init_values.velocity as u32) / 3)
    }
}

pub type ElectricPiano<const P_FREQ: u32, const U_FREQ: u32> = InstrumentTemplateBasic<
    P_FREQ,
    U_FREQ,
    50,                                      // Oscillator 0 pulse width
    100,                                     // Oscillator 0 volume
    { OscillatorType::SawTooth as usize },   // Oscillator 0 wave form
    0,                                       // Oscillator 0 note offset
    5,                                       // Oscillator 1 pulse width
    100,                                     // Oscillator 1 volume
    { OscillatorType::PulseWidth as usize }, // Oscillator 1 wave form
    33,                                      // Oscillator 1 note offset
    true,                                    // Sync Oscillator 1 to 0
    0,                                       // A
    5140,                                    // D
    50,                                      // S
    660,                                     // R
    ElectricPianoLowPassCalculator,
>;
