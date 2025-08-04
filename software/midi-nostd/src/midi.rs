use crate::amp_adder::AmpAdder;
use crate::midi_channels::Channels;
use crate::midi_time::MidiTime;
use crate::midi_track::MidiTrack;
use crate::sound_sample::SoundSampleI32;
use crate::sound_source_core::SoundSourceCore;
use midly::Timing;

pub struct Midi<
    'a,
    const P_FREQ: u32,
    const U_FREQ: u32,
    const MAX_NOTES: usize,
    const MAX_TRACKS: usize,
    const NO_SCALEDOWN: bool = false,
> {
    num_tracks: usize,
    tracks: [Option<MidiTrack<'a, P_FREQ, U_FREQ, MAX_NOTES, NO_SCALEDOWN>>; MAX_TRACKS],
    amp_adder: AmpAdder<P_FREQ, U_FREQ, MAX_NOTES, NO_SCALEDOWN>,
    channels: Channels,
    tempo: MidiTime<P_FREQ, U_FREQ>,
    skip_count: u32,
    tracks_still_playing: bool,
}

impl<
        'a,
        const P_FREQ: u32,
        const U_FREQ: u32,
        const MAX_NOTES: usize,
        const MAX_TRACKS: usize,
        const NO_SCALEDOWN: bool,
    > Midi<'a, P_FREQ, U_FREQ, MAX_NOTES, MAX_TRACKS, NO_SCALEDOWN>
{
    const SKIP: u32 = P_FREQ / U_FREQ;

    pub fn new_internal(
        header: &midly::Header,
        mut track_iter: midly::TrackIter<'a>,
        divider: i32,
    ) -> Self {
        assert_eq!(0, (P_FREQ % U_FREQ));
        let num_tracks = track_iter.clone().count();
        let tracks: [Option<MidiTrack<P_FREQ, U_FREQ, MAX_NOTES, NO_SCALEDOWN>>; MAX_TRACKS] =
            core::array::from_fn(|_idx| {
                let track = track_iter.next();

                if track.is_none() {
                    None
                } else {
                    Some(MidiTrack::<P_FREQ, U_FREQ, MAX_NOTES, NO_SCALEDOWN>::new(
                        track.unwrap().unwrap(),
                    ))
                }
            });

        let amp_adder = AmpAdder::<P_FREQ, U_FREQ, MAX_NOTES, NO_SCALEDOWN>::new(divider);

        let tpqn_midly = match header.timing {
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
            skip_count: 0,
            tracks_still_playing: true,
        }
    }

    pub fn get_loudest_sample(header: &midly::Header, track_iter: midly::TrackIter<'a>) -> i32 {
        let mut fast_forward_midi_player =
            Midi::<240, 240, MAX_NOTES, MAX_TRACKS, true>::new_internal(
                header,
                track_iter.clone(),
                0, // not used
            );
        let mut loudest: i32 = 0;
        while fast_forward_midi_player.has_next() {
            let sample = fast_forward_midi_player.get_next().to_i32();
            let abs_sample = if sample < 0 { -sample } else { sample };
            loudest = if abs_sample > loudest {
                abs_sample
            } else {
                loudest
            };
        }
        loudest
    }

    pub fn new(header: &midly::Header, track_iter: midly::TrackIter<'a>) -> Self {
        //
        // Limit to 255 (not 256) notes to save space in the midi data
        // structure.  On embedded platforms memory is often limited
        //
        assert!(MAX_NOTES < 0xff);
        let loudest = Self::get_loudest_sample(header, track_iter.clone());
        Midi::<P_FREQ, U_FREQ, MAX_NOTES, MAX_TRACKS, NO_SCALEDOWN>::new_internal(
            header,
            track_iter.clone(),
            loudest / 0x8000 + 1,
        )
    }

    pub fn get_current_num_mixed_notes(self: &mut Self) -> u32 {
        self.amp_adder.get_current_num_mixed_notes()
    }

    pub fn update(self: &mut Self) {
        self.tempo.advance_time();
        for i in 0..self.num_tracks {
            if self.tracks[i].is_some() {
                self.tracks[i].as_mut().unwrap().update(
                    &mut self.amp_adder,
                    &mut self.channels,
                    &mut self.tempo,
                );
            }
        }
        self.amp_adder.update();
        self.tracks_still_playing = false;
        for i in 0..self.num_tracks {
            if self.tracks[i].is_some() {
                if self.tracks[i].as_ref().unwrap().has_next() {
                    self.tracks_still_playing = true;
                }
            }
        }
    }

    pub fn get_next(self: &mut Self) -> SoundSampleI32 {
        if self.skip_count == 0 {
            self.update();
        }
        self.skip_count = self.skip_count + 1;
        if self.skip_count == Self::SKIP {
            self.skip_count = 0;
        }
        self.amp_adder.get_next()
    }

    pub fn has_next(self: &Self) -> bool {
        self.tracks_still_playing
    }
}

#[cfg(test)]
mod tests {

    use crate::midi::Midi;

    #[test]
    fn basic_midi_test() {
        let (header, tracks) = midly::parse(include_bytes!("../assets/twinkle.mid"))
            .expect("It's inlined data, so it better work, gosh darn it");
        let mut midi = Midi::<24000, 24000, 32, 16>::new_internal(&header, tracks, 1);

        assert_eq!(118, midi.get_next().to_i32());
        //assert_eq!(8192, midi.get_next(&smf).to_i32());
        //assert_eq!(8719, midi.get_next(&smf).to_i32());
        //assert_eq!(9246, midi.get_next(&smf).to_i32());
    }
}
