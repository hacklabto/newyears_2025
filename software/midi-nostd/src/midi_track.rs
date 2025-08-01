use crate::amp_adder::AmpAdder;
use crate::midi_channels::Channels;
use crate::midi_events::*;
use crate::midi_time::MidiTime;
use crate::sound_sample::U32Fraction;

pub struct MidiTrack<'a, const PLAY_FREQUENCY: u32, const MAX_NOTES: usize> {
    last_event: Option<Result<midly::TrackEvent<'a>, midly::Error>>,
    event_iter: midly::EventIter<'a>,
    current_time: U32Fraction<PLAY_FREQUENCY>,
    next_event_time: u32,
    last_delta: u32,
}

impl<'a, const PLAY_FREQUENCY: u32, const MAX_NOTES: usize>
    MidiTrack<'a, PLAY_FREQUENCY, MAX_NOTES>
{
    pub fn new(mut event_iter: midly::EventIter<'a>) -> Self {
        let last_event = event_iter.next();
        let next_event_time: u32 = if last_event.is_some() {
            last_event.as_ref().unwrap().as_ref().unwrap().delta.into()
        } else {
            0
        };
        let last_delta = 0;

        Self {
            event_iter,
            last_event,
            current_time: U32Fraction::<PLAY_FREQUENCY>::new(0, 0),
            next_event_time,
            last_delta,
        }
    }
    pub fn has_next(self: &Self) -> bool {
        return !self.last_event.is_none();
    }

    pub fn go_to_next_event(self: &mut Self) {
        if !self.has_next() {
            return;
        }
        self.last_event = self.event_iter.next();
    }

    pub fn update(
        self: &mut Self,
        notes: &mut AmpAdder<PLAY_FREQUENCY, MAX_NOTES>,
        channels: &mut Channels,
        tempo: &mut MidiTime<PLAY_FREQUENCY>,
    ) {
        if !self.has_next() {
            return;
        }
        while self.current_time.int_part >= self.next_event_time {
            let track_event = self.last_event.as_ref().unwrap().as_ref().unwrap();
            handle_track_event(&track_event, notes, channels, tempo);
            self.go_to_next_event();
            if !self.has_next() {
                return;
            }
            let new_track_event = self.last_event.as_ref().unwrap().as_ref().unwrap();
            let delta: u32 = new_track_event.delta.into();
            self.next_event_time = self.next_event_time + delta;
            self.last_delta = delta;
        }

        self.current_time.add(tempo.get_update_rate());
    }
}
