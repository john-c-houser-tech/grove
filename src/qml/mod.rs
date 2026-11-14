//! The Grove QML module's types: thin QObjects over the rules in the files
//! above this directory, which know nothing of Qt.

pub mod backend;
mod color;
mod field;
pub mod garden;
mod hills;
mod layer;
pub mod layers;
pub mod light;
pub mod notifier;
pub mod paint;
pub mod script;
pub mod shared;
pub mod sounds;
pub mod testing;
pub mod theme;
mod trackbed;
pub mod translator;
pub mod weather;

/// Makes the app behind the singletons, and has Qt know the Grove module.
/// Called on the GUI thread, once the application exists and before a window
/// is loaded.
pub fn start(options: crate::options::Options) {
    cxx_qt::init_crate!(grove);
    cxx_qt::init_qml_module!("Grove");
    shared::start(options);
}

/// Loads the window into an engine and starts the services that live in its
/// singletons. Returns the window, which the engine owns, or `None` if it
/// could not be made. `start` comes first.
pub fn load_window(
    engine: core::pin::Pin<&mut cxx_qt_lib::QQmlApplicationEngine>,
) -> Option<core::ptr::NonNull<cxx_qt::QObject>> {
    use std::sync::atomic::{AtomicPtr, Ordering};
    use std::sync::Arc;

    let mut engine = engine;
    let made = Arc::new(AtomicPtr::new(std::ptr::null_mut()));
    let seen = Arc::clone(&made);
    engine
        .as_mut()
        .on_object_created(move |_, object, _| seen.store(object, Ordering::SeqCst))
        .release();
    engine
        .as_mut()
        .load(&cxx_qt_lib::QUrl::from("qrc:/qt/qml/Grove/src/Main.qml"));
    let window = core::ptr::NonNull::new(made.load(Ordering::SeqCst))?;
    // SAFETY: the engine just made the window, and keeps it.
    for service in unsafe { script::start_services(window.as_ptr()) } {
        eprintln!("grove: {service} could not be started");
    }
    Some(window)
}
