#pragma once

#include <QFutureWatcher>
#include <QImage>
#include <QQuickItem>
#include <functional>

class QPainter;

// A still picture of part of the landscape, drawn with QPainter on another
// thread and shown as a single texture. Only the rows that hold something are
// kept, and nothing is drawn below `coveredFrom`, where the ground in front
// hides it; once the picture is on the GPU the copy here is let go.
class PaintedLayer : public QQuickItem {
    Q_OBJECT
    // The height in pixels from which this layer is hidden by what stands in
    // front of it; negative when nothing does.
    Q_PROPERTY(double coveredFrom MEMBER m_coveredFrom NOTIFY coveredFromChanged)

public:
    explicit PaintedLayer(QQuickItem *parent = nullptr);

    // Draws the layer at the given size in pixels. It must hold everything it
    // needs by value, since it runs on another thread.
    using Painting = std::function<void(QPainter &painter, double width, double height)>;

    // Draws a painting as this class does, with the empty rows above and below
    // trimmed. `top` is where the image starts, in pixels; the image is null if
    // nothing was drawn.
    struct Picture {
        QImage image;
        double top = 0;
    };
    static Picture draw(const Painting &painting, const QSizeF &size, double coveredFrom, qreal devicePixelRatio);

signals:
    void coveredFromChanged();

protected:
    // A drawing of the layer as it looks now.
    virtual Painting painting() const = 0;
    // Draws the layer again once the current event is handled, however many
    // properties change meanwhile.
    void scheduleRender();

    QSGNode *updatePaintNode(QSGNode *old, UpdatePaintNodeData *data) override;
    void geometryChange(const QRectF &newGeometry, const QRectF &oldGeometry) override;
    void itemChange(ItemChange change, const ItemChangeData &value) override;

private:
    void render();

    double m_coveredFrom = -1;
    bool m_scheduled = false;
    bool m_again = false;
    QFutureWatcher<Picture> m_watcher;
    // A finished picture waiting to become a texture.
    Picture m_ready;
    bool m_hasReady = false;
};
