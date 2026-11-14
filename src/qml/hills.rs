//! The view beyond the garden: distant granite peaks, rolling ridges and a
//! nearer hillside. The ridgelines come from fixed noise, so the landscape
//! is always the same one; only its light changes. Each layer is hazier the
//! farther away it is, fading toward the colour of the sky at the horizon.

use cxx_qt_lib::QColor;

use super::paint::color::{mix, with_alpha};
use super::paint::{Gradient, Painter, Path};
use crate::terrain::{hash, mid_ridge, noise, ridged};

/// Which of the hills a layer draws.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Part {
    /// The peaks and ridges, and the hillside nearest to us.
    All,
    /// The peaks and ridges only, so something can pass in front of them.
    Back,
    /// The hillside nearest to us only.
    Front,
}

impl Part {
    /// The part QML names "all", "back" or "front"; anything else is the
    /// back.
    pub fn named(name: &str) -> Self {
        match name {
            "all" => Part::All,
            "front" => Part::Front,
            _ => Part::Back,
        }
    }
}

/// Everything `paint` draws from, copied whole to the drawing thread.
pub struct Look {
    /// Where the horizon sits, as a fraction of the height.
    pub horizon: f64,
    pub part: Part,
    /// How low the snow reaches on the far peaks, as a fraction of their
    /// height.
    pub snowline: f64,
    pub far_color: QColor,
    pub mid_color: QColor,
    pub near_color: QColor,
    pub haze_color: QColor,
    /// Rock in shadow, and the forest along the range's foot.
    pub shade_color: QColor,
    pub forest_color: QColor,
    /// 1 when the right-hand faces of the peaks are shaded, -1 the left.
    pub shadow_side: i32,
    /// How strong the shadows are, from 0 to about 0.7.
    pub relief: f64,
    /// How much snow the range carries, from 0 to 1.
    pub snow_cover: f64,
    pub snow_color: QColor,
}

/// A summit of the far range: its middle, its top (fraction of the height
/// above the horizon), and how steeply it falls away on the left and on the
/// right.
struct Peak {
    at: f64,
    top: f64,
    left: f64,
    right: f64,
}

/// The far range, after the Tetons: no foothills, a wall of blunt, blocky
/// granite summits rising straight from the valley on one broad mass that is
/// highest in the middle, the tallest summit just left of centre.
const PEAKS: [Peak; 8] = [
    Peak {
        at: 0.0,
        top: 0.25,
        left: 1.3,
        right: 1.4,
    },
    Peak {
        at: 0.17,
        top: 0.27,
        left: 1.4,
        right: 1.5,
    },
    Peak {
        at: 0.32,
        top: 0.29,
        left: 1.5,
        right: 1.4,
    },
    Peak {
        at: 0.465,
        top: 0.335,
        left: 1.6,
        right: 1.7,
    },
    Peak {
        at: 0.585,
        top: 0.305,
        left: 1.5,
        right: 1.5,
    },
    Peak {
        at: 0.71,
        top: 0.285,
        left: 1.4,
        right: 1.3,
    },
    Peak {
        at: 0.85,
        top: 0.265,
        left: 1.3,
        right: 1.5,
    },
    Peak {
        at: 1.0,
        top: 0.25,
        left: 1.4,
        right: 1.3,
    },
];

/// A rounded, blunt summit that steepens into straight flanks.
fn peak_height(peak: &Peak, u: f64) -> f64 {
    let (d, r) = (u - peak.at, 0.03);
    peak.top - (if d < 0.0 { peak.left } else { peak.right }) * ((d * d + r * r).sqrt() - r)
}

/// Height of the far range above the horizon, in pixels, at fraction `u` of
/// the way across.
fn far(u: f64, height: f64) -> f64 {
    let centre = (u - 0.5) / 0.32;
    let mut h: f64 = 0.19 + 0.05 * (-centre * centre).exp();
    for peak in &PEAKS {
        h = h.max(peak_height(peak, u));
        // Lesser tops on each flank: granite massifs are never one clean pyramid.
        h = h.max(peak_height(
            &Peak {
                at: peak.at - 0.048,
                top: peak.top - 0.038,
                left: 2.2,
                right: 2.4,
            },
            u,
        ));
        h = h.max(peak_height(
            &Peak {
                at: peak.at + 0.052,
                top: peak.top - 0.045,
                left: 2.4,
                right: 2.2,
            },
            u,
        ));
    }
    h += 0.004 * (ridged(u * 30.0 + 7.0, 9.0) - 0.45) + 0.002 * (noise(u * 80.0, 10.0) - 0.5);
    height * h
}

