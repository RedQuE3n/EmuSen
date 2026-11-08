import QtQuick
import EmuSen.Kit

// A carousel over PathView: one step per wheel notch, a swipe is PathView's own flick, Reel between items (EmuSen_Qt.md §1.7, EmuSen_Kit.md §7.8).
PathView {
    id: root
    property real itemWidth: Tokens.tileWidth + Tokens.tileGap
    pathItemCount: Math.ceil(width / itemWidth) + 2
    preferredHighlightBegin: 0.5
    preferredHighlightEnd: 0.5
    highlightRangeMode: PathView.StrictlyEnforceRange
    snapMode: PathView.SnapOneItem
    highlightMoveDuration: Tokens.reel
    focus: true
    path: Path {
        startX: -root.itemWidth / 2
        startY: root.height / 2
        PathLine {
            x: root.width + root.itemWidth / 2
            y: root.height / 2
        }
    }
    WheelHandler {
        onWheel: (event) => event.angleDelta.y < 0 ? root.incrementCurrentIndex() : root.decrementCurrentIndex()
    }
    Keys.onLeftPressed: root.decrementCurrentIndex()
    Keys.onRightPressed: root.incrementCurrentIndex()
}
