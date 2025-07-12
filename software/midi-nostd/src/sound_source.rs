use crate::sound_sample::SoundSample;
use crate::sound_source_id::SoundSourceId;
use crate::sound_source_msgs::SoundSourceKey;
use crate::sound_source_msgs::SoundSourceMsgs;
use crate::sound_source_msgs::SoundSourceValue;
use crate::sound_sources::SoundSources;
//use core::marker::PhantomData;

///
/// Interface (so far) for a sound source  
///
/// A sound source is simply a source of sound.  The caller gets sound samples through
/// the get_next method.  This interface is abstract - an actual sound source may be
/// something like a waveform generator (i.e., sine or square waves) or may be something
/// more complicated
///
/// One idea is that we should be able to chain sound sources together.  For example,
/// a note might be created by  taking a waveform at the note's frequency and modifying
/// it using an ADSR amplitude envelope.
///
pub trait SoundSource<SAMPLE: SoundSample, const PLAY_FREQUENCY: u32> {
    /// Returns false if the sound source is done playing
    ///
    fn has_next(self: &Self, all_sources: &dyn SoundSources<SAMPLE, PLAY_FREQUENCY>) -> bool;

    /// Draw a sample from a source
    ///
    fn get_next(self: &Self, all_sources: &dyn SoundSources<SAMPLE, PLAY_FREQUENCY>) -> SAMPLE;

    /// Update the state one tick
    fn update(self: &mut Self, new_msgs: &mut SoundSourceMsgs);

    /// Handle a message
    fn handle_msg(
        self: &mut Self,
        origin: &SoundSourceId,
        key: SoundSourceKey,
        value: SoundSourceValue,
    );
}
