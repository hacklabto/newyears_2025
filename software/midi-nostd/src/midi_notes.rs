// All frequencies will be multiplied by 100 so low frequency notes (i.e.,
// 16.35 hz) can be represented as an integer (1635 in this case)
// TODO - Should this constant be in this file?  Probably not, bit it seems silly to
// make a frequency.rs with just this entry.

pub const FREQUENCY_MULTIPLIER: u32 = 100;

#[allow(unused)]
const MIDI_NOTE_TO_FREQ: [u32; 128] = [
    1254385, 1183982, 1117530, 1054808, 995606, 939727, 886984, 837202, 790213, 745862, 704000,
    664488, 627193, 591991, 558765, 527404, 497803, 469864, 443492, 418601, 395107, 372931, 352000,
    332244, 313596, 295996, 279383, 263702, 248902, 234932, 221746, 209300, 197553, 186466, 176000,
    166122, 156798, 147998, 139691, 131851, 124451, 117466, 110873, 104650, 98777, 93233, 88000,
    83061, 78399, 73999, 69846, 65926, 62225, 58733, 55437, 52325, 49388, 46616, 44000, 41530,
    39200, 36999, 34923, 32963, 31113, 29366, 27718, 26163, 24694, 23308, 22000, 20765, 19600,
    18500, 17461, 16481, 15556, 14683, 13859, 13081, 12347, 11654, 11000, 10383, 9800, 9250, 8731,
    8241, 7778, 7342, 6930, 6541, 6174, 5827, 5500, 5191, 4900, 4625, 4365, 4120, 3889, 3671, 3465,
    3270, 3087, 2914, 2750, 2596, 2450, 2312, 2183, 2060, 1945, 1835, 1732, 1635, 1543, 1457, 1375,
    1298, 1225, 1156, 1091, 1030, 972, 918, 866, 818,
];

#[allow(unused)]
pub fn midi_note_to_freq(midi_note: u8) -> u32 {
    let midi_note_flipped: usize = (127 - midi_note).into();
    let freq: u32 = MIDI_NOTE_TO_FREQ[midi_note_flipped];
    freq
}

#[cfg(test)]

mod tests {

    use crate::midi_notes::midi_note_to_freq;
    use crate::midi_notes::FREQUENCY_MULTIPLIER;

    #[test]
    // Sanity test a few notes against reference frequencies
    fn test_samples() {
        // C4 (261.63 Hz * 100)
        assert_eq!(
            261.63 * (FREQUENCY_MULTIPLIER as f32),
            midi_note_to_freq(60) as f32
        );
        assert_eq!(44000, midi_note_to_freq(69)); // A4 (440 Hz * 100)
        assert_eq!(2750, midi_note_to_freq(21)); // A0 (27.5 Hz * 100)
        assert_eq!(98777, midi_note_to_freq(83)); // B5 (987.77 * 100)
    }
}
