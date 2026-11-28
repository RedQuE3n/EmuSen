import QtQuick
import QtQuick.Shapes

// Crisp geometric vector icon in a 24-unit coordinate box without inline path strings.
Item {
    id: root
    // 0: sidebar, 1: inspector, 2: search, 3: chevronLeft, 4: chevronRight, 5: close, 6: play, 7: pause, 8: check, 9: folder, 10: grid, 11: table, 12: arrowUp, 13: arrowDown, 14: sliders
    required property int kind
    required property color color
    property real strokeWidth: 1.8

    Shape {
        anchors.fill: parent
        scale: Math.min(parent.width, parent.height) / 24
        transformOrigin: Item.TopLeft
        width: 24
        height: 24
        preferredRendererType: Shape.CurveRenderer

        // Sidebar: outer frame + vertical split at x=9.
        ShapePath {
            strokeColor: root.kind === 0 ? root.color : "transparent"
            fillColor: "transparent"
            strokeWidth: root.strokeWidth
            capStyle: ShapePath.RoundCap
            joinStyle: ShapePath.RoundJoin
            startX: 4; startY: 5
            PathLine { x: 20; y: 5 }
            PathLine { x: 20; y: 19 }
            PathLine { x: 4; y: 19 }
            PathLine { x: 4; y: 5 }
            PathMove { x: 9; y: 5 }
            PathLine { x: 9; y: 19 }
        }

        // Inspector: outer frame + vertical split at x=15.
        ShapePath {
            strokeColor: root.kind === 1 ? root.color : "transparent"
            fillColor: "transparent"
            strokeWidth: root.strokeWidth
            capStyle: ShapePath.RoundCap
            joinStyle: ShapePath.RoundJoin
            startX: 4; startY: 5
            PathLine { x: 20; y: 5 }
            PathLine { x: 20; y: 19 }
            PathLine { x: 4; y: 19 }
            PathLine { x: 4; y: 5 }
            PathMove { x: 15; y: 5 }
            PathLine { x: 15; y: 19 }
        }

        // Search: circle + handle.
        ShapePath {
            strokeColor: root.kind === 2 ? root.color : "transparent"
            fillColor: "transparent"
            strokeWidth: root.strokeWidth
            capStyle: ShapePath.RoundCap
            PathAngleArc { centerX: 10.5; centerY: 10.5; radiusX: 6.5; radiusY: 6.5; startAngle: 0; sweepAngle: 360 }
            PathMove { x: 15.5; y: 15.5 }
            PathLine { x: 20.5; y: 20.5 }
        }

        // Chevron Left (3) and Chevron Right (4).
        ShapePath {
            strokeColor: root.kind === 3 || root.kind === 4 ? root.color : "transparent"
            fillColor: "transparent"
            strokeWidth: root.strokeWidth
            capStyle: ShapePath.RoundCap
            joinStyle: ShapePath.RoundJoin
            startX: root.kind === 3 ? 15 : 9; startY: 6
            PathLine { x: root.kind === 3 ? 9 : 15; y: 12 }
            PathLine { x: root.kind === 3 ? 15 : 9; y: 18 }
        }

        // Close X (5).
        ShapePath {
            strokeColor: root.kind === 5 ? root.color : "transparent"
            fillColor: "transparent"
            strokeWidth: root.strokeWidth
            capStyle: ShapePath.RoundCap
            startX: 6; startY: 6
            PathLine { x: 18; y: 18 }
            PathMove { x: 18; y: 6 }
            PathLine { x: 6; y: 18 }
        }

        // Play triangle (6).
        ShapePath {
            strokeColor: "transparent"
            fillColor: root.kind === 6 ? root.color : "transparent"
            startX: 7; startY: 5
            PathLine { x: 19; y: 12 }
            PathLine { x: 7; y: 19 }
            PathLine { x: 7; y: 5 }
        }

        // Pause bars (7).
        ShapePath {
            strokeColor: "transparent"
            fillColor: root.kind === 7 ? root.color : "transparent"
            startX: 6; startY: 5
            PathLine { x: 10; y: 5 }
            PathLine { x: 10; y: 19 }
            PathLine { x: 6; y: 19 }
            PathLine { x: 6; y: 5 }
            PathMove { x: 14; y: 5 }
            PathLine { x: 18; y: 5 }
            PathLine { x: 18; y: 19 }
            PathLine { x: 14; y: 19 }
            PathLine { x: 14; y: 5 }
        }

        // Checkmark (8).
        ShapePath {
            strokeColor: root.kind === 8 ? root.color : "transparent"
            fillColor: "transparent"
            strokeWidth: 2.2
            capStyle: ShapePath.RoundCap
            joinStyle: ShapePath.RoundJoin
            startX: 5; startY: 12
            PathLine { x: 10; y: 17 }
            PathLine { x: 19; y: 7 }
        }

        // Folder (9).
        ShapePath {
            strokeColor: root.kind === 9 ? root.color : "transparent"
            fillColor: "transparent"
            strokeWidth: root.strokeWidth
            capStyle: ShapePath.RoundCap
            joinStyle: ShapePath.RoundJoin
            startX: 3; startY: 6
            PathLine { x: 9; y: 6 }
            PathLine { x: 11; y: 8.5 }
            PathLine { x: 21; y: 8.5 }
            PathLine { x: 21; y: 18 }
            PathLine { x: 3; y: 18 }
            PathLine { x: 3; y: 6 }
        }

        // Grid 2x2 (10).
        ShapePath {
            strokeColor: root.kind === 10 ? root.color : "transparent"
            fillColor: "transparent"
            strokeWidth: root.strokeWidth
            joinStyle: ShapePath.RoundJoin
            startX: 4; startY: 4
            PathLine { x: 10; y: 4 }
            PathLine { x: 10; y: 10 }
            PathLine { x: 4; y: 10 }
            PathLine { x: 4; y: 4 }
            PathMove { x: 14; y: 4 }
            PathLine { x: 20; y: 4 }
            PathLine { x: 20; y: 10 }
            PathLine { x: 14; y: 10 }
            PathLine { x: 14; y: 4 }
            PathMove { x: 4; y: 14 }
            PathLine { x: 10; y: 14 }
            PathLine { x: 10; y: 20 }
            PathLine { x: 4; y: 20 }
            PathLine { x: 4; y: 14 }
            PathMove { x: 14; y: 14 }
            PathLine { x: 20; y: 14 }
            PathLine { x: 20; y: 20 }
            PathLine { x: 14; y: 20 }
            PathLine { x: 14; y: 14 }
        }

        // Table rows (11).
        ShapePath {
            strokeColor: root.kind === 11 ? root.color : "transparent"
            fillColor: "transparent"
            strokeWidth: root.strokeWidth
            capStyle: ShapePath.RoundCap
            startX: 4; startY: 6
            PathLine { x: 20; y: 6 }
            PathMove { x: 4; y: 12 }
            PathLine { x: 20; y: 12 }
            PathMove { x: 4; y: 18 }
            PathLine { x: 20; y: 18 }
        }

        // Sort arrow Up (12) / Down (13).
        ShapePath {
            strokeColor: root.kind === 12 || root.kind === 13 ? root.color : "transparent"
            fillColor: "transparent"
            strokeWidth: root.strokeWidth
            capStyle: ShapePath.RoundCap
            joinStyle: ShapePath.RoundJoin
            startX: 12; startY: 5
            PathLine { x: 12; y: 19 }
            PathMove { x: 7; y: root.kind === 12 ? 10 : 14 }
            PathLine { x: 12; y: root.kind === 12 ? 5 : 19 }
            PathLine { x: 17; y: root.kind === 12 ? 10 : 14 }
        }

        // Sliders (14).
        ShapePath {
            strokeColor: root.kind === 14 ? root.color : "transparent"
            fillColor: "transparent"
            strokeWidth: root.strokeWidth
            capStyle: ShapePath.RoundCap
            startX: 4; startY: 7
            PathLine { x: 20; y: 7 }
            PathMove { x: 4; y: 12 }
            PathLine { x: 20; y: 12 }
            PathMove { x: 4; y: 17 }
            PathLine { x: 20; y: 17 }
            PathMove { x: 9; y: 4.5 }
            PathLine { x: 9; y: 9.5 }
            PathMove { x: 15; y: 9.5 }
            PathLine { x: 15; y: 14.5 }
            PathMove { x: 10; y: 14.5 }
            PathLine { x: 10; y: 19.5 }
        }
    }
}
