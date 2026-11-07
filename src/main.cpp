// grove — a focus timer that grows a plant while you work.

#include <QColor>
#include <QDateTime>
#include <QGuiApplication>
#include <QIcon>
#include <QQmlApplicationEngine>
#include <QQmlContext>
#include <QQuickStyle>
#include <QUrl>

#include "backend.h"
#include "light.h"
#include "theme.h"
#include "translator.h"

int main(int argc, char *argv[]) {
    QGuiApplication app(argc, argv);
    app.setApplicationName("grove");
    app.setApplicationVersion("0.1.0");

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

    // GROVE_TIME=HH:MM freezes the sky at that time, to see the light at any hour.
    const QTime forced = QTime::fromString(qEnvironmentVariable("GROVE_TIME"), "HH:mm");
    Light light(forced.isValid() ? QDateTime(QDate::currentDate(), forced) : QDateTime());
    light.setAccent(QColor(theme.accent()));
    QObject::connect(&theme, &Theme::changed, &light, [&] { light.setAccent(QColor(theme.accent())); });

    QQmlApplicationEngine engine;
    QObject::connect(&translator, &Translator::changed, &engine, &QQmlEngine::retranslate);
    engine.rootContext()->setContextProperty("translator", &translator);
    engine.rootContext()->setContextProperty("theme", &theme);
    engine.rootContext()->setContextProperty("backend", &backend);
    engine.rootContext()->setContextProperty("light", &light);
    engine.load(QUrl("qrc:/Main.qml"));
    if (engine.rootObjects().isEmpty())
        return 1;

    return app.exec();
}
