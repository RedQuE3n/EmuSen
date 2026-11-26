// Parses UTF-8 JSON payloads into native QVariantMap / QVariantList values for Qt Quick.
#include "json_variant.h"

#include <QtCore/QJsonDocument>

QVariant jsonToVariant(const QString &json)
{
    if (json.isEmpty()) {
        return QVariant();
    }
    const QJsonDocument doc = QJsonDocument::fromJson(json.toUtf8());
    return doc.isNull() ? QVariant() : doc.toVariant();
}
