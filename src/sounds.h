#pragma once

#include <QAudioSink>
#include <QBuffer>
#include <QByteArray>
#include <QFuture>
#include <QObject>
#include <QRandomGenerator>
#include <QString>
#include <QTimer>
#include <functional>
#include <list>

// Faint sounds from far away while a plant grows. At night a train approaches
// a crossing, one to three times in a session; by day a small plane flies
// over now and then. Both are synthesized, never shipped or fetched, and both
// are also seen: the plane crossing the sky, the train along the plain. M mutes
// the sound but not the picture.
class Sounds : public QObject {
    Q_OBJECT
    Q_PROPERTY(bool muted READ muted WRITE setMuted NOTIFY mutedChanged)
    Q_PROPERTY(QString problem READ problem NOTIFY problemChanged)

public:
    // `persist` stores the mute choice in the user's settings; tests turn it
    // off. `forcedKind` ("train" or "plane") makes one of them happen a few
    // seconds into the next session whatever the hour, for trying them out.
    explicit Sounds(bool persist = true, const QString &forcedKind = {}, QObject *parent = nullptr);
    ~Sounds() override;

    bool muted() const { return m_muted; }
    void setMuted(bool muted);
    // Empty when sound works, otherwise why it cannot (shown in the window).
    QString problem() const { return m_problem; }

    // "night", "day", or empty at dawn and dusk, when nothing happens.
    void setPeriodSource(std::function<QString()> period) { m_period = std::move(period); }
    // Whether the weather lets a plane be seen and heard.
    void setFlightWeatherOk(std::function<bool()> ok) { m_flightWeatherOk = std::move(ok); }

    // A session of this many seconds starts, or stops.
    void begin(int sessionSeconds);
    void end();

    // One phrase of each sound as 16-bit mono PCM.
    static QByteArray synthesizeTrain(int sampleRate = 44100);
    static QByteArray synthesizePlane(int sampleRate = 44100);
    static int trainMilliseconds();
    static int planeMilliseconds();
    // How many times a sound happens in one session: a train 1 to 3 times, a
    // plane 0 to 2.
    static int eventCount(QRandomGenerator &random, const QString &kind);
    // When, in seconds from the start of the session, they begin. Always
    // inside the session and well apart; fewer come back if they do not fit.
    static QList<double> eventTimes(QRandomGenerator &random, int count, double sessionSeconds, double soundSeconds);

signals:
    void mutedChanged();
    void problemChanged();
    // A plane crosses the sky: direction is 1 for left to right and -1 for the
    // other way, and altitude from 0 to 1 is how high it flies.
    void planeFlyover(int direction, int milliseconds, double altitude);
    // A train runs along the plain to a crossing: direction as for the plane,
    // and crossing is how far along from the edge it entered, from 0 to 1, that
    // the crossing lies. The flight times itself to the sound.
    void trainPassing(int direction, int milliseconds, double crossing);

private:
    void fire(const QString &kind);
    void play(const QString &kind, int retries = 8);
    // Shows what goes with the sound, starting together with it.
    void showVisual(const QString &kind);
    void prepare(const QString &kind);
    void setProblem(const QString &problem);

    bool m_persist;
    bool m_muted = false;
    double m_volumeScale = 1;
    QString m_forced;
    QString m_problem;
    std::function<QString()> m_period;
    std::function<bool()> m_flightWeatherOk;
    std::list<QTimer> m_timers;
    QFuture<QByteArray> m_train, m_plane;
    QByteArray m_playing;
    QBuffer m_buffer;
    QAudioSink *m_sink = nullptr;
};
