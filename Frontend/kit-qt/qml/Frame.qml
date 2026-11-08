pragma ComponentBehavior: Bound
import QtQuick
import EmuSen.Kit as Kit

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
    property real navigationWidth: Kit.Tokens.tileWidth + Kit.Tokens.tileGap * 2
    readonly property bool desktop: frameController.form === Kit.Forms.Desktop
    readonly property bool card: frameController.form === Kit.Forms.Card
    readonly property bool bigScreen: !desktop && !card
    readonly property bool detailBeside: frameController.hasDetail && frameController.detailBeside
    readonly property real safeX: desktop ? 0 : Kit.Tokens.safeAreaX
    readonly property real safeY: desktop ? 0 : Kit.Tokens.safeAreaY
    objectName: "frame"
    Kit.FrameController {
        id: frameController
    }
    Rectangle {
        anchors.fill: parent
        color: root.card ? Kit.Tokens.scrim : Kit.Tokens.surface
    }
    Item {
        id: navigationPane
        objectName: "navigation"
        visible: root.controller.navigationShown && !root.card
        x: root.safeX
        y: root.safeY
        width: root.desktop ? root.navigationWidth : root.width - 2 * root.safeX
        height: root.desktop ? actionsRow.y - y : Kit.Tokens.titleRow
    }
    Item {
        id: contentArea
        objectName: "content"
        x: root.card ? (root.width - Kit.Tokens.cardWidth) / 2 : (root.desktop && navigationPane.visible ? navigationPane.width + Kit.Tokens.paneGap : root.safeX)
        y: root.card ? root.safeY + Kit.Tokens.titleRow : (root.bigScreen && navigationPane.visible ? navigationPane.y + navigationPane.height + Kit.Tokens.paneGap : root.safeY)
        width: root.card ? Kit.Tokens.cardWidth : (root.detailBeside ? detailPane.x - x - Kit.Tokens.paneGap : root.width - x - root.safeX)
        height: actionsRow.y - y - (root.bigScreen || root.card ? Kit.Tokens.paneGap : 0)
        visible: !(root.bigScreen && root.controller.detailOpen && !root.detailBeside)
    }
    Item {
        id: detailPane
        objectName: "detail"
        visible: root.controller.hasDetail && (root.detailBeside || root.controller.detailOpen) && !root.card
        x: root.detailBeside ? root.width - root.safeX - Kit.Tokens.cardWidth : contentArea.x
        y: contentArea.y
        width: root.detailBeside ? Kit.Tokens.cardWidth : contentArea.width
        height: contentArea.height
    }
    Item {
        id: actionsRow
        objectName: "actions"
        visible: root.controller.actionsShown
        x: root.card ? contentArea.x : root.safeX
        width: root.card ? Kit.Tokens.cardWidth : root.width - 2 * root.safeX
        height: visible ? Kit.Tokens.controlHeight + Kit.Tokens.paneGap : 0
        y: helpBar.visible ? helpBar.y - height - Kit.Tokens.paneGap : root.height - root.safeY - height
    }
    Kit.HelpBar {
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
        target: Kit.PadRouter
        enabled: root.activeFocus || root.controller.sheetOpen
        function onPressed(button) { root.controller.offer(button) }
    }
    Timer {
        interval: 8
        repeat: true
        running: (root.activeFocus || root.controller.sheetOpen) && Kit.PadRouter.clockRunning
        onTriggered: Kit.PadRouter.pump()
    }
    Keys.onPressed: (event) => event.accepted = root.controller.key(event.key, event.modifiers)
}
