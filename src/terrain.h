#pragma once

#include <QColor>
#include <QPen>
#include <algorithm>
#include <cmath>

// What the landscape layers share: fixed noise, so the land is always the same
// land, colours mixed and darkened the way QML does it, and the farther
// ridge's skyline, which the railway's road climbs to.
namespace terrain {

// A fixed random number from 0 to 1 for any pair of numbers.
inline double hash(double i, double seed) {
    const double x = std::sin(i * 127.1 + seed * 311.7) * 43758.5453;
    return x - std::floor(x);
}

// Smooth one-dimensional value noise, from 0 to 1.
inline double noise(double x, double seed) {
    const double i = std::floor(x), f = x - i;
    const double u = f * f * (3 - 2 * f);
    return hash(i, seed) * (1 - u) + hash(i + 1, seed) * u;
}

// Ridged noise has sharp peaks and broad valleys, like a mountain range.
inline double ridged(double x, double seed) {
    const double v = 1 - std::abs(2 * noise(x, seed) - 1);
    return v * v;
}

// A colour from components, clamped as Qt.rgba() clamps them.
inline QColor rgba(double r, double g, double b, double a = 1) {
    const auto unit = [](double v) { return static_cast<float>(std::clamp(v, 0.0, 1.0)); };
    return QColor::fromRgbF(unit(r), unit(g), unit(b), unit(a));
}

// From a toward b by t, opaque.
inline QColor mix(const QColor &a, const QColor &b, double t) {
    return rgba(a.redF() + (b.redF() - a.redF()) * t, a.greenF() + (b.greenF() - a.greenF()) * t,
                a.blueF() + (b.blueF() - a.blueF()) * t);
}

inline QColor withAlpha(const QColor &c, double alpha) { return rgba(c.redF(), c.greenF(), c.blueF(), alpha); }

// Qt.darker() and Qt.lighter().
inline QColor darker(const QColor &c, double factor) { return c.darker(static_cast<int>(std::round(factor * 100))); }
inline QColor lighter(const QColor &c, double factor) { return c.lighter(static_cast<int>(std::round(factor * 100))); }

// A pen as a canvas strokes: flat ends and mitred corners.
inline QPen pen(const QColor &color, double width) {
    QPen p(color, width, Qt::SolidLine, Qt::FlatCap, Qt::MiterJoin);
    p.setMiterLimit(10);
    return p;
}

// The farther ridge's height above the horizon, in pixels, at fraction u of
// the way across. A shoulder of hill rises behind the level crossing, so the
// road beyond the rails has a slope to climb into, and a saddle dips in the
// ridge to the left of it: the pass the road goes over.
inline double midRidge(double u, double H) {
    const double shoulder = (u - 0.6) / 0.12;
    const double pass = (u - 0.25) / 0.04;
    const double west = (u - 0.14) / 0.07;
    return H * (0.035 + 0.15 * noise(u * 2.6 + 5, 4)) + H * 0.025 * noise(u * 8.2, 5)
           + H * 0.07 * std::exp(-shoulder * shoulder) - H * 0.04 * std::exp(-pass * pass)
           + H * 0.03 * std::exp(-west * west);
}

} // namespace terrain
