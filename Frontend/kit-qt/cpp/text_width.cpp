#include "text_width.h"

#include <QtGui/QFontMetricsF>

double text_width(const QFont& font, const QString& text)
{
    return QFontMetricsF(font).horizontalAdvance(text);
}
