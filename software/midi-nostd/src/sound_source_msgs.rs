use crate::sound_source_id::SoundSourceId;

#[derive(Clone, PartialEq, Debug)]
pub struct SoundSourceNoteInit {
    pub key: u8,
    pub instrument: u8,
}

impl SoundSourceNoteInit {
    pub fn new(key: u8, instrument: u8) -> Self {
        return Self { key, instrument };
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum SoundSourceValue {
    Uninitialized,
    NoteInit { init_values: SoundSourceNoteInit },
    AmpAdderInit,
    ReleaseAdsr,
    SoundSourceCreated,
    CreatedId { created_id: SoundSourceId },
}

impl Default for SoundSourceValue {
    fn default() -> Self {
        SoundSourceValue::Uninitialized
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct SoundSourceMsg {
    pub dest_id: SoundSourceId,
    pub src_id: SoundSourceId,
    pub value: SoundSourceValue,
}

impl Default for SoundSourceMsg {
    fn default() -> Self {
        let dest_id = SoundSourceId::default();
        let src_id = SoundSourceId::default();
        let value = SoundSourceValue::default();
        Self {
            src_id,
            dest_id,
            value,
        }
    }
}

impl SoundSourceMsg {
    pub fn new(dest_id: SoundSourceId, src_id: SoundSourceId, value: SoundSourceValue) -> Self {
        return Self {
            src_id: src_id,
            dest_id: dest_id,
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
    //use crate::sound_source_id::SoundSourceType;
    //use crate::sound_source_msgs::SoundSourceMsg;
    //use crate::sound_source_msgs::SoundSourceMsgs;
    //use crate::sound_source_msgs::SoundSourceValue;

    #[test]
    fn messages_should_work() {
        /* TODO */
        /*
        let mut messages = SoundSourceMsgs::default();
        assert_eq!(0, messages.get_msgs().len());

        let m0 = SoundSourceMsg::new(
            SoundSourceId::new(SoundSourceType::Oscillator, 5),
            SoundSourceId::new(SoundSourceType::Oscillator, 3),
            SoundSourceValue::ReleaseAdsr,
        );
        let m1 = SoundSourceMsg::new(
            SoundSourceId::new(SoundSourceType::Adsr, 3),
            SoundSourceId::new(SoundSourceType::Adsr, 5),
            SoundSourceValue::ReleaseAdsr,
        );
        messages.append(m0.clone());
        messages.append(m1.clone());
        assert_eq!(2, messages.get_msgs().len());
        assert_eq!(m0, messages.get_msgs()[0]);
        assert_eq!(m1, messages.get_msgs()[1]);

        messages.clear();
        assert_eq!(0, messages.get_msgs().len());
        */
    }
}
