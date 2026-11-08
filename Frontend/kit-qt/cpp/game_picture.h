// EmuSen.Kit.GamePicture: a picture drawn on the scene graph's own Vulkan device, sampled where it lies (EmuSen_Kit.md §7.16).
#pragma once

#include <QtCore/QString>
#include <QtQml/qqmlregistration.h>
#include <QtQuick/QQuickItem>

#include <atomic>
#include <cstdint>

class QQuickWindow;
class QSGTexture;

// The image a feed drew this frame: Vulkan's VkImage as an integer, its size, VkFormat and VkImageLayout.
struct EmuSenPicture {
    uint64_t image;
    uint32_t width, height, format, layout;
};

// A host's picture: both calls come on the render thread, `draw` at the item's sync and `swapped` after each swap.
struct EmuSenPictureFeed {
    void* context;
    int32_t (*draw)(void* context, int32_t width, int32_t height, EmuSenPicture* out);
    void (*swapped)(void* context, int64_t nanoseconds);
};

class GamePicture : public QQuickItem {
    Q_OBJECT
    QML_ELEMENT
    Q_PROPERTY(qulonglong feed READ feed WRITE setFeed NOTIFY feedChanged)
    Q_PROPERTY(bool live READ live WRITE setLive NOTIFY liveChanged)

public:
    explicit GamePicture(QQuickItem* parent = nullptr);

    qulonglong feed() const { return m_feed; }
    void setFeed(qulonglong feed);
    bool live() const { return m_live; }
    void setLive(bool live);

Q_SIGNALS:
    void feedChanged();
    void liveChanged();

protected:
    QSGNode* updatePaintNode(QSGNode* old, UpdatePaintNodeData* data) override;
    void itemChange(ItemChange change, const ItemChangeData& value) override;
    void releaseResources() override;

private:
    void attach(QQuickWindow* window);
    void onSwapped();

    qulonglong m_feed = 0;
    bool m_live = false;
    QMetaObject::Connection m_swapped;
    // What the render thread read at the last sync, for its swaps.
    std::atomic<qulonglong> m_synced{0};
    std::atomic<bool> m_syncedLive{false};
};

// The window's scene graph made on a device the host made (EmuSen_Kit.md §7.16): Vulkan forced, the instance and the
// device adopted. Called before the window is first shown; empty on success, else the words why.
QString kit_adopt_vulkan(QQuickWindow& window, uint64_t instance, uint64_t physical, uint64_t device, uint32_t family, uint32_t index, const QStringList& extensions);
