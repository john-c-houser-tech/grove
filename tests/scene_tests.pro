# Scene tests: the whole app, its real window drawn offscreen and driven by keys.
QT += testlib
CONFIG += testcase
TARGET = scene_tests
TEMPLATE = app

include(../src/sources.pri)
SOURCES += scene_tests.cpp
