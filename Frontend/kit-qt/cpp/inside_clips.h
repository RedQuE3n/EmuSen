// EmuSen.Kit.InsideClips: whether its box meets what every clipping ancestor leaves shown, kept as ancestors move (EmuSen_Kit.md §7.34).
#pragma once

#include <QtQml/qqmlregistration.h>
#include <QtQuick/QQuickItem>

class InsideClips : public QQuickItem {
    Q_OBJECT
    QML_ELEMENT
    Q_PROPERTY(bool inside READ inside NOTIFY insideChanged)

public:
    explicit InsideClips(QQuickItem* parent = nullptr);

    bool inside() const { return m_inside; }

Q_SIGNALS:
    void insideChanged();

protected:
    void itemChange(ItemChange change, const ItemChangeData& value) override;
    void geometryChange(const QRectF& now, const QRectF& was) override;

private:
    void schedule();
    void measure();

    bool m_inside = true;
    bool m_pending = false;
};
