#pragma once

#include <QJSEngine>
#include <QQmlEngine>
#include <QtQml/qqmlregistration.h>

#include "app.h"

// The objects App owns, as singletons of the Grove QML module, so QML reads
// `Backend.state` and qmllint knows what Backend has. They are declared here
// rather than in each class, which stays free of QML: each create() lends the
// engine the object of the App that made it, which App keeps owning.
namespace qmltypes {

// The App whose engine is making a singleton.
inline App &appOf(QQmlEngine *engine) {
    auto *appEngine = dynamic_cast<AppEngine *>(engine);
    Q_ASSERT(appEngine != nullptr);
    return appEngine->app();
}

// Marks an object as C++'s, so the engine never deletes it, and hands it over.
template <typename T>
T *lend(T *object) {
    QJSEngine::setObjectOwnership(object, QJSEngine::CppOwnership);
    return object;
}

struct BackendSingleton {
    Q_GADGET
    QML_FOREIGN(Backend)
    QML_NAMED_ELEMENT(Backend)
    QML_SINGLETON

public:
    static Backend *create(QQmlEngine *qml, QJSEngine * /*js*/) { return lend(&appOf(qml).backend()); }
};

struct LightSingleton {
    Q_GADGET
    QML_FOREIGN(Light)
    QML_NAMED_ELEMENT(Light)
    QML_SINGLETON

public:
    static Light *create(QQmlEngine *qml, QJSEngine * /*js*/) { return lend(&appOf(qml).light()); }
};

struct WeatherSingleton {
    Q_GADGET
    QML_FOREIGN(Weather)
    QML_NAMED_ELEMENT(Weather)
    QML_SINGLETON

public:
    static Weather *create(QQmlEngine *qml, QJSEngine * /*js*/) { return lend(&appOf(qml).weather()); }
};

struct ThemeSingleton {
    Q_GADGET
    QML_FOREIGN(Theme)
    QML_NAMED_ELEMENT(Theme)
    QML_SINGLETON

public:
    static Theme *create(QQmlEngine *qml, QJSEngine * /*js*/) { return lend(&appOf(qml).theme()); }
};

struct SoundsSingleton {
    Q_GADGET
    QML_FOREIGN(Sounds)
    QML_NAMED_ELEMENT(Sounds)
    QML_SINGLETON

public:
    static Sounds *create(QQmlEngine *qml, QJSEngine * /*js*/) { return lend(&appOf(qml).sounds()); }
};

struct GardenSingleton {
    Q_GADGET
    QML_FOREIGN(Garden)
    QML_NAMED_ELEMENT(Garden)
    QML_SINGLETON

public:
    static Garden *create(QQmlEngine *qml, QJSEngine * /*js*/) { return lend(&appOf(qml).garden()); }
};

struct NotifierSingleton {
    Q_GADGET
    QML_FOREIGN(Notifier)
    QML_NAMED_ELEMENT(Notifier)
    QML_SINGLETON

public:
    static Notifier *create(QQmlEngine *qml, QJSEngine * /*js*/) { return lend(&appOf(qml).notifier()); }
};

struct TranslatorSingleton {
    Q_GADGET
    QML_FOREIGN(Translator)
    QML_NAMED_ELEMENT(Translator)
    QML_SINGLETON

public:
    static Translator *create(QQmlEngine *qml, QJSEngine * /*js*/) { return lend(&appOf(qml).translator()); }
};

} // namespace qmltypes
