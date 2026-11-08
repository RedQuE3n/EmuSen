// QTest's mouse, keyboard and window helpers are declared only for a GUI build, which CMake marks and cc-rs does not.
#ifndef QT_GUI_LIB
#define QT_GUI_LIB 1
#endif
#include "harness.h"

#include <QtCore/QCoreApplication>
#include <QtCore/QJsonArray>
#include <QtCore/QJsonDocument>
#include <QtCore/QJsonObject>
#include <QtCore/QMetaProperty>
#include <QtGui/QAccessible>
#include <QtGui/QColor>
#include <QtGui/QFontMetricsF>
#include <QtGui/QImage>
#include <QtQml/QQmlContext>
#include <QtQml/QQmlEngine>
#include <QtQml/QQmlProperty>
#include <QtQuick/QQuickItem>
#include <QtQuick/QQuickWindow>
#include <QtQuickTest/quicktest.h>
#include <QtTest/QTest>
#include <algorithm>
#include <cmath>
#include <cstdlib>
#include <utility>

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

// Every warning Qt logs, kept for the test to read and shown as before; Qt's handler has no context, so the list is the file's (EmuSen_Kit.md §7.14).
struct Kept {
    QStringList messages;
    QtMessageHandler shown = nullptr;
};

Kept& kept()
{
    static Kept log;
    return log;
}

void keep(QtMsgType type, const QMessageLogContext& context, const QString& message)
{
    if (type != QtDebugMsg && type != QtInfoMsg) {
        kept().messages.append(message);
    }
    if (kept().shown != nullptr) {
        kept().shown(type, context, message);
    }
}

