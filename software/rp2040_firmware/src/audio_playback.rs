use midi_nostd::midi::Midi;
type NewYearsMidi<'a> = Midi<'a, 20292, { 89 * 3 }, 64, 32>;

#[allow(long_running_const_eval)]
pub struct AudioPlayback<'d> {
    midi: &'d mut NewYearsMidi<'d>,
    clear_count: u32,
}

/*
use softfloat::F32;

const fn gamma_func( int_sample: u32) -> i32 {
    let sample: F32 = F32::from_u32(int_sample).div(F32::from_u32(0x8000));
    let gamma_sample: F32 = sample.sqrt();
    let gamma_int: F32 = gamma_sample.mul(F32::from_u32(0x8000));
    return gamma_int.to_u32() as i32;
}
*/

/*
const fn build_gamma_array<const N: usize>() -> [ i32; N] {
    let mut gamma_array = [0; N];
    let mut idx: u32 = 0;
    while idx < (N as u32) {
        let g0: i32 = gamma_func( idx );
        let g1: i32 = gamma_func( idx + 32 );
        let mut sub_idx: i32 = 0;
        while sub_idx < 32 {
            gamma_array[ (idx+ sub_idx as u32) as usize ] = g0 + (g1-g0)*sub_idx / 32;
            sub_idx = sub_idx + 1;
        }
        idx = idx + 32;
    }
    return gamma_array
}
*/

/*
const fn build_gamma_array_2<const N: usize>() -> [ i32; N] {
    let mut gamma_array = [0; N];
    let mut idx: u32 = 0;
    while idx < (N as u32) {
        gamma_array[ idx as usize ] = gamma_func( idx );
        idx = idx + 1
    }
    return gamma_array
}
*/

impl<'d>
    AudioPlayback<'d>
{
    pub fn new(midi: &'d mut NewYearsMidi<'d>) -> Self {
        let clear_count: u32 = 0;
        Self { midi, clear_count }
    }

#[allow(long_running_const_eval)]
    // Ideas that didn't work...
    //const GAMMA_TABLE: [i32; 0x8000] = build_gamma_array::<0x8000>();

        /*
            let value_raw: i32 = self.midi.get_next().to_i32();
            let value_raw: i32 = 0;
            let mut value_raw_i8 = value_raw>>8;
            if value_raw_i8 > 127 {
                value_raw_i8 = 127
            }
            if value_raw_i8 < -127 {
                value_raw_i8 = -127
            }
            if (self.cycle & 64) == 0 { 
                value_raw_i8 = 127
            } else {
                value_raw_i8 = -127
            }
            if !self.midi.has_next() {
                self.clear_count = 1;
            }
        */

    fn gamma( sample: i32) -> i32 {
        const SAMPLE_TOP_0: i32 = 0x800;
        const MUL0: i32 = 8;
        const OUT_TOP_0: i32 = SAMPLE_TOP_0 * MUL0;

        const SAMPLE_TOP_1: i32 = 0x1000; 
        const MUL1: i32 = 4;
        const OUT_TOP_1: i32 = (SAMPLE_TOP_1 - SAMPLE_TOP_0) * MUL1 + OUT_TOP_0; 

        const SAMPLE_TOP_2: i32 = 0x1800; 
        const MUL2: i32 = 2;
        const OUT_TOP_2: i32 = (SAMPLE_TOP_2 - SAMPLE_TOP_1) * MUL2 + OUT_TOP_1; 
        const SAMPLE_TOP_3: i32 = 0x2000; 
        const MUL3: i32 = 1;
        const OUT_TOP_3: i32 = (SAMPLE_TOP_3 - SAMPLE_TOP_2) * MUL3 + OUT_TOP_2;
        if sample < SAMPLE_TOP_0 {
            sample * MUL0
        }
        else if sample < SAMPLE_TOP_1 {
            (sample - SAMPLE_TOP_0) * MUL1 + OUT_TOP_0
        }
        else if sample < SAMPLE_TOP_2 {
            (sample - SAMPLE_TOP_1) * MUL2 + OUT_TOP_1
        }
        else if sample < SAMPLE_TOP_3 {
            (sample - SAMPLE_TOP_2) * MUL3 + OUT_TOP_2
        }
        else {
            let candidate: i32 = (sample - SAMPLE_TOP_3) / 2 + OUT_TOP_3;
            if candidate < 0x7fff {
                candidate
            }
            else {
                0x7fff
            }
        }
    }
    pub fn populate_next_dma_buffer_with_audio(&mut self, buffer: &mut [u32]) {
        for entry in buffer.iter_mut() {
            let v0: i32 = self.midi.get_next().to_i32()/4;
            let v1: i32 = self.midi.get_next().to_i32()/4;

            // < 0x0000 - 0x0800     x 4            0x0000 - 0x2000
            //   0x0800 - 0x1000     x 3 + 0x0800   0x2000 - 0x3800
            //   0x1000 - 0x2000     x 2 + 0x1800   0x3800 - 0x5800
            //   0x2000 - 0x3800     x 1 + 0x3800   0x5800 - 0x7000
            //   0x3800 +            x .5 + 0x5400  0x7000 - 0x7fff

            let v0_filtered = if v0 < 0 { 
                -Self::gamma(-v0) 
            } else { 
                Self::gamma(v0) 
            };
            let v1_filtered = if v1 < 0 { 
                -Self::gamma(-v1) } 
            else { 
                Self::gamma(v1) 
            }; 

            let v0_u32 : u32 = v0_filtered as u32;
            let v1_u32 : u32 = v1_filtered as u32;
    
            let output: u32 =
                ((v0_u32 >> 8 ) & 0xff) << 0 |
                ((v0_u32 >> 0 ) & 0xff) << 8 |
                ((v1_u32 >> 8 ) & 0xff) << 16 |
                ((v1_u32 >> 0 ) & 0xff) << 24;
            *entry = output;

            if !self.midi.has_next() {
                self.clear_count = 1;
            }
            /*
            self.cycle = self.cycle + 1;
            let value_u32: u32 = if (self.cycle & 16)==0 {
                0x0020
            }
            else {
                0x0000
            };
            */
            //let value_u32: u32 = 0;

        }
    }

    pub fn is_done(&self) -> bool {
        return self.clear_count == 1;
    }
}
