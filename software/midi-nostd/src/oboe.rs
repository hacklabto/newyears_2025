use crate::instrument_low_pass_filters::GenericLowPassCalculator;
use crate::instrument_template_amp_lfo::InstrumentTemplateAmpLfo;
use crate::midi_notes::FREQUENCY_MULTIPLIER;
use crate::oscillator::OscillatorType;

pub type Oboe<const P_FREQ: u32, const U_FREQ: u32> = InstrumentTemplateAmpLfo<
    P_FREQ,
    U_FREQ,
    20,                                      // Oscillator 0 pulse width
    80,                                      // Oscillator 0 volume
    { OscillatorType::PulseWidth as usize }, // Oscillator 0 wave form
    0,                                       // Oscillator 0 note offset
    5,                                       // Oscillator 1 pulse width
    80,                                      // Oscillator 1 volume
    { OscillatorType::PulseWidth as usize }, // Oscillator 1 wave form
    0,                                       // Oscillator 1 note offset
    false,                                   // Sync Oscillator 1 to 0
    { OscillatorType::Sine as usize },       // LFO wave form
    { 15 * FREQUENCY_MULTIPLIER / 2 },       // LFO frequency (7.5 hz)
    5,                                       // LFO Depth
    20,                                      // A
    5000,                                    // D
    100,                                     // S
    140,                                     // R
    GenericLowPassCalculator<70, 100>,
>;
