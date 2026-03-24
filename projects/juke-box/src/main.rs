#![no_std]
#![no_main]

use defmt::info;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_nrf::{
    Peri,
    gpio::{
        AnyPin, Input, Level, Output, OutputDrive, Pull,
    },
};
use embassy_time::Timer;
use panic_halt as _;

const MS_BETWEEN_TONES: u64 = 25;
const MS_BETWEEN_PHRASES: u64 = 400;

const BEATS_PER_MINUTE: u64 = 180;
const MICROS_PER_BEAT: u64 = 60_000_000 / BEATS_PER_MINUTE;
// const TONE_FULL: u64 = MICROS_PER_BEAT;
// const TONE_HALF: u64 = MICROS_PER_BEAT / 2;
// const TONE_QUARTER: u64 = MICROS_PER_BEAT / 4;
// const TONE_EIGHTH: u64 = MICROS_PER_BEAT / 8;

struct Note {
    frequency: NoteFrequency,
    duration: NoteDuration,
}
impl Note {
    const fn new(
        frequency: NoteFrequency,
        duration: NoteDuration,
    ) -> Self {
        Self {
            frequency,
            duration,
        }
    }
    async fn play(&self, speaker: &mut Output<'_>) {
        let period = self.period_micros();
        for _ in 0..self
            .duration
            .micros()
            / period
        {
            speaker.set_high();
            Timer::after_micros(period / 2).await;
            speaker.set_low();
            Timer::after_micros(period / 2).await;
        }
    }
    const fn frequency(&self) -> u64 {
        self.frequency
            .frequency()
    }
    const fn period_millis(&self) -> u64 {
        1_000
            / self
                .frequency
                .frequency()
    }
    const fn period_micros(&self) -> u64 {
        1_000_000
            / self
                .frequency
                .frequency()
    }
}
enum NoteFrequency {
    Fs4,
    G4,
    D4,
    B3,
    A3,
    G3,
    D3,
}
impl NoteFrequency {
    const fn frequency(&self) -> u64 {
        use NoteFrequency as N;
        match self {
            N::Fs4 => 370,
            N::G4 => 392,
            N::D4 => 294,
            N::B3 => 247,
            N::A3 => 220,
            N::G3 => 196,
            N::D3 => 147,
        }
    }
    const fn period_millis(&self) -> u64 {
        1_000 / self.frequency()
    }
    const fn period_micros(&self) -> u64 {
        1_000_000 / self.frequency()
    }
}

enum NoteDuration {
    Four,
    Two,
    Full,
    Half,
    Quarter,
    Eighth,
    Sixteenth,
}
impl NoteDuration {
    const fn micros(&self) -> u64 {
        use NoteDuration as D;
        match self {
            D::Four => 4 * MICROS_PER_BEAT,
            D::Two => 2 * MICROS_PER_BEAT,
            D::Full => MICROS_PER_BEAT,
            D::Half => MICROS_PER_BEAT / 2,
            D::Quarter => MICROS_PER_BEAT / 4,
            D::Eighth => MICROS_PER_BEAT / 8,
            D::Sixteenth => MICROS_PER_BEAT / 16,
        }
    }
}

struct Phrase<const N_Notes: usize> {
    notes: [Note; N_Notes],
    pause: u64,
}
impl<const N_Notes: usize> Phrase<N_Notes> {
    const fn new(
        notes: [Note; N_Notes],
        pause: u64,
    ) -> Self {
        Self {
            notes,
            pause,
        }
    }
    async fn play(&self, speaker: &mut Output<'_>) {
        for note in &self.notes {
            note.play(speaker)
                .await;
            Timer::after_millis(self.pause).await;
        }
    }
}

struct Song<
    const N_Phrases: usize,
    const N_Notes_per_Phrase: usize,
> {
    phrases: [Phrase<N_Notes_per_Phrase>; N_Phrases],
    pause: u64,
}
impl<
    const N_Phrases: usize,
    const N_Notes_per_Phrase: usize,
> Song<N_Phrases, N_Notes_per_Phrase>
{
    const fn new(
        phrases: [Phrase<N_Notes_per_Phrase>; N_Phrases],
        pause: u64,
    ) -> Self {
        Self {
            phrases,
            pause,
        }
    }
    async fn play(&self, speaker: &mut Output<'_>) {
        for phrase in &self.phrases {
            phrase
                .play(speaker)
                .await;
            Timer::after_millis(self.pause).await;
        }
    }
}