// What a press may move and the walk puts back after each: a selection, a value or a scroll, as Qt Quick's own classes declare them (EmuSen_Kit.md §7.11).
QJsonObject keptOf(QQuickItem* item)
{
    QJsonObject kept;
    for (const char* name : {"currentIndex", "value", "contentX", "contentY", "offset"}) {
        const QMetaProperty p = item->metaObject()->property(item->metaObject()->indexOfProperty(name));
        if (p.isValid() && p.isWritable() && QByteArray(p.enclosingMetaObject()->className()).startsWith("QQuick")) {
            kept[QLatin1String(name)] = p.read(item).toDouble();
        }
    }
    return kept;
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
    node["enabled"] = item->isEnabled();
    node["scale"] = unit.height();
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
        node["face"] = font.family();
        const QRectF glyphs = glyphBox(item);
        node["glyphs"] = QJsonArray{glyphs.x(), glyphs.y(), glyphs.width(), glyphs.height()};
    }
    if (item->inherits("QQuickFlickable")) {
        const int direction = item->property("flickableDirection").toInt();
        node["sv"] = direction != 1 && item->property("contentHeight").toDouble() > item->height() + 0.5;
        node["sh"] = direction != 2 && item->property("contentWidth").toDouble() > item->width() + 0.5;
        node["wheel"] = item->property("interactive").toBool();
        node["ch"] = item->property("contentHeight").toDouble() * unit.height();
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
    const QAccessibleInterface* reader = QAccessible::queryAccessibleInterface(item); // the role a screen reader is given
    node["role"] = reader != nullptr ? static_cast<int>(reader->role()) : 0;
    node["colors"] = coloursOf(item);
    node["kept"] = keptOf(item);
    node["scoped"] = item->scopedFocusItem() != nullptr ? QString::number(idOf(item->scopedFocusItem())) : QString();
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

QQuickItem* itemOf(QObject& window, std::uint64_t id)
{
    QQuickWindow* w = quick(window);
    return w != nullptr ? find(w->contentItem(), id) : nullptr;
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

// The window's grab against a reference picture: a pixel differs when the other picture has no pixel within `reach` pixels
// and `tolerance` levels on every channel, either way round, so an edge antialiased under a pixel apart passes and a thing
// moved or recoloured does not; inside a `loose` box ("x,y,w,h,levels;...") its own levels hold. The differing pixels are
// marked in `marks` when it is named; -1 when the picture cannot be read or the sizes differ.
int harness_difference(QObject& window, const QString& reference, int tolerance, int reach, const QString& loose, const QString& marks)
{
    QList<QPair<QRect, int>> boxes;
    for (const QString& box : loose.split(QLatin1Char(';'), Qt::SkipEmptyParts)) {
        const QStringList n = box.split(QLatin1Char(','));
        boxes.append({QRect(n.value(0).toInt(), n.value(1).toInt(), n.value(2).toInt(), n.value(3).toInt()), n.value(4).toInt()});
    }
    QQuickWindow* w = quick(window);
    const QImage ref = QImage(reference).convertToFormat(QImage::Format_RGB32);
    if (w == nullptr || ref.isNull()) {
        return -1;
    }
    const QImage shot = w->grabWindow().convertToFormat(QImage::Format_RGB32);
    if (shot.size() != ref.size()) {
        return -1;
    }
    const auto near = [&](const QImage& a, const QImage& b, int x, int y, int levels) {
        const QRgb p = a.pixel(x, y);
        for (int by = std::max(0, y - reach); by <= std::min(b.height() - 1, y + reach); ++by) {
            for (int bx = std::max(0, x - reach); bx <= std::min(b.width() - 1, x + reach); ++bx) {
                const QRgb q = b.pixel(bx, by);
                if (std::abs(qRed(p) - qRed(q)) <= levels && std::abs(qGreen(p) - qGreen(q)) <= levels && std::abs(qBlue(p) - qBlue(q)) <= levels) {
                    return true;
                }
            }
        }
        return false;
    };
    QImage marked(shot.size(), QImage::Format_RGB32);
    marked.fill(Qt::white);
    int differing = 0;
    for (int y = 0; y < shot.height(); ++y) {
        for (int x = 0; x < shot.width(); ++x) {
            int levels = tolerance;
            for (const auto& [box, own] : boxes) {
                levels = box.contains(x, y) ? own : levels;
            }
            if (!near(shot, ref, x, y, levels) || !near(ref, shot, x, y, levels)) {
                ++differing;
                marked.setPixel(x, y, qRgb(220, 30, 60));
            }
        }
    }
    if (!marks.isEmpty()) {
        marked.save(marks);
    }
    return differing;
}

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
        QTest::qWaitFor([w, width, height] { return w->contentItem()->width() == width && w->contentItem()->height() == height; }, 2000); // the platform's geometry event delivered, as a window manager's arrives
        QQuickTest::qWaitForPolish(w); // what Qt declares: no item still to lay out
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

void harness_keep_messages() { kept().shown = qInstallMessageHandler(keep); }

QString harness_messages() { return std::exchange(kept().messages, {}).join(QChar(0x1e)); }

std::uint64_t harness_focus(QObject& window)
{
    QQuickWindow* w = quick(window);
    return w != nullptr && w->activeFocusItem() != nullptr ? idOf(w->activeFocusItem()) : 0;
}

bool harness_focus_item(QObject& window, std::uint64_t id)
{
    QQuickItem* item = itemOf(window, id);
    if (item == nullptr) {
        return false;
    }
    item->forceActiveFocus();
    return true;
}

// One item's number property by its id, read (NaN when there is none) or written back.
double harness_get(QObject& window, std::uint64_t id, const QString& property)
{
    QQuickItem* item = itemOf(window, id);
    return item != nullptr ? item->property(property.toUtf8().constData()).toDouble() : std::nan("");
}

bool harness_put(QObject& window, std::uint64_t id, const QString& property, double value)
{
    QQuickItem* item = itemOf(window, id);
    return item != nullptr && item->setProperty(property.toUtf8().constData(), value);
}

// A focus scope's remembered item, read by its id, or given back with setFocus, which moves no focus outside its scope.
std::uint64_t harness_scoped(QObject& window, std::uint64_t scope)
{
    QQuickItem* item = itemOf(window, scope);
    return item != nullptr && item->scopedFocusItem() != nullptr ? idOf(item->scopedFocusItem()) : 0;
}

void harness_give_focus(QObject& window, std::uint64_t id)
{
    if (QQuickItem* item = itemOf(window, id)) {
        item->setFocus(true);
    }
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
