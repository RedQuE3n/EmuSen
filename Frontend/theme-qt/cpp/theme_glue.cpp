// The module's glue (EmuSen_Qt.md §3.2): cxx-qt makes no QObject from Rust, and QQmlEngine::singletonInstance is not in cxx-qt-lib.
#include "theme_glue.h"

#include <QtQml/QJSEngine>
#include <QtQml/QQmlEngine>

#include "emusen-theme-qt/src/help.cxxqt.h"
#include "emusen-theme-qt/src/rows.cxxqt.h"

namespace {

// The singleton's list stays its own: no engine's collector may take an object a call handed to QML.
template <typename T> T* owned(T* object)
{
    QJSEngine::setObjectOwnership(object, QJSEngine::CppOwnership);
    return object;
}

}

ThemeRows* themeNewRows(QObject& parent) { return owned(new ThemeRows(&parent)); }

ThemeHelpEntries* themeNewHelpEntries(QObject& parent) { return owned(new ThemeHelpEntries(&parent)); }

QObject* themeSingleton(QObject& within, const QString& name)
{
    QQmlEngine* engine = qobject_cast<QQmlEngine*>(&within);
    if (engine == nullptr)
        engine = qmlEngine(&within);
    return engine != nullptr ? engine->singletonInstance<QObject*>(QStringLiteral("EmuSen.Theme"), name) : nullptr;
}
