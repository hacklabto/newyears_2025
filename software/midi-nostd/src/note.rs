use crate::adsr::CoreAdsr;
use crate::adsr::SoundSourceAdsrInit;
use crate::amp_mixer::AmpMixerCore;
use crate::midi_notes::midi_note_to_freq;
use crate::oscillator::CoreOscillator;
use crate::oscillator::OscillatorType;
use crate::oscillator::SoundSourceOscillatorInit;
use crate::sound_sample::SoundSample;
use crate::sound_sample::SoundSampleI32;
use crate::sound_source::SoundSource;
use crate::sound_source_core::SoundSourceCore;
use crate::sound_source_id::SoundSourceId;
use crate::sound_source_msgs::SoundSourceMsg;
use crate::sound_source_msgs::SoundSourceMsgs;
use crate::sound_source_msgs::SoundSourceNoteInit;
use crate::sound_source_msgs::SoundSourceValue;
use crate::sound_sources::SoundSources;

//use core::marker::PhantomData;

type OscilatorAdsrCore<'a, T, const PLAY_FREQUENCY: u32> = AmpMixerCore<
    'a,
    T,
    PLAY_FREQUENCY,
    CoreOscillator<T, PLAY_FREQUENCY, 50, 100>,
    CoreAdsr<T, PLAY_FREQUENCY, 1200, 2400, 2400, 75, 50>,
>;

///
/// Note.  Now sort of a proof of concept.
///
pub struct Note<'a, T: SoundSample, const PLAY_FREQUENCY: u32> {
    core: OscilatorAdsrCore<'a, T, PLAY_FREQUENCY>,
}

impl<T: SoundSample, const PLAY_FREQUENCY: u32> Default for Note<'_, T, PLAY_FREQUENCY> {
    fn default() -> Self {
        Self {
            core: OscilatorAdsrCore::<T, PLAY_FREQUENCY>::default(),
        }
    }
}

impl<T: SoundSample, const PLAY_FREQUENCY: u32> SoundSource<'_, T, PLAY_FREQUENCY>
    for Note<'_, T, PLAY_FREQUENCY>
{
    fn get_next(self: &Self, _all_sources: &dyn SoundSources<T, PLAY_FREQUENCY>) -> T {
        self.core.get_next()
    }

    fn has_next(self: &Self, _all_sources: &dyn SoundSources<T, PLAY_FREQUENCY>) -> bool {
        self.core.has_next()
    }

    fn update(&mut self, _new_msgs: &mut SoundSourceMsgs) {
        self.core.update()
    }

    fn handle_msg(&mut self, msg: &SoundSourceMsg, new_msgs: &mut SoundSourceMsgs) {
        match &msg.value {
            SoundSourceValue::NoteInit { init_values } => {
                let frequency = midi_note_to_freq(init_values.key);

                let oscilator_init =
                    SoundSourceOscillatorInit::new(OscillatorType::Triangle, frequency);

                let adsr_init = SoundSourceAdsrInit::new();

                self.core.init(&(oscilator_init, adsr_init));

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
}

pub fn create_note(
    all_pools: &mut dyn SoundSources<SoundSampleI32, 24000>,
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
