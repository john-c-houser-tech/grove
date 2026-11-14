//! Grove, a focus timer that grows a plant while you work: everything but
//! `main`, so the scene tests drive the very same thing.

// The bridges declare Qt's own functions, each with its safety note beside
// it; the bridge macro does not carry those notes onto what it generates.
#![expect(
    clippy::missing_safety_doc,
    reason = "the bridge macro drops the notes written beside each function"
)]
// Every unsafe block says why it is sound.
#![warn(clippy::undocumented_unsafe_blocks)]

pub mod app;
pub mod color;
pub mod easing;
pub mod follow;
pub mod garden;
pub mod language;
pub mod light;
pub mod notifier;
pub mod options;
pub mod player;
pub mod qml;
pub mod session;
pub mod settings;
pub mod sounds;
pub mod synth;
pub mod terrain;
pub mod theme;
pub mod weather;

use cxx_qt_lib::{QGuiApplication, QQmlApplicationEngine, QQuickStyle, QString};

/// Runs Grove as the environment sets it up. Returns the process's exit code.
pub fn run() -> i32 {
    // Pictures of the landscape are drawn on worker threads into buffers of a
    // few megabytes. glibc raises its threshold for giving such buffers their
    // own mapping after the first is freed, so later ones come from the heap
    // and, once freed, stay with the process. Fixing the threshold returns
    // each one to the system when it goes.
    #[cfg(target_env = "gnu")]
    // SAFETY: called before any other thread exists, with a valid parameter.
    unsafe {
        libc::mallopt(libc::M_MMAP_THRESHOLD, 128 * 1024);
    }

    let mut application = QGuiApplication::new();
    if let Some(application) = application.as_mut() {
        application.set_application_name(&QString::from("grove"));
    }
    if let Some(application) = application.as_mut() {
        application.set_application_version(&QString::from(env!("CARGO_PKG_VERSION")));
    }
    // Matches the window to grove.desktop, so the compositor (Wayland app_id)
    // and the launcher pick up the installed icon.
    QGuiApplication::set_desktop_file_name(&QString::from("grove"));
    qml::start(options::Options::from_environment());
    // The controls' style is the process's, and is fixed before the first
    // window is loaded.
    QQuickStyle::set_style(&QString::from("Material"));

    // The engine is made after the application and dropped before it, so
    // nothing in the window outlives what it shows.
    let mut engine = QQmlApplicationEngine::new();
    let Some(_window) = engine.as_mut().and_then(qml::load_window) else {
        return 1;
    };
    let code = application.as_mut().map_or(1, |application| application.exec());
    drop(engine);
    code
}

#[cfg(test)]
mod tests {
    /// Not a test of Grove: a leak planted for bin/check-leaks --canary,
    /// which passes only if the leak check catches it, so a clean report
    /// means something.
    #[test]
    fn leak_canary() {
        if std::env::var_os("GROVE_LEAK_CANARY").is_none() {
            return;
        }
        let lost = Box::new([b'x'; 1024]);
        assert_eq!(lost.len(), 1024);
        // The only pointer to it is dropped without freeing it.
        std::mem::forget(lost);
    }
}
