#include "sounds.h"

#include <QAudioDevice>
#include <QAudioFormat>
#include <QMediaDevices>
#include <QSettings>
#include <QtConcurrent>
#include <QtMath>
#include <algorithm>
#include <cmath>
#include <vector>

namespace {

constexpr double kTrainSeconds = 44.0;
constexpr double kPlaneSeconds = 30.0;

double clamp(double v, double lo = 0, double hi = 1) { return std::max(lo, std::min(hi, v)); }

// A one-pole low-pass filter: air takes the highs out of far-off sound.
struct LowPass {
    double alpha = 0, state = 0;
    LowPass() = default;
    LowPass(double hz, int rate) { setCutoff(hz, rate); }
    void setCutoff(double hz, int rate) { alpha = qExp(-2 * M_PI * hz / rate); }
    double operator()(double x) { return state = alpha * state + (1 - alpha) * x; }
};

// One train horn blast: three notes of a minor chord, each a touch out of
// tune so the chord beats, with a quick start and a short release.
void addBlast(std::vector<double> &out, int rate, double start, double seconds, double gain, double cutoff,
              double pass) {
    static const double notes[3] = {311.1, 370.0, 466.2};
    static const double detune[3] = {1.0, 1.004, 0.997};
    double phase[3] = {0, 0, 0};
    LowPass lowpass(cutoff, rate);
    const int first = static_cast<int>(start * rate), count = static_cast<int>(seconds * rate);
    for (int i = 0; i < count && first + i < static_cast<int>(out.size()); ++i) {
        const double t = start + static_cast<double>(i) / rate;
        // The horn is sharp as the train closes in and flat once it has gone by.
        const double doppler = 1 + 0.04 * std::tanh((pass - t) / 6.0);
        const double envelope = clamp((t - start) / 0.1) * clamp((start + seconds - t) / 0.22);
        double sample = 0;
        for (int n = 0; n < 3; ++n) {
            phase[n] += 2 * M_PI * notes[n] * detune[n] * doppler / rate;
            for (int h = 1; h <= 6; ++h)
                sample += qSin(h * phase[n]) / qPow(h, 0.9);
        }
        out[first + i] += gain * envelope * lowpass(sample) / 3.0;
    }
}

// The standard grade-crossing signal: long, long, short, long.
void addCrossingSignal(std::vector<double> &out, int rate, double start, double gain, double cutoff,
                       double lastLong, double pass) {
    addBlast(out, rate, start + 0.0, 2.4, gain, cutoff, pass);
    addBlast(out, rate, start + 2.9, 2.4, gain, cutoff, pass);
    addBlast(out, rate, start + 5.8, 0.7, gain, cutoff, pass);
    addBlast(out, rate, start + 7.0, lastLong, gain, cutoff, pass);
}

// A crossing bell strike: a few inharmonic partials that die away.
void addBell(std::vector<double> &out, int rate, double start, double gain, double pitch) {
    static const double ratio[4] = {1.0, 1.54, 2.47, 3.35};
    static const double level[4] = {1.0, 0.6, 0.35, 0.2};
    static const double decay[4] = {0.3, 0.22, 0.16, 0.1};
    const int first = static_cast<int>(start * rate), count = static_cast<int>(1.0 * rate);
    for (int i = 0; i < count && first + i < static_cast<int>(out.size()); ++i) {
        const double t = static_cast<double>(i) / rate;
        double sample = 0;
        for (int p = 0; p < 4; ++p)
            sample += level[p] * qExp(-t / decay[p]) * qSin(2 * M_PI * pitch * ratio[p] * t);
        out[first + i] += gain * sample;
    }
}

QByteArray toPcm(std::vector<double> &out, double peakLevel) {
    double peak = 0;
    for (double v : out)
        peak = std::max(peak, qAbs(v));
    QByteArray bytes(static_cast<qsizetype>(out.size() * 2), 0);
    auto *samples = reinterpret_cast<qint16 *>(bytes.data());
    for (size_t i = 0; i < out.size(); ++i)
        samples[i] = static_cast<qint16>(out[i] / std::max(peak, 1e-9) * peakLevel * 32767);
    return bytes;
}

// Distance in a soft echo: sound off the hills a moment later.
void addEcho(std::vector<double> &out, int rate, double delaySeconds, double feedback) {
    const size_t delay = static_cast<size_t>(delaySeconds * rate);
    for (size_t i = delay; i < out.size(); ++i)
        out[i] += feedback * out[i - delay];
}

} // namespace

