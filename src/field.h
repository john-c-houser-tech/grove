#pragma once

#include <QColor>

#include "layer.h"

// The ploughed field in the foreground, seen in true perspective by someone
// standing at the edge of it, eyes 1.6 m above the ground.
//
// The rows run straight away from the viewer and meet at the horizon. They
// are rounded ridges 70 cm apart and 20 cm high, with a narrow trough between
// them, wavering a little along their length as ploughed rows do. The plant is
// a sapling about 2.5 m tall, standing in the furrow between two rows, 4 m
// away. Every size here follows from those numbers, so the rows, the plant
// and the trunk agree.
class Field : public PaintedLayer {
    Q_OBJECT
    // The horizon, as a fraction of the height, and where the field's far edge
    // meets the meadow, in pixels from the top (by default, near the bottom).
    Q_PROPERTY(double horizon MEMBER m_horizon NOTIFY changed)
    Q_PROPERTY(double groundTop READ groundTop WRITE setGroundTop NOTIFY changed)
    // Where the rows meet on the horizon, and where the plant stands, as
    // fractions of the width.
    Q_PROPERTY(double vanishX MEMBER m_vanishX NOTIFY changed)
    Q_PROPERTY(double plantX MEMBER m_plantX NOTIFY changed)
    // "far" draws the ground and the rows beyond the plant, "near" the rows in
    // front of it, which are drawn over the plant: a ridge between the eye and
    // the trunk hides the trunk's foot, as it would in a real field.
    Q_PROPERTY(QString part MEMBER m_part NOTIFY changed)
    Q_PROPERTY(QColor soilColor MEMBER m_soilColor NOTIFY changed)
    // The sky at the horizon, for the haze over the far side of the field.
    Q_PROPERTY(QColor hazeColor MEMBER m_hazeColor NOTIFY changed)
    // Snow lying on the field, from 0 to 1, and frost on the crests in winter.
    Q_PROPERTY(double snow MEMBER m_snow NOTIFY changed)
    Q_PROPERTY(bool frost MEMBER m_frost NOTIFY changed)
    // Where the field ends, out at 40 m, and where the plant's foot is, in
    // pixels from the top.
    Q_PROPERTY(double farEdgeY READ farEdgeY NOTIFY changed)
    Q_PROPERTY(double plantBaseY READ plantBaseY NOTIFY changed)

public:
    struct Look {
        double horizon, groundTop, vanishX, plantX;
        QString part;
        QColor soilColor, hazeColor;
        double snow;
        bool frost;
    };

    explicit Field(QQuickItem *parent = nullptr);

    double groundTop() const { return m_groundTop >= 0 ? m_groundTop : height() * 0.874; }
    void setGroundTop(double groundTop);
    double farEdgeY() const;
    double plantBaseY() const;

    static void paint(QPainter &painter, const Look &look, double width, double height);

signals:
    void changed();

protected:
    Painting painting() const override;
    void geometryChange(const QRectF &newGeometry, const QRectF &oldGeometry) override;

private:
    double m_horizon = 0.6;
    double m_groundTop = -1;
    double m_vanishX = 0.5;
    double m_plantX = 0.33;
    QString m_part = "far";
    QColor m_soilColor{"#2a2018"};
    QColor m_hazeColor{"#9fb7c4"};
    double m_snow = 0;
    bool m_frost = false;
};
