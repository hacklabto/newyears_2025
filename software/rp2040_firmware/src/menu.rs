use crate::Devices;
use embedded_graphics::draw_target::DrawTarget;
use crate::display;
use embedded_graphics::pixelcolor::BinaryColor;

// start of menu system, but I think I need the devices grouped first.
//
pub struct MenuItem<'a> {
    pub text: &'a str,
    pub on_select: &'a dyn Fn( &mut Devices ),
}

pub async fn run_menu(test: &MenuItem<'_>, devices: &mut Devices<'_> )
{
    const MID: i32 = 16 + 16/2;
    for n in 0 .. MID {
        if (n % 2) == 0 {
            devices.display.clear(BinaryColor::Off).unwrap();
            display::draw_text(&mut devices.display, test.text, n, false );
            let mut ticker = embassy_time::Ticker::every(embassy_time::Duration::from_millis(10));
            ticker.next().await;
            devices.display.flush().unwrap();
        }
    }
    let mut ticker = embassy_time::Ticker::every(embassy_time::Duration::from_millis(2000));
    ticker.next().await;
    for n in MID .. 48 {
        if (n % 2) == 0 {
            devices.display.clear(BinaryColor::Off).unwrap();
            display::draw_text(&mut devices.display, test.text, n, false );
            let mut ticker = embassy_time::Ticker::every(embassy_time::Duration::from_millis(10));
            ticker.next().await;
            devices.display.flush().unwrap();
        }
    }
}

