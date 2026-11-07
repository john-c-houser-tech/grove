#include "trackbed.h"

#include <QLinearGradient>
#include <QPainter>
#include <QPainterPath>
#include <vector>

#include "terrain.h"

using namespace terrain;

namespace {

QPointF curve(const QPointF &p0, const QPointF &p1, const QPointF &p2, const QPointF &p3, double t) {
    const double u = 1 - t;
    return {u * u * u * p0.x() + 3 * u * u * t * p1.x() + 3 * u * t * t * p2.x() + t * t * t * p3.x(),
            u * u * u * p0.y() + 3 * u * u * t * p1.y() + 3 * u * t * t * p2.y() + t * t * t * p3.y()};
}

// A road along a curve, seen from low down: as wide as the crossing where it
// runs straight toward or away from us, thin where it turns to run across the
// view, and narrower with distance (farther).
struct Road {
    std::vector<QPointF> upper, lower;
    QPainterPath outline() const {
        QPainterPath path;
        path.setFillRule(Qt::WindingFill);
        path.moveTo(upper[0]);
        for (size_t a = 1; a < upper.size(); ++a)
            path.lineTo(upper[a]);
        for (size_t b = lower.size(); b-- > 0;)
            path.lineTo(lower[b]);
        path.closeSubpath();
        return path;
    }
};

Road road(const QPointF &p0, const QPointF &p1, const QPointF &p2, const QPointF &p3, double farther, double H) {
    Road r;
    for (int i = 0; i <= 40; ++i) {
        const double t = i / 40.0;
        const QPointF at = curve(p0, p1, p2, p3, t);
        const QPointF next = curve(p0, p1, p2, p3, std::min(1.0, t + 0.01));
        const QPointF prev = curve(p0, p1, p2, p3, std::max(0.0, t - 0.01));
        const double dx = next.x() - prev.x(), dy = next.y() - prev.y();
        double len = std::sqrt(dx * dx + dy * dy);
        if (len == 0)
            len = 1;
        const double toward = std::abs(dy) / len;
        const double shrink = 1 - farther * t;
        const double half = H * (0.006 + 0.019 * toward) * shrink;
        const double nx = -dy / len, ny = dx / len;
        r.upper.emplace_back(at.x() + nx * half, at.y() + ny * half * (1 - toward) - (1 - toward) * half * 0.2);
        r.lower.emplace_back(at.x() - nx * half, at.y() - ny * half * (1 - toward) + (1 - toward) * half * 0.2);
    }
    return r;
}

} // namespace

TrackBed::TrackBed(QQuickItem *parent) : PaintedLayer(parent) {
    connect(this, &TrackBed::changed, this, &TrackBed::scheduleRender);
}

PaintedLayer::Painting TrackBed::painting() const {
    const Look look{m_horizon,   m_crossingX, m_floorColor, m_ballastColor, m_railColor,
                    m_roadColor, m_cutColor,  m_fillColor,  m_hillColor,    m_hazeColor};
    return [look](QPainter &p, double W, double H) { paint(p, look, W, H); };
}

