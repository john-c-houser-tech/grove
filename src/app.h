#pragma once

#include <QDate>
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

class QQmlApplicationEngine;
class QQuickWindow;

// Everything the window runs on, made and wired together in one place, so the
// app and its scene tests build the very same thing. The QML engine is made
// last and destroyed first, so nothing in the window outlives what it shows.
class App {
public:
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

    explicit App(const Options &options);
    ~App();
    App(const App &) = delete;
    App &operator=(const App &) = delete;

    // Loads the window. False if it could not be made.
    bool load();
    // The window and its engine, once loaded.
    QQuickWindow *window() const;
    QQmlApplicationEngine *engine() const { return m_engine.get(); }

    Backend &backend() { return m_backend; }
    Garden &garden() { return m_garden; }
    Sounds &sounds() { return m_sounds; }
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
    std::unique_ptr<QQmlApplicationEngine> m_engine;
};
