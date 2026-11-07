#pragma once

#include <QObject>
#include <QString>

// Tells the desktop when a plant is fully grown, through the standard
// notification service, so it shows up even when the window is buried.
// Do-not-disturb in the shell silences it like any other notification.
class Notifier : public QObject {
    Q_OBJECT
    Q_PROPERTY(QString problem READ problem NOTIFY problemChanged)

public:
    // `service` is only changed by tests.
    explicit Notifier(const QString &service = "org.freedesktop.Notifications", QObject *parent = nullptr);

    // Empty when notifications work, otherwise why they cannot (shown in the window).
    QString problem() const { return m_problem; }

    void plantGrown();

signals:
    void problemChanged();

private:
    void send(const QString &summary, const QString &body);
    void setProblem(const QString &problem);

    QString m_service;
    QString m_problem;
    // Each notification replaces the last, so a day of sessions never stacks up.
    unsigned m_lastId = 0;
};
