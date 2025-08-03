// Frequency filter using fixed point math.

use crate::sound_sample::SoundSampleI32;
use crate::sound_source_core::SoundSourceCore;
use crate::sound_source_core::OscillatorInterface;

//
// Fixed point table of tan values.
//
// The tan table is here so I can compute my frequency co-efficients at compile
// time, which makes the rest of the code a bit more management.  Computing
// float values in Rust, at compile time, is still an ongoing research topic.
// Philosphically, the compiler problem is that you don't want compilers on
// different machines to produce different results because the float implementation
// changes (float results are definately 100% compiler/ machine/ optimization
// level dependent in C/ C++/ Rust).
//
// Values can be mapping back to floating point by dividing by (1<<31).  The 31 was
// chosen so two i64s can be multiplied and I have have a few bits of head room.
// So, first value non zero value, 6588417, maps to 6588417/2^31, or .0030679...
//
// The original function I was interested in is
//
// tan(pi * filter_frequency / playback_frequency)
//
// So table entries map to 2^10 * filter_frequency / playback_frequency
//
// i.e.,  if filter_frequency / playback_frequency is 0.1, then I need to
// compute tan( pi * .1 ), and the table entry I want is .1 * 2^10, or
// 102.
//
// There are only 204 table entries - I'm only supporting angles from 0 to
// about (204/1024) * pi, or about .2 * pi.  What that means is that the
// cut-off frequency for the filter can't be more than about 20% of the
// playback frequency, and the result of the tan call is always from
// about 0 to .73, which is a managable fixed point number
//
// Linear interpolation is done improve the function's accuracy.  The
// current accuracy is about 6 digits, which seems good enough, in that
// the Filters I'm getting seem to behave properly.
//
const TAN_TABLE: [i64; 204] = [
    0, 6588417, 13176960, 19765750, 26354912, 32944570, 39534849, 46125872, 52717764, 59310649,
    65904651, 72499895, 79096506, 85694607, 92294325, 98895783, 105499106, 112104420, 118711851,
    125321523, 131933562, 138548094, 145165246, 151785142, 158407910, 165033676, 171662568,
    178294711, 184930234, 191569264, 198211930, 204858358, 211508678, 218163018, 224821507,
    231484275, 238151451, 244823166, 251499549, 258180732, 264866845, 271558020, 278254389,
    284956084, 291663237, 298375983, 305094453, 311818783, 318549107, 325285560, 332028276,
    338777392, 345533044, 352295370, 359064505, 365840589, 372623760, 379414157, 386211919,
    393017187, 399830100, 406650802, 413479433, 420316137, 427161056, 434014334, 440876117,
    447746548, 454625775, 461513944, 468411202, 475317697, 482233579, 489158996, 496094099,
    503039040, 509993970, 516959042, 523934410, 530920227, 537916651, 544923835, 551941939,
    558971119, 566011534, 573063345, 580126712, 587201797, 594288762, 601387771, 608498990,
    615622583, 622758717, 629907561, 637069283, 644244053, 651432042, 658633423, 665848369,
    673077055, 680319656, 687576349, 694847313, 702132726, 709432770, 716747627, 724077479,
    731422512, 738782911, 746158863, 753550558, 760958185, 768381935, 775822002, 783278580,
    790751865, 798242054, 805749346, 813273941, 820816043, 828375853, 835953578, 843549424,
    851163600, 858796317, 866447785, 874118220, 881807836, 889516851, 897245485, 904993957,
    912762492, 920551313, 928360648, 936190725, 944041776, 951914032, 959807729, 967723104,
    975660395, 983619845, 991601695, 999606193, 1007633585, 1015684122, 1023758056, 1031855642,
    1039977138, 1048122803, 1056292898, 1064487689, 1072707443, 1080952429, 1089222920, 1097519190,
    1105841517, 1114190182, 1122565468, 1130967661, 1139397049, 1147853924, 1156338581, 1164851317,
    1173392433, 1181962234, 1190561025, 1199189117, 1207846823, 1216534460, 1225252347, 1234000808,
    1242780169, 1251590761, 1260432918, 1269306976, 1278213276, 1287152164, 1296123987, 1305129097,
    1314167852, 1323240610, 1332347736, 1341489598, 1350666568, 1359879022, 1369127341, 1378411911,
    1387733119, 1397091361, 1406487035, 1415920543, 1425392293, 1434902698, 1444452175, 1454041146,
    1463670038, 1473339283, 1483049319, 1492800589, 1502593540, 1512428625, 1522306304, 1532227041,
    1542191307,
];

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

// Constant time fixed point tan computation.
//
pub const fn const_tan(a: i64) -> i64 {
    // 2^10 = 1024 = Pi * angle in the table.  The table does only support angles to .2*pi
    //
    let tan_table_bits = 10;

    // a is 31 bit fixed point, so I need to divide by 2^31 and multiply by 2^10.
    // to get the table entry.  Or divide by 2^21.  Or shift by 31-10.
    //
    let tan_table_idx = (a >> (31 - tan_table_bits)) as usize;

    // Pull out the two table entries for linear interpolation
    //
    let e0 = TAN_TABLE[tan_table_idx];
    let e1 = TAN_TABLE[tan_table_idx + 1];

    // Now I need to get the last 21 bits of the angle, mask it off, and shift
    // it so it's a 31 bit fixed point number.
    //
    // 31 - tan_table_bits = 21, (2^21-1) gives me my mask, and a number from
    // 0 to 2^21-1.  Shifting left by 10 multiplies that to 0^31-1, which
    // represents a fixed point number from [0..1)
    //
    let fraction = (a & ((1i64 << (31 - tan_table_bits)) - 1)) << tan_table_bits;

    // And do the linear interpoltion.
    let one: i64 = 1i64 << 31;
    fixp_mul(e0, one - fraction) + fixp_mul(e1, fraction)
}

