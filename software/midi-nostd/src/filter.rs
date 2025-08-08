// Frequency filter using fixed point math.

use crate::sound_sample::SoundSampleI32;
use crate::sound_source_core::OscillatorInterface;
use crate::sound_source_core::SoundSourceCore;

use softfloat::F32;

//
// Multiply two fixed point numbers with 15 bits of precision.
// I'm guaranteeing that one of the inputs will be from -1 to 1 and
// the the 2nd will be from -2 to 2, which gives just enough room to
// avoid an overflow.
//
// Rust debug will check for overflows as an extra guard rail.  Rust
// release does not.
//
#[inline]
pub const fn fixp_mul(a: i32, b: i32) -> i32 {
    (a * b) >> 15
}

//
// Const compatible standard Tan function using a software floating
// point library.
//
const fn const_tan(angle_native: f32) -> f32 {
    let angle = F32::from_native_f32(angle_native);
    angle.sin().div(angle.cos()).to_native_f32()
}

//
// Compute butterworth filter co-efficients for a 2nd order filter at compile time.
// Returns (B0, B1, B2, A0, A1).
//
pub const fn lowpass_butterworth(cutoff_freq: i64, sample_freq: i64) -> (f32, f32, f32, f32, f32) {
    let ratio: f32 = (cutoff_freq as f32) / (sample_freq as f32);
    if ratio > 0.19 {
        //
        // I only support a cut-off frequency that's 19% the sample frequency.
        // If you want something faster, the filter will treat this value as
        // a pass through.  This could be increased, but I'd start losing
        // precision. And that might be the right trade off if I'm finding I need
        // to bump down the payback rate to something like 12000hz to get this
        // working in typical 2025 embedded devices like the RPi Pico.
        //
        // - glowmouse, August 2025.
        //
        return (0.0, 0.0, 0.0, 0.0, 0.0);
    }
    let k: f32 = const_tan(ratio * core::f32::consts::PI);
    let k_squared: f32 = k * k;
    const SQRT2: f32 = F32::from_native_f32(2.0).sqrt().to_native_f32();
    let a0_denom = 1.0 + SQRT2 * k + k_squared;
    let a1_numerator: f32 = 2.0 * (k_squared - 1.0); // range -2 to .-.88.  Often around -2
    let a1: f32 = a1_numerator / a0_denom; // should have just enough head room
    let a2_numerator: f32 = 1.0 - SQRT2 * k + k_squared; // range 1 to .53.
    let a2: f32 = a2_numerator / a0_denom;

    let b0: f32 = k_squared / a0_denom;
    let b1: f32 = b0 * 2.0;
    let b2: f32 = b0;

    return (b0, b1, b2, a1, a2);
}

//
// A container for the three filter parameters I use in my filter.
//
// b1    - the b1 filter co-efficient.  b0 and b2 are just half b1, so I don't store them.
// a1    - the a1 filter co-efficient,
// a2    - the a2 filter co-efficient.
//
#[derive(Copy, Clone)]
struct FilterParams {
    b1: i32,
    a1: i32,
    a2: i32,
}

impl FilterParams {
    const fn new(cutoff_frequency: u32, sample_frequency: u32) -> Self {
        let raw_params = lowpass_butterworth(cutoff_frequency as i64, sample_frequency as i64);
        const ONE: f32 = (1i32<< 15) as f32;
        Self {
            b1: (raw_params.1 * ONE) as i32,
            a1: (raw_params.3 * ONE) as i32,
            a2: (raw_params.4 * ONE) as i32,
        }
    }
    const fn const_default() -> Self {
        Self {
            b1: 0,
            a1: 0,
            a2: 0,
        }
    }
}

//
// Precomputes an array of filter parameters at compile time.
//
const fn build_filter_param_array<const N: usize>(
    max_cutoff: u32,
    sample_freq: u32,
) -> [FilterParams; N] {
    let mut filter_params = [FilterParams::const_default(); N];
    let mut idx: usize = 1;

    while idx < N - 1 {
        filter_params[idx] = FilterParams::new(max_cutoff * (idx as u32) / (N as u32), sample_freq);
        idx = idx + 1;
    }

    return filter_params;
}

pub struct Filter<const P_FREQ: u32, const U_FREQ: u32, Source: OscillatorInterface<P_FREQ, U_FREQ>>
{
    source: Source,
    d1: i32,
    d2: i32,
    params: &'static FilterParams,
}

