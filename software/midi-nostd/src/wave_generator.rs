use crate::midi_notes::FREQUENCY_MULTIPLIER;
use crate::sound_sample::SoundSample;
use crate::sound_sample::SoundSampleI32;
use crate::sound_sample::SoundScale;
use crate::sound_source::SoundSource;
use crate::sound_source_id::SoundSourceId;
use crate::sound_source_id::SoundSourceType;
use crate::sound_source_msgs::SoundSourceKey;
use crate::sound_source_msgs::SoundSourceMsg;
use crate::sound_source_msgs::SoundSourceMsgs;
use crate::sound_source_msgs::SoundSourceValue;
use crate::sound_source_msgs::WaveType;
use crate::sound_source_pool_impl::GenericSoundPool;
use crate::sound_sources::SoundSources;
use crate::wave_tables::SAWTOOTH_WAVE;
use crate::wave_tables::SINE_WAVE;
use crate::wave_tables::SQUARE_WAVE;
use crate::wave_tables::TRIANGLE_WAVE;

use crate::wave_tables::WAVE_TABLE_SIZE;
use crate::wave_tables::WAVE_TABLE_SIZE_U32;
use core::marker::PhantomData;

const ALL_WAVE_TABLES: [&[u16; WAVE_TABLE_SIZE]; 4] =
    [&TRIANGLE_WAVE, &SAWTOOTH_WAVE, &SINE_WAVE, &SQUARE_WAVE];

///
/// Wave source generic for a sample type and frequency
///
pub struct GenericWaveSource<T: SoundSample, const PLAY_FREQUENCY: u32> {
    volume: SoundScale,
    wave_type: WaveType,
    pulse_width_cutoff: u32,
    table_idx: u32,
    table_remainder: u32,
    table_idx_inc: u32,
    table_remainder_inc: u32,
    _marker: PhantomData<T>,
}
//impl<T: SoundSample, const PLAY_FREQUENCY: u32> Drop for GenericWaveSource<T, PLAY_FREQUENCY> {
//    fn drop(&mut self) {}
//}

impl<T: SoundSample, const PLAY_FREQUENCY: u32> Default for GenericWaveSource<T, PLAY_FREQUENCY> {
    fn default() -> Self {
        let volume = SoundScale::new_percent(100); // full volume
        let wave_type = WaveType::PulseWidth;
        let pulse_width_cutoff: u32 = WAVE_TABLE_SIZE_U32 / 2; // 50% duty cycle by default
        let table_idx_inc: u32 = 0;
        let table_remainder_inc: u32 = 0;
        let table_idx: u32 = 0;
        let inc_denominator: u32 = FREQUENCY_MULTIPLIER * PLAY_FREQUENCY;
        let table_remainder: u32 = inc_denominator / 2;
        Self {
            volume,
            wave_type,
            pulse_width_cutoff,
            table_idx,
            table_remainder,
            table_idx_inc,
            table_remainder_inc,
            _marker: PhantomData {},
        }
    }
}

impl<T: SoundSample, const PLAY_FREQUENCY: u32> GenericWaveSource<T, PLAY_FREQUENCY> {
    pub fn init(self: &mut Self, wave_type: WaveType, arg_sound_frequency: u32) {
        let volume = SoundScale::new_percent(100); // full volume
        let pulse_width_cutoff: u32 = WAVE_TABLE_SIZE_U32 / 2; // 50% duty cycle by default
                                                               // I want (arg_sound_frequency * WAVE_TABLE_SIZE) / (FREQUNCY_MULTIPLIER * PLAY_FREQUENCY);
        let inc_numerator: u32 = arg_sound_frequency * WAVE_TABLE_SIZE_U32;
        let inc_denominator: u32 = FREQUENCY_MULTIPLIER * PLAY_FREQUENCY;
        let table_idx_inc: u32 = inc_numerator / inc_denominator;
        let table_remainder_inc: u32 = inc_numerator % inc_denominator;
        let table_idx: u32 = 0;
        let table_remainder: u32 = inc_denominator / 2;
        *self = Self {
            volume,
            wave_type,
            pulse_width_cutoff,
            table_idx,
            table_remainder,
            table_idx_inc,
            table_remainder_inc,
            _marker: PhantomData {},
        }
    }

    // Read sample from table that has the wave's amplitude values
    //
    // The basic idea of this function is that we're going through the table
    // at some rate (i.e., N entries per second, where N might be a fractional
    // value).  If we go through the table faster, we play the wave back at a
    // higher frequeny.  Slower gives a lower frequency.
    //
    // The rate that we're going through the table is represented by
    //
    // self_table_idx_inc + self.table_remainder_inc / inc_denominator
    //
    // Where 0 <= self.table_remainder_inc/ inc_demonimator < 1 is always true.
    //
    // The position in the table is tracked by self.table_idx, but there's always
    // some fractional left over value.  That value is tracked by
    // self.table_remainder, and has a "real" value of
    //
    // self.table_remainder / inc_denominator, which is always [0..1) when the
    // function exits.
    //

