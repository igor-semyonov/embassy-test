#![no_std]
#![no_main]

use defmt::info;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_nrf::{
    Peri, bind_interrupts,
    gpio::{Input, Pin, Pull},
    temp,
};
use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex,
    signal::Signal,
};
use embassy_time::{Duration, Timer, WithTimeout};
use panic_probe as _;

#[derive(Clone, Copy)]
enum Button {
    A,
    B,
}

static SIGNAL: Signal<CriticalSectionRawMutex, Button> =
    Signal::new();

bind_interrupts!(struct Irqs {
    TEMP => temp::InterruptHandler;
});

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("Starting main");
    let p = embassy_nrf::init(Default::default());

    let temp = temp::Temp::new(
        p.TEMP, Irqs,
    );
    spawner.spawn(temp_task(temp)).unwrap();

    let button_a = button_task(
        p.P0_14,
        "A",
        Button::A,
    );
    let button_b = button_task(
        p.P0_23,
        "B",
        Button::B,
    );
    join(
        button_a, button_b,
    )
    .await;
}

async fn button_task<P: Pin>(
    button_pin: Peri<'static, P>,
    id: &'static str,
    b: Button,
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
        SIGNAL.signal(b);
        Timer::after_millis(50).await;
        button
            .wait_for_high()
            .await;
    }
}

#[embassy_executor::task]
async fn temp_task(mut temp: temp::Temp<'static>) {
    let mut delay_ms = 500;
    loop {
        let t = temp
            .read()
            .await
            .to_num::<u16>();
        info!(
            "Current temp is {}°C",
            t
        );
        let delay = Duration::from_millis(delay_ms);
        if let Ok(v) = SIGNAL
            .wait()
            .with_timeout(delay)
            .await
        {
            delay_ms = match v {
                Button::A => delay_ms + 100,
                Button::B => delay_ms.saturating_sub(100),
            };
            info!(
                "Delay = {}ms",
                delay_ms
            );
        }
    }
}
