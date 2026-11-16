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
    // The gap, the ring's thickness and the glow under them: the tokens', or a board's own where a look draws one apart.
    property real gap: Kit.Tokens.ringOffset
    property real thickness: Kit.Tokens.ringThickness
    property color glowColor: Kit.Tokens.glowColor
    property real glowBlur: Kit.Tokens.glowBlur
    property real glowSpread: Kit.Tokens.glowSpread
    // The ring's outer corners: the box's grown by the gap, as the kit has drawn it; a board drawing the ring as a box-shadow
    // grows them by the ring's whole reach (CSS Backgrounds 3 §6.1.1).
    property real bandRadius: root.radius + root.gap
    // True while the ring is the box's own border: a box binds its border to edgeWidth and edgeColor only then.
    readonly property bool onEdge: root.form === FocusRing.Edge
    // The ring as the box's own border, for a box that draws it in the Edge placement.
    readonly property real edgeWidth: root.thickness
    readonly property color edgeColor: Kit.Tokens.accent
    // How far past the box the ring's inner edge stands.
    readonly property real inner: root.form === FocusRing.Gapped ? root.gap
        : (root.form === FocusRing.Tight ? root.gap - root.thickness : -root.thickness)
    readonly property real outer: root.inner + root.thickness
    Kit.Glow {
        objectName: "focusGlow"
        anchors.fill: parent
        radius: root.radius
        color: root.glowColor
        blur: root.glowBlur
        spread: root.glowSpread
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
        radius: root.bandRadius
        color: "transparent"
        border.width: root.thickness
        border.color: Kit.Tokens.accent
    }
}
