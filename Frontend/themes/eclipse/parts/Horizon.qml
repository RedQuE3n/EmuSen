import QtQuick

// The page's lower ground and the horizon over it, its line clear at both ends (EclipseGame:23-24): every measure in board pixels.
Item {
    id: root
    // A board pixel in canvas units, Board.unit.
    required property real unit
    required property color ground
    required property color line
    required property color clear
    // The ground's top and the line's top and height, in board pixels (EclipseGame:23-24).
    property real groundTop: 620
    property real lineTop: 619
    property real lineHeight: 2
    Rectangle {
        objectName: "gameGround"
        y: root.groundTop * root.unit
        width: root.width
        height: root.height - y
        color: root.ground
    }
    Rectangle {
        objectName: "gameHorizon"
        y: root.lineTop * root.unit
        width: root.width
        height: root.lineHeight * root.unit
        gradient: Gradient {
            orientation: Gradient.Horizontal
            GradientStop { position: 0; color: root.clear }
            GradientStop { position: 0.3; color: root.line }
            GradientStop { position: 0.7; color: root.line }
            GradientStop { position: 1; color: root.clear }
        }
    }
}