//
// Compute butterworth filter co-efficients for a 2nd order filter at compile time.
// Returns (B0, B1, B2, A0, A1).
//
pub const fn lowpass_butterworth(cutoff_freq: i64, sample_freq: i64) -> (i64, i64, i64, i64, i64) {
    let one: i64 = 1i64 << 31;
    if cutoff_freq * 6 > sample_freq {
        //
        // I only support a cut-off frequency that's 20% the sample frequency.
        // If you want something faster, the filter will treat this value as
        // a pass through.
        //
        return (0, 0, 0, 0, 0);
    }
    let tan_fraction: i64 = one * cutoff_freq / sample_freq; // range 0 to 1/5
    let k: i64 = const_tan(tan_fraction); // range 0 to ~.73
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

pub struct Filter<const P_FREQ: u32, const U_FREQ: u32, Source: OscillatorInterface<P_FREQ, U_FREQ>> {
    source: Source,
    d1: i64,
    d2: i64,
    params: &'static FilterParams,
}

impl<const P_FREQ: u32, const U_FREQ: u32, Source: OscillatorInterface<P_FREQ, U_FREQ>>
    Filter<P_FREQ, U_FREQ, Source>
{
    // const ONE: i64 = 1i64 << 31;

    // TODO, move these comments to structure
    // Extract filter co-coefficients at compile time.
    // const B1: i64 = lowpass_butterworth(CUTOFF_FREQUENCY, P_FREQ as i64).1;
    // B2 is the same as B0, so I just use the B0 term in the filter.
    // const B2: i64 = lowpass_butterworth(CUTOFF_FREQUENCY, P_FREQ, U_FREQ as i64).2;

    // In the filter we subtract A1 * input, but.... A1 is a value from -1 to -2.
    // Added one to remap it to 0 to -1 because I actually do need the extra bit
    // of head room.  The oscillator pair can produce values from 0 to 2.
    //
    // const A1_P1: i64 = lowpass_butterworth(CUTOFF_FREQUENCY, P_FREQ as i64).3 + Self::ONE;
    // const A2: i64 = lowpass_butterworth(CUTOFF_FREQUENCY, P_FREQ as i64).4;

    //const FILTER_PARAMS: FilterParams = FilterParams::new(CUTOFF_FREQUENCY as u32, P_FREQ);
    const FILTER_PARAMS_ARRAY_SIZE: usize = 300;
    const FILTER_STEP_SHIFT: u32 = 4;
    const FILTER_STEP: u32 = (1 << Self::FILTER_STEP_SHIFT);
    const FILTER_PARAMS_MAX_CUTOFF: u32 =
        ((Self::FILTER_PARAMS_ARRAY_SIZE as u32) * Self::FILTER_STEP);
    const FILTER_PARAMS: [FilterParams; 300] =
        build_filter_param_array::<300>(Self::FILTER_PARAMS_MAX_CUTOFF, P_FREQ);

    fn freq_to_filter_param(cutoff_frequency: u32) -> &'static FilterParams {
        let raw_idx = (cutoff_frequency >> Self::FILTER_STEP_SHIFT) as usize;
        let idx = if raw_idx >= Self::FILTER_PARAMS.len() {
            Self::FILTER_PARAMS.len() - 1
        } else {
            raw_idx
        };
        &(Self::FILTER_PARAMS[idx])
    }
}

impl<const P_FREQ: u32, const U_FREQ: u32, Source: OscillatorInterface<P_FREQ, U_FREQ>>
    SoundSourceCore<P_FREQ, U_FREQ> for Filter<P_FREQ, U_FREQ, Source>
{
    type InitValuesType = (Source::InitValuesType, u32);

    fn new(init_values: Self::InitValuesType) -> Self {
        let source = Source::new(init_values.0);
        let d1 = 0;
        let d2 = 0;
        return Self {
            source,
            d1,
            d2,
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
    fn const_tan_accuracy_test_case<const CUTOFF: u32, const FREQ: u32>() {
        let target: f64 = (CUTOFF as f64) / (FREQ as f64);
        let expected = (PI * target).tan();
        let one_fixp: f64 = (1i64 << 31) as f64;
        let target_int = (target * one_fixp) as i64;
        let actual_int = const_tan(target_int);
        let actual = (actual_int as f64) / one_fixp;
        let worked = is_close(actual, expected);
        assert_eq!(
            (CUTOFF, actual, expected, true),
            (CUTOFF, actual, expected, worked)
        )
    }

    #[test]
    fn const_tan_accuracy() {
        const_tan_accuracy_test_case::<400, 24000>();
        const_tan_accuracy_test_case::<150, 24000>();
        const_tan_accuracy_test_case::<40, 24000>();
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
