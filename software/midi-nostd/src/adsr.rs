use crate::sound_sample::SoundSampleI32;
use crate::sound_sample::SoundScale;
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
    const ATTACK_VOLUME_SCALE: SoundScale = SoundScale::new_percent(ATTACK_VOLUME);
    const SUSTAIN_VOLUME_SCALE: SoundScale = SoundScale::new_percent(SUSTAIN_VOLUME);
}

impl<
        const PLAY_FREQUENCY: u32,
        const A: i32,
        const D: i32,
        const R: i32,
        const ATTACK_VOLUME: u8,
        const SUSTAIN_VOLUME: u8,
    > SoundSourceCore<'_, PLAY_FREQUENCY>
    for CoreAdsr<PLAY_FREQUENCY, A, D, R, ATTACK_VOLUME, SUSTAIN_VOLUME>
{
    type InitValuesType = SoundSourceAdsrInit;

    fn init(self: &mut Self, _init_values: &Self::InitValuesType) {
        self.state = AdsrState::Attack;
        self.time_since_state_start = 0;
    }

    // TODO, the operation that I really want for a lot of these is something like,
    //
    // const ATTACK_VOLUME_SCALE:SoundSampleI32 = SoundSampleI32.new_percent( ATTACK_VOLUME);
    //
    // If I add an integer fraction_divide to SoundSampleI32 I get,
    //
    // match self.state {
    //   AdsrState::Attack => {
    //      SoundSampleI32::ATTACK_VOLUME_SCALED.fraction_divide( self.time_since_state_start, A);
    //   }
    //
    fn get_next(self: &Self) -> SoundSampleI32 {
        let scale: SoundSampleI32 = match self.state {
            AdsrState::Attack => {
                let mut attack_value =
                    SoundSampleI32::new_i32(self.time_since_state_start * 0x7fff / A);
                attack_value.scale(Self::ATTACK_VOLUME_SCALE);
                attack_value
            }
            AdsrState::Delay => {
                let mut attack_contribution =
                    SoundSampleI32::new_i32((D - self.time_since_state_start) * 0x7fff / D);
                let mut sustain_contribution =
                    SoundSampleI32::new_i32(self.time_since_state_start * 0x7fff / D);
                attack_contribution.scale(Self::ATTACK_VOLUME_SCALE);
                sustain_contribution.scale(Self::SUSTAIN_VOLUME_SCALE);
                attack_contribution + sustain_contribution
            }
            AdsrState::Sustain => {
                let mut sustain_contribution = SoundSampleI32::MAX;
                sustain_contribution.scale(Self::SUSTAIN_VOLUME_SCALE);
                sustain_contribution
            }
            AdsrState::Release => {
                let mut release_value =
                    SoundSampleI32::new_i32((R - self.time_since_state_start) * 0x7fff / R);
                release_value.scale(Self::SUSTAIN_VOLUME_SCALE);
                release_value
            }
            AdsrState::Ended => SoundSampleI32::ZERO,
        };

        scale
    }

    fn has_next(self: &Self) -> bool {
        self.state != AdsrState::Ended
    }

    fn update(&mut self) {
        self.time_since_state_start = self.time_since_state_start + 1;
        match self.state {
            AdsrState::Attack => {
                if self.time_since_state_start >= A {
                    self.time_since_state_start = 0;
                    self.state = AdsrState::Delay;
                }
            }
            AdsrState::Delay => {
                if self.time_since_state_start >= D {
                    self.time_since_state_start = 0;
                    self.state = AdsrState::Sustain;
                }
            }
            AdsrState::Sustain => {}
            AdsrState::Release => {
                if self.time_since_state_start > R {
                    self.time_since_state_start = 0;
                    self.state = AdsrState::Ended;
                }
            }
            AdsrState::Ended => {}
        }
    }
    fn trigger_note_off(self: &mut Self) {
        // TODO, What if we aren't in sustain?  Probably I should take
        // the current volume and run the release on that.
        self.state = AdsrState::Release;
        self.time_since_state_start = 0;
    }
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

        let mut adsr = CoreAdsr::<24000, 2, 4, 4, 100, 50>::default();
        adsr.init(&adsr_init);

        // Attack state, 2 ticks to get to attack volume (max) from 0
        assert_eq!(0x8000, adsr.get_next().to_u16());
        adsr.update();
        assert_eq!(0xbfff, adsr.get_next().to_u16());
        adsr.update();
        assert_eq!(0xffff, adsr.get_next().to_u16());
        adsr.update();

        // Delay state, 4 ticks to get to Sustain Volume (50%) from attack volume
        assert_eq!(0xeffe, adsr.get_next().to_u16());
        adsr.update();
        assert_eq!(0xdffe, adsr.get_next().to_u16());
        adsr.update();
        assert_eq!(0xcffe, adsr.get_next().to_u16());
        adsr.update();
        assert_eq!(0xbfff, adsr.get_next().to_u16());
        adsr.update();

        // Sustain state
        assert_eq!(0xbfff, adsr.get_next().to_u16());
        adsr.update();
        assert_eq!(0xbfff, adsr.get_next().to_u16());
        adsr.update();
        assert_eq!(0xbfff, adsr.get_next().to_u16());
        adsr.update();
        adsr.trigger_note_off(); // Release doesn't start until update begins
        assert_eq!(0xbfff, adsr.get_next().to_u16());
        adsr.update();

        // Release state, 4 ticks to get to quiet from Sustain Volume
        assert_eq!(0xafff, adsr.get_next().to_u16());
        adsr.update();
        assert_eq!(0x9fff, adsr.get_next().to_u16());
        adsr.update();
        assert_eq!(0x8fff, adsr.get_next().to_u16());
        adsr.update();
        assert_eq!(true, adsr.has_next());
        assert_eq!(0x8000, adsr.get_next().to_u16());
        adsr.update();

        // End state.  Report silence and no more data
        assert_eq!(false, adsr.has_next());
        assert_eq!(0x8000, adsr.get_next().to_u16());
        adsr.update();
        assert_eq!(false, adsr.has_next());
    }
}
