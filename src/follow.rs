//! Following the desktop live: threads that watch the theme's files and
//! Hyprland's rounding, and call back when either may have changed. Nothing
//! here knows about Qt; the callbacks carry the news to the GUI thread.

use std::io::{BufRead, BufReader, Read};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::mpsc::{self, RecvTimeoutError, Sender};
use std::time::{Duration, Instant};

use notify::{RecursiveMode, Watcher};

use crate::theme::{paths_to_watch, rounding_in};

/// A theme switch touches many files; its changes are taken as one.
const SETTLE: Duration = Duration::from_millis(80);
/// How long hyprctl gets to answer.
const HYPRCTL_WAIT: Duration = Duration::from_millis(500);

/// How to reach Hyprland.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Hyprland {
    /// The hyprctl program, by name or path.
    pub hyprctl: PathBuf,
    /// Hyprland's event socket, which says when its config has been
    /// reloaded; `None` to ask for the rounding only when the theme changes.
    pub events: Option<PathBuf>,
}

/// What the watcher's thread is told.
enum News {
    Changed,
    Stop,
}

/// A thread watching files. It stops when this is dropped, or by itself once
/// its callback says there is no one left to tell.
pub struct Watching {
    thread: Sender<News>,
}

impl Drop for Watching {
    fn drop(&mut self) {
        let _ = self.thread.send(News::Stop);
    }
}

/// The threads following the desktop. They stop when this is dropped, or by
/// themselves once a callback says there is no one left to tell.
pub struct Following {
    _watching: Option<Watching>,
}

/// Watches the paths `paths` names, and calls `changed`, on a thread of its
/// own, when any of them changes; a burst of changes is told once, when it
/// has settled. `paths` is asked again after every change, since what was
/// watched may have been replaced. `changed` returns false when there is no
/// one left to tell, which ends the thread.
pub fn watch(
    paths: impl Fn() -> Vec<PathBuf> + Send + 'static,
    changed: impl Fn() -> bool + Send + 'static,
) -> Watching {
    let (tell, told) = mpsc::channel::<News>();
    let from_watcher = tell.clone();
    std::thread::spawn(move || {
        let Ok(mut watcher) = notify::recommended_watcher(move |_| {
            let _ = from_watcher.send(News::Changed);
        }) else {
            return;
        };
        let mut watched = rewatch(&mut watcher, paths(), Vec::new());
        while let Ok(News::Changed) = told.recv() {
            // Wait until the changes stop coming.
            loop {
                match told.recv_timeout(SETTLE) {
                    Ok(News::Changed) => {}
                    Ok(News::Stop) | Err(RecvTimeoutError::Disconnected) => return,
                    Err(RecvTimeoutError::Timeout) => break,
                }
            }
            watched = rewatch(&mut watcher, paths(), watched);
            if !changed() {
                return;
            }
        }
    });
    Watching { thread: tell }
}

/// Starts following the theme in `directory` and, when given, Hyprland's
/// rounding. `theme_changed` is called, on a thread of its own, when the
/// theme's files may have changed; `rounding` with each rounding Hyprland
/// reports: once at the start, after every theme change, and after every
/// reload of Hyprland's config. Either returns false when there is no one
/// left to tell, which ends its thread.
pub fn follow(
    directory: Option<PathBuf>,
    hyprland: Option<Hyprland>,
    theme_changed: impl Fn() -> bool + Send + 'static,
    rounding: impl Fn(i32) -> bool + Send + 'static,
) -> Following {
    // Corners follow Hyprland's effective rounding, which themes and personal
    // overrides can both set, so hyprctl is asked rather than its config read.
    let ask = hyprland.map(|hyprland| {
        let (ask, asked) = mpsc::channel::<()>();
        let hyprctl = hyprland.hyprctl.clone();
        std::thread::spawn(move || {
            while asked.recv().is_ok() {
                // Several requests waiting are answered by one question.
                while asked.try_recv().is_ok() {}
                if let Some(radius) = ask_rounding(&hyprctl) {
                    if !rounding(radius) {
                        break;
                    }
                }
            }
        });
        // Hyprland announces every config reload on its event socket, which
        // covers a theme switch and an edited override alike, so the rounding
        // is asked for then rather than polled.
        if let Some(events) = hyprland.events {
            let ask = ask.clone();
            std::thread::spawn(move || {
                let Ok(socket) = UnixStream::connect(events) else {
                    return;
                };
                for line in BufReader::new(socket).lines() {
                    match line {
                        Ok(line) if line.starts_with("configreloaded>>") => {
                            if ask.send(()).is_err() {
                                break;
                            }
                        }
                        Ok(_) => {}
                        Err(_) => break,
                    }
                }
            });
        }
        let _ = ask.send(());
        ask
    });

    let watching = directory.map(|directory| {
        watch(
            move || paths_to_watch(&directory),
            move || {
                if let Some(ask) = &ask {
                    let _ = ask.send(());
                }
                theme_changed()
            },
        )
    });
    Following { _watching: watching }
}

