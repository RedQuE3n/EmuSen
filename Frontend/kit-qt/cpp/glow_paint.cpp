#include "glow_paint.h"

#include <QtGui/QPainter>
#include <QtGui/QPainterPath>
#include <cmath>
#include <vector>

GlowPaint::GlowPaint(QQuickItem* parent) : QQuickPaintedItem(parent)
{
    setAntialiasing(true);
}

#define GLOW_SETTER(name, member)            \
    void GlowPaint::name(double v)            \
    {                                        \
        if (member == v)                     \
            return;                          \
        member = v;                          \
        reshape();                           \
    }
GLOW_SETTER(setCorner, m_corner)
GLOW_SETTER(setBlur, m_blur)
GLOW_SETTER(setSpread, m_spread)
GLOW_SETTER(setReach, m_reach)
GLOW_SETTER(setOffsetX, m_offsetX)
GLOW_SETTER(setOffsetY, m_offsetY)
#undef GLOW_SETTER

void GlowPaint::setColor(const QColor& c)
{
    if (m_color == c)
        return;
    m_color = c;
    Q_EMIT shapeChanged();
    update();
}

void GlowPaint::geometryChange(const QRectF& now, const QRectF& was)
{
    QQuickPaintedItem::geometryChange(now, was);
    if (now.size() != was.size())
        reshape();
}

void GlowPaint::reshape()
{
    m_shape = QImage();
    Q_EMIT shapeChanged();
    update();
}

// One pass of a box blur of radius r over each row of an alpha image held in 1/256ths, as a running sum, rounded.
static void boxRows(std::vector<int>& a, int w, int h, int r)
{
    if (r <= 0)
        return;
    std::vector<int> row(w);
    const int span = 2 * r + 1;
    for (int y = 0; y < h; ++y) {
        int* line = a.data() + y * w;
        long sum = 0;
        for (int i = -r; i <= r; ++i)
            sum += (i >= 0 && i < w) ? line[i] : 0;
        for (int x = 0; x < w; ++x) {
            row[x] = int((sum + span / 2) / span);
            const int out = x - r, in = x + r + 1;
            sum += (in < w ? line[in] : 0) - (out >= 0 ? line[out] : 0);
        }
        std::copy(row.begin(), row.end(), line);
    }
}

static void transpose(const std::vector<int>& a, std::vector<int>& b, int w, int h)
{
    for (int y = 0; y < h; ++y)
        for (int x = 0; x < w; ++x)
            b[x * h + y] = a[y * w + x];
}

void GlowPaint::makeShape()
{
    // The CSS rule: the box grown by the spread, its corners grown as CSS Backgrounds 3 §6.1.1 grows them, moved by the offsets,
    // blurred by a Gaussian of deviation blur / 2, here three box blurs of 2 round(blur / 2) + 1 as Qt's Context2D shadow makes
    // it (EmuSen_Kit.md §7.113). A wide blur is made at a smaller scale, a box of 16 pixels or more there, whose radius is the
    // one whose three passes have the deviation's variance, r (r + 1) = (deviation / scale)².
    const double deviation = m_blur / 2;
    m_scale = std::max(1, int(std::floor(deviation / 16)));
    const int w = std::max(1, int(std::ceil(width() / m_scale))), h = std::max(1, int(std::ceil(height() / m_scale)));
    QImage drawn(w, h, QImage::Format_ARGB32_Premultiplied);
    drawn.fill(Qt::transparent);
    {
        QPainter p(&drawn);
        p.setRenderHint(QPainter::Antialiasing);
        p.scale(1.0 / m_scale, 1.0 / m_scale);
        const double boxW = width() - 2 * m_reach, boxH = height() - 2 * m_reach;
        const double corner = std::min(m_corner, std::min(boxW, boxH) / 2);
        const double share = m_spread > 0 && corner < m_spread ? 1 + std::pow(corner / m_spread - 1, 3) : 1;
        const double rounded = corner + m_spread * share;
        QPainterPath grown;
        grown.addRoundedRect(QRectF(m_reach - m_spread + m_offsetX, m_reach - m_spread + m_offsetY, boxW + 2 * m_spread, boxH + 2 * m_spread), rounded, rounded);
        p.fillPath(grown, Qt::black);
    }
    std::vector<int> a(size_t(w) * h), t(size_t(w) * h);
    for (int y = 0; y < h; ++y) {
        const QRgb* line = reinterpret_cast<const QRgb*>(drawn.constScanLine(y));
        for (int x = 0; x < w; ++x)
            a[y * w + x] = qAlpha(line[x]) * 256;
    }
    const double scaled = deviation / m_scale;
    const int r = m_scale == 1 ? int(std::lround(deviation)) : int(std::lround((std::sqrt(1 + 4 * scaled * scaled) - 1) / 2));
    for (int pass = 0; pass < 3; ++pass)
        boxRows(a, w, h, r);
    transpose(a, t, w, h);
    for (int pass = 0; pass < 3; ++pass)
        boxRows(t, h, w, r);
    transpose(t, a, h, w);
    m_shape = QImage(w, h, QImage::Format_Alpha8);
    for (int y = 0; y < h; ++y) {
        uchar* line = m_shape.scanLine(y);
        for (int x = 0; x < w; ++x)
            line[x] = uchar(std::clamp((a[y * w + x] + 128) / 256, 0, 255));
    }
    m_tinted = QImage();
    ++m_blurs;
    Q_EMIT counted();
}

void GlowPaint::paint(QPainter* painter)
{
    if (m_color.alpha() == 0 || width() <= 2 * m_reach || height() <= 2 * m_reach)
        return;
    if (m_shape.isNull())
        makeShape();
    if (m_tinted.isNull() || m_tint != m_color) {
        // The kept shape in the colour: the colour where the shape is, at the colour's alpha times the shape's.
        m_tinted = QImage(m_shape.size(), QImage::Format_ARGB32_Premultiplied);
        m_tinted.fill(Qt::transparent);
        QPainter p(&m_tinted);
        p.drawImage(0, 0, m_shape);
        p.setCompositionMode(QPainter::CompositionMode_SourceIn);
        p.fillRect(m_tinted.rect(), m_color);
        m_tint = m_color;
    }
    // Outside the box alone, as an outer box-shadow is drawn (CSS Backgrounds 3 §6.1.1).
    const double boxW = width() - 2 * m_reach, boxH = height() - 2 * m_reach;
    const double corner = std::min(m_corner, std::min(boxW, boxH) / 2);
    QPainterPath outside;
    outside.addRect(QRectF(0, 0, width(), height()));
    outside.addRoundedRect(QRectF(m_reach, m_reach, boxW, boxH), corner, corner);
    outside.setFillRule(Qt::OddEvenFill);
    painter->setRenderHint(QPainter::Antialiasing);
    painter->setRenderHint(QPainter::SmoothPixmapTransform);
    painter->setClipPath(outside);
    painter->drawImage(QRectF(0, 0, m_shape.width() * m_scale, m_shape.height() * m_scale), m_tinted);
    ++m_paints;
    Q_EMIT counted();
}
