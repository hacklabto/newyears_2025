use crate::amp_adder::AmpAdder;
use crate::midi_time::MidiTime;
use crate::midi_track::Channel;
use crate::midi_track::Channels;
use crate::note::SoundSourceNoteInit;

pub fn handle_midi_event<const PLAY_FREQUENCY: u32, const MAX_NOTES: usize>(
    midi_event: &midly::MidiMessage,
    channel_u8: u8,
    notes: &mut AmpAdder<PLAY_FREQUENCY, MAX_NOTES>,
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
            let playing_note = channels.channels[channel].playing_notes[key_as_u32 as usize];
            let dst = if playing_note != Channel::UNUSED {
                playing_note as usize
            } else {
                notes.alloc()
            };
            assert!(dst < (Channel::UNUSED as usize));

            notes.new_note_at(dst, note_init);
            channels.channels[channel].playing_notes[key_as_u32 as usize] = dst as u8;
        }
        midly::MidiMessage::NoteOff { key, vel: _ } => {
            let key_as_u32: u8 = (*key).into();
            let playing_note = channels.channels[channel].playing_notes[key_as_u32 as usize];

            if playing_note != Channel::UNUSED {
                notes.trigger_note_off_at(playing_note as usize);
                channels.channels[channel].playing_notes[key_as_u32 as usize] = Channel::UNUSED;
            }
        }
        midly::MidiMessage::ProgramChange { program } => {
            channels.channels[channel].current_program = (*program).into();
        }
        _ => {}
    }
}

pub fn handle_track_event<'a, const PLAY_FREQUENCY: u32, const MAX_NOTES: usize>(
    track_event: &midly::TrackEventKind,
    notes: &mut AmpAdder<PLAY_FREQUENCY, MAX_NOTES>,
    channels: &mut Channels,
    tempo: &mut MidiTime<PLAY_FREQUENCY>,
) {
    match track_event {
        midly::TrackEventKind::Midi { message, channel } => {
            handle_midi_event(&message, (*channel).into(), notes, channels)
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
