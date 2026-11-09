import QtQuick
import EmuSen.Kit as Kit

// A carousel over PathView, an element the frame walks to: as many whole items as fit, an odd number about the centre, so none rests outside its box on any renderer, one step per wheel notch, a swipe is PathView's own flick, Reel between items, hovering focuses it and a click is A (EmuSen_Qt.md §1.7, EmuSen_Kit.md §7.8).
PathView {
    id: root
    required property Kit.FrameController controller
    property real itemWidth: Kit.Tokens.tileWidth + Kit.Tokens.tileGap
    readonly property int fits: Math.max(1, Math.floor((root.width + Kit.Tokens.tileGap) / root.itemWidth))
    pathItemCount: root.fits - 1 + root.fits % 2
    preferredHighlightBegin: 0.5
    preferredHighlightEnd: 0.5
    highlightRangeMode: PathView.StrictlyEnforceRange
    snapMode: PathView.SnapOneItem
    highlightMoveDuration: Kit.Tokens.reel
    focus: true
    activeFocusOnTab: true
    Accessible.focusable: true
    clip: true
    path: Path {
        startX: (root.width - root.pathItemCount * root.itemWidth) / 2
        startY: root.height / 2
        PathLine {
            x: (root.width + root.pathItemCount * root.itemWidth) / 2
            y: root.height / 2
        }
    }
    HoverHandler {
        id: hover
        onHoveredChanged: root.controller.hover(root, hover.hovered, hover.point.scenePosition)
        onPointChanged: if (hover.hovered) root.controller.pointerMoved(hover.point.scenePosition)
    }
    TapHandler {
        acceptedButtons: Qt.LeftButton | Qt.RightButton | Qt.BackButton
        onTapped: (point, button) => root.controller.click(root, button)
    }
    WheelHandler {
        onWheel: (event) => event.angleDelta.y < 0 ? root.incrementCurrentIndex() : root.decrementCurrentIndex()
    }
    Keys.onLeftPressed: root.decrementCurrentIndex()
    Keys.onRightPressed: root.incrementCurrentIndex()
}
