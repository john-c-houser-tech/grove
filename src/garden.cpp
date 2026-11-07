#include "garden.h"

#include <QDir>
#include <QFile>
#include <QFileInfo>
#include <QJsonArray>
#include <QJsonDocument>
#include <QJsonObject>
#include <QSaveFile>
#include <QStandardPaths>

QString Garden::defaultPath() {
    const QString base = QStandardPaths::writableLocation(QStandardPaths::GenericDataLocation);
    return (base.isEmpty() ? QDir::homePath() + "/.local/share" : base) + "/grove/garden.json";
}

Garden::Garden(const QString &path, QObject *parent)
    : QAbstractListModel(parent), m_path(path.isEmpty() ? defaultPath() : path) {
    load();
}

int Garden::rowCount(const QModelIndex &parent) const {
    return parent.isValid() ? 0 : static_cast<int>(m_plants.size());
}

QVariant Garden::data(const QModelIndex &index, int role) const {
    if (!index.isValid() || index.row() < 0 || index.row() >= rowCount())
        return {};
    const Plant &p = m_plants[static_cast<size_t>(index.row())];
    switch (role) {
    case SeedRole: return p.seed;
    case OutcomeRole: return p.outcome;
    case SeasonRole: return p.season;
    case WeatherRole: return p.weather;
    case ProgressRole: return p.progress;
    case WhenRole: return p.when;
    case MinutesRole: return p.minutes;
    }
    return {};
}

QHash<int, QByteArray> Garden::roleNames() const {
    return {{SeedRole, "seed"}, {OutcomeRole, "outcome"}, {SeasonRole, "season"}, {WeatherRole, "weather"},
            {ProgressRole, "progress"}, {WhenRole, "when"}, {MinutesRole, "minutes"}};
}

void Garden::setProblem(const QString &problem) {
    if (problem == m_problem)
        return;
    m_problem = problem;
    emit problemChanged();
}

void Garden::recount() {
    m_grown = m_wilted = m_focused = 0;
    for (const Plant &p : m_plants) {
        if (p.outcome == "done") {
            ++m_grown;
            m_focused += p.minutes;
        } else {
            ++m_wilted;
        }
    }
    emit countsChanged();
}

void Garden::load() {
    QFile file(m_path);
    if (!file.exists())
        return;
    if (!file.open(QIODevice::ReadOnly)) {
        m_readOnly = true;
        setProblem(tr("Could not open the garden file, so new plants will not be saved."));
        return;
    }
    QJsonParseError error;
    const QJsonDocument document = QJsonDocument::fromJson(file.readAll(), &error);
    const QJsonValue list = document.object().value("plants");
    std::vector<Plant> plants;
    bool valid = error.error == QJsonParseError::NoError && list.isArray();
    if (valid) {
        // On disk the oldest plant comes first; the model wants newest first.
        const QJsonArray array = list.toArray();
        for (const QJsonValue &value : array) {
            const QJsonObject o = value.toObject();
            Plant p;
            p.seed = o.value("seed").toInt(-1);
            p.outcome = o.value("outcome").toString();
            p.season = o.value("season").toString();
            p.weather = o.value("weather").toString();
            p.progress = o.value("progress").toDouble(-1);
            p.when = QDateTime::fromString(o.value("when").toString(), Qt::ISODate);
            p.minutes = o.value("minutes").toInt(-1);
            if (p.seed < 0 || (p.outcome != "done" && p.outcome != "wilted") || p.progress < 0 || p.progress > 1
                || !p.when.isValid() || p.minutes < 0) {
                valid = false;
                break;
            }
            plants.insert(plants.begin(), p);
        }
    }
    if (!valid) {
        m_readOnly = true;
        setProblem(tr("The garden file could not be read. It was left untouched, and new plants will not be saved."));
        return;
    }
    beginResetModel();
    m_plants = std::move(plants);
    endResetModel();
    recount();
}

bool Garden::save() {
    QJsonArray array;
    // Oldest first on disk, so a new plant is a simple append in the file.
    for (auto it = m_plants.rbegin(); it != m_plants.rend(); ++it) {
        QJsonObject o;
        o["seed"] = it->seed;
        o["outcome"] = it->outcome;
        o["season"] = it->season;
        o["weather"] = it->weather;
        o["progress"] = it->progress;
        o["when"] = it->when.toString(Qt::ISODate);
        o["minutes"] = it->minutes;
        array.append(o);
    }
    QJsonObject root;
    root["version"] = 1;
    root["plants"] = array;

    if (!QDir().mkpath(QFileInfo(m_path).absolutePath())) {
        setProblem(tr("Could not create the garden folder, so this plant was not saved."));
        return false;
    }
    // QSaveFile writes beside the target and renames, so a crash mid-write
    // never leaves half a garden.
    QSaveFile file(m_path);
    if (!file.open(QIODevice::WriteOnly) || file.write(QJsonDocument(root).toJson()) < 0 || !file.commit()) {
        setProblem(tr("Could not save the garden: %1").arg(file.errorString()));
        return false;
    }
    setProblem({});
    return true;
}

void Garden::record(const Plant &plant) {
    beginInsertRows({}, 0, 0);
    m_plants.insert(m_plants.begin(), plant);
    endInsertRows();
    recount();
    if (m_readOnly)
        return;
    save();
}
