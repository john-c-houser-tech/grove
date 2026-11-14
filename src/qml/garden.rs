//! The garden, as the Grove QML module's `Garden` singleton: a list model of
//! every plant, newest first. The rules are in `../garden.rs`.

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++Qt" {
        include!(<QtCore/QAbstractListModel>);
        #[qobject]
        type QAbstractListModel;
    }

    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
        include!("cxx-qt-lib/qhash.h");
        type QHash_i32_QByteArray = cxx_qt_lib::QHash<cxx_qt_lib::QHashPair_i32_QByteArray>;
        include!("cxx-qt-lib/qvariant.h");
        type QVariant = cxx_qt_lib::QVariant;
        include!("cxx-qt-lib/qmodelindex.h");
        type QModelIndex = cxx_qt_lib::QModelIndex;
    }

    extern "RustQt" {
        #[qobject]
        #[base = QAbstractListModel]
        #[qml_element]
        #[qml_singleton]
        #[qproperty(i32, grown, READ = grown, NOTIFY = counts_changed)]
        #[qproperty(i32, wilted, READ = wilted, NOTIFY = counts_changed)]
        #[qproperty(i32, focused_minutes, cxx_name = "focusedMinutes", READ = focused_minutes, NOTIFY = counts_changed)]
        #[qproperty(QString, problem, READ = problem, NOTIFY = problem_changed)]
        type Garden = super::GardenRust;

        /// How many plants grew to the end, and how many wilted.
        fn grown(self: &Garden) -> i32;
        fn wilted(self: &Garden) -> i32;
        /// Time spent on plants that grew to the end.
        #[cxx_name = "focusedMinutes"]
        fn focused_minutes(self: &Garden) -> i32;
        /// Empty when the garden is fine, otherwise why it cannot be read or
        /// saved.
        fn problem(self: &Garden) -> QString;

        /// grown, wilted or focusedMinutes changed.
        #[qsignal]
        #[cxx_name = "countsChanged"]
        fn counts_changed(self: Pin<&mut Garden>);
        /// The problem changed.
        #[qsignal]
        #[cxx_name = "problemChanged"]
        fn problem_changed(self: Pin<&mut Garden>);

        #[cxx_override]
        fn data(self: &Garden, index: &QModelIndex, role: i32) -> QVariant;
        #[cxx_override]
        #[cxx_name = "roleNames"]
        fn role_names(self: &Garden) -> QHash_i32_QByteArray;
        #[cxx_override]
        #[cxx_name = "rowCount"]
        fn row_count(self: &Garden, parent: &QModelIndex) -> i32;
    }

    // The model's own announcements, each pair made around one change to the
    // rows this object shows.
    extern "RustQt" {
        #[inherit]
        #[cxx_name = "beginInsertRows"]
        unsafe fn begin_insert_rows(self: Pin<&mut Garden>, parent: &QModelIndex, first: i32, last: i32);
        #[inherit]
        #[cxx_name = "endInsertRows"]
        unsafe fn end_insert_rows(self: Pin<&mut Garden>);
        #[inherit]
        #[cxx_name = "beginResetModel"]
        unsafe fn begin_reset_model(self: Pin<&mut Garden>);
        #[inherit]
        #[cxx_name = "endResetModel"]
        unsafe fn end_reset_model(self: Pin<&mut Garden>);
    }

    impl cxx_qt::Threading for Garden {}
    impl cxx_qt::Initialize for Garden {}
}

use core::pin::Pin;

use chrono::{Local, TimeZone};
use cxx_qt::{CxxQtType, Threading};
use cxx_qt_lib::{QByteArray, QDateTime, QHash, QHashPair_i32_QByteArray, QModelIndex, QString, QVariant};

use super::shared::{self, tr};
use crate::app::Event;
use crate::garden::{Change, Plant, Problem};

/// What the model calls each part of a plant; Qt's own roles end at 0x100.
const ROLES: [&str; 7] = ["seed", "outcome", "season", "weather", "progress", "when", "minutes"];
const FIRST_ROLE: i32 = 0x101;

/// The rows the model is showing. They are the app's plants as of the last
/// change the model announced, so its rows only ever change between a
/// `begin` and its `end`, as views require.
#[derive(Default)]
pub struct GardenRust {
    plants: Vec<Plant>,
}

