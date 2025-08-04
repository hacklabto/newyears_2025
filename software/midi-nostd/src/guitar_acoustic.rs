use crate::instrument_low_pass_filters::GenericLowPassCalculator;
use crate::instrument_template_basic::InstrumentTemplateBasic;
use crate::oscillator::OscillatorType;

pub type GuitarAcoustic<const P_FREQ: u32, const U_FREQ: u32> = InstrumentTemplateBasic<
    P_FREQ,
    U_FREQ,
    25,                                      // Oscillator 0 pulse width
    100,                                     // Oscillator 0 volume
    { OscillatorType::PulseWidth as usize }, // Oscillator 0 wave form
    0,                                       // Oscillator 0 note offset
    10,                                      // Oscillator 1 pulse width
    90,                                      // Oscillator 1 volume
    { OscillatorType::PulseWidth as usize }, // Oscillator 1 wave form
    10,                                      // Oscillator 1 note offset
    true,                                    // Sync Oscillator 1 to 0
    0,                                       // A
    1700,                                    // D
    0,                                       // S
    1700,                                    // R
    GenericLowPassCalculator<80, 400>,
>;
