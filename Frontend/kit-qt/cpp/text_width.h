// The width a font gives a line of words, which cxx-qt-lib does not bind (QFontMetricsF); the reason is EmuSen_Kit.md §7.43.
#pragma once

#include <QtGui/QFont>
#include <QtCore/QString>

double text_width(const QFont& font, const QString& text);
