// A moment's time in a locale's short form (EmuSen_Qt.md §7.1).
#include "short_time.h"

#include <QtCore/QDateTime>
#include <QtCore/QLocale>
#include <QtCore/QTimeZone>

QString clockShortTime(std::int64_t seconds, bool utc, const QString &locale)
{
    const QDateTime at = QDateTime::fromSecsSinceEpoch(seconds, utc ? QTimeZone::UTC : QTimeZone::LocalTime);
    const QLocale form = locale.isEmpty() ? QLocale::system() : QLocale(locale);
    // CLDR's narrow and no-break spaces drawn as the face's own space, as the boards write the time (EmuSen_Qt.md §7.1).
    return form.toString(at.time(), QLocale::ShortFormat).replace(QChar(0x202F), u' ').replace(QChar(0x00A0), u' ');
}
