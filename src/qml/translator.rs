//! The language, as the Grove QML module's `Translator` singleton, followed
//! live. It is the translator Qt asks for every qsTr() in QML, answered from
//! the catalogs in `../language.rs`.

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++Qt" {
        include!(<QtCore/QTranslator>);
        #[qobject]
        type QTranslator;
    }

    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
        include!("cxx-qt-lib/qt.h");
        #[namespace = "Qt"]
        type LayoutDirection = cxx_qt_lib::LayoutDirection;
        include!(<QtQml/QQmlEngine>);
        type QQmlEngine;

        // Qt's own functions, called as they are.
        include!(<QtCore/QCoreApplication>);
        type QCoreApplication;
        /// Has the application ask `translator` for its texts.
        ///
        /// # Safety
        /// `translator` must outlive the application or remove itself, as a
        /// QTranslator does when it is destroyed.
        #[Self = "QCoreApplication"]
        #[cxx_name = "installTranslator"]
        unsafe fn install_translator(translator: *mut QTranslator) -> bool;

        include!(<QtGui/QGuiApplication>);
        type QGuiApplication;
        /// Lays windows out in the language's direction.
        #[Self = "QGuiApplication"]
        #[cxx_name = "setLayoutDirection"]
        fn set_layout_direction(direction: LayoutDirection);

        /// The engine that made a QML object; null if none did.
        ///
        /// # Safety
        /// `object` must be a valid QObject.
        #[cxx_name = "qmlEngine"]
        unsafe fn qml_engine(object: *const QObject) -> *mut QQmlEngine;
        /// Has the engine translate every text in its QML again.
        fn retranslate(self: Pin<&mut QQmlEngine>);
    }

    extern "RustQt" {
        #[qobject]
        #[base = QTranslator]
        #[qml_element]
        #[qml_singleton]
        #[qproperty(QString, language, READ = language, NOTIFY = changed)]
        #[qproperty(QString, locale, READ = locale, NOTIFY = changed)]
        #[qproperty(bool, right_to_left, cxx_name = "rightToLeft", READ = right_to_left, NOTIFY = changed)]
        type Translator = super::TranslatorRust;

        /// The Omarchy language code ("en", "pt-BR").
        fn language(self: &Translator) -> QString;
        /// The name of the locale Omarchy formats the language with, for
        /// Qt.locale(), so numbers and dates match the desktop's.
        fn locale(self: &Translator) -> QString;
        /// Whether the language reads right to left, so the window mirrors.
        #[cxx_name = "rightToLeft"]
        fn right_to_left(self: &Translator) -> bool;

        /// The language changed; QML is translated again.
        #[qsignal]
        fn changed(self: Pin<&mut Translator>);

        /// Qt's question: the translation of `source_text`, for a count `n`
        /// (negative when the string is not a counted one). A null string
        /// tells Qt there is none, so it shows the English.
        ///
        /// # Safety
        /// `source_text` must be null or a NUL-terminated string, as Qt
        /// passes it.
        #[cxx_override]
        unsafe fn translate(
            self: &Translator,
            context: *const c_char,
            source_text: *const c_char,
            disambiguation: *const c_char,
            n: i32,
        ) -> QString;
        /// Never empty: Qt skips a translator that says it is.
        #[cxx_override]
        #[cxx_name = "isEmpty"]
        fn is_empty(self: &Translator) -> bool;
    }

    impl cxx_qt::Threading for Translator {}
    impl cxx_qt::Initialize for Translator {}
}

use core::ffi::{c_char, CStr};
use core::pin::Pin;

use cxx_qt::casting::Upcast;
use cxx_qt::{CxxQtType, Threading};
use cxx_qt_lib::{LayoutDirection, QString};

use super::shared;
use crate::app::Event;
use crate::follow::{watch, Watching};
use crate::language::{paths_to_watch, right_to_left};

/// What the `Translator` QObject keeps: the thread watching the language
/// file, which stops when it goes.
#[derive(Default)]
pub struct TranslatorRust {
    watching: Option<Watching>,
}

impl cxx_qt::Initialize for qobject::Translator {
    fn initialize(mut self: Pin<&mut Self>) {
        shared::started("Translator");
        let translator: Pin<&mut qobject::QTranslator> = self.as_mut().upcast_pin();
        // SAFETY: a QTranslator removes itself from the application when it
        // is destroyed, so the application never asks one that is gone.
        unsafe { qobject::QCoreApplication::install_translator(translator.get_unchecked_mut()) };
        self.as_ref().lay_out();

        let gui = self.qt_thread();
        shared::listen(move |event| {
            if event == Event::Language {
                let _ = gui.queue(|translator| translator.speak_anew());
            }
        });
        // Texts read before the window first asked for this singleton were
        // not translated; they are, once the engine has finished making it.
        let _ = self.qt_thread().queue(|translator| translator.as_ref().retranslate());

        if let Some(file) = shared::read(|app| app.options().language_file.clone()) {
            let gui = self.qt_thread();
            let watching = watch(
                move || paths_to_watch(&file),
                move || gui.queue(|_| shared::update(|app| app.reload_language())).is_ok(),
            );
            self.as_mut().rust_mut().watching = Some(watching);
        }
    }
}

impl qobject::Translator {
    pub fn language(&self) -> QString {
        QString::from(shared::read(|app| app.catalog().code().to_string()).as_str())
    }

    pub fn locale(&self) -> QString {
        QString::from(shared::read(|app| app.format_locale().to_string()).as_str())
    }

    pub fn right_to_left(&self) -> bool {
        shared::read(|app| right_to_left(app.catalog().code()))
    }

    /// # Safety
    /// `source_text` must be null or a NUL-terminated string.
    pub unsafe fn translate(
        &self,
        _context: *const c_char,
        source_text: *const c_char,
        _disambiguation: *const c_char,
        n: i32,
    ) -> QString {
        if source_text.is_null() {
            return QString::default();
        }
        // SAFETY: not null, and NUL-terminated by the caller's promise.
        let source = unsafe { CStr::from_ptr(source_text) };
        let count = (n >= 0).then_some(i64::from(n));
        source
            .to_str()
            .ok()
            .and_then(|source| shared::translation(source, count))
            .map_or_else(QString::default, |text| QString::from(text.as_str()))
    }

    pub fn is_empty(&self) -> bool {
        false
    }

    /// Lays windows out in the language's direction.
    fn lay_out(&self) {
        qobject::QGuiApplication::set_layout_direction(if self.right_to_left() {
            LayoutDirection::RightToLeft
        } else {
            LayoutDirection::LeftToRight
        });
    }

    /// Has the engine that made this singleton translate its QML again.
    fn retranslate(&self) {
        let translator: &qobject::QTranslator = self.upcast();
        let object: &qobject::QObject = translator.upcast();
        // SAFETY: `object` is this live QObject, and the engine that made it
        // outlives it.
        if let Some(engine) = unsafe { qobject::qml_engine(object).as_mut() } {
            // SAFETY: Qt's objects are never moved.
            unsafe { Pin::new_unchecked(engine) }.retranslate();
        }
    }

    /// Takes up a new language: its direction, and every text in the window
    /// said again.
    fn speak_anew(self: Pin<&mut Self>) {
        self.as_ref().lay_out();
        self.as_ref().retranslate();
        self.changed();
    }
}
