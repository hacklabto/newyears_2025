use crate::sound_sample::time_to_ticks;
use crate::sound_sample::I32Fraction;
use crate::sound_sample::SoundSampleI32;
use crate::sound_source_core::SoundSourceCore;

const ADSR_FRACTION_DENOMINATOR: i64 = 0x8000000;
type AdsrFraction = I32Fraction<{ ADSR_FRACTION_DENOMINATOR as i32 }>;

///
/// ADSR envelope
///
pub struct CoreAdsr<
    const P_FREQ: u32,
    const U_FREQ: u32,
    const A: i32,
    const D: i32,
    const SUSTAIN_VOLUME: u8,
    const R: i32,
> {
    cached_adsr: SoundSampleI32,
    time_since_state_start: i32, // units are 1/P_FREQ
    last_sound: AdsrFraction,
    volume: i32,
}

impl<
        const P_FREQ: u32,
        const U_FREQ: u32,
        const A: i32,
        const D: i32,
        const SUSTAIN_VOLUME: u8,
        const R: i32,
    > CoreAdsr<P_FREQ, U_FREQ, A, D, SUSTAIN_VOLUME, R>
{
    const ATTACK_VOLUME_SCALE: SoundSampleI32 = SoundSampleI32::MAX;
    const SUSTAIN_VOLUME_SCALE: SoundSampleI32 = SoundSampleI32::new_percent(SUSTAIN_VOLUME);

    const A_TICKS: i32 = time_to_ticks::<U_FREQ>(A);
    const D_TICKS: i32 = time_to_ticks::<U_FREQ>(D);
    const R_TICKS: i32 = time_to_ticks::<U_FREQ>(R);

    const A_GAIN: AdsrFraction = if Self::A_TICKS != 0 {
        let a_diff: i64 = Self::ATTACK_VOLUME_SCALE.to_i32() as i64;
        AdsrFraction::new(
            (a_diff / (Self::A_TICKS as i64)) as i32,
            ((a_diff) % (Self::A_TICKS as i64) * ADSR_FRACTION_DENOMINATOR / (Self::A_TICKS as i64))
                as i32,
        )
    } else {
        AdsrFraction::new(0, 0)
    };

    const D_GAIN: AdsrFraction = if Self::D_TICKS != 0 {
        let d_diff: i64 =
            (Self::SUSTAIN_VOLUME_SCALE.to_i32() - Self::ATTACK_VOLUME_SCALE.to_i32()) as i64;
        AdsrFraction::new(
            (d_diff / (Self::D_TICKS as i64)) as i32,
            ((d_diff) % (Self::D_TICKS as i64) * ADSR_FRACTION_DENOMINATOR / (Self::D_TICKS as i64))
                as i32,
        )
    } else {
        AdsrFraction::new(0, 0)
    };

    const R_GAIN: AdsrFraction = if Self::R_TICKS != 0 {
        let assumed_start_volume = if Self::SUSTAIN_VOLUME_SCALE.to_i32() != 0 {
            Self::SUSTAIN_VOLUME_SCALE
        } else {
            Self::ATTACK_VOLUME_SCALE
        };
        let r_diff: i64 = (-assumed_start_volume.to_i32()) as i64;
        AdsrFraction::new(
            (r_diff / (Self::R_TICKS as i64)) as i32,
            ((r_diff) % (Self::R_TICKS as i64) * ADSR_FRACTION_DENOMINATOR / (Self::R_TICKS as i64))
                as i32,
        )
    } else {
        AdsrFraction::new(0, 0)
    };

    const A_END: i32 = Self::A_TICKS;
    const D_END: i32 = Self::A_END + Self::D_TICKS;
    const R_START: i32 = time_to_ticks::<U_FREQ>(10000);
    const R_END: i32 = Self::R_START + Self::R_TICKS;
}

