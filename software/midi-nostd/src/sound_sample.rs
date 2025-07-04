use core::cmp::Ordering;
use core::ops::Add;
use core::ops::Div;
use core::ops::Sub;

#[allow(unused)]
#[derive(Clone, Copy)]
pub struct SoundScale {
    // 1.8 fixed point value.  Valid values are 0 to 256
    pub scale_by_int: i32,
}

#[allow(unused)]
impl SoundScale {
    pub fn new(scale_by: u16) -> Self {
        let scale_by_int: i32 = scale_by.into();
        return Self { scale_by_int };
    }
    pub fn new_percent(scale_by_percent: u16) -> Self {
        return Self::new((scale_by_percent as u16) * 256 / 100);
    }
    pub fn get_scale_by_int(&self) -> i32 {
        self.scale_by_int
    }
}

/// A short explanation for this file....
///
/// I'm abstracting some basic math concepts because I want this library to work
/// well on embeded devices, and a lot of them won't have some of the things we
/// take forgranted on desktop computers, like hardware floating point or CPU
/// speeds in the GHz.  That means I'll be doing a lot of fixed point math.
///

///
/// Interface for a sound sample
///
/// Fundementally, a value from -1 to 1, where 0 represents no sound, and -1 and 1 are
/// the maximum amplitudes off the sound waveform...  Floating point would be an obvious and
/// easy choice to represent this information... except a lot of micro-controllers don't
/// have hardware floating point support.
///
#[allow(unused)]
pub trait SoundSample: Clone + Eq + PartialOrd + Add + Copy + Sub + Default {
    /// Maximum playable sound sample
    ///
    fn max() -> Self;

    /// Minimum playabe sound sample
    ///
    fn min() -> Self;

    /// Convert a playable sound sample to a u16 more suitable for hardware
    ///
    /// Sample must be playable.  "zero" maps to 0x8000, min -> 0, max -> ffff
    ///
    fn to_u16(&self) -> u16;

    /// Create a sound sample from something more suitable for tables
    fn new(init_val: u16) -> Self;

    /// scale by some value
    ///
    fn scale(&mut self, scale_by: SoundScale);

    /// Guarantee that a sample is playable
    ///
    /// out of bounds samples are cliped.
    ///
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
    fn new(sample: u16) -> Self {
        let int_sample: i32 = (sample as i32) - 0x8000;
        Self::new(int_sample)
    }

    fn to_u16(&self) -> u16 {
        (self.val + 0x8000) as u16
    }

    fn scale(&mut self, scale_by: SoundScale) {
        self.val = (self.val * scale_by.get_scale_by_int()) >> 8;
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
        let v0 = SoundSampleI32::new(0);
        let v1 = SoundSampleI32::new(1);
        assert!(v0 < v1);
        assert!(!(v1 < v0));
    }

    #[test]
    fn samplei32_should_greater_than_properly() {
        let v0 = SoundSampleI32::new(0);
        let v1 = SoundSampleI32::new(1);
        assert!(v1 > v0);
        assert!(!(v0 > v1));
    }

    #[test]
    fn samplei32_should_equals_properly() {
        let v0 = SoundSampleI32::new(0);
        let v1 = SoundSampleI32::new(0);
        let v2 = SoundSampleI32::new(1);
        assert!(v0 == v1);
        assert!(v0 != v2);
    }

    #[test]
    fn samplei32_should_clip_properly() {
        let v0 = SoundSampleI32::new(0x100000);
        assert_eq!(v0.clip().to_u16(), 0xffff);
        let v1 = SoundSampleI32::new(-0x100000);
        assert_eq!(v1.clip().to_u16(), 0);
        let v2 = SoundSampleI32::new(5);
        assert!(v2 == v2.clip());
    }

    #[test]
    fn samplei32_should_add_and_sub_properly() {
        let v0 = SoundSampleI32::new(10);
        let v1 = SoundSampleI32::new(5);

        assert!(v0 == v1 + v1);
        assert!(v1 == v0 - v1);
    }

    #[test]
    fn samplei32_should_scale_properly() {
        let v0 = SoundSampleI32::new(0);
        let v1 = SoundSampleI32::new(100);
        let mut v2 = SoundSampleI32::new(100);
        let mut v3 = SoundSampleI32::new(100);
        let mut v4 = SoundSampleI32::new(200);

        v2.scale(SoundScale::new_percent(100));
        v3.scale(SoundScale::new(0));
        v4.scale(SoundScale::new_percent(50));

        assert!(v1 == v2); // scaled by 1, unchanged
        assert!(v0 == v3); // scaled by 0
        assert!(v1 == v4); // scaled by .5
    }
}
