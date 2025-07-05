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
