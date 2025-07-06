use crate::sound_source_id::SoundSourceId;
use crate::free_list::FreeListImpl;

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

#[allow(unused)]
pub struct SoundSourceMsg {
    dest_id: SoundSourceId,
    attribute: SoundSourceAttributes,
    value: usize 
}

#[allow(unused)]
impl Default for SoundSourceMsg {
    fn default() -> Self {
        let dest_id = SoundSourceId::default();
        let attribute = SoundSourceAttributes::Frequency;
        let value = 0;
        Self{ dest_id, attribute, value }
    }
}

#[allow(unused)]
pub struct SoundSourceMsgs<const N: usize> {
    messages: [SoundSourceMsg; N ],
    free_list: FreeListImpl<N>
}

//#[allow(unused)]
//impl Default for SoundSourceMsgs<const N: usize> {
//    fn default() -> Self {
//        let messages: [SoundSourceMsg; N] = default();
//    }
//}

