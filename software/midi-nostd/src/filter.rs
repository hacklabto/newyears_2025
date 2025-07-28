use crate::sound_sample::SoundSampleI32;
use crate::sound_source_core::SoundSourceCore;

pub struct Filter<
    const PLAY_FREQUENCY: u32,
    Source: SoundSourceCore<PLAY_FREQUENCY>,
    const B0: i64,
    const B1: i64,
    const B2: i64,
> {
    source: Source,
    z1: i64,
    z2: i64,
}

impl<
        const PLAY_FREQUENCY: u32,
        Source: SoundSourceCore<PLAY_FREQUENCY>,
        const B0: i64,
        const B1: i64,
        const B2: i64,
    > SoundSourceCore<PLAY_FREQUENCY> for Filter<PLAY_FREQUENCY, Source, B0, B1, B2>
{
    type InitValuesType = Source::InitValuesType;

    fn new(init_values: &Self::InitValuesType) -> Self {
        let source = Source::new(&init_values);
        let z1 = 0;
        let z2 = 0;
        return Self { source, z1, z2 };
    }

    fn get_next(self: &mut Self) -> SoundSampleI32 {
        let raw_value = self.source.get_next().to_i32();
        let input = (raw_value as i64) << 16; // 31 bits of decimal precision
        let output: i64 =
            ((input * B0) >> 31) + self.z1 + ((self.z1 * B1) >> 31) + ((self.z2 * B2) >> 31);
        self.z2 = self.z1;
        self.z1 = output;
        let output_i32 = (output >> 16) as i32;
        SoundSampleI32::new_i32(output_i32)
    }

    fn has_next(self: &Self) -> bool {
        self.source.has_next()
    }

    fn trigger_note_off(self: &mut Self) {
        self.source.trigger_note_off();
    }
}
