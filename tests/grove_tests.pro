# Unit tests: each class on its own, with no window.
QT += testlib
CONFIG += testcase
TARGET = grove_tests
TEMPLATE = app

include(../src/sources.pri)
SOURCES += grove_tests.cpp
