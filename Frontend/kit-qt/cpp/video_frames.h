// EmuSen.Kit.VideoFrames: a video's frames drawn as pictures, for the software backend, where VideoOutput draws nothing (EmuSen_Kit.md §7.48).
#pragma once

#include <QtGui/QImage>
#include <QtQml/qqmlregistration.h>
#include <QtQuick/QQuickItem>

#include <QtMultimedia/QVideoSink>
class QVideoFrame;

class VideoFrames : public QQuickItem {
    Q_OBJECT
    QML_ELEMENT
    Q_PROPERTY(QVideoSink* videoSink READ videoSink CONSTANT)
    Q_PROPERTY(int frames READ frames NOTIFY framesChanged)

public:
    explicit VideoFrames(QQuickItem* parent = nullptr);

    QVideoSink* videoSink() const { return m_sink; }
    int frames() const { return m_frames; }
    Q_INVOKABLE void clear();

Q_SIGNALS:
    void framesChanged();

protected:
    QSGNode* updatePaintNode(QSGNode* old, UpdatePaintNodeData* data) override;

private:
    void take(const QVideoFrame& frame);

    QVideoSink* m_sink = nullptr;
    QImage m_image;
    bool m_fresh = false;
    int m_frames = 0;
};