use NoteDuration as ND;
use NoteFrequency as NF;
static LAMOUR: Song<4, 7> = Song::new(
    [
        Phrase::new(
            [
                Note::new(
                    NF::Fs4,
                    ND::Full,
                ),
                Note::new(
                    NF::Fs4,
                    ND::Full,
                ),
                Note::new(
                    NF::Fs4,
                    ND::Full,
                ),
                Note::new(
                    NF::Fs4,
                    ND::Half,
                ),
                Note::new(
                    NF::G4,
                    ND::Half,
                ),
                Note::new(
                    NF::D4,
                    ND::Full,
                ),
                Note::new(
                    NF::B3,
                    ND::Full,
                ),
            ],
            25,
        ),
        Phrase::new(
            [
                Note::new(
                    NF::Fs4,
                    ND::Full,
                ),
                Note::new(
                    NF::Fs4,
                    ND::Full,
                ),
                Note::new(
                    NF::Fs4,
                    ND::Full,
                ),
                Note::new(
                    NF::Fs4,
                    ND::Half,
                ),
                Note::new(
                    NF::G4,
                    ND::Half,
                ),
                Note::new(
                    NF::D4,
                    ND::Full,
                ),
                Note::new(
                    NF::A3,
                    ND::Full,
                ),
            ],
            25,
        ),
        Phrase::new(
            [
                Note::new(
                    NF::Fs4,
                    ND::Full,
                ),
                Note::new(
                    NF::Fs4,
                    ND::Full,
                ),
                Note::new(
                    NF::Fs4,
                    ND::Full,
                ),
                Note::new(
                    NF::Fs4,
                    ND::Half,
                ),
                Note::new(
                    NF::G4,
                    ND::Half,
                ),
                Note::new(
                    NF::D4,
                    ND::Full,
                ),
                Note::new(
                    NF::G3,
                    ND::Full,
                ),
            ],
            25,
        ),
        Phrase::new(
            [
                Note::new(
                    NF::Fs4,
                    ND::Full,
                ),
                Note::new(
                    NF::Fs4,
                    ND::Full,
                ),
                Note::new(
                    NF::Fs4,
                    ND::Full,
                ),
                Note::new(
                    NF::Fs4,
                    ND::Half,
                ),
                Note::new(
                    NF::G4,
                    ND::Half,
                ),
                Note::new(
                    NF::D4,
                    ND::Full,
                ),
                Note::new(
                    NF::D3,
                    ND::Full,
                ),
            ],
            25,
        ),
    ],
    400,
);

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());

    spawner
        .spawn(
            play_noise(
                p.P0_00
                    .into(),
                p.P0_14
                    .into(),
            ),
        )
        .unwrap();
}

#[embassy_executor::task]
async fn play_noise(
    speaker_pin: Peri<'static, AnyPin>,
    button_pin: Peri<'static, AnyPin>,
) {
    let mut speaker = Output::new(
        speaker_pin,
        Level::Low,
        OutputDrive::Standard,
    );
    let mut button = Input::new(
        button_pin,
        Pull::Up,
    );

    loop {
        LAMOUR
            .play(&mut speaker)
            .await;

        // use Note as N;
        // for held_note in [N::B3, N::A3, N::G3, N::D3] {
        //     N::Fs4
        //         .play(
        //             &mut speaker,
        //             TONE_FULL,
        //         )
        //         .await;
        //     Timer::after_millis(MS_BETWEEN_TONES);

        //     N::Fs4
        //         .play(
        //             &mut speaker,
        //             TONE_FULL,
        //         )
        //         .await;
        //     Timer::after_millis(MS_BETWEEN_TONES);

        //     N::Fs4
        //         .play(
        //             &mut speaker,
        //             TONE_HALF,
        //         )
        //         .await;
        //     Timer::after_millis(MS_BETWEEN_TONES);

        //     N::G4
        //         .play(
        //             &mut speaker,
        //             TONE_HALF,
        //         )
        //         .await;
        //     Timer::after_millis(MS_BETWEEN_TONES);

        //     N::Fs4
        //         .play(
        //             &mut speaker,
        //             TONE_FULL,
        //         )
        //         .await;
        //     Timer::after_millis(MS_BETWEEN_TONES);

        //     N::D4
        //         .play(
        //             &mut speaker,
        //             TONE_FULL,
        //         )
        //         .await;
        //     Timer::after_millis(MS_BETWEEN_TONES);

        //     held_note
        //         .play(
        //             &mut speaker,
        //             TONE_FULL * 2,
        //         )
        //         .await;

        //     Timer::after_millis(MS_BETWEEN_PHRASES);
        // }

        button
            .wait_for_falling_edge()
            .await;
    }
}
