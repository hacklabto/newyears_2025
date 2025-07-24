use crate::adsr::CoreAdsr;
use crate::oscillator::CoreOscillator;
use crate::sound_sample::SoundSample;
use crate::sound_sample::SoundSampleI32;
use crate::sound_source::SoundSource;
use crate::sound_source_core::SoundSourceCore;
use crate::sound_source_id::SoundSourceId;
use crate::sound_source_msgs::SoundSourceAmpMixerInit;
use crate::sound_source_msgs::SoundSourceMsg;
use crate::sound_source_msgs::SoundSourceMsgs;
use crate::sound_source_msgs::SoundSourceValue;
use crate::sound_sources::SoundSources;
use core::marker::PhantomData;

pub struct AmpMixerCore<
    'a,
    T: SoundSample,
    const PLAY_FREQUENCY: u32,
    MixSource0: SoundSourceCore<'a, T, PLAY_FREQUENCY> + Default,
    MixSource1: SoundSourceCore<'a, T, PLAY_FREQUENCY> + Default,
> {
    pub source_0: MixSource0,
    pub source_1: MixSource1,
    _marker: PhantomData<T>,
    _lifetime_marker: PhantomData<&'a ()>,
}

impl<
        'a,
        T: SoundSample,
        const PLAY_FREQUENCY: u32,
        MixSource0: SoundSourceCore<'a, T, PLAY_FREQUENCY> + Default,
        MixSource1: SoundSourceCore<'a, T, PLAY_FREQUENCY> + Default,
    > Default for AmpMixerCore<'a, T, PLAY_FREQUENCY, MixSource0, MixSource1>
{
    fn default() -> Self {
        return Self {
            source_0: MixSource0::default(),
            source_1: MixSource1::default(),
            _marker: PhantomData {},
            _lifetime_marker: PhantomData {},
        };
    }
}

impl<
        'a,
        T: SoundSample,
        const PLAY_FREQUENCY: u32,
        MixSource0: SoundSourceCore<'a, T, PLAY_FREQUENCY> + Default,
        MixSource1: SoundSourceCore<'a, T, PLAY_FREQUENCY> + Default,
    > SoundSourceCore<'a, T, PLAY_FREQUENCY>
    for AmpMixerCore<'a, T, PLAY_FREQUENCY, MixSource0, MixSource1>
{
    fn get_next(self: &Self) -> T {
        let sample_0 = self.source_0.get_next();
        let sample_1 = self.source_1.get_next();

        let sample_0i = (sample_0.to_u16() as i32) - 0x8000;
        let sample_1i = (sample_1.to_u16() as i32) - 0x8000;

        let out_i = ((sample_0i >> 1) * (sample_1i >> 1)) >> 14;
        let out: u16 = (out_i + 0x8000) as u16;

        T::new(out)
    }

    fn has_next(self: &Self) -> bool {
        self.source_0.has_next() && self.source_1.has_next()
    }

    fn update(&mut self) {
        self.source_0.update();
        self.source_1.update();
    }
}

type OscilatorAdsrCore<'a, T, const PLAY_FREQUENCY: u32> = AmpMixerCore<
    'a,
    T,
    PLAY_FREQUENCY,
    CoreOscillator<T, PLAY_FREQUENCY>,
    CoreAdsr<T, PLAY_FREQUENCY>,
>;

///
/// Amp Mixer.  Now sort of a proof of concept.
///
pub struct AmpMixer<'a, T: SoundSample, const PLAY_FREQUENCY: u32> {
    core: OscilatorAdsrCore<'a, T, PLAY_FREQUENCY>,
}

impl<T: SoundSample, const PLAY_FREQUENCY: u32> Default for AmpMixer<'_, T, PLAY_FREQUENCY> {
    fn default() -> Self {
        Self {
            core: OscilatorAdsrCore::<T, PLAY_FREQUENCY>::default(),
        }
    }
}

//impl<T: SoundSample, const PLAY_FREQUENCY: u32> AmpMixer<'_, T, PLAY_FREQUENCY> {}

