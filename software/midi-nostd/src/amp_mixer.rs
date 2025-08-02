use crate::sound_sample::SoundSampleI32;
use crate::sound_source_core::SoundSourceCore;

pub struct AmpMixerCore<
    const P_FREQ: u32,
    const U_FREQ: u32,
    MixSource0: SoundSourceCore<P_FREQ, U_FREQ>,
    MixSource1: SoundSourceCore<P_FREQ, U_FREQ>,
> {
    source_0: MixSource0,
    source_1: MixSource1,
}

impl<
        const P_FREQ: u32,
        const U_FREQ: u32,
        MixSource0: SoundSourceCore<P_FREQ, U_FREQ>,
        MixSource1: SoundSourceCore<P_FREQ, U_FREQ>,
    > SoundSourceCore<P_FREQ, U_FREQ> for AmpMixerCore<P_FREQ, U_FREQ, MixSource0, MixSource1>
{
    type InitValuesType = (MixSource0::InitValuesType, MixSource1::InitValuesType);

    fn new(init_values: Self::InitValuesType) -> Self {
        let source_0 = MixSource0::new(init_values.0);
        let source_1 = MixSource1::new(init_values.1);
        Self { source_0, source_1 }
    }

    #[inline]
    fn get_next(self: &mut Self) -> SoundSampleI32 {
        self.source_0.get_next() * self.source_1.get_next()
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

// Todo, am I even keeping this class?
#[cfg(test)]
mod tests {
    //use crate::midi_notes::FREQUENCY_MULTIPLIER;
    /*
    use crate::amp_mixer::*;
    use crate::oscillator::CoreOscillator;
    use crate::oscillator::OscillatorType;
    */

    /*
    type OscilatorAdsrCore<const P_FREQ: u32, const U_FREQ: u32> = AmpMixerCore<
        P_FREQ,
        U_FREQ,
        CoreOscillator<P_FREQ, U_FREQ, 50, 50, { OscillatorType::PulseWidth as usize }>,
        CoreOscillator<P_FREQ, U_FREQ, 50, 50, { OscillatorType::PulseWidth as usize }>,
    >;
    */

    /*
    #[test]
    fn basic_amp_mixer_test() {
        let frequency: u32 = 1 * FREQUENCY_MULTIPLIER; // 1 hz

        let adsr_init: i32 = 0x8000;

        let mut amp_mixer = OscilatorAdsrCore::<1000, 1000>::new((frequency, adsr_init));

        // Should mirror the ADSR test, about about half volume because I set the oscilator to half
        // volume.

        amp_mixer.update();
        assert_eq!(0x0000, amp_mixer.get_next().to_i32());
    }
    */
}
