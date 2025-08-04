use crate::adsr::CoreAdsr;
use crate::double_oscillator::DoubleOscillator;
use crate::filter::Filter;
use crate::instrument_low_pass_filters::FrequencyCalculator;
use crate::lfo_amplitude::LfoAmplitude;
use crate::midi_notes::midi_note_to_freq;
use crate::note::SoundSourceNoteInit;
use crate::oscillator::CoreOscillator;
use crate::sound_sample::SoundSampleI32;
use crate::sound_source_core::SoundSourceCore;
use core::marker::PhantomData;

pub struct InstrumentTemplateAmpLfo<
    const P_FREQ: u32,
    const U_FREQ: u32,
    const OSC_0_PULSE_WIDTH: u8,
    const OSC_0_VOLUME: u8,
    const OSC_0_WAVE_FORM: usize,
    const OSC_0_TUNE: u8,
    const OSC_1_PULSE_WIDTH: u8,
    const OSC_1_VOLUME: u8,
    const OSC_1_WAVE_FORM: usize,
    const OSC_1_TUNE: u8,
    const OSC_1_SYNC_TO_0: bool,
    const LFO_OSC_WAVE_FORM: usize,
    const LFO_OSC_FREQ: u32,
    const LFO_OSC_DEPTH: u8,
    const A: i32,
    const D: i32,
    const S: u8,
    const R: i32,
    CutoffFrequencyCalculator: FrequencyCalculator,
> {
    core: CoreAdsr<
        P_FREQ,
        U_FREQ,
        A,
        D,
        S,
        R,
        Filter<
            P_FREQ,
            U_FREQ,
            LfoAmplitude<
                P_FREQ,
                U_FREQ,
                DoubleOscillator<
                    P_FREQ,
                    U_FREQ,
                    CoreOscillator<
                        P_FREQ,
                        U_FREQ,
                        OSC_0_PULSE_WIDTH,
                        OSC_0_VOLUME,
                        OSC_0_WAVE_FORM,
                    >,
                    CoreOscillator<
                        P_FREQ,
                        U_FREQ,
                        OSC_1_PULSE_WIDTH,
                        OSC_1_VOLUME,
                        OSC_1_WAVE_FORM,
                    >,
                    OSC_1_SYNC_TO_0,
                >,
                LFO_OSC_WAVE_FORM,
                LFO_OSC_FREQ,
                LFO_OSC_DEPTH,
            >,
        >,
    >,
    _marker: PhantomData<CutoffFrequencyCalculator>,
}

impl<
        const P_FREQ: u32,
        const U_FREQ: u32,
        const OSC_0_PULSE_WIDTH: u8,
        const OSC_0_VOLUME: u8,
        const OSC_0_WAVE_FORM: usize,
        const OSC_0_TUNE: u8,
        const OSC_1_PULSE_WIDTH: u8,
        const OSC_1_VOLUME: u8,
        const OSC_1_WAVE_FORM: usize,
        const OSC_1_TUNE: u8,
        const OSC_1_SYNC_TO_0: bool,
        const LFO_OSC_WAVE_FORM: usize,
        const LFO_OSC_FREQ: u32,
        const LFO_OSC_DEPTH: u8,
        const A: i32,
        const D: i32,
        const S: u8,
        const R: i32,
        CutoffFrequencyCalculator: FrequencyCalculator,
    > SoundSourceCore<P_FREQ, U_FREQ>
    for InstrumentTemplateAmpLfo<
        P_FREQ,
        U_FREQ,
        OSC_0_PULSE_WIDTH,
        OSC_0_VOLUME,
        OSC_0_WAVE_FORM,
        OSC_0_TUNE,
        OSC_1_PULSE_WIDTH,
        OSC_1_VOLUME,
        OSC_1_WAVE_FORM,
        OSC_1_TUNE,
        OSC_1_SYNC_TO_0,
        LFO_OSC_WAVE_FORM,
        LFO_OSC_FREQ,
        LFO_OSC_DEPTH,
        A,
        D,
        S,
        R,
        CutoffFrequencyCalculator,
    >
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

    fn trigger_note_off(self: &mut Self) {
        self.core.trigger_note_off();
    }

    fn restart(self: &mut Self, vel: u8) {
        self.core.restart(vel);
    }

    fn new(init_values: Self::InitValuesType) -> Self {
        let frequency_1 = midi_note_to_freq(init_values.key + OSC_0_TUNE);
        let frequency_2 = midi_note_to_freq(init_values.key + OSC_1_TUNE);
        let cutoff_frequency = CutoffFrequencyCalculator::get_cutoff_frequency(&init_values);
        let adsr_init = (init_values.velocity as i32) << 8;
        let core = CoreAdsr::new((((frequency_1, frequency_2), cutoff_frequency), adsr_init));
        Self {
            core,
            _marker: PhantomData,
        }
    }
}
