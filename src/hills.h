#pragma once

#include <QColor>

#include "layer.h"

// The view beyond the garden: distant granite peaks, rolling ridges and a
// nearer hillside. The ridgelines come from fixed noise, so the landscape is
// always the same one; only its light changes. Each layer is hazier the
// farther away it is, fading toward the colour of the sky at the horizon.
class Hills : public PaintedLayer {
    Q_OBJECT
    // Where the horizon sits, as a fraction of the height.
    Q_PROPERTY(double horizon MEMBER m_horizon NOTIFY changed)
    // "back" draws the peaks and ridges, "front" the hillside nearest to us, so
    // something can pass between them; "all" draws both.
    Q_PROPERTY(QString part MEMBER m_part NOTIFY changed)
    // How low the snow reaches on the far peaks, as a fraction of their height.
    Q_PROPERTY(double snowline MEMBER m_snowline NOTIFY changed)
    Q_PROPERTY(QColor farColor MEMBER m_farColor NOTIFY changed)
    Q_PROPERTY(QColor midColor MEMBER m_midColor NOTIFY changed)
    Q_PROPERTY(QColor nearColor MEMBER m_nearColor NOTIFY changed)
    Q_PROPERTY(QColor hazeColor MEMBER m_hazeColor NOTIFY changed)
    // The far range is bare granite: lit rock, rock in shadow, and forest along
    // its foot. Which side of each peak is in shadow follows the sun (1 when
    // the right-hand faces are shaded), and relief is how strong the shadows
    // are, from 0 under cloud or at night to about 0.7 in clear sun.
    Q_PROPERTY(QColor shadeColor MEMBER m_shadeColor NOTIFY changed)
    Q_PROPERTY(QColor forestColor MEMBER m_forestColor NOTIFY changed)
    Q_PROPERTY(int shadowSide MEMBER m_shadowSide NOTIFY changed)
    Q_PROPERTY(double relief MEMBER m_relief NOTIFY changed)
    // How much snow the range carries, from a few high patches (0) to fully
    // white (1).
    Q_PROPERTY(double snowCover MEMBER m_snowCover NOTIFY changed)
    Q_PROPERTY(QColor snowColor MEMBER m_snowColor NOTIFY changed)

public:
    struct Look {
        double horizon = 0;
        QString part;
        double snowline = 0;
        QColor farColor, midColor, nearColor, hazeColor, shadeColor, forestColor;
        int shadowSide = 1;
        double relief = 0;
        double snowCover = 0;
        QColor snowColor;
    };

    explicit Hills(QQuickItem *parent = nullptr);

    static void paint(QPainter &painter, const Look &look, double width, double height);

signals:
    void changed();

protected:
    Painting painting() const override;

private:
    double m_horizon = 0.88;
    QString m_part = "all";
    double m_snowline = 0.65;
    QColor m_farColor{"#6b7f9c"};
    QColor m_midColor{"#46605a"};
    QColor m_nearColor{"#30493b"};
    QColor m_hazeColor{"#9fb7c4"};
    QColor m_shadeColor{"#4d5a6c"};
    QColor m_forestColor{"#2f3e35"};
    int m_shadowSide = 1;
    double m_relief = 0.5;
    double m_snowCover = 0.3;
    QColor m_snowColor{"#f2f6fa"};
};
