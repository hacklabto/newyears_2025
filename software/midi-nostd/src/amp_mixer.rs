use crate::sound_sample::SoundSampleI32;
use crate::sound_source_core::SoundSourceCore;

pub struct AmpMixerCore<
    const PLAY_FREQUENCY: u32,
    MixSource0: SoundSourceCore<PLAY_FREQUENCY> + Default,
    MixSource1: SoundSourceCore<PLAY_FREQUENCY> + Default,
> {
    source_0: MixSource0,
    source_1: MixSource1,
}

impl<
        const PLAY_FREQUENCY: u32,
        MixSource0: SoundSourceCore<PLAY_FREQUENCY> + Default,
        MixSource1: SoundSourceCore<PLAY_FREQUENCY> + Default,
    > Default for AmpMixerCore<PLAY_FREQUENCY, MixSource0, MixSource1>
{
    fn default() -> Self {
        return Self {
            source_0: MixSource0::default(),
            source_1: MixSource1::default(),
        };
    }
}

impl<
        const PLAY_FREQUENCY: u32,
        MixSource0: SoundSourceCore<PLAY_FREQUENCY> + Default,
        MixSource1: SoundSourceCore<PLAY_FREQUENCY> + Default,
    > SoundSourceCore<PLAY_FREQUENCY> for AmpMixerCore<PLAY_FREQUENCY, MixSource0, MixSource1>
{
    type InitValuesType = (MixSource0::InitValuesType, MixSource1::InitValuesType);

    fn init(self: &mut Self, init_values: &Self::InitValuesType) {
        self.source_0.init(&(init_values.0));
        self.source_1.init(&(init_values.1));
    }

    fn get_next(self: &mut Self) -> SoundSampleI32 {
        self.source_0.get_next() * self.source_1.get_next()
    }

    fn has_next(self: &Self) -> bool {
        self.source_0.has_next() && self.source_1.has_next()
    }

    fn trigger_note_off(self: &mut Self) {
        self.source_0.trigger_note_off();
        self.source_1.trigger_note_off();
    }

    fn reset_oscillator(self: &mut Self) {}
}

#[cfg(test)]
mod tests {
    use crate::adsr::CoreAdsr;
    use crate::adsr::SoundSourceAdsrInit;
    use crate::amp_mixer::*;
    use crate::midi_notes::FREQUENCY_MULTIPLIER;
    use crate::oscillator::CoreOscillator;
    use crate::oscillator::OscillatorType;
    use crate::oscillator::SoundSourceOscillatorInit;

    type OscilatorAdsrCore<const PLAY_FREQUENCY: u32> = AmpMixerCore<
        PLAY_FREQUENCY,
        CoreOscillator<PLAY_FREQUENCY, 50, 50, { OscillatorType::PulseWidth as usize }>,
        CoreAdsr<PLAY_FREQUENCY, 2, 4, 4, 100, 50>,
    >;

    #[test]
    fn basic_amp_mixer_test() {
        let oscilator_init = SoundSourceOscillatorInit::new(FREQUENCY_MULTIPLIER); // 1 hz

        let adsr_init = SoundSourceAdsrInit::new();

        let mut amp_mixer = OscilatorAdsrCore::<1000>::default();
        amp_mixer.source_0.init(&oscilator_init);
        amp_mixer.source_1.init(&adsr_init);

        // Should mirror the ADSR test, about about half volume because I set the oscilator to half
        // volume.

        assert_eq!(0x0000, amp_mixer.get_next().to_i32());
        assert_eq!(0x2000, amp_mixer.get_next().to_i32());
        assert_eq!(0x4000, amp_mixer.get_next().to_i32());

        // Delay state, 4 ticks to get to Sustain Volume (50%) from attack volume
        assert_eq!(0x3800, amp_mixer.get_next().to_i32());
        assert_eq!(0x3000, amp_mixer.get_next().to_i32());
        assert_eq!(0x2800, amp_mixer.get_next().to_i32());
        assert_eq!(0x2000, amp_mixer.get_next().to_i32());

        // Sustain state
        assert_eq!(0x2000, amp_mixer.get_next().to_i32());
        assert_eq!(0x2000, amp_mixer.get_next().to_i32());
        assert_eq!(0x2000, amp_mixer.get_next().to_i32());
        amp_mixer.trigger_note_off();
        // Release doesn't start until update.
        assert_eq!(0x2000, amp_mixer.get_next().to_i32());

        // Release state, 4 ticks to get to quiet from Sustain Volume
        assert_eq!(0x1800, amp_mixer.get_next().to_i32());
        assert_eq!(0x1000, amp_mixer.get_next().to_i32());
        assert_eq!(0x0800, amp_mixer.get_next().to_i32());
        assert_eq!(true, amp_mixer.has_next());
        assert_eq!(0x0000, amp_mixer.get_next().to_i32());

        // End state.  Report silence and no more data
        assert_eq!(false, amp_mixer.has_next());
        assert_eq!(0x0000, amp_mixer.get_next().to_i32());
        assert_eq!(false, amp_mixer.has_next());
    }
}