int Sounds::trainMilliseconds() { return static_cast<int>(kTrainSeconds * 1000); }
int Sounds::planeMilliseconds() { return static_cast<int>(kPlaneSeconds * 1000); }

QByteArray Sounds::synthesizeTrain(int rate) {
    std::vector<double> out(static_cast<size_t>(kTrainSeconds * rate), 0.0);
    QRandomGenerator noise(11);
    // The train reaches the crossing here; everything swells to it and fades after.
    const double pass = 26.0;

    // The rumble of the rails, and the clickety-clack of the joints.
    LowPass slow(90, rate), softer(160, rate);
    for (size_t i = 0; i < out.size(); ++i) {
        const double t = static_cast<double>(i) / rate;
        const double near = qExp(-qPow((t - pass) / 8.5, 2));
        out[i] += 5.0 * near * softer(slow(noise.generateDouble() * 2 - 1));
    }
    for (double t = 2.0; t < kTrainSeconds - 2; t += 0.27) {
        const double near = qExp(-qPow((t - pass) / 8.5, 2));
        for (double offset : {0.0, 0.07}) {
            const int first = static_cast<int>((t + offset) * rate);
            LowPass tick(600, rate);
            for (int j = 0; j < rate / 20 && first + j < static_cast<int>(out.size()); ++j)
                out[first + j] += 0.7 * near * qExp(-j / (0.007 * rate)) * tick(noise.generateDouble() * 2 - 1);
        }
    }

    // Two crossing signals: first far off and soft, then closer and brighter.
    addCrossingSignal(out, rate, 3.0, 0.45, 650, 2.0, pass);
    addCrossingSignal(out, rate, 16.0, 0.85, 1100, 3.0, pass);

    // Crossing bells start before the train is there and run until it is clear.
    for (double t = 12.5; t < 34.0; t += 0.62) {
        const double level = 0.06 + 0.3 * qExp(-qPow((t - pass) / 7.0, 2));
        addBell(out, rate, t, level, 1450);
    }

    // Everything is far away: duller, with the echo of the hills.
    LowPass dull(3200, rate), dullAgain(3200, rate);
    for (double &v : out)
        v = dullAgain(dull(v));
    addEcho(out, rate, 0.29, 0.3);
    // Fade in from nothing and out to nothing, with no click at either end.
    for (size_t i = 0; i < out.size(); ++i) {
        const double t = static_cast<double>(i) / rate;
        out[i] *= clamp(t / 1.5) * clamp((kTrainSeconds - t) / 3.0);
    }
    return toPcm(out, 0.6);
}

QByteArray Sounds::synthesizePlane(int rate) {
    std::vector<double> out(static_cast<size_t>(kPlaneSeconds * rate), 0.0);
    QRandomGenerator noise(23);
    // Overhead at the middle of the flight: loudest and brightest then, with
    // the pitch falling as it passes.
    const double center = kPlaneSeconds / 2;
    double phase = 0;
    LowPass airA, airB, wind(200, rate);
    for (size_t i = 0; i < out.size(); ++i) {
        const double t = static_cast<double>(i) / rate;
        const double near = qExp(-qPow((t - center) / 6.5, 2));
        const double doppler = 1 + 0.03 * std::tanh((center - t) / 4.0);
        // A little drift in the revs, as a hand on a throttle gives.
        const double revs = 1 + 0.004 * qSin(2 * M_PI * 0.7 * t) + 0.003 * qSin(2 * M_PI * 1.9 * t);
        phase += 2 * M_PI * 86.0 * doppler * revs / rate;
        double sample = 0;
        for (int h = 1; h <= 10; ++h)
            sample += qSin(h * phase + 0.3 * h) / qPow(h, 0.8);
        sample *= 1 + 0.18 * qSin(phase / 2);
        sample += 1.4 * wind(noise.generateDouble() * 2 - 1);
        const double cutoff = 380 + 520 * near;
        // Nearer means brighter: far-off engines lose their highs to the air.
        airA.setCutoff(cutoff, rate);
        airB.setCutoff(cutoff, rate);
        out[i] = (0.04 + 0.96 * near) * airB(airA(sample));
    }
    for (size_t i = 0; i < out.size(); ++i) {
        const double t = static_cast<double>(i) / rate;
        out[i] *= clamp(t / 2.0) * clamp((kPlaneSeconds - t) / 3.0);
    }
    // Much fainter than the train: this is a plane very far up.
    return toPcm(out, 0.13);
}

