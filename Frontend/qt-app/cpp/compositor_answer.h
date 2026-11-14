// EmuSen.App.CompositorAnswer: whether the compositor's last answer gave a window full screen (EmuSen_Qt.md §7.7.H7).
#pragma once

#include <QtCore/QObject>
#include <QtCore/QPointer>
#include <QtGui/QWindow>
#include <QtQml/qqmlregistration.h>

class CompositorAnswer : public QObject {
    Q_OBJECT
    QML_ELEMENT
    Q_PROPERTY(QWindow* window READ window WRITE setWindow NOTIFY windowChanged)
    // The full-screen state the window's platform applied last, where QWindow::visibility is the state last asked.
    Q_PROPERTY(bool fullScreen READ fullScreen NOTIFY fullScreenChanged)

public:
    explicit CompositorAnswer(QObject* parent = nullptr);

    QWindow* window() const { return m_window; }
    void setWindow(QWindow* window);
    bool fullScreen() const { return m_fullScreen; }

Q_SIGNALS:
    void windowChanged();
    void fullScreenChanged();

protected:
    bool eventFilter(QObject* watched, QEvent* event) override;

private:
    QPointer<QWindow> m_window;
    bool m_fullScreen = false;
};
