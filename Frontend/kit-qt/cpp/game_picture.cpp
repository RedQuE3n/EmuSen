#include "game_picture.h"

#include <QtGui/QVulkanInstance>
#include <QtQuick/QQuickGraphicsDevice>
#include <QtQuick/QQuickWindow>
#include <QtQuick/QSGSimpleTextureNode>
#include <QtQuick/qsgtexture_platform.h>

#include <chrono>

namespace {

// The texture node and what it shows, kept by the render thread.
struct PictureNode : QSGSimpleTextureNode {
    uint64_t image = 0;
    QSize size;
};

// Owns the adopted instance for as long as the window lives; a child, so it goes after the window's scene graph.
struct InstanceHolder : QObject {
    explicit InstanceHolder(QObject* parent) : QObject(parent) {}
    QVulkanInstance instance;
};

}

GamePicture::GamePicture(QQuickItem* parent) : QQuickItem(parent)
{
    setFlag(ItemHasContents, true);
}

void GamePicture::setFeed(qulonglong feed)
{
    if (feed == m_feed) {
        return;
    }
    m_feed = feed;
    update();
    Q_EMIT feedChanged();
}

void GamePicture::setLive(bool live)
{
    if (live == m_live) {
        return;
    }
    m_live = live;
    update();
    Q_EMIT liveChanged();
}

void GamePicture::itemChange(ItemChange change, const ItemChangeData& value)
{
    if (change == ItemSceneChange) {
        attach(value.window);
    }
    QQuickItem::itemChange(change, value);
}

void GamePicture::attach(QQuickWindow* window)
{
    QObject::disconnect(m_swapped);
    if (window != nullptr) {
        m_swapped = connect(window, &QQuickWindow::frameSwapped, this, &GamePicture::onSwapped, Qt::DirectConnection);
    }
}

void GamePicture::onSwapped()
{
    const auto feed = reinterpret_cast<EmuSenPictureFeed*>(m_synced.load());
    if (feed != nullptr && feed->swapped != nullptr) {
        const auto now = std::chrono::duration_cast<std::chrono::nanoseconds>(std::chrono::steady_clock::now().time_since_epoch());
        feed->swapped(feed->context, now.count());
    }
    if (m_syncedLive.load()) {
        QMetaObject::invokeMethod(this, &QQuickItem::update, Qt::QueuedConnection);
    }
}

QSGNode* GamePicture::updatePaintNode(QSGNode* old, UpdatePaintNodeData*)
{
    auto* node = static_cast<PictureNode*>(old);
    const auto feed = reinterpret_cast<EmuSenPictureFeed*>(m_feed);
    m_synced.store(m_feed);
    m_syncedLive.store(m_live);
    const qreal ratio = window() != nullptr ? window()->effectiveDevicePixelRatio() : 1.0;
    const QSize pixels(qMax(1, qRound(width() * ratio)), qMax(1, qRound(height() * ratio)));
    EmuSenPicture picture{};
    if (feed == nullptr || feed->draw == nullptr || feed->draw(feed->context, pixels.width(), pixels.height(), &picture) != 1 || picture.image == 0) {
        delete node;
        return nullptr;
    }
    if (node == nullptr) {
        node = new PictureNode;
        node->setOwnsTexture(true);
        node->setFiltering(QSGTexture::Nearest);
    }
    const QSize size(int(picture.width), int(picture.height));
    if (node->image != picture.image || node->size != size) {
        QSGTexture* texture = QNativeInterface::QSGVulkanTexture::fromNative(VkImage(picture.image), VkImageLayout(picture.layout), window(), size);
        node->setTexture(texture);
        node->image = picture.image;
        node->size = size;
    }
    node->setRect(boundingRect());
    node->markDirty(QSGNode::DirtyMaterial);
    return node;
}

void GamePicture::releaseResources()
{
    m_synced.store(0);
}

QString kit_adopt_vulkan(QQuickWindow& window, uint64_t instance, uint64_t physical, uint64_t device, uint32_t family, uint32_t index, const QStringList& extensions)
{
    auto* holder = new InstanceHolder(&window);
    holder->instance.setVkInstance(VkInstance(instance));
    holder->instance.setApiVersion(QVersionNumber(1, 2));
    QByteArrayList names;
    for (const QString& name : extensions) {
        names.append(name.toUtf8());
    }
    holder->instance.setExtensions(names);
    if (!holder->instance.create()) {
        const QString words = QStringLiteral("The scene graph could not adopt the Vulkan instance (%1).").arg(int(holder->instance.errorCode()));
        delete holder;
        return words;
    }
    window.setVulkanInstance(&holder->instance);
    window.setGraphicsDevice(QQuickGraphicsDevice::fromDeviceObjects(VkPhysicalDevice(physical), VkDevice(device), int(family), int(index)));
    return {};
}
