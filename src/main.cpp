// grove — a focus timer that grows a plant while you work.

#include <QColor>
#include <QDateTime>
#include <QGuiApplication>
#include <QIcon>
#include <QQmlApplicationEngine>
#include <QQmlContext>
#include <QQuickStyle>
#include <QRandomGenerator>
#include <QUrl>

#include "backend.h"
#include "garden.h"
#include "light.h"
#include "notifier.h"
#include "sounds.h"
#include "theme.h"
#include "translator.h"
#include "weather.h"

int main(int argc, char *argv[]) {
    QGuiApplication app(argc, argv);
    app.setApplicationName("grove");
    app.setApplicationVersion("0.4.5");

    // Matches the window to grove.desktop, so the compositor (Wayland app_id)
    // and the launcher pick up the installed icon.
    app.setDesktopFileName("grove");
    app.setWindowIcon(QIcon::fromTheme("grove"));

    QQuickStyle::setStyle("Material");

    // Installed before anything is translated, and every qsTr() binding is
    // re-evaluated when the Omarchy language changes.
    Translator translator;
    app.installTranslator(&translator);

    Theme theme;
    // GROVE_SECONDS shortens a session so the growth can be watched quickly.
    bool ok = false;
    const int seconds = qEnvironmentVariableIntValue("GROVE_SECONDS", &ok);
    Backend backend(ok && seconds > 0 ? seconds : 25 * 60);

    // GROVE_TIME=HH:MM and GROVE_DATE=YYYY-MM-DD freeze the sky, and
    // GROVE_WEATHER=clear|cloudy|rain|fog pins the weather, to see any of them.
    const QTime forcedTime = QTime::fromString(qEnvironmentVariable("GROVE_TIME"), "HH:mm");
    const QDate forcedDate = QDate::fromString(qEnvironmentVariable("GROVE_DATE"), Qt::ISODate);
    const QDate today = forcedDate.isValid() ? forcedDate : QDate::currentDate();
    Light light(forcedTime.isValid() || forcedDate.isValid()
                    ? QDateTime(today, forcedTime.isValid() ? forcedTime : QTime::currentTime())
                    : QDateTime());
    light.setAccent(QColor(theme.accent()));
    QObject::connect(&theme, &Theme::changed, &light, [&] { light.setAccent(QColor(theme.accent())); });

    Weather weather(qEnvironmentVariable("GROVE_WEATHER"));
    weather.begin(1 + QRandomGenerator::global()->bounded(1 << 30), today);
    // GROVE_SOUND_TEST=train or plane makes that sound happen a few seconds into
    // the next session, whatever the hour.
    Sounds sounds(true, qEnvironmentVariable("GROVE_SOUND_TEST"));
    sounds.setPeriodSource([&]() -> QString {
        if (light.sunUp() && light.ambient() > 0.6)
            return "day";
        if (!light.sunUp() && light.ambient() < 0.35)
            return "night";
        return {};
    });
    sounds.setFlightWeatherOk([&] { return weather.rain() < 0.25 && weather.snow() < 0.25 && weather.fog() < 0.35; });
    Notifier notifier;

    // GROVE_GARDEN points the garden at another file. A shortened test session
    // (GROVE_SECONDS) is not saved unless it does, so trying the app out never
    // adds make-believe plants to the real garden.
    const bool shortened = qEnvironmentVariableIsSet("GROVE_SECONDS");
    Garden garden(qEnvironmentVariable("GROVE_GARDEN"));
    const bool saving = !shortened || qEnvironmentVariableIsSet("GROVE_GARDEN");
    // A new session rolls new weather; the plant's progress carries it along.
    QObject::connect(&backend, &Backend::stateChanged, &weather, [&] {
        if (backend.state() == "growing")
            weather.begin(backend.seed(), today);
        if (backend.state() == "growing")
            sounds.begin(backend.seconds());
        else
            sounds.end();
        // A shortened test session stays quiet too, like it stays out of the garden.
        if (backend.state() == "done" && saving)
            notifier.plantGrown();
        if (saving && (backend.state() == "done" || backend.state() == "wilted")) {
            Garden::Plant plant;
            plant.seed = backend.seed();
            plant.outcome = backend.state();
            plant.season = weather.season();
            plant.weather = weather.kind();
            plant.progress = weather.growth();
            plant.when = QDateTime::currentDateTime();
            plant.minutes = backend.state() == "done" ? backend.minutes() : 0;
            garden.record(plant);
        }
    });
    QObject::connect(&backend, &Backend::progressChanged, &weather, [&] { weather.setProgress(backend.progress()); });

    QQmlApplicationEngine engine;
    QObject::connect(&translator, &Translator::changed, &engine, &QQmlEngine::retranslate);
    engine.rootContext()->setContextProperty("translator", &translator);
    engine.rootContext()->setContextProperty("theme", &theme);
    engine.rootContext()->setContextProperty("backend", &backend);
    engine.rootContext()->setContextProperty("light", &light);
    engine.rootContext()->setContextProperty("weather", &weather);
    engine.rootContext()->setContextProperty("sounds", &sounds);
    engine.rootContext()->setContextProperty("garden", &garden);
    engine.rootContext()->setContextProperty("notifier", &notifier);
    engine.load(QUrl("qrc:/Main.qml"));
    if (engine.rootObjects().isEmpty())
        return 1;

    return app.exec();
}
