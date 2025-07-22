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
impl<T: SoundSample, const PLAY_FREQUENCY: u32> SoundSource<'_, T, PLAY_FREQUENCY>
    for GenericAdsr<T, PLAY_FREQUENCY>
{
    fn get_next(self: &Self, _all_sources: &dyn SoundSources<T, PLAY_FREQUENCY>) -> T {
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
                if self.time_since_state_start > self.r {
                    self.time_since_state_start = 0;
                    self.state = AdsrState::Ended;
                    rerun_update = true;
                }
            }
            AdsrState::Ended => {}
        }
    }

    fn handle_msg(&mut self, msg: &SoundSourceMsg, new_msgs: &mut SoundSourceMsgs) {
        match &msg.value {
            SoundSourceValue::AdsrInit { init_values } => {
                self.state = AdsrState::Attack;
                self.attack_max_volume = init_values.attack_max_volume;
                self.a = init_values.a;
                self.d = init_values.d;
                self.sustain_volume = init_values.sustain_volume;
                self.r = init_values.r;
                self.time_since_state_start = 0;

                let creation_msg = SoundSourceMsg::new(
                    msg.src_id.clone(),
                    msg.dest_id.clone(),
                    SoundSourceKey::SoundSourceCreated,
                    SoundSourceValue::default(),
                );
                new_msgs.append(creation_msg);
            }
            SoundSourceValue::ReleaseAdsr => {
                // TODO, What if we aren't in sustain?  Probably I should take
                // the current volume and run the release on that.
                self.state = AdsrState::Release;
                self.time_since_state_start = 0;
            }
            _ => todo!(),
        }
    }
}

pub fn create_adsr(
    all_pools: &mut dyn SoundSources<SoundSampleI32, 24000>,
    init_values: SoundSourceAdsrInit,
) -> SoundSourceId {
    let mut msgs = SoundSourceMsgs::default();
    msgs.append(SoundSourceMsg::new(
        SoundSourceId::get_top_id(),
        SoundSourceId::get_top_id(),
        SoundSourceKey::Refactored,
        SoundSourceValue::AdsrInit { init_values },
    ));
    all_pools.process_and_clear_msgs(&mut msgs);

    all_pools
        .get_last_created_sound_source()
        .expect("Id should have been recorded")
        .clone()
}

#[cfg(test)]
mod tests {
    use crate::adsr::*;
    use crate::sound_sources::SoundSources;
    use crate::sound_sources_impl::SoundSourcesImpl;

    #[test]
    fn basic_adsr_test() {
        let mut all_pools = SoundSourcesImpl::<SoundSampleI32, 24000, 3, 3, 3>::default();
        let adsr_id = create_adsr(
            &mut all_pools,
            SoundSourceAdsrInit::new(
                SoundScale::new_percent(100),
                SoundScale::new_percent(50),
                2,
                4,
                4,
            ),
        );

        let mut new_msgs = SoundSourceMsgs::default();

        // Attack state, 2 ticks to get to attack volume (max) from 0
        assert_eq!(0x8000, all_pools.get_next(&adsr_id).to_u16());
        all_pools.update(&mut new_msgs);
        assert_eq!(0xbfff, all_pools.get_next(&adsr_id).to_u16());
        all_pools.update(&mut new_msgs);
        assert_eq!(0xffff, all_pools.get_next(&adsr_id).to_u16());

        // Delay state, 4 ticks to get to Sustain Volume (50%) from attack volume
        all_pools.update(&mut new_msgs);
        assert_eq!(0xeffe, all_pools.get_next(&adsr_id).to_u16());
        all_pools.update(&mut new_msgs);
        assert_eq!(0xdffe, all_pools.get_next(&adsr_id).to_u16());
        all_pools.update(&mut new_msgs);
        assert_eq!(0xcffe, all_pools.get_next(&adsr_id).to_u16());
        all_pools.update(&mut new_msgs);
        assert_eq!(0xbfff, all_pools.get_next(&adsr_id).to_u16());

        // Sustain state
        all_pools.update(&mut new_msgs);
        assert_eq!(0xbfff, all_pools.get_next(&adsr_id).to_u16());
        all_pools.update(&mut new_msgs);
        assert_eq!(0xbfff, all_pools.get_next(&adsr_id).to_u16());
        all_pools.update(&mut new_msgs);
        assert_eq!(0xbfff, all_pools.get_next(&adsr_id).to_u16());

        let mut msgs = SoundSourceMsgs::default();
        msgs.append(SoundSourceMsg::new(
            adsr_id.clone(),
            SoundSourceId::get_top_id(),
            SoundSourceKey::Refactored,
            SoundSourceValue::ReleaseAdsr,
        ));
        all_pools.process_and_clear_msgs(&mut msgs);

        // Release state, 4 ticks to get to quiet from Sustain Volume
        all_pools.update(&mut new_msgs);
        assert_eq!(0xafff, all_pools.get_next(&adsr_id).to_u16());
        all_pools.update(&mut new_msgs);
        assert_eq!(0x9fff, all_pools.get_next(&adsr_id).to_u16());
        all_pools.update(&mut new_msgs);
        assert_eq!(0x8fff, all_pools.get_next(&adsr_id).to_u16());
        all_pools.update(&mut new_msgs);
        assert_eq!(0x8000, all_pools.get_next(&adsr_id).to_u16());
        assert_eq!(true, all_pools.has_next(&adsr_id));

        // End state.  Report silence and no more data
        all_pools.update(&mut new_msgs);
        assert_eq!(0x8000, all_pools.get_next(&adsr_id).to_u16());
        assert_eq!(false, all_pools.has_next(&adsr_id));

        all_pools.free(adsr_id);
    }
}
