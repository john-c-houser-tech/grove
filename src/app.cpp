#include "app.h"

#include <QColor>
#include <QCoreApplication>
#include <QDateTime>
#include <QQmlApplicationEngine>
#include <QQmlContext>
#include <QQuickStyle>
#include <QQuickWindow>
#include <QRandomGenerator>
#include <QUrl>
#include <QtQml/qqml.h>

#include "field.h"
#include "hills.h"
#include "trackbed.h"

namespace {

// What every window in the process shares, set up once before the first is
// loaded: the controls' style, and the landscape types, drawn in C++.
void prepareQml() {
    static const bool prepared = [] {
        QQuickStyle::setStyle("Material");
        qmlRegisterType<Hills>("Grove", 1, 0, "Hills");
        qmlRegisterType<TrackBed>("Grove", 1, 0, "TrackBed");
        qmlRegisterType<Field>("Grove", 1, 0, "Field");
        return true;
    }();
    Q_UNUSED(prepared);
}

} // namespace

App::Options App::fromEnvironment() {
    Options options;
    // GROVE_SECONDS shortens a session so the growth can be watched quickly.
    bool ok = false;
    const int seconds = qEnvironmentVariableIntValue("GROVE_SECONDS", &ok);
    if (ok && seconds > 0)
        options.seconds = seconds;
    // GROVE_TIME and GROVE_DATE freeze the sky, and GROVE_WEATHER pins the
    // weather, to see any of them.
    options.time = QTime::fromString(qEnvironmentVariable("GROVE_TIME"), "HH:mm");
    options.date = QDate::fromString(qEnvironmentVariable("GROVE_DATE"), Qt::ISODate);
    options.weather = qEnvironmentVariable("GROVE_WEATHER");
    options.soundTest = qEnvironmentVariable("GROVE_SOUND_TEST");
    options.gardenPath = qEnvironmentVariable("GROVE_GARDEN");
    return options;
}

App::App(const Options &options)
    : m_options(options),
      m_translator(options.languageFile),
      m_theme(options.themeDirectory),
      m_backend(options.seconds),
      m_light(options.time.isValid() || options.date.isValid()
                  ? QDateTime(today(), options.time.isValid() ? options.time : QTime::currentTime())
                  : QDateTime()),
      m_weather(options.weather),
      m_sounds(options.rememberMute, options.soundTest),
      m_notifier(options.notificationService),
      m_garden(options.gardenPath) {
    // Installed before anything is translated.
    QCoreApplication::installTranslator(&m_translator);

    m_light.setAccent(QColor(m_theme.accent()));
    QObject::connect(&m_theme, &Theme::changed, &m_light, [this] { m_light.setAccent(QColor(m_theme.accent())); });

    m_weather.begin(QRandomGenerator::global()->bounded(1, Backend::kSeedLimit), today());
    m_sounds.setPeriodSource([this]() -> QString {
        if (m_light.sunUp() && m_light.ambient() > 0.6)
            return "day";
        if (!m_light.sunUp() && m_light.ambient() < 0.35)
            return "night";
        return {};
    });
    m_sounds.setFlightWeatherOk(
        [this] { return m_weather.rain() < 0.25 && m_weather.snow() < 0.25 && m_weather.fog() < 0.35; });

    QObject::connect(&m_backend, &Backend::stateChanged, &m_weather, [this] { sessionChanged(); });
    QObject::connect(&m_backend, &Backend::progressChanged, &m_weather,
                     [this] { m_weather.setProgress(m_backend.progress()); });

    // Problems reported from C++ are put into words when shown, so asking the
    // window to show them again puts them in the new language.
    QObject::connect(&m_translator, &Translator::changed, &m_garden, &Garden::problemChanged);
    QObject::connect(&m_translator, &Translator::changed, &m_notifier, &Notifier::problemChanged);
    QObject::connect(&m_translator, &Translator::changed, &m_sounds, &Sounds::problemChanged);
}

App::~App() {
    // The window goes before what it shows.
    m_engine.reset();
    QCoreApplication::removeTranslator(&m_translator);
}

QDate App::today() const {
    // Read afresh for each session, since Grove may stay open for days.
    return m_options.date.isValid() ? m_options.date : QDate::currentDate();
}

void App::sessionChanged() {
    const QString state = m_backend.state();
    // A shortened test session (GROVE_SECONDS) is not saved unless a garden is
    // given, so trying the app out never adds make-believe plants to the real
    // garden. It stays quiet too.
    const bool saving = m_backend.seconds() == 25 * 60 || !m_options.gardenPath.isEmpty();
    // A new session rolls new weather; the plant's progress carries it along.
    if (state == "growing") {
        m_weather.begin(m_backend.seed(), today());
        m_sounds.begin(m_backend.seconds());
    } else {
        m_sounds.end();
    }
    if (state == "done" && saving)
        m_notifier.plantGrown();
    if (saving && (state == "done" || state == "wilted")) {
        Garden::Plant plant;
        plant.seed = m_backend.seed();
        plant.outcome = state;
        plant.season = m_weather.season();
        plant.weather = m_weather.kind();
        plant.progress = m_weather.growth();
        plant.when = QDateTime::currentDateTime();
        plant.minutes = state == "done" ? m_backend.minutes() : 0;
        m_garden.record(plant);
    }
}

bool App::load() {
    prepareQml();
    m_engine = std::make_unique<QQmlApplicationEngine>();
    QObject::connect(&m_translator, &Translator::changed, m_engine.get(), &QQmlEngine::retranslate);
    QQmlContext *context = m_engine->rootContext();
    context->setContextProperty("translator", &m_translator);
    context->setContextProperty("theme", &m_theme);
    context->setContextProperty("backend", &m_backend);
    context->setContextProperty("light", &m_light);
    context->setContextProperty("weather", &m_weather);
    context->setContextProperty("sounds", &m_sounds);
    context->setContextProperty("garden", &m_garden);
    context->setContextProperty("notifier", &m_notifier);
    m_engine->load(QUrl("qrc:/Main.qml"));
    return window() != nullptr;
}

QQuickWindow *App::window() const {
    if (!m_engine || m_engine->rootObjects().isEmpty())
        return nullptr;
    return qobject_cast<QQuickWindow *>(m_engine->rootObjects().constFirst());
}
