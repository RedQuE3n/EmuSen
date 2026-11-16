// A moment's time in a locale's short form through QLocale, which cxx-qt-lib does not wrap (EmuSen_Qt.md §7.1).
#ifndef EMUSEN_QT_MODELS_SHORT_TIME_H
#define EMUSEN_QT_MODELS_SHORT_TIME_H

#include <QtCore/QString>

#include <cstdint>

QString clockShortTime(std::int64_t seconds, bool utc, const QString &locale);
#endif
