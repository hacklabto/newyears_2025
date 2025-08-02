use crate::free_list::FreeList;
use crate::note::Note;
use crate::note::SoundSourceNoteInit;
use crate::sound_sample::SoundSampleI32;
use crate::sound_source_core::SoundSourceCore;

///
/// Amp Adder
///
pub struct AmpAdder<const P_FREQ: u32, const U_FREQ: u32, const NUM_CHANNELS: usize> {
    free_list: FreeList<NUM_CHANNELS>,
    channels: [Note<P_FREQ, U_FREQ>; NUM_CHANNELS],
    divider: i32,
}

impl<const P_FREQ: u32, const U_FREQ: u32, const NUM_CHANNELS: usize>
    AmpAdder<P_FREQ, U_FREQ, NUM_CHANNELS>
{
    pub fn alloc(self: &mut Self) -> usize {
        self.free_list.alloc()
    }

    pub fn trigger_note_off_at(self: &mut Self, element: usize) {
        self.channels[element].trigger_note_off();
    }

    pub fn new_note_at(self: &mut Self, element: usize, note_init: SoundSourceNoteInit) {
        self.channels[element] = Note::<P_FREQ, U_FREQ>::new(note_init);
    }
}

impl<const P_FREQ: u32, const U_FREQ: u32, const NUM_CHANNELS: usize>
    SoundSourceCore<P_FREQ, U_FREQ> for AmpAdder<P_FREQ, U_FREQ, NUM_CHANNELS>
{
    type InitValuesType = i32;

    fn new(divider: Self::InitValuesType) -> Self {
        Self {
            free_list: { FreeList::<NUM_CHANNELS>::default() },
            channels: { core::array::from_fn(|_idx| Note::<P_FREQ, U_FREQ>::default()) },
            divider,
        }
    }

    fn get_next(self: &mut Self) -> SoundSampleI32 {
        let mut output: SoundSampleI32 = SoundSampleI32::ZERO;

        for i in 0..NUM_CHANNELS {
            if self.free_list.is_active(i) {
                let entry = &mut (self.channels[i]);
                output = output + entry.get_next();
            }
        }

        SoundSampleI32::new_i32(output.to_i32() / self.divider)
    }

    fn update(self: &mut Self) {
        for i in 0..NUM_CHANNELS {
            if self.free_list.is_active(i) {
                let entry = &mut (self.channels[i]);
                if !entry.has_next() {
                    self.free_list.free(i);
                } else {
                    entry.update();
                }
            }
        }
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
        let mut amp_adder = AmpAdder::<24000, 24000, 2>::new(1);

        assert_eq!(0, amp_adder.get_next().to_i32());
        assert_eq!(0, amp_adder.get_next().to_i32());
        assert_eq!(0, amp_adder.get_next().to_i32());
    }
}
