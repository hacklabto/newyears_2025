use crate::instrument_template_basic::FrequencyCalculator;
use crate::instrument_template_basic::InstrumentTemplateBasic;
use crate::note::SoundSourceNoteInit;
use crate::oscillator::OscillatorType;

pub struct PianoLowPassCalculator {}

impl FrequencyCalculator for PianoLowPassCalculator {
    fn get_cutoff_frequency(init_values: &SoundSourceNoteInit) -> u32 {
        // experimentally derived
        200 + ((init_values.key as u32) * 5) + ((init_values.velocity as u32) / 3)
    }
}

pub type Piano<const P_FREQ: u32, const U_FREQ: u32> = InstrumentTemplateBasic<
    P_FREQ,
    U_FREQ,
    50,                                      // Oscillator 0 pulse width
    75,                                      // Oscillator 0 volume
    { OscillatorType::SawTooth as usize },   // Oscillator 0 wave form
    0,                                       // Oscillator 0 note offset
    15,                                      // Oscillator 1 pulse width
    75,                                      // Oscillator 1 volume
    { OscillatorType::PulseWidth as usize }, // Oscillator 1 wave form
    14,                                      // Oscillator 1 note offset
    true,                                    // Sync Oscillator 1 to 0
    0,                                       // A
    670,                                     // D
    25,                                      // S
    300,                                     // R
    PianoLowPassCalculator,
>;
