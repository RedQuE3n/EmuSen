// The width a line of words takes as Qt Quick's Text lays it out, and a face's capitals at a size, which cxx-qt-lib does not bind (QTextLayout, QFontMetricsF); the reasons are EmuSen_Kit.md §7.43, §7.106 and §7.121.
#pragma once

#include <QtGui/QFont>
#include <QtCore/QString>

double text_width(const QFont& font, const QString& text);
double cap_height(const QFont& font, double size);
