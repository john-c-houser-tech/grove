//! What the scene tests drive the window with: keys as the window system
//! sends them, the window's picture, and a look at the QML. The app itself
//! uses none of it.

#[cxx_qt::bridge]
pub mod ffi {
    unsafe extern "C++" {
        include!("cxx-qt-lib/common.h");
        include!("cxx-qt-lib/qimage.h");
        type QImage = cxx_qt_lib::QImage;
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
        include!("cxx-qt-lib/qvariant.h");
        type QVariant = cxx_qt_lib::QVariant;
        include!("cxx-qt-lib/qt.h");
        #[namespace = "Qt"]
        type KeyboardModifiers = cxx_qt_lib::KeyboardModifiers;

        include!(<QtCore/QObject>);
        type QObject = cxx_qt::QObject;

        include!(<QtQuick/QQuickWindow>);
        /// The window, as the window system knows it.
        type QWindow;
        /// The window, as a scene.
        #[cxx_name = "QQuickWindow"]
        type Scenery;
        /// The window's picture as it is drawn now.
        #[cxx_name = "grabWindow"]
        fn q_grab_scenery(self: Pin<&mut Scenery>) -> QImage;

        include!(<QtGui/qtestsupport_gui.h>);
        /// Waits until the window is shown. False if it is not in time.
        ///
        /// # Safety
        /// `window` must be a live window.
        #[namespace = "QTest"]
        #[cxx_name = "qWaitForWindowExposed"]
        unsafe fn wait_for_window_exposed(window: *mut QWindow, timeout: i32) -> bool;

        include!(<QtCore/qtestsupport_core.h>);
        /// Lets the application run for that many milliseconds.
        #[namespace = "QTest"]
        #[cxx_name = "qWait"]
        fn wait(milliseconds: i32);

        include!(<QtTest/qtestkeyboard.h>);
        #[namespace = "QEvent"]
        type Type = crate::qml::testing::EventType;
        /// Hands the window a key event the way the window system does, so
        /// shortcuts see it.
        ///
        /// # Safety
        /// `window` must be a live window.
        #[cxx_name = "qt_handleKeyEvent"]
        unsafe fn handle_key_event(
            window: *mut QWindow,
            kind: Type,
            key: i32,
            modifiers: KeyboardModifiers,
            text: &QString,
            repeat: bool,
            count: u16,
        );
    }
}

use core::pin::Pin;
use std::path::Path;

use cxx::UniquePtr;
use cxx_qt_lib::{KeyboardModifiers, QImage, QQmlApplicationEngine, QString, QVariant};

use super::{paint, script};
use crate::options::Options;

/// The kind of an event, which Qt declares inside its event class.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EventType(i32);

impl EventType {
    const KEY_PRESS: Self = Self(6);
    const KEY_RELEASE: Self = Self(7);
}

// SAFETY: QEvent::Type is an enumeration the size of an int, passed by value
// as one.
unsafe impl cxx::ExternType for EventType {
    type Id = cxx::type_id!("QEvent::Type");
    type Kind = cxx::kind::Trivial;
}

/// A key a test presses, by Qt's number for it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Key {
    Space = 0x20,
    G = 0x47,
    M = 0x4d,
    Escape = 0x0100_0000,
    Return = 0x0100_0004,
    Right = 0x0100_0014,
}

impl Key {
    /// What typing the key puts in a text field, if anything.
    fn text(self) -> &'static str {
        match self {
            Key::Space => " ",
            Key::G => "g",
            Key::M => "m",
            Key::Escape | Key::Return | Key::Right => "",
        }
    }
}

/// Lets the application run for that many milliseconds.
pub fn wait(milliseconds: i32) {
    ffi::wait(milliseconds);
}

/// Lets the application run until `done` says so, for up to `milliseconds`.
/// Returns whether it did in time.
pub fn wait_until(milliseconds: i32, mut done: impl FnMut() -> bool) -> bool {
    let mut waited = 0;
    while !done() {
        if waited >= milliseconds {
            return false;
        }
        wait(10);
        waited += 10;
    }
    true
}

/// A picture of the window.
pub struct Frame(QImage);

impl Frame {
    /// The frame as the painting code knows images, which is the same Qt
    /// class under another of this crate's names.
    fn picture(&self) -> &paint::ffi::Picture {
        let image: *const QImage = &self.0;
        // SAFETY: both types stand for a QImage, and this one is borrowed
        // for as long as the result is.
        unsafe { &*image.cast::<paint::ffi::Picture>() }
    }

