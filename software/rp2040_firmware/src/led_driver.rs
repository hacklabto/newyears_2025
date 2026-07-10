use crate::backlight::BacklightUser;
use crate::backlight::LED_COLUMNS;
use crate::backlight::LED_ROWS;

pub struct LedDriver {
    back_light_user: BacklightUser,
    counter: u32,
    counter_2: u8
}

impl Default for LedDriver {
    fn default() -> Self {
        Self {
            back_light_user: BacklightUser::default(),
            counter: 0,
            counter_2: 0,
        }
    }
}

impl LedDriver {
    pub async fn run(self: &mut Self) {
        loop {
            self.counter = (self.counter + 3) & 0xff;
            self.counter_2 = self.counter_2.wrapping_add(1);
            let led_level: u8 = self.counter as u8;
            let mut position:u8 = 0;

            for row in 0..LED_ROWS {
                for column in 0..LED_COLUMNS {
                    position = position.wrapping_add(28);
                    let blue:u8= led_level.wrapping_add(position).wrapping_add(self.counter_2);
                    self.back_light_user
                        .set(row, column, led_level.wrapping_add(self.counter_2), led_level, blue );
                }
            }
            self.back_light_user.update_led_dma_buffer();

            let mut ticker = embassy_time::Ticker::every(embassy_time::Duration::from_millis(10));
            ticker.next().await;
        }
    }
    fn record_midi_notes(self: &mut Self, notes: &[u8; 128]) {
        let mut row: usize = 0;
        let mut column: usize = 0;
        let mut led = 0;
        let mut r = 0;
        let mut g = 0;
        for note in notes {
            if led == 0 {
                r = *note;
            } else if led == 1 {
                g = *note;
            } else if led == 2 {
                let b = *note;
                self.back_light_user.set(row, column, r, g, b);
                column = column + 1;
                if column == LED_COLUMNS {
                    column = 0;
                    row = row + 1;
                    if row == LED_ROWS {
                        row = 0;
                    }
                }
            }
            led = led + 1;
            if led == 3 {
                led = 0;
            }
        }
    }
}