impl<const P_FREQ: u32, const U_FREQ: u32, Source: OscillatorInterface<P_FREQ, U_FREQ>>
    Filter<P_FREQ, U_FREQ, Source>
{
    //
    // For building an array of filter parameter constants
    //

    // The size of the array.  This is going into flash on most embedded devices,
    // so 300 seems reasonable.
    //
    const FILTER_PARAMS_ARRAY_SIZE: usize = 300;

    // What cut off freqeuncy gets shiftedby to get th the desired filter param table entry
    //
    const FILTER_CUTOFF_TO_TABLE_ENTRY_SHIFT: u32 = 4;

    // What cut off freqeuncy gets divided by to get th the desired filter param table entry
    //
    const FILTER_CUTOFF_TO_TABLE_ENTRY_DIVIDE: u32 =
        (1 << Self::FILTER_CUTOFF_TO_TABLE_ENTRY_SHIFT);

    // Maximum supported cutoff frequency.
    //
    const FILTER_PARAMS_MAX_CUTOFF_FREQUENCY: u32 =
        ((Self::FILTER_PARAMS_ARRAY_SIZE as u32) * Self::FILTER_CUTOFF_TO_TABLE_ENTRY_DIVIDE);

    // The actual table of filter parameters
    //
    const FILTER_PARAMS: [FilterParams; 300] =
        build_filter_param_array::<300>(Self::FILTER_PARAMS_MAX_CUTOFF_FREQUENCY, P_FREQ);

    // Helper function to go from a cutoff frequency to the parameter table entry
    // for that frequency.
    //
    fn freq_to_filter_param(cutoff_frequency: u32) -> &'static FilterParams {
        let idx = core::cmp::min(
            (cutoff_frequency >> Self::FILTER_CUTOFF_TO_TABLE_ENTRY_SHIFT) as usize,
            Self::FILTER_PARAMS.len() - 1,
        );
        &(Self::FILTER_PARAMS[idx])
    }
}

impl<const P_FREQ: u32, const U_FREQ: u32, Source: OscillatorInterface<P_FREQ, U_FREQ>>
    SoundSourceCore<P_FREQ, U_FREQ> for Filter<P_FREQ, U_FREQ, Source>
{
    type InitValuesType = (Source::InitValuesType, u32);

    fn new(init_values: Self::InitValuesType) -> Self {
        return Self {
            source: Source::new(init_values.0),
            d1: 0,
            d2: 0,
            params: Self::freq_to_filter_param(init_values.1),
        };
    }

    #[inline]
    fn get_next(self: &mut Self) -> SoundSampleI32 {
        if self.params.b1 == 0 {
            // Special case,
            //
            // If the filter frequency is more than 20% of the playback frequency,
            // just do a pass-through.  See the filter co-efficient code for the
            // other side of that logic.
            //
            // This code is hit when the midi player estimates the midi's loudness
            // by "fast forwarding" throgh the song at high speed and looking for
            // the biggest amplitude values.  The expected amplitude reduction
            // from the filter is
            //
            //              1
            // -------------------------------------------
            // sqrt( 1 + (note_frequncy / cutoff_freuency) ^ 2 )
            //
            // So this can result in the volume being substantially under-estimated.
            // TODO - put together a fixed point friendly version of this formula
            // and provide a hint in the filter fixes this.
            //
            self.source.get_next()
        } else {
            let input = self.source.get_next().to_i32();

            // Compute input * B0, input * B1, input * B2.
            //
            // I'm using 32 bit numbers here, which is a bit sketchy.
            // If I have a 400hz filter and a 24000hz playback, b0 will be about
            // 0.0027137.  The input will be from -(1<<15) to (1<<15), so 
            // b0_input_term will be from -88 to 88.  That's a lot of precision loss
            // on the values going into d1 and d2.  And yet it seems to sound okay.
            //
            // 64 bit fixed point would be better, but the CPU cost on 32 bit processors
            // is is bad, and I'm having trouble keeping the DMA buffer filled, so go with
            // this for now.  If it becomes a problem, I could use a 40 bit hybrid fixed
            // point for extra precision.
            //
            let b1_input_term = fixp_mul(input, self.params.b1);
            // B0 = B1/2, B2 = B1/2, so just take the b1 input term and divide by 2
            let b0_input_term = b1_input_term >> 1;
            let b2_input_term = b0_input_term;

            //
            // Compute output, output * a1, output * a2
            //
            let output: i32 = b0_input_term + self.d1;
            let a1_output_term = fixp_mul(output, self.params.a1);
            let a2_output_term = fixp_mul(output, self.params.a2);

            // Record d1 and d2, then return the output
            //
            self.d1 = self.d2 + b1_input_term - a1_output_term;
            self.d2 = b2_input_term - a2_output_term;
            SoundSampleI32::new_i32(output)
        }
    }

    fn update(self: &mut Self) {
        self.source.update();
    }

    fn restart(self: &mut Self, vel: u8) {
        self.source.restart(vel);
    }

    fn has_next(self: &Self) -> bool {
        self.source.has_next()
    }

    fn trigger_note_off(self: &mut Self) {
        self.source.trigger_note_off();
    }
}

