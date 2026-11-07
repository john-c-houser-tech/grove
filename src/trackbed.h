#pragma once

#include <QColor>
#include <QtQml/qqmlregistration.h>

#include "layer.h"

// The valley floor with the railway along its far edge, the level crossing's
// dirt road, and the nearer hill the road goes behind. The hills in front
// cover the floor's lower part; the train runs between this and them.
class TrackBed : public PaintedLayer {
    Q_OBJECT
    QML_ELEMENT
    // Where the horizon sits, as a fraction of the height.
    Q_PROPERTY(double horizon MEMBER m_horizon NOTIFY changed)
    // Where the crossing stands, as a fraction of the width.
    Q_PROPERTY(double crossingX MEMBER m_crossingX NOTIFY changed)
    Q_PROPERTY(QColor floorColor MEMBER m_floorColor NOTIFY changed)
    Q_PROPERTY(QColor ballastColor MEMBER m_ballastColor NOTIFY changed)
    Q_PROPERTY(QColor railColor MEMBER m_railColor NOTIFY changed)
    Q_PROPERTY(QColor roadColor MEMBER m_roadColor NOTIFY changed)
    // Beyond the crossing: the bare earth of the cut slope above the road, the
    // tipped earth of the fill slope below it, and the hillside itself.
    Q_PROPERTY(QColor cutColor MEMBER m_cutColor NOTIFY changed)
    Q_PROPERTY(QColor fillColor MEMBER m_fillColor NOTIFY changed)
    Q_PROPERTY(QColor hillColor MEMBER m_hillColor NOTIFY changed)
    Q_PROPERTY(QColor hazeColor MEMBER m_hazeColor NOTIFY changed)

public:
    struct Look {
        double horizon = 0, crossingX = 0;
        QColor floorColor, ballastColor, railColor, roadColor, cutColor, fillColor, hillColor, hazeColor;
    };

    explicit TrackBed(QQuickItem *parent = nullptr);

    // The track is straight and level across the valley, this far down.
    static double trackY(double horizon, double height) { return height * (horizon - 0.085); }
    static void paint(QPainter &painter, const Look &look, double width, double height);

signals:
    void changed();

protected:
    Painting painting() const override;

private:
    double m_horizon = 0.88;
    double m_crossingX = 0.68;
    QColor m_floorColor{"#33503f"};
    QColor m_ballastColor{"#26382d"};
    QColor m_railColor{"#6d7f78"};
    QColor m_roadColor{"#7a7258"};
    QColor m_cutColor{"#4a4232"};
    QColor m_fillColor{"#3d3a2c"};
    QColor m_hillColor{"#44605a"};
    QColor m_hazeColor{"#9fb7c4"};
};
