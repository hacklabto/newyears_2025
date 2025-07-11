use crate::sound_sample::SoundSample;
use crate::sound_sample::SoundSampleI32;
use crate::sound_sample::SoundScale;
use crate::sound_source::SoundSource;
use crate::sound_source_id::SoundSourceId;
use crate::sound_source_msgs::SoundSourceAdsrInit;
use crate::sound_source_msgs::SoundSourceKey;
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
#[allow(unused)]
pub struct GenericAdsr<T: SoundSample, const PLAY_FREQUENCY: u32> {
    state: AdsrState,              // CurrentState
    attack_max_volume: SoundScale, // Reduction in volume after attack finishes
    sustain_volume: SoundScale,    // Reduction in volume during sustain phase
    a: u32,                        // timed, units are 1/PLAY_FREQUENCY
    d: u32,                        // timed, units are 1/PLAY_FREQUENCY
    r: u32,                        // timed, units are 1/PLAY_FREQUENCY
    time_since_state_start: u32,   // units are 1/PLAY_FREQUENCY
    _marker: PhantomData<T>,
}

#[allow(unused)]
impl<T: SoundSample, const PLAY_FREQUENCY: u32> Default for GenericAdsr<T, PLAY_FREQUENCY> {
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

#[allow(unused)]
impl<T: SoundSample, const PLAY_FREQUENCY: u32> GenericAdsr<T, PLAY_FREQUENCY> {}

#[allow(unused)]
impl<T: SoundSample, const PLAY_FREQUENCY: u32> SoundSource<T, PLAY_FREQUENCY>
    for GenericAdsr<T, PLAY_FREQUENCY>
{
    fn get_next(self: &Self, _all_sources: &dyn SoundSources<T, PLAY_FREQUENCY>) -> T {
        let scale: T = match self.state {
            AdsrState::Attack => {
                let mut attack_value =
                    T::new((self.time_since_state_start * 0xffff / self.a) as u16);
                attack_value.scale(self.attack_max_volume);
                attack_value
            }
            AdsrState::Ended => {
                panic!("Agggggg!")
            }
            AdsrState::Delay | AdsrState::Sustain | AdsrState::Release => todo!(),
        };

        scale
    }

    fn has_next(self: &Self, _all_sources: &dyn SoundSources<T, PLAY_FREQUENCY>) -> bool {
        self.state != AdsrState::Ended
    }

    fn update(&mut self, new_msgs: &mut SoundSourceMsgs) {
        let mut rerun_update = false;
        self.time_since_state_start = self.time_since_state_start + 1;
        match self.state {
            AdsrState::Attack => {
                if self.time_since_state_start >= self.a {
                    self.time_since_state_start = 0;
                    self.state = AdsrState::Delay;
                    rerun_update = true;
                }
            }
            AdsrState::Delay => {
                if self.time_since_state_start >= self.d {
                    self.time_since_state_start = 0;
                    self.state = AdsrState::Sustain;
                    rerun_update = true;
                }
            }
            AdsrState::Sustain => {}
            AdsrState::Release => {
                if self.time_since_state_start >= self.r {
                    self.time_since_state_start = 0;
                    self.state = AdsrState::Ended;
                    rerun_update = true;
                }
            }
            AdsrState::Ended => {}
        }
    }

    fn set_attribute(&mut self, key: SoundSourceKey, value: SoundSourceValue) {}
}

pub fn set_adsr_properties(
    all_pools: &mut dyn SoundSources<SoundSampleI32, 24000>,
    adsr_id: SoundSourceId,
    init_values: SoundSourceAdsrInit,
) {
    let mut msgs = SoundSourceMsgs::default();
    msgs.append(SoundSourceMsg::new(
        adsr_id,
        SoundSourceKey::InitAdsr,
        SoundSourceValue::new_adsr_init(init_values),
    ));
    all_pools.process_and_clear_msgs(&mut msgs);
}

#[cfg(test)]
mod tests {
    use crate::adsr::*;
    use crate::sound_source_id::SoundSourceType;
    use crate::sound_sources::SoundSources;
    use crate::sound_sources_impl::SoundSourcesImpl;

    #[test]
    fn basic_adsr_test() {
        let mut all_pools = SoundSourcesImpl::<SoundSampleI32, 24000, 3, 3>::default();
        let adsr_id = all_pools.alloc(SoundSourceType::Adsr);

        set_adsr_properties(
            &mut all_pools,
            adsr_id,
            SoundSourceAdsrInit::new(
                SoundScale::new_percent(100),
                SoundScale::new_percent(50),
                2,
                3,
                4,
            ),
        );

        all_pools.free(adsr_id);
    }
}
