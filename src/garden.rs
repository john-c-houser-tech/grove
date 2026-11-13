//! Every plant that was finished or wilted, kept in one small JSON file so
//! the garden fills up over days. The newest plant comes first.

use std::fs::{self, File};
use std::io::{self, Write};
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use chrono::{DateTime, NaiveDateTime};
use serde_json::{json, Map, Value};

/// How a session ended.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    Done,
    Wilted,
}

impl Outcome {
    /// The name QML and the garden file know the outcome by.
    pub fn name(self) -> &'static str {
        match self {
            Outcome::Done => "done",
            Outcome::Wilted => "wilted",
        }
    }

    fn named(name: &str) -> Option<Self> {
        [Outcome::Done, Outcome::Wilted]
            .into_iter()
            .find(|outcome| outcome.name() == name)
    }
}

/// One plant as the garden keeps it.
#[derive(Clone, Debug, PartialEq)]
pub struct Plant {
    /// Grows the same plant again (see GrowingPlant.qml).
    pub seed: i32,
    pub outcome: Outcome,
    /// The season and the weather it grew in, by their names.
    pub season: String,
    pub weather: String,
    /// How much of the plant had grown, as the window showed it, 0 to 1.
    pub progress: f64,
    /// When the session ended, in local time.
    pub when: NaiveDateTime,
    /// How long the session was, if the plant grew to the end; else 0.
    pub minutes: i32,
}

/// Why the garden cannot be read or saved, for the window to put into words.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Problem {
    /// The file is there but could not be opened.
    CannotOpen,
    /// The file is not a garden. It is left untouched.
    CannotRead,
    /// The folder for the file could not be made.
    NoFolder,
    /// Another Grove held the lock for too long.
    Busy,
    /// Writing failed, for the reason given.
    CannotSave(String),
}

/// How the list of plants changed, for a model showing it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Change {
    /// One plant was put first.
    Inserted,
    /// The whole list was read again.
    Reset,
}

/// What reading the file found.
enum Read {
    Missing,
    Ok(Vec<Plant>),
    Unreadable,
    Invalid,
}

/// How long to wait for another Grove to finish saving.
const LOCK_WAIT: Duration = Duration::from_secs(5);
/// How dates are written, as Qt's ISO dates were.
const WHEN_FORMAT: &str = "%Y-%m-%dT%H:%M:%S";

/// The garden and its file.
#[derive(Debug)]
pub struct Garden {
    path: PathBuf,
    /// Newest first.
    plants: Vec<Plant>,
    problem: Option<Problem>,
    /// A file that could not be read is left exactly as it is, so nothing is
    /// saved over it.
    read_only: bool,
}

impl Garden {
    /// Opens the garden kept at `path`, which need not exist yet.
    pub fn open(path: impl Into<PathBuf>) -> Self {
        let mut garden = Self {
            path: path.into(),
            plants: Vec::new(),
            problem: None,
            read_only: false,
        };
        match garden.read() {
            Read::Missing => {}
            Read::Ok(plants) => garden.plants = plants,
            Read::Unreadable => {
                garden.read_only = true;
                garden.problem = Some(Problem::CannotOpen);
            }
            Read::Invalid => {
                garden.read_only = true;
                garden.problem = Some(Problem::CannotRead);
            }
        }
        garden
    }

