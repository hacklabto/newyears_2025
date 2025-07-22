use crate::sound_sample::SoundSample;
use crate::sound_sample::SoundSampleI32;
use crate::sound_source::SoundSource;
use crate::sound_source_id::SoundSourceId;
use crate::sound_source_msgs::SoundSourceAmpMixerInit;
use crate::sound_source_msgs::SoundSourceMsg;
use crate::sound_source_msgs::SoundSourceMsgs;
use crate::sound_source_msgs::SoundSourceValue;
use crate::sound_sources::SoundSources;
use core::marker::PhantomData;

///
/// Amp Mixer
///
pub struct AmpMixer<T: SoundSample, const PLAY_FREQUENCY: u32> {
    source_0: SoundSourceId,
    source_1: SoundSourceId,
    _marker: PhantomData<T>,
}

impl<T: SoundSample, const PLAY_FREQUENCY: u32> Default for AmpMixer<T, PLAY_FREQUENCY> {
    fn default() -> Self {
        let source_0 = SoundSourceId::default();
        let source_1 = SoundSourceId::default();

        Self {
            source_0,
            source_1,
            _marker: PhantomData {},
        }
    }
}

#[allow(unused)]
impl<T: SoundSample, const PLAY_FREQUENCY: u32> AmpMixer<T, PLAY_FREQUENCY> {}

#[allow(unused)]
impl<T: SoundSample, const PLAY_FREQUENCY: u32> SoundSource<'_, T, PLAY_FREQUENCY>
    for AmpMixer<T, PLAY_FREQUENCY>
{
    fn get_next(self: &Self, all_sources: &dyn SoundSources<T, PLAY_FREQUENCY>) -> T {
        let sample_0 = all_sources.get_next(&self.source_0);
        let sample_1 = all_sources.get_next(&self.source_1);

        let sample_0i = (sample_0.to_u16() as i32) - 0x8000;
        let sample_1i = (sample_1.to_u16() as i32) - 0x8000;

        let out_i = ((sample_0i >> 1) * (sample_1i >> 1)) >> 14;
        let out: u16 = (out_i + 0x8000) as u16;

        T::new(out)
    }

    fn has_next(self: &Self, all_sources: &dyn SoundSources<T, PLAY_FREQUENCY>) -> bool {
        all_sources.has_next(&self.source_0) && all_sources.has_next(&self.source_1)
    }

    fn update(&mut self, _new_msgs: &mut SoundSourceMsgs) {}

    fn handle_msg(&mut self, msg: &SoundSourceMsg, new_msgs: &mut SoundSourceMsgs) {
        match &msg.value {
            SoundSourceValue::AmpMixerInit { init_values } => {
                self.source_0 = init_values.source_0;
                self.source_1 = init_values.source_1;

                let creation_msg = SoundSourceMsg::new(
                    msg.src_id.clone(),
                    msg.dest_id.clone(),
                    SoundSourceValue::SoundSourceCreated,
                );
                new_msgs.append(creation_msg);
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
    use crate::adsr::create_adsr;
    use crate::midi_notes::FREQUENCY_MULTIPLIER;
    use crate::oscillator::create_oscillator;
    use crate::sound_sample::SoundScale;
    use crate::sound_source_msgs::OscillatorType;
    use crate::sound_source_msgs::SoundSourceAdsrInit;
    use crate::sound_source_msgs::SoundSourceOscillatorInit;
    use crate::sound_sources_impl::SoundSourcesImpl;

    #[test]
    fn basic_amp_mixer_test() {
        let mut all_pools = SoundSourcesImpl::<SoundSampleI32, 24000, 3, 3, 3>::default();

        let oscillator_id = create_oscillator(
            &mut all_pools,
            SoundSourceOscillatorInit::new(
                OscillatorType::PulseWidth,
                260 * FREQUENCY_MULTIPLIER,
                50,
                50,
            ),
        );

        let adsr_id = create_adsr(
            &mut all_pools,
            SoundSourceAdsrInit::new(
                SoundScale::new_percent(100),
                SoundScale::new_percent(50),
                2,
                4,
                4,
            ),
        );

        let amp_id = create_amp_mixer(
            &mut all_pools,
            SoundSourceAmpMixerInit::new(oscillator_id, adsr_id),
        );

        let mut new_msgs = SoundSourceMsgs::default();

        // Should mirror the ADSR test, about about half volume because I set the oscilator to half
        // volume.

        assert_eq!(0x8000 + 0, all_pools.get_next(&amp_id).to_u16());
        all_pools.update(&mut new_msgs);
        assert_eq!(0x8000 + 0xfff, all_pools.get_next(&amp_id).to_u16());
        all_pools.update(&mut new_msgs);
        assert_eq!(0x8000 + 0x1ffe, all_pools.get_next(&amp_id).to_u16());

        // Delay state, 4 ticks to get to Sustain Volume (50%) from attack volume
        all_pools.update(&mut new_msgs);
        assert_eq!(0x8000 + 0x1bfe, all_pools.get_next(&amp_id).to_u16());
        all_pools.update(&mut new_msgs);
        assert_eq!(0x8000 + 0x17fe, all_pools.get_next(&amp_id).to_u16());
        all_pools.update(&mut new_msgs);
        assert_eq!(0x8000 + 0x13fe, all_pools.get_next(&amp_id).to_u16());
        all_pools.update(&mut new_msgs);
        assert_eq!(0x8000 + 0x0fff, all_pools.get_next(&amp_id).to_u16());

        // Sustain state
        all_pools.update(&mut new_msgs);
        assert_eq!(0x8000 + 0x0fff, all_pools.get_next(&amp_id).to_u16());
        all_pools.update(&mut new_msgs);
        assert_eq!(0x8000 + 0x0fff, all_pools.get_next(&amp_id).to_u16());
        all_pools.update(&mut new_msgs);
        assert_eq!(0x8000 + 0x0fff, all_pools.get_next(&amp_id).to_u16());

        let mut msgs = SoundSourceMsgs::default();
        msgs.append(SoundSourceMsg::new(
            adsr_id.clone(),
            SoundSourceId::get_top_id(),
            SoundSourceValue::ReleaseAdsr,
        ));
        all_pools.process_and_clear_msgs(&mut msgs);

        // Release state, 4 ticks to get to quiet from Sustain Volume
        all_pools.update(&mut new_msgs);
        assert_eq!(0x8000 + 0x0bff, all_pools.get_next(&amp_id).to_u16());
        all_pools.update(&mut new_msgs);
        assert_eq!(0x8000 + 0x07ff, all_pools.get_next(&amp_id).to_u16());
        all_pools.update(&mut new_msgs);
        assert_eq!(0x8000 + 0x03ff, all_pools.get_next(&amp_id).to_u16());
        all_pools.update(&mut new_msgs);
        assert_eq!(0x8000 + 0, all_pools.get_next(&amp_id).to_u16());
        assert_eq!(true, all_pools.has_next(&amp_id));

        // End state.  Report silence and no more data
        all_pools.update(&mut new_msgs);
        assert_eq!(0x8000, all_pools.get_next(&amp_id).to_u16());
        assert_eq!(false, all_pools.has_next(&amp_id));

        all_pools.free(adsr_id);
        all_pools.free(oscillator_id);
        all_pools.free(amp_id);
    }
}
