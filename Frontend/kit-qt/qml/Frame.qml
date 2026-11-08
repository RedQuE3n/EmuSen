pragma ComponentBehavior: Bound
import QtQuick
import EmuSen.Kit

// The frame in its three forms: the walk's roles laid out by form, the pad through PadRouter, the keys a control declined, the help bar on the foot (EmuSen_Kit.md §7.8).
FocusScope {
    id: root
    property alias form: frameController.form
    readonly property alias controller: frameController
    property alias navigation: navigationPane.data
    default property alias content: contentArea.data
    property alias detail: detailPane.data
    property alias actions: actionsRow.data
    property alias help: helpBar.model
    property real navigationWidth: Tokens.tileWidth + Tokens.tileGap * 2
    readonly property bool desktop: frameController.form === Kit.Desktop
    readonly property bool card: frameController.form === Kit.Card
    readonly property bool bigScreen: !desktop && !card
    readonly property bool detailBeside: frameController.hasDetail && frameController.detailBeside
    readonly property real safeX: desktop ? 0 : Tokens.safeAreaX
    readonly property real safeY: desktop ? 0 : Tokens.safeAreaY
    objectName: "frame"
    FrameController {
        id: frameController
    }
    Rectangle {
        anchors.fill: parent
        color: root.card ? Tokens.scrim : Tokens.surface
    }
    Item {
        id: navigationPane
        objectName: "navigation"
        visible: root.controller.navigationShown && !root.card
        x: root.safeX
        y: root.safeY
        width: root.desktop ? root.navigationWidth : root.width - 2 * root.safeX
        height: root.desktop ? actionsRow.y - y : Tokens.titleRow
    }
    Item {
        id: contentArea
        objectName: "content"
        x: root.card ? (root.width - Tokens.cardWidth) / 2 : (root.desktop && navigationPane.visible ? navigationPane.width + Tokens.paneGap : root.safeX)
        y: root.card ? root.safeY + Tokens.titleRow : (root.bigScreen && navigationPane.visible ? navigationPane.y + navigationPane.height + Tokens.paneGap : root.safeY)
        width: root.card ? Tokens.cardWidth : (root.detailBeside ? detailPane.x - x - Tokens.paneGap : root.width - x - root.safeX)
        height: actionsRow.y - y - (root.bigScreen || root.card ? Tokens.paneGap : 0)
        visible: !(root.bigScreen && root.controller.detailOpen && !root.detailBeside)
    }
    Item {
        id: detailPane
        objectName: "detail"
        visible: root.controller.hasDetail && (root.detailBeside || root.controller.detailOpen) && !root.card
        x: root.detailBeside ? root.width - root.safeX - Tokens.cardWidth : contentArea.x
        y: contentArea.y
        width: root.detailBeside ? Tokens.cardWidth : contentArea.width
        height: contentArea.height
    }
    Item {
        id: actionsRow
        objectName: "actions"
        visible: root.controller.actionsShown
        x: root.card ? contentArea.x : root.safeX
        width: root.card ? Tokens.cardWidth : root.width - 2 * root.safeX
        height: visible ? Tokens.controlHeight + Tokens.paneGap : 0
        y: helpBar.visible ? helpBar.y - height - Tokens.paneGap : root.height - root.safeY - height
    }
    HelpBar {
        id: helpBar
        objectName: "helpBar"
        controller: root.controller
        visible: !root.desktop
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.bottom: parent.bottom
        leftInset: root.safeX
    }
    Connections {
        target: PadRouter
        enabled: root.activeFocus
        function onPressed(button) { root.controller.offer(button) }
    }
    Timer {
        interval: 8
        repeat: true
        running: root.activeFocus && PadRouter.clockRunning
        onTriggered: PadRouter.pump()
    }
    Keys.onPressed: (event) => event.accepted = root.controller.key(event.key, event.modifiers)
}
