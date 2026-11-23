pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Shapes
import EmuSen.Theme

// Settings' sections down the left, the one shown filled in the accent, a click opening another (G:101-108; EmuSen_Qt.md §3.4.11).
Column {
    id: root
    // A board pixel in canvas units, Board.unit.
    required property real unit
    // The board's links, in board pixels: 300 by 66, 6 apart, 20 in, the icon's 28 and its 16 px gap before the words (G:101-104).
    readonly property real wide: 300
    readonly property real tall: 66
    readonly property real inset: 20
    readonly property real icon: 28
    readonly property real gap: 16
    objectName: "settingsNav"
    spacing: 6 * root.unit
    Accessible.role: Accessible.PageTabList
    Ink {
        id: ink
    }
    SettingsInk {
        id: own
    }
    Repeater {
        model: Settings.sections
        delegate: Rectangle {
            id: entry
            required property int index
            required property string modelData
            readonly property bool current: entry.index === Settings.section
            objectName: "section." + entry.index
            width: root.wide * root.unit
            height: root.tall * root.unit
            radius: 16 * root.unit
            color: entry.current ? Tokens.accent : "transparent"
            Accessible.role: Accessible.PageTab
            Accessible.name: entry.modelData
            Accessible.selected: entry.current
            // The section's icon, 28 px, stroked 1.7 in the words' colour with round ends (G:104).
            Item {
                objectName: "sectionIcon." + entry.index
                x: root.inset * root.unit
                y: (entry.height - height) / 2
                width: root.icon * root.unit
                height: width
                // What is drawn, which the tests read: the stroke and the path.
                readonly property color paint: stroke.strokeColor
                readonly property string path: stroke.shape
                Shape {
                    width: 24
                    height: 24
                    scale: parent.width / 24
                    transformOrigin: Item.TopLeft
                    preferredRendererType: Shape.CurveRenderer
                    ShapePath {
                        id: stroke
                        readonly property string shape: own.sectionIcons[Settings.sectionKeys[entry.index] ?? ""] ?? ""
                        fillColor: "transparent"
                        strokeColor: words.color
                        strokeWidth: 1.7
                        capStyle: ShapePath.RoundCap
                        joinStyle: ShapePath.RoundJoin
                        PathSvg { path: stroke.shape }
                    }
                }
            }
            Line {
                id: words
                objectName: "sectionWords." + entry.index
                unit: root.unit
                x: (root.inset + root.icon + root.gap) * root.unit
                y: (entry.height - height) / 2
                text: entry.modelData
                color: entry.current ? Tokens.overAccent : ink.help
                size: 23
                weight: entry.current ? Tokens.weightTitle : Font.Normal
            }
            TapHandler {
                onTapped: Actions.settingsSection(entry.index)
            }
        }
    }
}
