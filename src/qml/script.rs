//! JavaScript run as part of the window's QML, where its ids, properties and
//! the Grove module's singletons are in reach.

#[cxx_qt::bridge]
pub mod ffi {
    unsafe extern "C++" {
        include!("cxx-qt-lib/common.h");
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
        include!("cxx-qt-lib/qvariant.h");
        type QVariant = cxx_qt_lib::QVariant;
        include!(<QtCore/QObject>);
        type QObject = cxx_qt::QObject;

        include!(<QtQml/QQmlExpression>);
        include!(<QtQml/QQmlContext>);
        include!(<QtQml/QQmlEngine>);
        /// Where a QML object's ids and properties are looked up.
        type QQmlContext;
        /// The context a QML object was made in; null if none.
        ///
        /// # Safety
        /// `object` must be a live QObject.
        #[cxx_name = "qmlContext"]
        unsafe fn qml_context(object: *const QObject) -> *mut QQmlContext;
        /// A piece of JavaScript, to run as part of the QML.
        type QQmlExpression;
        /// # Safety
        /// `context` and `scope` must outlive the expression.
        #[namespace = "rust::cxxqtlib1"]
        #[cxx_name = "make_unique"]
        unsafe fn new_expression(
            context: *mut QQmlContext,
            scope: *mut QObject,
            expression: &QString,
        ) -> UniquePtr<QQmlExpression>;
        /// Runs the expression and returns its value.
        ///
        /// # Safety
        /// `undefined` must be null or point at a bool.
        unsafe fn evaluate(self: Pin<&mut QQmlExpression>, undefined: *mut bool) -> QVariant;
        #[cxx_name = "hasError"]
        fn has_error(self: &QQmlExpression) -> bool;
    }
}

use cxx_qt_lib::{QString, QVariant};

use super::shared;

/// Runs a piece of JavaScript as part of the QML that made `object`, and
/// returns its value; `None` if it failed, or the object is not QML's.
///
/// # Safety
/// `object` must be a live QObject.
pub unsafe fn evaluate(object: *mut cxx_qt::QObject, script: &str) -> Option<QVariant> {
    // SAFETY: the object is live by the caller's promise, its context lives
    // as long as it does, and the expression is gone before this returns.
    unsafe {
        let context = ffi::qml_context(object);
        if context.is_null() {
            return None;
        }
        let mut expression = ffi::new_expression(context, object, &QString::from(script));
        let value = expression.pin_mut().evaluate(std::ptr::null_mut());
        (!expression.has_error()).then_some(value)
    }
}

/// Starts the services that live in QML singletons, whatever the window's
/// QML happens to mention: naming a singleton has the engine make it. A
/// window that never reads `Notifier.problem` still gets its notification.
/// Returns the services that could not be started, which is none unless the
/// window does without the Grove module.
///
/// # Safety
/// `window` must be the live root object of a loaded window.
pub unsafe fn start_services(window: *mut cxx_qt::QObject) -> Vec<&'static str> {
    let named = shared::SERVICES.join(", ");
    // SAFETY: passed on as received.
    let _ = unsafe { evaluate(window, &format!("[{named}].length")) };
    shared::missing()
}
