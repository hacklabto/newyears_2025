use crate::sound_sample::time_to_ticks;
use crate::sound_sample::SoundSampleI32;
use crate::sound_source_core::SoundSourceCore;

#[derive(Clone, PartialEq, Debug)]
pub struct SoundSourceAdsrInit {}

impl SoundSourceAdsrInit {
    pub fn new() -> Self {
        return Self {};
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AdsrState {
    Attack,
    Delay,
    Sustain,
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
    const R: i32,
    const ATTACK_VOLUME: u8,
    const SUSTAIN_VOLUME: u8,
> {
    state: AdsrState,            // CurrentState
    time_since_state_start: i32, // units are 1/PLAY_FREQUENCY
}

impl<
        const PLAY_FREQUENCY: u32,
        const A: i32,
        const D: i32,
        const R: i32,
        const ATTACK_VOLUME: u8,
        const SUSTAIN_VOLUME: u8,
    > CoreAdsr<PLAY_FREQUENCY, A, D, R, ATTACK_VOLUME, SUSTAIN_VOLUME>
{
    const ATTACK_VOLUME_SCALE: SoundSampleI32 = SoundSampleI32::new_percent(ATTACK_VOLUME);
    const SUSTAIN_VOLUME_SCALE: SoundSampleI32 = SoundSampleI32::new_percent(SUSTAIN_VOLUME);

    const A_TICKS: i32 = time_to_ticks::<PLAY_FREQUENCY>(A);
    const D_TICKS: i32 = time_to_ticks::<PLAY_FREQUENCY>(D);
    const R_TICKS: i32 = time_to_ticks::<PLAY_FREQUENCY>(R);

    fn update_internal(&mut self) {
        if self.state == AdsrState::Attack {
            if self.time_since_state_start >= Self::A_TICKS {
                self.time_since_state_start = 0;
                self.state = AdsrState::Delay;
            }
        }
        if self.state == AdsrState::Delay {
            if self.time_since_state_start >= Self::D_TICKS {
                self.time_since_state_start = 0;
                self.state = AdsrState::Sustain;
            }
        }
        if self.state == AdsrState::Release {
            if self.time_since_state_start > Self::R_TICKS {
                self.time_since_state_start = 0;
                self.state = AdsrState::Ended;
            }
        }
    }
}

impl<
        const PLAY_FREQUENCY: u32,
        const A: i32,
        const D: i32,
        const R: i32,
        const ATTACK_VOLUME: u8,
        const SUSTAIN_VOLUME: u8,
    > SoundSourceCore<PLAY_FREQUENCY>
    for CoreAdsr<PLAY_FREQUENCY, A, D, R, ATTACK_VOLUME, SUSTAIN_VOLUME>
{
    type InitValuesType = SoundSourceAdsrInit;

    fn init(self: &mut Self, _init_values: &Self::InitValuesType) {
        self.state = AdsrState::Attack;
        self.time_since_state_start = 0;
        self.update_internal();
    }

    fn get_next(self: &mut Self) -> SoundSampleI32 {
        let scale: SoundSampleI32 = match self.state {
            AdsrState::Attack => Self::ATTACK_VOLUME_SCALE
                .mul_by_fraction(self.time_since_state_start, Self::A_TICKS),
            AdsrState::Delay => {
                Self::ATTACK_VOLUME_SCALE
                    .mul_by_fraction(Self::D_TICKS - self.time_since_state_start, Self::D_TICKS)
                    + Self::SUSTAIN_VOLUME_SCALE
                        .mul_by_fraction(self.time_since_state_start, Self::D_TICKS)
            }
            AdsrState::Sustain => Self::SUSTAIN_VOLUME_SCALE,
            AdsrState::Release => Self::SUSTAIN_VOLUME_SCALE
                .mul_by_fraction(Self::R_TICKS - self.time_since_state_start, Self::R_TICKS),
            AdsrState::Ended => SoundSampleI32::ZERO,
        };
        self.time_since_state_start = self.time_since_state_start + 1;
        self.update_internal();

        scale
    }

    fn has_next(self: &Self) -> bool {
        self.state != AdsrState::Ended
    }

    fn trigger_note_off(self: &mut Self) {
        // TODO, What if we aren't in sustain?  Probably I should take
        // the current volume and run the release on that.
        self.state = AdsrState::Release;
        self.time_since_state_start = 0;
    }

    fn reset_oscillator(self: &mut Self) {}
}

impl<
        const PLAY_FREQUENCY: u32,
        const A: i32,
        const D: i32,
        const R: i32,
        const ATTACK_VOLUME: u8,
        const SUSTAIN_VOLUME: u8,
    > Default for CoreAdsr<PLAY_FREQUENCY, A, D, R, ATTACK_VOLUME, SUSTAIN_VOLUME>
{
    fn default() -> Self {
        let state = AdsrState::Ended;
        let time_since_state_start = 0;

        Self {
            state,
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

        let mut adsr = CoreAdsr::<1000, 2, 4, 4, 100, 50>::default();
        adsr.init(&adsr_init);

        // Attack state, 2 ticks to get to attack volume (max) from 0
        assert_eq!(0x0000, adsr.get_next().to_i32());
        assert_eq!(0x4000, adsr.get_next().to_i32());
        assert_eq!(0x8000, adsr.get_next().to_i32());

        // Delay state, 4 ticks to get to Sustain Volume (50%) from attack volume
        assert_eq!(0x7000, adsr.get_next().to_i32());
        assert_eq!(0x6000, adsr.get_next().to_i32());
        assert_eq!(0x5000, adsr.get_next().to_i32());
        assert_eq!(0x4000, adsr.get_next().to_i32());

        // Sustain state
        assert_eq!(0x4000, adsr.get_next().to_i32());
        assert_eq!(0x4000, adsr.get_next().to_i32());
        assert_eq!(0x4000, adsr.get_next().to_i32());
        adsr.trigger_note_off(); // Release doesn't start until update begins
        assert_eq!(0x4000, adsr.get_next().to_i32());

        // Release state, 4 ticks to get to quiet from Sustain Volume
        assert_eq!(0x3000, adsr.get_next().to_i32());
        assert_eq!(0x2000, adsr.get_next().to_i32());
        assert_eq!(0x1000, adsr.get_next().to_i32());
        assert_eq!(true, adsr.has_next());
        assert_eq!(0x0000, adsr.get_next().to_i32());

        // End state.  Report silence and no more data
        assert_eq!(false, adsr.has_next());
        assert_eq!(0x0000, adsr.get_next().to_i32());
        assert_eq!(false, adsr.has_next());
    }
}
