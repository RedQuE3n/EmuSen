pragma ComponentBehavior: Bound
import QtQuick
import EmuSen.Theme

// Firmware page's right-hand file status card: checkmark for found, muted ring for optional, rose ring for missing or wrong size (EclipseSettingsFirmware:68).
Rectangle {
    id: root
    required property real unit
    objectName: "asideFirmwareCard"
    width: parent ? parent.width : 0
    height: list.height
    radius: 22 * root.unit
    color: own.facesGround
    border.width: root.unit
    border.color: own.facesLine
    SettingsInk {
        id: own
    }
    Column {
        id: list
        x: 24 * root.unit
        width: root.width - 48 * root.unit
        Repeater {
            model: Settings.firmwareFiles
            delegate: Item {
                id: item
                required property int index
                required property string label
                required property string status
                required property int state
                objectName: "firmwareRow." + item.index
                width: list.width
                height: words.height + 33 * root.unit
                Item {
                    id: icon
                    objectName: "firmwareIcon." + item.index
                    x: 0
                    y: (item.height - root.unit - height) / 2
                    width: 26 * root.unit
                    height: width
                    Image {
                        objectName: "firmwareCheck." + item.index
                        visible: item.state === 0
                        anchors.fill: parent
                        source: "../assets/firmware-check.svg"
                        sourceSize: Qt.size(104, 104)
                        smooth: true
                    }
                    Rectangle {
                        objectName: "firmwareRing." + item.index
                        visible: item.state !== 0
                        anchors.centerIn: parent
                        width: 22 * root.unit
                        height: width
                        radius: width / 2
                        color: "transparent"
                        border.width: 2 * root.unit
                        border.color: item.state === 1 ? own.ringMuted : own.ringEnd
                    }
                }
                Column {
                    id: words
                    x: (26 + 14) * root.unit
                    y: 16 * root.unit
                    width: item.width - x
                    spacing: 3 * root.unit
                    Line {
                        objectName: "firmwareName." + item.index
                        unit: root.unit
                        boardLine: true
                        width: parent.width
                        text: item.label
                        color: Tokens.text
                        elide: Text.ElideRight
                        size: 21
                        weight: Tokens.weightTitle
                    }
                    Line {
                        objectName: "firmwareStatus." + item.index
                        unit: root.unit
                        boardLine: true
                        width: parent.width
                        text: item.status
                        color: Tokens.muted
                        elide: Text.ElideRight
                        size: 17
                    }
                }
                Rectangle {
                    objectName: "firmwareRule." + item.index
                    x: 0
                    y: item.height - root.unit
                    width: item.width
                    height: root.unit
                    color: own.facesLine
                }
            }
        }
    }
}
