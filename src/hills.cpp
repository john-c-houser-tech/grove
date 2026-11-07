#include "hills.h"

#include <QLinearGradient>
#include <QPainter>
#include <QPainterPath>
#include <array>
#include <functional>

#include "terrain.h"

using namespace terrain;

namespace {

// The far range, after the Tetons: no foothills, a wall of blunt, blocky
// granite summits rising straight from the valley on one broad mass that is
// highest in the middle, the tallest summit just left of centre. Each peak is
// its middle, its top (fraction of the height above the horizon), and how
// steeply it falls away on the left and on the right.
struct Peak {
    double at, top, left, right;
};
constexpr std::array<Peak, 8> kPeaks{{{0.0, 0.25, 1.3, 1.4},
                                      {0.17, 0.27, 1.4, 1.5},
                                      {0.32, 0.29, 1.5, 1.4},
                                      {0.465, 0.335, 1.6, 1.7},
                                      {0.585, 0.305, 1.5, 1.5},
                                      {0.71, 0.285, 1.4, 1.3},
                                      {0.85, 0.265, 1.3, 1.5},
                                      {1.0, 0.25, 1.4, 1.3}}};

// A rounded, blunt summit that steepens into straight flanks.
double peakHeight(const Peak &p, double u) {
    const double d = u - p.at, r = 0.03;
    return p.top - (d < 0 ? p.left : p.right) * (std::sqrt(d * d + r * r) - r);
}

// Height of each layer's ridge above the horizon, in pixels, at fraction u of
// the way across.
double far(double u, double H) {
    const double centre = (u - 0.5) / 0.32;
    double h = 0.19 + 0.05 * std::exp(-centre * centre);
    for (const Peak &p : kPeaks) {
        h = std::max(h, peakHeight(p, u));
        // Lesser tops on each flank: granite massifs are never one clean pyramid.
        h = std::max(h, peakHeight({p.at - 0.048, p.top - 0.038, 2.2, 2.4}, u));
        h = std::max(h, peakHeight({p.at + 0.052, p.top - 0.045, 2.4, 2.2}, u));
    }
    h += 0.004 * (ridged(u * 30 + 7, 9) - 0.45) + 0.002 * (noise(u * 80, 10) - 0.5);
    return H * h;
}

// Two hills, one at each edge, with a low meadow between them. The railway
// runs behind them, so a train comes out from behind one and goes in behind
// the other.
double near(double u, double H) {
    const double left = std::pow(1 - u, 3), right = std::pow(u, 3);
    return H * (0.008 + 0.12 * left + 0.11 * right) + H * 0.012 * noise(u * 6.5 + 2, 6);
}

// The outline of a ridge, closed along the bottom of the window.
QPainterPath skyline(const std::function<double(double, double)> &heightAt, double W, double H, double y0) {
    QPainterPath path;
    path.setFillRule(Qt::WindingFill);
    path.moveTo(0, H);
    for (int i = 0; 4.0 * i <= W; ++i) {
        const double x = 4.0 * i;
        path.lineTo(x, y0 - heightAt(x / W, H));
    }
    path.lineTo(W, H);
    path.closeSubpath();
    return path;
}

void paintRidge(QPainter &p, const std::function<double(double, double)> &heightAt, double top, const QColor &color,
                double mist, const Hills::Look &look, double W, double H) {
    const double y0 = H * look.horizon;
    // Mist gathers at the foot of each layer.
    QLinearGradient g(0, y0 - top, 0, y0 + 2);
    g.setColorAt(0, color);
    g.setColorAt(1, mix(color, look.hazeColor, mist));
    p.fillPath(skyline(heightAt, W, H, y0), g);
}

} // namespace

Hills::Hills(QQuickItem *parent) : PaintedLayer(parent) {
    connect(this, &Hills::changed, this, &Hills::scheduleRender);
}

PaintedLayer::Painting Hills::painting() const {
    const Look look{m_horizon,    m_part,        m_snowline,   m_farColor, m_midColor,  m_nearColor, m_hazeColor,
                    m_shadeColor, m_forestColor, m_shadowSide, m_relief,   m_snowCover, m_snowColor};
    return [look](QPainter &p, double W, double H) { paint(p, look, W, H); };
}

