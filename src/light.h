#pragma once

#include <QColor>
#include <QDateTime>
#include <QObject>
#include <QTimer>

// The sky at one moment, worked out from the clock: where the sun or moon is
// and how everything is lit.
struct Sky {
    // The sky's colour overhead and at the horizon.
    QColor top, bottom;
    // Height of the sun above the horizon from -1 to 1, negative at night.
    double elevation = 0;
    // How far across the sky the sun or moon is, 0 at the left, 1 at the right.
    double sunX = 0.5, moonX = 0.5;
    // How high the moon is, 0 at the horizon and 1 overhead.
    double moonElevation = 0;
    // Whether the sun is above the horizon; when it is not, the moon is up.
    bool sunUp = false;
    // 0 in darkness, 1 in full daylight; used to dim the plant and the ground.
    double ambient = 1;
    // How much the stars show, 0 by day to 1 in full dark.
    double stars = 0;
    // Peaks near sunrise and sunset, when the light turns golden.
    double warmth = 0;
    // The colour that golden light has, drawn toward the desktop accent.
    QColor warmTint;
};

// The light in the garden, from the clock alone. There is no location lookup:
// day length follows the calendar for a mid-northern latitude, which is close
// enough for a toy and keeps the app offline.
class Light : public QObject {
    Q_OBJECT
    Q_PROPERTY(QColor skyTop READ skyTop NOTIFY changed)
    Q_PROPERTY(QColor skyBottom READ skyBottom NOTIFY changed)
    Q_PROPERTY(double elevation READ elevation NOTIFY changed)
    Q_PROPERTY(double sunX READ sunX NOTIFY changed)
    Q_PROPERTY(double moonX READ moonX NOTIFY changed)
    Q_PROPERTY(double moonElevation READ moonElevation NOTIFY changed)
    Q_PROPERTY(bool sunUp READ sunUp NOTIFY changed)
    Q_PROPERTY(double ambient READ ambient NOTIFY changed)
    Q_PROPERTY(double stars READ stars NOTIFY changed)
    Q_PROPERTY(double warmth READ warmth NOTIFY changed)
    Q_PROPERTY(QColor warmTint READ warmTint NOTIFY changed)
    Q_PROPERTY(QColor nightTint READ nightTint NOTIFY changed)

public:
    // A valid `fixed` time freezes the clock, for screenshots and tests.
    explicit Light(QDateTime fixed = {}, QObject *parent = nullptr);

    // The sky now, field by field; see Sky for what each one means.
    QColor skyTop() const { return m_sky.top; }
    QColor skyBottom() const { return m_sky.bottom; }
    double elevation() const { return m_sky.elevation; }
    double sunX() const { return m_sky.sunX; }
    double moonX() const { return m_sky.moonX; }
    double moonElevation() const { return m_sky.moonElevation; }
    bool sunUp() const { return m_sky.sunUp; }
    double ambient() const { return m_sky.ambient; }
    double stars() const { return m_sky.stars; }
    double warmth() const { return m_sky.warmth; }
    // The golden colour, with an alpha that rises with warmth, for tinting
    // what the low sun lights.
    QColor warmTint() const;
    // A translucent blue that dims whatever it is applied to at night.
    QColor nightTint() const;

    // Dusk warms toward the desktop accent.
    void setAccent(const QColor &accent);

    // The sky at a moment, its dusk warmed toward `accent`.
    static Sky sky(const QDateTime &when, const QColor &accent);
    // Sunrise and sunset as hours of the day, for the date.
    static void sunTimes(const QDate &date, double &sunrise, double &sunset);

signals:
    // The sky has moved on, as it does twice a minute, or the accent changed.
    void changed();

private:
    void update();

    QDateTime m_fixed;
    QColor m_accent{"#FFD60A"};
    Sky m_sky;
    QTimer m_timer;
};
