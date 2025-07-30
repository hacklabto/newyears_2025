use crate::free_list::FreeList;
use crate::note::Note;
use crate::sound_sample::SoundSampleI32;
use crate::sound_source_core::SoundSourceCore;

///
/// Amp Adder
///
pub struct AmpAdder<const PLAY_FREQUENCY: u32, const NUM_CHANNELS: usize> {
    pub free_list: FreeList<NUM_CHANNELS>,
    pub channels: [Note<PLAY_FREQUENCY>; NUM_CHANNELS],
}

impl<const PLAY_FREQUENCY: u32, const NUM_CHANNELS: usize> Default
    for AmpAdder<PLAY_FREQUENCY, NUM_CHANNELS>
{
    fn default() -> Self {
        Self {
            free_list: { FreeList::<NUM_CHANNELS>::default() },
            channels: { core::array::from_fn(|_idx| Note::<PLAY_FREQUENCY>::default()) },
        }
    }
}

impl<const PLAY_FREQUENCY: u32, const NUM_CHANNELS: usize> AmpAdder<PLAY_FREQUENCY, NUM_CHANNELS> {
    pub fn alloc(self: &mut Self) -> usize {
        self.free_list.alloc()
    }
}

impl<const PLAY_FREQUENCY: u32, const NUM_CHANNELS: usize> SoundSourceCore<PLAY_FREQUENCY>
    for AmpAdder<PLAY_FREQUENCY, NUM_CHANNELS>
{
    type InitValuesType = u32;

    fn new(_init_values: Self::InitValuesType) -> Self {
        return Self::default();
    }

    fn get_next(self: &mut Self) -> SoundSampleI32 {
        let mut output: SoundSampleI32 = SoundSampleI32::ZERO;

        for i in 0..NUM_CHANNELS {
            if self.free_list.is_active(i) {
                let entry = &mut (self.channels[i]);
                if !entry.has_next() {
                    self.free_list.free(i);
                }
                let this_source: SoundSampleI32 = entry.get_next();
                output = output + this_source;
            }
        }

        output = SoundSampleI32::new_i32(output.to_i32() / 32);
        if output.to_i32() > 0x8000 || output.to_i32() < -0x8000 {
            println!("clip {}", output.to_i32());
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