    /// ~/.local/share/grove/garden.json, or wherever XDG_DATA_HOME says.
    pub fn default_path(variable: impl Fn(&str) -> Option<String>) -> PathBuf {
        let base = variable("XDG_DATA_HOME")
            .filter(|base| !base.is_empty())
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(variable("HOME").unwrap_or_default()).join(".local/share"));
        base.join("grove/garden.json")
    }

    /// The plants, newest first.
    pub fn plants(&self) -> &[Plant] {
        &self.plants
    }

    /// How many plants grew to the end.
    pub fn grown(&self) -> i32 {
        self.plants
            .iter()
            .filter(|plant| plant.outcome == Outcome::Done)
            .count() as i32
    }

    /// How many plants wilted.
    pub fn wilted(&self) -> i32 {
        self.plants.len() as i32 - self.grown()
    }

    /// Time spent on plants that grew to the end; a wilted plant earns none.
    pub fn focused_minutes(&self) -> i32 {
        self.plants
            .iter()
            .filter(|plant| plant.outcome == Outcome::Done)
            .map(|plant| plant.minutes)
            .sum()
    }

    /// `None` when the garden is fine, otherwise why it cannot be read or saved.
    pub fn problem(&self) -> Option<&Problem> {
        self.problem.as_ref()
    }

    /// Adds a plant and saves the garden. Another Grove may be open on the
    /// same file, so this re-reads it under a lock first and keeps the plants
    /// it saved. A file that cannot be read is not saved over, and the plant
    /// is still shown; `problem()` says why it was not kept.
    pub fn record(&mut self, plant: Plant) -> Change {
        if self.read_only {
            return self.keep_unsaved(plant, None);
        }
        let folder = self.path.parent().unwrap_or(Path::new("."));
        if fs::create_dir_all(folder).is_err() {
            return self.keep_unsaved(plant, Some(Problem::NoFolder));
        }
        // Only one Grove saves at a time, and each starts from the file as it
        // is now, so neither drops the plants the other has saved since it
        // was read.
        let Some(_lock) = Lock::take(&self.lock_path(), LOCK_WAIT) else {
            return self.keep_unsaved(plant, Some(Problem::Busy));
        };
        let mut plants = match self.read() {
            Read::Missing => Vec::new(),
            Read::Ok(plants) => plants,
            Read::Unreadable | Read::Invalid => {
                // Changed into something that cannot be read since: leave it be.
                self.read_only = true;
                return self.keep_unsaved(plant, Some(Problem::CannotRead));
            }
        };
        plants.insert(0, plant);
        self.plants = plants;
        self.problem = self.save().err().map(|error| Problem::CannotSave(error.to_string()));
        Change::Reset
    }

    /// Shows a plant that could not be kept.
    fn keep_unsaved(&mut self, plant: Plant, problem: Option<Problem>) -> Change {
        self.plants.insert(0, plant);
        if problem.is_some() {
            self.problem = problem;
        }
        Change::Inserted
    }

    fn lock_path(&self) -> PathBuf {
        let mut name = self.path.clone().into_os_string();
        name.push(".lock");
        PathBuf::from(name)
    }

    /// Reads the file as it is now, newest plant first.
    fn read(&self) -> Read {
        let text = match fs::read(&self.path) {
            Ok(text) => text,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Read::Missing,
            Err(_) => return Read::Unreadable,
        };
        let Ok(document) = serde_json::from_slice::<Value>(&text) else {
            return Read::Invalid;
        };
        let Some(list) = document.get("plants").and_then(Value::as_array) else {
            return Read::Invalid;
        };
        // On disk the oldest plant comes first; the garden wants newest first.
        let plants: Option<Vec<Plant>> = list.iter().rev().map(plant_from).collect();
        plants.map_or(Read::Invalid, Read::Ok)
    }

    /// Writes the garden beside the file and renames it into place, so a
    /// crash mid-write never leaves half a garden.
    fn save(&self) -> io::Result<()> {
        // Oldest first on disk, so a new plant is a simple append in the file.
        let plants: Vec<Value> = self.plants.iter().rev().map(plant_to).collect();
        let text = serde_json::to_vec_pretty(&json!({ "version": 1, "plants": plants }))?;
        let mut beside = self.path.clone().into_os_string();
        beside.push(format!(".{}.tmp", std::process::id()));
        let beside = PathBuf::from(beside);
        let written = File::create(&beside)
            .and_then(|mut file| {
                file.write_all(&text)?;
                file.write_all(b"\n")?;
                file.sync_all()
            })
            .and_then(|()| fs::rename(&beside, &self.path));
        if written.is_err() {
            let _ = fs::remove_file(&beside);
        }
        written
    }
}

/// A whole number from JSON, as Qt read it: a number with no fraction.
fn whole(value: Option<&Value>) -> Option<i32> {
    let number = value?.as_f64()?;
    (number.fract() == 0.0 && number.abs() <= f64::from(i32::MAX)).then_some(number as i32)
}

/// A date as it is written in the file. Older files may carry fractions of a
/// second or a UTC offset, which Qt wrote for some dates.
fn when_from(text: &str) -> Option<NaiveDateTime> {
    NaiveDateTime::parse_from_str(text, WHEN_FORMAT)
        .or_else(|_| NaiveDateTime::parse_from_str(text, "%Y-%m-%dT%H:%M:%S%.f"))
        .ok()
        .or_else(|| DateTime::parse_from_rfc3339(text).ok().map(|when| when.naive_local()))
}

