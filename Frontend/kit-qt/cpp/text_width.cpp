#include "text_width.h"

#include <QtGui/QFontDatabase>
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
    // At the whole pixel size Qt draws: a real handed to a Text's font.pixelSize is taken to the nearest (seen 2026-11-15, EmuSen_Kit.md §7.121).
    QFont sized(font);
    sized.setPixelSize(qRound(size));
    return QFontMetricsF(sized).capHeight();
}

double body_cap_height(double size)
{
    // The kit's body face from its own resources, loaded once, as the program's rows draw in it (EmuSen_Kit.md §7.121).
    static const QFont body = [] {
        const QStringList families = QFontDatabase::applicationFontFamilies(QFontDatabase::addApplicationFont(QStringLiteral(":/emusen/fonts/Barlow-Regular.ttf")));
        QFont face;
        if (!families.isEmpty())
            face.setFamilies(families);
        return face;
    }();
    return cap_height(body, size);
}
