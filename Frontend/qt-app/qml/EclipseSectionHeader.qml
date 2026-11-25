pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Shapes
import EmuSen.Kit as Kit
import EmuSen.Models

// Top Eclipse header bar: EmuSen crescent brand, section nav links with active underline, and battery/clock.
Item {
    id: root
    required property real unit
    required property var navSections
    property int battery: -1
    property string clockText: ""
    signal sectionSelected(string key)

    x: 96 * root.unit
    y: 48 * root.unit
    width: parent ? parent.width - 192 * root.unit : 0
    height: 64 * root.unit

    Row {
        anchors.left: parent.left
        height: root.height
        spacing: 64 * root.unit

        Row {
            height: root.height
            spacing: 14 * root.unit

            Item {
                objectName: "mark"
                width: 34 * root.unit
                height: 34 * root.unit
                y: (parent.height - height) / 2
                Shape {
                    width: 34
                    height: 34
                    scale: root.unit
                    transformOrigin: Item.TopLeft
                    preferredRendererType: Shape.CurveRenderer
                    ShapePath {
                        fillColor: "#F6E7D8"
                        strokeColor: "transparent"
                        fillRule: ShapePath.OddEvenFill
                        PathSvg { path: Marks.menuCrescent }
                    }
                }
            }

            CardLine {
                objectName: "wordmark"
                y: (parent.height - height) / 2
                unit: root.unit
                boardLine: true
                text: Words.name
                size: 24
                weight: Font.DemiBold
                spacing: 24 * 0.02
                color: "#F4F1EC"
            }
        }

        Row {
            objectName: "nav"
            y: (parent.height - height) / 2
            spacing: 44 * root.unit

            Repeater {
                model: root.navSections
                Item {
                    id: tab
                    required property var modelData
                    objectName: tab.modelData.itemName ?? ""
                    width: navText.width
                    height: navText.height + 13 * root.unit
                    y: (parent.height - height) / 2

                    CardLine {
                        id: navText
                        objectName: tab.modelData.wordsName ?? ""
                        unit: root.unit
                        boardLine: true
                        text: tab.modelData.name ?? ""
                        size: 24
                        weight: tab.modelData.active === true ? Font.DemiBold : Font.Normal
                        color: tab.modelData.active === true ? "#FFFFFF" : "#B9BED0"
                    }

                    Rectangle {
                        objectName: tab.modelData.underlineName ?? ""
                        visible: tab.modelData.active === true
                        y: navText.height + 10 * root.unit
                        width: navText.width
                        height: 3 * root.unit
                        color: "#F6E7D8"
                    }

                    TapHandler {
                        onTapped: root.sectionSelected(tab.modelData.key ?? "")
                    }
                }
            }
        }
    }

    Row {
        objectName: "headerStatus"
        anchors.right: parent.right
        height: root.height
        spacing: 22 * root.unit

        CardLine {
            objectName: "battery"
            visible: root.battery >= 0
            y: (parent.height - height) / 2
            unit: root.unit
            boardLine: true
            text: root.battery >= 0 ? Words.percent(root.battery) : ""
            size: 22
            weight: Font.Normal
            color: "#F4F1EC"
        }

        CardLine {
            objectName: "clock"
            visible: root.clockText.length > 0
            y: (parent.height - height) / 2
            unit: root.unit
            boardLine: true
            text: root.clockText
            size: 22
            weight: Font.DemiBold
            color: "#F4F1EC"
        }
    }
}
