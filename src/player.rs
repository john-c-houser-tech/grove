//! Plays the distant sounds, on a thread of its own: nothing of the audio
//! system is ever touched from the GUI thread, so a slow or stuck sound
//! server cannot hold up the window. The output is opened afresh for each
//! sound and let go when it ends, so a sound follows whatever the default
//! output is by then: headphones, a Bluetooth headset, a dock.

use std::collections::HashMap;
use std::num::NonZero;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::sync::Arc;
use std::time::Duration;

use rodio::buffer::SamplesBuffer;
use rodio::{DeviceSinkBuilder, DeviceSinkError, MixerDeviceSink};

use crate::sounds::Kind;
use crate::synth;

/// Why a sound could not be heard, for the window to put into words.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Problem {
    /// There is no audio output.
    NoOutput,
    /// The output could not be used, or went away mid-sound.
    Unusable,
}

/// What the player's thread reports back.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Report {
    /// The sound has started, so its picture should too: a sound that was
    /// late being made never leaves them out of step.
    Started(Kind),
    /// The sound could not be played; its picture still goes.
    Failed(Kind, Problem),
    /// The sounds were muted while this one was being made: the picture
    /// still goes, without it.
    Muted(Kind),
    /// The output went away while a sound was playing.
    Broke(Problem),
}

/// Somewhere to play a phrase: the audio system, or a stand-in for tests.
pub trait Output {
    /// Starts playing mono samples at `synth::SAMPLE_RATE`, at a volume
    /// from 0 to 1, in place of anything playing.
    fn start(&mut self, samples: &[f32], volume: f32) -> Result<(), Problem>;
    /// The volume of what is playing.
    fn volume(&self) -> f32;
    fn set_volume(&mut self, volume: f32);
    /// Whether a sound is still playing.
    fn playing(&self) -> bool;
    /// Whether the output failed since the sound started.
    fn broken(&self) -> bool;
    /// Stops the sound and lets the output go.
    fn stop(&mut self);
}

/// The audio system, through rodio.
#[derive(Default)]
pub struct Speakers {
    playing: Option<(MixerDeviceSink, rodio::Player)>,
    broken: Arc<AtomicBool>,
}

impl Output for Speakers {
    fn start(&mut self, samples: &[f32], volume: f32) -> Result<(), Problem> {
        self.stop();
        let broken = Arc::new(AtomicBool::new(false));
        let on_error = Arc::clone(&broken);
        let mut sink = DeviceSinkBuilder::from_default_device()
            .and_then(|builder| {
                builder
                    .with_error_callback(move |_| on_error.store(true, Ordering::SeqCst))
                    .open_sink_or_fallback()
            })
            .map_err(|error| match error {
                DeviceSinkError::NoDevice => Problem::NoOutput,
                _ => Problem::Unusable,
            })?;
        // Letting the output go is how a sound ends here; nothing to log.
        sink.log_on_drop(false);
        let player = rodio::Player::connect_new(sink.mixer());
        player.set_volume(volume);
        let (mono, rate) = (
            NonZero::<u16>::MIN,
            NonZero::new(synth::SAMPLE_RATE).unwrap_or(NonZero::<u32>::MIN),
        );
        player.append(SamplesBuffer::new(mono, rate, samples));
        self.playing = Some((sink, player));
        self.broken = broken;
        Ok(())
    }

    fn volume(&self) -> f32 {
        self.playing.as_ref().map_or(0.0, |(_, player)| player.volume())
    }

    fn set_volume(&mut self, volume: f32) {
        if let Some((_, player)) = &self.playing {
            player.set_volume(volume);
        }
    }

    fn playing(&self) -> bool {
        self.playing.as_ref().is_some_and(|(_, player)| !player.empty())
    }

    fn broken(&self) -> bool {
        self.broken.load(Ordering::SeqCst)
    }

    fn stop(&mut self) {
        self.playing = None;
    }
}

/// What the player's thread is asked to do.
enum Command {
    /// Make a sound ahead of time.
    Prepare(Kind),
    /// Play a sound of the session of that number, at that volume.
    Play(Kind, f32, u64),
    /// Fade out what is playing.
    FadeOut,
    /// Stop what is playing at once.
    Stop,
}

/// How often a playing sound is looked at, to fade it or to see it end.
const TICK: Duration = Duration::from_millis(30);
/// A fade takes this many ticks: a second and a half.
const FADE_TICKS: f32 = 50.0;

/// A handle on the player's thread, which ends when the handle is dropped.
pub struct Player {
    commands: Sender<Command>,
    /// Counts sessions, so a sound still waiting to be made when its session
    /// ends is not played in the next.
    session: Arc<AtomicU64>,
    muted: Arc<AtomicBool>,
}

impl Player {
    /// Starts a player on the speakers. `report` is called on the player's
    /// thread.
    pub fn new(report: impl Fn(Report) + Send + 'static) -> Self {
        Self::with_output(Speakers::default, report)
    }