int Sounds::eventCount(QRandomGenerator &random, const QString &kind) {
    const double r = random.generateDouble();
    if (kind == "train")
        return r < 0.45 ? 1 : r < 0.8 ? 2 : 3;
    return r < 0.35 ? 0 : r < 0.8 ? 1 : 2;
}

QList<double> Sounds::eventTimes(QRandomGenerator &random, int count, double sessionSeconds, double soundSeconds) {
    // Nothing right at the start or the end, and never two close together.
    const double margin = 20, gap = soundSeconds + 90;
    const double latest = sessionSeconds - soundSeconds - margin;
    for (; count > 0; --count) {
        if (latest < margin + (count - 1) * gap)
            continue;
        for (int attempt = 0; attempt < 200; ++attempt) {
            QList<double> times;
            for (int i = 0; i < count; ++i)
                times.append(margin + random.generateDouble() * (latest - margin));
            std::sort(times.begin(), times.end());
            bool apart = true;
            for (int i = 1; i < times.size(); ++i)
                apart = apart && times[i] - times[i - 1] >= gap;
            if (apart)
                return times;
        }
    }
    return {};
}

Sounds::Sounds(bool persist, const QString &forcedKind, QObject *parent)
    : QObject(parent), m_persist(persist), m_forced(forcedKind == "train" || forcedKind == "plane" ? forcedKind : QString()) {
    m_fade.setInterval(30);
    connect(&m_fade, &QTimer::timeout, this, [this] {
        const double volume = m_sink ? m_sink->volume() - m_fadeStep : 0;
        if (volume <= 0.001) {
            m_fade.stop();
            if (m_sink)
                m_sink->stop();
            return;
        }
        m_sink->setVolume(volume);
    });
    if (m_persist)
        m_muted = QSettings("grove", "grove").value("muted", false).toBool();
    // GROVE_SOUND_VOLUME scales the volume from 0 to 1, so the sound path can
    // be exercised without a sound.
    bool ok = false;
    const double scale = qEnvironmentVariable("GROVE_SOUND_VOLUME").toDouble(&ok);
    if (ok)
        m_volumeScale = clamp(scale);
}

Sounds::~Sounds() {
    // Synthesis runs on another thread and must finish before its owner goes.
    m_train.waitForFinished();
    m_plane.waitForFinished();
}

void Sounds::setMuted(bool muted) {
    if (muted == m_muted)
        return;
    m_muted = muted;
    if (m_persist)
        QSettings("grove", "grove").setValue("muted", muted);
    if (muted && m_sink) {
        m_fade.stop();
        m_sink->stop();
    }
    emit mutedChanged();
}

void Sounds::setProblem(const QString &problem) {
    if (problem == m_problem)
        return;
    m_problem = problem;
    emit problemChanged();
}

void Sounds::prepare(const QString &kind) {
    // Synthesis takes a moment, so it happens in the background, well before
    // the sound is due.
    QFuture<QByteArray> &future = kind == "train" ? m_train : m_plane;
    if (future.isValid())
        return;
    if (kind == "train")
        future = QtConcurrent::run([] { return Sounds::synthesizeTrain(); });
    else
        future = QtConcurrent::run([] { return Sounds::synthesizePlane(); });
}

