use crate::sound_sample::U32Fraction;

pub struct MidiTime<const PLAY_FREQUENCY: u32> {
    current_ms_per_quarter_note: u32,
    ticks_per_quarter_note: u32,
    midi_event_update_rate: U32Fraction<PLAY_FREQUENCY>,
}

impl<const PLAY_FREQUENCY: u32> MidiTime<PLAY_FREQUENCY> {
    fn compute_midi_events_per_second(self: &mut Self) {
        // We are being called PLAY_FREQUENCY times a second.
        // one quarter note = current_ms_per_qn / 1 000 000 seconds
        // 1 tick = (current_ms_per_qn / 1 000 000 seconds) / ticks_per_qn
        // midi events / second = (1 000 000 * ticks_per_qn ) / current_ms_per_qn
        // TODO, rethink timing a bit.
        //
        // I probably want a meta data class of some kind for things like the timing
        // updates.  Also, I want to handle the case where the playback is slower
        // than the midi playback, so I can fast forward through the track to get
        // a good maximum output voltage.
        //
        let midi_events_per_second: u32 = (1000000u64 * (self.ticks_per_quarter_note as u64)
            / (self.current_ms_per_quarter_note as u64))
            as u32;
        let midi_events_per_sample = midi_events_per_second / PLAY_FREQUENCY;
        let midi_events_per_sample_remainder = midi_events_per_second % PLAY_FREQUENCY;

        self.midi_event_update_rate =
            U32Fraction::new(midi_events_per_sample, midi_events_per_sample_remainder);
    }
    pub fn set_ms_per_quarter_note(self: &mut Self, current_ms_per_quarter_note: u32) {
        self.current_ms_per_quarter_note = current_ms_per_quarter_note;
        self.compute_midi_events_per_second();
    }

    pub fn new(current_ms_per_quarter_note: u32, ticks_per_quarter_note: u32) -> Self {
        let mut rval = Self {
            current_ms_per_quarter_note,
            ticks_per_quarter_note,
            midi_event_update_rate: U32Fraction::new(0, 0),
        };
        rval.compute_midi_events_per_second();
        rval
    }

    pub fn get_update_rate(self: &Self) -> &U32Fraction<PLAY_FREQUENCY> {
        return &self.midi_event_update_rate;
    }
}