/// Two hills, one at each edge, with a low meadow between them. The railway
/// runs behind them, so a train comes out from behind one and goes in behind
/// the other.
fn near(u: f64, height: f64) -> f64 {
    let (left, right) = ((1.0 - u).powi(3), u.powi(3));
    height * (0.008 + 0.12 * left + 0.11 * right) + height * 0.012 * noise(u * 6.5 + 2.0, 6.0)
}

/// The outline of a ridge, closed along the bottom of the window.
fn skyline(height_at: fn(f64, f64) -> f64, width: f64, height: f64, y0: f64) -> Path {
    let mut path = Path::new();
    path.move_to(0.0, height);
    let mut i = 0.0;
    while 4.0 * i <= width {
        let x = 4.0 * i;
        path.line_to(x, y0 - height_at(x / width, height));
        i += 1.0;
    }
    path.line_to(width, height).close();
    path
}

/// One ridge of the view.
struct Ridge<'look> {
    /// Its height above the horizon, in pixels, at a fraction of the way
    /// across a window of a height.
    height_at: fn(f64, f64) -> f64,
    /// Its tallest, as a fraction of the window's height.
    top: f64,
    color: &'look QColor,
    /// How far its foot fades toward the haze, where mist gathers.
    mist: f64,
}

impl Ridge<'_> {
    fn paint(&self, painter: &mut Painter<'_>, look: &Look, width: f64, height: f64) {
        let y0 = height * look.horizon;
        let gradient = Gradient::new(0.0, y0 - height * self.top, 0.0, y0 + 2.0)
            .stop(0.0, self.color)
            .stop(1.0, &mix(self.color, &look.haze_color, self.mist));
        painter.fill_path(&skyline(self.height_at, width, height, y0), &gradient);
    }
}

