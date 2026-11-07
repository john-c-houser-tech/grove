#pragma once

#include <QColor>
#include <QtQml/qqmlregistration.h>

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
    QML_ELEMENT
    // The horizon, as a fraction of the height, and where the field's far edge
    // meets the meadow, in pixels from the top (by default, near the bottom).
    Q_PROPERTY(double horizon READ horizon WRITE setHorizon NOTIFY changed)
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
    // pixels from the top. They follow only the size and the horizon, so they
    // have a signal of their own: groundTop is usually bound to farEdgeY, and
    // sharing `changed` with it would make that binding loop.
    Q_PROPERTY(double farEdgeY READ farEdgeY NOTIFY viewChanged)
    Q_PROPERTY(double plantBaseY READ plantBaseY NOTIFY viewChanged)

public:
    // Everything paint() draws from, copied whole to the drawing thread.
    struct Look {
        double horizon = 0, groundTop = 0, vanishX = 0, plantX = 0;
        QString part;
        QColor soilColor, hazeColor;
        double snow = 0;
        bool frost = false;
    };

    explicit Field(QQuickItem *parent = nullptr);

    double horizon() const { return m_horizon; }
    void setHorizon(double horizon);
    double groundTop() const { return m_groundTop >= 0 ? m_groundTop : height() * 0.874; }
    void setGroundTop(double groundTop);
    double farEdgeY() const;
    double plantBaseY() const;

    // Draws the field at `width` by `height` pixels. It runs on the drawing
    // thread, and tests call it directly.
    static void paint(QPainter &painter, const Look &look, double width, double height);

signals:
    // A property changed, and the layer is drawn again.
    void changed();
    void viewChanged();

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
