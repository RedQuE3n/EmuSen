pragma ComponentBehavior: Bound
import QtQuick
import EmuSen.Kit as Kit
import EmuSen.Models

// Eclipse Pause Card detail pane: save slots (#6), disc cover (#38), reel/moment, and sentences.
Item {
    id: root
    required property PaneModel paneModel
    required property real unit
    required property real pixelScale
    readonly property real pad: 36 * root.unit
    readonly property real contentWidth: 688 * root.unit
    readonly property real contentHeight: root.paneModel.cover ? Math.max(coverFrame.height, coverTextCol.implicitHeight) : bodyCol.implicitHeight
    Item {
        id: coverRow
        objectName: "PaneCoverRow"
        visible: root.paneModel.cover
        x: root.pad
        y: root.pad
        width: root.contentWidth
        height: Math.max(coverFrame.height, coverTextCol.implicitHeight)
        Item {
            id: coverFrame
            objectName: "PaneCoverFrame"
            x: 0
            y: 0
            width: 241.15625 * root.unit
            height: 300 * root.unit
            Kit.Glow {
                objectName: "PaneCoverShadow"
                anchors.fill: parent
                radius: 16 * root.unit
                color: Qt.rgba(0, 0, 0, 0.5)
                blur: 50 * root.unit
                spread: 0
                offsetX: 0
                offsetY: 18 * root.unit
                pixelScale: root.pixelScale
            }
            Rectangle {
                objectName: "PaneCoverBox"
                anchors.fill: parent
                radius: 16 * root.unit
                color: Kit.Tokens.inputSurface
                Kit.RoundPicture {
                    id: discPicture
                    objectName: "PaneCover"
                    anchors.fill: parent
                    visible: root.paneModel.big.length > 0
                    source: root.paneModel.big
                    radius: 16 * root.unit
                    crop: true
                    pixelScale: root.pixelScale
                }
                CardLine {
                    objectName: "PaneCoverFallback"
                    visible: root.paneModel.big.length === 0 || discPicture.status === Image.Error
                    unit: root.unit
                    boardLine: true
                    x: 16 * root.unit
                    y: parent.height - height - 16 * root.unit
                    width: parent.width - 32 * root.unit
                    text: root.paneModel.title
                    color: Kit.Tokens.textStrong
                    size: 20
                    weight: Font.DemiBold
                    wrapMode: Text.WordWrap
                    elide: Text.ElideRight
                    maximumLineCount: 3
                }
            }
            Rectangle {
                objectName: "PaneCoverRing"
                x: -2 * root.unit
                y: -2 * root.unit
                width: parent.width + 4 * root.unit
                height: parent.height + 4 * root.unit
                radius: 18 * root.unit
                color: "transparent"
                border.width: 2 * root.unit
                border.color: Qt.alpha(Kit.Tokens.accent, 0.5)
            }
        }
        Column {
            id: coverTextCol
            objectName: "PaneCoverText"
            x: coverFrame.width + 32 * root.unit
            y: 0
            width: root.contentWidth - x
            spacing: 16 * root.unit
            CardLine {
                objectName: "PaneTitle"
                unit: root.unit
                boardLine: true
                boardLineHeight: 41.4
                width: parent.width
                text: root.paneModel.title
                color: Kit.Tokens.textStrong
                size: 36
                weight: Font.DemiBold
                wrapMode: Text.WordWrap
                lineHeight: 41.4
                lineHeightMode: Text.FixedHeight
            }
            CardLine {
                objectName: "PaneHint"
                unit: root.unit
                boardLine: true
                boardLineHeight: 34.8
                width: parent.width
                text: root.paneModel.sentence
                color: Kit.Tokens.textStrong
                size: 24
                weight: Font.Normal
                wrapMode: Text.WordWrap
                lineHeight: 34.8
                lineHeightMode: Text.FixedHeight
            }
        }
    }
    Column {
        id: bodyCol
        objectName: "PauseCardPane"
        visible: !root.paneModel.cover
        x: root.pad
        y: root.pad
        width: root.contentWidth
        spacing: 22 * root.unit
        Rectangle {
            objectName: "PaneMoment"
            visible: root.paneModel.moment
            width: root.contentWidth
            height: 300 * root.unit
            radius: 14 * root.unit
            color: Kit.Tokens.inputSurface
            Kit.RoundPicture {
                objectName: "PaneMomentPicture"
                anchors.fill: parent
                visible: root.paneModel.big.length > 0
                source: root.paneModel.big
                radius: 14 * root.unit
                crop: false
                pixelScale: root.pixelScale
            }
        }
        Row {
            objectName: "PaneSlots"
            visible: !root.paneModel.drawer && slotRepeater.count > 0
            spacing: root.paneModel.moment ? 8 * root.unit : 24 * root.unit
            Repeater {
                id: slotRepeater
                model: root.paneModel.drawer ? null : root.paneModel
                delegate: Column {
                    id: tile
                    required property int index
                    required property string picture
                    required property string name
                    required property string when
                    required property bool ringed
                    readonly property real tileWidth: root.paneModel.moment ? 108 * root.unit : 210 * root.unit
                    readonly property real tileHeight: root.paneModel.moment ? 81 * root.unit : 158 * root.unit
                    readonly property real tileRadius: root.paneModel.moment ? 8 * root.unit : 14 * root.unit
                    width: tile.tileWidth
                    spacing: root.paneModel.moment ? 6 * root.unit : 10 * root.unit
                    Item {
                        objectName: "PaneSlot" + (tile.index + 1)
                        width: tile.tileWidth
                        height: tile.tileHeight
                        Rectangle {
                            objectName: "PaneSlotBox" + (tile.index + 1)
                            anchors.fill: parent
                            radius: tile.tileRadius
                            color: Kit.Tokens.inputSurface
                            Kit.RoundPicture {
                                objectName: "PaneSlotPicture" + (tile.index + 1)
                                anchors.fill: parent
                                visible: tile.picture.length > 0
                                source: tile.picture
                                radius: tile.tileRadius
                                crop: true
                                pixelScale: root.pixelScale
                            }
                        }
                        Rectangle {
                            objectName: "PaneSlotBorder" + (tile.index + 1)
                            visible: !tile.ringed
                            x: -2 * root.unit
                            y: -2 * root.unit
                            width: parent.width + 4 * root.unit
                            height: parent.height + 4 * root.unit
                            radius: tile.tileRadius + 2 * root.unit
                            color: "transparent"
                            border.width: 2 * root.unit
                            border.color: Kit.Tokens.rule
                        }
                        Kit.FocusRing {
                            objectName: "PaneSlotRing" + (tile.index + 1)
                            visible: tile.ringed
                            anchors.fill: parent
                            radius: tile.tileRadius
                            gap: 3 * root.unit
                            thickness: 3 * root.unit
                            bandRadius: tile.tileRadius + 6 * root.unit
                            glowColor: Qt.rgba(240 / 255, 180 / 255, 196 / 255, 0.45)
                            glowBlur: 34 * root.unit
                            glowSpread: 8 * root.unit
                            pixelScale: root.pixelScale
                        }
                    }
                    CardLine {
                        objectName: "PaneSlotName" + (tile.index + 1)
                        unit: root.unit
                        boardLine: true
                        width: parent.width
                        text: tile.name
                        color: tile.ringed || !root.paneModel.moment ? Kit.Tokens.textStrong : Kit.Tokens.muted
                        size: root.paneModel.moment ? 16 : 20
                        weight: Font.DemiBold
                        elide: Text.ElideRight
                    }
                    CardLine {
                        objectName: "PaneSlotWhen" + (tile.index + 1)
                        visible: !root.paneModel.moment && tile.when.length > 0
                        unit: root.unit
                        boardLine: true
                        width: parent.width
                        text: tile.when
                        color: Kit.Tokens.muted
                        size: 18
                        weight: Font.Normal
                        elide: Text.ElideRight
                    }
                }
            }
        }
        Row {
            objectName: "PaneDrawerCards"
            visible: root.paneModel.drawer && drawerRepeater.count > 0
            spacing: 22 * root.unit
            Repeater {
                id: drawerRepeater
                model: root.paneModel.drawer ? root.paneModel : null
                delegate: Rectangle {
                    id: drawerCard
                    required property int index
                    required property string name
                    required property bool ringed
                    objectName: "PaneDrawerCard." + drawerCard.index
                    width: 150 * root.unit
                    height: 190 * root.unit
                    topLeftRadius: 14 * root.unit
                    topRightRadius: 14 * root.unit
                    bottomLeftRadius: 22 * root.unit
                    bottomRightRadius: 22 * root.unit
                    color: "#151826"
                    border.width: 2 * root.unit
                    border.color: drawerCard.ringed ? Kit.Tokens.accent : "#5A6076"
                    Rectangle {
                        objectName: "PaneDrawerCardSlot." + drawerCard.index
                        x: 22 * root.unit
                        y: 22 * root.unit
                        width: 106 * root.unit
                        height: 54 * root.unit
                        radius: 8 * root.unit
                        color: "#262A3A"
                    }
                    CardLine {
                        objectName: "PaneDrawerCardName." + drawerCard.index
                        unit: root.unit
                        boardLine: true
                        x: 0
                        y: parent.height - 26 * root.unit - height
                        width: parent.width
                        text: drawerCard.name
                        color: Kit.Tokens.textStrong
                        size: 20
                        weight: Font.Normal
                        horizontalAlignment: Text.AlignHCenter
                        elide: Text.ElideRight
                    }
                }
            }
        }
        Row {
            objectName: "PaneMeter"
            visible: root.paneModel.meter
            width: root.contentWidth
            height: 56 * root.unit
            spacing: 8 * root.unit
            Repeater {
                model: 5
                delegate: Rectangle {
                    id: bar
                    required property int index
                    objectName: "PaneMeterBar." + bar.index
                    y: (56 - (16 + bar.index * 8)) * root.unit
                    width: 22 * root.unit
                    height: (16 + bar.index * 8) * root.unit
                    radius: 6 * root.unit
                    color: bar.index < root.paneModel.bars ? Kit.Tokens.accent : "#262A3A"
                }
            }
        }
        CardLine {
            objectName: "PaneSentence"
            visible: text.length > 0
            unit: root.unit
            boardLine: true
            boardLineHeight: 34.8
            width: root.contentWidth
            text: root.paneModel.sentence
            color: Kit.Tokens.textStrong
            size: 24
            weight: Font.Normal
            wrapMode: Text.WordWrap
            lineHeight: 34.8
            lineHeightMode: Text.FixedHeight
        }
        CardLine {
            objectName: "PaneMuted"
            visible: text.length > 0
            unit: root.unit
            boardLine: true
            boardLineHeight: 27.55
            width: root.contentWidth
            text: root.paneModel.muted
            color: Kit.Tokens.muted
            size: 19
            weight: Font.Normal
            wrapMode: Text.WordWrap
            lineHeight: 27.55
            lineHeightMode: Text.FixedHeight
        }
    }
}
