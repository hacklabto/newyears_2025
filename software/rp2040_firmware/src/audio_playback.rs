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

            self.cycle = self.cycle + 1;
            let value_u32: u32 = if (self.cycle & 16)==0 {
                0x4444
            }
            else {
                0x0000
            };

            *entry = value_u32 | (value_u32 << 16); 
        }
    }

    pub fn is_done(&self) -> bool {
        return self.clear_count == 1;
    }
}
