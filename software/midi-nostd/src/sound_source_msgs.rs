use crate::sound_source_id::SoundSourceId;

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

#[derive(Clone, PartialEq, Debug)]
#[allow(unused)]
pub struct SoundSourceMsg {
    pub dest_id: SoundSourceId,
    pub attribute: SoundSourceAttributes,
    pub value: usize,
}

#[allow(unused)]
impl Default for SoundSourceMsg {
    fn default() -> Self {
        let dest_id = SoundSourceId::default();
        let attribute = SoundSourceAttributes::Frequency;
        let value = 0;
        Self {
            dest_id,
            attribute,
            value,
        }
    }
}

#[allow(unused)]
impl SoundSourceMsg {
    pub fn new(dest_id: SoundSourceId, attribute: SoundSourceAttributes, value: usize) -> Self {
        return Self {
            dest_id,
            attribute,
            value,
        };
    }
}

#[allow(unused)]
pub struct SoundSourceMsgPool<const N: usize> {
    messages: [SoundSourceMsg; N],
    last: usize,
}

#[allow(unused)]
impl<const N: usize> Default for SoundSourceMsgPool<N> {
    fn default() -> Self {
        let messages: [SoundSourceMsg; N] = core::array::from_fn(|_i| SoundSourceMsg::default());
        let last: usize = 0;
        return Self { messages, last };
    }
}

#[allow(unused)]
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

#[allow(unused)]
pub type SoundSourceMsgs = SoundSourceMsgPool<100>;

#[cfg(test)]
mod tests {
    use crate::sound_source_id::SoundSourceId;
    use crate::sound_source_id::SoundSourceType;
    use crate::sound_source_msgs::SoundSourceAttributes;
    use crate::sound_source_msgs::SoundSourceMsg;
    use crate::sound_source_msgs::SoundSourceMsgs;

    #[test]
    fn messages_should_work() {
        let mut messages = SoundSourceMsgs::default();
        assert_eq!(0, messages.get_msgs().len());

        let m0 = SoundSourceMsg::new(
            SoundSourceId::new(SoundSourceType::WaveGenerator, 5),
            SoundSourceAttributes::Frequency,
            2600,
        );
        let m1 = SoundSourceMsg::new(
            SoundSourceId::new(SoundSourceType::AdsrEnvelope, 3),
            SoundSourceAttributes::Volume,
            100,
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