impl<T: SoundSample, const PLAY_FREQUENCY: u32> SoundSource<'_, T, PLAY_FREQUENCY>
    for AmpMixer<'_, T, PLAY_FREQUENCY>
{
    fn get_next(self: &Self, _all_sources: &dyn SoundSources<T, PLAY_FREQUENCY>) -> T {
        self.core.get_next()
    }

    fn has_next(self: &Self, _all_sources: &dyn SoundSources<T, PLAY_FREQUENCY>) -> bool {
        self.core.has_next()
    }

    fn update(&mut self, _new_msgs: &mut SoundSourceMsgs) {
        self.core.update()
    }

    fn handle_msg(&mut self, msg: &SoundSourceMsg, new_msgs: &mut SoundSourceMsgs) {
        match &msg.value {
            SoundSourceValue::AmpMixerInit { init_values } => {
                self.core.source_0.init(&init_values.oscilator_init);
                self.core.source_1.init(&init_values.adsr_init);

                let creation_msg = SoundSourceMsg::new(
                    msg.src_id.clone(),
                    msg.dest_id.clone(),
                    SoundSourceValue::SoundSourceCreated,
                );
                new_msgs.append(creation_msg);
            }
            SoundSourceValue::ReleaseAdsr => {
                // TODO, What if we aren't in sustain?  Probably I should take
                // the current volume and run the release on that.
                self.core.source_1.trigger_release();
            }
            _ => todo!(),
        }
    }
}

pub fn create_amp_mixer(
    all_pools: &mut dyn SoundSources<SoundSampleI32, 24000>,
    init_values: SoundSourceAmpMixerInit,
) -> SoundSourceId {
    let mut msgs = SoundSourceMsgs::default();
    msgs.append(SoundSourceMsg::new(
        SoundSourceId::get_top_id(),
        SoundSourceId::get_top_id(),
        SoundSourceValue::AmpMixerInit { init_values },
    ));
    all_pools.process_and_clear_msgs(&mut msgs);

    all_pools
        .get_last_created_sound_source()
        .expect("Id should have been recorded")
        .clone()
}

#[cfg(test)]
mod tests {
    use crate::amp_mixer::*;
    //use crate::sound_sources::SoundSources;
    use crate::adsr::SoundSourceAdsrInit;
    use crate::midi_notes::FREQUENCY_MULTIPLIER;
    use crate::oscillator::OscillatorType;
    use crate::oscillator::SoundSourceOscillatorInit;
    use crate::sound_sample::SoundScale;

    #[test]
    fn basic_amp_mixer_test() {
        let oscilator_init = SoundSourceOscillatorInit::new(
            OscillatorType::PulseWidth,
            260 * FREQUENCY_MULTIPLIER,
            50,
            50,
        );

        let adsr_init = SoundSourceAdsrInit::new(
            SoundScale::new_percent(100),
            SoundScale::new_percent(50),
            2,
            4,
            4,
        );

        let mut amp_mixer = OscilatorAdsrCore::<'_, SoundSampleI32, 240000>::default();
        amp_mixer.source_0.init(&oscilator_init);
        amp_mixer.source_1.init(&adsr_init);

        // Should mirror the ADSR test, about about half volume because I set the oscilator to half
        // volume.

        assert_eq!(0x8000 + 0, amp_mixer.get_next().to_u16());
        amp_mixer.update();
        assert_eq!(0x8000 + 0xfff, amp_mixer.get_next().to_u16());
        amp_mixer.update();
        assert_eq!(0x8000 + 0x1ffe, amp_mixer.get_next().to_u16());

        // Delay state, 4 ticks to get to Sustain Volume (50%) from attack volume
        amp_mixer.update();
        assert_eq!(0x8000 + 0x1bfe, amp_mixer.get_next().to_u16());
        amp_mixer.update();
        assert_eq!(0x8000 + 0x17fe, amp_mixer.get_next().to_u16());
        amp_mixer.update();
        assert_eq!(0x8000 + 0x13fe, amp_mixer.get_next().to_u16());
        amp_mixer.update();
        assert_eq!(0x8000 + 0x0fff, amp_mixer.get_next().to_u16());

        // Sustain state
        amp_mixer.update();
        assert_eq!(0x8000 + 0x0fff, amp_mixer.get_next().to_u16());
        amp_mixer.update();
        assert_eq!(0x8000 + 0x0fff, amp_mixer.get_next().to_u16());
        amp_mixer.update();
        assert_eq!(0x8000 + 0x0fff, amp_mixer.get_next().to_u16());

        amp_mixer.source_1.trigger_release();

        // Release state, 4 ticks to get to quiet from Sustain Volume
        amp_mixer.update();
        assert_eq!(0x8000 + 0x0bff, amp_mixer.get_next().to_u16());
        amp_mixer.update();
        assert_eq!(0x8000 + 0x07ff, amp_mixer.get_next().to_u16());
        amp_mixer.update();
        assert_eq!(0x8000 + 0x03ff, amp_mixer.get_next().to_u16());
        amp_mixer.update();
        assert_eq!(0x8000 + 0, amp_mixer.get_next().to_u16());
        assert_eq!(true, amp_mixer.has_next());

        // End state.  Report silence and no more data
        amp_mixer.update();
        assert_eq!(0x8000, amp_mixer.get_next().to_u16());
        assert_eq!(false, amp_mixer.has_next());
    }
}
