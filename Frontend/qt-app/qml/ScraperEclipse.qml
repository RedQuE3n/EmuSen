pragma ComponentBehavior: Bound
import QtQuick
import EmuSen.Kit as Kit
import EmuSen.Models
import EmuSen.App

// Eclipse Scraper (#66): per-game table, 3-tile media aside, and live Videos/Overwrite toggles.
FocusScope {
    id: root
    objectName: "scraperEclipse"
    required property ToolModel tool
    required property int padFamily
    property real pixelScale: 1
    property int battery: -1
    property string clockText: ""
    signal backed()
    readonly property alias controller: scaffold.controller
    readonly property bool sheetOpen: scaffold.sheetOpen

    ToolTableEclipse {
        id: scaffold
        anchors.fill: parent
        focus: true
        tool: root.tool
        padFamily: root.padFamily
        pixelScale: root.pixelScale
        battery: root.battery
        clockText: root.clockText
        tableShown: true
        screenCode: 2
        asideMediaSlot: mediaTiles
        markBarSlot: optionPills
        onBacked: root.backed()
    }

    Component {
        id: mediaTiles
        Row {
            id: tileRow
            spacing: 14 * scaffold.unit
            Repeater {
                model: scaffold.aside.tiles ?? []
                Column {
                    id: tileCol
                    required property var modelData
                    objectName: tileCol.modelData.objectName ?? ""
                    width: 140 * scaffold.unit
                    spacing: 8 * scaffold.unit
                    Rectangle {
                        objectName: tileCol.modelData.boxObjectName ?? ""
                        width: 140 * scaffold.unit
                        height: 105 * scaffold.unit
                        radius: 12 * scaffold.unit
                        color: tileCol.modelData.present ? ThemeInk.successFill : "transparent"
                        border.width: tileCol.modelData.present ? 0 : 2 * scaffold.unit
                        border.color: ThemeInk.pillBorder
                        Kit.RoundPicture {
                            anchors.fill: parent
                            visible: tileCol.modelData.present && (tileCol.modelData.source ?? "").length > 0
                            source: tileCol.modelData.source ?? ""
                            radius: 12 * scaffold.unit
                            crop: true
                            pixelScale: root.pixelScale
                        }
                    }
                    CardLine {
                        objectName: tileCol.modelData.labelObjectName ?? ""
                        width: 140 * scaffold.unit
                        unit: scaffold.unit
                        boardLine: true
                        text: tileCol.modelData.label ?? ""
                        size: 17
                        weight: Font.Normal
                        color: Kit.Tokens.muted
                    }
                }
            }
        }
    }

    Component {
        id: optionPills
        Row {
            spacing: 12 * scaffold.unit
            Repeater {
                model: scaffold.view.toggles ?? []
                Item {
                    id: tog
                    required property var modelData
                    objectName: tog.modelData.objectName ?? ""
                    height: 38 * scaffold.unit
                    width: togWords.implicitWidth + 32 * scaffold.unit
                    Kit.FocusRing {
                        visible: tog.modelData.focused === true
                        anchors.fill: togBox
                        radius: 19 * scaffold.unit
                        gap: 3 * scaffold.unit
                        thickness: 3 * scaffold.unit
                        bandRadius: 25 * scaffold.unit
                        glowColor: ThemeInk.glowStrong
                        glowBlur: 32 * scaffold.unit
                        glowSpread: 8 * scaffold.unit
                        pixelScale: root.pixelScale
                    }
                    Rectangle {
                        id: togBox
                        anchors.fill: parent
                        radius: 19 * scaffold.unit
                        color: tog.modelData.on ? Kit.Tokens.accent : "transparent"
                        border.width: tog.modelData.on ? 0 : 2 * scaffold.unit
                        border.color: ThemeInk.pillBorder
                    }
                    CardLine {
                        id: togWords
                        objectName: tog.modelData.labelObjectName ?? ""
                        x: 16 * scaffold.unit
                        y: (parent.height - height) / 2
                        unit: scaffold.unit
                        boardLine: true
                        text: tog.modelData.label ?? ""
                        size: 18
                        weight: tog.modelData.on ? Font.DemiBold : Font.Normal
                        color: tog.modelData.on ? Kit.Tokens.overAccent : Kit.Tokens.muted
                    }
                    MouseArea {
                        anchors.fill: parent
                        onClicked: root.tool.eclipseActCode(4, tog.modelData.key ?? "")
                    }
                }
            }
        }
    }
}
