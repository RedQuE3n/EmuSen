// EmuSen.Kit.EdgeMask: on the software backend, the layer it lies in multiplied by EdgeFade's ramps (EmuSen_Kit.md §7.35).
#pragma once

#include <QtQml/qqmlregistration.h>
#include <QtQuick/QQuickItem>

class EdgeMask : public QQuickItem {
    Q_OBJECT
    QML_ELEMENT
    Q_PROPERTY(bool fadeTop READ fadeTop WRITE setFadeTop NOTIFY edgesChanged)
    Q_PROPERTY(bool fadeBottom READ fadeBottom WRITE setFadeBottom NOTIFY edgesChanged)
    Q_PROPERTY(bool fadeLeft READ fadeLeft WRITE setFadeLeft NOTIFY edgesChanged)
    Q_PROPERTY(bool fadeRight READ fadeRight WRITE setFadeRight NOTIFY edgesChanged)
    Q_PROPERTY(double length READ length WRITE setLength NOTIFY edgesChanged)
    // Whether the layer it last painted on holds an alpha channel; false again whenever its size or visibility changes.
    Q_PROPERTY(bool primed READ primed NOTIFY primedChanged)

public:
    explicit EdgeMask(QQuickItem* parent = nullptr);

    bool fadeTop() const { return m_edges[0]; }
    bool fadeBottom() const { return m_edges[1]; }
    bool fadeLeft() const { return m_edges[2]; }
    bool fadeRight() const { return m_edges[3]; }
    double length() const { return m_length; }
    bool primed() const { return m_primed; }
    void setFadeTop(bool on) { setEdge(0, on); }
    void setFadeBottom(bool on) { setEdge(1, on); }
    void setFadeLeft(bool on) { setEdge(2, on); }
    void setFadeRight(bool on) { setEdge(3, on); }
    void setLength(double length);
    void setPrimed(bool primed);

Q_SIGNALS:
    void edgesChanged();
    void primedChanged();

protected:
    QSGNode* updatePaintNode(QSGNode* old, UpdatePaintNodeData* data) override;
    void itemChange(ItemChange change, const ItemChangeData& value) override;
    void geometryChange(const QRectF& now, const QRectF& was) override;

private:
    void setEdge(int edge, bool on);

    bool m_edges[4] = {false, false, false, false};
    double m_length = 0;
    bool m_primed = false;
};
