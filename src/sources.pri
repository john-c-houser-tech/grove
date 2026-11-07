# Everything but main(), shared by the app and its two test programs, so a new
# file is added in one place.
QT += core gui qml quick quickcontrols2 dbus multimedia concurrent network
CONFIG += c++17 warn_on
INCLUDEPATH += $$PWD
# Beyond qmake's -Wall -Wextra. bin/lint builds with -Werror as well, so a
# warning fails the check without breaking a packager's build on a newer
# compiler.
QMAKE_CXXFLAGS += -Wpedantic -Wshadow

# The C++ types QML uses form the Grove module. qmltyperegistrar registers
# them, and writes <target>.qmltypes, which bin/lint gives qmllint so it knows
# every type and property the QML refers to.
CONFIG += qmltypes
QML_IMPORT_NAME = Grove
QML_IMPORT_MAJOR_VERSION = 1

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
    $$PWD/qmltypes.h \
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
