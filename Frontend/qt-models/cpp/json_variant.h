// Converts JSON strings to Qt QVariant values via QJsonDocument so QML views never run JSON.parse.
#ifndef EMUSEN_QT_MODELS_JSON_VARIANT_H
#define EMUSEN_QT_MODELS_JSON_VARIANT_H

#include <QtCore/QString>
#include <QtCore/QVariant>

QVariant jsonToVariant(const QString &json);

#endif
