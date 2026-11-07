#include "weather.h"

#include <algorithm>

namespace {

double smoothstep(double a, double b, double x) {
    const double t = std::max(0.0, std::min(1.0, (x - a) / (b - a)));
    return t * t * (3 - 2 * t);
}

bool known(const QString &kind) {
    return kind == "clear" || kind == "cloudy" || kind == "rain" || kind == "snow" || kind == "fog";
}

} // namespace

Weather::Weather(const QString &forcedKind, QObject *parent)
    : QObject(parent), m_forced(known(forcedKind) ? forcedKind : QString()) {
    begin(QRandomGenerator::global()->bounded(1, 1 << 30), QDate::currentDate());
}

Weather::Conditions Weather::conditions(const QString &kind) {
    if (kind == "cloudy")
        return {0.75, 0, 0, 0};
    if (kind == "rain")
        return {0.95, 0.8, 0, 0};
    if (kind == "snow")
        return {0.9, 0, 0, 0.85};
    if (kind == "fog")
        return {0.15, 0, 1.0, 0};
    return {0.0, 0, 0, 0};
}

QString Weather::seasonOf(const QDate &date) {
    // Northern hemisphere, by calendar month.
    switch (date.month()) {
    case 3: case 4: case 5: return "spring";
    case 6: case 7: case 8: return "summer";
    case 9: case 10: case 11: return "autumn";
    default: return "winter";
    }
}

QString Weather::pickKind(QRandomGenerator &random, const QString &season) {
    static const QStringList kinds{"clear", "cloudy", "rain", "fog", "snow"};
    // Weights for clear, cloudy, rain, fog and snow. Summer is mostly clear,
    // autumn is the foggy season, and only winter brings snow.
    static const double spring[5] = {3, 3, 3, 1, 0}, summer[5] = {6, 2, 0.7, 0.5, 0}, autumn[5] = {2, 3, 3, 2.5, 0},
                        winter[5] = {1, 3, 1.5, 2, 2.5};
    const double *w = season == "summer" ? summer : season == "autumn" ? autumn : season == "winter" ? winter : spring;
    double pick = random.generateDouble() * (w[0] + w[1] + w[2] + w[3] + w[4]);
    for (int i = 0; i < 5; ++i) {
        if (pick < w[i])
            return kinds[i];
        pick -= w[i];
    }
    return "clear";
}

void Weather::begin(int seed, const QDate &date) {
    QRandomGenerator random(static_cast<quint32>(seed));
    m_season = seasonOf(date);
    if (!m_forced.isEmpty()) {
        m_from = m_to = m_forced;
    } else {
        m_from = pickKind(random, m_season);
        // About half of all sessions change weather part-way through.
        m_to = random.generateDouble() < 0.5 ? m_from : pickKind(random, m_season);
    }
    m_progress = 0;
    m_boost = 0;
    refresh();
}

void Weather::setProgress(double progress) {
    progress = std::max(0.0, std::min(1.0, progress));
    // Rain makes the plant grow a little faster, never more than a sliver.
    const double gained = progress - m_progress;
    if (gained > 0)
        m_boost = std::min(0.15, m_boost + gained * m_now.rain * 0.3);
    m_progress = progress;
    refresh();
}

QString Weather::kind() const {
    if (m_now.fog >= 0.45)
        return "fog";
    if (m_now.snow >= 0.4)
        return "snow";
    if (m_now.rain >= 0.4)
        return "rain";
    if (m_now.cloud >= 0.5)
        return "cloudy";
    return "clear";
}

void Weather::refresh() {
    // The change happens in the middle of the session, not at a fixed moment.
    const double s = smoothstep(0.3, 0.7, m_progress);
    const Conditions a = conditions(m_from), b = conditions(m_to);
    m_now = {a.cloud + (b.cloud - a.cloud) * s, a.rain + (b.rain - a.rain) * s, a.fog + (b.fog - a.fog) * s,
             a.snow + (b.snow - a.snow) * s};
    emit changed();
}