    pub fn width(&self) -> i32 {
        self.picture().q_width()
    }

    pub fn height(&self) -> i32 {
        self.picture().q_height()
    }

    /// One pixel as ARGB.
    pub fn pixel(&self, x: i32, y: i32) -> u32 {
        self.picture().q_pixel(x, y)
    }

    /// Writes the frame to a file, in the format its name ends with.
    /// Returns whether it was written.
    pub fn save(&self, file: &Path) -> bool {
        // SAFETY: a null format has Qt go by the file's name.
        unsafe {
            self.picture()
                .q_save(&QString::from(file.to_string_lossy().as_ref()), std::ptr::null(), -1)
        }
    }

    /// How many different colours there are in a band of the frame, given
    /// as fractions of its height.
    pub fn tones_in(&self, from: f64, to: f64) -> usize {
        let height = f64::from(self.height());
        let mut tones = std::collections::HashSet::new();
        for y in ((height * from) as i32..(height * to) as i32).step_by(2) {
            for x in (0..self.width()).step_by(2) {
                tones.insert(self.pixel(x, y));
            }
        }
        tones.len()
    }
}

/// The whole app with its real window, for a test to drive. The application
/// must exist, and only one scene at a time.
pub struct Scene {
    engine: UniquePtr<QQmlApplicationEngine>,
    /// The window, which the engine owns.
    window: *mut cxx_qt::QObject,
}

impl Scene {
    /// Makes the app from `options` and loads its window, muted or not.
    /// `None` if the window could not be made or shown.
    pub fn open(options: Options, muted: bool) -> Option<Self> {
        super::start(options);
        super::shared::update(|app| app.set_muted(muted));
        let mut engine = QQmlApplicationEngine::new();
        let window = super::load_window(engine.as_mut()?)?.as_ptr();
        let scene = Self { engine, window };
        // SAFETY: the window lives as long as the engine.
        unsafe { ffi::wait_for_window_exposed(scene.window.cast(), 5000) }.then_some(scene)
    }

    /// Presses and lets go of a key, as a user would.
    pub fn press(&self, key: Key) {
        let text = QString::from(key.text());
        for kind in [EventType::KEY_PRESS, EventType::KEY_RELEASE] {
            // SAFETY: the window lives as long as the engine this owns.
            unsafe {
                ffi::handle_key_event(
                    self.window.cast(),
                    kind,
                    key as i32,
                    KeyboardModifiers::default(),
                    &text,
                    false,
                    1,
                );
            }
            wait(0);
        }
    }

    /// The window's picture as it is drawn now.
    pub fn grab(&self) -> Frame {
        // SAFETY: the window is a QQuickWindow, alive and never moved.
        let window = unsafe { Pin::new_unchecked(&mut *self.window.cast::<ffi::Scenery>()) };
        Frame(window.q_grab_scenery())
    }

    /// Runs a piece of JavaScript in the window, where its ids and
    /// properties are in reach, and returns its value. `find(name)` in it is
    /// the item of that object name, or null.
    pub fn evaluate(&self, script: &str) -> QVariant {
        // Items are found by their object names, among the window's items.
        let script = format!(
            "(function() {{
                function within(item, name) {{
                    if (!item) return null
                    if (item.objectName === name) return item
                    const children = item.children || []
                    for (let i = 0; i < children.length; ++i) {{
                        const found = within(children[i], name)
                        if (found) return found
                    }}
                    return null
                }}
                function find(name) {{ return within(win.contentItem, name) }}
                return ({script})
            }})()"
        );
        // SAFETY: the window lives as long as the engine this owns.
        unsafe { script::evaluate(self.window, &script) }.unwrap_or_else(|| panic!("the script failed: {script}"))
    }

    /// Whether a piece of JavaScript in the window is true.
    pub fn is(&self, script: &str) -> bool {
        self.evaluate(&format!("!!({script})")).value::<bool>().unwrap_or(false)
    }

    /// The whole number a piece of JavaScript in the window comes to.
    pub fn number(&self, script: &str) -> i32 {
        self.evaluate(script).value::<i32>().unwrap_or(i32::MIN)
    }

    /// The text a piece of JavaScript in the window comes to.
    pub fn text(&self, script: &str) -> String {
        self.evaluate(&format!("String({script})"))
            .value::<QString>()
            .map(|text| text.to_string())
            .unwrap_or_default()
    }
}

impl Drop for Scene {
    fn drop(&mut self) {
        // The window goes with its engine; what it had queued for deletion
        // is given a moment to go too.
        self.engine = UniquePtr::null();
        wait(50);
    }
}
