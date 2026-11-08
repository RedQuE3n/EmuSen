// QTest's mouse, keyboard and window helpers are declared only for a GUI build, which CMake marks and cc-rs does not.
#ifndef QT_GUI_LIB
#define QT_GUI_LIB 1
#endif
#include "harness.h"

#include <QtCore/QCoreApplication>
#include <QtCore/QJsonArray>
#include <QtCore/QJsonDocument>
#include <QtCore/QJsonObject>
#include <QtGui/QColor>
#include <QtGui/QFontMetricsF>
#include <QtGui/QImage>
#include <QtQml/QQmlContext>
#include <QtQml/QQmlEngine>
#include <QtQml/QQmlProperty>
#include <QtQuick/QQuickItem>
#include <QtQuick/QQuickWindow>
#include <QtTest/QTest>
#include <algorithm>

namespace {

QQuickWindow* quick(QObject& window) { return qobject_cast<QQuickWindow*>(&window); }

std::uint64_t idOf(const QObject* object) { return reinterpret_cast<std::uintptr_t>(object); }

bool hasPointerHandler(const QQuickItem* item)
{
    for (const QObject* child : item->children()) {
        if (child->inherits("QQuickPointerHandler")) {
            return true;
        }
    }
    return false;
}

// The QML type: Qt names a component's generated meta-object after it (MenuRow_QMLTYPE_3); a plain Qt item keeps its C++ class.
QString typeOf(QQuickItem* item)
{
    const QString cls = QString::fromLatin1(item->metaObject()->className());
    return cls.section(QLatin1String("_QMLTYPE_"), 0, 0).section(QLatin1String("_QML_"), 0, 0);
}

void addColour(QJsonArray& colours, const QVariant& value)
{
    if (value.canConvert<QColor>()) {
        const QColor colour = value.value<QColor>();
        if (colour.isValid() && colour.alpha() > 0) {
            colours.append(colour.name(QColor::HexArgb));
        }
    }
}

QJsonArray coloursOf(QQuickItem* item)
{
    QJsonArray colours;
    addColour(colours, item->property("color"));
    if (QObject* border = item->property("border").value<QObject*>(); border != nullptr && border->property("width").toDouble() > 0) {
        addColour(colours, border->property("color"));
    }
    for (QObject* child : item->children()) {
        if (child->inherits("QQuickShapePath")) {
            addColour(colours, child->property("fillColor"));
            if (child->property("strokeWidth").toDouble() > 0) {
                addColour(colours, child->property("strokeColor"));
            }
        }
    }
    return colours;
}

// Where the words are laid out: inside the paddings, the text's own size placed by its alignment, in the scene.
QRectF glyphBox(QQuickItem* item)
{
    const auto number = [item](const char* name) { return item->property(name).toDouble(); };
    const double left = number("leftPadding"), top = number("topPadding");
    const double w = item->width() - left - number("rightPadding"), h = item->height() - top - number("bottomPadding");
    const double tw = std::min(number("contentWidth"), w), th = std::min(number("contentHeight"), h);
    const int across = item->property("effectiveHorizontalAlignment").toInt(), down = item->property("verticalAlignment").toInt();
    const double x = left + ((across & Qt::AlignRight) ? w - tw : (across & Qt::AlignHCenter) ? (w - tw) / 2 : 0);
    const double y = top + ((down & Qt::AlignBottom) ? h - th : (down & Qt::AlignVCenter) ? (h - th) / 2 : 0);
    return item->mapRectToScene(QRectF(x, y, tw, th));
}

void dump(QQuickItem* item, std::uint64_t parent, double parentOpacity, QJsonArray& out)
{
    const QRectF rect = item->mapRectToScene(QRectF(0, 0, item->width(), item->height()));
    const QRectF unit = item->mapRectToScene(QRectF(0, 0, 1, 1)); // words are measured in the item's units; the rules read the scene's
    const double opacity = parentOpacity * item->opacity();
    const bool control = item->inherits("QQuickControl");
    const bool handlers = hasPointerHandler(item);
    QJsonObject node;
    node["id"] = QString::number(idOf(item));
    node["parent"] = QString::number(parent);
    node["ty"] = typeOf(item);
    node["cls"] = QString::fromLatin1(item->metaObject()->className());
    node["name"] = item->objectName();
    node["x"] = rect.x();
    node["y"] = rect.y();
    node["w"] = rect.width();
    node["h"] = rect.height();
    node["visible"] = item->isVisible() && opacity > 0;
    node["opacity"] = opacity;
    node["clip"] = item->clip();
    node["hit"] = item->isVisible() && item->isEnabled() && (item->acceptedMouseButtons() != Qt::NoButton || handlers);
    const QQuickItem* inner = item->isFocusScope() ? item->scopedFocusItem() : nullptr;
    const bool container = inner != nullptr && (inner->activeFocusOnTab() || inner->inherits("QQuickControl"));
    const bool typed = item->inherits("QQuickTextInput") || item->inherits("QQuickTextEdit");
    node["operable"] = item->isEnabled() && !container && ((control && item->property("focusPolicy").toInt() != 0) || ((handlers || typed) && item->activeFocusOnTab()));
    node["control"] = control;
    node["focus"] = item->hasActiveFocus();
    node["popup"] = item->inherits("QQuickPopupItem");
    node["menubar"] = item->inherits("QQuickMenuBar");
    node["picture"] = item->inherits("QQuickImage") || item->inherits("QQuickShape");
    node["backdrop"] = item->property("backdrop").toBool();
    if (item->inherits("QQuickText") || item->inherits("QQuickTextInput")) {
        node["text"] = item->property("text").toString();
        node["elided"] = item->property("truncated").toBool();
        node["tw"] = item->property("contentWidth").toDouble() * unit.width();
        node["th"] = item->property("contentHeight").toDouble() * unit.height();
        const QFont font = item->property("font").value<QFont>();
        node["cap"] = QFontMetricsF(font).capHeight() * unit.height();
        QColor ink = item->property("color").value<QColor>();
        ink.setAlphaF(ink.alphaF() * opacity);
        node["ink"] = item->isEnabled() ? ink.name(QColor::HexArgb) : QString(); // words of a disabled control are exempt (WCAG 2.1, 1.4.3)
        node["size"] = font.pixelSize() > 0 ? font.pixelSize() : font.pointSizeF() * 4.0 / 3.0;
        node["bold"] = font.weight() >= QFont::Bold;
        const QRectF glyphs = glyphBox(item);
        node["glyphs"] = QJsonArray{glyphs.x(), glyphs.y(), glyphs.width(), glyphs.height()};
    }
    if (item->inherits("QQuickFlickable")) {
        const int direction = item->property("flickableDirection").toInt();
        node["sv"] = direction != 1 && item->property("contentHeight").toDouble() > item->height() + 0.5;
        node["sh"] = direction != 2 && item->property("contentWidth").toDouble() > item->width() + 0.5;
        node["wheel"] = item->property("interactive").toBool();
    }
    if (item->inherits("QQuickPathView")) {
        node["sh"] = item->property("count").toInt() > item->property("pathItemCount").toInt();
        node["wheel"] = item->property("interactive").toBool();
    }
    for (const char* edge : {"fadeTop", "fadeBottom", "fadeLeft", "fadeRight"}) {
        node[QLatin1String(edge)] = item->property(edge).toBool();
    }
    QString heard = QQmlProperty(item, QStringLiteral("Accessible.name"), qmlContext(item)).read().toString();
    if (heard.isEmpty()) {
        heard = item->property("text").toString();
    }
    if (heard.isEmpty()) {
        heard = item->property("placeholderText").toString();
    }
    node["heard"] = heard;
    node["role"] = QQmlProperty(item, QStringLiteral("Accessible.role"), qmlContext(item)).read().toInt();
    node["colors"] = coloursOf(item);
    out.append(node);
    for (QQuickItem* child : item->childItems()) {
        dump(child, idOf(item), opacity, out);
    }
}

QQuickItem* find(QQuickItem* item, std::uint64_t id)
{
    if (idOf(item) == id) {
        return item;
    }
    for (QQuickItem* child : item->childItems()) {
        if (QQuickItem* found = find(child, id)) {
            return found;
        }
    }
    return nullptr;
}

// Every text at no opacity, or back at the opacity it had, so a grab shows what is behind the words.
void hideWords(QQuickItem* item, bool hidden)
{
    if (item->inherits("QQuickText") || item->inherits("QQuickTextInput")) {
        if (hidden) {
            item->setProperty("harnessOpacity", item->opacity());
            item->setOpacity(0.0);
        } else if (item->property("harnessOpacity").isValid()) {
            item->setOpacity(item->property("harnessOpacity").toDouble());
            item->setProperty("harnessOpacity", QVariant());
        }
    }
    for (QQuickItem* child : item->childItems()) {
        hideWords(child, hidden);
    }
}

} // namespace

