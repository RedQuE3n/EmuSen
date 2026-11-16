import QtQuick
import QtQuick.Shapes
import EmuSen.Theme

// The header's status at its right: the search, where a board offers it, and the clock, on the glass where a board draws it (Main:40-50, EclipseLibrary:32-34, EclipseConsoles:36-39; EmuSen_Qt.md §3.4.8).
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
    Line {
        id: clock
        name: "clock"
        unit: root.unit
        x: root.searchable ? searchButton.x + searchButton.width + root.gap * root.unit : root.pad * root.unit
        y: (root.height - height) / 2
        text: Device.clock
        color: Tokens.text
        size: root.size
        weight: Font.DemiBold
    }
}
