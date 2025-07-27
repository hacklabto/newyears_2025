use crate::sound_sample::time_to_ticks;
//use crate::sound_sample::I32Fraction;
use crate::sound_sample::SoundSampleI32;
use crate::sound_source_core::SoundSourceCore;

#[derive(Clone, PartialEq, Debug)]
pub struct SoundSourceAdsrInit {}

impl SoundSourceAdsrInit {
    pub fn new() -> Self {
        return Self {};
    }
}

/*
#[derive(PartialEq, Eq, Debug)]
pub enum AdsrState<const A: i32, const D: i32, const R: i32> {
    Attack { value: I32Fraction<A>},
    Delay { value: I32Fraction<D> },
    Sustain,
    Release { value: I32Fraction<R> },
    Ended,
}
*/
#[derive(PartialEq, Eq, Debug)]
pub enum AdsrState {
    ADS,
    Release,
    Ended,
}

///
/// ADSR envelope
///
pub struct CoreAdsr<
    const PLAY_FREQUENCY: u32,
    const A: i32,
    const D: i32,
    const SUSTAIN_VOLUME: u8,
    const R: i32,
> {
    time_since_state_start: i32, // units are 1/PLAY_FREQUENCY
}

impl<
        const PLAY_FREQUENCY: u32,
        const A: i32,
        const D: i32,
        const SUSTAIN_VOLUME: u8,
        const R: i32,
    > CoreAdsr<PLAY_FREQUENCY, A, D, SUSTAIN_VOLUME, R>
{
    const ATTACK_VOLUME_SCALE: SoundSampleI32 = SoundSampleI32::MAX;
    const SUSTAIN_VOLUME_SCALE: SoundSampleI32 = SoundSampleI32::new_percent(SUSTAIN_VOLUME);

    const A_TICKS: i32 = time_to_ticks::<PLAY_FREQUENCY>(A);
    const D_TICKS: i32 = time_to_ticks::<PLAY_FREQUENCY>(D);
    const R_TICKS: i32 = time_to_ticks::<PLAY_FREQUENCY>(R);

    const A_END: i32 = Self::A_TICKS;
    const D_END: i32 = Self::A_END + Self::D_TICKS;
    const R_START: i32 = time_to_ticks::<PLAY_FREQUENCY>(10000);
    const R_END: i32 = Self::R_START + Self::R_TICKS;
}

impl<
        const PLAY_FREQUENCY: u32,
        const A: i32,
        const D: i32,
        const SUSTAIN_VOLUME: u8,
        const R: i32,
    > SoundSourceCore<PLAY_FREQUENCY> for CoreAdsr<PLAY_FREQUENCY, A, D, SUSTAIN_VOLUME, R>
{
    type InitValuesType = SoundSourceAdsrInit;

    fn init(self: &mut Self, _init_values: &Self::InitValuesType) {
        self.time_since_state_start = 0;
    }

    fn get_next(self: &mut Self) -> SoundSampleI32 {
        let scale: SoundSampleI32 = if self.time_since_state_start < Self::A_END {
            Self::ATTACK_VOLUME_SCALE.mul_by_fraction(self.time_since_state_start, Self::A_TICKS)
        } else if self.time_since_state_start < Self::D_END {
            let time_since_d_start = self.time_since_state_start - Self::A_TICKS;
            Self::ATTACK_VOLUME_SCALE
                .mul_by_fraction(Self::D_TICKS - time_since_d_start, Self::D_TICKS)
                + Self::SUSTAIN_VOLUME_SCALE.mul_by_fraction(time_since_d_start, Self::D_TICKS)
        } else if self.time_since_state_start < Self::R_START {
            Self::SUSTAIN_VOLUME_SCALE
        } else if self.time_since_state_start <= Self::R_END {
            let time_since_r_start = self.time_since_state_start - Self::R_START;
            Self::SUSTAIN_VOLUME_SCALE
                .mul_by_fraction(Self::R_TICKS - time_since_r_start, Self::R_TICKS)
        } else {
            SoundSampleI32::ZERO
        };
        self.time_since_state_start = self.time_since_state_start + 1;
        scale
    }

    fn has_next(self: &Self) -> bool {
        self.time_since_state_start <= Self::R_END
    }

    fn trigger_note_off(self: &mut Self) {
        self.time_since_state_start = Self::R_START;
    }
}

impl<
        const PLAY_FREQUENCY: u32,
        const A: i32,
        const D: i32,
        const SUSTAIN_VOLUME: u8,
        const R: i32,
    > Default for CoreAdsr<PLAY_FREQUENCY, A, D, SUSTAIN_VOLUME, R>
{
    fn default() -> Self {
        let time_since_state_start = 0;

        Self {
            time_since_state_start,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::adsr::*;

    #[test]
    fn basic_adsr_test() {
        let adsr_init = SoundSourceAdsrInit::new();

        let mut adsr = CoreAdsr::<1000, 2, 4, 50, 8>::default();
        adsr.init(&adsr_init);

        // Attack state, 2 ticks to get to attack volume (max) from 0
        assert_eq!(true, adsr.has_next());
        assert_eq!(0x0000, adsr.get_next().to_i32());
        assert_eq!(0x4000, adsr.get_next().to_i32());
        assert_eq!(0x8000, adsr.get_next().to_i32());

        // Delay state, 4 ticks to get to Sustain Volume (50%) from attack volume
        assert_eq!(true, adsr.has_next());
        assert_eq!(0x7000, adsr.get_next().to_i32());
        assert_eq!(0x6000, adsr.get_next().to_i32());
        assert_eq!(0x5000, adsr.get_next().to_i32());
        assert_eq!(0x4000, adsr.get_next().to_i32());

        // Sustain state
        assert_eq!(true, adsr.has_next());
        assert_eq!(0x4000, adsr.get_next().to_i32());
        assert_eq!(0x4000, adsr.get_next().to_i32());
        assert_eq!(0x4000, adsr.get_next().to_i32());
        adsr.trigger_note_off(); // Release doesn't start until update begins
        assert_eq!(0x4000, adsr.get_next().to_i32());

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
    }
}