impl<const P_FREQ: u32, const U_FREQ: u32, Source: OscillatorInterface<P_FREQ, U_FREQ>>
    OscillatorInterface<P_FREQ, U_FREQ> for Filter<P_FREQ, U_FREQ, Source>
{
    fn set_amplitude_adjust(self: &mut Self, adjust: SoundSampleI32) {
        self.source.set_amplitude_adjust(adjust);
    }
}

#[cfg(test)]
mod tests {
    use crate::filter::*;
    use crate::midi_notes::FREQUENCY_MULTIPLIER;
    use crate::oscillator::*;

    //
    // Helper function to get the average amplitude of some sound source.
    // Used to figure out if the filter is actually working.
    //
    fn get_avg_amplitude<T>(source: &mut T) -> (i32, i32)
    where
        T: SoundSourceCore<24000, 24000>,
    {
        let mut amplitude_sum: i32 = 0;
        let mut switches: i32 = 0;
        let mut last = 0;
        let samples = 24000;

        for _ in 0..samples {
            let sample = source.get_next().to_i32();
            let abs_sample = if sample > 0 { sample } else { -sample };
            amplitude_sum += abs_sample;
            if last >= 0 && sample < 0 {
                switches = switches + 1
            }
            last = sample
        }
        (amplitude_sum / samples, switches)
    }

    #[test]
    fn filter_behavior_400() {
        type Oscillator = CoreOscillator<24000, 24000, 50, 100, { OscillatorType::Sine as usize }>;
        type FilteredOscillator = Filter<24000, 24000, Oscillator>;

        let mut sine_50hz = Oscillator::new(50 * FREQUENCY_MULTIPLIER);
        let mut filtered_sine_50hz = FilteredOscillator::new((50 * FREQUENCY_MULTIPLIER, 400));

        // Unfiltered amplitude should be about 2/pi, or 20861 in 31 bit fixed point.
        // 50hz is below the 400hz cut off, so the average filtered amplitude should be similar.
        //
        assert_eq!((10430, 50), get_avg_amplitude(&mut sine_50hz));
        assert_eq!((10425, 50), get_avg_amplitude(&mut filtered_sine_50hz));

        let mut sine_100hz = Oscillator::new(100 * FREQUENCY_MULTIPLIER);
        let mut filtered_sine_100hz = FilteredOscillator::new((100 * FREQUENCY_MULTIPLIER, 400));
        assert_eq!((10430, 100), get_avg_amplitude(&mut sine_100hz));
        assert_eq!((10411, 100), get_avg_amplitude(&mut filtered_sine_100hz));

        let mut sine_200hz = Oscillator::new(200 * FREQUENCY_MULTIPLIER);
        let mut filtered_sine_200hz = FilteredOscillator::new((200 * FREQUENCY_MULTIPLIER, 400));
        assert_eq!((10430, 200), get_avg_amplitude(&mut sine_200hz));
        assert_eq!((10116, 200), get_avg_amplitude(&mut filtered_sine_200hz));

        // This is the cut-off frequency.  The filtered average amplitude should be 1/sqrt(2)
        // of the original average amplitude, or 70.71%.  We're getting 70.72%.
        //
        let mut sine_400hz = Oscillator::new(400 * FREQUENCY_MULTIPLIER);
        let mut filtered_sine_400hz = FilteredOscillator::new((400 * FREQUENCY_MULTIPLIER, 400));
        assert_eq!((10431, 400), get_avg_amplitude(&mut sine_400hz));
        assert_eq!((7361, 400), get_avg_amplitude(&mut filtered_sine_400hz));

        let mut sine_800hz = Oscillator::new(800 * FREQUENCY_MULTIPLIER);
        let mut filtered_sine_800hz = FilteredOscillator::new((800 * FREQUENCY_MULTIPLIER, 400));
        assert_eq!((10404, 800), get_avg_amplitude(&mut sine_800hz));
        assert_eq!((2520, 800), get_avg_amplitude(&mut filtered_sine_800hz));

        let mut sine_1600hz = Oscillator::new(1600 * FREQUENCY_MULTIPLIER);
        let mut filtered_sine_1600hz = FilteredOscillator::new((1600 * FREQUENCY_MULTIPLIER, 400));
        assert_eq!((10404, 1600), get_avg_amplitude(&mut sine_1600hz));
        assert_eq!((630, 1599), get_avg_amplitude(&mut filtered_sine_1600hz));
    }