    /// Starts a player on an output made on the player's own thread, where
    /// it stays.
    pub fn with_output<O: Output>(
        output: impl FnOnce() -> O + Send + 'static,
        report: impl Fn(Report) + Send + 'static,
    ) -> Self {
        let (commands, received) = mpsc::channel();
        let session = Arc::new(AtomicU64::new(0));
        let muted = Arc::new(AtomicBool::new(false));
        let (current, silenced) = (Arc::clone(&session), Arc::clone(&muted));
        std::thread::spawn(move || run(output(), &received, &current, &silenced, &report));
        Self {
            commands,
            session,
            muted,
        }
    }

    /// Has a sound made now, in the background, well before it is due.
    pub fn prepare(&self, kind: Kind) {
        let _ = self.commands.send(Command::Prepare(kind));
    }

    /// Plays a sound of the current session at a volume from 0 to 1, once
    /// it is made, unless something is already playing or the session has
    /// ended by then.
    pub fn play(&self, kind: Kind, volume: f64) {
        let session = self.session.load(Ordering::SeqCst);
        let _ = self.commands.send(Command::Play(kind, volume as f32, session));
    }

    /// The session is over: a sound under way fades out over a second and a
    /// half rather than stopping with a click, and one still waiting to
    /// start is dropped.
    pub fn end_session(&self) {
        self.session.fetch_add(1, Ordering::SeqCst);
        let _ = self.commands.send(Command::FadeOut);
    }

    /// Muting stops what is playing at once, and keeps a sound still being
    /// made from starting.
    pub fn set_muted(&self, muted: bool) {
        self.muted.store(muted, Ordering::SeqCst);
        if muted {
            let _ = self.commands.send(Command::Stop);
        }
    }
}

