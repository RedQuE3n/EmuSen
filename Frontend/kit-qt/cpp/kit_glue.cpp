#include "kit_glue.h"

#include <QtCore/QCoreApplication>
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
