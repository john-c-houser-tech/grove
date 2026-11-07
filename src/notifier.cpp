#include "notifier.h"

#include <QDBusConnection>
#include <QDBusMessage>
#include <QDBusPendingCall>
#include <QDBusPendingCallWatcher>
#include <QDBusPendingReply>
#include <QVariantMap>

Notifier::Notifier(const QString &service, QObject *parent) : QObject(parent), m_service(service) {}

void Notifier::setProblem(const QString &problem) {
    if (problem == m_problem)
        return;
    m_problem = problem;
    emit problemChanged();
}

void Notifier::plantGrown() {
    send(tr("Your plant is fully grown"), tr("Take a breath, then plant another."));
}

void Notifier::send(const QString &summary, const QString &body) {
    QDBusConnection bus = QDBusConnection::sessionBus();
    if (!bus.isConnected()) {
        setProblem(tr("No desktop session bus, so Grove cannot send notifications."));
        return;
    }
    QDBusMessage message = QDBusMessage::createMethodCall(m_service, "/org/freedesktop/Notifications",
                                                          "org.freedesktop.Notifications", "Notify");
    // app name, replaces id, icon, summary, body, actions, hints, timeout.
    message << "Grove" << m_lastId << "grove" << summary << body << QStringList()
            << QVariantMap{{"desktop-entry", "grove"}} << -1;

    // Asynchronous, so a slow or missing service never freezes the window.
    auto *watcher = new QDBusPendingCallWatcher(bus.asyncCall(message), this);
    connect(watcher, &QDBusPendingCallWatcher::finished, this, [this](QDBusPendingCallWatcher *call) {
        const QDBusPendingReply<unsigned> reply = *call;
        if (reply.isError()) {
            setProblem(tr("No notification service is running, so Grove cannot tell you when a plant is done."));
        } else {
            m_lastId = reply.value();
            setProblem({});
        }
        call->deleteLater();
    });
}
