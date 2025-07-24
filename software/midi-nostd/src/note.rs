use crate::adsr::CoreAdsr;
use crate::adsr::SoundSourceAdsrInit;
use crate::amp_mixer::AmpMixerCore;
use crate::midi_notes::midi_note_to_freq;
use crate::oscillator::CoreOscillator;
use crate::oscillator::OscillatorType;
use crate::oscillator::SoundSourceOscillatorInit;
use crate::sound_sample::SoundSampleI32;
use crate::sound_source_core::SoundSourceCore;
use crate::sound_source_msgs::SoundSourceNoteInit;

//use core::marker::PhantomData;

type OscilatorAdsrCore<const PLAY_FREQUENCY: u32> = AmpMixerCore<
    PLAY_FREQUENCY,
    CoreOscillator<PLAY_FREQUENCY, 50, 100, { OscillatorType::Triangle as usize }>,
    CoreAdsr<PLAY_FREQUENCY, 1200, 2400, 2400, 75, 50>,
>;

///
/// Note.  Now sort of a proof of concept.
///
pub struct Note<const PLAY_FREQUENCY: u32> {
    core: OscilatorAdsrCore<PLAY_FREQUENCY>,
}

impl<const PLAY_FREQUENCY: u32> Default for Note<PLAY_FREQUENCY> {
    fn default() -> Self {
        Self {
            core: OscilatorAdsrCore::<PLAY_FREQUENCY>::default(),
        }
    }
}

impl<const PLAY_FREQUENCY: u32> SoundSourceCore<PLAY_FREQUENCY> for Note<PLAY_FREQUENCY> {
    type InitValuesType = SoundSourceNoteInit;

    fn get_next(self: &mut Self) -> SoundSampleI32 {
        self.core.get_next()
    }

    fn has_next(self: &Self) -> bool {
        self.core.has_next()
    }

    fn init(&mut self, init_values: &Self::InitValuesType) {
        let frequency = midi_note_to_freq(init_values.key);
        let oscilator_init = SoundSourceOscillatorInit::new(frequency);
        let adsr_init = SoundSourceAdsrInit::new();
        self.core.init(&(oscilator_init, adsr_init));
    }

    fn trigger_note_off(self: &mut Self) {
        self.core.trigger_note_off();
    }

    /*
        fn handle_msg(&mut self, msg: &SoundSourceMsg, new_msgs: &mut SoundSourceMsgs) {
            match &msg.value {
                SoundSourceValue::NoteInit { init_values } => {

                    let creation_msg = SoundSourceMsg::new(
                        msg.src_id.clone(),
                        msg.dest_id.clone(),
                        SoundSourceValue::SoundSourceCreated,
                    );
                    new_msgs.append(creation_msg);
                }
                SoundSourceValue::ReleaseAdsr => {
                    // TODO, What if we aren't in sustain?  Probably I should take
                    // the current volume and run the release on that.
                    self.core.trigger_note_off();
                }
                _ => todo!(),
            }
        }
    */
}

/*
pub fn create_note(
    all_pools: &mut dyn SoundSources<24000>,
    init_values: SoundSourceNoteInit,
) -> SoundSourceId {
    let mut msgs = SoundSourceMsgs::default();
    msgs.append(SoundSourceMsg::new(
        SoundSourceId::get_top_id(),
        SoundSourceId::get_top_id(),
        SoundSourceValue::NoteInit { init_values },
    ));
    all_pools.process_and_clear_msgs(&mut msgs);

    all_pools
        .get_last_created_sound_source()
        .expect("Id should have been recorded")
        .clone()
}
*/
