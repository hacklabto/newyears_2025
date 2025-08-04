use crate::midi_notes::midi_note_to_freq;
use crate::note::SoundSourceNoteInit;
use crate::midi_notes::FREQUENCY_MULTIPLIER;

pub trait FrequencyCalculator {
    fn get_cutoff_frequency(init_values: &SoundSourceNoteInit) -> u32;
}

pub struct CelloLowPassCalculator {}

impl FrequencyCalculator for CelloLowPassCalculator {
    fn get_cutoff_frequency(init_values: &SoundSourceNoteInit) -> u32 {
        let frequency_1 = midi_note_to_freq(init_values.key);
        ((frequency_1 / FREQUENCY_MULTIPLIER) * 90 / 100) + 400
    }
}

pub struct PianoLowPassCalculator {}

impl FrequencyCalculator for PianoLowPassCalculator {
    fn get_cutoff_frequency(init_values: &SoundSourceNoteInit) -> u32 {
        // experimentally derived
        200 + ((init_values.key as u32) * 5) + ((init_values.velocity as u32) / 3)
    }
}

