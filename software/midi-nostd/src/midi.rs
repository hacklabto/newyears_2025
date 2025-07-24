use crate::amp_adder::AmpAdder;
use crate::midi_notes::midi_note_to_freq;
use crate::sound_sample::SoundSample;
use crate::sound_sample::SoundScale;
use crate::sound_source::SoundSource;
use crate::sound_source_id::SoundSourceId;
use crate::sound_source_msgs::OscillatorType;
use crate::sound_source_msgs::SoundSourceAdsrInit;
use crate::sound_source_msgs::SoundSourceAmpMixerInit;
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
    ignore_hack: u8,
    last_delta: u32,
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

    pub fn handle_midi_event(
        self: &mut Self,
        midi_event: &midly::MidiMessage,
        new_msgs: &mut SoundSourceMsgs,
        sound_id_playing: Option<SoundSourceId>,
    ) {
        match midi_event {
            midly::MidiMessage::NoteOn { key, vel: _ } => {
                let key_as_u32: u8 = (*key).into();
                if key_as_u32 != self.ignore_hack || self.last_delta != 0 {
                    self.ignore_hack = key_as_u32;
                    let frequency = midi_note_to_freq((*key).into());

                    let oscilator_init = SoundSourceOscillatorInit::new(
                        OscillatorType::Triangle,
                        frequency,
                        100,
                        100,
                    );
                    let adsr_init = SoundSourceAdsrInit::new(
                        SoundScale::new_percent(75),
                        SoundScale::new_percent(50),
                        1200,
                        2400,
                        2400,
                    );

                    let amp_mixer_init = SoundSourceAmpMixerInit::new(oscilator_init, adsr_init);
                    let amp_mixer_value = SoundSourceValue::AmpMixerInit {
                        init_values: amp_mixer_init,
                    };

                    new_msgs.append(SoundSourceMsg::new(
                        SoundSourceId::get_top_id(),
                        SoundSourceId::get_midi_id(),
                        amp_mixer_value,
                    ));
                }
            }
            midly::MidiMessage::NoteOff { key: _, vel: _ } => {
                self.ignore_hack = 0;
                if let Some(old_sound_id) = sound_id_playing {
                    new_msgs.append(SoundSourceMsg::new(
                        old_sound_id,
                        SoundSourceId::get_top_id(),
                        SoundSourceValue::ReleaseAdsr,
                    ));
                }
            }
            _ => {}
        }
    }

    pub fn handle_track_event(
        self: &mut Self,
        track_event: &midly::TrackEventKind,
        new_msgs: &mut SoundSourceMsgs,
        sound_id_playing: Option<SoundSourceId>,
    ) {
        match track_event {
            midly::TrackEventKind::Midi {
                message,
                channel: _,
            } => self.handle_midi_event(&message, new_msgs, sound_id_playing),
            _ => {}
        }
    }

    pub fn update<'a>(
        self: &mut Self,
        events: &'a midly::Track<'a>,
        new_msgs: &mut SoundSourceMsgs,
        sound_id_playing: Option<SoundSourceId>,
    ) {
        if !self.active {
            return;
        }
        while self.current_time == self.next_event_time {
            self.handle_track_event(
                &(events[self.current_event_idx]).kind,
                new_msgs,
                sound_id_playing,
            );
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
        if (self.current_remainder) & 15 == 0 {
            self.current_time = self.current_time + 1;
        }
    }
}

pub struct MidiReal<'a, T: SoundSample, const PLAY_FREQUENCY: u32> {
    smf: Smf<'a>,
    track: MidiTrack<T, PLAY_FREQUENCY>,
    amp_adder: AmpAdder<T, PLAY_FREQUENCY, 5>,
}

impl<T: SoundSample, const PLAY_FREQUENCY: u32> MidiReal<'_, T, PLAY_FREQUENCY> {
    pub fn new(midi_bytes: &'static [u8]) -> Self {
        let smf = midly::Smf::parse(midi_bytes)
            .expect("It's inlined data, so it better work, gosh darn it");
        let track = MidiTrack::new(&smf.tracks[0]);
        let amp_adder = AmpAdder::<T, PLAY_FREQUENCY, 5>::default();
        Self {
            smf,
            track,
            amp_adder,
        }
    }
}

#[allow(unused)]
impl<'a, T: SoundSample, const PLAY_FREQUENCY: u32> SoundSource<'a, T, PLAY_FREQUENCY>
    for MidiReal<'a, T, PLAY_FREQUENCY>
{
    fn get_next(self: &Self, all_sources: &dyn SoundSources<T, PLAY_FREQUENCY>) -> T {
        self.amp_adder.get_next(all_sources)
    }

    fn has_next(self: &Self, _all_sources: &dyn SoundSources<T, PLAY_FREQUENCY>) -> bool {
        self.track.has_next()
    }

    fn update(&mut self, new_msgs: &mut SoundSourceMsgs) {
        self.track
            .update(&self.smf.tracks[0], new_msgs, self.amp_adder.channels[0])
    }

    fn handle_msg(&mut self, msg: &SoundSourceMsg, new_msgs: &mut SoundSourceMsgs) {
        match &msg.value {
            SoundSourceValue::SoundSourceCreated => {
                for idx in (0..(self.amp_adder.channels.len() - 1)).rev() {
                    self.amp_adder.channels[idx + 1] = self.amp_adder.channels[idx];
                }
                self.amp_adder.channels[0] = Some(msg.src_id.clone());
            }
            _ => todo!(),
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
        let mut all_pools = SoundSourcesImpl::<SoundSampleI32, 24000, 3, 3>::default();
        let mut new_msgs = SoundSourceMsgs::default();
        let midi_id = all_pools.alloc(SoundSourceType::Midi);

        assert_eq!(0x8000, all_pools.get_next(&midi_id).to_u16());
        all_pools.update(&mut new_msgs);
        assert_eq!(0x8000, all_pools.get_next(&midi_id).to_u16());
        all_pools.update(&mut new_msgs);
        assert_eq!(0x8000 - 10, all_pools.get_next(&midi_id).to_u16());
        all_pools.update(&mut new_msgs);
        assert_eq!(0x8000 - 19, all_pools.get_next(&midi_id).to_u16());
        all_pools.update(&mut new_msgs);
    }
}
