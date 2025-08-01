use crate::amp_adder::AmpAdder;
use crate::midi_channels::Channels;
use crate::midi_events::*;
use crate::midi_time::MidiTime;
use crate::sound_sample::U32Fraction;

pub struct MidiTrack<const PLAY_FREQUENCY: u32, const MAX_NOTES: usize> {
    active: bool,
    current_event_idx: usize,
    current_time: U32Fraction<PLAY_FREQUENCY>,
    next_event_time: u32,
    last_delta: u32,
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
    pub fn has_next(self: &Self) -> bool {
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
            handle_track_event(
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

        self.current_time.add(tempo.get_update_rate());
    }
}
