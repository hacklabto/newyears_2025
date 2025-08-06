// Frequency filter using fixed point math.

use crate::sound_sample::SoundSampleI32;
use crate::sound_source_core::OscillatorInterface;
use crate::sound_source_core::SoundSourceCore;

use softfloat::F32;

//
// Multiply two fixed point numbers with 31 bits of precision.
// Works well as long as but numbers are under 1, otherwise
// overflows.  Rust debug will check for overflows.
//
#[inline]
pub const fn fixp_mul(a: i64, b: i64) -> i64 {
    (a * b) >> 31
}

//
// Divide two fixed point numbers with 31 bits of precision.
// The numerator is shifted up to improve the precision of
// the result.  Again, breaks if the numerator represents
// a number above 2.
//
#[inline]
pub const fn fixp_div(numerator: i64, denominator: i64) -> i64 {
    (numerator << 31) / denominator
}

const fn const_tan(angle_native: f32) -> f32 {
    let angle = F32::from_native_f32(angle_native);
    angle.sin().div(angle.cos()).to_native_f32()
}

//
// Computes tangent at compile time for filter co-efficients.
//
const fn const_tan_int(a: i64) -> i64 {
    const ONE: f32 = (1i64 << 31) as f32;
    (const_tan((a as f32) / ONE * core::f32::consts::PI) * ONE) as i64
}

//
// Compute butterworth filter co-efficients for a 2nd order filter at compile time.
// Returns (B0, B1, B2, A0, A1).
//
pub const fn lowpass_butterworth(cutoff_freq: i64, sample_freq: i64) -> (i64, i64, i64, i64, i64) {
    let one: i64 = 1i64 << 31;
    if cutoff_freq > sample_freq * 19 / 100 {
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
        return (0, 0, 0, 0, 0);
    }
    let tan_fraction: i64 = one * cutoff_freq / sample_freq; // range 0 to 1/5
    let k: i64 = const_tan_int(tan_fraction); // range 0 to ~.73
    let sqrt2: i64 = 3037000500i64; // range 0 to ~1.42
    let k_squared: i64 = fixp_mul(k, k); // range 0 to ~.54
    let a0_denom = one + fixp_mul(sqrt2, k) + k_squared; // range 1 to 2.58
    let a1_numerator: i64 = 2 * (k_squared - one); // range -2 to .-.88.  Often around -2
    let a1: i64 = (a1_numerator << 31) / a0_denom; // should have just enough head room
    let a2_numerator: i64 = one - fixp_mul(sqrt2, k) + k_squared; // range 1 to .53.
    let a2: i64 = fixp_div(a2_numerator, a0_denom);

    // I'm did a special case for very small k values to try to get
    // a bit more accuracy for very low frequency filters.
    //
    let small_point = 5;
    let small: i64 = 1i64 << (31 - small_point);
    if k > small {
        let b0: i64 = fixp_div(k_squared, a0_denom);
        let b1: i64 = fixp_div(k_squared * 2, a0_denom);
        let b2: i64 = fixp_div(k_squared, a0_denom);
        return (b0, b1, b2, a1, a2);
    } else {
        //
        // k is under 1/2^5 (small_point=5).  Shift it up before the divide
        // to take advantage of the extra head room for a more accurate
        // k^2 computation.  Remember to shift back down by 10.
        //
        let k_shift = k << small_point;
        let k_squared_shift: i64 = fixp_mul(k_shift, k_shift); // range 0 to ~.54
        let b0: i64 = fixp_div(k_squared_shift, a0_denom) >> (small_point * 2);
        let b1: i64 = fixp_div(k_squared_shift * 2, a0_denom) >> (small_point * 2);
        let b2: i64 = fixp_div(k_squared_shift, a0_denom) >> (small_point * 2);
        return (b0, b1, b2, a1, a2);
    }
}

//
// A container for the three filter parameters I use in my filter.
//
// b1    - the b1 filter co-efficient.  b0 and b2 are just half b1, so I don't store them.
// a1_p1 - the a1 filter co-efficient, plus 1.  a1 is usually around -2.  Adding 1 gets an
//         extra bit of precision, at the cost of an extra 64 bit add in the filter.
// a2    - the a2 filter co-efficient.
//
#[derive(Copy, Clone)]
struct FilterParams {
    b1: i64,
    a1_p1: i64,
    a2: i64,
}

