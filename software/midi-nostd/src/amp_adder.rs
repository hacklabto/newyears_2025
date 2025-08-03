use crate::free_list::FreeList;
use crate::note::Note;
use crate::note::SoundSourceNoteInit;
use crate::sound_sample::SoundSampleI32;
use crate::sound_source_core::SoundSourceCore;

///
/// Amp Adder
///
pub struct AmpAdder<
    const P_FREQ: u32,
    const U_FREQ: u32,
    const NUM_CHANNELS: usize,
    const NO_SCALEDOWN: bool,
> {
    free_list: FreeList<NUM_CHANNELS>,
    channels: [Note<P_FREQ, U_FREQ>; NUM_CHANNELS],
    active_channel_list: [usize; NUM_CHANNELS],
    num_active_channels: usize,
    scale: SoundSampleI32,
}

impl<const P_FREQ: u32, const U_FREQ: u32, const NUM_CHANNELS: usize, const NO_SCALEDOWN: bool>
    AmpAdder<P_FREQ, U_FREQ, NUM_CHANNELS, NO_SCALEDOWN>
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

    pub fn restart_note_at(self: &mut Self, element: usize, vel: u8) {
        self.channels[element].restart(vel);
    }

    pub fn get_current_num_mixed_notes(self: &mut Self) -> u32 {
        return self.num_active_channels as u32;
    }
}

impl<const P_FREQ: u32, const U_FREQ: u32, const NUM_CHANNELS: usize, const NO_SCALEDOWN: bool>
    SoundSourceCore<P_FREQ, U_FREQ> for AmpAdder<P_FREQ, U_FREQ, NUM_CHANNELS, NO_SCALEDOWN>
{
    type InitValuesType = i32;

    fn new(divider: Self::InitValuesType) -> Self {
        let scale: SoundSampleI32 = if NO_SCALEDOWN {
            SoundSampleI32::ZERO
        } else {
            SoundSampleI32::new_i32(0x8000 / divider)
        };

        Self {
            free_list: { FreeList::<NUM_CHANNELS>::default() },
            channels: { core::array::from_fn(|_idx| Note::<P_FREQ, U_FREQ>::default()) },
            num_active_channels: 0,
            scale,
            active_channel_list: { core::array::from_fn(|_idx| 0) },
        }
    }

    #[inline(never)]
    fn get_next(self: &mut Self) -> SoundSampleI32 {
        let mut output: SoundSampleI32 = SoundSampleI32::ZERO;

        let active_channels = &self.active_channel_list[0..self.num_active_channels];

        for i in active_channels {
            output = output + self.channels[*i].get_next();
        }

        if NO_SCALEDOWN {
            output
        } else {
            output * self.scale
        }
    }

    fn update(self: &mut Self) {
        self.num_active_channels = 0;
        for i in 0..NUM_CHANNELS {
            if self.free_list.is_active(i) {
                let entry = &mut (self.channels[i]);
                if !entry.has_next() {
                    self.free_list.free(i);
                } else {
                    entry.update();
                    self.active_channel_list[self.num_active_channels] = i;
                    self.num_active_channels = self.num_active_channels + 1;
                }
            }
        }
    }

    fn has_next(self: &Self) -> bool {
        true
    }

    fn restart(self: &mut Self, _vel: u8) {}
}

#[cfg(test)]
mod tests {
    use crate::amp_adder::*;

    #[test]
    fn basic_amp_adder_test() {
        let mut amp_adder = AmpAdder::<24000, 24000, 2, false>::new(1);

        assert_eq!(0, amp_adder.get_next().to_i32());
        assert_eq!(0, amp_adder.get_next().to_i32());
        assert_eq!(0, amp_adder.get_next().to_i32());
    }
}
