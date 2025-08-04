use crate::oscillator::CoreOscillator;
use crate::sound_sample::SoundSampleI32;
use crate::sound_source_core::OscillatorInterface;
use crate::sound_source_core::SoundSourceCore;

pub struct LfoAmplitude<
    const P_FREQ: u32,
    const U_FREQ: u32,
    Source: OscillatorInterface<P_FREQ, U_FREQ>,
    const WAVE: usize,
    const LFO_FREQUENCY: u32,
    const DEPTH: u8,
> {
    source: Source,
    oscillator: CoreOscillator<U_FREQ, U_FREQ, 50, DEPTH, WAVE>,
    amplitude_adjust: SoundSampleI32,
}

impl<
        const P_FREQ: u32,
        const U_FREQ: u32,
        Source: OscillatorInterface<P_FREQ, U_FREQ>,
        const WAVE: usize,
        const LFO_FREQUENCY: u32,
        const DEPTH: u8,
    > LfoAmplitude<P_FREQ, U_FREQ, Source, WAVE, LFO_FREQUENCY, DEPTH>
{
    const AMPLITUDE_OFFSET: SoundSampleI32 =
        SoundSampleI32::new_i32(SoundSampleI32::MAX.to_i32() * (100 - (DEPTH as i32)) / 100);
}

impl<
        const P_FREQ: u32,
        const U_FREQ: u32,
        Source: OscillatorInterface<P_FREQ, U_FREQ>,
        const WAVE: usize,
        const LFO_FREQUENCY: u32,
        const DEPTH: u8,
    > SoundSourceCore<P_FREQ, U_FREQ>
    for LfoAmplitude<P_FREQ, U_FREQ, Source, WAVE, LFO_FREQUENCY, DEPTH>
{
    type InitValuesType = Source::InitValuesType;

    fn new(init_values: Self::InitValuesType) -> Self {
        let source = Source::new(init_values);
        let oscillator = CoreOscillator::<U_FREQ, U_FREQ, 50, DEPTH, WAVE>::new(LFO_FREQUENCY);
        let amplitude_adjust = SoundSampleI32::new_i32(0x8000 - 0x8000 * (DEPTH as i32) / 100);

        Self {
            source,
            oscillator,
            amplitude_adjust,
        }
    }

    fn get_next(self: &mut Self) -> SoundSampleI32 {
        self.source.get_next()
    }

    fn update(self: &mut Self) {
        let lfo_oscillation = self.oscillator.get_next() + Self::AMPLITUDE_OFFSET;
        self.amplitude_adjust = lfo_oscillation;
        self.source.update();
    }

    fn has_next(self: &Self) -> bool {
        return self.source.has_next();
    }

    fn trigger_note_off(self: &mut Self) {
        self.source.trigger_note_off();
    }

    fn restart(self: &mut Self, _vel: u8) {}
}

impl<
        const P_FREQ: u32,
        const U_FREQ: u32,
        Source: OscillatorInterface<P_FREQ, U_FREQ>,
        const WAVE: usize,
        const LFO_FREQUENCY: u32,
        const DEPTH: u8,
    > OscillatorInterface<P_FREQ, U_FREQ>
    for LfoAmplitude<P_FREQ, U_FREQ, Source, WAVE, LFO_FREQUENCY, DEPTH>
{
    fn set_amplitude_adjust(self: &mut Self, adjust: SoundSampleI32) {
        self.source
            .set_amplitude_adjust(adjust * self.amplitude_adjust);
    }
}
