pragma ComponentBehavior: Bound
import QtQuick
import EmuSen.Models

// Eclipse Library Manager (#68): finding filters, 6-column file table, and games.db safety aside.
FocusScope {
    id: root
    objectName: "libraryEclipse"
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
        screenCode: 4
        onBacked: root.backed()
    }
}
