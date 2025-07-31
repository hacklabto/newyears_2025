use crate::oscillator::CoreOscillator;
use crate::sound_sample::SoundSampleI32;
use crate::sound_source_core::SoundSourceCore;

pub struct LfoAmplitude<
    const PLAY_FREQUENCY: u32,
    Source: SoundSourceCore<PLAY_FREQUENCY>,
    const WAVE: usize,
    const LFO_FREQUENCY: u32,
    const DEPTH: u8,
> {
    source: Source,
    oscillator: CoreOscillator<PLAY_FREQUENCY, 50, DEPTH, WAVE>,
}

impl<
        const PLAY_FREQUENCY: u32,
        Source: SoundSourceCore<PLAY_FREQUENCY>,
        const WAVE: usize,
        const LFO_FREQUENCY: u32,
        const DEPTH: u8,
    > LfoAmplitude<PLAY_FREQUENCY, Source, WAVE, LFO_FREQUENCY, DEPTH>
{
    const AMPLITUDE_OFFSET: SoundSampleI32 =
        SoundSampleI32::new_i32(SoundSampleI32::MAX.to_i32() * (100 - (DEPTH as i32)) / 100);
}

impl<
        const PLAY_FREQUENCY: u32,
        Source: SoundSourceCore<PLAY_FREQUENCY>,
        const WAVE: usize,
        const LFO_FREQUENCY: u32,
        const DEPTH: u8,
    > SoundSourceCore<PLAY_FREQUENCY>
    for LfoAmplitude<PLAY_FREQUENCY, Source, WAVE, LFO_FREQUENCY, DEPTH>
{
    type InitValuesType = Source::InitValuesType;

    fn new(init_values: Self::InitValuesType) -> Self {
        let source = Source::new(init_values);
        let oscillator = CoreOscillator::<PLAY_FREQUENCY, 50, DEPTH, WAVE>::new(LFO_FREQUENCY);

        Self { source, oscillator }
    }

    fn get_next(self: &mut Self) -> SoundSampleI32 {
        let lfo_oscillation = self.oscillator.get_next() + Self::AMPLITUDE_OFFSET;
        let sound = self.source.get_next();
        assert!(lfo_oscillation.to_i32() > 0 && lfo_oscillation.to_i32() <= 0x8000);

        return lfo_oscillation * sound;
    }

    fn has_next(self: &Self) -> bool {
        return self.source.has_next();
    }

    fn trigger_note_off(self: &mut Self) {
        self.source.trigger_note_off();
    }
}