void harness_hide_words(QObject& window, bool hidden)
{
    if (QQuickWindow* w = quick(window)) {
        hideWords(w->contentItem(), hidden);
    }
}

bool harness_show(QObject& window, int width, int height)
{
    QQuickWindow* w = quick(window);
    if (w == nullptr) {
        return false;
    }
    w->resize(width, height);
    w->show();
    w->requestActivate();
    return QTest::qWaitForWindowExposed(w) && QTest::qWaitForWindowActive(w);
}

void harness_resize(QObject& window, int width, int height)
{
    if (QQuickWindow* w = quick(window)) {
        w->resize(width, height);
        QTest::qWait(20);
    }
}

QString harness_tree(QObject& window)
{
    QJsonArray nodes;
    if (QQuickWindow* w = quick(window)) {
        dump(w->contentItem(), 0, 1.0, nodes);
    }
    return QString::fromUtf8(QJsonDocument(nodes).toJson(QJsonDocument::Compact));
}

void harness_click(QObject& window, double x, double y, int button)
{
    if (QQuickWindow* w = quick(window)) {
        QTest::mouseClick(w, static_cast<Qt::MouseButton>(button), Qt::NoModifier, QPoint(static_cast<int>(x), static_cast<int>(y)));
    }
}

void harness_key(QObject& window, int key, int modifiers)
{
    if (QQuickWindow* w = quick(window)) {
        QTest::keyClick(w, static_cast<Qt::Key>(key), Qt::KeyboardModifiers::fromInt(modifiers));
    }
}

