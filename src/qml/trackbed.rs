//! The valley floor with the railway along its far edge, the level
//! crossing's dirt road, and the nearer hill the road goes behind. The hills
//! in front cover the floor's lower part; the train runs between this and
//! them.

use std::f64::consts::PI;

use cxx_qt_lib::QColor;

use super::paint::color::{darker, mix, rgba};
use super::paint::{Gradient, Painter, Path};
use crate::terrain::mid_ridge;

/// Everything `paint` draws from, copied whole to the drawing thread.
pub struct Look {
    /// Where the horizon sits, as a fraction of the height.
    pub horizon: f64,
    /// Where the crossing stands, as a fraction of the width.
    pub crossing_x: f64,
    pub floor_color: QColor,
    pub ballast_color: QColor,
    pub rail_color: QColor,
    pub road_color: QColor,
    /// Beyond the crossing: the bare earth of the cut slope above the road,
    /// the tipped earth of the fill slope below it, and the hillside itself.
    pub cut_color: QColor,
    pub fill_color: QColor,
    pub hill_color: QColor,
    pub haze_color: QColor,
}

/// A point on the screen.
type Point = (f64, f64);

/// The track is straight and level across the valley, this far down.
pub fn track_y(horizon: f64, height: f64) -> f64 {
    height * (horizon - 0.085)
}

/// The point at `t` of the way along a curve from `p0` to `p3`, bent toward
/// `p1` and `p2`.
fn curve(p0: Point, p1: Point, p2: Point, p3: Point, t: f64) -> Point {
    let u = 1.0 - t;
    let along =
        |a: f64, b: f64, c: f64, d: f64| u * u * u * a + 3.0 * u * u * t * b + 3.0 * u * t * t * c + t * t * t * d;
    (along(p0.0, p1.0, p2.0, p3.0), along(p0.1, p1.1, p2.1, p3.1))
}

/// A road along a curve, seen from low down: as wide as the crossing where
/// it runs straight toward or away from us, thin where it turns to run
/// across the view, and narrower with distance.
struct Road {
    upper: Vec<Point>,
    lower: Vec<Point>,
}

impl Road {
    /// The road along the curve from `p0` to `p3`, narrowing by `farther` of
    /// its width by the far end.
    fn along(p0: Point, p1: Point, p2: Point, p3: Point, farther: f64, height: f64) -> Self {
        let (mut upper, mut lower) = (Vec::new(), Vec::new());
        for i in 0..=40 {
            let t = f64::from(i) / 40.0;
            let at = curve(p0, p1, p2, p3, t);
            let next = curve(p0, p1, p2, p3, (t + 0.01).min(1.0));
            let prev = curve(p0, p1, p2, p3, (t - 0.01).max(0.0));
            let (dx, dy) = (next.0 - prev.0, next.1 - prev.1);
            let mut len = (dx * dx + dy * dy).sqrt();
            if len == 0.0 {
                len = 1.0;
            }
            let toward = dy.abs() / len;
            let shrink = 1.0 - farther * t;
            let half = height * (0.006 + 0.019 * toward) * shrink;
            let (nx, ny) = (-dy / len, dx / len);
            upper.push((
                at.0 + nx * half,
                at.1 + ny * half * (1.0 - toward) - (1.0 - toward) * half * 0.2,
            ));
            lower.push((
                at.0 - nx * half,
                at.1 - ny * half * (1.0 - toward) + (1.0 - toward) * half * 0.2,
            ));
        }
        Self { upper, lower }
    }

    fn outline(&self) -> Path {
        let mut path = Path::new();
        path.move_to(self.upper[0].0, self.upper[0].1);
        for point in &self.upper[1..] {
            path.line_to(point.0, point.1);
        }
        for point in self.lower.iter().rev() {
            path.line_to(point.0, point.1);
        }
        path.close();
        path
    }
}

