use crate::amp_adder::AmpAdder;
use crate::midi_channels::Channels;
use crate::midi_events::*;
use crate::midi_time::MidiTime;

pub struct MidiTrack<
    'a,
    const P_FREQ: u32,
    const U_FREQ: u32,
    const MAX_NOTES: usize,
    const NO_SCALEDOWN: bool,
> {
    last_event: Option<Result<midly::TrackEvent<'a>, midly::Error>>,
    event_iter: midly::EventIter<'a>,
    next_event_time: u32,
}

impl<
        'a,
        const P_FREQ: u32,
        const U_FREQ: u32,
        const MAX_NOTES: usize,
        const NO_SCALEDOWN: bool,
    > MidiTrack<'a, P_FREQ, U_FREQ, MAX_NOTES, NO_SCALEDOWN>
{
    pub fn new(mut event_iter: midly::EventIter<'a>) -> Self {
        let last_event = event_iter.next();
        let next_event_time: u32 = if last_event.is_some() {
            last_event.as_ref().unwrap().as_ref().unwrap().delta.into()
        } else {
            0
        };

        Self {
            event_iter,
            last_event,
            next_event_time,
        }
    }
    #[inline]
    pub fn has_next(self: &Self) -> bool {
        return !self.last_event.is_none();
    }

    pub fn update(
        self: &mut Self,
        notes: &mut AmpAdder<P_FREQ, U_FREQ, MAX_NOTES, NO_SCALEDOWN>,
        channels: &mut Channels,
        tempo: &mut MidiTime<P_FREQ, U_FREQ>,
    ) {
        if !self.has_next() {
            return;
        }
        while tempo.get_current_time() >= self.next_event_time {
            let track_event = self.last_event.as_ref().unwrap().as_ref().unwrap();
            let end_of_track = handle_track_event(&track_event, notes, channels, tempo);

            if !end_of_track {
                self.last_event = self.event_iter.next();
            } else {
                self.last_event = None;
            }

            if !self.has_next() {
                return;
            }
            let new_track_event = self.last_event.as_ref().unwrap().as_ref().unwrap();
            let delta: u32 = new_track_event.delta.into();
            self.next_event_time = self.next_event_time + delta;
        }
    }
}
