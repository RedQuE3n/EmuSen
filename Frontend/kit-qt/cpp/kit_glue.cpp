#include "kit_glue.h"

#include <QtCore/QCoreApplication>
#include <QtGui/QGuiApplication>
#include <QtGui/QKeyEvent>
#include <QtGui/QWindow>

void send_key(int key, int modifiers)
{
    QWindow* window = QGuiApplication::focusWindow();
    if (window == nullptr) {
        return;
    }
    QKeyEvent press(QEvent::KeyPress, key, Qt::KeyboardModifiers::fromInt(modifiers));
    QKeyEvent release(QEvent::KeyRelease, key, Qt::KeyboardModifiers::fromInt(modifiers));
    QCoreApplication::sendEvent(window, &press);
    QCoreApplication::sendEvent(window, &release);
}
