// The session's pictures served to QML's Image by address (EmuSen_Qt.md §7.3).
#include "session_pictures.h"

#include <QtQml/QQmlEngine>

#include "emusen-qt-models/src/pictures.cxxqt.h"

SessionPictures::SessionPictures(rust::Box<PictureSource> source)
    : QQuickImageProvider(QQuickImageProvider::Image)
    , m_source(std::move(source))
{
}

SessionPictures::~SessionPictures() = default;

QImage SessionPictures::requestImage(const QString &id, QSize *size, const QSize &requestedSize)
{
    QImage image = pictureOf(*m_source, id);
    if (size)
        *size = image.size();
    if (requestedSize.isValid() && !image.isNull())
        image = image.scaled(requestedSize, Qt::KeepAspectRatio, Qt::SmoothTransformation);
    return image;
}

bool savePicture(const QImage &image, const QString &path)
{
    return !image.isNull() && image.save(path, "PNG");
}

bool sessionPicturesServe(const QObject &object, const QString &name, rust::Box<PictureSource> source)
{
    QQmlEngine *engine = qmlEngine(&object);
    if (!engine)
        return false;
    // The engine owns its providers; one model serves a name, so a second one in the same engine takes it over.
    engine->removeImageProvider(name);
    engine->addImageProvider(name, new SessionPictures(std::move(source)));
    return true;
}
