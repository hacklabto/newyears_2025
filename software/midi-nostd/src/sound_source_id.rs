/// Different types source sources
///
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(usize)]
pub enum SoundSourceType {
    Top = 0,
    AmpMixer = 1,
    Midi = 2,
}

impl SoundSourceType {
    pub fn from_usize(usize_value: usize) -> Self {
        let optional_enum_value: Option<Self> = match usize_value {
            0 => Some(SoundSourceType::Top),
            1 => Some(SoundSourceType::AmpMixer),
            2 => Some(SoundSourceType::Midi),
            _ => None,
        };
        optional_enum_value.expect("bad usize to SoundSourceType")
    }
    pub const fn all_variants() -> &'static [SoundSourceType] {
        &[
            SoundSourceType::Top,
            SoundSourceType::AmpMixer,
            SoundSourceType::Midi,
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
    use crate::sound_source_id::*;

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

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct SoundSourceRealId {
    pub source_type: SoundSourceType,
    pub id: usize,
}

impl SoundSourceRealId {
    pub fn new(source_type: SoundSourceType, id: usize) -> Self {
        Self { source_type, id }
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum SoundSourceId {
    Uninitialized,
    UseMsgDefault,
    Real { real: SoundSourceRealId },
}

impl Default for SoundSourceId {
    fn default() -> Self {
        return SoundSourceId::Uninitialized;
    }
}

impl SoundSourceId {
    pub fn new(source_type: SoundSourceType, id: usize) -> Self {
        SoundSourceId::Real {
            real: SoundSourceRealId::new(source_type, id),
        }
    }
    pub fn source_type(self: Self) -> SoundSourceType {
        match self {
            SoundSourceId::Real { real } => real.source_type,
            _ => panic!("ID only exists if the enum is real"),
        }
    }
    pub fn id(self: Self) -> usize {
        match self {
            SoundSourceId::Real { real } => real.id,
            _ => panic!("ID only exists if the enum is real"),
        }
    }
    pub fn get_top_id() -> Self {
        Self::new(SoundSourceType::Top, 0)
    }
    pub fn get_midi_id() -> Self {
        Self::new(SoundSourceType::Midi, 0)
    }
}
