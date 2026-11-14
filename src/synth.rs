//! The distant sounds, synthesized: a train approaching a crossing, and a
//! small plane flying over. Each is one phrase of 16-bit mono samples, never
//! shipped or fetched. They take a moment to make, so they are made on
//! another thread well before they are due.

use std::f64::consts::PI;

use fastrand::Rng;

use crate::easing::clamp01;

/// The rate the phrases are made at, in samples a second.
pub const SAMPLE_RATE: u32 = 44100;
/// How long each phrase lasts, which is how long its picture lasts too.
pub const TRAIN_SECONDS: f64 = 44.0;
pub const PLANE_SECONDS: f64 = 30.0;

/// A one-pole low-pass filter: air takes the highs out of far-off sound.
#[derive(Default)]
struct LowPass {
    alpha: f64,
    state: f64,
}

impl LowPass {
    fn new(hz: f64, rate: f64) -> Self {
        let mut filter = Self::default();
        filter.set_cutoff(hz, rate);
        filter
    }

    fn set_cutoff(&mut self, hz: f64, rate: f64) {
        self.alpha = (-2.0 * PI * hz / rate).exp();
    }

    fn pass(&mut self, x: f64) -> f64 {
        self.state = self.alpha * self.state + (1.0 - self.alpha) * x;
        self.state
    }
}

/// Noise from -1 to 1.
fn hiss(noise: &mut Rng) -> f64 {
    noise.f64() * 2.0 - 1.0
}

/// How near something passing at `pass` seconds is at `t`: 1 as it passes,
/// falling away over `spread` seconds either side.
fn nearness(t: f64, pass: f64, spread: f64) -> f64 {
    (-((t - pass) / spread).powi(2)).exp()
}

/// One train horn blast: three notes of a minor chord, each a touch out of
/// tune so the chord beats, with a quick start and a short release.
fn add_blast(out: &mut [f64], rate: f64, start: f64, seconds: f64, gain: f64, cutoff: f64, pass: f64) {
    const NOTES: [f64; 3] = [311.1, 370.0, 466.2];
    const DETUNE: [f64; 3] = [1.0, 1.004, 0.997];
    let mut phase = [0.0_f64; 3];
    let mut lowpass = LowPass::new(cutoff, rate);
    let first = (start * rate) as usize;
    let count = (seconds * rate) as usize;
    for (i, value) in out.iter_mut().skip(first).take(count).enumerate() {
        let t = start + i as f64 / rate;
        // The horn is sharp as the train closes in and flat once it has gone by.
        let doppler = 1.0 + 0.04 * ((pass - t) / 6.0).tanh();
        let envelope = clamp01((t - start) / 0.1) * clamp01((start + seconds - t) / 0.22);
        let mut sample = 0.0;
        for n in 0..3 {
            phase[n] += 2.0 * PI * NOTES[n] * DETUNE[n] * doppler / rate;
            for h in 1..=6 {
                let h = f64::from(h);
                sample += (h * phase[n]).sin() / h.powf(0.9);
            }
        }
        *value += gain * envelope * lowpass.pass(sample) / 3.0;
    }
}

/// The standard grade-crossing signal: long, long, short, long.
fn add_crossing_signal(out: &mut [f64], rate: f64, start: f64, gain: f64, cutoff: f64, last_long: f64, pass: f64) {
    add_blast(out, rate, start, 2.4, gain, cutoff, pass);
    add_blast(out, rate, start + 2.9, 2.4, gain, cutoff, pass);
    add_blast(out, rate, start + 5.8, 0.7, gain, cutoff, pass);
    add_blast(out, rate, start + 7.0, last_long, gain, cutoff, pass);
}

/// A crossing bell strike: a few inharmonic partials that die away.
fn add_bell(out: &mut [f64], rate: f64, start: f64, gain: f64, pitch: f64) {
    const RATIO: [f64; 4] = [1.0, 1.54, 2.47, 3.35];
    const LEVEL: [f64; 4] = [1.0, 0.6, 0.35, 0.2];
    const DECAY: [f64; 4] = [0.3, 0.22, 0.16, 0.1];
    let first = (start * rate) as usize;
    for (i, value) in out.iter_mut().skip(first).take(rate as usize).enumerate() {
        let t = i as f64 / rate;
        let sample: f64 = (0..4)
            .map(|p| LEVEL[p] * (-t / DECAY[p]).exp() * (2.0 * PI * pitch * RATIO[p] * t).sin())
            .sum();
        *value += gain * sample;
    }
}

