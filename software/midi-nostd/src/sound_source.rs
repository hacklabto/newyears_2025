use crate::sound_sample::SoundSample;
//use core::marker::PhantomData;

/// Different types source sources
///
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(usize)]
#[allow(unused)]
pub enum SoundSourceType {
    WaveGenerator = 0,
    AdsrEnvelope = 1,
}

#[allow(unused)]
impl SoundSourceType {
    pub fn from_usize(usize_value: usize) -> Self {
        let optional_enum_value: Option<Self> = match usize_value {
            0 => Some(SoundSourceType::WaveGenerator),
            1 => Some(SoundSourceType::AdsrEnvelope),
            _ => None,
        };
        optional_enum_value.expect("bad usize to SoundSourceType")
    }
    pub const fn all_variants() -> &'static [SoundSourceType] {
        &[
            SoundSourceType::WaveGenerator,
            SoundSourceType::AdsrEnvelope,
        ]
    }
    pub const fn max_variant_id() -> usize {
        let mut max_variant_id: Option<usize> = None;
        let slice = SoundSourceType::all_variants();
        let mut idx = 0;

        while idx < slice.len() {
            let enum_value = slice[idx];
            let usize_value = enum_value as usize;
            max_variant_id = if max_variant_id.is_none() {
                Some(usize_value)
            } else {
                if usize_value > max_variant_id.expect("") {
                    Some(usize_value)
                } else {
                    max_variant_id
                }
            };
            idx = idx + 1;
        }
        max_variant_id.expect("ENUM had no values!?!?")
    }
}

#[cfg(test)]
mod tests {
    use crate::sound_source::*;

    #[test]
    fn sound_source_enum_and_usize_bindings_are_consistent() {
        for enum_value in SoundSourceType::all_variants().iter().copied() {
            let usize_value = enum_value as usize;
            let enum_value_for_check = SoundSourceType::from_usize(usize_value);
            assert_eq!(enum_value, enum_value_for_check);
        }
    }

    #[test]
    // Each enum value should have a single usize map
    fn sound_source_enum_and_usize_bindings_are_sensible() {
        const MAX_ENUM_MAP: usize = SoundSourceType::max_variant_id() + 1;
        let mut times_seen: [u32; MAX_ENUM_MAP] = [0; MAX_ENUM_MAP];
        for enum_value in SoundSourceType::all_variants().iter().copied() {
            let usize_value = enum_value as usize;
            times_seen[usize_value] = times_seen[usize_value] + 1;
        }
        for times_element_was_seen in times_seen {
            assert_eq!(1, times_element_was_seen);
        }
    }
}

#[allow(unused)]
pub struct SoundSourceId {
    pub source_type: SoundSourceType,
    pub id: usize,
}

///
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[allow(unused)]
pub enum SoundSourceAttributes {
    WaveType,
    Frequency,
    Volume,
}

/// Start with just square waves
///
#[derive(Clone, Copy)]
#[allow(unused)]
#[repr(usize)]
pub enum WaveType {
    Square = 0,
    Triangle = 1,
    SawTooth = 2,
    Sine = 3,
}

#[allow(unused)]
impl SoundSourceId {
    pub fn new(source_type: SoundSourceType, id: usize) -> Self {
        Self { source_type, id }
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