/// A plant from its entry in the file, or `None` if it is not one.
fn plant_from(entry: &Value) -> Option<Plant> {
    let text = |key: &str| entry.get(key).and_then(Value::as_str).unwrap_or_default();
    let plant = Plant {
        seed: whole(entry.get("seed"))?,
        outcome: Outcome::named(text("outcome"))?,
        season: text("season").to_string(),
        weather: text("weather").to_string(),
        progress: entry.get("progress")?.as_f64()?,
        when: when_from(text("when"))?,
        minutes: whole(entry.get("minutes"))?,
    };
    (plant.seed >= 0 && (0.0..=1.0).contains(&plant.progress) && plant.minutes >= 0).then_some(plant)
}

/// A plant's entry in the file.
fn plant_to(plant: &Plant) -> Value {
    let mut entry = Map::new();
    entry.insert("seed".into(), json!(plant.seed));
    entry.insert("outcome".into(), json!(plant.outcome.name()));
    entry.insert("season".into(), json!(plant.season));
    entry.insert("weather".into(), json!(plant.weather));
    entry.insert("progress".into(), json!(plant.progress));
    entry.insert("when".into(), json!(plant.when.format(WHEN_FORMAT).to_string()));
    entry.insert("minutes".into(), json!(plant.minutes));
    Value::Object(entry)
}

/// The right to save the garden, held through a lock file beside it. The file
/// is removed when the lock is let go.
struct Lock {
    file: File,
    path: PathBuf,
}

impl Lock {
    /// Takes the lock, waiting up to `wait` for whoever holds it.
    fn take(path: &Path, wait: Duration) -> Option<Self> {
        let deadline = Instant::now() + wait;
        loop {
            let file = File::options()
                .create(true)
                .truncate(false)
                .write(true)
                .open(path)
                .ok()?;
            if file.try_lock().is_ok() {
                // The holder before may have removed the file between our
                // opening and locking it; a lock on a file that is gone
                // guards nothing, so try again on the one that is there.
                let same = fs::metadata(path)
                    .and_then(
                        |there| Ok((there.dev(), there.ino()) == (file.metadata()?.dev(), file.metadata()?.ino())),
                    )
                    .unwrap_or(false);
                if same {
                    return Some(Self {
                        file,
                        path: path.to_path_buf(),
                    });
                }
            } else if Instant::now() >= deadline {
                return None;
            } else {
                std::thread::sleep(Duration::from_millis(50));
            }
        }
    }
}

impl Drop for Lock {
    fn drop(&mut self) {
        // Removed while still held, so no one can take a lock on it after.
        let _ = fs::remove_file(&self.path);
        let _ = self.file.unlock();
    }
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;
    use tempfile::TempDir;

    use super::*;

    fn plant(seed: i32, outcome: Outcome, progress: f64, minutes: i32) -> Plant {
        Plant {
            seed,
            outcome,
            season: "autumn".into(),
            weather: "fog".into(),
            progress,
            when: NaiveDate::from_ymd_opt(2026, 11, 6)
                .unwrap()
                .and_hms_opt(9, 30, 0)
                .unwrap(),
            minutes,
        }
    }

    fn seeds(garden: &Garden) -> Vec<i32> {
        garden.plants().iter().map(|plant| plant.seed).collect()
    }

