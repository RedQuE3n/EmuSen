pragma ComponentBehavior: Bound
import QtQuick
import EmuSen.Theme
import "../parts"

// Horizon's Games, Rise: the focused game's hero risen as a world, its title, facts and description in the sky, the section's covers on the land under the clamped camera; while searching the covers rise to a line under the host's field (EmuSen_Qt.md §3.4.3).
ThemeScreen {
    id: screen
    objectName: "horizonGames"
    role: ThemeScreen.Games
    NormalizedCanvas {
        id: canvas
        objectName: "gamesCanvas"
        FocusScope {
            id: root
            objectName: "games"
            anchors.fill: parent
            focus: true
            readonly property real pixelScale: canvas.unit
            readonly property var focused: covers.currentItem ? covers.currentItem.cell : null
            readonly property bool empty: covers.count === 0 && !Browse.searching
            readonly property real hue: root.focused ? root.focused.hue : Browse.hue
            readonly property real wordsWidth: Math.min(820, root.width - 480)
            // While searching the matches stand 140 tall on a line at 380, under the host's field; else 192 on the foot line.
            readonly property real coverHeight: Browse.searching ? 140 : 192
            Backdrop {
                anchors.fill: parent
                hue: root.empty || Browse.searching ? -1 : root.hue
            }
            Frame {
                id: frame
                anchors.fill: parent
                surfaceDrawn: false
                padFamily: Device.family
                helpFont: Qt.font({ family: faces.regular, pixelSize: 20 })
                helpSpacing: 40
                helpGlyphGap: 10
                // While the search's keyboard is typed on, its help is EmuSen's, drawn with it.
                helpShown: !Browse.typing
                focus: true
                controller.navigationShown: false
                help: Help.entries(Browse.searched ? Help.SectionSearched : Help.Section)
                Connections {
                    target: frame.controller
                    function onBackDeclined() { Actions.back() }
                    function onActionInvoked(key) { Actions.act(key) }
                }
                Item {
                    x: -frame.safeX
                    y: -frame.safeY
                    width: root.width
                    height: root.height
                    Row {
                        id: head
                        objectName: "gamesHead"
                        x: 64
                        y: 40.8
                        height: 38.4
                        spacing: 16
                        // The heading's name elides so the row ends 24 before the neighbours.
                        readonly property real room: neighbours.x - 24 - 64 - path.width - planet.width - count.width - 3 * 16
                        Text {
                            id: path
                            objectName: "gamesPath"
                            anchors.verticalCenter: parent.verticalCenter
                            text: Words.gamesPath
                            color: Tokens.muted
                            font.family: faces.regular
                            font.pixelSize: 20
                            font.letterSpacing: 2
                        }
                        Body {
                            id: planet
                            objectName: "gamesPlanet"
                            anchors.verticalCenter: parent.verticalCenter
                            width: Browse.world.length > 0 ? 34 : 0
                            height: 34
                            world: Browse.world
                            visible: Browse.world.length > 0
                        }
                        Text {
                            objectName: "gamesTitle"
                            anchors.verticalCenter: parent.verticalCenter
                            width: Math.min(implicitWidth, Math.max(0, head.room))
                            text: Browse.heading
                            elide: Text.ElideRight
                            color: Tokens.textStrong
                            font.family: faces.condensed
                            font.weight: Font.Medium
                            font.pixelSize: 32
                            font.letterSpacing: 3
                            font.capitalization: Font.AllUppercase
                            lineHeight: 38.4
                            lineHeightMode: Text.FixedHeight
                        }
                        Text {
                            id: count
                            objectName: "gamesCount"
                            anchors.verticalCenter: parent.verticalCenter
                            text: Browse.countLine
                            color: Tokens.muted
                            font.family: faces.regular
                            font.pixelSize: 20
                        }
                    }
                    Row {
                        id: neighbours
                        objectName: "gamesNeighbours"
                        x: root.width - 64 - width
                        y: 34
                        height: 52
                        spacing: 12
                        PadGlyph {
                            anchors.verticalCenter: parent.verticalCenter
                            visible: Browse.previous.length > 0
                            width: 30
                            height: 30
                            family: Device.family
                            button: PadGlyph.PageUp
                        }
                        Text {
                            objectName: "gamesPrevious"
                            anchors.verticalCenter: parent.verticalCenter
                            width: Math.min(160, implicitWidth)
                            text: Browse.previous
                            elide: Text.ElideRight
                            color: Tokens.muted
                            font.family: faces.regular
                            font.pixelSize: 19
                        }
                        Text {
                            anchors.verticalCenter: parent.verticalCenter
                            visible: Browse.previous.length > 0 && Browse.next.length > 0
                            text: Words.between
                            color: Tokens.muted
                            font.family: faces.regular
                            font.pixelSize: 19
                        }
                        Text {
                            objectName: "gamesNext"
                            anchors.verticalCenter: parent.verticalCenter
                            width: Math.min(160, implicitWidth)
                            text: Browse.next
                            elide: Text.ElideRight
                            color: Tokens.muted
                            font.family: faces.regular
                            font.pixelSize: 19
                        }
                        PadGlyph {
                            anchors.verticalCenter: parent.verticalCenter
                            visible: Browse.next.length > 0
                            width: 30
                            height: 30
                            family: Device.family
                            button: PadGlyph.PageDown
                        }
                    }
                    World {
                        visible: !root.empty && !Browse.searching
                        x: root.width - 364
                        y: 140
                        key: root.focused ? root.focused.key : ""
                        picture: root.focused ? root.focused.picture : ""
                        hue: root.hue
                        pixelScale: root.pixelScale
                    }
                    Column {
                        objectName: "gamesWords"
                        visible: !Browse.searching
                        x: 64
                        y: 408 - height
                        width: root.wordsWidth
                        spacing: 8
                        Text {
                            objectName: "gamesFocusTitle"
                            width: parent.width
                            text: root.empty ? Words.noGames : (root.focused ? root.focused.title : "")
                            wrapMode: Text.Wrap
                            maximumLineCount: 2
                            elide: Text.ElideRight
                            color: Tokens.textStrong
                            font.family: faces.condensed
                            font.weight: Font.Medium
                            font.pixelSize: 44
                            lineHeight: 52.8
                            lineHeightMode: Text.FixedHeight
                        }
                        Row {
                            objectName: "gamesFacts"
                            visible: !root.empty
                            spacing: 10
                            Body {
                                anchors.verticalCenter: parent.verticalCenter
                                width: 14
                                height: 14
                                visible: root.hue >= 0
                                hue: root.hue
                            }
                            Text {
                                width: root.wordsWidth - 24
                                text: root.focused ? root.focused.facts : ""
                                elide: Text.ElideRight
                                color: Tokens.muted
                                font.family: faces.regular
                                font.pixelSize: 22
                                lineHeight: 26.4
                                lineHeightMode: Text.FixedHeight
                            }
                        }
                        Text {
                            objectName: "gamesDescription"
                            visible: root.empty || text.length > 0
                            width: parent.width
                            text: root.empty ? Words.addFolder : (root.focused ? root.focused.description : "")
                            wrapMode: Text.Wrap
                            maximumLineCount: 3
                            elide: Text.ElideRight
                            color: root.empty ? Tokens.muted : Tokens.text
                            font.family: faces.regular
                            font.pixelSize: 22
                            lineHeight: 26.4
                            lineHeightMode: Text.FixedHeight
                        }
                    }
                    HorizonLine {
                        visible: !root.empty && !Browse.searching
                        width: root.width
                        total: covers.contentWidth
                        off: covers.contentX + 64
                    }
                    Item {
                        id: land
                        objectName: "gamesLand"
                        visible: !root.empty
                        width: root.width
                        y: Browse.searching ? 120 : 440
                        height: 302
                        Behavior on y {
                            NumberAnimation {
                                duration: Tokens.pan
                                easing.type: Easing.OutCubic
                            }
                        }
                        EdgeFade {
                            anchors.fill: parent
                            fadeLeft: covers.fadeLeft
                            fadeRight: covers.fadeRight
                            length: 40
                            ListView {
                                id: covers
                                objectName: "strip"
                                // The fit audit's edge rule reads a faded edge from these.
                                property bool fadeLeft: true
                                property bool fadeRight: true
                                readonly property real focusWidth: covers.currentItem ? covers.currentItem.width : 0
                                anchors.fill: parent
                                orientation: ListView.Horizontal
                                spacing: 28
                                leftMargin: 64
                                rightMargin: 64
                                focus: true
                                boundsBehavior: Flickable.StopAtBounds
                                // The clamped camera: the focus walks to the centre, then the row moves under it, stopping at either end.
                                highlightRangeMode: ListView.ApplyRange
                                preferredHighlightBegin: (root.width - covers.focusWidth) / 2
                                preferredHighlightEnd: (root.width + covers.focusWidth) / 2
                                highlightMoveDuration: Tokens.reel
                                highlightMoveVelocity: -1
                                highlight: Item {}
                                keyNavigationWraps: false
                                Keys.onUpPressed: (event) => event.accepted = true
                                Keys.onDownPressed: (event) => event.accepted = true
                                model: Browse.games
                                // A horizontal list sets each delegate's y to 0, so the cover stands on the foot line inside a slot as tall as the land.
                                delegate: FocusScope {
                                    id: slot
                                    required property int index
                                    required property string key
                                    required property string title
                                    required property string facts
                                    required property string description
                                    required property string cover
                                    required property string hero
                                    required property real coverShape
                                    required property real consoleHue
                                    required property bool favourite
                                    readonly property alias cell: cell
                                    width: cell.width
                                    height: 302
                                    StripItem {
                                        id: cell
                                        readonly property string description: slot.description
                                        controller: frame.controller
                                        objectName: "gameCover." + slot.index
                                        focus: true
                                        key: slot.key
                                        title: slot.title
                                        facts: slot.facts
                                        steps: false
                                        still: Browse.searching
                                        kind: 1
                                        picture: slot.hero
                                        hue: slot.consoleHue
                                        y: 260 - height
                                        width: root.coverHeight * slot.coverShape
                                        height: root.coverHeight
                                        Accessible.name: slot.title
                                        onActivated: Actions.play(slot.key)
                                        onActiveFocusChanged: if (cell.activeFocus) Actions.focus(slot.key)
                                        Cover {
                                            anchors.fill: parent
                                            source: slot.cover
                                            title: slot.title
                                            hue: slot.consoleHue
                                            favourite: slot.favourite
                                            pixelScale: root.pixelScale
                                        }
                                    }
                                }
                                WheelHandler {
                                    onWheel: (event) => covers.currentIndex = Math.max(0, Math.min(covers.count - 1, covers.currentIndex + (event.angleDelta.y + event.angleDelta.x < 0 ? 1 : -1)))
                                }
                            }
                        }
                    }
                    // The focus as the program holds it: the theme reports each move (Actions.focus) and the List card moves it too.
                    Binding {
                        covers.currentIndex: Browse.focusRow
                    }
                }
            }
            Faces {
                id: faces
            }
        }
    }
}
