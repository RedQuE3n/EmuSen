#include "edge_mask.h"

#include <QtGui/QLinearGradient>
#include <QtGui/QPainter>
#include <QtGui/QPixmap>
#include <QtQuick/QQuickWindow>
#include <QtQuick/QSGRenderNode>
#include <QtQuick/QSGRendererInterface>

namespace {

// Paints each faded edge's ramp with DestinationIn, so what the layer holds keeps none of its alpha at the edge and all of it a length in.
struct MaskNode : QSGRenderNode {
    EdgeMask* item = nullptr;
    QQuickWindow* window = nullptr;
    bool edges[4] = {false, false, false, false};
    qreal length = 0;
    QSizeF size;
    bool primed = false;

    void render(const RenderState* state) override
    {
        auto* painter = static_cast<QPainter*>(window->rendererInterface()->getResource(window, QSGRendererInterface::PainterResource));
        if (painter == nullptr || painter->device() == nullptr || painter->device()->devType() != QInternal::Pixmap) {
            return;
        }
        const bool alpha = static_cast<QPixmap*>(painter->device())->hasAlphaChannel();
        if (alpha != primed) {
            EdgeMask* target = item;
            QMetaObject::invokeMethod(target, [target, alpha] { target->setPrimed(alpha); }, Qt::QueuedConnection);
        }
        if (length <= 0) {
            return;
        }
        painter->setTransform(matrix()->toTransform());
        painter->setOpacity(1.0);
        if (state->clipRegion() != nullptr) {
            painter->setClipRegion(*state->clipRegion(), Qt::IntersectClip);
        }
        painter->setCompositionMode(QPainter::CompositionMode_DestinationIn);
        const qreal w = size.width();
        const qreal h = size.height();
        ramp(painter, edges[0], QPointF(0, 0), QPointF(0, length), QRectF(0, 0, w, length));
        ramp(painter, edges[1], QPointF(0, h), QPointF(0, h - length), QRectF(0, h - length, w, length));
        ramp(painter, edges[2], QPointF(0, 0), QPointF(length, 0), QRectF(0, 0, length, h));
        ramp(painter, edges[3], QPointF(w, 0), QPointF(w - length, 0), QRectF(w - length, 0, length, h));
    }

    static void ramp(QPainter* painter, bool shown, QPointF edge, QPointF in, const QRectF& band)
    {
        if (!shown) {
            return;
        }
        QLinearGradient gradient(edge, in);
        gradient.setColorAt(0, QColor(0, 0, 0, 0));
        gradient.setColorAt(1, QColor(0, 0, 0, 255));
        painter->fillRect(band, gradient);
    }

    StateFlags changedStates() const override { return {}; }
    RenderingFlags flags() const override { return BoundedRectRendering; }
    QRectF rect() const override { return QRectF(QPointF(0, 0), size); }
};

}

EdgeMask::EdgeMask(QQuickItem* parent) : QQuickItem(parent)
{
    setFlag(ItemHasContents, true);
}

void EdgeMask::setEdge(int edge, bool on)
{
    if (m_edges[edge] == on) {
        return;
    }
    m_edges[edge] = on;
    update();
    Q_EMIT edgesChanged();
}

void EdgeMask::setLength(double length)
{
    if (qFuzzyCompare(m_length, length)) {
        return;
    }
    m_length = length;
    update();
    Q_EMIT edgesChanged();
}

void EdgeMask::setPrimed(bool primed)
{
    if (m_primed == primed) {
        return;
    }
    m_primed = primed;
    update();
    Q_EMIT primedChanged();
}

void EdgeMask::itemChange(ItemChange change, const ItemChangeData& value)
{
    // A layer made again (shown again, or on another window) starts without an alpha channel.
    if (change == ItemVisibleHasChanged || change == ItemSceneChange || change == ItemDevicePixelRatioHasChanged) {
        setPrimed(false);
    }
    QQuickItem::itemChange(change, value);
}

void EdgeMask::geometryChange(const QRectF& now, const QRectF& was)
{
    QQuickItem::geometryChange(now, was);
    if (now.size() != was.size()) {
        setPrimed(false);
    }
    update();
}

QSGNode* EdgeMask::updatePaintNode(QSGNode* old, UpdatePaintNodeData*)
{
    auto* node = static_cast<MaskNode*>(old);
    if (node == nullptr) {
        node = new MaskNode;
    }
    node->item = this;
    node->window = window();
    for (int i = 0; i < 4; ++i) {
        node->edges[i] = m_edges[i];
    }
    node->length = m_length;
    node->size = size();
    node->primed = m_primed;
    node->markDirty(QSGNode::DirtyMaterial);
    return node;
}
