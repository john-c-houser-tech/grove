//! The ploughed field in the foreground, seen in true perspective by someone
//! standing at the edge of it, eyes 1.6 m above the ground.
//!
//! The rows run straight away from the viewer and meet at the horizon. They
//! are rounded ridges 70 cm apart and 20 cm high, with a narrow trough
//! between them, wavering a little along their length as ploughed rows do.
//! The plant is a sapling about 2.5 m tall, standing in the furrow between
//! two rows, 4 m away. Every size here follows from those numbers, so the
//! rows, the plant and the trunk agree.

use std::f64::consts::PI;

use cxx_qt_lib::QColor;

use super::paint::color::{darker, lighter, mix, rgba};
use super::paint::{Gradient, Painter, Path};
use crate::terrain::{hash, noise};

// The camera. A metre at the plant's distance is this much of the height,
// which with the plant 4 m off fixes the focal length. The eye is at standing
// height, so the horizon crosses the plant 1.6 m up its trunk.
const METRE_AT_PLANT: f64 = 0.21;
const PLANT_DEPTH: f64 = 4.0;
const EYE: f64 = 1.6;
/// The field ends this far away, where the meadow takes over.
const FIELD_DEPTH: f64 = 40.0;

const ROW_SPACING: f64 = 0.7;
const RIDGE_HEIGHT: f64 = 0.2;
/// Ridges 60 cm across at the base, leaving a 10 cm trough between them.
const RIDGE_HALF_WIDTH: f64 = 0.3;

fn focal_for(height: f64) -> f64 {
    height * METRE_AT_PLANT * PLANT_DEPTH
}

/// Where the field ends, out at 40 m, in pixels from the top of a window
/// `height` high with its horizon at that fraction of it.
pub fn far_edge_y(height: f64, horizon: f64) -> f64 {
    height * horizon + focal_for(height) * EYE / FIELD_DEPTH
}

/// Where the plant's foot is, in pixels from the top.
pub fn plant_base_y(height: f64, horizon: f64) -> f64 {
    height * horizon + focal_for(height) * EYE / PLANT_DEPTH
}

/// Which of the field a layer draws.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Part {
    /// The ground and the rows beyond the plant.
    Far,
    /// The rows in front of the plant, which are drawn over it: a ridge
    /// between the eye and the trunk hides the trunk's foot, as it would in
    /// a real field.
    Near,
}

impl Part {
    /// The part QML names "far"; anything else is the near one.
    pub fn named(name: &str) -> Self {
        if name == "far" {
            Part::Far
        } else {
            Part::Near
        }
    }
}

/// Everything `paint` draws from, copied whole to the drawing thread.
pub struct Look {
    /// The horizon, as a fraction of the height.
    pub horizon: f64,
    /// Where the field's far edge meets the meadow, in pixels from the top.
    pub ground_top: f64,
    /// Where the rows meet on the horizon, and where the plant stands, as
    /// fractions of the width.
    pub vanish_x: f64,
    pub plant_x: f64,
    pub part: Part,
    pub soil_color: QColor,
    /// The sky at the horizon, for the haze over the far side of the field.
    pub haze_color: QColor,
    /// Snow lying on the field, from 0 to 1, and frost on the crests in
    /// winter.
    pub snow: f64,
    pub frost: bool,
}

/// A row of the field: its number, counted from the plant's trough, and how
/// far its crest is to the side of straight ahead, in metres.
struct Ridge {
    j: i32,
    c: f64,
}

/// The height of a rounded ridge at `t` of the way from its crest (0) to its
/// foot (1), as a fraction of its full height.
fn profile(t: f64) -> f64 {
    1.0 - t * t
}

/// How bright a ridge is at `t` of the way from crest to foot. Light comes
/// from the upper left. Across a rounded ridge the slope turns from flat at
/// the crest to steep at the foot, so the side facing the light brightens
/// down from the crest, the other side darkens, and both fall into shadow in
/// the trough.
fn brightness(t: f64, facing_light: bool) -> f64 {
    let slope = 2.0 * RIDGE_HEIGHT * t / RIDGE_HALF_WIDTH;
    let len = (1.0 + slope * slope).sqrt();
    let nx = (if facing_light { -slope } else { slope }) / len;
    let nz = 1.0 / len;
    let lit = (nx * -0.55 + nz * 0.83).max(0.0);
    (0.25 + 0.75 * lit) * (1.0 - 0.45 * t * t)
}

/// The rows waver together in long, gentle curves, as a plough drawn by eye
/// does.
fn wave(d: f64) -> f64 {
    0.07 * (d * 0.35 + 1.3).sin() + 0.025 * (d * 0.9 + 0.4).sin()
}

