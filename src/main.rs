//! grove — a focus timer that grows a plant while you work.

mod app;
mod color;
mod easing;
mod follow;
mod garden;
mod language;
mod light;
mod notifier;
mod options;
mod player;
mod qml;
mod session;
mod settings;
mod sounds;
mod synth;
mod terrain;
mod theme;
mod weather;

use std::sync::atomic::{AtomicBool, Ordering};

use cxx_qt_lib::{QGuiApplication, QQmlApplicationEngine, QQuickStyle, QString, QUrl};

/// Whether the window failed to load.
static FAILED: AtomicBool = AtomicBool::new(false);

fn main() {
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

    // Everything is made from these; nothing else reads the environment.
    options::install(options::Options::from_environment());

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
    // The controls' style is the process's, and is fixed before the first
    // window is loaded.
    QQuickStyle::set_style(&QString::from("Material"));

    // The engine is made after the application and dropped before it, so
    // nothing in the window outlives what it shows.
    let mut engine = QQmlApplicationEngine::new();
    if let Some(engine) = engine.as_mut() {
        engine
            .on_object_creation_failed(|_, _| FAILED.store(true, Ordering::SeqCst))
            .release();
    }
    if let Some(engine) = engine.as_mut() {
        engine.load(&QUrl::from("qrc:/qt/qml/Grove/src/Main.qml"));
    }
    if FAILED.load(Ordering::SeqCst) {
        std::process::exit(1);
    }
    let code = application.as_mut().map_or(1, |application| application.exec());
    drop(engine);
    drop(application);
    std::process::exit(code);
}
