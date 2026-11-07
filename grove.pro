QT += core gui qml quick quickcontrols2 dbus multimedia concurrent network

CONFIG += c++17 release
TARGET = grove
TEMPLATE = app

HEADERS += \
    src/backend.h \
    src/easing.h \
    src/field.h \
    src/hills.h \
    src/layer.h \
    src/terrain.h \
    src/trackbed.h \
    src/light.h \
    src/weather.h \
    src/sounds.h \
    src/garden.h \
    src/notifier.h \
    src/problem.h \
    src/theme.h \
    src/translator.h

SOURCES += \
    src/main.cpp \
    src/backend.cpp \
    src/field.cpp \
    src/hills.cpp \
    src/layer.cpp \
    src/trackbed.cpp \
    src/light.cpp \
    src/weather.cpp \
    src/sounds.cpp \
    src/garden.cpp \
    src/notifier.cpp \
    src/theme.cpp \
    src/translator.cpp

RESOURCES += src/resources.qrc
