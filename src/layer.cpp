#include "layer.h"

#include <QPainter>
#include <QQuickWindow>
#include <QSGSimpleTextureNode>
#include <QtConcurrent>
#include <QtMath>
#include <algorithm>

PaintedLayer::PaintedLayer(QQuickItem *parent) : QQuickItem(parent) {
    setFlag(ItemHasContents);
    connect(this, &PaintedLayer::coveredFromChanged, this, &PaintedLayer::scheduleRender);
    connect(&m_watcher, &QFutureWatcherBase::finished, this, [this] {
        m_ready = m_watcher.result();
        m_hasReady = true;
        update();
        if (m_again) {
            m_again = false;
            render();
        }
    });
}

void PaintedLayer::scheduleRender() {
    if (m_scheduled)
        return;
    m_scheduled = true;
    QMetaObject::invokeMethod(this, &PaintedLayer::render, Qt::QueuedConnection);
}

void PaintedLayer::render() {
    m_scheduled = false;
    // One drawing at a time; a change meanwhile is drawn when it is done.
    if (m_watcher.isRunning()) {
        m_again = true;
        return;
    }
    if (window() == nullptr || width() <= 0 || height() <= 0)
        return;
    const Painting painting = this->painting();
    const QSizeF size = this->size();
    const double coveredFrom = m_coveredFrom;
    const qreal ratio = window()->effectiveDevicePixelRatio();
    m_watcher.setFuture(QtConcurrent::run([=] { return draw(painting, size, coveredFrom, ratio); }));
}

PaintedLayer::Picture PaintedLayer::draw(const Painting &painting, const QSizeF &size, double coveredFrom,
                                         qreal devicePixelRatio) {
    Picture picture;
    const double bottom = coveredFrom >= 0 ? std::min(size.height(), coveredFrom) : size.height();
    const QSize pixels(qCeil(size.width() * devicePixelRatio), qCeil(bottom * devicePixelRatio));
    if (pixels.isEmpty())
        return picture;
    QImage image(pixels, QImage::Format_ARGB32_Premultiplied);
    image.fill(Qt::transparent);
    {
        QPainter painter(&image);
        painter.setRenderHint(QPainter::Antialiasing);
        painter.scale(devicePixelRatio, devicePixelRatio);
        painting(painter, size.width(), size.height());
    }

    // Keep only the rows with something in them.
    const auto empty = [&image](int y) {
        const uchar *row = image.constScanLine(y);
        return std::all_of(row, row + static_cast<ptrdiff_t>(image.width()) * 4, [](uchar byte) { return byte == 0; });
    };
    int top = 0, end = image.height();
    while (top < end && empty(top))
        ++top;
    while (end > top && empty(end - 1))
        --end;
    if (top == end)
        return picture;
    picture.image = image.copy(0, top, image.width(), end - top);
    picture.image.setDevicePixelRatio(devicePixelRatio);
    picture.top = top / devicePixelRatio;
    return picture;
}

QSGNode *PaintedLayer::updatePaintNode(QSGNode *old, UpdatePaintNodeData * /*data*/) {
    auto *node = dynamic_cast<QSGSimpleTextureNode *>(old);
    if (!m_hasReady)
        return node;
    m_hasReady = false;
    const Picture picture = std::exchange(m_ready, {});
    if (picture.image.isNull()) {
        delete node;
        return nullptr;
    }
    if (node == nullptr) {
        node = new QSGSimpleTextureNode;
        node->setOwnsTexture(true);
        node->setFiltering(QSGTexture::Linear);
    }
    // The texture takes its own copy, and this one goes when `picture` does.
    node->setTexture(window()->createTextureFromImage(picture.image));
    const qreal ratio = picture.image.devicePixelRatio();
    node->setRect(QRectF(0, picture.top, picture.image.width() / ratio, picture.image.height() / ratio));
    return node;
}

void PaintedLayer::geometryChange(const QRectF &newGeometry, const QRectF &oldGeometry) {
    QQuickItem::geometryChange(newGeometry, oldGeometry);
    if (newGeometry.size() != oldGeometry.size())
        scheduleRender();
}

void PaintedLayer::itemChange(ItemChange change, const ItemChangeData &value) {
    QQuickItem::itemChange(change, value);
    if (change == ItemSceneChange || change == ItemDevicePixelRatioHasChanged)
        scheduleRender();
}
