use core::cmp::Ordering;

pub trait SoundSample: Clone + Eq + PartialOrd {
    fn max() -> Self;
    fn min() -> Self;
    fn to_u16(&self) -> u16;
    fn clip(&self) -> Self {
        if *self > Self::max() {
            Self::max()
        } else if *self < Self::min() {
            Self::min()
        } else {
            self.clone()
        }
    }
}

#[derive(Clone, Eq)]
pub struct SoundSampleI32 {
    pub val: i32,
}

impl SoundSampleI32 {
    const fn new(val: i32) -> Self {
        Self { val }
    }
}

impl SoundSample for SoundSampleI32 {
    fn max() -> Self {
        Self::new(0x7fff)
    }
    fn min() -> Self {
        Self::new(-0x8000)
    }

    fn to_u16(&self) -> u16 {
        (self.val + 0x8000) as u16
    }
}

impl PartialEq for SoundSampleI32 {
    fn eq(&self, other: &Self) -> bool {
        self.val == other.val
    }
}

impl PartialOrd for SoundSampleI32 {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.val.partial_cmp(&other.val)
    }
}

#[cfg(test)]
mod tests {
    use crate::sound_sample::*;

    #[test]
    fn test_i32_lt() {
        let v0 = SoundSampleI32::new(0);
        let v1 = SoundSampleI32::new(1);
        assert!(v0 < v1);
        assert!(!(v1 < v0));
    }

    #[test]
    fn test_i32_gt() {
        let v0 = SoundSampleI32::new(0);
        let v1 = SoundSampleI32::new(1);
        assert!(v1 > v0);
        assert!(!(v0 > v1));
    }

    #[test]
    fn test_i32_eq() {
        let v0 = SoundSampleI32::new(0);
        let v1 = SoundSampleI32::new(0);
        let v2 = SoundSampleI32::new(1);
        assert!(v0 == v1);
        assert!(v0 != v2);
    }

    #[test]
    fn test_i32_tou16() {
        let v0 = SoundSampleI32::new(0);
        assert_eq!(v0.to_u16(), 0x8000);
        let v1 = SoundSampleI32::new(-1);
        assert_eq!(v1.to_u16(), 0x7fff);
        let v2 = SoundSampleI32::new(1);
        assert_eq!(v2.to_u16(), 0x8001);
    }

    #[test]
    fn test_clip() {
        let v0 = SoundSampleI32::new(0x100000);
        assert_eq!(v0.clip().to_u16(), 0xffff);
        let v1 = SoundSampleI32::new(-0x100000);
        assert_eq!(v1.clip().to_u16(), 0);
        let v2 = SoundSampleI32::new(5);
        assert!(v2 == v2.clip());
    }
}
