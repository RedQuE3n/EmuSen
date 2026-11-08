// The theme host's C++ shells, each with its reason in EmuSen_Qt.md §3.5 as built: cxx-qt declares no subclass of a Qt C++ interface.
#ifndef EMUSEN_THEME_HOST_SANDBOX_H
#define EMUSEN_THEME_HOST_SANDBOX_H

#include <QtCore/QMutex>
#include <QtCore/QPointer>
#include <QtCore/QStringList>
#include <QtNetwork/QNetworkAccessManager>
#include <QtQml/QQmlAbstractUrlInterceptor>
#include <QtQml/QQmlEngine>
#include <QtQml/QQmlNetworkAccessManagerFactory>
#include <QtQuick/QQuickItem>
#include <memory>

struct Policy;

// Lines from the engine's threads, the type loader's and the picture readers', taken by the host on its own.
class SandboxLog {
public:
    void add(const QString& line);
    QString take();

private:
    QMutex mutex;
    QStringList lines;
};

// Asks the policy of every URL the engine resolves; a refused one is rewritten to a scheme the refusing network declines.
class SandboxInterceptor : public QQmlAbstractUrlInterceptor {
public:
    SandboxInterceptor(const Policy& policy, SandboxLog& log) : policy(policy), log(log) {}
    QUrl intercept(const QUrl& url, DataType type) override;

private:
    const Policy& policy;
    SandboxLog& log;
};

// Makes the engine's network managers, each of which refuses every request.
class SandboxNetwork : public QQmlNetworkAccessManagerFactory {
public:
    explicit SandboxNetwork(SandboxLog& log) : log(log) {}
    QNetworkAccessManager* create(QObject* parent) override;

private:
    SandboxLog& log;
};

// One theme's engine; the screens it made go first, then the engine, then the shells it was given.
class Sandbox {
public:
    explicit Sandbox(const Policy& policy);
    ~Sandbox();
    bool place(const QString& url, QQuickItem* slot);
    SandboxLog log;
    SandboxInterceptor interceptor;
    SandboxNetwork network;
    std::unique_ptr<QQmlEngine> engine;
    QList<QPointer<QQuickItem>> screens;
};

// Whether an object still lives.
struct Watch {
    QPointer<QObject> object;
};

std::unique_ptr<Sandbox> sandbox_new(const Policy& policy);
bool sandbox_place(Sandbox& sandbox, const QString& url, QQuickItem* slot);
QString sandbox_take_log(Sandbox& sandbox);
QObject* sandbox_engine(const Sandbox& sandbox);
bool sandbox_read(const QString& url, QString& text);
QString sandbox_qt_qml();
std::unique_ptr<Watch> watch_new(QObject* object);
bool watch_alive(const Watch& watch);

#endif
