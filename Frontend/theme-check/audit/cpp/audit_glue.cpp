// A kit singleton of the engine `within` was made in: each theme's engine has its own Tokens (man theme-check).
#include "audit_glue.h"

#include <QtQml/QQmlEngine>

QObject* auditKitSingleton(QObject& within, const QString& name)
{
    QQmlEngine* engine = qmlEngine(&within);
    return engine != nullptr ? engine->singletonInstance<QObject*>(QStringLiteral("EmuSen.Kit"), name) : nullptr;
}