/// Stops watching `watched` and watches `paths`, as far as they can be.
fn rewatch(watcher: &mut impl Watcher, mut paths: Vec<PathBuf>, watched: Vec<PathBuf>) -> Vec<PathBuf> {
    for path in &watched {
        let _ = watcher.unwatch(path);
    }
    paths.retain(|path| watcher.watch(path, RecursiveMode::NonRecursive).is_ok());
    paths
}

/// Asks hyprctl for the rounding. `None` if it fails, takes too long or
/// answers with something else; without Hyprland the corners stay as they are.
pub fn ask_rounding(hyprctl: &Path) -> Option<i32> {
    let mut child = Command::new(hyprctl)
        .args(["-j", "getoption", "decoration:rounding"])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;
    let deadline = Instant::now() + HYPRCTL_WAIT;
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) if Instant::now() < deadline => std::thread::sleep(Duration::from_millis(5)),
            _ => {
                let _ = child.kill();
                let _ = child.wait();
                return None;
            }
        }
    };
    let mut output = Vec::new();
    child.stdout.take()?.read_to_end(&mut output).ok()?;
    status.success().then(|| rounding_in(&output)).flatten()
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::io::Write;
    use std::os::unix::fs::PermissionsExt;
    use std::os::unix::net::UnixListener;

    use tempfile::TempDir;

    use super::*;

    const SOON: Duration = Duration::from_secs(5);

    /// A stand-in for hyprctl that prints `output`.
    fn hyprctl(dir: &TempDir, script: &str) -> PathBuf {
        let path = dir.path().join("hyprctl");
        fs::write(&path, format!("#!/bin/sh\n{script}\n")).unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
        path
    }

    #[test]
    fn hyprctl_is_asked_for_the_rounding() {
        let dir = TempDir::new().unwrap();
        // The arguments are checked, so only the right question is answered.
        let script = r#"[ "$*" = "-j getoption decoration:rounding" ] && echo '{"int": 12}'"#;
        assert_eq!(ask_rounding(&hyprctl(&dir, script)), Some(12));
    }

    #[test]
    fn a_failing_slow_or_missing_hyprctl_gives_no_rounding() {
        let dir = TempDir::new().unwrap();
        assert_eq!(ask_rounding(&hyprctl(&dir, "echo '{\"int\": 12}'; exit 1")), None);
        assert_eq!(ask_rounding(&hyprctl(&dir, "echo nonsense")), None);
        let started = Instant::now();
        assert_eq!(ask_rounding(&hyprctl(&dir, "exec sleep 30")), None);
        assert!(started.elapsed() < Duration::from_secs(5));
        assert_eq!(ask_rounding(&dir.path().join("no-such-program")), None);
    }

    #[test]
    fn a_theme_change_is_followed() {
        let dir = TempDir::new().unwrap();
        let current = dir.path().join("current");
        let (changed, changes) = mpsc::channel();
        let _following = follow(Some(current.clone()), None, move || changed.send(()).is_ok(), |_| true);
        // The theme arrives after Grove started, as when Omarchy is installed later.
        std::thread::sleep(Duration::from_millis(100));
        fs::create_dir_all(current.join("theme")).unwrap();
        changes
            .recv_timeout(SOON)
            .expect("the new theme folder was not noticed");
        // By now the new folders are watched, so the colours are seen arriving.
        while changes.recv_timeout(Duration::from_millis(300)).is_ok() {}
        fs::write(current.join("theme/colors.toml"), "accent = \"#112233\"\n").unwrap();
        changes.recv_timeout(SOON).expect("the new colours were not noticed");
    }

    #[test]
    fn the_rounding_is_asked_at_the_start_and_when_hyprland_reloads() {
        let dir = TempDir::new().unwrap();
        let events = dir.path().join("events.sock");
        let listener = UnixListener::bind(&events).unwrap();
        let hyprland = Hyprland {
            hyprctl: hyprctl(&dir, "echo '{\"int\": 6}'"),
            events: Some(events),
        };
        let (rounded, roundings) = mpsc::channel();
        let _following = follow(
            None,
            Some(hyprland),
            || true,
            move |radius| rounded.send(radius).is_ok(),
        );
        assert_eq!(roundings.recv_timeout(SOON), Ok(6));

        let (mut hyprland, _) = listener.accept().unwrap();
        hyprland.write_all(b"workspace>>2\n").unwrap();
        assert!(roundings.recv_timeout(Duration::from_millis(300)).is_err());
        hyprland.write_all(b"configreloaded>>\n").unwrap();
        assert_eq!(roundings.recv_timeout(SOON), Ok(6));
    }
}
