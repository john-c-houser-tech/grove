#pragma once

#include <QAudioSink>
#include <QBuffer>
#include <QObject>
#include <QRandomGenerator>
#include <QTimer>

// Fog horns: a deep call and a more distant answer, made from sine waves at
// start-up rather than shipped as a recording. They sound only while a plant
// is growing in fog, and the M key mutes them.
class FogHorn : public QObject {
    Q_OBJECT
    Q_PROPERTY(bool muted READ muted WRITE setMuted NOTIFY mutedChanged)
    Q_PROPERTY(QString problem READ problem NOTIFY problemChanged)

public:
    // `persist` stores the mute choice in the user's settings; tests turn it off.
    explicit FogHorn(bool persist = true, QObject *parent = nullptr);

    bool muted() const { return m_muted; }
    void setMuted(bool muted);
    // Empty when sound works, otherwise why it cannot (shown in the window).
    QString problem() const { return m_problem; }

    // True while a horn should be heard: growing in fog, and not muted.
    void setActive(bool active);

    // One call-and-answer phrase as 16-bit mono PCM.
    static QByteArray synthesize(int sampleRate = 44100);
    static int phraseMilliseconds(int sampleRate = 44100);
    // Quiet gap before the next phrase.
    static int nextGapMilliseconds(QRandomGenerator &random);

signals:
    void mutedChanged();
    void problemChanged();

private:
    void sound();
    void setProblem(const QString &problem);

    bool m_persist;
    bool m_muted = false;
    bool m_active = false;
    QString m_problem;
    QTimer m_timer;
    QByteArray m_data;
    QBuffer m_buffer;
    QAudioSink *m_sink = nullptr;
};
