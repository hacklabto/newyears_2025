use crate::sound_sample::SoundSample;
use crate::sound_sample::SoundScale;
use crate::sound_source::SoundSource;
use crate::sound_source_core::SoundSourceCore;
use crate::sound_source_msgs::SoundSourceAdsrInit;
use crate::sound_source_msgs::SoundSourceMsg;
use crate::sound_source_msgs::SoundSourceMsgs;
use crate::sound_source_msgs::SoundSourceValue;
use crate::sound_sources::SoundSources;
use core::marker::PhantomData;

#[allow(unused)]
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
pub struct CoreAdsr<T: SoundSample, const PLAY_FREQUENCY: u32> {
    state: AdsrState,              // CurrentState
    attack_max_volume: SoundScale, // Reduction in volume after attack finishes
    sustain_volume: SoundScale,    // Reduction in volume during sustain phase
    a: u32,                        // timed, units are 1/PLAY_FREQUENCY
    d: u32,                        // timed, units are 1/PLAY_FREQUENCY
    r: u32,                        // timed, units are 1/PLAY_FREQUENCY
    time_since_state_start: u32,   // units are 1/PLAY_FREQUENCY
    _marker: PhantomData<T>,
}

impl<T: SoundSample, const PLAY_FREQUENCY: u32> SoundSourceCore<'_, T, PLAY_FREQUENCY>
    for CoreAdsr<T, PLAY_FREQUENCY>
{
    fn get_next(self: &Self) -> T {
        let scale: T = match self.state {
            AdsrState::Attack => {
                let mut attack_value =
                    T::new((self.time_since_state_start * 0x7fff / self.a + 0x8000) as u16);
                attack_value.scale(self.attack_max_volume);
                attack_value
            }
            AdsrState::Delay => {
                let mut attack_contribution = T::new(
                    ((self.d - self.time_since_state_start) * 0x7fff / self.d + 0x8000) as u16,
                );
                let mut sustain_contribution =
                    T::new(((self.time_since_state_start) * 0x7fff / self.d + 0x8000) as u16);
                attack_contribution.scale(self.attack_max_volume);
                sustain_contribution.scale(self.sustain_volume);
                T::new(
                    (attack_contribution.to_u16() - 0x8000)
                        + (sustain_contribution.to_u16() - 0x8000)
                        + 0x8000,
                )
            }
            AdsrState::Sustain => {
                let mut sustain_contribution = T::new(0xffff);
                sustain_contribution.scale(self.sustain_volume);
                sustain_contribution
            }
            AdsrState::Release => {
                let mut release_value = T::new(
                    ((self.r - self.time_since_state_start) * 0x7fff / self.r + 0x8000) as u16,
                );
                release_value.scale(self.sustain_volume);
                release_value
            }
            AdsrState::Ended => T::new(0x8000),
        };

        scale
    }

    fn has_next(self: &Self) -> bool {
        self.state != AdsrState::Ended
    }

    fn update(&mut self) {
        //let mut rerun_update = false;
        self.time_since_state_start = self.time_since_state_start + 1;
        match self.state {
            AdsrState::Attack => {
                if self.time_since_state_start >= self.a {
                    self.time_since_state_start = 0;
                    self.state = AdsrState::Delay;
                    //rerun_update = true;
                }
            }
            AdsrState::Delay => {
                if self.time_since_state_start >= self.d {
                    self.time_since_state_start = 0;
                    self.state = AdsrState::Sustain;
                    //rerun_update = true;
                }
            }
            AdsrState::Sustain => {}
            AdsrState::Release => {
                if self.time_since_state_start > self.r {
                    self.time_since_state_start = 0;
                    self.state = AdsrState::Ended;
                    //rerun_update = true;
                }
            }
            AdsrState::Ended => {}
        }
    }
}

impl<T: SoundSample, const PLAY_FREQUENCY: u32> CoreAdsr<T, PLAY_FREQUENCY> {
    pub fn init(self: &mut Self, init_values: &SoundSourceAdsrInit) {
        self.state = AdsrState::Attack;
        self.attack_max_volume = init_values.attack_max_volume;
        self.a = init_values.a;
        self.d = init_values.d;
        self.sustain_volume = init_values.sustain_volume;
        self.r = init_values.r;
        self.time_since_state_start = 0;
    }
    pub fn trigger_release(self: &mut Self) {
        // TODO, What if we aren't in sustain?  Probably I should take
        // the current volume and run the release on that.
        self.state = AdsrState::Release;
        self.time_since_state_start = 0;
    }
}

