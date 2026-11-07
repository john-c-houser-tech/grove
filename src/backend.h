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

    // "idle", "growing", "done" or "wilted".
    QString state() const { return m_state; }
    double progress() const { return m_progress; }
    int remaining() const;
    int minutes() const { return m_seconds / 60; }
    int seconds() const { return m_seconds; }
    int seed() const { return m_seed; }

    Q_INVOKABLE void start();
    // Giving up early leaves a wilted plant behind.
    Q_INVOKABLE void giveUp();
    // Advances the session clock; the timer calls it, and tests call it directly.
    void advance(qint64 milliseconds);

signals:
    void stateChanged();
    void progressChanged();

private:
    void setState(const QString &state);

    int m_seconds;
    QString m_state = "idle";
    qint64 m_elapsed = 0;
    double m_progress = 0;
    int m_seed = 1;
    QTimer m_timer;
    QElapsedTimer m_clock;
};
