QT += core gui quick quickcontrols2 testlib dbus multimedia concurrent network
CONFIG += c++17 testcase
TARGET = grove_tests
TEMPLATE = app

INCLUDEPATH += ../src

HEADERS += ../src/easing.h ../src/problem.h ../src/layer.h ../src/terrain.h ../src/hills.h ../src/field.h ../src/trackbed.h ../src/backend.h ../src/light.h ../src/weather.h ../src/sounds.h ../src/garden.h ../src/notifier.h ../src/theme.h ../src/translator.h
SOURCES += grove_tests.cpp ../src/layer.cpp ../src/hills.cpp ../src/field.cpp ../src/trackbed.cpp ../src/backend.cpp ../src/light.cpp ../src/weather.cpp ../src/sounds.cpp ../src/garden.cpp ../src/notifier.cpp ../src/theme.cpp ../src/translator.cpp
