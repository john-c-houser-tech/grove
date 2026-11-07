#include "field.h"

#include <QLinearGradient>
#include <QPainter>
#include <QPainterPath>
#include <algorithm>
#include <vector>

#include "terrain.h"

using namespace terrain;

namespace {

// The camera. A metre at the plant's distance is this many pixels, which with
// the plant 4 m off fixes the focal length. The eye is at standing height, so
// the horizon crosses the plant 1.6 m up its trunk.
constexpr double kMetreAtPlant = 0.21; // of the height
constexpr double kPlantDepth = 4;
constexpr double kEye = 1.6;
// The field ends this far away, where the meadow takes over.
constexpr double kFieldDepth = 40;

constexpr double kRowSpacing = 0.7;
constexpr double kRidgeHeight = 0.2;
// Ridges 60 cm across at the base, leaving a 10 cm trough between them.
constexpr double kRidgeHalfWidth = 0.3;

double focalFor(double H) { return H * kMetreAtPlant * kPlantDepth; }

} // namespace

Field::Field(QQuickItem *parent) : PaintedLayer(parent) {
    connect(this, &Field::changed, this, &Field::scheduleRender);
}

void Field::setGroundTop(double groundTop) {
    if (groundTop == m_groundTop)
        return;
    m_groundTop = groundTop;
    emit changed();
}

double Field::farEdgeY() const { return height() * m_horizon + focalFor(height()) * kEye / kFieldDepth; }
double Field::plantBaseY() const { return height() * m_horizon + focalFor(height()) * kEye / kPlantDepth; }

void Field::geometryChange(const QRectF &newGeometry, const QRectF &oldGeometry) {
    PaintedLayer::geometryChange(newGeometry, oldGeometry);
    // The far edge and the plant's foot move with the size.
    if (newGeometry.size() != oldGeometry.size())
        emit changed();
}

PaintedLayer::Painting Field::painting() const {
    const Look look{m_horizon, groundTop(), m_vanishX, m_plantX, m_part, m_soilColor, m_hazeColor, m_snow, m_frost};
    return [look](QPainter &p, double W, double H) { paint(p, look, W, H); };
}

