#![no_std]
#![no_main]

use defmt::info;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_nrf::{
    Peri,
    gpio::{AnyPin, Level, Output, OutputDrive},
};
use embassy_time::Timer;
use panic_halt as _;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());

    spawner
        .spawn(
            blink(
                p.P0_21
                    .into(),
                p.P0_28
                    .into(),
                250
            ),
        )
        .unwrap();
}

#[embassy_executor::task]
async fn blink(
    row_pin: Peri<'static, AnyPin>,
    col_pin: Peri<'static, AnyPin>,
    delay_ms: u64,
) {
    let _col = Output::new(
        col_pin,
        Level::Low,
        OutputDrive::Standard,
    );
    let mut row = Output::new(
        row_pin,
        Level::High,
        OutputDrive::Standard,
    );
    loop {
        row.set_high();
        Timer::after_millis(delay_ms).await;
        row.set_low();
        Timer::after_millis(delay_ms).await;
        info!("meow");
    }
}
