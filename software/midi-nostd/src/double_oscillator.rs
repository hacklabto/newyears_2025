use crate::sound_sample::SoundSampleI32;
use crate::sound_source_core::SoundSourceCore;

pub struct DoubleOscillator<
    const P_FREQ: u32,
    const U_FREQ: u32,
    MixSource0: SoundSourceCore<P_FREQ, U_FREQ>,
    MixSource1: SoundSourceCore<P_FREQ, U_FREQ>,
    const SYNC_1_FROM_0: bool,
> {
    last_source_0_sample: SoundSampleI32,
    source_0: MixSource0,
    source_1: MixSource1,
}

impl<
        const P_FREQ: u32,
        const U_FREQ: u32,
        MixSource0: SoundSourceCore<P_FREQ, U_FREQ>,
        MixSource1: SoundSourceCore<P_FREQ, U_FREQ>,
        const SYNC_1_FROM_0: bool,
    > SoundSourceCore<P_FREQ, U_FREQ>
    for DoubleOscillator<P_FREQ, U_FREQ, MixSource0, MixSource1, SYNC_1_FROM_0>
{
    type InitValuesType = (MixSource0::InitValuesType, MixSource1::InitValuesType);

    fn new(init_values: Self::InitValuesType) -> Self {
        let last_source_0_sample = SoundSampleI32::ZERO;
        let source_0 = MixSource0::new(init_values.0);
        let source_1 = MixSource1::new(init_values.1);
        Self {
            source_0,
            source_1,
            last_source_0_sample,
        }
    }

    fn get_next(self: &mut Self) -> SoundSampleI32 {
        let s0 = self.source_0.get_next();
        if SYNC_1_FROM_0
            && s0 >= SoundSampleI32::ZERO
            && self.last_source_0_sample < SoundSampleI32::ZERO
        {
            self.source_1.reset_oscillator();
        }
        self.last_source_0_sample = s0;
        let s1 = self.source_1.get_next();
        s0 + s1
    }

    fn update(self: &mut Self) {
        self.source_0.update();
        self.source_1.update();
    }

    fn has_next(self: &Self) -> bool {
        self.source_0.has_next() && self.source_1.has_next()
    }

    fn trigger_note_off(self: &mut Self) {
        self.source_0.trigger_note_off();
        self.source_1.trigger_note_off();
    }
}

/*
#[cfg(test)]
mod tests {
    use crate::adsr::CoreAdsr;
    use crate::adsr::SoundSourceAdsrInit;
    use crate::amp_mixer::*;
    use crate::midi_notes::FREQUENCY_MULTIPLIER;
    use crate::oscillator::CoreOscillator;
    use crate::oscillator::OscillatorType;
    use crate::oscillator::SoundSourceOscillatorInit;

    type OscilatorAdsrCore<const P_FREQ: u32, const U_FREQ: u32> = DoubleOscillator<
        P_FREQ, U_FREQ,
        CoreOscillator<P_FREQ, U_FREQ, 50, 50, { OscillatorType::PulseWidth as usize }>,
        CoreAdsr<P_FREQ, U_FREQ, 2, 4, 4, 100, 50>,
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
*/
