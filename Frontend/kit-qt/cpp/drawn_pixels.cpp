#include "drawn_pixels.h"

#include <QtCore/QtMath>
#include <QtQuick/QQuickWindow>

DrawnPixels::DrawnPixels(QQuickItem* parent) : QQuickItem(parent)
{
    // Told when any ancestor moves or scales, not only this item (Qt's ItemTransformHasChanged), as InsideClips is.
    setFlag(ItemObservesViewport, true);
}

void DrawnPixels::componentComplete()
{
    QQuickItem::componentComplete();
    measure();
}

void DrawnPixels::itemChange(ItemChange change, const ItemChangeData& value)
{
    QQuickItem::itemChange(change, value);
    if (change == ItemTransformHasChanged || change == ItemSceneChange || change == ItemParentHasChanged || change == ItemDevicePixelRatioHasChanged) {
        measure();
    }
}

void DrawnPixels::geometryChange(const QRectF& now, const QRectF& was)
{
    QQuickItem::geometryChange(now, was);
    measure();
}

// The box mapped through every ancestor's scale and transform to the scene, times the window's device pixel ratio; none outside a window.
void DrawnPixels::measure()
{
    QSize pixels(0, 0);
    if (QQuickWindow* shown = window()) {
        const QSizeF scene = mapRectToScene(QRectF(0, 0, width(), height())).size() * shown->effectiveDevicePixelRatio();
        pixels = QSize(qCeil(scene.width()), qCeil(scene.height()));
    }
    if (pixels != m_pixels) {
        m_pixels = pixels;
        Q_EMIT pixelsChanged();
    }
}
