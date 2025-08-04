use crate::instrument_low_pass_filters::GenericLowPassCalculator;
use crate::instrument_template_amp_lfo::InstrumentTemplateAmpLfo;
use crate::midi_notes::FREQUENCY_MULTIPLIER;
use crate::oscillator::OscillatorType;

pub type Choir<const P_FREQ: u32, const U_FREQ: u32> = InstrumentTemplateAmpLfo<
    P_FREQ,
    U_FREQ,
    15,                                      // Oscillator 0 pulse width
    100,                                     // Oscillator 0 volume
    { OscillatorType::PulseWidth as usize }, // Oscillator 0 wave form
    0,                                       // Oscillator 0 note offset
    25,                                      // Oscillator 1 pulse width
    50,                                      // Oscillator 1 volume
    { OscillatorType::PulseWidth as usize }, // Oscillator 1 wave form
    -12,                                     // Oscillator 1 note offset
    false,                                   // Sync Oscillator 1 to 0
    { OscillatorType::Triangle as usize },   // LFO wave form
    { 24 * FREQUENCY_MULTIPLIER / 10 },      // LFO frequency (12hz)
    10,                                      // LFO depth
    300,                                     // A
    5000,                                    // D
    100,                                     // S
    930,                                     // R
    GenericLowPassCalculator<90, 400>,
>;
