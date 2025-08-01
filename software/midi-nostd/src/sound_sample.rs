use core::cmp::Ordering;
use core::ops::Add;
use core::ops::Div;
use core::ops::Mul;
use core::ops::Sub;

pub const fn time_to_ticks<const PLAY_FREQUENCY: u32>(time_in_ms: i32) -> i32 {
    return ((PLAY_FREQUENCY as i32) * time_in_ms) / 1000;
}

#[derive(Debug, PartialEq, Eq)]
pub struct I32Fraction<const DENOMINATOR: i32> {
    pub int_part: i32,
    pub numerator_part: i32,
}

impl<const DENOMINATOR: i32> I32Fraction<DENOMINATOR> {
    pub const fn new(int_part: i32, numerator_part: i32) -> Self {
        Self {
            int_part,
            numerator_part,
        }
    }
    #[inline]
    pub const fn add(self: &mut Self, other: &Self) {
        self.int_part += other.int_part;
        self.numerator_part += other.numerator_part;
        if self.numerator_part > DENOMINATOR {
            if other.int_part >= 0 {
                self.int_part = self.int_part + 1;
            } else {
                panic!("unexpected");
            }
            self.numerator_part -= DENOMINATOR;
        }
        if self.numerator_part < -DENOMINATOR {
            if other.int_part <= 0 {
                self.int_part = self.int_part - 1;
            } else {
                panic!("unexpected");
            }
            self.numerator_part += DENOMINATOR;
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct U32Fraction<const DENOMINATOR: u32> {
    pub int_part: u32,
    pub numerator_part: u32,
}

impl<const DENOMINATOR: u32> U32Fraction<DENOMINATOR> {
    pub const fn new(int_part: u32, numerator_part: u32) -> Self {
        Self {
            int_part,
            numerator_part,
        }
    }
    #[inline]
    pub const fn add(self: &mut Self, other: &Self) {
        self.int_part += other.int_part;
        self.numerator_part += other.numerator_part;
        if self.numerator_part > DENOMINATOR {
            self.int_part = self.int_part + 1;
            self.numerator_part -= DENOMINATOR;
        }
    }
}

///
/// Concrete implementation of Sound Sample using fixed point
///
/// ~15 bits are used for the fractional component.  Playable sound is from
/// -0x8000 (-1) to 0x7fff (1).  0 maps to zero.
///
#[derive(Clone, Eq, Copy, Debug)]
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

    pub const fn new_percent(scale_by_percent: u8) -> Self {
        return Self::new_i32(0x8000 * (scale_by_percent as i32) / 100);
    }

    //
    // Making this 0x8000.  If we convert to 16 or 8 bit output we'll need
    // the extra logic of a clip phase, but it makes the integer math so much
    // more sane.
    //
    pub const MAX: Self = Self::new_i32(0x8000);
    pub const MIN: Self = Self::new_i32(-0x8000);
    pub const ZERO: Self = Self::new_i32(0);

    pub const fn to_i32(&self) -> i32 {
        self.val
    }

    pub const fn const_clone(&self) -> Self {
        Self::new_i32(self.val)
    }

    pub const fn const_lt(&self, other: &Self) -> bool {
        return self.val < other.val;
    }

    pub const fn const_gt(&self, other: &Self) -> bool {
        return self.val > other.val;
    }

    pub const fn const_mul(mut self, rhs: Self) -> Self {
        self.val = ((self.val) * (rhs.val)) >> 15;
        self
    }

    pub const fn mul_by_fraction(mut self, numerator: i32, denominator: i32) -> Self {
        self.val = self.val * numerator / denominator;
        self
    }

    /// Guarantee that a sample is playable
    ///
    /// out of bounds samples are cliped.
    ///
    pub const fn clip(self) -> Self {
        if self.const_gt(&Self::MAX) {
            Self::MAX
        } else if self.const_lt(&Self::MIN) {
            Self::MIN
        } else {
            self
        }
    }
    pub const fn pos_clip(self) -> Self {
        if self.const_gt(&Self::MAX) {
            Self::MAX
        } else if self.const_lt(&Self::ZERO) {
            Self::ZERO
        } else {
            self
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
    fn mul(self, rhs: SoundSampleI32) -> SoundSampleI32 {
        self.const_mul(rhs)
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
        assert_eq!(v0.clip().to_i32(), 0x8000);
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

        v2 = v2 * SoundSampleI32::new_percent(100);
        v3 = v3 * SoundSampleI32::new_percent(0);
        v4 = v4 * SoundSampleI32::new_percent(50);

        assert_eq!(v1, v2); // scaled by 1, unchanged
        assert_eq!(v0, v3); // scaled by 0
        assert_eq!(v1, v4); // scaled by .5
    }
}
