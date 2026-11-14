//! The one `App` behind the QML singletons, which the engine makes one by one
//! as the window first uses them. It lives on the GUI thread; each singleton
//! reads what it shows from here and asks to hear about what changes.

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, RwLock};

use crate::app::{App, Event};
use crate::language::Catalog;
use crate::options;

type Listener = Rc<dyn Fn(Event)>;

thread_local! {
    static APP: RefCell<Option<App>> = const { RefCell::new(None) };
    static LISTENERS: RefCell<Vec<Listener>> = const { RefCell::new(Vec::new()) };
}

/// The current language's strings, where any thread can reach them: Qt may
/// ask for a translation from wherever it likes.
static CATALOG: RwLock<Option<Arc<Catalog>>> = RwLock::new(None);

/// The app in `slot`, made from the installed options on first use.
fn app(slot: &mut Option<App>) -> &mut App {
    slot.get_or_insert_with(|| {
        let app = App::new(options::installed());
        publish(&app);
        app
    })
}

/// Puts the app's catalog where translations are looked up.
fn publish(app: &App) {
    *CATALOG.write().unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(Arc::clone(app.catalog()));
}

/// Reads from the app, making it from the installed options on first use.
pub fn read<R>(read: impl FnOnce(&App) -> R) -> R {
    APP.with_borrow_mut(|slot| read(app(slot)))
}

/// The translation of an English source text in the current language, for a
/// count when the string is a counted one. `None` when there is none.
pub fn translation(source: &str, count: Option<i64>) -> Option<String> {
    let catalog = CATALOG.read().unwrap_or_else(|poisoned| poisoned.into_inner());
    catalog.as_ref()?.translate(source, count).map(str::to_string)
}

/// An English text in the current language. Every string Rust shows the user
/// goes through here as a literal, `tr("...")`, which is how bin/check-i18n
/// finds it.
pub fn tr(source: &str) -> String {
    translation(source, None).unwrap_or_else(|| source.to_string())
}

/// Changes the app, then tells every listener what the change brought about.
/// The listeners run once the app is free again, so they may read from it.
pub fn update(change: impl FnOnce(&mut App) -> Vec<Event>) {
    let events = APP.with_borrow_mut(|slot| {
        let app = app(slot);
        let events = change(app);
        if events.contains(&Event::Language) {
            publish(app);
        }
        events
    });
    if events.is_empty() {
        return;
    }
    let listeners: Vec<Listener> = LISTENERS.with_borrow(|listeners| listeners.clone());
    for event in events {
        for listener in &listeners {
            listener(event);
        }
    }
}

/// Calls `listener` on the GUI thread with every event from now on.
pub fn listen(listener: impl Fn(Event) + 'static) {
    LISTENERS.with_borrow_mut(|listeners| listeners.push(Rc::new(listener)));
}
