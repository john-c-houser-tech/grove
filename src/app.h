#pragma once

#include <QDate>
#include <QQmlApplicationEngine>
#include <QString>
#include <QTime>
#include <memory>

#include "backend.h"
#include "garden.h"
#include "light.h"
#include "notifier.h"
#include "sounds.h"
#include "theme.h"
#include "translator.h"
#include "weather.h"

class QQuickWindow;
class App;

// The QML engine of an App. The Grove module's singletons (qmltypes.h) are
// made by the engine, and find the App whose objects they are through it.
class AppEngine : public QQmlApplicationEngine {
public:
    explicit AppEngine(App *app) : m_app(app) {}
    App &app() const { return *m_app; }

private:
    App *m_app;
};

// Everything the window runs on, made and wired together in one place, so the
// app and its scene tests build the very same thing. The QML engine is made
// last and destroyed first, so nothing in the window outlives what it shows.
class App {
public:
    // How an App is set up: from the environment for the real app, and by
    // hand for tests, which keep everything of the desktop's out of reach.
    struct Options {
        // How long a session lasts.
        int seconds = 25 * 60;
        // A fixed time of day and date for the sky, when valid.
        QTime time;
        QDate date;
        // A pinned weather ("clear", "rain"...), and a sound ("train" or
        // "plane") to hear a few seconds into every session.
        QString weather;
        QString soundTest;
        // Where the garden is kept; empty for the user's data directory. A
        // shortened session is only saved when a garden is given.
        QString gardenPath;
        // Where the theme and the language are read from; empty for Omarchy's.
        QString themeDirectory;
        QString languageFile;
        // The desktop notification service.
        QString notificationService = "org.freedesktop.Notifications";
        // Whether muting is remembered between runs.
        bool rememberMute = true;
    };

    // Options from the environment: GROVE_SECONDS, GROVE_TIME (HH:MM),
    // GROVE_DATE (YYYY-MM-DD), GROVE_WEATHER, GROVE_SOUND_TEST and GROVE_GARDEN.
    static Options fromEnvironment();

    // Makes and wires everything; the window comes with load().
    explicit App(const Options &options);
    ~App();
    App(const App &) = delete;
    App &operator=(const App &) = delete;
    App(App &&) = delete;
    App &operator=(App &&) = delete;

    // Loads the window. False if it could not be made.
    bool load();
    // The window and its engine, once loaded.
    QQuickWindow *window() const;
    QQmlApplicationEngine *engine() const { return m_engine.get(); }

    Backend &backend() { return m_backend; }
    Garden &garden() { return m_garden; }
    Light &light() { return m_light; }
    Notifier &notifier() { return m_notifier; }
    Sounds &sounds() { return m_sounds; }
    Theme &theme() { return m_theme; }
    Translator &translator() { return m_translator; }
    Weather &weather() { return m_weather; }

private:
    QDate today() const;
    void sessionChanged();

    Options m_options;
    Translator m_translator;
    Theme m_theme;
    Backend m_backend;
    Light m_light;
    Weather m_weather;
    Sounds m_sounds;
    Notifier m_notifier;
    Garden m_garden;
    std::unique_ptr<AppEngine> m_engine;
};