/// The player's thread: makes the sounds, and plays them one at a time.
fn run<O: Output>(
    mut output: O,
    commands: &Receiver<Command>,
    session: &AtomicU64,
    muted: &AtomicBool,
    report: &dyn Fn(Report),
) {
    let mut phrases: HashMap<u8, Vec<f32>> = HashMap::new();
    let mut phrase = |kind: Kind| -> Vec<f32> {
        phrases
            .entry(kind as u8)
            .or_insert_with(|| {
                let pcm = match kind {
                    Kind::Train => synth::train(synth::SAMPLE_RATE),
                    Kind::Plane => synth::plane(synth::SAMPLE_RATE),
                };
                pcm.into_iter().map(|sample| f32::from(sample) / 32768.0).collect()
            })
            .clone()
    };
    // How much quieter each tick makes a sound that is fading out.
    let mut fade_step: Option<f32> = None;
    loop {
        // While a sound plays it is looked at every tick; otherwise the
        // thread sleeps until it is asked for something.
        let command = if output.playing() {
            match commands.recv_timeout(TICK) {
                Ok(command) => Some(command),
                Err(RecvTimeoutError::Timeout) => None,
                Err(RecvTimeoutError::Disconnected) => break,
            }
        } else {
            output.stop();
            fade_step = None;
            match commands.recv() {
                Ok(command) => Some(command),
                Err(_) => break,
            }
        };
        match command {
            Some(Command::Prepare(kind)) => {
                phrase(kind);
            }
            Some(Command::Play(kind, volume, of_session)) => {
                if output.playing() || of_session != session.load(Ordering::SeqCst) {
                    continue;
                }
                let samples = phrase(kind);
                // The session may have ended while the sound was being made.
                if of_session != session.load(Ordering::SeqCst) {
                    continue;
                }
                if muted.load(Ordering::SeqCst) {
                    report(Report::Muted(kind));
                    continue;
                }
                fade_step = None;
                match output.start(&samples, volume) {
                    Ok(()) => report(Report::Started(kind)),
                    Err(problem) => report(Report::Failed(kind, problem)),
                }
            }
            Some(Command::FadeOut) => {
                if output.playing() && fade_step.is_none() {
                    fade_step = Some(output.volume() / FADE_TICKS);
                }
            }
            Some(Command::Stop) => output.stop(),
            None => {
                if output.broken() {
                    output.stop();
                    report(Report::Broke(Problem::Unusable));
                } else if let Some(step) = fade_step {
                    let volume = output.volume() - step;
                    if volume <= 0.001 {
                        output.stop();
                    } else {
                        output.set_volume(volume);
                    }
                }
            }
        }
    }
    output.stop();
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;
    use std::time::Instant;

    use super::*;

    /// What a stand-in output was asked to do, seen from the test.
    #[derive(Default)]
    struct Heard {
        starts: Vec<(usize, f32)>,
        volume: f32,
        playing: bool,
        broken: bool,
        fails: Option<Problem>,
    }

    /// An output that plays nothing and never ends a sound by itself.
    struct Stand(Arc<Mutex<Heard>>);

    impl Output for Stand {
        fn start(&mut self, samples: &[f32], volume: f32) -> Result<(), Problem> {
            let mut heard = self.0.lock().unwrap();
            if let Some(problem) = heard.fails {
                return Err(problem);
            }
            heard.starts.push((samples.len(), volume));
            heard.volume = volume;
            heard.playing = true;
            Ok(())
        }
        fn volume(&self) -> f32 {
            self.0.lock().unwrap().volume
        }
        fn set_volume(&mut self, volume: f32) {
            self.0.lock().unwrap().volume = volume;
        }
        fn playing(&self) -> bool {
            self.0.lock().unwrap().playing
        }
        fn broken(&self) -> bool {
            self.0.lock().unwrap().broken
        }
        fn stop(&mut self) {
            self.0.lock().unwrap().playing = false;
        }
    }

    const SOON: Duration = Duration::from_secs(20);

    fn player() -> (Player, Arc<Mutex<Heard>>, Receiver<Report>) {
        let heard = Arc::new(Mutex::new(Heard::default()));
        let (report, reports) = mpsc::channel();
        let output = Arc::clone(&heard);
        let player = Player::with_output(
            move || Stand(output),
            move |told| {
                let _ = report.send(told);
            },
        );
        (player, heard, reports)
    }

    fn wait_until(what: &str, done: impl Fn() -> bool) {
        let deadline = Instant::now() + SOON;
        while !done() {
            assert!(Instant::now() < deadline, "{what}");
            std::thread::sleep(Duration::from_millis(5));
        }
    }

    #[test]
    fn a_sound_starts_once_it_is_made_and_says_so() {
        let (player, heard, reports) = player();
        player.prepare(Kind::Plane);
        player.play(Kind::Plane, 0.3);
        assert_eq!(reports.recv_timeout(SOON), Ok(Report::Started(Kind::Plane)));
        let starts = heard.lock().unwrap().starts.clone();
        assert_eq!(
            starts,
            [(synth::PLANE_SECONDS as usize * synth::SAMPLE_RATE as usize, 0.3)]
        );
        // One sound at a time: a second is not started over the first.
        player.play(Kind::Plane, 0.3);
        assert!(reports.recv_timeout(Duration::from_millis(300)).is_err());
    }

    #[test]
    fn a_sound_whose_session_ended_while_it_was_made_is_not_played() {
        let (player, heard, reports) = player();
        player.play(Kind::Plane, 0.3);
        player.end_session();
        // Had it been played, a later sound could not start over it.
        player.play(Kind::Plane, 0.2);
        assert_eq!(reports.recv_timeout(SOON), Ok(Report::Started(Kind::Plane)));
        assert_eq!(heard.lock().unwrap().starts.len(), 1);
        assert_eq!(heard.lock().unwrap().starts[0].1, 0.2);
    }

    #[test]
    fn ending_the_session_fades_the_sound_out() {
        let (player, heard, reports) = player();
        player.play(Kind::Plane, 0.3);
        reports.recv_timeout(SOON).unwrap();
        player.end_session();
        wait_until("the sound never got quieter", || heard.lock().unwrap().volume < 0.25);
        assert!(heard.lock().unwrap().playing, "it should fade, not stop at once");
        wait_until("the sound never stopped", || !heard.lock().unwrap().playing);
    }

    #[test]
    fn muting_stops_the_sound_at_once() {
        let (player, heard, reports) = player();
        player.play(Kind::Plane, 0.3);
        reports.recv_timeout(SOON).unwrap();
        player.set_muted(true);
        wait_until("the sound never stopped", || !heard.lock().unwrap().playing);
        assert_eq!(heard.lock().unwrap().volume, 0.3);
    }

    #[test]
    fn a_sound_muted_while_it_was_made_keeps_its_picture() {
        let (player, heard, reports) = player();
        player.set_muted(true);
        player.play(Kind::Plane, 0.3);
        assert_eq!(reports.recv_timeout(SOON), Ok(Report::Muted(Kind::Plane)));
        assert!(heard.lock().unwrap().starts.is_empty());
        player.set_muted(false);
        player.play(Kind::Plane, 0.3);
        assert_eq!(reports.recv_timeout(SOON), Ok(Report::Started(Kind::Plane)));
    }

    #[test]
    fn a_sound_that_cannot_be_played_still_reports_for_its_picture() {
        let (player, heard, reports) = player();
        heard.lock().unwrap().fails = Some(Problem::NoOutput);
        player.play(Kind::Plane, 0.3);
        assert_eq!(
            reports.recv_timeout(SOON),
            Ok(Report::Failed(Kind::Plane, Problem::NoOutput))
        );
    }

    #[test]
    fn an_output_that_goes_away_mid_sound_is_reported() {
        let (player, heard, reports) = player();
        player.play(Kind::Plane, 0.3);
        reports.recv_timeout(SOON).unwrap();
        heard.lock().unwrap().broken = true;
        assert_eq!(reports.recv_timeout(SOON), Ok(Report::Broke(Problem::Unusable)));
        wait_until("the sound never stopped", || !heard.lock().unwrap().playing);
    }
}