/// Draws the hills at `width` by `height` in the window's units. It runs on
/// the drawing thread, and tests call it directly.
pub fn paint(painter: &mut Painter<'_>, look: &Look, width: f64, height: f64) {
    let (w, h) = (width, height);
    if w <= 0.0 || h <= 0.0 {
        return;
    }
    let y0 = h * look.horizon;
    if look.part == Part::Front {
        Ridge {
            height_at: near,
            top: 0.14,
            color: &look.near_color,
            mist: 0.18,
        }
        .paint(painter, look, w, h);
        return;
    }

    // The far range: rock, hazier toward its foot.
    Ridge {
        height_at: far,
        top: 0.34,
        color: &look.far_color,
        mist: 0.45,
    }
    .paint(painter, look, w, h);
    let snow_y = y0 - h * 0.34 * look.snowline;
    painter.save();
    painter.clip_to_path(&skyline(far, w, h, y0));

    // Snow. In a snowy season it caps the range above a ragged snowline,
    // whitest on the summits where it is coldest.
    if look.snow_cover >= 0.5 {
        let mut cap = Path::new();
        cap.move_to(0.0, 0.0).line_to(w, 0.0);
        let mut i = 0.0;
        while w - 4.0 * i >= 0.0 {
            let sx = w - 4.0 * i;
            let u = sx / w;
            // Each face holds its snow differently: some low, some high.
            let wobble = h
                * (0.06 * (noise(u * 7.0, 7.0) - 0.5)
                    + 0.03 * (noise(u * 23.0, 13.0) - 0.5)
                    + 0.012 * (noise(u * 60.0, 8.0) - 0.5));
            cap.line_to(sx, snow_y + wobble);
            i += 1.0;
        }
        cap.close();
        let gradient = Gradient::new(0.0, y0 - h * 0.33, 0.0, snow_y + h * 0.02)
            .stop(0.0, &look.snow_color)
            .stop(0.45, &mix(&look.snow_color, &look.far_color, 0.2))
            .stop(1.0, &mix(&look.snow_color, &look.far_color, 0.6));
        painter.fill_path(&cap, &gradient);
    }

    let side = f64::from(look.shadow_side);
    // Each summit has a face toward the sun and a face away from it, divided
    // along a spur running down from the top. The shaded face runs from that
    // spur round to the next notch along the skyline.
    if look.relief > 0.01 {
        let shade = with_alpha(&look.shade_color, look.relief);
        let rib = with_alpha(&look.shade_color, look.relief * 0.45);
        for (pk, peak) in PEAKS.iter().enumerate() {
            let pk = pk as f64;
            let su = peak.at;
            // Along the skyline, in steps, to the low point before the next peak.
            let step = side * 0.003;
            let mut steps = 0;
            let mut previous = far(su, h);
            while steps < 60 {
                let next = far(su + step * f64::from(steps + 1), h);
                if next > previous + 0.5 && steps > 4 {
                    break;
                }
                previous = next;
                steps += 1;
            }
            let top = y0 - far(su, h);
            let foot_u = su + side * (0.03 + 0.015 * noise(pk * 1.7, 14.0));
            let mut face = Path::new();
            face.move_to(w * su, top);
            for k in 0..=steps {
                let at = su + step * f64::from(k);
                face.line_to(w * at, y0 - far(at, h));
            }
            // From the notch down a diagonal to the foot of the spur, then
            // back up the spur, a little bowed, to the summit.
            face.line_to(w * foot_u, y0 - h * 0.1);
            face.quad_to(w * (su + side * 0.006), top + h * 0.06, w * su, top)
                .close();
            painter.fill_path(&face, &shade);
            // Buttresses break up the sunlit face: narrow ribs whose sides
            // away from the sun fall into shade.
            for rb in 0..2 {
                let rb = f64::from(rb);
                let ru = su - side * (0.022 + 0.026 * rb + 0.008 * noise(pk * 3.1 + rb, 19.0));
                let rib_top = y0 - far(ru, h) + h * 0.004;
                let wide = 0.008 + 0.006 * noise(pk + rb * 5.0, 20.0);
                // Ribs fan out from the summit as they come down.
                let spread = 0.035 + 0.02 * rb;
                let mut path = Path::new();
                path.move_to(w * ru, rib_top)
                    .line_to(w * (ru + side * wide), y0 - far(ru + side * wide, h) + h * 0.006)
                    .line_to(w * (ru - side * spread + side * 0.01), y0 - h * 0.13)
                    .line_to(w * (ru - side * spread), y0 - h * 0.13)
                    .close();
                painter.fill_path(&path, &rib);
            }
        }
    }

    // Forest along the foot of the range, its top edge ragged with trees.
    {
        let mut forest = Path::new();
        forest.move_to(0.0, h);
        let mut i = 0.0;
        while 3.0 * i <= w {
            let fx = 3.0 * i;
            let fu = fx / w;
            let line = 0.14 + 0.022 * noise(fu * 9.0, 16.0) + 0.01 * noise(fu * 31.0, 17.0);
            let tips = 0.006 * (hash(i, 18.0) - 0.5);
            forest.line_to(fx, y0 - h * (line + tips));
            i += 1.0;
        }
        forest.line_to(w, h).close();
        painter.fill_path(&forest, &look.forest_color);
    }

    // Whatever the season, snow lies in the gullies high on the faces, mostly
    // on the side out of the sun: thin, ragged streaks down the fall line,
    // with a few patches beside them. Short and few in summer, long and many
    // in winter.
    for (pp, peak) in PEAKS.iter().enumerate() {
        let gullies = 1 + (2.0 * look.snow_cover).round() as i32;
        for n in 0..=gullies {
            let seed = (pp as i32 * 31 + n) as f64;
            // The last one, if any, is on the sunlit face.
            let pside = if n < gullies { side } else { -side };
            if n == gullies && hash(seed, 26.0) > 0.5 + look.snow_cover * 0.5 {
                continue;
            }
            let pu = peak.at + pside * (0.008 + 0.035 * hash(seed, 22.0));
            let ridge_at = far(pu, h) / h;
            if ridge_at < 0.22 {
                continue;
            }
            let on_shade = pside == side;
            let white = with_alpha(
                &mix(
                    &look.snow_color,
                    &look.shade_color,
                    if on_shade { 0.3 * look.relief } else { 0.0 },
                ),
                0.85,
            );
            let gx = w * pu;
            let gy = y0 - h * ridge_at + h * 0.006;
            let len = h * (0.018 + 0.03 * hash(seed, 24.0)) * (0.5 + look.snow_cover);
            let drift = pside * len * 0.35;
            // A ragged streak: its edges wander as it runs down.
            let steps = 6;
            let wide_at = |t: f64, salt: f64| h * 0.0035 * (1.0 - t * 0.7) * (0.6 + 0.8 * hash(seed, salt));
            let mut streak = Path::new();
            streak.move_to(gx, gy);
            for st in 1..=steps {
                let t = f64::from(st) / f64::from(steps);
                streak.line_to(gx + drift * t + wide_at(t, f64::from(50 + st)), gy + len * t);
            }
            for sb in (1..=steps).rev() {
                let t = f64::from(sb) / f64::from(steps);
                streak.line_to(gx + drift * t - wide_at(t, f64::from(60 + sb)), gy + len * t);
            }
            streak.close();
            painter.fill_path(&streak, &white);
            // Patches lying beside the gully.
            for pb in 0..2 {
                let salt = f64::from(pb);
                let bt = 0.2 + 0.5 * hash(seed, 70.0 + salt);
                let bx = gx
                    + drift * bt
                    + pside * h * (0.006 + 0.006 * hash(seed, 72.0 + salt)) * (if pb == 0 { 1.0 } else { -1.0 });
                let by = gy + len * bt;
                let size = h * (0.002 + 0.003 * hash(seed, 74.0 + salt)) * (0.6 + look.snow_cover);
                let mut patch = Path::new();
                patch.add_ellipse(bx - size, by - size * 0.6, size * 2.0, size * 1.2);
                painter.fill_path(&patch, &white);
            }
        }
    }
    painter.restore();

    Ridge {
        height_at: mid_ridge,
        top: 0.2,
        color: &look.mid_color,
        mist: 0.4,
    }
    .paint(painter, look, w, h);
    if look.part == Part::All {
        Ridge {
            height_at: near,
            top: 0.14,
            color: &look.near_color,
            mist: 0.18,
        }
        .paint(painter, look, w, h);
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::qml::layer::draw;

    /// The look the C++ tests drew the hills with.
    pub fn look(part: Part) -> Look {
        let color = |hex: &str| QColor::try_from(hex).unwrap();
        Look {
            horizon: 0.6,
            part,
            snowline: 0.6,
            far_color: color("#6b7f9c"),
            mid_color: color("#46605a"),
            near_color: color("#30493b"),
            haze_color: color("#9fb7c4"),
            shade_color: color("#4d5a6c"),
            forest_color: color("#2f3e35"),
            shadow_side: 1,
            relief: 0.5,
            snow_cover: 0.3,
            snow_color: color("#f2f6fa"),
        }
    }

    #[test]
    fn the_hills_keep_to_their_own_band() {
        let ground = 500.0 * 0.6 + 500.0 * 0.21 * 4.0 * 1.6 / 40.0;
        let hills = |part| {
            let look = look(part);
            draw(
                Box::new(move |painter, w, h| paint(painter, &look, w, h)),
                400.0,
                500.0,
                ground,
                1.0,
            )
            .unwrap()
        };
        // The hills stop where the ground covers them, and the near hills
        // are lower than the far range.
        let (back, front) = (hills(Part::Back), hills(Part::Front));
        assert_eq!(back.top + f64::from(back.image.height()), ground.ceil());
        assert!(back.top > 500.0 * 0.6 - 500.0 * 0.36);
        assert!(front.top > back.top + 500.0 * 0.15);
    }

    /// Not a test of the hills by itself: with GROVE_LAYER_SHOTS naming a
    /// folder, saves the hills in several lights there, to be compared with
    /// what another build draws.
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
                picture.unwrap().image.save(&folder.join(format!("hills-{name}.png"))),
                "{name} not saved"
            );
        };
        save("back", look(Part::Back));
        save("front", look(Part::Front));
        save("all", look(Part::All));
        save(
            "winter",
            Look {
                snow_cover: 1.0,
                snowline: 0.4,
                relief: 0.7,
                shadow_side: -1,
                ..look(Part::All)
            },
        );
        save(
            "flat",
            Look {
                snow_cover: 0.0,
                relief: 0.0,
                ..look(Part::Back)
            },
        );
    }

    #[test]
    fn parts_are_known_by_name() {
        assert_eq!(Part::named("all"), Part::All);
        assert_eq!(Part::named("front"), Part::Front);
        assert_eq!(Part::named("back"), Part::Back);
    }
}
