use crate::backlight::BacklightUser;

pub struct LedDriver {
    back_light_user: BacklightUser,
    counter: u32,
}

impl Default for LedDriver {
    fn default() -> Self {
        Self {
            back_light_user: BacklightUser::default(),
            counter: 0,
        }
    }
}

impl LedDriver {
    pub async fn run(self: &mut Self) {
        loop {
            self.counter = (self.counter + 1) & 0xff;
            let led_level: u8 = self.counter as u8;

            for row in 0..5 {
                for column in 0..9 {
                    self.back_light_user
                        .set(row, column, led_level, led_level, led_level);
                }
            }
            self.back_light_user.update_led_dma_buffer();

            let mut ticker = embassy_time::Ticker::every(embassy_time::Duration::from_millis(100));
            ticker.next().await;
        }
    }
}
