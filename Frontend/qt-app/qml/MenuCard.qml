pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Effects
import EmuSen.Kit as Kit
import EmuSen.Models

// The Card form, one drawing for every card (EmuSen_Interface.md §2.3, §2.8, §18.1, §20.6; EmuSen_Qt.md §7.1, stage 2): the scrim, the panel at the left edge with its shadow, the title row, the page's rows, the pane beside them, the help band, sliding in and out. The host's model gives the words; the pane is lines or a picture, or the host's own drawing in this item's children.
FocusScope {
    id: root
    objectName: "PadCard"
    // The page's rows, with the roles label, value, steps, opens and sectionEnd.
    property var rows
    property string title
    property string breadcrumb
    // A cover's URL before the title, the Pause Card's; empty for none.
    property string cover
    // The row the page opens on, as the host's model says it.
    property int current: 0
    // The focused row's pane: lines with a muted line under each, or a picture with its caption.
    property var paneLines
    property var paneMuted
    property string panePicture
    property string paneCaption
    // The pane's own drawing, for a card whose pane holds more than lines or a picture (the rewind reel, the slots): the host's items, declared inside the card, are laid in the pane's box.
    default property alias paneContent: paneArea.data
    property HelpModel help
    property int padFamily: Kit.Families.Generic
    // Whether the card is on screen; it slides in as this turns on and out as it turns off.
    property bool open: false
    readonly property real panelWidth: Math.round(root.width * 0.4)
    readonly property real bandHeight: 98
    // 1 out of place, 0 in place: the panel moves left by its right edge and 40 units, the band down by its height, the pane fades (LunaP §218).
    property real slide: 1
    readonly property bool leaving: !root.open && root.slide < 1
    readonly property bool paneShown: (root.paneLines?.length ?? 0) > 0 || root.panePicture.length > 0 || root.paneCaption.length > 0 || paneArea.children.length > 0
    readonly property alias controller: frame.controller
    signal chosen(int index)
    // Left (-1) or Right (+1) on a row; `held` while the press repeats.
    signal stepped(int index, int by, bool held)
    signal focused(int index)
    signal backed()
    signal acted(string key)
    // The card has left the screen.
    signal gone()
    Accessible.role: Accessible.Pane
    Accessible.name: root.title
    onOpenChanged: root.open ? enter.restart() : exit.restart()
    onCurrentChanged: list.currentIndex = root.current
    NumberAnimation {
        id: enter
        target: root
        property: "slide"
        to: 0
        duration: Kit.Tokens.enter
        easing.type: Kit.Tokens.enterCurve
    }
    NumberAnimation {
        id: exit
        target: root
        property: "slide"
        to: 1
        duration: Kit.Tokens.exit
        easing.type: Kit.Tokens.exitCurve
        onFinished: root.gone()
    }
    Faces {
        id: faces
    }
    Rectangle {
        objectName: "AdaptiveCardScrim"
        anchors.fill: parent
        color: Kit.Tokens.scrim
    }
    Rectangle {
        objectName: "AdaptiveCardPane"
        x: 576
        y: 156
        width: 640
        height: root.height - 156 - 114
        radius: Kit.Tokens.tileRadius
        color: Kit.Tokens.surfaceRaised
        visible: root.paneShown
        opacity: 1 - root.slide
        Column {
            objectName: "PaneLines"
            x: 24
            y: 24
            width: parent.width - 48
            spacing: 4
            Image {
                objectName: "PanePicture"
                width: parent.width
                height: 444
                visible: root.panePicture.length > 0
                source: root.panePicture
                fillMode: Image.PreserveAspectFit
                asynchronous: true
            }
            Text {
                objectName: "PaneCaption"
                width: parent.width
                visible: root.paneCaption.length > 0
                text: root.paneCaption
                color: Kit.Tokens.muted
                font.family: faces.regular
                font.pixelSize: 20
                elide: Text.ElideRight
            }
            Repeater {
                model: root.paneLines
                Column {
                    id: said
                    required property int index
                    required property string modelData
                    width: parent.width
                    Text {
                        width: parent.width
                        visible: said.modelData.length > 0
                        text: said.modelData
                        color: Kit.Tokens.textStrong
                        font.family: faces.regular
                        font.pixelSize: 22
                        wrapMode: Text.WordWrap
                    }
                    Text {
                        width: parent.width
                        visible: text.length > 0
                        text: root.paneMuted?.[said.index] ?? ""
                        color: Kit.Tokens.muted
                        font.family: faces.regular
                        font.pixelSize: 20
                        wrapMode: Text.WordWrap
                    }
                }
            }
        }
        Item {
            id: paneArea
            anchors.fill: parent
        }
    }
    Rectangle {
        id: band
        objectName: "cardBand"
        x: 0
        y: root.height - root.bandHeight + root.bandHeight * root.slide
        width: root.width
        height: root.bandHeight
        color: Kit.Tokens.surface
        Rectangle {
            width: parent.width
            height: Kit.Tokens.hoverBorder
            color: Kit.Tokens.border
        }
    }
    Kit.Frame {
        id: frame
        x: -(root.panelWidth + 40) * root.slide
        width: root.width
        height: root.height
        surfaceDrawn: false
        helpShown: false
        padFamily: root.padFamily
        focus: true
        controller.navigationShown: false
        help: root.help
        Connections {
            target: frame.controller
            function onBackDeclined() { root.backed() }
            function onActionInvoked(key) { root.acted(key) }
        }
        Item {
            x: -frame.safeX
            y: -frame.safeY
            width: root.width
            height: root.height
            Rectangle {
                id: caster
                width: root.panelWidth
                height: root.height
                color: Kit.Tokens.surface
                visible: false
            }
            // One shadow on the card's open edge: offset 0,8, blur 24, black at 50% (EmuSen_Interface.md §5.4).
            MultiEffect {
                source: caster
                anchors.fill: caster
                shadowEnabled: true
                shadowColor: "#000000"
                shadowOpacity: 0.5
                shadowVerticalOffset: 8
                blurMax: 24
                shadowBlur: 1
            }
            Rectangle {
                objectName: "AdaptiveCard"
                width: root.panelWidth
                height: root.height
                color: Kit.Tokens.surface
                Accessible.role: Accessible.PopupMenu
                Accessible.name: root.title
            }
            Image {
                id: coverArt
                objectName: "cardCover"
                x: 64
                y: 50
                width: 60
                height: 80
                visible: root.cover.length > 0
                source: root.cover
                fillMode: Image.PreserveAspectFit
                asynchronous: true
            }
            Column {
                x: coverArt.visible ? coverArt.x + coverArt.width + 16 : 64
                width: root.panelWidth - x - 24
                anchors.verticalCenter: coverArt.verticalCenter
                Text {
                    objectName: "cardBreadcrumb"
                    width: parent.width
                    visible: text.length > 0
                    text: root.breadcrumb
                    color: Kit.Tokens.muted
                    font.family: faces.regular
                    font.pixelSize: 20
                    elide: Text.ElideRight
                }
                Text {
                    objectName: "cardTitle"
                    width: parent.width
                    text: root.title
                    color: Kit.Tokens.textStrong
                    font.family: faces.condensedRegular
                    font.weight: Font.Normal
                    font.pixelSize: 32
                    fontSizeMode: Text.HorizontalFit
                    minimumPixelSize: 22
                    elide: Text.ElideRight
                }
            }
            Item {
                objectName: "AdaptiveCardRows"
                x: 64
                y: 156
                width: root.panelWidth - 88
                height: list.count * Kit.Tokens.rowHeight
                ListView {
                    id: list
                    objectName: "PadMenuList"
                    anchors.fill: parent
                    model: root.rows
                    focus: true
                    interactive: false
                    keyNavigationEnabled: true
                    keyNavigationWraps: false
                    highlightMoveDuration: Kit.Tokens.instant
                    highlightResizeDuration: Kit.Tokens.instant
                    Accessible.role: Accessible.List
                    onCurrentIndexChanged: root.focused(list.currentIndex)
                    delegate: Kit.MenuRow {
                        id: row
                        required property int index
                        required property var model
                        width: ListView.view.width
                        controller: frame.controller
                        label: row.model.label
                        value: row.model.value
                        steps: row.model.steps
                        opens: row.model.opens
                        ruled: true
                        sectionEnd: row.model.sectionEnd
                        inset: 24
                        labelFont: Qt.font({ family: faces.regular, pixelSize: Kit.Tokens.fontRow })
                        current: ListView.isCurrentItem
                        focus: ListView.isCurrentItem
                        onActivated: root.chosen(row.index)
                        onActiveFocusChanged: list.currentIndex = row.activeFocus ? row.index : list.currentIndex
                        Keys.onLeftPressed: (event) => root.stepped(row.index, -1, event.isAutoRepeat)
                        Keys.onRightPressed: (event) => root.stepped(row.index, 1, event.isAutoRepeat)
                    }
                }
            }
        }
    }
    Kit.HelpBar {
        objectName: "cardHelp"
        y: band.y + (root.bandHeight - Kit.Tokens.helpBar) / 2
        width: root.width
        controller: frame.controller
        model: frame.help
        family: root.padFamily
        color: Kit.Tokens.surface
        ruled: false
        leftInset: 64
        labelFont: Qt.font({ family: faces.regular, pixelSize: 20 })
        entrySpacing: 40
        glyphGap: 10
    }
}
