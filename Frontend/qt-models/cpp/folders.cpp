// A folder shown in the system's file manager (EmuSen_Qt.md §7.7.E).
#include "folders.h"

#include <QtCore/QUrl>
#include <QtGui/QDesktopServices>

QString folderUrl(const QString &path)
{
    return QUrl::fromLocalFile(path).toString(QUrl::FullyEncoded);
}

bool folderOpen(const QString &path)
{
    return QDesktopServices::openUrl(QUrl::fromLocalFile(path));
}

QString folderPath(const QUrl &url)
{
    return url.isLocalFile() ? url.toLocalFile() : QString();
}
