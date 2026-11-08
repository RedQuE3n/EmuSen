// theme-audit's glue where cxx-qt-lib does not reach: QQmlEngine::singletonInstance (man theme-check).
#pragma once

#include <QtCore/QObject>
#include <QtCore/QString>

QObject* auditKitSingleton(QObject& within, const QString& name);
