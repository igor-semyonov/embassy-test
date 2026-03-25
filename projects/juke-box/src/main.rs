#![no_std]
#![no_main]

// use defmt::info;
use core::sync::atomic::{AtomicUsize, Ordering};
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_futures::select::{Either, select};
use embassy_nrf::gpio::{
    Input, Level, Output, OutputDrive, Pull,
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
    frequency: Option<NoteFrequency>,
    duration: NoteDuration,
}
impl Note {
    const fn new(
        frequency: Option<NoteFrequency>,
        duration: NoteDuration,
    ) -> Self {
        Self {
            frequency,
            duration,
        }
    }
    async fn play(&self, speaker: &mut Output<'_>) {
        if let Some(frequency) = &self.frequency {
            let period = frequency.period_micros();
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
        } else {
            Timer::after_micros(
                self.duration
                    .micros(),
            )
            .await;
        }
    }
}
enum NoteFrequency {
    G4,
    Fs4,
    E4,
    D4,
    B3,
    A3,
    G3,
    D3,
    B2,
    G2,
    Fs2,
}
impl NoteFrequency {
    const fn frequency(&self) -> u64 {
        use NoteFrequency as N;
        match self {
            N::G4 => 392,
            N::Fs4 => 370,
            N::E4 => 330,
            N::D4 => 294,
            N::B3 => 247,
            N::A3 => 220,
            N::G3 => 196,
            N::D3 => 147,
            N::B2 => 123,
            N::G2 => 98,
            N::Fs2 => 93,
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

struct Phrase {
    notes: &'static [Note],
    pause: u64,
}
impl Phrase {
    const fn new(
        notes: &'static [Note],
        pause: u64,
    ) -> Self {
        Self {
            notes,
            pause,
        }
    }
    async fn play(&self, speaker: &mut Output<'_>) {
        for note in self.notes {
            note.play(speaker)
                .await;
            Timer::after_millis(self.pause).await;
        }
    }
}

struct Song {
    phrases: &'static [Phrase],
    pause: u64,
}
impl Song {
    const fn new(
        phrases: &'static [Phrase],
        pause: u64,
    ) -> Self {
        Self {
            phrases,
            pause,
        }
    }
    async fn play(&self, speaker: &mut Output<'_>) {
        for phrase in self.phrases {
            phrase
                .play(speaker)
                .await;
            Timer::after_millis(self.pause).await;
        }
    }
}

static CURRENT_SONG_IDX: AtomicUsize = AtomicUsize::new(0);
static SONGS: &[&Song] = &[
    &LAMOUR,
    &REMEMBER_VOCAL,
    &REMEMBER_BASSLINE,
];

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());

    let mut button_a = Input::new(
        p.P0_14,
        Pull::Up,
    );
    let mut button_b = Input::new(
        p.P0_23,
        Pull::Up,
    );
    let mut speaker = Output::new(
        p.P0_00,
        Level::Low,
        OutputDrive::Standard,
    );

    loop {
        let race_result = select(
            play_noise(
                &mut speaker,
                &mut button_a,
            ),
            button_b.wait_for_falling_edge(),
        )
        .await;
        match race_result {
            Either::First(_) => {}
            Either::Second(_) => {
                let current_song_idx = CURRENT_SONG_IDX
                    .load(Ordering::Relaxed);
                let new_song_idx = if current_song_idx
                    == SONGS.len() - 1
                {
                    0
                } else {
                    current_song_idx + 1
                };
                CURRENT_SONG_IDX.store(
                    new_song_idx,
                    Ordering::Relaxed,
                );
            }
        };
    }
}

async fn play_noise(
    speaker: &mut Output<'_>,
    button: &mut Input<'_>,
) {
    loop {
        // reading from CURRENT_SONG_IDX should be safe in this context since the tasks aer
        // executing cooperatively
        SONGS[CURRENT_SONG_IDX.load(Ordering::Relaxed)]
            .play(speaker)
            .await;
        button
            .wait_for_falling_edge()
            .await;
    }
}

use NoteDuration as ND;
use NoteFrequency as NF;
static REMEMBER_VOCAL: Song = Song::new(
    &[Phrase::new(
        &[
            Note::new(
                Some(NF::Fs4),
                ND::Eighth,
            ),
            Note::new(
                Some(NF::E4),
                ND::Eighth,
            ),
            Note::new(
                Some(NF::D4),
                ND::Eighth,
            ),
            Note::new(
                Some(NF::B3),
                ND::Full,
            ),
            Note::new(
                None, // Rest
                ND::Full,
            ),
            Note::new(
                Some(NF::Fs4),
                ND::Eighth,
            ),
            Note::new(
                Some(NF::E4),
                ND::Eighth,
            ),
            Note::new(
                Some(NF::D4),
                ND::Eighth,
            ),
            Note::new(
                Some(NF::B3),
                ND::Full,
            ),
        ],
        25,
    )],
    400,
);
static LAMOUR: Song = Song::new(
    &[
        Phrase::new(
            &[
                Note::new(
                    Some(NF::Fs4),
                    ND::Full,
                ),
                Note::new(
                    Some(NF::Fs4),
                    ND::Full,
                ),
                Note::new(
                    Some(NF::Fs4),
                    ND::Full,
                ),
                Note::new(
                    Some(NF::Fs4),
                    ND::Half,
                ),
                Note::new(
                    Some(NF::G4),
                    ND::Half,
                ),
                Note::new(
                    Some(NF::D4),
                    ND::Full,
                ),
                Note::new(
                    Some(NF::B3),
                    ND::Full,
                ),
            ],
            25,
        ),
        Phrase::new(
            &[
                Note::new(
                    Some(NF::Fs4),
                    ND::Full,
                ),
                Note::new(
                    Some(NF::Fs4),
                    ND::Full,
                ),
                Note::new(
                    Some(NF::Fs4),
                    ND::Full,
                ),
                Note::new(
                    Some(NF::Fs4),
                    ND::Half,
                ),
                Note::new(
                    Some(NF::G4),
                    ND::Half,
                ),
                Note::new(
                    Some(NF::D4),
                    ND::Full,
                ),
                Note::new(
                    Some(NF::A3),
                    ND::Full,
                ),
            ],
            25,
        ),
        Phrase::new(
            &[
                Note::new(
                    Some(NF::Fs4),
                    ND::Full,
                ),
                Note::new(
                    Some(NF::Fs4),
                    ND::Full,
                ),
                Note::new(
                    Some(NF::Fs4),
                    ND::Full,
                ),
                Note::new(
                    Some(NF::Fs4),
                    ND::Half,
                ),
                Note::new(
                    Some(NF::G4),
                    ND::Half,
                ),
                Note::new(
                    Some(NF::D4),
                    ND::Full,
                ),
                Note::new(
                    Some(NF::G3),
                    ND::Full,
                ),
            ],
            25,
        ),
        Phrase::new(
            &[
                Note::new(
                    Some(NF::Fs4),
                    ND::Full,
                ),
                Note::new(
                    Some(NF::Fs4),
                    ND::Full,
                ),
                Note::new(
                    Some(NF::Fs4),
                    ND::Full,
                ),
                Note::new(
                    Some(NF::Fs4),
                    ND::Half,
                ),
                Note::new(
                    Some(NF::G4),
                    ND::Half,
                ),
                Note::new(
                    Some(NF::D4),
                    ND::Full,
                ),
                Note::new(
                    Some(NF::D3),
                    ND::Full,
                ),
            ],
            25,
        ),
    ],
    400,
);
static REMEMBER_BASSLINE: Song = Song::new(
    &[Phrase::new(
        &[
            Note::new(
                Some(NF::B2),
                ND::Full,
            ),
            Note::new(
                Some(NF::D3),
                ND::Full,
            ),
            Note::new(
                Some(NF::G2),
                ND::Full,
            ),
            Note::new(
                Some(NF::G2),
                ND::Full,
            ),
            Note::new(
                Some(NF::Fs2),
                ND::Full,
            ),
            Note::new(
                Some(NF::D3),
                ND::Full,
            ),
            Note::new(
                Some(NF::G2),
                ND::Full,
            ),
            Note::new(
                Some(NF::G2),
                ND::Full,
            ),
        ],
        25,
    )],
    400,
);
