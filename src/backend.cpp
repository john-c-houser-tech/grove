#include "backend.h"

#include <QRandomGenerator>
#include <algorithm>

Backend::Backend(int seconds, QObject *parent) : QObject(parent), m_seconds(std::max(1, seconds)) {
    m_timer.setInterval(50);
    connect(&m_timer, &QTimer::timeout, this, [this] {
        // Measure real time rather than counting ticks, which drift under load.
        advance(m_clock.restart());
    });
}

int Backend::remaining() const {
    const qint64 left = static_cast<qint64>(m_seconds) * 1000 - m_elapsed;
    return static_cast<int>((std::max<qint64>(0, left) + 999) / 1000);
}

void Backend::setState(const QString &state) {
    if (state == m_state)
        return;
    m_state = state;
    emit stateChanged();
}

void Backend::start() {
    if (m_state == "growing")
        return;
    m_elapsed = 0;
    m_progress = 0;
    // A fresh seed per session, so no two plants are alike.
    m_seed = QRandomGenerator::global()->bounded(1, kSeedLimit);
    m_shownRemaining = remaining();
    m_shownProgress = 0;
    setState("growing");
    emit progressChanged();
    m_clock.start();
    m_timer.start();
}

void Backend::giveUp() {
    if (m_state != "growing")
        return;
    m_timer.stop();
    setState("wilted");
}

void Backend::advance(qint64 milliseconds) {
    if (m_state != "growing")
        return;
    m_elapsed += std::max<qint64>(0, milliseconds);
    const qint64 total = static_cast<qint64>(m_seconds) * 1000;
    m_progress = std::min(1.0, static_cast<double>(m_elapsed) / static_cast<double>(total));
    // A tick grows the plant by a thirty-thousandth of a full session, far too
    // little to see, and every listener redraws for it. So the window hears
    // only when the countdown moves on a second or the plant has grown enough
    // to show, which in a short test session is still many times a second.
    const int left = remaining();
    if (left != m_shownRemaining || m_progress - m_shownProgress >= 0.002 || m_elapsed >= total) {
        m_shownRemaining = left;
        m_shownProgress = m_progress;
        emit progressChanged();
    }
    if (m_elapsed >= total) {
        m_timer.stop();
        setState("done");
    }
}
