use crate::amp_adder::AmpAdder;
use crate::note::Note;
use crate::note::SoundSourceNoteInit;
use crate::sound_sample::SoundSampleI32;
use crate::sound_sample::U32Fraction;
use crate::sound_source_core::SoundSourceCore;
use midly::Smf;
use midly::Timing;

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
    fn set_ms_per_quarter_note(self: &mut Self, current_ms_per_quarter_note: u32) {
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
}

pub struct MidiTrack<const PLAY_FREQUENCY: u32, const MAX_NOTES: usize> {
    active: bool,
    current_event_idx: usize,
    current_time: U32Fraction<PLAY_FREQUENCY>,
    next_event_time: u32,
    last_delta: u32,
}

pub struct Channel {
    pub current_program: u8,
    pub playing_notes: [u8; 128],
}

impl Channel {
    const UNUSED: u8 = 0xff;
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

impl Default for Channels {
    fn default() -> Self {
        Self {
            channels: core::array::from_fn(|_idx| Channel::default()),
        }
    }
}

impl<const PLAY_FREQUENCY: u32, const MAX_NOTES: usize> MidiTrack<PLAY_FREQUENCY, MAX_NOTES> {
    pub fn new<'a>(events: &'a midly::Track<'a>) -> Self {
        let active = events.len() != 0;
        let current_event_idx: usize = 0;
        let next_event_time: u32 = if active {
            events[current_event_idx].delta.into()
        } else {
            0
        };
        let last_delta = 0;

        Self {
            active,
            current_event_idx,
            current_time: U32Fraction::<PLAY_FREQUENCY>::new(0, 0),
            next_event_time,
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
        channel_u8: u8,
        notes: &mut AmpAdder<PLAY_FREQUENCY, NUM_CHANNELS>,
        channels: &mut Channels,
    ) {
        let channel: usize = channel_u8 as usize;
        if channel == 10 {
            return;
        }
        match midi_event {
            midly::MidiMessage::NoteOn { key, vel } => {
                let key_as_u32: u8 = (*key).into();

                let note_init = SoundSourceNoteInit::new(
                    (*key).into(),
                    channels.channels[channel].current_program,
                    (*vel).into(),
                );
                let playing_note =
                    channels.channels[channel].playing_notes[key_as_u32 as usize];
                let dst = if playing_note != Channel::UNUSED
                {
                    playing_note as usize
                } else {
                    notes.alloc()
                };
                assert!( dst < (Channel::UNUSED as usize));

                notes.channels[dst] = Note::<PLAY_FREQUENCY>::new(note_init);
                channels.channels[channel].playing_notes[key_as_u32 as usize] = dst as u8;
            }
            midly::MidiMessage::NoteOff { key, vel: _ } => {
                let key_as_u32: u8 = (*key).into();
                let playing_note =
                    channels.channels[channel].playing_notes[key_as_u32 as usize];

                if playing_note != Channel::UNUSED
                {
                    notes.channels[playing_note as usize].trigger_note_off();
                    channels.channels[channel].playing_notes[key_as_u32 as usize] = Channel::UNUSED;
                }
            }
            midly::MidiMessage::ProgramChange { program } => {
                channels.channels[channel].current_program = (*program).into();
            }
            _ => {}
        }
    }

    pub fn handle_track_event<'a, const NUM_CHANNELS: usize>(
        self: &mut Self,
        track_event: &midly::TrackEventKind,
        notes: &mut AmpAdder<PLAY_FREQUENCY, NUM_CHANNELS>,
        channels: &mut Channels,
        tempo: &mut MidiTime<PLAY_FREQUENCY>,
    ) {
        match track_event {
            midly::TrackEventKind::Midi { message, channel } => {
                self.handle_midi_event(&message, (*channel).into(), notes, channels)
            }
            midly::TrackEventKind::Meta(message) => match message {
                midly::MetaMessage::Tempo(ms_per_qn_midly) => {
                    let ms_per_qn: u32 = (*ms_per_qn_midly).into();
                    tempo.set_ms_per_quarter_note(ms_per_qn as u32);
                }
                _ => {}
            },
            _ => {}
        }
    }