    #[test]
    fn filter_behavior_1600() {
        type Oscillator = CoreOscillator<24000, 24000, 50, 100, { OscillatorType::Sine as usize }>;
        type FilteredOscillator = Filter<24000, 24000, Oscillator>;

        let mut sine_50hz = Oscillator::new(50 * FREQUENCY_MULTIPLIER);
        let mut filtered_sine_50hz = FilteredOscillator::new((50 * FREQUENCY_MULTIPLIER, 1600));

        // Unfiltered amplitude should be about 2/pi, or 20861 in 31 bit fixed point.
        // 50hz is below the 1600hz cut off, so the average filtered amplitude should be similar.
        //
        assert_eq!((10430, 50), get_avg_amplitude(&mut sine_50hz));
        assert_eq!((10430, 50), get_avg_amplitude(&mut filtered_sine_50hz));

        let mut sine_100hz = Oscillator::new(100 * FREQUENCY_MULTIPLIER);
        let mut filtered_sine_100hz = FilteredOscillator::new((100 * FREQUENCY_MULTIPLIER, 1600));
        assert_eq!((10430, 100), get_avg_amplitude(&mut sine_100hz));
        assert_eq!((10430, 100), get_avg_amplitude(&mut filtered_sine_100hz));

        let mut sine_200hz = Oscillator::new(200 * FREQUENCY_MULTIPLIER);
        let mut filtered_sine_200hz = FilteredOscillator::new((200 * FREQUENCY_MULTIPLIER, 1600));
        assert_eq!((10430, 200), get_avg_amplitude(&mut sine_200hz));
        assert_eq!((10430, 200), get_avg_amplitude(&mut filtered_sine_200hz));

        let mut sine_400hz = Oscillator::new(400 * FREQUENCY_MULTIPLIER);
        let mut filtered_sine_400hz = FilteredOscillator::new((400 * FREQUENCY_MULTIPLIER, 1600));
        assert_eq!((10431, 400), get_avg_amplitude(&mut sine_400hz));
        assert_eq!((10417, 400), get_avg_amplitude(&mut filtered_sine_400hz));

        let mut sine_800hz = Oscillator::new(800 * FREQUENCY_MULTIPLIER);
        let mut filtered_sine_800hz = FilteredOscillator::new((800 * FREQUENCY_MULTIPLIER, 1600));
        assert_eq!((10404, 800), get_avg_amplitude(&mut sine_800hz));
        assert_eq!((10148, 800), get_avg_amplitude(&mut filtered_sine_800hz));

        // This is the cut-off frequency.  The filtered average amplitude should be 1/sqrt(2)
        // of the original average amplitude, or 70.71%.  We're getting 71.12%.
        //
        let mut sine_1600hz = Oscillator::new(1600 * FREQUENCY_MULTIPLIER);
        let mut filtered_sine_1600hz = FilteredOscillator::new((1600 * FREQUENCY_MULTIPLIER, 1600));
        assert_eq!((10404, 1600), get_avg_amplitude(&mut sine_1600hz));
        assert_eq!((7385, 1600), get_avg_amplitude(&mut filtered_sine_1600hz));
    }
    #[test]
    fn filter_behavior_24000() {
        // This should give us a simple pass through.
        type Oscillator = CoreOscillator<24000, 24000, 50, 100, { OscillatorType::Sine as usize }>;
        type FilteredOscillator = Filter<24000, 24000, Oscillator>;

        let mut sine_50hz = Oscillator::new(50 * FREQUENCY_MULTIPLIER);
        let mut filtered_sine_50hz = FilteredOscillator::new((50 * FREQUENCY_MULTIPLIER, 24000));

        // The average amplitude should match, it's a pass through
        //
        assert_eq!((10430, 50), get_avg_amplitude(&mut sine_50hz));
        assert_eq!((10430, 50), get_avg_amplitude(&mut filtered_sine_50hz));

        // A silly oscillator for a silly filter.  The measured frequency is 2000
        // hz because of aliasing.
        //
        let mut sine_22000hz = Oscillator::new(22000 * FREQUENCY_MULTIPLIER);
        let mut filtered_sine_22000hz =
            FilteredOscillator::new((22000 * FREQUENCY_MULTIPLIER, 24000));
        assert_eq!((10209, 2001), get_avg_amplitude(&mut sine_22000hz));
        assert_eq!((10209, 2001), get_avg_amplitude(&mut filtered_sine_22000hz));
    }
}
