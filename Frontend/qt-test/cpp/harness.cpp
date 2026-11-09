// QTest's mouse, keyboard and window helpers are declared only for a GUI build, which CMake marks and cc-rs does not.
#ifndef QT_GUI_LIB
#define QT_GUI_LIB 1
#endif
#include "harness.h"

#include <QtCore/QAbstractAnimation>
#include <QtCore/QCoreApplication>
#include <QtCore/QJsonArray>
#include <QtCore/QJsonDocument>
#include <QtCore/QJsonObject>
#include <QtCore/QLoggingCategory>
#include <QtCore/QMutex>
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
#include <QtTest/QSignalSpy>
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
    // Qt's own account of each frame's dirty items while a still span is watched (qt.quick.dirty), kept from the log.
    bool dirty = false;
    QStringList frames;
    QMutex lock;
};

Kept& kept()
{
    static Kept log;
    return log;
}

void keep(QtMsgType type, const QMessageLogContext& context, const QString& message)
{
    if (context.category != nullptr && qstrcmp(context.category, "qt.quick.dirty") == 0) {
        QMutexLocker locked(&kept().lock); // the threaded render loop syncs, and so logs, on its own thread
        if (kept().dirty) {
            kept().frames.append(message.trimmed());
        }
        return;
    }
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
        const bool number = p.typeId() == QMetaType::Double || p.typeId() == QMetaType::Float || p.typeId() == QMetaType::Int;
        if (p.isValid() && p.isWritable() && number && QByteArray(p.enclosingMetaObject()->className()).startsWith("QQuick")) { // a shadow's offset is a vector, not a view's
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
    return QRectF(x, y, tw, th);
}

void dump(QQuickItem* item, std::uint64_t parent, double parentOpacity, QJsonArray& out)
{
    const QRectF rect = item->mapRectToScene(QRectF(0, 0, item->width(), item->height()));
    const QRectF unit = item->mapRectToScene(QRectF(0, 0, 1, 1)); // words are measured in the item's units; the rules read the scene's
    const double opacity = parentOpacity * item->opacity();
    const bool control = item->inherits("QQuickControl");
    const bool handlers = hasPointerHandler(item);
    item->setProperty("harnessRead", true); // read once, so an item made since (a delegate made again) is told apart
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
    // A turned item's shape is its four corners in the scene, which the overlap rule reads in place of its scene box.
    const QPointF c0 = item->mapToScene(QPointF(0, 0)), c1 = item->mapToScene(QPointF(item->width(), 0));
    const QPointF c2 = item->mapToScene(QPointF(item->width(), item->height())), c3 = item->mapToScene(QPointF(0, item->height()));
    if (std::abs(c1.y() - c0.y()) > 1e-6 || std::abs(c3.x() - c0.x()) > 1e-6) {
        node["corners"] = QJsonArray{c0.x(), c0.y(), c1.x(), c1.y(), c2.x(), c2.y(), c3.x(), c3.y()};
    }
    // A member of a stack the kit lays out (a StackMember child, which no theme can place): its stack, itself, its layer.
    for (QQuickItem* up = item; up != nullptr; up = up->parentItem()) {
        const QObjectList& owned = up->children();
        if (std::none_of(owned.begin(), owned.end(), [](const QObject* o) { return o->inherits("StackMember"); })) {
            continue;
        }
        const QList<QQuickItem*> siblings = up->parentItem() != nullptr ? up->parentItem()->childItems() : QList<QQuickItem*>{};
        node["stack"] = QString::number(idOf(up->parentItem()));
        node["member"] = QString::number(idOf(up));
        node["layer"] = up->z() * 100000.0 + siblings.indexOf(up);
        node["memberFocus"] = up->hasActiveFocus();
        node["isMember"] = up == item;
        break;
    }
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
        // Words run along the item's own axes: measured by the scene's scale along each, and held to the item's own box when turned.
        const QPointF origin = item->mapToScene(QPointF(0, 0)), across = item->mapToScene(QPointF(1, 0)) - origin, down = item->mapToScene(QPointF(0, 1)) - origin;
        const double sx = std::hypot(across.x(), across.y()), sy = std::hypot(down.x(), down.y());
        node["tw"] = item->property("contentWidth").toDouble() * sx;
        node["th"] = item->property("contentHeight").toDouble() * sy;
        if (std::abs(across.y()) > 1e-6 || std::abs(down.x()) > 1e-6) {
            node["own"] = QJsonArray{item->width() * sx, item->height() * sy};
        }
        const QFont font = item->property("font").value<QFont>();
        node["cap"] = QFontMetricsF(font).capHeight() * sy;
        QColor ink = item->property("color").value<QColor>();
        ink.setAlphaF(ink.alphaF() * opacity);
        node["ink"] = item->isEnabled() ? ink.name(QColor::HexArgb) : QString(); // words of a disabled control are exempt (WCAG 2.1, 1.4.3)
        node["size"] = font.pixelSize() > 0 ? font.pixelSize() : font.pointSizeF() * 4.0 / 3.0;
        node["bold"] = font.weight() >= QFont::Bold;
        node["face"] = font.family();
        const QRectF laid = glyphBox(item), glyphs = item->mapRectToScene(laid);
        node["glyphs"] = QJsonArray{glyphs.x(), glyphs.y(), glyphs.width(), glyphs.height()};
        // The glyphs' own four corners in the scene, which a stack's rule holds uncovered (EmuSen_Kit.md §7.33).
        QJsonArray inked;
        for (const QPointF& p : {laid.topLeft(), laid.topRight(), laid.bottomRight(), laid.bottomLeft()}) {
            const QPointF at = item->mapToScene(p);
            inked.append(at.x());
            inked.append(at.y());
        }
        node["inked"] = inked;
    }
    if (item->inherits("QQuickFlickable")) {
        const int direction = item->property("flickableDirection").toInt();
        const auto extent = [item](const char* size, const char* before, const char* after) { return item->property(size).toDouble() + item->property(before).toDouble() + item->property(after).toDouble(); };
        node["sv"] = direction != 1 && extent("contentHeight", "topMargin", "bottomMargin") > item->height() + 0.5; // Qt scrolls over its margins too
        node["sh"] = direction != 2 && extent("contentWidth", "leftMargin", "rightMargin") > item->width() + 0.5;
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

bool madeSince(QQuickItem* item)
{
    if (!item->property("harnessRead").isValid()) {
        return true;
    }
    const QList<QQuickItem*> children = item->childItems();
    return std::any_of(children.begin(), children.end(), madeSince);
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

bool harness_made_since(QObject& window)
{
    QQuickWindow* w = quick(window);
    return w != nullptr && madeSince(w->contentItem());
}

// Qt's item views move their highlight to the current item by an animation of their own, which `moving` does not report.
bool harness_turning(QObject& window, std::uint64_t id)
{
    QQuickItem* view = itemOf(window, id);
    auto* highlight = view != nullptr && view->property("highlightFollowsCurrentItem").toBool() ? view->property("highlightItem").value<QQuickItem*>() : nullptr;
    auto* current = view != nullptr ? view->property("currentItem").value<QQuickItem*>() : nullptr;
    if (highlight == nullptr || current == nullptr || highlight->position() == current->position()) {
        if (view != nullptr) {
            view->setProperty("harnessHighlightAt", QVariant());
        }
        return false;
    }
    // Off its item, the highlight rests once three frames pass without moving it: a view may place its delegates itself.
    const std::uint64_t frame = harness_frames(window);
    const QVariant seen = view->property("harnessHighlightAt");
    if (!seen.isValid() || seen.toPointF() != highlight->position()) {
        view->setProperty("harnessHighlightAt", highlight->position());
        view->setProperty("harnessHighlightFrame", QVariant::fromValue<qulonglong>(frame));
    }
    if (frame < view->property("harnessHighlightFrame").toULongLong() + 3) {
        quick(window)->update();
        return true;
    }
    return false;
}

QString harness_tree(QObject& window)
{
    QJsonArray nodes;
    if (QQuickWindow* w = quick(window)) {
        // Read as the next frame draws it: a positioner lays out at the polish before a frame (EmuSen_Kit.md §7.41).
        if (QQuickTest::qIsPolishScheduled(w)) {
            QQuickTest::qWaitForPolish(w, 1000);
        }
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

void harness_hover(QObject& window, double x, double y)
{
    if (QQuickWindow* w = quick(window)) {
        QTest::mouseMove(w, QPoint(static_cast<int>(x), static_cast<int>(y)));
    }
}

void harness_key(QObject& window, int key, int modifiers)
{
    if (QQuickWindow* w = quick(window)) {
        QTest::keyClick(w, static_cast<Qt::Key>(key), Qt::KeyboardModifiers::fromInt(modifiers));
    }
}

void harness_key_hold(QObject& window, int key, bool down)
{
    if (QQuickWindow* w = quick(window)) {
        if (down) {
            QTest::keyPress(w, static_cast<Qt::Key>(key));
        } else {
            QTest::keyRelease(w, static_cast<Qt::Key>(key));
        }
    }
}

HeldClock::~HeldClock() { uninstall(); }

std::unique_ptr<HeldClock> harness_clock_hold()
{
    auto clock = std::make_unique<HeldClock>();
    clock->install();
    return clock;
}

void harness_clock_move(HeldClock& clock, std::int64_t ms)
{
    // A frame's step at a time, as a display would move it, so what a step starts runs from its own time.
    while (clock.at < ms) {
        clock.at = std::min(ms, clock.at + 16);
        clock.advance();
        QCoreApplication::processEvents();
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

void harness_watch_dirty(bool on)
{
    {
        QMutexLocker locked(&kept().lock);
        kept().dirty = on;
        kept().frames.clear();
    }
    QLoggingCategory::setFilterRules(on ? QStringLiteral("qt.quick.dirty.debug=true") : QString());
}

QString harness_dirty()
{
    QMutexLocker locked(&kept().lock);
    return std::exchange(kept().frames, {}).join(QChar('\n'));
}

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

QObject* harness_focused_singleton(QObject& window, const QString& module, const QString& name)
{
    QQuickWindow* w = quick(window);
    QQmlEngine* engine = w != nullptr && w->activeFocusItem() != nullptr ? qmlEngine(w->activeFocusItem()) : nullptr;
    return engine != nullptr ? engine->singletonInstance<QObject*>(module, name) : nullptr;
}

int harness_roots(QQmlApplicationEngine& engine) { return static_cast<int>(engine.rootObjects().size()); }

bool harness_set_property(QObject& root, const QString& object_name, const QString& property, const QString& value)
{
    QObject* object = root.objectName() == object_name ? &root : root.findChild<QObject*>(object_name);
    return object != nullptr && QQmlProperty(object, property, qmlContext(object)).write(QVariant(value));
}

// Deletes a named object at once, as a window's teardown deletes what it holds (EmuSen_Kit.md §7.40).
bool harness_destroy(QObject& root, const QString& object_name)
{
    QObject* object = root.findChild<QObject*>(object_name);
    delete object;
    return object != nullptr;
}

namespace {

QObject* named(QObject& root, const QString& object_name)
{
    return root.objectName() == object_name ? &root : root.findChild<QObject*>(object_name);
}

QString spyName(const QString& property) { return QStringLiteral("harness-notify:") + property; }

// QSignalSpy is no QObject in Qt 6.9 on, so a child object holds one, to live and be found with the object it watches.
class Watch : public QObject {
public:
    Watch(QObject* object, const QMetaMethod& signal) : spy(object, signal) {}
    QSignalSpy spy;
};

}

// A property's change signal counted from now: a QSignalSpy on its notify signal, a child of the object, so it lives as long as it does.
bool harness_watch(QObject& root, const QString& object_name, const QString& property)
{
    QObject* object = named(root, object_name);
    if (object == nullptr) {
        return false;
    }
    const QMetaObject* meta = object->metaObject();
    const QMetaProperty p = meta->property(meta->indexOfProperty(property.toUtf8().constData()));
    if (!p.isValid() || !p.hasNotifySignal()) {
        return false;
    }
    auto* watch = new Watch(object, p.notifySignal());
    watch->setObjectName(spyName(property));
    watch->setParent(object);
    return watch->spy.isValid();
}

// How many times the watched property said it changed; -1 where it is not watched.
int harness_notified(QObject& root, const QString& object_name, const QString& property)
{
    QObject* object = named(root, object_name);
    QObject* child = object != nullptr ? object->findChild<QObject*>(spyName(property), Qt::FindDirectChildrenOnly) : nullptr;
    const auto* watch = dynamic_cast<Watch*>(child);
    return watch != nullptr ? static_cast<int>(watch->spy.count()) : -1;
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

// The frames the window has swapped since the first call, counted at Qt's own frameSwapped (EmuSen_Qt.md §9.3.2).
std::uint64_t harness_frames(QObject& window)
{
    QQuickWindow* w = quick(window);
    if (w == nullptr) {
        return 0;
    }
    if (!w->property("harnessFrames").isValid()) {
        w->setProperty("harnessFrames", QVariant::fromValue<qulonglong>(0));
        QObject::connect(w, &QQuickWindow::frameSwapped, w, [w] { w->setProperty("harnessFrames", w->property("harnessFrames").toULongLong() + 1); });
    }
    return w->property("harnessFrames").toULongLong();
}

namespace {

// An animation running, an item's own or inside one of its objects (a Behavior's), not inside a child item.
bool animating(QObject* object)
{
    return (object->inherits("QQuickFrameAnimation") || object->inherits("QQuickAbstractAnimation")) && object->property("running").toBool();
}

// Whether an animation repeats without end, itself or a group it runs in: a design's resting motion, not a transition.
bool looping(QObject* object)
{
    for (QObject* at = object; at != nullptr; at = at->parent()) {
        if (at->inherits("QQuickFrameAnimation") || (at->inherits("QQuickAbstractAnimation") && at->property("loops").toInt() < 0)) {
            return true;
        }
    }
    return false;
}

// Each running animation under an item, as "Class in name", the name its nearest named item's, and "looping" when it never ends.
void runningUnder(QQuickItem* item, const QString& named, QStringList& out)
{
    const QString here = item->objectName().isEmpty() ? named : item->objectName();
    for (QObject* object : item->findChildren<QObject*>(Qt::FindDirectChildrenOnly)) {
        if (qobject_cast<QQuickItem*>(object) != nullptr) {
            continue;
        }
        QList<QObject*> inside = object->findChildren<QObject*>();
        inside.prepend(object);
        for (QObject* each : inside) {
            if (animating(each)) {
                out.append(QString::fromLatin1(each->metaObject()->className()) + QLatin1String(" in ") + here + (looping(each) ? QLatin1String(" looping") : QLatin1String("")));
            }
        }
    }
    for (QQuickItem* child : item->childItems()) {
        runningUnder(child, here, out);
    }
}

}

QString harness_running(QObject& window)
{
    QStringList out;
    if (QQuickWindow* w = quick(window)) {
        runningUnder(w->contentItem(), QString(), out);
    }
    out.removeDuplicates();
    return out.join(QLatin1Char('\n'));
}

void harness_drag(QObject& window, double x0, double y0, double x1, double y1)
{
    QQuickWindow* w = quick(window);
    if (w == nullptr) {
        return;
    }
    const auto at = [&](double t) { return QPoint(static_cast<int>(x0 + (x1 - x0) * t), static_cast<int>(y0 + (y1 - y0) * t)); };
    QTest::mousePress(w, Qt::LeftButton, Qt::NoModifier, at(0));
    for (int step = 1; step <= 8; ++step) {
        QTest::mouseMove(w, at(step / 8.0), 10);
    }
    QTest::mouseRelease(w, Qt::LeftButton, Qt::NoModifier, at(1));
}
