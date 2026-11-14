//! The whole app, its real window drawn offscreen and driven by keys, as a
//! user would: nothing appears on the desktop. `main` below keeps it
//! offscreen however it is started, and draws with the GPU renderer, so
//! particles and textures are drawn as they are on screen.
//!
//! Run by `cargo test`, or one test by name: `cargo test --test scene --
//! frames_for_review`.

use std::io::{BufRead, BufReader, Write};
use std::os::fd::FromRawFd;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use chrono::{NaiveDate, NaiveTime};
use cxx_qt_lib::{QGuiApplication, QQuickStyle, QString};
use grove::garden::{Outcome, Plant};
use grove::options::Options;
use grove::qml::shared;
use grove::qml::testing::{wait, wait_until, Key, Scene};
use grove::session::State;
use grove::weather::Kind;
use tempfile::TempDir;

/// How a test ended, when it did not fail.
enum Ended {
    Passed,
    /// Not run, for the reason given.
    Skipped(&'static str),
}

/// What each test starts from: a garden, theme and language of its own.
struct Start {
    dir: TempDir,
    /// What Qt may say during this test without failing it: parts of lines,
    /// each with its reason where the test names it.
    tolerated: std::cell::RefCell<Vec<&'static str>>,
}

impl Start {
    fn new() -> Self {
        Self {
            dir: TempDir::new().expect("a folder for the test"),
            tolerated: Default::default(),
        }
    }

    /// A clear noon on the first of July, with nothing of the desktop's read
    /// or written: an empty theme folder gives the default accent, and no
    /// language file means English.
    fn options(&self, seconds: i32) -> Options {
        Options {
            seconds,
            date: NaiveDate::from_ymd_opt(2026, 7, 1),
            time: NaiveTime::from_hms_opt(12, 0, 0),
            weather: Some(Kind::Clear),
            garden_path: Some(self.dir.path().join("garden.json")),
            theme_directory: Some(self.dir.path().join("theme")),
            language_file: Some(self.dir.path().join("language")),
            notification_service: "org.grove.tests.NoOneListening".into(),
            ..Options::default()
        }
    }

    /// Lets Qt say something during this test: any line with `part` in it.
    fn tolerate(&self, part: &'static str) {
        self.tolerated.borrow_mut().push(part);
    }

    /// The app and its window, muted, with sessions of that many seconds.
    fn scene(&self, seconds: i32) -> Scene {
        self.open(self.options(seconds))
    }

