pragma ComponentBehavior: Bound
import QtQuick
import EmuSen.Theme

// The header's trail: the places before the page, each a link the pad and the pointer take, a slash between, the page's own name last in the title's weight (EclipseGame:29-35; EmuSen_Qt.md §3.4.11).
Item {
    id: root
    // A board pixel in canvas units, Board.unit.
    required property real unit
    required property var controller
    // The trail's words, the page's own last.
    required property var words
    required property color slash
    // The words' size and the gap either side of a slash, in board pixels (EclipseGame:27, 29).
    property real size: 24
    property real gap: 14
    // A link by its place, looked up as a key is pressed, since the words are made again at each feed; the page's own name is none.
    function linkAt(i: int): Item { return i >= 0 && i < links.count - 1 ? (links.itemAt(i) as Row)?.children[0] ?? null : null }
    // The last link, which the walk reaches from below.
    function lastLink(): Item { return root.linkAt(links.count - 2) }
    signal chosen(int index)
    // Down from a link, which the screen sends where it goes.
    signal downward()
    objectName: "trail"
    width: row.width
    height: row.height
    Row {
        id: row
        spacing: root.gap * root.unit
        Repeater {
            id: links
            model: root.words
            delegate: Row {
                id: crumb
                required property string modelData
                required property int index
                readonly property bool here: crumb.index === links.count - 1
                spacing: root.gap * root.unit
                Walkable {
                    id: link
                    objectName: "trailLink." + crumb.index
                    controller: root.controller
                    enabled: !crumb.here
                    activeFocusOnTab: !crumb.here
                    width: words.width
                    height: words.height
                    Accessible.role: crumb.here ? Accessible.StaticText : Accessible.Link
                    Accessible.name: crumb.modelData
                    onActivated: root.chosen(crumb.index)
                    Keys.onDownPressed: (event) => {
                        root.downward()
                        event.accepted = true
                    }
                    Keys.onLeftPressed: (event) => {
                        root.linkAt(crumb.index - 1)?.forceActiveFocus(Qt.TabFocusReason)
                        event.accepted = true
                    }
                    Keys.onRightPressed: (event) => {
                        root.linkAt(crumb.index + 1)?.forceActiveFocus(Qt.TabFocusReason)
                        event.accepted = true
                    }
                    Keys.onUpPressed: (event) => event.accepted = true
                    Line {
                        id: words
                        objectName: "trailWords." + crumb.index
                        unit: root.unit
                        text: crumb.modelData
                        color: crumb.here ? Tokens.text : Tokens.muted
                        size: root.size
                        weight: crumb.here ? Font.DemiBold : Font.Normal
                    }
                    FocusRing {
                        anchors.fill: parent
                        visible: link.activeFocus
                    }
                }
                Line {
                    objectName: "trailSlash." + crumb.index
                    visible: !crumb.here
                    unit: root.unit
                    text: Words.crumb
                    color: root.slash
                    size: root.size
                }
            }
        }
    }
}
