use crate::amp_adder::AmpAdder;
use crate::note::SoundSourceNoteInit;
use crate::sound_sample::SoundSampleI32;
use crate::sound_source_core::SoundSourceCore;
use midly::Smf;

pub struct MidiTrack<const PLAY_FREQUENCY: u32> {
    active: bool,
    current_event_idx: usize,
    current_time: u32,
    current_remainder: u32,
    next_event_time: u32,
    ignore_hack: u8,
    last_delta: u32,
}

impl<const PLAY_FREQUENCY: u32> MidiTrack<PLAY_FREQUENCY> {
    pub fn new<'a>(events: &'a midly::Track<'a>) -> Self {
        let active = events.len() != 0;
        let current_event_idx: usize = 0;
        let current_time: u32 = 0;
        let current_remainder: u32 = 0;
        let next_event_time: u32 = if active {
            events[current_event_idx].delta.into()
        } else {
            0
        };
        let ignore_hack = 0;
        let last_delta = 0;

        Self {
            active,
            current_event_idx,
            current_time,
            current_remainder,
            next_event_time,
            ignore_hack,
            last_delta,
        }
    }
    fn has_next(self: &Self) -> bool {
        self.active
    }

    pub fn go_to_next_event<'a>(self: &mut Self, events: &'a midly::Track<'a>) {
        if !self.active {
            return;
        }
        self.current_event_idx += 1;
        if self.current_event_idx >= events.len() {
            self.active = false;
            return;
        }
    }

    pub fn handle_midi_event<const NUM_CHANNELS: usize>(
        self: &mut Self,
        midi_event: &midly::MidiMessage,
        notes: &mut AmpAdder<PLAY_FREQUENCY, NUM_CHANNELS>,
        dest_note: &mut usize,
    ) {
        match midi_event {
            midly::MidiMessage::NoteOn { key, vel: _ } => {
                let key_as_u32: u8 = (*key).into();
                if key_as_u32 != self.ignore_hack || self.last_delta != 0 {
                    self.ignore_hack = key_as_u32;

                    let note_init = SoundSourceNoteInit::new((*key).into(), 0);
                    notes.channels[*dest_note].init(&note_init);
                    *dest_note = 1 - *dest_note;
                }
            }
            midly::MidiMessage::NoteOff { key: _, vel: _ } => {
                self.ignore_hack = 0;
                notes.channels[1 - *dest_note].trigger_note_off();
            }
            _ => {}
        }
    }

    pub fn handle_track_event<'a, const NUM_CHANNELS: usize>(
        self: &mut Self,
        track_event: &midly::TrackEventKind,
        notes: &mut AmpAdder<PLAY_FREQUENCY, NUM_CHANNELS>,
        dest_note: &mut usize,
    ) {
        match track_event {
            midly::TrackEventKind::Midi {
                message,
                channel: _,
            } => self.handle_midi_event(&message, notes, dest_note),
            _ => {}
        }
    }

    pub fn update<'a, const NUM_CHANNELS: usize>(
        self: &mut Self,
        events: &'a midly::Track<'a>,
        notes: &mut AmpAdder<PLAY_FREQUENCY, NUM_CHANNELS>,
        dest_note: &mut usize,
    ) {
        if !self.active {
            return;
        }
        while self.current_time == self.next_event_time {
            self.handle_track_event(&(events[self.current_event_idx]).kind, notes, dest_note);
            self.go_to_next_event(events);
            if !self.active {
                return;
            }
            let delta: u32 = events[self.current_event_idx].delta.into();
            self.next_event_time = self.current_time + delta;
            self.last_delta = delta;
        }
        self.current_remainder = self.current_remainder + 1;
        // TODO, adjust properly.
        if (self.current_remainder) % 25 == 0 {
            self.current_time = self.current_time + 1;
        }
    }
}

pub struct MidiReal<'a, const PLAY_FREQUENCY: u32> {
    smf: Smf<'a>,
    track: MidiTrack<PLAY_FREQUENCY>,
    amp_adder: AmpAdder<PLAY_FREQUENCY, 5>,
    dest_note: usize,
}

impl<'a, const PLAY_FREQUENCY: u32> MidiReal<'a, PLAY_FREQUENCY> {
    pub fn new(midi_bytes: &'static [u8]) -> Self {
        let smf = midly::Smf::parse(midi_bytes)
            .expect("It's inlined data, so it better work, gosh darn it");
        let track = MidiTrack::new(&smf.tracks[0]);
        let amp_adder = AmpAdder::<PLAY_FREQUENCY, 5>::default();
        let dest_note = 0;
        Self {
            smf,
            track,
            amp_adder,
            dest_note,
        }
    }
    fn get_next(self: &mut Self) -> SoundSampleI32 {
        let result = self.amp_adder.get_next();
        self.track.update::<5>(
            &self.smf.tracks[0],
            &mut self.amp_adder,
            &mut self.dest_note,
        );
        result
    }

    fn has_next(self: &Self) -> bool {
        self.track.has_next()
    }
}

///
/// Midi Playback
///
pub struct Midi<'a, const PLAY_FREQUENCY: u32> {
    midi_maybe: Option<MidiReal<'a, PLAY_FREQUENCY>>,
}

impl<const PLAY_FREQUENCY: u32> Default for Midi<'_, PLAY_FREQUENCY> {
    fn default() -> Self {
        let midi = MidiReal::<PLAY_FREQUENCY>::new(include_bytes!("../assets/twinkle.mid"));
        Self {
            midi_maybe: Some(midi),
        }
    }
}

impl<const PLAY_FREQUENCY: u32> Midi<'_, PLAY_FREQUENCY> {
    pub fn new(midi_bytes: &'static [u8]) -> Self {
        Self {
            midi_maybe: Some(MidiReal::new(midi_bytes)),
        }
    }
    pub fn get_next(self: &mut Self) -> SoundSampleI32 {
        self.midi_maybe.as_mut().unwrap().get_next()
    }

    pub fn has_next(self: &Self) -> bool {
        self.midi_maybe.as_ref().unwrap().has_next()
    }
}

#[cfg(test)]
mod tests {

    use crate::midi::Midi;

    #[test]
    fn basic_midi_test() {
        let mut all_pools = Midi::<24000>::default();

        assert_eq!(0, all_pools.get_next().to_i32());
        assert_eq!(0, all_pools.get_next().to_i32());
        assert_eq!(-6, all_pools.get_next().to_i32());
        assert_eq!(-11, all_pools.get_next().to_i32());
        assert_eq!(-15, all_pools.get_next().to_i32());
    }
}
