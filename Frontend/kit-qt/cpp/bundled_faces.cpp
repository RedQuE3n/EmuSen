#include "bundled_faces.h"

#include <QtCore/QCoreApplication>
#include <QtCore/QDirIterator>
#include <QtGui/QFont>
#include <QtGui/QFontDatabase>
#include <QtGui/QGuiApplication>

QStringList bundled_families()
{
    // Every face in the kit's resources, added once for the process: Qt's font database is the application's (EmuSen_Kit.md §7.117).
    static const QStringList families = [] {
        QStringList found;
        QDirIterator files(QStringLiteral(":/emusen/fonts"), { QStringLiteral("*.ttf") }, QDir::Files);
        while (files.hasNext()) {
            // A static weight's file also carries its legacy name ("Sora Light"); the first is its typographic family, FontLoader's.
            const QStringList families = QFontDatabase::applicationFontFamilies(QFontDatabase::addApplicationFont(files.next()));
            if (!families.isEmpty() && !found.contains(families.first()))
                found.append(families.first());
        }
        return found;
    }();
    return families;
}

QString platform_family()
{
    // The family a text naming none is given: the application's font (QGuiApplication::font).
    if (!qobject_cast<QGuiApplication*>(QCoreApplication::instance()))
        return QString();
    return QGuiApplication::font().family();
}
