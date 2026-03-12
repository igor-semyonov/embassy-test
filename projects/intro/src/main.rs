#![no_std]
#![no_main]

use defmt::info;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_nrf::{
    Peri,
    gpio::{AnyPin, Input, Pull},
};
use embassy_time::Timer;
use panic_probe as _;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("Starting main");
    let p = embassy_nrf::init(Default::default());

    let button_a = button_task(
        p.P0_14
            .into(),
        "A",
    );
    let button_b = button_task(
        p.P0_23
            .into(),
        "B",
    );
    join(button_a, button_b).await;
}

async fn button_task(
    button_pin: Peri<'static, AnyPin>,
    id: &'static str,
) {
    info!(
        "Starting task for button {}",
        id
    );
    let mut button = Input::new(
        button_pin,
        Pull::None,
    );
    loop {
        button
            .wait_for_low()
            .await;
        info!(
            "Button {} pushed! (fut)",
            id
        );
        Timer::after_millis(50).await;
        button
            .wait_for_high()
            .await;
    }
}