void Field::paint(QPainter &p, const Look &look, double W, double H) {
    if (W <= 0 || H <= 0)
        return;
    const bool far = look.part == "far";
    const double focal = focalFor(H);
    // How far the plant stands to the side of straight ahead, in metres.
    const double plantLateral = (look.plantX - look.vanishX) * W / (H * kMetreAtPlant);
    // A point in the field, in metres (across, up, ahead), on the screen.
    const auto project = [&](double across, double up, double ahead) {
        return QPointF(W * look.vanishX + focal * across / ahead, H * look.horizon + focal * (kEye - up) / ahead);
    };

    const QColor snowColor = rgba(0.9, 0.93, 0.96);
    const double cover = std::max(look.snow * 0.85, look.frost ? 0.25 : 0.0);
    // The furrow floors are in shadow; the faces of the ridges that turn toward
    // the light are brighter, and snow settles on those first. The soil's own
    // colour runs from the dark of the troughs to the warm brown of a crest in
    // sun; snow lightens it all, the tops most.
    const QColor deep = darker(look.soilColor, 1.5);
    const QColor bright = lighter(look.soilColor, 2.7);
    const QColor floorColor = mix(mix(deep, bright, 0.14), snowColor, cover * 0.6);
    const auto tone = [&](double level, double height) {
        const QColor c = mix(deep, bright, std::clamp(level, 0.0, 1.0));
        return mix(c, snowColor, cover * (0.55 + 0.45 * height));
    };
    const QColor distantTone = mix(tone(0.55, 0.6), look.hazeColor, 0.06);
    p.save();
    p.setClipRect(QRectF(0, look.groundTop, W, H - look.groundTop), Qt::IntersectClip);
    // Far off, ridges and furrows blur into one average tone with a little
    // haze in it, so the floor lightens toward the far edge of the field.
    if (far) {
        QLinearGradient fade(0, look.groundTop, 0, project(0, 0, 10).y());
        fade.setColorAt(0, distantTone);
        fade.setColorAt(1, floorColor);
        p.fillRect(QRectF(0, look.groundTop, W, H - look.groundTop), fade);
    }

    // From just in front of the bottom of the window out to where the rows are
    // too close together to see, split at the plant. The rows start well in
    // front of the bottom of the window: a ridge stands up from the ground, so
    // its near end would show if they began where the flat ground leaves the
    // window.
    const double nearest = 2.4;
    const double from = far ? kPlantDepth : nearest;
    // The near rows run a hair past the plant so the two halves meet without a seam.
    const double to = far ? kFieldDepth : kPlantDepth * 1.01;
    const int steps = far ? 40 : 30;
    std::vector<double> depths;
    for (int i = 0; i <= steps; ++i)
        depths.push_back(from * std::pow(to / from, static_cast<double>(i) / steps));

    // The rows waver together in long, gentle curves, as a plough drawn by eye
    // does, each with a little unevenness of its own. The curve is measured
    // from the plant, so the plant stays in the middle of its trough.
    const auto wave = [](double d) { return 0.07 * std::sin(d * 0.35 + 1.3) + 0.025 * std::sin(d * 0.9 + 0.4); };
    const auto shift = [&](int j, double d) {
        return wave(d) - wave(kPlantDepth) + 0.012 * (noise(d * 0.6 + j * 7.7, 22) - 0.5);
    };
    const auto crestHeight = [](int j, double d) { return kRidgeHeight * (0.85 + 0.3 * noise(d * 0.9 + j * 13.1, 21)); };

    // Crests sit half a row either side of the plant's trough. Only the side of
    // each ridge that turns toward us can be seen, so the outer rows are drawn
    // first and the inner ones over them.
    struct Ridge {
        int j;
        double c;
    };
    std::vector<Ridge> ridges;
    for (int j = -36; j <= 36; ++j)
        ridges.push_back({j, plantLateral + kRowSpacing / 2 + j * kRowSpacing});
    std::stable_sort(ridges.begin(), ridges.end(),
                     [](const Ridge &a, const Ridge &b) { return std::abs(a.c) > std::abs(b.c); });

    // Light comes from the upper left. Across a rounded ridge the slope turns
    // from flat at the crest to steep at the foot, so the side facing the light
    // brightens down from the crest, the other side darkens, and both fall into
    // shadow in the trough.
    const int bands = 8;
    const auto profile = [](double t) { return 1 - t * t; };
    const auto brightness = [](double t, bool facingLight) {
        const double slope = 2 * kRidgeHeight * t / kRidgeHalfWidth;
        const double len = std::sqrt(1 + slope * slope);
        const double nx = (facingLight ? -slope : slope) / len, nz = 1 / len;
        const double lit = std::max(0.0, nx * -0.55 + nz * 0.83);
        return (0.25 + 0.75 * lit) * (1 - 0.45 * t * t);
    };

    // One clod: an irregular lump with a shadow under it, a body, and a lighter
    // top where the light catches it. Sizes are in metres.
    const auto clod = [&](double across, double up, double ahead, double size, double seed, const QColor &body,
                          const QColor &top, const QColor &shade) {
        const QPointF at = project(across, up, ahead);
        const double r = focal * size / ahead;
        if (r < 0.9 || at.x() < -r * 2 || at.x() > W + r * 2)
            return;
        const int corners = 6;
        const auto outline = [&](double cx, double cy, double scale) {
            QPainterPath path;
            path.setFillRule(Qt::WindingFill);
            for (int v = 0; v < corners; ++v) {
                const double angle = (v + 0.35 * hash(seed, v)) / corners * M_PI * 2;
                const double reach = r * scale * (0.65 + 0.5 * hash(seed, v + 9));
                const double x = cx + std::cos(angle) * reach, y = cy + std::sin(angle) * reach * 0.72;
                v == 0 ? path.moveTo(x, y) : path.lineTo(x, y);
            }
            path.closeSubpath();
            return path;
        };
        p.fillPath(outline(at.x() + r * 0.18, at.y() + r * 0.32, 1.0), shade);
        p.fillPath(outline(at.x(), at.y(), 1.0), body);
        if (r >= 2.5)
            p.fillPath(outline(at.x() - r * 0.18, at.y() - r * 0.24, 0.55), top);
    };

    // Clods and grain are drawn out to where they would be under a pixel.
    const double clodLimit = std::min(to, 14.0);
    const int segments = 8;
    const int last = static_cast<int>(depths.size()) - 1;

    for (const Ridge &ridge : ridges) {
        const int j0 = ridge.j;
        const double c = ridge.c;
        const int side = c > 0 ? -1 : 1;
        const bool facingLight = c > 0;
        // From standing height you look down into the rows, so the far side of a
        // ridge shows too, down to where it is steeper than your line of sight
        // across it. Looking straight along a row you see all of it; rows off to
        // the side show only their tops.
        const double across = std::max(0.05, std::abs(c));
        const double backReach = std::min(1.0, (kEye / across) * kRidgeHalfWidth / (2 * kRidgeHeight));
        const auto strip = [&](int sideSign, double t0, double t1, const QColor &fill, int lo, int hi) {
            QPainterPath path;
            path.setFillRule(Qt::WindingFill);
            for (int k = lo; k <= hi; ++k) {
                const double dk = depths[k], ch = crestHeight(j0, dk);
                const QPointF pt = project(c + shift(j0, dk) + sideSign * kRidgeHalfWidth * t0, ch * profile(t0), dk);
                k == lo ? path.moveTo(pt) : path.lineTo(pt);
            }
            for (int q = hi; q >= lo; --q) {
                const double dq = depths[q], cq = crestHeight(j0, dq);
                path.lineTo(project(c + shift(j0, dq) + sideSign * kRidgeHalfWidth * t1, cq * profile(t1), dq));
            }
            path.closeSubpath();
            p.fillPath(path, fill);
        };
        // A ridge runs straight in depth, so if both ends of a stretch are off
        // the same side of the window, all of it is.
        const auto offscreen = [&](int lo, int hi) {
            const double a = project(c, 0, depths[lo]).x(), b = project(c, 0, depths[hi]).x();
            const double margin = focal * kRidgeHalfWidth * 1.5 / depths[lo];
            return (a < -margin && b < -margin) || (a > W + margin && b > W + margin);
        };
        for (int gb = 0; gb < segments; ++gb) {
            const int blo = gb * last / segments;
            const int bhi = (gb + 1) * last / segments;
            if (offscreen(blo, bhi))
                continue;
            const double bmid = depths[(blo + bhi) / 2];
            const double bhaze = std::min(1.0, std::max(0.0, (bmid - 8) / 30));
            // As many shading bands as the ridge is wide on screen allows.
            const double bwide = focal * kRidgeHalfWidth * backReach / bmid;
            if (bwide < 1)
                continue;
            const int bn = std::max(1, std::min(bands, static_cast<int>(std::round(bwide / 4))));
            for (int bb = bn - 1; bb >= 0; --bb) {
                const double b0 = static_cast<double>(bb) / bn * backReach, b1 = static_cast<double>(bb + 1) / bn * backReach;
                const double bm = (b0 + b1) / 2;
                strip(-side, b0, b1, mix(tone(brightness(bm, !facingLight) * 1.15, profile(bm)), distantTone, bhaze), blo, bhi);
            }
        }

        // The ridge in bands from crest to foot, and in stretches from far to
        // near so the far ones can take on the soil's distant tone.
        for (int g = 0; g < segments; ++g) {
            const int lo = g * last / segments;
            const int hi = (g + 1) * last / segments;
            if (offscreen(lo, hi))
                continue;
            const double mid = depths[(lo + hi) / 2];
            const double haze = std::min(1.0, std::max(0.0, (mid - 8) / 30));
            const double wide = focal * kRidgeHalfWidth / mid;
            const int fn = std::max(2, std::min(bands, static_cast<int>(std::round(wide / 4))));
            for (int band = 0; band < fn; ++band) {
                const double t0 = static_cast<double>(band) / fn, t1 = static_cast<double>(band + 1) / fn;
                const double tm = (t0 + t1) / 2;
                strip(side, t0, t1, mix(tone(brightness(tm, facingLight) * 1.15, profile(tm)), distantTone, haze), lo, hi);
            }
        }

        // Fine grain over the near part of the ridge: specks of soil a
        // centimetre or two across, a little lighter or darker than the ground
        // under them.
        int count = 0;
        for (double gd = std::max(from, 0.8); gd < std::min(clodLimit, 9.0) && count < 120; ++count) {
            const double gseed = j0 * 977 + count;
            const double gt = std::sqrt(hash(gseed, 11));
            const double ghere = crestHeight(j0, gd);
            const QPointF gp = project(c + shift(j0, gd) + side * kRidgeHalfWidth * gt, ghere * profile(gt), gd);
            const double speck = std::max(1.0, focal * (0.008 + 0.012 * hash(gseed, 12)) / gd);
            if (gp.x() > -4 && gp.x() < W + 4) {
                const QColor base = tone(brightness(gt, facingLight) * 1.15, profile(gt));
                p.fillRect(QRectF(gp.x(), gp.y(), speck, speck * 0.7),
                           hash(gseed, 13) < 0.5 ? lighter(base, 1.25) : darker(base, 1.3));
            }
            gd += 0.025 + 0.06 * hash(gseed, 14) * gd / 3;
        }

        // A few small clods close by, high on the ridge.
        int cn = 0;
        for (double cd = std::max(from, 0.9); cd < std::min(to, 6.0); ++cn) {
            const double cseed = j0 * 1000 + cn;
            const double ct = 0.15 + 0.6 * hash(cseed, 2);
            const double chh = crestHeight(j0, cd);
            const double csize = 0.015 + 0.025 * hash(cseed, 3);
            const QColor cbody = tone(brightness(ct, facingLight) * 1.05, profile(ct));
            clod(c + shift(j0, cd) + side * kRidgeHalfWidth * ct, chh * profile(ct), cd, csize, cseed, cbody,
                 lighter(cbody, 1.25), darker(cbody, 1.5));
            cd += 0.35 + 0.6 * hash(cseed, 1);
        }
    }
    p.restore();
}
