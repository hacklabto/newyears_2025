use crate::midi_notes::midi_note_to_freq;
use crate::sound_sample::SoundSample;
use crate::sound_source::SoundSource;
use crate::sound_source_id::SoundSourceId;
use crate::sound_source_msgs::OscillatorType;
use crate::sound_source_msgs::SoundSourceKey;
use crate::sound_source_msgs::SoundSourceMsg;
use crate::sound_source_msgs::SoundSourceMsgs;
use crate::sound_source_msgs::SoundSourceOscillatorInit;
use crate::sound_source_msgs::SoundSourceValue;
use crate::sound_sources::SoundSources;
use core::marker::PhantomData;
//use core::slice;
use midly::Smf;

#[allow(unused)]
pub struct MidiTrack<T: SoundSample, const PLAY_FREQUENCY: u32> {
    active: bool,
    current_event_idx: usize,
    current_time: u32,
    current_remainder: u32,
    next_event_time: u32,
    _marker: PhantomData<T>,
}

impl<T: SoundSample, const PLAY_FREQUENCY: u32> MidiTrack<T, PLAY_FREQUENCY> {
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

        Self {
            active,
            current_event_idx,
            current_time,
            current_remainder,
            next_event_time,
            _marker: PhantomData {},
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

    pub fn handle_midi_event(midi_event: &midly::MidiMessage, new_msgs: &mut SoundSourceMsgs) {
        match midi_event {
            midly::MidiMessage::NoteOn { key, vel: _ } => {
                let frequency = midi_note_to_freq((*key).into());

                let oscilator_properties =
                    SoundSourceOscillatorInit::new(OscillatorType::Triangle, frequency, 100, 100);
                new_msgs.append(SoundSourceMsg::new(
                    SoundSourceId::get_top_id(),
                    SoundSourceId::get_midi_id(),
                    SoundSourceKey::InitOscillator,
                    SoundSourceValue::new_oscillator_init(oscilator_properties),
                ));
            }
            midly::MidiMessage::NoteOff { key: _, vel: _ } => {}
            _ => {}
        }
    }

    pub fn handle_track_event(track_event: &midly::TrackEventKind, new_msgs: &mut SoundSourceMsgs) {
        match track_event {
            midly::TrackEventKind::Midi {
                message,
                channel: _,
            } => Self::handle_midi_event(&message, new_msgs),
            _ => {}
        }
    }

    pub fn update<'a>(
        self: &mut Self,
        events: &'a midly::Track<'a>,
        new_msgs: &mut SoundSourceMsgs,
    ) {
        if !self.active {
            return;
        }
        while self.current_time == self.next_event_time {
            Self::handle_track_event(&(events[self.current_event_idx]).kind, new_msgs);
            self.go_to_next_event(events);
            if !self.active {
                return;
            }
            let delta: u32 = events[self.current_event_idx].delta.into();
            self.next_event_time = self.current_time + delta;
        }
        self.current_remainder = self.current_remainder + 1;
        // TODO, adjust properly.
        if (self.current_remainder) & 15 == 0 {
            self.current_time = self.current_time + 1;
        }
    }
}

pub struct MidiReal<'a, T: SoundSample, const PLAY_FREQUENCY: u32> {
    smf: Smf<'a>,
    track: MidiTrack<T, PLAY_FREQUENCY>,
    note_0: Option<SoundSourceId>,
    note_1: Option<SoundSourceId>,
    _marker: PhantomData<T>,
}

impl<T: SoundSample, const PLAY_FREQUENCY: u32> MidiReal<'_, T, PLAY_FREQUENCY> {
    pub fn new(midi_bytes: &'static [u8]) -> Self {
        let smf = midly::Smf::parse(midi_bytes)
            .expect("It's inlined data, so it better work, gosh darn it");
        let track = MidiTrack::new(&smf.tracks[0]);
        Self {
            smf,
            track,
            note_0: None,
            note_1: None,
            _marker: PhantomData {},
        }
    }
}

