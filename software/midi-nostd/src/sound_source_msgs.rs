use crate::sound_sample::SoundScale;
use crate::sound_source_id::SoundSourceId;

///
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SoundSourceKey {
    InitOscillator,
    InitAdsr,
    InitAmpMixer,
    InitAmpAdder,
    ReleaseAdsr,
    SoundSourceCreated,
}

/// Different Wave Types
///
#[derive(Clone, Copy, PartialEq, Debug)]
#[repr(usize)]
pub enum OscillatorType {
    Triangle = 0,
    SawTooth = 1,
    Sine = 2,
    PulseWidth = 3,
}

impl OscillatorType {
    // TODO, delete.
    pub fn from_usize(usize_value: usize) -> Self {
        let optional_enum_value: Option<Self> = match usize_value {
            0 => Some(OscillatorType::Triangle),
            1 => Some(OscillatorType::SawTooth),
            2 => Some(OscillatorType::Sine),
            3 => Some(OscillatorType::PulseWidth),
            _ => None,
        };
        let enum_value = optional_enum_value.expect("bad usize  aveType");
        assert_eq!(usize_value, enum_value as usize); // cheap sanity check
        enum_value
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct SoundSourceOscillatorInit {
    pub oscillator_type: OscillatorType,
    pub frequency: u32,
    pub pulse_width: u8,
    pub volume: u8,
}

impl SoundSourceOscillatorInit {
    pub fn new(
        oscillator_type: OscillatorType,
        frequency: u32,
        pulse_width: u8,
        volume: u8,
    ) -> Self {
        return Self {
            oscillator_type,
            frequency,
            pulse_width,
            volume,
        };
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct SoundSourceAdsrInit {
    pub attack_max_volume: SoundScale,
    pub sustain_volume: SoundScale,
    pub a: u32,
    pub d: u32,
    pub r: u32,
}

impl SoundSourceAdsrInit {
    pub fn new(
        attack_max_volume: SoundScale,
        sustain_volume: SoundScale,
        a: u32,
        d: u32,
        r: u32,
    ) -> Self {
        return Self {
            attack_max_volume,
            sustain_volume,
            a,
            d,
            r,
        };
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct SoundSourceAmpMixerInit {
    pub source_0: SoundSourceId,
    pub source_1: SoundSourceId,
}

impl SoundSourceAmpMixerInit {
    pub fn new(source_0: SoundSourceId, source_1: SoundSourceId) -> Self {
        return Self { source_0, source_1 };
    }
}

#[derive(Clone, PartialEq, Debug, Default)]
pub struct SoundSourceAmpAdderInit {}

#[derive(Clone, PartialEq, Debug)]
pub enum SoundSourceValue {
    Uninitialized,
    U32Type {
        num: u32,
    },
    U8Type {
        num: u8,
    },
    OscillatorInit {
        init_values: SoundSourceOscillatorInit,
    },
    AdsrInit {
        init_values: SoundSourceAdsrInit,
    },
    AmpMixerInit {
        init_values: SoundSourceAmpMixerInit,
    },
    AmpAdderInit {},
    CreatedId {
        created_id: SoundSourceId,
    },
}

impl SoundSourceValue {
    pub fn new_u32(num: u32) -> Self {
        SoundSourceValue::U32Type { num }
    }
    pub fn new_u8(num: u8) -> Self {
        SoundSourceValue::U8Type { num }
    }
    pub fn new_oscillator_init(init_values: SoundSourceOscillatorInit) -> Self {
        SoundSourceValue::OscillatorInit { init_values }
    }
    pub fn new_adsr_init(init_values: SoundSourceAdsrInit) -> Self {
        SoundSourceValue::AdsrInit { init_values }
    }
    pub fn new_amp_mixer_init(init_values: SoundSourceAmpMixerInit) -> Self {
        SoundSourceValue::AmpMixerInit { init_values }
    }
    pub fn new_amp_adder_init() -> Self {
        SoundSourceValue::AmpAdderInit {}
    }
    pub fn new_created_id(created_id: SoundSourceId) -> Self {
        SoundSourceValue::CreatedId { created_id }
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
    pub fn get_oscillator_init(self: &Self) -> &SoundSourceOscillatorInit {
        match self {
            SoundSourceValue::OscillatorInit { init_values } => init_values,
            _ => panic!("This isn't a wave type"),
        }
    }
    pub fn get_adsr_init(self: &Self) -> &SoundSourceAdsrInit {
        match self {
            SoundSourceValue::AdsrInit { init_values } => init_values,
            _ => panic!("This isn't a wave type"),
        }
    }
    pub fn get_amp_mixer_init(self: &Self) -> &SoundSourceAmpMixerInit {
        match self {
            SoundSourceValue::AmpMixerInit { init_values } => init_values,
            _ => panic!("This isn't an amp mixer type"),
        }
    }
    pub fn get_created_id(self: &Self) -> &SoundSourceId {
        match self {
            SoundSourceValue::CreatedId { created_id } => created_id,
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
    pub src_id: SoundSourceId,
    pub key: SoundSourceKey,
    pub value: SoundSourceValue,
}

impl Default for SoundSourceMsg {
    fn default() -> Self {
        let dest_id = SoundSourceId::default();
        let src_id = SoundSourceId::default();
        let key = SoundSourceKey::InitOscillator;
        let value = SoundSourceValue::default();
        Self {
            src_id,
            dest_id,
            key,
            value,
        }
    }
}

impl SoundSourceMsg {
    pub fn new(
        dest_id: SoundSourceId,
        src_id: SoundSourceId,
        key: SoundSourceKey,
        value: SoundSourceValue,
    ) -> Self {
        return Self {
            src_id: src_id,
            dest_id: dest_id,
            key,
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
            SoundSourceId::new(SoundSourceType::Oscillator, 5),
            SoundSourceId::new(SoundSourceType::Oscillator, 3),
            SoundSourceKey::InitOscillator,
            SoundSourceValue::new_u32(2600),
        );
        let m1 = SoundSourceMsg::new(
            SoundSourceId::new(SoundSourceType::Adsr, 3),
            SoundSourceId::new(SoundSourceType::Adsr, 5),
            SoundSourceKey::InitOscillator,
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