impl<
        const P_FREQ: u32,
        const U_FREQ: u32,
        const A: i32,
        const D: i32,
        const SUSTAIN_VOLUME: u8,
        const R: i32,
    > SoundSourceCore<P_FREQ, U_FREQ> for CoreAdsr<P_FREQ, U_FREQ, A, D, SUSTAIN_VOLUME, R>
{
    type InitValuesType = i32;

    fn new(init_volume: Self::InitValuesType) -> Self {
        let time_since_state_start = 0;

        let last_sound = if A != 0 {
            AdsrFraction::new(0, 0)
        } else if D != 0 {
            AdsrFraction::new(Self::ATTACK_VOLUME_SCALE.to_i32(), 0)
        } else {
            AdsrFraction::new(Self::SUSTAIN_VOLUME_SCALE.to_i32(), 0)
        };

        Self {
            time_since_state_start,
            last_sound,
            volume: init_volume,
            cached_adsr: SoundSampleI32::new_i32(0),
        }
    }

    #[inline]
    fn get_next(self: &mut Self) -> SoundSampleI32 {
        self.cached_adsr
    }

    fn update(self: &mut Self) {
        let scale: SoundSampleI32 = if self.time_since_state_start < Self::A_END {
            let rval = SoundSampleI32::new_i32(self.last_sound.int_part);
            self.last_sound.add(&Self::A_GAIN);
            rval
        } else if self.time_since_state_start < Self::D_END {
            let rval = SoundSampleI32::new_i32(self.last_sound.int_part);
            self.last_sound.add(&Self::D_GAIN);
            rval
        } else if self.time_since_state_start < Self::R_START {
            let rval = SoundSampleI32::new_i32(self.last_sound.int_part);
            self.last_sound = AdsrFraction::new(Self::SUSTAIN_VOLUME_SCALE.to_i32(), 0);
            rval
        } else if self.time_since_state_start <= Self::R_END {
            let rval = SoundSampleI32::new_i32(self.last_sound.int_part);
            self.last_sound.add(&Self::R_GAIN);
            if self.last_sound.int_part < 0 {
                self.time_since_state_start = Self::R_END + 1;
            }
            rval
        } else {
            SoundSampleI32::ZERO
        };
        self.time_since_state_start = self.time_since_state_start + 1;
        let volume_adjusted_scale = SoundSampleI32::new_i32((self.volume * scale.to_i32()) >> 15);
        self.cached_adsr = volume_adjusted_scale.pos_clip()
    }

    fn has_next(self: &Self) -> bool {
        self.time_since_state_start <= Self::R_END
    }

    fn trigger_note_off(self: &mut Self) {
        self.time_since_state_start = Self::R_START;
    }
}

#[cfg(test)]
mod tests {
    use crate::adsr::*;

