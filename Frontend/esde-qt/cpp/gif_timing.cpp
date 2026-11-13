// QImageReader's animation calls, the reason in EmuSen_Qt.md §4.5.9.
#include "gif_timing.h"

#include <QtGui/QImage>
#include <QtGui/QImageReader>

std::int32_t gif_frame_count(const QString& path)
{
    QImageReader reader(path);
    return reader.supportsAnimation() ? reader.imageCount() : 0;
}

std::int32_t gif_first_delay(const QString& path)
{
    QImageReader reader(path);
    QImage first;
    return reader.read(&first) ? reader.nextImageDelay() : -1;
}