    fn update_table_index(&mut self) {
        // Update table position and fractional value
        //
        self.table_idx += self.table_idx_inc;
        self.table_remainder += self.table_remainder_inc;
        let inc_denominator: u32 = FREQUENCY_MULTIPLIER * PLAY_FREQUENCY;

        // If the fractional value represents a number greater than 1, increment
        // the table index and decease the fractional value so it's [0..1).
        //
        if self.table_remainder > inc_denominator {
            self.table_remainder -= inc_denominator;
            self.table_idx += 1;
        }
        self.table_idx = self.table_idx & (WAVE_TABLE_SIZE_U32 - 1);
    }

    fn get_next_table(&self, table: &[u16; WAVE_TABLE_SIZE]) -> T {
        let mut rval = T::new(table[self.table_idx as usize]);
        rval.scale(self.volume);
        rval
    }

    fn get_next_pulse_entry(&self) -> T {
        let mut rval = if self.table_idx < self.pulse_width_cutoff {
            T::max()
        } else {
            T::min()
        };
        rval.scale(self.volume);
        rval
    }
}

pub fn set_wave_properties(
    all_pools: &mut dyn SoundSources<SoundSampleI32, 24000>,
    wave_id: &SoundSourceId,
    wave_type: WaveType,
    frequency: u32,
    pulse_width: u8,
    volume: u8,
) {
    let mut msgs = SoundSourceMsgs::default();
    msgs.append(SoundSourceMsg::new(
        wave_id.clone(),
        SoundSourceKey::Frequency,
        SoundSourceValue::new_u32(frequency * FREQUENCY_MULTIPLIER),
    ));
    msgs.append(SoundSourceMsg::new(
        wave_id.clone(),
        SoundSourceKey::WaveType,
        SoundSourceValue::new_wave_type(wave_type),
    ));
    msgs.append(SoundSourceMsg::new(
        wave_id.clone(),
        SoundSourceKey::PulseWidth,
        SoundSourceValue::new_u8(pulse_width),
    ));
    msgs.append(SoundSourceMsg::new(
        wave_id.clone(),
        SoundSourceKey::Volume,
        SoundSourceValue::new_u8(volume),
    ));
    all_pools.process_and_clear_msgs(&mut msgs);
}

impl<T: SoundSample, const PLAY_FREQUENCY: u32> SoundSource<T, PLAY_FREQUENCY>
    for GenericWaveSource<T, PLAY_FREQUENCY>
{
    fn get_next(self: &Self, _all_sources: &dyn SoundSources<T, PLAY_FREQUENCY>) -> T {
        if self.wave_type == WaveType::PulseWidth {
            self.get_next_pulse_entry()
        } else {
            self.get_next_table(ALL_WAVE_TABLES[self.wave_type as usize])
        }
    }

    fn has_next(self: &Self, _all_sources: &dyn SoundSources<T, PLAY_FREQUENCY>) -> bool {
        true
    }

    fn update(&mut self, _new_msgs: &mut SoundSourceMsgs) {
        self.update_table_index();
    }

    fn set_attribute(&mut self, key: SoundSourceKey, value: SoundSourceValue) {
        if key == SoundSourceKey::Frequency {
            let inc_numerator: u32 = value.get_u32() * WAVE_TABLE_SIZE_U32;
            let inc_denominator: u32 = FREQUENCY_MULTIPLIER * PLAY_FREQUENCY;
            self.table_idx_inc = inc_numerator / inc_denominator;
            self.table_remainder_inc = inc_numerator % inc_denominator;
        }
        if key == SoundSourceKey::WaveType {
            self.wave_type = value.get_wave_type();
        }
        if key == SoundSourceKey::PulseWidth {
            let new_pulse_width_cutoff: u32 = WAVE_TABLE_SIZE_U32 * (value.get_u8() as u32) / 100;
            self.pulse_width_cutoff = new_pulse_width_cutoff;
        }
        if key == SoundSourceKey::Volume {
            self.volume = SoundScale::new_percent(value.get_u8());
        }
    }

    fn peer_sound_source(self: &Self) -> Option<SoundSourceId> {
        None
    }

    fn child_sound_source(self: &Self) -> Option<SoundSourceId> {
        None
    }
}

type WaveSource = GenericWaveSource<SoundSampleI32, 24000>;
pub type WavePool = GenericSoundPool<
    SoundSampleI32,
    24000,
    WaveSource,
    3,
    { SoundSourceType::WaveGenerator as usize },
>;

#[cfg(test)]
mod tests {
    use crate::sound_sources::SoundSources;
    use crate::sound_sources_impl::SoundSourcesImpl;
    use crate::wave_generator::*;

