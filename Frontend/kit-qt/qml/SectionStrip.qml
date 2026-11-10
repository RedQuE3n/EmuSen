pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls
import EmuSen.Kit as Kit

// A screen's section strip: the frame's sections in Caption upper case, the chosen one strong over the accent bar, only whole names drawn; a click chooses one as the shoulders step them (EmuSen_Kit.md §7.43).
ListView {
    id: root
    required property Kit.FrameController controller
    orientation: ListView.Horizontal
    interactive: false
    clip: true
    spacing: Kit.Tokens.base * 2
    model: root.controller.sections
    currentIndex: root.controller.sections.indexOf(root.controller.selectedSection)
    highlightRangeMode: ListView.ApplyRange
    preferredHighlightBegin: 0
    preferredHighlightEnd: root.width
    highlightMoveDuration: Kit.Tokens.instant
    delegate: TabButton {
        id: tab
        required property string modelData
        readonly property real shownX: tab.x - root.contentX
        visible: tab.shownX >= 0 && tab.shownX + tab.width <= root.width
        height: root.height
        text: tab.modelData
        font.pixelSize: Kit.Tokens.fontCaption
        font.capitalization: Font.AllUppercase
        focusPolicy: Qt.NoFocus
        checkable: false
        checked: tab.modelData === root.controller.selectedSection
        onClicked: root.controller.selectedSection = tab.modelData
    }
}
