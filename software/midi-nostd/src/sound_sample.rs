use core::cmp::Ordering;
use core::ops::Add;
use core::ops::Div;
use core::ops::Mul;
use core::ops::Sub;

#[derive(Clone, Copy, Default, PartialEq, Debug)]
pub struct SoundScale {
    // 1.8 fixed point value.  Valid values are 0 to 256
    pub scale_by_int: i32,
}

impl SoundScale {
    pub const fn new(scale_by: u16) -> Self {
        let scale_by_int: i32 = scale_by as i32;
        return Self { scale_by_int };
    }
    pub const fn new_percent(scale_by_percent: u8) -> Self {
        return Self::new((scale_by_percent as u16) * 256 / 100);
    }
    pub fn get_scale_by_int(&self) -> i32 {
        self.scale_by_int
    }
}

///
/// Concrete implementation of Sound Sample using fixed point
///
/// ~15 bits are used for the fractional component.  Playable sound is from
/// -0x8000 (-1) to 0x7fff (1).  0 maps to zero.
///
#[derive(Clone, Eq, Copy, Default, Debug)]
pub struct SoundSampleI32 {
    pub val: i32,
}

impl SoundSampleI32 {
    ///
    /// Constructor
    ///
    pub const fn new_i32(val: i32) -> Self {
        Self { val }
    }

    pub const MAX: Self = Self::new_i32(0x7fff);
    pub const MIN: Self = Self::new_i32(-0x8000);
    pub const ZERO: Self = Self::new_i32(0);

    pub fn to_i32(&self) -> i32 {
        self.val
    }

    pub fn scale(&mut self, scale_by: SoundScale) {
        self.val = (self.val * scale_by.get_scale_by_int()) >> 8;
    }

    /// Guarantee that a sample is playable
    ///
    /// out of bounds samples are cliped.
    ///
    pub fn clip(&self) -> Self {
        if *self > Self::MAX {
            Self::MAX
        } else if *self < Self::MIN {
            Self::MIN
        } else {
            self.clone()
        }
    }
}

impl Add for SoundSampleI32 {
    type Output = SoundSampleI32;

    fn add(mut self, rhs: SoundSampleI32) -> SoundSampleI32 {
        self.val += rhs.val;
        self
    }
}

impl Sub for SoundSampleI32 {
    type Output = SoundSampleI32;
    fn sub(mut self, rhs: SoundSampleI32) -> SoundSampleI32 {
        self.val -= rhs.val;
        self
    }
}

impl Mul for SoundSampleI32 {
    type Output = SoundSampleI32;
    fn mul(mut self, rhs: SoundSampleI32) -> SoundSampleI32 {
        self.val = ((self.val >> 1) * (rhs.val >> 1)) >> 14;
        self
    }
}

impl Div<i32> for SoundSampleI32 {
    type Output = SoundSampleI32;
    fn div(mut self, rhs: i32) -> SoundSampleI32 {
        self.val = self.val / rhs;
        self
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
    fn samplei32_should_less_than_properly() {
        let v0 = SoundSampleI32::new_i32(0);
        let v1 = SoundSampleI32::new_i32(1);
        assert!(v0 < v1);
        assert!(!(v1 < v0));
    }

    #[test]
    fn samplei32_should_greater_than_properly() {
        let v0 = SoundSampleI32::new_i32(0);
        let v1 = SoundSampleI32::new_i32(1);
        assert!(v1 > v0);
        assert!(!(v0 > v1));
    }

    #[test]
    fn samplei32_should_equals_properly() {
        let v0 = SoundSampleI32::new_i32(0);
        let v1 = SoundSampleI32::new_i32(0);
        let v2 = SoundSampleI32::new_i32(1);
        assert!(v0 == v1);
        assert!(v0 != v2);
    }

    #[test]
    fn samplei32_should_clip_properly() {
        let v0 = SoundSampleI32::new_i32(0x100000);
        assert_eq!(v0.clip().to_i32(), 0x7fff);
        let v1 = SoundSampleI32::new_i32(-0x100000);
        assert_eq!(v1.clip().to_i32(), -0x8000);
        let v2 = SoundSampleI32::new_i32(5);
        assert!(v2 == v2.clip());
    }

    #[test]
    fn samplei32_should_add_and_sub_properly() {
        let v0 = SoundSampleI32::new_i32(10);
        let v1 = SoundSampleI32::new_i32(5);

        assert!(v0 == v1 + v1);
        assert!(v1 == v0 - v1);
    }

    #[test]
    fn samplei32_should_scale_properly() {
        let v0 = SoundSampleI32::new_i32(0);
        let v1 = SoundSampleI32::new_i32(100);
        let mut v2 = SoundSampleI32::new_i32(100);
        let mut v3 = SoundSampleI32::new_i32(100);
        let mut v4 = SoundSampleI32::new_i32(200);

        v2.scale(SoundScale::new_percent(100));
        v3.scale(SoundScale::new(0));
        v4.scale(SoundScale::new_percent(50));

        assert!(v1 == v2); // scaled by 1, unchanged
        assert!(v0 == v3); // scaled by 0
        assert!(v1 == v4); // scaled by .5
    }
}
