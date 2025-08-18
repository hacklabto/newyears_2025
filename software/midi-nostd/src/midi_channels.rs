// For keeping track of what notes are playing on the 16 midi channels.
// Binds (channel, note) to an array element in the AmpAdder data structure;
// these entries are stored as a u8 because space is at a premium.
//

pub struct Channel {
    pub current_program: u8,
    pub playing_notes: [u8; 128],
}

impl Channel {
    pub const UNUSED: u8 = 0xff;

    fn get_note_state(self: &Self, note_volume: &mut [u8; 128]) {
        for idx in 0..128 {
            if self.playing_notes[idx] != Self::UNUSED && self.playing_notes[idx] > note_volume[idx]
            {
                note_volume[idx] = self.playing_notes[idx];
            }
        }
    }
}

impl Default for Channel {
    fn default() -> Self {
        Self {
            current_program: 0,
            playing_notes: [Self::UNUSED; 128],
        }
    }
}

pub struct Channels {
    pub channels: [Channel; 16],
}

impl Channels {
    pub fn get_note_state(self: &Self, note_volume: &mut [u8; 128]) {
        for channel in self.channels.iter() {
            channel.get_note_state(note_volume);
        }
    }
}

impl Default for Channels {
    fn default() -> Self {
        Self {
            channels: core::array::from_fn(|_idx| Channel::default()),
        }
    }
}
