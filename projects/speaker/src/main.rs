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
            make_tone(
                p.P0_00
                    .into(),
                1_000 / 110,
            ),
        )
        .unwrap();
}

#[embassy_executor::task]
async fn make_tone(
    speaker_pin: Peri<'static, AnyPin>,
    period: u64,
) {
    let mut speaker = Output::new(
        speaker_pin,
        Level::Low,
        OutputDrive::Standard,
    );

    for _ in 0..2_000 / period {
        speaker.set_high();
        Timer::after_millis(period / 2).await;
        speaker.set_low();
        Timer::after_millis(period / 2).await;
    }
}
