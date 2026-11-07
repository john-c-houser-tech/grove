#pragma once

#include <QDate>
#include <QObject>
#include <QRandomGenerator>
#include <QString>

// The sky's mood, which is invented and never fetched. Each session rolls a
// weather for its start and another for its end, weighted by the season, and
// the plant grows through the change between them.
class Weather : public QObject {
    Q_OBJECT
    Q_PROPERTY(QString kind READ kind NOTIFY changed)
    Q_PROPERTY(QString season READ season NOTIFY changed)
    Q_PROPERTY(double cloud READ cloud NOTIFY changed)
    Q_PROPERTY(double rain READ rain NOTIFY changed)
    Q_PROPERTY(double fog READ fog NOTIFY changed)
    Q_PROPERTY(double growth READ growth NOTIFY changed)

public:
    struct Conditions {
        double cloud = 0, rain = 0, fog = 0;
    };

    // A kind name ("clear", "cloudy", "rain" or "fog") pins the weather, for
    // screenshots and tests; the empty string lets it roll.
    explicit Weather(const QString &forcedKind = {}, QObject *parent = nullptr);

    // The dominant weather right now.
    QString kind() const;
    QString season() const { return m_season; }
    double cloud() const { return m_now.cloud; }
    double rain() const { return m_now.rain; }
    double fog() const { return m_now.fog; }
    // Session progress as the plant shows it: rain nudges growth along, and a
    // finished session is always a finished plant.
    double growth() const { return m_progress + m_boost * (1 - m_progress); }

    // Rolls the weather for a new session on `date`.
    void begin(int seed, const QDate &date);
    void setProgress(double progress);

    static Conditions conditions(const QString &kind);
    static QString seasonOf(const QDate &date);
    static QString pickKind(QRandomGenerator &random, const QString &season);

signals:
    void changed();

private:
    void refresh();

    QString m_forced;
    QString m_season = "summer";
    QString m_from = "clear", m_to = "clear";
    Conditions m_now;
    double m_progress = 0;
    double m_boost = 0;
};
