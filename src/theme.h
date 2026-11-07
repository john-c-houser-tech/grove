#pragma once

#include <QFileSystemWatcher>
#include <QLocalSocket>
#include <QObject>
#include <QProcess>
#include <QTimer>

// The desktop's look, followed live: the accent colour from the current
// Omarchy theme, and the corner radius Hyprland gives windows.
class Theme : public QObject {
    Q_OBJECT
    Q_DISABLE_COPY_MOVE(Theme)
    // The theme's accent as #rrggbb, and the black or white that reads on it.
    Q_PROPERTY(QString accent READ accent NOTIFY changed)
    Q_PROPERTY(QString accentForeground READ accentForeground NOTIFY changed)
    // The corner radius, in pixels, as Hyprland rounds windows.
    Q_PROPERTY(double radius READ radius NOTIFY changed)

public:
    // `currentDirectory` is Omarchy's current-theme directory; empty means the
    // user's, and only then is Hyprland asked for the rounding. Tests pass a
    // directory of their own, and may pass a stand-in for `hyprctl`.
    explicit Theme(const QString &currentDirectory = {}, QObject *parent = nullptr, const QString &hyprctl = {});
    ~Theme() override;

    QString accent() const { return m_accent; }
    QString accentForeground() const;
    double radius() const { return m_radius; }

    // The accent in a theme's colors.toml, or Omarchy's default yellow.
    static QString readAccent(const QString &path);

signals:
    // The accent or the radius changed.
    void changed();

private:
    void reload();
    void refreshRounding();
    void askAgainIfPending();

    QString m_directory;
    QString m_accent = "#FFD60A";
    QFileSystemWatcher m_watcher;
    QTimer m_debounce;
    QString m_hyprctl;
    int m_radius = 0;
    QProcess m_roundingProcess;
    QTimer m_roundingTimeout;
    bool m_roundingPending = false;
    // Hyprland's event socket, which says when its config has been reloaded.
    QLocalSocket m_events;
};
