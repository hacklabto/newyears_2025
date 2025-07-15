use crate::sound_sample::SoundSample;
//use crate::sound_sample::SoundSampleI32;
use crate::sound_source::SoundSource;
//use crate::sound_source_id::SoundSourceId;
//use crate::sound_source_msgs::SoundSourceAmpMixerInit;
//use crate::sound_source_msgs::SoundSourceKey;
use crate::sound_source_msgs::SoundSourceMsg;
use crate::sound_source_msgs::SoundSourceMsgs;
//use crate::sound_source_msgs::SoundSourceValue;
use crate::sound_sources::SoundSources;
use midly::Smf;
use core::marker::PhantomData;

///
/// Midi Playback
//
pub struct Midi<'a, T: SoundSample, const PLAY_FREQUENCY: u32> {
    smf: Option<Smf<'a>>, 
    _marker: PhantomData<T>,
}

impl<T: SoundSample, const PLAY_FREQUENCY: u32> Default for Midi<'_, T, PLAY_FREQUENCY> {
    fn default() -> Self {
        Self {
            smf: None,
            _marker: PhantomData {},
        }
    }
}

impl<T: SoundSample, const PLAY_FREQUENCY: u32> Midi<'_, T, PLAY_FREQUENCY> {
    pub fn new(midi_bytes: &'static [u8]) -> Self {
        let smf = Some(midly::Smf::parse( midi_bytes ).expect("It's inlined data, so it better work, gosh darn it"));
        Self{
            smf,
            _marker: PhantomData {},
        }
    }
}   

#[allow(unused)]
impl<T: SoundSample, const PLAY_FREQUENCY: u32> SoundSource<T, PLAY_FREQUENCY>
    for Midi<'static, T, PLAY_FREQUENCY>
{
    fn get_next(self: &Self, all_sources: &dyn SoundSources<T, PLAY_FREQUENCY>) -> T {
        T::max()
    }

    fn has_next(self: &Self, all_sources: &dyn SoundSources<T, PLAY_FREQUENCY>) -> bool {
        true
    }

    fn update(&mut self, _new_msgs: &mut SoundSourceMsgs) {}

    fn handle_msg(&mut self, msg: &SoundSourceMsg, new_msgs: &mut SoundSourceMsgs) {
    }
}

/*
 * This is kind of top level, does it make sense to create via message?
 */

/*
pub fn create_amp_mixer(
    all_pools: &mut dyn SoundSources<SoundSampleI32, 24000>,
    amp_mixer_properties: SoundSourceAmpMixerInit,
) -> SoundSourceId {
    let mut msgs = SoundSourceMsgs::default();
    msgs.append(SoundSourceMsg::new(
        all_pools.get_top_id(),
        all_pools.get_top_id(),
        SoundSourceKey::InitAmpMixer,
        SoundSourceValue::new_amp_mixer_init(amp_mixer_properties),
    ));
    all_pools.process_and_clear_msgs(&mut msgs);

    all_pools
        .get_last_created_sound_source()
        .expect("Id should have been recorded")
        .clone()
}
*/

#[cfg(test)]
mod tests {
    use crate::sound_sources_impl::SoundSourcesImpl;
    use crate::sound_sample::SoundSampleI32;
    use crate::midi::Midi;

    #[test]
    fn basic_midi_test() {
        let mut _all_pools = SoundSourcesImpl::<SoundSampleI32, 24000, 3, 3, 3>::default();

        let mut _test_idi = Midi::<SoundSampleI32, 24000>::new( include_bytes!("../assets/twinkle.mid") );
    }
}

