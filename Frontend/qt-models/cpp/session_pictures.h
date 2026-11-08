// The session's pictures served to QML's Image by address (EmuSen_Qt.md §7.3): a QQuickImageProvider, which cxx-qt cannot subclass.
#ifndef EMUSEN_QT_MODELS_SESSION_PICTURES_H
#define EMUSEN_QT_MODELS_SESSION_PICTURES_H

#include <QtCore/QObject>
#include <QtCore/QString>
#include <QtGui/QImage>
#include <QtQuick/QQuickImageProvider>

#include "rust/cxx.h"

struct PictureSource;

class SessionPictures : public QQuickImageProvider
{
public:
    explicit SessionPictures(rust::Box<PictureSource> source);
    ~SessionPictures() override;
    QImage requestImage(const QString &id, QSize *size, const QSize &requestedSize) override;

private:
    rust::Box<PictureSource> m_source;
};

bool sessionPicturesServe(const QObject &object, const QString &name, rust::Box<PictureSource> source);
#endif
