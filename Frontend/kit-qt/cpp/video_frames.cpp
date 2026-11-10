#include "video_frames.h"

#include <QtMultimedia/QVideoFrame>
#include <QtMultimedia/QVideoSink>
#include <QtQuick/QQuickWindow>
#include <QtQuick/QSGImageNode>
#include <QtQuick/QSGTexture>

VideoFrames::VideoFrames(QQuickItem* parent)
    : QQuickItem(parent)
    , m_sink(new QVideoSink(this))
{
    setFlag(ItemHasContents, true);
    connect(m_sink, &QVideoSink::videoFrameChanged, this, &VideoFrames::take);
}

void VideoFrames::take(const QVideoFrame& frame)
{
    if (!frame.isValid()) {
        return;
    }
    m_image = frame.toImage();
    m_fresh = true;
    ++m_frames;
    Q_EMIT framesChanged();
    update();
}

void VideoFrames::clear()
{
    m_image = QImage();
    m_fresh = true;
    m_frames = 0;
    Q_EMIT framesChanged();
    update();
}

void VideoFrames::setFitted(bool fitted)
{
    if (m_fitted == fitted)
        return;
    m_fitted = fitted;
    update();
    Q_EMIT fittedChanged();
}

void VideoFrames::setStretched(bool stretched)
{
    if (m_stretched == stretched)
        return;
    m_stretched = stretched;
    update();
    Q_EMIT stretchedChanged();
}

QSGNode* VideoFrames::updatePaintNode(QSGNode* old, UpdatePaintNodeData*)
{
    if (m_image.isNull() || window() == nullptr) {
        delete old;
        return nullptr;
    }
    auto* node = static_cast<QSGImageNode*>(old);
    if (node == nullptr) {
        node = window()->createImageNode();
        node->setOwnsTexture(true);
        m_fresh = true;
    }
    if (m_fresh) {
        node->setTexture(window()->createTextureFromImage(m_image));
        m_fresh = false;
    }
    const QSizeF size = m_image.size();
    if (m_stretched) {
        // Stretched to the item, as a media viewer's video may be (Image.Stretch).
        node->setSourceRect(QRectF(QPointF(0, 0), size));
        node->setRect(QRectF(0, 0, width(), height()));
    } else if (m_fitted) {
        // Fitted inside, as a media viewer's video is (Image.PreserveAspectFit).
        const qreal scale = std::min(width() / size.width(), height() / size.height());
        const QSizeF drawn(size.width() * scale, size.height() * scale);
        node->setSourceRect(QRectF(QPointF(0, 0), size));
        node->setRect(QRectF(QPointF((width() - drawn.width()) / 2, (height() - drawn.height()) / 2), drawn));
    } else {
        // Cropped to fill, as a hero's picture is (Image.PreserveAspectCrop).
        const qreal scale = std::max(width() / size.width(), height() / size.height());
        const QSizeF shown(width() / scale, height() / scale);
        node->setSourceRect(QRectF(QPointF((size.width() - shown.width()) / 2, (size.height() - shown.height()) / 2), shown));
        node->setRect(QRectF(0, 0, width(), height()));
    }
    node->setFiltering(QSGTexture::Linear);
    return node;
}
