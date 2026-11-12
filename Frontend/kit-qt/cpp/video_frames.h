// EmuSen.Kit.VideoFrames: a video's frames drawn as pictures, for the software backend, where VideoOutput draws nothing (EmuSen_Kit.md §7.50).
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
    // Fitted inside the item rather than cropped to fill it (EmuSen_Kit.md §7.61).
    Q_PROPERTY(bool fitted READ fitted WRITE setFitted NOTIFY fittedChanged)
    // Stretched to the item, its shape not kept (Image.Stretch), before fitted (EmuSen_Kit.md §7.70).
    Q_PROPERTY(bool stretched READ stretched WRITE setStretched NOTIFY stretchedChanged)

public:
    explicit VideoFrames(QQuickItem* parent = nullptr);

    QVideoSink* videoSink() const { return m_sink; }
    int frames() const { return m_frames; }
    bool fitted() const { return m_fitted; }
    void setFitted(bool fitted);
    bool stretched() const { return m_stretched; }
    void setStretched(bool stretched);
    Q_INVOKABLE void clear();

Q_SIGNALS:
    void framesChanged();
    void fittedChanged();
    void stretchedChanged();

protected:
    QSGNode* updatePaintNode(QSGNode* old, UpdatePaintNodeData* data) override;

private:
    void take(const QVideoFrame& frame);

    QVideoSink* m_sink = nullptr;
    QImage m_image;
    bool m_fresh = false;
    int m_frames = 0;
    bool m_fitted = false;
    bool m_stretched = false;
};