    fn open(&self, options: Options) -> Scene {
        Scene::open(options, true).expect("the window was not shown")
    }
}

fn state() -> State {
    shared::read(|app| app.session().state())
}

/// A plant for the garden: odd seeds grew, even ones wilted.
fn plant(seed: i32) -> Plant {
    let grew = seed % 2 == 1;
    Plant {
        seed,
        outcome: if grew { Outcome::Done } else { Outcome::Wilted },
        season: "summer".into(),
        weather: "clear".into(),
        progress: if grew { 1.0 } else { 0.4 },
        when: NaiveDate::from_ymd_opt(2026, 7, seed as u32)
            .unwrap()
            .and_hms_opt(9, 0, 0)
            .unwrap(),
        minutes: if grew { 25 } else { 0 },
    }
}

/// Puts a plant in the garden, and lets the window hear of it: the model
/// the garden view shows is told on the next turn of the event loop.
fn record(plant: Plant) {
    shared::update(|app| app.record(plant));
    wait(20);
}

fn space_plants_a_seed(start: &Start) -> Ended {
    let scene = start.scene(10);
    assert_eq!(state(), State::Idle);
    scene.press(Key::Space);
    assert_eq!(state(), State::Growing);
    Ended::Passed
}

fn a_session_runs_to_the_end_and_is_kept(start: &Start) -> Ended {
    let scene = start.scene(2);
    scene.press(Key::Space);
    assert!(
        wait_until(5000, || state() == State::Done),
        "the session never finished"
    );
    assert_eq!(shared::read(|app| app.garden().grown()), 1);
    Ended::Passed
}

fn giving_up_asks_first_and_wilts_the_plant(start: &Start) -> Ended {
    let scene = start.scene(10);
    scene.press(Key::Space);
    scene.press(Key::Space);
    assert!(wait_until(5000, || scene.is("confirm.opened")), "giving up did not ask");
    // Still growing until the answer is yes.
    assert_eq!(state(), State::Growing);
    scene.press(Key::Return);
    assert_eq!(state(), State::Wilted);
    assert_eq!(shared::read(|app| app.garden().wilted()), 1);
    Ended::Passed
}

fn the_garden_opens_takes_the_keys_and_closes(start: &Start) -> Ended {
    let scene = start.scene(10);
    for seed in 1..=3 {
        record(plant(seed));
    }
    assert!(!scene.is("find('gardenLoader').item"));

    scene.press(Key::G);
    assert!(
        wait_until(5000, || scene.is("find('gardenLoader').item")),
        "the garden did not open"
    );
    assert!(
        wait_until(5000, || scene.is("find('gardenGrid').activeFocus")),
        "the garden did not take the keys"
    );
    assert_eq!(scene.number("find('gardenGrid').currentIndex"), 0);
    scene.press(Key::Right);
    assert_eq!(scene.number("find('gardenGrid').currentIndex"), 1);
    // The tile with the keys wears the accent: the theme's fallback, with no
    // Omarchy theme to read.
    let border = "find('gardenGrid').currentItem.children[0].border.color";
    assert!(
        wait_until(5000, || scene.text(border).eq_ignore_ascii_case("#FFD60A")),
        "{}",
        scene.text(border)
    );

    scene.press(Key::Escape);
    assert!(
        wait_until(5000, || !scene.is("find('gardenLoader').item")),
        "the garden did not close"
    );
    Ended::Passed
}

fn every_service_is_started_with_the_window(start: &Start) -> Ended {
    // Following the theme and the language, the notification and the sounds
    // live in QML singletons, which the engine only makes when something
    // reads from them. Rust reads from them all at launch, so no edit to the
    // QML can switch one off. With a garden that cannot be read, the window
    // never gets as far as asking the Notifier for its problem, so here the
    // QML alone would leave it unmade.
    let options = start.options(10);
    std::fs::write(options.garden_path.as_ref().unwrap(), "this is not json").unwrap();
    let _scene = start.open(options);
    assert_eq!(shared::missing(), Vec::<&str>::new());
    Ended::Passed
}

fn m_mutes_and_unmutes_the_sounds(start: &Start) -> Ended {
    let scene = start.scene(10);
    assert!(scene.is("Sounds.muted"));
    scene.press(Key::M);
    assert!(wait_until(5000, || !scene.is("Sounds.muted")), "M did not unmute");
    assert!(!shared::read(|app| app.muted()));
    scene.press(Key::M);
    assert!(wait_until(5000, || scene.is("Sounds.muted")), "M did not mute");
    Ended::Passed
}

fn the_language_is_followed_while_the_window_is_open(start: &Start) -> Ended {
    // When every text changes at once, Qt's own Material dialog reports a
    // binding loop in its width, which then settles. The C++ Grove's QML
    // was the same; no test of its window ever changed the language.
    start.tolerate("QML Dialog: Binding loop detected for property \"implicitWidth\"");
    start.tolerate("QtQuick/Controls/Material/Dialog.qml");
    // A garden that cannot be read, so there is a problem for Rust to put
    // into words as well as the QML's own texts.
    let options = start.options(25 * 60);
    std::fs::write(options.garden_path.as_ref().unwrap(), "this is not json").unwrap();
    let language = options.language_file.clone().unwrap();
    let scene = start.open(options);
    let hint = "find('heading').children[1].text";
    assert_eq!(scene.text("Translator.language"), "en");
    assert_eq!(scene.text(hint), "Press Space to plant a seed");
    let problem = scene.text("Garden.problem");
    assert!(problem.starts_with("The garden file could not be read"), "{problem}");

    std::fs::write(&language, "de\n").unwrap();
    assert!(
        wait_until(5000, || scene.text("Translator.language") == "de"),
        "the new language was not noticed"
    );
    // The window's texts are said again, the counted one in its plural, and
    // so is the problem that came from Rust.
    assert!(
        wait_until(5000, || scene.text(hint) != "Press Space to plant a seed"),
        "the window was not retranslated"
    );
    assert_eq!(scene.text("find('heading').children[0].text"), "25 Minuten");
    assert!(
        wait_until(5000, || scene.text("Garden.problem") != problem),
        "the problem kept its English"
    );
    assert!(!scene.is("Translator.rightToLeft"));
    Ended::Passed
}

fn the_theme_is_followed_while_the_window_is_open(start: &Start) -> Ended {
    let options = start.options(10);
    let theme = options.theme_directory.clone().unwrap().join("theme");
    let scene = start.open(options);
    assert!(scene.text("Theme.accent").eq_ignore_ascii_case("#FFD60A"));
    // The theme arrives after Grove started, as when Omarchy is installed
    // later, and then changes.
    std::fs::create_dir_all(&theme).unwrap();
    std::fs::write(theme.join("colors.toml"), "accent = \"#112233\"\n").unwrap();
    assert!(
        wait_until(8000, || scene.text("Theme.accent") == "#112233"),
        "the theme was not noticed"
    );
    std::fs::write(theme.join("colors.toml"), "accent = \"#445566\"\n").unwrap();
    assert!(
        wait_until(8000, || scene.text("Theme.accent") == "#445566"),
        "the change was not noticed"
    );
    Ended::Passed
}

fn a_train_passes_even_when_muted(start: &Start) -> Ended {
    // A train asked for by name comes three seconds into the session; muted,
    // there is no sound to wait for, and its picture still goes.
    let scene = start.open(Options {
        sound_test: Some(grove::sounds::Kind::Train),
        ..start.options(60)
    });
    assert!(!scene.is("find('train').running"));
    scene.press(Key::Space);
    assert!(
        wait_until(8000, || scene.is("find('train').running")),
        "no train passed"
    );
    Ended::Passed
}

fn the_landscape_is_drawn(start: &Start) -> Ended {
    let scene = start.scene(10);
    // The layers draw on another thread; the field brings many tones to the
    // bottom of the window once it is there.
    assert!(
        wait_until(5000, || scene.grab().tones_in(0.8, 1.0) > 200),
        "the field was not drawn"
    );
    // And the hills to the middle, each layer in its own time.
    assert!(
        wait_until(5000, || scene.grab().tones_in(0.3, 0.6) > 50),
        "the hills were not drawn"
    );
    Ended::Passed
}

/// Not a test: with GROVE_SHOTS set to a folder, saves the scene at several
/// hours, seasons and weathers there for a person to look at.
fn frames_for_review(start: &Start) -> Ended {
    let Some(out) = std::env::var_os("GROVE_SHOTS").map(PathBuf::from) else {
        return Ended::Skipped("set GROVE_SHOTS to a folder to save frames");
    };
    std::fs::create_dir_all(&out).expect("the folder for the frames");
    let save = |scene: &Scene, name: &str| assert!(scene.grab().save(&out.join(format!("{name}.png"))), "{name}");
    let scenes = [
        ("summer-noon", (2026, 7, 1), (12, 0), Kind::Clear),
        ("summer-dusk", (2026, 7, 1), (20, 40), Kind::Clear),
        ("winter-noon", (2026, 1, 15), (12, 0), Kind::Clear),
        ("night", (2026, 7, 1), (1, 0), Kind::Clear),
        ("rain", (2026, 7, 1), (12, 0), Kind::Rain),
        ("snow", (2026, 1, 15), (12, 0), Kind::Snow),
        ("fog", (2026, 10, 15), (9, 0), Kind::Fog),
    ];
    for (name, (year, month, day), (hour, minute), weather) in scenes {
        let scene = start.open(Options {
            date: NaiveDate::from_ymd_opt(year, month, day),
            time: NaiveTime::from_hms_opt(hour, minute, 0),
            weather: Some(weather),
            ..start.options(25 * 60)
        });
        wait(2500);
        save(&scene, name);
    }
    // The night train, stopped at the crossing with its signal ringing.
    {
        let scene = start.open(Options {
            time: NaiveTime::from_hms_opt(1, 0, 0),
            ..start.options(25 * 60)
        });
        scene.evaluate("find('train').show(0.6)");
        wait(2500);
        save(&scene, "night-train");
    }
    let scene = start.scene(10);
    for seed in 1..=6 {
        record(plant(seed));
    }
    scene.press(Key::G);
    scene.press(Key::Right);
    wait(1000);
    save(&scene, "garden");
    Ended::Passed
}

/// Not a test: with GROVE_README_SHOTS set to a folder, saves the README's
/// two pictures there. A session's plant comes from a random seed, so look
/// before keeping one.
fn frames_for_readme(start: &Start) -> Ended {
    let Some(out) = std::env::var_os("GROVE_README_SHOTS").map(PathBuf::from) else {
        return Ended::Skipped("set GROVE_README_SHOTS to a folder to save the README's pictures");
    };
    std::fs::create_dir_all(&out).expect("the folder for the pictures");
    // Sound stays on but silent, so the window shows no "sounds are off".
    let unmuted = |options: Options| {
        Scene::open(
            Options {
                sound_volume: 0.0,
                ..options
            },
            false,
        )
        .expect("the window was not shown")
    };

    // An autumn plant nearly grown on a clear morning, a minute and a half
    // from the end of its session.
    {
        let scene = unmuted(Options {
            date: NaiveDate::from_ymd_opt(2026, 10, 20),
            time: NaiveTime::from_hms_opt(9, 45, 0),
            ..start.options(25 * 60)
        });
        scene.press(Key::Space);
        shared::update(|app| app.advance((25 * 60 - 89) * 1000));
        wait(2500);
        assert!(scene.grab().save(&out.join("grove.png")));
    }

    // Twelve October days in the garden, ten grown and two wilted: three
    // full rows, nothing cut off at the bottom.
    let scene = unmuted(start.options(10));
    let weathers = [
        "clear", "rain", "clear", "cloudy", "clear", "cloudy", "clear", "cloudy", "clear", "fog", "clear", "rain",
    ];
    for (day, weather) in (0u32..).zip(weathers) {
        let grew = day != 3 && day != 8;
        record(Plant {
            seed: 101 + day as i32 * 37,
            outcome: if grew { Outcome::Done } else { Outcome::Wilted },
            season: "autumn".into(),
            weather: weather.into(),
            progress: if grew { 1.0 } else { 0.3 + 0.1 * f64::from(day % 3) },
            when: NaiveDate::from_ymd_opt(2026, 10, 9 + day / 2 + day % 2)
                .unwrap()
                .and_hms_opt(9 + day % 5, 0, 0)
                .unwrap(),
            minutes: if grew { 25 } else { 0 },
        });
    }
    scene.press(Key::G);
    wait(1500);
    assert!(scene.grab().save(&out.join("garden.png")));
    Ended::Passed
}

/// Bytes the program holds from the heap, including large blocks mapped on
/// their own.
fn heap_in_use() -> i64 {
    // SAFETY: mallinfo2 only reads the allocator's own counters.
    let info = unsafe { libc::mallinfo2() };
    (info.uordblks + info.hblkhd) as i64
}

/// The heap once JavaScript garbage is collected and deferred deletes are
/// done, so only what is really kept is counted.
fn settled_heap(scene: &Scene) -> i64 {
    for _ in 0..3 {
        scene.evaluate("gc()");
        wait(50);
    }
    heap_in_use()
}

/// The slope of the least-squares line through the readings, per reading.
fn slope(values: &[i64]) -> f64 {
    let n = values.len() as f64;
    let (mut sum_x, mut sum_y, mut sum_xy, mut sum_xx) = (0.0, 0.0, 0.0, 0.0);
    for (x, y) in values.iter().enumerate() {
        let (x, y) = (x as f64, *y as f64);
        sum_x += x;
        sum_y += y;
        sum_xy += x * y;
        sum_xx += x * x;
    }
    (n * sum_xy - sum_x * sum_y) / (n * sum_xx - sum_x * sum_x)
}

/// Memory still referenced but never let go, which no leak checker sees,
/// shows as a heap that keeps growing. The heap is read after each of forty
/// rounds of one part of a day (opening the garden, giving up, or finishing)
/// once garbage is collected and deferred deletes are done, and the line
/// through those readings must rise less than 10 KB a round. A garden tile
/// with its plant is about 100 KB, so losing one a round would show; caches
/// filling up early and then holding steady do not. It takes a minute or
/// more, so bin/check-leaks runs it (GROVE_SOAK=1) rather than bin/test.
fn repeating_the_day_does_not_grow_the_heap(start: &Start, part: &str) -> Ended {
    if std::env::var_os("GROVE_SOAK").is_none() {
        return Ended::Skipped("set GROVE_SOAK=1 to run; bin/check-leaks does");
    }
    let scene = start.scene(1);
    for seed in 1..=6 {
        record(plant(seed));
    }
    let once = || match part {
        "garden" => {
            scene.press(Key::G);
            assert!(
                wait_until(5000, || scene.is("find('gardenLoader').item")),
                "the garden did not open"
            );
            scene.press(Key::Escape);
            assert!(
                wait_until(5000, || !scene.is("find('gardenLoader').item")),
                "the garden did not close"
            );
        }
        "giveup" => {
            scene.press(Key::Space);
            scene.press(Key::Space);
            assert!(wait_until(5000, || scene.is("confirm.opened")), "giving up did not ask");
            scene.press(Key::Return);
            assert!(
                wait_until(5000, || !scene.is("confirm.opened")),
                "the question did not close"
            );
        }
        _ => {
            scene.press(Key::Space);
            assert!(
                wait_until(3000, || state() == State::Done),
                "the session never finished"
            );
        }
    };
    for _ in 0..5 {
        once();
    }
    // For bin/check-leaks --canary: keep a garden tile's worth a round, to
    // show that the test sees memory kept like that.
    let canary = std::env::var_os("GROVE_LEAK_CANARY").is_some();
    let mut kept = Vec::new();
    let mut heap = Vec::new();
    let times = 40;
    for _ in 0..times {
        once();
        if canary {
            kept.push(vec![b'x'; 100 * 1024]);
        }
        heap.push(settled_heap(&scene));
    }
    let per_round = slope(&heap);
    println!(
        "{part} x{times}: heap grew by {} bytes, {per_round:.0} bytes a round",
        heap[heap.len() - 1] - heap[0]
    );
    // The readings themselves say whether a rise is steady or a single step.
    let readings: Vec<i64> = heap.iter().map(|bytes| (bytes - heap[0]) / 1024).collect();
    assert!(
        per_round < 10.0 * 1024.0,
        "rising {per_round:.0} bytes a round; KB over the first reading: {readings:?}"
    );
    Ended::Passed
}

fn soak_garden(start: &Start) -> Ended {
    repeating_the_day_does_not_grow_the_heap(start, "garden")
}

fn soak_giveup(start: &Start) -> Ended {
    repeating_the_day_does_not_grow_the_heap(start, "giveup")
}

fn soak_finish(start: &Start) -> Ended {
    repeating_the_day_does_not_grow_the_heap(start, "finish")
}

type Test = fn(&Start) -> Ended;

const TESTS: [(&str, Test); 15] = [
    (
        "every_service_is_started_with_the_window",
        every_service_is_started_with_the_window,
    ),
    ("space_plants_a_seed", space_plants_a_seed),
    (
        "a_session_runs_to_the_end_and_is_kept",
        a_session_runs_to_the_end_and_is_kept,
    ),
    (
        "giving_up_asks_first_and_wilts_the_plant",
        giving_up_asks_first_and_wilts_the_plant,
    ),
    (
        "the_garden_opens_takes_the_keys_and_closes",
        the_garden_opens_takes_the_keys_and_closes,
    ),
    ("m_mutes_and_unmutes_the_sounds", m_mutes_and_unmutes_the_sounds),
    (
        "the_language_is_followed_while_the_window_is_open",
        the_language_is_followed_while_the_window_is_open,
    ),
    (
        "the_theme_is_followed_while_the_window_is_open",
        the_theme_is_followed_while_the_window_is_open,
    ),
    ("a_train_passes_even_when_muted", a_train_passes_even_when_muted),
    ("the_landscape_is_drawn", the_landscape_is_drawn),
    ("soak_garden", soak_garden),
    ("soak_giveup", soak_giveup),
    ("soak_finish", soak_finish),
    ("frames_for_review", frames_for_review),
    ("frames_for_readme", frames_for_readme),
];

/// Everything written to standard error, which is where Qt says what it has
/// to say. The lines are passed on, and kept for the tests to look through.
struct Said {
    lines: Arc<Mutex<Vec<String>>>,
}

impl Said {
    /// Starts listening to standard error.
    fn listen() -> Self {
        let lines = Arc::new(Mutex::new(Vec::new()));
        let mut ends = [0; 2];
        // SAFETY: plain file descriptor calls, checked, made before any
        // other thread exists; the reading end and the copy of the old
        // standard error each get one owner.
        let (heard, mut passed_on) = unsafe {
            assert_eq!(libc::pipe(ends.as_mut_ptr()), 0, "a pipe for standard error");
            let old = libc::dup(2);
            assert!(
                old >= 0 && libc::dup2(ends[1], 2) >= 0,
                "standard error could not be redirected"
            );
            libc::close(ends[1]);
            (std::fs::File::from_raw_fd(ends[0]), std::fs::File::from_raw_fd(old))
        };
        let kept = Arc::clone(&lines);
        std::thread::spawn(move || {
            for line in BufReader::new(heard).lines().map_while(Result::ok) {
                let _ = writeln!(passed_on, "{line}");
                kept.lock().unwrap_or_else(|poisoned| poisoned.into_inner()).push(line);
            }
        });
        Self { lines }
    }