impl FilterParams {
    const fn new(cutoff_frequency: u32, sample_frequency: u32) -> Self {
        let raw_params = lowpass_butterworth(cutoff_frequency as i64, sample_frequency as i64);
        const ONE: i64 = 1i64 << 31;
        Self {
            b1: raw_params.1,
            a1_p1: raw_params.3 + ONE,
            a2: raw_params.4,
        }
    }
    const fn const_default() -> Self {
        Self {
            b1: 0,
            a1_p1: 0,
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
    d1: i64,
    d2: i64,
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
            let raw_value = self.source.get_next().to_i32();
            // raw_value starts as fix point with 15 decimal bits.  Shift by
            // 16 to get a 64 bit fixed point with 31 decimal bits.
            let input = (raw_value as i64) << 16;

            // Compute input * B0, input * B1, input * B2.
            //
            let b1_input_term = fixp_mul(input, self.params.b1);
            // B0 = B1/2, B2 = B1/2, so just take the b1 input term and divide by 2
            let b0_input_term = b1_input_term >> 1;
            let b2_input_term = b0_input_term;

            //
            // Compute output, output * a1, output * a2
            //
            // For the a1 term, I added 1 to A1 to get an extra bit of head
            // room so the fixed point multiply doesn't overflow (A1_P1 being
            // A1 Plus 1).  Pay an extra subtract to unwind that.
            //
            let output: i64 = b0_input_term + self.d1;
            let a1_output_term = fixp_mul(output, self.params.a1_p1) - output;
            let a2_output_term = fixp_mul(output, self.params.a2);

            // Record d1 and d2, then return the output
            //
            self.d1 = self.d2 + b1_input_term - a1_output_term;
            self.d2 = b2_input_term - a2_output_term;
            let output_i32 = (output >> 16) as i32;
            SoundSampleI32::new_i32(output_i32)
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
    use std::f64::consts::PI;

    // Helper to convert 31 bit fixed point to float
    //
    fn fixp_to_float(i: i64) -> f64 {
        (i as f64) / ((1i64 << 31) as f64)
    }

    //
    // Helper to figure out if the actual is close to expected
    // (within 5 digits)
    //
    fn is_close(actual: f64, expected: f64) -> bool {
        let accuracy = actual / expected;
        accuracy > 0.99999 && accuracy < 1.00001
    }

    //
    // Test a tangent test case.  TODO - better ways to do this.
    //
    fn const_tan_int_accuracy_test_case<const CUTOFF: u32, const FREQ: u32>() {
        let target: f64 = (CUTOFF as f64) / (FREQ as f64);
        let expected = (PI * target).tan();
        let one_fixp: f64 = (1i64 << 31) as f64;
        let target_int = (target * one_fixp) as i64;
        let actual_int = const_tan_int(target_int);
        let actual = (actual_int as f64) / one_fixp;
        let worked = is_close(actual, expected);
        assert_eq!(
            (CUTOFF, actual, expected, true),
            (CUTOFF, actual, expected, worked)
        )
    }

    #[test]
    fn const_tan_int_accuracy() {
        const_tan_int_accuracy_test_case::<400, 24000>();
        const_tan_int_accuracy_test_case::<150, 24000>();
        const_tan_int_accuracy_test_case::<40, 24000>();
    }

    #[test]
    fn filter_params_150hz() {
        let params = lowpass_butterworth(150, 24000);

        // Reference values from the Python script
        //
        let b0_expected = 0.0003750696;
        let b1_expected = 0.0007501392;
        let b2_expected = 0.0003750696;
        let a1_expected = -1.9444776578;
        let a2_expected = 0.9459779362;

        let b0_actual = fixp_to_float(params.0);
        let b1_actual = fixp_to_float(params.1);
        let b2_actual = fixp_to_float(params.2);
        let a1_actual = fixp_to_float(params.3);
        let a2_actual = fixp_to_float(params.4);

        assert_eq!(
            (b0_actual, b0_expected, true),
            (b0_actual, b0_expected, is_close(b0_actual, b0_expected))
        );
        assert_eq!(
            (b1_actual, b1_expected, true),
            (b1_actual, b1_expected, is_close(b1_actual, b1_expected))
        );
        assert_eq!(
            (b2_actual, b2_expected, true),
            (b2_actual, b2_expected, is_close(b2_actual, b2_expected))
        );
        assert_eq!(
            (a1_actual, a1_expected, true),
            (a1_actual, a1_expected, is_close(a1_actual, a1_expected))
        );
        assert_eq!(
            (a2_actual, a2_expected, true),
            (a2_actual, a2_expected, is_close(a2_actual, a2_expected))
        );
    }
    #[test]
    fn filter_params_40hz() {
        let params = lowpass_butterworth(40, 24000);

        // Reference values from the Python script
        //
        let b0_expected = 0.0000272138;
        let b1_expected = 0.0000544276;
        let b2_expected = 0.0000272138;
        let a1_expected = -1.9851906579;
        let a2_expected = 0.9852995131;

        let b0_actual = fixp_to_float(params.0);
        let b1_actual = fixp_to_float(params.1);
        let b2_actual = fixp_to_float(params.2);
        let a1_actual = fixp_to_float(params.3);
        let a2_actual = fixp_to_float(params.4);

        assert_eq!(
            (b0_actual, b0_expected, true),
            (b0_actual, b0_expected, is_close(b0_actual, b0_expected))
        );
        assert_eq!(
            (b1_actual, b1_expected, true),
            (b1_actual, b1_expected, is_close(b1_actual, b1_expected))
        );
        assert_eq!(
            (b2_actual, b2_expected, true),
            (b2_actual, b2_expected, is_close(b2_actual, b2_expected))
        );
        assert_eq!(
            (a1_actual, a1_expected, true),
            (a1_actual, a1_expected, is_close(a1_actual, a1_expected))
        );
        assert_eq!(
            (a2_actual, a2_expected, true),
            (a2_actual, a2_expected, is_close(a2_actual, a2_expected))
        );
    }
    #[test]
    fn filter_params_400hz() {
        let params = lowpass_butterworth(400, 24000);

        // Reference values from the Python script
        //
        let b0_expected = 0.0025505352;
        let b1_expected = 0.0051010703;
        let b2_expected = 0.0025505352;
        let a1_expected = -1.8521464854;
        let a2_expected = 0.8623486260;

        let b0_actual = fixp_to_float(params.0);
        let b1_actual = fixp_to_float(params.1);
        let b2_actual = fixp_to_float(params.2);
        let a1_actual = fixp_to_float(params.3);
        let a2_actual = fixp_to_float(params.4);

        assert_eq!(
            (b0_actual, b0_expected, true),
            (b0_actual, b0_expected, is_close(b0_actual, b0_expected))
        );
        assert_eq!(
            (b1_actual, b1_expected, true),
            (b1_actual, b1_expected, is_close(b1_actual, b1_expected))
        );
        assert_eq!(
            (b2_actual, b2_expected, true),
            (b2_actual, b2_expected, is_close(b2_actual, b2_expected))
        );
        assert_eq!(
            (a1_actual, a1_expected, true),
            (a1_actual, a1_expected, is_close(a1_actual, a1_expected))
        );
        assert_eq!(
            (a2_actual, a2_expected, true),
            (a2_actual, a2_expected, is_close(a2_actual, a2_expected))
        );
    }

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
        assert_eq!((20861, 50), get_avg_amplitude(&mut sine_50hz));
        assert_eq!((20856, 50), get_avg_amplitude(&mut filtered_sine_50hz));

        let mut sine_100hz = Oscillator::new(100 * FREQUENCY_MULTIPLIER);
        let mut filtered_sine_100hz = FilteredOscillator::new((100 * FREQUENCY_MULTIPLIER, 400));
        assert_eq!((20861, 100), get_avg_amplitude(&mut sine_100hz));
        assert_eq!((20816, 100), get_avg_amplitude(&mut filtered_sine_100hz));

        let mut sine_200hz = Oscillator::new(200 * FREQUENCY_MULTIPLIER);
        let mut filtered_sine_200hz = FilteredOscillator::new((200 * FREQUENCY_MULTIPLIER, 400));
        assert_eq!((20861, 200), get_avg_amplitude(&mut sine_200hz));
        assert_eq!((20235, 200), get_avg_amplitude(&mut filtered_sine_200hz));

        // This is the cut-off frequency.  The filtered average amplitude should be 1/sqrt(2)
        // of the original average amplitude, or 70.71%.  We're getting 70.72%.
        //
        let mut sine_400hz = Oscillator::new(400 * FREQUENCY_MULTIPLIER);
        let mut filtered_sine_400hz = FilteredOscillator::new((400 * FREQUENCY_MULTIPLIER, 400));
        assert_eq!((20862, 400), get_avg_amplitude(&mut sine_400hz));
        assert_eq!((14735, 400), get_avg_amplitude(&mut filtered_sine_400hz));

        let mut sine_800hz = Oscillator::new(800 * FREQUENCY_MULTIPLIER);
        let mut filtered_sine_800hz = FilteredOscillator::new((800 * FREQUENCY_MULTIPLIER, 400));
        assert_eq!((20808, 800), get_avg_amplitude(&mut sine_800hz));
        assert_eq!((5042, 800), get_avg_amplitude(&mut filtered_sine_800hz));

        let mut sine_1600hz = Oscillator::new(1600 * FREQUENCY_MULTIPLIER);
        let mut filtered_sine_1600hz = FilteredOscillator::new((1600 * FREQUENCY_MULTIPLIER, 400));
        assert_eq!((20808, 1600), get_avg_amplitude(&mut sine_1600hz));
        assert_eq!((1268, 1599), get_avg_amplitude(&mut filtered_sine_1600hz));
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
        assert_eq!((20861, 50), get_avg_amplitude(&mut sine_50hz));
        assert_eq!((20860, 50), get_avg_amplitude(&mut filtered_sine_50hz));

        let mut sine_100hz = Oscillator::new(100 * FREQUENCY_MULTIPLIER);
        let mut filtered_sine_100hz = FilteredOscillator::new((100 * FREQUENCY_MULTIPLIER, 1600));
        assert_eq!((20861, 100), get_avg_amplitude(&mut sine_100hz));
        assert_eq!((20860, 100), get_avg_amplitude(&mut filtered_sine_100hz));

        let mut sine_200hz = Oscillator::new(200 * FREQUENCY_MULTIPLIER);
        let mut filtered_sine_200hz = FilteredOscillator::new((200 * FREQUENCY_MULTIPLIER, 1600));
        assert_eq!((20861, 200), get_avg_amplitude(&mut sine_200hz));
        assert_eq!((20861, 200), get_avg_amplitude(&mut filtered_sine_200hz));

        let mut sine_400hz = Oscillator::new(400 * FREQUENCY_MULTIPLIER);
        let mut filtered_sine_400hz = FilteredOscillator::new((400 * FREQUENCY_MULTIPLIER, 1600));
        assert_eq!((20862, 400), get_avg_amplitude(&mut sine_400hz));
        assert_eq!((20838, 400), get_avg_amplitude(&mut filtered_sine_400hz));

        let mut sine_800hz = Oscillator::new(800 * FREQUENCY_MULTIPLIER);
        let mut filtered_sine_800hz = FilteredOscillator::new((800 * FREQUENCY_MULTIPLIER, 1600));
        assert_eq!((20808, 800), get_avg_amplitude(&mut sine_800hz));
        assert_eq!((20299, 800), get_avg_amplitude(&mut filtered_sine_800hz));

        // This is the cut-off frequency.  The filtered average amplitude should be 1/sqrt(2)
        // of the original average amplitude, or 70.71%.  We're getting 71.12%.
        //
        let mut sine_1600hz = Oscillator::new(1600 * FREQUENCY_MULTIPLIER);
        let mut filtered_sine_1600hz = FilteredOscillator::new((1600 * FREQUENCY_MULTIPLIER, 1600));
        assert_eq!((20808, 1600), get_avg_amplitude(&mut sine_1600hz));
        assert_eq!((14775, 1600), get_avg_amplitude(&mut filtered_sine_1600hz));
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
        assert_eq!((20861, 50), get_avg_amplitude(&mut sine_50hz));
        assert_eq!((20861, 50), get_avg_amplitude(&mut filtered_sine_50hz));

        // A silly oscillator for a silly filter.  The measured frequency is 2000
        // hz because of aliasing.
        //
        let mut sine_22000hz = Oscillator::new(22000 * FREQUENCY_MULTIPLIER);
        let mut filtered_sine_22000hz =
            FilteredOscillator::new((22000 * FREQUENCY_MULTIPLIER, 24000));
        assert_eq!((20418, 2001), get_avg_amplitude(&mut sine_22000hz));
        assert_eq!((20418, 2001), get_avg_amplitude(&mut filtered_sine_22000hz));
    }
}
