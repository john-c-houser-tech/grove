#include "foghorn.h"

#include <QAudioDevice>
#include <QAudioFormat>
#include <QMediaDevices>
#include <QSettings>
#include <QtMath>
#include <algorithm>
#include <vector>

namespace {

constexpr double kPhraseSeconds = 9.0;

// One horn: a few detuned stacks of harmonics, a breath of noise, a slow
// swell in and a long fade out, softened by a low-pass filter.
void addHorn(std::vector<double> &out, int rate, double start, double seconds, double hz, double gain,
             double lowpass, QRandomGenerator &noise) {
    const int first = static_cast<int>(start * rate), count = static_cast<int>(seconds * rate);
    const double alpha = qExp(-2 * M_PI * lowpass / rate);
    double phase = 0, phaseB = 0, state = 0;
    for (int i = 0; i < count && first + i < static_cast<int>(out.size()); ++i) {
        const double t = static_cast<double>(i) / rate;
        const double attack = qSin(M_PI / 2 * std::min(1.0, t / 0.45));
        const double release = qSin(M_PI / 2 * std::min(1.0, (seconds - t) / 1.1));
        const double envelope = attack * attack * release;
        // The note sags a little at first and rises into its pitch, as a horn does.
        const double pitch = hz * (1 - 0.03 * qExp(-t * 5));
        phase += 2 * M_PI * pitch / rate;
        // A second, slightly sharp voice makes the slow beating of a big horn.
        phaseB += 2 * M_PI * pitch * 1.006 / rate;
        double sample = 0;
        for (int h = 1; h <= 14; ++h)
            sample += (qSin(h * phase) + 0.7 * qSin(h * phaseB)) / qPow(h, 1.15);
        sample += (noise.generateDouble() - 0.5) * 0.35;
        state = alpha * state + (1 - alpha) * sample;
        out[first + i] += gain * envelope * state;
    }
}

} // namespace

QByteArray FogHorn::synthesize(int rate) {
    QRandomGenerator noise(7);
    std::vector<double> out(static_cast<size_t>(kPhraseSeconds * rate), 0.0);
    // The call, then a higher, quieter, muffled answer from farther out.
    addHorn(out, rate, 0.0, 3.2, 69.3, 1.0, 900, noise);
    addHorn(out, rate, 3.6, 2.8, 82.4, 0.45, 450, noise);

    // Fog swallows sound; a soft echo gives the horn some air around it.
    const int delay = static_cast<int>(0.27 * rate);
    for (size_t i = delay; i < out.size(); ++i)
        out[i] += 0.35 * out[i - delay];

    double peak = 0;
    for (double v : out)
        peak = std::max(peak, qAbs(v));
    QByteArray bytes(static_cast<qsizetype>(out.size() * 2), 0);
    auto *samples = reinterpret_cast<qint16 *>(bytes.data());
    for (size_t i = 0; i < out.size(); ++i)
        samples[i] = static_cast<qint16>(out[i] / std::max(peak, 1e-9) * 0.8 * 32767);
    return bytes;
}

int FogHorn::phraseMilliseconds(int) { return static_cast<int>(kPhraseSeconds * 1000); }

int FogHorn::nextGapMilliseconds(QRandomGenerator &random) { return 16000 + static_cast<int>(random.bounded(18000)); }

FogHorn::FogHorn(bool persist, QObject *parent) : QObject(parent), m_persist(persist) {
    if (m_persist)
        m_muted = QSettings("grove", "grove").value("muted", false).toBool();
    m_timer.setSingleShot(true);
    connect(&m_timer, &QTimer::timeout, this, [this] {
        if (!m_active || m_muted)
            return;
        sound();
        m_timer.start(phraseMilliseconds() + nextGapMilliseconds(*QRandomGenerator::global()));
    });
}

void FogHorn::setActive(bool active) {
    if (active == m_active)
        return;
    m_active = active;
    if (!active) {
        // A phrase already sounding is left to finish.
        m_timer.stop();
        return;
    }
    if (!m_muted)
        m_timer.start(4000);
}

void FogHorn::setMuted(bool muted) {
    if (muted == m_muted)
        return;
    m_muted = muted;
    if (m_persist)
        QSettings("grove", "grove").setValue("muted", muted);
    if (muted) {
        m_timer.stop();
        if (m_sink)
            m_sink->stop();
    } else if (m_active) {
        m_timer.start(2000);
    }
    emit mutedChanged();
}

void FogHorn::setProblem(const QString &problem) {
    if (problem == m_problem)
        return;
    m_problem = problem;
    emit problemChanged();
}

void FogHorn::sound() {
    if (m_sink && m_sink->state() == QAudio::ActiveState)
        return;
    if (!m_sink) {
        const QAudioDevice device = QMediaDevices::defaultAudioOutput();
        if (device.isNull()) {
            setProblem(tr("No audio output found, so the fog horns are silent."));
            return;
        }
        QAudioFormat format;
        format.setSampleRate(44100);
        format.setChannelCount(1);
        format.setSampleFormat(QAudioFormat::Int16);
        if (!device.isFormatSupported(format)) {
            setProblem(tr("The audio output cannot play 16-bit sound, so the fog horns are silent."));
            return;
        }
        m_data = synthesize(format.sampleRate());
        m_buffer.setData(m_data);
        m_buffer.open(QIODevice::ReadOnly);
        m_sink = new QAudioSink(device, format, this);
        // Quiet: a horn heard through fog, not an alarm.
        m_sink->setVolume(0.35);
    }
    setProblem({});
    m_buffer.seek(0);
    m_sink->start(&m_buffer);
}
