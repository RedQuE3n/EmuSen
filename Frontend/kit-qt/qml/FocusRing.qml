pragma ComponentBehavior: Bound
import QtQuick
import EmuSen.Kit as Kit

// The focus mark around the box it fills: a gap, the ring and the glow under them, sized and coloured by the tokens (EmuSen_Kit.md §7.113).
Item {
    id: root
    enum Placement { Gapped, Tight, Edge }
    // The box's corner radius.
    property real radius
    // Where the ring stands to the box: past a gap of ringOffset, within ringOffset of it, or on its edge (§7.113).
    property int placement: FocusRing.Gapped
    // The placement drawn: every ring Gapped where the look says so (ringGapped, R-20261114-21), else the place's own.
    readonly property int form: Kit.Tokens.ringGapped ? FocusRing.Gapped : root.placement
    // True while the ring is the box's own border: a box binds its border to edgeWidth and edgeColor only then.
    readonly property bool onEdge: root.form === FocusRing.Edge
    // The ring as the box's own border, for a box that draws it in the Edge placement.
    readonly property real edgeWidth: Kit.Tokens.ringThickness
    readonly property color edgeColor: Kit.Tokens.accent
    // How far past the box the ring's inner edge stands.
    readonly property real inner: root.form === FocusRing.Gapped ? Kit.Tokens.ringOffset
        : (root.form === FocusRing.Tight ? Kit.Tokens.ringOffset - Kit.Tokens.ringThickness : -Kit.Tokens.ringThickness)
    readonly property real outer: root.inner + Kit.Tokens.ringThickness
    Kit.Glow {
        objectName: "focusGlow"
        anchors.fill: parent
        radius: root.radius
        color: Kit.Tokens.glowColor
        blur: Kit.Tokens.glowBlur
        spread: Kit.Tokens.glowSpread
    }
    Rectangle {
        objectName: "focusGap"
        visible: root.inner > 0 && Kit.Tokens.ringGapColor.a > 0
        x: -root.inner
        y: -root.inner
        width: root.width + 2 * root.inner
        height: root.height + 2 * root.inner
        radius: root.radius + root.inner
        color: "transparent"
        border.width: root.inner
        border.color: Kit.Tokens.ringGapColor
    }
    Rectangle {
        objectName: "focusBand"
        visible: root.form !== FocusRing.Edge
        x: -root.outer
        y: -root.outer
        width: root.width + 2 * root.outer
        height: root.height + 2 * root.outer
        radius: root.radius + Kit.Tokens.ringOffset
        color: "transparent"
        border.width: Kit.Tokens.ringThickness
        border.color: Kit.Tokens.accent
    }
}
