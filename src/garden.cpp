#include "garden.h"

#include <QDir>
#include <QFile>
#include <QFileInfo>
#include <QJsonArray>
#include <QJsonDocument>
#include <QJsonObject>
#include <QLockFile>
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
    default: break;
    }
    return {};
}

QHash<int, QByteArray> Garden::roleNames() const {
    return {{SeedRole, "seed"},         {OutcomeRole, "outcome"}, {SeasonRole, "season"},  {WeatherRole, "weather"},
            {ProgressRole, "progress"}, {WhenRole, "when"},       {MinutesRole, "minutes"}};
}

void Garden::setProblem(Problem::Words words) {
    if (m_problem.set(std::move(words)))
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

Garden::Read Garden::read(std::vector<Plant> &plants) const {
    QFile file(m_path);
    if (!file.exists())
        return Read::Missing;
    if (!file.open(QIODevice::ReadOnly))
        return Read::Unreadable;
    QJsonParseError error;
    const QJsonDocument document = QJsonDocument::fromJson(file.readAll(), &error);
    const QJsonValue list = document.object().value("plants");
    if (error.error != QJsonParseError::NoError || !list.isArray())
        return Read::Invalid;
    // On disk the oldest plant comes first; the model wants newest first.
    plants.clear();
    const QJsonArray array = list.toArray();
    for (const auto &value : array) {
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
            || !p.when.isValid() || p.minutes < 0)
            return Read::Invalid;
        plants.insert(plants.begin(), p);
    }
    return Read::Ok;
}

void Garden::load() {
    std::vector<Plant> plants;
    switch (read(plants)) {
    case Read::Missing: return;
    case Read::Unreadable:
        m_readOnly = true;
        setProblem([] { return tr("Could not open the garden file, so new plants will not be saved."); });
        return;
    case Read::Invalid:
        m_readOnly = true;
        setProblem([] {
            return tr("The garden file could not be read. It was left untouched, and new plants will not be saved.");
        });
        return;
    case Read::Ok: break;
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

    // QSaveFile writes beside the target and renames, so a crash mid-write
    // never leaves half a garden.
    QSaveFile file(m_path);
    if (!file.open(QIODevice::WriteOnly) || file.write(QJsonDocument(root).toJson()) < 0 || !file.commit()) {
        setProblem([why = file.errorString()] { return tr("Could not save the garden: %1").arg(why); });
        return false;
    }
    setProblem({});
    return true;
}

void Garden::record(const Plant &plant) {
    const auto keepUnsaved = [this, &plant] {
        beginInsertRows({}, 0, 0);
        m_plants.insert(m_plants.begin(), plant);
        endInsertRows();
        recount();
    };
    if (m_readOnly) {
        keepUnsaved();
        return;
    }
    if (!QDir().mkpath(QFileInfo(m_path).absolutePath())) {
        keepUnsaved();
        setProblem([] { return tr("Could not create the garden folder, so this plant was not saved."); });
        return;
    }
    // Another Grove may be open on the same garden. Only one saves at a time,
    // and each starts from the file as it is now, so neither drops the
    // plants the other has saved since it was read.
    QLockFile lock(m_path + ".lock");
    if (!lock.tryLock(5000)) {
        keepUnsaved();
        setProblem([] { return tr("Another Grove kept the garden busy, so this plant was not saved."); });
        return;
    }
    std::vector<Plant> plants;
    switch (read(plants)) {
    case Read::Missing: plants.clear(); break;
    case Read::Ok: break;
    case Read::Unreadable:
    case Read::Invalid:
        // Changed into something that cannot be read since: leave it be.
        m_readOnly = true;
        keepUnsaved();
        setProblem([] {
            return tr("The garden file could not be read. It was left untouched, and new plants will not be saved.");
        });
        return;
    }
    plants.insert(plants.begin(), plant);
    beginResetModel();
    m_plants = std::move(plants);
    endResetModel();
    recount();
    save();
}