    fn abs_sample(sample: u16) -> u16 {
        if sample >= 0x8000 {
            sample - 0x8000
        } else {
            0x8000 - sample
        }
    }

    fn sample_wave(
        all_pools: &mut dyn SoundSources<SoundSampleI32, 24000>,
        wave_id: &SoundSourceId,
        new_msgs: &mut SoundSourceMsgs,
    ) -> (u32, u32) {
        all_pools.update(new_msgs);
        let mut last = all_pools.get_next(&wave_id);
        let mut transitions: u32 = 0;
        let mut area: u32 = abs_sample(last.to_u16()) as u32;
        for _ in 1..24000 {
            all_pools.update(new_msgs);
            let current = all_pools.get_next(&wave_id);
            let last_above_0 = last.to_u16() >= 0x8000;
            let current_above_0 = current.to_u16() >= 0x8000;
            if last_above_0 != current_above_0 {
                transitions = transitions + 1;
            }
            area = area + abs_sample(current.to_u16()) as u32;
            last = current;
        }
        (transitions, area)
    }
    #[test]
    fn test_pulse_50_from_pool() {
        let mut all_pools = SoundSourcesImpl::<SoundSampleI32, 24000, 3>::new();
        let wave_id = all_pools.alloc(SoundSourceType::WaveGenerator);
        set_wave_properties(
            &mut all_pools,
            &wave_id,
            WaveType::PulseWidth,
            2600,
            50,
            100,
        );
        let mut new_msgs = SoundSourceMsgs::default();
        let (transitions, area) = sample_wave(&mut all_pools, &wave_id, &mut new_msgs);

        assert_eq!(2600 * 2, transitions);

        assert_eq!(0x7fff * 12000 + 0x8000 * 12000, area);
        all_pools.free(wave_id);
    }

    #[test]
    fn test_pulse_50_vol_50_from_pool() {
        let mut all_pools = SoundSourcesImpl::<SoundSampleI32, 24000, 3>::new();
        let wave_id = all_pools.alloc(SoundSourceType::WaveGenerator);
        set_wave_properties(&mut all_pools, &wave_id, WaveType::PulseWidth, 2600, 50, 50);
        let mut new_msgs = SoundSourceMsgs::default();
        let (transitions, area) = sample_wave(&mut all_pools, &wave_id, &mut new_msgs);

        assert_eq!(2600 * 2, transitions);
        assert_eq!(0x3fff * 12000 + 0x4000 * 12000, area);
        all_pools.free(wave_id);
    }

    #[test]
    fn test_pulse_25_from_pool() {
        let mut all_pools = SoundSourcesImpl::<SoundSampleI32, 24000, 3>::new();
        let wave_id = all_pools.alloc(SoundSourceType::WaveGenerator);
        set_wave_properties(
            &mut all_pools,
            &wave_id,
            WaveType::PulseWidth,
            2600,
            25,
            100,
        );
        let mut new_msgs = SoundSourceMsgs::default();
        let (transitions, area) = sample_wave(&mut all_pools, &wave_id, &mut new_msgs);

        assert_eq!(2600 * 2, transitions); // we don't get the last transition in square.
        assert_eq!(0x7fff * 6000 + 0x8000 * 18000, area);
        all_pools.free(wave_id);
    }

    #[test]
    fn test_triangle_from_pool() {
        let mut all_pools = SoundSourcesImpl::<SoundSampleI32, 24000, 3>::new();
        let wave_id = all_pools.alloc(SoundSourceType::WaveGenerator);
        set_wave_properties(&mut all_pools, &wave_id, WaveType::Triangle, 2600, 0, 100);

        let mut new_msgs = SoundSourceMsgs::default();
        let (transitions, area) = sample_wave(&mut all_pools, &wave_id, &mut new_msgs);
        assert_eq!(transitions, 2600 * 2);
        // Triangles are half the area squares are.
        assert_eq!(12000 * 0x4000 + 12000 * 0x3fff, area);
        all_pools.free(wave_id);
    }

    #[test]
    fn test_triangle_from_pool_vol_50percent() {
        let mut all_pools = SoundSourcesImpl::<SoundSampleI32, 24000, 3>::new();
        let wave_id = all_pools.alloc(SoundSourceType::WaveGenerator);
        set_wave_properties(&mut all_pools, &wave_id, WaveType::Triangle, 2600, 0, 50);

        let mut new_msgs = SoundSourceMsgs::default();
        let (transitions, area) = sample_wave(&mut all_pools, &wave_id, &mut new_msgs);
        assert_eq!(transitions, 2600 * 2);
        // Triangles are half the area squares are.  200 is rounding error or a bug.
        assert_eq!(12000 * 0x2000 + 12000 * 0x1fff + 200, area);
        all_pools.free(wave_id);
    }
}
