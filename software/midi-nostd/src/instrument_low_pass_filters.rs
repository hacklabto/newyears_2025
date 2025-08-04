use crate::midi_notes::midi_note_to_freq;
use crate::midi_notes::FREQUENCY_MULTIPLIER;
use crate::note::SoundSourceNoteInit;

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

pub struct GenericKeyBased<const KEY_SCALE: u32, const BASE: u32, const VEL_DIVIDE: u32> {}

impl<const KEY_SCALE: u32, const BASE: u32, const VEL_DIVIDE: u32> FrequencyCalculator
    for GenericKeyBased<KEY_SCALE, BASE, VEL_DIVIDE>
{
    fn get_cutoff_frequency(init_values: &SoundSourceNoteInit) -> u32 {
        BASE + ((init_values.key as u32) * KEY_SCALE) + ((init_values.velocity as u32) / VEL_DIVIDE)
    }
}

pub struct GenericLowPassCalculator<const SCALE_DOWN_PERCENT: u32, const OFFSET: u32> {}

impl<const SCALE_DOWN_PERCENT: u32, const OFFSET: u32> FrequencyCalculator
    for GenericLowPassCalculator<SCALE_DOWN_PERCENT, OFFSET>
{
    fn get_cutoff_frequency(init_values: &SoundSourceNoteInit) -> u32 {
        let frequency = midi_note_to_freq(init_values.key);
        ((frequency / FREQUENCY_MULTIPLIER) * SCALE_DOWN_PERCENT / 100) + OFFSET
    }
}
