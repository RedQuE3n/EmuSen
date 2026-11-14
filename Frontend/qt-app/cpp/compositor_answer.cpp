#include "compositor_answer.h"

#include <QtCore/QEvent>

CompositorAnswer::CompositorAnswer(QObject* parent)
    : QObject(parent)
{
}

void CompositorAnswer::setWindow(QWindow* window)
{
    if (window == m_window) {
        return;
    }
    if (m_window != nullptr) {
        m_window->removeEventFilter(this);
    }
    m_window = window;
    if (m_window != nullptr) {
        m_window->installEventFilter(this);
    }
    emit windowChanged();
}

// Qt sends the window a spontaneous WindowStateChange once its platform applies a state, the state already the window's (EmuSen_Qt.md §7.7.H7).
bool CompositorAnswer::eventFilter(QObject* watched, QEvent* event)
{
    if (watched == m_window && event->type() == QEvent::WindowStateChange && event->spontaneous()) {
        const bool fullScreen = m_window->windowStates().testFlag(Qt::WindowFullScreen);
        if (fullScreen != m_fullScreen) {
            m_fullScreen = fullScreen;
            emit fullScreenChanged();
        }
    }
    return QObject::eventFilter(watched, event);
}
