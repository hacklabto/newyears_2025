use crate::note::Note;
use crate::sound_sample::SoundSampleI32;
use crate::sound_source_core::SoundSourceCore;

///
/// Amp Adder
///
pub struct AmpAdder<const PLAY_FREQUENCY: u32, const NUM_CHANNELS: usize> {
    pub channels: [Note<PLAY_FREQUENCY>; NUM_CHANNELS],
}

impl<const PLAY_FREQUENCY: u32, const NUM_CHANNELS: usize> Default
    for AmpAdder<PLAY_FREQUENCY, NUM_CHANNELS>
{
    fn default() -> Self {
        Self {
            channels: { core::array::from_fn(|_idx| Note::<PLAY_FREQUENCY>::default()) },
        }
    }
}

impl<const PLAY_FREQUENCY: u32, const NUM_CHANNELS: usize> AmpAdder<PLAY_FREQUENCY, NUM_CHANNELS> {}

impl<const PLAY_FREQUENCY: u32, const NUM_CHANNELS: usize> SoundSourceCore<PLAY_FREQUENCY>
    for AmpAdder<PLAY_FREQUENCY, NUM_CHANNELS>
{
    type InitValuesType = u32;

    fn init(&mut self, _init_values: &Self::InitValuesType) {}

    fn trigger_note_off(self: &mut Self) {}

    fn get_next(self: &mut Self) -> SoundSampleI32 {
        let mut output: SoundSampleI32 = SoundSampleI32::ZERO;

        for entry in &mut self.channels {
            let this_source: SoundSampleI32 = entry.get_next();
            output = output + this_source;
        }
        output.clip()
    }

    fn has_next(self: &Self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use crate::amp_adder::*;

    #[test]
    fn basic_amp_adder_test() {
        let mut amp_adder = AmpAdder::<24000, 2>::default();

        assert_eq!(0, amp_adder.get_next().to_i32());
        assert_eq!(0, amp_adder.get_next().to_i32());
        assert_eq!(0, amp_adder.get_next().to_i32());
    }
}
