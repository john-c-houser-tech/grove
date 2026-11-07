#include "light.h"

#include <QtMath>
#include <algorithm>
#include <utility>
#include <vector>

#include "easing.h"

namespace {

QColor mix(const QColor &a, const QColor &b, double t) {
    return QColor::fromRgbF(a.redF() + (b.redF() - a.redF()) * t, a.greenF() + (b.greenF() - a.greenF()) * t,
                            a.blueF() + (b.blueF() - a.blueF()) * t);
}

struct Stop {
    double elevation;
    QColor top, bottom;
};

// Sky colours by the sun's height, from deep night to high noon.
QColor along(const std::vector<Stop> &stops, double elevation, bool top) {
    if (elevation <= stops.front().elevation)
        return top ? stops.front().top : stops.front().bottom;
    for (size_t i = 1; i < stops.size(); ++i) {
        if (elevation <= stops[i].elevation) {
            const double t = (elevation - stops[i - 1].elevation) / (stops[i].elevation - stops[i - 1].elevation);
            return top ? mix(stops[i - 1].top, stops[i].top, t) : mix(stops[i - 1].bottom, stops[i].bottom, t);
        }
    }
    return top ? stops.back().top : stops.back().bottom;
}

} // namespace

void Light::sunTimes(const QDate &date, double &sunrise, double &sunset) {
    // Day length swings about 4.3 to 15.7 hours across the year, longest at
    // the June solstice (day 172); solar noon sits near 12:30 on the clock.
    const double season = qCos(2 * M_PI * (date.dayOfYear() - 172) / 365.0);
    const double half = 6.07 + 1.78 * season;
    sunrise = 12.5 - half;
    sunset = 12.5 + half;
}

Sky Light::sky(const QDateTime &when, const QColor &accent) {
    double sunrise = 0, sunset = 0;
    sunTimes(when.date(), sunrise, sunset);
    const QTime t = when.time();
    const double hour = t.hour() + t.minute() / 60.0 + t.second() / 3600.0;

    Sky s;
    if (hour >= sunrise && hour <= sunset) {
        const double u = (hour - sunrise) / (sunset - sunrise);
        s.sunUp = true;
        s.elevation = qSin(M_PI * u);
        s.sunX = u;
        s.moonElevation = -1;
    } else {
        // The sun sinks only a little below the horizon, enough to take the
        // sky through twilight and into darkness and back.
        const double sinceSunset = hour >= sunset ? hour - sunset : hour + 24 - sunset;
        const double v = sinceSunset / (24 - (sunset - sunrise));
        s.elevation = -0.4 * qSin(M_PI * v);
        s.moonX = v;
        s.moonElevation = qSin(M_PI * v);
    }

    // The horizon at golden hour is the warm colour, pulled toward the accent.
    const QColor warm = mix(QColor("#e8905a"), accent, 0.45);
    const std::vector<Stop> stops = {
        {-0.40, QColor("#060a14"), QColor("#0e1624")},
        {-0.15, QColor("#0b1226"), QColor("#1c1f3a")},
        {-0.02, QColor("#1d2550"), QColor("#6a4660")},
        {0.06, QColor("#33497a"), warm},
        {0.25, QColor("#4a78a8"), QColor("#9fc0cc")},
        {1.00, QColor("#3d6f9c"), QColor("#8fb4c8")},
    };
    s.top = along(stops, s.elevation, true);
    s.bottom = along(stops, s.elevation, false);
    s.ambient = 0.22 + 0.78 * smoothstep(-0.12, 0.3, s.elevation);
    s.stars = clamp01((-s.elevation - 0.02) / 0.2);
    const double d = (s.elevation - 0.04) / 0.12;
    s.warmth = qExp(-d * d);
    s.warmTint = warm;
    return s;
}

Light::Light(const QDateTime &fixed, QObject *parent) : QObject(parent), m_fixed(fixed) {
    // The sky moves slowly; twice a minute is smooth enough to never notice.
    m_timer.setInterval(30000);
    connect(&m_timer, &QTimer::timeout, this, &Light::update);
    if (!m_fixed.isValid())
        m_timer.start();
    update();
}

QColor Light::warmTint() const {
    QColor c = m_sky.warmTint;
    c.setAlphaF(clamp01(m_sky.warmth * 0.2));
    return c;
}

QColor Light::nightTint() const {
    return QColor::fromRgbF(0.02, 0.04, 0.12, (1 - m_sky.ambient) * 0.62);
}

void Light::setAccent(const QColor &accent) {
    if (accent == m_accent)
        return;
    m_accent = accent;
    update();
}

void Light::update() {
    m_sky = sky(m_fixed.isValid() ? m_fixed : QDateTime::currentDateTime(), m_accent);
    emit changed();
}
