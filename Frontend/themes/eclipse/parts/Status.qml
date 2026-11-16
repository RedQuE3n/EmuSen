import QtQuick
import QtQuick.Shapes
import EmuSen.Theme

// The header's status at its right: the search, where a board offers it, the machine's battery where it has one, and the clock, on the glass where a board draws it (Main:40-50, EclipseLibrary:32-34, EclipseConsoles:36-39; EmuSen_Qt.md §3.4.8).
Item {
    id: root
    // A board pixel in canvas units, Board.unit.
    required property real unit
    required property var controller
    property bool searchable: true
    property bool glass: true
    property color glassColor: "transparent"
    property real tall: 56
    property real pad: 22
    property real gap: 22
    property real size: 22
    // Scene pixels to a canvas unit, so the search's ring glows at the pixels drawn (EmuSen_Kit.md §7.119).
    property real pixelScale: 1
    readonly property Item search: searchButton
    // Where Down takes the focus from the search.
    property Item below: null
    width: clock.x + clock.width + root.pad * root.unit
    height: root.tall * root.unit
    Rectangle {
        objectName: "status"
        anchors.fill: parent
        visible: root.glass
        radius: height / 2
        color: root.glassColor
    }
    Walkable {
        id: searchButton
        objectName: "search"
        controller: root.controller
        visible: root.searchable
        x: root.pad * root.unit
        y: (root.height - height) / 2
        width: 44 * root.unit
        height: width
        Accessible.name: Words.search
        onActivated: Actions.search()
        KeyNavigation.down: root.below
        FocusRing {
            anchors.fill: parent
            radius: width / 2
            pixelScale: root.pixelScale
            visible: searchButton.activeFocus
        }
        Item {
            id: icon
            objectName: "searchIcon"
            // The colour the magnifier is stroked in.
            readonly property color ink: Tokens.text
            x: (searchButton.width - width) / 2
            y: (searchButton.height - height) / 2
            width: 24 * root.unit
            height: width
            Shape {
                // The board's magnifier in its 24-unit box, a circle of 7 at (11, 11) and a stroke to (21, 21) (Main:42): constants of the design.
                readonly property real box: 24
                width: 24
                height: 24
                scale: parent.width / 24
                transformOrigin: Item.TopLeft
                preferredRendererType: Shape.CurveRenderer
                ShapePath {
                    fillColor: "transparent"
                    strokeColor: icon.ink
                    strokeWidth: 2
                    capStyle: ShapePath.RoundCap
                    PathAngleArc {
                        centerX: 11
                        centerY: 11
                        radiusX: 7
                        radiusY: 7
                        sweepAngle: 360
                    }
                    PathMove {
                        x: 16.5
                        y: 16.5
                    }
                    PathLine {
                        x: 21
                        y: 21
                    }
                }
            }
        }
    }
    Item {
        id: battery
        objectName: "battery"
        visible: Device.battery >= 0
        x: root.searchable ? searchButton.x + searchButton.width + root.gap * root.unit : root.pad * root.unit
        y: (root.height - height) / 2
        width: level.x + level.width
        height: Math.max(batteryIcon.height, level.height)
        Item {
            id: batteryIcon
            objectName: "batteryIcon"
            // The colour the battery is drawn in.
            readonly property color ink: Tokens.text
            y: (battery.height - height) / 2
            width: 30 * root.unit
            height: 18 * root.unit
            Rectangle {
                // The board's body, a rect of 24 by 16 at (1, 1) stroked 2 about its edge, so 26 by 18 with its corner 4 + 1 (Main:45).
                objectName: "batteryBody"
                readonly property color ink: batteryIcon.ink
                width: 26 * root.unit
                height: 18 * root.unit
                radius: 5 * root.unit
                color: "transparent"
                border.color: ink
                border.width: 2 * root.unit
            }
            Rectangle {
                // The charge, at most the body's inner 18 less its gaps, in whole board pixels as the board draws 15 for 82 (Main:45).
                objectName: "batteryCharge"
                x: 4 * root.unit
                y: 4 * root.unit
                width: Math.round(18 * Math.min(Device.battery, 100) / 100) * root.unit
                height: 10 * root.unit
                radius: 2 * root.unit
                color: batteryIcon.ink
            }
            Rectangle {
                // The terminal, 2.5 by 6 at (27, 6), its corner 1 (Main:45).
                objectName: "batteryNub"
                x: 27 * root.unit
                y: 6 * root.unit
                width: 2.5 * root.unit
                height: 6 * root.unit
                radius: 1 * root.unit
                color: batteryIcon.ink
            }
        }
        Line {
            id: level
            objectName: "batteryLevel"
            unit: root.unit
            x: batteryIcon.width + 8 * root.unit
            y: (battery.height - height) / 2
            text: Device.battery.toLocaleString(Qt.locale(), "f", 0) + Qt.locale().percent
            color: Tokens.text
            size: root.size
        }
    }
    Line {
        id: clock
        objectName: "clock"
        unit: root.unit
        x: battery.visible ? battery.x + battery.width + root.gap * root.unit : root.searchable ? searchButton.x + searchButton.width + root.gap * root.unit : root.pad * root.unit
        y: (root.height - height) / 2
        text: Device.shortTime
        color: Tokens.text
        size: root.size
        weight: Font.DemiBold
    }
}
