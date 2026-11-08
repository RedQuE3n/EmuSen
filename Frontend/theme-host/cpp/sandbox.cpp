// The theme host's C++ shells; the rules they apply are Rust's (src/policy.rs), the reasons EmuSen_Qt.md §3.5 as built.
#include "sandbox.h"

#include "emusen-theme-host/src/sandbox.cxxqt.h"

#include <QtCore/QEventLoop>
#include <QtCore/QFile>
#include <QtCore/QLibraryInfo>
#include <QtCore/QMutexLocker>
#include <QtCore/QTimer>
#include <QtNetwork/QNetworkRequest>
#include <QtQml/QQmlComponent>
#include <QtQml/QQmlContext>

namespace {

// The folder a refused URL is rewritten into: it never exists, so the engine reports the file missing, synchronously, with no network.
const QString REFUSED = QStringLiteral("/emusen-refused");

// Declines every request, whatever its URL, so XMLHttpRequest and a remote picture fail with a diagnostic.
class RefusingManager : public QNetworkAccessManager {
public:
    RefusingManager(SandboxLog& log, QObject* parent) : QNetworkAccessManager(parent), log(log) {}

protected:
    QNetworkReply* createRequest(Operation op, const QNetworkRequest& request, QIODevice*) override
    {
        log.add(QStringLiteral("%1: a theme reaches no network; the request was refused").arg(request.url().toString()));
        QNetworkRequest refused(QUrl(QStringLiteral("emusen-refused:") + request.url().toString()));
        return QNetworkAccessManager::createRequest(op, refused, nullptr);
    }

private:
    SandboxLog& log;
};

}

void SandboxLog::add(const QString& line)
{
    QMutexLocker hold(&mutex);
    lines.append(line);
}

QString SandboxLog::take()
{
    QMutexLocker hold(&mutex);
    const QString all = lines.join(QLatin1Char('\n'));
    lines.clear();
    return all;
}

QUrl SandboxInterceptor::intercept(const QUrl& url, DataType type)
{
    if (url.isLocalFile() && url.path().startsWith(REFUSED + QLatin1Char('/'))) {
        return url;
    }
    const QString address = url.toString();
    const QString local = url.isLocalFile() ? url.toLocalFile() : QString();
    QString text;
    if (policy_reads(policy, address, local, int(type))) {
        sandbox_read(address, text);
    }
    const QString refusal = policy_judge(policy, address, local, int(type), text);
    if (refusal.isEmpty()) {
        return url;
    }
    log.add(refusal);
    qWarning("theme sandbox: %s", qPrintable(refusal));
    return QUrl::fromLocalFile(REFUSED + QLatin1Char('/') + url.scheme() + QLatin1Char('/') + url.host() + url.path());
}

QNetworkAccessManager* SandboxNetwork::create(QObject* parent)
{
    return new RefusingManager(log, parent);
}

Sandbox::Sandbox(const Policy& policy) : interceptor(policy, log), network(log), engine(std::make_unique<QQmlEngine>())
{
    engine->setImportPathList({ sandbox_qt_qml(), QStringLiteral("qrc:/qt/qml"), QStringLiteral("qrc:/qt-project.org/imports") });
    engine->setPluginPathList({});
    engine->addUrlInterceptor(&interceptor);
    engine->setNetworkAccessManagerFactory(&network);
    engine->setOfflineStoragePath(QString());
    QObject::connect(engine.get(), &QQmlEngine::warnings, engine.get(), [this](const QList<QQmlError>& warnings) {
        for (const QQmlError& w : warnings) {
            log.add(w.toString());
        }
    });
}

Sandbox::~Sandbox()
{
    for (const QPointer<QQuickItem>& screen : screens) {
        delete screen.data();
    }
    engine.reset();
}

bool Sandbox::place(const QString& url, QQuickItem* slot)
{
    if (slot == nullptr) {
        return false;
    }
    QQmlComponent component(engine.get(), QUrl(url), QQmlComponent::PreferSynchronous);
    if (component.isLoading()) {
        QEventLoop loop;
        QObject::connect(&component, &QQmlComponent::statusChanged, &loop, &QEventLoop::quit);
        QTimer::singleShot(3000, &loop, &QEventLoop::quit);
        loop.exec();
    }
    if (!component.isReady()) {
        for (const QQmlError& e : component.errors()) {
            log.add(e.toString());
        }
        return false;
    }
    QObject* made = component.beginCreate(engine->rootContext());
    auto* item = qobject_cast<QQuickItem*>(made);
    if (item == nullptr) {
        if (made != nullptr) {
            component.completeCreate();
            delete made;
            log.add(QStringLiteral("%1: a screen's root is an Item").arg(url));
        }
        for (const QQmlError& e : component.errors()) {
            log.add(e.toString());
        }
        return false;
    }
    QQmlEngine::setObjectOwnership(item, QQmlEngine::CppOwnership);
    item->setParent(slot);
    item->setParentItem(slot);
    component.completeCreate();
    item->setSize(slot->size());
    QObject::connect(slot, &QQuickItem::widthChanged, item, [item, slot] { item->setWidth(slot->width()); });
    QObject::connect(slot, &QQuickItem::heightChanged, item, [item, slot] { item->setHeight(slot->height()); });
    item->setFocus(true);
    screens.append(item);
    return true;
}

std::unique_ptr<Sandbox> sandbox_new(const Policy& policy)
{
    return std::make_unique<Sandbox>(policy);
}

bool sandbox_place(Sandbox& sandbox, const QString& url, QQuickItem* slot)
{
    return sandbox.place(url, slot);
}

QString sandbox_take_log(Sandbox& sandbox)
{
    return sandbox.log.take();
}

QObject* sandbox_engine(const Sandbox& sandbox)
{
    return sandbox.engine.get();
}

bool sandbox_read(const QString& url, QString& text)
{
    const QUrl at(url);
    const QString path = at.isLocalFile() ? at.toLocalFile() : at.scheme() == QLatin1String("qrc") ? QLatin1Char(':') + at.path() : QString();
    QFile file(path);
    if (path.isEmpty() || !file.open(QIODevice::ReadOnly)) {
        return false;
    }
    text = QString::fromUtf8(file.readAll());
    return true;
}

QString sandbox_qt_qml()
{
    return QLibraryInfo::path(QLibraryInfo::QmlImportsPath);
}

std::unique_ptr<Watch> watch_new(QObject* object)
{
    return std::make_unique<Watch>(Watch { object });
}

bool watch_alive(const Watch& watch)
{
    return !watch.object.isNull();
}
