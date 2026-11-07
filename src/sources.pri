# Everything but main(), shared by the app and its two test programs, so a new
# file is added in one place.
QT += core gui qml quick quickcontrols2 dbus multimedia concurrent network
CONFIG += c++17
INCLUDEPATH += $$PWD

HEADERS += \
    $$PWD/app.h \
    $$PWD/backend.h \
    $$PWD/easing.h \
    $$PWD/field.h \
    $$PWD/garden.h \
    $$PWD/hills.h \
    $$PWD/layer.h \
    $$PWD/light.h \
    $$PWD/notifier.h \
    $$PWD/problem.h \
    $$PWD/sounds.h \
    $$PWD/terrain.h \
    $$PWD/theme.h \
    $$PWD/trackbed.h \
    $$PWD/translator.h \
    $$PWD/weather.h

SOURCES += \
    $$PWD/app.cpp \
    $$PWD/backend.cpp \
    $$PWD/field.cpp \
    $$PWD/garden.cpp \
    $$PWD/hills.cpp \
    $$PWD/layer.cpp \
    $$PWD/light.cpp \
    $$PWD/notifier.cpp \
    $$PWD/sounds.cpp \
    $$PWD/theme.cpp \
    $$PWD/trackbed.cpp \
    $$PWD/translator.cpp \
    $$PWD/weather.cpp

RESOURCES += $$PWD/resources.qrc