void harness_wheel(QObject& window, double x, double y, int delta)
{
    if (QQuickWindow* w = quick(window)) {
        const QPointF point(x, y);
        QWheelEvent event(point, w->mapToGlobal(point.toPoint()), QPoint(), QPoint(0, delta), Qt::NoButton, Qt::NoModifier, Qt::NoScrollPhase, false);
        QCoreApplication::sendEvent(w, &event);
    }
}

void harness_wait(int ms) { QTest::qWait(ms); }

std::uint64_t harness_focus(QObject& window)
{
    QQuickWindow* w = quick(window);
    return w != nullptr && w->activeFocusItem() != nullptr ? idOf(w->activeFocusItem()) : 0;
}

bool harness_focus_item(QObject& window, std::uint64_t id)
{
    QQuickWindow* w = quick(window);
    QQuickItem* item = w != nullptr ? find(w->contentItem(), id) : nullptr;
    if (item == nullptr) {
        return false;
    }
    item->forceActiveFocus();
    return true;
}

bool harness_grab(QObject& window, const QString& path)
{
    QQuickWindow* w = quick(window);
    return w != nullptr && w->grabWindow().save(path);
}

QString harness_property(QObject& root, const QString& object_name, const QString& property)
{
    QObject* object = root.objectName() == object_name ? &root : root.findChild<QObject*>(object_name);
    return object != nullptr ? object->property(property.toUtf8().constData()).toString() : QString();
}

QObject* harness_singleton(QQmlApplicationEngine& engine, const QString& module, const QString& name)
{
    return engine.singletonInstance<QObject*>(module, name);
}

int harness_roots(QQmlApplicationEngine& engine) { return static_cast<int>(engine.rootObjects().size()); }

bool harness_set_property(QObject& root, const QString& object_name, const QString& property, const QString& value)
{
    QObject* object = root.objectName() == object_name ? &root : root.findChild<QObject*>(object_name);
    return object != nullptr && QQmlProperty(object, property, qmlContext(object)).write(QVariant(value));
}

// One grab, many samples: "x,y;x,y" in, "#AARRGGBB;#AARRGGBB" out.
QString harness_colours_at(QObject& window, const QString& points)
{
    QQuickWindow* w = quick(window);
    if (w == nullptr) {
        return QString();
    }
    const QImage image = w->grabWindow();
    QStringList colours;
    for (const QString& point : points.split(QLatin1Char(';'), Qt::SkipEmptyParts)) {
        const QStringList xy = point.split(QLatin1Char(','));
        const QPoint at(xy.value(0).toInt(), xy.value(1).toInt());
        colours.append(image.valid(at) ? image.pixelColor(at).name(QColor::HexArgb) : QString());
    }
    return colours.join(QLatin1Char(';'));
}
