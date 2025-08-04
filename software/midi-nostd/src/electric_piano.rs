use crate::instrument_low_pass_filters::PianoLowPassCalculator;
use crate::instrument_template_basic::InstrumentTemplateBasic;
use crate::oscillator::OscillatorType;

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
    PianoLowPassCalculator,
>;
