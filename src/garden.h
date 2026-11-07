#pragma once

#include <QAbstractListModel>
#include <QDateTime>
#include <QString>
#include <vector>

#include "problem.h"

// Every plant that was finished or wilted, kept in one small JSON file so the
// garden fills up over days. The model lists the newest plant first.
class Garden : public QAbstractListModel {
    Q_OBJECT
    Q_PROPERTY(int grown READ grown NOTIFY countsChanged)
    Q_PROPERTY(int wilted READ wilted NOTIFY countsChanged)
    Q_PROPERTY(int focusedMinutes READ focusedMinutes NOTIFY countsChanged)
    Q_PROPERTY(QString problem READ problem NOTIFY problemChanged)

public:
    struct Plant {
        int seed = 1;
        // "done" or "wilted".
        QString outcome;
        QString season, weather;
        // How much of the plant had grown, as the window showed it.
        double progress = 0;
        QDateTime when;
        int minutes = 0;
    };

    enum Roles { SeedRole = Qt::UserRole + 1, OutcomeRole, SeasonRole, WeatherRole, ProgressRole, WhenRole, MinutesRole };

    // An empty path means the user's data directory.
    explicit Garden(const QString &path = {}, QObject *parent = nullptr);

    int rowCount(const QModelIndex &parent = {}) const override;
    QVariant data(const QModelIndex &index, int role) const override;
    QHash<int, QByteArray> roleNames() const override;

    int grown() const { return m_grown; }
    int wilted() const { return m_wilted; }
    // Time spent on plants that grew to the end; a wilted plant earns none.
    int focusedMinutes() const { return m_focused; }
    // Empty when the garden is fine, otherwise why it cannot be read or saved.
    QString problem() const { return m_problem.text(); }

    void record(const Plant &plant);

    static QString defaultPath();

signals:
    void countsChanged();
    void problemChanged();

private:
    enum class Read { Missing, Ok, Unreadable, Invalid };
    // Reads the file as it is now, newest plant first.
    Read read(std::vector<Plant> &plants) const;
    void load();
    bool save();
    void setProblem(Problem::Words words);
    void recount();

    QString m_path;
    // Newest first.
    std::vector<Plant> m_plants;
    int m_grown = 0, m_wilted = 0, m_focused = 0;
    Problem m_problem;
    // A file that could not be read is left exactly as it is, so nothing is
    // saved over it.
    bool m_readOnly = false;
};
