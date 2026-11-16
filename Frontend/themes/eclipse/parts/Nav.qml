pragma ComponentBehavior: Bound
import QtQuick
import EmuSen.Theme

// The header's nav: the program's sections (Library.sections), the one shown marked, the shoulders stepping them through the frame's walk and a section tapped opening it (Main:33-39, EclipseConsoles:28-34; EmuSen_Qt.md §3.4.8).
Row {
    id: root
    // A board pixel in canvas units, Board.unit.
    required property real unit
    // The frame's controller, whose walk the shoulders step the sections in.
    required property var controller
    // The boards' nav, in board pixels: its words' size, the gap between sections, the underline's gap and thickness (Main:33-34).
    property real size: 24
    property real gap: 44
    property real underGap: 10
    property real underline: 3
    // The sections' keys, in the nav's order, as the walk steps them.
    readonly property var keys: {
        const out = [];
        for (let i = 0; i < sections.count; i++)
            out.push((sections.itemAt(i) as Entry)?.key ?? "");
        return out;
    }
    objectName: "nav"
    spacing: root.gap * root.unit
    Accessible.role: Accessible.PageTabList
    Ink {
        id: ink
    }
    Binding {
        target: root.controller
        property: "sections"
        value: root.keys
    }
    Binding {
        target: root.controller
        property: "selectedSection"
        value: Library.section
    }
    Connections {
        target: root.controller
        // The host opens another section's screen and declines the one shown (qt-models/src/sections.rs).
        function onSectionChosen(key) { Actions.openSection(key) }
    }
    component Entry: Item {
        id: entry
        required property string key
        required property string name
        readonly property bool current: entry.key === Library.section
        objectName: "nav." + entry.key
        width: words.width
        height: words.height + (root.underGap + root.underline) * root.unit
        Accessible.role: Accessible.PageTab
        Accessible.name: entry.name
        Accessible.selected: entry.current
        Line {
            id: words
            objectName: "navWords." + entry.key
            unit: root.unit
            text: entry.name
            color: entry.current ? ink.current : Tokens.muted
            size: root.size
            weight: entry.current ? Font.DemiBold : Font.Normal
        }
        Rectangle {
            objectName: "navUnderline." + entry.key
            y: entry.height - height
            width: entry.width
            height: root.underline * root.unit
            color: Tokens.accent
            visible: entry.current
        }
        TapHandler {
            onTapped: Actions.openSection(entry.key)
        }
    }
    Repeater {
        id: sections
        model: Library.sections
        delegate: Entry {}
    }
}
