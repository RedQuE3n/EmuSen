#include "text_width.h"

#include <QtGui/QFontMetricsF>
#include <QtGui/QTextLayout>

double text_width(const QFont& font, const QString& text)
{
    // Laid out as Qt Quick's Text lays out a line, with design metrics (EmuSen_Kit.md §7.106).
    QTextLayout layout(text, font);
    QTextOption option;
    option.setUseDesignMetrics(true);
    option.setWrapMode(QTextOption::NoWrap);
    layout.setTextOption(option);
    layout.beginLayout();
    QTextLine line = layout.createLine();
    if (line.isValid())
        line.setLineWidth(1.0e6);
    layout.endLayout();
    return line.isValid() ? line.naturalTextWidth() : 0.0;
}

double cap_height(const QFont& font, double size)
{
    // At the whole pixel size Qt draws: a real handed to a Text's font.pixelSize is taken to the nearest (seen 2026-11-15, EmuSen_Kit.md §7.120).
    QFont sized(font);
    sized.setPixelSize(qRound(size));
    return QFontMetricsF(sized).capHeight();
}