    #[test]
    fn basic_adsr_test() {
        let adsr_init: i32 = 0x8000;

        let mut adsr = CoreAdsr::<1000, 1000, 2, 4, 50, 8>::new(adsr_init);

        // Attack state, 2 ticks to get to attack volume (max) from 0
        adsr.update();
        assert_eq!(true, adsr.has_next());
        assert_eq!(0x0000, adsr.get_next().to_i32());
        adsr.update();
        assert_eq!(0x4000, adsr.get_next().to_i32());
        adsr.update();
        assert_eq!(0x8000, adsr.get_next().to_i32());

        // Delay state, 4 ticks to get to Sustain Volume (50%) from attack volume
        adsr.update();
        assert_eq!(true, adsr.has_next());
        assert_eq!(0x7000, adsr.get_next().to_i32());
        adsr.update();
        assert_eq!(0x6000, adsr.get_next().to_i32());
        adsr.update();
        assert_eq!(0x5000, adsr.get_next().to_i32());
        adsr.update();
        assert_eq!(0x4000, adsr.get_next().to_i32());

        // Sustain state
        adsr.update();
        assert_eq!(true, adsr.has_next());
        assert_eq!(0x4000, adsr.get_next().to_i32());
        adsr.update();
        assert_eq!(0x4000, adsr.get_next().to_i32());
        adsr.update();
        assert_eq!(0x4000, adsr.get_next().to_i32());
        adsr.trigger_note_off(); // Release doesn't start until update begins
        adsr.update();
        assert_eq!(0x4000, adsr.get_next().to_i32());

        // Release state, 4 ticks to get to quiet from Sustain Volume
        adsr.update();
        assert_eq!(true, adsr.has_next());
        assert_eq!(0x3800, adsr.get_next().to_i32());
        adsr.update();
        assert_eq!(0x3000, adsr.get_next().to_i32());
        adsr.update();
        assert_eq!(0x2800, adsr.get_next().to_i32());
        adsr.update();
        assert_eq!(0x2000, adsr.get_next().to_i32());
        adsr.update();
        assert_eq!(0x1800, adsr.get_next().to_i32());
        adsr.update();
        assert_eq!(0x1000, adsr.get_next().to_i32());
        adsr.update();
        assert_eq!(0x0800, adsr.get_next().to_i32());
        adsr.update();
        assert_eq!(0x0000, adsr.get_next().to_i32());

        // End state.  Report silence and no more data
        adsr.update();
        assert_eq!(false, adsr.has_next());
        assert_eq!(0x0000, adsr.get_next().to_i32());
        adsr.update();
        assert_eq!(0x0000, adsr.get_next().to_i32());
        adsr.update();
        assert_eq!(0x0000, adsr.get_next().to_i32());
        adsr.update();
        assert_eq!(0x0000, adsr.get_next().to_i32());
        adsr.update();
        assert_eq!(0x0000, adsr.get_next().to_i32());
        adsr.update();
        assert_eq!(0x0000, adsr.get_next().to_i32());
        assert_eq!(false, adsr.has_next());
    }
    #[test]
    fn no_attack_adsr_test() {
        let adsr_init: i32 = 0x8000;

        const D_RANGE: i32 = 1000;

        let mut adsr = CoreAdsr::<10000, 10000, 0, 100, 50, 8>::new(adsr_init);

        // Attack state, 2 ticks to get to attack volume (max) from 0
        assert_eq!(true, adsr.has_next());

        for i in 0..D_RANGE {
            adsr.update();
            assert_eq!((i, true), (i, adsr.has_next()));
            let desired: i32 = 0x4000 * i / D_RANGE + 0x8000 * (D_RANGE - i) / D_RANGE;
            let actual: i32 = adsr.get_next().to_i32();
            let diff = desired - actual;
            let is_less_than_2 = diff >= -2 && diff <= 2;

            assert_eq!(
                (i, desired, actual, is_less_than_2),
                (i, desired, actual, true)
            )
        }

        // Sustain state
        adsr.update();
        assert_eq!(true, adsr.has_next());
        assert_eq!(0x4001, adsr.get_next().to_i32());
        adsr.update();
        assert_eq!(0x4000, adsr.get_next().to_i32());
        adsr.update();
        assert_eq!(0x4000, adsr.get_next().to_i32());
        adsr.update();
        adsr.trigger_note_off(); // Release doesn't start until update begins
        assert_eq!(0x4000, adsr.get_next().to_i32());

        /*
        // Release state, 4 ticks to get to quiet from Sustain Volume
        assert_eq!(true, adsr.has_next());
        assert_eq!(0x3800, adsr.get_next().to_i32());
        assert_eq!(0x3000, adsr.get_next().to_i32());
        assert_eq!(0x2800, adsr.get_next().to_i32());
        assert_eq!(0x2000, adsr.get_next().to_i32());
        assert_eq!(0x1800, adsr.get_next().to_i32());
        assert_eq!(0x1000, adsr.get_next().to_i32());
        assert_eq!(0x0800, adsr.get_next().to_i32());
        assert_eq!(true, adsr.has_next());
        assert_eq!(0x0000, adsr.get_next().to_i32());

        // End state.  Report silence and no more data
        assert_eq!(false, adsr.has_next());
        assert_eq!(0x0000, adsr.get_next().to_i32());
        assert_eq!(0x0000, adsr.get_next().to_i32());
        assert_eq!(0x0000, adsr.get_next().to_i32());
        assert_eq!(0x0000, adsr.get_next().to_i32());
        assert_eq!(0x0000, adsr.get_next().to_i32());
        assert_eq!(0x0000, adsr.get_next().to_i32());
        assert_eq!(false, adsr.has_next());
        */
    }
}
