use crate::instrument_low_pass_filters::CelloLowPassCalculator;
use crate::instrument_template_amp_lfo::InstrumentTemplateAmpLfo;
use crate::midi_notes::FREQUENCY_MULTIPLIER;
use crate::oscillator::OscillatorType;

pub type Cello<const P_FREQ: u32, const U_FREQ: u32> = InstrumentTemplateAmpLfo<
    P_FREQ,
    U_FREQ,
    10,                                      // Oscillator 0 pulse width
    100,                                     // Oscillator 0 volume
    { OscillatorType::PulseWidth as usize }, // Oscillator 0 wave form
    0,                                       // Oscillator 0 note offset
    50,                                      // Oscillator 1 pulse width
    100,                                     // Oscillator 1 volume
    { OscillatorType::PulseWidth as usize }, // Oscillator 1 wave form
    0,                                       // Oscillator 1 note offset
    true,                                    // Sync Oscillator 1 to 0
    { OscillatorType::Sine as usize },       // LFO wave form
    { 15 * FREQUENCY_MULTIPLIER / 2 },       // LFO frequency (7.5 hz)
    5,                                       // LFO Depth
    6,                                       // A
    5000,                                    // D
    100,                                     // S
    300,                                     // R
    CelloLowPassCalculator,
>;
