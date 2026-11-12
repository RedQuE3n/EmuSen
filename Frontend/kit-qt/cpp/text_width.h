// The width a line of words takes as Qt Quick's Text lays it out, which cxx-qt-lib does not bind (QTextLayout); the reason is EmuSen_Kit.md §7.43 and §7.106.
#pragma once

#include <QtGui/QFont>
#include <QtCore/QString>

double text_width(const QFont& font, const QString& text);