void Hills::paint(QPainter &p, const Look &look, double W, double H) {
    if (W <= 0 || H <= 0)
        return;
    const double y0 = H * look.horizon;
    if (look.part == "front") {
        paintRidge(p, near, H * 0.14, look.nearColor, 0.18, look, W, H);
        return;
    }

    // The far range: rock, hazier toward its foot.
    paintRidge(p, far, H * 0.34, look.farColor, 0.45, look, W, H);
    const double snowY = y0 - H * 0.34 * look.snowline;
    p.save();
    p.setClipPath(skyline(far, W, H, y0), Qt::IntersectClip);

    // Snow. In a snowy season it caps the range above a ragged snowline,
    // whitest on the summits where it is coldest.
    if (look.snowCover >= 0.5) {
        QPainterPath cap;
        cap.setFillRule(Qt::WindingFill);
        cap.moveTo(0, 0);
        cap.lineTo(W, 0);
        for (int i = 0; W - 4.0 * i >= 0; ++i) {
            const double sx = W - 4.0 * i;
            const double u = sx / W;
            // Each face holds its snow differently: some low, some high.
            const double wobble = H
                                  * (0.06 * (noise(u * 7, 7) - 0.5) + 0.03 * (noise(u * 23, 13) - 0.5)
                                     + 0.012 * (noise(u * 60, 8) - 0.5));
            cap.lineTo(sx, snowY + wobble);
        }
        cap.closeSubpath();
        QLinearGradient g(0, y0 - H * 0.33, 0, snowY + H * 0.02);
        g.setColorAt(0, look.snowColor);
        g.setColorAt(0.45, mix(look.snowColor, look.farColor, 0.2));
        g.setColorAt(1, mix(look.snowColor, look.farColor, 0.6));
        p.fillPath(cap, g);
    }

    // Each summit has a face toward the sun and a face away from it, divided
    // along a spur running down from the top. The shaded face runs from that
    // spur round to the next notch along the skyline.
    if (look.relief > 0.01) {
        const QColor shade = withAlpha(look.shadeColor, look.relief);
        const QColor rib = withAlpha(look.shadeColor, look.relief * 0.45);
        for (int pk = 0; pk < static_cast<int>(kPeaks.size()); ++pk) {
            const Peak &peak = kPeaks[pk];
            const int side = look.shadowSide;
            const double su = peak.at;
            // Along the skyline, in steps, to the low point before the next peak.
            const double step = side * 0.003;
            int steps = 0;
            for (double prevH = far(su, H); steps < 60; ++steps) {
                const double nh = far(su + step * (steps + 1), H);
                if (nh > prevH + 0.5 && steps > 4)
                    break;
                prevH = nh;
            }
            const double top = y0 - far(su, H);
            const double footU = su + side * (0.03 + 0.015 * noise(pk * 1.7, 14));
            QPainterPath face;
            face.setFillRule(Qt::WindingFill);
            face.moveTo(W * su, top);
            for (int k = 0; k <= steps; ++k)
                face.lineTo(W * (su + step * k), y0 - far(su + step * k, H));
            // From the notch down a diagonal to the foot of the spur, then back
            // up the spur, a little bowed, to the summit.
            face.lineTo(W * footU, y0 - H * 0.1);
            face.quadTo(W * (su + side * 0.006), top + H * 0.06, W * su, top);
            face.closeSubpath();
            p.fillPath(face, shade);
            // Buttresses break up the sunlit face: narrow ribs whose sides away
            // from the sun fall into shade.
            for (int rb = 0; rb < 2; ++rb) {
                const double ru = su - side * (0.022 + 0.026 * rb + 0.008 * noise(pk * 3.1 + rb, 19));
                const double rtop = y0 - far(ru, H) + H * 0.004;
                const double rwide = 0.008 + 0.006 * noise(pk + rb * 5, 20);
                QPainterPath path;
                path.setFillRule(Qt::WindingFill);
                path.moveTo(W * ru, rtop);
                path.lineTo(W * (ru + side * rwide), y0 - far(ru + side * rwide, H) + H * 0.006);
                // Ribs fan out from the summit as they come down.
                const double spread = 0.035 + 0.02 * rb;
                path.lineTo(W * (ru - side * spread + side * 0.01), y0 - H * 0.13);
                path.lineTo(W * (ru - side * spread), y0 - H * 0.13);
                path.closeSubpath();
                p.fillPath(path, rib);
            }
        }
    }

    // Forest along the foot of the range, its top edge ragged with trees.
    {
        QPainterPath forest;
        forest.setFillRule(Qt::WindingFill);
        forest.moveTo(0, H);
        for (int i = 0; 3.0 * i <= W; ++i) {
            const double fx = 3.0 * i;
            const double fu = fx / W;
            const double line = 0.14 + 0.022 * noise(fu * 9, 16) + 0.01 * noise(fu * 31, 17);
            const double tips = 0.006 * (hash(i, 18) - 0.5);
            forest.lineTo(fx, y0 - H * (line + tips));
        }
        forest.lineTo(W, H);
        forest.closeSubpath();
        p.fillPath(forest, look.forestColor);
    }

    // Whatever the season, snow lies in the gullies high on the faces, mostly
    // on the side out of the sun: thin, ragged streaks down the fall line,
    // with a few patches beside them. Short and few in summer, long and many
    // in winter.
    for (int pp = 0; pp < static_cast<int>(kPeaks.size()); ++pp) {
        const Peak &peak = kPeaks[pp];
        const int gullies = 1 + static_cast<int>(std::round(2 * look.snowCover));
        for (int n = 0; n < gullies + 1; ++n) {
            const double seed = pp * 31 + n;
            // The last one, if any, is on the sunlit face.
            const int pside = n < gullies ? look.shadowSide : -look.shadowSide;
            if (n == gullies && hash(seed, 26) > 0.5 + look.snowCover * 0.5)
                continue;
            const double pu = peak.at + pside * (0.008 + 0.035 * hash(seed, 22));
            const double ridgeAt = far(pu, H) / H;
            if (ridgeAt < 0.22)
                continue;
            const bool onShade = pside == look.shadowSide;
            const QColor white = withAlpha(mix(look.snowColor, look.shadeColor, onShade ? 0.3 * look.relief : 0), 0.85);
            const double gx = W * pu;
            const double gy = y0 - H * ridgeAt + H * 0.006;
            const double len = H * (0.018 + 0.03 * hash(seed, 24)) * (0.5 + look.snowCover);
            const double drift = pside * len * 0.35;
            // A ragged streak: its edges wander as it runs down.
            const int steps = 6;
            QPainterPath streak;
            streak.setFillRule(Qt::WindingFill);
            streak.moveTo(gx, gy);
            for (int st = 1; st <= steps; ++st) {
                const double t = static_cast<double>(st) / steps;
                const double wide = H * 0.0035 * (1 - t * 0.7) * (0.6 + 0.8 * hash(seed, 50 + st));
                streak.lineTo(gx + drift * t + wide, gy + len * t);
            }
            for (int sb = steps; sb >= 1; --sb) {
                const double tb = static_cast<double>(sb) / steps;
                const double wb = H * 0.0035 * (1 - tb * 0.7) * (0.6 + 0.8 * hash(seed, 60 + sb));
                streak.lineTo(gx + drift * tb - wb, gy + len * tb);
            }
            streak.closeSubpath();
            p.fillPath(streak, white);
            // Patches lying beside the gully.
            for (int pb = 0; pb < 2; ++pb) {
                const double bt = 0.2 + 0.5 * hash(seed, 70 + pb);
                const double bx =
                    gx + drift * bt + pside * H * (0.006 + 0.006 * hash(seed, 72 + pb)) * (pb == 0 ? 1 : -1);
                const double by = gy + len * bt;
                const double bsize = H * (0.002 + 0.003 * hash(seed, 74 + pb)) * (0.6 + look.snowCover);
                QPainterPath patch;
                patch.addEllipse(QRectF(bx - bsize, by - bsize * 0.6, bsize * 2, bsize * 1.2));
                p.fillPath(patch, white);
            }
        }
    }
    p.restore();

    paintRidge(p, midRidge, H * 0.2, look.midColor, 0.4, look, W, H);
    if (look.part == "all")
        paintRidge(p, near, H * 0.14, look.nearColor, 0.18, look, W, H);
}
