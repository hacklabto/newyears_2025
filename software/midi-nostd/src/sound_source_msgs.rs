use crate::sound_source_id::SoundSourceId;

///
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SoundSourceKey {
    WaveType,
    Frequency,
    Volume,
    PulseWidth,
}

/// Different Wave Types
///
#[derive(Clone, Copy, PartialEq, Debug)]
#[repr(usize)]
pub enum WaveType {
    Triangle = 0,
    SawTooth = 1,
    Sine = 2,
    PulseWidth = 3,
}

impl WaveType {
    // TODO, delete.
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

#[derive(Clone, PartialEq, Debug)]
pub enum SoundSourceValue {
    Uninitialized,
    WaveType { wave_type: WaveType },
    U32Type { num: u32 },
    U8Type { num: u8 },
}

impl SoundSourceValue {
    pub fn new_u32(num: u32) -> Self {
        SoundSourceValue::U32Type { num }
    }
    pub fn new_u8(num: u8) -> Self {
        SoundSourceValue::U8Type { num }
    }
    pub fn new_wave_type(wave_type: WaveType) -> Self {
        SoundSourceValue::WaveType { wave_type }
    }
    pub fn get_u32(self: &Self) -> u32 {
        match self {
            SoundSourceValue::U32Type { num } => *num,
            _ => panic!("This isn't a u32"),
        }
    }
    pub fn get_u8(self: &Self) -> u8 {
        match self {
            SoundSourceValue::U8Type { num } => *num,
            _ => panic!("This isn't a u8"),
        }
    }
    pub fn get_wave_type(self: &Self) -> WaveType {
        match self {
            SoundSourceValue::WaveType { wave_type } => *wave_type,
            _ => panic!("This isn't a wave type"),
        }
    }
}

impl Default for SoundSourceValue {
    fn default() -> Self {
        SoundSourceValue::Uninitialized
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct SoundSourceMsg {
    pub dest_id: SoundSourceId,
    pub attribute: SoundSourceKey,
    pub value: SoundSourceValue,
}

impl Default for SoundSourceMsg {
    fn default() -> Self {
        let dest_id = SoundSourceId::default();
        let attribute = SoundSourceKey::Frequency;
        let value = SoundSourceValue::default();
        Self {
            dest_id,
            attribute,
            value,
        }
    }
}

impl SoundSourceMsg {
    pub fn new(dest_id: SoundSourceId, attribute: SoundSourceKey, value: SoundSourceValue) -> Self {
        return Self {
            dest_id,
            attribute,
            value,
        };
    }
}

pub struct SoundSourceMsgPool<const N: usize> {
    messages: [SoundSourceMsg; N],
    last: usize,
}

impl<const N: usize> Default for SoundSourceMsgPool<N> {
    fn default() -> Self {
        let messages: [SoundSourceMsg; N] = core::array::from_fn(|_i| SoundSourceMsg::default());
        let last: usize = 0;
        return Self { messages, last };
    }
}

impl<const N: usize> SoundSourceMsgPool<N> {
    pub fn append(self: &mut Self, msg: SoundSourceMsg) {
        assert!(self.last != N); // mostly for clarity, rust will check anyway
        self.messages[self.last] = msg;
        self.last = self.last + 1;
    }
    pub fn clear(self: &mut Self) {
        self.last = 0;
    }
    pub fn get_msgs(self: &Self) -> &[SoundSourceMsg] {
        &(self.messages[0..self.last])
    }
}

pub type SoundSourceMsgs = SoundSourceMsgPool<100>;

#[cfg(test)]
mod tests {
    use crate::sound_source_id::SoundSourceId;
    use crate::sound_source_id::SoundSourceType;
    use crate::sound_source_msgs::SoundSourceKey;
    use crate::sound_source_msgs::SoundSourceMsg;
    use crate::sound_source_msgs::SoundSourceMsgs;
    use crate::sound_source_msgs::SoundSourceValue;

    #[test]
    fn messages_should_work() {
        let mut messages = SoundSourceMsgs::default();
        assert_eq!(0, messages.get_msgs().len());

        let m0 = SoundSourceMsg::new(
            SoundSourceId::new(SoundSourceType::WaveGenerator, 5),
            SoundSourceKey::Frequency,
            SoundSourceValue::new_u32(2600),
        );
        let m1 = SoundSourceMsg::new(
            SoundSourceId::new(SoundSourceType::AdsrEnvelope, 3),
            SoundSourceKey::Volume,
            SoundSourceValue::new_u32(100),
        );
        messages.append(m0.clone());
        messages.append(m1.clone());
        assert_eq!(2, messages.get_msgs().len());
        assert_eq!(m0, messages.get_msgs()[0]);
        assert_eq!(m1, messages.get_msgs()[1]);

        messages.clear();
        assert_eq!(0, messages.get_msgs().len());
    }
}
