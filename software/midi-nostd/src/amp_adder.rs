use crate::sound_sample::SoundSampleI32;
use crate::sound_source::SoundSource;
use crate::sound_source_id::SoundSourceId;
use crate::sound_source_msgs::SoundSourceMsg;
use crate::sound_source_msgs::SoundSourceMsgs;
use crate::sound_sources::SoundSources;

///
/// Amp Adder
///
pub struct AmpAdder<const PLAY_FREQUENCY: u32, const NUM_CHANNELS: usize> {
    pub channels: [Option<SoundSourceId>; NUM_CHANNELS],
}

impl<const PLAY_FREQUENCY: u32, const NUM_CHANNELS: usize> Default
    for AmpAdder<PLAY_FREQUENCY, NUM_CHANNELS>
{
    fn default() -> Self {
        Self {
            channels: [None; NUM_CHANNELS],
        }
    }
}

#[allow(unused)]
impl<const PLAY_FREQUENCY: u32, const NUM_CHANNELS: usize> AmpAdder<PLAY_FREQUENCY, NUM_CHANNELS> {}

#[allow(unused)]
impl<const PLAY_FREQUENCY: u32, const NUM_CHANNELS: usize> SoundSource<'_, PLAY_FREQUENCY>
    for AmpAdder<PLAY_FREQUENCY, NUM_CHANNELS>
{
    fn get_next(self: &Self, all_sources: &dyn SoundSources<PLAY_FREQUENCY>) -> SoundSampleI32 {
        let mut output: SoundSampleI32 = SoundSampleI32::ZERO;

        for entry in self.channels {
            if let Some(source_id_ref) = &entry {
                let this_source: SoundSampleI32 = all_sources.get_next(source_id_ref);
                output = output + this_source;
            }
        }
        output.clip()
    }

    fn has_next(self: &Self, all_sources: &dyn SoundSources<PLAY_FREQUENCY>) -> bool {
        true
    }

    fn update(&mut self, _new_msgs: &mut SoundSourceMsgs) {}

    fn handle_msg(&mut self, msg: &SoundSourceMsg, new_msgs: &mut SoundSourceMsgs) {}
}

#[cfg(test)]
mod tests {
    use crate::amp_adder::*;
    use crate::sound_source_msgs::SoundSourceMsgs;
    use crate::sound_sources_impl::SoundSourcesImpl;

    #[test]
    fn basic_amp_adder_test() {
        let mut all_pools = SoundSourcesImpl::<24000, 3>::default();

        let amp_adder = AmpAdder::<24000, 2>::default();
        let mut new_msgs = SoundSourceMsgs::default();

        assert_eq!(0x8000 + 0, amp_adder.get_next(&all_pools).to_u16());
        all_pools.update(&mut new_msgs);

        assert_eq!(0x8000 + 0, amp_adder.get_next(&all_pools).to_u16());
        all_pools.update(&mut new_msgs);

        assert_eq!(0x8000 + 0, amp_adder.get_next(&all_pools).to_u16());
        all_pools.update(&mut new_msgs);
    }
}