pub struct GenericAdsr<T: SoundSample, const PLAY_FREQUENCY: u32> {
    core: CoreAdsr<T, PLAY_FREQUENCY>,
}

impl<T: SoundSample, const PLAY_FREQUENCY: u32> Default for CoreAdsr<T, PLAY_FREQUENCY> {
    fn default() -> Self {
        let state = AdsrState::Ended;
        let attack_max_volume = SoundScale::default();
        let a = PLAY_FREQUENCY / 8;
        let d = PLAY_FREQUENCY / 3;
        let sustain_volume = SoundScale::default();
        let r = PLAY_FREQUENCY / 5;
        let time_since_state_start = 0;

        Self {
            state,
            attack_max_volume,
            a,
            d,
            sustain_volume,
            r,
            time_since_state_start,
            _marker: PhantomData {},
        }
    }
}

impl<T: SoundSample, const PLAY_FREQUENCY: u32> Default for GenericAdsr<T, PLAY_FREQUENCY> {
    fn default() -> Self {
        return Self {
            core: CoreAdsr::default(),
        };
    }
}

#[allow(unused)]
impl<T: SoundSample, const PLAY_FREQUENCY: u32> SoundSource<'_, T, PLAY_FREQUENCY>
    for GenericAdsr<T, PLAY_FREQUENCY>
{
    fn get_next(self: &Self, _all_sources: &dyn SoundSources<T, PLAY_FREQUENCY>) -> T {
        self.core.get_next()
    }

    fn has_next(self: &Self, _all_sources: &dyn SoundSources<T, PLAY_FREQUENCY>) -> bool {
        self.core.has_next()
    }

    fn update(&mut self, new_msgs: &mut SoundSourceMsgs) {
        self.core.update()
    }

    fn handle_msg(&mut self, msg: &SoundSourceMsg, new_msgs: &mut SoundSourceMsgs) {
        match &msg.value {
            SoundSourceValue::AdsrInit { init_values } => {
                self.core.init(&init_values);

                let creation_msg = SoundSourceMsg::new(
                    msg.src_id.clone(),
                    msg.dest_id.clone(),
                    SoundSourceValue::SoundSourceCreated,
                );
                new_msgs.append(creation_msg);
            }
            SoundSourceValue::ReleaseAdsr => {
                // TODO, What if we aren't in sustain?  Probably I should take
                // the current volume and run the release on that.
                self.core.state = AdsrState::Release;
                self.core.time_since_state_start = 0;
            }
            _ => todo!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::adsr::*;
    use crate::sound_sample::SoundSampleI32;

    #[test]
    fn basic_adsr_test() {
        let adsr_init = SoundSourceAdsrInit::new(
            SoundScale::new_percent(100),
            SoundScale::new_percent(50),
            2,
            4,
            4,
        );

        let mut adsr = CoreAdsr::<SoundSampleI32, 24000>::default();
        adsr.init(&adsr_init);

        // Attack state, 2 ticks to get to attack volume (max) from 0
        assert_eq!(0x8000, adsr.get_next().to_u16());
        adsr.update();
        assert_eq!(0xbfff, adsr.get_next().to_u16());
        adsr.update();
        assert_eq!(0xffff, adsr.get_next().to_u16());

        // Delay state, 4 ticks to get to Sustain Volume (50%) from attack volume
        adsr.update();
        assert_eq!(0xeffe, adsr.get_next().to_u16());
        adsr.update();
        assert_eq!(0xdffe, adsr.get_next().to_u16());
        adsr.update();
        assert_eq!(0xcffe, adsr.get_next().to_u16());
        adsr.update();
        assert_eq!(0xbfff, adsr.get_next().to_u16());

        // Sustain state
        adsr.update();
        assert_eq!(0xbfff, adsr.get_next().to_u16());
        adsr.update();
        assert_eq!(0xbfff, adsr.get_next().to_u16());
        adsr.update();
        assert_eq!(0xbfff, adsr.get_next().to_u16());

        adsr.trigger_release();

        // Release state, 4 ticks to get to quiet from Sustain Volume
        adsr.update();
        assert_eq!(0xafff, adsr.get_next().to_u16());
        adsr.update();
        assert_eq!(0x9fff, adsr.get_next().to_u16());
        adsr.update();
        assert_eq!(0x8fff, adsr.get_next().to_u16());
        adsr.update();
        assert_eq!(0x8000, adsr.get_next().to_u16());
        assert_eq!(true, adsr.has_next());

        // End state.  Report silence and no more data
        adsr.update();
        assert_eq!(0x8000, adsr.get_next().to_u16());
        assert_eq!(false, adsr.has_next());
    }
}