    /// How many lines have been said so far.
    fn mark(&self) -> usize {
        self.lines.lock().unwrap_or_else(|poisoned| poisoned.into_inner()).len()
    }

    /// The lines said since a mark.
    fn since(&self, mark: usize) -> Vec<String> {
        // What was just written takes a moment to come through the pipe.
        std::thread::sleep(std::time::Duration::from_millis(50));
        self.lines.lock().unwrap_or_else(|poisoned| poisoned.into_inner())[mark..].to_vec()
    }
}

fn main() {
    // Never on the desktop, however the program is started: the session's
    // own QT_QPA_PLATFORM (Omarchy sets wayland) is overridden. The renderer
    // is the GPU's, as on screen, unless one is named.
    std::env::set_var("QT_QPA_PLATFORM", "offscreen");
    if std::env::var_os("QT_QUICK_BACKEND").is_none() {
        std::env::set_var("QT_QUICK_BACKEND", "rhi");
        std::env::set_var("QSG_RHI_BACKEND", "opengl");
    }
    std::env::set_var("QT_FORCE_STDERR_LOGGING", "1");
    let said = Said::listen();

    let wanted: Vec<String> = std::env::args()
        .skip(1)
        .filter(|argument| !argument.starts_with('-'))
        .collect();
    let application = QGuiApplication::new();
    QQuickStyle::set_style(&QString::from("Material"));

    let (mut passed, mut skipped, mut failed) = (0, 0, Vec::new());
    for (name, test) in TESTS {
        if !wanted.is_empty() && !wanted.iter().any(|want| name.contains(want.as_str())) {
            continue;
        }
        let mark = said.mark();
        let start = Start::new();
        let ended = catch_unwind(AssertUnwindSafe(|| test(&start)));
        // Any warning is a failure: a binding loop or a QML error is a bug
        // even when the window still looks right.
        let tolerated = start.tolerated.borrow().clone();
        let warnings: Vec<String> = said
            .since(mark)
            .into_iter()
            .filter(|line| !tolerated.iter().any(|part| line.contains(part)))
            .collect();
        match ended {
            Ok(Ended::Passed) if warnings.is_empty() => {
                println!("test {name} ... ok");
                passed += 1;
            }
            Ok(Ended::Skipped(why)) => {
                println!("test {name} ... skipped: {why}");
                skipped += 1;
            }
            Ok(Ended::Passed) => {
                println!("test {name} ... FAILED: Qt had something to say:");
                for line in &warnings {
                    println!("    {line}");
                }
                failed.push(name);
            }
            Err(_) => {
                println!("test {name} ... FAILED");
                failed.push(name);
            }
        }
    }
    drop(application);
    println!(
        "\nscene tests: {passed} passed, {} failed, {skipped} skipped",
        failed.len()
    );
    std::process::exit(i32::from(!failed.is_empty()));
}
