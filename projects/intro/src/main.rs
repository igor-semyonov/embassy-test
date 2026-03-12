#![no_std]
#![no_main]

use defmt::info;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_nrf::{
    Peri, PeripheralType,
    gpio::{AnyPin, Input, Pin, Pull},
};
use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex,
    signal::Signal,
};
use embassy_time::Timer;
use panic_probe as _;

enum Button {
    A,
    B,
}

static SIGNAL: Signal<CriticalSectionRawMutex, Button> =
    Signal::new();

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("Starting main");
    let p = embassy_nrf::init(Default::default());

    let button_a = button_task(
        p.P0_14,
        "A",
    );
    let button_b = button_task(
        p.P0_23,
        "B",
    );
    join(
        button_a, button_b,
    )
    .await;
}

async fn button_task<P: Pin>(
    button_pin: Peri<'static, P>,
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
