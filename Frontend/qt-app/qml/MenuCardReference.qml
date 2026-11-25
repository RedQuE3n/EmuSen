pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Effects
import EmuSen.Kit as Kit
import EmuSen.Models

// Reference theme Card layout preserved pixel-identical (R-20261114-21).
FocusScope {
    id: root
    required property var controller
    required property var rows
    property string title
    property string breadcrumb
    property string footer
    property string cover
    property int current: 0
    property var paneLines
    property var paneMuted
    property string panePicture
    property string paneCaption
    property bool paneDrawn: false
    property HelpModel help
    property int padFamily: Kit.Families.Generic
    property real slide: 1
    readonly property real panelWidth: Math.round(root.width * 0.4)
    readonly property real bandHeight: 98
    readonly property bool paneShown: (root.paneLines?.length ?? 0) > 0 || root.panePicture.length > 0 || root.paneCaption.length > 0 || root.paneDrawn
    readonly property real shadowOpacity: 0.9
    readonly property real shadowRadius: 48
    readonly property Item paneSlot: refPaneSlot
    signal chosen(int index)
    signal stepped(int index, int by, bool held)
    signal focused(int index)
    signal keyed(int key)
    onCurrentChanged: list.currentIndex = root.current
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
            objectName: "AdaptiveDetail"
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
                font.family: Kit.Tokens.faceText
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
                        font.family: Kit.Tokens.faceText
                        font.pixelSize: 22
                        wrapMode: Text.WordWrap
                    }
                    Text {
                        width: parent.width
                        visible: text.length > 0
                        text: root.paneMuted?.[said.index] ?? ""
                        color: Kit.Tokens.muted
                        font.family: Kit.Tokens.faceText
                        font.pixelSize: 20
                        wrapMode: Text.WordWrap
                    }
                }
            }
        }
        Item {
            id: refPaneSlot
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
    Item {
        id: panelGroup
        x: -(root.panelWidth + 40) * root.slide
        width: root.width
        height: root.height
        Rectangle {
            id: caster
            objectName: "cardShadowCaster"
            width: root.panelWidth
            height: root.height
            color: Kit.Tokens.surface
            visible: false
        }
        MultiEffect {
            source: caster
            anchors.fill: caster
            shadowEnabled: true
            shadowColor: Kit.Tokens.shadowColor
            shadowOpacity: root.shadowOpacity
            shadowVerticalOffset: Kit.Tokens.shadowOffsetY
            blurMax: root.shadowRadius
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
                font.family: Kit.Tokens.faceText
                font.pixelSize: 20
                elide: Text.ElideRight
            }
            Text {
                objectName: "cardTitle"
                width: parent.width
                text: root.title
                color: Kit.Tokens.textStrong
                font.family: Kit.Tokens.faceLabel
                font.weight: Kit.Tokens.weightTitle
                font.pixelSize: 32
                fontSizeMode: Text.HorizontalFit
                minimumPixelSize: 22
                elide: Text.ElideRight
            }
        }
        Text {
            objectName: "cardFooter"
            x: 64
            y: 156 + list.count * Kit.Tokens.rowHeight + 22
            width: root.panelWidth - 88
            visible: text.length > 0
            text: root.footer
            color: Kit.Tokens.muted
            font.family: Kit.Tokens.faceText
            font.pixelSize: 26
            wrapMode: Text.WordWrap
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
                    controller: root.controller
                    label: row.model.label
                    value: row.model.value
                    steps: row.model.steps
                    opens: row.model.opens
                    ruled: true
                    sectionEnd: row.model.sectionEnd
                    inset: 24
                    labelFont: Qt.font({ family: Kit.Tokens.faceText, pixelSize: Kit.Tokens.fontRow })
                    current: ListView.isCurrentItem
                    focus: ListView.isCurrentItem
                    onActivated: root.chosen(row.index)
                    onActiveFocusChanged: list.currentIndex = row.activeFocus ? row.index : list.currentIndex
                    Keys.onLeftPressed: (event) => root.stepped(row.index, -1, event.isAutoRepeat)
                    Keys.onRightPressed: (event) => root.stepped(row.index, 1, event.isAutoRepeat)
                    Keys.onPressed: (event) => root.keyed(event.key)
                }
            }
        }
    }
    Kit.HelpBar {
        objectName: "cardHelp"
        y: band.y + (root.bandHeight - Kit.Tokens.helpBar) / 2
        width: root.width
        controller: root.controller
        model: root.help
        family: root.padFamily
        color: Kit.Tokens.surface
        ruled: false
        leftInset: 64
        labelFont: Qt.font({ family: Kit.Tokens.faceText, pixelSize: 20 })
        entrySpacing: 40
        glyphGap: 10
    }
    Item {
        objectName: "bandShadow"
        x: panelGroup.x + root.panelWidth
        y: band.y
        width: 64
        height: root.bandHeight
        clip: true
        MultiEffect {
            x: -root.panelWidth
            y: -band.y
            width: root.panelWidth
            height: root.height
            source: caster
            shadowEnabled: true
            shadowColor: Kit.Tokens.shadowColor
            shadowOpacity: root.shadowOpacity
            shadowVerticalOffset: Kit.Tokens.shadowOffsetY
            blurMax: root.shadowRadius
            shadowBlur: 1
        }
    }
}
