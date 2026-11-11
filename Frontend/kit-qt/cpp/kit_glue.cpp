#include "kit_glue.h"

#include <QtCore/QCoreApplication>
#include <QtCore/QEvent>
#include <QtCore/QMetaObject>
#include <QtCore/QObject>
#include <QtGui/QGuiApplication>
#include <QtGui/QKeyEvent>
#include <QtGui/QWindow>
#include <QtQuick/QQuickItem>
#include <QtQuick/QQuickWindow>

bool send_key(int key, int modifiers)
{
    QWindow* window = QGuiApplication::focusWindow();
    if (window == nullptr) {
        return false;
    }
    auto* quick = qobject_cast<QQuickWindow*>(window);
    const bool focused = quick == nullptr || quick->activeFocusItem() != nullptr;
    QKeyEvent press(QEvent::KeyPress, key, Qt::KeyboardModifiers::fromInt(modifiers));
    QKeyEvent release(QEvent::KeyRelease, key, Qt::KeyboardModifiers::fromInt(modifiers));
    QCoreApplication::sendEvent(window, &press);
    QCoreApplication::sendEvent(window, &release);
    return focused && press.isAccepted();
}

bool pass_key(QQuickItem* item, int key, int modifiers)
{
    for (QQuickItem* up = item != nullptr ? item->parentItem() : nullptr; up != nullptr; up = up->parentItem()) {
        QKeyEvent press(QEvent::KeyPress, key, Qt::KeyboardModifiers::fromInt(modifiers));
        QCoreApplication::sendEvent(up, &press);
        if (press.isAccepted()) {
            QKeyEvent release(QEvent::KeyRelease, key, Qt::KeyboardModifiers::fromInt(modifiers));
            QCoreApplication::sendEvent(up, &release);
            return true;
        }
    }
    return false;
}

void make_current(QQuickItem* item)
{
    for (QQuickItem* delegate = item; delegate != nullptr; delegate = delegate->parentItem()) {
        QQuickItem* content = delegate->parentItem();
        QQuickItem* view = content != nullptr ? content->parentItem() : nullptr;
        if (view == nullptr || !view->inherits("QQuickItemView") || view->property("contentItem").value<QQuickItem*>() != content) {
            continue;
        }
        // The view's own current item, focused as the view makes it, is not yet placed, so its box says nothing.
        if (view->property("currentItem").value<QQuickItem*>() != delegate) {
            int index = -1;
            const QPointF centre = delegate->position() + QPointF(delegate->width() / 2, delegate->height() / 2);
            QMetaObject::invokeMethod(view, "indexAt", Q_RETURN_ARG(int, index), Q_ARG(qreal, centre.x()), Q_ARG(qreal, centre.y()));
            if (index >= 0) {
                view->setProperty("currentIndex", index);
            }
        }
        return;
    }
}

namespace {

const char* const POINTER_WATCH = "EmuSen.Kit pointer watch";

// The window's one pointer fact: a real move, press or touch of the pointer is being delivered to it, until the event loop next turns (EmuSen_Kit.md §7.108).
class PointerWatch : public QObject {
public:
    explicit PointerWatch(QWindow* window) : QObject(window) { setObjectName(QString::fromLatin1(POINTER_WATCH)); }
    bool moving = false;

protected:
    bool eventFilter(QObject* watched, QEvent* event) override
    {
        switch (event->type()) {
        case QEvent::MouseMove:
        case QEvent::MouseButtonPress:
        case QEvent::TabletMove:
        case QEvent::TabletPress:
        case QEvent::TouchBegin:
        case QEvent::TouchUpdate:
            if (!moving) {
                moving = true;
                QMetaObject::invokeMethod(this, [this] { moving = false; }, Qt::QueuedConnection);
            }
            break;
        default:
            break;
        }
        return QObject::eventFilter(watched, event);
    }
};

PointerWatch* watch_of(QWindow* window)
{
    return static_cast<PointerWatch*>(window->findChild<QObject*>(QString::fromLatin1(POINTER_WATCH), Qt::FindDirectChildrenOnly));
}

void start_watch(QWindow* window)
{
    if (watch_of(window) == nullptr) {
        window->installEventFilter(new PointerWatch(window));
    }
}

} // namespace

void watch_windows()
{
    for (QWindow* window : QGuiApplication::allWindows()) {
        if (qobject_cast<QQuickWindow*>(window) != nullptr) {
            start_watch(window);
        }
    }
}

bool pointer_moving(QQuickItem* item)
{
    QQuickWindow* window = item != nullptr ? item->window() : nullptr;
    if (window == nullptr) {
        return false;
    }
    const PointerWatch* watch = watch_of(window);
    if (watch == nullptr) {
        // Unwatched, it cannot tell a move from a frame's hover, so it says no (EmuSen_Kit.md §7.108).
        start_watch(window);
        return false;
    }
    return watch->moving;
}
