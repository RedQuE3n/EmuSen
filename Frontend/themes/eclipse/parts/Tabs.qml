pragma ComponentBehavior: Bound
import QtQuick
import EmuSen.Theme

// A page's tabs as the game board draws them: the one shown marked and underlined, the shoulders stepping them through the frame's walk and a tab tapped choosing it (EclipseGame:55-62; EmuSen_Qt.md §3.4.12).
Row {
    id: root
    // A board pixel in canvas units, Board.unit.
    required property real unit
    required property var controller
    // The tabs' words and the place of the one shown.
    required property var tabs
    required property int current
    // The shown tab's words.
    property color currentInk: "#FFFFFF"
    // The board's words' size, the gap between tabs, the underline's gap and thickness, in board pixels (EclipseGame:55-57).
    property real size: 24
    property real gap: 40
    property real underGap: 10
    property real underline: 3
    signal chosen(int index)
    objectName: "tabs"
    spacing: root.gap * root.unit
    Accessible.role: Accessible.PageTabList
    Binding {
        target: root.controller
        property: "sections"
        value: root.tabs
    }
    Binding {
        target: root.controller
        property: "selectedSection"
        value: root.tabs[root.current] ?? ""
    }
    Connections {
        target: root.controller
        function onSectionChosen(key) { root.chosen(root.tabs.indexOf(key)) }
    }
    Repeater {
        model: root.tabs
        delegate: Item {
            id: tab
            required property string modelData
            required property int index
            readonly property bool current: tab.index === root.current
            objectName: "tab." + tab.index
            width: words.width
            height: words.height + (root.underGap + root.underline) * root.unit
            Accessible.role: Accessible.PageTab
            Accessible.name: tab.modelData
            Accessible.selected: tab.current
            Line {
                id: words
                objectName: "tabWords." + tab.index
                unit: root.unit
                text: tab.modelData
                boardLine: true
                color: tab.current ? root.currentInk : Tokens.muted
                size: root.size
                weight: tab.current ? Font.DemiBold : Font.Normal
            }
            Rectangle {
                objectName: "tabUnderline." + tab.index
                y: tab.height - height
                width: tab.width
                height: root.underline * root.unit
                color: Tokens.accent
                visible: tab.current
            }
            TapHandler {
                onTapped: root.chosen(tab.index)
            }
        }
    }
}