/// How far row `j` has wandered sideways at depth `d`, in metres: the shared
/// curve, measured from the plant so the plant stays in the middle of its
/// trough, and a little unevenness of the row's own.
fn shift(j: i32, d: f64) -> f64 {
    wave(d) - wave(PLANT_DEPTH) + 0.012 * (noise(d * 0.6 + f64::from(j) * 7.7, 22.0) - 0.5)
}

/// How high row `j`'s crest stands at depth `d`, in metres.
fn crest_height(j: i32, d: f64) -> f64 {
    RIDGE_HEIGHT * (0.85 + 0.3 * noise(d * 0.9 + f64::from(j) * 13.1, 21.0))
}

/// Draws the field at `width` by `height` in the window's units. It runs on
/// the drawing thread, and tests call it directly.
pub fn paint(painter: &mut Painter<'_>, look: &Look, width: f64, height: f64) {
    let (w, h) = (width, height);
    if w <= 0.0 || h <= 0.0 {
        return;
    }
    let far = look.part == Part::Far;
    let focal = focal_for(h);
    // How far the plant stands to the side of straight ahead, in metres.
    let plant_lateral = (look.plant_x - look.vanish_x) * w / (h * METRE_AT_PLANT);
    // A point in the field, in metres (across, up, ahead), on the screen.
    let project = |across: f64, up: f64, ahead: f64| -> (f64, f64) {
        (
            w * look.vanish_x + focal * across / ahead,
            h * look.horizon + focal * (EYE - up) / ahead,
        )
    };

    let snow_color = rgba(0.9, 0.93, 0.96, 1.0);
    let cover = (look.snow * 0.85).max(if look.frost { 0.25 } else { 0.0 });
    // The furrow floors are in shadow; the faces of the ridges that turn
    // toward the light are brighter, and snow settles on those first. The
    // soil's own colour runs from the dark of the troughs to the warm brown
    // of a crest in sun; snow lightens it all, the tops most.
    let deep = darker(&look.soil_color, 1.5);
    let bright = lighter(&look.soil_color, 2.7);
    let floor_color = mix(&mix(&deep, &bright, 0.14), &snow_color, cover * 0.6);
    let tone = |level: f64, up: f64| -> QColor {
        let soil = mix(&deep, &bright, level.clamp(0.0, 1.0));
        mix(&soil, &snow_color, cover * (0.55 + 0.45 * up))
    };
    let distant_tone = mix(&tone(0.55, 0.6), &look.haze_color, 0.06);
    painter.save();
    painter.clip_to_rect(0.0, look.ground_top, w, h - look.ground_top);
    // Far off, ridges and furrows blur into one average tone with a little
    // haze in it, so the floor lightens toward the far edge of the field.
    if far {
        let fade = Gradient::new(0.0, look.ground_top, 0.0, project(0.0, 0.0, 10.0).1)
            .stop(0.0, &distant_tone)
            .stop(1.0, &floor_color);
        painter.fill_rect(0.0, look.ground_top, w, h - look.ground_top, &fade);
    }

    // From just in front of the bottom of the window out to where the rows
    // are too close together to see, split at the plant. The rows start well
    // in front of the bottom of the window: a ridge stands up from the
    // ground, so its near end would show if they began where the flat ground
    // leaves the window.
    let nearest = 2.4;
    let from = if far { PLANT_DEPTH } else { nearest };
    // The near rows run a hair past the plant so the two halves meet without a seam.
    let to = if far { FIELD_DEPTH } else { PLANT_DEPTH * 1.01 };
    let steps = if far { 40 } else { 30 };
    let depths: Vec<f64> = (0..=steps)
        .map(|i| from * (to / from).powf(f64::from(i) / f64::from(steps)))
        .collect();

    // Crests sit half a row either side of the plant's trough. Only the side
    // of each ridge that turns toward us can be seen, so the outer rows are
    // drawn first and the inner ones over them.
    let mut ridges: Vec<Ridge> = (-36..=36)
        .map(|j| Ridge {
            j,
            c: plant_lateral + ROW_SPACING / 2.0 + f64::from(j) * ROW_SPACING,
        })
        .collect();
    ridges.sort_by(|a, b| b.c.abs().total_cmp(&a.c.abs()));

    let bands = 8;

    // Clods and grain are drawn out to where they would be under a pixel.
    let clod_limit = to.min(14.0);
    let segments = 8;
    let last = depths.len() - 1;

    for ridge in &ridges {
        let (j0, c) = (ridge.j, ridge.c);
        let side = if c > 0.0 { -1.0 } else { 1.0 };
        let facing_light = c > 0.0;
        // From standing height you look down into the rows, so the far side
        // of a ridge shows too, down to where it is steeper than your line of
        // sight across it. Looking straight along a row you see all of it;
        // rows off to the side show only their tops.
        let across = c.abs().max(0.05);
        let back_reach = ((EYE / across) * RIDGE_HALF_WIDTH / (2.0 * RIDGE_HEIGHT)).min(1.0);
        // A strip of the ridge between two heights on one of its sides, from
        // one depth to another.
        let strip =
            |painter: &mut Painter<'_>, side_sign: f64, t0: f64, t1: f64, fill: &QColor, lo: usize, hi: usize| {
                let edge = |k: usize, t: f64| {
                    let d = depths[k];
                    project(
                        c + shift(j0, d) + side_sign * RIDGE_HALF_WIDTH * t,
                        crest_height(j0, d) * profile(t),
                        d,
                    )
                };
                let mut path = Path::new();
                for k in lo..=hi {
                    let (x, y) = edge(k, t0);
                    if k == lo {
                        path.move_to(x, y);
                    } else {
                        path.line_to(x, y);
                    }
                }
                for q in (lo..=hi).rev() {
                    let (x, y) = edge(q, t1);
                    path.line_to(x, y);
                }
                path.close();
                painter.fill_path(&path, fill);
            };
        // A ridge runs straight in depth, so if both ends of a stretch are
        // off the same side of the window, all of it is.
        let offscreen = |lo: usize, hi: usize| {
            let (a, b) = (project(c, 0.0, depths[lo]).0, project(c, 0.0, depths[hi]).0);
            let margin = focal * RIDGE_HALF_WIDTH * 1.5 / depths[lo];
            (a < -margin && b < -margin) || (a > w + margin && b > w + margin)
        };
        // The far side of the ridge, where it shows.
        for g in 0..segments {
            let (lo, hi) = (g * last / segments, (g + 1) * last / segments);
            if offscreen(lo, hi) {
                continue;
            }
            let mid = depths[(lo + hi) / 2];
            let haze = ((mid - 8.0) / 30.0).clamp(0.0, 1.0);
            // As many shading bands as the ridge is wide on screen allows.
            let wide = focal * RIDGE_HALF_WIDTH * back_reach / mid;
            if wide < 1.0 {
                continue;
            }
            let count = ((wide / 4.0).round() as i32).clamp(1, bands);
            for band in (0..count).rev() {
                let t0 = f64::from(band) / f64::from(count) * back_reach;
                let t1 = f64::from(band + 1) / f64::from(count) * back_reach;
                let tm = (t0 + t1) / 2.0;
                let fill = mix(
                    &tone(brightness(tm, !facing_light) * 1.15, profile(tm)),
                    &distant_tone,
                    haze,
                );
                strip(painter, -side, t0, t1, &fill, lo, hi);
            }
        }

        // The ridge in bands from crest to foot, and in stretches from far to
        // near so the far ones can take on the soil's distant tone.
        for g in 0..segments {
            let (lo, hi) = (g * last / segments, (g + 1) * last / segments);
            if offscreen(lo, hi) {
                continue;
            }
            let mid = depths[(lo + hi) / 2];
            let haze = ((mid - 8.0) / 30.0).clamp(0.0, 1.0);
            let wide = focal * RIDGE_HALF_WIDTH / mid;
            let count = ((wide / 4.0).round() as i32).min(bands).max(2);
            for band in 0..count {
                let t0 = f64::from(band) / f64::from(count);
                let t1 = f64::from(band + 1) / f64::from(count);
                let tm = (t0 + t1) / 2.0;
                let fill = mix(
                    &tone(brightness(tm, facing_light) * 1.15, profile(tm)),
                    &distant_tone,
                    haze,
                );
                strip(painter, side, t0, t1, &fill, lo, hi);
            }
        }

        // Fine grain over the near part of the ridge: specks of soil a
        // centimetre or two across, a little lighter or darker than the
        // ground under them.
        let mut count = 0;
        let mut d = from.max(0.8);
        while d < clod_limit.min(9.0) && count < 120 {
            let seed = f64::from(j0 * 977 + count);
            let t = hash(seed, 11.0).sqrt();
            let (x, y) = project(
                c + shift(j0, d) + side * RIDGE_HALF_WIDTH * t,
                crest_height(j0, d) * profile(t),
                d,
            );
            let speck = (focal * (0.008 + 0.012 * hash(seed, 12.0)) / d).max(1.0);
            if x > -4.0 && x < w + 4.0 {
                let base = tone(brightness(t, facing_light) * 1.15, profile(t));
                let grain = if hash(seed, 13.0) < 0.5 {
                    lighter(&base, 1.25)
                } else {
                    darker(&base, 1.3)
                };
                painter.fill_rect(x, y, speck, speck * 0.7, &grain);
            }
            d += 0.025 + 0.06 * hash(seed, 14.0) * d / 3.0;
            count += 1;
        }

        // A few small clods close by, high on the ridge.
        let mut count = 0;
        let mut d = from.max(0.9);
        while d < to.min(6.0) {
            let seed = f64::from(j0 * 1000 + count);
            let t = 0.15 + 0.6 * hash(seed, 2.0);
            let size = 0.015 + 0.025 * hash(seed, 3.0);
            let body = tone(brightness(t, facing_light) * 1.05, profile(t));
            let at = project(
                c + shift(j0, d) + side * RIDGE_HALF_WIDTH * t,
                crest_height(j0, d) * profile(t),
                d,
            );
            clod(painter, at, focal * size / d, w, seed, &body);
            d += 0.35 + 0.6 * hash(seed, 1.0);
            count += 1;
        }
    }
    painter.restore();
}