    #[test]
    fn empty_garden_starts_empty() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("garden.json");
        let garden = Garden::open(&path);
        assert!(garden.plants().is_empty());
        assert_eq!(garden.grown(), 0);
        assert_eq!(garden.problem(), None);
        // Nothing is written until there is something to keep.
        assert!(!path.exists());
    }

    #[test]
    fn recorded_plants_survive_a_restart_newest_first() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("nested/garden.json");
        {
            let mut garden = Garden::open(&path);
            garden.record(plant(11, Outcome::Done, 1.0, 25));
            garden.record(plant(22, Outcome::Wilted, 0.4, 0));
            garden.record(plant(33, Outcome::Done, 1.0, 25));
            assert_eq!(garden.grown(), 2);
            assert_eq!(garden.wilted(), 1);
            assert_eq!(garden.focused_minutes(), 50);
            assert_eq!(seeds(&garden), [33, 22, 11]);
        }
        let again = Garden::open(&path);
        assert_eq!(seeds(&again), [33, 22, 11]);
        assert_eq!(again.plants()[1], plant(22, Outcome::Wilted, 0.4, 0));
        assert_eq!(again.focused_minutes(), 50);
        assert_eq!(again.problem(), None);
    }

    #[test]
    fn two_groves_open_together_keep_each_others_plants() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("garden.json");
        let (mut first, mut second) = (Garden::open(&path), Garden::open(&path));
        first.record(plant(11, Outcome::Done, 1.0, 25));
        // The second read the garden before the first saved its plant.
        assert_eq!(second.record(plant(22, Outcome::Wilted, 0.4, 0)), Change::Reset);
        assert_eq!(seeds(&second), [22, 11]);
        first.record(plant(33, Outcome::Done, 1.0, 25));
        let again = Garden::open(&path);
        assert_eq!(again.plants().len(), 3);
        assert_eq!(again.focused_minutes(), 50);
        assert!(!dir.path().join("garden.json.lock").exists());
        // Nothing else is left beside the garden either.
        assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 1);
    }

    #[test]
    fn a_garden_spoiled_while_open_is_not_saved_over() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("garden.json");
        let mut garden = Garden::open(&path);
        garden.record(plant(11, Outcome::Done, 1.0, 25));
        fs::write(&path, "this is not json").unwrap();
        assert_eq!(garden.record(plant(22, Outcome::Done, 1.0, 25)), Change::Inserted);
        assert_eq!(garden.problem(), Some(&Problem::CannotRead));
        assert_eq!(fs::read(&path).unwrap(), b"this is not json");
        // The plant is still shown, though it could not be kept.
        assert_eq!(seeds(&garden), [22, 11]);
    }

    #[test]
    fn unreadable_garden_is_left_untouched() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("garden.json");
        fs::write(&path, "this is not json").unwrap();
        let mut garden = Garden::open(&path);
        assert_eq!(garden.problem(), Some(&Problem::CannotRead));
        garden.record(plant(5, Outcome::Done, 1.0, 25));
        // The new plant shows in the window but must not replace the old file.
        assert_eq!(seeds(&garden), [5]);
        assert_eq!(fs::read(&path).unwrap(), b"this is not json");
    }

    #[test]
    fn garden_with_a_bad_entry_is_not_overwritten() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("garden.json");
        let bad = br#"{"version":1,"plants":[{"seed":1,"outcome":"sprouted"}]}"#;
        fs::write(&path, bad).unwrap();
        let mut garden = Garden::open(&path);
        assert_eq!(garden.problem(), Some(&Problem::CannotRead));
        garden.record(plant(5, Outcome::Done, 1.0, 25));
        assert_eq!(fs::read(&path).unwrap(), bad);
    }

    #[test]
    fn a_garden_the_cpp_grove_wrote_is_read() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("garden.json");
        // As QJsonDocument wrote it: keys in alphabetical order, whole
        // numbers for a finished plant's progress.
        let written = r#"{
    "plants": [
        {
            "minutes": 25,
            "outcome": "done",
            "progress": 1,
            "season": "autumn",
            "seed": 11,
            "weather": "fog",
            "when": "2026-11-06T09:30:00"
        }
    ],
    "version": 1
}
"#;
        fs::write(&path, written).unwrap();
        let garden = Garden::open(&path);
        assert_eq!(garden.problem(), None);
        assert_eq!(garden.plants(), [plant(11, Outcome::Done, 1.0, 25)]);
    }

    #[test]
    fn a_busy_garden_keeps_the_plant_unsaved() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("garden.json");
        let garden = Garden::open(&path);
        let _held = Lock::take(&garden.lock_path(), Duration::ZERO).unwrap();
        assert!(Lock::take(&garden.lock_path(), Duration::from_millis(120)).is_none());
    }

    #[test]
    fn the_garden_is_kept_in_the_data_directory() {
        let home = |name: &str| (name == "HOME").then(|| "/home/someone".to_string());
        assert_eq!(
            Garden::default_path(home),
            PathBuf::from("/home/someone/.local/share/grove/garden.json")
        );
        let xdg = |name: &str| (name == "XDG_DATA_HOME").then(|| "/data".to_string());
        assert_eq!(Garden::default_path(xdg), PathBuf::from("/data/grove/garden.json"));
    }
}
