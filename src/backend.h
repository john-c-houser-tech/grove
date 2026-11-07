#pragma once

#include <QElapsedTimer>
#include <QObject>
#include <QTimer>

// One focus session at a time. The plant on screen is a pure function of
// `progress` and `seed`, so everything the window shows comes from here.
class Backend : public QObject {
    Q_OBJECT
    Q_PROPERTY(QString state READ state NOTIFY stateChanged)
    Q_PROPERTY(double progress READ progress NOTIFY progressChanged)
    Q_PROPERTY(int remaining READ remaining NOTIFY progressChanged)
    Q_PROPERTY(int minutes READ minutes CONSTANT)
    Q_PROPERTY(int seconds READ seconds CONSTANT)
    Q_PROPERTY(int seed READ seed NOTIFY stateChanged)

public:
    // `seconds` is the session length; tests and GROVE_SECONDS shorten it.
    explicit Backend(int seconds = 25 * 60, QObject *parent = nullptr);

    // Seeds are drawn from 1 up to this, 2^30.
    static constexpr int kSeedLimit = 0x4000'0000;

    // "idle", "growing", "done" or "wilted".
    QString state() const { return m_state; }
    // How far through the session, from 0 to 1.
    double progress() const { return m_progress; }
    // Whole seconds left, rounded up, so the countdown never shows 0 early.
    int remaining() const;
    // The session's full length.
    int minutes() const { return m_seconds / 60; }
    int seconds() const { return m_seconds; }
    // The plant's seed, new each session, so no two plants are alike.
    int seed() const { return m_seed; }

    // Starts a session, unless one is already growing.
    Q_INVOKABLE void start();
    // Giving up early leaves a wilted plant behind.
    Q_INVOKABLE void giveUp();
    // Advances the session clock; the timer calls it, and tests call it directly.
    void advance(qint64 milliseconds);

signals:
    // A session started, finished or was given up.
    void stateChanged();
    // The countdown moved on a second, or the plant grew enough to show
    // (0.2% of a session); not on every tick, which no one could see.
    void progressChanged();

private:
    void setState(const QString &state);

    int m_seconds;
    QString m_state = "idle";
    qint64 m_elapsed = 0;
    double m_progress = 0;
    // What the window was last told, so ticks too small to see go unannounced.
    int m_shownRemaining = 0;
    double m_shownProgress = 0;
    int m_seed = 1;
    QTimer m_timer;
    QElapsedTimer m_clock;
};
