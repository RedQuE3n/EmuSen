// The module's glue where cxx-qt-lib does not reach: making a list a singleton owns, and finding a singleton in an engine (EmuSen_Qt.md §3.2).
#pragma once

#include <QtCore/QObject>
#include <QtCore/QString>

class ThemeRows;
class ThemeHelpEntries;

ThemeRows* themeNewRows(QObject& parent);
ThemeHelpEntries* themeNewHelpEntries(QObject& parent);
QObject* themeSingleton(QObject& within, const QString& name);