#[allow(unused)]
impl<'a, T: SoundSample, const PLAY_FREQUENCY: u32> SoundSource<'a, T, PLAY_FREQUENCY>
    for MidiReal<'a, T, PLAY_FREQUENCY>
{
    fn get_next(self: &Self, all_sources: &dyn SoundSources<T, PLAY_FREQUENCY>) -> T {
        if self.note_0.is_some() {
            all_sources.get_next(&self.note_0.unwrap())
        } else if self.note_1.is_some() {
            all_sources.get_next(&self.note_1.unwrap())
        } else {
            T::new(0x8000)
        }
    }

    fn has_next(self: &Self, _all_sources: &dyn SoundSources<T, PLAY_FREQUENCY>) -> bool {
        self.track.has_next()
    }

    fn update(&mut self, new_msgs: &mut SoundSourceMsgs) {
        self.track.update(&self.smf.tracks[0], new_msgs)
    }

    fn handle_msg(&mut self, msg: &SoundSourceMsg, new_msgs: &mut SoundSourceMsgs) {
        if msg.key == SoundSourceKey::SoundSourceCreated {
            //panic!("TODO - This message needs to go off and isn't");
            self.note_0 = Some(msg.src_id.clone());
        }
    }
}

///
/// Midi Playback
///
#[allow(unused)]
pub struct Midi<'a, T: SoundSample, const PLAY_FREQUENCY: u32> {
    midi_maybe: Option<MidiReal<'a, T, PLAY_FREQUENCY>>,
}

impl<T: SoundSample, const PLAY_FREQUENCY: u32> Default for Midi<'_, T, PLAY_FREQUENCY> {
    fn default() -> Self {
        let midi = MidiReal::<T, PLAY_FREQUENCY>::new(include_bytes!("../assets/twinkle.mid"));
        Self {
            midi_maybe: Some(midi),
        }
    }
}

impl<T: SoundSample, const PLAY_FREQUENCY: u32> Midi<'_, T, PLAY_FREQUENCY> {
    pub fn new(midi_bytes: &'static [u8]) -> Self {
        Self {
            midi_maybe: Some(MidiReal::new(midi_bytes)),
        }
    }
}

#[allow(unused)]
impl<'a, T: SoundSample, const PLAY_FREQUENCY: u32> SoundSource<'a, T, PLAY_FREQUENCY>
    for Midi<'a, T, PLAY_FREQUENCY>
{
    fn get_next(self: &Self, all_sources: &dyn SoundSources<T, PLAY_FREQUENCY>) -> T {
        self.midi_maybe.as_ref().unwrap().get_next(all_sources)
    }

    fn has_next(self: &Self, all_sources: &dyn SoundSources<T, PLAY_FREQUENCY>) -> bool {
        self.midi_maybe.as_ref().unwrap().has_next(all_sources)
    }

    fn update(&mut self, new_msgs: &mut SoundSourceMsgs) {
        self.midi_maybe.as_mut().unwrap().update(new_msgs)
    }

    fn handle_msg(&mut self, msg: &SoundSourceMsg, new_msgs: &mut SoundSourceMsgs) {
        self.midi_maybe.as_mut().unwrap().handle_msg(msg, new_msgs);
    }
}

#[cfg(test)]
mod tests {
    use crate::sound_sample::SoundSample;
    use crate::sound_sample::SoundSampleI32;
    use crate::sound_source_id::SoundSourceType;
    use crate::sound_source_msgs::SoundSourceMsgs;
    use crate::sound_sources::SoundSources;
    use crate::sound_sources_impl::SoundSourcesImpl;

    #[test]
    fn basic_midi_test() {
        let mut all_pools = SoundSourcesImpl::<SoundSampleI32, 24000, 3, 3, 3>::default();
        let mut new_msgs = SoundSourceMsgs::default();
        let midi_id = all_pools.alloc(SoundSourceType::Midi);

        assert_eq!(0x8000, all_pools.get_next(&midi_id).to_u16());
        all_pools.update(&mut new_msgs);
        assert_eq!(1, all_pools.get_next(&midi_id).to_u16());
        all_pools.update(&mut new_msgs);
        assert_eq!(1408, all_pools.get_next(&midi_id).to_u16());
        all_pools.update(&mut new_msgs);
        assert_eq!(2816, all_pools.get_next(&midi_id).to_u16());
        all_pools.update(&mut new_msgs);
    }
}
