use crate::sound_sample::SoundSampleI32;
use crate::sound_source_core::SoundSourceCore;

const TAN_TABLE: [i64;204] = [
    0,
    6588417,
    13176960,
    19765750,
    26354912,
    32944570,
    39534849,
    46125872,
    52717764,
    59310649,
    65904651,
    72499895,
    79096506,
    85694607,
    92294325,
    98895783,
    105499106,
    112104420,
    118711851,
    125321523,
    131933562,
    138548094,
    145165246,
    151785142,
    158407910,
    165033676,
    171662568,
    178294711,
    184930234,
    191569264,
    198211930,
    204858358,
    211508678,
    218163018,
    224821507,
    231484275,
    238151451,
    244823166,
    251499549,
    258180732,
    264866845,
    271558020,
    278254389,
    284956084,
    291663237,
    298375983,
    305094453,
    311818783,
    318549107,
    325285560,
    332028276,
    338777392,
    345533044,
    352295370,
    359064505,
    365840589,
    372623760,
    379414157,
    386211919,
    393017187,
    399830100,
    406650802,
    413479433,
    420316137,
    427161056,
    434014334,
    440876117,
    447746548,
    454625775,
    461513944,
    468411202,
    475317697,
    482233579,
    489158996,
    496094099,
    503039040,
    509993970,
    516959042,
    523934410,
    530920227,
    537916651,
    544923835,
    551941939,
    558971119,
    566011534,
    573063345,
    580126712,
    587201797,
    594288762,
    601387771,
    608498990,
    615622583,
    622758717,
    629907561,
    637069283,
    644244053,
    651432042,
    658633423,
    665848369,
    673077055,
    680319656,
    687576349,
    694847313,
    702132726,
    709432770,
    716747627,
    724077479,
    731422512,
    738782911,
    746158863,
    753550558,
    760958185,
    768381935,
    775822002,
    783278580,
    790751865,
    798242054,
    805749346,
    813273941,
    820816043,
    828375853,
    835953578,
    843549424,
    851163600,
    858796317,
    866447785,
    874118220,
    881807836,
    889516851,
    897245485,
    904993957,
    912762492,
    920551313,
    928360648,
    936190725,
    944041776,
    951914032,
    959807729,
    967723104,
    975660395,
    983619845,
    991601695,
    999606193,
    1007633585,
    1015684122,
    1023758056,
    1031855642,
    1039977138,
    1048122803,
    1056292898,
    1064487689,
    1072707443,
    1080952429,
    1089222920,
    1097519190,
    1105841517,
    1114190182,
    1122565468,
    1130967661,
    1139397049,
    1147853924,
    1156338581,
    1164851317,
    1173392433,
    1181962234,
    1190561025,
    1199189117,
    1207846823,
    1216534460,
    1225252347,
    1234000808,
    1242780169,
    1251590761,
    1260432918,
    1269306976,
    1278213276,
    1287152164,
    1296123987,
    1305129097,
    1314167852,
    1323240610,
    1332347736,
    1341489598,
    1350666568,
    1359879022,
    1369127341,
    1378411911,
    1387733119,
    1397091361,
    1406487035,
    1415920543,
    1425392293,
    1434902698,
    1444452175,
    1454041146,
    1463670038,
    1473339283,
    1483049319,
    1492800589,
    1502593540,
    1512428625,
    1522306304,
    1532227041,
    1542191307,
];

pub const fn fixp_mul(a: i64, b: i64) -> i64 {
    (a*b)>>31
}

pub const fn fixp_div(a: i64, b: i64) -> i64 {
    (a<<31)/b
}

pub const fn const_tan(a: i64) -> i64
{
    let tan_table_bits = 10;
    let tan_table_idx = (a>>(31-tan_table_bits)) as usize;
    let e0 = TAN_TABLE[tan_table_idx];
    let e1 = TAN_TABLE[tan_table_idx+1];
    let fraction = (a & ((1i64 << (31-tan_table_bits)) - 1) ) << tan_table_bits;
    let one: i64 = 1i64<<31;
    fixp_mul(e0, one-fraction) + fixp_mul(e1, fraction)
}


pub const fn lowpass_butterworth(cutoff: i64, sample: i64) -> (i64, i64, i64, i64)
{
    let one: i64 = 1i64<<31;
    if cutoff*5 > sample {
        return (one, 0, 0, 0);
    }
    let tan_fraction: i64= one * cutoff / sample;   // range 0 to 1/5
    let k: i64 = const_tan(tan_fraction);       // range 0 to ~.73
    let k_squared: i64 = fixp_mul(k, k);        // range 0 to ~.54
    let sqrt2: i64 = 3037000500i64;                // range 0 to ~1.42
    let a0_denom = one + fixp_mul(sqrt2, k) + k_squared;  // range 1 to 2.58
    let b0: i64= fixp_div(k_squared,a0_denom);
    let b1: i64= fixp_div(k_squared*2, a0_denom);
    let b2: i64= fixp_div(k_squared,a0_denom);
    let a1_numerator: i64 = 2*(k_squared-one);   // range -2 to .-.88.  Often around -2  
    let a1: i64 = (a1_numerator<<31) / a0_denom; // should have just enough head room
    let a2_numerator: i64= one - fixp_mul(sqrt2, k) + k_squared;  // range 1 to .53.
    let a2:i64 = fixp_div(a2_numerator,a0_denom);
    return (b0, b1+a1, b2+a2, k_squared)
}

#[cfg(test)]
mod tests { 
    use crate::filter::*;
    use std::f64::consts::PI;

    fn fixp_to_float(i: i64) -> f64 
    {
        (i as f64) / ((1i64 << 31) as f64)
    }

    fn is_fairly_accurate(actual: f64, expected: f64 ) -> bool {
        let accuracy = actual / expected;
        accuracy > 0.99999 && accuracy < 1.00001
    }

    #[test]
    fn const_tan_accuracy() {
        let target: f64 = 150.0/24000.0;
        let expected = (PI*target).tan();
        let one_fixp: f64 = (1i64 << 31) as f64;
        let target_int = (target * one_fixp) as i64;
        let actual_int = const_tan(target_int);
        let actual = (actual_int as f64)/ one_fixp;
        assert_eq!((actual, expected, true), (actual, expected, is_fairly_accurate(actual, expected)))
    }

    #[test]
    fn filter_params_150hz() {
        let params = lowpass_butterworth(150, 24000);

        let b0_expected = 0.0003750696;
        let b1_expected = 0.0007501392-1.9444776578;
        let b2_expected = 0.0003750696+0.9459779362;
    
        let b0_actual = fixp_to_float( params.0 );
        let b1_actual = fixp_to_float( params.1 );
        let b2_actual = fixp_to_float( params.2 );

        assert_eq!((b0_actual, b0_expected, true), 
            (b0_actual, b0_expected, is_fairly_accurate(b0_actual, b0_expected)));
        assert_eq!((b1_actual, b1_expected, true), 
            (b1_actual, b1_expected, is_fairly_accurate(b1_actual, b1_expected)));
        assert_eq!((b2_actual, b2_expected, true), 
            (b2_actual, b2_expected, is_fairly_accurate(b2_actual, b2_expected)));
    }
}

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

    fn new(init_values: Self::InitValuesType) -> Self {
        let source = Source::new(init_values);
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
        if !self.source.has_next() {
            self.z1 < -0x200 || self.z1 > 0x200
        } else {
            true
        }
    }

    fn trigger_note_off(self: &mut Self) {
        self.source.trigger_note_off();
    }
}
