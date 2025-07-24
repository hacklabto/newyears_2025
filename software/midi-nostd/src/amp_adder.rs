use crate::sound_sample::SoundSample;
use crate::sound_source::SoundSource;
use crate::sound_source_id::SoundSourceId;
use crate::sound_source_msgs::SoundSourceMsg;
use crate::sound_source_msgs::SoundSourceMsgs;
use crate::sound_sources::SoundSources;
use core::marker::PhantomData;

///
/// Amp Adder
///
pub struct AmpAdder<T: SoundSample, const PLAY_FREQUENCY: u32, const NUM_CHANNELS: usize> {
    pub channels: [Option<SoundSourceId>; NUM_CHANNELS],
    _marker: PhantomData<T>,
}

impl<T: SoundSample, const PLAY_FREQUENCY: u32, const NUM_CHANNELS: usize> Default
    for AmpAdder<T, PLAY_FREQUENCY, NUM_CHANNELS>
{
    fn default() -> Self {
        Self {
            channels: [None; NUM_CHANNELS],
            _marker: PhantomData {},
        }
    }
}

#[allow(unused)]
impl<T: SoundSample, const PLAY_FREQUENCY: u32, const NUM_CHANNELS: usize>
    AmpAdder<T, PLAY_FREQUENCY, NUM_CHANNELS>
{
}

#[allow(unused)]
impl<T: SoundSample, const PLAY_FREQUENCY: u32, const NUM_CHANNELS: usize>
    SoundSource<'_, T, PLAY_FREQUENCY> for AmpAdder<T, PLAY_FREQUENCY, NUM_CHANNELS>
{
    fn get_next(self: &Self, all_sources: &dyn SoundSources<T, PLAY_FREQUENCY>) -> T {
        let mut output: T = T::default();

        for entry in self.channels {
            if let Some(source_id_ref) = &entry {
                let this_source: T = all_sources.get_next(source_id_ref);
                output = output + this_source;
            }
        }
        output.clip()
    }

    fn has_next(self: &Self, all_sources: &dyn SoundSources<T, PLAY_FREQUENCY>) -> bool {
        true
    }

    fn update(&mut self, _new_msgs: &mut SoundSourceMsgs) {}

    fn handle_msg(&mut self, msg: &SoundSourceMsg, new_msgs: &mut SoundSourceMsgs) {}
}

#[cfg(test)]
mod tests {
    use crate::amp_adder::*;
    use crate::sound_sample::SoundSampleI32;
    use crate::sound_source_msgs::SoundSourceMsgs;
    use crate::sound_sources_impl::SoundSourcesImpl;

    #[test]
    fn basic_amp_adder_test() {
        let mut all_pools = SoundSourcesImpl::<SoundSampleI32, 24000, 3, 3>::default();

        let amp_adder = AmpAdder::<SoundSampleI32, 24000, 2>::default();
        let mut new_msgs = SoundSourceMsgs::default();

        assert_eq!(0x8000 + 0, amp_adder.get_next(&all_pools).to_u16());
        all_pools.update(&mut new_msgs);

        assert_eq!(0x8000 + 0, amp_adder.get_next(&all_pools).to_u16());
        all_pools.update(&mut new_msgs);

        assert_eq!(0x8000 + 0, amp_adder.get_next(&all_pools).to_u16());
        all_pools.update(&mut new_msgs);
    }
}