    pub fn update<'a>(
        self: &mut Self,
        events: &'a midly::Track<'a>,
        notes: &mut AmpAdder<PLAY_FREQUENCY, MAX_NOTES>,
        channels: &mut Channels,
        tempo: &mut MidiTime<PLAY_FREQUENCY>,
    ) {
        if !self.active {
            return;
        }
        while self.current_time.int_part >= self.next_event_time {
            self.handle_track_event(
                &(events[self.current_event_idx]).kind,
                notes,
                channels,
                tempo,
            );
            self.go_to_next_event(events);
            if !self.active {
                return;
            }
            let delta: u32 = events[self.current_event_idx].delta.into();
            self.next_event_time = self.next_event_time + delta;
            self.last_delta = delta;
        }

        self.current_time.add(&tempo.midi_event_update_rate);
    }
}

pub struct Midi<const PLAY_FREQUENCY: u32, const MAX_NOTES: usize, const MAX_TRACKS: usize> {
    num_tracks: usize,
    tracks: [MidiTrack<PLAY_FREQUENCY, MAX_NOTES>; MAX_TRACKS],
    amp_adder: AmpAdder<PLAY_FREQUENCY, MAX_NOTES>,
    channels: Channels,
    tempo: MidiTime<PLAY_FREQUENCY>,
}

impl<const PLAY_FREQUENCY: u32, const MAX_NOTES: usize, const MAX_TRACKS: usize>
    Midi<PLAY_FREQUENCY, MAX_NOTES, MAX_TRACKS>
{
    pub fn new_internal(smf: &Smf, divider: i32) -> Self {
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

        let tpqn_midly = match smf.header.timing {
            Timing::Metrical(ticks) => ticks,
            Timing::Timecode(_, _) => todo!(),
        };
        let tpqn_u16: u16 = tpqn_midly.into();
        let tempo = MidiTime::new(500000, tpqn_u16 as u32);

        Self {
            num_tracks,
            tracks,
            amp_adder,
            channels: Channels::default(),
            tempo,
        }
    }

    pub fn get_loudest_sample(smf: &Smf) -> i32 {
        let mut fast_forward_midi_player = Midi::<24, MAX_NOTES, MAX_TRACKS>::new_internal(&smf, 1);
        let mut loudest: i32 = 0;
        while fast_forward_midi_player.has_next() {
            let sample = fast_forward_midi_player.get_next(&smf).to_i32();
            let abs_sample = if sample < 0 { -sample } else { sample };
            loudest = if abs_sample > loudest {
                abs_sample
            } else {
                loudest
            };
        }
        loudest
    }

    pub fn new(smf: &Smf) -> Self {
        //
        // Limit to 255 (not 256) notes to save space in the midi data
        // structure.  On embedded platforms memory is often limited
        //
        assert!(MAX_NOTES < 0xff);
        let loudest = Self::get_loudest_sample(smf);
        Midi::<PLAY_FREQUENCY, MAX_NOTES, MAX_TRACKS>::new_internal(&smf, loudest / 0x8000 + 1)
    }

    pub fn get_next(self: &mut Self, smf: &Smf) -> SoundSampleI32 {
        let result = self.amp_adder.get_next();
        for i in 0..self.num_tracks {
            self.tracks[i].update(
                &smf.tracks[i],
                &mut self.amp_adder,
                &mut self.channels,
                &mut self.tempo,
            );
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
        let mut midi = Midi::<24000, 32, 16>::new_internal(&smf, 1);

        assert_eq!(0, midi.get_next(&smf).to_i32());
        //assert_eq!(8192, midi.get_next(&smf).to_i32());
        //assert_eq!(8719, midi.get_next(&smf).to_i32());
        //assert_eq!(9246, midi.get_next(&smf).to_i32());
    }
}
