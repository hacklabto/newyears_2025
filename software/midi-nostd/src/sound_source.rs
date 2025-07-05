use crate::sound_sample::SoundSample;
use crate::sound_source_id::SoundSourceId;
//use core::marker::PhantomData;

///
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[allow(unused)]
pub enum SoundSourceAttributes {
    WaveType,
    Frequency,
    Volume,
    PulseWidth,
}

/// Different Wave Types
///
#[derive(Clone, Copy, PartialEq)]
#[allow(unused)]
#[repr(usize)]
pub enum WaveType {
    Triangle = 0,
    SawTooth = 1,
    Sine = 2,
    PulseWidth = 3,
}

impl WaveType {
    pub fn from_usize(usize_value: usize) -> Self {
        let optional_enum_value: Option<Self> = match usize_value {
            0 => Some(WaveType::Triangle),
            1 => Some(WaveType::SawTooth),
            2 => Some(WaveType::Sine),
            3 => Some(WaveType::PulseWidth),
            _ => None,
        };
        let enum_value = optional_enum_value.expect("bad usize  aveType");
        assert_eq!(usize_value, enum_value as usize); // cheap sanity check
        enum_value
    }
}

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
#[allow(unused)]
pub trait SoundSource<SAMPLE: SoundSample, const PLAY_FREQUENCY: u32> {
    /// Returns false if the sound source is done playing
    ///
    fn has_next(self: &Self) -> bool;

    /// Draw a sample from a source
    ///
    fn get_next(self: &mut Self) -> SAMPLE;

    /// Set Attribute
    fn set_attribute(self: &mut Self, key: SoundSourceAttributes, value: usize);

    fn peer_sound_source(self: &Self) -> Option<SoundSourceId>;
    fn child_sound_source(self: &Self) -> Option<SoundSourceId>;
}