/// Distance in a soft echo: sound off the hills a moment later.
fn add_echo(out: &mut [f64], rate: f64, delay_seconds: f64, feedback: f64) {
    let delay = (delay_seconds * rate) as usize;
    for i in delay..out.len() {
        out[i] += feedback * out[i - delay];
    }
}

/// Fades in from nothing over `rise` seconds and out to nothing over `fall`,
/// so there is no click at either end.
fn fade_ends(out: &mut [f64], rate: f64, seconds: f64, rise: f64, fall: f64) {
    for (i, value) in out.iter_mut().enumerate() {
        let t = i as f64 / rate;
        *value *= clamp01(t / rise) * clamp01((seconds - t) / fall);
    }
}

/// Samples scaled so the loudest reaches `peak_level` of full scale.
fn to_pcm(out: &[f64], peak_level: f64) -> Vec<i16> {
    let peak = out.iter().fold(1e-9_f64, |peak, value| peak.max(value.abs()));
    out.iter()
        .map(|value| (value / peak * peak_level * 32767.0) as i16)
        .collect()
}

/// A train coming to a crossing and going by: the rails, two crossing
/// signals on the horn, and the crossing's bells.
pub fn train(sample_rate: u32) -> Vec<i16> {
    let rate = f64::from(sample_rate);
    let mut out = vec![0.0_f64; (TRAIN_SECONDS * rate) as usize];
    let mut noise = Rng::with_seed(11);
    // The train reaches the crossing here; everything swells to it and fades after.
    let pass = 26.0;

    // The rumble of the rails, and the clickety-clack of the joints.
    let (mut slow, mut softer) = (LowPass::new(90.0, rate), LowPass::new(160.0, rate));
    for (i, value) in out.iter_mut().enumerate() {
        let near = nearness(i as f64 / rate, pass, 8.5);
        *value += 5.0 * near * softer.pass(slow.pass(hiss(&mut noise)));
    }
    let mut t = 2.0;
    while t < TRAIN_SECONDS - 2.0 {
        let near = nearness(t, pass, 8.5);
        for offset in [0.0, 0.07] {
            let first = ((t + offset) * rate) as usize;
            let mut tick = LowPass::new(600.0, rate);
            for (j, value) in out.iter_mut().skip(first).take(sample_rate as usize / 20).enumerate() {
                *value += 0.7 * near * (-(j as f64) / (0.007 * rate)).exp() * tick.pass(hiss(&mut noise));
            }
        }
        t += 0.27;
    }

    // Two crossing signals: first far off and soft, then closer and brighter.
    add_crossing_signal(&mut out, rate, 3.0, 0.45, 650.0, 2.0, pass);
    add_crossing_signal(&mut out, rate, 16.0, 0.85, 1100.0, 3.0, pass);

    // Crossing bells start before the train is there and run until it is clear.
    let mut t = 12.5;
    while t < 34.0 {
        add_bell(&mut out, rate, t, 0.06 + 0.3 * nearness(t, pass, 7.0), 1450.0);
        t += 0.62;
    }

    // Everything is far away: duller, with the echo of the hills.
    let (mut dull, mut dull_again) = (LowPass::new(3200.0, rate), LowPass::new(3200.0, rate));
    for value in &mut out {
        *value = dull_again.pass(dull.pass(*value));
    }
    add_echo(&mut out, rate, 0.29, 0.3);
    fade_ends(&mut out, rate, TRAIN_SECONDS, 1.5, 3.0);
    to_pcm(&out, 0.6)
}

