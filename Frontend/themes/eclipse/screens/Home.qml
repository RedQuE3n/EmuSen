pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Shapes
import EmuSen.Theme
import "../parts"

// Eclipse's Home as its board draws it (Main.dc.html): the disc, the header, the focused game's name and buttons, the Recent row and the help line, every measure the board's in canvas units (EmuSen_Qt.md §3.4.8).
ThemeScreen {
    id: screen
    objectName: "homeScreen"
    role: ThemeScreen.Home
    // Where the focus starts, and where Start on Home takes it back: the first game of Recent, else the search.
    focusItems: [
        root.firstTile ?? status.search
    ]
    NormalizedCanvas {
        id: canvas
        objectName: "homeCanvas"
        FocusScope {
            id: root
            objectName: "home"
            anchors.fill: parent
            focus: true
            // The board's three buttons, the entries that act on the game (Main:61-71): a constant of the design.
            readonly property int gameButtons: 3
            // The tile whose game the top shows: the focused one, else the last focused.
            property int current: 0
            property Item firstTile: null
            // The first button, read again whenever its Repeater's count moves.
            readonly property Item firstButton: buttons.count > 0 ? buttons.itemAt(0) : null
            readonly property Item currentTile: tiles.count > 0 ? tiles.itemAt(Math.min(root.current, tiles.count - 1)) : null
            readonly property bool resuming: Session.hasGame && root.currentTile !== null && root.currentTile.key === Session.key
            readonly property int helpKind: root.resuming ? Help.Continue : Help.Game
            Board {
                id: design
                designHeight: canvas.designHeight
            }
            Ink {
                id: ink
            }
            // The ground, and the shown game's art blurred over it: the board's colour fields stand for that art (Main:19-23, 75); its blur
            // and opacity are the board's first field's, 90 px at 0.85 (Main:19), provisional until the tester judges them. A game
            // with no art leaves the ground alone.
            Backdrop {
                objectName: "backdrop"
                anchors.fill: parent
                ground: ink.ground
                pixelScale: canvas.unit
                source: root.currentTile?.hero ?? ""
                sourceBlur: 90 * design.unit
                sourceOpacity: 0.85
            }
            CoronaDisc {
                objectName: "disc"
                x: -900 * design.unit
                y: -450 * design.unit
                width: 2100 * design.unit
                height: width
            }
            Frame {
                id: frame
                anchors.fill: parent
                surfaceDrawn: false
                helpShown: false
                // The help's entries are the frame's actions, which the pad's X, Y, Select and Start invoke; the help line draws them.
                help: Help.entries(root.helpKind)
                padFamily: Device.family
                focus: true
                controller.navigationShown: false
                Connections {
                    target: frame.controller
                    function onActionInvoked(key) { Actions.act(key) }
                }
                Item {
                    x: -frame.safeX
                    y: -frame.safeY
                    width: root.width
                    height: root.height
                    Brand {
                        unit: design.unit
                        x: 96 * design.unit
                        y: 80 * design.unit - height / 2
                    }
                    Status {
                        id: status
                        unit: design.unit
                        controller: frame.controller
                        glassColor: ink.glass
                        x: parent.width - 96 * design.unit - width
                        y: 80 * design.unit - height / 2
                        below: root.firstButton ?? root.currentTile
                    }
                    // With no game on the row the search is where the focus is, so the pad reaches it.
                    Binding {
                        target: status.search
                        property: "focus"
                        value: tiles.count === 0
                    }
                    Text {
                        id: eyebrow
                        objectName: "eyebrow"
                        x: 96 * design.unit
                        y: 250 * design.unit
                        width: 980 * design.unit
                        // "Continue · Super Nintendo" in capitals, as the board draws it (Main:53).
                        text: root.resuming ? Words.continueLine(Session.console) : ""
                        font.capitalization: Font.AllUppercase
                        color: ink.rose
                        elide: Text.ElideRight
                        font.pixelSize: Type.sizeIn(20 * design.unit, Tokens.textScale, Tokens.faceText, Font.DemiBold)
                        font.family: Tokens.faceText
                        font.weight: Font.DemiBold
                        font.letterSpacing: 0.22 * 20 * design.unit
                    }
                    // The title's line box, the board's line height of 1.02 (Main:55): the next line follows it, as the board's column does.
                    Item {
                        id: titleLine
                        objectName: "titleLine"
                        x: 96 * design.unit
                        y: eyebrow.y + eyebrow.height + 26 * design.unit
                        width: 980 * design.unit
                        height: title.lineCount * title.lineHeight
                    }
                    Text {
                        id: title
                        objectName: "title"
                        x: 96 * design.unit
                        // The face's own height about the line box's middle, as CSS spreads a line's leading equally above and below.
                        anchors.verticalCenter: titleLine.verticalCenter
                        width: 980 * design.unit
                        text: root.resuming ? Session.title : root.currentTile?.title ?? (Library.consoles.count === 0 ? Words.addFolder : Words.noRecent)
                        color: Tokens.textStrong
                        // A game's name keeps the board's one line; the program's own words in its place (Words.addFolder, Words.noRecent) take a second.
                        wrapMode: root.currentTile === null ? Text.WordWrap : Text.NoWrap
                        maximumLineCount: 2
                        elide: Text.ElideRight
                        lineHeightMode: Text.FixedHeight
                        lineHeight: 1.02 * 112 * design.unit
                        font.pixelSize: Type.sizeIn(112 * design.unit, Tokens.textScale, Tokens.faceText, Tokens.weightTitle)
                        font.family: Tokens.faceText
                        font.weight: Tokens.weightTitle
                        font.letterSpacing: -0.02 * 112 * design.unit
                    }
                    Text {
                        id: facts
                        objectName: "facts"
                        x: 96 * design.unit
                        y: titleLine.y + titleLine.height + 26 * design.unit
                        width: 980 * design.unit
                        // The board's line (Main:56): when it was last played and the time played; it follows the clock, which ticks each minute.
                        text: root.resuming ? (Device.clock.length >= 0 ? Words.playedFacts(Session.pausedSince, Session.played) : "") : root.currentTile?.facts ?? ""
                        color: Tokens.muted
                        elide: Text.ElideRight
                        font.pixelSize: Type.sizeIn(26 * design.unit, Tokens.textScale, Tokens.faceText, Tokens.weightSentence)
                        font.family: Tokens.faceText
                        font.weight: Tokens.weightSentence
                    }
                    // The resume point (Main:57-60): the suspended game's paused picture and when it was saved, while that game is shown.
                    Item {
                        id: resume
                        objectName: "resume"
                        x: 96 * design.unit
                        y: facts.y + facts.height + 26 * design.unit
                        width: 980 * design.unit
                        height: 60 * design.unit
                        visible: root.resuming
                        // The game's own picture at its centre, cut to the board's 96 by 60.
                        ShaderEffectSource {
                            id: pausedTexture
                            readonly property Item frame: Session.frame
                            readonly property real cutWidth: pausedTexture.frame ? Math.min(pausedTexture.frame.width, pausedTexture.frame.height * 96 / 60) : 0
                            readonly property real cutHeight: pausedTexture.cutWidth * 60 / 96
                            sourceItem: root.resuming && resume.visible ? pausedTexture.frame : null
                            hideSource: false
                            visible: false
                            sourceRect: pausedTexture.frame ? Qt.rect((pausedTexture.frame.width - pausedTexture.cutWidth) / 2, (pausedTexture.frame.height - pausedTexture.cutHeight) / 2, pausedTexture.cutWidth, pausedTexture.cutHeight) : Qt.rect(0, 0, 0, 0)
                            textureSize: Qt.size(Math.ceil(resumePicture.width * canvas.unit), Math.ceil(resumePicture.height * canvas.unit))
                        }
                        Shape {
                            id: resumePicture
                            objectName: "resumePicture"
                            width: 96 * design.unit
                            height: 60 * design.unit
                            readonly property real radius: 10 * design.unit
                            visible: Session.frame !== null
                            preferredRendererType: Shape.CurveRenderer
                            ShapePath {
                                strokeColor: "transparent"
                                fillColor: "transparent"
                                fillItem: pausedTexture
                                fillTransform: PlanarTransform.fromScale(resumePicture.width / Math.max(1, pausedTexture.textureSize.width), resumePicture.height / Math.max(1, pausedTexture.textureSize.height))
                                PathRectangle {
                                    width: resumePicture.width
                                    height: resumePicture.height
                                    radius: resumePicture.radius
                                }
                            }
                        }
                        // The picture's inset rim, 2 px of rgba(246, 231, 216, 0.5) (Main:58).
                        Rectangle {
                            objectName: "resumeRim"
                            // The rim's colour, which the test reads.
                            readonly property color paint: ink.resumeRim
                            anchors.fill: resumePicture
                            radius: resumePicture.radius
                            color: "transparent"
                            border.color: paint
                            border.width: 2 * design.unit
                        }
                        Text {
                            id: resumeWords
                            objectName: "resumeWords"
                            x: resumePicture.width + 16 * design.unit
                            y: (resume.height - height) / 2
                            text: root.resuming ? Words.resumePoint(Session.pausedSince) : ""
                            color: ink.help
                            font.pixelSize: Type.sizeIn(22 * design.unit, Tokens.textScale, Tokens.faceText, Font.Normal)
                            font.family: Tokens.faceText
                        }
                    }
                    Row {
                        objectName: "buttons"
                        x: 96 * design.unit
                        // The board's column gap and the buttons' own margin above them (Main:53, 61), under the resume point when it is drawn.
                        y: (resume.visible ? resume.y + resume.height : facts.y + facts.height) + (26 + 18) * design.unit
                        spacing: 18 * design.unit
                        visible: root.currentTile !== null
                        Repeater {
                            id: buttons
                            model: Help.entries(root.helpKind, { card: true })
                            Pill {
                                id: pill
                                required property int index
                                required property var model
                                unit: design.unit
                                controller: frame.controller
                                objectName: "button." + pill.index
                                place: pill.index
                                visible: pill.index < root.gameButtons
                                live: pill.model.enabled ?? true
                                filled: pill.index === 0
                                outline: ink.pillLine
                                glyphSide: pill.filled ? 46 : 44
                                labelSize: pill.filled ? 26 : 24
                                trail: pill.filled ? 34 : 30
                                gap: pill.filled ? 16 : 14
                                // A plays the game; the others are their buttons' presses, which the host acts on as the help line does.
                                onActivated: pill.filled ? Actions.play(root.currentTile?.key ?? "") : frame.controller.offer(pill.button)
                                KeyNavigation.left: pill.index > 0 ? buttons.itemAt(pill.index - 1) : null
                                KeyNavigation.right: pill.index + 1 < Math.min(root.gameButtons, buttons.count) ? buttons.itemAt(pill.index + 1) : null
                                KeyNavigation.up: status.search
                                KeyNavigation.down: root.currentTile
                            }
                        }
                    }
                    Text {
                        id: recentHeading
                        objectName: "recentHeading"
                        x: 96 * design.unit
                        y: 790 * design.unit
                        text: Words.recentEntry
                        color: Tokens.text
                        font.pixelSize: Type.sizeIn(28 * design.unit, Tokens.textScale, Tokens.faceText, Font.DemiBold)
                        font.family: Tokens.faceText
                        font.weight: Font.DemiBold
                    }
                    Item {
                        id: row
                        objectName: "recent"
                        x: 96 * design.unit
                        y: recentHeading.y + recentHeading.height + 22 * design.unit
                        width: parent.width - x
                        height: 196 * design.unit
                        Repeater {
                            id: tiles
                            // Every game played, the suspended one first among them as the board puts it.
                            model: Library.recent
                            onItemAdded: (index, item) => root.firstTile = index === 0 ? item : root.firstTile
                            Walkable {
                                id: tile
                                required property int index
                                required property string key
                                required property string title
                                required property string hero
                                required property string consoleName
                                readonly property bool big: tile.index === root.current
                                readonly property real radius: 18 * design.unit
                                controller: frame.controller
                                objectName: "tile." + tile.index
                                focus: tile.index === 0
                                Accessible.name: tile.title
                                // The board's tiles, 264 by 165 and the shown one 312 by 196, 24 apart, their feet on the row's foot.
                                x: tile.index * (264 + 24) * design.unit + (tile.index > root.current ? (312 - 264) * design.unit : 0)
                                width: (tile.big ? 312 : 264) * design.unit
                                height: (tile.big ? 196 : 165) * design.unit
                                y: row.height - height
                                visible: tile.x + tile.width <= row.width
                                onActivated: Actions.play(tile.key)
                                onActiveFocusChanged: {
                                    if (!tile.activeFocus)
                                        return
                                    root.current = tile.index
                                    Actions.focus(tile.key)
                                }
                                KeyNavigation.left: tile.index > 0 ? tiles.itemAt(tile.index - 1) : null
                                KeyNavigation.right: tiles.count > tile.index + 1 ? tiles.itemAt(tile.index + 1) : null
                                KeyNavigation.up: root.firstButton ?? status.search
                                // A resting tile's shadow (Main:83): 0 10 px 30 px rgba(0, 0, 0, 0.45); the focused one has its ring instead.
                                Glow {
                                    objectName: "tileShadow." + tile.index
                                    anchors.fill: parent
                                    visible: !tile.activeFocus
                                    radius: tile.radius
                                    color: "#73000000"
                                    blur: 30 * design.unit
                                    offsetY: 10 * design.unit
                                }
                                RoundPicture {
                                    objectName: "tileArt." + tile.index
                                    anchors.fill: parent
                                    source: tile.hero
                                    radius: tile.radius
                                    crop: true
                                    pixelScale: canvas.unit
                                }
                                TileWords {
                                    unit: design.unit
                                    place: tile.index
                                    // The tile's corners, which clip the shade on the board (Main:83).
                                    radius: 18
                                    title: tile.title
                                    // The console alone, as the board's tile line draws {{t.system}} (Main:86).
                                    facts: tile.consoleName
                                    titleInk: ink.tileTitle
                                    factsInk: ink.tileFacts
                                    shadeInk: ink.shade
                                    shadeClear: ink.shadeClear
                                }
                                FocusRing {
                                    objectName: "tileRing"
                                    anchors.fill: parent
                                    radius: tile.radius
                                    visible: tile.activeFocus
                                }
                            }
                        }
                    }
                    HelpLine {
                        unit: design.unit
                        controller: frame.controller
                        // The board's words for the frame's entries: X Options, Start the Home card, no Select search (Main:94-97).
                        model: Help.entries(root.helpKind, { card: true })
                        ink: ink.help
                    }
                }
            }
        }
    }
}