/// Draws the valley floor and the railway at `width` by `height` in the
/// window's units. It runs on the drawing thread, and tests call it
/// directly.
pub fn paint(painter: &mut Painter<'_>, look: &Look, width: f64, height: f64) {
    let (w, h) = (width, height);
    if w <= 0.0 || h <= 0.0 {
        return;
    }
    let ty = track_y(look.horizon, h);
    // The line of the track, from one side of the window to the other.
    let line = || {
        let mut path = Path::new();
        path.move_to(0.0, ty);
        let mut i = 1.0;
        while 4.0 * i <= w {
            path.line_to(4.0 * i, ty);
            i += 1.0;
        }
        path
    };
    {
        let mut floor = line();
        floor.line_to(w, h).line_to(0.0, h).close();
        painter.fill_path(&floor, &look.floor_color);
    }
    // A dirt road. On this side it comes a little toward us, then curves away
    // to the right along the valley floor and goes behind the hill there.
    // Beyond the rails it climbs the hillside to the left, where the hill is
    // higher, and goes into a cut in the slope. The signal stands on its
    // right-hand verge.
    let (post, cy) = (w * look.crossing_x, ty);
    let (gap, crossing) = (h * 0.006, h * 0.05);
    let centre = post - gap - crossing / 2.0;

    // Near side.
    let near_side = Road::along(
        (centre, cy),
        (centre, cy + h * 0.03),
        (centre + w * 0.1, cy + h * 0.034),
        (w * 1.08, cy + h * 0.028),
        0.0,
        h,
    );
    painter.fill_path(&near_side.outline(), &look.road_color);
    // Far side: the road is cut into the hillside as it climbs, like an
    // inslope road. It leaves the rails at full width, bends left and climbs
    // across the face of the shoulder of hill, with a cut slope of bare earth
    // above it and a fill slope below, then curves around the shoulder and
    // out of sight.
    let (far0, far1) = ((centre, cy), (centre - w * 0.006, cy - h * 0.02));
    let (far2, far3) = ((centre - w * 0.06, cy - h * 0.03), (centre - w * 0.13, cy - h * 0.056));
    let edges = Road::along(far0, far1, far2, far3, 0.7, h);
    let n = edges.upper.len();
    let along = |i: usize| i as f64 / (n - 1) as f64;
    // The fill slope: earth tipped down the hill below the road, deepest
    // where the road is farthest out on the slope.
    {
        let fill_from = (n as f64 * 0.2).floor() as usize;
        let mut fill = Path::new();
        fill.move_to(edges.lower[fill_from].0, edges.lower[fill_from].1);
        for point in &edges.lower[fill_from + 1..] {
            fill.line_to(point.0, point.1);
        }
        for k in (fill_from..n).rev() {
            let tf = (along(k) - along(fill_from)) / (1.0 - along(fill_from));
            let drop = h * 0.012 * (PI * (tf * 1.1).min(1.0)).sin() * (1.0 - 0.5 * along(k));
            fill.line_to(edges.lower[k].0 + w * 0.004 * tf, edges.lower[k].1 + drop);
        }
        fill.close();
        painter.fill_path(&fill, &look.fill_color);
    }
    // The road itself.
    painter.fill_path(&edges.outline(), &look.road_color);
    // The cut slope: the bank left where the road was dug into the hill,
    // tallest partway up and gone where the road leaves the flat.
    let cut_from = (n as f64 * 0.25).floor() as usize;
    let cut_top: Vec<Point> = (cut_from..n)
        .map(|c| {
            let tc = (along(c) - along(cut_from)) / (1.0 - along(cut_from));
            let tall = h * (0.004 + 0.02 * (PI * 0.5 * (tc * 1.6).min(1.0)).sin()) * (1.0 - 0.35 * tc);
            (edges.upper[c].0 - w * 0.003 * tc, edges.upper[c].1 - tall)
        })
        .collect();
    {
        let mut cut = Path::new();
        cut.move_to(edges.upper[cut_from].0, edges.upper[cut_from].1);
        for point in &edges.upper[cut_from + 1..] {
            cut.line_to(point.0, point.1);
        }
        for point in cut_top.iter().rev() {
            cut.line_to(point.0, point.1);
        }
        cut.close();
        painter.fill_path(&cut, &look.cut_color);
    }
    // Rain has scored the cut with short runnels down its face.
    {
        let darkened = |channel: f32| f64::from(channel) * 0.82;
        let scored = rgba(
            darkened(look.cut_color.red_f()),
            darkened(look.cut_color.green_f()),
            darkened(look.cut_color.blue_f()),
            0.45,
        );
        let mut r = 3;
        while r + 2 < cut_top.len() {
            let (top, foot) = (cut_top[r], edges.upper[cut_from + r]);
            let mut runnel = Path::new();
            runnel
                .move_to(top.0, top.1 + (foot.1 - top.1) * 0.15)
                .line_to(top.0 + (foot.0 - top.0) * 0.7 + 1.0, top.1 + (foot.1 - top.1) * 0.75);
            painter.stroke_path(&runnel, &scored, 1.0);
            r += 5;
        }
    }
    // Beyond the near hill the road keeps climbing, and comes back into view
    // higher up and farther off: a thin line traversing the farther hill to
    // the saddle in its ridge, where it goes over the pass. It is drawn
    // before the near hill, so only the stretch above it shows.
    let saddle = (w * 0.25, h * look.horizon - mid_ridge(0.25, h) + h * 0.002);
    // It comes out from behind the near hill's top and rises steadily across
    // the farther hill to the pass.
    let bend = curve(far0, far1, far2, far3, 0.86);
    let hidden = (bend.0 - w * 0.12, h * look.horizon - h * 0.128);
    let mut upper = Path::new();
    upper.move_to(hidden.0, hidden.1).cubic_to(
        (hidden.0 - w * 0.06, hidden.1 - h * 0.006),
        (saddle.0 + w * 0.07, saddle.1 + h * 0.01),
        saddle.0,
        saddle.1,
    );
    // The sliver of cut above it, then the road.
    painter.save();
    painter.translate(0.0, -h * 0.003);
    painter.stroke_path(&upper, &darker(&look.cut_color, 1.1), (h * 0.0025).max(1.0));
    painter.restore();
    painter.stroke_path(
        &upper,
        &mix(&look.road_color, &look.haze_color, 0.2),
        (h * 0.0045).max(2.0),
    );

    // A nearer hill stands to the left of the cut. Its right flank comes
    // steeply down across the farther hill and the road's last stretch, so
    // the road passes out of sight behind it. Being nearer, it is darker and
    // less hazy than the hill behind.
    let y0 = h * look.horizon;
    // A modest rounded hill: it rises out of the valley on its left to a
    // gentle top, then comes down steeply across the road's last stretch.
    let crest = y0 - h * 0.155;
    let top_x = bend.0 - w * 0.11;
    let foot_left = bend.0 - w * 0.34;
    let near_top = darker(&look.hill_color, 1.12);
    let near_fill = Gradient::new(0.0, crest, 0.0, cy)
        .stop(0.0, &near_top)
        .stop(1.0, &mix(&near_top, &look.haze_color, 0.25));
    let near_hill = || {
        let mut hill = Path::new();
        hill.move_to(foot_left, cy + 1.0);
        // The long left slope up out of the valley.
        hill.cubic_to(
            (foot_left + w * 0.1, cy - h * 0.02),
            (top_x - w * 0.12, crest + h * 0.004),
            top_x,
            crest,
        );
        // Over the top and down the steep right flank to the road's end.
        hill.cubic_to(
            (top_x + w * 0.06, crest - h * 0.002),
            (bend.0 - w * 0.012, bend.1 - h * 0.045),
            bend.0 + w * 0.004,
            bend.1 + h * 0.008,
        );
        // Its foot reaches the valley floor just beyond the road.
        hill.quad_to(bend.0 + w * 0.012, cy - h * 0.006, bend.0 + w * 0.03, cy + 1.0);
        hill
    };
    let mut shape = near_hill();
    shape.close();
    painter.fill_path(&shape, &near_fill);
    // A faint line where the near hill's edge stands against the far one.
    painter.stroke_path(&near_hill(), &darker(&near_top, 1.15), (h * 0.0018).max(1.0));
    // Ballast, then a rail catching what light there is on top of it.
    painter.stroke_path(&line(), &look.ballast_color, (h * 0.004).max(2.0));
    painter.stroke_path(&line(), &look.rail_color, 1.0);
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::qml::layer::draw;

    /// The trackbed in its own default colours.
    pub fn look() -> Look {
        let color = |hex: &str| QColor::try_from(hex).unwrap();
        Look {
            horizon: 0.6,
            crossing_x: 0.68,
            floor_color: color("#33503f"),
            ballast_color: color("#26382d"),
            rail_color: color("#6d7f78"),
            road_color: color("#7a7258"),
            cut_color: color("#4a4232"),
            fill_color: color("#3d3a2c"),
            hill_color: color("#44605a"),
            haze_color: color("#9fb7c4"),
        }
    }

    #[test]
    fn the_track_runs_level_above_the_horizon() {
        assert_eq!(track_y(0.6, 1000.0), 515.0);
        let look = look();
        let picture = draw(
            Box::new(move |painter, w, h| paint(painter, &look, w, h)),
            400.0,
            500.0,
            -1.0,
            1.0,
        )
        .unwrap();
        // The rail is drawn right across, at the track's height.
        let rail = track_y(0.6, 500.0) as i32 - picture.top as i32;
        assert!(picture.image.pixel(5, rail).0 > 0 && picture.image.pixel(395, rail).0 > 0);
        // The near hill and the road over the pass stand above the track.
        assert!(picture.top < track_y(0.6, 500.0) - 20.0);
        assert_eq!(picture.top + f64::from(picture.image.height()), 500.0);
    }

    /// With GROVE_LAYER_SHOTS naming a folder, saves the trackbed there, to
    /// be compared with what another build draws.
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
                picture
                    .unwrap()
                    .image
                    .save(&folder.join(format!("trackbed-{name}.png"))),
                "{name} not saved"
            );
        };
        save("day", look());
        save(
            "left",
            Look {
                crossing_x: 0.4,
                horizon: 0.7,
                ..look()
            },
        );
    }
}