impl cxx_qt::Initialize for qobject::Garden {
    fn initialize(mut self: Pin<&mut Self>) {
        self.as_mut().rust_mut().plants = shared::read(|app| app.garden().plants().to_vec());
        let gui = self.qt_thread();
        shared::listen(move |event| {
            let _ = match event {
                Event::Garden(change) => gui.queue(move |garden| garden.show(change)),
                // A new language puts the problem into other words.
                Event::GardenProblem | Event::Language => gui.queue(|garden| garden.problem_changed()),
                _ => Ok(()),
            };
        });
    }
}

impl qobject::Garden {
    pub fn grown(&self) -> i32 {
        shared::read(|app| app.garden().grown())
    }

    pub fn wilted(&self) -> i32 {
        shared::read(|app| app.garden().wilted())
    }

    pub fn focused_minutes(&self) -> i32 {
        shared::read(|app| app.garden().focused_minutes())
    }

    pub fn problem(&self) -> QString {
        QString::from(
            shared::read(|app| app.garden().problem().map(words))
                .unwrap_or_default()
                .as_str(),
        )
    }

    pub fn row_count(&self, parent: &QModelIndex) -> i32 {
        if parent.is_valid() {
            0
        } else {
            self.rust().plants.len() as i32
        }
    }

    pub fn role_names(&self) -> QHash<QHashPair_i32_QByteArray> {
        let mut names = QHash::<QHashPair_i32_QByteArray>::default();
        for (role, name) in (FIRST_ROLE..).zip(ROLES) {
            names.insert(role, QByteArray::from(name));
        }
        names
    }

    pub fn data(&self, index: &QModelIndex, role: i32) -> QVariant {
        let row = usize::try_from(index.row()).ok().filter(|_| index.is_valid());
        let Some(plant) = row.and_then(|row| self.rust().plants.get(row)) else {
            return QVariant::default();
        };
        let name = usize::try_from(role - FIRST_ROLE).ok().and_then(|role| ROLES.get(role));
        match name.copied() {
            Some("seed") => QVariant::from(&plant.seed),
            Some("outcome") => QVariant::from(&QString::from(plant.outcome.name())),
            Some("season") => QVariant::from(&QString::from(plant.season.as_str())),
            Some("weather") => QVariant::from(&QString::from(plant.weather.as_str())),
            Some("progress") => QVariant::from(&plant.progress),
            Some("when") => when(plant).map(|when| QVariant::from(&when)).unwrap_or_default(),
            Some("minutes") => QVariant::from(&plant.minutes),
            _ => QVariant::default(),
        }
    }

    /// Brings the rows up to date with the app's plants, announcing the
    /// change as the model must.
    fn show(mut self: Pin<&mut Self>, change: Change) {
        let plants = shared::read(|app| app.garden().plants().to_vec());
        // SAFETY: each begin is matched by its end, with the rows changed
        // between them exactly as announced.
        unsafe {
            match change {
                Change::Inserted if plants.len() == self.rust().plants.len() + 1 => {
                    self.as_mut().begin_insert_rows(&QModelIndex::default(), 0, 0);
                    self.as_mut().rust_mut().plants = plants;
                    self.as_mut().end_insert_rows();
                }
                // Also when several changes were queued before the first was
                // shown, and the rows are more than one plant behind.
                _ => {
                    self.as_mut().begin_reset_model();
                    self.as_mut().rust_mut().plants = plants;
                    self.as_mut().end_reset_model();
                }
            }
        }
        self.counts_changed();
    }
}

/// When a plant's session ended, as QML takes dates.
fn when(plant: &Plant) -> Option<QDateTime> {
    // A time the clocks skipped when they went forward is shown as the hour
    // after, which is what the clock read a moment later.
    let local = Local.from_local_datetime(&plant.when).earliest().or_else(|| {
        Local
            .from_local_datetime(&(plant.when + chrono::TimeDelta::hours(1)))
            .earliest()
    })?;
    QDateTime::try_from(local).ok()
}

/// A problem, in words for the window, in the language of the moment.
fn words(problem: &Problem) -> String {
    match problem {
        Problem::CannotOpen => tr("Could not open the garden file, so new plants will not be saved."),
        Problem::CannotRead => {
            tr("The garden file could not be read. It was left untouched, and new plants will not be saved.")
        }
        Problem::NoFolder => tr("Could not create the garden folder, so this plant was not saved."),
        Problem::Busy => tr("Another Grove kept the garden busy, so this plant was not saved."),
        Problem::CannotSave(why) => tr("Could not save the garden: %1").replace("%1", why),
    }
}