/// One clod at a point on the screen, `r` pixels across: an irregular lump
/// with a shadow under it, a body, and a lighter top where the light catches
/// it. Too small to see, or off the window, it is not drawn.
fn clod(painter: &mut Painter<'_>, at: (f64, f64), r: f64, width: f64, seed: f64, body: &QColor) {
    if r < 0.9 || at.0 < -r * 2.0 || at.0 > width + r * 2.0 {
        return;
    }
    let corners = 6;
    let outline = |cx: f64, cy: f64, scale: f64| {
        let mut path = Path::new();
        for v in 0..corners {
            let corner = f64::from(v);
            let angle = (corner + 0.35 * hash(seed, corner)) / f64::from(corners) * PI * 2.0;
            let reach = r * scale * (0.65 + 0.5 * hash(seed, corner + 9.0));
            let (x, y) = (cx + angle.cos() * reach, cy + angle.sin() * reach * 0.72);
            if v == 0 {
                path.move_to(x, y);
            } else {
                path.line_to(x, y);
            }
        }
        path.close();
        path
    };
    painter.fill_path(&outline(at.0 + r * 0.18, at.1 + r * 0.32, 1.0), &darker(body, 1.5));
    painter.fill_path(&outline(at.0, at.1, 1.0), body);
    if r >= 2.5 {
        painter.fill_path(&outline(at.0 - r * 0.18, at.1 - r * 0.24, 0.55), &lighter(body, 1.25));
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::qml::layer::draw;

    /// The look the C++ tests drew the field with, for a window 500 high.
    pub fn look(part: Part, height: f64) -> Look {
        Look {
            horizon: 0.6,
            ground_top: far_edge_y(height, 0.6),
            vanish_x: 0.5,
            plant_x: 0.33,
            part,
            soil_color: QColor::try_from("#2a2018").unwrap(),
            haze_color: QColor::try_from("#9fb7c4").unwrap(),
            snow: 0.0,
            frost: false,
        }
    }

    #[test]
    fn the_field_is_seen_from_standing_height() {
        // The far edge is 40 m off and the plant 4 m, seen from 1.6 m up.
        assert_eq!(far_edge_y(1000.0, 0.6), 600.0 + 1000.0 * 0.21 * 4.0 * 1.6 / 40.0);
        assert_eq!(plant_base_y(1000.0, 0.6), 600.0 + 1000.0 * 0.21 * 4.0 * 1.6 / 4.0);
    }

    #[test]
    fn each_part_of_the_field_keeps_to_its_own_band() {
        let ground = 500.0 * 0.6 + 500.0 * 0.21 * 4.0 * 1.6 / 40.0;
        let field = |part| {
            let look = look(part, 500.0);
            draw(
                Box::new(move |painter, w, h| paint(painter, &look, w, h)),
                400.0,
                500.0,
                -1.0,
                1.0,
            )
            .unwrap()
        };
        // The far field starts at its far edge and runs to the bottom.
        let far = field(Part::Far);
        assert!((far.top - ground).abs() < 1.0);
        assert_eq!(far.top + f64::from(far.image.height()), 500.0);
        // The rows nearer than the plant are only the bottom of the window.
        let nearer = field(Part::Near);
        assert!(nearer.top > ground + 100.0);
    }

    /// With GROVE_LAYER_SHOTS naming a folder, saves the field in several
    /// states there, to be compared with what another build draws.
    #[test]
    fn frames_for_comparison() {
        let Some(folder) = std::env::var_os("GROVE_LAYER_SHOTS").map(std::path::PathBuf::from) else {
            return;
        };
        let save = |name: &str, look: Look| {
            let picture = draw(
                Box::new(move |painter, w, h| paint(painter, &look, w, h)),
                800.0,
                600.0,
                -1.0,
                1.0,
            );
            assert!(
                picture.unwrap().image.save(&folder.join(format!("field-{name}.png"))),
                "{name} not saved"
            );
        };
        save("far", look(Part::Far, 600.0));
        save("near", look(Part::Near, 600.0));
        save(
            "far-snow",
            Look {
                snow: 0.8,
                ..look(Part::Far, 600.0)
            },
        );
        save(
            "near-frost",
            Look {
                frost: true,
                plant_x: 0.6,
                ..look(Part::Near, 600.0)
            },
        );
    }
}