void Sounds::begin(int sessionSeconds) {
    end();
    QString kind;
    QList<double> times;
    QRandomGenerator *random = QRandomGenerator::global();
    if (!m_forced.isEmpty()) {
        kind = m_forced;
        times = {3.0};
    } else {
        const QString period = m_period ? m_period() : QString();
        kind = period == "night" ? "train" : period == "day" ? "plane" : QString();
        if (kind.isEmpty())
            return;
        const int count = eventCount(*random, kind);
        const int soundMilliseconds = kind == "train" ? trainMilliseconds() : planeMilliseconds();
        times = eventTimes(*random, count, sessionSeconds, soundMilliseconds / 1000.0);
    }
    if (times.isEmpty())
        return;
    prepare(kind);
    for (double seconds : times) {
        m_timers.emplace_back();
        QTimer &timer = m_timers.back();
        timer.setSingleShot(true);
        connect(&timer, &QTimer::timeout, this, [this, kind] { fire(kind); });
        timer.start(static_cast<int>(seconds * 1000));
    }
}

void Sounds::end() {
    for (QTimer &timer : m_timers)
        timer.stop();
    m_timers.clear();
    ++m_session;
    // A sound under way fades out over a second and a half rather than
    // stopping with a click, and what goes with it fades too.
    if (m_sink && m_sink->state() == QAudio::ActiveState && !m_fade.isActive()) {
        m_fadeStep = m_sink->volume() / 50;
        m_fade.start();
    }
    emit fadingOut();
}

void Sounds::fire(const QString &kind) {
    if (m_forced.isEmpty()) {
        // Re-check the sky: night may have ended, or the weather turned.
        const QString period = m_period ? m_period() : QString();
        if (kind == "train" && period != "night")
            return;
        if (kind == "plane" && (period != "day" || (m_flightWeatherOk && !m_flightWeatherOk())))
            return;
    }
    if (m_muted)
        showVisual(kind);
    else
        play(kind);
}

void Sounds::showVisual(const QString &kind) {
    if (kind == "plane") {
        QRandomGenerator *random = QRandomGenerator::global();
        emit planeFlyover(random->bounded(2) ? 1 : -1, planeMilliseconds(), random->generateDouble());
    } else {
        emit trainPassing(trainMilliseconds());
    }
}

void Sounds::play(const QString &kind, int retries) {
    if (m_sink && m_sink->state() == QAudio::ActiveState)
        return;
    // Muted while the sound was being made: the picture still goes.
    if (m_muted) {
        showVisual(kind);
        return;
    }
    prepare(kind);
    QFuture<QByteArray> &future = kind == "train" ? m_train : m_plane;
    if (!future.isFinished()) {
        // Still being made; try again in a moment rather than miss it, unless
        // the session is over by then.
        if (retries > 0)
            QTimer::singleShot(1000, this, [this, kind, retries, session = m_session] {
                if (session == m_session)
                    play(kind, retries - 1);
            });
        else
            showVisual(kind);
        return;
    }
    if (!m_sink) {
        const QAudioDevice device = QMediaDevices::defaultAudioOutput();
        if (device.isNull()) {
            setProblem(tr("No audio output found, so the distant sounds are silent."));
            showVisual(kind);
            return;
        }
        QAudioFormat format;
        format.setSampleRate(44100);
        format.setChannelCount(1);
        format.setSampleFormat(QAudioFormat::Int16);
        if (!device.isFormatSupported(format)) {
            setProblem(tr("The audio output cannot play 16-bit sound, so the distant sounds are silent."));
            showVisual(kind);
            return;
        }
        m_sink = new QAudioSink(device, format, this);
    }
    setProblem({});
    m_fade.stop();
    m_sink->setVolume((kind == "train" ? 0.45 : 0.3) * m_volumeScale);
    m_playing = future.result();
    m_buffer.close();
    m_buffer.setData(m_playing);
    m_buffer.open(QIODevice::ReadOnly);
    // The picture starts as the sound does, so a sound that was late being made
    // never leaves them out of step.
    showVisual(kind);
    m_sink->start(&m_buffer);
}
