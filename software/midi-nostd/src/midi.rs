use crate::amp_adder::AmpAdder;
use crate::note::Note;
use crate::note::SoundSourceNoteInit;
use crate::sound_sample::SoundSampleI32;
use crate::sound_source_core::SoundSourceCore;
use midly::Smf;

pub struct MidiTrack<const PLAY_FREQUENCY: u32, const MAX_NOTES: usize> {
    active: bool,
    current_event_idx: usize,
    current_time: u32,
    current_remainder: u32,
    next_event_time: u32,
    last_delta: u32,
    playing_notes: [Option<usize>; 128],
}

impl<const PLAY_FREQUENCY: u32, const MAX_NOTES: usize> MidiTrack<PLAY_FREQUENCY, MAX_NOTES> {
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
        let last_delta = 0;

        Self {
            active,
            current_event_idx,
            current_time,
            current_remainder,
            next_event_time,
            last_delta,
            playing_notes: [Option::<usize>::default(); 128],
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
    ) {
        match midi_event {
            midly::MidiMessage::NoteOn { key, vel } => {
                let key_as_u32: u8 = (*key).into();

                let note_init = SoundSourceNoteInit::new((*key).into(), 0, (*vel).into());
                let dst = if let Some(playing_note) = self.playing_notes[key_as_u32 as usize] {
                    playing_note
                } else {
                    notes.alloc()
                };

                notes.channels[dst] = Note::<PLAY_FREQUENCY>::new(note_init);
                self.playing_notes[key_as_u32 as usize] = Some(dst)
            }
            midly::MidiMessage::NoteOff { key, vel: _ } => {
                let key_as_u32: u8 = (*key).into();
                if let Some(playing_note) = self.playing_notes[key_as_u32 as usize] {
                    notes.channels[playing_note].trigger_note_off();
                    self.playing_notes[key_as_u32 as usize] = None;
                }
            }
            _ => {}
        }
    }

    pub fn handle_track_event<'a, const NUM_CHANNELS: usize>(
        self: &mut Self,
        track_event: &midly::TrackEventKind,
        notes: &mut AmpAdder<PLAY_FREQUENCY, NUM_CHANNELS>,
    ) {
        match track_event {
            midly::TrackEventKind::Midi {
                message,
                channel: _,
            } => self.handle_midi_event(&message, notes),
            _ => {}
        }
    }

    pub fn update<'a>(
        self: &mut Self,
        events: &'a midly::Track<'a>,
        notes: &mut AmpAdder<PLAY_FREQUENCY, MAX_NOTES>,
    ) {
        if !self.active {
            return;
        }
        while self.current_time == self.next_event_time {
            self.handle_track_event(&(events[self.current_event_idx]).kind, notes);
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
        if (self.current_remainder) % 32 == 0 {
            self.current_time = self.current_time + 1;
        }
    }
}

pub struct Midi<const PLAY_FREQUENCY: u32, const MAX_NOTES: usize, const MAX_TRACKS: usize> {
    num_tracks: usize,
    tracks: [MidiTrack<PLAY_FREQUENCY, MAX_NOTES>; MAX_TRACKS],
    amp_adder: AmpAdder<PLAY_FREQUENCY, MAX_NOTES>,
}

impl<const PLAY_FREQUENCY: u32, const MAX_NOTES: usize, const MAX_TRACKS: usize>
    Midi<PLAY_FREQUENCY, MAX_NOTES, MAX_TRACKS>
{
    pub fn new(smf: &Smf, divider: i32) -> Self {
        let num_tracks = smf.tracks.len();
        let tracks: [MidiTrack<PLAY_FREQUENCY, MAX_NOTES>; MAX_TRACKS] =
            core::array::from_fn(|idx| {
                let smf_idx = if idx < num_tracks {
                    idx
                } else {
                    num_tracks - 1
                };
                MidiTrack::<PLAY_FREQUENCY, MAX_NOTES>::new(&smf.tracks[smf_idx])
            });
        let amp_adder = AmpAdder::<PLAY_FREQUENCY, MAX_NOTES>::new(divider);
        Self {
            num_tracks,
            tracks,
            amp_adder,
        }
    }

    pub fn get_loudest_sample(self: &mut Self, smf: &Smf) -> i32 {
        let mut loudest: i32 = 0;
        let mut count: u32 = 0;
        while self.has_next() && count < 200000 {
            let sample = self.get_next(&smf).to_i32();
            let abs_sample = if sample < 0 { -sample } else { sample };
            loudest = if abs_sample > loudest {
                abs_sample
            } else {
                loudest
            };
            count = count + 1;
        }
        loudest
    }

    pub fn get_next(self: &mut Self, smf: &Smf) -> SoundSampleI32 {
        let result = self.amp_adder.get_next();
        for i in 0..self.num_tracks {
            if i != 10 {
                self.tracks[i].update(&smf.tracks[i], &mut self.amp_adder);
            }
        }
        result
    }

    pub fn has_next(self: &Self) -> bool {
        let mut has_next: bool = false;
        for i in 0..self.num_tracks {
            has_next = has_next || self.tracks[i].has_next()
        }
        has_next
    }
}

#[cfg(test)]
mod tests {

    use crate::midi::Midi;

    #[test]
    fn basic_midi_test() {
        let smf = midly::Smf::parse(include_bytes!("../assets/twinkle.mid"))
            .expect("It's inlined data, so it better work, gosh darn it");
        let mut midi = Midi::<24000, 32, 16>::new(&smf);

        assert_eq!(0, midi.get_next(&smf).to_i32());
        //assert_eq!(8192, midi.get_next(&smf).to_i32());
        //assert_eq!(8719, midi.get_next(&smf).to_i32());
        //assert_eq!(9246, midi.get_next(&smf).to_i32());
    }
}
