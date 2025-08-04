use crate::amp_adder::AmpAdder;
use crate::midi_channels::Channel;
use crate::midi_channels::Channels;
use crate::midi_time::MidiTime;
use crate::note::SoundSourceNoteInit;

pub fn handle_midi_event<
    const P_FREQ: u32,
    const U_FREQ: u32,
    const MAX_NOTES: usize,
    const NO_SCALEDOWN: bool,
>(
    midi_event: &midly::MidiMessage,
    channel_u8: u8,
    notes: &mut AmpAdder<P_FREQ, U_FREQ, MAX_NOTES, NO_SCALEDOWN>,
    channels: &mut Channels,
    program_override: i32,
) {
    let channel: usize = channel_u8 as usize;
    match midi_event {
        midly::MidiMessage::NoteOn { key, vel } => {
            let key_as_u32: u8 = (*key).into();

            if *vel == 0 {
                let playing_note = channels.channels[channel].playing_notes[key_as_u32 as usize];

                if playing_note != Channel::UNUSED {
                    notes.trigger_note_off_at(playing_note as usize);
                    channels.channels[channel].playing_notes[key_as_u32 as usize] = Channel::UNUSED;
                }
            } else {
                let mut instrument: u8 = channels.channels[channel].current_program;
                if program_override != -1 {
                    instrument = program_override as u8;
                }

                let note_init = SoundSourceNoteInit::new((*key).into(), instrument, (*vel).into());
                let playing_note_u8 = channels.channels[channel].playing_notes[key_as_u32 as usize];
                if playing_note_u8 != Channel::UNUSED {
                    let playing_note = playing_note_u8 as usize;
                    notes.restart_note_at(playing_note, note_init.velocity);
                } else {
                    let new_note = notes.alloc();
                    notes.new_note_at(new_note, note_init);
                    channels.channels[channel].playing_notes[key_as_u32 as usize] = new_note as u8;
                }
            }
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

pub fn handle_track_event<
    const P_FREQ: u32,
    const U_FREQ: u32,
    const MAX_NOTES: usize,
    const NO_SCALEDOWN: bool,
>(
    track_event: &midly::TrackEvent,
    notes: &mut AmpAdder<P_FREQ, U_FREQ, MAX_NOTES, NO_SCALEDOWN>,
    channels: &mut Channels,
    tempo: &mut MidiTime<P_FREQ, U_FREQ>,
    program_override: i32,
) -> bool {
    match track_event.kind {
        midly::TrackEventKind::Midi { message, channel } => {
            if channel == 10 {
                return true;
            }
            handle_midi_event(&message, channel.into(), notes, channels, program_override)
        }
        midly::TrackEventKind::Meta(message) => match message {
            midly::MetaMessage::Tempo(ms_per_qn_midly) => {
                let ms_per_qn: u32 = ms_per_qn_midly.into();
                tempo.set_ms_per_quarter_note(ms_per_qn as u32);
            }
            midly::MetaMessage::EndOfTrack => {
                return true;
            }
            _ => {}
        },
        _ => {}
    }
    false
}
