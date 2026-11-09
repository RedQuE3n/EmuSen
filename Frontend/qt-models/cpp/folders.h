// A folder shown in the system's file manager through QDesktopServices, which cxx-qt-lib does not wrap (EmuSen_Qt.md §7.7.E).
#ifndef EMUSEN_QT_MODELS_FOLDERS_H
#define EMUSEN_QT_MODELS_FOLDERS_H

#include <QtCore/QObject>
#include <QtCore/QString>
#include <QtGui/QDesktopServices>

QString folderUrl(const QString &path);

bool folderOpen(const QString &path);

// Qt's own way to take a scheme's URLs in place of the desktop's handler, which a test uses so no file manager starts.
template <typename Receiver>
bool folderUrlsTo(const Receiver &receiver, const QString &method)
{
    QDesktopServices::setUrlHandler(QStringLiteral("file"), const_cast<Receiver *>(&receiver), method.toUtf8().constData());
    return true;
}
#endif
