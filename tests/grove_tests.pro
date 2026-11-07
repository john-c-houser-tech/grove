QT += core gui quick quickcontrols2 testlib dbus
CONFIG += c++17 testcase
TARGET = grove_tests
TEMPLATE = app

INCLUDEPATH += ../src

HEADERS += ../src/backend.h ../src/light.h ../src/theme.h ../src/translator.h
SOURCES += grove_tests.cpp ../src/backend.cpp ../src/light.cpp ../src/theme.cpp ../src/translator.cpp