/// A small plane flying over, very far up.
pub fn plane(sample_rate: u32) -> Vec<i16> {
    let rate = f64::from(sample_rate);
    let mut out = vec![0.0_f64; (PLANE_SECONDS * rate) as usize];
    let mut noise = Rng::with_seed(23);
    // Overhead at the middle of the flight: loudest and brightest then, with
    // the pitch falling as it passes.
    let center = PLANE_SECONDS / 2.0;
    let mut phase = 0.0_f64;
    let (mut air_a, mut air_b, mut wind) = (LowPass::default(), LowPass::default(), LowPass::new(200.0, rate));
    for (i, value) in out.iter_mut().enumerate() {
        let t = i as f64 / rate;
        let near = nearness(t, center, 6.5);
        let doppler = 1.0 + 0.03 * ((center - t) / 4.0).tanh();
        // A little drift in the revs, as a hand on a throttle gives.
        let revs = 1.0 + 0.004 * (2.0 * PI * 0.7 * t).sin() + 0.003 * (2.0 * PI * 1.9 * t).sin();
        phase += 2.0 * PI * 86.0 * doppler * revs / rate;
        let mut sample: f64 = (1..=10)
            .map(|h| {
                let h = f64::from(h);
                (h * phase + 0.3 * h).sin() / h.powf(0.8)
            })
            .sum();
        sample *= 1.0 + 0.18 * (phase / 2.0).sin();
        sample += 1.4 * wind.pass(hiss(&mut noise));
        // Nearer means brighter: far-off engines lose their highs to the air.
        let cutoff = 380.0 + 520.0 * near;
        air_a.set_cutoff(cutoff, rate);
        air_b.set_cutoff(cutoff, rate);
        *value = (0.04 + 0.96 * near) * air_b.pass(air_a.pass(sample));
    }
    fade_ends(&mut out, rate, PLANE_SECONDS, 2.0, 3.0);
    // Much fainter than the train: this is a plane very far up.
    to_pcm(&out, 0.13)
}

#[cfg(test)]
mod tests {
    use super::*;

    const RATE: usize = SAMPLE_RATE as usize;

    fn peak(pcm: &[i16]) -> i32 {
        pcm.iter().map(|sample| i32::from(*sample).abs()).max().unwrap_or(0)
    }

    /// The loudness of the seconds from `from` to `to`.
    fn rms(pcm: &[i16], from: f64, to: f64) -> f64 {
        let part = &pcm[(from * RATE as f64) as usize..(to * RATE as f64) as usize];
        (part.iter().map(|sample| f64::from(*sample).powi(2)).sum::<f64>() / part.len() as f64).sqrt()
    }

    #[test]
    fn train_fills_its_length_and_fades_in_and_out() {
        let pcm = train(SAMPLE_RATE);
        assert_eq!(pcm.len(), TRAIN_SECONDS as usize * RATE);
        assert!(peak(&pcm) > 15000);
        // No click at either end.
        assert!(peak(&pcm[..200]) < 300);
        assert!(peak(&pcm[pcm.len() - 200..]) < 300);
    }

    #[test]
    fn train_horns_are_loudest_as_it_arrives() {
        let pcm = train(SAMPLE_RATE);
        // The first signal is far off, the second close, and the rails alone are quiet.
        let (rails, far, close) = (rms(&pcm, 0.5, 2.5), rms(&pcm, 3.2, 5.2), rms(&pcm, 16.2, 18.2));
        assert!(far > rails * 2.0);
        assert!(close > far * 1.5);
        // It has gone by well before the end.
        assert!(rms(&pcm, 40.0, 42.0) < close * 0.3);
    }

    #[test]
    fn plane_is_much_fainter_than_the_train() {
        let (plane, train) = (plane(SAMPLE_RATE), train(SAMPLE_RATE));
        assert_eq!(plane.len(), PLANE_SECONDS as usize * RATE);
        assert!(peak(&plane) > 3000);
        assert!(peak(&plane) < peak(&train) / 2);
    }

    #[test]
    fn plane_passes_overhead_in_the_middle() {
        let pcm = plane(SAMPLE_RATE);
        let (middle, early, late) = (rms(&pcm, 13.0, 17.0), rms(&pcm, 2.0, 5.0), rms(&pcm, 25.0, 28.0));
        assert!(middle > early * 3.0);
        assert!(middle > late * 3.0);
    }

    #[test]
    fn a_phrase_is_the_same_every_time() {
        assert_eq!(plane(8000), plane(8000));
    }
}
