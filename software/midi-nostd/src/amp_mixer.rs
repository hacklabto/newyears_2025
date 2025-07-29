use crate::sound_sample::SoundSampleI32;
use crate::sound_source_core::SoundSourceCore;

pub struct AmpMixerCore<
    const PLAY_FREQUENCY: u32,
    MixSource0: SoundSourceCore<PLAY_FREQUENCY>,
    MixSource1: SoundSourceCore<PLAY_FREQUENCY>,
> {
    source_0: MixSource0,
    source_1: MixSource1,
}

impl<
        const PLAY_FREQUENCY: u32,
        MixSource0: SoundSourceCore<PLAY_FREQUENCY>,
        MixSource1: SoundSourceCore<PLAY_FREQUENCY>,
    > SoundSourceCore<PLAY_FREQUENCY> for AmpMixerCore<PLAY_FREQUENCY, MixSource0, MixSource1>
{
    type InitValuesType = (MixSource0::InitValuesType, MixSource1::InitValuesType);

    fn new(init_values: &Self::InitValuesType) -> Self {
        let source_0 = MixSource0::new(&(init_values.0));
        let source_1 = MixSource1::new(&(init_values.1));
        Self { source_0, source_1 }
    }

    #[inline]
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
}

#[cfg(test)]
mod tests {
    use crate::adsr::CoreAdsr;
    use crate::amp_mixer::*;
    use crate::midi_notes::FREQUENCY_MULTIPLIER;
    use crate::oscillator::CoreOscillator;
    use crate::oscillator::OscillatorType;
    use crate::oscillator::SoundSourceOscillatorInit;

    type OscilatorAdsrCore<const PLAY_FREQUENCY: u32> = AmpMixerCore<
        PLAY_FREQUENCY,
        CoreOscillator<PLAY_FREQUENCY, 50, 50, { OscillatorType::PulseWidth as usize }>,
        CoreAdsr<PLAY_FREQUENCY, 2, 4, 50, 4>,
    >;

    #[test]
    fn basic_amp_mixer_test() {
        let oscillator_init = SoundSourceOscillatorInit::new(FREQUENCY_MULTIPLIER); // 1 hz

        let adsr_init: i32 = 0x8000;

        let mut amp_mixer = OscilatorAdsrCore::<1000>::new(&(oscillator_init, adsr_init));

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
