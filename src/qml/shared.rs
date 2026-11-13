//! The one `App` behind the QML singletons, which the engine makes one by one
//! as the window first uses them. It lives on the GUI thread; each singleton
//! reads what it shows from here and asks to hear about what changes.

use std::cell::RefCell;
use std::rc::Rc;

use crate::app::{App, Event};
use crate::options;

type Listener = Rc<dyn Fn(Event)>;

thread_local! {
    static APP: RefCell<Option<App>> = const { RefCell::new(None) };
    static LISTENERS: RefCell<Vec<Listener>> = const { RefCell::new(Vec::new()) };
}

/// Reads from the app, making it from the installed options on first use.
pub fn read<R>(read: impl FnOnce(&App) -> R) -> R {
    APP.with_borrow_mut(|app| read(app.get_or_insert_with(|| App::new(options::installed()))))
}

/// Changes the app, then tells every listener what the change brought about.
/// The listeners run once the app is free again, so they may read from it.
pub fn update(change: impl FnOnce(&mut App) -> Vec<Event>) {
    let events = APP.with_borrow_mut(|app| change(app.get_or_insert_with(|| App::new(options::installed()))));
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
