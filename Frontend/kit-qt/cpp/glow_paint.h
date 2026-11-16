// EmuSen.Kit.GlowPaint: a rounded box's glow or shadow on the software backend, outside the box alone, made in native code once a shape and tinted once a colour; the rule and its measures are EmuSen_Kit.md §7.113.
#pragma once

#include <QtGui/QColor>
#include <QtGui/QImage>
#include <QtQml/qqmlregistration.h>
#include <QtQuick/QQuickPaintedItem>

class GlowPaint : public QQuickPaintedItem {
    Q_OBJECT
    QML_ELEMENT
    QML_ADDED_IN_VERSION(1, 8)
    Q_PROPERTY(QColor color READ color WRITE setColor NOTIFY shapeChanged)
    Q_PROPERTY(double corner READ corner WRITE setCorner NOTIFY shapeChanged)
    Q_PROPERTY(double blur READ blur WRITE setBlur NOTIFY shapeChanged)
    Q_PROPERTY(double spread READ spread WRITE setSpread NOTIFY shapeChanged)
    Q_PROPERTY(double reach READ reach WRITE setReach NOTIFY shapeChanged)
    Q_PROPERTY(double offsetX READ offsetX WRITE setOffsetX NOTIFY shapeChanged)
    Q_PROPERTY(double offsetY READ offsetY WRITE setOffsetY NOTIFY shapeChanged)
    Q_PROPERTY(int blurs READ blurs NOTIFY counted)
    Q_PROPERTY(int paints READ paints NOTIFY counted)

public:
    explicit GlowPaint(QQuickItem* parent = nullptr);
    void paint(QPainter* painter) override;

    QColor color() const { return m_color; }
    double corner() const { return m_corner; }
    double blur() const { return m_blur; }
    double spread() const { return m_spread; }
    double reach() const { return m_reach; }
    double offsetX() const { return m_offsetX; }
    double offsetY() const { return m_offsetY; }
    int blurs() const { return m_blurs; }
    int paints() const { return m_paints; }
    void setColor(const QColor& c);
    void setCorner(double v);
    void setBlur(double v);
    void setSpread(double v);
    void setReach(double v);
    void setOffsetX(double v);
    void setOffsetY(double v);

Q_SIGNALS:
    void shapeChanged();
    void counted();

protected:
    void geometryChange(const QRectF& now, const QRectF& was) override;

private:
    void reshape();
    void makeShape();

    QColor m_color = Qt::transparent;
    double m_corner = 0, m_blur = 0, m_spread = 0, m_reach = 0, m_offsetX = 0, m_offsetY = 0;
    int m_blurs = 0, m_paints = 0;
    // The blurred shape's alpha, at 1 / m_scale of the item's size, and the same tinted in the colour last painted.
    QImage m_shape;
    QImage m_tinted;
    QColor m_tint;
    int m_scale = 1;
};
