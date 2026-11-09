#include "inside_clips.h"

InsideClips::InsideClips(QQuickItem* parent) : QQuickItem(parent)
{
    // Told when any ancestor moves, scrolls or scales, not only this item (Qt's ItemTransformHasChanged).
    setFlag(ItemObservesViewport, true);
}

void InsideClips::itemChange(ItemChange change, const ItemChangeData& value)
{
    QQuickItem::itemChange(change, value);
    if (change == ItemTransformHasChanged || change == ItemSceneChange || change == ItemParentHasChanged) {
        schedule();
    }
}

void InsideClips::geometryChange(const QRectF& now, const QRectF& was)
{
    QQuickItem::geometryChange(now, was);
    schedule();
}

// Measured from the event loop, after the bindings that moved it have settled: its answer hides its Shape, whose bounds size this item.
void InsideClips::schedule()
{
    if (!m_pending) {
        m_pending = true;
        QMetaObject::invokeMethod(this, &InsideClips::measure, Qt::QueuedConnection);
    }
}

void InsideClips::measure()
{
    m_pending = false;
    bool inside = true;
    const QRectF box(0, 0, width(), height());
    for (QQuickItem* ancestor = parentItem(); ancestor != nullptr && inside; ancestor = ancestor->parentItem()) {
        if (ancestor->clip()) {
            inside = mapRectToItem(ancestor, box).intersects(ancestor->clipRect());
        }
    }
    if (inside != m_inside) {
        m_inside = inside;
        Q_EMIT insideChanged();
    }
}
