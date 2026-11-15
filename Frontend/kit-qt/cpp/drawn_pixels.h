// EmuSen.Kit.DrawnPixels: the device pixels its box covers in its window, kept as it and its ancestors move or scale (EmuSen_Kit.md §7.111).
#pragma once

#include <QtCore/QSize>
#include <QtQml/qqmlregistration.h>
#include <QtQuick/QQuickItem>

class DrawnPixels : public QQuickItem {
    Q_OBJECT
    QML_ELEMENT
    QML_ADDED_IN_VERSION(1, 4)
    Q_PROPERTY(QSize pixels READ pixels NOTIFY pixelsChanged)

public:
    explicit DrawnPixels(QQuickItem* parent = nullptr);

    QSize pixels() const { return m_pixels; }

Q_SIGNALS:
    void pixelsChanged();

protected:
    void componentComplete() override;
    void itemChange(ItemChange change, const ItemChangeData& value) override;
    void geometryChange(const QRectF& now, const QRectF& was) override;

private:
    void measure();

    QSize m_pixels = QSize(0, 0);
};
