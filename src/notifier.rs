//! Tells the desktop when a plant is fully grown, through the standard
//! notification service, so it shows up even when the window is buried.
//! Do-not-disturb in the shell silences it like any other notification.

use std::collections::HashMap;

use zbus::{Connection, Variant};

/// The desktop's notification service; only tests name another.
pub const SERVICE: &str = "org.freedesktop.Notifications";

/// Why a notification could not be sent, for the window to put into words.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Problem {
    /// There is no desktop session bus to send it on.
    NoBus,
    /// Nothing on the bus takes notifications.
    NoService,
}

/// Sends a notification to `service` and waits for its answer, so it is
/// called from a thread of its own: a slow or missing service never holds up
/// the window. `replaces` is the id of the last notification sent, or 0, so
/// a day of sessions never stacks up. Returns the new notification's id.
pub fn send(service: &str, replaces: u32, summary: &str, body: &str) -> Result<u32, Problem> {
    zbus::block_on(async {
        let bus = Connection::session().await.map_err(|_| Problem::NoBus)?;
        let actions: [&str; 0] = [];
        let hints = HashMap::from([("desktop-entry", Variant::from("grove"))]);
        // No timeout of our own: the desktop decides how long it shows.
        let expires = -1_i32;
        let reply = bus
            .call_method(
                Some(service),
                "/org/freedesktop/Notifications",
                Some("org.freedesktop.Notifications"),
                "Notify",
                &(
                    "Grove",
                    replaces,
                    "grove",
                    summary,
                    body,
                    actions.as_slice(),
                    hints,
                    expires,
                ),
            )
            .await
            .map_err(|_| Problem::NoService)?;
        reply.body().deserialize::<u32>().map_err(|_| Problem::NoService)
    })
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc;
    use std::time::Duration;

    use zbus::connection::Builder;
    use zbus::interface;

    use super::*;

    /// A stand-in for the desktop's notification service, which passes on
    /// what it was sent and never shows anything.
    struct Stand {
        sent: mpsc::Sender<(String, u32, String, String, String)>,
    }

    #[interface(name = "org.freedesktop.Notifications")]
    impl Stand {
        #[expect(
            clippy::too_many_arguments,
            reason = "the notification service's own Notify takes eight"
        )]
        fn notify(
            &self,
            app: String,
            replaces: u32,
            icon: String,
            summary: String,
            body: String,
            _actions: Vec<String>,
            hints: HashMap<String, zbus::OwnedVariant>,
            _expires: i32,
        ) -> u32 {
            let entry = hints
                .get("desktop-entry")
                .and_then(|entry| String::try_from(entry.clone()).ok());
            let _ = self
                .sent
                .send((app, replaces, icon, summary, entry.unwrap_or_default() + "|" + &body));
            replaces + 7
        }
    }

    /// Tests that talk to a bus are skipped on a desktop with none, where
    /// `send` itself reports `Problem::NoBus`. CI runs them under a session
    /// bus of its own, and fails without one: a skipped test checks nothing.
    fn bus_missing() -> bool {
        let missing = zbus::block_on(Connection::session()).is_err();
        assert!(
            !(missing && std::env::var_os("CI").is_some()),
            "CI must run the tests under a session bus"
        );
        missing
    }

    #[test]
    fn a_notification_reaches_the_service_and_replaces_the_last() {
        if bus_missing() {
            return;
        }
        let name = format!("org.grove.TestNotifications{}", std::process::id());
        let (sent, received) = mpsc::channel();
        let stand = Builder::session()
            .name(name.as_str())
            .serve_at("/org/freedesktop/Notifications", Stand { sent });
        let _stand = zbus::block_on(stand.build()).unwrap();

        assert_eq!(send(&name, 0, "Grown", "Plant another."), Ok(7));
        let (app, replaces, icon, summary, rest) = received.recv_timeout(Duration::from_secs(5)).unwrap();
        assert_eq!(
            (app.as_str(), replaces, icon.as_str(), summary.as_str()),
            ("Grove", 0, "grove", "Grown")
        );
        assert_eq!(rest, "grove|Plant another.");
        assert_eq!(send(&name, 7, "Grown", "Plant another."), Ok(14));
        assert_eq!(received.recv_timeout(Duration::from_secs(5)).unwrap().1, 7);
    }

    #[test]
    fn notifier_says_so_when_there_is_no_service() {
        let sent = send("org.grove.NoSuchService", 0, "Grown", "Plant another.");
        assert_eq!(
            sent,
            Err(if bus_missing() {
                Problem::NoBus
            } else {
                Problem::NoService
            })
        );
    }
}
