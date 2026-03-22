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
            ),
        )
        .unwrap();
}

#[embassy_executor::task]
async fn make_tone(speaker_pin: Peri<'static, AnyPin>) {
    async fn play_tone(
        speaker: &mut Output<'_>,
        frequency: u64,
        cycles: u64,
    ) {
        let period = 1_000 / frequency;
        for _ in 0..cycles / period {
            speaker.set_high();
            Timer::after_millis(period / 2).await;
            speaker.set_low();
            Timer::after_millis(period / 2).await;
        }
    }
    let mut speaker = Output::new(
        speaker_pin,
        Level::Low,
        OutputDrive::Standard,
    );

    for held_note in [NOTE_B3, NOTE_A3, NOTE_G3, NOTE_D3] {
        play_tone(
            &mut speaker,
            NOTE_FS4,
            TONE_FULL,
        )
        .await;
        Timer::after_millis(MS_BETWEEN_TONES);

        play_tone(
            &mut speaker,
            NOTE_FS4,
            TONE_FULL,
        )
        .await;
        Timer::after_millis(MS_BETWEEN_TONES);

        play_tone(
            &mut speaker,
            NOTE_FS4,
            TONE_HALF,
        )
        .await;
        Timer::after_millis(MS_BETWEEN_TONES);

        play_tone(
            &mut speaker,
            NOTE_G4,
            TONE_HALF,
        )
        .await;
        Timer::after_millis(MS_BETWEEN_TONES);

        play_tone(
            &mut speaker,
            NOTE_FS4,
            TONE_FULL,
        )
        .await;
        Timer::after_millis(MS_BETWEEN_TONES);

        play_tone(
            &mut speaker,
            NOTE_D4,
            TONE_FULL,
        )
        .await;
        Timer::after_millis(MS_BETWEEN_TONES);

        play_tone(
            &mut speaker,
            held_note,
            TONE_FULL * 2,
        )
        .await;

        Timer::after_millis(MS_BETWEEN_PHRASES);
    }
}

const MS_BETWEEN_TONES: u64 = 150;
const MS_BETWEEN_PHRASES: u64 = 400;

const BPM: u64 = 160;
const MSPB: u64 = 60_000 / BPM;
const TONE_FULL: u64 = MSPB;
const TONE_HALF: u64 = MSPB / 2;
const TONE_QUARTER: u64 = MSPB / 4;
const TONE_EIGHTH: u64 = MSPB / 8;

const NOTE_FS4: u64 = 370;
const NOTE_G4: u64 = 392;
const NOTE_D4: u64 = 294;
const NOTE_B3: u64 = 247;
const NOTE_A3: u64 = 220;
const NOTE_G3: u64 = 196;
const NOTE_D3: u64 = 147;
