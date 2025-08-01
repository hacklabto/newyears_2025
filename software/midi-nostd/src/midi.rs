use crate::amp_adder::AmpAdder;
use crate::midi_time::MidiTime;
use crate::midi_track::Channels;
use crate::midi_track::MidiTrack;
use crate::sound_sample::SoundSampleI32;
use crate::sound_source_core::SoundSourceCore;
use midly::Smf;
use midly::Timing;

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
