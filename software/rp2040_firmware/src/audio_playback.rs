use midi_nostd::midi::Midi;
type NewYearsMidi<'a> = Midi<'a, 20292, { 89 * 3 }, 64, 32>;

pub struct AudioPlayback<'d> {
    midi: &'d mut NewYearsMidi<'d>,
    clear_count: u32,
    cycle: u32,
}

impl<'d>
    AudioPlayback<'d>
{
    pub fn new(midi: &'d mut NewYearsMidi<'d>) -> Self {
        let clear_count: u32 = 0;
        let cycle: u32 = 0;
        Self { midi, clear_count, cycle }
    }

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
    pub fn populate_next_dma_buffer_with_audio(&mut self, buffer: &mut [u32]) {
        for entry in buffer.iter_mut() {
            let mut v0: i32 = self.midi.get_next().to_i32() * 2;
            let mut v1: i32 = self.midi.get_next().to_i32() * 2;

            if v0 < -0x7fff {
                v0 = -0x7fff;
            }
            if v0 > 0x7fff {
                v0 = 0x7fff;
            }
            if v1 < -0x7fff {
                v1 = -0x7fff;
            }
            if v1 > 0x7fff {
                v1 = 0x7fff;
            }

            let v0_u32 : u32 = v0 as u32;
            let v1_u32 : u32 = v1 as u32;
    
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
