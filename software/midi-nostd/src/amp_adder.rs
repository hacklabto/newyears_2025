use crate::free_list::FreeList;
use crate::note::Note;
use crate::note::SoundSourceNoteInit;
use crate::sound_sample::SoundSampleI32;
use crate::sound_source_core::SoundSourceCore;

///
/// Amp Adder
///
pub struct AmpAdder<const PLAY_FREQUENCY: u32, const NUM_CHANNELS: usize> {
    free_list: FreeList<NUM_CHANNELS>,
    channels: [Note<PLAY_FREQUENCY>; NUM_CHANNELS],
    divider: i32,
}

impl<const PLAY_FREQUENCY: u32, const NUM_CHANNELS: usize> AmpAdder<PLAY_FREQUENCY, NUM_CHANNELS> {
    pub fn alloc(self: &mut Self) -> usize {
        self.free_list.alloc()
    }

    pub fn trigger_note_off_at(self: &mut Self, element: usize) {
        self.channels[element].trigger_note_off();
    }

    pub fn new_note_at(self: &mut Self, element: usize, note_init: SoundSourceNoteInit) {
        self.channels[element] = Note::<PLAY_FREQUENCY>::new(note_init);
    }
}

impl<const PLAY_FREQUENCY: u32, const NUM_CHANNELS: usize> SoundSourceCore<PLAY_FREQUENCY>
    for AmpAdder<PLAY_FREQUENCY, NUM_CHANNELS>
{
    type InitValuesType = i32;

    fn new(divider: Self::InitValuesType) -> Self {
        Self {
            free_list: { FreeList::<NUM_CHANNELS>::default() },
            channels: { core::array::from_fn(|_idx| Note::<PLAY_FREQUENCY>::default()) },
            divider,
        }
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

        SoundSampleI32::new_i32(output.to_i32() / self.divider)
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
        let mut amp_adder = AmpAdder::<24000, 2>::new(1);

        assert_eq!(0, amp_adder.get_next().to_i32());
        assert_eq!(0, amp_adder.get_next().to_i32());
        assert_eq!(0, amp_adder.get_next().to_i32());
    }
}
