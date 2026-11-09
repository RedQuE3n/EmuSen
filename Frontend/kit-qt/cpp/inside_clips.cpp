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
        measure();
    }
}

void InsideClips::geometryChange(const QRectF& now, const QRectF& was)
{
    QQuickItem::geometryChange(now, was);
    measure();
}

void InsideClips::measure()
{
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