void TrackBed::paint(QPainter &p, const Look &look, double W, double H) {
    if (W <= 0 || H <= 0)
        return;
    const double ty = trackY(look.horizon, H);
    const auto line = [&] {
        QPainterPath path;
        path.setFillRule(Qt::WindingFill);
        path.moveTo(0, ty);
        for (int i = 1; 4.0 * i <= W; ++i)
            path.lineTo(4.0 * i, ty);
        return path;
    };
    {
        QPainterPath floor = line();
        floor.lineTo(W, H);
        floor.lineTo(0, H);
        floor.closeSubpath();
        p.fillPath(floor, look.floorColor);
    }
    // A dirt road. On this side it comes a little toward us, then curves away
    // to the right along the valley floor and goes behind the hill there.
    // Beyond the rails it climbs the hillside to the left, where the hill is
    // higher, and goes into a cut in the slope. The signal stands on its
    // right-hand verge.
    const double post = W * look.crossingX, cy = ty;
    const double gap = H * 0.006, crossing = H * 0.05;
    const double centre = post - gap - crossing / 2;

    // Near side.
    p.fillPath(road({centre, cy}, {centre, cy + H * 0.03}, {centre + W * 0.1, cy + H * 0.034},
                    {W * 1.08, cy + H * 0.028}, 0, H)
                   .outline(),
               look.roadColor);
    // Far side: the road is cut into the hillside as it climbs, like an inslope
    // road. It leaves the rails at full width, bends left and climbs across the
    // face of the shoulder of hill, with a cut slope of bare earth above it and
    // a fill slope below, then curves around the shoulder and out of sight.
    const QPointF far0{centre, cy}, far1{centre - W * 0.006, cy - H * 0.02};
    const QPointF far2{centre - W * 0.06, cy - H * 0.03}, far3{centre - W * 0.13, cy - H * 0.056};
    const Road edges = road(far0, far1, far2, far3, 0.7, H);
    const int n = static_cast<int>(edges.upper.size());
    const auto along = [n](int i) { return static_cast<double>(i) / (n - 1); };
    // The fill slope: earth tipped down the hill below the road, deepest where
    // the road is farthest out on the slope.
    {
        const int fillFrom = static_cast<int>(std::floor(n * 0.2));
        QPainterPath fill;
        fill.setFillRule(Qt::WindingFill);
        fill.moveTo(edges.lower[fillFrom]);
        for (int i = fillFrom + 1; i < n; ++i)
            fill.lineTo(edges.lower[i]);
        for (int k = n - 1; k >= fillFrom; --k) {
            const double tf = (along(k) - along(fillFrom)) / (1 - along(fillFrom));
            const double drop = H * 0.012 * std::sin(M_PI * std::min(1.0, tf * 1.1)) * (1 - 0.5 * along(k));
            fill.lineTo(edges.lower[k].x() + W * 0.004 * tf, edges.lower[k].y() + drop);
        }
        fill.closeSubpath();
        p.fillPath(fill, look.fillColor);
    }
    // The road itself.
    p.fillPath(edges.outline(), look.roadColor);
    // The cut slope: the bank left where the road was dug into the hill,
    // tallest partway up and gone where the road leaves the flat.
    const int cutFrom = static_cast<int>(std::floor(n * 0.25));
    std::vector<QPointF> cutTop;
    for (int c = cutFrom; c < n; ++c) {
        const double tc = (along(c) - along(cutFrom)) / (1 - along(cutFrom));
        const double tall = H * (0.004 + 0.02 * std::sin(M_PI * 0.5 * std::min(1.0, tc * 1.6))) * (1 - 0.35 * tc);
        cutTop.emplace_back(edges.upper[c].x() - W * 0.003 * tc, edges.upper[c].y() - tall);
    }
    {
        QPainterPath cut;
        cut.setFillRule(Qt::WindingFill);
        cut.moveTo(edges.upper[cutFrom]);
        for (int u2 = cutFrom + 1; u2 < n; ++u2)
            cut.lineTo(edges.upper[u2]);
        for (size_t v2 = cutTop.size(); v2-- > 0;)
            cut.lineTo(cutTop[v2]);
        cut.closeSubpath();
        p.fillPath(cut, look.cutColor);
    }
    // Rain has scored the cut with short runnels down its face.
    {
        p.setPen(pen(
            rgba(look.cutColor.redF() * 0.82, look.cutColor.greenF() * 0.82, look.cutColor.blueF() * 0.82, 0.45), 1));
        p.setBrush(Qt::NoBrush);
        for (int r2 = 3; r2 < static_cast<int>(cutTop.size()) - 2; r2 += 5) {
            const QPointF topPt = cutTop[r2], footPt = edges.upper[cutFrom + r2];
            QPainterPath runnel;
            runnel.moveTo(topPt.x(), topPt.y() + (footPt.y() - topPt.y()) * 0.15);
            runnel.lineTo(topPt.x() + (footPt.x() - topPt.x()) * 0.7 + 1, topPt.y() + (footPt.y() - topPt.y()) * 0.75);
            p.drawPath(runnel);
        }
    }
    // Beyond the near hill the road keeps climbing, and comes back into view
    // higher up and farther off: a thin line traversing the farther hill to the
    // saddle in its ridge, where it goes over the pass. It is drawn before the
    // near hill, so only the stretch above it shows.
    const QPointF saddle{W * 0.25, H * look.horizon - midRidge(0.25, H) + H * 0.002};
    // It comes out from behind the near hill's top and rises steadily across
    // the farther hill to the pass.
    const QPointF hidden{curve(far0, far1, far2, far3, 0.86).x() - W * 0.12, H * look.horizon - H * 0.128};
    QPainterPath upper;
    upper.moveTo(hidden);
    upper.cubicTo(hidden.x() - W * 0.06, hidden.y() - H * 0.006, saddle.x() + W * 0.07, saddle.y() + H * 0.01,
                  saddle.x(), saddle.y());
    // The sliver of cut above it, then the road.
    p.save();
    p.translate(0, -H * 0.003);
    p.strokePath(upper, pen(darker(look.cutColor, 1.1), std::max(1.0, H * 0.0025)));
    p.restore();
    p.strokePath(upper, pen(mix(look.roadColor, look.hazeColor, 0.2), std::max(2.0, H * 0.0045)));

    // A nearer hill stands to the left of the cut. Its right flank comes
    // steeply down across the farther hill and the road's last stretch, so the
    // road passes out of sight behind it. Being nearer, it is darker and less
    // hazy than the hill behind.
    const QPointF bend = curve(far0, far1, far2, far3, 0.86);
    const double y0 = H * look.horizon;
    // A modest rounded hill: it rises out of the valley on its left to a gentle
    // top, then comes down steeply across the road's last stretch.
    const double crest = y0 - H * 0.155;
    const double topX = bend.x() - W * 0.11;
    const double footLeft = bend.x() - W * 0.34;
    const QColor nearTop = darker(look.hillColor, 1.12);
    QLinearGradient nearFill(0, crest, 0, cy);
    nearFill.setColorAt(0, nearTop);
    nearFill.setColorAt(1, mix(nearTop, look.hazeColor, 0.25));
    QPainterPath nearHill;
    nearHill.setFillRule(Qt::WindingFill);
    nearHill.moveTo(footLeft, cy + 1);
    // The long left slope up out of the valley.
    nearHill.cubicTo(footLeft + W * 0.1, cy - H * 0.02, topX - W * 0.12, crest + H * 0.004, topX, crest);
    // Over the top and down the steep right flank to the road's end.
    nearHill.cubicTo(topX + W * 0.06, crest - H * 0.002, bend.x() - W * 0.012, bend.y() - H * 0.045,
                     bend.x() + W * 0.004, bend.y() + H * 0.008);
    // Its foot reaches the valley floor just beyond the road.
    nearHill.quadTo(bend.x() + W * 0.012, cy - H * 0.006, bend.x() + W * 0.03, cy + 1);
    QPainterPath nearHillShape = nearHill;
    nearHillShape.closeSubpath();
    p.fillPath(nearHillShape, nearFill);
    // A faint line where the near hill's edge stands against the far one.
    p.strokePath(nearHill, pen(darker(nearTop, 1.15), std::max(1.0, H * 0.0018)));
    // Ballast, then a rail catching what light there is on top of it.
    p.strokePath(line(), pen(look.ballastColor, std::max(2.0, H * 0.004)));
    p.strokePath(line(), pen(look.railColor, 1));
}
