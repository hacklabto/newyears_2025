use crate::sound_sample::SoundSample;
//use crate::sound_sample::SoundSampleI32;
use crate::sound_source::SoundSource;
//use crate::sound_source_id::SoundSourceId;
//use crate::sound_source_msgs::SoundSourceAmpMixerInit;
//use crate::sound_source_msgs::SoundSourceKey;
use crate::sound_source_msgs::SoundSourceMsg;
use crate::sound_source_msgs::SoundSourceMsgs;
//use crate::sound_source_msgs::SoundSourceValue;
use crate::sound_sources::SoundSources;
use core::marker::PhantomData;
//use core::slice;
use midly::Smf;

#[allow(unused)]
pub struct MidiTrack<T: SoundSample, const PLAY_FREQUENCY: u32> {
    time: u32,
    next_event: u32,
    next_idx: usize,
    //track_iter: slice::Iter<'a, midly::TrackEvent<'a>>,
    //current_event: Option<&'a midly::TrackEvent<'a>>,
    _marker: PhantomData<T>,
}

impl<T: SoundSample, const PLAY_FREQUENCY: u32> MidiTrack<T, PLAY_FREQUENCY> {
    pub fn new() -> Self {
        let time: u32 = 0;
        let next_event: u32 = 0;
        let next_idx: usize = 0;
        Self {
            time,
            next_event,
            next_idx,
            _marker: PhantomData {},
        }
    }

    pub fn update<'a>(
        self: &mut Self,
        events: &'a midly::Track<'a>,
        _new_msgs: &mut SoundSourceMsgs,
    ) {
        let _current_event = events[self.next_idx];
    }

    /*
    pub fn new(track: midly::Track) -> Self {
        let time: u32 = 0;
        let mut track_iter = track.iter();
        let current_event = track_iter.next();
        let next_event: u32 = if current_event.is_some() {
            let delta_u32: u32 = current_event.unwrap().delta.into();
            time + delta_u32
        } else {
            0
        };
        Self {
            time,
            next_event,
            track_iter,
            current_event,
            _marker: PhantomData {},
        }
    }*/
}

pub struct MidiReal<'a, T: SoundSample, const PLAY_FREQUENCY: u32> {
    smf: Smf<'a>,
    track: MidiTrack<T, PLAY_FREQUENCY>,
    _marker: PhantomData<T>,
}

impl<T: SoundSample, const PLAY_FREQUENCY: u32> MidiReal<'_, T, PLAY_FREQUENCY> {
    pub fn new(midi_bytes: &'static [u8]) -> Self {
        let smf = midly::Smf::parse(midi_bytes)
            .expect("It's inlined data, so it better work, gosh darn it");
        let track = MidiTrack::new();
        Self {
            smf,
            track,
            _marker: PhantomData {},
        }
    }
}

#[allow(unused)]
impl<'a, T: SoundSample, const PLAY_FREQUENCY: u32> SoundSource<'a, T, PLAY_FREQUENCY>
    for MidiReal<'a, T, PLAY_FREQUENCY>
{
    fn get_next(self: &Self, all_sources: &dyn SoundSources<T, PLAY_FREQUENCY>) -> T {
        T::max()
    }

    fn has_next(self: &Self, all_sources: &dyn SoundSources<T, PLAY_FREQUENCY>) -> bool {
        true
    }

    fn update(&mut self, new_msgs: &mut SoundSourceMsgs) {
        self.track.update(&self.smf.tracks[0], new_msgs)
    }

    fn handle_msg(&mut self, msg: &SoundSourceMsg, new_msgs: &mut SoundSourceMsgs) {}
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
        Self { midi_maybe: None }
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

/*
 * This is kind of top level, does it make sense to create via message?
 */

/*
pub fn create_amp_mixer(
    all_pools: &mut dyn SoundSources<SoundSampleI32, 24000>,
    amp_mixer_properties: SoundSourceAmpMixerInit,
) -> SoundSourceId {
    let mut msgs = SoundSourceMsgs::default();
    msgs.append(SoundSourceMsg::new(
        all_pools.get_top_id(),
        all_pools.get_top_id(),
        SoundSourceKey::InitAmpMixer,
        SoundSourceValue::new_amp_mixer_init(amp_mixer_properties),
    ));
    all_pools.process_and_clear_msgs(&mut msgs);

    all_pools
        .get_last_created_sound_source()
        .expect("Id should have been recorded")
        .clone()
}
*/

#[cfg(test)]
mod tests {
    use crate::midi::Midi;
    use crate::sound_sample::SoundSampleI32;
    use crate::sound_sources_impl::SoundSourcesImpl;

    #[test]
    fn basic_midi_test() {
        let mut _all_pools = SoundSourcesImpl::<SoundSampleI32, 24000, 3, 3, 3>::default();

        let mut _test_idi =
            Midi::<SoundSampleI32, 24000>::new(include_bytes!("../assets/twinkle.mid"));
    }
}
